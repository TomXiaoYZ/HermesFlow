use std::time::{SystemTime, UNIX_EPOCH};
use reqwest::Client;
use serde::de::DeserializeOwned;
use tracing::debug;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

use crate::error::{HuobiError, RestErrorKind};
use crate::types::{ApiResponse, Symbol};

type HmacSha256 = Hmac<Sha256>;

/// REST API 客户端
pub struct RestClient {
    client: Client,
    endpoint: String,
    api_key: Option<String>,
    api_secret: Option<String>,
}

impl RestClient {
    /// 创建新的 REST API 客户端实例
    pub fn new(endpoint: &str, api_key: Option<&str>, api_secret: Option<&str>) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        Self {
            client,
            endpoint: endpoint.to_string(),
            api_key: api_key.map(String::from),
            api_secret: api_secret.map(String::from),
        }
    }

    /// 生成签名
    fn sign(&self, method: &str, path: &str, params: &str) -> Result<String, HuobiError> {
        if let Some(secret) = &self.api_secret {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                .to_string();

            let sign_content = format!(
                "{}\napi.huobi.pro\n{}\n{}\n{}",
                method.to_uppercase(),
                path,
                timestamp,
                params
            );

            let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
                .map_err(|e| HuobiError::RestError(RestErrorKind::AuthenticationError(e.to_string())))?;
            mac.update(sign_content.as_bytes());
            let result = mac.finalize();
            Ok(BASE64.encode(result.into_bytes()))
        } else {
            Err(HuobiError::RestError(RestErrorKind::AuthenticationError(
                "API secret not configured".to_string(),
            )))
        }
    }

    /// 构建请求
    async fn request<T: DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        params: Option<String>,
    ) -> Result<T, HuobiError> {
        let url = format!("{}{}", self.endpoint, path);
        let mut request_builder = self.client.request(method.clone(), &url);

        // 添加认证信息（如果需要）
        if let Some(api_key) = &self.api_key {
            let signature = self.sign(
                method.as_str(),
                path,
                &params.clone().unwrap_or_default(),
            )?;
            request_builder = request_builder
                .header("AccessKeyId", api_key)
                .header("SignatureMethod", "HmacSHA256")
                .header("SignatureVersion", "2")
                .header("Signature", signature);
        }

        // 添加查询参数
        if let Some(params) = params {
            request_builder = request_builder.body(params);
        }

        debug!("发送请求: {} {}", method, url);
        let response = request_builder
            .send()
            .await
            .map_err(|e| HuobiError::RestError(RestErrorKind::RequestError(e.to_string())))?;

        // 检查响应状态
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(HuobiError::RestError(RestErrorKind::ResponseError(format!(
                "HTTP error {}: {}",
                status,
                error_text
            ))));
        }

        // 解析响应
        let response_text = response
            .text()
            .await
            .map_err(|e| HuobiError::RestError(RestErrorKind::ResponseError(e.to_string())))?;
        debug!("收到响应: {}", response_text);

        serde_json::from_str(&response_text)
            .map_err(|e| HuobiError::RestError(RestErrorKind::ResponseError(e.to_string())))
    }

    /// 构建请求（带重试机制）
    async fn request_with_retry<T: DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        params: Option<String>,
        max_retries: u32,
    ) -> Result<T, HuobiError> {
        let mut retries = 0;
        let mut last_error = None;

        while retries < max_retries {
            match self.request(method.clone(), path, params.clone()).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    retries += 1;
                    if retries < max_retries {
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            HuobiError::RestError(RestErrorKind::RequestError(
                "Maximum retries exceeded".to_string(),
            ))
        }))
    }

    /// 获取所有交易对信息（带重试）
    pub async fn get_symbols(&self) -> Result<Vec<Symbol>, HuobiError> {
        let response: ApiResponse<Vec<Symbol>> = self
            .request_with_retry(reqwest::Method::GET, "/v1/common/symbols", None, 3)
            .await?;

        match response.data {
            Some(symbols) => Ok(symbols),
            None => Err(HuobiError::RestError(RestErrorKind::ResponseError(
                "No symbols data in response".to_string(),
            ))),
        }
    }

    /// 获取指定交易对的最新行情（带重试）
    pub async fn get_ticker(&self, symbol: &str) -> Result<serde_json::Value, HuobiError> {
        let path = format!("/market/detail/merged?symbol={}", symbol);
        self.request_with_retry(reqwest::Method::GET, &path, None, 3).await
    }

    /// 获取指定交易对的最新深度数据（带重试）
    pub async fn get_depth(
        &self,
        symbol: &str,
        depth: u32,
        step: &str,
    ) -> Result<serde_json::Value, HuobiError> {
        let path = format!(
            "/market/depth?symbol={}&depth={}&type={}",
            symbol, depth, step
        );
        self.request_with_retry(reqwest::Method::GET, &path, None, 3).await
    }

    /// 获取指定交易对的最新成交记录（带重试）
    pub async fn get_trades(&self, symbol: &str) -> Result<serde_json::Value, HuobiError> {
        let path = format!("/market/history/trade?symbol={}&size=20", symbol);
        self.request_with_retry(reqwest::Method::GET, &path, None, 3).await
    }

    /// 获取指定交易对的K线数据（带重试）
    pub async fn get_klines(
        &self,
        symbol: &str,
        period: &str,
        size: u32,
    ) -> Result<serde_json::Value, HuobiError> {
        let path = format!(
            "/market/history/kline?symbol={}&period={}&size={}",
            symbol, period, size
        );
        self.request_with_retry(reqwest::Method::GET, &path, None, 3).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;
    use std::time::Duration;

    const TEST_TIMEOUT: Duration = Duration::from_secs(15);

    async fn create_test_client() -> RestClient {
        RestClient::new("https://api.huobi.pro", None, None)
    }

    #[tokio::test]
    async fn test_get_symbols() {
        let client = create_test_client().await;
        let result = timeout(TEST_TIMEOUT, client.get_symbols()).await;
        
        match result {
            Ok(Ok(symbols)) => {
                assert!(!symbols.is_empty(), "应该返回至少一个交易对");
                let btc_symbol = symbols.iter().find(|s| s.symbol == "btcusdt");
                assert!(btc_symbol.is_some(), "应该包含 BTCUSDT 交易对");
            }
            Ok(Err(e)) => {
                println!("获取交易对信息失败: {:?}", e);
                assert!(false, "获取交易对信息不应该失败");
            }
            Err(_) => {
                println!("获取交易对信息超时");
                assert!(false, "获取交易对信息不应该超时");
            }
        }
    }

    #[tokio::test]
    async fn test_get_ticker() {
        let client = create_test_client().await;
        let result = timeout(TEST_TIMEOUT, client.get_ticker("btcusdt")).await;
        
        match result {
            Ok(Ok(ticker)) => {
                assert!(ticker.get("status").is_some(), "应该包含状态字段");
                assert!(ticker.get("tick").is_some(), "应该包含行情数据");
            }
            Ok(Err(e)) => {
                println!("获取行情数据失败: {:?}", e);
                assert!(false, "获取行情数据不应该失败");
            }
            Err(_) => {
                println!("获取行情数据超时");
                assert!(false, "获取行情数据不应该超时");
            }
        }
    }

    #[tokio::test]
    async fn test_get_depth() {
        let client = create_test_client().await;
        let result = timeout(TEST_TIMEOUT, client.get_depth("btcusdt", 20, "step0")).await;
        
        match result {
            Ok(Ok(depth)) => {
                assert!(depth.get("status").is_some(), "应该包含状态字段");
                assert!(depth.get("tick").is_some(), "应该包含深度数据");
            }
            Ok(Err(e)) => {
                println!("获取深度数据失败: {:?}", e);
                assert!(false, "获取深度数据不应该失败");
            }
            Err(_) => {
                println!("获取深度数据超时");
                assert!(false, "获取深度数据不应该超时");
            }
        }
    }

    #[tokio::test]
    async fn test_get_trades() {
        let client = create_test_client().await;
        let result = timeout(TEST_TIMEOUT, client.get_trades("btcusdt")).await;
        
        match result {
            Ok(Ok(trades)) => {
                assert!(trades.get("status").is_some(), "应该包含状态字段");
                assert!(trades.get("data").is_some(), "应该包含成交数据");
            }
            Ok(Err(e)) => {
                println!("获取成交数据失败: {:?}", e);
                assert!(false, "获取成交数据不应该失败");
            }
            Err(_) => {
                println!("获取成交数据超时");
                assert!(false, "获取成交数据不应该超时");
            }
        }
    }

    #[tokio::test]
    async fn test_get_klines() {
        let client = create_test_client().await;
        let result = timeout(TEST_TIMEOUT, client.get_klines("btcusdt", "1min", 10)).await;
        
        match result {
            Ok(Ok(klines)) => {
                assert!(klines.get("status").is_some(), "应该包含状态字段");
                assert!(klines.get("data").is_some(), "应该包含K线数据");
            }
            Ok(Err(e)) => {
                println!("获取K线数据失败: {:?}", e);
                assert!(false, "获取K线数据不应该失败");
            }
            Err(_) => {
                println!("获取K线数据超时");
                assert!(false, "获取K线数据不应该超时");
            }
        }
    }
} 