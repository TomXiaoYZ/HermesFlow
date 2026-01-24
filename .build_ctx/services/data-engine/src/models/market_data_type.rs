use serde::{Deserialize, Serialize};

/// Market data type classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MarketDataType {
    /// Individual trade execution
    Trade,
    /// 24-hour ticker statistics
    Ticker,
    /// Candlestick/OHLCV data
    Candle,
    /// Crypto Kline data independent of Candle to avoid conflicts if needed, or aliased logially
    Kline,
    /// Order book depth update
    OrderBook,
    /// Funding rate for perpetual contracts
    FundingRate,
}

impl MarketDataType {
    /// Returns a string representation of the market data type
    pub fn as_str(&self) -> &'static str {
        match self {
            MarketDataType::Trade => "Trade",
            MarketDataType::Ticker => "Ticker",
            MarketDataType::Candle => "Candle",
            MarketDataType::Kline => "Kline",
            MarketDataType::OrderBook => "OrderBook",
            MarketDataType::FundingRate => "FundingRate",
        }
    }
}

impl std::fmt::Display for MarketDataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_data_type_serialization() {
        let data_type = MarketDataType::Trade;
        let json = serde_json::to_string(&data_type).unwrap();
        assert_eq!(json, "\"Trade\"");

        let deserialized: MarketDataType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, MarketDataType::Trade);
    }

    #[test]
    fn test_market_data_type_display() {
        assert_eq!(MarketDataType::Trade.to_string(), "Trade");
        assert_eq!(MarketDataType::Ticker.to_string(), "Ticker");
        assert_eq!(MarketDataType::OrderBook.to_string(), "OrderBook");
    }
}
