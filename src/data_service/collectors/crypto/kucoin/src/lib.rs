use async_trait::async_trait;

mod error;
mod types;
mod models;
mod rest;
mod websocket;
mod collector;
mod processor;

pub use error::{KucoinError, Result};
pub use models::*;
pub use types::*;

/// Kucoin配置
#[derive(Debug, Clone)]
pub struct KucoinConfig {
    /// REST API基础URL
    pub rest_base_url: String,
    /// WebSocket基础URL
    pub ws_base_url: String,
    /// API Key
    pub api_key: Option<String>,
    /// API Secret
    pub api_secret: Option<String>,
    /// API Passphrase
    pub api_passphrase: Option<String>,
}

impl Default for KucoinConfig {
    fn default() -> Self {
        Self {
            rest_base_url: "https://api.kucoin.com".to_string(),
            ws_base_url: "wss://ws-api.kucoin.com/endpoint".to_string(),
            api_key: None,
            api_secret: None,
            api_passphrase: None,
        }
    }
}

/// Kucoin数据采集器接口
#[async_trait]
pub trait KucoinCollector {
    /// 获取所有交易对信息
    async fn get_symbols(&self) -> Result<Vec<Symbol>>;
    
    /// 获取指定交易对的Ticker数据
    async fn get_ticker(&self, symbol: &str) -> Result<Ticker>;
    
    /// 获取指定交易对的深度数据
    async fn get_orderbook(&self, symbol: &str, limit: Option<u32>) -> Result<Orderbook>;
    
    /// 获取指定交易对的最新成交
    async fn get_trades(&self, symbol: &str, limit: Option<u32>) -> Result<Vec<Trade>>;
    
    /// 订阅市场数据
    async fn subscribe_market_data(&mut self, symbols: Vec<String>, channels: Vec<String>) -> Result<()>;
    
    /// 取消订阅市场数据
    async fn unsubscribe_market_data(&mut self, symbols: Vec<String>, channels: Vec<String>) -> Result<()>;
}
