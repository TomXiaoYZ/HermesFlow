use std::collections::HashMap;
use chrono::Utc;
use common::{MarketData, DataQuality, Trade};
use rust_decimal::Decimal;

use crate::error::PancakeSwapError;
use crate::models::{PoolInfo, PriceData, LiquidityData, SwapData, FarmData, PredictionData};

/// PancakeSwap数据处理器
#[derive(Default)]
pub struct PancakeSwapProcessor {
    /// 池子信息缓存
    pool_info_cache: HashMap<String, PoolInfo>,
    /// 最新价格缓存
    price_cache: HashMap<String, PriceData>,
}

impl PancakeSwapProcessor {
    /// 创建新的数据处理器
    pub fn new() -> Self {
        Self {
            pool_info_cache: HashMap::new(),
            price_cache: HashMap::new(),
        }
    }

    /// 处理池子信息
    pub fn process_pool_info(&mut self, pool_info: &PoolInfo) -> Result<MarketData, PancakeSwapError> {
        // 更新缓存
        self.pool_info_cache.insert(pool_info.address.clone(), pool_info.clone());

        Ok(MarketData::Custom {
            data_type: "pool_info".to_string(),
            symbol: format!("{}-{}", pool_info.token0, pool_info.token1),
            timestamp: Utc::now(),
            data: serde_json::to_value(pool_info)
                .map_err(|e| PancakeSwapError::ParseError(format!("Failed to serialize pool info: {}", e)))?,
            quality: DataQuality::Real,
        })
    }

    /// 处理价格数据
    pub fn process_price_data(&mut self, price_data: &PriceData) -> Result<MarketData, PancakeSwapError> {
        // 更新缓存
        self.price_cache.insert(price_data.token.clone(), price_data.clone());

        Ok(MarketData::Custom {
            data_type: "price_data".to_string(),
            symbol: price_data.token.clone(),
            timestamp: price_data.timestamp,
            data: serde_json::to_value(price_data)
                .map_err(|e| PancakeSwapError::ParseError(format!("Failed to serialize price data: {}", e)))?,
            quality: DataQuality::Real,
        })
    }

    /// 处理流动性数据
    pub fn process_liquidity_data(&self, liquidity_data: &LiquidityData) -> Result<MarketData, PancakeSwapError> {
        Ok(MarketData::Custom {
            data_type: "liquidity_data".to_string(),
            symbol: liquidity_data.pool.clone(),
            timestamp: liquidity_data.timestamp,
            data: serde_json::to_value(liquidity_data)
                .map_err(|e| PancakeSwapError::ParseError(format!("Failed to serialize liquidity data: {}", e)))?,
            quality: DataQuality::Real,
        })
    }

    /// 处理交易数据
    pub fn process_swap_data(&self, swap_data: &SwapData) -> Result<MarketData, PancakeSwapError> {
        // 获取池子信息
        let pool_info = self.pool_info_cache.get(&swap_data.pool)
            .ok_or_else(|| PancakeSwapError::ProcessError("Pool info not found in cache".to_string()))?;

        // 创建标准化的交易数据
        let trade = Trade {
            symbol: format!("{}-{}", pool_info.token0, pool_info.token1),
            id: swap_data.tx_hash.clone(),
            price: swap_data.price,
            quantity: if swap_data.amount0 > Decimal::ZERO {
                swap_data.amount0
            } else {
                -swap_data.amount0
            },
            timestamp: swap_data.timestamp,
            is_buyer_maker: swap_data.amount0 > Decimal::ZERO,
            quality: DataQuality::Real,
        };

        Ok(MarketData::Trade(trade))
    }

    /// 处理农场数据
    pub fn process_farm_data(&self, farm_data: &FarmData) -> Result<MarketData, PancakeSwapError> {
        Ok(MarketData::Custom {
            data_type: "farm_data".to_string(),
            symbol: format!("{}-{}", farm_data.lp_token, farm_data.reward_token),
            timestamp: farm_data.timestamp,
            data: serde_json::to_value(farm_data)
                .map_err(|e| PancakeSwapError::ParseError(format!("Failed to serialize farm data: {}", e)))?,
            quality: DataQuality::Real,
        })
    }

    /// 处理预测市场数据
    pub fn process_prediction_data(&self, prediction_data: &PredictionData) -> Result<MarketData, PancakeSwapError> {
        Ok(MarketData::Custom {
            data_type: "prediction_data".to_string(),
            symbol: format!("PREDICTION-{}", prediction_data.round_id),
            timestamp: prediction_data.start_time,
            data: serde_json::to_value(prediction_data)
                .map_err(|e| PancakeSwapError::ParseError(format!("Failed to serialize prediction data: {}", e)))?,
            quality: DataQuality::Real,
        })
    }

    /// 处理批量交易数据
    pub fn process_swap_data_batch(&self, swap_data: &[SwapData]) -> Result<Vec<MarketData>, PancakeSwapError> {
        let mut market_data = Vec::new();
        for swap in swap_data {
            market_data.push(self.process_swap_data(swap)?);
        }
        Ok(market_data)
    }
} 