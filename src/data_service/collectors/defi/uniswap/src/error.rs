use thiserror::Error;
use ethers::providers::ProviderError;
use web3::Error as Web3Error;

/// Uniswap错误类型
#[derive(Error, Debug)]
pub enum UniswapError {
    /// 连接错误
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// 合约错误
    #[error("Contract error: {0}")]
    ContractError(String),

    /// Graph API错误
    #[error("Graph API error: {0}")]
    GraphError(String),

    /// 解析错误
    #[error("Parse error: {0}")]
    ParseError(String),

    /// 请求错误
    #[error("Request error: {0}")]
    RequestError(String),

    /// 响应错误
    #[error("Response error: {0}")]
    ResponseError(String),

    /// 事件错误
    #[error("Event error: {0}")]
    EventError(String),

    /// 配置错误
    #[error("Config error: {0}")]
    ConfigError(String),

    /// 处理错误
    #[error("Process error: {0}")]
    ProcessError(String),

    /// 其他错误
    #[error("Other error: {0}")]
    Other(String),
}

impl From<ProviderError> for UniswapError {
    fn from(err: ProviderError) -> Self {
        UniswapError::ConnectionError(err.to_string())
    }
}

impl From<Web3Error> for UniswapError {
    fn from(err: Web3Error) -> Self {
        UniswapError::ConnectionError(err.to_string())
    }
}

impl From<reqwest::Error> for UniswapError {
    fn from(err: reqwest::Error) -> Self {
        UniswapError::RequestError(err.to_string())
    }
}

impl From<serde_json::Error> for UniswapError {
    fn from(err: serde_json::Error) -> Self {
        UniswapError::ParseError(err.to_string())
    }
} 