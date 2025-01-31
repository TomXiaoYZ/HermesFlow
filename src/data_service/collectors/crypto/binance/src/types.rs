use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};

/// WebSocket 订阅请求
#[derive(Debug, Serialize)]
pub struct SubscribeRequest {
    pub method: String,
    pub params: Vec<String>,
    pub id: i64,
}

/// WebSocket 响应
#[derive(Debug, Deserialize)]
pub struct WebSocketResponse {
    pub result: Option<Vec<String>>,
    pub id: Option<i64>,
    #[serde(flatten)]
    pub event: Option<WebSocketEvent>,
}

/// WebSocket 事件类型
#[derive(Debug, Deserialize)]
#[serde(tag = "e", rename_all = "camelCase")]
pub enum WebSocketEvent {
    /// 交易事件
    #[serde(rename = "trade")]
    Trade(TradeEvent),
    
    /// K线事件
    #[serde(rename = "kline")]
    Kline(KlineEvent),
    
    /// 深度更新事件
    #[serde(rename = "depthUpdate")]
    Depth(DepthEvent),
    
    /// 24小时行情事件
    #[serde(rename = "24hrTicker")]
    Ticker(TickerEvent),
}

/// 交易事件
#[derive(Debug, Deserialize)]
pub struct TradeEvent {
    /// 交易对
    #[serde(rename = "s")]
    pub symbol: String,
    
    /// 成交价格
    #[serde(rename = "p")]
    pub price: Decimal,
    
    /// 成交数量
    #[serde(rename = "q")]
    pub quantity: Decimal,
    
    /// 成交时间
    #[serde(rename = "T")]
    pub time: i64,
    
    /// 是否是买方主动成交
    #[serde(rename = "m")]
    pub is_buyer_maker: bool,
    
    /// 成交ID
    #[serde(rename = "t")]
    pub trade_id: i64,
}

/// K线数据
#[derive(Debug, Deserialize)]
pub struct Kline {
    /// 开盘时间
    #[serde(rename = "t")]
    pub open_time: i64,
    
    /// 收盘时间
    #[serde(rename = "T")]
    pub close_time: i64,
    
    /// 交易对
    #[serde(rename = "s")]
    pub symbol: String,
    
    /// K线间隔
    #[serde(rename = "i")]
    pub interval: String,
    
    /// 开盘价
    #[serde(rename = "o")]
    pub open: Decimal,
    
    /// 最高价
    #[serde(rename = "h")]
    pub high: Decimal,
    
    /// 最低价
    #[serde(rename = "l")]
    pub low: Decimal,
    
    /// 收盘价
    #[serde(rename = "c")]
    pub close: Decimal,
    
    /// 成交量
    #[serde(rename = "v")]
    pub volume: Decimal,
    
    /// 成交额
    #[serde(rename = "q")]
    pub quote_volume: Decimal,
}

/// K线事件
#[derive(Debug, Deserialize)]
pub struct KlineEvent {
    /// 交易对
    #[serde(rename = "s")]
    pub symbol: String,
    
    /// K线数据
    #[serde(rename = "k")]
    pub kline: Kline,
}

/// 深度更新事件
#[derive(Debug, Deserialize)]
pub struct DepthEvent {
    /// 交易对
    #[serde(rename = "s")]
    pub symbol: String,
    
    /// 事件时间
    #[serde(rename = "E")]
    pub event_time: i64,
    
    /// 更新ID
    #[serde(rename = "u")]
    pub update_id: i64,
    
    /// 买单更新
    #[serde(rename = "b")]
    pub bids: Vec<(Decimal, Decimal)>,
    
    /// 卖单更新
    #[serde(rename = "a")]
    pub asks: Vec<(Decimal, Decimal)>,
}

/// 24小时行情事件
#[derive(Debug, Deserialize)]
pub struct TickerEvent {
    /// 交易对
    #[serde(rename = "s")]
    pub symbol: String,
    
    /// 最新价格
    #[serde(rename = "c")]
    pub close: Decimal,
    
    /// 24小时开盘价
    #[serde(rename = "o")]
    pub open: Decimal,
    
    /// 24小时最高价
    #[serde(rename = "h")]
    pub high: Decimal,
    
    /// 24小时最低价
    #[serde(rename = "l")]
    pub low: Decimal,
    
    /// 24小时成交量
    #[serde(rename = "v")]
    pub volume: Decimal,
    
    /// 24小时成交额
    #[serde(rename = "q")]
    pub quote_volume: Decimal,
    
    /// 最新成交时间
    #[serde(rename = "E")]
    pub event_time: i64,
}

/// REST API 响应包装器
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub code: Option<i32>,
    pub msg: Option<String>,
    pub data: Option<T>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_trade_event_deserialization() {
        let json = json!({
            "e": "trade",
            "s": "BTCUSDT",
            "p": "50000.00",
            "q": "1.0",
            "T": 1609459200000,
            "m": true,
            "t": 12345
        });

        let event: WebSocketEvent = serde_json::from_value(json).unwrap();
        if let WebSocketEvent::Trade(trade) = event {
            assert_eq!(trade.symbol, "BTCUSDT");
            assert_eq!(trade.price.to_string(), "50000.00");
            assert_eq!(trade.quantity.to_string(), "1.0");
            assert_eq!(trade.time, 1609459200000);
            assert!(trade.is_buyer_maker);
            assert_eq!(trade.trade_id, 12345);
        } else {
            panic!("Expected Trade event");
        }
    }

    #[test]
    fn test_kline_event_deserialization() {
        let json = json!({
            "e": "kline",
            "s": "BTCUSDT",
            "k": {
                "t": 1609459200000,
                "T": 1609459500000,
                "s": "BTCUSDT",
                "i": "5m",
                "o": "50000.00",
                "h": "51000.00",
                "l": "49000.00",
                "c": "50500.00",
                "v": "100.0",
                "q": "5050000.00"
            }
        });

        let event: WebSocketEvent = serde_json::from_value(json).unwrap();
        if let WebSocketEvent::Kline(kline) = event {
            assert_eq!(kline.symbol, "BTCUSDT");
            assert_eq!(kline.kline.interval, "5m");
            assert_eq!(kline.kline.open.to_string(), "50000.00");
            assert_eq!(kline.kline.high.to_string(), "51000.00");
            assert_eq!(kline.kline.volume.to_string(), "100.0");
        } else {
            panic!("Expected Kline event");
        }
    }

    #[test]
    fn test_depth_event_deserialization() {
        let json = json!({
            "e": "depthUpdate",
            "s": "BTCUSDT",
            "E": 1609459200000,
            "u": 12345,
            "b": [["50000.00", "1.0"], ["49900.00", "2.0"]],
            "a": [["50100.00", "1.5"], ["50200.00", "2.5"]]
        });

        let event: WebSocketEvent = serde_json::from_value(json).unwrap();
        if let WebSocketEvent::Depth(depth) = event {
            assert_eq!(depth.symbol, "BTCUSDT");
            assert_eq!(depth.update_id, 12345);
            assert_eq!(depth.bids[0].0.to_string(), "50000.00");
            assert_eq!(depth.asks[0].0.to_string(), "50100.00");
        } else {
            panic!("Expected Depth event");
        }
    }

    #[test]
    fn test_ticker_event_deserialization() {
        let json = json!({
            "e": "24hrTicker",
            "s": "BTCUSDT",
            "E": 1609459200000,
            "c": "50000.00",
            "o": "49000.00",
            "h": "51000.00",
            "l": "48000.00",
            "v": "1000.0",
            "q": "50000000.00"
        });

        let event: WebSocketEvent = serde_json::from_value(json).unwrap();
        if let WebSocketEvent::Ticker(ticker) = event {
            assert_eq!(ticker.symbol, "BTCUSDT");
            assert_eq!(ticker.close.to_string(), "50000.00");
            assert_eq!(ticker.volume.to_string(), "1000.0");
        } else {
            panic!("Expected Ticker event");
        }
    }
} 