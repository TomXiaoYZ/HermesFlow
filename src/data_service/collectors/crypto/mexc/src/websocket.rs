use std::time::{SystemTime, UNIX_EPOCH};
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::protocol::Message,
    MaybeTlsStream,
    WebSocketStream,
};
use url::Url;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use serde_json::json;

use crate::{MexcConfig, MexcError, Result};
use crate::models::{SubscribeMessage, ResponseMessage};

type HmacSha256 = Hmac<Sha256>;
type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// MEXC WebSocket客户端
pub struct MexcWebsocketClient {
    /// WebSocket流
    stream: Option<WsStream>,
    /// 配置信息
    config: MexcConfig,
    /// 请求ID
    request_id: u64,
}

impl MexcWebsocketClient {
    /// 创建新的WebSocket客户端
    pub fn new(config: MexcConfig) -> Self {
        Self {
            stream: None,
            config,
            request_id: 1,
        }
    }

    /// 生成签名
    fn sign(&self, timestamp: u64) -> Result<String> {
        if let Some(secret) = &self.config.api_secret {
            let message = format!("{}", timestamp);
            let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
                .map_err(|e| MexcError::InternalError(e.to_string()))?;
            mac.update(message.as_bytes());
            Ok(hex::encode(mac.finalize().into_bytes()))
        } else {
            Err(MexcError::AuthenticationError("Missing API secret".to_string()))
        }
    }

    /// 连接WebSocket服务器
    pub async fn connect(&mut self) -> Result<()> {
        let url = Url::parse(&self.config.ws_base_url)
            .map_err(MexcError::UrlParseError)?;

        let (ws_stream, _) = connect_async(url).await
            .map_err(MexcError::WebSocketError)?;

        self.stream = Some(ws_stream);
        
        // 如果配置了API密钥，进行登录认证
        if self.config.api_key.is_some() {
            self.login().await?;
        }

        Ok(())
    }

    /// 登录认证
    async fn login(&mut self) -> Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| MexcError::InternalError(e.to_string()))?
            .as_millis() as u64;

        let signature = self.sign(timestamp)?;
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| MexcError::AuthenticationError("Missing API key".to_string()))?;

        let login_message = json!({
            "method": "login",
            "params": {
                "apiKey": api_key,
                "signature": signature,
                "timestamp": timestamp
            },
            "id": self.next_id()
        });

        self.send(&login_message.to_string()).await?;

        // 等待登录响应
        if let Some(response) = self.receive().await? {
            if response.channel == "login" {
                Ok(())
            } else {
                Err(MexcError::AuthenticationError("Login failed".to_string()))
            }
        } else {
            Err(MexcError::AuthenticationError("No login response".to_string()))
        }
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
                .map_err(MexcError::WebSocketError)?;
            Ok(())
        } else {
            Err(MexcError::NetworkError("WebSocket not connected".to_string()))
        }
    }

    /// 接收消息
    pub async fn receive(&mut self) -> Result<Option<ResponseMessage>> {
        if let Some(stream) = &mut self.stream {
            match stream.next().await {
                Some(Ok(Message::Text(text))) => {
                    let response: ResponseMessage = serde_json::from_str(&text)
                        .map_err(MexcError::JsonError)?;
                    Ok(Some(response))
                }
                Some(Ok(Message::Ping(_))) => {
                    stream.send(Message::Pong(vec![])).await
                        .map_err(MexcError::WebSocketError)?;
                    Ok(None)
                }
                Some(Ok(Message::Close(_))) => {
                    Err(MexcError::NetworkError("WebSocket closed by server".to_string()))
                }
                Some(Err(e)) => {
                    Err(MexcError::WebSocketError(e))
                }
                _ => Ok(None),
            }
        } else {
            Err(MexcError::NetworkError("WebSocket not connected".to_string()))
        }
    }

    /// 订阅频道
    pub async fn subscribe(&mut self, channels: Vec<String>) -> Result<()> {
        let subscribe_message = SubscribeMessage {
            method: "SUBSCRIPTION".to_string(),
            params: channels,
            id: self.next_id(),
        };

        let message = serde_json::to_string(&subscribe_message)
            .map_err(MexcError::JsonError)?;

        self.send(&message).await
    }

    /// 取消订阅频道
    pub async fn unsubscribe(&mut self, channels: Vec<String>) -> Result<()> {
        let unsubscribe_message = SubscribeMessage {
            method: "UNSUBSCRIPTION".to_string(),
            params: channels,
            id: self.next_id(),
        };

        let message = serde_json::to_string(&unsubscribe_message)
            .map_err(MexcError::JsonError)?;

        self.send(&message).await
    }

    /// 保持连接活跃
    pub async fn keep_alive(&mut self) -> Result<()> {
        if let Some(stream) = &mut self.stream {
            stream.send(Message::Ping(vec![])).await
                .map_err(MexcError::WebSocketError)?;
        }
        Ok(())
    }

    /// 关闭连接
    pub async fn close(&mut self) -> Result<()> {
        if let Some(stream) = &mut self.stream {
            stream.close(None).await
                .map_err(MexcError::WebSocketError)?;
            self.stream = None;
        }
        Ok(())
    }
} 