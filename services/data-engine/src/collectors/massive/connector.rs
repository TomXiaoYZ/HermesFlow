use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use super::client::{AggregateResult, MassiveClient};
use crate::error::{DataError, Result};
use crate::models::{AssetType, DataSourceType, MarketDataType, StandardMarketData};
use crate::traits::connector::ConnectorStats;
use crate::traits::DataSourceConnector;

use crate::config::MassiveConfig;

pub struct MassiveConnector {
    client: MassiveClient,
    stats: ConnectorStats,
    source_type: DataSourceType,
    config: MassiveConfig,
}

impl MassiveConnector {
    pub fn new(config: MassiveConfig) -> Self {
        Self {
            client: MassiveClient::new(config.api_key.clone(), config.rate_limit_per_min),
            stats: ConnectorStats::default(),
            source_type: DataSourceType::PolygonStock,
            config,
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
        let aggregates = self
            .client
            .get_aggregates(ticker, multiplier, timespan, from, to)
            .await
            .map_err(|e| DataError::ConnectionFailed {
                data_source: "Polygon".to_string(),
                reason: e.to_string(),
            })?;

        // Convert to StandardMarketData
        let mut data_points = Vec::with_capacity(aggregates.len());

        for agg in aggregates {
            let data = self.map_aggregate(ticker, agg);
            data_points.push(data);
        }

        Ok(data_points)
    }

    fn map_aggregate(&self, ticker: &str, agg: AggregateResult) -> StandardMarketData {
        let price = Decimal::from_f64_retain(agg.c).unwrap_or(Decimal::ZERO);
        let volume = Decimal::from_f64_retain(agg.v).unwrap_or(Decimal::ZERO);
        // Turn timestamp (msec) into raw i64
        let timestamp = agg.t;

        // Metadata for JSON storage
        let metadata = serde_json::json!({
            "vwap": agg.vw,
            "transactions": agg.n,
            "open": agg.o,
            "high": agg.h,
            "low": agg.l,
            "close": agg.c,
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
            high_24h: Some(Decimal::from_f64_retain(agg.h).unwrap_or(Decimal::ZERO)),
            low_24h: Some(Decimal::from_f64_retain(agg.l).unwrap_or(Decimal::ZERO)),
            volume_24h: None, // This is a single candle, not 24h rolling
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
        let streamer = super::websocket::MassiveStreamer::new(api_key, self.config.ws_url.clone());
        streamer.connect().await
    }

    async fn disconnect(&mut self) -> Result<()> {
        Ok(())
    }

    async fn is_healthy(&self) -> bool {
        // Simple heuristic: if we can create the client, we assume healthy for REST
        true
    }

    fn stats(&self) -> ConnectorStats {
        self.stats.clone()
    }
}
