use crate::collectors::birdeye::client::BirdeyeClient;
use crate::collectors::birdeye::config::BirdeyeConfig;
use crate::models::Candle;
use crate::repository::TokenRepository;
use chrono::{TimeZone, Utc};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use std::error::Error;
use std::sync::Arc;
use tracing::{error, info, warn};

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
        let items = self
            .client
            .get_history(address, from_ts, to_ts, resolution)
            .await?;

        let mut candles = Vec::new();
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

            candles.push(candle);
        }

        Ok(candles)
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
