pub mod config;
pub mod error;
pub mod models;
pub mod websocket;

pub use config::EthConfig;
pub use error::{EthError, Result};
pub use models::{BlockInfo, ChainEvent, ChainStats, TransactionInfo};
pub use websocket::WebsocketClient;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::broadcast;

    #[tokio::test]
    async fn test_websocket_connection() {
        let (tx, _) = broadcast::channel(100);
        let config = Arc::new(EthConfig::default());
        
        let client = WebsocketClient::new(config, tx).await;
        assert!(client.is_ok(), "Failed to create WebSocket client");
    }

    #[tokio::test]
    async fn test_config_validation() {
        let config = EthConfig::default();
        assert!(config.validate().is_ok(), "Default config should be valid");
    }
} 