use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{
    connect_async,
    tungstenite::Message,
    WebSocketStream,
};
use futures::{SinkExt, StreamExt};
use serde_json::json;
use chrono::Utc;
use tracing::{error, info, warn};
use url::Url;
use std::time::{Duration, Instant};
use tokio::time::timeout;

use crate::error::{BybitError, WebSocketErrorKind};
use crate::models::{SubscribeRequest, WebSocketResponse};

type WebSocketConnection = WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

const MAX_RECONNECT_ATTEMPTS: u32 = 5;
const INITIAL_RECONNECT_DELAY: Duration = Duration::from_secs(1);
const MAX_RECONNECT_DELAY: Duration = Duration::from_secs(60);
const PING_INTERVAL: Duration = Duration::from_secs(20);
const PONG_TIMEOUT: Duration = Duration::from_secs(5);

/// WebSocket 客户端状态
#[derive(Debug)]
struct WebSocketState {
    /// 是否已连接
    is_connected: bool,
    /// 已订阅的频道列表
    subscribed_channels: Vec<String>,
    /// 重连尝试次数
    reconnect_attempts: u32,
    /// 最后一次重连时间
    last_reconnect: Option<Instant>,
    /// 最后一次发送 ping 的时间戳（毫秒）
    last_ping: Option<i64>,
    /// 最后一次接收 pong 的时间戳（毫秒）
    last_pong: Option<i64>,
}

/// WebSocket 客户端
pub struct WebSocketClient {
    endpoint: String,
    state: Arc<Mutex<WebSocketState>>,
    ws_stream: Option<WebSocketConnection>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl WebSocketClient {
    /// 创建新的 WebSocket 客户端实例
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            state: Arc::new(Mutex::new(WebSocketState {
                is_connected: false,
                subscribed_channels: Vec::new(),
                reconnect_attempts: 0,
                last_reconnect: None,
                last_ping: None,
                last_pong: None,
            })),
            ws_stream: None,
            shutdown_tx: None,
        }
    }

    /// 连接到 WebSocket 服务器
    pub async fn connect(&mut self) -> Result<(), BybitError> {
        let url = Url::parse(&self.endpoint)
            .map_err(|e| BybitError::ConfigError {
                msg: format!("Invalid WebSocket URL: {}", e),
                source: Some(Box::new(e)),
            })?;

        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| BybitError::WebSocketError {
                kind: WebSocketErrorKind::ConnectionFailed,
                source: Some(Box::new(e)),
            })?;

        self.ws_stream = Some(ws_stream);
        
        let mut state = self.state.lock().await;
        state.is_connected = true;
        state.reconnect_attempts = 0;
        state.last_reconnect = Some(Instant::now());

        Ok(())
    }

    /// 订阅特定频道
    pub async fn subscribe(&mut self, channel: &str, symbol: &str) -> Result<(), BybitError> {
        let topic = format!("{}.{}", channel, symbol);
        let request = SubscribeRequest {
            op: "subscribe".to_string(),
            args: vec![topic.clone()],
        };

        let message = serde_json::to_string(&request)
            .map_err(|e| BybitError::ParseError {
                kind: crate::error::ParseErrorKind::JsonError,
                source: Some(Box::new(e)),
            })?;

        if let Some(ws_stream) = &mut self.ws_stream {
            ws_stream.send(Message::Text(message)).await
                .map_err(|e| BybitError::WebSocketError {
                    kind: WebSocketErrorKind::SendFailed,
                    source: Some(Box::new(e)),
                })?;

            let mut state = self.state.lock().await;
            state.subscribed_channels.push(topic);
            Ok(())
        } else {
            Err(BybitError::WebSocketError {
                kind: WebSocketErrorKind::ConnectionClosed,
                source: None,
            })
        }
    }

    /// 取消订阅特定频道
    pub async fn unsubscribe(&mut self, channel: &str, symbol: &str) -> Result<(), BybitError> {
        let topic = format!("{}.{}", channel, symbol);
        let request = SubscribeRequest {
            op: "unsubscribe".to_string(),
            args: vec![topic.clone()],
        };

        let message = serde_json::to_string(&request)
            .map_err(|e| BybitError::ParseError {
                kind: crate::error::ParseErrorKind::JsonError,
                source: Some(Box::new(e)),
            })?;

        if let Some(ws_stream) = &mut self.ws_stream {
            ws_stream.send(Message::Text(message)).await
                .map_err(|e| BybitError::WebSocketError {
                    kind: WebSocketErrorKind::SendFailed,
                    source: Some(Box::new(e)),
                })?;

            let mut state = self.state.lock().await;
            state.subscribed_channels.retain(|c| c != &topic);
            Ok(())
        } else {
            Err(BybitError::WebSocketError {
                kind: WebSocketErrorKind::ConnectionClosed,
                source: None,
            })
        }
    }

    /// 发送 ping 消息
    async fn send_ping(&mut self) -> Result<(), BybitError> {
        if let Some(ws_stream) = &mut self.ws_stream {
            let ping_msg = json!({
                "op": "ping"
            });
            let message = serde_json::to_string(&ping_msg)
                .map_err(|e| BybitError::ParseError {
                    kind: crate::error::ParseErrorKind::JsonError,
                    source: Some(Box::new(e)),
                })?;

            ws_stream.send(Message::Text(message)).await
                .map_err(|e| BybitError::WebSocketError {
                    kind: WebSocketErrorKind::SendFailed,
                    source: Some(Box::new(e)),
                })?;

            let mut state = self.state.lock().await;
            state.last_ping = Some(Utc::now().timestamp_millis());
            Ok(())
        } else {
            Err(BybitError::WebSocketError {
                kind: WebSocketErrorKind::ConnectionClosed,
                source: None,
            })
        }
    }

    /// 关闭 WebSocket 连接
    pub async fn close(&mut self) -> Result<(), BybitError> {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(()).await;
        }

        if let Some(ws_stream) = &mut self.ws_stream {
            ws_stream.close(None).await
                .map_err(|e| BybitError::WebSocketError {
                    kind: WebSocketErrorKind::Other("Failed to close connection".to_string()),
                    source: Some(Box::new(e)),
                })?;
        }

        let mut state = self.state.lock().await;
        state.is_connected = false;
        state.subscribed_channels.clear();

        Ok(())
    }

    /// 接收 WebSocket 消息
    pub async fn receive_message(&mut self) -> Result<Option<String>, BybitError> {
        if let Some(ws_stream) = &mut self.ws_stream {
            match ws_stream.next().await {
                Some(Ok(message)) => {
                    match message {
                        Message::Text(text) => {
                            // 处理 pong 消息
                            if let Ok(response) = serde_json::from_str::<WebSocketResponse>(&text) {
                                if response.event == Some("pong".to_string()) {
                                    let mut state = self.state.lock().await;
                                    state.last_pong = Some(Utc::now().timestamp_millis());
                                    return Ok(None);
                                }
                            }
                            Ok(Some(text))
                        }
                        Message::Close(frame) => {
                            error!("WebSocket closed: {:?}", frame);
                            let mut state = self.state.lock().await;
                            state.is_connected = false;
                            Err(BybitError::WebSocketError {
                                kind: WebSocketErrorKind::ConnectionClosed,
                                source: None,
                            })
                        }
                        _ => Ok(None),
                    }
                }
                Some(Err(e)) => {
                    Err(BybitError::WebSocketError {
                        kind: WebSocketErrorKind::ReceiveFailed,
                        source: Some(Box::new(e)),
                    })
                }
                None => {
                    Err(BybitError::WebSocketError {
                        kind: WebSocketErrorKind::ConnectionClosed,
                        source: None,
                    })
                }
            }
        } else {
            Err(BybitError::WebSocketError {
                kind: WebSocketErrorKind::ConnectionClosed,
                source: None,
            })
        }
    }

    /// 重新连接到服务器
    pub async fn reconnect(&mut self) -> Result<(), BybitError> {
        let mut state = self.state.lock().await;
        if state.reconnect_attempts >= MAX_RECONNECT_ATTEMPTS {
            return Err(BybitError::WebSocketError {
                kind: WebSocketErrorKind::ConnectionFailed,
                source: None,
            });
        }

        let delay = std::cmp::min(
            INITIAL_RECONNECT_DELAY * 2u32.pow(state.reconnect_attempts),
            MAX_RECONNECT_DELAY,
        );
        
        state.reconnect_attempts += 1;
        state.last_reconnect = Some(Instant::now());
        let channels = state.subscribed_channels.clone();
        drop(state);

        tokio::time::sleep(delay).await;
        self.connect().await?;

        // 重新订阅之前的频道
        for channel in channels {
            let parts: Vec<&str> = channel.split('.').collect();
            if parts.len() == 2 {
                self.subscribe(parts[0], parts[1]).await?;
            }
        }

        Ok(())
    }

    /// 检查连接状态
    pub async fn check_connection(&mut self) -> Result<(), BybitError> {
        let state = self.state.lock().await;
        if !state.is_connected {
            return Err(BybitError::WebSocketError {
                kind: WebSocketErrorKind::ConnectionClosed,
                source: None,
            });
        }

        if let (Some(last_ping), Some(last_pong)) = (state.last_ping, state.last_pong) {
            if last_ping - last_pong > PONG_TIMEOUT.as_millis() as i64 {
                return Err(BybitError::WebSocketError {
                    kind: WebSocketErrorKind::PingPongTimeout,
                    source: None,
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;

    const TEST_WS_ENDPOINT: &str = "wss://stream.bybit.com/v5/public/spot";

    #[tokio::test]
    async fn test_websocket_connection() {
        let mut client = WebSocketClient::new(TEST_WS_ENDPOINT);
        
        // 测试连接
        let result = client.connect().await;
        assert!(result.is_ok(), "Failed to connect: {:?}", result);
        
        // 测试连接状态
        let result = client.check_connection().await;
        assert!(result.is_ok(), "Connection check failed: {:?}", result);
        
        // 测试关闭连接
        let result = client.close().await;
        assert!(result.is_ok(), "Failed to close connection: {:?}", result);
    }

    #[tokio::test]
    async fn test_subscribe_unsubscribe() {
        let mut client = WebSocketClient::new(TEST_WS_ENDPOINT);
        
        // 连接
        client.connect().await.expect("Failed to connect");
        
        // 测试订阅
        let result = client.subscribe("orderbook", "BTCUSDT").await;
        assert!(result.is_ok(), "Failed to subscribe: {:?}", result);
        
        // 等待接收消息
        let timeout_duration = Duration::from_secs(5);
        let result = timeout(timeout_duration, async {
            loop {
                match client.receive_message().await {
                    Ok(Some(_)) => return Ok(()),
                    Ok(None) => continue,
                    Err(e) => return Err(e),
                }
            }
        }).await;
        assert!(result.is_ok(), "Failed to receive message within timeout");
        
        // 测试取消订阅
        let result = client.unsubscribe("orderbook", "BTCUSDT").await;
        assert!(result.is_ok(), "Failed to unsubscribe: {:?}", result);
        
        // 关闭连接
        client.close().await.expect("Failed to close connection");
    }

    #[tokio::test]
    async fn test_reconnection() {
        let mut client = WebSocketClient::new(TEST_WS_ENDPOINT);
        
        // 连接
        client.connect().await.expect("Failed to connect");
        
        // 订阅以确保有活跃的频道
        client.subscribe("orderbook", "BTCUSDT").await.expect("Failed to subscribe");
        
        // 模拟连接断开
        client.ws_stream = None;
        
        // 测试重连
        let result = client.reconnect().await;
        assert!(result.is_ok(), "Failed to reconnect: {:?}", result);
        
        // 验证重连后的状态
        let result = client.check_connection().await;
        assert!(result.is_ok(), "Connection check failed after reconnect: {:?}", result);
        
        // 关闭连接
        client.close().await.expect("Failed to close connection");
    }

    #[tokio::test]
    async fn test_ping_pong() {
        let mut client = WebSocketClient::new(TEST_WS_ENDPOINT);
        
        // 连接
        client.connect().await.expect("Failed to connect");
        
        // 发送ping
        let result = client.send_ping().await;
        assert!(result.is_ok(), "Failed to send ping: {:?}", result);
        
        // 等待接收pong
        let timeout_duration = Duration::from_secs(5);
        let result = timeout(timeout_duration, async {
            loop {
                match client.receive_message().await {
                    Ok(Some(msg)) => {
                        if let Ok(response) = serde_json::from_str::<WebSocketResponse>(&msg) {
                            if response.event == Some("pong".to_string()) {
                                return Ok(());
                            }
                        }
                    }
                    Ok(None) => continue,
                    Err(e) => return Err(e),
                }
            }
        }).await;
        assert!(result.is_ok(), "Failed to receive pong within timeout");
        
        // 关闭连接
        client.close().await.expect("Failed to close connection");
    }
} 