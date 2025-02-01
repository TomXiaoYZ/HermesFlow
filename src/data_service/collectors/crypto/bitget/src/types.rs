use serde::{Deserialize, Serialize};
use crate::models::*;

/// API响应结构
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    /// 响应代码
    pub code: String,
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
    /// 基础货币
    pub base_coin: String,
    /// 计价货币
    pub quote_coin: String,
    /// 价格精度
    pub price_scale: i32,
    /// 数量精度
    pub size_scale: i32,
    /// 最小交易数量
    pub min_size: String,
    /// 最小交易金额
    pub min_notional: String,
    /// 状态
    pub status: String,
}

/// Ticker信息
#[derive(Debug, Clone, Deserialize)]
pub struct TickerInfo {
    /// 交易对
    pub symbol: String,
    /// 最新价格
    pub last: String,
    /// 24小时最高价
    pub high24h: String,
    /// 24小时最低价
    pub low24h: String,
    /// 24小时成交量
    pub volume24h: String,
    /// 24小时成交额
    pub usd_volume24h: String,
    /// 24小时涨跌幅
    pub price_change_pct: String,
    /// 时间戳
    pub timestamp: u64,
}

/// 深度信息
#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookInfo {
    /// 交易对
    pub symbol: String,
    /// 买单列表 [价格, 数量]
    pub bids: Vec<[String; 2]>,
    /// 卖单列表 [价格, 数量]
    pub asks: Vec<[String; 2]>,
    /// 时间戳
    pub timestamp: u64,
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
    /// 成交方向
    pub side: String,
    /// 时间戳
    pub timestamp: u64,
}

/// WebSocket认证信息
#[derive(Debug, Clone, Serialize)]
pub struct WsAuthRequest {
    /// API密钥
    pub api_key: String,
    /// 时间戳
    pub timestamp: String,
    /// 签名
    pub sign: String,
}

/// WebSocket订阅请求
#[derive(Debug, Clone, Serialize)]
pub struct WsSubscribeRequest {
    /// 操作类型
    #[serde(rename = "op")]
    pub operation: String,
    /// 参数列表
    pub args: Vec<String>,
}

/// WebSocket响应消息
#[derive(Debug, Clone, Deserialize)]
pub struct WsResponse {
    /// 操作类型
    #[serde(rename = "op")]
    pub operation: String,
    /// 频道名称
    pub channel: Option<String>,
    /// 交易对
    pub symbol: Option<String>,
    /// 数据
    pub data: Option<serde_json::Value>,
    /// 错误代码
    pub code: Option<String>,
    /// 错误消息
    pub msg: Option<String>,
}

impl From<SymbolInfo> for Symbol {
    fn from(info: SymbolInfo) -> Self {
        Self {
            symbol: info.symbol,
            base_currency: info.base_coin,
            quote_currency: info.quote_coin,
            price_precision: info.price_scale as u32,
            size_precision: info.size_scale as u32,
            min_size: info.min_size,
            min_notional: info.min_notional,
            status: info.status,
        }
    }
}

impl From<TickerInfo> for Ticker {
    fn from(info: TickerInfo) -> Self {
        Self {
            symbol: info.symbol,
            last_price: info.last,
            high_24h: info.high24h,
            low_24h: info.low24h,
            volume_24h: info.volume24h,
            quote_volume_24h: info.usd_volume24h,
            price_change_pct: info.price_change_pct,
        }
    }
}

impl From<(String, OrderbookInfo)> for Orderbook {
    fn from((symbol, info): (String, OrderbookInfo)) -> Self {
        Self {
            symbol,
            timestamp: info.timestamp,
            bids: info.bids.into_iter().map(|x| (x[0].clone(), x[1].clone())).collect(),
            asks: info.asks.into_iter().map(|x| (x[0].clone(), x[1].clone())).collect(),
        }
    }
}

impl From<(String, TradeInfo)> for Trade {
    fn from((symbol, info): (String, TradeInfo)) -> Self {
        Self {
            symbol,
            trade_id: info.trade_id,
            price: info.price,
            size: info.size,
            side: info.side,
            timestamp: info.timestamp,
        }
    }
} 