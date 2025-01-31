use std::error::Error;
use async_trait::async_trait;
use tokio::sync::mpsc;

use common::{
    CollectorConfig, DataQuality, MarketData, DataCollector, CollectorError,
    MarketDataType, OrderBook, Trade, Kline, Ticker
};

pub mod error;
pub mod rest;
pub mod websocket;

use crate::error::BinanceError;
use crate::rest::RestClient;
use crate::websocket::WebSocketClient;

/// Binance数据采集器
pub struct BinanceCollector {
    config: Option<CollectorConfig>,
    ws_client: Option<WebSocketClient>,
    rest_client: Option<RestClient>,
    data_tx: Option<mpsc::Sender<(MarketData, DataQuality)>>,
}

impl BinanceCollector {
    pub fn new() -> Self {
        Self {
            config: None,
            ws_client: None,
            rest_client: None,
            data_tx: None,
        }
    }
}

#[async_trait]
impl DataCollector for BinanceCollector {
    type Error = BinanceError;

    async fn init(&mut self, config: CollectorConfig) -> Result<(), Self::Error> {
        self.config = Some(config.clone());
        self.ws_client = Some(WebSocketClient::new(&config.endpoint));
        self.rest_client = Some(RestClient::new(
            &config.endpoint,
            config.api_key,
            config.api_secret,
        ));
        Ok(())
    }

    async fn connect(&mut self) -> Result<(), Self::Error> {
        if let Some(ws_client) = &mut self.ws_client {
            ws_client.connect().await.map_err(|e| CollectorError::WebSocketError(e.to_string()))?;
        }
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), Self::Error> {
        if let Some(ws_client) = &mut self.ws_client {
            ws_client.disconnect().await.map_err(|e| CollectorError::WebSocketError(e.to_string()))?;
        }
        Ok(())
    }

    async fn subscribe(&mut self, channels: Vec<String>) -> Result<(), Self::Error> {
        if let Some(ws_client) = &mut self.ws_client {
            ws_client.subscribe(channels).await.map_err(|e| CollectorError::WebSocketError(e.to_string()))?;
        }
        Ok(())
    }

    async fn unsubscribe(&mut self, channels: Vec<String>) -> Result<(), Self::Error> {
        if let Some(ws_client) = &mut self.ws_client {
            ws_client.unsubscribe(channels).await.map_err(|e| CollectorError::WebSocketError(e.to_string()))?;
        }
        Ok(())
    }

    async fn start(
        &mut self,
        data_tx: mpsc::Sender<(MarketData, DataQuality)>,
    ) -> Result<(), Self::Error> {
        self.data_tx = Some(data_tx.clone());
        if let Some(ws_client) = &mut self.ws_client {
            ws_client.start(data_tx).await.map_err(|e| CollectorError::WebSocketError(e.to_string()))?;
        }
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Self::Error> {
        if let Some(ws_client) = &mut self.ws_client {
            ws_client.stop().await.map_err(|e| CollectorError::WebSocketError(e.to_string()))?;
        }
        Ok(())
    }
}

impl Default for BinanceCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;
    use tokio::time::timeout;
    use std::time::Duration;

    #[tokio::test]
    async fn test_binance_collector_init() {
        let mut collector = BinanceCollector::new();
        let config = CollectorConfig {
            endpoint: "wss://stream.binance.com:9443".to_string(),
            api_key: None,
            api_secret: None,
            symbols: vec!["BTCUSDT".to_string()],
            channels: vec!["btcusdt@trade".to_string()],
            options: Default::default(),
        };

        let result = collector.init(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_binance_collector_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
        let mut collector = BinanceCollector::new();
        let config = CollectorConfig {
            endpoint: "wss://stream.binance.com:9443".to_string(),
            api_key: None,
            api_secret: None,
            symbols: vec!["BTCUSDT".to_string()],
            channels: vec!["btcusdt@trade".to_string()],
            options: Default::default(),
        };

        // 初始化
        collector.init(config).await?;

        // 连接
        collector.connect().await?;

        // 创建数据通道
        let (tx, mut rx) = mpsc::channel(100);

        // 启动收集器
        collector.start(tx).await?;

        // 订阅频道
        collector
            .subscribe(vec!["btcusdt@trade".to_string()])
            .await?;

        // 等待数据，最多等待10秒
        let receive_result = timeout(Duration::from_secs(10), rx.recv()).await;
        match receive_result {
            Ok(Some(_)) => println!("成功接收到数据"),
            Ok(None) => println!("通道已关闭"),
            Err(_) => println!("等待数据超时"),
        }

        // 取消订阅
        collector
            .unsubscribe(vec!["btcusdt@trade".to_string()])
            .await?;

        // 停止收集器
        collector.stop().await?;

        Ok(())
    }
}