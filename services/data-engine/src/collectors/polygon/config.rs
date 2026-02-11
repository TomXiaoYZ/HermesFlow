use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolygonConfig {
    /// Polygon.io API key
    pub api_key: String,

    /// REST API base URL
    #[serde(default = "default_rest_url")]
    pub rest_base_url: String,

    /// WebSocket URL for real-time data
    #[serde(default = "default_ws_url")]
    pub ws_url: String,

    /// Rate limit (requests per second)
    #[serde(default = "default_rate_limit")]
    pub rate_limit_per_sec: u32,

    /// Maximum retry attempts for failed requests
    #[serde(default = "default_max_retries")]
    pub retry_max_attempts: u32,

    /// Retry delay in milliseconds
    #[serde(default = "default_retry_delay")]
    pub retry_delay_ms: u64,

    /// Enable WebSocket real-time connection
    #[serde(default = "default_ws_enabled")]
    pub ws_enabled: bool,
}

fn default_rest_url() -> String {
    "https://api.polygon.io".to_string()
}

fn default_ws_url() -> String {
    "wss://socket.polygon.io/stocks".to_string()
}

fn default_rate_limit() -> u32 {
    5 // Conservative default (5 req/sec)
}

fn default_max_retries() -> u32 {
    3
}

fn default_retry_delay() -> u64 {
    1000 // 1 second
}

fn default_ws_enabled() -> bool {
    true
}

impl PolygonConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let api_key = std::env::var("POLYGON_API_KEY")
            .map_err(|_| "POLYGON_API_KEY environment variable not set")?;

        Ok(Self {
            api_key,
            rest_base_url: std::env::var("POLYGON_REST_URL").unwrap_or_else(|_| default_rest_url()),
            ws_url: std::env::var("POLYGON_WS_URL").unwrap_or_else(|_| default_ws_url()),
            rate_limit_per_sec: std::env::var("POLYGON_RATE_LIMIT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or_else(default_rate_limit),
            retry_max_attempts: default_max_retries(),
            retry_delay_ms: default_retry_delay(),
            ws_enabled: std::env::var("POLYGON_WS_ENABLED")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
        })
    }
}
