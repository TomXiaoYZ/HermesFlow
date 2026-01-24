use serde::{Deserialize, Serialize};

/// Data source type identification for different exchanges and markets
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DataSourceType {
    // Binance
    BinanceSpot,
    BinanceFutures,
    BinancePerp,

    // OKX
    OkxSpot,
    OkxFutures,
    OkxPerp,

    // Bybit
    BybitSpot,
    BybitFutures,
    BybitPerp,

    // Futu
    FutuStock,

    // Bitget
    BitgetSpot,
    BitgetFutures,

    // DEX
    GmgnDex,
    UniswapV3,
    Birdeye,

    // Traditional Finance
    IbkrStock,
    IbkrOption,
    PolygonStock,
    AlpacaStock,

    // Sentiment Data
    TwitterSentiment,
    NewsApiSentiment,

    // Prediction Markets
    PolymarketGamma,

    // A-Shares
    AkShare,

    // Macro Data
    FredMacro,
}

impl DataSourceType {
    /// Returns a string representation of the data source type
    pub fn as_str(&self) -> &'static str {
        match self {
            DataSourceType::BinanceSpot => "BinanceSpot",
            DataSourceType::BinanceFutures => "BinanceFutures",
            DataSourceType::BinancePerp => "BinancePerp",
            DataSourceType::OkxSpot => "OkxSpot",
            DataSourceType::OkxFutures => "OkxFutures",
            DataSourceType::OkxPerp => "OkxPerp",
            DataSourceType::BybitSpot => "BybitSpot",
            DataSourceType::BybitFutures => "BybitFutures",
            DataSourceType::BybitPerp => "BybitPerp",
            DataSourceType::FutuStock => "FutuStock",
            DataSourceType::BitgetSpot => "BitgetSpot",
            DataSourceType::BitgetFutures => "BitgetFutures",
            DataSourceType::GmgnDex => "GmgnDex",
            DataSourceType::UniswapV3 => "UniswapV3",
            DataSourceType::Birdeye => "Birdeye",
            DataSourceType::IbkrStock => "IbkrStock",
            DataSourceType::IbkrOption => "IbkrOption",
            DataSourceType::PolygonStock => "PolygonStock",
            DataSourceType::AlpacaStock => "AlpacaStock",
            DataSourceType::TwitterSentiment => "TwitterSentiment",
            DataSourceType::NewsApiSentiment => "NewsApiSentiment",
            DataSourceType::PolymarketGamma => "PolymarketGamma",
            DataSourceType::AkShare => "AkShare",
            DataSourceType::FredMacro => "FredMacro",
        }
    }

    /// Returns the exchange name
    pub fn exchange(&self) -> &'static str {
        match self {
            DataSourceType::BinanceSpot
            | DataSourceType::BinanceFutures
            | DataSourceType::BinancePerp => "Binance",
            DataSourceType::OkxSpot | DataSourceType::OkxFutures | DataSourceType::OkxPerp => "OKX",
            DataSourceType::BybitSpot | DataSourceType::BybitFutures | DataSourceType::BybitPerp => "Bybit",
            DataSourceType::FutuStock => "Futu",
            DataSourceType::BitgetSpot | DataSourceType::BitgetFutures => "Bitget",
            DataSourceType::GmgnDex => "GMGN",
            DataSourceType::UniswapV3 => "Uniswap",
            DataSourceType::Birdeye => "Birdeye",
            DataSourceType::IbkrStock | DataSourceType::IbkrOption => "IBKR",
            DataSourceType::PolygonStock => "Polygon",
            DataSourceType::AlpacaStock => "Alpaca",
            DataSourceType::TwitterSentiment => "Twitter",
            DataSourceType::NewsApiSentiment => "NewsAPI",
            DataSourceType::PolymarketGamma => "Polymarket",
            DataSourceType::AkShare => "AkShare",
            DataSourceType::FredMacro => "FRED",
        }
    }
}

impl std::fmt::Display for DataSourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_source_type_serialization() {
        let source = DataSourceType::BinanceSpot;
        let json = serde_json::to_string(&source).unwrap();
        assert_eq!(json, "\"BinanceSpot\"");

        let deserialized: DataSourceType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, DataSourceType::BinanceSpot);
    }

    #[test]
    fn test_data_source_type_display() {
        assert_eq!(DataSourceType::BinanceSpot.to_string(), "BinanceSpot");
        assert_eq!(DataSourceType::OkxSpot.to_string(), "OkxSpot");
    }

    #[test]
    fn test_exchange_name() {
        assert_eq!(DataSourceType::BinanceSpot.exchange(), "Binance");
        assert_eq!(DataSourceType::OkxSpot.exchange(), "OKX");
        assert_eq!(DataSourceType::IbkrStock.exchange(), "IBKR");
        assert_eq!(DataSourceType::FutuStock.exchange(), "Futu");
    }
}
