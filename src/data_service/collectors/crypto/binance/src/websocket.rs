use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{
    connect_async,
    tungstenite::Message,
    WebSocketStream,
    MaybeTlsStream,
};
use futures::{SinkExt, StreamExt};
use serde_json::json;
use chrono::Utc;
use tracing::{error, info, warn, debug};
use url::Url;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tokio::net::TcpStream;
use tokio::time::sleep;

use common::{MarketData, DataQuality, MarketDataType, CollectorError};
use crate::error::{BinanceError, WebSocketErrorKind};
use crate::types::{WebSocketResponse, WebSocketEvent, SubscribeRequest, UnsubscribeRequest};

type WebSocketConnection = WebSocketStream<MaybeTlsStream<TcpStream>>;

const MAX_RECONNECT_ATTEMPTS: u32 = 5;
const INITIAL_RECONNECT_DELAY: Duration = Duration::from_secs(1);
const MAX_RECONNECT_DELAY: Duration = Duration::from_secs(60);
const PING_INTERVAL: Duration = Duration::from_secs(20);
const PONG_TIMEOUT: Duration = Duration::from_secs(5);
const RECONNECT_DELAY: Duration = Duration::from_secs(5);

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
    last_ping: Option<Instant>,
    last_pong: Option<Instant>,
    subscriptions: Vec<String>,
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
            last_ping: None,
            last_pong: None,
            subscriptions: Vec::new(),
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
                self.last_ping = Some(Instant::now());
                self.last_pong = Some(Instant::now());
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
        // 解析 WebSocket 响应
        let response: WebSocketResponse = serde_json::from_str(&text)?;

        // 如果是订阅响应，直接返回
        if response.id.is_some() {
            debug!("Received subscription response: {:?}", response);
            return Ok(());
        }

        // 处理市场数据事件
        if let Some(event) = response.event {
            let (market_data, data_quality) = match event {
                WebSocketEvent::Trade(trade) => {
                    let market_data = MarketData {
                        exchange: "binance".to_string(),
                        symbol: trade.symbol,
                        data_type: MarketDataType::Trade,
                        timestamp: Utc::now(), // 使用 trade.time 创建 DateTime
                        received_at: Utc::now(),
                        raw_data: serde_json::to_value(&trade)?,
                        metadata: Default::default(),
                    };

                    let data_quality = DataQuality {
                        latency: Utc::now().timestamp_millis() - trade.time,
                        is_gap: false,
                        gap_size: None,
                        is_valid: true,
                        error_type: None,
                        metadata: Default::default(),
                    };

                    (market_data, data_quality)
                }

                WebSocketEvent::Kline(kline_event) => {
                    let market_data = MarketData {
                        exchange: "binance".to_string(),
                        symbol: kline_event.symbol,
                        data_type: MarketDataType::Kline,
                        timestamp: Utc::now(), // 使用 kline.close_time 创建 DateTime
                        received_at: Utc::now(),
                        raw_data: serde_json::to_value(&kline_event)?,
                        metadata: {
                            let mut map = std::collections::HashMap::new();
                            map.insert("interval".to_string(), kline_event.kline.interval);
                            map
                        },
                    };

                    let data_quality = DataQuality {
                        latency: Utc::now().timestamp_millis() - kline_event.kline.close_time,
                        is_gap: false,
                        gap_size: None,
                        is_valid: true,
                        error_type: None,
                        metadata: Default::default(),
                    };

                    (market_data, data_quality)
                }

                WebSocketEvent::Depth(depth) => {
                    let market_data = MarketData {
                        exchange: "binance".to_string(),
                        symbol: depth.symbol,
                        data_type: MarketDataType::OrderBook,
                        timestamp: Utc::now(), // 使用 depth.event_time 创建 DateTime
                        received_at: Utc::now(),
                        raw_data: serde_json::to_value(&depth)?,
                        metadata: {
                            let mut map = std::collections::HashMap::new();
                            map.insert("update_id".to_string(), depth.update_id.to_string());
                            map
                        },
                    };

                    let data_quality = DataQuality {
                        latency: Utc::now().timestamp_millis() - depth.event_time,
                        is_gap: false,
                        gap_size: None,
                        is_valid: true,
                        error_type: None,
                        metadata: Default::default(),
                    };

                    (market_data, data_quality)
                }

                WebSocketEvent::Ticker(ticker) => {
                    let market_data = MarketData {
                        exchange: "binance".to_string(),
                        symbol: ticker.symbol,
                        data_type: MarketDataType::Ticker,
                        timestamp: Utc::now(), // 使用 ticker.event_time 创建 DateTime
                        received_at: Utc::now(),
                        raw_data: serde_json::to_value(&ticker)?,
                        metadata: Default::default(),
                    };

                    let data_quality = DataQuality {
                        latency: Utc::now().timestamp_millis() - ticker.event_time,
                        is_gap: false,
                        gap_size: None,
                        is_valid: true,
                        error_type: None,
                        metadata: Default::default(),
                    };

                    (market_data, data_quality)
                }
            };

            // 发送数据
            tx.send((market_data, data_quality))
                .await
                .map_err(|e| BinanceError::SystemError {
                    msg: format!("Failed to send market data: {}", e),
                    source: None,
                })?;
        }

        Ok(())
    }

    /// 重新连接
    pub async fn reconnect(&mut self) -> Result<(), BinanceError> {
        warn!("正在尝试重新连接...");
        self.ws_stream = None;
        sleep(RECONNECT_DELAY).await;
        self.connect().await
    }

    /// 关闭连接
    pub async fn close(&mut self) -> Result<(), BinanceError> {
        if let Some(ws_stream) = self.ws_stream.as_mut() {
            ws_stream.close(None).await.map_err(|e| {
                BinanceError::WebSocketError(WebSocketErrorKind::ConnectionError(e.to_string()))
            })?;
        }
        self.ws_stream = None;
        self.subscriptions.clear();
        Ok(())
    }

    /// 订阅特定主题
    pub async fn subscribe_topic(&mut self, topic: &str) -> Result<(), BinanceError> {
        let request = SubscribeRequest {
            method: "SUBSCRIBE".to_string(),
            params: vec![topic.to_string()],
            id: 1,
        };

        if let Some(ws_stream) = self.ws_stream.as_mut() {
            let message = serde_json::to_string(&request).map_err(|e| {
                BinanceError::WebSocketError(WebSocketErrorKind::SendError(e.to_string()))
            })?;

            ws_stream
                .send(Message::Text(message))
                .await
                .map_err(|e| {
                    BinanceError::WebSocketError(WebSocketErrorKind::SendError(e.to_string()))
                })?;

            self.subscriptions.push(topic.to_string());
            debug!("已订阅主题: {}", topic);
        } else {
            return Err(BinanceError::WebSocketError(
                WebSocketErrorKind::ConnectionError("WebSocket未连接".to_string()),
            ));
        }

        Ok(())
    }

    /// 取消订阅特定主题
    pub async fn unsubscribe_topic(&mut self, topic: &str) -> Result<(), BinanceError> {
        let request = UnsubscribeRequest {
            method: "UNSUBSCRIBE".to_string(),
            params: vec![topic.to_string()],
            id: 1,
        };

        if let Some(ws_stream) = self.ws_stream.as_mut() {
            let message = serde_json::to_string(&request).map_err(|e| {
                BinanceError::WebSocketError(WebSocketErrorKind::SendError(e.to_string()))
            })?;

            ws_stream
                .send(Message::Text(message))
                .await
                .map_err(|e| {
                    BinanceError::WebSocketError(WebSocketErrorKind::SendError(e.to_string()))
                })?;

            self.subscriptions.retain(|x| x != topic);
            debug!("已取消订阅主题: {}", topic);
        } else {
            return Err(BinanceError::WebSocketError(
                WebSocketErrorKind::ConnectionError("WebSocket未连接".to_string()),
            ));
        }

        Ok(())
    }

    /// 接收消息
    pub async fn receive_message(&mut self) -> Result<Option<String>, BinanceError> {
        if let Some(ws_stream) = self.ws_stream.as_mut() {
            match ws_stream.next().await {
                Some(Ok(message)) => match message {
                    Message::Text(text) => {
                        debug!("收到文本消息: {}", text);
                        Ok(Some(text))
                    }
                    Message::Binary(data) => {
                        let text = String::from_utf8(data).map_err(|e| {
                            BinanceError::WebSocketError(WebSocketErrorKind::ReceiveError(
                                e.to_string(),
                            ))
                        })?;
                        debug!("收到二进制消息: {}", text);
                        Ok(Some(text))
                    }
                    Message::Ping(_) => {
                        ws_stream.send(Message::Pong(vec![])).await.map_err(|e| {
                            BinanceError::WebSocketError(WebSocketErrorKind::SendError(e.to_string()))
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
                },
                Some(Err(e)) => {
                    error!("WebSocket 错误: {}", e);
                    Err(BinanceError::WebSocketError(WebSocketErrorKind::ReceiveError(
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
            Err(BinanceError::WebSocketError(WebSocketErrorKind::ConnectionError(
                "WebSocket 未连接".to_string(),
            )))
        }
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
    async fn test_handle_trade_message() {
        let (tx, mut rx) = mpsc::channel(1);

        let trade_msg = r#"{
            "e": "trade",
            "s": "BTCUSDT",
            "p": "50000.00",
            "q": "1.0",
            "T": 1609459200000,
            "m": true,
            "t": 12345
        }"#;

        WebSocketClient::handle_message(trade_msg.to_string(), &tx).await.unwrap();

        let receive_result = timeout(Duration::from_secs(1), rx.recv()).await;
        assert!(receive_result.is_ok());

        if let Ok(Some((market_data, data_quality))) = receive_result {
            assert_eq!(market_data.exchange, "binance");
            assert_eq!(market_data.symbol, "BTCUSDT");
            assert!(matches!(market_data.data_type, MarketDataType::Trade));
            assert!(data_quality.is_valid);
        }
    }

    #[tokio::test]
    async fn test_handle_kline_message() {
        let (tx, mut rx) = mpsc::channel(1);

        let kline_msg = r#"{
            "e": "kline",
            "s": "BTCUSDT",
            "k": {
                "t": 1609459200000,
                "T": 1609459500000,
                "s": "BTCUSDT",
                "i": "5m",
                "o": "50000.00",
                "h": "51000.00",
                "l": "49000.00",
                "c": "50500.00",
                "v": "100.0",
                "q": "5050000.00"
            }
        }"#;

        WebSocketClient::handle_message(kline_msg.to_string(), &tx).await.unwrap();

        let receive_result = timeout(Duration::from_secs(1), rx.recv()).await;
        assert!(receive_result.is_ok());

        if let Ok(Some((market_data, data_quality))) = receive_result {
            assert_eq!(market_data.exchange, "binance");
            assert_eq!(market_data.symbol, "BTCUSDT");
            assert!(matches!(market_data.data_type, MarketDataType::Kline));
            assert!(data_quality.is_valid);
            assert_eq!(market_data.metadata.get("interval").unwrap(), "5m");
        }
    }

    #[tokio::test]
    async fn test_websocket_connection() {
        let mut client = WebSocketClient::new("wss://stream.binance.com:9443/ws");
        assert!(client.connect().await.is_ok());
        assert!(client.close().await.is_ok());
    }

    #[tokio::test]
    async fn test_subscription() {
        let mut client = WebSocketClient::new("wss://stream.binance.com:9443/ws");
        client.connect().await.unwrap();

        // 订阅 BTC/USDT K线数据
        let topic = "btcusdt@kline_1m";
        assert!(client.subscribe_topic(topic).await.is_ok());
        assert!(client.subscriptions.contains(&topic.to_string()));

        // 等待并接收消息
        let timeout_duration = Duration::from_secs(5);
        let result = timeout(timeout_duration, client.receive_message()).await;
        assert!(result.is_ok());

        // 取消订阅
        assert!(client.unsubscribe_topic(topic).await.is_ok());
        assert!(!client.subscriptions.contains(&topic.to_string()));

        client.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_ping_pong() {
        let mut client = WebSocketClient::new("wss://stream.binance.com:9443/ws");
        client.connect().await.unwrap();

        // 订阅一个主题来验证连接
        let topic = "btcusdt@kline_1m";
        assert!(client.subscribe_topic(topic).await.is_ok());

        // 等待接收订阅响应
        let timeout_duration = Duration::from_secs(5);
        let start = Instant::now();
        let mut received_response = false;

        while start.elapsed() < timeout_duration && !received_response {
            if let Ok(Some(msg)) = client.receive_message().await {
                if msg.contains("\"result\"") {
                    received_response = true;
                }
            }
            sleep(Duration::from_millis(100)).await;
        }

        assert!(received_response, "未收到订阅响应");

        // 取消订阅并关闭连接
        assert!(client.unsubscribe_topic(topic).await.is_ok());
        client.close().await.unwrap();
    }
} 