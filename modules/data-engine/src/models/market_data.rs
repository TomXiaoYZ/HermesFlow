use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{AssetType, DataSourceType, MarketDataType};

/// Standardized market data structure for all data sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardMarketData {
    // Source identification
    /// Data source type (e.g., BinanceSpot, OkxFutures)
    pub source: DataSourceType,
    /// Exchange name (e.g., "Binance", "OKX")
    pub exchange: String,
    /// Trading pair symbol (e.g., "BTCUSDT")
    pub symbol: String,
    /// Asset type (e.g., Spot, Perpetual)
    pub asset_type: AssetType,

    // Market data
    /// Type of market data (Trade, Ticker, Kline, etc.)
    pub data_type: MarketDataType,
    /// Last/current price
    pub price: Decimal,
    /// Volume/quantity
    pub quantity: Decimal,
    /// Exchange timestamp (milliseconds since epoch)
    pub timestamp: i64,
    /// System received timestamp (milliseconds since epoch)
    pub received_at: i64,

    // Optional fields
    /// Best bid price
    pub bid: Option<Decimal>,
    /// Best ask price
    pub ask: Option<Decimal>,
    /// 24-hour high price
    pub high_24h: Option<Decimal>,
    /// 24-hour low price
    pub low_24h: Option<Decimal>,
    /// 24-hour trading volume
    pub volume_24h: Option<Decimal>,
    /// Open interest (for futures/perpetuals)
    pub open_interest: Option<Decimal>,
    /// Funding rate (for perpetuals)
    pub funding_rate: Option<Decimal>,

    // Metadata
    /// Sequence ID for message ordering
    pub sequence_id: Option<u64>,
    /// Original raw message (for debugging)
    pub raw_data: String,
}

impl Default for StandardMarketData {
    fn default() -> Self {
        Self {
            source: DataSourceType::BinanceSpot,
            exchange: String::new(),
            symbol: String::new(),
            asset_type: AssetType::Spot,
            data_type: MarketDataType::Trade,
            price: Decimal::ZERO,
            quantity: Decimal::ZERO,
            timestamp: 0,
            received_at: 0,
            bid: None,
            ask: None,
            high_24h: None,
            low_24h: None,
            volume_24h: None,
            open_interest: None,
            funding_rate: None,
            sequence_id: None,
            raw_data: String::new(),
        }
    }
}

impl StandardMarketData {
    /// Creates a new StandardMarketData instance with required fields
    pub fn new(
        source: DataSourceType,
        symbol: String,
        asset_type: AssetType,
        data_type: MarketDataType,
        price: Decimal,
        quantity: Decimal,
        timestamp: i64,
    ) -> Self {
        Self {
            source,
            exchange: source.exchange().to_string(),
            symbol,
            asset_type,
            data_type,
            price,
            quantity,
            timestamp,
            received_at: chrono::Utc::now().timestamp_millis(),
            ..Default::default()
        }
    }

    /// Calculates the mid price from bid and ask if available
    pub fn mid_price(&self) -> Option<Decimal> {
        match (self.bid, self.ask) {
            (Some(bid), Some(ask)) => Some((bid + ask) / Decimal::from(2)),
            _ => None,
        }
    }

    /// Calculates the spread from bid and ask if available
    pub fn spread(&self) -> Option<Decimal> {
        match (self.bid, self.ask) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }

    /// Calculates the spread percentage
    pub fn spread_percentage(&self) -> Option<Decimal> {
        match (self.spread(), self.mid_price()) {
            (Some(spread), Some(mid)) if mid > Decimal::ZERO => {
                Some(spread / mid * Decimal::from(100))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_standard_market_data_default() {
        let data = StandardMarketData::default();
        assert_eq!(data.price, Decimal::ZERO);
        assert_eq!(data.quantity, Decimal::ZERO);
        assert!(data.bid.is_none());
        assert!(data.raw_data.is_empty());
    }

    #[test]
    fn test_standard_market_data_new() {
        let data = StandardMarketData::new(
            DataSourceType::BinanceSpot,
            "BTCUSDT".to_string(),
            AssetType::Spot,
            MarketDataType::Trade,
            dec!(50000.0),
            dec!(0.1),
            1234567890000,
        );

        assert_eq!(data.source, DataSourceType::BinanceSpot);
        assert_eq!(data.symbol, "BTCUSDT");
        assert_eq!(data.exchange, "Binance");
        assert_eq!(data.price, dec!(50000.0));
        assert_eq!(data.quantity, dec!(0.1));
    }

    #[test]
    fn test_serialization_deserialization() {
        let data = StandardMarketData {
            source: DataSourceType::BinanceSpot,
            exchange: "Binance".to_string(),
            symbol: "BTCUSDT".to_string(),
            asset_type: AssetType::Spot,
            data_type: MarketDataType::Trade,
            price: dec!(50000.12345678),
            quantity: dec!(0.001),
            timestamp: 1234567890000,
            received_at: 1234567890100,
            bid: Some(dec!(49999.99)),
            ask: Some(dec!(50000.01)),
            high_24h: Some(dec!(51000.0)),
            low_24h: Some(dec!(49000.0)),
            volume_24h: Some(dec!(1000.0)),
            open_interest: None,
            funding_rate: None,
            sequence_id: Some(12345),
            raw_data: r#"{"test":"data"}"#.to_string(),
        };

        let json = serde_json::to_string(&data).unwrap();
        let deserialized: StandardMarketData = serde_json::from_str(&json).unwrap();

        assert_eq!(data.price, deserialized.price);
        assert_eq!(data.symbol, deserialized.symbol);
        assert_eq!(data.bid, deserialized.bid);
        assert_eq!(data.sequence_id, deserialized.sequence_id);
    }

    #[test]
    fn test_mid_price_calculation() {
        let data = StandardMarketData {
            bid: Some(dec!(100.0)),
            ask: Some(dec!(102.0)),
            ..Default::default()
        };

        let mid = data.mid_price().unwrap();
        assert_eq!(mid, dec!(101.0));
    }

    #[test]
    fn test_spread_calculation() {
        let data = StandardMarketData {
            bid: Some(dec!(100.0)),
            ask: Some(dec!(102.0)),
            ..Default::default()
        };

        let spread = data.spread().unwrap();
        assert_eq!(spread, dec!(2.0));
    }

    #[test]
    fn test_spread_percentage() {
        let data = StandardMarketData {
            bid: Some(dec!(100.0)),
            ask: Some(dec!(102.0)),
            ..Default::default()
        };

        let spread_pct = data.spread_percentage().unwrap();
        assert!(spread_pct > dec!(1.98) && spread_pct < dec!(1.99));
    }

    #[test]
    fn test_decimal_precision() {
        let data = StandardMarketData {
            price: dec!(50000.12345678),
            ..Default::default()
        };

        // Verify no precision loss
        assert_eq!(data.price.to_string(), "50000.12345678");
    }
}
