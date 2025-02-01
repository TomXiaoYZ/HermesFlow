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

    #[error("速率限制错误: {0}")]
    RateLimitError(String),
}

/// 解析错误类型
#[derive(Debug, Error)]
pub enum ParseErrorKind {
    #[error("JSON解析错误: {0}")]
    JsonError(String),

    #[error("数字解析错误: {0}")]
    NumberParseError(String),

    #[error("时间解析错误: {0}")]
    TimeParseError(String),
}

/// OKX 错误类型
#[derive(Debug, Error)]
pub enum OkxError {
    #[error("WebSocket错误: {0}")]
    WebSocketError(#[from] WebSocketErrorKind),

    #[error("REST API错误: {0}")]
    RestError(#[from] RestErrorKind),

    #[error("解析错误: {0}")]
    ParseError(#[from] ParseErrorKind),

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
            Self::RateLimitError(msg) => write!(f, "REST API rate limit error: {}", msg),
        }
    }
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::JsonError(msg) => write!(f, "JSON parse error: {}", msg),
            Self::NumberParseError(msg) => write!(f, "Number parse error: {}", msg),
            Self::TimeParseError(msg) => write!(f, "Time parse error: {}", msg),
        }
    }
}

impl From<tungstenite::Error> for OkxError {
    fn from(err: tungstenite::Error) -> Self {
        OkxError::WebSocketError(WebSocketErrorKind::ConnectionError(err.to_string()))
    }
}

impl From<reqwest::Error> for OkxError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            OkxError::RestError(RestErrorKind::RequestError(format!("Request timeout: {}", err)))
        } else if err.is_connect() {
            OkxError::RestError(RestErrorKind::RequestError(format!("Connection failed: {}", err)))
        } else if err.is_status() {
            OkxError::RestError(RestErrorKind::ResponseError(format!("HTTP error: {}", err)))
        } else {
            OkxError::RestError(RestErrorKind::RequestError(err.to_string()))
        }
    }
}

impl From<serde_json::Error> for OkxError {
    fn from(err: serde_json::Error) -> Self {
        OkxError::ParseError(ParseErrorKind::JsonError(err.to_string()))
    }
}

impl From<std::num::ParseIntError> for OkxError {
    fn from(err: std::num::ParseIntError) -> Self {
        OkxError::ParseError(ParseErrorKind::NumberParseError(err.to_string()))
    }
}

impl From<std::num::ParseFloatError> for OkxError {
    fn from(err: std::num::ParseFloatError) -> Self {
        OkxError::ParseError(ParseErrorKind::NumberParseError(err.to_string()))
    }
}

impl From<chrono::ParseError> for OkxError {
    fn from(err: chrono::ParseError) -> Self {
        OkxError::ParseError(ParseErrorKind::TimeParseError(err.to_string()))
    }
}

impl From<OkxError> for CollectorError {
    fn from(error: OkxError) -> Self {
        match error {
            OkxError::WebSocketError(kind) => {
                CollectorError::WebSocketError(kind.to_string())
            }
            OkxError::RestError(kind) => {
                CollectorError::RestError(kind.to_string())
            }
            OkxError::ParseError(kind) => {
                CollectorError::ParseError(kind.to_string())
            }
            OkxError::ConfigError(msg) => {
                CollectorError::ConfigError(msg)
            }
            OkxError::SystemError(msg) => {
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
        // 测试WebSocket错误转换
        let ws_error = WebSocketErrorKind::ConnectionError("连接失败".to_string());
        let error: OkxError = ws_error.into();
        assert!(matches!(error, OkxError::WebSocketError(_)));

        // 测试REST错误转换
        let rest_error = RestErrorKind::RequestError("请求超时".to_string());
        let error: OkxError = rest_error.into();
        assert!(matches!(error, OkxError::RestError(_)));

        // 测试解析错误转换
        let parse_error = ParseErrorKind::JsonError("无效的JSON格式".to_string());
        let error: OkxError = parse_error.into();
        assert!(matches!(error, OkxError::ParseError(_)));
    }

    #[test]
    fn test_error_display() {
        let error = OkxError::ConfigError("无效的配置".to_string());
        assert_eq!(error.to_string(), "配置错误: 无效的配置");

        let error = OkxError::SystemError("系统错误".to_string());
        assert_eq!(error.to_string(), "系统错误: 系统错误");
    }

    #[test]
    fn test_error_source() {
        let err = OkxError::SystemError("Connection timeout".to_string());
        assert!(err.source().is_none());
        assert!(err.to_string().contains("Connection timeout"));
    }

    #[test]
    fn test_collector_error_conversion() {
        let okx_error = OkxError::WebSocketError(WebSocketErrorKind::ConnectionError("连接失败".to_string()));
        let collector_error: CollectorError = okx_error.into();
        assert!(matches!(collector_error, CollectorError::WebSocketError(_)));

        let okx_error = OkxError::RestError(RestErrorKind::RequestError("请求失败".to_string()));
        let collector_error: CollectorError = okx_error.into();
        assert!(matches!(collector_error, CollectorError::RestError(_)));
    }
} 