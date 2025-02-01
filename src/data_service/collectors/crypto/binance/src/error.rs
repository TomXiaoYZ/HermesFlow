use std::fmt;
use thiserror::Error;
use tokio_tungstenite::tungstenite;
use reqwest;
use serde_json;
use common::CollectorError;

/// WebSocket 错误类型
#[derive(Debug, Error)]
pub enum WebSocketErrorKind {
    #[error("连接错误: {0}")]
    ConnectionError(String),

    #[error("发送错误: {0}")]
    SendError(String),

    #[error("接收错误: {0}")]
    ReceiveError(String),

    #[error("订阅错误: {0}")]
    SubscriptionError(String),
}

/// REST API 错误类型
#[derive(Debug, Error)]
pub enum RestErrorKind {
    #[error("请求错误: {0}")]
    RequestError(String),

    #[error("响应错误: {0}")]
    ResponseError(String),

    #[error("认证错误: {0}")]
    AuthenticationError(String),
}

/// Binance 错误类型
#[derive(Debug, Error)]
pub enum BinanceError {
    #[error("WebSocket错误: {0}")]
    WebSocketError(#[from] WebSocketErrorKind),

    #[error("REST API错误: {0}")]
    RestError(#[from] RestErrorKind),

    #[error("解析错误: {0}")]
    ParseError(String),

    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("系统错误: {0}")]
    SystemError(String),
}

impl fmt::Display for WebSocketErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConnectionError(msg) => write!(f, "WebSocket connection error: {}", msg),
            Self::SendError(msg) => write!(f, "WebSocket send error: {}", msg),
            Self::ReceiveError(msg) => write!(f, "WebSocket receive error: {}", msg),
            Self::SubscriptionError(msg) => write!(f, "WebSocket subscription error: {}", msg),
        }
    }
}

impl fmt::Display for RestErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RequestError(msg) => write!(f, "REST API request error: {}", msg),
            Self::ResponseError(msg) => write!(f, "REST API response error: {}", msg),
            Self::AuthenticationError(msg) => write!(f, "REST API authentication error: {}", msg),
        }
    }
}

impl From<tungstenite::Error> for BinanceError {
    fn from(err: tungstenite::Error) -> Self {
        BinanceError::WebSocketError(WebSocketErrorKind::ConnectionError(err.to_string()))
    }
}

impl From<reqwest::Error> for BinanceError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            BinanceError::SystemError(format!("Network error: Request timeout. {}", err))
        } else if err.is_connect() {
            BinanceError::SystemError(format!("Network error: Connection failed. {}", err))
        } else {
            BinanceError::RestError(RestErrorKind::RequestError(err.to_string()))
        }
    }
}

impl From<serde_json::Error> for BinanceError {
    fn from(err: serde_json::Error) -> Self {
        BinanceError::ParseError(err.to_string())
    }
}

impl From<BinanceError> for CollectorError {
    fn from(error: BinanceError) -> Self {
        match error {
            BinanceError::WebSocketError(kind) => {
                CollectorError::WebSocketError(kind.to_string())
            }
            BinanceError::RestError(kind) => {
                CollectorError::RestError(kind.to_string())
            }
            BinanceError::ParseError(kind) => {
                CollectorError::ParseError(kind)
            }
            BinanceError::ConfigError(msg) => {
                CollectorError::ConfigError(msg)
            }
            BinanceError::SystemError(msg) => {
                CollectorError::SystemError(msg)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_error_conversion() {
        let ws_error = WebSocketErrorKind::ConnectionError("连接失败".to_string());
        let error: BinanceError = ws_error.into();
        assert!(matches!(error, BinanceError::WebSocketError(_)));
    }

    #[test]
    fn test_error_display() {
        let error = BinanceError::ParseError("无效的JSON格式".to_string());
        assert_eq!(error.to_string(), "解析错误: 无效的JSON格式");
    }

    #[test]
    fn test_error_source() {
        let err = BinanceError::SystemError("Connection timeout".to_string());
        
        assert!(err.source().is_none());
        assert!(err.to_string().contains("Connection timeout"));
    }
} 