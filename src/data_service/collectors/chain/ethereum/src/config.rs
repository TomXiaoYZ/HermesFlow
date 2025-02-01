use serde::{Deserialize, Serialize};
use url::Url;
use crate::error::{EthError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthConfig {
    /// Primary RPC endpoint (Alchemy)
    pub primary_url: String,
    /// Backup RPC endpoint (optional)
    pub backup_url: Option<String>,
    /// WebSocket endpoint
    pub ws_url: String,
    /// Chain ID
    pub chain_id: u64,
    /// Max reconnection attempts
    pub max_reconnect_attempts: u32,
    /// Reconnection interval in milliseconds
    pub reconnect_interval_ms: u64,
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
    /// Max requests per second
    pub max_requests_per_second: u32,
}

impl Default for EthConfig {
    fn default() -> Self {
        Self {
            primary_url: "https://eth-mainnet.g.alchemy.com/v2/your-api-key".to_string(),
            backup_url: None,
            ws_url: "wss://eth-mainnet.g.alchemy.com/v2/your-api-key".to_string(),
            chain_id: 1, // Ethereum mainnet
            max_reconnect_attempts: 5,
            reconnect_interval_ms: 1000,
            request_timeout_secs: 30,
            max_requests_per_second: 50,
        }
    }
}

impl EthConfig {
    pub fn validate(&self) -> Result<()> {
        // Validate primary URL
        Url::parse(&self.primary_url)
            .map_err(|e| EthError::ConfigError(format!("Invalid primary URL: {}", e)))?;

        // Validate backup URL if present
        if let Some(ref backup) = self.backup_url {
            Url::parse(backup)
                .map_err(|e| EthError::ConfigError(format!("Invalid backup URL: {}", e)))?;
        }

        // Validate WebSocket URL
        Url::parse(&self.ws_url)
            .map_err(|e| EthError::ConfigError(format!("Invalid WebSocket URL: {}", e)))?;

        // Validate chain ID
        if self.chain_id == 0 {
            return Err(EthError::ConfigError("Chain ID cannot be 0".to_string()));
        }

        // Validate timeouts and intervals
        if self.reconnect_interval_ms == 0 {
            return Err(EthError::ConfigError("Reconnect interval cannot be 0".to_string()));
        }

        if self.request_timeout_secs == 0 {
            return Err(EthError::ConfigError("Request timeout cannot be 0".to_string()));
        }

        Ok(())
    }

    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();
        
        let config = Self {
            primary_url: std::env::var("ETH_PRIMARY_URL")
                .map_err(|_| EthError::ConfigError("ETH_PRIMARY_URL not set".to_string()))?,
            backup_url: std::env::var("ETH_BACKUP_URL").ok(),
            ws_url: std::env::var("ETH_WS_URL")
                .map_err(|_| EthError::ConfigError("ETH_WS_URL not set".to_string()))?,
            chain_id: std::env::var("ETH_CHAIN_ID")
                .map_err(|_| EthError::ConfigError("ETH_CHAIN_ID not set".to_string()))?
                .parse()
                .map_err(|_| EthError::ConfigError("Invalid ETH_CHAIN_ID".to_string()))?,
            max_reconnect_attempts: std::env::var("ETH_MAX_RECONNECT_ATTEMPTS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            reconnect_interval_ms: std::env::var("ETH_RECONNECT_INTERVAL_MS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
            request_timeout_secs: std::env::var("ETH_REQUEST_TIMEOUT_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            max_requests_per_second: std::env::var("ETH_MAX_REQUESTS_PER_SECOND")
                .unwrap_or_else(|_| "50".to_string())
                .parse()
                .unwrap_or(50),
        };

        config.validate()?;
        Ok(config)
    }
} 