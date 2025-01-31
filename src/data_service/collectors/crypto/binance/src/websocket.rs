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

use common::{MarketData, DataQuality, MarketDataType, CollectorError};
use crate::error::BinanceError;

type WebSocketConnection = WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

const MAX_RECONNECT_ATTEMPTS: u32 = 5;
const INITIAL_RECONNECT_DELAY: Duration = Duration::from_secs(1);
const MAX_RECONNECT_DELAY: Duration = Duration::from_secs(60);
const PING_INTERVAL: Duration = Duration::from_secs(20);
const PONG_TIMEOUT: Duration = Duration::from_secs(5);

/// WebSocket客户端状态
#[derive(Debug, Clone)]
pub struct WebSocketState {
    is_connected: bool,
    last_ping: Option<i64>,
    last_pong: Option<i64>,
    subscribed_channels: Vec<String>,
    reconnect_attempts: u32,
    last_reconnect: Option<Instant>,
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
                reconnect_attempts: 0,
                last_reconnect: None,
            })),
            ws_stream: None,
            shutdown_tx: None,
        }
    }

    /// 计算重连延迟
    fn calculate_reconnect_delay(attempts: u32) -> Duration {
        let base_delay = INITIAL_RECONNECT_DELAY.as_secs() as u32;
        let delay = base_delay * 2u32.pow(attempts);
        Duration::from_secs(delay.min(MAX_RECONNECT_DELAY.as_secs() as u32) as u64)
    }

    /// 建立WebSocket连接
    pub async fn connect(&mut self) -> Result<(), BinanceError> {
        let mut state = self.state.lock().await;
        if state.reconnect_attempts >= MAX_RECONNECT_ATTEMPTS {
            return Err(CollectorError::ConnectionError(
                "Max reconnection attempts reached".to_string()
            ).into());
        }

        let url = Url::parse(&self.endpoint)?;

        match connect_async(url).await {
            Ok((ws_stream, _)) => {
                self.ws_stream = Some(ws_stream);
                state.is_connected = true;
                state.last_reconnect = Some(Instant::now());
                state.reconnect_attempts = 0;
                info!("Connected to Binance WebSocket server");
                Ok(())
            }
            Err(e) => {
                state.reconnect_attempts += 1;
                let delay = Self::calculate_reconnect_delay(state.reconnect_attempts);
                error!(
                    "Connection failed (attempt {}/{}), retrying in {:?}: {}",
                    state.reconnect_attempts, MAX_RECONNECT_ATTEMPTS, delay, e
                );
                tokio::time::sleep(delay).await;
                Err(e.into())
            }
        }
    }

    /// 重新订阅之前的频道
    async fn resubscribe(&mut self) -> Result<(), BinanceError> {
        let channels = {
            let state = self.state.lock().await;
            state.subscribed_channels.clone()
        };

        if !channels.is_empty() {
            self.subscribe(channels).await?;
        }
        Ok(())
    }

    /// 检查连接状态并在必要时重连
    async fn ensure_connection(&mut self) -> Result<(), BinanceError> {
        let should_reconnect = {
            let state = self.state.lock().await;
            !state.is_connected
                || state.last_pong.is_some()
                && state.last_ping.is_some()
                && state
                    .last_pong
                    .unwrap()
                    .saturating_sub(state.last_ping.unwrap())
                    > PONG_TIMEOUT.as_millis() as i64
        };

        if should_reconnect {
            self.disconnect().await?;
            self.connect().await?;
            self.resubscribe().await?;
        }
        Ok(())
    }

    /// 断开WebSocket连接
    pub async fn disconnect(&mut self) -> Result<(), BinanceError> {
        if let Some(mut ws_stream) = self.ws_stream.take() {
            ws_stream.close(None).await?;
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

            ws_stream.send(Message::Text(subscribe_msg.to_string())).await?;

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

            ws_stream.send(Message::Text(unsubscribe_msg.to_string())).await?;

            let mut state = self.state.lock().await;
            state.subscribed_channels.retain(|c| !channels.contains(c));

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
            return Err(CollectorError::WebSocketError("Not connected".to_string()).into());
        }

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        let mut ws_stream = self.ws_stream.take().unwrap();
        let state = Arc::clone(&self.state);

        // 启动消息处理任务
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(PING_INTERVAL);

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
                                state.lock().await.is_connected = false;
                                break;
                            }
                            None => {
                                info!("WebSocket stream ended");
                                state.lock().await.is_connected = false;
                                break;
                            }
                        }
                    }
                    // 处理心跳
                    _ = interval.tick() => {
                        let should_ping = {
                            let state = state.lock().await;
                            state.is_connected
                        };

                        if !should_ping {
                            break;
                        }

                        if let Err(e) = ws_stream.send(Message::Ping(vec![])).await {
                            error!("Failed to send ping: {}", e);
                            break;
                        }

                        state.lock().await.last_ping = Some(Utc::now().timestamp_millis());

                        // 等待PONG_TIMEOUT
                        tokio::time::sleep(PONG_TIMEOUT).await;

                        let should_reconnect = {
                            let state = state.lock().await;
                            state.last_pong.is_none()
                                || state.last_ping.unwrap() - state.last_pong.unwrap()
                                    > PONG_TIMEOUT.as_millis() as i64
                        };

                        if should_reconnect {
                            error!("Pong timeout, connection may be dead");
                            state.lock().await.is_connected = false;
                            break;
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
            .map_err(|e| CollectorError::ParseError(format!("Failed to parse message: {}", e)))?;

        // 解析消息类型
        let data_type = if value["e"].as_str() == Some("trade") {
            MarketDataType::Trade
        } else if value["e"].as_str() == Some("kline") {
            MarketDataType::Kline
        } else if value["e"].as_str() == Some("depthUpdate") {
            MarketDataType::OrderBook
        } else if value["e"].as_str() == Some("24hrTicker") {
            MarketDataType::Ticker
        } else {
            MarketDataType::Unknown
        };

        // 创建市场数据
        let market_data = MarketData {
            exchange: "binance".to_string(),
            symbol: value["s"]
                .as_str()
                .ok_or_else(|| CollectorError::ParseError("Missing symbol".to_string()))?
                .to_string(),
            data_type,
            timestamp: Utc::now(),
            received_at: Utc::now(),
            raw_data: value.clone(),
            metadata: Default::default(),
        };

        // 创建数据质量信息
        let data_quality = DataQuality {
            latency: 0, // 需要根据实际情况计算
            is_gap: false,
            gap_size: None,
            is_valid: true,
            error_type: None,
            metadata: Default::default(),
        };

        // 发送数据
        tx.send((market_data, data_quality))
            .await
            .map_err(|e| CollectorError::ProcessingError(format!("Failed to send data: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;
    use tokio::time::timeout;
    use std::time::Duration;

    #[tokio::test]
    async fn test_websocket_client_init() {
        let client = WebSocketClient::new("wss://stream.binance.com:9443");
        assert_eq!(client.endpoint, "wss://stream.binance.com:9443");
    }

    #[tokio::test]
    async fn test_calculate_reconnect_delay() {
        let delay1 = WebSocketClient::calculate_reconnect_delay(0);
        let delay2 = WebSocketClient::calculate_reconnect_delay(1);
        let delay3 = WebSocketClient::calculate_reconnect_delay(2);

        assert!(delay1 < delay2);
        assert!(delay2 < delay3);
        assert!(delay3 <= MAX_RECONNECT_DELAY);
    }

    #[tokio::test]
    async fn test_websocket_client_lifecycle() -> Result<(), BinanceError> {
        let mut client = WebSocketClient::new("wss://stream.binance.com:9443");
        
        // 连接
        client.connect().await?;

        // 创建数据通道
        let (tx, mut rx) = mpsc::channel(100);

        // 启动
        client.start(tx).await?;

        // 订阅
        let channels = vec!["btcusdt@trade".to_string()];
        client.subscribe(channels.clone()).await?;

        // 等待数据，最多等待10秒
        let receive_result = timeout(Duration::from_secs(10), rx.recv()).await;
        match receive_result {
            Ok(Some(_)) => println!("成功接收到数据"),
            Ok(None) => println!("通道已关闭"),
            Err(_) => println!("等待数据超时"),
        }

        // 取消订阅
        client.unsubscribe(channels).await?;

        // 停止
        client.stop().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_handle_message() -> Result<(), BinanceError> {
        let (tx, mut rx) = mpsc::channel(1);

        // 测试交易消息
        let trade_msg = r#"{
            "e": "trade",
            "s": "BTCUSDT",
            "p": "50000.00",
            "q": "1.0",
            "T": 1609459200000,
            "m": true
        }"#;

        WebSocketClient::handle_message(trade_msg.to_string(), &tx).await?;

        // 等待消息
        let receive_result = timeout(Duration::from_secs(1), rx.recv()).await;
        assert!(receive_result.is_ok());

        if let Ok(Some((market_data, data_quality))) = receive_result {
            assert_eq!(market_data.exchange, "binance");
            assert_eq!(market_data.symbol, "BTCUSDT");
            assert!(matches!(market_data.data_type, MarketDataType::Trade));
            assert!(data_quality.is_valid);
        }

        Ok(())
    }
} 