use thiserror::Error;
use ethers::providers::ProviderError;

#[derive(Error, Debug)]
pub enum UniswapV3Error {
    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),

    #[error("HTTP request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Contract error: {0}")]
    Contract(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Pool not found: {0}")]
    PoolNotFound(String),

    #[error("Invalid configuration: {0}")]
    Config(String),

    #[error("Event processing error: {0}")]
    Event(String),

    #[error("Data processing error: {0}")]
    Processing(String),

    #[error("Decimal conversion error: {0}")]
    Decimal(#[from] rust_decimal::Error),

    #[error("Invalid tick range: {0}")]
    TickRange(String),

    #[error("Position not found: {0}")]
    PositionNotFound(String),

    #[error("Liquidity error: {0}")]
    Liquidity(String),

    #[error("Other error: {0}")]
    Other(String),
} 