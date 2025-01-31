pub mod error;
pub mod types;
pub mod rest;
pub mod websocket;
pub mod processor;

use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use common::{MarketData, DataQuality, MarketDataType, CollectorError};
use tracing::{info, error};

pub use error::OkxError;
pub use types::*;
pub use rest::RestClient;
pub use websocket::WebSocketClient;
pub use processor::OkxProcessor;

const CHANNEL_BUFFER_SIZE: usize = 1000;

/// OKX 数据采集器配置
/// 
/// 用于配置 OKX 数据采集器的连接参数和认证信息。
/// 
/// # 示例
/// ```
/// use okx::OkxCollectorConfig;
/// 
/// let config = OkxCollectorConfig {
///     rest_endpoint: "https://www.okx.com".to_string(),
///     ws_endpoint: "wss://ws.okx.com:8443/ws/v5/public".to_string(),
///     api_key: Some("your-api-key".to_string()),
///     api_secret: Some("your-api-secret".to_string()),
///     passphrase: Some("your-passphrase".to_string()),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct OkxCollectorConfig {
    /// REST API 端点地址
    pub rest_endpoint: String,
    /// WebSocket API 端点地址
    pub ws_endpoint: String,
    /// API Key，用于认证（可选）
    pub api_key: Option<String>,
    /// API Secret，用于签名（可选）
    pub api_secret: Option<String>,
    /// API Passphrase，用于认证（可选）
    pub passphrase: Option<String>,
}

impl Default for OkxCollectorConfig {
    fn default() -> Self {
        Self {
            rest_endpoint: "https://www.okx.com".to_string(),
            ws_endpoint: "wss://ws.okx.com:8443/ws/v5/public".to_string(),
            api_key: None,
            api_secret: None,
            passphrase: None,
        }
    }
}

/// OKX 数据采集器
/// 
/// 负责从 OKX 交易所收集市场数据，包括行情、交易、深度和K线数据。
/// 支持通过 WebSocket 实时订阅数据，并通过 REST API 获取历史数据。
/// 
/// # 示例
/// ```
/// use okx::{OkxCollector, OkxCollectorConfig};
/// 
/// #[tokio::main]
/// async fn main() {
///     let config = OkxCollectorConfig::default();
///     let mut collector = OkxCollector::new(config);
///     
///     // 启动收集器
///     let rx = collector.start().await.unwrap();
///     
///     // 订阅 BTC-USDT 交易对
///     collector.subscribe("BTC-USDT").await.unwrap();
///     
///     // 处理接收到的数据
///     while let Some(market_data) = rx.recv().await {
///         println!("Received: {:?}", market_data);
///     }
/// }
/// ```
pub struct OkxCollector {
    config: OkxCollectorConfig,
    rest_client: Arc<RestClient>,
    ws_client: Arc<Mutex<WebSocketClient>>,
    processor: Arc<Mutex<OkxProcessor>>,
    market_data_tx: Option<mpsc::Sender<MarketData>>,
}

impl OkxCollector {
    /// 创建新的 OKX 数据采集器实例
    /// 
    /// # 参数
    /// * `config` - 采集器配置
    /// 
    /// # 返回值
    /// 返回配置好的 OKX 数据采集器实例
    pub fn new(config: OkxCollectorConfig) -> Self {
        let rest_client = Arc::new(RestClient::new(
            &config.rest_endpoint,
            config.api_key.as_deref(),
            config.api_secret.as_deref(),
            config.passphrase.as_deref(),
        ));

        let ws_client = Arc::new(Mutex::new(WebSocketClient::new(&config.ws_endpoint)));
        let processor = Arc::new(Mutex::new(OkxProcessor::new()));

        Self {
            config,
            rest_client,
            ws_client,
            processor,
            market_data_tx: None,
        }
    }

    /// 启动数据采集
    /// 
    /// 建立 WebSocket 连接并开始接收数据。返回一个接收通道，用于获取采集到的市场数据。
    /// 
    /// # 返回值
    /// * `Ok(Receiver<MarketData>)` - 成功时返回市场数据接收通道
    /// * `Err(OkxError)` - 启动失败时返回错误
    /// 
    /// # 错误
    /// 当 WebSocket 连接失败或无法创建消息处理任务时返回错误
    pub async fn start(&mut self) -> Result<mpsc::Receiver<MarketData>, OkxError> {
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
    /// 
    /// 关闭 WebSocket 连接并停止所有数据采集活动。
    /// 
    /// # 返回值
    /// * `Ok(())` - 成功停止
    /// * `Err(OkxError)` - 停止过程中出现错误
    pub async fn stop(&self) -> Result<(), OkxError> {
        let mut ws_client = self.ws_client.lock().await;
        ws_client.close().await?;
        Ok(())
    }

    /// 订阅特定交易对的数据
    /// 
    /// 订阅指定交易对的行情、交易、深度和K线数据。同时获取并更新交易对的基本信息。
    /// 
    /// # 参数
    /// * `symbol` - 交易对名称，例如 "BTC-USDT"
    /// 
    /// # 返回值
    /// * `Ok(())` - 订阅成功
    /// * `Err(OkxError)` - 订阅失败
    /// 
    /// # 错误
    /// 当 WebSocket 订阅失败或无法获取交易对信息时返回错误
    pub async fn subscribe(&self, symbol: &str) -> Result<(), OkxError> {
        let mut ws_client = self.ws_client.lock().await;
        
        // 订阅Ticker
        ws_client.subscribe("tickers", symbol).await?;
        // 订阅交易数据
        ws_client.subscribe("trades", symbol).await?;
        // 订阅深度数据
        ws_client.subscribe("books", symbol).await?;
        // 订阅K线数据
        ws_client.subscribe("candle1m", symbol).await?;

        // 获取并更新交易对信息
        let instruments = self.rest_client.get_instruments("SPOT").await?;
        if let Some(instrument) = instruments.iter().find(|i| i.inst_id == symbol) {
            let mut processor = self.processor.lock().await;
            processor.update_symbol_info(
                symbol.to_string(),
                serde_json::to_value(instrument).map_err(|e| OkxError::ParseError(e.to_string()))?,
            );
        }

        Ok(())
    }

    /// 取消订阅特定交易对的数据
    /// 
    /// 取消订阅指定交易对的所有数据频道。
    /// 
    /// # 参数
    /// * `symbol` - 交易对名称，例如 "BTC-USDT"
    /// 
    /// # 返回值
    /// * `Ok(())` - 取消订阅成功
    /// * `Err(OkxError)` - 取消订阅失败
    pub async fn unsubscribe(&self, symbol: &str) -> Result<(), OkxError> {
        let mut ws_client = self.ws_client.lock().await;
        
        // 取消订阅所有相关频道
        ws_client.unsubscribe("tickers", symbol).await?;
        ws_client.unsubscribe("trades", symbol).await?;
        ws_client.unsubscribe("books", symbol).await?;
        ws_client.unsubscribe("candle1m", symbol).await?;

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
        let config = OkxCollectorConfig::default();
        let collector = OkxCollector::new(config);
        assert!(collector.market_data_tx.is_none());
    }

    #[tokio::test]
    async fn test_collector_start_stop() {
        let config = OkxCollectorConfig::default();
        let mut collector = OkxCollector::new(config);
        
        // 启动收集器
        let rx = collector.start().await.expect("Failed to start collector");
        
        // 等待一段时间确保连接建立
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // 停止收集器
        collector.stop().await.expect("Failed to stop collector");
    }

    #[tokio::test]
    async fn test_subscribe_unsubscribe() {
        let config = OkxCollectorConfig::default();
        let mut collector = OkxCollector::new(config);
        
        // 启动收集器
        let mut rx = collector.start().await.expect("Failed to start collector");
        
        // 等待连接建立
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // 订阅测试交易对
        collector.subscribe("BTC-USDT").await.expect("Failed to subscribe");
        
        // 等待接收数据
        let timeout_duration = Duration::from_secs(5);
        let result = timeout(timeout_duration, rx.recv()).await;
        assert!(result.is_ok(), "Failed to receive market data within timeout");
        
        // 取消订阅
        collector.unsubscribe("BTC-USDT").await.expect("Failed to unsubscribe");
        
        // 停止收集器
        collector.stop().await.expect("Failed to stop collector");
    }
} 