use thiserror::Error;

#[derive(Error, Debug)]
pub enum MexcError {
    #[error("HTTP请求错误: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("WebSocket错误: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("JSON解析错误: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("URL解析错误: {0}")]
    UrlParseError(#[from] url::ParseError),

    #[error("API错误: 代码 = {code}, 消息 = {message}")]
    ApiError {
        code: i32,
        message: String,
    },

    #[error("参数错误: {0}")]
    ParameterError(String),

    #[error("认证错误: {0}")]
    AuthenticationError(String),

    #[error("网络错误: {0}")]
    NetworkError(String),

    #[error("内部错误: {0}")]
    InternalError(String),

    #[error("未知错误: {0}")]
    Unknown(String),
} 