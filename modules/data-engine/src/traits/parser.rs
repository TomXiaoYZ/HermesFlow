use async_trait::async_trait;

use crate::error::Result;
use crate::models::{DataSourceType, StandardMarketData};

/// Trait for message parsers
///
/// This trait defines the interface for parsing raw messages from data sources
/// into standardized `StandardMarketData` structures. Each data source typically
/// has its own message format, and parsers handle the transformation into our
/// unified data model.
///
/// # Example
///
/// ```ignore
/// use async_trait::async_trait;
/// use data_engine::traits::MessageParser;
/// use data_engine::models::{DataSourceType, StandardMarketData};
///
/// struct MyParser;
///
/// #[async_trait]
/// impl MessageParser for MyParser {
///     fn source_type(&self) -> DataSourceType {
///         DataSourceType::BinanceSpot
///     }
///
///     async fn parse(&self, raw: &str) -> Result<Option<StandardMarketData>> {
///         // Parse raw message and return standardized data
///         // Return Ok(None) for messages that should be ignored (e.g., heartbeats)
///         Ok(None)
///     }
///
///     fn validate(&self, raw: &str) -> bool {
///         // Validate message format
///         !raw.is_empty()
///     }
/// }
/// ```
#[async_trait]
pub trait MessageParser: Send + Sync {
    /// Returns the data source type this parser handles
    ///
    /// Each parser is responsible for a specific data source type.
    fn source_type(&self) -> DataSourceType;

    /// Parses a raw message into standardized format
    ///
    /// This method takes a raw message string (typically JSON) and converts it
    /// into a `StandardMarketData` structure. If the message is not relevant
    /// (e.g., a heartbeat or subscription confirmation), it should return `Ok(None)`.
    ///
    /// # Arguments
    ///
    /// * `raw` - The raw message string to parse
    ///
    /// # Returns
    ///
    /// * `Ok(Some(StandardMarketData))` - Successfully parsed market data
    /// * `Ok(None)` - Message should be ignored (not an error)
    /// * `Err(_)` - Parsing error occurred
    ///
    /// # Errors
    ///
    /// Returns an error if the message cannot be parsed or contains invalid data.
    async fn parse(&self, raw: &str) -> Result<Option<StandardMarketData>>;

    /// Validates message format before parsing
    ///
    /// This method performs a quick validation check on the raw message format
    /// without performing the full parse operation. It's useful for filtering
    /// out obviously malformed messages early.
    ///
    /// # Arguments
    ///
    /// * `raw` - The raw message string to validate
    ///
    /// # Returns
    ///
    /// `true` if the message appears to be valid, `false` otherwise
    fn validate(&self, raw: &str) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AssetType, DataSourceType, MarketDataType, StandardMarketData};
    use rust_decimal_macros::dec;

    // Mock parser for testing
    struct MockParser;

    #[async_trait]
    impl MessageParser for MockParser {
        fn source_type(&self) -> DataSourceType {
            DataSourceType::BinanceSpot
        }

        async fn parse(&self, raw: &str) -> Result<Option<StandardMarketData>> {
            if raw == "heartbeat" {
                return Ok(None);
            }

            if raw.starts_with('{') && raw.ends_with('}') {
                Ok(Some(StandardMarketData::new(
                    DataSourceType::BinanceSpot,
                    "BTCUSDT".to_string(),
                    AssetType::Spot,
                    MarketDataType::Trade,
                    dec!(50000.0),
                    dec!(0.1),
                    chrono::Utc::now().timestamp_millis(),
                )))
            } else {
                Err(crate::error::DataError::ParseError {
                    data_source: "MockParser".to_string(),
                    message: "Invalid format".to_string(),
                    raw_data: raw.to_string(),
                })
            }
        }

        fn validate(&self, raw: &str) -> bool {
            !raw.is_empty()
        }
    }

    #[tokio::test]
    async fn test_mock_parser_source_type() {
        let parser = MockParser;
        assert_eq!(parser.source_type(), DataSourceType::BinanceSpot);
    }

    #[tokio::test]
    async fn test_mock_parser_validate() {
        let parser = MockParser;
        assert!(parser.validate("some message"));
        assert!(!parser.validate(""));
    }

    #[tokio::test]
    async fn test_mock_parser_parse_valid() {
        let parser = MockParser;
        let result = parser.parse(r#"{"test": "data"}"#).await;

        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(data.is_some());

        let data = data.unwrap();
        assert_eq!(data.symbol, "BTCUSDT");
        assert_eq!(data.source, DataSourceType::BinanceSpot);
    }

    #[tokio::test]
    async fn test_mock_parser_parse_heartbeat() {
        let parser = MockParser;
        let result = parser.parse("heartbeat").await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_mock_parser_parse_invalid() {
        let parser = MockParser;
        let result = parser.parse("invalid").await;

        assert!(result.is_err());
    }
}
