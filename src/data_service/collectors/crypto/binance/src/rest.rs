use std::time::{Duration, Instant};
use reqwest::{Client, RequestBuilder, Response};
use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::Mutex;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;
use url::Url;
use tracing::{debug, error, info, warn};

use crate::error::BinanceError;
use crate::collectors::common::{MarketData, DataQuality, MarketDataType};

type HmacSha256 = Hmac<Sha256>;

/// REST客户端配置
#[derive(Debug, Clone)]
pub struct RestClientConfig {
    pub endpoint: String,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub recv_window: Option<u64>,
}

/// 请求权重跟踪器
#[derive(Debug)]
struct RateLimiter {
    last_request: Instant,
    weight_count: u32,
    reset_time: Instant,
}

impl RateLimiter {
    fn new() -> Self {
        Self {
            last_request: Instant::now(),
            weight_count: 0,
            reset_time: Instant::now(),
        }
    }

    async fn check_rate_limit(&mut self) -> Result<(), BinanceError> {
        let now = Instant::now();
        if now >= self.reset_time {
            self.weight_count = 0;
            self.reset_time = now + Duration::from_secs(60);
        }

        if self.weight_count >= 1200 {
            return Err(BinanceError::RateLimitError);
        }

        // 确保请求间隔至少20ms
        if now - self.last_request < Duration::from_millis(20) {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }

        self.weight_count += 1;
        self.last_request = Instant::now();
        Ok(())
    }
}

/// REST API客户端
pub struct RestClient {
    config: RestClientConfig,
    client: Client,
    rate_limiter: Mutex<RateLimiter>,
}

impl RestClient {
    pub fn new(
        endpoint: &str,
        api_key: Option<&str>,
        api_secret: Option<&str>,
    ) -> Self {
        let config = RestClientConfig {
            endpoint: endpoint.to_string(),
            api_key: api_key.map(String::from),
            api_secret: api_secret.map(String::from),
            recv_window: Some(5000),
        };

        Self {
            config,
            client: Client::new(),
            rate_limiter: Mutex::new(RateLimiter::new()),
        }
    }

    /// 生成签名
    fn sign_request(&self, params: &str) -> Result<String, BinanceError> {
        if let Some(secret) = &self.config.api_secret {
            let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
                .map_err(|e| BinanceError::ConfigError(format!("Invalid API secret: {}", e)))?;
            mac.update(params.as_bytes());
            let signature = hex::encode(mac.finalize().into_bytes());
            Ok(signature)
        } else {
            Err(BinanceError::AuthError("API secret not configured".to_string()))
        }
    }

    /// 添加通用请求头
    fn add_headers(&self, builder: RequestBuilder) -> RequestBuilder {
        let mut builder = builder.header("User-Agent", "HermesFlow/1.0");
        
        if let Some(api_key) = &self.config.api_key {
            builder = builder.header("X-MBX-APIKEY", api_key);
        }
        
        builder
    }

    /// 发送公共GET请求
    pub async fn public_get<T: DeserializeOwned>(
        &self,
        path: &str,
        params: Option<&HashMap<String, String>>,
    ) -> Result<T, BinanceError> {
        self.rate_limiter.lock().await.check_rate_limit().await?;

        let url = format!("{}{}", self.config.endpoint, path);
        let mut url = Url::parse(&url)
            .map_err(|e| BinanceError::ConfigError(format!("Invalid URL: {}", e)))?;

        if let Some(params) = params {
            let mut query_pairs = url.query_pairs_mut();
            for (key, value) in params {
                query_pairs.append_pair(key, value);
            }
        }

        let response = self
            .add_headers(self.client.get(url.as_str()))
            .send()
            .await
            .map_err(|e| BinanceError::ReqwestError(e))?;

        self.handle_response(response).await
    }

    /// 发送签名GET请求
    pub async fn signed_get<T: DeserializeOwned>(
        &self,
        path: &str,
        mut params: HashMap<String, String>,
    ) -> Result<T, BinanceError> {
        self.rate_limiter.lock().await.check_rate_limit().await?;

        // 添加时间戳和接收窗口
        params.insert("timestamp".to_string(), chrono::Utc::now().timestamp_millis().to_string());
        if let Some(recv_window) = self.config.recv_window {
            params.insert("recvWindow".to_string(), recv_window.to_string());
        }

        // 生成签名
        let mut param_str = String::new();
        for (key, value) in &params {
            if !param_str.is_empty() {
                param_str.push('&');
            }
            param_str.push_str(&format!("{}={}", key, value));
        }
        let signature = self.sign_request(&param_str)?;
        params.insert("signature".to_string(), signature);

        let url = format!("{}{}", self.config.endpoint, path);
        let mut url = Url::parse(&url)
            .map_err(|e| BinanceError::ConfigError(format!("Invalid URL: {}", e)))?;

        {
            let mut query_pairs = url.query_pairs_mut();
            for (key, value) in params {
                query_pairs.append_pair(&key, &value);
            }
        }

        let response = self
            .add_headers(self.client.get(url.as_str()))
            .send()
            .await
            .map_err(|e| BinanceError::ReqwestError(e))?;

        self.handle_response(response).await
    }

    /// 处理API响应
    async fn handle_response<T: DeserializeOwned>(&self, response: Response) -> Result<T, BinanceError> {
        let status = response.status();
        let text = response.text().await
            .map_err(|e| BinanceError::ReqwestError(e))?;

        if !status.is_success() {
            let error: serde_json::Value = serde_json::from_str(&text)
                .map_err(|e| BinanceError::ParseError(format!("Failed to parse error response: {}", e)))?;
            
            return Err(BinanceError::ApiError {
                code: error["code"].as_i64().unwrap_or(-1) as i32,
                msg: error["msg"].as_str().unwrap_or("Unknown error").to_string(),
            });
        }

        serde_json::from_str(&text)
            .map_err(|e| BinanceError::ParseError(format!("Failed to parse response: {}", e)))
    }

    /// 获取交易对信息
    pub async fn get_exchange_info(&self) -> Result<serde_json::Value, BinanceError> {
        self.public_get("/api/v3/exchangeInfo", None).await
    }

    /// 获取最新价格
    pub async fn get_ticker_price(&self, symbol: &str) -> Result<serde_json::Value, BinanceError> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        self.public_get("/api/v3/ticker/price", Some(&params)).await
    }

    /// 获取24小时价格统计
    pub async fn get_ticker_24h(&self, symbol: &str) -> Result<serde_json::Value, BinanceError> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        self.public_get("/api/v3/ticker/24hr", Some(&params)).await
    }

    /// 获取K线数据
    pub async fn get_klines(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<u32>,
    ) -> Result<serde_json::Value, BinanceError> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        params.insert("interval".to_string(), interval.to_string());
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        self.public_get("/api/v3/klines", Some(&params)).await
    }

    /// 获取深度信息
    pub async fn get_depth(
        &self,
        symbol: &str,
        limit: Option<u32>,
    ) -> Result<serde_json::Value, BinanceError> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        self.public_get("/api/v3/depth", Some(&params)).await
    }

    /// 获取最近成交
    pub async fn get_trades(
        &self,
        symbol: &str,
        limit: Option<u32>,
    ) -> Result<serde_json::Value, BinanceError> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        self.public_get("/api/v3/trades", Some(&params)).await
    }
} 