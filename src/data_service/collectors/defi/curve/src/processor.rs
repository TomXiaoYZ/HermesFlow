use std::collections::HashMap;
use chrono::Utc;
use rust_decimal::Decimal;

use common::{MarketData, DataQuality, Trade};
use crate::{
    error::CurveError,
    model::{PoolInfo, PriceData, SwapData, GaugeData, VotingEscrowData, FactoryData},
};

pub struct CurveProcessor {
    pool_info_cache: HashMap<String, PoolInfo>,
    price_cache: HashMap<String, PriceData>,
    gauge_cache: HashMap<String, GaugeData>,
}

impl CurveProcessor {
    pub fn new() -> Self {
        Self {
            pool_info_cache: HashMap::new(),
            price_cache: HashMap::new(),
            gauge_cache: HashMap::new(),
        }
    }

    pub fn process_pool_info(&mut self, pool_info: &PoolInfo) -> Result<MarketData, CurveError> {
        // 更新缓存
        self.pool_info_cache.insert(pool_info.address.clone(), pool_info.clone());

        // 构建市场数据
        Ok(MarketData::Custom {
            symbol: format!("CURVE:POOL:{}", pool_info.name),
            data_type: "pool_info".to_string(),
            timestamp: Utc::now().timestamp_millis(),
            quality: DataQuality::Real,
            data: serde_json::to_value(pool_info)?,
        })
    }

    pub fn process_price_data(&mut self, price_data: &PriceData) -> Result<MarketData, CurveError> {
        // 更新缓存
        self.price_cache.insert(price_data.token_address.clone(), price_data.clone());

        // 构建市场数据
        Ok(MarketData::Custom {
            symbol: format!("CURVE:PRICE:{}", price_data.token_address),
            data_type: "price_data".to_string(),
            timestamp: price_data.timestamp,
            quality: DataQuality::Real,
            data: serde_json::to_value(price_data)?,
        })
    }

    pub fn process_swap_data(&self, swap_data: &SwapData) -> Result<MarketData, CurveError> {
        // 获取池子信息
        let pool_info = self.pool_info_cache.get(&swap_data.pool_address)
            .ok_or_else(|| CurveError::Processing(format!(
                "Pool info not found for address: {}", swap_data.pool_address
            )))?;

        // 构建交易数据
        let trade = Trade {
            id: swap_data.tx_hash.clone(),
            exchange: "curve".to_string(),
            pair: format!("{}-{}", swap_data.token_in, swap_data.token_out),
            price: swap_data.amount_out / swap_data.amount_in,
            quantity: swap_data.amount_in,
            side: "sell".to_string(), // 或根据实际情况确定
            timestamp: swap_data.timestamp,
            fee: Some(swap_data.fee),
            fee_currency: Some(swap_data.token_in.clone()),
            block_number: Some(swap_data.block_number),
            tx_hash: Some(swap_data.tx_hash.clone()),
        };

        Ok(MarketData::Trade(trade))
    }

    pub fn process_gauge_data(&mut self, gauge_data: &GaugeData) -> Result<MarketData, CurveError> {
        // 更新缓存
        self.gauge_cache.insert(gauge_data.gauge_address.clone(), gauge_data.clone());

        // 构建市场数据
        Ok(MarketData::Custom {
            symbol: format!("CURVE:GAUGE:{}", gauge_data.gauge_address),
            data_type: "gauge_data".to_string(),
            timestamp: Utc::now().timestamp_millis(),
            quality: DataQuality::Real,
            data: serde_json::to_value(gauge_data)?,
        })
    }

    pub fn process_voting_escrow_data(&self, ve_data: &VotingEscrowData) -> Result<MarketData, CurveError> {
        Ok(MarketData::Custom {
            symbol: format!("CURVE:VOTING:{}", ve_data.user_address),
            data_type: "voting_escrow_data".to_string(),
            timestamp: Utc::now().timestamp_millis(),
            quality: DataQuality::Real,
            data: serde_json::to_value(ve_data)?,
        })
    }

    pub fn process_factory_data(&self, factory_data: &FactoryData) -> Result<MarketData, CurveError> {
        Ok(MarketData::Custom {
            symbol: "CURVE:FACTORY".to_string(),
            data_type: "factory_data".to_string(),
            timestamp: factory_data.last_pool_timestamp,
            quality: DataQuality::Real,
            data: serde_json::to_value(factory_data)?,
        })
    }
} 