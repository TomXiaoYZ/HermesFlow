use std::time::Duration;
use thiserror::Error;

/// Custom error types for the data engine
#[derive(Error, Debug)]
pub enum DataError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Exchange API error: {0}")]
    ExchangeError(String),

    /// Connection failed to establish
    #[error("Connection failed for {data_source}: {reason}")]
    ConnectionFailed { data_source: String, reason: String },

    /// Error parsing message
    #[error("Parse error for {data_source}: {message}")]
    ParseError {
        data_source: String,
        message: String,
        raw_data: String,
    },

    /// Redis operation error
    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),

    /// ClickHouse operation error
    #[error("ClickHouse error: {0}")]
    ClickHouseError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(#[from] config::ConfigError),

    /// WebSocket error
    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    /// Parser not found for source
    #[error("Parser not found for source: {0}")]
    ParserNotFound(String),

    /// Invalid data
    #[error("Invalid data: {0}")]
    ValidationError(String),

    /// Timeout error
    #[error("Timeout after {timeout_secs}s: {operation}")]
    TimeoutError {
        operation: String,
        timeout_secs: u64,
    },

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Decimal parsing error
    #[error("Decimal parse error: {0}")]
    DecimalError(#[from] rust_decimal::Error),

    /// Generic error
    #[error("Error: {0}")]
    Other(String),

    /// IBKR error
    #[error("IBKR error: {0}")]
    IbkrError(#[from] ibapi::Error),
}

/// Extended error types for collectors
#[derive(Error, Debug)]
pub enum DataEngineError {
    /// Database operation error
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// Network/HTTP error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Browser automation error
    #[error("Browser error: {0}")]
    BrowserError(String),

    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Generic error
    #[error("Error: {0}")]
    Other(String),
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, DataError>;

/// Retry an async operation with exponential backoff
///
/// # Arguments
///
/// * `operation` - A closure that returns a future
/// * `max_attempts` - Maximum number of retry attempts
/// * `initial_delay_ms` - Initial delay in milliseconds before first retry
///
/// # Returns
///
/// The result of the operation if successful, or the last error if all attempts fail
///
/// # Example
///
/// ```ignore
/// use data_engine::error::{retry_with_backoff, Result};
///
/// async fn flaky_operation() -> Result<String> {
///     // Your operation here
///     Ok("success".to_string())
/// }
///
/// let result = retry_with_backoff(
///     || Box::pin(flaky_operation()),
///     5,
///     1000,
/// ).await;
/// ```
pub async fn retry_with_backoff<F, Fut, T>(
    mut operation: F,
    max_attempts: u32,
    initial_delay_ms: u64,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut delay = initial_delay_ms;
    let max_delay = 60000; // Cap at 60 seconds

    for attempt in 1..=max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt == max_attempts => {
                tracing::error!("Operation failed after {} attempts: {}", max_attempts, e);
                return Err(e);
            }
            Err(e) => {
                tracing::warn!(
                    "Attempt {}/{} failed: {}. Retrying in {}ms",
                    attempt,
                    max_attempts,
                    e,
                    delay
                );
                tokio::time::sleep(Duration::from_millis(delay)).await;
                // Exponential backoff with cap
                delay = std::cmp::min(delay * 2, max_delay);
            }
        }
    }

    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_error_display() {
        let err = DataError::ConnectionFailed {
            data_source: "Binance".to_string(),
            reason: "Network timeout".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Connection failed for Binance: Network timeout"
        );

        let err = DataError::ParseError {
            data_source: "Parser".to_string(),
            message: "Invalid JSON".to_string(),
            raw_data: "{}".to_string(),
        };
        assert_eq!(err.to_string(), "Parse error for Parser: Invalid JSON");

        let err = DataError::ParserNotFound("BinanceSpot".to_string());
        assert_eq!(err.to_string(), "Parser not found for source: BinanceSpot");
    }

    #[tokio::test]
    async fn test_retry_with_backoff_success_first_try() {
        let result = retry_with_backoff(|| async { Ok::<i32, DataError>(42) }, 5, 100).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_with_backoff_success_after_retries() {
        let attempts = Arc::new(Mutex::new(0));
        let attempts_clone = attempts.clone();

        let result = retry_with_backoff(
            move || {
                let attempts = attempts_clone.clone();
                async move {
                    let mut count = attempts.lock().unwrap();
                    *count += 1;
                    if *count < 3 {
                        Err(DataError::Other("Temporary failure".to_string()))
                    } else {
                        Ok(42)
                    }
                }
            },
            5,
            10, // Small delay for faster test
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(*attempts.lock().unwrap(), 3);
    }

    #[tokio::test]
    async fn test_retry_with_backoff_max_attempts() {
        let attempts = Arc::new(Mutex::new(0));
        let attempts_clone = attempts.clone();

        let result = retry_with_backoff(
            move || {
                let attempts = attempts_clone.clone();
                async move {
                    let mut count = attempts.lock().unwrap();
                    *count += 1;
                    Err::<i32, DataError>(DataError::Other("Always fails".to_string()))
                }
            },
            3,
            10,
        )
        .await;

        assert!(result.is_err());
        assert_eq!(*attempts.lock().unwrap(), 3);
    }

    #[tokio::test]
    async fn test_retry_exponential_backoff_timing() {
        let start = std::time::Instant::now();
        let attempts = Arc::new(Mutex::new(0));
        let attempts_clone = attempts.clone();

        let _ = retry_with_backoff(
            move || {
                let attempts = attempts_clone.clone();
                async move {
                    let mut count = attempts.lock().unwrap();
                    *count += 1;
                    Err::<i32, DataError>(DataError::Other("Test".to_string()))
                }
            },
            3,
            10, // 10ms initial delay
        )
        .await;

        let elapsed = start.elapsed().as_millis();
        // Should be roughly 10ms + 20ms = 30ms (with some tolerance)
        assert!((25..100).contains(&elapsed));
    }

    #[test]
    fn test_error_from_conversion() {
        // Test From conversions
        let redis_err: DataError =
            redis::RedisError::from((redis::ErrorKind::IoError, "Connection failed")).into();
        assert!(matches!(redis_err, DataError::RedisError(_)));

        let io_err: DataError =
            std::io::Error::new(std::io::ErrorKind::NotFound, "File not found").into();
        assert!(matches!(io_err, DataError::IoError(_)));
    }
}
