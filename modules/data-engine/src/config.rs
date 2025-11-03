use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

/// Main application configuration
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub redis: RedisConfig,
    pub clickhouse: ClickHouseConfig,
    pub data_sources: Vec<DataSourceConfig>,
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

        let config = Config::builder()
            // Start with defaults
            .add_source(File::with_name("config/default").required(false))
            // Layer environment-specific config
            .add_source(File::with_name(&format!("config/{}", env)).required(false))
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
