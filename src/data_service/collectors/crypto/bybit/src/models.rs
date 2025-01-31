use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};

/// WebSocket 订阅请求
#[derive(Debug, Serialize)]
pub struct SubscribeRequest {
    pub op: String,
    pub args: Vec<String>,
}

/// WebSocket 响应
#[derive(Debug, Deserialize)]
pub struct WebSocketResponse {
    pub topic: Option<String>,
    pub event: Option<String>,
    pub data: Option<serde_json::Value>,
    pub ts: Option<i64>,
}

/// 交易对信息
#[derive(Debug, Deserialize)]
pub struct InstrumentInfo {
    pub symbol: String,
    pub status: String,
    pub base_coin: String,
    pub quote_coin: String,
    pub price_scale: i32,
    pub taker_fee: String,
    pub maker_fee: String,
    pub min_trading_qty: String,
    pub max_trading_qty: String,
    pub min_base_qty: String,
    pub min_quote_qty: String,
}

/// 行情数据
#[derive(Debug, Deserialize)]
pub struct Ticker {
    pub symbol: String,
    pub last_price: String,
    pub high_price_24h: String,
    pub low_price_24h: String,
    pub prev_price_24h: String,
    pub volume_24h: String,
    pub turnover_24h: String,
    pub price_24h_pcnt: String,
    pub usd_index_price: Option<String>,
}

/// 深度数据
#[derive(Debug, Deserialize)]
pub struct OrderBook {
    pub symbol: String,
    pub asks: Vec<[String; 2]>,
    pub bids: Vec<[String; 2]>,
    pub timestamp: i64,
}

/// K线数据
#[derive(Debug, Deserialize)]
pub struct Kline {
    pub start: i64,
    pub end: i64,
    pub interval: String,
    pub open: String,
    pub close: String,
    pub high: String,
    pub low: String,
    pub volume: String,
    pub turnover: String,
    pub confirm: bool,
    pub timestamp: i64,
}

/// 交易数据
#[derive(Debug, Deserialize)]
pub struct Trade {
    pub symbol: String,
    pub tick_direction: String,
    pub price: String,
    pub size: String,
    pub timestamp: i64,
    pub trade_time_ms: i64,
    pub side: String,
    pub trade_id: String,
} 