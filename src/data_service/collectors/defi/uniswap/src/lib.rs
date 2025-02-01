pub mod error;
pub mod models;
pub mod types;
pub mod client;
pub mod processor;
pub mod contracts;

use std::sync::Arc;
use tokio::sync::RwLock;
use ethers::providers::{Provider, Http};
use common::{MarketData, DataQuality};

use crate::error::UniswapError;
use crate::client::UniswapClient;
use crate::processor::UniswapProcessor;

/// Uniswap数据收集器
/// 
/// 负责从Uniswap协议获取数据，包括：
/// - 价格数据
/// - 流动性数据
/// - 交易数据
/// - 池子信息
pub struct UniswapCollector {
    /// 以太坊节点客户端
    provider: Provider<Http>,
    /// Uniswap客户端
    client: Arc<UniswapClient>,
    /// 数据处理器
    processor: Arc<RwLock<UniswapProcessor>>,
}

impl UniswapCollector {
    /// 创建新的Uniswap数据收集器
    /// 
    /// # 参数
    /// * `rpc_url` - 以太坊节点RPC地址
    /// * `graph_url` - Uniswap Graph API地址
    pub async fn new(rpc_url: &str, graph_url: &str) -> Result<Self, UniswapError> {
        let provider = Provider::<Http>::try_from(rpc_url)
            .map_err(|e| UniswapError::ConnectionError(format!("Failed to connect to Ethereum node: {}", e)))?;

        let client = Arc::new(UniswapClient::new(provider.clone(), graph_url));
        let processor = Arc::new(RwLock::new(UniswapProcessor::new()));

        Ok(Self {
            provider,
            client,
            processor,
        })
    }

    /// 获取池子信息
    /// 
    /// # 参数
    /// * `pool_address` - 池子合约地址
    pub async fn get_pool_info(&self, pool_address: &str) -> Result<MarketData, UniswapError> {
        let pool_data = self.client.get_pool_info(pool_address).await?;
        let market_data = self.processor.read().await.process_pool_info(&pool_data)?;
        Ok(market_data)
    }

    /// 获取价格数据
    /// 
    /// # 参数
    /// * `token_address` - 代币合约地址
    pub async fn get_price_data(&self, token_address: &str) -> Result<MarketData, UniswapError> {
        let price_data = self.client.get_price_data(token_address).await?;
        let market_data = self.processor.read().await.process_price_data(&price_data)?;
        Ok(market_data)
    }

    /// 获取流动性数据
    /// 
    /// # 参数
    /// * `pool_address` - 池子合约地址
    pub async fn get_liquidity_data(&self, pool_address: &str) -> Result<MarketData, UniswapError> {
        let liquidity_data = self.client.get_liquidity_data(pool_address).await?;
        let market_data = self.processor.read().await.process_liquidity_data(&liquidity_data)?;
        Ok(market_data)
    }

    /// 获取交易数据
    /// 
    /// # 参数
    /// * `pool_address` - 池子合约地址
    pub async fn get_swap_data(&self, pool_address: &str) -> Result<MarketData, UniswapError> {
        let swap_data = self.client.get_swap_data(pool_address).await?;
        let market_data = self.processor.read().await.process_swap_data(&swap_data)?;
        Ok(market_data)
    }

    /// 订阅事件
    /// 
    /// # 参数
    /// * `pool_address` - 池子合约地址
    pub async fn subscribe_events(&self, pool_address: &str) -> Result<(), UniswapError> {
        self.client.subscribe_events(pool_address).await
    }
} 