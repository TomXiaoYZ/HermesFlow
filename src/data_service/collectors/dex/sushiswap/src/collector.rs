use std::sync::Arc;
use tokio::sync::Mutex;
use async_trait::async_trait;
use ethers::prelude::*;
use graphql_client::GraphQLQuery;

use common::{
    DataCollector, DataSourceConfig, CollectorStatus, MarketData, DataQuality,
    metrics,
};
use crate::{
    graph::GraphQLClient,
    contract::ContractClient,
    error::SushiSwapError,
};

pub struct SushiSwapCollector {
    graph_client: Arc<Mutex<GraphQLClient>>,
    contract_client: Arc<Mutex<ContractClient>>,
    status: Arc<Mutex<CollectorStatus>>,
}

impl SushiSwapCollector {
    pub fn new() -> Self {
        Self {
            graph_client: Arc::new(Mutex::new(GraphQLClient::new(""))),
            contract_client: Arc::new(Mutex::new(ContractClient::new(""))),
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
impl DataCollector for SushiSwapCollector {
    async fn init(&mut self, config: DataSourceConfig) -> Result<(), Box<dyn std::error::Error>> {
        // 从配置中获取Graph API和合约的endpoint
        let graph_endpoint = config.connection.get("graph_endpoint")
            .ok_or("Missing graph_endpoint in config")?;
        let rpc_endpoint = config.connection.get("rpc_endpoint")
            .ok_or("Missing rpc_endpoint in config")?;

        // 初始化Graph API客户端
        let mut graph_client = self.graph_client.lock().await;
        *graph_client = GraphQLClient::new(graph_endpoint);

        // 初始化合约客户端
        let mut contract_client = self.contract_client.lock().await;
        *contract_client = ContractClient::new(rpc_endpoint);

        // 初始化监控
        metrics::init();

        Ok(())
    }

    async fn start(&mut self, tx: tokio::sync::mpsc::Sender<(MarketData, DataQuality)>) -> Result<(), Box<dyn std::error::Error>> {
        // 启动合约事件监听
        let mut contract_client = self.contract_client.lock().await;
        contract_client.start_listening(tx.clone()).await?;

        // 更新状态
        let mut status = self.status.lock().await;
        status.is_connected = true;
        metrics::update_connection_status("sushiswap", "contract", true);

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 停止合约事件监听
        let mut contract_client = self.contract_client.lock().await;
        contract_client.stop_listening().await?;

        // 更新状态
        let mut status = self.status.lock().await;
        status.is_connected = false;
        metrics::update_connection_status("sushiswap", "contract", false);

        Ok(())
    }

    async fn subscribe(&mut self, topics: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        // 订阅合约事件
        let mut contract_client = self.contract_client.lock().await;
        contract_client.subscribe_events(topics.clone()).await?;

        // 更新状态
        let mut status = self.status.lock().await;
        status.subscribed_topics.extend(topics);

        Ok(())
    }

    async fn unsubscribe(&mut self, topics: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        // 取消订阅合约事件
        let mut contract_client = self.contract_client.lock().await;
        contract_client.unsubscribe_events(topics.clone()).await?;

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
    async fn test_sushiswap_collector() {
        // 创建收集器
        let mut collector = SushiSwapCollector::new();

        // 创建配置
        let mut connection = std::collections::HashMap::new();
        connection.insert(
            "graph_endpoint".to_string(),
            "https://api.thegraph.com/subgraphs/name/sushiswap/exchange".to_string(),
        );
        connection.insert(
            "rpc_endpoint".to_string(),
            "https://eth-mainnet.alchemyapi.io/v2/your-api-key".to_string(),
        );

        let config = DataSourceConfig {
            name: "sushiswap".to_string(),
            source_type: "dex".to_string(),
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
        let topics = vec!["Swap".to_string(), "Mint".to_string(), "Burn".to_string()];
        assert!(collector.subscribe(topics.clone()).await.is_ok());

        // 检查订阅状态
        let status = collector.get_status().await.unwrap();
        assert!(status.subscribed_topics.contains(&"Swap".to_string()));

        // 取消订阅
        assert!(collector.unsubscribe(topics).await.is_ok());

        // 停止
        assert!(collector.stop().await.is_ok());

        // 检查停止状态
        let status = collector.get_status().await.unwrap();
        assert!(!status.is_connected);
    }
} 