use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};

use crate::config::AppConfig;
use crate::repository::postgres::PostgresRepositories;

pub struct TaskManager {
    scheduler: Arc<RwLock<JobScheduler>>,
    repos: Arc<PostgresRepositories>,
    config: AppConfig,
}

impl TaskManager {
    pub async fn new(
        config: AppConfig,
        repos: Arc<PostgresRepositories>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let scheduler = JobScheduler::new().await?;
        Ok(Self {
            scheduler: Arc::new(RwLock::new(scheduler)),
            repos,
            config,
        })
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting Task Manager...");
        let scheduler = self.scheduler.read().await;
        scheduler.start().await?;

        // Launch Historical Backfill on Startup (if Birdeye enabled)
        // Check config first
        if let Some(birdeye_config) = &self.config.birdeye {
            if birdeye_config.enabled {
                info!("Triggering Startup Backfill for Birdeye Symbols...");
                let task = crate::tasks::historical_sync::HistoricalSyncTask::new(
                    birdeye_config.clone(),
                    self.repos.clone(),
                );

                // Spawn task to avoid blocking main thread start
                tokio::spawn(async move {
                    task.run().await;
                });
            }
        }

        Ok(())
    }

    pub async fn register_eod_job(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Registering EOD Data Collection Job...");

        let repos_clone = self.repos.clone();
        let config_clone = self.config.clone();

        // Schedule: Every day at 01:00 UTC
        let job = Job::new_async("0 0 1 * * *", move |_uuid, _l| {
            let repos = repos_clone.clone();
            let config = config_clone.clone();

            Box::pin(async move {
                info!("Executing Scheduled EOD Data Collection...");
                use crate::collectors::MassiveConnector;
                // use crate::models::Candle; // Already available if imported at top or re-import
                // To be safe:
                use crate::models::Candle;
                use crate::repository::MarketDataRepository;
                use chrono::{TimeZone, Utc};
                use rust_decimal::prelude::FromPrimitive;
                use rust_decimal::Decimal;
                use serde_json::Value;

                let yesterday = Utc::now().date_naive().pred_opt().unwrap();
                let date_str = yesterday.format("%Y-%m-%d").to_string();

                match repos.market_data.get_active_symbols().await {
                    Ok(symbols) => {
                        info!("Found {} active symbols for EOD correction", symbols.len());
                        if let Some(massive_cfg) = config.massive {
                            let connector = MassiveConnector::new(massive_cfg);

                            for symbol in symbols {
                                match connector
                                    .fetch_history_candles(&symbol, 1, "day", &date_str, &date_str)
                                    .await
                                {
                                    Ok(candles) => {
                                        for data in candles {
                                            let meta_value =
                                                serde_json::from_str::<Value>(&data.raw_data).ok();
                                            let open = if let Some(json) = &meta_value {
                                                json.get("open")
                                                    .and_then(|v| v.as_f64())
                                                    .map(|f| {
                                                        Decimal::from_f64_retain(f)
                                                            .unwrap_or_default()
                                                    })
                                                    .unwrap_or(data.price)
                                            } else {
                                                data.price
                                            };

                                            let candle = Candle {
                                                exchange: "Polygon".to_string(),
                                                symbol: data.symbol.clone(),
                                                resolution: "1d".to_string(),
                                                open,
                                                high: data.high_24h.unwrap_or(data.price),
                                                low: data.low_24h.unwrap_or(data.price),
                                                close: data.price,
                                                volume: data.quantity,
                                                amount: None,
                                                liquidity: None,
                                                fdv: None,
                                                metadata: meta_value,
                                                time: Utc
                                                    .timestamp_opt(data.timestamp / 1000, 0)
                                                    .unwrap(),
                                            };

                                            if let Err(e) =
                                                repos.market_data.insert_candle(&candle).await
                                            {
                                                error!(
                                                    "EOD: Failed to insert {} for {}: {}",
                                                    symbol, date_str, e
                                                );
                                            }
                                        }
                                    }
                                    Err(e) => error!(
                                        "EOD: Failed to fetch {} for {}: {}",
                                        symbol, date_str, e
                                    ),
                                }
                                // Rate limit sleep
                                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                            }
                        } else {
                            error!("EOD: Massive/Polygon not configured.");
                        }
                    }
                    Err(e) => error!("Failed to fetch active symbols: {}", e),
                }
                info!("EOD Data Collection Completed.");
            })
        })?;

        let scheduler = self.scheduler.write().await;
        scheduler.add(job).await?;

        Ok(())
    }

    pub async fn trigger_backfill(
        &self,
        symbol: String,
        from: String,
        to: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!(
            "Triggering manual backfill for {} from {} to {:?}",
            symbol, from, to
        );

        let config = self.config.clone();
        let repos_clone_for_task = self.repos.clone();

        // Spawn async task to avoid blocking API
        tokio::spawn(async move {
            info!("Starting background backfill for {}", symbol);

            use crate::collectors::{BirdeyeConnector, MassiveConnector};
            use crate::models::{Candle, MarketDataType};
            use crate::repository::MarketDataRepository; // Import Trait
            use chrono::{TimeZone, Utc};
            use rust_decimal::prelude::FromPrimitive;
            use rust_decimal::Decimal;
            use serde_json::Value;

            // Simple heuristic: If symbol length > 10 and no dot, likely a contract address (e.g. Solana)
            let is_contract = symbol.len() > 10 && !symbol.contains('.');

            if is_contract {
                if let Some(birdeye_cfg) = config.birdeye {
                    let connector = BirdeyeConnector::new(birdeye_cfg);
                    info!("Backfill (Birdeye) task running for {}...", symbol);
                    let to_ts = Utc::now().timestamp();
                    let from_ts = Utc::now()
                        .checked_sub_signed(chrono::Duration::days(30))
                        .unwrap()
                        .timestamp(); // Default 30 days? API needs explicit.

                    match connector
                        .fetch_history_candles(&symbol, "1H", from_ts, to_ts)
                        .await
                    {
                        Ok(candles) => {
                            let total = candles.len();
                            let mut success_count = 0;
                            for candle in candles {
                                // BirdeyeConnector returns Candle with None liquidity/fdv usually,
                                // unless we enhanced it.
                                if let Err(e) = repos_clone_for_task
                                    .market_data
                                    .insert_candle(&candle)
                                    .await
                                {
                                    error!("Failed to insert Birdeye candle for {}: {}", symbol, e);
                                } else {
                                    success_count += 1;
                                }
                            }
                            info!(
                                "Birdeye Backfill complete for {}. Inserted {}/{}",
                                symbol, success_count, total
                            );
                        }
                        Err(e) => error!("Failed to fetch Birdeye history for {}: {}", symbol, e),
                    }
                } else {
                    error!("Birdeye not configured for contract address {}", symbol);
                }
            } else {
                if let Some(massive_cfg) = config.massive {
                    let connector = MassiveConnector::new(massive_cfg);
                    info!("Backfill (Massive) task running for {}...", symbol);

                    // Default to 5 years if 'to' is not specified
                    let to_date = to.unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string());

                    // Fetch Day Aggregates (1d)
                    match connector
                        .fetch_history_candles(&symbol, 1, "day", &from, &to_date)
                        .await
                    {
                        Ok(candles) => {
                            let total_candles = candles.len();
                            info!("Fetched {} candles for {}", total_candles, symbol);
                            let mut success_count = 0;
                            for data in candles {
                                // Extract OHLC from raw_data or fields
                                let meta_value = serde_json::from_str::<Value>(&data.raw_data).ok();

                                let open = if let Some(json) = &meta_value {
                                    json.get("open")
                                        .and_then(|v| v.as_f64())
                                        .map(|f| Decimal::from_f64_retain(f).unwrap_or_default())
                                        .unwrap_or(data.price)
                                } else {
                                    data.price // Fallback
                                };

                                let high = data.high_24h.unwrap_or(data.price);
                                let low = data.low_24h.unwrap_or(data.price);

                                let candle = Candle {
                                    exchange: "Polygon".to_string(),
                                    symbol: data.symbol.clone(),
                                    resolution: "1d".to_string(),
                                    open,
                                    high,
                                    low,
                                    close: data.price,
                                    volume: data.quantity,
                                    amount: None,
                                    liquidity: None,
                                    fdv: None,
                                    metadata: meta_value,
                                    time: Utc.timestamp_opt(data.timestamp / 1000, 0).unwrap(),
                                };

                                if let Err(e) = repos_clone_for_task
                                    .market_data
                                    .insert_candle(&candle)
                                    .await
                                {
                                    error!("Failed to insert candle for {}: {}", symbol, e);
                                } else {
                                    success_count += 1;
                                }
                            }
                            info!(
                                "Backfill complete for {}. Inserted {}/{}",
                                symbol, success_count, total_candles
                            );
                        }
                        Err(e) => {
                            error!("Failed to fetch history for {}: {}", symbol, e);
                        }
                    }
                } else {
                    error!("Massive/Polygon is not configured!");
                }
            }
        });

        Ok(())
    }

    pub async fn register_token_discovery_job(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Registering Token Discovery Job (Every 15 min)...");

        let repos_clone = self.repos.clone();
        let config_clone = self.config.clone();

        // Every 15 minutes: 0 */15 * * * *
        let job = Job::new_async("0 */15 * * * *", move |_uuid, _l| {
            let repos = repos_clone.clone();
            let config = config_clone.clone();

            Box::pin(async move {
                info!("🔍 Executing Scheduled Token Discovery...");

                if let Some(birdeye_cfg) = config.birdeye {
                    use crate::tasks::token_discovery::TokenDiscoveryTask;

                    let task = TokenDiscoveryTask::new(
                        birdeye_cfg.clone(),
                        repos.token.clone(),
                        500_000.0,     // min_liquidity_usd: $500k
                        10_000_000.0,  // min_fdv: $10M
                        f64::INFINITY, // max_fdv: unlimited
                    );

                    task.run().await;
                } else {
                    info!("Birdeye not configured, skipping token discovery");
                }
            })
        })?;

        let scheduler = self.scheduler.write().await;
        scheduler.add(job).await?;

        Ok(())
    }

    pub async fn register_data_quality_job(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Registering Data Quality Job (Every hour)...");

        let repos_clone = self.repos.clone();

        // Every hour at minute 0: 0 0 * * * *
        // Every hour at minute 0: 0 0 * * * *
        let job = Job::new_async("0 0 * * * *", move |_uuid, _l| {
            let repos = repos_clone.clone();

            Box::pin(async move {
                use crate::tasks::data_quality::DataQualityTask;
                let task = DataQualityTask::new(repos);
                task.run().await;
            })
        })?;

        let scheduler = self.scheduler.write().await;
        scheduler.add(job).await?;

        // Run immediately on startup for visibility
        let repos_startup = self.repos.clone();
        tokio::spawn(async move {
            info!("🚀 Running initial Data Quality Check...");
            use crate::tasks::data_quality::DataQualityTask;
            let task = DataQualityTask::new(repos_startup);
            task.run().await;
        });

        Ok(())
    }
}
