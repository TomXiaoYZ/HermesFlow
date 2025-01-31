use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};

/// WebSocket 订阅请求
#[derive(Debug, Serialize)]
pub struct SubscribeRequest {
    pub op: String,
    pub args: Vec<SubscribeArgs>,
}

/// WebSocket 订阅参数
#[derive(Debug, Serialize)]
pub struct SubscribeArgs {
    pub channel: String,
    pub inst_id: String,
}

/// WebSocket 响应
#[derive(Debug, Deserialize)]
pub struct WebSocketResponse {
    pub event: Option<String>,
    pub arg: Option<WebSocketArg>,
    pub data: Option<Vec<serde_json::Value>>,
}

/// WebSocket 参数
#[derive(Debug, Deserialize)]
pub struct WebSocketArg {
    pub channel: String,
    pub inst_id: String,
}

/// 交易对信息
#[derive(Debug, Deserialize)]
pub struct InstrumentInfo {
    pub inst_type: String,
    pub inst_id: String,
    pub base_ccy: String,
    pub quote_ccy: String,
    pub min_sz: String,
    pub tick_sz: String,
    pub lot_sz: String,
    pub state: String,
}

/// 行情数据
#[derive(Debug, Deserialize)]
pub struct Ticker {
    pub inst_id: String,
    pub last: String,
    pub last_sz: String,
    pub ask: String,
    pub ask_sz: String,
    pub bid: String,
    pub bid_sz: String,
    pub open_24h: String,
    pub high_24h: String,
    pub low_24h: String,
    pub vol_24h: String,
    pub ts: String,
}

/// 深度数据
#[derive(Debug, Deserialize)]
pub struct OrderBook {
    pub asks: Vec<[String; 4]>,
    pub bids: Vec<[String; 4]>,
    pub ts: String,
}

/// K线数据
#[derive(Debug, Deserialize)]
pub struct Kline {
    pub ts: String,
    pub o: String,
    pub h: String,
    pub l: String,
    pub c: String,
    pub vol: String,
    pub vol_ccy: String,
}

/// 交易数据
#[derive(Debug, Deserialize)]
pub struct Trade {
    pub inst_id: String,
    pub trade_id: String,
    pub px: String,
    pub sz: String,
    pub side: String,
    pub ts: String,
}

/// 账户余额
#[derive(Debug, Deserialize)]
pub struct Balance {
    /// 币种
    pub ccy: String,
    /// 可用余额
    pub avail_bal: String,
    /// 冻结余额
    pub frozen_bal: String,
    /// 总余额
    pub bal: String,
}

/// 持仓信息
#[derive(Debug, Deserialize)]
pub struct Position {
    /// 产品ID
    pub inst_id: String,
    /// 持仓方向 long/short
    pub pos_side: String,
    /// 持仓数量
    pub pos: String,
    /// 可平仓数量
    pub avail_pos: String,
    /// 开仓均价
    pub avg_px: String,
    /// 未实现收益
    pub upl: String,
    /// 杠杆倍数
    pub lever: String,
    /// 预估强平价
    pub liq_px: String,
}

/// 订单信息
#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    /// 产品ID
    pub inst_id: String,
    /// 订单ID
    pub ord_id: String,
    /// 客户订单ID
    pub cl_ord_id: Option<String>,
    /// 价格
    pub px: String,
    /// 数量
    pub sz: String,
    /// 订单状态
    pub state: String,
    /// 订单类型 market/limit
    pub ord_type: String,
    /// 交易方向 buy/sell
    pub side: String,
    /// 成交均价
    pub avg_px: Option<String>,
    /// 已成交数量
    pub acc_fill_sz: String,
    /// 手续费
    pub fee: Option<String>,
    /// 创建时间
    pub ctime: String,
}

/// 下单请求
#[derive(Debug, Serialize)]
pub struct PlaceOrderRequest {
    /// 产品ID
    pub inst_id: String,
    /// 交易方向
    pub side: String,
    /// 订单类型
    #[serde(rename = "ordType")]
    pub ord_type: String,
    /// 价格，市价单不需要
    pub px: Option<String>,
    /// 数量
    pub sz: String,
    /// 是否只减仓
    #[serde(rename = "reduceOnly")]
    pub reduce_only: Option<bool>,
    /// 客户订单ID
    #[serde(rename = "clOrdId")]
    pub cl_ord_id: Option<String>,
}

/// 撤单请求
#[derive(Debug, Serialize)]
pub struct CancelOrderRequest {
    /// 产品ID
    pub inst_id: String,
    /// 订单ID
    pub ord_id: Option<String>,
    /// 客户订单ID
    pub cl_ord_id: Option<String>,
}

/// 批量下单请求
#[derive(Debug, Serialize)]
pub struct BatchPlaceOrderRequest {
    pub orders: Vec<PlaceOrderRequest>,
}

/// 批量撤单请求
#[derive(Debug, Serialize)]
pub struct BatchCancelOrderRequest {
    pub orders: Vec<CancelOrderRequest>,
}

/// 杠杆配置
#[derive(Debug, Deserialize)]
pub struct LeverageInfo {
    /// 产品ID
    pub inst_id: String,
    /// 杠杆倍数
    pub lever: String,
    /// 保证金模式 cross/isolated
    pub mgn_mode: String,
    /// 最大杠杆倍数
    pub max_lever: String,
} 