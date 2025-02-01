use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::protocol::Message,
    WebSocketStream,
    MaybeTlsStream,
};
use tokio::time::sleep;
use url::Url;
use tracing::{debug, error, info, warn};
use futures::{SinkExt, StreamExt};
use flate2::read::GzDecoder;
use std::io::Read;

use crate::error::{HuobiError, WebSocketErrorKind};
use crate::types::{SubscribeRequest, UnsubscribeRequest, PingRequest};

const PING_INTERVAL: Duration = Duration::from_secs(5);
const PING_TIMEOUT: Duration = Duration::from_secs(10);
const RECONNECT_DELAY: Duration = Duration::from_secs(5);

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// WebSocket 客户端
pub struct WebSocketClient {
    endpoint: String,
    ws_stream: Option<WsStream>,
    last_ping: Option<Instant>,
    last_pong: Option<Instant>,
    subscriptions: Vec<String>,
}

impl WebSocketClient {
    /// 创建新的 WebSocket 客户端实例
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            ws_stream: None,
            last_ping: None,
            last_pong: None,
            subscriptions: Vec::new(),
        }
    }

    /// 连接到 WebSocket 服务器
    pub async fn connect(&mut self) -> Result<(), HuobiError> {
        let url = Url::parse(&self.endpoint).map_err(|e| {
            HuobiError::WebSocketError(WebSocketErrorKind::ConnectionError(e.to_string()))
        })?;

        info!("正在连接到火币 WebSocket 服务器: {}", url);

        let (ws_stream, _) = connect_async(url).await.map_err(|e| {
            HuobiError::WebSocketError(WebSocketErrorKind::ConnectionError(e.to_string()))
        })?;

        self.ws_stream = Some(ws_stream);
        self.last_ping = Some(Instant::now());
        self.last_pong = Some(Instant::now());

        info!("已成功连接到火币 WebSocket 服务器");

        // 重新订阅之前的频道
        for topic in self.subscriptions.clone() {
            self.subscribe(&topic).await?;
        }

        Ok(())
    }

    /// 重新连接
    pub async fn reconnect(&mut self) -> Result<(), HuobiError> {
        warn!("正在尝试重新连接...");
        self.ws_stream = None;
        sleep(RECONNECT_DELAY).await;
        self.connect().await
    }

    /// 关闭连接
    pub async fn close(&mut self) -> Result<(), HuobiError> {
        if let Some(ws_stream) = self.ws_stream.as_mut() {
            ws_stream.close(None).await.map_err(|e| {
                HuobiError::WebSocketError(WebSocketErrorKind::ConnectionError(e.to_string()))
            })?;
        }
        self.ws_stream = None;
        self.subscriptions.clear();
        Ok(())
    }

    /// 订阅特定主题
    pub async fn subscribe(&mut self, topic: &str) -> Result<(), HuobiError> {
        let request = SubscribeRequest {
            sub: topic.to_string(),
            id: uuid::Uuid::new_v4().to_string(),
        };

        self.send_message(&serde_json::to_string(&request)?).await?;
        if !self.subscriptions.contains(&topic.to_string()) {
            self.subscriptions.push(topic.to_string());
        }
        debug!("已订阅主题: {}", topic);
        Ok(())
    }

    /// 取消订阅特定主题
    pub async fn unsubscribe(&mut self, topic: &str) -> Result<(), HuobiError> {
        let request = UnsubscribeRequest {
            unsub: topic.to_string(),
            id: uuid::Uuid::new_v4().to_string(),
        };

        self.send_message(&serde_json::to_string(&request)?).await?;
        self.subscriptions.retain(|t| t != topic);
        debug!("已取消订阅主题: {}", topic);
        Ok(())
    }

    /// 发送心跳消息
    async fn send_ping(&mut self) -> Result<(), HuobiError> {
        let timestamp = chrono::Utc::now().timestamp_millis();
        let request = PingRequest { ping: timestamp };
        self.send_message(&serde_json::to_string(&request)?).await?;
        self.last_ping = Some(Instant::now());
        debug!("已发送心跳消息");
        Ok(())
    }

    /// 发送消息
    async fn send_message(&mut self, message: &str) -> Result<(), HuobiError> {
        if let Some(ws_stream) = self.ws_stream.as_mut() {
            ws_stream.send(Message::Text(message.to_string())).await.map_err(|e| {
                HuobiError::WebSocketError(WebSocketErrorKind::SendError(e.to_string()))
            })?;
            Ok(())
        } else {
            Err(HuobiError::WebSocketError(WebSocketErrorKind::ConnectionError(
                "WebSocket 未连接".to_string(),
            )))
        }
    }

    /// 接收消息
    pub async fn receive_message(&mut self) -> Result<Option<String>, HuobiError> {
        // 检查是否需要发送心跳
        if let Some(last_ping) = self.last_ping {
            if last_ping.elapsed() >= PING_INTERVAL {
                self.send_ping().await?;
            }
        }

        // 检查心跳超时
        if let Some(last_pong) = self.last_pong {
            if last_pong.elapsed() >= PING_TIMEOUT {
                warn!("心跳超时，准备重新连接");
                self.reconnect().await?;
                return Ok(None);
            }
        }

        if let Some(ws_stream) = self.ws_stream.as_mut() {
            match ws_stream.next().await {
                Some(Ok(msg)) => {
                    match msg {
                        Message::Text(text) => {
                            debug!("收到文本消息: {}", text);
                            Ok(Some(text))
                        }
                        Message::Binary(binary) => {
                            // 解压 GZIP 数据
                            let mut decoder = GzDecoder::new(&binary[..]);
                            let mut decompressed = String::new();
                            decoder.read_to_string(&mut decompressed).map_err(|e| {
                                HuobiError::WebSocketError(WebSocketErrorKind::ReceiveError(e.to_string()))
                            })?;
                            debug!("收到二进制消息（已解压）: {}", decompressed);
                            Ok(Some(decompressed))
                        }
                        Message::Ping(_) => {
                            ws_stream.send(Message::Pong(vec![])).await.map_err(|e| {
                                HuobiError::WebSocketError(WebSocketErrorKind::SendError(e.to_string()))
                            })?;
                            Ok(None)
                        }
                        Message::Pong(_) => {
                            self.last_pong = Some(Instant::now());
                            debug!("收到 Pong 响应");
                            Ok(None)
                        }
                        Message::Close(frame) => {
                            warn!("收到关闭帧: {:?}", frame);
                            self.reconnect().await?;
                            Ok(None)
                        }
                        Message::Frame(_) => Ok(None),
                    }
                }
                Some(Err(e)) => {
                    error!("WebSocket 错误: {}", e);
                    Err(HuobiError::WebSocketError(WebSocketErrorKind::ReceiveError(
                        e.to_string(),
                    )))
                }
                None => {
                    warn!("WebSocket 流已关闭");
                    self.reconnect().await?;
                    Ok(None)
                }
            }
        } else {
            Err(HuobiError::WebSocketError(WebSocketErrorKind::ConnectionError(
                "WebSocket 未连接".to_string(),
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_websocket_connection() {
        let mut client = WebSocketClient::new("wss://api.huobi.pro/ws");
        assert!(client.connect().await.is_ok());
        assert!(client.close().await.is_ok());
    }

    #[tokio::test]
    async fn test_subscription() {
        let mut client = WebSocketClient::new("wss://api.huobi.pro/ws");
        client.connect().await.unwrap();

        // 订阅 BTC/USDT K线数据
        let topic = "market.btcusdt.kline.1min";
        assert!(client.subscribe(topic).await.is_ok());
        assert!(client.subscriptions.contains(&topic.to_string()));

        // 等待并接收消息
        let timeout_duration = Duration::from_secs(5);
        let result = timeout(timeout_duration, client.receive_message()).await;
        assert!(result.is_ok());

        // 取消订阅
        assert!(client.unsubscribe(topic).await.is_ok());
        assert!(!client.subscriptions.contains(&topic.to_string()));

        client.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_ping_pong() {
        let mut client = WebSocketClient::new("wss://api.huobi.pro/ws");
        client.connect().await.unwrap();

        // 订阅一个主题来验证连接
        let topic = "market.btcusdt.kline.1min";
        assert!(client.subscribe(topic).await.is_ok());

        // 等待接收订阅响应
        let timeout_duration = Duration::from_secs(5);
        let start = Instant::now();
        let mut received_response = false;

        while start.elapsed() < timeout_duration && !received_response {
            if let Ok(Some(msg)) = client.receive_message().await {
                if msg.contains("\"subbed\"") {
                    received_response = true;
                }
            }
            sleep(Duration::from_millis(100)).await;
        }

        assert!(received_response, "未收到订阅响应");

        // 取消订阅并关闭连接
        assert!(client.unsubscribe(topic).await.is_ok());
        client.close().await.unwrap();
    }
} 