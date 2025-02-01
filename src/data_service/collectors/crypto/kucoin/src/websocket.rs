use std::time::{SystemTime, UNIX_EPOCH, Duration};
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::protocol::Message,
    MaybeTlsStream,
    WebSocketStream,
};
use url::Url;
use serde_json::json;

use crate::{KucoinConfig, KucoinError, Result};
use crate::models::{SubscribeMessage, ResponseMessage};
use crate::rest::KucoinRestClient;

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Kucoin WebSocket客户端
pub struct KucoinWebsocketClient {
    /// WebSocket流
    stream: Option<WsStream>,
    /// REST客户端
    rest_client: KucoinRestClient,
    /// 配置信息
    config: KucoinConfig,
    /// 请求ID
    request_id: u64,
    /// 心跳间隔
    ping_interval: Duration,
    /// 心跳超时时间
    ping_timeout: Duration,
}

impl KucoinWebsocketClient {
    /// 创建新的WebSocket客户端
    pub fn new(config: KucoinConfig) -> Self {
        Self {
            stream: None,
            rest_client: KucoinRestClient::new(config.clone()),
            config,
            request_id: 1,
            ping_interval: Duration::from_secs(30),
            ping_timeout: Duration::from_secs(10),
        }
    }

    /// 连接WebSocket服务器
    pub async fn connect(&mut self) -> Result<()> {
        // 获取WebSocket token
        let token_info = self.rest_client.get_ws_token(false).await?;
        let server = token_info.servers.first()
            .ok_or_else(|| KucoinError::NetworkError("No available WebSocket server".to_string()))?;

        // 构建WebSocket URL
        let url = format!("{}?token={}", server.endpoint, token_info.token);
        let url = Url::parse(&url)
            .map_err(KucoinError::UrlParseError)?;

        // 连接WebSocket服务器
        let (ws_stream, _) = connect_async(url).await
            .map_err(KucoinError::WebSocketError)?;

        // 更新心跳配置
        self.ping_interval = Duration::from_millis(server.ping_interval);
        self.ping_timeout = Duration::from_millis(server.ping_timeout);

        self.stream = Some(ws_stream);
        Ok(())
    }

    /// 获取下一个请求ID
    fn next_id(&mut self) -> u64 {
        let id = self.request_id;
        self.request_id += 1;
        id
    }

    /// 发送消息
    pub async fn send(&mut self, message: &str) -> Result<()> {
        if let Some(stream) = &mut self.stream {
            stream.send(Message::Text(message.to_string())).await
                .map_err(KucoinError::WebSocketError)?;
            Ok(())
        } else {
            Err(KucoinError::NetworkError("WebSocket not connected".to_string()))
        }
    }

    /// 接收消息
    pub async fn receive(&mut self) -> Result<Option<ResponseMessage>> {
        if let Some(stream) = &mut self.stream {
            match stream.next().await {
                Some(Ok(Message::Text(text))) => {
                    let response: ResponseMessage = serde_json::from_str(&text)
                        .map_err(KucoinError::JsonError)?;
                    Ok(Some(response))
                }
                Some(Ok(Message::Ping(_))) => {
                    stream.send(Message::Pong(vec![])).await
                        .map_err(KucoinError::WebSocketError)?;
                    Ok(None)
                }
                Some(Ok(Message::Close(_))) => {
                    Err(KucoinError::NetworkError("WebSocket closed by server".to_string()))
                }
                Some(Err(e)) => {
                    Err(KucoinError::WebSocketError(e))
                }
                _ => Ok(None),
            }
        } else {
            Err(KucoinError::NetworkError("WebSocket not connected".to_string()))
        }
    }

    /// 订阅频道
    pub async fn subscribe(&mut self, channels: Vec<String>) -> Result<()> {
        let subscribe_message = SubscribeMessage {
            method: "subscribe".to_string(),
            params: channels,
            id: self.next_id(),
        };

        let message = serde_json::to_string(&subscribe_message)
            .map_err(KucoinError::JsonError)?;

        self.send(&message).await
    }

    /// 取消订阅频道
    pub async fn unsubscribe(&mut self, channels: Vec<String>) -> Result<()> {
        let unsubscribe_message = SubscribeMessage {
            method: "unsubscribe".to_string(),
            params: channels,
            id: self.next_id(),
        };

        let message = serde_json::to_string(&unsubscribe_message)
            .map_err(KucoinError::JsonError)?;

        self.send(&message).await
    }

    /// 发送心跳
    pub async fn ping(&mut self) -> Result<()> {
        let ping_message = json!({
            "id": self.next_id(),
            "type": "ping"
        });

        let message = serde_json::to_string(&ping_message)
            .map_err(KucoinError::JsonError)?;

        self.send(&message).await
    }

    /// 关闭连接
    pub async fn close(&mut self) -> Result<()> {
        if let Some(stream) = &mut self.stream {
            stream.close(None).await
                .map_err(KucoinError::WebSocketError)?;
            self.stream = None;
        }
        Ok(())
    }
}
