pub mod error;
pub mod types;
pub mod rest;
pub mod websocket;
pub mod processor;

use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use common::MarketData;
use tracing::error;

pub use error::BinanceError;
pub use types::*;
pub use rest::RestClient;
pub use websocket::WebSocketClient;
pub use processor::BinanceProcessor;

const CHANNEL_BUFFER_SIZE: usize = 1000;

/// Binance 数据采集器配置
#[derive(Debug, Clone)]
pub struct BinanceCollectorConfig {
    /// REST API 端点地址
    pub rest_endpoint: String,
    /// WebSocket API 端点地址
    pub ws_endpoint: String,
    /// API Key，用于认证（可选）
    pub api_key: Option<String>,
    /// API Secret，用于签名（可选）
    pub api_secret: Option<String>,
}

impl Default for BinanceCollectorConfig {
    fn default() -> Self {
        Self {
            rest_endpoint: "https://api.binance.com".to_string(),
            ws_endpoint: "wss://stream.binance.com:9443/ws".to_string(),
            api_key: None,
            api_secret: None,
        }
    }
}

/// Binance 数据采集器
pub struct BinanceCollector {
    config: BinanceCollectorConfig,
    rest_client: Arc<RestClient>,
    ws_client: Arc<Mutex<WebSocketClient>>,
    processor: Arc<Mutex<BinanceProcessor>>,
    market_data_tx: Option<mpsc::Sender<MarketData>>,
}

impl BinanceCollector {
    /// 创建新的 Binance 数据采集器实例
    pub fn new(config: BinanceCollectorConfig) -> Self {
        let rest_client = Arc::new(RestClient::new(
            &config.rest_endpoint,
            config.api_key.as_deref(),
            config.api_secret.as_deref(),
        ));

        let ws_client = Arc::new(Mutex::new(WebSocketClient::new(&config.ws_endpoint)));
        let processor = Arc::new(Mutex::new(BinanceProcessor::new()));

        Self {
            config,
            rest_client,
            ws_client,
            processor,
            market_data_tx: None,
        }
    }

    /// 获取当前配置
    pub fn get_config(&self) -> &BinanceCollectorConfig {
        &self.config
    }

    /// 更新配置
    pub async fn update_config(&mut self, config: BinanceCollectorConfig) -> Result<(), BinanceError> {
        // 如果收集器正在运行，需要先停止
        if self.market_data_tx.is_some() {
            self.stop().await?;
        }

        // 更新客户端
        self.rest_client = Arc::new(RestClient::new(
            &config.rest_endpoint,
            config.api_key.as_deref(),
            config.api_secret.as_deref(),
        ));
        self.ws_client = Arc::new(Mutex::new(WebSocketClient::new(&config.ws_endpoint)));

        // 更新配置
        self.config = config;
        Ok(())
    }

    /// 启动数据采集
    pub async fn start(&mut self) -> Result<mpsc::Receiver<MarketData>, BinanceError> {
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
            loop {
                let mut ws_client = ws_client_clone.lock().await;
                match ws_client.receive_message().await {
                    Ok(Some(message)) => {
                        let mut processor = processor_clone.lock().await;
                        if let Ok(Some(market_data)) = processor.process_ws_message(&message).await {
                            if tx_clone.send(market_data).await.is_err() {
                                error!("无法发送市场数据，接收端可能已关闭");
                                break;
                            }
                        }
                    }
                    Ok(None) => continue,
                    Err(e) => {
                        error!("处理WebSocket消息时出错: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }

    /// 停止数据采集
    pub async fn stop(&mut self) -> Result<(), BinanceError> {
        // 关闭WebSocket连接
        let mut ws_client = self.ws_client.lock().await;
        ws_client.close().await?;

        // 清理发送器
        self.market_data_tx = None;

        Ok(())
    }

    /// 订阅特定交易对的数据
    pub async fn subscribe(&self, symbol: &str) -> Result<(), BinanceError> {
        let mut ws_client = self.ws_client.lock().await;
        
        // 订阅各种数据流
        ws_client.subscribe(&format!("{}@trade", symbol.to_lowercase())).await?;
        ws_client.subscribe(&format!("{}@depth20@100ms", symbol.to_lowercase())).await?;
        ws_client.subscribe(&format!("{}@kline_1m", symbol.to_lowercase())).await?;
        ws_client.subscribe(&format!("{}@ticker", symbol.to_lowercase())).await?;

        Ok(())
    }

    /// 取消订阅特定交易对的数据
    pub async fn unsubscribe(&self, symbol: &str) -> Result<(), BinanceError> {
        let mut ws_client = self.ws_client.lock().await;
        
        // 取消订阅所有相关频道
        ws_client.unsubscribe(&format!("{}@trade", symbol.to_lowercase())).await?;
        ws_client.unsubscribe(&format!("{}@depth20@100ms", symbol.to_lowercase())).await?;
        ws_client.unsubscribe(&format!("{}@kline_1m", symbol.to_lowercase())).await?;
        ws_client.unsubscribe(&format!("{}@ticker", symbol.to_lowercase())).await?;

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
        let config = BinanceCollectorConfig::default();
        let collector = BinanceCollector::new(config);
        assert!(collector.market_data_tx.is_none());
    }

    #[tokio::test]
    async fn test_config_operations() {
        let initial_config = BinanceCollectorConfig::default();
        let mut collector = BinanceCollector::new(initial_config.clone());

        // 验证初始配置
        assert_eq!(collector.get_config().rest_endpoint, initial_config.rest_endpoint);
        assert_eq!(collector.get_config().ws_endpoint, initial_config.ws_endpoint);

        // 创建新配置
        let mut new_config = BinanceCollectorConfig::default();
        new_config.rest_endpoint = "https://api1.binance.com".to_string();
        new_config.ws_endpoint = "wss://stream.binance.com:9443/stream".to_string();

        // 更新配置
        collector.update_config(new_config.clone()).await.expect("更新配置失败");

        // 验证新配置
        assert_eq!(collector.get_config().rest_endpoint, new_config.rest_endpoint);
        assert_eq!(collector.get_config().ws_endpoint, new_config.ws_endpoint);
    }

    #[tokio::test]
    async fn test_collector_start_stop() {
        let config = BinanceCollectorConfig::default();
        let mut collector = BinanceCollector::new(config);
        
        // 启动收集器
        let _rx = collector.start().await.expect("Failed to start collector");
        
        // 等待一段时间确保连接建立
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // 停止收集器
        collector.stop().await.expect("Failed to stop collector");
    }
}