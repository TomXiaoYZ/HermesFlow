use std::sync::Arc;
use reqwest::{Client, RequestBuilder, Response};
use serde::de::DeserializeOwned;
use tokio::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tracing::error;
use chrono::Utc;
use serde_json::Value;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use url::Url;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

use common::{MarketData, DataQuality, MarketDataType, CollectorError};
use crate::error::{BinanceError, RestErrorKind};
use crate::types::{ApiResponse, Kline, Symbol};

type HmacSha256 = Hmac<Sha256>;

const DEFAULT_RECV_WINDOW: u64 = 5000;
const DEFAULT_WEIGHT_PER_MINUTE: u32 = 1200;
const API_BASE_URL: &str = "https://api.binance.com";
const API_VERSION: &str = "v3";

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
    base_url: String,
}

/// 交易对信息
#[derive(Debug, Deserialize)]
pub struct Symbol {
    /// 交易对
    pub symbol: String,
    /// 状态
    pub status: String,
    /// 基础资产
    #[serde(rename = "baseAsset")]
    pub base_asset: String,
    /// 计价资产
    #[serde(rename = "quoteAsset")]
    pub quote_asset: String,
    /// 价格精度
    #[serde(rename = "pricePrecision")]
    pub price_precision: i32,
    /// 数量精度
    #[serde(rename = "quantityPrecision")]
    pub quantity_precision: i32,
}

/// 深度数据
#[derive(Debug, Deserialize)]
pub struct OrderBook {
    /// 最后更新ID
    #[serde(rename = "lastUpdateId")]
    pub last_update_id: i64,
    /// 买单
    pub bids: Vec<(String, String)>,
    /// 卖单
    pub asks: Vec<(String, String)>,
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
            base_url: endpoint.to_string(),
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

    /// 获取深度数据
    pub async fn get_order_book(
        &self,
        symbol: &str,
        limit: Option<u32>,
    ) -> Result<OrderBook, BinanceError> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }

        self.get("depth", Some(params)).await
    }

    /// 获取K线数据
    pub async fn get_klines(
        &self,
        symbol: &str,
        interval: &str,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> Result<Vec<Kline>, BinanceError> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());

        if let Some(start_time) = start_time {
            params.insert("startTime".to_string(), start_time.to_string());
        }
        if let Some(end_time) = end_time {
            params.insert("endTime".to_string(), end_time.to_string());
        }
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }

        self.get("klines", Some(params)).await
    }

    /// 获取24小时价格统计
    pub async fn get_24h_ticker(&self, symbol: &str) -> Result<ApiResponse<TickerEvent>, BinanceError> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());

        self.get("ticker/24hr", Some(params)).await
    }

    /// 获取最新价格
    pub async fn get_price(&self, symbol: &str) -> Result<Decimal, BinanceError> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());

        #[derive(Deserialize)]
        struct PriceResponse {
            price: String,
        }

        let response: PriceResponse = self.get("ticker/price", Some(params)).await?;
        response.price.parse().map_err(|e| BinanceError::ParseError {
            kind: crate::error::ParseErrorKind::NumberParseError,
            source: Some(Box::new(e)),
        })
    }

    /// 生成签名
    fn sign(&self, method: &str, path: &str, params: &str) -> Result<String, BinanceError> {
        if let Some(secret) = &self.config.api_secret {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                .to_string();

            let sign_content = format!(
                "{}\n{}\n{}\n{}",
                method.to_uppercase(),
                path,
                timestamp,
                params
            );

            let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
                .map_err(|e| BinanceError::RestError(RestErrorKind::AuthenticationError(e.to_string())))?;
            mac.update(sign_content.as_bytes());
            let result = mac.finalize();
            Ok(BASE64.encode(result.into_bytes()))
        } else {
            Err(BinanceError::RestError(RestErrorKind::AuthenticationError(
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
    ) -> Result<T, BinanceError> {
        let url = format!("{}{}", self.config.endpoint, path);
        let mut request_builder = self.client.request(method.clone(), &url);

        // 添加认证信息（如果需要）
        if let Some(api_key) = &self.config.api_key {
            let signature = self.sign(
                method.as_str(),
                path,
                &params.clone().unwrap_or_default(),
            )?;
            request_builder = request_builder
                .header("X-MBX-APIKEY", api_key)
                .header("Content-Type", "application/json");
        }

        // 添加查询参数
        if let Some(params) = params {
            request_builder = request_builder.body(params);
        }

        debug!("发送请求: {} {}", method, url);
        let response = request_builder
            .send()
            .await
            .map_err(|e| BinanceError::RestError(RestErrorKind::RequestError(e.to_string())))?;

        // 检查响应状态
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(BinanceError::RestError(RestErrorKind::ResponseError(format!(
                "HTTP error {}: {}",
                status,
                error_text
            )))));
        }

        // 解析响应
        let response_text = response
            .text()
            .await
            .map_err(|e| BinanceError::RestError(RestErrorKind::ResponseError(e.to_string())))?;
        debug!("收到响应: {}", response_text);

        serde_json::from_str(&response_text)
            .map_err(|e| BinanceError::RestError(RestErrorKind::ResponseError(e.to_string())))
    }

    /// 构建请求（带重试机制）
    async fn request_with_retry<T: DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        params: Option<String>,
        max_retries: u32,
    ) -> Result<T, BinanceError> {
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
            BinanceError::RestError(RestErrorKind::RequestError(
                "Maximum retries exceeded".to_string(),
            ))
        }))
    }

    /// 获取所有交易对信息（带重试）
    pub async fn get_symbols(&self) -> Result<Vec<Symbol>, BinanceError> {
        let response: ApiResponse<Vec<Symbol>> = self
            .request_with_retry(reqwest::Method::GET, "/api/v3/exchangeInfo", None, 3)
            .await?;

        match response.data {
            Some(symbols) => Ok(symbols),
            None => Err(BinanceError::RestError(RestErrorKind::ResponseError(
                "No symbols data in response".to_string(),
            ))),
        }
    }

    /// 获取指定交易对的最新行情（带重试）
    pub async fn get_ticker(&self, symbol: &str) -> Result<serde_json::Value, BinanceError> {
        let path = format!("/api/v3/ticker/24hr?symbol={}", symbol.to_uppercase());
        self.request_with_retry(reqwest::Method::GET, &path, None, 3).await
    }

    /// 获取指定交易对的最新深度数据（带重试）
    pub async fn get_depth(
        &self,
        symbol: &str,
        limit: u32,
    ) -> Result<serde_json::Value, BinanceError> {
        let path = format!(
            "/api/v3/depth?symbol={}&limit={}",
            symbol.to_uppercase(),
            limit
        );
        self.request_with_retry(reqwest::Method::GET, &path, None, 3).await
    }

    /// 获取指定交易对的最新成交记录（带重试）
    pub async fn get_trades(&self, symbol: &str) -> Result<serde_json::Value, BinanceError> {
        let path = format!("/api/v3/trades?symbol={}&limit=20", symbol.to_uppercase());
        self.request_with_retry(reqwest::Method::GET, &path, None, 3).await
    }

    /// 获取指定交易对的K线数据（带重试）
    pub async fn get_klines(
        &self,
        symbol: &str,
        interval: &str,
        limit: u32,
    ) -> Result<serde_json::Value, BinanceError> {
        let path = format!(
            "/api/v3/klines?symbol={}&interval={}&limit={}",
            symbol.to_uppercase(),
            interval,
            limit
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
        RestClient::new("https://api.binance.com", None, None)
    }

    #[tokio::test]
    async fn test_get_symbols() {
        let client = create_test_client().await;
        let result = timeout(TEST_TIMEOUT, client.get_symbols()).await;
        
        match result {
            Ok(Ok(symbols)) => {
                assert!(!symbols.is_empty(), "应该返回至少一个交易对");
                let btc_symbol = symbols.iter().find(|s| s.symbol == "BTCUSDT");
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
        let result = timeout(TEST_TIMEOUT, client.get_ticker("BTCUSDT")).await;
        
        match result {
            Ok(Ok(ticker)) => {
                assert!(ticker.get("symbol").is_some(), "应该包含交易对字段");
                assert!(ticker.get("lastPrice").is_some(), "应该包含最新价格字段");
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
        let result = timeout(TEST_TIMEOUT, client.get_depth("BTCUSDT", 20)).await;
        
        match result {
            Ok(Ok(depth)) => {
                assert!(depth.get("lastUpdateId").is_some(), "应该包含更新ID字段");
                assert!(depth.get("bids").is_some(), "应该包含买单数据");
                assert!(depth.get("asks").is_some(), "应该包含卖单数据");
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
        let result = timeout(TEST_TIMEOUT, client.get_trades("BTCUSDT")).await;
        
        match result {
            Ok(Ok(trades)) => {
                assert!(trades.as_array().is_some(), "应该返回数组格式的成交记录");
                if let Some(trades_array) = trades.as_array() {
                    assert!(!trades_array.is_empty(), "应该包含至少一条成交记录");
                }
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
        let result = timeout(TEST_TIMEOUT, client.get_klines("BTCUSDT", "1m", 10)).await;
        
        match result {
            Ok(Ok(klines)) => {
                assert!(klines.as_array().is_some(), "应该返回数组格式的K线数据");
                if let Some(klines_array) = klines.as_array() {
                    assert!(!klines_array.is_empty(), "应该包含至少一条K线数据");
                }
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