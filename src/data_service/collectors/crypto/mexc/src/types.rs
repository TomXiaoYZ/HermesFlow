use serde::{Deserialize, Serialize};
use crate::models::*;

/// API响应的基础结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// 响应代码
    pub code: i32,
    /// 响应消息
    pub msg: String,
    /// 响应数据
    pub data: Option<T>,
}

/// 交易对信息的API响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInfo {
    #[serde(rename = "symbol")]
    pub symbol: String,
    #[serde(rename = "baseAsset")]
    pub base_currency: String,
    #[serde(rename = "quoteAsset")]
    pub quote_currency: String,
    #[serde(rename = "pricePrecision")]
    pub price_precision: u32,
    #[serde(rename = "quantityPrecision")]
    pub quantity_precision: u32,
    #[serde(rename = "minQuantity")]
    pub min_quantity: String,
    #[serde(rename = "minAmount")]
    pub min_amount: String,
    #[serde(rename = "status")]
    pub status: String,
}

/// Ticker数据的API响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerInfo {
    #[serde(rename = "symbol")]
    pub symbol: String,
    #[serde(rename = "lastPrice")]
    pub last_price: String,
    #[serde(rename = "highPrice")]
    pub high_24h: String,
    #[serde(rename = "lowPrice")]
    pub low_24h: String,
    #[serde(rename = "volume")]
    pub volume_24h: String,
    #[serde(rename = "amount")]
    pub amount_24h: String,
    #[serde(rename = "priceChangePercent")]
    pub price_change_pct: String,
}

/// 深度数据的API响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookInfo {
    pub timestamp: u64,
    pub bids: Vec<[String; 2]>,
    pub asks: Vec<[String; 2]>,
}

/// 成交数据的API响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeInfo {
    #[serde(rename = "tradeId")]
    pub trade_id: String,
    pub price: String,
    #[serde(rename = "quantity")]
    pub quantity: String,
    pub side: String,
    pub timestamp: u64,
}

impl From<SymbolInfo> for Symbol {
    fn from(info: SymbolInfo) -> Self {
        Self {
            symbol: info.symbol,
            base_currency: info.base_currency,
            quote_currency: info.quote_currency,
            price_precision: info.price_precision,
            quantity_precision: info.quantity_precision,
            min_quantity: info.min_quantity,
            min_amount: info.min_amount,
            status: info.status,
        }
    }
}

impl From<TickerInfo> for Ticker {
    fn from(info: TickerInfo) -> Self {
        Self {
            symbol: info.symbol,
            last_price: info.last_price,
            high_24h: info.high_24h,
            low_24h: info.low_24h,
            volume_24h: info.volume_24h,
            amount_24h: info.amount_24h,
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
            quantity: info.quantity,
            side: info.side,
            timestamp: info.timestamp,
        }
    }
} 