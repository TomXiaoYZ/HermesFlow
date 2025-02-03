use crate::{
    error::UniswapV3Error,
    metrics::{MetricsCollector, Timer},
    models::{Pool, PoolState, SwapEvent, TickData},
    cache::{Cache, LayeredCache, CacheConfig},
};

use ethers::{
    contract::Contract,
    core::types::{Address, U256},
    providers::{Http, Provider},
};
use std::sync::Arc;
use tracing::{debug, error, info};

const POOL_ABI: &str = include_str!("./abi/IUniswapV3Pool.json");
const ERC20_ABI: &str = include_str!("./abi/IERC20.json");

/// Uniswap V3 数据采集器
pub struct UniswapV3Collector {
    provider: Arc<Provider<Http>>,
    cache: Arc<LayeredCache>,
}

impl UniswapV3Collector {
    /// 创建新的采集器实例
    pub fn new(endpoint: &str, cache_config: Option<CacheConfig>) -> Result<Self, UniswapV3Error> {
        info!("创建 Uniswap V3 采集器，endpoint: {}", endpoint);
        let provider = Provider::<Http>::try_from(endpoint)
            .map_err(|e| {
                error!("创建 HTTP Provider 失败: {}", e);
                MetricsCollector::record_error("provider_creation");
                UniswapV3Error::NetworkError(e.to_string())
            })?;

        let cache = Arc::new(LayeredCache::new(cache_config.unwrap_or_default()));
        
        // 启动缓存清理任务
        let cache_clone = cache.clone();
        tokio::spawn(async move {
            LayeredCache::start_cleanup_task(cache_clone).await;
        });

        Ok(Self {
            provider: Arc::new(provider),
            cache,
        })
    }

    /// 获取池子信息
    pub async fn get_pool(&self, address: &str) -> Result<Pool, UniswapV3Error> {
        info!("获取池子信息: {}", address);
        
        // 尝试从缓存获取
        if let Some(pool) = self.cache.memory.get_pool(address).await {
            return Ok(pool);
        }
        
        let timer = Timer::new("pool_data_fetch");
        let result = async {
            let pool_address = address.parse::<Address>()
                .map_err(|e| UniswapV3Error::InvalidAddress(e.to_string()))?;
                
            let pool_contract = Contract::new(pool_address, POOL_ABI.parse().unwrap(), self.provider.clone());
            
            // 获取代币地址
            let rpc_timer = Timer::new("rpc_request");
            let token0: Address = pool_contract.method("token0")?.call().await?;
            let token1: Address = pool_contract.method("token1")?.call().await?;
            let fee: u32 = pool_contract.method("fee")?.call().await?;
            rpc_timer.stop(true);
            
            // 获取代币信息
            let token0_contract = Contract::new(token0, ERC20_ABI.parse().unwrap(), self.provider.clone());
            let token1_contract = Contract::new(token1, ERC20_ABI.parse().unwrap(), self.provider.clone());
            
            let rpc_timer = Timer::new("rpc_request");
            let token0_symbol: String = token0_contract.method("symbol")?.call().await?;
            let token1_symbol: String = token1_contract.method("symbol")?.call().await?;
            let token0_decimals: u8 = token0_contract.method("decimals")?.call().await?;
            let token1_decimals: u8 = token1_contract.method("decimals")?.call().await?;
            rpc_timer.stop(true);
            
            // 获取池子状态
            let rpc_timer = Timer::new("rpc_request");
            let slot0: (U256, i32, u16, u16, u16, u8, bool) = pool_contract.method("slot0")?.call().await?;
            let sqrt_price_x96 = slot0.0;
            let tick = slot0.1;
            rpc_timer.stop(true);
            
            // 计算价格
            let token0_price = self.calculate_price(sqrt_price_x96, token0_decimals, token1_decimals)?;
            let token1_price = 1.0 / token0_price;
            
            // 计算TVL
            let rpc_timer = Timer::new("rpc_request");
            let liquidity: U256 = pool_contract.method("liquidity")?.call().await?;
            rpc_timer.stop(true);
            
            let tvl_usd = self.calculate_tvl(liquidity, sqrt_price_x96, token0_decimals, token1_decimals)?;
            
            // 更新指标
            MetricsCollector::update_pool_liquidity(address, liquidity.as_u128());
            MetricsCollector::update_pool_tvl(address, tvl_usd);
            
            debug!("获取到池子数据: token0={}, token1={}, fee={}", token0_symbol, token1_symbol, fee);
            
            let pool = Pool {
                address: address.to_string(),
                token0: token0.to_string(),
                token1: token1.to_string(),
                fee,
                token0_price,
                token1_price,
                tvl_usd,
                token0_symbol,
                token1_symbol,
                token0_decimals,
                token1_decimals,
            };
            
            // 更新缓存
            self.cache.memory.set_pool(address, pool.clone()).await;
            
            Ok(pool)
        }.await;
        
        match &result {
            Ok(_) => timer.stop(true),
            Err(e) => {
                MetricsCollector::record_error("pool_data_fetch");
                timer.stop(false);
            }
        }
        
        result
    }

    /// 获取流动性分布
    pub async fn get_liquidity_distribution(&self, address: &str) -> Result<Vec<TickData>, UniswapV3Error> {
        info!("获取流动性分布: {}", address);
        
        // 尝试从缓存获取
        if let Some(ticks) = self.cache.memory.get_liquidity(address).await {
            return Ok(ticks);
        }
        
        let timer = Timer::new("pool_data_fetch");
        let result = async {
            let pool_address = address.parse::<Address>()
                .map_err(|e| UniswapV3Error::InvalidAddress(e.to_string()))?;
                
            let pool_contract = Contract::new(pool_address, POOL_ABI.parse().unwrap(), self.provider.clone());
            
            // 获取当前tick
            let rpc_timer = Timer::new("rpc_request");
            let slot0: (U256, i32, u16, u16, u16, u8, bool) = pool_contract.method("slot0")?.call().await?;
            rpc_timer.stop(true);
            
            let current_tick = slot0.1;
            
            // 获取周围的ticks
            let mut ticks = Vec::new();
            let range = 50; // 获取当前tick上下50个tick的数据
            
            for tick_idx in (current_tick - range)..=(current_tick + range) {
                let rpc_timer = Timer::new("rpc_request");
                if let Ok((liquidity_gross, liquidity_net)) = pool_contract.method("ticks")?.call(tick_idx).await {
                    rpc_timer.stop(true);
                    let (price, price0, price1) = self.calculate_tick_prices(tick_idx)?;
                    
                    ticks.push(TickData {
                        tick_idx,
                        liquidity_gross,
                        liquidity_net,
                        price,
                        price0,
                        price1,
                    });
                } else {
                    rpc_timer.stop(false);
                }
            }
            
            debug!("获取到{}个tick的流动性数据", ticks.len());
            
            // 更新缓存
            self.cache.memory.set_liquidity(address, ticks.clone()).await;
            
            Ok(ticks)
        }.await;
        
        match &result {
            Ok(_) => timer.stop(true),
            Err(e) => {
                MetricsCollector::record_error("liquidity_distribution_fetch");
                timer.stop(false);
            }
        }
        
        result
    }

    /// 获取最近的交易
    pub async fn get_recent_swaps(&self, address: &str) -> Result<Vec<SwapEvent>, UniswapV3Error> {
        info!("获取最近交易: {}", address);
        
        // 尝试从缓存获取
        if let Some(swaps) = self.cache.memory.get_swaps(address).await {
            return Ok(swaps);
        }
        
        let timer = Timer::new("pool_data_fetch");
        let result = async {
            let pool_address = address.parse::<Address>()
                .map_err(|e| UniswapV3Error::InvalidAddress(e.to_string()))?;
                
            let pool_contract = Contract::new(pool_address, POOL_ABI.parse().unwrap(), self.provider.clone());
            
            // 获取最近的区块
            let rpc_timer = Timer::new("rpc_request");
            let latest_block = self.provider.get_block_number().await?;
            rpc_timer.stop(true);
            
            let from_block = latest_block.saturating_sub(100.into()); // 获取最近100个区块的数据
            
            // 过滤Swap事件
            let rpc_timer = Timer::new("rpc_request");
            let events = pool_contract
                .event("Swap")?
                .from_block(from_block)
                .query()
                .await?;
            rpc_timer.stop(true);
                
            let mut swaps = Vec::new();
            
            for event in events {
                let rpc_timer = Timer::new("rpc_request");
                let block = self.provider.get_block(event.block_number).await?
                    .ok_or_else(|| UniswapV3Error::BlockNotFound)?;
                rpc_timer.stop(true);
                    
                let swap = SwapEvent {
                    transaction_hash: event.transaction_hash.to_string(),
                    timestamp: block.timestamp.as_u64() as i64,
                    amount0: event.amount0.as_f64(),
                    amount1: event.amount1.as_f64(),
                    sqrt_price_x96: event.sqrt_price_x96.as_u128(),
                    liquidity: event.liquidity.as_u128(),
                    tick: event.tick,
                    sender: event.sender.to_string(),
                    recipient: event.recipient.to_string(),
                };
                
                // 记录交易量
                let volume_usd = if event.amount0.is_positive() {
                    event.amount0.as_f64()
                } else {
                    event.amount1.as_f64()
                };
                MetricsCollector::record_swap_event(address, volume_usd.abs());
                
                swaps.push(swap);
            }
            
            debug!("获取到{}笔交易", swaps.len());
            
            // 更新缓存
            self.cache.memory.set_swaps(address, swaps.clone()).await;
            
            Ok(swaps)
        }.await;
        
        match &result {
            Ok(_) => timer.stop(true),
            Err(e) => {
                MetricsCollector::record_error("recent_swaps_fetch");
                timer.stop(false);
            }
        }
        
        result
    }

    // 辅助方法：计算价格
    fn calculate_price(&self, sqrt_price_x96: U256, decimals0: u8, decimals1: u8) -> Result<f64, UniswapV3Error> {
        let price = (sqrt_price_x96.pow(2) * U256::from(10).pow(decimals0.into()))
            / (U256::from(2).pow(192) * U256::from(10).pow(decimals1.into()));
            
        Ok(price.as_f64())
    }

    // 辅助方法：计算TVL
    fn calculate_tvl(&self, liquidity: U256, sqrt_price_x96: U256, decimals0: u8, decimals1: u8) -> Result<f64, UniswapV3Error> {
        let price = self.calculate_price(sqrt_price_x96, decimals0, decimals1)?;
        let tvl = liquidity.as_f64() * (price + 1.0/price);
        Ok(tvl)
    }

    // 辅助方法：计算tick的价格
    fn calculate_tick_prices(&self, tick: i32) -> Result<(f64, f64, f64), UniswapV3Error> {
        let price = 1.0001f64.powi(tick);
        Ok((price, price, 1.0/price))
    }
} 