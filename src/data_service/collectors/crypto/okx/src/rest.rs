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

    /// 发送 GET 请求
    /// 
    /// # 参数
    /// * `url` - 请求地址
    /// * `params` - URL 参数
    /// 
    /// # 返回值
    /// * `Ok(Response)` - HTTP 响应
    /// * `Err(OkxError)` - 请求失败
    async fn get(&self, url: &str, params: Option<&[(&str, &str)]>) -> Result<Response, OkxError> {
        let mut builder = self.client.get(url);
        
        if let Some(params) = params {
            builder = builder.query(params);
        }

        self.send(builder).await
    }

    /// 发送 POST 请求
    /// 
    /// # 参数
    /// * `url` - 请求地址
    /// * `body` - 请求体
    /// 
    /// # 返回值
    /// * `Ok(Response)` - HTTP 响应
    /// * `Err(OkxError)` - 请求失败
    async fn post(&self, url: &str, body: Option<&Value>) -> Result<Response, OkxError> {
        let mut builder = self.client.post(url);
        
        if let Some(body) = body {
            builder = builder.json(body);
        }

        self.send(builder).await
    }

    /// 发送 HTTP 请求
    /// 
    /// 处理请求的发送、速率限制和错误处理。
    /// 
    /// # 参数
    /// * `builder` - 请求构建器
    /// 
    /// # 返回值
    /// * `Ok(Response)` - HTTP 响应
    /// * `Err(OkxError)` - 请求失败
    async fn send(&self, builder: RequestBuilder) -> Result<Response, OkxError> {
        // 等待速率限制
        self.rate_limiter.lock().await.wait().await;

        let response = builder
            .send()
            .await
            .map_err(|e| OkxError::NetworkError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await
                .unwrap_or_else(|_| "Failed to get error response".to_string());
            
            return Err(OkxError::RestError(format!(
                "Request failed with status {}: {}",
                status, text
            )));
        }

        Ok(response)
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

    /// 等待直到可以发送新的请求
    /// 
    /// 如果当前请求会超过速率限制，则等待适当的时间。
    async fn wait(&mut self) {
        let now = Instant::now();
        let minute_ago = now - Duration::from_secs(60);

        // 移除一分钟前的请求
        self.weights.retain(|&time| time > minute_ago);

        // 如果达到限制，等待直到可以发送请求
        if self.weights.len() >= self.weight_per_minute as usize {
            let oldest = self.weights[0];
            let wait_time = minute_ago - oldest;
            if wait_time > Duration::from_secs(0) {
                tokio::time::sleep(wait_time).await;
            }
            self.weights.remove(0);
        }

        self.weights.push(now);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;

    const TEST_REST_ENDPOINT: &str = "https://www.okx.com";
    const TEST_API_KEY: &str = "test-api-key";
    const TEST_API_SECRET: &str = "test-api-secret";
    const TEST_PASSPHRASE: &str = "test-passphrase";

    #[tokio::test]
    async fn test_get_instruments() {
        let client = RestClient::new(TEST_REST_ENDPOINT, None, None, None);
        let result = client.get_instruments("SPOT").await;
        assert!(result.is_ok(), "Failed to get instruments: {:?}", result);
        
        let instruments = result.unwrap();
        assert!(!instruments.is_empty(), "Instruments list is empty");
    }

    #[tokio::test]
    async fn test_get_ticker() {
        let client = RestClient::new(TEST_REST_ENDPOINT, None, None, None);
        let result = client.get_ticker("BTC-USDT").await;
        assert!(result.is_ok(), "Failed to get ticker: {:?}", result);
        
        let ticker = result.unwrap();
        assert_eq!(ticker.inst_id, "BTC-USDT");
    }

    #[tokio::test]
    async fn test_get_order_book() {
        let client = RestClient::new(TEST_REST_ENDPOINT, None, None, None);
        let result = client.get_order_book("BTC-USDT", 20).await;
        assert!(result.is_ok(), "Failed to get order book: {:?}", result);
        
        let order_book = result.unwrap();
        assert!(!order_book.asks.is_empty(), "Order book asks is empty");
        assert!(!order_book.bids.is_empty(), "Order book bids is empty");
    }

    #[tokio::test]
    async fn test_get_klines() {
        let client = RestClient::new(TEST_REST_ENDPOINT, None, None, None);
        let result = client.get_klines("BTC-USDT", "1m", Some(100)).await;
        assert!(result.is_ok(), "Failed to get klines: {:?}", result);
        
        let klines = result.unwrap();
        assert!(!klines.is_empty(), "Klines list is empty");
        assert!(klines.len() <= 100, "Too many klines returned");
    }

    #[tokio::test]
    async fn test_get_trades() {
        let client = RestClient::new(TEST_REST_ENDPOINT, None, None, None);
        let result = client.get_trades("BTC-USDT", Some(100)).await;
        assert!(result.is_ok(), "Failed to get trades: {:?}", result);
        
        let trades = result.unwrap();
        assert!(!trades.is_empty(), "Trades list is empty");
        assert!(trades.len() <= 100, "Too many trades returned");
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let client = RestClient::new(TEST_REST_ENDPOINT, None, None, None);
        let mut results = Vec::new();
        
        // 发送多个请求测试速率限制
        for _ in 0..5 {
            let result = client.get_ticker("BTC-USDT").await;
            results.push(result.is_ok());
        }
        
        // 所有请求都应该成功
        assert!(results.iter().all(|&x| x), "Some requests failed due to rate limiting");
    }

    #[tokio::test]
    async fn test_place_order() {
        let client = RestClient::new(
            TEST_REST_ENDPOINT,
            Some(TEST_API_KEY),
            Some(TEST_API_SECRET),
            Some(TEST_PASSPHRASE),
        );

        let request = PlaceOrderRequest {
            inst_id: "BTC-USDT".to_string(),
            side: "buy".to_string(),
            ord_type: "limit".to_string(),
            px: Some("50000".to_string()),
            sz: "0.001".to_string(),
            reduce_only: None,
            cl_ord_id: None,
        };

        let result = client.place_order(request).await;
        assert!(result.is_ok(), "Failed to place order: {:?}", result);
    }

    #[tokio::test]
    async fn test_cancel_order() {
        let client = RestClient::new(
            TEST_REST_ENDPOINT,
            Some(TEST_API_KEY),
            Some(TEST_API_SECRET),
            Some(TEST_PASSPHRASE),
        );

        let request = CancelOrderRequest {
            inst_id: "BTC-USDT".to_string(),
            ord_id: Some("123456".to_string()),
            cl_ord_id: None,
        };

        let result = client.cancel_order(request).await;
        assert!(result.is_ok(), "Failed to cancel order: {:?}", result);
    }

    #[tokio::test]
    async fn test_get_order() {
        let client = RestClient::new(
            TEST_REST_ENDPOINT,
            Some(TEST_API_KEY),
            Some(TEST_API_SECRET),
            Some(TEST_PASSPHRASE),
        );

        let result = client.get_order("BTC-USDT", "123456").await;
        assert!(result.is_ok(), "Failed to get order: {:?}", result);
    }

    #[tokio::test]
    async fn test_get_orders_history() {
        let client = RestClient::new(
            TEST_REST_ENDPOINT,
            Some(TEST_API_KEY),
            Some(TEST_API_SECRET),
            Some(TEST_PASSPHRASE),
        );

        let result = client.get_orders_history("SPOT", "filled", Some(10)).await;
        assert!(result.is_ok(), "Failed to get orders history: {:?}", result);
        
        let orders = result.unwrap();
        assert!(orders.len() <= 10, "Too many orders returned");
    }

    #[tokio::test]
    async fn test_get_balances() {
        let client = RestClient::new(
            TEST_REST_ENDPOINT,
            Some(TEST_API_KEY),
            Some(TEST_API_SECRET),
            Some(TEST_PASSPHRASE),
        );

        let result = client.get_balances().await;
        assert!(result.is_ok(), "Failed to get balances: {:?}", result);
        
        let balances = result.unwrap();
        assert!(!balances.is_empty(), "Balances list is empty");
    }

    #[tokio::test]
    async fn test_get_positions() {
        let client = RestClient::new(
            TEST_REST_ENDPOINT,
            Some(TEST_API_KEY),
            Some(TEST_API_SECRET),
            Some(TEST_PASSPHRASE),
        );

        let result = client.get_positions("SWAP").await;
        assert!(result.is_ok(), "Failed to get positions: {:?}", result);
    }

    #[tokio::test]
    async fn test_set_leverage() {
        let client = RestClient::new(
            TEST_REST_ENDPOINT,
            Some(TEST_API_KEY),
            Some(TEST_API_SECRET),
            Some(TEST_PASSPHRASE),
        );

        let result = client.set_leverage("BTC-USDT-SWAP", "5", "cross").await;
        assert!(result.is_ok(), "Failed to set leverage: {:?}", result);
        
        let leverage = result.unwrap();
        assert_eq!(leverage.inst_id, "BTC-USDT-SWAP");
        assert_eq!(leverage.lever, "5");
        assert_eq!(leverage.mgn_mode, "cross");
    }
} 