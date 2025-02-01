pub mod error;
pub mod types;
pub mod rest;
pub mod websocket;
pub mod processor;

use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use common::MarketData;
use tracing::error;

pub use error::HuobiError;
pub use types::*;
pub use rest::RestClient;
pub use websocket::WebSocketClient;
pub use processor::HuobiProcessor;

const CHANNEL_BUFFER_SIZE: usize = 1000;

/// Huobi 数据采集器配置
#[derive(Debug, Clone)]
pub struct HuobiCollectorConfig {
    /// REST API 端点地址
    pub rest_endpoint: String,
    /// WebSocket API 端点地址
    pub ws_endpoint: String,
    /// API Key，用于认证（可选）
    pub api_key: Option<String>,
    /// API Secret，用于签名（可选）
    pub api_secret: Option<String>,
}

impl Default for HuobiCollectorConfig {
    fn default() -> Self {
        Self {
            rest_endpoint: "https://api.huobi.pro".to_string(),
            ws_endpoint: "wss://api.huobi.pro/ws".to_string(),
            api_key: None,
            api_secret: None,
        }
    }
}

/// Huobi 数据采集器
pub struct HuobiCollector {
    config: HuobiCollectorConfig,
    rest_client: Arc<RestClient>,
    ws_client: Arc<Mutex<WebSocketClient>>,
    processor: Arc<Mutex<HuobiProcessor>>,
    market_data_tx: Option<mpsc::Sender<MarketData>>,
}

impl HuobiCollector {
    /// 创建新的 Huobi 数据采集器实例
    pub fn new(config: HuobiCollectorConfig) -> Self {
        let rest_client = Arc::new(RestClient::new(
            &config.rest_endpoint,
            config.api_key.as_deref(),
            config.api_secret.as_deref(),
        ));

        let ws_client = Arc::new(Mutex::new(WebSocketClient::new(&config.ws_endpoint)));
        let processor = Arc::new(Mutex::new(HuobiProcessor::new()));

        Self {
            config,
            rest_client,
            ws_client,
            processor,
            market_data_tx: None,
        }
    }

    /// 获取当前配置
    pub fn get_config(&self) -> &HuobiCollectorConfig {
        &self.config
    }

    /// 更新配置
    pub async fn update_config(&mut self, config: HuobiCollectorConfig) -> Result<(), HuobiError> {
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
    pub async fn start(&mut self) -> Result<mpsc::Receiver<MarketData>, HuobiError> {
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
    pub async fn stop(&self) -> Result<(), HuobiError> {
        let mut ws_client = self.ws_client.lock().await;
        ws_client.close().await?;
        Ok(())
    }

    /// 订阅特定交易对的数据
    pub async fn subscribe(&self, symbol: &str) -> Result<(), HuobiError> {
        let mut ws_client = self.ws_client.lock().await;
        
        // 订阅市场概要
        ws_client.subscribe(&format!("market.{}.detail", symbol)).await?;
        // 订阅交易明细
        ws_client.subscribe(&format!("market.{}.trade.detail", symbol)).await?;
        // 订阅深度数据
        ws_client.subscribe(&format!("market.{}.depth.step0", symbol)).await?;
        // 订阅K线数据
        ws_client.subscribe(&format!("market.{}.kline.1min", symbol)).await?;

        // 获取并更新交易对信息
        let symbols = self.rest_client.get_symbols().await?;
        if let Some(symbol_info) = symbols.iter().find(|s| s.symbol == symbol) {
            let mut processor = self.processor.lock().await;
            processor.update_symbol_info(
                symbol.to_string(),
                serde_json::to_value(symbol_info).map_err(|e| HuobiError::ParseError(e.to_string()))?,
            );
        }

        Ok(())
    }

    /// 取消订阅特定交易对的数据
    pub async fn unsubscribe(&self, symbol: &str) -> Result<(), HuobiError> {
        let mut ws_client = self.ws_client.lock().await;
        
        // 取消订阅所有相关频道
        ws_client.unsubscribe(&format!("market.{}.detail", symbol)).await?;
        ws_client.unsubscribe(&format!("market.{}.trade.detail", symbol)).await?;
        ws_client.unsubscribe(&format!("market.{}.depth.step0", symbol)).await?;
        ws_client.unsubscribe(&format!("market.{}.kline.1min", symbol)).await?;

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
        let config = HuobiCollectorConfig::default();
        let collector = HuobiCollector::new(config);
        assert!(collector.market_data_tx.is_none());
    }

    #[tokio::test]
    async fn test_config_operations() {
        let initial_config = HuobiCollectorConfig::default();
        let mut collector = HuobiCollector::new(initial_config.clone());

        // 验证初始配置
        assert_eq!(collector.get_config().rest_endpoint, initial_config.rest_endpoint);
        assert_eq!(collector.get_config().ws_endpoint, initial_config.ws_endpoint);

        // 创建新配置
        let mut new_config = HuobiCollectorConfig::default();
        new_config.rest_endpoint = "https://api-aws.huobi.pro".to_string();
        new_config.ws_endpoint = "wss://api-aws.huobi.pro/ws".to_string();

        // 更新配置
        collector.update_config(new_config.clone()).await.expect("更新配置失败");

        // 验证新配置
        assert_eq!(collector.get_config().rest_endpoint, new_config.rest_endpoint);
        assert_eq!(collector.get_config().ws_endpoint, new_config.ws_endpoint);
    }

    #[tokio::test]
    async fn test_collector_start_stop() {
        let config = HuobiCollectorConfig::default();
        let mut collector = HuobiCollector::new(config);
        
        // 启动收集器
        let _rx = collector.start().await.expect("Failed to start collector");
        
        // 等待一段时间确保连接建立
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // 停止收集器
        collector.stop().await.expect("Failed to stop collector");
    }

    #[tokio::test]
    async fn test_subscribe_unsubscribe() {
        let config = HuobiCollectorConfig::default();
        let mut collector = HuobiCollector::new(config);
        
        // 启动收集器
        let mut rx = collector.start().await.expect("Failed to start collector");
        
        // 等待连接建立
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // 订阅测试交易对
        collector.subscribe("btcusdt").await.expect("Failed to subscribe");
        
        // 等待接收数据
        let timeout_duration = Duration::from_secs(5);
        let result = timeout(timeout_duration, rx.recv()).await;
        assert!(result.is_ok(), "Failed to receive market data within timeout");
        
        // 取消订阅
        collector.unsubscribe("btcusdt").await.expect("Failed to unsubscribe");
        
        // 停止收集器
        collector.stop().await.expect("Failed to stop collector");
    }
} 