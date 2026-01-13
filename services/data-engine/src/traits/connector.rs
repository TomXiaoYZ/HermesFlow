use async_trait::async_trait;
use std::time::SystemTime;
use tokio::sync::mpsc;

use crate::error::Result;
use crate::models::{AssetType, DataSourceType, StandardMarketData};

/// Statistics for a data source connector
#[derive(Debug, Clone, Default)]
pub struct ConnectorStats {
    /// Total messages received from the data source
    pub messages_received: u64,
    /// Total messages successfully processed
    pub messages_processed: u64,
    /// Total errors encountered
    pub errors: u64,
    /// Uptime in seconds since connection established
    pub uptime_secs: u64,
    /// Timestamp of the last message received
    pub last_message_at: Option<SystemTime>,
}

/// Trait for data source connectors
///
/// This trait defines the interface that all data source connectors must implement.
/// It provides a standardized way to connect to different data sources (exchanges, APIs, etc.)
/// and receive market data through a unified channel-based interface.
///
/// # Example
///
/// ```ignore
/// use async_trait::async_trait;
/// use data_engine::traits::DataSourceConnector;
/// use data_engine::models::{DataSourceType, AssetType};
///
/// struct MyConnector;
///
/// #[async_trait]
/// impl DataSourceConnector for MyConnector {
///     fn source_type(&self) -> DataSourceType {
///         DataSourceType::BinanceSpot
///     }
///
///     fn supported_assets(&self) -> Vec<AssetType> {
///         vec![AssetType::Spot]
///     }
///
///     async fn connect(&mut self) -> Result<mpsc::Receiver<StandardMarketData>> {
///         // Implementation here
///     }
///
///     async fn disconnect(&mut self) -> Result<()> {
///         // Implementation here
///     }
///
///     async fn is_healthy(&self) -> bool {
///         true
///     }
///
///     fn stats(&self) -> ConnectorStats {
///         ConnectorStats::default()
///     }
/// }
/// ```
#[async_trait]
pub trait DataSourceConnector: Send + Sync {
    /// Returns the data source type for this connector
    ///
    /// This method identifies which specific data source this connector handles,
    /// such as BinanceSpot, OkxFutures, etc.
    fn source_type(&self) -> DataSourceType;

    /// Returns the asset types supported by this connector
    ///
    /// Different connectors may support different asset types. For example,
    /// a spot exchange connector would return `vec![AssetType::Spot]`, while
    /// a derivatives exchange might return `vec![AssetType::Perpetual, AssetType::Future]`.
    fn supported_assets(&self) -> Vec<AssetType>;

    /// Connects to the data source and starts streaming data
    ///
    /// This method establishes a connection to the data source and returns a receiver
    /// channel that will stream `StandardMarketData` messages. The implementation
    /// should handle the connection lifecycle, including authentication if required.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `mpsc::Receiver` that will receive market data messages.
    ///
    /// # Errors
    ///
    /// This method will return an error if the connection fails to establish.
    async fn connect(&mut self) -> Result<mpsc::Receiver<StandardMarketData>>;

    /// Gracefully disconnects from the data source
    ///
    /// This method should cleanly shut down the connection, ensuring that any
    /// pending messages are processed and resources are properly released.
    ///
    /// # Errors
    ///
    /// Returns an error if the disconnection encounters issues.
    async fn disconnect(&mut self) -> Result<()>;

    /// Health check - returns true if the connection is healthy
    ///
    /// This method should perform a quick check to determine if the connection
    /// is still alive and functioning properly. It should not block for long periods.
    async fn is_healthy(&self) -> bool;

    /// Returns connection statistics
    ///
    /// This method provides metrics about the connector's operation, including
    /// message counts, errors, and uptime information.
    fn stats(&self) -> ConnectorStats;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AssetType, DataSourceType, MarketDataType, StandardMarketData};
    use rust_decimal_macros::dec;

    // Mock connector for testing
    struct MockConnector {
        healthy: bool,
        stats: ConnectorStats,
    }

    #[async_trait]
    impl DataSourceConnector for MockConnector {
        fn source_type(&self) -> DataSourceType {
            DataSourceType::BinanceSpot
        }

        fn supported_assets(&self) -> Vec<AssetType> {
            vec![AssetType::Spot, AssetType::Perpetual]
        }

        async fn connect(&mut self) -> Result<mpsc::Receiver<StandardMarketData>> {
            let (tx, rx) = mpsc::channel(100);

            // Spawn a task to send mock data
            tokio::spawn(async move {
                let data = StandardMarketData::new(
                    DataSourceType::BinanceSpot,
                    "BTCUSDT".to_string(),
                    AssetType::Spot,
                    MarketDataType::Trade,
                    dec!(50000.0),
                    dec!(0.1),
                    chrono::Utc::now().timestamp_millis(),
                );
                let _ = tx.send(data).await;
            });

            Ok(rx)
        }

        async fn disconnect(&mut self) -> Result<()> {
            self.healthy = false;
            Ok(())
        }

        async fn is_healthy(&self) -> bool {
            self.healthy
        }

        fn stats(&self) -> ConnectorStats {
            self.stats.clone()
        }
    }

    #[tokio::test]
    async fn test_mock_connector_source_type() {
        let connector = MockConnector {
            healthy: true,
            stats: ConnectorStats::default(),
        };
        assert_eq!(connector.source_type(), DataSourceType::BinanceSpot);
    }

    #[tokio::test]
    async fn test_mock_connector_supported_assets() {
        let connector = MockConnector {
            healthy: true,
            stats: ConnectorStats::default(),
        };
        let assets = connector.supported_assets();
        assert_eq!(assets.len(), 2);
        assert!(assets.contains(&AssetType::Spot));
        assert!(assets.contains(&AssetType::Perpetual));
    }

    #[tokio::test]
    async fn test_mock_connector_connect() {
        let mut connector = MockConnector {
            healthy: true,
            stats: ConnectorStats::default(),
        };

        let mut rx = connector.connect().await.unwrap();
        let data = rx.recv().await;

        assert!(data.is_some());
        let data = data.unwrap();
        assert_eq!(data.symbol, "BTCUSDT");
        assert_eq!(data.source, DataSourceType::BinanceSpot);
    }

    #[tokio::test]
    async fn test_mock_connector_health_check() {
        let mut connector = MockConnector {
            healthy: true,
            stats: ConnectorStats::default(),
        };

        assert!(connector.is_healthy().await);

        connector.disconnect().await.unwrap();
        assert!(!connector.is_healthy().await);
    }

    #[test]
    fn test_connector_stats_default() {
        let stats = ConnectorStats::default();
        assert_eq!(stats.messages_received, 0);
        assert_eq!(stats.messages_processed, 0);
        assert_eq!(stats.errors, 0);
        assert!(stats.last_message_at.is_none());
    }
}
