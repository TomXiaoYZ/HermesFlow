use std::time::{SystemTime, UNIX_EPOCH};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use reqwest::{Client, RequestBuilder};
use crate::{KucoinConfig, KucoinError, Result};
use crate::types::*;

type HmacSha256 = Hmac<Sha256>;

/// Kucoin REST API客户端
#[derive(Debug, Clone)]
pub struct KucoinRestClient {
    /// HTTP客户端
    client: Client,
    /// 配置信息
    config: KucoinConfig,
}

impl KucoinRestClient {
    /// 创建新的REST客户端
    pub fn new(config: KucoinConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    /// 生成签名
    fn sign(&self, timestamp: u64, method: &str, endpoint: &str, body: &str) -> Result<String> {
        if let Some(secret) = &self.config.api_secret {
            let message = format!("{}{}{}{}", timestamp, method, endpoint, body);
            let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
                .map_err(|e| KucoinError::InternalError(e.to_string()))?;
            mac.update(message.as_bytes());
            Ok(BASE64.encode(mac.finalize().into_bytes()))
        } else {
            Err(KucoinError::AuthenticationError("Missing API secret".to_string()))
        }
    }

    /// 生成KC-API-SIGN
    fn get_kc_api_sign(&self, timestamp: u64, method: &str, endpoint: &str, body: &str) -> Result<String> {
        self.sign(timestamp, method, endpoint, body)
    }

    /// 生成KC-API-PASSPHRASE
    fn get_kc_api_passphrase(&self) -> Result<String> {
        if let Some(passphrase) = &self.config.api_passphrase {
            if let Some(secret) = &self.config.api_secret {
                let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
                    .map_err(|e| KucoinError::InternalError(e.to_string()))?;
                mac.update(passphrase.as_bytes());
                Ok(BASE64.encode(mac.finalize().into_bytes()))
            } else {
                Err(KucoinError::AuthenticationError("Missing API secret".to_string()))
            }
        } else {
            Err(KucoinError::AuthenticationError("Missing API passphrase".to_string()))
        }
    }

    /// 添加认证信息
    fn auth_request(&self, builder: RequestBuilder, method: &str, endpoint: &str, body: &str) -> Result<RequestBuilder> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| KucoinError::InternalError(e.to_string()))?
            .as_millis() as u64;

        let sign = self.get_kc_api_sign(timestamp, method, endpoint, body)?;
        let passphrase = self.get_kc_api_passphrase()?;

        Ok(builder
            .header("KC-API-KEY", self.config.api_key.as_ref().ok_or_else(|| KucoinError::AuthenticationError("Missing API key".to_string()))?)
            .header("KC-API-SIGN", sign)
            .header("KC-API-TIMESTAMP", timestamp.to_string())
            .header("KC-API-PASSPHRASE", passphrase)
            .header("KC-API-KEY-VERSION", "2"))
    }

    /// 发送GET请求
    async fn get<T: for<'de> serde::Deserialize<'de>>(&self, endpoint: &str, need_auth: bool) -> Result<T> {
        let url = format!("{}{}", self.config.rest_base_url, endpoint);
        let mut builder = self.client.get(&url);

        if need_auth {
            builder = self.auth_request(builder, "GET", endpoint, "")?;
        }

        let response = builder
            .send()
            .await
            .map_err(KucoinError::HttpError)?;

        let api_response: ApiResponse<T> = response
            .json()
            .await
            .map_err(KucoinError::HttpError)?;

        if api_response.code == 200000 {
            api_response.data.ok_or_else(|| KucoinError::ApiError {
                code: api_response.code,
                message: api_response.msg,
            })
        } else {
            Err(KucoinError::ApiError {
                code: api_response.code,
                message: api_response.msg,
            })
        }
    }

    /// 获取所有交易对信息
    pub async fn get_symbols(&self) -> Result<Vec<SymbolInfo>> {
        self.get("/api/v2/symbols", false).await
    }

    /// 获取指定交易对的Ticker数据
    pub async fn get_ticker(&self, symbol: &str) -> Result<TickerInfo> {
        self.get(&format!("/api/v1/market/orderbook/level1?symbol={}", symbol), false).await
    }

    /// 获取指定交易对的深度数据
    pub async fn get_orderbook(&self, symbol: &str, limit: Option<u32>) -> Result<OrderbookInfo> {
        let limit = limit.unwrap_or(100).min(100);
        self.get(&format!("/api/v1/market/orderbook/level2_{}?symbol={}", limit, symbol), false).await
    }

    /// 获取指定交易对的最新成交
    pub async fn get_trades(&self, symbol: &str, limit: Option<u32>) -> Result<Vec<TradeInfo>> {
        let limit = limit.unwrap_or(100).min(100);
        self.get(&format!("/api/v1/market/histories?symbol={}&limit={}", symbol, limit), false).await
    }

    /// 获取WebSocket连接Token
    pub async fn get_ws_token(&self, is_private: bool) -> Result<WebsocketTokenInfo> {
        let endpoint = if is_private {
            "/api/v1/bullet-private"
        } else {
            "/api/v1/bullet-public"
        };
        self.get(endpoint, is_private).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;
    use serde_json::json;

    fn setup() -> (Server, KucoinConfig) {
        let mut server = Server::new();
        let config = KucoinConfig {
            rest_base_url: server.url(),
            ws_base_url: "wss://test.com".to_string(),
            api_key: Some("test_key".to_string()),
            api_secret: Some("test_secret".to_string()),
            api_passphrase: Some("test_passphrase".to_string()),
        };
        (server, config)
    }

    #[tokio::test]
    async fn test_get_symbols() {
        let (mut server, config) = setup();
        let mock = server.mock("GET", "/api/v2/symbols")
            .with_status(200)
            .with_body(r#"
                {
                    "code": 200000,
                    "data": [{
                        "symbol": "BTC-USDT",
                        "base_currency": "BTC",
                        "quote_currency": "USDT",
                        "price_precision": 2,
                        "size_precision": 6,
                        "min_size": "0.0001",
                        "min_funds": "5",
                        "enable_trading": true
                    }]
                }
            "#)
            .create();

        let client = KucoinRestClient::new(config);
        let symbols = client.get_symbols().await.unwrap();
        
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].symbol, "BTC-USDT");
        assert_eq!(symbols[0].base_currency, "BTC");
        mock.assert();
    }

    #[tokio::test]
    async fn test_get_ticker() {
        let (mut server, config) = setup();
        let mock = server.mock("GET", "/api/v1/market/orderbook/level1?symbol=BTC-USDT")
            .with_status(200)
            .with_body(r#"
                {
                    "code": 200000,
                    "data": {
                        "symbol": "BTC-USDT",
                        "last_price": "50000.5",
                        "high_24h": "51000.0",
                        "low_24h": "49000.0",
                        "volume_24h": "100.5",
                        "amount_24h": "5025025.25"
                    }
                }
            "#)
            .create();

        let client = KucoinRestClient::new(config);
        let ticker = client.get_ticker("BTC-USDT").await.unwrap();
        
        assert_eq!(ticker.symbol, "BTC-USDT");
        assert_eq!(ticker.last_price, "50000.5");
        mock.assert();
    }

    #[tokio::test]
    async fn test_get_orderbook() {
        let (mut server, config) = setup();
        let mock = server.mock("GET", "/api/v1/market/orderbook/level2_100?symbol=BTC-USDT")
            .with_status(200)
            .with_body(r#"
                {
                    "code": 200000,
                    "data": {
                        "timestamp": 1234567890,
                        "bids": [["50000.5", "1.5"], ["49999.5", "2.0"]],
                        "asks": [["50001.0", "1.0"], ["50002.0", "2.5"]]
                    }
                }
            "#)
            .create();

        let client = KucoinRestClient::new(config);
        let orderbook = client.get_orderbook("BTC-USDT", None).await.unwrap();
        
        assert_eq!(orderbook.timestamp, 1234567890);
        assert_eq!(orderbook.bids.len(), 2);
        assert_eq!(orderbook.asks.len(), 2);
        mock.assert();
    }

    #[tokio::test]
    async fn test_get_trades() {
        let (mut server, config) = setup();
        let mock = server.mock("GET", "/api/v1/market/histories?symbol=BTC-USDT&limit=100")
            .with_status(200)
            .with_body(r#"
                {
                    "code": 200000,
                    "data": [{
                        "trade_id": "123456",
                        "price": "50000.5",
                        "size": "1.5",
                        "timestamp": 1234567890,
                        "side": "buy"
                    }]
                }
            "#)
            .create();

        let client = KucoinRestClient::new(config);
        let trades = client.get_trades("BTC-USDT", None).await.unwrap();
        
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].trade_id, "123456");
        assert_eq!(trades[0].price, "50000.5");
        mock.assert();
    }

    #[tokio::test]
    async fn test_get_ws_token() {
        let (mut server, config) = setup();
        let mock = server.mock("GET", "/api/v1/bullet-public")
            .with_status(200)
            .with_body(r#"
                {
                    "code": 200000,
                    "data": {
                        "token": "test_token",
                        "servers": [{
                            "endpoint": "wss://test.com",
                            "protocol": "websocket",
                            "encrypt": false,
                            "ping_interval": 18000,
                            "ping_timeout": 10000
                        }]
                    }
                }
            "#)
            .create();

        let client = KucoinRestClient::new(config);
        let token_info = client.get_ws_token(false).await.unwrap();
        
        assert_eq!(token_info.token, "test_token");
        assert_eq!(token_info.servers[0].endpoint, "wss://test.com");
        mock.assert();
    }

    #[tokio::test]
    async fn test_api_error() {
        let (mut server, config) = setup();
        let mock = server.mock("GET", "/api/v2/symbols")
            .with_status(200)
            .with_body(r#"
                {
                    "code": 400100,
                    "msg": "Invalid parameter"
                }
            "#)
            .create();

        let client = KucoinRestClient::new(config);
        let result = client.get_symbols().await;
        
        assert!(result.is_err());
        if let Err(KucoinError::ApiError { code, message }) = result {
            assert_eq!(code, 400100);
            assert_eq!(message, "Invalid parameter");
        } else {
            panic!("Expected ApiError");
        }
        mock.assert();
    }
}
