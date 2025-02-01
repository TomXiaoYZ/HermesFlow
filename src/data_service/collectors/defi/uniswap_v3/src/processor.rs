use std::collections::HashMap;
use chrono::Utc;
use rust_decimal::Decimal;

use common::{MarketData, DataQuality, Trade};
use crate::{
    error::UniswapV3Error,
    model::{PoolInfo, TokenInfo, SwapData, Position, TickData, FactoryData},
};

pub struct UniswapV3Processor {
    pool_info_cache: HashMap<String, PoolInfo>,
    token_info_cache: HashMap<String, TokenInfo>,
    position_cache: HashMap<u128, Position>,
    tick_cache: HashMap<(String, i32), TickData>,
}

impl UniswapV3Processor {
    pub fn new() -> Self {
        Self {
            pool_info_cache: HashMap::new(),
            token_info_cache: HashMap::new(),
            position_cache: HashMap::new(),
            tick_cache: HashMap::new(),
        }
    }

    pub fn process_pool_info(&mut self, pool_info: &PoolInfo) -> Result<MarketData, UniswapV3Error> {
        // 更新缓存
        self.pool_info_cache.insert(pool_info.address.clone(), pool_info.clone());

        // 构建市场数据
        Ok(MarketData::Custom {
            symbol: format!("UNISWAP_V3:POOL:{}-{}", pool_info.token0, pool_info.token1),
            data_type: "pool_info".to_string(),
            timestamp: Utc::now().timestamp_millis(),
            quality: DataQuality::Real,
            data: serde_json::to_value(pool_info)?,
        })
    }

    pub fn process_token_info(&mut self, token_info: &TokenInfo) -> Result<MarketData, UniswapV3Error> {
        // 更新缓存
        self.token_info_cache.insert(token_info.address.clone(), token_info.clone());

        // 构建市场数据
        Ok(MarketData::Custom {
            symbol: format!("UNISWAP_V3:TOKEN:{}", token_info.symbol),
            data_type: "token_info".to_string(),
            timestamp: Utc::now().timestamp_millis(),
            quality: DataQuality::Real,
            data: serde_json::to_value(token_info)?,
        })
    }

    pub fn process_swap_data(&self, swap_data: &SwapData) -> Result<MarketData, UniswapV3Error> {
        // 获取池子信息
        let pool_info = self.pool_info_cache.get(&swap_data.pool_address)
            .ok_or_else(|| UniswapV3Error::Processing(format!(
                "Pool info not found for address: {}", swap_data.pool_address
            )))?;

        // 构建交易数据
        let trade = Trade {
            id: format!("{}-{}", swap_data.tx_hash, swap_data.log_index),
            exchange: "uniswap_v3".to_string(),
            pair: format!("{}-{}", swap_data.token0, swap_data.token1),
            price: if swap_data.amount0 != Decimal::ZERO {
                swap_data.amount1 / swap_data.amount0
            } else {
                Decimal::ZERO
            },
            quantity: swap_data.amount0.abs(),
            side: if swap_data.amount0 > Decimal::ZERO { "buy" } else { "sell" }.to_string(),
            timestamp: swap_data.timestamp,
            fee: Some(swap_data.fee),
            fee_currency: Some(swap_data.token0.clone()),
            block_number: Some(swap_data.block_number),
            tx_hash: Some(swap_data.tx_hash.clone()),
        };

        Ok(MarketData::Trade(trade))
    }

    pub fn process_position(&mut self, position: &Position) -> Result<MarketData, UniswapV3Error> {
        // 更新缓存
        self.position_cache.insert(position.token_id, position.clone());

        // 构建市场数据
        Ok(MarketData::Custom {
            symbol: format!("UNISWAP_V3:POSITION:{}", position.token_id),
            data_type: "position".to_string(),
            timestamp: Utc::now().timestamp_millis(),
            quality: DataQuality::Real,
            data: serde_json::to_value(position)?,
        })
    }

    pub fn process_tick_data(&mut self, tick_data: &TickData) -> Result<MarketData, UniswapV3Error> {
        // 更新缓存
        self.tick_cache.insert(
            (tick_data.pool_address.clone(), tick_data.tick_idx),
            tick_data.clone(),
        );

        // 构建市场数据
        Ok(MarketData::Custom {
            symbol: format!("UNISWAP_V3:TICK:{}:{}", tick_data.pool_address, tick_data.tick_idx),
            data_type: "tick_data".to_string(),
            timestamp: Utc::now().timestamp_millis(),
            quality: DataQuality::Real,
            data: serde_json::to_value(tick_data)?,
        })
    }

    pub fn process_factory_data(&self, factory_data: &FactoryData) -> Result<MarketData, UniswapV3Error> {
        Ok(MarketData::Custom {
            symbol: "UNISWAP_V3:FACTORY".to_string(),
            data_type: "factory_data".to_string(),
            timestamp: Utc::now().timestamp_millis(),
            quality: DataQuality::Real,
            data: serde_json::to_value(factory_data)?,
        })
    }
} 