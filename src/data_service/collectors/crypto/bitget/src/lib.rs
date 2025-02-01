pub mod types;
pub mod error;
pub mod models;
pub mod rest;
pub mod websocket;
pub mod collector;
pub mod processor;

use async_trait::async_trait;
use error::BitgetError;
use models::*;
use rest::BitgetRestClient;
use websocket::BitgetWebsocketClient;

pub type Result<T> = std::result::Result<T, BitgetError>;

/// Bitget数据采集器的配置
#[derive(Debug, Clone)]
pub struct BitgetConfig {
    /// REST API的基础URL
    pub rest_base_url: String,
    /// WebSocket的基础URL
    pub ws_base_url: String,
    /// API Key
    pub api_key: Option<String>,
    /// API Secret
    pub api_secret: Option<String>,
    /// 是否使用测试网络
    pub is_testnet: bool,
}

impl Default for BitgetConfig {
    fn default() -> Self {
        Self {
            rest_base_url: "https://api.bitget.com".to_string(),
            ws_base_url: "wss://ws.bitget.com/spot/v1/stream".to_string(),
            api_key: None,
            api_secret: None,
            is_testnet: false,
        }
    }
}

/// Bitget数据采集器的特征定义
#[async_trait]
pub trait BitgetCollector {
    /// 获取所有交易对信息
    async fn get_symbols(&self) -> Result<Vec<Symbol>>;
    
    /// 获取指定交易对的Ticker数据
    async fn get_ticker(&self, symbol: &str) -> Result<Ticker>;
    
    /// 获取指定交易对的深度数据
    async fn get_orderbook(&self, symbol: &str, limit: Option<u32>) -> Result<Orderbook>;
    
    /// 获取指定交易对的最新成交
    async fn get_trades(&self, symbol: &str, limit: Option<u32>) -> Result<Vec<Trade>>;
    
    /// 订阅指定交易对的实时数据
    async fn subscribe_market_data(&mut self, symbols: Vec<String>, channels: Vec<String>) -> Result<()>;
    
    /// 取消订阅指定交易对的实时数据
    async fn unsubscribe_market_data(&mut self, symbols: Vec<String>, channels: Vec<String>) -> Result<()>;
} 