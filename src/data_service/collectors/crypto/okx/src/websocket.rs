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
use std::collections::HashSet;

use common::{MarketData, DataQuality, MarketDataType, CollectorError};
use crate::error::OkxError;
use crate::models::{SubscribeRequest, SubscribeArgs, WebSocketResponse};

type WebSocketConnection = WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

const MAX_RECONNECT_ATTEMPTS: u32 = 5;
const INITIAL_RECONNECT_DELAY: Duration = Duration::from_secs(1);
const MAX_RECONNECT_DELAY: Duration = Duration::from_secs(60);
const PING_INTERVAL: Duration = Duration::from_secs(20);
const PONG_TIMEOUT: Duration = Duration::from_secs(5);

/// WebSocket 客户端状态
/// 
/// 用于跟踪 WebSocket 连接的当前状态，包括连接状态、订阅的频道、
/// 重连次数以及最后一次 ping/pong 的时间戳。
#[derive(Debug, Clone)]
pub struct WebSocketState {
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
/// 
/// 负责管理与 OKX WebSocket 服务器的连接，处理消息的发送和接收，
/// 以及自动重连和心跳检测。
/// 
/// # 示例
/// ```
/// use okx::WebSocketClient;
/// 
/// #[tokio::main]
/// async fn main() {
///     let mut client = WebSocketClient::new("wss://ws.okx.com:8443/ws/v5/public");
///     
///     // 连接到服务器
///     client.connect().await.unwrap();
///     
///     // 订阅 BTC-USDT 的行情数据
///     client.subscribe("tickers", "BTC-USDT").await.unwrap();
///     
///     // 接收消息
///     while let Ok(Some(message)) = client.receive_message().await {
///         println!("Received: {}", message);
///     }
/// }
/// ```
pub struct WebSocketClient {
    endpoint: String,
    state: Arc<Mutex<WebSocketState>>,
    ws_stream: Option<WebSocketConnection>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl WebSocketClient {
    /// 创建新的 WebSocket 客户端实例
    /// 
    /// # 参数
    /// * `endpoint` - WebSocket 服务器地址
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

    /// 连接到 WebSocket 服务器
    /// 
    /// 建立与服务器的连接，并初始化连接状态。
    /// 
    /// # 返回值
    /// * `Ok(())` - 连接成功
    /// * `Err(OkxError)` - 连接失败
    /// 
    /// # 错误
    /// 当无法建立 WebSocket 连接时返回错误
    pub async fn connect(&mut self) -> Result<(), OkxError> {
        let url = Url::parse(&self.endpoint)
            .map_err(|e| OkxError::ConfigError(format!("Invalid WebSocket URL: {}", e)))?;

        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| OkxError::WebSocketError(format!("Failed to connect: {}", e)))?;

        self.ws_stream = Some(ws_stream);
        
        let mut state = self.state.lock().await;
        state.is_connected = true;
        state.reconnect_attempts = 0;
        state.last_reconnect = Some(Instant::now());

        Ok(())
    }

    /// 订阅特定频道
    /// 
    /// # 参数
    /// * `channel` - 频道名称，如 "tickers", "trades" 等
    /// * `symbol` - 交易对名称，如 "BTC-USDT"
    /// 
    /// # 返回值
    /// * `Ok(())` - 订阅成功
    /// * `Err(OkxError)` - 订阅失败
    /// 
    /// # 错误
    /// 当 WebSocket 未连接或发送订阅消息失败时返回错误
    pub async fn subscribe(&mut self, channel: &str, inst_id: &str) -> Result<(), OkxError> {
        let request = SubscribeRequest {
            op: "subscribe".to_string(),
            args: vec![SubscribeArgs {
                channel: channel.to_string(),
                inst_id: inst_id.to_string(),
            }],
        };

        let message = serde_json::to_string(&request)
            .map_err(|e| OkxError::ParseError(format!("Failed to serialize subscribe request: {}", e)))?;

        if let Some(ws_stream) = &mut self.ws_stream {
            ws_stream.send(Message::Text(message)).await
                .map_err(|e| OkxError::WebSocketError(format!("Failed to send subscribe request: {}", e)))?;

            let mut state = self.state.lock().await;
            state.subscribed_channels.push(format!("{}:{}", channel, inst_id));
        } else {
            return Err(OkxError::WebSocketError("WebSocket not connected".to_string()));
        }

        Ok(())
    }

    /// 取消订阅特定频道
    /// 
    /// # 参数
    /// * `channel` - 频道名称
    /// * `symbol` - 交易对名称
    /// 
    /// # 返回值
    /// * `Ok(())` - 取消订阅成功
    /// * `Err(OkxError)` - 取消订阅失败
    pub async fn unsubscribe(&mut self, channel: &str, inst_id: &str) -> Result<(), OkxError> {
        let request = json!({
            "op": "unsubscribe",
            "args": [{
                "channel": channel,
                "inst_id": inst_id
            }]
        });

        let message = serde_json::to_string(&request)
            .map_err(|e| OkxError::ParseError(format!("Failed to serialize unsubscribe request: {}", e)))?;

        if let Some(ws_stream) = &mut self.ws_stream {
            ws_stream.send(Message::Text(message)).await
                .map_err(|e| OkxError::WebSocketError(format!("Failed to send unsubscribe request: {}", e)))?;

            let mut state = self.state.lock().await;
            let channel_key = format!("{}:{}", channel, inst_id);
            state.subscribed_channels.retain(|c| c != &channel_key);
        } else {
            return Err(OkxError::WebSocketError("WebSocket not connected".to_string()));
        }

        Ok(())
    }

    /// 发送 ping 消息
    /// 
    /// 用于保持连接活跃的心跳检测。
    /// 
    /// # 返回值
    /// * `Ok(())` - 发送成功
    /// * `Err(OkxError)` - 发送失败
    async fn send_ping(&mut self) -> Result<(), OkxError> {
        if let Some(ws_stream) = &mut self.ws_stream {
            ws_stream.send(Message::Ping(vec![])).await
                .map_err(|e| OkxError::WebSocketError(format!("Failed to send ping: {}", e)))?;

            let mut state = self.state.lock().await;
            state.last_ping = Some(Utc::now().timestamp_millis());
        }

        Ok(())
    }

    /// 处理接收到的消息
    async fn handle_message(&mut self, message: Message) -> Result<Option<MarketData>, OkxError> {
        match message {
            Message::Text(text) => {
                let response: WebSocketResponse = serde_json::from_str(&text)
                    .map_err(|e| OkxError::ParseError(format!("Failed to parse message: {}", e)))?;

                match response.event.as_deref() {
                    Some("subscribe") => {
                        info!("Successfully subscribed to channel");
                        Ok(None)
                    }
                    Some("unsubscribe") => {
                        info!("Successfully unsubscribed from channel");
                        Ok(None)
                    }
                    Some("error") => {
                        error!("Error from server: {:?}", response);
                        Err(OkxError::WebSocketError(format!("Server error: {:?}", response)))
                    }
                    None => {
                        // 处理市场数据
                        if let Some(data) = response.data {
                            let market_data = match response.channel.as_deref() {
                                Some("tickers") => self.process_ticker(data).await?,
                                Some("trades") => self.process_trade(data).await?,
                                Some("books") => self.process_orderbook(data).await?,
                                Some("candles") => self.process_kline(data).await?,
                                _ => return Ok(None),
                            };
                            Ok(Some(market_data))
                        } else {
                            Ok(None)
                        }
                    }
                    _ => Ok(None),
                }
            }
            Message::Binary(data) => {
                // 处理二进制消息（如果有的话）
                warn!("Received binary message, length: {}", data.len());
                Ok(None)
            }
            Message::Ping(_) => {
                if let Some(ws_stream) = &mut self.ws_stream {
                    ws_stream.send(Message::Pong(vec![])).await
                        .map_err(|e| OkxError::WebSocketError(format!("Failed to send pong: {}", e)))?;
                }
                Ok(None)
            }
            Message::Pong(_) => {
                let mut state = self.state.lock().await;
                state.last_pong = Some(Utc::now().timestamp_millis());
                Ok(None)
            }
            Message::Close(frame) => {
                error!("WebSocket closed: {:?}", frame);
                self.handle_disconnect().await?;
                Ok(None)
            }
            _ => Ok(None)
        }
    }

    /// 处理断开连接
    async fn handle_disconnect(&mut self) -> Result<(), OkxError> {
        let mut state = self.state.lock().await;
        state.is_connected = false;
        state.reconnect_attempts += 1;

        if state.reconnect_attempts >= MAX_RECONNECT_ATTEMPTS {
            return Err(OkxError::WebSocketError("Max reconnection attempts reached".to_string()));
        }

        let delay = self.calculate_reconnect_delay(state.reconnect_attempts);
        drop(state); // 释放锁

        tokio::time::sleep(delay).await;
        self.reconnect().await
    }

    /// 重新连接
    async fn reconnect(&mut self) -> Result<(), OkxError> {
        self.connect().await?;
        
        // 重新订阅之前的频道
        let channels = {
            let state = self.state.lock().await;
            state.subscribed_channels.clone()
        };

        for channel in channels {
            let parts: Vec<&str> = channel.split(':').collect();
            if parts.len() == 2 {
                self.subscribe(parts[0], parts[1]).await?;
            }
        }

        Ok(())
    }

    /// 计算重连延迟
    fn calculate_reconnect_delay(&self, attempts: u32) -> Duration {
        let base_delay = INITIAL_RECONNECT_DELAY.as_secs() as u32;
        let delay = base_delay * 2u32.pow(attempts.saturating_sub(1));
        Duration::from_secs(delay.min(MAX_RECONNECT_DELAY.as_secs() as u32) as u64)
    }

    /// 启动消息处理循环
    pub async fn start(&mut self, tx: mpsc::Sender<MarketData>) -> Result<(), OkxError> {
        if self.ws_stream.is_none() {
            self.connect().await?;
        }

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        let ws_stream = self.ws_stream.take()
            .ok_or_else(|| OkxError::WebSocketError("WebSocket not connected".to_string()))?;
        let (mut write, mut read) = ws_stream.split();

        let state = self.state.clone();
        
        // 心跳检测任务
        let heartbeat_state = state.clone();
        let mut heartbeat_interval = tokio::time::interval(PING_INTERVAL);
        
        tokio::spawn(async move {
            loop {
                heartbeat_interval.tick().await;
                
                let should_ping = {
                    let state = heartbeat_state.lock().await;
                    state.is_connected
                };

                if should_ping {
                    if let Err(e) = write.send(Message::Ping(vec![])).await {
                        error!("Failed to send ping: {}", e);
                        break;
                    }
                }
            }
        });

        // 消息处理循环
        loop {
            tokio::select! {
                Some(message) = read.next() => {
                    match message {
                        Ok(msg) => {
                            if let Ok(Some(market_data)) = self.handle_message(msg).await {
                                if let Err(e) = tx.send(market_data).await {
                                    error!("Failed to send market data: {}", e);
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            error!("WebSocket error: {}", e);
                            if let Err(e) = self.handle_disconnect().await {
                                error!("Failed to handle disconnect: {}", e);
                                break;
                            }
                        }
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("Received shutdown signal");
                    break;
                }
            }
        }

        Ok(())
    }

    /// 停止消息处理
    pub async fn stop(&mut self) -> Result<(), OkxError> {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(()).await;
        }

        if let Some(ws_stream) = &mut self.ws_stream {
            ws_stream.close(None).await
                .map_err(|e| OkxError::WebSocketError(format!("Failed to close connection: {}", e)))?;
        }

        let mut state = self.state.lock().await;
        state.is_connected = false;
        state.subscribed_channels.clear();

        Ok(())
    }

    /// 接收 WebSocket 消息
    /// 
    /// 接收并处理来自服务器的消息，包括文本消息、ping/pong 和关闭帧。
    /// 
    /// # 返回值
    /// * `Ok(Some(String))` - 接收到文本消息
    /// * `Ok(None)` - 接收到控制消息（如 ping/pong）
    /// * `Err(OkxError)` - 接收消息失败
    pub async fn receive_message(&mut self) -> Result<Option<String>, OkxError> {
        if let Some(ws_stream) = &mut self.ws_stream {
            match ws_stream.next().await {
                Some(Ok(message)) => {
                    match message {
                        Message::Text(text) => Ok(Some(text)),
                        Message::Ping(_) => {
                            ws_stream.send(Message::Pong(vec![])).await
                                .map_err(|e| OkxError::WebSocketError(format!("Failed to send pong: {}", e)))?;
                            Ok(None)
                        }
                        Message::Pong(_) => {
                            let mut state = self.state.lock().await;
                            state.last_pong = Some(Utc::now().timestamp_millis());
                            Ok(None)
                        }
                        Message::Close(frame) => {
                            error!("WebSocket closed: {:?}", frame);
                            let mut state = self.state.lock().await;
                            state.is_connected = false;
                            Err(OkxError::WebSocketError("Connection closed".to_string()))
                        }
                        _ => Ok(None),
                    }
                }
                Some(Err(e)) => {
                    Err(OkxError::WebSocketError(format!("WebSocket error: {}", e)))
                }
                None => {
                    Err(OkxError::WebSocketError("WebSocket stream ended".to_string()))
                }
            }
        } else {
            Err(OkxError::WebSocketError("WebSocket not connected".to_string()))
        }
    }

    /// 检查连接状态
    /// 
    /// 验证连接是否正常，包括检查连接标志和 ping/pong 超时。
    /// 
    /// # 返回值
    /// * `Ok(())` - 连接正常
    /// * `Err(OkxError)` - 连接异常或已断开
    pub async fn check_connection(&mut self) -> Result<(), OkxError> {
        let state = self.state.lock().await;
        if !state.is_connected {
            return Err(OkxError::WebSocketError("WebSocket not connected".to_string()));
        }

        if let (Some(last_ping), Some(last_pong)) = (state.last_ping, state.last_pong) {
            if last_ping - last_pong > PONG_TIMEOUT.as_millis() as i64 {
                return Err(OkxError::WebSocketError("Pong timeout".to_string()));
            }
        }

        Ok(())
    }

    /// 处理行情数据
    async fn process_ticker(&self, data: Vec<serde_json::Value>) -> Result<MarketData, OkxError> {
        let ticker: Ticker = serde_json::from_value(data[0].clone())
            .map_err(|e| OkxError::ParseError(format!("Failed to parse ticker: {}", e)))?;

        let market_data = MarketData {
            exchange: "okx".to_string(),
            symbol: ticker.inst_id,
            data_type: MarketDataType::Ticker,
            timestamp: Utc::now(),
            received_at: Utc::now(),
            raw_data: serde_json::to_value(&ticker)?,
            metadata: {
                let mut map = std::collections::HashMap::new();
                map.insert("volume_24h".to_string(), ticker.vol_24h.to_string());
                map
            },
        };

        Ok(market_data)
    }

    /// 处理成交数据
    async fn process_trade(&self, data: Vec<serde_json::Value>) -> Result<MarketData, OkxError> {
        let trade: Trade = serde_json::from_value(data[0].clone())
            .map_err(|e| OkxError::ParseError(format!("Failed to parse trade: {}", e)))?;

        let market_data = MarketData {
            exchange: "okx".to_string(),
            symbol: trade.inst_id,
            data_type: MarketDataType::Trade,
            timestamp: Utc::now(),
            received_at: Utc::now(),
            raw_data: serde_json::to_value(&trade)?,
            metadata: {
                let mut map = std::collections::HashMap::new();
                map.insert("trade_id".to_string(), trade.trade_id);
                map.insert("side".to_string(), trade.side);
                map
            },
        };

        Ok(market_data)
    }

    /// 处理深度数据
    async fn process_orderbook(&self, data: Vec<serde_json::Value>) -> Result<MarketData, OkxError> {
        let orderbook: OrderBook = serde_json::from_value(data[0].clone())
            .map_err(|e| OkxError::ParseError(format!("Failed to parse orderbook: {}", e)))?;

        let market_data = MarketData {
            exchange: "okx".to_string(),
            symbol: orderbook.inst_id,
            data_type: MarketDataType::OrderBook,
            timestamp: Utc::now(),
            received_at: Utc::now(),
            raw_data: serde_json::to_value(&orderbook)?,
            metadata: {
                let mut map = std::collections::HashMap::new();
                map.insert("asks_count".to_string(), orderbook.asks.len().to_string());
                map.insert("bids_count".to_string(), orderbook.bids.len().to_string());
                map
            },
        };

        Ok(market_data)
    }

    /// 处理K线数据
    async fn process_kline(&self, data: Vec<serde_json::Value>) -> Result<MarketData, OkxError> {
        let kline: Kline = serde_json::from_value(data[0].clone())
            .map_err(|e| OkxError::ParseError(format!("Failed to parse kline: {}", e)))?;

        let market_data = MarketData {
            exchange: "okx".to_string(),
            symbol: kline.inst_id,
            data_type: MarketDataType::Kline,
            timestamp: Utc::now(),
            received_at: Utc::now(),
            raw_data: serde_json::to_value(&kline)?,
            metadata: {
                let mut map = std::collections::HashMap::new();
                map.insert("volume".to_string(), kline.vol.to_string());
                map
            },
        };

        Ok(market_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;

    const TEST_WS_ENDPOINT: &str = "wss://ws.okx.com:8443/ws/v5/public";

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
        let result = client.stop().await;
        assert!(result.is_ok(), "Failed to close connection: {:?}", result);
    }

    #[tokio::test]
    async fn test_subscribe_unsubscribe() {
        let mut client = WebSocketClient::new(TEST_WS_ENDPOINT);
        
        // 连接
        client.connect().await.expect("Failed to connect");
        
        // 测试订阅
        let result = client.subscribe("tickers", "BTC-USDT").await;
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
        let result = client.unsubscribe("tickers", "BTC-USDT").await;
        assert!(result.is_ok(), "Failed to unsubscribe: {:?}", result);
        
        // 关闭连接
        client.stop().await.expect("Failed to close connection");
    }

    #[tokio::test]
    async fn test_reconnection() {
        let mut client = WebSocketClient::new(TEST_WS_ENDPOINT);
        
        // 连接
        client.connect().await.expect("Failed to connect");
        
        // 订阅以确保有活跃的频道
        client.subscribe("tickers", "BTC-USDT").await.expect("Failed to subscribe");
        
        // 模拟连接断开
        client.ws_stream = None;
        
        // 测试重连
        let result = client.reconnect().await;
        assert!(result.is_ok(), "Failed to reconnect: {:?}", result);
        
        // 验证重连后的状态
        let result = client.check_connection().await;
        assert!(result.is_ok(), "Connection check failed after reconnect: {:?}", result);
        
        // 验证频道是否自动重新订阅
        let state = client.state.lock().await;
        assert!(state.subscribed_channels.contains(&"tickers:BTC-USDT".to_string()));
        
        // 关闭连接
        drop(state);
        client.stop().await.expect("Failed to close connection");
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
                    Ok(Some(_)) => continue,
                    Ok(None) => {
                        let state = client.state.lock().await;
                        if state.last_pong.is_some() {
                            return Ok(());
                        }
                    }
                    Err(e) => return Err(e),
                }
            }
        }).await;
        assert!(result.is_ok(), "Failed to receive pong within timeout");
        
        // 关闭连接
        client.stop().await.expect("Failed to close connection");
    }
} 