use std::error::Error;
use async_trait::async_trait;
use tokio::sync::mpsc;

use super::{MarketData, DataQuality, CollectorConfig};

#[async_trait]
pub trait DataCollector {
    type Error: Error + Send + Sync + 'static;

    /// 初始化收集器
    async fn init(&mut self, config: CollectorConfig) -> Result<(), Self::Error>;

    /// 连接到数据源
    async fn connect(&mut self) -> Result<(), Self::Error>;

    /// 断开连接
    async fn disconnect(&mut self) -> Result<(), Self::Error>;

    /// 订阅数据流
    async fn subscribe(&mut self, channels: Vec<String>) -> Result<(), Self::Error>;

    /// 取消订阅数据流
    async fn unsubscribe(&mut self, channels: Vec<String>) -> Result<(), Self::Error>;

    /// 启动数据收集
    async fn start(
        &mut self,
        data_tx: mpsc::Sender<(MarketData, DataQuality)>,
    ) -> Result<(), Self::Error>;

    /// 停止数据收集
    async fn stop(&mut self) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait DataProcessor {
    type Error: Error + Send + Sync + 'static;

    /// 处理市场数据
    async fn process(&self, data: MarketData) -> Result<MarketData, Self::Error>;

    /// 验证数据质量
    async fn validate(&self, data: &MarketData) -> Result<DataQuality, Self::Error>;
} 