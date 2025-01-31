use std::error::Error;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};

// 引入通用框架
use crate::collectors::common::{
    DataCollector, CollectorConfig, CollectorStatus,
    MarketData, DataQuality,
};

pub mod models;
pub mod websocket;
pub mod rest;
pub mod error;

use error::BinanceError;
use models::*;

/// Binance数据采集器
pub struct BinanceCollector {
    config: Option<CollectorConfig>,
    status: CollectorStatus,
    ws_client: Option<websocket::WebSocketClient>,
    rest_client: Option<rest::RestClient>,
}

impl BinanceCollector {
    pub fn new() -> Self {
        Self {
            config: None,
            status: CollectorStatus {
                is_connected: false,
                last_received: None,
                subscribed_channels: Vec::new(),
                error_count: 0,
                reconnect_count: 0,
                metadata: Default::default(),
            },
            ws_client: None,
            rest_client: None,
        }
    }
}

#[async_trait]
impl DataCollector for BinanceCollector {
    type Error = BinanceError;

    async fn init(&mut self, config: CollectorConfig) -> Result<(), Self::Error> {
        self.config = Some(config.clone());
        self.ws_client = Some(websocket::WebSocketClient::new(&config.ws_endpoint));
        self.rest_client = Some(rest::RestClient::new(
            &config.rest_endpoint,
            config.api_key.as_deref(),
            config.api_secret.as_deref(),
        ));
        Ok(())
    }

    async fn connect(&mut self) -> Result<(), Self::Error> {
        if let Some(ws_client) = &mut self.ws_client {
            ws_client.connect().await?;
            self.status.is_connected = true;
            self.status.reconnect_count += 1;
        }
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), Self::Error> {
        if let Some(ws_client) = &mut self.ws_client {
            ws_client.disconnect().await?;
            self.status.is_connected = false;
        }
        Ok(())
    }

    async fn subscribe(&mut self, channels: Vec<String>) -> Result<(), Self::Error> {
        if let Some(ws_client) = &mut self.ws_client {
            ws_client.subscribe(channels.clone()).await?;
            self.status.subscribed_channels.extend(channels);
        }
        Ok(())
    }

    async fn unsubscribe(&mut self, channels: Vec<String>) -> Result<(), Self::Error> {
        if let Some(ws_client) = &mut self.ws_client {
            ws_client.unsubscribe(channels.clone()).await?;
            self.status.subscribed_channels.retain(|c| !channels.contains(c));
        }
        Ok(())
    }

    async fn get_status(&self) -> CollectorStatus {
        self.status.clone()
    }

    async fn start(
        &mut self,
        tx: tokio::sync::mpsc::Sender<(MarketData, DataQuality)>,
    ) -> Result<(), Self::Error> {
        if let Some(ws_client) = &mut self.ws_client {
            ws_client.start(tx).await?;
        }
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Self::Error> {
        if let Some(ws_client) = &mut self.ws_client {
            ws_client.stop().await?;
        }
        Ok(())
    }
}

impl Default for BinanceCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub exchange: String,
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub data_type: MarketDataType,
    pub raw_data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketDataType {
    Trade,
    OrderBook,
    Kline,
    Ticker,
} 