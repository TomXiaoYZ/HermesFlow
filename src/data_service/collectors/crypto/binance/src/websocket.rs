use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{
    connect_async,
    tungstenite::protocol::Message,
    WebSocketStream,
};
use futures::{SinkExt, StreamExt};
use serde_json::json;
use chrono::Utc;
use tracing::{debug, error, info, warn};
use url::Url;

use crate::error::BinanceError;
use crate::collectors::common::{MarketData, DataQuality, MarketDataType};

type WebSocketConnection = WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// WebSocket客户端状态
#[derive(Debug, Clone)]
pub struct WebSocketState {
    is_connected: bool,
    last_ping: Option<i64>,
    last_pong: Option<i64>,
    subscribed_channels: Vec<String>,
}

/// WebSocket客户端
pub struct WebSocketClient {
    endpoint: String,
    state: Arc<Mutex<WebSocketState>>,
    ws_stream: Option<WebSocketConnection>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl WebSocketClient {
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            state: Arc::new(Mutex::new(WebSocketState {
                is_connected: false,
                last_ping: None,
                last_pong: None,
                subscribed_channels: Vec::new(),
            })),
            ws_stream: None,
            shutdown_tx: None,
        }
    }

    /// 建立WebSocket连接
    pub async fn connect(&mut self) -> Result<(), BinanceError> {
        let url = Url::parse(&self.endpoint)
            .map_err(|e| BinanceError::ConfigError(format!("Invalid WebSocket URL: {}", e)))?;

        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| BinanceError::WebSocketError(format!("Connection failed: {}", e)))?;

        self.ws_stream = Some(ws_stream);
        self.state.lock().await.is_connected = true;

        info!("Connected to Binance WebSocket server");
        Ok(())
    }

    /// 断开WebSocket连接
    pub async fn disconnect(&mut self) -> Result<(), BinanceError> {
        if let Some(mut ws_stream) = self.ws_stream.take() {
            ws_stream
                .close(None)
                .await
                .map_err(|e| BinanceError::WebSocketError(format!("Disconnect failed: {}", e)))?;
        }

        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(()).await;
        }

        self.state.lock().await.is_connected = false;
        info!("Disconnected from Binance WebSocket server");
        Ok(())
    }

    /// 订阅数据流
    pub async fn subscribe(&mut self, channels: Vec<String>) -> Result<(), BinanceError> {
        if let Some(ws_stream) = &mut self.ws_stream {
            let subscribe_msg = json!({
                "method": "SUBSCRIBE",
                "params": channels,
                "id": Utc::now().timestamp_millis()
            });

            ws_stream
                .send(Message::Text(subscribe_msg.to_string()))
                .await
                .map_err(|e| BinanceError::WebSocketError(format!("Subscribe failed: {}", e)))?;

            self.state
                .lock()
                .await
                .subscribed_channels
                .extend(channels.clone());

            info!("Subscribed to channels: {:?}", channels);
        }
        Ok(())
    }

    /// 取消订阅数据流
    pub async fn unsubscribe(&mut self, channels: Vec<String>) -> Result<(), BinanceError> {
        if let Some(ws_stream) = &mut self.ws_stream {
            let unsubscribe_msg = json!({
                "method": "UNSUBSCRIBE",
                "params": channels,
                "id": Utc::now().timestamp_millis()
            });

            ws_stream
                .send(Message::Text(unsubscribe_msg.to_string()))
                .await
                .map_err(|e| BinanceError::WebSocketError(format!("Unsubscribe failed: {}", e)))?;

            let mut state = self.state.lock().await;
            state
                .subscribed_channels
                .retain(|c| !channels.contains(c));

            info!("Unsubscribed from channels: {:?}", channels);
        }
        Ok(())
    }

    /// 启动数据接收
    pub async fn start(
        &mut self,
        tx: mpsc::Sender<(MarketData, DataQuality)>,
    ) -> Result<(), BinanceError> {
        if self.ws_stream.is_none() {
            return Err(BinanceError::WebSocketError("Not connected".to_string()));
        }

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        let mut ws_stream = self.ws_stream.take().unwrap();
        let state = Arc::clone(&self.state);

        // 启动消息处理任务
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // 处理WebSocket消息
                    msg = ws_stream.next() => {
                        match msg {
                            Some(Ok(msg)) => {
                                match msg {
                                    Message::Text(text) => {
                                        if let Err(e) = Self::handle_message(text, &tx).await {
                                            error!("Failed to handle message: {}", e);
                                        }
                                    }
                                    Message::Ping(data) => {
                                        if let Err(e) = ws_stream.send(Message::Pong(data)).await {
                                            error!("Failed to send pong: {}", e);
                                        }
                                        state.lock().await.last_ping = Some(Utc::now().timestamp_millis());
                                    }
                                    Message::Pong(_) => {
                                        state.lock().await.last_pong = Some(Utc::now().timestamp_millis());
                                    }
                                    Message::Close(frame) => {
                                        warn!("Received close frame: {:?}", frame);
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                            Some(Err(e)) => {
                                error!("WebSocket error: {}", e);
                                break;
                            }
                            None => {
                                info!("WebSocket stream ended");
                                break;
                            }
                        }
                    }
                    // 处理关闭信号
                    _ = shutdown_rx.recv() => {
                        info!("Received shutdown signal");
                        break;
                    }
                }
            }

            // 清理连接状态
            state.lock().await.is_connected = false;
            if let Err(e) = ws_stream.close(None).await {
                error!("Error closing WebSocket connection: {}", e);
            }
        });

        Ok(())
    }

    /// 停止数据接收
    pub async fn stop(&mut self) -> Result<(), BinanceError> {
        self.disconnect().await
    }

    /// 处理接收到的消息
    async fn handle_message(
        text: String,
        tx: &mpsc::Sender<(MarketData, DataQuality)>,
    ) -> Result<(), BinanceError> {
        let value: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| BinanceError::ParseError(format!("Failed to parse message: {}", e)))?;

        // 解析消息类型
        let data_type = if text.contains("@trade") {
            MarketDataType::Trade
        } else if text.contains("@depth") {
            MarketDataType::OrderBook
        } else if text.contains("@kline") {
            MarketDataType::Kline
        } else if text.contains("@ticker") {
            MarketDataType::Ticker
        } else {
            return Ok(());
        };

        // 提取symbol
        let symbol = value["s"]
            .as_str()
            .or_else(|| value["symbol"].as_str())
            .unwrap_or("UNKNOWN")
            .to_string();

        // 提取时间戳
        let timestamp = value["E"]
            .as_i64()
            .or_else(|| value["T"].as_i64())
            .unwrap_or_else(|| Utc::now().timestamp_millis());

        let market_data = MarketData::new(
            "binance".to_string(),
            symbol,
            data_type,
            value,
        );

        let quality = DataQuality {
            latency: Utc::now().timestamp_millis() - timestamp,
            is_gap: false,
            gap_size: None,
            is_valid: true,
            error_type: None,
            metadata: Default::default(),
        };

        tx.send((market_data, quality))
            .await
            .map_err(|e| BinanceError::WebSocketError(format!("Failed to send data: {}", e)))?;

        Ok(())
    }
} 