use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use ethers::types::{Address, U256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolInfo {
    pub address: String,
    pub name: String,
    pub coins: Vec<String>,
    pub underlying_coins: Vec<String>,
    pub balances: Vec<Decimal>,
    pub a: Decimal,
    pub fee: Decimal,
    pub admin_fee: Decimal,
    pub virtual_price: Decimal,
    pub total_supply: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub pool_address: String,
    pub token_address: String,
    pub price_usd: Decimal,
    pub volume_24h: Decimal,
    pub tvl: Decimal,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapData {
    pub pool_address: String,
    pub token_in: String,
    pub token_out: String,
    pub amount_in: Decimal,
    pub amount_out: Decimal,
    pub fee: Decimal,
    pub tx_hash: String,
    pub block_number: u64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugeData {
    pub gauge_address: String,
    pub pool_address: String,
    pub total_supply: Decimal,
    pub working_supply: Decimal,
    pub relative_weight: Decimal,
    pub inflation_rate: Decimal,
    pub reward_tokens: Vec<String>,
    pub reward_rates: Vec<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingEscrowData {
    pub user_address: String,
    pub locked_amount: Decimal,
    pub unlock_time: i64,
    pub voting_power: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoryData {
    pub implementation: String,
    pub pool_count: u32,
    pub last_pool_address: String,
    pub last_pool_timestamp: i64,
} 