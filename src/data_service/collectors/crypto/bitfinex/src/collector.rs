use crate::error::Result;
use crate::models::{ExchangeInfo, Kline, Orderbook, Ticker, Trade};
use crate::rest::{BitfinexRestClient, BitfinexRestConfig};
use crate::websocket::{BitfinexWebsocketClient, BitfinexWebsocketConfig};

use async_trait::async_trait;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::task::JoinHandle;
use std::collections::HashMap;

use crate::error::BitfinexError;

/// 数据收集器配置
#[derive(Debug, Clone)]
pub struct BitfinexCollectorConfig {
    /// REST客户端配置
    pub rest_config: BitfinexRestConfig,
    /// WebSocket客户端配置
    pub ws_config: BitfinexWebsocketConfig,
    /// 订单簿深度
    pub orderbook_depth: Option<u32>,
    /// 最新成交条数
    pub trades_limit: Option<u32>,
    /// K线数据条数
    pub kline_limit: Option<u32>,
}

impl Default for BitfinexCollectorConfig {
    fn default() -> Self {
        Self {
            rest_config: BitfinexRestConfig::default(),
            ws_config: BitfinexWebsocketConfig::default(),
            orderbook_depth: Some(100),
            trades_limit: Some(1000),
            kline_limit: Some(1000),
        }
    }
}

/// 数据收集器
pub struct BitfinexCollector {
    /// 配置信息
    config: BitfinexCollectorConfig,
    /// REST客户端
    rest_client: Arc<BitfinexRestClient>,
    /// WebSocket客户端
    ws_client: Arc<RwLock<Option<BitfinexWebsocketClient>>>,
    /// 订阅的交易对
    subscriptions: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// 数据广播通道
    event_tx: broadcast::Sender<CollectorEvent>,
    /// 数据接收通道
    event_rx: broadcast::Receiver<CollectorEvent>,
    /// WebSocket任务句柄
    ws_task: Arc<RwLock<Option<JoinHandle<()>>>>,
}

/// 收集器事件
#[derive(Debug, Clone)]
pub enum CollectorEvent {
    /// Ticker更新
    TickerUpdate(Ticker),
    /// 订单簿更新
    OrderbookUpdate(Orderbook),
    /// 成交更新
    TradeUpdate(Trade),
    /// K线更新
    KlineUpdate(Kline),
    /// 错误
    Error(String),
}

#[async_trait]
impl BitfinexCollector {
    /// 创建新的数据收集器
    pub fn new(config: BitfinexCollectorConfig) -> Result<Self, BitfinexError> {
        let rest_client = Arc::new(BitfinexRestClient::new(config.rest_config.clone())?);
        let (event_tx, event_rx) = broadcast::channel(1000);

        Ok(Self {
            config,
            rest_client,
            ws_client: Arc::new(RwLock::new(None)),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            event_rx,
            ws_task: Arc::new(RwLock::new(None)),
        })
    }

    /// 启动数据收集
    pub async fn start(&self) -> Result<(), BitfinexError> {
        // 创建WebSocket消息通道
        let (ws_tx, mut ws_rx) = mpsc::unbounded_channel();
        
        // 创建并连接WebSocket客户端
        let ws_client = BitfinexWebsocketClient::new(
            self.config.ws_config.clone(),
            ws_tx,
        ).await?;

        // 保存WebSocket客户端
        *self.ws_client.write().await = Some(ws_client);

        // 启动WebSocket消息处理任务
        let event_tx = self.event_tx.clone();
        let ws_task = tokio::spawn(async move {
            while let Some(msg) = ws_rx.recv().await {
                match msg {
                    Ok(value) => {
                        // 处理WebSocket消息
                        if let Some(event) = Self::process_ws_message(value) {
                            if let Err(e) = event_tx.send(event) {
                                eprintln!("Failed to send event: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        if let Err(e) = event_tx.send(CollectorEvent::Error(e.to_string())) {
                            eprintln!("Failed to send error event: {}", e);
                        }
                    }
                }
            }
        });

        // 保存任务句柄
        *self.ws_task.write().await = Some(ws_task);

        Ok(())
    }

    /// 停止数据收集
    pub async fn stop(&self) -> Result<(), BitfinexError> {
        // 停止WebSocket任务
        if let Some(task) = self.ws_task.write().await.take() {
            task.abort();
        }

        // 清理WebSocket客户端
        *self.ws_client.write().await = None;

        // 清理订阅信息
        self.subscriptions.write().await.clear();

        Ok(())
    }

    /// 订阅交易对数据
    pub async fn subscribe_ticker(&self, symbol: &str) -> Result<(), BitfinexError> {
        // 添加到订阅列表
        let mut subs = self.subscriptions.write().await;
        subs.entry("ticker".to_string())
            .or_insert_with(Vec::new)
            .push(symbol.to_string());

        // 通过WebSocket订阅
        if let Some(ws) = &*self.ws_client.read().await {
            ws.subscribe_ticker(symbol).await?;
        }

        // 获取初始数据
        let ticker = self.rest_client.get_ticker(symbol).await?;
        self.event_tx.send(CollectorEvent::TickerUpdate(ticker))
            .map_err(|e| BitfinexError::InternalError(e.to_string()))?;

        Ok(())
    }

    /// 订阅订单簿数据
    pub async fn subscribe_orderbook(&self, symbol: &str) -> Result<(), BitfinexError> {
        // 添加到订阅列表
        let mut subs = self.subscriptions.write().await;
        subs.entry("orderbook".to_string())
            .or_insert_with(Vec::new)
            .push(symbol.to_string());

        // 通过WebSocket订阅
        if let Some(ws) = &*self.ws_client.read().await {
            ws.subscribe_orderbook(symbol, "P0", "F0", self.config.orderbook_depth.unwrap_or(100))
                .await?;
        }

        // 获取初始数据
        let orderbook = self.rest_client.get_orderbook(symbol, self.config.orderbook_depth).await?;
        self.event_tx.send(CollectorEvent::OrderbookUpdate(orderbook))
            .map_err(|e| BitfinexError::InternalError(e.to_string()))?;

        Ok(())
    }

    /// 订阅成交数据
    pub async fn subscribe_trades(&self, symbol: &str) -> Result<(), BitfinexError> {
        // 添加到订阅列表
        let mut subs = self.subscriptions.write().await;
        subs.entry("trades".to_string())
            .or_insert_with(Vec::new)
            .push(symbol.to_string());

        // 通过WebSocket订阅
        if let Some(ws) = &*self.ws_client.read().await {
            ws.subscribe_trades(symbol).await?;
        }

        // 获取初始数据
        let trades = self.rest_client.get_trades(symbol, self.config.trades_limit).await?;
        for trade in trades {
            self.event_tx.send(CollectorEvent::TradeUpdate(trade))
                .map_err(|e| BitfinexError::InternalError(e.to_string()))?;
        }

        Ok(())
    }

    /// 取消订阅交易对数据
    pub async fn unsubscribe(&self, channel: &str, symbol: &str) -> Result<(), BitfinexError> {
        // 从订阅列表中移除
        let mut subs = self.subscriptions.write().await;
        if let Some(symbols) = subs.get_mut(channel) {
            symbols.retain(|s| s != symbol);
        }

        // 通过WebSocket取消订阅
        if let Some(ws) = &*self.ws_client.read().await {
            // 这里需要找到对应的channel_id
            // TODO: 实现channel_id的管理
        }

        Ok(())
    }

    /// 获取交易所信息
    pub async fn get_exchange_info(&self) -> Result<ExchangeInfo> {
        self.rest_client.get_symbols().await
    }

    /// 获取Ticker数据
    pub async fn get_ticker(&self, symbol: &str) -> Result<Ticker> {
        self.rest_client.get_ticker(symbol).await
    }

    /// 获取订单簿数据
    pub async fn get_orderbook(&self, symbol: &str) -> Result<Orderbook> {
        self.rest_client
            .get_orderbook(symbol, self.config.orderbook_depth.unwrap_or(100))
            .await
    }

    /// 获取最新成交数据
    pub async fn get_trades(&self, symbol: &str) -> Result<Vec<Trade>> {
        self.rest_client
            .get_trades(symbol, self.config.trades_limit.unwrap_or(1000))
            .await
    }

    /// 获取K线数据
    pub async fn get_klines(
        &self,
        symbol: &str,
        interval: &str,
    ) -> Result<Vec<Kline>> {
        self.rest_client
            .get_klines(symbol, interval, self.config.kline_limit.unwrap_or(1000))
            .await
    }

    /// 订阅数据更新
    pub fn subscribe_events(&self) -> broadcast::Receiver<CollectorEvent> {
        self.event_tx.subscribe()
    }

    fn process_ws_message(value: serde_json::Value) -> Option<CollectorEvent> {
        // 根据消息类型处理数据
        if let Some(event_type) = value.get("event") {
            match event_type.as_str() {
                Some("ticker") => {
                    // 处理Ticker更新
                    if let Ok(ticker) = serde_json::from_value(value.clone()) {
                        return Some(CollectorEvent::TickerUpdate(ticker));
                    }
                }
                Some("book") => {
                    // 处理订单簿更新
                    if let Ok(orderbook) = serde_json::from_value(value.clone()) {
                        return Some(CollectorEvent::OrderbookUpdate(orderbook));
                    }
                }
                Some("trades") => {
                    // 处理交易更新
                    if let Ok(trade) = serde_json::from_value(value.clone()) {
                        return Some(CollectorEvent::TradeUpdate(trade));
                    }
                }
                Some("error") => {
                    // 处理错误消息
                    if let Some(msg) = value.get("msg") {
                        return Some(CollectorEvent::Error(msg.to_string()));
                    }
                }
                _ => {}
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_collector() {
        let config = BitfinexCollectorConfig::default();
        let collector = BitfinexCollector::new(config).unwrap();
        
        // 启动收集器
        collector.start().await.unwrap();

        // 订阅事件
        let mut events = collector.subscribe_events();

        // 订阅BTCUSD的Ticker
        collector.subscribe_ticker("BTCUSD").await.unwrap();

        // 等待并处理一些事件
        tokio::spawn(async move {
            while let Ok(event) = events.recv().await {
                match event {
                    CollectorEvent::TickerUpdate(ticker) => {
                        println!("Received ticker update: {:?}", ticker);
                        break;
                    }
                    CollectorEvent::Error(err) => {
                        eprintln!("Error: {}", err);
                        break;
                    }
                    _ => {}
                }
            }
        });

        // 等待一段时间
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        // 停止收集器
        collector.stop().await.unwrap();
    }
}
