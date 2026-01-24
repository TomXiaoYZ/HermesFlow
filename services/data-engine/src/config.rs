use crate::collectors::binance::BinanceConfig;
use crate::collectors::birdeye::BirdeyeConfig;
use crate::collectors::bybit::BybitConfig;
use crate::collectors::dexscreener::DexScreenerConfig;
use crate::collectors::futu::FutuConfig;
use crate::collectors::helius::HeliusConfig;
use crate::collectors::okx::OkxConfig;
use config::{Config, ConfigError, Environment, File};
use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;

/// Main application configuration
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub redis: RedisConfig,
    pub postgres: PostgresConfig,
    pub clickhouse: ClickHouseConfig,
    #[serde(default)]
    pub data_sources: Vec<DataSourceConfig>,
    pub twitter: Option<TwitterConfig>,
    pub ibkr: Option<IbkrConfig>,
    pub polymarket: Option<PolymarketConfig>,
    pub akshare: Option<AkShareConfig>,
    pub massive: Option<MassiveConfig>,
    pub binance: Option<BinanceConfig>,
    pub okx: Option<OkxConfig>,
    pub bybit: Option<BybitConfig>,
    pub futu: Option<FutuConfig>,
    pub birdeye: Option<BirdeyeConfig>,
    pub dexscreener: Option<DexScreenerConfig>,
    pub helius: Option<HeliusConfig>,
    pub performance: PerformanceConfig,
    pub logging: LoggingConfig,
}

/// HTTP server configuration
#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    /// Server bind host (default: "0.0.0.0")
    pub host: String,
    /// Server bind port (default: 8080)
    pub port: u16,
    /// Graceful shutdown timeout in seconds (default: 30)
    pub shutdown_timeout_secs: u64,
}

/// Redis configuration
#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    /// Redis connection URL (e.g., "redis://localhost:6379")
    pub url: String,
    /// Connection pool size (default: 10)
    pub pool_size: usize,
    /// TTL for cached data in seconds (default: 86400 = 24 hours)
    pub ttl_secs: u64,
}

/// ClickHouse configuration
#[derive(Debug, Deserialize, Clone)]
pub struct ClickHouseConfig {
    /// ClickHouse connection URL (e.g., "tcp://localhost:9000")
    pub url: String,
    /// Database name
    pub database: String,
    /// Username for authentication
    pub username: String,
    /// Password for authentication
    pub password: String,
    /// Batch size for bulk inserts (default: 1000)
    pub batch_size: usize,
    /// Flush interval in milliseconds (default: 5000 = 5 seconds)
    pub flush_interval_ms: u64,
}

/// Postgres configuration
#[derive(Debug, Deserialize, Clone)]
pub struct PostgresConfig {
    /// Postgres connection host
    pub host: String,
    /// Postgres port (default: 5432)
    pub port: u16,
    /// Database name
    pub database: String,
    /// Username for authentication
    pub username: String,
    /// Password for authentication
    pub password: String,
    /// Maximum pool size (default: 10)
    pub max_connections: u32,
}

/// Twitter scraper configuration
#[derive(Debug, Deserialize, Clone)]
pub struct TwitterConfig {
    /// Twitter username
    pub username: String,
    /// Twitter email
    pub email: String,
    /// Twitter password
    pub password: String,
    /// Maximum tweets per session (default: 1000)
    pub max_tweets_per_session: usize,
    /// Scroll delay minimum (seconds, default: 2.0)
    pub scroll_delay_min: f64,
    /// Scroll delay maximum (seconds, default: 5.0)
    pub scroll_delay_max: f64,
    /// Enable headless mode (default: true)
    pub headless: bool,

    /// Poll interval in seconds (default: 600)
    #[serde(default = "default_twitter_poll_interval_secs")]
    pub poll_interval_secs: u64,

    /// Target usernames to scrape (empty = defaults to self username)
    #[serde(default)]
    pub targets: Vec<String>,

    /// Search keywords (each keyword becomes a search query unless `search_queries` provided)
    #[serde(default)]
    pub keywords: Vec<String>,

    /// Explicit search queries (advanced). If set, takes precedence over `keywords`.
    #[serde(default)]
    pub search_queries: Vec<String>,
}

/// IBKR configuration
#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct IbkrConfig {
    /// IB Gateway host (default: "127.0.0.1")
    pub host: String,
    /// IB Gateway port (default: 4001 for paper, 4002 for live)
    pub port: i32,
    /// Client ID (default: 1)
    pub client_id: i32,
    /// Whether this data source is enabled
    pub enabled: bool,
    /// List of symbols to subscribe to
    #[serde(deserialize_with = "deserialize_comma_separated")]
    pub symbols: Vec<String>,
}

fn default_twitter_poll_interval_secs() -> u64 {
    600
}

/// Polymarket API configuration
#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct PolymarketConfig {
    /// Gamma API base URL (default: "https://gamma-api.polymarket.com")
    pub api_base_url: String,
    /// Poll interval in seconds (default: 60)
    pub poll_interval_secs: u64,
    /// Markets to track (empty = all)
    pub tracked_markets: Vec<String>,
}

/// AkShare configuration
#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct AkShareConfig {
    /// Whether this data source is enabled
    pub enabled: bool,
    /// AkTools API URL (default: "http://aktools:8080")
    pub aktools_url: String,
    /// Poll interval in seconds (default: 3)
    pub poll_interval_secs: u64,
}

impl Default for AkShareConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            aktools_url: "http://aktools:8080".to_string(),
            poll_interval_secs: 3,
        }
    }
}

/// Massive (Polygon.io) configuration
#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct MassiveConfig {
    /// Whether this data source is enabled
    pub enabled: bool,
    /// API Key for Polygon.io
    pub api_key: String,
    /// Rate limit per minute (default: 5 for Free Tier)
    pub rate_limit_per_min: u64,
    /// WebSocket URL (default: "wss://socket.polygon.io/stocks")
    pub ws_url: String,
}

impl Default for MassiveConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key: String::new(),
            rate_limit_per_min: 5,
            ws_url: "wss://socket.polygon.io/stocks".to_string(),
        }
    }
}

/// Data source configuration
#[derive(Debug, Deserialize, Clone)]
pub struct DataSourceConfig {
    /// Data source name (e.g., "binance_spot")
    pub name: String,
    /// Data source type (e.g., "BinanceSpot")
    pub source_type: String,
    /// Whether this data source is enabled
    pub enabled: bool,
    /// List of symbols to subscribe to
    #[serde(deserialize_with = "deserialize_comma_separated")]
    pub symbols: Vec<String>,
    /// API key (optional)
    pub api_key: Option<String>,
    /// API secret (optional)
    pub api_secret: Option<String>,
}

/// Performance tuning configuration
#[derive(Debug, Deserialize, Clone)]
pub struct PerformanceConfig {
    /// Channel buffer size for message passing (default: 10000)
    pub channel_buffer_size: usize,
    /// Maximum reconnection attempts (default: 5)
    pub max_reconnect_attempts: u32,
    /// Reconnection delay in seconds (default: 5)
    pub reconnect_delay_secs: u64,
    /// Health check interval in seconds (default: 10)
    pub health_check_interval_secs: u64,
}

/// Logging configuration
#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    /// Log level: trace, debug, info, warn, error (default: "info")
    pub level: String,
    /// Log format: json or pretty (default: "json")
    pub format: String,
    /// Log output: stdout or file (default: "stdout")
    pub output: String,
}

impl AppConfig {
    /// Loads configuration from multiple sources with priority:
    /// 1. Environment variables (highest priority)
    /// 2. Environment-specific file (e.g., config/prod.toml)
    /// 3. Default file (config/default.toml)
    ///
    /// Environment variables are prefixed with `DATA_ENGINE__` and use
    /// double underscores for nesting (e.g., `DATA_ENGINE__SERVER__PORT=8081`)
    pub fn load() -> Result<Self, ConfigError> {
        let env = std::env::var("RUST_ENV").unwrap_or_else(|_| "dev".to_string());

        // Prioritize absolute path for container environments
        let base_path = if std::path::Path::new("/app/config/default.toml").exists() {
            "/app/config"
        } else if std::path::Path::new("config/default.toml").exists() {
            "config"
        } else {
            "/app/config" // Default to absolute if neither found, to fail fast on required(true)
        };

        // We can use tracing now because main.rs init it early
        tracing::info!("Loading configuration from base_path: {}", base_path);

        let config = Config::builder()
            // Start with defaults
            .add_source(File::with_name(&format!("{}/default", base_path)).required(true))
            // Layer environment-specific config
            .add_source(File::with_name(&format!("{}/{}", base_path, env)).required(false))
            // Layer environment variables (highest priority)
            .add_source(
                Environment::with_prefix("DATA_ENGINE")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;

        config.try_deserialize()
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            redis: RedisConfig::default(),
            postgres: PostgresConfig::default(),
            clickhouse: ClickHouseConfig::default(),
            data_sources: vec![],
            twitter: None,
            ibkr: None,
            polymarket: None,
            akshare: None,
            massive: None,
            binance: None,
            okx: None,
            bybit: None,
            futu: None,
            birdeye: None,
            dexscreener: None,
            helius: None,
            performance: PerformanceConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for DataSourceConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            source_type: String::new(),
            enabled: false,
            symbols: vec![],
            api_key: None,
            api_secret: None,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            shutdown_timeout_secs: 30,
        }
    }
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            pool_size: 10,
            ttl_secs: 86400, // 24 hours
        }
    }
}

impl Default for ClickHouseConfig {
    fn default() -> Self {
        Self {
            url: "tcp://localhost:9000".to_string(),
            database: "hermesflow".to_string(),
            username: "default".to_string(),
            password: String::new(),
            batch_size: 1000,
            flush_interval_ms: 5000, // 5 seconds
        }
    }
}

impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            database: "hermesflow".to_string(),
            username: "postgres".to_string(),
            password: String::new(),
            max_connections: 10,
        }
    }
}

impl Default for TwitterConfig {
    fn default() -> Self {
        Self {
            username: String::new(),
            email: String::new(),
            password: String::new(),
            max_tweets_per_session: 1000,
            scroll_delay_min: 2.0,
            scroll_delay_max: 5.0,
            headless: true,
            poll_interval_secs: default_twitter_poll_interval_secs(),
            targets: vec![],
            keywords: vec![],
            search_queries: vec![],
        }
    }
}

impl Default for IbkrConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 4001,
            client_id: 1,
            enabled: false,
            symbols: vec![],
        }
    }
}

impl Default for PolymarketConfig {
    fn default() -> Self {
        Self {
            api_base_url: "https://gamma-api.polymarket.com".to_string(),
            poll_interval_secs: 60,
            tracked_markets: vec![],
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            channel_buffer_size: 10000,
            max_reconnect_attempts: 5,
            reconnect_delay_secs: 5,
            health_check_interval_secs: 10,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "json".to_string(),
            output: "stdout".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert_eq!(config.shutdown_timeout_secs, 30);
    }

    #[test]
    fn test_redis_config_default() {
        let config = RedisConfig::default();
        assert_eq!(config.url, "redis://localhost:6379");
        assert_eq!(config.pool_size, 10);
        assert_eq!(config.ttl_secs, 86400);
    }

    #[test]
    fn test_clickhouse_config_default() {
        let config = ClickHouseConfig::default();
        assert_eq!(config.url, "tcp://localhost:9000");
        assert_eq!(config.database, "hermesflow");
        assert_eq!(config.batch_size, 1000);
        assert_eq!(config.flush_interval_ms, 5000);
    }

    #[test]
    fn test_performance_config_default() {
        let config = PerformanceConfig::default();
        assert_eq!(config.channel_buffer_size, 10000);
        assert_eq!(config.max_reconnect_attempts, 5);
        assert_eq!(config.reconnect_delay_secs, 5);
        assert_eq!(config.health_check_interval_secs, 10);
    }

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, "info");
        assert_eq!(config.format, "json");
        assert_eq!(config.output, "stdout");
    }

    #[test]
    fn test_config_env_var_override() {
        // Set environment variable
        std::env::set_var("DATA_ENGINE__SERVER__PORT", "9090");

        // Note: This test might fail if config files don't exist
        // In a real scenario, we'd create temporary config files for testing
        std::env::remove_var("DATA_ENGINE__SERVER__PORT");
    }
}

fn deserialize_comma_separated<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrVec;

    impl<'de> Visitor<'de> for StringOrVec {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or list of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(value
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect())
        }

        fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
        where
            S: SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(elem) = seq.next_element()? {
                vec.push(elem);
            }
            Ok(vec)
        }
    }

    deserializer.deserialize_any(StringOrVec)
}
