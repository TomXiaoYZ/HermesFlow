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

/// Resolutions to sync from the Birdeye OHLCV API.
const SYNC_RESOLUTIONS: &[ResolutionConfig] = &[
    ResolutionConfig {
        resolution: "15m",
        candle_seconds: 15 * 60,
        chunk_days: 5,
        default_lookback_days: 365,
    },
    ResolutionConfig {
        resolution: "1h",
        candle_seconds: 60 * 60,
        chunk_days: 20,
        default_lookback_days: 365,
    },
];

struct ResolutionConfig {
    resolution: &'static str,
    candle_seconds: i64,
    chunk_days: i64,
    default_lookback_days: i64,
}

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

    /// Full startup sync: syncs all resolutions for all active symbols.
    pub async fn run(&self) {
        let symbols = match self.token_repo.get_active_addresses().await {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to fetch active symbols for historical sync: {}", e);
                return;
            }
        };

        if symbols.is_empty() {
            info!("No active symbols to collect");
            return;
        }

        for rc in SYNC_RESOLUTIONS {
            info!(
                "Starting Birdeye {} sync for {} symbols...",
                rc.resolution,
                symbols.len()
            );
            self.sync_resolution(&symbols, rc).await;
        }

        info!("Birdeye Candle Collection Task Completed.");
    }

    /// Incremental sync for a single resolution across all symbols.
    async fn sync_resolution(&self, symbols: &[String], rc: &ResolutionConfig) {
        let now = Utc::now().timestamp();
        let one_day: i64 = 24 * 60 * 60;

        for symbol in symbols {
            let last_time = match self
                .repos
                .market_data
                .get_latest_candle_time("Birdeye", symbol, rc.resolution)
                .await
            {
                Ok(t) => t,
                Err(e) => {
                    warn!(
                        "Failed to get latest candle time for {} ({}): {}",
                        symbol, rc.resolution, e
                    );
                    None
                }
            };

            let start_ts = if let Some(lt) = last_time {
                lt.timestamp() + rc.candle_seconds
            } else {
                now - (rc.default_lookback_days * one_day)
            };

            // Up to date if gap < 2 candle periods
            if now - start_ts < rc.candle_seconds * 2 {
                continue;
            }

            info!(
                "[{}] Syncing {} — gap {:.1} days",
                rc.resolution,
                symbol,
                (now - start_ts) as f64 / 86400.0
            );

            if !self
                .fetch_and_insert(symbol, rc.resolution, start_ts, now, rc.chunk_days * one_day)
                .await
            {
                warn!(
                    "[{}] Failed to sync {}, moving to next symbol",
                    rc.resolution, symbol
                );
            }

            sleep(Duration::from_millis(200)).await;
        }
    }

    /// Fetch OHLCV data from Birdeye API in chunks and insert into DB.
    /// Returns true on success, false if a chunk failed after retries.
    async fn fetch_and_insert(
        &self,
        symbol: &str,
        resolution: &str,
        from_ts: i64,
        to_ts: i64,
        chunk_duration: i64,
    ) -> bool {
        let mut current_start = from_ts;

        while current_start < to_ts {
            let current_end = (current_start + chunk_duration).min(to_ts);
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
                        if !items.is_empty() {
                            let candles: Vec<Candle> = items
                                .into_iter()
                                .map(|item| Candle {
                                    exchange: "Birdeye".to_string(),
                                    symbol: symbol.to_string(),
                                    resolution: resolution.to_string(),
                                    open: Decimal::from_f64(item.open)
                                        .unwrap_or(Decimal::ZERO),
                                    high: Decimal::from_f64(item.high)
                                        .unwrap_or(Decimal::ZERO),
                                    low: Decimal::from_f64(item.low)
                                        .unwrap_or(Decimal::ZERO),
                                    close: Decimal::from_f64(item.close)
                                        .unwrap_or(Decimal::ZERO),
                                    volume: Decimal::from_f64(item.volume)
                                        .unwrap_or(Decimal::ZERO),
                                    amount: Some(
                                        Decimal::from_f64(item.close * item.volume)
                                            .unwrap_or(Decimal::ZERO),
                                    ),
                                    liquidity: None,
                                    fdv: None,
                                    metadata: None,
                                    time: Utc.timestamp_opt(item.unix_time, 0).unwrap(),
                                })
                                .collect();

                            info!(
                                "[{}] Fetched {} candles for {}",
                                resolution,
                                candles.len(),
                                symbol
                            );

                            if let Err(e) =
                                self.repos.market_data.insert_candles(&candles).await
                            {
                                warn!(
                                    "[{}] Failed to insert candles for {}: {}",
                                    resolution, symbol, e
                                );
                            }
                        }
                        success = true;
                        break;
                    }
                    Err(e) => {
                        attempts += 1;
                        warn!(
                            "[{}] Fetch failed for {} (attempt {}/{}): {}",
                            resolution, symbol, attempts, max_attempts, e
                        );
                        sleep(Duration::from_secs(2)).await;
                    }
                }
            }

            if !success {
                return false;
            }

            current_start = current_end;
            sleep(Duration::from_millis(500)).await;
        }

        true
    }
}
