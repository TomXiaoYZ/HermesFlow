pub mod error;
pub mod models;
pub mod rest;
pub mod websocket;
pub mod processor;

use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use common::{MarketData, DataQuality, MarketDataType, CollectorError};
use tracing::{info, error};

pub use error::BybitError;
pub use models::*;
pub use rest::*;
pub use websocket::*;
pub use processor::*;

const CHANNEL_BUFFER_SIZE: usize = 1000;

/// ByBit 数据采集器配置
#[derive(Debug, Clone)]
pub struct BybitCollectorConfig {
    /// REST API 端点地址
    pub rest_endpoint: String,
    /// WebSocket API 端点地址
    pub ws_endpoint: String,
    /// API Key（可选）
    pub api_key: Option<String>,
    /// API Secret（可选）
    pub api_secret: Option<String>,
}

impl Default for BybitCollectorConfig {
    fn default() -> Self {
        Self {
            rest_endpoint: "https://api.bybit.com".to_string(),
            ws_endpoint: "wss://stream.bybit.com/v5/public/spot".to_string(),
            api_key: None,
            api_secret: None,
        }
    }
}

/// ByBit 数据采集器
pub struct BybitCollector {
    config: BybitCollectorConfig,
    rest_client: Arc<RestClient>,
    ws_client: Arc<Mutex<WebSocketClient>>,
    processor: Arc<Mutex<BybitProcessor>>,
    market_data_tx: Option<mpsc::Sender<MarketData>>,
}

impl BybitCollector {
    /// 创建新的 ByBit 数据采集器实例
    pub fn new(config: BybitCollectorConfig) -> Self {
        let rest_client = Arc::new(RestClient::new(
            &config.rest_endpoint,
            config.api_key.as_deref(),
            config.api_secret.as_deref(),
        ));

        let ws_client = Arc::new(Mutex::new(WebSocketClient::new(&config.ws_endpoint)));
        let processor = Arc::new(Mutex::new(BybitProcessor::new()));

        Self {
            config,
            rest_client,
            ws_client,
            processor,
            market_data_tx: None,
        }
    }

    /// 启动数据采集
    pub async fn start(&mut self) -> Result<mpsc::Receiver<MarketData>, BybitError> {
        let (tx, rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);
        self.market_data_tx = Some(tx.clone());

        // 连接WebSocket
        let mut ws_client = self.ws_client.lock().await;
        ws_client.connect().await?;

        // 启动消息处理循环
        let ws_client_clone = self.ws_client.clone();
        let processor_clone = self.processor.clone();
        let tx_clone = tx.clone();

        tokio::spawn(async move {
            let mut ws_client = ws_client_clone.lock().await;
            loop {
                match ws_client.receive_message().await {
                    Ok(Some(message)) => {
                        let processor = processor_clone.lock().await;
                        match processor.process_ws_message(&message).await {
                            Ok(Some(market_data)) => {
                                if let Err(e) = tx_clone.send(market_data).await {
                                    error!("Failed to send market data: {}", e);
                                }
                            }
                            Err(e) => {
                                error!("Failed to process message: {}", e);
                            }
                            _ => {}
                        }
                    }
                    Ok(None) => {}
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        // 尝试重连
                        if let Err(e) = ws_client.reconnect().await {
                            error!("Failed to reconnect: {}", e);
                            break;
                        }
                    }
                }
            }
        });

        Ok(rx)
    }

    /// 停止数据采集
    pub async fn stop(&self) -> Result<(), BybitError> {
        let mut ws_client = self.ws_client.lock().await;
        ws_client.close().await?;
        Ok(())
    }

    /// 订阅特定交易对的数据
    pub async fn subscribe(&self, symbol: &str) -> Result<(), BybitError> {
        let mut ws_client = self.ws_client.lock().await;
        
        // 订阅Ticker
        ws_client.subscribe("tickers", symbol).await?;
        // 订阅交易数据
        ws_client.subscribe("trades", symbol).await?;
        // 订阅深度数据
        ws_client.subscribe("orderbook", symbol).await?;
        // 订阅K线数据
        ws_client.subscribe("kline.1m", symbol).await?;

        // 获取并更新交易对信息
        let instruments = self.rest_client.get_instruments("SPOT").await?;
        if let Some(instrument) = instruments.iter().find(|i| i.symbol == symbol) {
            let mut processor = self.processor.lock().await;
            processor.update_symbol_info(
                symbol.to_string(),
                serde_json::to_value(instrument).map_err(|e| BybitError::ParseError(e.to_string()))?,
            );
        }

        Ok(())
    }

    /// 取消订阅特定交易对的数据
    pub async fn unsubscribe(&self, symbol: &str) -> Result<(), BybitError> {
        let mut ws_client = self.ws_client.lock().await;
        
        // 取消订阅所有相关频道
        ws_client.unsubscribe("tickers", symbol).await?;
        ws_client.unsubscribe("trades", symbol).await?;
        ws_client.unsubscribe("orderbook", symbol).await?;
        ws_client.unsubscribe("kline.1m", symbol).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;
    use std::time::Duration;

    #[tokio::test]
    async fn test_collector_creation() {
        let config = BybitCollectorConfig::default();
        let collector = BybitCollector::new(config);
        assert!(collector.market_data_tx.is_none());
    }

    #[tokio::test]
    async fn test_collector_start_stop() {
        let config = BybitCollectorConfig::default();
        let mut collector = BybitCollector::new(config);
        
        // 启动收集器
        let rx = collector.start().await.expect("Failed to start collector");
        
        // 等待一段时间确保连接建立
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // 停止收集器
        collector.stop().await.expect("Failed to stop collector");
    }

    #[tokio::test]
    async fn test_subscribe_unsubscribe() {
        let config = BybitCollectorConfig::default();
        let mut collector = BybitCollector::new(config);
        
        // 启动收集器
        let mut rx = collector.start().await.expect("Failed to start collector");
        
        // 等待连接建立
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // 订阅测试交易对
        collector.subscribe("BTCUSDT").await.expect("Failed to subscribe");
        
        // 等待接收数据
        let timeout_duration = Duration::from_secs(5);
        let result = timeout(timeout_duration, rx.recv()).await;
        assert!(result.is_ok(), "Failed to receive market data within timeout");
        
        // 取消订阅
        collector.unsubscribe("BTCUSDT").await.expect("Failed to unsubscribe");
        
        // 停止收集器
        collector.stop().await.expect("Failed to stop collector");
    }
} 