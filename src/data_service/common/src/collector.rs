use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;

use crate::{DataQuality, MarketData};

/// 数据源配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceConfig {
    /// 数据源名称
    pub name: String,
    /// 数据源类型
    pub source_type: String,
    /// 连接配置
    pub connection: HashMap<String, String>,
    /// 数据配置
    pub data: HashMap<String, String>,
    /// 监控配置
    pub monitoring: HashMap<String, String>,
}

/// 数据收集器状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorStatus {
    /// 是否已连接
    pub is_connected: bool,
    /// 已订阅的主题
    pub subscribed_topics: Vec<String>,
    /// 最后一次心跳时间
    pub last_heartbeat: Option<i64>,
    /// 错误计数
    pub error_count: u64,
    /// 消息计数
    pub message_count: u64,
    /// 额外状态信息
    pub metadata: HashMap<String, String>,
}

/// 数据收集器trait
#[async_trait]
pub trait DataCollector: Send + Sync {
    /// 初始化收集器
    async fn init(&mut self, config: DataSourceConfig) -> Result<(), Box<dyn std::error::Error>>;

    /// 启动收集器
    async fn start(&mut self, tx: mpsc::Sender<(MarketData, DataQuality)>) -> Result<(), Box<dyn std::error::Error>>;

    /// 停止收集器
    async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    /// 订阅主题
    async fn subscribe(&mut self, topics: Vec<String>) -> Result<(), Box<dyn std::error::Error>>;

    /// 取消订阅主题
    async fn unsubscribe(&mut self, topics: Vec<String>) -> Result<(), Box<dyn std::error::Error>>;

    /// 获取收集器状态
    async fn get_status(&self) -> Result<CollectorStatus, Box<dyn std::error::Error>>;

    /// 重置收集器
    async fn reset(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    /// 健康检查
    async fn health_check(&self) -> Result<bool, Box<dyn std::error::Error>>;
}

/// 数据清洗器trait
#[async_trait]
pub trait DataProcessor: Send + Sync {
    /// 处理数据
    async fn process(&self, data: MarketData) -> Result<MarketData, Box<dyn std::error::Error>>;

    /// 验证数据质量
    async fn validate(&self, data: &MarketData) -> Result<DataQuality, Box<dyn std::error::Error>>;
}

/// 数据源管理器
pub struct DataSourceManager {
    collectors: HashMap<String, Box<dyn DataCollector>>,
    processors: HashMap<String, Box<dyn DataProcessor>>,
    config: HashMap<String, DataSourceConfig>,
}

impl DataSourceManager {
    /// 创建新的数据源管理器
    pub fn new() -> Self {
        Self {
            collectors: HashMap::new(),
            processors: HashMap::new(),
            config: HashMap::new(),
        }
    }

    /// 注册数据收集器
    pub fn register_collector(&mut self, name: String, collector: Box<dyn DataCollector>) {
        self.collectors.insert(name, collector);
    }

    /// 注册数据处理器
    pub fn register_processor(&mut self, name: String, processor: Box<dyn DataProcessor>) {
        self.processors.insert(name, processor);
    }

    /// 加载配置
    pub fn load_config(&mut self, config: HashMap<String, DataSourceConfig>) {
        self.config = config;
    }

    /// 启动所有数据源
    pub async fn start_all(&mut self, tx: mpsc::Sender<(MarketData, DataQuality)>) -> Result<(), Box<dyn std::error::Error>> {
        for (name, collector) in self.collectors.iter_mut() {
            if let Some(config) = self.config.get(name) {
                collector.init(config.clone()).await?;
                collector.start(tx.clone()).await?;
            }
        }
        Ok(())
    }

    /// 停止所有数据源
    pub async fn stop_all(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for collector in self.collectors.values_mut() {
            collector.stop().await?;
        }
        Ok(())
    }

    /// 获取所有数据源状态
    pub async fn get_all_status(&self) -> HashMap<String, Result<CollectorStatus, Box<dyn std::error::Error>>> {
        let mut status = HashMap::new();
        for (name, collector) in &self.collectors {
            status.insert(name.clone(), collector.get_status().await);
        }
        status
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    // 模拟数据收集器
    struct MockCollector {
        status: Arc<Mutex<CollectorStatus>>,
    }

    #[async_trait]
    impl DataCollector for MockCollector {
        async fn init(&mut self, _config: DataSourceConfig) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }

        async fn start(&mut self, _tx: mpsc::Sender<(MarketData, DataQuality)>) -> Result<(), Box<dyn std::error::Error>> {
            let mut status = self.status.lock().await;
            status.is_connected = true;
            Ok(())
        }

        async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            let mut status = self.status.lock().await;
            status.is_connected = false;
            Ok(())
        }

        async fn subscribe(&mut self, topics: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
            let mut status = self.status.lock().await;
            status.subscribed_topics.extend(topics);
            Ok(())
        }

        async fn unsubscribe(&mut self, topics: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
            let mut status = self.status.lock().await;
            status.subscribed_topics.retain(|t| !topics.contains(t));
            Ok(())
        }

        async fn get_status(&self) -> Result<CollectorStatus, Box<dyn std::error::Error>> {
            Ok(self.status.lock().await.clone())
        }

        async fn reset(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            let mut status = self.status.lock().await;
            *status = CollectorStatus {
                is_connected: false,
                subscribed_topics: vec![],
                last_heartbeat: None,
                error_count: 0,
                message_count: 0,
                metadata: HashMap::new(),
            };
            Ok(())
        }

        async fn health_check(&self) -> Result<bool, Box<dyn std::error::Error>> {
            Ok(self.status.lock().await.is_connected)
        }
    }

    #[tokio::test]
    async fn test_data_source_manager() {
        let mut manager = DataSourceManager::new();
        
        // 创建模拟收集器
        let collector = MockCollector {
            status: Arc::new(Mutex::new(CollectorStatus {
                is_connected: false,
                subscribed_topics: vec![],
                last_heartbeat: None,
                error_count: 0,
                message_count: 0,
                metadata: HashMap::new(),
            })),
        };

        // 注册收集器
        manager.register_collector("mock".to_string(), Box::new(collector));

        // 加载配置
        let mut config = HashMap::new();
        config.insert(
            "mock".to_string(),
            DataSourceConfig {
                name: "mock".to_string(),
                source_type: "test".to_string(),
                connection: HashMap::new(),
                data: HashMap::new(),
                monitoring: HashMap::new(),
            },
        );
        manager.load_config(config);

        // 创建数据通道
        let (tx, _rx) = mpsc::channel(100);

        // 测试启动
        assert!(manager.start_all(tx).await.is_ok());

        // 获取状态
        let status = manager.get_all_status().await;
        assert!(status.contains_key("mock"));
        if let Ok(status) = &status["mock"] {
            assert!(status.is_connected);
        }

        // 测试停止
        assert!(manager.stop_all().await.is_ok());
    }
} 