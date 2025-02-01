use serde::{Deserialize, Serialize};

/// API响应结构
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    /// 响应代码
    pub code: i32,
    /// 响应消息
    pub msg: String,
    /// 响应数据
    pub data: Option<T>,
}

/// 交易对信息
#[derive(Debug, Clone, Deserialize)]
pub struct SymbolInfo {
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
    pub min_size: String,
    /// 最小交易金额
    pub min_funds: String,
    /// 是否可交易
    pub enable_trading: bool,
}

/// Ticker信息
#[derive(Debug, Clone, Deserialize)]
pub struct TickerInfo {
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
}

/// 深度信息
#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookInfo {
    /// 时间戳
    pub timestamp: u64,
    /// 买单列表
    pub bids: Vec<(String, String)>,
    /// 卖单列表
    pub asks: Vec<(String, String)>,
}

/// 成交信息
#[derive(Debug, Clone, Deserialize)]
pub struct TradeInfo {
    /// 成交ID
    pub trade_id: String,
    /// 价格
    pub price: String,
    /// 数量
    pub size: String,
    /// 成交时间
    pub timestamp: u64,
    /// 成交方向
    pub side: String,
}

/// WebSocket连接信息
#[derive(Debug, Clone, Deserialize)]
pub struct WebsocketTokenInfo {
    /// Token
    pub token: String,
    /// 服务器实例列表
    pub servers: Vec<WebsocketServerInfo>,
}

/// WebSocket服务器信息
#[derive(Debug, Clone, Deserialize)]
pub struct WebsocketServerInfo {
    /// 服务器地址
    pub endpoint: String,
    /// 协议类型
    pub protocol: String,
    /// 是否加密
    pub encrypt: bool,
    /// 心跳间隔(毫秒)
    pub ping_interval: u64,
    /// 心跳超时时间(毫秒)
    pub ping_timeout: u64,
}
