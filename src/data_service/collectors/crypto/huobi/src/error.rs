use thiserror::Error;
use common::CollectorError;

/// WebSocket 相关错误
#[derive(Debug, Error)]
pub enum WebSocketErrorKind {
    #[error("连接错误: {0}")]
    ConnectionError(String),
    #[error("消息发送错误: {0}")]
    SendError(String),
    #[error("消息接收错误: {0}")]
    ReceiveError(String),
    #[error("订阅错误: {0}")]
    SubscriptionError(String),
}

/// REST API 相关错误
#[derive(Debug, Error)]
pub enum RestErrorKind {
    #[error("请求错误: {0}")]
    RequestError(String),
    #[error("响应错误: {0}")]
    ResponseError(String),
    #[error("认证错误: {0}")]
    AuthenticationError(String),
    #[error("API 限流: {0}")]
    RateLimitError(String),
}

/// 数据解析相关错误
#[derive(Debug, Error)]
pub enum ParseErrorKind {
    #[error("JSON 解析错误: {0}")]
    JsonError(String),
    #[error("数据格式错误: {0}")]
    FormatError(String),
    #[error("数据验证错误: {0}")]
    ValidationError(String),
}

/// 火币交易所相关错误
#[derive(Debug, Error)]
pub enum HuobiError {
    #[error("WebSocket 错误: {0}")]
    WebSocketError(#[from] WebSocketErrorKind),
    
    #[error("REST API 错误: {0}")]
    RestError(#[from] RestErrorKind),
    
    #[error("解析错误: {0}")]
    ParseError(String),
    
    #[error("配置错误: {0}")]
    ConfigError(String),
    
    #[error("系统错误: {0}")]
    SystemError(String),
}

impl From<tokio_tungstenite::tungstenite::Error> for HuobiError {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        HuobiError::WebSocketError(WebSocketErrorKind::ConnectionError(err.to_string()))
    }
}

impl From<reqwest::Error> for HuobiError {
    fn from(err: reqwest::Error) -> Self {
        HuobiError::RestError(RestErrorKind::RequestError(err.to_string()))
    }
}

impl From<serde_json::Error> for HuobiError {
    fn from(err: serde_json::Error) -> Self {
        HuobiError::ParseError(err.to_string())
    }
}

impl From<HuobiError> for CollectorError {
    fn from(err: HuobiError) -> Self {
        match err {
            HuobiError::WebSocketError(e) => CollectorError::WebSocketError(e.to_string()),
            HuobiError::RestError(e) => CollectorError::RestError(e.to_string()),
            HuobiError::ParseError(e) => CollectorError::ParseError(e),
            HuobiError::ConfigError(e) => CollectorError::ConfigError(e),
            HuobiError::SystemError(e) => CollectorError::SystemError(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion() {
        // 测试 WebSocket 错误转换
        let ws_err = HuobiError::WebSocketError(WebSocketErrorKind::ConnectionError("连接失败".to_string()));
        let collector_err: CollectorError = ws_err.into();
        assert!(matches!(collector_err, CollectorError::WebSocketError(_)));

        // 测试 REST 错误转换
        let rest_err = HuobiError::RestError(RestErrorKind::RequestError("请求失败".to_string()));
        let collector_err: CollectorError = rest_err.into();
        assert!(matches!(collector_err, CollectorError::RestError(_)));

        // 测试解析错误转换
        let parse_err = HuobiError::ParseError("解析失败".to_string());
        let collector_err: CollectorError = parse_err.into();
        assert!(matches!(collector_err, CollectorError::ParseError(_)));
    }

    #[test]
    fn test_error_display() {
        let err = HuobiError::WebSocketError(WebSocketErrorKind::ConnectionError("测试错误".to_string()));
        assert!(err.to_string().contains("测试错误"));

        let err = HuobiError::RestError(RestErrorKind::RequestError("API错误".to_string()));
        assert!(err.to_string().contains("API错误"));

        let err = HuobiError::ParseError("JSON错误".to_string());
        assert!(err.to_string().contains("JSON错误"));
    }
} 