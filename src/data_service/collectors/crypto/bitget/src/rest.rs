use std::time::{SystemTime, UNIX_EPOCH};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use reqwest::{Client, RequestBuilder};
use crate::{BitgetConfig, BitgetError, Result};
use crate::types::*;

type HmacSha256 = Hmac<Sha256>;

/// Bitget REST API客户端
#[derive(Debug, Clone)]
pub struct BitgetRestClient {
    /// HTTP客户端
    client: Client,
    /// 配置信息
    config: BitgetConfig,
}

impl BitgetRestClient {
    /// 创建新的REST客户端
    pub fn new(config: BitgetConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    /// 生成签名
    fn sign(&self, timestamp: u64, method: &str, path: &str, body: Option<&str>) -> Result<String> {
        if let Some(secret) = &self.config.api_secret {
            let message = format!("{}{}{}{}", timestamp, method, path, body.unwrap_or(""));
            let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
                .map_err(|e| BitgetError::InternalError(e.to_string()))?;
            mac.update(message.as_bytes());
            Ok(hex::encode(mac.finalize().into_bytes()))
        } else {
            Err(BitgetError::AuthenticationError("Missing API secret".to_string()))
        }
    }

    /// 添加认证信息
    fn auth_request(&self, builder: RequestBuilder, method: &str, path: &str, body: Option<&str>) -> Result<RequestBuilder> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| BitgetError::InternalError(e.to_string()))?
            .as_millis() as u64;

        let signature = self.sign(timestamp, method, path, body)?;

        Ok(builder
            .header("ACCESS-KEY", self.config.api_key.as_ref().ok_or_else(|| BitgetError::AuthenticationError("Missing API key".to_string()))?)
            .header("ACCESS-SIGN", signature)
            .header("ACCESS-TIMESTAMP", timestamp.to_string())
            .header("ACCESS-PASSPHRASE", "bitget"))
    }

    /// 发送GET请求
    async fn get<T: for<'de> serde::Deserialize<'de>>(&self, path: &str, need_auth: bool) -> Result<T> {
        let url = format!("{}{}", self.config.rest_base_url, path);
        let mut builder = self.client.get(&url);

        if need_auth {
            builder = self.auth_request(builder, "GET", path, None)?;
        }

        let response = builder
            .send()
            .await
            .map_err(BitgetError::HttpError)?;

        let api_response: ApiResponse<T> = response
            .json()
            .await
            .map_err(BitgetError::HttpError)?;

        if api_response.code == "00000" {
            api_response.data.ok_or_else(|| BitgetError::ApiError {
                code: api_response.code,
                message: api_response.msg,
            })
        } else {
            Err(BitgetError::ApiError {
                code: api_response.code,
                message: api_response.msg,
            })
        }
    }

    /// 获取所有交易对信息
    pub async fn get_symbols(&self) -> Result<Vec<SymbolInfo>> {
        self.get("/api/spot/v1/public/products", false).await
    }

    /// 获取指定交易对的Ticker数据
    pub async fn get_ticker(&self, symbol: &str) -> Result<TickerInfo> {
        self.get(&format!("/api/spot/v1/market/ticker?symbol={}", symbol), false).await
    }

    /// 获取指定交易对的深度数据
    pub async fn get_orderbook(&self, symbol: &str, limit: Option<u32>) -> Result<OrderbookInfo> {
        let limit = limit.unwrap_or(100).min(100);
        self.get(&format!("/api/spot/v1/market/depth?symbol={}&limit={}", symbol, limit), false).await
    }

    /// 获取指定交易对的最新成交
    pub async fn get_trades(&self, symbol: &str, limit: Option<u32>) -> Result<Vec<TradeInfo>> {
        let limit = limit.unwrap_or(100).min(100);
        self.get(&format!("/api/spot/v1/market/trades?symbol={}&limit={}", symbol, limit), false).await
    }
} 