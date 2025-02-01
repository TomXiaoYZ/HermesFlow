use thiserror::Error;
use web3::Error as Web3Error;

#[derive(Debug, Error)]
pub enum EthError {
    #[error("Web3 error: {0}")]
    Web3Error(#[from] Web3Error),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Websocket error: {0}")]
    WebsocketError(String),

    #[error("Data parsing error: {0}")]
    ParseError(String),

    #[error("Block processing error: {0}")]
    BlockError(String),

    #[error("Transaction processing error: {0}")]
    TransactionError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

pub type Result<T> = std::result::Result<T, EthError>; 