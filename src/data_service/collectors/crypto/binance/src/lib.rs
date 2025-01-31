use std::error::Error;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};

pub mod models;
pub mod websocket;
pub mod rest;
pub mod error;

#[async_trait]
pub trait DataCollector {
    type Error: Error + Send + Sync + 'static;
    
    async fn connect(&mut self) -> Result<(), Self::Error>;
    async fn disconnect(&mut self) -> Result<(), Self::Error>;
    async fn subscribe(&mut self, topics: Vec<String>) -> Result<(), Self::Error>;
    async fn unsubscribe(&mut self, topics: Vec<String>) -> Result<(), Self::Error>;
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