use rust_decimal::Decimal;

/// Uniswap V3 池子基本信息
#[derive(Debug, Clone)]
pub struct Pool {
    pub address: String,
    pub token0: String,
    pub token1: String,
    pub fee: u32,
    pub token0_price: f64,
    pub token1_price: f64,
    pub tvl_usd: f64,
    pub token0_symbol: String,
    pub token1_symbol: String,
    pub token0_decimals: u8,
    pub token1_decimals: u8,
}

/// 池子详细数据
#[derive(Debug, Clone)]
pub struct PoolData {
    pub pool: Pool,
    pub token0_decimals: u8,
    pub token1_decimals: u8,
    pub liquidity: u128,
    pub sqrt_price_x96: u128,
    pub tick: i32,
    pub price: Decimal,
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
    pub tick_idx: i32,
    pub liquidity_gross: u128,
    pub liquidity_net: i128,
    pub price: f64,
    pub price0: f64,
    pub price1: f64,
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
pub enum PoolEvent {
    /// Swap事件
    Swap {
        sender: String,
        recipient: String,
        amount0: Decimal,
        amount1: Decimal,
        sqrt_price_x96: Decimal,
        liquidity: Decimal,
        tick: i32,
    },
    /// Mint事件（添加流动性）
    Mint {
        sender: String,
        owner: String,
        tick_lower: i32,
        tick_upper: i32,
        amount: Decimal,
        amount0: Decimal,
        amount1: Decimal,
    },
    /// Burn事件（移除流动性）
    Burn {
        owner: String,
        tick_lower: i32,
        tick_upper: i32,
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
        tick_lower: i32,
        tick_upper: i32,
        amount0: Decimal,
        amount1: Decimal,
    },
}

/// 池子状态
#[derive(Debug, Clone)]
pub struct PoolState {
    pub liquidity: u128,
    pub sqrt_price_x96: u128,
    pub tick: i32,
    pub observation_index: u16,
    pub observation_cardinality: u16,
    pub observation_cardinality_next: u16,
    pub fee_protocol: u8,
    pub unlocked: bool,
}

/// 交换事件
#[derive(Debug, Clone)]
pub struct SwapEvent {
    pub transaction_hash: String,
    pub timestamp: i64,
    pub amount0: f64,
    pub amount1: f64,
    pub sqrt_price_x96: u128,
    pub liquidity: u128,
    pub tick: i32,
    pub sender: String,
    pub recipient: String,
}

impl Pool {
    pub fn validate_state(&self) -> bool {
        // 验证基本数据
        if self.address.is_empty() || self.token0.is_empty() || self.token1.is_empty() {
            return false;
        }

        // 验证价格和流动性
        if self.token0_price <= 0.0 || self.token1_price <= 0.0 {
            return false;
        }

        if self.tvl_usd < 0.0 {
            return false;
        }

        true
    }
}

impl TickData {
    pub fn validate(&self) -> bool {
        // 验证价格
        if self.price <= 0.0 || self.price0 <= 0.0 || self.price1 <= 0.0 {
            return false;
        }

        // 流动性可以为负（移除流动性的情况）
        true
    }
} 