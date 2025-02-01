use thiserror::Error;
use solana_client::client_error::ClientError;

#[derive(Debug, Error)]
pub enum SolError {
    #[error("Solana client error: {0}")]
    ClientError(#[from] ClientError),

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

    #[error("Account processing error: {0}")]
    AccountError(String),

    #[error("Program error: {0}")]
    ProgramError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

pub type Result<T> = std::result::Result<T, SolError>; 