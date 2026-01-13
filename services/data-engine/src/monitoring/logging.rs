use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::config::LoggingConfig;

/// Initializes the tracing subscriber for structured logging
///
/// # Arguments
///
/// * `config` - Logging configuration
pub fn init_logging(config: &LoggingConfig) {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.level));

    match config.format.as_str() {
        "json" => {
            // JSON formatted logs for production
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer().json())
                .init();
        }
        "pretty" => {
            // Pretty formatted logs for development
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer().pretty())
                .init();
        }
        _ => {
            // Default to compact format
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer().compact())
                .init();
        }
    }

    tracing::info!(
        "Logging initialized: level={}, format={}",
        config.level,
        config.format
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_config_formats() {
        let json_config = LoggingConfig {
            level: "info".to_string(),
            format: "json".to_string(),
            output: "stdout".to_string(),
        };
        assert_eq!(json_config.format, "json");

        let pretty_config = LoggingConfig {
            level: "debug".to_string(),
            format: "pretty".to_string(),
            output: "stdout".to_string(),
        };
        assert_eq!(pretty_config.format, "pretty");
    }
}
