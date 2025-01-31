use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// K线数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kline {
    pub symbol: String,
    pub interval: String,
    pub start_time: DateTime<Utc>,
    pub close_time: DateTime<Utc>,
    pub open_price: Decimal,
    pub high_price: Decimal,
    pub low_price: Decimal,
    pub close_price: Decimal,
    pub volume: Decimal,
    pub quote_volume: Decimal,
    pub trades_count: i64,
    pub is_closed: bool,
}

/// 逐笔交易数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub symbol: String,
    pub trade_id: i64,
    pub price: Decimal,
    pub quantity: Decimal,
    pub buyer_order_id: i64,
    pub seller_order_id: i64,
    pub trade_time: DateTime<Utc>,
    pub is_buyer_maker: bool,
    pub is_best_match: bool,
}

/// 订单簿价格深度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookLevel {
    pub price: Decimal,
    pub quantity: Decimal,
}

/// 订单簿快照数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub symbol: String,
    pub last_update_id: i64,
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
    pub timestamp: DateTime<Utc>,
}

/// 24小时价格变动统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker24h {
    pub symbol: String,
    pub price_change: Decimal,
    pub price_change_percent: Decimal,
    pub weighted_avg_price: Decimal,
    pub last_price: Decimal,
    pub last_quantity: Decimal,
    pub open_price: Decimal,
    pub high_price: Decimal,
    pub low_price: Decimal,
    pub volume: Decimal,
    pub quote_volume: Decimal,
    pub open_time: DateTime<Utc>,
    pub close_time: DateTime<Utc>,
    pub first_trade_id: i64,
    pub last_trade_id: i64,
    pub trades_count: i64,
}

/// 迷你Ticker数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiniTicker {
    pub symbol: String,
    pub close_price: Decimal,
    pub open_price: Decimal,
    pub high_price: Decimal,
    pub low_price: Decimal,
    pub volume: Decimal,
    pub quote_volume: Decimal,
    pub event_time: DateTime<Utc>,
}

/// WebSocket订阅请求
#[derive(Debug, Clone, Serialize)]
pub struct SubscribeRequest {
    pub method: String,
    pub params: Vec<String>,
    pub id: i64,
}

/// WebSocket响应
#[derive(Debug, Clone, Deserialize)]
pub struct WebSocketResponse<T> {
    pub stream: String,
    pub data: T,
    pub event_time: DateTime<Utc>,
}

/// REST API错误响应
#[derive(Debug, Clone, Deserialize)]
pub struct ApiError {
    pub code: i32,
    pub msg: String,
}

impl SubscribeRequest {
    pub fn new(stream_names: Vec<String>) -> Self {
        Self {
            method: "SUBSCRIBE".to_string(),
            params: stream_names,
            id: chrono::Utc::now().timestamp_millis(),
        }
    }
}

impl OrderBook {
    pub fn new(symbol: String, last_update_id: i64, timestamp: DateTime<Utc>) -> Self {
        Self {
            symbol,
            last_update_id,
            bids: Vec::new(),
            asks: Vec::new(),
            timestamp,
        }
    }

    pub fn with_levels(
        symbol: String,
        last_update_id: i64,
        bids: Vec<OrderBookLevel>,
        asks: Vec<OrderBookLevel>,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            symbol,
            last_update_id,
            bids,
            asks,
            timestamp,
        }
    }
} 