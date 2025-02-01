use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};

/// WebSocket 订阅请求
#[derive(Debug, Serialize)]
pub struct SubscribeRequest {
    pub op: String,
    pub args: Vec<SubscribeArgs>,
}

/// WebSocket 订阅参数
#[derive(Debug, Serialize)]
pub struct SubscribeArgs {
    pub channel: String,
    pub inst_id: String,
}

/// WebSocket 响应
#[derive(Debug, Deserialize)]
pub struct WebSocketResponse {
    pub event: Option<String>,
    pub channel: Option<String>,
    pub data: Option<Vec<serde_json::Value>>,
}

/// 行情数据
#[derive(Debug, Deserialize)]
pub struct Ticker {
    pub inst_id: String,
    pub last: Decimal,
    pub last_sz: Decimal,
    pub ask_px: Decimal,
    pub ask_sz: Decimal,
    pub bid_px: Decimal,
    pub bid_sz: Decimal,
    pub open_24h: Decimal,
    pub high_24h: Decimal,
    pub low_24h: Decimal,
    pub vol_24h: Decimal,
    pub ts: i64,
}

/// 成交数据
#[derive(Debug, Deserialize)]
pub struct Trade {
    pub inst_id: String,
    pub trade_id: String,
    pub px: Decimal,
    pub sz: Decimal,
    pub side: String,
    pub ts: i64,
}

/// 深度数据
#[derive(Debug, Deserialize)]
pub struct OrderBook {
    pub inst_id: String,
    pub asks: Vec<(Decimal, Decimal)>,
    pub bids: Vec<(Decimal, Decimal)>,
    pub ts: i64,
}

/// K线数据
#[derive(Debug, Deserialize)]
pub struct Kline {
    pub inst_id: String,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub vol: Decimal,
    pub ts: i64,
}

/// 市场数据类型
#[derive(Debug)]
pub enum MarketDataType {
    Trade,
    OrderBook,
    Ticker,
    Kline,
}

/// 市场数据质量
#[derive(Debug)]
pub struct DataQuality {
    pub latency: i64,
    pub is_gap: bool,
    pub gap_size: Option<i64>,
    pub is_valid: bool,
    pub error_type: Option<String>,
}

impl Default for DataQuality {
    fn default() -> Self {
        Self {
            latency: 0,
            is_gap: false,
            gap_size: None,
            is_valid: true,
            error_type: None,
        }
    }
} 