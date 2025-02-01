use std::time::{SystemTime, UNIX_EPOCH};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use reqwest::{Client, RequestBuilder};
use crate::{MexcConfig, MexcError, Result};
use crate::types::*;

type HmacSha256 = Hmac<Sha256>;

/// MEXC REST API客户端
#[derive(Debug, Clone)]
pub struct MexcRestClient {
    /// HTTP客户端
    client: Client,
    /// 配置信息
    config: MexcConfig,
}

impl MexcRestClient {
    /// 创建新的REST客户端
    pub fn new(config: MexcConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    /// 生成签名
    fn sign(&self, timestamp: u64, params: &str) -> Result<String> {
        if let Some(secret) = &self.config.api_secret {
            let message = format!("{}{}", timestamp, params);
            let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
                .map_err(|e| MexcError::InternalError(e.to_string()))?;
            mac.update(message.as_bytes());
            Ok(hex::encode(mac.finalize().into_bytes()))
        } else {
            Err(MexcError::AuthenticationError("Missing API secret".to_string()))
        }
    }

    /// 添加认证信息
    fn auth_request(&self, builder: RequestBuilder, params: &str) -> Result<RequestBuilder> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| MexcError::InternalError(e.to_string()))?
            .as_millis() as u64;

        let signature = self.sign(timestamp, params)?;

        Ok(builder
            .header("ApiKey", self.config.api_key.as_ref().ok_or_else(|| MexcError::AuthenticationError("Missing API key".to_string()))?)
            .header("Signature", signature)
            .header("Request-Time", timestamp.to_string()))
    }

    /// 发送GET请求
    async fn get<T: for<'de> serde::Deserialize<'de>>(&self, path: &str, need_auth: bool) -> Result<T> {
        let url = format!("{}{}", self.config.rest_base_url, path);
        let mut builder = self.client.get(&url);

        if need_auth {
            builder = self.auth_request(builder, path)?;
        }

        let response = builder
            .send()
            .await
            .map_err(MexcError::HttpError)?;

        let api_response: ApiResponse<T> = response
            .json()
            .await
            .map_err(MexcError::HttpError)?;

        if api_response.code == 200 {
            api_response.data.ok_or_else(|| MexcError::ApiError {
                code: api_response.code,
                message: api_response.msg,
            })
        } else {
            Err(MexcError::ApiError {
                code: api_response.code,
                message: api_response.msg,
            })
        }
    }

    /// 获取所有交易对信息
    pub async fn get_symbols(&self) -> Result<Vec<SymbolInfo>> {
        self.get("/api/v3/exchangeInfo", false).await
    }

    /// 获取指定交易对的Ticker数据
    pub async fn get_ticker(&self, symbol: &str) -> Result<TickerInfo> {
        self.get(&format!("/api/v3/ticker/24hr?symbol={}", symbol), false).await
    }

    /// 获取指定交易对的深度数据
    pub async fn get_orderbook(&self, symbol: &str, limit: Option<u32>) -> Result<OrderbookInfo> {
        let limit = limit.unwrap_or(100).min(5000);
        self.get(&format!("/api/v3/depth?symbol={}&limit={}", symbol, limit), false).await
    }

    /// 获取指定交易对的最新成交
    pub async fn get_trades(&self, symbol: &str, limit: Option<u32>) -> Result<Vec<TradeInfo>> {
        let limit = limit.unwrap_or(100).min(1000);
        self.get(&format!("/api/v3/trades?symbol={}&limit={}", symbol, limit), false).await
    }
} 