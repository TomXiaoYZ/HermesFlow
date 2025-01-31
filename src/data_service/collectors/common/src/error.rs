use thiserror::Error;
use std::io;

/// 数据采集错误类型
#[derive(Debug, Error)]
pub enum CollectorError {
    #[error("初始化错误: {0}")]
    InitError(String),

    #[error("连接错误: {0}")]
    ConnectionError(String),

    #[error("认证错误: {0}")]
    AuthError(String),

    #[error("订阅错误: {0}")]
    SubscriptionError(String),

    #[error("数据处理错误: {0}")]
    ProcessingError(String),

    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("API错误: {status_code} - {message}")]
    ApiError {
        status_code: u16,
        message: String,
    },

    #[error("WebSocket错误: {0}")]
    WebSocketError(String),

    #[error("REST API错误: {code} - {msg}")]
    RestApiError { code: i32, msg: String },

    #[error("解析错误: {0}")]
    ParseError(String),

    #[error("速率限制错误")]
    RateLimitError,

    #[error("网络错误: {0}")]
    NetworkError(#[from] io::Error),

    #[error("序列化错误: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("未知错误: {0}")]
    Unknown(String),
}

/// 数据处理错误类型
#[derive(Error, Debug)]
pub enum ProcessorError {
    #[error("数据验证错误: {0}")]
    ValidationError(String),

    #[error("数据转换错误: {0}")]
    TransformError(String),

    #[error("数据质量错误: {0}")]
    QualityError(String),

    #[error("未知错误: {0}")]
    Unknown(String),
}

/// 数据发布错误类型
#[derive(Error, Debug)]
pub enum PublisherError {
    #[error("发布错误: {0}")]
    PublishError(String),

    #[error("连接错误: {0}")]
    ConnectionError(String),

    #[error("序列化错误: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("未知错误: {0}")]
    Unknown(String),
}

/// 管理器错误类型
#[derive(Error, Debug)]
pub enum ManagerError {
    #[error("采集器错误: {0}")]
    CollectorError(#[from] CollectorError),

    #[error("处理器错误: {0}")]
    ProcessorError(#[from] ProcessorError),

    #[error("发布器错误: {0}")]
    PublisherError(#[from] PublisherError),

    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("未知错误: {0}")]
    Unknown(String),
} 