use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

/// WebSocket 订阅请求
#[derive(Debug, Serialize)]
pub struct SubscribeRequest {
    pub sub: String,
    pub id: String,
}

/// WebSocket 取消订阅请求
#[derive(Debug, Serialize)]
pub struct UnsubscribeRequest {
    pub unsub: String,
    pub id: String,
}

/// WebSocket Ping 请求
#[derive(Debug, Serialize)]
pub struct PingRequest {
    pub ping: i64,
}

/// WebSocket Pong 响应
#[derive(Debug, Deserialize)]
pub struct PongResponse {
    pub pong: i64,
}

/// WebSocket 订阅响应
#[derive(Debug, Deserialize)]
pub struct SubscriptionResponse {
    pub id: String,
    pub status: String,
    #[serde(default)]
    pub subbed: String,
    #[serde(default)]
    pub ts: i64,
    #[serde(default)]
    pub err_code: Option<i32>,
    #[serde(default)]
    pub err_msg: Option<String>,
}

/// K线数据
#[derive(Debug, Clone, Deserialize)]
pub struct Kline {
    pub id: i64,              // K线ID
    pub amount: Decimal,      // 成交量(币)
    pub count: i64,           // 成交笔数
    pub open: Decimal,        // 开盘价
    pub close: Decimal,       // 收盘价
    pub low: Decimal,         // 最低价
    pub high: Decimal,        // 最高价
    pub vol: Decimal,         // 成交额(计价币)
}

/// K线数据响应
#[derive(Debug, Deserialize)]
pub struct KlineResponse {
    pub ch: String,
    pub ts: i64,
    pub tick: Kline,
}

/// 市场深度数据
#[derive(Debug, Clone, Deserialize)]
pub struct Depth {
    pub ts: i64,
    pub version: i64,
    pub bids: Vec<[Decimal; 2]>,  // [价格, 数量]
    pub asks: Vec<[Decimal; 2]>,  // [价格, 数量]
}

/// 深度数据响应
#[derive(Debug, Deserialize)]
pub struct DepthResponse {
    pub ch: String,
    pub ts: i64,
    pub tick: Depth,
}

/// 交易详情
#[derive(Debug, Clone, Deserialize)]
pub struct Trade {
    pub id: i64,
    pub ts: i64,
    pub amount: Decimal,
    pub price: Decimal,
    pub direction: String,    // "buy" 或 "sell"
}

/// 交易数据响应
#[derive(Debug, Deserialize)]
pub struct TradeResponse {
    pub ch: String,
    pub ts: i64,
    pub tick: TradeDetail,
}

#[derive(Debug, Deserialize)]
pub struct TradeDetail {
    pub id: i64,
    pub ts: i64,
    pub data: Vec<Trade>,
}

/// 24小时行情数据
#[derive(Debug, Clone, Deserialize)]
pub struct Ticker {
    pub id: i64,
    pub amount: Decimal,      // 24小时成交量
    pub count: i64,          // 24小时成交笔数
    pub open: Decimal,       // 24小时开盘价
    pub close: Decimal,      // 最新价
    pub low: Decimal,        // 24小时最低价
    pub high: Decimal,       // 24小时最高价
    pub vol: Decimal,        // 24小时成交额
    pub bid: Decimal,        // 买一价
    #[serde(rename = "bidSize")]
    pub bid_size: Decimal,    // 买一量
    pub ask: Decimal,        // 卖一价
    #[serde(rename = "askSize")]
    pub ask_size: Decimal,    // 卖一量
}

/// 行情数据响应
#[derive(Debug, Deserialize)]
pub struct TickerResponse {
    pub ch: String,
    pub ts: i64,
    pub tick: Ticker,
}

/// 交易对信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub symbol: String,                  // 交易对
    pub state: String,                   // 交易对状态
    #[serde(rename = "base-currency")]
    pub base_currency: String,           // 基础币种
    #[serde(rename = "quote-currency")]
    pub quote_currency: String,          // 计价币种
    #[serde(rename = "price-precision")]
    pub price_precision: i32,            // 价格精度
    #[serde(rename = "amount-precision")]
    pub amount_precision: i32,           // 数量精度
    #[serde(rename = "value-precision")]
    pub value_precision: i32,            // 交易额精度
    #[serde(rename = "min-order-amt")]
    pub min_order_amount: Decimal,       // 最小下单数量
    #[serde(rename = "max-order-amt")]
    pub max_order_amount: Decimal,       // 最大下单数量
    #[serde(rename = "min-order-value")]
    pub min_order_value: Decimal,        // 最小下单金额
    #[serde(rename = "leverage-ratio", default)]
    pub leverage_ratio: Option<Decimal>, // 杠杆比例
}

/// REST API 通用响应格式
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub status: String,
    pub data: Option<T>,
    #[serde(default)]
    pub ts: i64,
    #[serde(rename = "err-code")]
    pub err_code: Option<String>,
    #[serde(rename = "err-msg")]
    pub err_msg: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_subscribe_request_serialization() {
        let req = SubscribeRequest {
            sub: "market.btcusdt.kline.1min".to_string(),
            id: "id1".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("market.btcusdt.kline.1min"));
    }

    #[test]
    fn test_kline_deserialization() {
        let json = json!({
            "ch": "market.btcusdt.kline.1min",
            "ts": 1630000000000_i64,
            "tick": {
                "id": 1630000000,
                "amount": "1.23",
                "count": 100,
                "open": "40000.0",
                "close": "41000.0",
                "low": "39800.0",
                "high": "41200.0",
                "vol": "49200.0"
            }
        });
        let resp: KlineResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.tick.count, 100);
    }

    #[test]
    fn test_trade_deserialization() {
        let json = json!({
            "ch": "market.btcusdt.trade.detail",
            "ts": 1630000000000_i64,
            "tick": {
                "id": 1630000000,
                "ts": 1630000000000_i64,
                "data": [{
                    "id": 1,
                    "ts": 1630000000000_i64,
                    "amount": "0.1",
                    "price": "40000.0",
                    "direction": "buy"
                }]
            }
        });
        let resp: TradeResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.tick.data[0].direction, "buy");
    }

    #[test]
    fn test_depth_deserialization() {
        let json = json!({
            "ch": "market.btcusdt.depth.step0",
            "ts": 1630000000000_i64,
            "tick": {
                "ts": 1630000000000_i64,
                "version": 100,
                "bids": [["40000.0", "1.0"]],
                "asks": [["40100.0", "1.0"]]
            }
        });
        let resp: DepthResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.tick.version, 100);
    }

    #[test]
    fn test_symbol_serialization() {
        let symbol = Symbol {
            symbol: "btcusdt".to_string(),
            state: "online".to_string(),
            base_currency: "btc".to_string(),
            quote_currency: "usdt".to_string(),
            price_precision: 2,
            amount_precision: 6,
            value_precision: 2,
            min_order_amount: Decimal::new(1, 4),
            max_order_amount: Decimal::new(1000, 0),
            min_order_value: Decimal::new(5, 0),
            leverage_ratio: Some(Decimal::new(5, 0)),
        };

        let json = serde_json::to_string(&symbol).unwrap();
        let deserialized: Symbol = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.symbol, "btcusdt");
    }
} 