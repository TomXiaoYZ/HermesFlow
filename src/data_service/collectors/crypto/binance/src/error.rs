use thiserror::Error;
use common::CollectorError;
use tokio_tungstenite::tungstenite;
use url::ParseError;
use reqwest;

#[derive(Debug, Error)]
pub enum BinanceError {
    #[error(transparent)]
    Collector(#[from] CollectorError),

    #[error(transparent)]
    WebSocket(#[from] tungstenite::Error),

    #[error(transparent)]
    Url(#[from] ParseError),

    #[error(transparent)]
    Request(#[from] reqwest::Error),
}

// 为了方便使用，添加类型别名
pub type Result<T> = std::result::Result<T, BinanceError>;

// 实现从各种错误类型到 CollectorError 的转换
impl From<BinanceError> for CollectorError {
    fn from(err: BinanceError) -> Self {
        match err {
            BinanceError::Collector(e) => e,
            BinanceError::WebSocket(e) => CollectorError::WebSocketError(e.to_string()),
            BinanceError::Url(e) => CollectorError::ConfigError(e.to_string()),
            BinanceError::Request(e) => CollectorError::NetworkError(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )),
        }
    }
} 