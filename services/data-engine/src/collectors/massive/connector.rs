use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{mpsc, RwLock};
use tracing::info;

use super::client::{AggregateResult, MassiveClient};
use crate::error::{DataError, Result};
use crate::models::{AssetType, DataSourceType, MarketDataType, StandardMarketData};
use crate::traits::connector::ConnectorStats;
use crate::traits::DataSourceConnector;

use crate::config::MassiveConfig;

pub struct MassiveConnector {
    client: MassiveClient,
    stats: Arc<RwLock<ConnectorStats>>,
    source_type: DataSourceType,
    config: MassiveConfig,
    last_success: Arc<AtomicU64>,
    consecutive_errors: Arc<AtomicU32>,
}

impl MassiveConnector {
    pub fn new(config: MassiveConfig) -> Self {
        let now_millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            client: MassiveClient::new(
                config.api_key.clone(),
                config.rate_limit_per_min,
                "https://api.polygon.io".to_string(),
            ),
            stats: Arc::new(RwLock::new(ConnectorStats::default())),
            source_type: DataSourceType::PolygonStock,
            config,
            last_success: Arc::new(AtomicU64::new(now_millis)),
            consecutive_errors: Arc::new(AtomicU32::new(0)),
        }
    }

    /// Fetch historical candles for a specific ticker and range
    /// Returns a vector of StandardMarketData
    pub async fn fetch_history_candles(
        &self,
        ticker: &str,
        multiplier: i32,
        timespan: &str,
        from: &str,
        to: &str,
    ) -> Result<Vec<StandardMarketData>> {
        let result = self
            .client
            .get_aggregates(ticker, multiplier, timespan, from, to)
            .await
            .map_err(|e| DataError::ConnectionFailed {
                data_source: "Polygon".to_string(),
                reason: e.to_string(),
            });

        match result {
            Ok(aggregates) => {
                let now_millis = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                self.last_success.store(now_millis, Ordering::Relaxed);
                self.consecutive_errors.store(0, Ordering::Relaxed);

                let count = aggregates.len() as u64;
                let mut data_points = Vec::with_capacity(aggregates.len());
                for agg in aggregates {
                    let data = self.map_aggregate(ticker, agg);
                    data_points.push(data);
                }

                // Track stats
                let mut s = self.stats.write().await;
                s.messages_received += count;
                s.messages_processed += count;
                s.last_message_at = Some(std::time::SystemTime::now());

                Ok(data_points)
            }
            Err(e) => {
                self.consecutive_errors.fetch_add(1, Ordering::Relaxed);
                let mut s = self.stats.write().await;
                s.errors += 1;
                Err(e)
            }
        }
    }

    fn map_aggregate(&self, ticker: &str, agg: AggregateResult) -> StandardMarketData {
        let price = agg.c;
        let volume = agg.v;
        let timestamp = agg.t;

        // Metadata for JSON storage (convert Decimal to string for serde_json compatibility)
        let metadata = serde_json::json!({
            "vwap": agg.vw.map(|d| d.to_string()),
            "transactions": agg.n,
            "open": agg.o.to_string(),
            "high": agg.h.to_string(),
            "low": agg.l.to_string(),
            "close": agg.c.to_string(),
        });

        StandardMarketData {
            source: self.source_type.clone(),
            exchange: "Polygon".to_string(),
            symbol: ticker.to_string(),
            asset_type: if ticker.starts_with("X:") {
                AssetType::Crypto
            } else if ticker.starts_with("O:") {
                AssetType::Option
            } else {
                AssetType::Stock
            },
            data_type: MarketDataType::Candle,
            price,
            quantity: volume,
            timestamp,
            received_at: chrono::Utc::now().timestamp_millis(),
            bid: None,
            ask: None,
            high_24h: None,   // Single candle cannot represent 24h range
            low_24h: None,    // Single candle cannot represent 24h range
            volume_24h: None, // Single candle, not 24h rolling
            open_interest: None,
            funding_rate: None,
            liquidity: None,
            fdv: None,
            sequence_id: None,
            raw_data: metadata.to_string(),
        }
    }
}

#[async_trait]
impl DataSourceConnector for MassiveConnector {
    fn source_type(&self) -> DataSourceType {
        self.source_type.clone()
    }

    fn supported_assets(&self) -> Vec<AssetType> {
        vec![
            AssetType::Spot,
            AssetType::Stock,
            AssetType::Crypto,
            AssetType::Option,
        ]
    }

    async fn connect(&mut self) -> Result<mpsc::Receiver<StandardMarketData>> {
        info!("Connecting to Massive/Polygon WebSocket for Real-Time data...");
        let api_key = self.client.api_key().to_string(); // Access api_key via getter
        let streamer = super::websocket::MassiveStreamer::with_stats(
            api_key,
            self.config.ws_url.clone(),
            self.stats.clone(),
        );
        streamer.connect().await
    }

    async fn disconnect(&mut self) -> Result<()> {
        Ok(())
    }

    async fn is_healthy(&self) -> bool {
        let now_millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let last = self.last_success.load(Ordering::Relaxed);
        let errors = self.consecutive_errors.load(Ordering::Relaxed);

        (now_millis.saturating_sub(last) < 300_000) && (errors < 3)
    }

    fn stats(&self) -> ConnectorStats {
        // Use try_read to avoid blocking; fall back to default if lock is held
        match self.stats.try_read() {
            Ok(guard) => guard.clone(),
            Err(_) => ConnectorStats::default(),
        }
    }
}
