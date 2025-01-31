use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::collections::HashMap;
use tokio::sync::mpsc;

use super::models::{MarketData, DataQuality};

/// 数据采集配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorConfig {
    pub exchange: String,                    // 交易所名称
    pub api_key: Option<String>,             // API Key
    pub api_secret: Option<String>,          // API Secret
    pub ws_endpoint: String,                 // WebSocket端点
    pub rest_endpoint: String,               // REST API端点
    pub symbols: Vec<String>,                // 交易对列表
    pub channels: Vec<String>,               // 订阅频道
    pub options: HashMap<String, String>,    // 其他配置选项
}

/// 数据采集状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorStatus {
    pub is_connected: bool,                  // 是否已连接
    pub last_received: Option<i64>,          // 最后接收数据时间
    pub subscribed_channels: Vec<String>,    // 已订阅频道
    pub error_count: i64,                    // 错误计数
    pub reconnect_count: i64,                // 重连次数
    pub metadata: HashMap<String, String>,   // 其他状态信息
}

/// 数据采集器接口
#[async_trait]
pub trait DataCollector: Send + Sync {
    type Error: Error + Send + Sync + 'static;

    /// 初始化采集器
    async fn init(&mut self, config: CollectorConfig) -> Result<(), Self::Error>;

    /// 建立连接
    async fn connect(&mut self) -> Result<(), Self::Error>;

    /// 断开连接
    async fn disconnect(&mut self) -> Result<(), Self::Error>;

    /// 订阅数据
    async fn subscribe(&mut self, channels: Vec<String>) -> Result<(), Self::Error>;

    /// 取消订阅
    async fn unsubscribe(&mut self, channels: Vec<String>) -> Result<(), Self::Error>;

    /// 获取采集器状态
    async fn get_status(&self) -> CollectorStatus;

    /// 启动数据采集
    async fn start(
        &mut self,
        tx: mpsc::Sender<(MarketData, DataQuality)>,
    ) -> Result<(), Self::Error>;

    /// 停止数据采集
    async fn stop(&mut self) -> Result<(), Self::Error>;
}

/// 数据处理器接口
#[async_trait]
pub trait DataProcessor: Send + Sync {
    type Error: Error + Send + Sync + 'static;

    /// 处理数据
    async fn process(&self, data: MarketData) -> Result<MarketData, Self::Error>;

    /// 验证数据
    async fn validate(&self, data: &MarketData) -> Result<DataQuality, Self::Error>;
}

/// 数据发布器接口
#[async_trait]
pub trait DataPublisher: Send + Sync {
    type Error: Error + Send + Sync + 'static;

    /// 发布数据
    async fn publish(&self, data: MarketData, quality: DataQuality) -> Result<(), Self::Error>;
}

/// 数据采集管理器
#[async_trait]
pub trait CollectorManager: Send + Sync {
    type Error: Error + Send + Sync + 'static;

    /// 添加采集器
    async fn add_collector<T: DataCollector>(
        &mut self,
        collector: T,
        config: CollectorConfig,
    ) -> Result<(), Self::Error>;

    /// 移除采集器
    async fn remove_collector(&mut self, exchange: &str) -> Result<(), Self::Error>;

    /// 启动所有采集器
    async fn start_all(&mut self) -> Result<(), Self::Error>;

    /// 停止所有采集器
    async fn stop_all(&mut self) -> Result<(), Self::Error>;

    /// 获取所有采集器状态
    async fn get_all_status(&self) -> HashMap<String, CollectorStatus>;
} 