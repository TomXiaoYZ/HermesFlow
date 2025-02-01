use thiserror::Error;

/// Kucoin错误类型
#[derive(Error, Debug)]
pub enum KucoinError {
    /// API错误
    #[error("API error: code={code}, message={message}")]
    ApiError {
        code: i32,
        message: String,
    },

    /// 认证错误
    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    /// HTTP错误
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    /// WebSocket错误
    #[error("WebSocket error: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),

    /// JSON解析错误
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// URL解析错误
    #[error("URL parse error: {0}")]
    UrlParseError(#[from] url::ParseError),

    /// 网络错误
    #[error("Network error: {0}")]
    NetworkError(String),

    /// 内部错误
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// 结果类型别名
pub type Result<T> = std::result::Result<T, KucoinError>;