use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// 市场数据类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MarketDataType {
    Trade,           // 交易数据
    OrderBook,       // 订单簿数据
    Kline,          // K线数据
    Ticker,         // 行情数据
    BestQuote,      // 最优报价
    IndexPrice,     // 指数价格
    MarkPrice,      // 标记价格
    FundingRate,    // 资金费率
}

/// 统一市场数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub exchange: String,                    // 交易所名称
    pub symbol: String,                      // 交易对
    pub data_type: MarketDataType,          // 数据类型
    pub timestamp: DateTime<Utc>,           // 数据时间戳
    pub received_at: DateTime<Utc>,         // 数据接收时间
    pub raw_data: serde_json::Value,        // 原始数据
    pub metadata: HashMap<String, String>,   // 元数据
}

/// 统一K线数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kline {
    pub exchange: String,
    pub symbol: String,
    pub interval: String,
    pub start_time: DateTime<Utc>,
    pub close_time: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub quote_volume: Decimal,
    pub trades_count: i64,
    pub is_closed: bool,
}

/// 统一交易数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub exchange: String,
    pub symbol: String,
    pub trade_id: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub side: TradeSide,
    pub trade_time: DateTime<Utc>,
    pub is_maker: bool,
    pub metadata: HashMap<String, String>,
}

/// 交易方向
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradeSide {
    Buy,
    Sell,
}

/// 统一订单簿数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub exchange: String,
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
    pub update_id: i64,
    pub metadata: HashMap<String, String>,
}

/// 价格深度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price: Decimal,
    pub quantity: Decimal,
}

/// 统一Ticker数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub exchange: String,
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub last_price: Decimal,
    pub last_quantity: Option<Decimal>,
    pub best_bid: Decimal,
    pub best_ask: Decimal,
    pub volume_24h: Decimal,
    pub quote_volume_24h: Decimal,
    pub high_24h: Option<Decimal>,
    pub low_24h: Option<Decimal>,
    pub open_24h: Option<Decimal>,
    pub metadata: HashMap<String, String>,
}

/// 数据质量指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQuality {
    pub latency: i64,              // 数据延迟（毫秒）
    pub is_gap: bool,              // 是否存在数据gap
    pub gap_size: Option<i64>,     // gap大小（如果存在）
    pub is_valid: bool,            // 数据是否有效
    pub error_type: Option<String>, // 错误类型（如果存在）
    pub metadata: HashMap<String, String>, // 其他质量指标
}

impl MarketData {
    pub fn new(
        exchange: String,
        symbol: String,
        data_type: MarketDataType,
        raw_data: serde_json::Value,
    ) -> Self {
        let now = Utc::now();
        Self {
            exchange,
            symbol,
            data_type,
            timestamp: now,
            received_at: now,
            raw_data,
            metadata: HashMap::new(),
        }
    }

    pub fn with_timestamp(
        exchange: String,
        symbol: String,
        data_type: MarketDataType,
        timestamp: DateTime<Utc>,
        raw_data: serde_json::Value,
    ) -> Self {
        Self {
            exchange,
            symbol,
            data_type,
            timestamp,
            received_at: Utc::now(),
            raw_data,
            metadata: HashMap::new(),
        }
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
} 