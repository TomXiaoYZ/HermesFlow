use crate::collectors::birdeye::client::BirdeyeClient;
use crate::collectors::birdeye::config::BirdeyeConfig;
use crate::models::Candle;
use crate::repository::TokenRepository;
use chrono::{TimeZone, Utc};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use std::error::Error;
use std::sync::Arc;
use tracing::{error, info};

pub struct BirdeyeConnector {
    client: BirdeyeClient,
}

impl BirdeyeConnector {
    pub fn new(config: BirdeyeConfig) -> Self {
        let client = BirdeyeClient::new(config);
        Self { client }
    }

    pub async fn fetch_history_candles(
        &self,
        address: &str,
        resolution: &str,
        from_ts: i64,
        to_ts: i64,
    ) -> Result<Vec<Candle>, Box<dyn Error + Send + Sync>> {
        let mut all_candles = Vec::new();
        // Birdeye seems to limit to ~720-1000 candles per request.
        // Safe chunk size: 25 days for 1H resolution (approx 600 items), or 1 day for 1m resolution (1440 items).
        // Since we mainly use 1H for backfill now, let's pick a duration that is safe for 1H.
        // For 1H: 30 days = 720 items. Limit seems to be around 720-1000.
        // Let's safe chunk at 20 days (20 * 24 = 480 items).
        // However, if resolution is '1m', 20 days is too much (20 * 1440 = 28800).
        // For '1m', limit is likely similar (1000 items -> ~16 hours).
        let chunk_duration: i64 = if resolution == "1m" {
            12 * 60 * 60 // 12 hours
        } else if resolution == "15m" {
            10 * 24 * 60 * 60 // 10 days (10 * 96 = 960)
        } else {
            20 * 24 * 60 * 60 // 20 days for 1H/1D (20 * 24 = 480)
        };

        let mut current_from = from_ts;
        while current_from < to_ts {
            let mut current_to = current_from + chunk_duration;
            if current_to > to_ts {
                current_to = to_ts;
            }

            info!(
                "Fetching history chunk for {} ({}): {} to {}",
                address, resolution, current_from, current_to
            );

            match self
                .client
                .get_history(address, current_from, current_to, resolution)
                .await
            {
                Ok(items) => {
                    for item in items {
                        let open = Decimal::from_f64(item.open).unwrap_or_default();
                        let high = Decimal::from_f64(item.high).unwrap_or_default();
                        let low = Decimal::from_f64(item.low).unwrap_or_default();
                        let close = Decimal::from_f64(item.close).unwrap_or_default();
                        let volume = Decimal::from_f64(item.volume).unwrap_or_default();

                        let time = Utc.timestamp_opt(item.unix_time, 0).unwrap();

                        let candle = Candle::new(
                            "Birdeye".to_string(),
                            address.to_string(),
                            resolution.to_string(),
                            open,
                            high,
                            low,
                            close,
                            volume,
                            time,
                        );

                        all_candles.push(candle);
                    }
                }
                Err(e) => {
                    error!("Failed to fetch chunk: {}", e);
                    // continue? or fail?
                    // if fail, we lose whole backfill. Better to log and continue to try next chunk?
                    // But usually network error persists.
                    // Let's error out to be safe.
                    return Err(e);
                }
            }

            // Advance
            current_from = current_to;

            // Tiny sleep to be nice
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }

        Ok(all_candles)
    }

    pub async fn connect(
        &self,
        _token_repo: Arc<dyn TokenRepository>,
    ) -> Result<
        tokio::sync::mpsc::Receiver<crate::models::StandardMarketData>,
        Box<dyn Error + Send + Sync>,
    > {
        let (_tx, rx) = tokio::sync::mpsc::channel(100);

        // Birdeye polling is disabled in favor of Jupiter Price API
        // This connector now only serves on-demand history/overview requests
        tokio::spawn(async move {
            info!("Birdeye Collector: Real-time polling is DISABLED (API Cost Optimization).");
            info!("Using Jupiter Price API for monitoring instead.");

            // Keep the task alive just in case, but do nothing
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
            }
        });

        Ok(rx)
    }

    pub async fn disconnect(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }

    pub async fn fetch_token_overview(
        &self,
        address: &str,
    ) -> Result<crate::collectors::birdeye::client::TokenOverview, Box<dyn Error + Send + Sync>>
    {
        self.client.get_token_overview(address).await
    }
}
