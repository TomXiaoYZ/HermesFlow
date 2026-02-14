use crate::collectors::MassiveConnector;
use crate::config::MassiveConfig;
use crate::models::Candle;
use crate::repository::postgres::PostgresRepositories;
use crate::repository::MarketDataRepository;
use chrono::{TimeZone, Utc};
use rust_decimal::Decimal;
use serde_json::Value;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

/// Resolutions to sync from the Polygon aggregate API.
/// 1d is already handled by the EOD job — no need to duplicate.
const SYNC_RESOLUTIONS: &[ResolutionConfig] = &[
    ResolutionConfig {
        resolution: "15m",
        multiplier: 15,
        timespan: "minute",
        chunk_days: 5,
        default_lookback_days: 90,
    },
    ResolutionConfig {
        resolution: "1h",
        multiplier: 1,
        timespan: "hour",
        chunk_days: 20,
        default_lookback_days: 365,
    },
];

struct ResolutionConfig {
    resolution: &'static str,
    multiplier: i32,
    timespan: &'static str,
    chunk_days: i64,
    default_lookback_days: i64,
}

pub struct PolygonSyncTask {
    connector: MassiveConnector,
    repos: Arc<PostgresRepositories>,
}

impl PolygonSyncTask {
    pub fn new(config: MassiveConfig, repos: Arc<PostgresRepositories>) -> Self {
        let connector = MassiveConnector::new(config);
        Self { connector, repos }
    }

    /// Full sync: syncs all resolutions for all watchlist symbols.
    pub async fn run(&self) {
        let symbols = match self.repos.market_data.get_watchlist_symbols().await {
            Ok(s) => s,
            Err(e) => {
                warn!("Polygon sync: Failed to fetch watchlist symbols: {}", e);
                return;
            }
        };

        if symbols.is_empty() {
            info!("Polygon sync: No watchlist symbols found, skipping.");
            return;
        }

        for rc in SYNC_RESOLUTIONS {
            info!(
                "Starting Polygon {} sync for {} symbols...",
                rc.resolution,
                symbols.len()
            );
            self.sync_resolution(&symbols, rc).await;
        }

        info!("Polygon OHLCV sync completed.");
    }

    /// Incremental sync for a single resolution across all symbols.
    async fn sync_resolution(&self, symbols: &[String], rc: &ResolutionConfig) {
        let now = Utc::now();
        let one_day: i64 = 24 * 60 * 60;
        let candle_seconds: i64 = match rc.timespan {
            "minute" => rc.multiplier as i64 * 60,
            "hour" => rc.multiplier as i64 * 3600,
            "day" => rc.multiplier as i64 * 86400,
            _ => rc.multiplier as i64 * 60,
        };

        for symbol in symbols {
            let last_time = match self
                .repos
                .market_data
                .get_latest_candle_time("Polygon", symbol, rc.resolution)
                .await
            {
                Ok(t) => t,
                Err(e) => {
                    warn!(
                        "Polygon sync: Failed to get latest candle time for {} ({}): {}",
                        symbol, rc.resolution, e
                    );
                    None
                }
            };

            let start_ts = if let Some(lt) = last_time {
                lt.timestamp() + candle_seconds
            } else {
                now.timestamp() - (rc.default_lookback_days * one_day)
            };

            // Up to date if gap < 2 candle periods
            if now.timestamp() - start_ts < candle_seconds * 2 {
                continue;
            }

            info!(
                "[Polygon {}] Syncing {} — gap {:.1} days",
                rc.resolution,
                symbol,
                (now.timestamp() - start_ts) as f64 / 86400.0
            );

            if !self
                .fetch_and_insert(symbol, rc, start_ts, now.timestamp())
                .await
            {
                warn!(
                    "[Polygon {}] Failed to sync {}, moving to next symbol",
                    rc.resolution, symbol
                );
            }

            sleep(Duration::from_millis(200)).await;
        }
    }

    /// Fetch aggregate data from Polygon API in date-based chunks and insert as candles.
    /// Returns true on success, false if a chunk failed after retries.
    async fn fetch_and_insert(
        &self,
        symbol: &str,
        rc: &ResolutionConfig,
        from_ts: i64,
        to_ts: i64,
    ) -> bool {
        let chunk_secs = rc.chunk_days * 24 * 60 * 60;
        let mut current_start = from_ts;

        while current_start < to_ts {
            let current_end = (current_start + chunk_secs).min(to_ts);
            if current_end <= current_start {
                break;
            }

            let from_date = Utc
                .timestamp_opt(current_start, 0)
                .unwrap()
                .format("%Y-%m-%d")
                .to_string();
            let to_date = Utc
                .timestamp_opt(current_end, 0)
                .unwrap()
                .format("%Y-%m-%d")
                .to_string();

            let mut attempts = 0;
            let max_attempts = 3;
            let mut success = false;

            while attempts < max_attempts {
                match self
                    .connector
                    .fetch_history_candles(symbol, rc.multiplier, rc.timespan, &from_date, &to_date)
                    .await
                {
                    Ok(data_points) => {
                        if !data_points.is_empty() {
                            let candles: Vec<Candle> = data_points
                                .into_iter()
                                .map(|data| {
                                    let meta_value =
                                        serde_json::from_str::<Value>(&data.raw_data).ok();
                                    let open = if let Some(json) = &meta_value {
                                        json.get("open")
                                            .and_then(|v| v.as_f64())
                                            .map(|f| {
                                                Decimal::from_f64_retain(f).unwrap_or_default()
                                            })
                                            .unwrap_or(data.price)
                                    } else {
                                        data.price
                                    };

                                    let high = meta_value
                                        .as_ref()
                                        .and_then(|j| j.get("high"))
                                        .and_then(|v| v.as_str())
                                        .and_then(|s| s.parse::<f64>().ok())
                                        .map(|f| Decimal::from_f64_retain(f).unwrap_or_default())
                                        .unwrap_or(data.price);

                                    let low = meta_value
                                        .as_ref()
                                        .and_then(|j| j.get("low"))
                                        .and_then(|v| v.as_str())
                                        .and_then(|s| s.parse::<f64>().ok())
                                        .map(|f| Decimal::from_f64_retain(f).unwrap_or_default())
                                        .unwrap_or(data.price);

                                    Candle {
                                        exchange: "Polygon".to_string(),
                                        symbol: data.symbol.clone(),
                                        resolution: rc.resolution.to_string(),
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
                                    }
                                })
                                .collect();

                            info!(
                                "[Polygon {}] Fetched {} candles for {}",
                                rc.resolution,
                                candles.len(),
                                symbol
                            );

                            if let Err(e) = self.repos.market_data.insert_candles(&candles).await {
                                warn!(
                                    "[Polygon {}] Failed to insert candles for {}: {}",
                                    rc.resolution, symbol, e
                                );
                            }
                        }
                        success = true;
                        break;
                    }
                    Err(e) => {
                        attempts += 1;
                        warn!(
                            "[Polygon {}] Fetch failed for {} (attempt {}/{}): {}",
                            rc.resolution, symbol, attempts, max_attempts, e
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
