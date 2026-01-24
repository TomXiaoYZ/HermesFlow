use super::client::BinanceClient;
use super::config::BinanceConfig;
use super::websocket::BinanceStreamer;
use crate::error::Result;
use crate::models::{AssetType, Candle, DataSourceType, StandardMarketData};
use crate::traits::{ConnectorStats, DataSourceConnector};
use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use reqwest::Method;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::BTreeMap;
use tokio::sync::mpsc;
use tracing::{info, warn};

pub struct BinanceConnector {
    config: BinanceConfig,
    client: BinanceClient,
    stats: ConnectorStats,
    running: bool,
}

impl BinanceConnector {
    pub fn new(config: BinanceConfig) -> Self {
        let client = BinanceClient::new(config.clone());
        Self {
            config,
            client,
            stats: ConnectorStats::default(),
            running: false,
        }
    }

    /// Fetch historical klines (candles)
    /// Limit 1000 per request
    pub async fn fetch_history_candles(
        &self,
        symbol: &str,
        interval: &str,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> Result<Vec<StandardMarketData>> {
        // GET /api/v3/klines
        let mut params = BTreeMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        params.insert("interval".to_string(), interval.to_string());
        
        if let Some(start) = start_time {
            params.insert("startTime".to_string(), start.to_string());
        }
        if let Some(end) = end_time {
            params.insert("endTime".to_string(), end.to_string());
        }
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }

        // Binance Kline format:
        // [
        //   1499040000000,      // Open time
        //   "0.01634790",       // Open
        //   "0.80000000",       // High
        //   "0.01575800",       // Low
        //   "0.01577100",       // Close
        //   "148976.11427815",  // Volume
        //   1499644799999,      // Close time
        //   "2434.19055334",    // Quote asset volume
        //   308,                // Number of trades
        //   "1756.87402397",    // Taker buy base asset volume
        //   "28.46694368",      // Taker buy quote asset volume
        //   "17928899.62484339" // Ignore
        // ]
        
        // Deserialize as Vec<Vec<Value>> because mixed types (int, string)
        let raw_klines: Vec<Vec<serde_json::Value>> = self.client.public_request("/api/v3/klines", &params).await?;
        
        let mut results = Vec::new();

        use rust_decimal::prelude::FromPrimitive;
        use rust_decimal::prelude::FromStr;

        for kline in raw_klines {
            if kline.len() < 6 { continue; }
            
            let open_time = kline[0].as_i64().unwrap_or_default();
            let open = kline[1].as_str().unwrap_or("0");
            let high = kline[2].as_str().unwrap_or("0");
            let low = kline[3].as_str().unwrap_or("0");
            let close = kline[4].as_str().unwrap_or("0");
            let volume = kline[5].as_str().unwrap_or("0");
            
            let price = Decimal::from_str(close).unwrap_or_default();
            let qty = Decimal::from_str(volume).unwrap_or_default();
            
            // Construct mapping
            let md = StandardMarketData {
                source: self.source_type(),
                exchange: "Binance".to_string(),
                symbol: symbol.to_string(),
                asset_type: AssetType::Crypto,
                data_type: crate::models::MarketDataType::Candle, // Use Candle or appropriate type
                price,
                quantity: qty,
                timestamp: open_time,
                received_at: Utc::now().timestamp_millis(),
                bid: None,
                ask: None,
                volume_24h: None,
                high_24h: Some(Decimal::from_str(high).unwrap_or_default()),
                low_24h: Some(Decimal::from_str(low).unwrap_or_default()),
                open_interest: None,
                funding_rate: None,
                liquidity: None,
                fdv: None,
                sequence_id: None,
                raw_data: serde_json::to_string(&kline).unwrap_or_default(),
            };
            results.push(md);
        }
        
        Ok(results)
    }
}

#[async_trait]
impl DataSourceConnector for BinanceConnector {
    fn source_type(&self) -> DataSourceType {
        DataSourceType::BinanceSpot
    }

    fn supported_assets(&self) -> Vec<AssetType> {
        vec![AssetType::Crypto]
    }

    async fn connect(&mut self) -> Result<mpsc::Receiver<StandardMarketData>> {
        self.running = true;
        let streamer = BinanceStreamer::new(self.config.ws_url.clone(), self.config.symbols.clone()); // Assuming AppConfig has Symbols
        // Actually BinanceConfig struct doesn't have symbols, but AppConfig.DataSourceConfig DOES.
        // Wait, BinanceConfig in config.rs does NOT have symbols field.
        // But AppConfig `data_sources` list has symbols.
        // We need to clarify config structure. 
        // In main.rs, existing collectors use specific config structs (massiv_config, etc).
        // Let's check config.rs again. 
        // BinanceConfig does NOT have symbols.
        // I should add `symbols` to BinanceConfig or pass it in constructor.
        // For now, let's assume valid symbols are passed via constructor wrapper or added to config.
        // I will add `symbols` to `BinanceConfig` first.
        
        streamer.connect().await
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.running = false;
        // WebSocket drop handles disconnect
        Ok(())
    }

    async fn is_healthy(&self) -> bool {
        self.running
    }

    fn stats(&self) -> ConnectorStats {
        self.stats.clone()
    }
}
