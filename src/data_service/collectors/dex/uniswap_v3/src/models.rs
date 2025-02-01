use rust_decimal::Decimal;
use chrono::{DateTime, Utc};

/// Uniswap V3 池子信息
#[derive(Debug, Clone)]
pub struct Pool {
    pub id: String,
    pub token0: String,
    pub token1: String,
    pub fee_tier: u64,
    pub liquidity: Decimal,
    pub sqrt_price_x96: Decimal,
    pub tick: i64,
    pub token0_price: Decimal,
    pub token1_price: Decimal,
    pub updated_at: DateTime<Utc>,
}

/// 代币信息
#[derive(Debug, Clone)]
pub struct Token {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u64,
    pub total_supply: Decimal,
    pub volume: Decimal,
    pub tx_count: u64,
}

/// Tick数据
#[derive(Debug, Clone)]
pub struct TickData {
    pub tick_idx: i64,
    pub liquidity_gross: Decimal,
    pub liquidity_net: Decimal,
    pub price0: Decimal,
    pub price1: Decimal,
}

/// 头寸信息
#[derive(Debug, Clone)]
pub struct Position {
    pub id: String,
    pub owner: String,
    pub pool: String,
    pub tick_lower: i64,
    pub tick_upper: i64,
    pub liquidity: Decimal,
    pub token0_owed: Decimal,
    pub token1_owed: Decimal,
}

/// 池子事件数据
#[derive(Debug, Clone)]
pub enum PoolData {
    /// Swap事件
    Swap {
        sender: String,
        recipient: String,
        amount0: Decimal,
        amount1: Decimal,
        sqrt_price_x96: Decimal,
        liquidity: Decimal,
        tick: i64,
    },
    /// Mint事件（添加流动性）
    Mint {
        sender: String,
        owner: String,
        tick_lower: i64,
        tick_upper: i64,
        amount: Decimal,
        amount0: Decimal,
        amount1: Decimal,
    },
    /// Burn事件（移除流动性）
    Burn {
        owner: String,
        tick_lower: i64,
        tick_upper: i64,
        amount: Decimal,
        amount0: Decimal,
        amount1: Decimal,
    },
    /// Flash事件（闪电贷）
    Flash {
        sender: String,
        recipient: String,
        amount0: Decimal,
        amount1: Decimal,
        paid0: Decimal,
        paid1: Decimal,
    },
    /// Collect事件（收集手续费）
    Collect {
        owner: String,
        recipient: String,
        tick_lower: i64,
        tick_upper: i64,
        amount0: Decimal,
        amount1: Decimal,
    },
} 