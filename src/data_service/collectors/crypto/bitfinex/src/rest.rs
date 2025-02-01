use crate::error::{BitfinexError, Result};
use crate::models::{ExchangeInfo, ExchangeStatus, Kline, Orderbook, Symbol, Ticker, Trade, TradeSide};
use crate::types::{ApiResponse, OrderbookInfo, SymbolInfo, TickerInfo, TradeInfo};

use async_trait::async_trait;
use hmac::{Hmac, Mac};
use reqwest::{Client, RequestBuilder, Method};
use sha2::Sha384;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use url::Url;
use serde::{de::DeserializeOwned, Serialize};
use hex;

const API_URL: &str = "https://api.bitfinex.com/v2";

/// REST API客户端配置
#[derive(Debug, Clone)]
pub struct BitfinexRestConfig {
    /// API密钥
    pub api_key: Option<String>,
    /// API密钥
    pub api_secret: Option<String>,
    /// 接收窗口时间(毫秒)
    pub recv_window: Option<u64>,
    /// 请求超时时间(秒)
    pub timeout: Option<Duration>,
}

impl Default for BitfinexRestConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            api_secret: None,
            recv_window: Some(5000),
            timeout: Some(Duration::from_secs(10)),
        }
    }
}

/// REST API客户端
#[derive(Debug, Clone)]
pub struct BitfinexRestClient {
    /// HTTP客户端
    client: Client,
    /// 配置信息
    config: BitfinexRestConfig,
}

impl BitfinexRestClient {
    /// 创建新的REST客户端
    pub fn new(config: BitfinexRestConfig) -> Result<Self, BitfinexError> {
        let client = Client::builder()
            .timeout(config.timeout.unwrap_or(Duration::from_secs(10)))
            .build()
            .map_err(|e| BitfinexError::HttpError(e.to_string()))?;

        Ok(Self { client, config })
    }

    /// 生成签名
    fn sign(&self, path: &str, nonce: &str, body: Option<&str>) -> Result<String, BitfinexError> {
        if let Some(secret) = &self.config.api_secret {
            let mut mac = Hmac::<Sha384>::new_from_slice(secret.as_bytes())
                .map_err(|e| BitfinexError::AuthenticationError(e.to_string()))?;

            let message = match body {
                Some(b) => format!("/api/v2{}{}{}",path, nonce, b),
                None => format!("/api/v2{}{}", path, nonce),
            };

            mac.update(message.as_bytes());
            let result = mac.finalize();
            Ok(hex::encode(result.into_bytes()))
        } else {
            Err(BitfinexError::AuthenticationError("API secret not configured".to_string()))
        }
    }

    /// 发送请求
    async fn send_request<T, R>(&self, method: Method, path: &str, body: Option<T>) -> Result<R, BitfinexError>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        let url = format!("{}{}", API_URL, path);
        let mut builder = self.client.request(method.clone(), &url);

        if let Some(b) = body {
            builder = builder.json(&b);
        }

        // 添加认证信息
        if method != Method::GET {
            if let Some(api_key) = &self.config.api_key {
                let nonce = chrono::Utc::now().timestamp_millis().to_string();
                let body_str = body.map(|b| serde_json::to_string(&b)
                    .map_err(|e| BitfinexError::JsonError(e.to_string())).unwrap());
                
                let signature = self.sign(path, &nonce, body_str.as_deref())?;
                
                builder = builder
                    .header("bfx-apikey", api_key)
                    .header("bfx-nonce", &nonce)
                    .header("bfx-signature", signature);
            }
        }

        let response = builder
            .send()
            .await
            .map_err(|e| BitfinexError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error = response.text().await
                .map_err(|e| BitfinexError::NetworkError(e.to_string()))?;
            return Err(BitfinexError::ApiError(error));
        }

        response.json::<R>()
            .await
            .map_err(|e| BitfinexError::JsonError(e.to_string()))
    }

    /// 获取所有交易对信息
    pub async fn get_symbols(&self) -> Result<Vec<Symbol>, BitfinexError> {
        let response: Vec<SymbolInfo> = self.send_request(Method::GET, "/conf/pub:info:pair", None).await?;
        Ok(response.into_iter().map(Into::into).collect())
    }

    /// 获取Ticker信息
    pub async fn get_ticker(&self, symbol: &str) -> Result<Ticker, BitfinexError> {
        let path = format!("/ticker/t{}", symbol.to_uppercase());
        let response: TickerInfo = self.send_request(Method::GET, &path, None).await?;
        Ok(response.into())
    }

    /// 获取订单簿
    pub async fn get_orderbook(&self, symbol: &str, depth: Option<u32>) -> Result<Orderbook, BitfinexError> {
        let depth = depth.unwrap_or(100);
        let path = format!("/book/t{}/P0?len={}", symbol.to_uppercase(), depth);
        let response: OrderbookInfo = self.send_request(Method::GET, &path, None).await?;
        Ok(response.into())
    }

    /// 获取最新成交
    pub async fn get_trades(&self, symbol: &str, limit: Option<u32>) -> Result<Vec<Trade>, BitfinexError> {
        let limit = limit.unwrap_or(100);
        let path = format!("/trades/t{}/hist?limit={}", symbol.to_uppercase(), limit);
        let response: Vec<TradeInfo> = self.send_request(Method::GET, &path, None).await?;
        Ok(response.into_iter().map(Into::into).collect())
    }

    /// 获取K线数据
    pub async fn get_klines(&self, symbol: &str, interval: &str, limit: Option<u32>) -> Result<Vec<Kline>, BitfinexError> {
        let limit = limit.unwrap_or(100);
        let path = format!("/candles/trade:{}:t{}/hist?limit={}", 
            interval, symbol.to_uppercase(), limit);
        let response: Vec<Vec<f64>> = self.send_request(Method::GET, &path, None).await?;
        
        Ok(response.into_iter().map(|k| Kline {
            timestamp: k[0] as i64,
            open: k[1],
            close: k[2],
            high: k[3],
            low: k[4],
            volume: k[5],
        }).collect())
    }
}

/// 将时间间隔转换为毫秒数
fn interval_to_millis(interval: &str) -> f64 {
    match interval {
        "1m" => 60_000.0,
        "5m" => 300_000.0,
        "15m" => 900_000.0,
        "30m" => 1_800_000.0,
        "1h" => 3_600_000.0,
        "3h" => 10_800_000.0,
        "6h" => 21_600_000.0,
        "12h" => 43_200_000.0,
        "1D" => 86_400_000.0,
        "1W" => 604_800_000.0,
        "1M" => 2_592_000_000.0,
        _ => 60_000.0, // 默认1分钟
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_get_symbols() {
        let client = BitfinexRestClient::new(BitfinexRestConfig::default()).unwrap();
        let symbols = client.get_symbols().await.unwrap();
        assert!(!symbols.is_empty());
    }

    #[tokio::test]
    async fn test_get_ticker() {
        let client = BitfinexRestClient::new(BitfinexRestConfig::default()).unwrap();
        let ticker = client.get_ticker("BTCUSD").await.unwrap();
        assert!(ticker.last_price > 0.0);
    }

    #[tokio::test]
    async fn test_get_orderbook() {
        let client = BitfinexRestClient::new(BitfinexRestConfig::default()).unwrap();
        let orderbook = client.get_orderbook("BTCUSD", Some(10)).await.unwrap();
        assert!(!orderbook.bids.is_empty());
        assert!(!orderbook.asks.is_empty());
    }

    #[tokio::test]
    async fn test_get_trades() {
        let client = BitfinexRestClient::new(BitfinexRestConfig::default()).unwrap();
        let trades = client.get_trades("BTCUSD", Some(10)).await.unwrap();
        assert!(!trades.is_empty());
    }

    #[tokio::test]
    async fn test_get_klines() {
        let client = BitfinexRestClient::new(BitfinexRestConfig::default()).unwrap();
        let klines = client.get_klines("BTCUSD", "1m", Some(10)).await.unwrap();
        assert!(!klines.is_empty());
    }
}
