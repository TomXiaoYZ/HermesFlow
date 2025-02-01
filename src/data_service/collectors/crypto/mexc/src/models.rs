use serde::{Deserialize, Serialize};

/// 交易对信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    /// 交易对名称
    pub symbol: String,
    /// 基础货币
    pub base_currency: String,
    /// 计价货币
    pub quote_currency: String,
    /// 价格精度
    pub price_precision: u32,
    /// 数量精度
    pub quantity_precision: u32,
    /// 最小交易数量
    pub min_quantity: String,
    /// 最小交易金额
    pub min_amount: String,
    /// 状态
    pub status: String,
}

/// Ticker数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    /// 交易对
    pub symbol: String,
    /// 最新价格
    pub last_price: String,
    /// 24小时最高价
    pub high_24h: String,
    /// 24小时最低价
    pub low_24h: String,
    /// 24小时成交量
    pub volume_24h: String,
    /// 24小时成交额
    pub amount_24h: String,
    /// 24小时涨跌幅
    pub price_change_pct: String,
}

/// 深度数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orderbook {
    /// 交易对
    pub symbol: String,
    /// 时间戳
    pub timestamp: u64,
    /// 买单
    pub bids: Vec<(String, String)>,
    /// 卖单
    pub asks: Vec<(String, String)>,
}

/// 成交数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    /// 交易对
    pub symbol: String,
    /// 成交ID
    pub trade_id: String,
    /// 成交价格
    pub price: String,
    /// 成交数量
    pub quantity: String,
    /// 成交方向
    pub side: String,
    /// 成交时间
    pub timestamp: u64,
}

/// WebSocket订阅消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeMessage {
    /// 方法
    pub method: String,
    /// 参数
    pub params: Vec<String>,
    /// 请求ID
    pub id: u64,
}

/// WebSocket响应消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMessage {
    /// 频道
    pub channel: String,
    /// 数据
    pub data: serde_json::Value,
    /// 时间戳
    pub ts: u64,
} 