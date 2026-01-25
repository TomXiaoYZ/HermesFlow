use std::collections::HashMap;
use std::sync::Arc;

use crate::error::{DataError, Result};
use crate::models::{DataSourceType, StandardMarketData};
use crate::traits::MessageParser;

/// Registry for managing and routing message parsers
///
/// The `ParserRegistry` provides a centralized way to register and dispatch
/// message parsers based on data source types. It uses a thread-safe HashMap
/// to store parser instances and provides O(1) lookup performance.
///
/// # Example
///
/// ```ignore
/// use data_engine::registry::ParserRegistry;
/// use std::sync::Arc;
///
/// let mut registry = ParserRegistry::new();
/// let parser = Arc::new(MyParser::new());
/// registry.register(parser);
///
/// // Later, parse a message
/// let result = registry.parse(DataSourceType::BinanceSpot, raw_message).await?;
/// ```
pub struct ParserRegistry {
    parsers: HashMap<DataSourceType, Arc<dyn MessageParser>>,
}

impl ParserRegistry {
    /// Creates a new empty parser registry
    pub fn new() -> Self {
        Self {
            parsers: HashMap::new(),
        }
    }

    /// Registers a parser for its associated data source type
    ///
    /// If a parser for this source type already exists, it will be replaced.
    ///
    /// # Arguments
    ///
    /// * `parser` - An Arc-wrapped parser implementing the MessageParser trait
    pub fn register(&mut self, parser: Arc<dyn MessageParser>) {
        let source_type = parser.source_type();
        tracing::info!("Registering parser for source: {:?}", source_type);
        self.parsers.insert(source_type, parser);
    }

    /// Parses a raw message using the appropriate parser
    ///
    /// This method looks up the parser for the given source type and uses it
    /// to parse the raw message string.
    ///
    /// # Arguments
    ///
    /// * `source` - The data source type
    /// * `raw` - The raw message string to parse
    ///
    /// # Returns
    ///
    /// * `Ok(Some(StandardMarketData))` - Successfully parsed market data
    /// * `Ok(None)` - Message should be ignored (e.g., heartbeat)
    /// * `Err(DataError::ParserNotFound)` - No parser registered for this source
    /// * `Err(_)` - Parser encountered an error
    pub async fn parse(
        &self,
        source: DataSourceType,
        raw: &str,
    ) -> Result<Option<StandardMarketData>> {
        let parser = self.parsers.get(&source).ok_or_else(|| {
            DataError::ParserNotFound(format!("No parser registered for {:?}", source))
        })?;

        parser.parse(raw).await
    }

    /// Checks if a parser is registered for the given source type
    pub fn has_parser(&self, source: &DataSourceType) -> bool {
        self.parsers.contains_key(source)
    }

    /// Returns the number of registered parsers
    pub fn len(&self) -> usize {
        self.parsers.len()
    }

    /// Checks if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.parsers.is_empty()
    }

    /// Removes a parser for the given source type
    ///
    /// Returns the removed parser if it existed
    pub fn remove(&mut self, source: &DataSourceType) -> Option<Arc<dyn MessageParser>> {
        self.parsers.remove(source)
    }

    /// Clears all registered parsers
    pub fn clear(&mut self) {
        self.parsers.clear();
    }

    /// Returns a list of all registered data source types
    pub fn registered_sources(&self) -> Vec<DataSourceType> {
        self.parsers.keys().cloned().collect()
    }
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AssetType, MarketDataType};
    use async_trait::async_trait;
    use rust_decimal_macros::dec;

    // Mock parser for testing
    struct MockBinanceParser;

    #[async_trait]
    impl MessageParser for MockBinanceParser {
        fn source_type(&self) -> DataSourceType {
            DataSourceType::BinanceSpot
        }

        async fn parse(&self, raw: &str) -> Result<Option<StandardMarketData>> {
            if raw == "heartbeat" {
                return Ok(None);
            }

            Ok(Some(StandardMarketData::new(
                DataSourceType::BinanceSpot,
                "BTCUSDT".to_string(),
                AssetType::Spot,
                MarketDataType::Trade,
                dec!(50000.0),
                dec!(0.1),
                chrono::Utc::now().timestamp_millis(),
            )))
        }

        fn validate(&self, raw: &str) -> bool {
            !raw.is_empty()
        }
    }

    struct MockOkxParser;

    #[async_trait]
    impl MessageParser for MockOkxParser {
        fn source_type(&self) -> DataSourceType {
            DataSourceType::OkxSpot
        }

        async fn parse(&self, _raw: &str) -> Result<Option<StandardMarketData>> {
            Ok(Some(StandardMarketData::new(
                DataSourceType::OkxSpot,
                "BTC-USDT".to_string(),
                AssetType::Spot,
                MarketDataType::Trade,
                dec!(50000.0),
                dec!(0.1),
                chrono::Utc::now().timestamp_millis(),
            )))
        }

        fn validate(&self, raw: &str) -> bool {
            !raw.is_empty()
        }
    }

    #[test]
    fn test_registry_new() {
        let registry = ParserRegistry::new();
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());
    }

    #[test]
    fn test_registry_register() {
        let mut registry = ParserRegistry::new();
        let parser = Arc::new(MockBinanceParser);

        registry.register(parser);
        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
        assert!(registry.has_parser(&DataSourceType::BinanceSpot));
    }

    #[test]
    fn test_registry_register_multiple() {
        let mut registry = ParserRegistry::new();
        registry.register(Arc::new(MockBinanceParser));
        registry.register(Arc::new(MockOkxParser));

        assert_eq!(registry.len(), 2);
        assert!(registry.has_parser(&DataSourceType::BinanceSpot));
        assert!(registry.has_parser(&DataSourceType::OkxSpot));
    }

    #[tokio::test]
    async fn test_registry_parse_success() {
        let mut registry = ParserRegistry::new();
        registry.register(Arc::new(MockBinanceParser));

        let result = registry
            .parse(DataSourceType::BinanceSpot, r#"{"test": "data"}"#)
            .await;

        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(data.is_some());

        let data = data.unwrap();
        assert_eq!(data.source, DataSourceType::BinanceSpot);
        assert_eq!(data.symbol, "BTCUSDT");
    }

    #[tokio::test]
    async fn test_registry_parse_heartbeat() {
        let mut registry = ParserRegistry::new();
        registry.register(Arc::new(MockBinanceParser));

        let result = registry
            .parse(DataSourceType::BinanceSpot, "heartbeat")
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_registry_parse_parser_not_found() {
        let registry = ParserRegistry::new();

        let result = registry
            .parse(DataSourceType::BinanceSpot, "some data")
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            DataError::ParserNotFound(msg) => {
                assert!(msg.contains("BinanceSpot"));
            }
            _ => panic!("Expected ParserNotFound error"),
        }
    }

    #[test]
    fn test_registry_remove() {
        let mut registry = ParserRegistry::new();
        registry.register(Arc::new(MockBinanceParser));

        assert!(registry.has_parser(&DataSourceType::BinanceSpot));

        let removed = registry.remove(&DataSourceType::BinanceSpot);
        assert!(removed.is_some());
        assert!(!registry.has_parser(&DataSourceType::BinanceSpot));
        assert!(registry.is_empty());
    }

    #[test]
    fn test_registry_clear() {
        let mut registry = ParserRegistry::new();
        registry.register(Arc::new(MockBinanceParser));
        registry.register(Arc::new(MockOkxParser));

        assert_eq!(registry.len(), 2);

        registry.clear();
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());
    }

    #[test]
    fn test_registry_registered_sources() {
        let mut registry = ParserRegistry::new();
        registry.register(Arc::new(MockBinanceParser));
        registry.register(Arc::new(MockOkxParser));

        let sources = registry.registered_sources();
        assert_eq!(sources.len(), 2);
        assert!(sources.contains(&DataSourceType::BinanceSpot));
        assert!(sources.contains(&DataSourceType::OkxSpot));
    }

    #[test]
    fn test_registry_replace_parser() {
        let mut registry = ParserRegistry::new();
        registry.register(Arc::new(MockBinanceParser));
        assert_eq!(registry.len(), 1);

        // Register another parser for the same source - should replace
        registry.register(Arc::new(MockBinanceParser));
        assert_eq!(registry.len(), 1);
    }
}
