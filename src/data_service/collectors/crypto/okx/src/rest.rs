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
use base64;

use common::{MarketData, DataQuality, MarketDataType, CollectorError};
use crate::error::OkxError;
use crate::models::*;

type HmacSha256 = Hmac<Sha256>;

const DEFAULT_RECV_WINDOW: u64 = 5000;
const DEFAULT_WEIGHT_PER_MINUTE: u32 = 1200;

/// REST客户端配置
#[derive(Debug, Clone)]
pub struct RestClientConfig {
    pub endpoint: String,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub passphrase: Option<String>,
    pub recv_window: u64,
}

impl Default for RestClientConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://www.okx.com".to_string(),
            api_key: None,
            api_secret: None,
            passphrase: None,
            recv_window: DEFAULT_RECV_WINDOW,
        }
    }
}

/// REST API 客户端
/// 
/// 负责与 OKX REST API 服务器通信，提供市场数据查询功能。
/// 包含速率限制管理和自动重试机制。
/// 
/// # 示例
/// ```
/// use okx::RestClient;
/// 
/// #[tokio::main]
/// async fn main() {
///     let client = RestClient::new(
///         "https://www.okx.com",
///         Some("api-key"),
///         Some("api-secret"),
///         Some("passphrase")
///     );
///     
///     // 获取所有现货交易对信息
///     let instruments = client.get_instruments("SPOT").await.unwrap();
///     println!("Instruments: {:?}", instruments);
///     
///     // 获取 BTC-USDT 的行情数据
///     let ticker = client.get_ticker("BTC-USDT").await.unwrap();
///     println!("Ticker: {:?}", ticker);
/// }
/// ```
pub struct RestClient {
    config: RestClientConfig,
    client: Client,
    rate_limiter: Arc<Mutex<RateLimiter>>,
}

impl RestClient {
    /// 创建新的 REST 客户端实例
    /// 
    /// # 参数
    /// * `endpoint` - REST API 服务器地址
    /// * `api_key` - API Key（可选）
    /// * `api_secret` - API Secret（可选）
    /// * `passphrase` - API Passphrase（可选）
    pub fn new(
        endpoint: &str,
        api_key: Option<&str>,
        api_secret: Option<&str>,
        passphrase: Option<&str>,
    ) -> Self {
        let config = RestClientConfig {
            endpoint: endpoint.to_string(),
            api_key: api_key.map(String::from),
            api_secret: api_secret.map(String::from),
            passphrase: passphrase.map(String::from),
            recv_window: DEFAULT_RECV_WINDOW,
        };

        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            client,
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(DEFAULT_WEIGHT_PER_MINUTE))),
        }
    }

    /// 获取交易对信息
    /// 
    /// # 参数
    /// * `inst_type` - 产品类型，如 "SPOT", "SWAP", "FUTURES" 等
    /// 
    /// # 返回值
    /// * `Ok(Vec<InstrumentInfo>)` - 交易对信息列表
    /// * `Err(OkxError)` - 请求失败
    pub async fn get_instruments(&self, inst_type: &str) -> Result<Vec<InstrumentInfo>, OkxError> {
        let endpoint = format!("{}/api/v5/public/instruments", self.config.endpoint);
        let response = self.get(&endpoint, Some(&[("instType", inst_type)])).await?;
        let data: Value = response.json().await
            .map_err(|e| OkxError::ParseError(format!("Failed to parse response: {}", e)))?;
        
        let instruments: Vec<InstrumentInfo> = serde_json::from_value(data["data"].clone())
            .map_err(|e| OkxError::ParseError(format!("Failed to parse instruments: {}", e)))?;

        Ok(instruments)
    }

    /// 获取单个交易对的行情数据
    /// 
    /// # 参数
    /// * `inst_id` - 交易对名称，如 "BTC-USDT"
    /// 
    /// # 返回值
    /// * `Ok(Ticker)` - 行情数据
    /// * `Err(OkxError)` - 请求失败
    pub async fn get_ticker(&self, inst_id: &str) -> Result<Ticker, OkxError> {
        let endpoint = format!("{}/api/v5/market/ticker", self.config.endpoint);
        let response = self.get(&endpoint, Some(&[("instId", inst_id)])).await?;
        let data: Value = response.json().await
            .map_err(|e| OkxError::ParseError(format!("Failed to parse response: {}", e)))?;

        let ticker: Ticker = serde_json::from_value(data["data"][0].clone())
            .map_err(|e| OkxError::ParseError(format!("Failed to parse ticker: {}", e)))?;

        Ok(ticker)
    }

    /// 获取交易对的深度数据
    /// 
    /// # 参数
    /// * `inst_id` - 交易对名称
    /// * `size` - 深度档数，最大值为 400
    /// 
    /// # 返回值
    /// * `Ok(OrderBook)` - 深度数据
    /// * `Err(OkxError)` - 请求失败
    pub async fn get_order_book(&self, inst_id: &str, size: u32) -> Result<OrderBook, OkxError> {
        let endpoint = format!("{}/api/v5/market/books", self.config.endpoint);
        let response = self.get(&endpoint, Some(&[
            ("instId", inst_id),
            ("sz", &size.to_string()),
        ])).await?;
        
        let data: Value = response.json().await
            .map_err(|e| OkxError::ParseError(format!("Failed to parse response: {}", e)))?;

        let order_book: OrderBook = serde_json::from_value(data["data"][0].clone())
            .map_err(|e| OkxError::ParseError(format!("Failed to parse order book: {}", e)))?;

        Ok(order_book)
    }

    /// 获取K线数据
    /// 
    /// # 参数
    /// * `inst_id` - 交易对名称
    /// * `bar` - K线周期，如 "1m", "5m", "1H", "1D" 等
    /// * `limit` - 返回的数据条数，默认 100，最大 100
    /// 
    /// # 返回值
    /// * `Ok(Vec<Kline>)` - K线数据列表
    /// * `Err(OkxError)` - 请求失败
    pub async fn get_klines(
        &self,
        inst_id: &str,
        bar: &str,
        limit: Option<u32>,
    ) -> Result<Vec<Kline>, OkxError> {
        let endpoint = format!("{}/api/v5/market/candles", self.config.endpoint);
        let mut params = vec![
            ("instId", inst_id),
            ("bar", bar),
        ];
        
        if let Some(limit) = limit {
            params.push(("limit", &limit.to_string()));
        }

        let response = self.get(&endpoint, Some(&params)).await?;
        let data: Value = response.json().await
            .map_err(|e| OkxError::ParseError(format!("Failed to parse response: {}", e)))?;

        let klines: Vec<Kline> = serde_json::from_value(data["data"].clone())
            .map_err(|e| OkxError::ParseError(format!("Failed to parse klines: {}", e)))?;

        Ok(klines)
    }

    /// 获取最近的成交数据
    /// 
    /// # 参数
    /// * `inst_id` - 交易对名称
    /// * `limit` - 返回的数据条数，默认 100，最大 100
    /// 
    /// # 返回值
    /// * `Ok(Vec<Trade>)` - 成交数据列表
    /// * `Err(OkxError)` - 请求失败
    pub async fn get_trades(
        &self,
        inst_id: &str,
        limit: Option<u32>,
    ) -> Result<Vec<Trade>, OkxError> {
        let endpoint = format!("{}/api/v5/market/trades", self.config.endpoint);
        let mut params = vec![("instId", inst_id)];
        
        if let Some(limit) = limit {
            params.push(("limit", &limit.to_string()));
        }

        let response = self.get(&endpoint, Some(&params)).await?;
        let data: Value = response.json().await
            .map_err(|e| OkxError::ParseError(format!("Failed to parse response: {}", e)))?;

        let trades: Vec<Trade> = serde_json::from_value(data["data"].clone())
            .map_err(|e| OkxError::ParseError(format!("Failed to parse trades: {}", e)))?;

        Ok(trades)
    }

    /// 获取账户余额
    /// 
    /// # 返回值
    /// * `Ok(Vec<Balance>)` - 账户余额列表
    /// * `Err(OkxError)` - 请求失败
    pub async fn get_balances(&self) -> Result<Vec<Balance>, OkxError> {
        let endpoint = format!("{}/api/v5/account/balance", self.config.endpoint);
        let response = self.get_with_auth(&endpoint, None).await?;
        
        let data: Value = response.json().await
            .map_err(|e| OkxError::ParseError(format!("Failed to parse response: {}", e)))?;

        let balances: Vec<Balance> = serde_json::from_value(data["data"].clone())
            .map_err(|e| OkxError::ParseError(format!("Failed to parse balances: {}", e)))?;

        Ok(balances)
    }

    /// 获取持仓信息
    /// 
    /// # 参数
    /// * `inst_type` - 产品类型，如 "SPOT", "MARGIN", "SWAP", "FUTURES"
    /// 
    /// # 返回值
    /// * `Ok(Vec<Position>)` - 持仓信息列表
    /// * `Err(OkxError)` - 请求失败
    pub async fn get_positions(&self, inst_type: &str) -> Result<Vec<Position>, OkxError> {
        let endpoint = format!("{}/api/v5/account/positions", self.config.endpoint);
        let response = self.get_with_auth(&endpoint, Some(&[("instType", inst_type)])).await?;
        
        let data: Value = response.json().await
            .map_err(|e| OkxError::ParseError(format!("Failed to parse response: {}", e)))?;

        let positions: Vec<Position> = serde_json::from_value(data["data"].clone())
            .map_err(|e| OkxError::ParseError(format!("Failed to parse positions: {}", e)))?;

        Ok(positions)
    }

    /// 设置杠杆倍数
    /// 
    /// # 参数
    /// * `inst_id` - 产品ID
    /// * `lever` - 杠杆倍数
    /// * `mgn_mode` - 保证金模式 cross/isolated
    /// 
    /// # 返回值
    /// * `Ok(LeverageInfo)` - 杠杆配置信息
    /// * `Err(OkxError)` - 设置失败
    pub async fn set_leverage(
        &self,
        inst_id: &str,
        lever: &str,
        mgn_mode: &str,
    ) -> Result<LeverageInfo, OkxError> {
        let endpoint = format!("{}/api/v5/account/set-leverage", self.config.endpoint);
        let body = json!({
            "instId": inst_id,
            "lever": lever,
            "mgnMode": mgn_mode,
        });

        let response = self.post_with_auth(&endpoint, Some(&body)).await?;
        let data: Value = response.json().await
            .map_err(|e| OkxError::ParseError(format!("Failed to parse response: {}", e)))?;

        let leverage: LeverageInfo = serde_json::from_value(data["data"][0].clone())
            .map_err(|e| OkxError::ParseError(format!("Failed to parse leverage info: {}", e)))?;

        Ok(leverage)
    }

    /// 下单
    /// 
    /// # 参数
    /// * `request` - 下单请求参数
    /// 
    /// # 返回值
    /// * `Ok(Order)` - 订单信息
    /// * `Err(OkxError)` - 下单失败
    pub async fn place_order(&self, request: PlaceOrderRequest) -> Result<Order, OkxError> {
        let endpoint = format!("{}/api/v5/trade/order", self.config.endpoint);
        let body = serde_json::to_value(request)
            .map_err(|e| OkxError::ParseError(format!("Failed to serialize request: {}", e)))?;
        
        let response = self.post_with_auth(&endpoint, Some(&body)).await?;
        let data: Value = response.json().await
            .map_err(|e| OkxError::ParseError(format!("Failed to parse response: {}", e)))?;

        let order: Order = serde_json::from_value(data["data"][0].clone())
            .map_err(|e| OkxError::ParseError(format!("Failed to parse order: {}", e)))?;

        Ok(order)
    }

    /// 批量下单
    /// 
    /// # 参数
    /// * `request` - 批量下单请求参数
    /// 
    /// # 返回值
    /// * `Ok(Vec<Order>)` - 订单信息列表
    /// * `Err(OkxError)` - 下单失败
    pub async fn place_multiple_orders(&self, request: BatchPlaceOrderRequest) -> Result<Vec<Order>, OkxError> {
        let endpoint = format!("{}/api/v5/trade/batch-orders", self.config.endpoint);
        let body = serde_json::to_value(request)
            .map_err(|e| OkxError::ParseError(format!("Failed to serialize request: {}", e)))?;
        
        let response = self.post_with_auth(&endpoint, Some(&body)).await?;
        let data: Value = response.json().await
            .map_err(|e| OkxError::ParseError(format!("Failed to parse response: {}", e)))?;

        let orders: Vec<Order> = serde_json::from_value(data["data"].clone())
            .map_err(|e| OkxError::ParseError(format!("Failed to parse orders: {}", e)))?;

        Ok(orders)
    }

    /// 撤单
    /// 
    /// # 参数
    /// * `request` - 撤单请求参数
    /// 
    /// # 返回值
    /// * `Ok(Order)` - 订单信息
    /// * `Err(OkxError)` - 撤单失败
    pub async fn cancel_order(&self, request: CancelOrderRequest) -> Result<Order, OkxError> {
        let endpoint = format!("{}/api/v5/trade/cancel-order", self.config.endpoint);
        let body = serde_json::to_value(request)
            .map_err(|e| OkxError::ParseError(format!("Failed to serialize request: {}", e)))?;
        
        let response = self.post_with_auth(&endpoint, Some(&body)).await?;
        let data: Value = response.json().await
            .map_err(|e| OkxError::ParseError(format!("Failed to parse response: {}", e)))?;

        let order: Order = serde_json::from_value(data["data"][0].clone())
            .map_err(|e| OkxError::ParseError(format!("Failed to parse order: {}", e)))?;

        Ok(order)
    }

    /// 批量撤单
    /// 
    /// # 参数
    /// * `request` - 批量撤单请求参数
    /// 
    /// # 返回值
    /// * `Ok(Vec<Order>)` - 订单信息列表
    /// * `Err(OkxError)` - 撤单失败
    pub async fn cancel_multiple_orders(&self, request: BatchCancelOrderRequest) -> Result<Vec<Order>, OkxError> {
        let endpoint = format!("{}/api/v5/trade/cancel-batch-orders", self.config.endpoint);
        let body = serde_json::to_value(request)
            .map_err(|e| OkxError::ParseError(format!("Failed to serialize request: {}", e)))?;
        
        let response = self.post_with_auth(&endpoint, Some(&body)).await?;
        let data: Value = response.json().await
            .map_err(|e| OkxError::ParseError(format!("Failed to parse response: {}", e)))?;

        let orders: Vec<Order> = serde_json::from_value(data["data"].clone())
            .map_err(|e| OkxError::ParseError(format!("Failed to parse orders: {}", e)))?;

        Ok(orders)
    }

    /// 获取订单信息
    /// 
    /// # 参数
    /// * `inst_id` - 产品ID
    /// * `ord_id` - 订单ID
    /// 
    /// # 返回值
    /// * `Ok(Order)` - 订单信息
    /// * `Err(OkxError)` - 请求失败
    pub async fn get_order(&self, inst_id: &str, ord_id: &str) -> Result<Order, OkxError> {
        let endpoint = format!("{}/api/v5/trade/order", self.config.endpoint);
        let response = self.get_with_auth(&endpoint, Some(&[
            ("instId", inst_id),
            ("ordId", ord_id),
        ])).await?;
        
        let data: Value = response.json().await
            .map_err(|e| OkxError::ParseError(format!("Failed to parse response: {}", e)))?;

        let order: Order = serde_json::from_value(data["data"][0].clone())
            .map_err(|e| OkxError::ParseError(format!("Failed to parse order: {}", e)))?;

        Ok(order)
    }

    /// 获取历史订单列表
    /// 
    /// # 参数
    /// * `inst_type` - 产品类型
    /// * `state` - 订单状态
    /// * `limit` - 返回结果数量，默认100
    /// 
    /// # 返回值
    /// * `Ok(Vec<Order>)` - 订单列表
    /// * `Err(OkxError)` - 请求失败
    pub async fn get_orders_history(
        &self,
        inst_type: &str,
        state: &str,
        limit: Option<u32>,
    ) -> Result<Vec<Order>, OkxError> {
        let endpoint = format!("{}/api/v5/trade/orders-history", self.config.endpoint);
        let mut params = vec![
            ("instType", inst_type),
            ("state", state),
        ];
        
        if let Some(limit) = limit {
            params.push(("limit", &limit.to_string()));
        }

        let response = self.get_with_auth(&endpoint, Some(&params)).await?;
        let data: Value = response.json().await
            .map_err(|e| OkxError::ParseError(format!("Failed to parse response: {}", e)))?;

        let orders: Vec<Order> = serde_json::from_value(data["data"].clone())
            .map_err(|e| OkxError::ParseError(format!("Failed to parse orders: {}", e)))?;

        Ok(orders)
    }

    /// 生成签名
    fn sign(&self, timestamp: &str, method: &str, path: &str, body: &str) -> Result<String, OkxError> {
        if let Some(secret) = &self.config.api_secret {
            let sign_content = format!("{}{}{}{}", timestamp, method, path, body);
            let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
                .map_err(|e| OkxError::RestError(RestErrorKind::AuthenticationError(e.to_string())))?;
            mac.update(sign_content.as_bytes());
            let result = mac.finalize();
            Ok(base64::encode(result.into_bytes()))
        } else {
            Err(OkxError::RestError(RestErrorKind::AuthenticationError(
                "API secret not configured".to_string(),
            )))
        }
    }

    /// 添加认证头
    fn add_auth_headers(&self, builder: RequestBuilder, method: &str, path: &str, body: &str) -> Result<RequestBuilder, OkxError> {
        let timestamp = Utc::now().timestamp_millis().to_string();
        let signature = self.sign(&timestamp, method, path, body)?;

        let builder = builder
            .header("OK-ACCESS-KEY", self.config.api_key.as_ref().ok_or_else(|| {
                OkxError::RestError(RestErrorKind::AuthenticationError(
                    "API key not configured".to_string(),
                ))
            })?)
            .header("OK-ACCESS-SIGN", signature)
            .header("OK-ACCESS-TIMESTAMP", timestamp)
            .header("OK-ACCESS-PASSPHRASE", self.config.passphrase.as_ref().ok_or_else(|| {
                OkxError::RestError(RestErrorKind::AuthenticationError(
                    "API passphrase not configured".to_string(),
                ))
            })?);

        Ok(builder)
    }

    /// 发送GET请求（带认证）
    async fn get_with_auth<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: Option<&[(&str, &str)]>,
    ) -> Result<T, OkxError> {
        let url = if let Some(params) = params {
            let query = params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            format!("{}?{}", endpoint, query)
        } else {
            endpoint.to_string()
        };

        let mut builder = self.client.get(&url);
        builder = self.add_auth_headers(builder, "GET", &url, "")?;

        self.send_request(builder).await
    }

    /// 发送POST请求（带认证）
    async fn post_with_auth<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &str,
    ) -> Result<T, OkxError> {
        let mut builder = self.client.post(endpoint).body(body.to_string());
        builder = self.add_auth_headers(builder, "POST", endpoint, body)?;

        self.send_request(builder).await
    }

    /// 发送请求并处理响应（带速率限制）
    async fn send_request<T: DeserializeOwned>(&self, builder: RequestBuilder) -> Result<T, OkxError> {
        // 等待速率限制
        self.rate_limiter.lock().await.wait().await?;

        let response = builder
            .send()
            .await
            .map_err(|e| OkxError::RestError(RestErrorKind::RequestError(e.to_string())))?;

        let status = response.status();
        if !status.is_success() {
            // 处理速率限制错误
            if status.as_u16() == 429 {
                return Err(OkxError::RestError(RestErrorKind::RateLimitError(
                    "Rate limit exceeded".to_string(),
                )));
            }

            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to get error response".to_string());
            
            return Err(OkxError::RestError(RestErrorKind::ResponseError(format!(
                "HTTP error {}: {}",
                status,
                error_text
            ))));
        }

        let text = response
            .text()
            .await
            .map_err(|e| OkxError::RestError(RestErrorKind::ResponseError(e.to_string())))?;

        serde_json::from_str(&text)
            .map_err(|e| OkxError::ParseError(ParseErrorKind::JsonError(e.to_string())))
    }

    /// 发送请求并自动重试
    async fn send_request_with_retry<T: DeserializeOwned>(
        &self,
        builder: RequestBuilder,
        max_retries: u32,
    ) -> Result<T, OkxError> {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < max_retries {
            match self.send_request(builder.try_clone().ok_or_else(|| {
                OkxError::RestError(RestErrorKind::RequestError(
                    "Failed to clone request".to_string(),
                ))
            })?).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    match &e {
                        OkxError::RestError(RestErrorKind::RateLimitError(_)) => {
                            // 速率限制错误，等待更长时间
                            tokio::time::sleep(Duration::from_secs(2_u64.pow(attempts))).await;
                        }
                        _ => {
                            // 其他错误，等待较短时间
                            tokio::time::sleep(Duration::from_millis(100 * 2_u64.pow(attempts))).await;
                        }
                    }
                    last_error = Some(e);
                    attempts += 1;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            OkxError::RestError(RestErrorKind::RequestError(
                "Max retries exceeded".to_string(),
            ))
        }))
    }

    /// 发送GET请求（不带认证）
    async fn get<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: Option<&[(&str, &str)]>,
    ) -> Result<T, OkxError> {
        let url = if let Some(params) = params {
            let query = params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            format!("{}?{}", endpoint, query)
        } else {
            endpoint.to_string()
        };

        let builder = self.client.get(&url);
        self.send_request(builder).await
    }
}

/// 速率限制器
/// 
/// 用于控制 API 请求的频率，防止超过交易所的限制。
/// 使用令牌桶算法实现。
#[derive(Debug)]
struct RateLimiter {
    /// 每分钟允许的请求权重
    weight_per_minute: u32,
    /// 已使用的请求时间戳列表
    weights: Vec<Instant>,
}

impl RateLimiter {
    /// 创建新的速率限制器
    /// 
    /// # 参数
    /// * `weight_per_minute` - 每分钟允许的请求权重
    fn new(weight_per_minute: u32) -> Self {
        Self {
            weight_per_minute,
            weights: Vec::new(),
        }
    }

    /// 检查是否超过速率限制
    fn check_rate_limit(&mut self) -> bool {
        let now = Instant::now();
        self.weights.retain(|&t| now.duration_since(t) < Duration::from_secs(60));
        self.weights.len() as u32 <= self.weight_per_minute
    }

    /// 添加权重
    fn add_weight(&mut self, weight: u32) {
        let now = Instant::now();
        for _ in 0..weight {
            self.weights.push(now);
        }
    }

    /// 等待直到可以发送请求
    async fn wait(&mut self) -> Result<(), OkxError> {
        let mut attempts = 0;
        while !self.check_rate_limit() {
            if attempts >= 10 {
                return Err(OkxError::RestError(RestErrorKind::RateLimitError(
                    "Rate limit exceeded, too many attempts".to_string(),
                )));
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
            attempts += 1;
        }
        self.add_weight(1);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;
    use std::time::Duration;

    const TEST_API_KEY: &str = "test-api-key";
    const TEST_API_SECRET: &str = "test-api-secret";
    const TEST_PASSPHRASE: &str = "test-passphrase";
    const TEST_ENDPOINT: &str = "https://www.okx.com";

    fn create_test_client() -> RestClient {
        RestClient::new(
            TEST_ENDPOINT,
            Some(TEST_API_KEY),
            Some(TEST_API_SECRET),
            Some(TEST_PASSPHRASE),
        )
    }

    #[tokio::test]
    async fn test_get_instruments() {
        let client = create_test_client();
        let result = timeout(
            Duration::from_secs(10),
            client.get_instruments("SPOT"),
        ).await;

        match result {
            Ok(Ok(instruments)) => {
                assert!(!instruments.is_empty(), "Should return at least one instrument");
                let btc_usdt = instruments.iter().find(|i| i.inst_id == "BTC-USDT");
                assert!(btc_usdt.is_some(), "Should contain BTC-USDT instrument");
            }
            Ok(Err(e)) => panic!("Failed to get instruments: {:?}", e),
            Err(_) => panic!("Request timed out"),
        }
    }

    #[tokio::test]
    async fn test_get_ticker() {
        let client = create_test_client();
        let result = timeout(
            Duration::from_secs(10),
            client.get_ticker("BTC-USDT"),
        ).await;

        match result {
            Ok(Ok(ticker)) => {
                assert_eq!(ticker.inst_id, "BTC-USDT");
                assert!(ticker.last > Decimal::zero());
                assert!(ticker.vol_24h > Decimal::zero());
            }
            Ok(Err(e)) => panic!("Failed to get ticker: {:?}", e),
            Err(_) => panic!("Request timed out"),
        }
    }

    #[tokio::test]
    async fn test_get_order_book() {
        let client = create_test_client();
        let result = timeout(
            Duration::from_secs(10),
            client.get_order_book("BTC-USDT", 20),
        ).await;

        match result {
            Ok(Ok(order_book)) => {
                assert_eq!(order_book.inst_id, "BTC-USDT");
                assert!(!order_book.asks.is_empty());
                assert!(!order_book.bids.is_empty());
                assert!(order_book.asks.len() <= 20);
                assert!(order_book.bids.len() <= 20);
            }
            Ok(Err(e)) => panic!("Failed to get order book: {:?}", e),
            Err(_) => panic!("Request timed out"),
        }
    }

    #[tokio::test]
    async fn test_get_trades() {
        let client = create_test_client();
        let result = timeout(
            Duration::from_secs(10),
            client.get_trades("BTC-USDT", Some(10)),
        ).await;

        match result {
            Ok(Ok(trades)) => {
                assert!(!trades.is_empty());
                assert!(trades.len() <= 10);
                let trade = &trades[0];
                assert_eq!(trade.inst_id, "BTC-USDT");
                assert!(trade.px > Decimal::zero());
                assert!(trade.sz > Decimal::zero());
            }
            Ok(Err(e)) => panic!("Failed to get trades: {:?}", e),
            Err(_) => panic!("Request timed out"),
        }
    }

    #[tokio::test]
    async fn test_get_klines() {
        let client = create_test_client();
        let result = timeout(
            Duration::from_secs(10),
            client.get_klines("BTC-USDT", "1m", Some(10)),
        ).await;

        match result {
            Ok(Ok(klines)) => {
                assert!(!klines.is_empty());
                assert!(klines.len() <= 10);
                let kline = &klines[0];
                assert!(kline.open > Decimal::zero());
                assert!(kline.high >= kline.low);
                assert!(kline.vol > Decimal::zero());
            }
            Ok(Err(e)) => panic!("Failed to get klines: {:?}", e),
            Err(_) => panic!("Request timed out"),
        }
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(2);
        
        // 第一个请求应该成功
        assert!(limiter.check_rate_limit());
        limiter.add_weight(1);
        
        // 第二个请求应该成功
        assert!(limiter.check_rate_limit());
        limiter.add_weight(1);
        
        // 第三个请求应该失败
        assert!(!limiter.check_rate_limit());
        
        // 等待一分钟后应该可以继续请求
        tokio::time::sleep(Duration::from_secs(60)).await;
        assert!(limiter.check_rate_limit());
    }

    #[tokio::test]
    async fn test_authentication() {
        let client = create_test_client();
        let timestamp = "1234567890000";
        let method = "GET";
        let path = "/api/v5/account/balance";
        let body = "";

        let signature = client.sign(timestamp, method, path, body).unwrap();
        assert!(!signature.is_empty());
    }

    #[tokio::test]
    async fn test_request_retry() {
        let client = create_test_client();
        let builder = client.client.get("https://non-existent-url.com");
        
        let result = client.send_request_with_retry::<Value>(builder, 3).await;
        assert!(result.is_err());
        
        match result {
            Err(OkxError::RestError(RestErrorKind::RequestError(_))) => (),
            _ => panic!("Expected RequestError"),
        }
    }
} 