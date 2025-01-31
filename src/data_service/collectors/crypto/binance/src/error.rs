use thiserror::Error;
use crate::common::error::CollectorError;

#[derive(Error, Debug)]
pub enum BinanceError {
    #[error("WebSocket错误: {0}")]
    WebSocketError(String),

    #[error("REST API错误: {code} - {msg}")]
    ApiError { code: i32, msg: String },

    #[error("数据解析错误: {0}")]
    ParseError(String),

    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error(transparent)]
    CollectorError(#[from] CollectorError),

    #[error(transparent)]
    WebSocketProtocolError(#[from] tokio_tungstenite::tungstenite::Error),

    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
}

impl From<BinanceError> for CollectorError {
    fn from(err: BinanceError) -> Self {
        match err {
            BinanceError::WebSocketError(msg) => CollectorError::ConnectionError(msg),
            BinanceError::ApiError { code, msg } => CollectorError::ApiError {
                status_code: code as u16,
                message: msg,
            },
            BinanceError::ParseError(msg) => CollectorError::ParseError(msg),
            BinanceError::ConfigError(msg) => CollectorError::ConfigError(msg),
            BinanceError::CollectorError(e) => e,
            BinanceError::WebSocketProtocolError(e) => CollectorError::ConnectionError(e.to_string()),
            BinanceError::ReqwestError(e) => CollectorError::NetworkError(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )),
            BinanceError::SerdeError(e) => CollectorError::SerializationError(e),
        }
    }
} 