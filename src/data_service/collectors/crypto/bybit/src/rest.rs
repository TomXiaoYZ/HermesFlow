use std::time::{SystemTime, UNIX_EPOCH};
use reqwest::{Client, RequestBuilder, StatusCode};
use serde_json::Value;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tracing::{error, info, warn};
use url::Url;

use crate::error::{BybitError, RestErrorKind, ParseErrorKind};
use crate::models::InstrumentInfo;

type HmacSha256 = Hmac<Sha256>;

/// REST API 客户端
pub struct RestClient {
    /// HTTP 客户端
    client: Client,
    /// API 端点
    endpoint: String,
    /// API Key
    api_key: Option<String>,
    /// API Secret
    api_secret: Option<String>,
}

impl RestClient {
    /// 创建新的 REST API 客户端实例
    pub fn new(endpoint: &str, api_key: Option<&str>, api_secret: Option<&str>) -> Self {
        Self {
            client: Client::new(),
            endpoint: endpoint.to_string(),
            api_key: api_key.map(String::from),
            api_secret: api_secret.map(String::from),
        }
    }

    /// 生成签名
    fn sign(&self, timestamp: i64, params: &str) -> Result<String, BybitError> {
        if let Some(secret) = &self.api_secret {
            let sign_str = format!("{}{}{}", timestamp, self.api_key.as_ref().unwrap(), params);
            let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
                .map_err(|e| BybitError::AuthError {
                    msg: format!("Failed to create HMAC: {}", e),
                    source: Some(Box::new(e)),
                })?;
            mac.update(sign_str.as_bytes());
            let result = mac.finalize();
            Ok(hex::encode(result.into_bytes()))
        } else {
            Err(BybitError::AuthError {
                msg: "API secret not configured".to_string(),
                source: None,
            })
        }
    }

    /// 添加认证信息
    fn add_auth(&self, builder: RequestBuilder) -> Result<RequestBuilder, BybitError> {
        if let (Some(api_key), Some(_)) = (&self.api_key, &self.api_secret) {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| BybitError::SystemError {
                    msg: format!("Failed to get system time: {}", e),
                    source: Some(Box::new(e)),
                })?
                .as_millis() as i64;
            
            let sign = self.sign(timestamp, "")?;
            
            Ok(builder
                .header("X-BAPI-API-KEY", api_key)
                .header("X-BAPI-TIMESTAMP", timestamp.to_string())
                .header("X-BAPI-SIGN", sign))
        } else {
            Ok(builder)
        }
    }

    /// 发送 GET 请求
    async fn get(&self, path: &str, params: Option<&[(&str, &str)]>) -> Result<Value, BybitError> {
        let url = format!("{}{}", self.endpoint, path);
        let mut builder = self.client.get(&url);
        
        if let Some(params) = params {
            builder = builder.query(params);
        }
        
        builder = self.add_auth(builder)?;
        
        let response = builder
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    BybitError::NetworkError {
                        msg: "Request timeout".to_string(),
                        source: Some(Box::new(e)),
                    }
                } else if e.is_connect() {
                    BybitError::NetworkError {
                        msg: "Connection failed".to_string(),
                        source: Some(Box::new(e)),
                    }
                } else {
                    BybitError::RestError {
                        kind: RestErrorKind::RequestFailed,
                        source: Some(Box::new(e)),
                    }
                }
            })?;
        
        match response.status() {
            StatusCode::OK => {
                let data = response
                    .json::<Value>()
                    .await
                    .map_err(|e| BybitError::ParseError {
                        kind: ParseErrorKind::JsonError,
                        source: Some(Box::new(e)),
                    })?;
                
                if let Some(ret_code) = data.get("retCode").and_then(|v| v.as_i64()) {
                    if ret_code != 0 {
                        let msg = data.get("retMsg")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown error");
                        
                        return Err(BybitError::BusinessError {
                            code: ret_code as i32,
                            msg: msg.to_string(),
                        });
                    }
                }
                
                Ok(data)
            }
            StatusCode::TOO_MANY_REQUESTS => {
                let retry_after = response.headers()
                    .get("Retry-After")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .map(std::time::Duration::from_secs);
                
                Err(BybitError::RateLimitError {
                    msg: "Rate limit exceeded".to_string(),
                    retry_after,
                })
            }
            StatusCode::UNAUTHORIZED => {
                Err(BybitError::RestError {
                    kind: RestErrorKind::AuthenticationFailed,
                    source: None,
                })
            }
            StatusCode::BAD_REQUEST => {
                Err(BybitError::RestError {
                    kind: RestErrorKind::InvalidParameters,
                    source: None,
                })
            }
            _ => {
                Err(BybitError::RestError {
                    kind: RestErrorKind::ResponseError,
                    source: None,
                })
            }
        }
    }

    /// 获取交易对信息
    pub async fn get_instruments(&self, category: &str) -> Result<Vec<InstrumentInfo>, BybitError> {
        let params = &[("category", category)];
        let response = self.get("/v5/market/instruments-info", Some(params)).await?;
        
        if let Some(list) = response.get("result").and_then(|v| v.get("list")) {
            serde_json::from_value(list.clone())
                .map_err(|e| BybitError::ParseError {
                    kind: ParseErrorKind::JsonError,
                    source: Some(Box::new(e)),
                })
        } else {
            Err(BybitError::ParseError {
                kind: ParseErrorKind::MissingField("instruments data".to_string()),
                source: None,
            })
        }
    }

    /// 获取行情快照
    pub async fn get_tickers(&self, category: &str, symbol: Option<&str>) -> Result<Value, BybitError> {
        let mut params = vec![("category", category)];
        if let Some(symbol) = symbol {
            params.push(("symbol", symbol));
        }
        self.get("/v5/market/tickers", Some(&params)).await
    }

    /// 获取最新深度数据
    pub async fn get_orderbook(&self, category: &str, symbol: &str, limit: Option<u32>) -> Result<Value, BybitError> {
        let mut params = vec![
            ("category", category),
            ("symbol", symbol),
        ];
        
        if let Some(limit) = limit {
            params.push(("limit", &limit.to_string()));
        }
        
        self.get("/v5/market/orderbook", Some(&params)).await
    }

    /// 获取最新成交数据
    pub async fn get_trades(&self, category: &str, symbol: &str, limit: Option<u32>) -> Result<Value, BybitError> {
        let mut params = vec![
            ("category", category),
            ("symbol", symbol),
        ];
        
        if let Some(limit) = limit {
            params.push(("limit", &limit.to_string()));
        }
        
        self.get("/v5/market/recent-trade", Some(&params)).await
    }

    /// 获取K线数据
    pub async fn get_klines(&self, category: &str, symbol: &str, interval: &str, limit: Option<u32>) -> Result<Value, BybitError> {
        let mut params = vec![
            ("category", category),
            ("symbol", symbol),
            ("interval", interval),
        ];
        
        if let Some(limit) = limit {
            params.push(("limit", &limit.to_string()));
        }
        
        self.get("/v5/market/kline", Some(&params)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    const TEST_REST_ENDPOINT: &str = "https://api.bybit.com";

    #[tokio::test]
    async fn test_get_instruments() {
        let client = RestClient::new(TEST_REST_ENDPOINT, None, None);
        let result = client.get_instruments("spot").await;
        assert!(result.is_ok(), "Failed to get instruments: {:?}", result);
        
        let instruments = result.unwrap();
        assert!(!instruments.is_empty(), "Instruments list is empty");
    }

    #[tokio::test]
    async fn test_get_tickers() {
        let client = RestClient::new(TEST_REST_ENDPOINT, None, None);
        let result = client.get_tickers("spot", Some("BTCUSDT")).await;
        assert!(result.is_ok(), "Failed to get tickers: {:?}", result);
    }

    #[tokio::test]
    async fn test_get_orderbook() {
        let client = RestClient::new(TEST_REST_ENDPOINT, None, None);
        let result = client.get_orderbook("spot", "BTCUSDT", Some(50)).await;
        assert!(result.is_ok(), "Failed to get orderbook: {:?}", result);
    }

    #[tokio::test]
    async fn test_get_trades() {
        let client = RestClient::new(TEST_REST_ENDPOINT, None, None);
        let result = client.get_trades("spot", "BTCUSDT", Some(50)).await;
        assert!(result.is_ok(), "Failed to get trades: {:?}", result);
    }

    #[tokio::test]
    async fn test_get_klines() {
        let client = RestClient::new(TEST_REST_ENDPOINT, None, None);
        let result = client.get_klines("spot", "BTCUSDT", "1", Some(100)).await;
        assert!(result.is_ok(), "Failed to get klines: {:?}", result);
    }

    #[tokio::test]
    async fn test_error_handling() {
        let client = RestClient::new(TEST_REST_ENDPOINT, None, None);
        
        // 测试无效参数错误
        let result = client.get_orderbook("invalid", "BTCUSDT", None).await;
        assert!(matches!(result,
            Err(BybitError::RestError {
                kind: RestErrorKind::InvalidParameters,
                ..
            })
        ));
        
        // 测试认证错误
        let client = RestClient::new(
            TEST_REST_ENDPOINT,
            Some("invalid_key"),
            Some("invalid_secret"),
        );
        let result = client.get_instruments("spot").await;
        assert!(matches!(result,
            Err(BybitError::RestError {
                kind: RestErrorKind::AuthenticationFailed,
                ..
            })
        ));
    }
} 