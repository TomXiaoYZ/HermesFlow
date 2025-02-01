use crate::error::{BitfinexError, Result};
use crate::models::{Orderbook, OrderbookLevel, Ticker, Trade, TradeSide};
use crate::types::{WsAuthRequest, WsResponse, WsSubscribeRequest};

use futures::{SinkExt, StreamExt};
use hmac::{Hmac, Mac};
use sha2::Sha384;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::protocol::Message,
    MaybeTlsStream,
    WebSocketStream,
};
use url::Url;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use serde_json::{json, Value};
use hex;

const WS_URL: &str = "wss://api-pub.bitfinex.com/ws/2";

/// WebSocket客户端配置
#[derive(Debug, Clone)]
pub struct BitfinexWebsocketConfig {
    /// API密钥
    pub api_key: Option<String>,
    /// API密钥
    pub api_secret: Option<String>,
    /// 心跳间隔(秒)
    pub ping_interval: Option<Duration>,
}

impl Default for BitfinexWebsocketConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            api_secret: None,
            ping_interval: Some(Duration::from_secs(10)),
        }
    }
}

/// WebSocket客户端
pub struct BitfinexWebsocketClient {
    /// WebSocket流
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    /// 配置信息
    config: BitfinexWebsocketConfig,
    /// 频道订阅映射
    channels: HashMap<i32, String>,
    subscriptions: Arc<RwLock<HashMap<i64, String>>>,
    event_tx: mpsc::UnboundedSender<Result<Value, BitfinexError>>,
}

impl BitfinexWebsocketClient {
    /// 创建新的WebSocket客户端
    pub async fn new(
        config: BitfinexWebsocketConfig,
        event_tx: mpsc::UnboundedSender<Result<Value, BitfinexError>>,
    ) -> Result<Self> {
        let url = Url::parse(WS_URL).map_err(BitfinexError::UrlParseError)?;
        
        let (stream, _) = connect_async(url)
            .await
            .map_err(|e| BitfinexError::WebSocketError(e))?;

        let mut client = Self {
            stream,
            config,
            channels: HashMap::new(),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
        };

        // 如果配置了API密钥，进行认证
        if let (Some(api_key), Some(api_secret)) = (
            &client.config.api_key.clone(),
            &client.config.api_secret.clone(),
        ) {
            client.authenticate(api_key, api_secret).await?;
        }

        Ok(client)
    }

    /// 认证
    async fn authenticate(&mut self, api_key: &str, api_secret: &str) -> Result<()> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string();

        let auth_payload = format!("AUTH{}", &nonce);
        let mut mac = Hmac::<Sha384>::new_from_slice(api_secret.as_bytes())
            .map_err(|e| BitfinexError::InternalError(e.to_string()))?;
        mac.update(auth_payload.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());

        let auth = WsAuthRequest {
            api_key: api_key.to_string(),
            signature,
            nonce,
        };

        let msg = serde_json::to_string(&auth).map_err(BitfinexError::JsonError)?;
        self.send(&msg).await?;

        Ok(())
    }

    /// 发送消息
    async fn send(&mut self, msg: &str) -> Result<()> {
        self.stream
            .send(Message::Text(msg.to_string()))
            .await
            .map_err(|e| BitfinexError::WebSocketError(e))?;
        Ok(())
    }

    /// 接收消息
    pub async fn receive(&mut self) -> Result<Option<WsResponse>> {
        if let Some(msg) = self.stream.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    let response: WsResponse =
                        serde_json::from_str(&text).map_err(BitfinexError::JsonError)?;
                    Ok(Some(response))
                }
                Ok(Message::Ping(_)) => {
                    self.stream
                        .send(Message::Pong(vec![]))
                        .await
                        .map_err(|e| BitfinexError::WebSocketError(e))?;
                    Ok(None)
                }
                Ok(Message::Close(_)) => {
                    Err(BitfinexError::WebSocketError("Connection closed".into()))
                }
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// 订阅Ticker
    pub async fn subscribe_ticker(&mut self, symbol: &str) -> Result<()> {
        let sub = WsSubscribeRequest {
            event: "subscribe".to_string(),
            channel: "ticker".to_string(),
            pair: symbol.to_uppercase(),
        };

        let msg = serde_json::to_string(&sub).map_err(BitfinexError::JsonError)?;
        self.send(&msg).await?;

        // 等待订阅确认
        while let Some(response) = self.receive().await? {
            if response.event == "subscribed" && response.channel == Some("ticker".to_string()) {
                if let Some(channel_id) = response.channel_id {
                    self.channels.insert(channel_id, format!("ticker:{}", symbol));
                    break;
                }
            }
        }

        Ok(())
    }

    /// 订阅订单簿
    pub async fn subscribe_orderbook(&mut self, symbol: &str, precision: &str) -> Result<()> {
        let sub = WsSubscribeRequest {
            event: "subscribe".to_string(),
            channel: "book".to_string(),
            pair: format!("{}:{}:P0", symbol.to_uppercase(), precision),
        };

        let msg = serde_json::to_string(&sub).map_err(BitfinexError::JsonError)?;
        self.send(&msg).await?;

        // 等待订阅确认
        while let Some(response) = self.receive().await? {
            if response.event == "subscribed" && response.channel == Some("book".to_string()) {
                if let Some(channel_id) = response.channel_id {
                    self.channels.insert(channel_id, format!("book:{}", symbol));
                    break;
                }
            }
        }

        Ok(())
    }

    /// 订阅成交
    pub async fn subscribe_trades(&mut self, symbol: &str) -> Result<()> {
        let sub = WsSubscribeRequest {
            event: "subscribe".to_string(),
            channel: "trades".to_string(),
            pair: symbol.to_uppercase(),
        };

        let msg = serde_json::to_string(&sub).map_err(BitfinexError::JsonError)?;
        self.send(&msg).await?;

        // 等待订阅确认
        while let Some(response) = self.receive().await? {
            if response.event == "subscribed" && response.channel == Some("trades".to_string()) {
                if let Some(channel_id) = response.channel_id {
                    self.channels.insert(channel_id, format!("trades:{}", symbol));
                    break;
                }
            }
        }

        Ok(())
    }

    /// 取消订阅
    pub async fn unsubscribe(&mut self, channel_id: i32) -> Result<()> {
        let unsub = serde_json::json!({
            "event": "unsubscribe",
            "chanId": channel_id,
        });

        let msg = serde_json::to_string(&unsub).map_err(BitfinexError::JsonError)?;
        self.send(&msg).await?;

        // 等待取消订阅确认
        while let Some(response) = self.receive().await? {
            if response.event == "unsubscribed" && response.channel_id == Some(channel_id) {
                self.channels.remove(&channel_id);
                break;
            }
        }

        Ok(())
    }

    /// 解析Ticker数据
    pub fn parse_ticker(&self, data: &serde_json::Value) -> Result<Ticker> {
        if let Some(arr) = data.as_array() {
            if arr.len() >= 10 {
                return Ok(Ticker {
                    symbol: "".to_string(), // 从channels映射中获取
                    last_price: arr[6].as_f64().unwrap_or(0.0),
                    high_24h: arr[8].as_f64().unwrap_or(0.0),
                    low_24h: arr[9].as_f64().unwrap_or(0.0),
                    volume_24h: arr[7].as_f64().unwrap_or(0.0),
                    amount_24h: 0.0,
                    price_change_24h: arr[5].as_f64().unwrap_or(0.0),
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                });
            }
        }
        Err(BitfinexError::JsonError("Invalid ticker data".into()))
    }

    /// 解析订单簿数据
    pub fn parse_orderbook(&self, data: &serde_json::Value) -> Result<Orderbook> {
        if let Some(arr) = data.as_array() {
            let mut bids = Vec::new();
            let mut asks = Vec::new();

            for item in arr {
                if let Some(level) = item.as_array() {
                    if level.len() >= 3 {
                        let price = level[0].as_f64().unwrap_or(0.0);
                        let count = level[1].as_i64().unwrap_or(0) as u32;
                        let amount = level[2].as_f64().unwrap_or(0.0);

                        let order_level = OrderbookLevel {
                            price,
                            amount: amount.abs(),
                            count,
                        };

                        if amount > 0.0 {
                            bids.push(order_level);
                        } else {
                            asks.push(order_level);
                        }
                    }
                }
            }

            return Ok(Orderbook {
                symbol: "".to_string(), // 从channels映射中获取
                bids,
                asks,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            });
        }
        Err(BitfinexError::JsonError("Invalid orderbook data".into()))
    }

    /// 解析成交数据
    pub fn parse_trade(&self, data: &serde_json::Value) -> Result<Trade> {
        if let Some(arr) = data.as_array() {
            if arr.len() >= 4 {
                return Ok(Trade {
                    id: arr[0].as_i64().unwrap_or(0).to_string(),
                    symbol: "".to_string(), // 从channels映射中获取
                    price: arr[3].as_f64().unwrap_or(0.0),
                    amount: arr[2].as_f64().unwrap_or(0.0).abs(),
                    side: if arr[2].as_f64().unwrap_or(0.0) > 0.0 {
                        TradeSide::Buy
                    } else {
                        TradeSide::Sell
                    },
                    timestamp: arr[1].as_i64().unwrap_or(0) as u64,
                });
            }
        }
        Err(BitfinexError::JsonError("Invalid trade data".into()))
    }

    /// 关闭连接
    pub async fn close(&mut self) -> Result<()> {
        self.stream
            .close(None)
            .await
            .map_err(|e| BitfinexError::WebSocketError(e))?;
        Ok(())
    }
}
