use std::fmt;
use thiserror::Error;
use tokio_tungstenite::tungstenite;
use reqwest;
use serde_json;
use common::CollectorError;

/// ByBit 错误类型
#[derive(Error, Debug)]
pub enum BybitError {
    /// WebSocket 相关错误
    #[error("WebSocket error: {kind}")]
    WebSocketError {
        kind: WebSocketErrorKind,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// REST API 相关错误
    #[error("REST API error: {kind}")]
    RestError {
        kind: RestErrorKind,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 数据解析错误
    #[error("Parse error: {kind}")]
    ParseError {
        kind: ParseErrorKind,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 配置错误
    #[error("Config error: {msg}")]
    ConfigError {
        msg: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 网络错误
    #[error("Network error: {msg}")]
    NetworkError {
        msg: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 认证错误
    #[error("Authentication error: {msg}")]
    AuthError {
        msg: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 限流错误
    #[error("Rate limit exceeded: {msg}")]
    RateLimitError {
        msg: String,
        retry_after: Option<std::time::Duration>,
    },

    /// 业务逻辑错误
    #[error("Business error: {code} - {msg}")]
    BusinessError {
        code: i32,
        msg: String,
    },

    /// 系统错误
    #[error("System error: {msg}")]
    SystemError {
        msg: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

/// WebSocket 错误类型
#[derive(Debug)]
pub enum WebSocketErrorKind {
    /// 连接错误
    ConnectionFailed,
    /// 连接断开
    ConnectionClosed,
    /// 消息发送失败
    SendFailed,
    /// 消息接收失败
    ReceiveFailed,
    /// 认证失败
    AuthenticationFailed,
    /// 订阅失败
    SubscriptionFailed,
    /// Ping/Pong 超时
    PingPongTimeout,
    /// 其他错误
    Other(String),
}

impl fmt::Display for WebSocketErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConnectionFailed => write!(f, "Failed to establish WebSocket connection"),
            Self::ConnectionClosed => write!(f, "WebSocket connection closed"),
            Self::SendFailed => write!(f, "Failed to send WebSocket message"),
            Self::ReceiveFailed => write!(f, "Failed to receive WebSocket message"),
            Self::AuthenticationFailed => write!(f, "WebSocket authentication failed"),
            Self::SubscriptionFailed => write!(f, "Failed to subscribe to WebSocket channel"),
            Self::PingPongTimeout => write!(f, "WebSocket ping/pong timeout"),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

/// REST API 错误类型
#[derive(Debug)]
pub enum RestErrorKind {
    /// 请求错误
    RequestFailed,
    /// 响应错误
    ResponseError,
    /// API 限流
    RateLimit,
    /// 认证失败
    AuthenticationFailed,
    /// 参数错误
    InvalidParameters,
    /// 其他错误
    Other(String),
}

impl fmt::Display for RestErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RequestFailed => write!(f, "Failed to send REST request"),
            Self::ResponseError => write!(f, "Invalid REST response"),
            Self::RateLimit => write!(f, "REST API rate limit exceeded"),
            Self::AuthenticationFailed => write!(f, "REST API authentication failed"),
            Self::InvalidParameters => write!(f, "Invalid REST API parameters"),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

/// 解析错误类型
#[derive(Debug)]
pub enum ParseErrorKind {
    /// JSON 解析错误
    JsonError,
    /// 数值解析错误
    NumberParseError,
    /// 时间解析错误
    TimeParseError,
    /// 字段缺失
    MissingField(String),
    /// 字段类型错误
    InvalidFieldType(String),
    /// 其他错误
    Other(String),
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::JsonError => write!(f, "Failed to parse JSON"),
            Self::NumberParseError => write!(f, "Failed to parse number"),
            Self::TimeParseError => write!(f, "Failed to parse timestamp"),
            Self::MissingField(field) => write!(f, "Missing required field: {}", field),
            Self::InvalidFieldType(field) => write!(f, "Invalid field type: {}", field),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl From<tungstenite::Error> for BybitError {
    fn from(err: tungstenite::Error) -> Self {
        BybitError::WebSocketError {
            kind: WebSocketErrorKind::Other(err.to_string()),
            source: Some(Box::new(err)),
        }
    }
}

impl From<reqwest::Error> for BybitError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            BybitError::NetworkError {
                msg: "Request timeout".to_string(),
                source: Some(Box::new(err)),
            }
        } else if err.is_connect() {
            BybitError::NetworkError {
                msg: "Connection failed".to_string(),
                source: Some(Box::new(err)),
            }
        } else {
            BybitError::RestError {
                kind: RestErrorKind::Other(err.to_string()),
                source: Some(Box::new(err)),
            }
        }
    }
}

impl From<serde_json::Error> for BybitError {
    fn from(err: serde_json::Error) -> Self {
        BybitError::ParseError {
            kind: ParseErrorKind::JsonError,
            source: Some(Box::new(err)),
        }
    }
}

impl From<BybitError> for CollectorError {
    fn from(error: BybitError) -> Self {
        match error {
            BybitError::WebSocketError { kind, .. } => {
                CollectorError::WebSocketError(kind.to_string())
            }
            BybitError::RestError { kind, .. } => {
                CollectorError::RestError(kind.to_string())
            }
            BybitError::ParseError { kind, .. } => {
                CollectorError::ParseError(kind.to_string())
            }
            BybitError::ConfigError { msg, .. } => {
                CollectorError::ConfigError(msg)
            }
            BybitError::NetworkError { msg, .. } => {
                CollectorError::NetworkError(msg)
            }
            BybitError::AuthError { msg, .. } => {
                CollectorError::AuthError(msg)
            }
            BybitError::RateLimitError { msg, .. } => {
                CollectorError::RestError(format!("Rate limit exceeded: {}", msg))
            }
            BybitError::BusinessError { code, msg } => {
                CollectorError::RestError(format!("Business error {}: {}", code, msg))
            }
            BybitError::SystemError { msg, .. } => {
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
    fn test_websocket_error() {
        let err = BybitError::WebSocketError {
            kind: WebSocketErrorKind::ConnectionFailed,
            source: None,
        };
        assert!(err.to_string().contains("WebSocket error"));
        assert!(err.to_string().contains("Failed to establish"));
    }

    #[test]
    fn test_rest_error() {
        let err = BybitError::RestError {
            kind: RestErrorKind::RateLimit,
            source: None,
        };
        assert!(err.to_string().contains("REST API error"));
        assert!(err.to_string().contains("rate limit"));
    }

    #[test]
    fn test_parse_error() {
        let err = BybitError::ParseError {
            kind: ParseErrorKind::MissingField("price".to_string()),
            source: None,
        };
        assert!(err.to_string().contains("Parse error"));
        assert!(err.to_string().contains("Missing required field"));
    }

    #[test]
    fn test_error_conversion() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let bybit_err: BybitError = json_err.into();
        let collector_err: CollectorError = bybit_err.into();
        
        assert!(matches!(collector_err, CollectorError::ParseError(_)));
    }

    #[test]
    fn test_error_source() {
        let err = BybitError::NetworkError {
            msg: "Connection timeout".to_string(),
            source: Some(Box::new(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Timeout",
            ))),
        };
        
        assert!(err.source().is_some());
        assert!(err.to_string().contains("Connection timeout"));
    }
} 