pub mod config;
pub mod error;
pub mod models;
pub mod websocket;

pub use config::SolConfig;
pub use error::{SolError, Result};
pub use models::{BlockInfo, ChainEvent, ChainStats, TransactionInfo, AccountInfo, ProgramInfo};
pub use websocket::WebsocketClient;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::broadcast;

    #[tokio::test]
    async fn test_websocket_connection() {
        let (tx, _) = broadcast::channel(100);
        let config = Arc::new(SolConfig::default());
        
        let client = WebsocketClient::new(config, tx);
        assert!(client.is_ok(), "Failed to create WebSocket client");
    }

    #[tokio::test]
    async fn test_config_validation() {
        let config = SolConfig::default();
        assert!(config.validate().is_ok(), "Default config should be valid");
    }
} 