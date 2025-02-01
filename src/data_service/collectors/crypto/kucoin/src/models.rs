use serde::{Deserialize, Serialize};
use crate::types::*;

/// 标准化的交易对信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    /// 交易所名称
    pub exchange: String,
    /// 交易对名称
    pub symbol: String,
    /// 基础币种
    pub base_currency: String,
    /// 计价币种
    pub quote_currency: String,
    /// 价格精度
    pub price_precision: i32,
    /// 数量精度
    pub size_precision: i32,
    /// 最小交易数量
    pub min_size: f64,
    /// 最小交易金额
    pub min_funds: f64,
}

impl From<SymbolInfo> for Symbol {
    fn from(info: SymbolInfo) -> Self {
        Self {
            exchange: "kucoin".to_string(),
            symbol: info.symbol,
            base_currency: info.base_currency,
            quote_currency: info.quote_currency,
            price_precision: info.price_precision,
            size_precision: info.size_precision,
            min_size: info.min_size.parse().unwrap_or(0.0),
            min_funds: info.min_funds.parse().unwrap_or(0.0),
        }
    }
}

/// 标准化的Ticker信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    /// 交易所名称
    pub exchange: String,
    /// 交易对名称
    pub symbol: String,
    /// 最新价格
    pub last_price: f64,
    /// 24小时最高价
    pub high_24h: f64,
    /// 24小时最低价
    pub low_24h: f64,
    /// 24小时成交量
    pub volume_24h: f64,
    /// 24小时成交额
    pub amount_24h: f64,
}

impl From<TickerInfo> for Ticker {
    fn from(info: TickerInfo) -> Self {
        Self {
            exchange: "kucoin".to_string(),
            symbol: info.symbol,
            last_price: info.last_price.parse().unwrap_or(0.0),
            high_24h: info.high_24h.parse().unwrap_or(0.0),
            low_24h: info.low_24h.parse().unwrap_or(0.0),
            volume_24h: info.volume_24h.parse().unwrap_or(0.0),
            amount_24h: info.amount_24h.parse().unwrap_or(0.0),
        }
    }
}

/// 标准化的深度信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orderbook {
    /// 交易所名称
    pub exchange: String,
    /// 交易对名称
    pub symbol: String,
    /// 时间戳
    pub timestamp: u64,
    /// 买单列表
    pub bids: Vec<(f64, f64)>,
    /// 卖单列表
    pub asks: Vec<(f64, f64)>,
}

impl From<(String, OrderbookInfo)> for Orderbook {
    fn from((symbol, info): (String, OrderbookInfo)) -> Self {
        Self {
            exchange: "kucoin".to_string(),
            symbol,
            timestamp: info.timestamp,
            bids: info.bids
                .into_iter()
                .map(|(price, size)| (
                    price.parse().unwrap_or(0.0),
                    size.parse().unwrap_or(0.0)
                ))
                .collect(),
            asks: info.asks
                .into_iter()
                .map(|(price, size)| (
                    price.parse().unwrap_or(0.0),
                    size.parse().unwrap_or(0.0)
                ))
                .collect(),
        }
    }
}

/// 标准化的成交信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    /// 交易所名称
    pub exchange: String,
    /// 交易对名称
    pub symbol: String,
    /// 成交ID
    pub trade_id: String,
    /// 价格
    pub price: f64,
    /// 数量
    pub size: f64,
    /// 成交时间
    pub timestamp: u64,
    /// 成交方向
    pub side: String,
}

impl From<(String, TradeInfo)> for Trade {
    fn from((symbol, info): (String, TradeInfo)) -> Self {
        Self {
            exchange: "kucoin".to_string(),
            symbol,
            trade_id: info.trade_id,
            price: info.price.parse().unwrap_or(0.0),
            size: info.size.parse().unwrap_or(0.0),
            timestamp: info.timestamp,
            side: info.side,
        }
    }
}

/// WebSocket订阅消息
#[derive(Debug, Serialize)]
pub struct SubscribeMessage {
    /// 请求方法
    pub method: String,
    /// 订阅参数
    pub params: Vec<String>,
    /// 请求ID
    pub id: u64,
}

/// WebSocket响应消息
#[derive(Debug, Deserialize)]
pub struct ResponseMessage {
    /// 频道名称
    pub channel: String,
    /// 消息数据
    pub data: serde_json::Value,
}
