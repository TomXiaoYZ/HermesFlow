use std::sync::Arc;
use tokio::sync::Mutex;
use async_trait::async_trait;
use common::{
    DataCollector, DataSourceConfig, CollectorStatus, MarketData, DataQuality,
    metrics,
};
use crate::{
    websocket::WebSocketClient,
    rest::RestClient,
    error::OKXError,
};

pub struct OKXCollector {
    ws_client: Arc<Mutex<WebSocketClient>>,
    rest_client: Arc<Mutex<RestClient>>,
    status: Arc<Mutex<CollectorStatus>>,
}

impl OKXCollector {
    pub fn new() -> Self {
        Self {
            ws_client: Arc::new(Mutex::new(WebSocketClient::new(""))),
            rest_client: Arc::new(Mutex::new(RestClient::new(""))),
            status: Arc::new(Mutex::new(CollectorStatus {
                is_connected: false,
                subscribed_topics: vec![],
                last_heartbeat: None,
                error_count: 0,
                message_count: 0,
                metadata: Default::default(),
            })),
        }
    }
}

#[async_trait]
impl DataCollector for OKXCollector {
    async fn init(&mut self, config: DataSourceConfig) -> Result<(), Box<dyn std::error::Error>> {
        // 从配置中获取WebSocket和REST API的endpoint
        let ws_endpoint = config.connection.get("ws_endpoint")
            .ok_or("Missing ws_endpoint in config")?;
        let rest_endpoint = config.connection.get("rest_endpoint")
            .ok_or("Missing rest_endpoint in config")?;

        // 初始化WebSocket客户端
        let mut ws_client = self.ws_client.lock().await;
        *ws_client = WebSocketClient::new(ws_endpoint);

        // 初始化REST客户端
        let mut rest_client = self.rest_client.lock().await;
        *rest_client = RestClient::new(rest_endpoint);

        // 初始化监控
        metrics::init();

        Ok(())
    }

    async fn start(&mut self, tx: tokio::sync::mpsc::Sender<(MarketData, DataQuality)>) -> Result<(), Box<dyn std::error::Error>> {
        // 启动WebSocket连接
        let mut ws_client = self.ws_client.lock().await;
        ws_client.connect().await?;
        ws_client.start(tx).await?;

        // 更新状态
        let mut status = self.status.lock().await;
        status.is_connected = true;
        metrics::update_connection_status("okx", "websocket", true);

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 停止WebSocket连接
        let mut ws_client = self.ws_client.lock().await;
        ws_client.stop().await?;

        // 更新状态
        let mut status = self.status.lock().await;
        status.is_connected = false;
        metrics::update_connection_status("okx", "websocket", false);

        Ok(())
    }

    async fn subscribe(&mut self, topics: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        // 订阅WebSocket主题
        let mut ws_client = self.ws_client.lock().await;
        ws_client.subscribe(topics.clone()).await?;

        // 更新状态
        let mut status = self.status.lock().await;
        status.subscribed_topics.extend(topics);

        Ok(())
    }

    async fn unsubscribe(&mut self, topics: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        // 取消订阅WebSocket主题
        let mut ws_client = self.ws_client.lock().await;
        ws_client.unsubscribe(topics.clone()).await?;

        // 更新状态
        let mut status = self.status.lock().await;
        status.subscribed_topics.retain(|t| !topics.contains(t));

        Ok(())
    }

    async fn get_status(&self) -> Result<CollectorStatus, Box<dyn std::error::Error>> {
        Ok(self.status.lock().await.clone())
    }

    async fn reset(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 停止现有连接
        self.stop().await?;

        // 重置状态
        let mut status = self.status.lock().await;
        *status = CollectorStatus {
            is_connected: false,
            subscribed_topics: vec![],
            last_heartbeat: None,
            error_count: 0,
            message_count: 0,
            metadata: Default::default(),
        };

        Ok(())
    }

    async fn health_check(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let status = self.status.lock().await;
        Ok(status.is_connected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_okx_collector() {
        // 创建收集器
        let mut collector = OKXCollector::new();

        // 创建配置
        let mut connection = std::collections::HashMap::new();
        connection.insert(
            "ws_endpoint".to_string(),
            "wss://ws.okx.com:8443/ws/v5/public".to_string(),
        );
        connection.insert(
            "rest_endpoint".to_string(),
            "https://www.okx.com".to_string(),
        );

        let config = DataSourceConfig {
            name: "okx".to_string(),
            source_type: "crypto".to_string(),
            connection,
            data: Default::default(),
            monitoring: Default::default(),
        };

        // 初始化
        assert!(collector.init(config).await.is_ok());

        // 创建数据通道
        let (tx, _rx) = mpsc::channel(100);

        // 启动
        assert!(collector.start(tx).await.is_ok());

        // 检查状态
        let status = collector.get_status().await.unwrap();
        assert!(status.is_connected);

        // 订阅主题
        let topics = vec!["spot/ticker:BTC-USDT".to_string()];
        assert!(collector.subscribe(topics.clone()).await.is_ok());

        // 检查订阅状态
        let status = collector.get_status().await.unwrap();
        assert!(status.subscribed_topics.contains(&"spot/ticker:BTC-USDT".to_string()));

        // 取消订阅
        assert!(collector.unsubscribe(topics).await.is_ok());

        // 停止
        assert!(collector.stop().await.is_ok());

        // 检查停止状态
        let status = collector.get_status().await.unwrap();
        assert!(!status.is_connected);
    }
} 