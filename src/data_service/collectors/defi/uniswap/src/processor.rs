use std::collections::HashMap;
use chrono::Utc;
use common::{MarketData, DataQuality, Trade};
use rust_decimal::Decimal;
use crate::error::UniswapError;
use crate::models::{PoolInfo, PriceData, LiquidityData, SwapData};

/// Uniswap数据处理器
#[derive(Default)]
pub struct UniswapProcessor {
    /// 池子信息缓存
    pool_info_cache: HashMap<String, PoolInfo>,
    /// 最新价格缓存
    price_cache: HashMap<String, PriceData>,
}

impl UniswapProcessor {
    /// 创建新的数据处理器
    pub fn new() -> Self {
        Self {
            pool_info_cache: HashMap::new(),
            price_cache: HashMap::new(),
        }
    }

    /// 处理池子信息
    pub fn process_pool_info(&mut self, pool_info: &PoolInfo) -> Result<MarketData, UniswapError> {
        // 更新缓存
        self.pool_info_cache.insert(pool_info.address.clone(), pool_info.clone());

        Ok(MarketData::Custom {
            data_type: "pool_info".to_string(),
            symbol: pool_info.address.clone(),
            timestamp: Utc::now(),
            data: serde_json::to_value(pool_info)
                .map_err(|e| UniswapError::ParseError(format!("Failed to serialize pool info: {}", e)))?,
            quality: DataQuality::Real,
        })
    }

    /// 处理价格数据
    pub fn process_price_data(&mut self, price_data: &PriceData) -> Result<MarketData, UniswapError> {
        // 更新缓存
        self.price_cache.insert(price_data.token.clone(), price_data.clone());

        Ok(MarketData::Custom {
            data_type: "price_data".to_string(),
            symbol: price_data.token.clone(),
            timestamp: price_data.timestamp,
            data: serde_json::to_value(price_data)
                .map_err(|e| UniswapError::ParseError(format!("Failed to serialize price data: {}", e)))?,
            quality: DataQuality::Real,
        })
    }

    /// 处理流动性数据
    pub fn process_liquidity_data(&self, liquidity_data: &LiquidityData) -> Result<MarketData, UniswapError> {
        Ok(MarketData::Custom {
            data_type: "liquidity_data".to_string(),
            symbol: liquidity_data.pool.clone(),
            timestamp: liquidity_data.timestamp,
            data: serde_json::to_value(liquidity_data)
                .map_err(|e| UniswapError::ParseError(format!("Failed to serialize liquidity data: {}", e)))?,
            quality: DataQuality::Real,
        })
    }

    /// 处理交易数据
    pub fn process_swap_data(&self, swap_data: &SwapData) -> Result<MarketData, UniswapError> {
        // 获取池子信息
        let pool_info = self.pool_info_cache.get(&swap_data.pool)
            .ok_or_else(|| UniswapError::ProcessError("Pool info not found in cache".to_string()))?;

        // 确定买卖方向
        let (base_amount, quote_amount) = if swap_data.amount0 > Decimal::ZERO {
            (swap_data.amount0, -swap_data.amount1)
        } else {
            (-swap_data.amount0, swap_data.amount1)
        };

        // 创建标准化的交易数据
        let trade = Trade {
            symbol: format!("{}-{}", pool_info.token0, pool_info.token1),
            id: swap_data.tx_hash.clone(),
            price: swap_data.price,
            quantity: base_amount.abs(),
            timestamp: swap_data.timestamp,
            is_buyer_maker: base_amount > Decimal::ZERO,
            quality: DataQuality::Real,
        };

        Ok(MarketData::Trade(trade))
    }

    /// 处理批量交易数据
    pub fn process_swap_data_batch(&self, swap_data: &[SwapData]) -> Result<Vec<MarketData>, UniswapError> {
        let mut market_data = Vec::new();
        for swap in swap_data {
            market_data.push(self.process_swap_data(swap)?);
        }
        Ok(market_data)
    }
} 