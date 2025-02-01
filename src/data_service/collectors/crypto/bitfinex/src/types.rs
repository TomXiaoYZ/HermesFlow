use serde::{Deserialize, Serialize};

/// API响应结构
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    /// 响应代码
    pub code: i32,
    /// 响应消息
    pub message: String,
    /// 响应数据
    pub data: Option<T>,
}

/// 交易对信息
#[derive(Debug, Clone, Deserialize)]
pub struct SymbolInfo {
    /// 交易对名称
    pub pair: String,
    /// 价格精度
    pub price_precision: i32,
    /// 初始保证金
    pub initial_margin: String,
    /// 最小保证金
    pub minimum_margin: String,
    /// 最小订单数量
    pub minimum_order_size: String,
    /// 最大订单数量
    pub maximum_order_size: String,
    /// 最小价格增量
    pub minimum_price_increment: String,
    /// 是否可交易
    pub is_trading: bool,
}

/// Ticker信息
#[derive(Debug, Clone, Deserialize)]
pub struct TickerInfo {
    /// 交易对
    pub pair: String,
    /// 最新价格
    pub last_price: String,
    /// 24小时最高价
    pub high: String,
    /// 24小时最低价
    pub low: String,
    /// 24小时成交量
    pub volume: String,
    /// 24小时涨跌幅
    pub daily_change_perc: String,
}

/// 深度信息
#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookInfo {
    /// 交易对
    pub pair: String,
    /// 买单列表 [价格, 数量, 订单数]
    pub bids: Vec<(String, String, i32)>,
    /// 卖单列表 [价格, 数量, 订单数]
    pub asks: Vec<(String, String, i32)>,
}

/// 成交信息
#[derive(Debug, Clone, Deserialize)]
pub struct TradeInfo {
    /// 成交ID
    pub id: i64,
    /// 时间戳
    pub timestamp: u64,
    /// 价格
    pub price: String,
    /// 数量
    pub amount: String,
    /// 买卖方向
    pub side: String,
}

/// WebSocket认证信息
#[derive(Debug, Clone, Serialize)]
pub struct WsAuthRequest {
    /// API密钥
    pub api_key: String,
    /// 签名
    pub signature: String,
    /// 随机数
    pub nonce: String,
}

/// WebSocket订阅请求
#[derive(Debug, Clone, Serialize)]
pub struct WsSubscribeRequest {
    /// 事件类型
    pub event: String,
    /// 频道名称
    pub channel: String,
    /// 交易对
    pub pair: String,
}

/// WebSocket响应消息
#[derive(Debug, Clone, Deserialize)]
pub struct WsResponse {
    /// 事件类型
    pub event: String,
    /// 频道ID
    pub channel_id: Option<i32>,
    /// 频道名称
    pub channel: Option<String>,
    /// 消息数据
    pub data: Option<serde_json::Value>,
}
