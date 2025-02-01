use std::fmt;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod error;
pub mod models;
pub mod metrics;

pub use error::*;
pub use models::*;
pub use metrics::*;

/// 交易所枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Exchange {
    Binance,
    Bybit,
    OKX,
    Huobi,
}

impl fmt::Display for Exchange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Exchange::Binance => write!(f, "Binance"),
            Exchange::Bybit => write!(f, "Bybit"),
            Exchange::OKX => write!(f, "OKX"),
            Exchange::Huobi => write!(f, "Huobi"),
        }
    }
}

/// 交易方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
    Unknown,
}

/// 数据质量
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataQuality {
    Real,      // 实时数据
    Delay,     // 延迟数据
    History,   // 历史数据
}

/// K线数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candlestick {
    pub timestamp: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub turnover: Decimal,
    pub trade_count: u64,
}

/// 订单簿深度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookLevel {
    pub price: Decimal,
    pub amount: Decimal,
}

/// 成交记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub price: Decimal,
    pub amount: Decimal,
    pub side: Side,
}

/// 市场数据类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketDataType {
    Trade(Vec<Trade>),
    OrderBook {
        bids: Vec<OrderBookLevel>,
        asks: Vec<OrderBookLevel>,
    },
    Candlestick(Candlestick),
}

/// 市场数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub exchange: Exchange,
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub data_type: MarketDataType,
    pub quality: DataQuality,
}

/// 采集器错误类型
#[derive(Debug, Error)]
pub enum CollectorError {
    #[error("WebSocket错误: {0}")]
    WebSocketError(String),

    #[error("REST API错误: {0}")]
    RestError(String),

    #[error("解析错误: {0}")]
    ParseError(String),

    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("系统错误: {0}")]
    SystemError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_data_creation() {
        let trade = Trade {
            id: "1".to_string(),
            timestamp: Utc::now(),
            price: Decimal::new(40000, 0),
            amount: Decimal::new(1, 0),
            side: Side::Buy,
        };

        let market_data = MarketData {
            exchange: Exchange::Huobi,
            symbol: "BTCUSDT".to_string(),
            timestamp: Utc::now(),
            data_type: MarketDataType::Trade(vec![trade]),
            quality: DataQuality::Real,
        };

        assert_eq!(market_data.exchange, Exchange::Huobi);
        assert_eq!(market_data.symbol, "BTCUSDT");
        assert_eq!(market_data.quality, DataQuality::Real);
    }

    #[test]
    fn test_order_book_creation() {
        let bids = vec![OrderBookLevel {
            price: Decimal::new(40000, 0),
            amount: Decimal::new(1, 0),
        }];
        let asks = vec![OrderBookLevel {
            price: Decimal::new(40100, 0),
            amount: Decimal::new(1, 0),
        }];

        let market_data = MarketData {
            exchange: Exchange::Huobi,
            symbol: "BTCUSDT".to_string(),
            timestamp: Utc::now(),
            data_type: MarketDataType::OrderBook { bids, asks },
            quality: DataQuality::Real,
        };

        if let MarketDataType::OrderBook { bids, asks } = market_data.data_type {
            assert_eq!(bids.len(), 1);
            assert_eq!(asks.len(), 1);
        } else {
            panic!("Wrong market data type");
        }
    }

    #[test]
    fn test_candlestick_creation() {
        let candlestick = Candlestick {
            timestamp: Utc::now(),
            open: Decimal::new(40000, 0),
            high: Decimal::new(41000, 0),
            low: Decimal::new(39000, 0),
            close: Decimal::new(40500, 0),
            volume: Decimal::new(100, 0),
            turnover: Decimal::new(4000000, 0),
            trade_count: 1000,
        };

        let market_data = MarketData {
            exchange: Exchange::Huobi,
            symbol: "BTCUSDT".to_string(),
            timestamp: Utc::now(),
            data_type: MarketDataType::Candlestick(candlestick),
            quality: DataQuality::Real,
        };

        if let MarketDataType::Candlestick(k) = market_data.data_type {
            assert_eq!(k.trade_count, 1000);
        } else {
            panic!("Wrong market data type");
        }
    }
} 