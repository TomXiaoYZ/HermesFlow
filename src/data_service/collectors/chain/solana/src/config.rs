use serde::{Deserialize, Serialize};
use url::Url;
use crate::error::{SolError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolConfig {
    /// Primary RPC endpoint
    pub primary_url: String,
    /// Backup RPC endpoint (optional)
    pub backup_url: Option<String>,
    /// WebSocket endpoint
    pub ws_url: String,
    /// Max reconnection attempts
    pub max_reconnect_attempts: u32,
    /// Reconnection interval in milliseconds
    pub reconnect_interval_ms: u64,
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
    /// Max requests per second
    pub max_requests_per_second: u32,
    /// Commitment level
    pub commitment: String,
}

impl Default for SolConfig {
    fn default() -> Self {
        Self {
            primary_url: "https://api.mainnet-beta.solana.com".to_string(),
            backup_url: Some("https://solana-api.projectserum.com".to_string()),
            ws_url: "wss://api.mainnet-beta.solana.com".to_string(),
            max_reconnect_attempts: 5,
            reconnect_interval_ms: 1000,
            request_timeout_secs: 30,
            max_requests_per_second: 40,
            commitment: "confirmed".to_string(),
        }
    }
}

impl SolConfig {
    pub fn validate(&self) -> Result<()> {
        // Validate primary URL
        Url::parse(&self.primary_url)
            .map_err(|e| SolError::ConfigError(format!("Invalid primary URL: {}", e)))?;

        // Validate backup URL if present
        if let Some(ref backup) = self.backup_url {
            Url::parse(backup)
                .map_err(|e| SolError::ConfigError(format!("Invalid backup URL: {}", e)))?;
        }

        // Validate WebSocket URL
        Url::parse(&self.ws_url)
            .map_err(|e| SolError::ConfigError(format!("Invalid WebSocket URL: {}", e)))?;

        // Validate timeouts and intervals
        if self.reconnect_interval_ms == 0 {
            return Err(SolError::ConfigError("Reconnect interval cannot be 0".to_string()));
        }

        if self.request_timeout_secs == 0 {
            return Err(SolError::ConfigError("Request timeout cannot be 0".to_string()));
        }

        // Validate commitment level
        match self.commitment.as_str() {
            "processed" | "confirmed" | "finalized" => Ok(()),
            _ => Err(SolError::ConfigError("Invalid commitment level".to_string())),
        }
    }

    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();
        
        let config = Self {
            primary_url: std::env::var("SOL_PRIMARY_URL")
                .map_err(|_| SolError::ConfigError("SOL_PRIMARY_URL not set".to_string()))?,
            backup_url: std::env::var("SOL_BACKUP_URL").ok(),
            ws_url: std::env::var("SOL_WS_URL")
                .map_err(|_| SolError::ConfigError("SOL_WS_URL not set".to_string()))?,
            max_reconnect_attempts: std::env::var("SOL_MAX_RECONNECT_ATTEMPTS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            reconnect_interval_ms: std::env::var("SOL_RECONNECT_INTERVAL_MS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
            request_timeout_secs: std::env::var("SOL_REQUEST_TIMEOUT_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            max_requests_per_second: std::env::var("SOL_MAX_REQUESTS_PER_SECOND")
                .unwrap_or_else(|_| "40".to_string())
                .parse()
                .unwrap_or(40),
            commitment: std::env::var("SOL_COMMITMENT")
                .unwrap_or_else(|_| "confirmed".to_string()),
        };

        config.validate()?;
        Ok(config)
    }
} 