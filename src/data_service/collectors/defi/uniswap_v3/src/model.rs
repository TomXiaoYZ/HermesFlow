use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use ethers::types::{Address, U256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolInfo {
    pub address: String,
    pub token0: String,
    pub token1: String,
    pub fee_tier: u32,
    pub tick_spacing: i32,
    pub liquidity: Decimal,
    pub sqrt_price_x96: Decimal,
    pub tick: i32,
    pub observation_index: u16,
    pub observation_cardinality: u16,
    pub observation_cardinality_next: u16,
    pub fee_protocol: u8,
    pub unlocked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub total_supply: Decimal,
    pub volume_usd: Decimal,
    pub volume_token: Decimal,
    pub tx_count: u64,
    pub pool_count: u32,
    pub total_value_locked: Decimal,
    pub total_value_locked_usd: Decimal,
    pub price_usd: Decimal,
    pub fee_usd: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapData {
    pub pool_address: String,
    pub token0: String,
    pub token1: String,
    pub sender: String,
    pub recipient: String,
    pub origin: String,
    pub amount0: Decimal,
    pub amount1: Decimal,
    pub sqrt_price_x96: Decimal,
    pub liquidity: Decimal,
    pub tick: i32,
    pub fee: Decimal,
    pub tx_hash: String,
    pub log_index: u32,
    pub block_number: u64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub token_id: u128,
    pub owner: String,
    pub pool_address: String,
    pub token0: String,
    pub token1: String,
    pub fee_tier: u32,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub liquidity: Decimal,
    pub fee_growth_inside0_last_x128: Decimal,
    pub fee_growth_inside1_last_x128: Decimal,
    pub tokens_owed0: Decimal,
    pub tokens_owed1: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickData {
    pub pool_address: String,
    pub tick_idx: i32,
    pub liquidity_gross: Decimal,
    pub liquidity_net: Decimal,
    pub price0_x96: Decimal,
    pub price1_x96: Decimal,
    pub fee_growth_outside0_x128: Decimal,
    pub fee_growth_outside1_x128: Decimal,
    pub initialized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoryData {
    pub pool_count: u32,
    pub total_volume_usd: Decimal,
    pub total_fees_usd: Decimal,
    pub total_value_locked_usd: Decimal,
    pub tx_count: u64,
} 