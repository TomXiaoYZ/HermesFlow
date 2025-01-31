use std::sync::Arc;
use reqwest::{Client, RequestBuilder, Response};
use serde::de::DeserializeOwned;
use tokio::sync::Mutex;
use std::time::{Duration, Instant};
use tracing::error;
use chrono::Utc;
use serde_json::Value;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use common::{MarketData, DataQuality, MarketDataType, CollectorError};
use crate::error::BinanceError;

type HmacSha256 = Hmac<Sha256>;

const DEFAULT_RECV_WINDOW: u64 = 5000;
const DEFAULT_WEIGHT_PER_MINUTE: u32 = 1200;

/// REST客户端配置
#[derive(Debug, Clone)]
pub struct RestClientConfig {
    pub endpoint: String,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub recv_window: u64,
}

impl Default for RestClientConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://api.binance.com".to_string(),
            api_key: None,
            api_secret: None,
            recv_window: DEFAULT_RECV_WINDOW,
        }
    }
}

/// 速率限制器
#[derive(Debug)]
struct RateLimiter {
    weight_per_minute: u32,
    weights: Vec<Instant>,
}

impl RateLimiter {
    fn new(weight_per_minute: u32) -> Self {
        Self {
            weight_per_minute,
            weights: Vec::new(),
        }
    }

    fn check_rate_limit(&mut self) -> bool {
        let now = Instant::now();
        self.weights.retain(|&t| now.duration_since(t) < Duration::from_secs(60));
        self.weights.len() as u32 <= self.weight_per_minute
    }

    fn add_weight(&mut self, weight: u32) {
        let now = Instant::now();
        for _ in 0..weight {
            self.weights.push(now);
        }
    }
}

/// REST客户端
pub struct RestClient {
    config: RestClientConfig,
    client: Client,
    rate_limiter: Arc<Mutex<RateLimiter>>,
}

impl RestClient {
    pub fn new(
        endpoint: &str,
        api_key: Option<String>,
        api_secret: Option<String>,
    ) -> Self {
        Self {
            config: RestClientConfig {
                endpoint: endpoint.to_string(),
                api_key,
                api_secret,
                recv_window: DEFAULT_RECV_WINDOW,
            },
            client: Client::new(),
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(DEFAULT_WEIGHT_PER_MINUTE))),
        }
    }

    async fn check_rate_limit(&self) -> Result<(), BinanceError> {
        let mut rate_limiter = self.rate_limiter.lock().await;
        if !rate_limiter.check_rate_limit() {
            return Err(CollectorError::RateLimitError.into());
        }
        rate_limiter.add_weight(1);
        Ok(())
    }

    fn sign_request(&self, params: &str) -> Result<String, BinanceError> {
        if let Some(secret) = &self.config.api_secret {
            let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
                .map_err(|e| CollectorError::ConfigError(format!("Invalid API secret: {}", e)))?;
            mac.update(params.as_bytes());
            let signature = hex::encode(mac.finalize().into_bytes());
            Ok(signature)
        } else {
            Err(CollectorError::ConfigError("Missing API secret".to_string()).into())
        }
    }

    fn add_api_key_header(&self, builder: RequestBuilder) -> RequestBuilder {
        if let Some(api_key) = &self.config.api_key {
            builder.header("X-MBX-APIKEY", api_key)
        } else {
            builder
        }
    }

    async fn handle_response<T: DeserializeOwned>(
        &self,
        response: Response,
    ) -> Result<T, BinanceError> {
        let status = response.status();
        let text = response.text().await.map_err(|e| {
            CollectorError::ApiError {
                status_code: status.as_u16(),
                message: e.to_string(),
            }
        })?;

        if !status.is_success() {
            return Err(CollectorError::ApiError {
                status_code: status.as_u16(),
                message: text,
            }.into());
        }

        serde_json::from_str(&text).map_err(|e| {
            CollectorError::ParseError(format!("Failed to parse response: {}", e))
        })?;

        Ok(serde_json::from_str(&text)?)
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, BinanceError> {
        self.check_rate_limit().await?;

        let url = format!("{}{}", self.config.endpoint, path);
        let response = self.client.get(&url).send().await?;

        self.handle_response(response).await
    }

    pub async fn get_signed<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &str,
    ) -> Result<T, BinanceError> {
        self.check_rate_limit().await?;

        let timestamp = Utc::now().timestamp_millis();
        let mut signed_params = format!(
            "{}{}timestamp={}&recvWindow={}",
            params,
            if params.is_empty() { "" } else { "&" },
            timestamp,
            self.config.recv_window
        );

        let signature = self.sign_request(&signed_params)?;
        signed_params.push_str(&format!("&signature={}", signature));

        let url = format!("{}{}?{}", self.config.endpoint, path, signed_params);
        let response = self.add_api_key_header(self.client.get(&url)).send().await?;

        self.handle_response(response).await
    }

    pub async fn get_exchange_info(&self) -> Result<Value, BinanceError> {
        self.get("/api/v3/exchangeInfo").await
    }

    pub async fn get_ticker_price(&self, symbol: &str) -> Result<Value, BinanceError> {
        self.get(&format!("/api/v3/ticker/price?symbol={}", symbol))
            .await
    }

    pub async fn get_ticker_24h(&self, symbol: &str) -> Result<Value, BinanceError> {
        self.get(&format!("/api/v3/ticker/24hr?symbol={}", symbol))
            .await
    }

    pub async fn get_klines(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<u32>,
    ) -> Result<Value, BinanceError> {
        let mut url = format!(
            "/api/v3/klines?symbol={}&interval={}",
            symbol, interval
        );
        if let Some(limit) = limit {
            url.push_str(&format!("&limit={}", limit));
        }
        self.get(&url).await
    }

    pub async fn get_depth(
        &self,
        symbol: &str,
        limit: Option<u32>,
    ) -> Result<Value, BinanceError> {
        let mut url = format!("/api/v3/depth?symbol={}", symbol);
        if let Some(limit) = limit {
            url.push_str(&format!("&limit={}", limit));
        }
        self.get(&url).await
    }

    pub async fn get_recent_trades(
        &self,
        symbol: &str,
        limit: Option<u32>,
    ) -> Result<Value, BinanceError> {
        let mut url = format!("/api/v3/trades?symbol={}", symbol);
        if let Some(limit) = limit {
            url.push_str(&format!("&limit={}", limit));
        }
        self.get(&url).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rest_client_init() {
        let client = RestClient::new(
            "https://api.binance.com",
            Some("test_key".to_string()),
            Some("test_secret".to_string()),
        );
        assert_eq!(client.config.endpoint, "https://api.binance.com");
        assert_eq!(client.config.api_key, Some("test_key".to_string()));
        assert_eq!(client.config.api_secret, Some("test_secret".to_string()));
    }

    #[tokio::test]
    async fn test_rest_client_rate_limit() {
        let client = RestClient::new("https://api.binance.com", None, None);
        
        // 测试速率限制检查
        for _ in 0..5 {
            let result = client.check_rate_limit().await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_rest_client_get_ticker() {
        let client = RestClient::new("https://api.binance.com", None, None);
        let result = client.get_ticker_price("BTCUSDT").await;
        assert!(result.is_ok());

        if let Ok(data) = result {
            assert!(data["symbol"].as_str().unwrap() == "BTCUSDT");
            assert!(data["price"].as_str().is_some());
        }
    }

    #[tokio::test]
    async fn test_rest_client_get_klines() {
        let client = RestClient::new("https://api.binance.com", None, None);
        let result = client.get_klines("BTCUSDT", "1m", Some(10)).await;
        assert!(result.is_ok());

        if let Ok(data) = result {
            assert!(data.as_array().unwrap().len() <= 10);
        }
    }

    #[tokio::test]
    async fn test_rest_client_get_depth() {
        let client = RestClient::new("https://api.binance.com", None, None);
        let result = client.get_depth("BTCUSDT", Some(5)).await;
        assert!(result.is_ok());

        if let Ok(data) = result {
            assert!(data["bids"].as_array().is_some());
            assert!(data["asks"].as_array().is_some());
        }
    }
} 