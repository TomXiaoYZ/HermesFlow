use crate::collectors::birdeye::client::BirdeyeClient;
use crate::collectors::birdeye::config::BirdeyeConfig;
use crate::models::Candle;
use crate::repository::postgres::PostgresRepositories;
use crate::repository::MarketDataRepository;
use crate::repository::TokenRepository;
use chrono::{TimeZone, Utc};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

pub struct HistoricalSyncTask {
    birdeye_client: BirdeyeClient,
    repos: Arc<PostgresRepositories>,
    token_repo: Arc<dyn TokenRepository>,
}

impl HistoricalSyncTask {
    pub fn new(config: BirdeyeConfig, repos: Arc<PostgresRepositories>) -> Self {
        let client = BirdeyeClient::new(config);
        let token_repo = repos.token.clone();

        Self {
            birdeye_client: client,
            repos,
            token_repo,
        }
    }

    pub async fn run(&self) {
        // Fetch active symbols from database instead of config
        let symbols = match self.token_repo.get_active_addresses().await {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to fetch active symbols for historical sync: {}", e);
                return;
            }
        };

        info!(
            "Starting Candle Collection (Incremental) for {} symbols...",
            symbols.len()
        );

        if symbols.is_empty() {
            info!("No active symbols to collect");
            return;
        }

        let now = Utc::now().timestamp();
        let resolution = "15m";
        let default_lookback_days = 365;
        let one_day = 24 * 60 * 60;
        let chunk_size_days = 5; // Birdeye limit

        for symbol in &symbols {
            // Check latest candle time in DB
            let last_time = match self
                .repos
                .market_data
                .get_latest_candle_time("Birdeye", symbol, resolution)
                .await
            {
                Ok(t) => t,
                Err(e) => {
                    warn!("Failed to get latest candle time for {}: {}", symbol, e);
                    None
                }
            };

            let start_ts = if let Some(lt) = last_time {
                // If we have data, start from the next candle
                lt.timestamp() + (15 * 60)
            } else {
                // If no data, start from 365 days ago
                now - (default_lookback_days * one_day)
            };

            // Check if we are up to date (within 30 mins)
            if now - start_ts < 30 * 60 {
                info!("Symbol {} is up to date. Skipping.", symbol);
                continue;
            }

            info!(
                "Syncing {} from {} to {} (Gap: {:.1} days)",
                symbol,
                start_ts,
                now,
                (now - start_ts) as f64 / 86400.0
            );

            // Fetch in chunks
            let mut current_start = start_ts;
            while current_start < now {
                let mut current_end = current_start + (chunk_size_days * one_day);
                if current_end > now {
                    current_end = now;
                }

                if current_end <= current_start {
                    break;
                }

                let mut attempts = 0;
                let max_attempts = 3;
                let mut success = false;

                while attempts < max_attempts {
                    match self
                        .birdeye_client
                        .get_history(symbol, current_start, current_end, resolution)
                        .await
                    {
                        Ok(items) => {
                            let count = items.len();
                            if count > 0 {
                                info!("    Fetched {} candles for {}. Inserting...", count, symbol);

                                let mut candles_to_insert = Vec::with_capacity(count);

                                for item in items {
                                    let candle = Candle {
                                        exchange: "Birdeye".to_string(),
                                        symbol: symbol.clone(),
                                        resolution: resolution.to_string(),
                                        open: Decimal::from_f64(item.open).unwrap_or(Decimal::ZERO),
                                        high: Decimal::from_f64(item.high).unwrap_or(Decimal::ZERO),
                                        low: Decimal::from_f64(item.low).unwrap_or(Decimal::ZERO),
                                        close: Decimal::from_f64(item.close)
                                            .unwrap_or(Decimal::ZERO),
                                        volume: Decimal::from_f64(item.volume)
                                            .unwrap_or(Decimal::ZERO),
                                        amount: Some(
                                            Decimal::from_f64(item.close * item.volume)
                                                .unwrap_or(Decimal::ZERO),
                                        ),
                                        liquidity: None,
                                        fdv: None, // History API lacks FDV
                                        metadata: None,
                                        time: Utc.timestamp_opt(item.unix_time, 0).unwrap(),
                                    };
                                    candles_to_insert.push(candle);
                                }

                                if let Err(e) = self
                                    .repos
                                    .market_data
                                    .insert_candles(&candles_to_insert)
                                    .await
                                {
                                    warn!("Failed to insert batch candles for {}: {}", symbol, e);
                                }
                            }
                            success = true;
                            break;
                        }
                        Err(e) => {
                            attempts += 1;
                            warn!(
                                "    Failed to fetch history (Attempt {}/{}): {}",
                                attempts, max_attempts, e
                            );
                            sleep(Duration::from_secs(2)).await;
                        }
                    }
                }

                if !success {
                    warn!(
                        "Failed to fetch chunk for {}, skipping remaining chunks for this symbol",
                        symbol
                    );
                    break;
                }

                current_start = current_end;
                // Rate limit between chunks
                sleep(Duration::from_millis(500)).await;
            }
            // Rate limit between symbols
            sleep(Duration::from_millis(200)).await;
        }

        info!("Candle Collection Task Completed.");
    }
}
