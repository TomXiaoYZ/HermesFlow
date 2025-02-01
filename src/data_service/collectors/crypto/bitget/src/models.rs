use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 标准化的交易对信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    /// 基础货币
    pub base_currency: String,
    /// 计价货币
    pub quote_currency: String,
    /// 价格精度
    pub price_precision: u32,
    /// 数量精度
    pub amount_precision: u32,
    /// 最小交易数量
    pub min_amount: f64,
    /// 最小交易金额
    pub min_value: f64,
    /// 是否可交易
    pub is_trading: bool,
    /// 额外信息
    pub extra: HashMap<String, String>,
}

/// 标准化的Ticker信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    /// 交易对
    pub symbol: String,
    /// 最新价格
    pub last_price: f64,
    /// 24小时最高价
    pub high_24h: f64,
    /// 24小时最低价
    pub low_24h: f64,
    /// 24小时成交量(基础货币)
    pub volume_24h: f64,
    /// 24小时成交额(计价货币)
    pub amount_24h: f64,
    /// 24小时涨跌幅
    pub price_change_24h: f64,
    /// 时间戳(毫秒)
    pub timestamp: u64,
}

/// 标准化的深度信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orderbook {
    /// 交易对
    pub symbol: String,
    /// 买单列表
    pub bids: Vec<OrderbookLevel>,
    /// 卖单列表
    pub asks: Vec<OrderbookLevel>,
    /// 时间戳(毫秒)
    pub timestamp: u64,
}

/// 深度档位信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookLevel {
    /// 价格
    pub price: f64,
    /// 数量
    pub amount: f64,
    /// 订单数量
    pub count: u32,
}

/// 标准化的成交信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    /// 成交ID
    pub id: String,
    /// 交易对
    pub symbol: String,
    /// 价格
    pub price: f64,
    /// 数量
    pub amount: f64,
    /// 成交方向
    pub side: TradeSide,
    /// 时间戳(毫秒)
    pub timestamp: u64,
}

/// 成交方向
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TradeSide {
    /// 买入
    Buy,
    /// 卖出
    Sell,
}

/// 标准化的K线数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kline {
    /// 交易对
    pub symbol: String,
    /// 开盘时间(毫秒)
    pub open_time: u64,
    /// 收盘时间(毫秒)
    pub close_time: u64,
    /// 开盘价
    pub open: f64,
    /// 最高价
    pub high: f64,
    /// 最低价
    pub low: f64,
    /// 收盘价
    pub close: f64,
    /// 成交量(基础货币)
    pub volume: f64,
    /// 成交额(计价货币)
    pub amount: f64,
}

/// 标准化的交易所信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeInfo {
    /// 交易所名称
    pub name: String,
    /// 交易所状态
    pub status: ExchangeStatus,
    /// 交易对列表
    pub symbols: Vec<Symbol>,
    /// 时间戳(毫秒)
    pub timestamp: u64,
}

/// 交易所状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExchangeStatus {
    /// 正常
    Normal,
    /// 维护中
    Maintenance,
    /// 故障
    Error,
}

/// WebSocket订阅消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeMessage {
    /// 操作类型
    pub op: String,
    /// 参数
    pub args: Vec<String>,
}

/// WebSocket响应消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMessage {
    /// 事件类型
    pub event: String,
    /// 频道
    pub channel: String,
    /// 数据
    pub data: serde_json::Value,
} 