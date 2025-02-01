use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Graph API响应
#[derive(Debug, Deserialize)]
pub struct GraphResponse<T> {
    pub data: T,
}

/// 池子信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolInfo {
    /// 池子地址
    pub address: String,
    /// 代币0地址
    pub token0: String,
    /// 代币1地址
    pub token1: String,
    /// 手续费率
    pub fee_tier: String,
    /// 流动性
    pub liquidity: String,
    /// 代币0价格
    pub token0_price: String,
    /// 代币1价格
    pub token1_price: String,
    /// 代币0锁仓量
    pub reserve0: String,
    /// 代币1锁仓量
    pub reserve1: String,
}

/// 价格数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    /// 代币地址
    pub token: String,
    /// BNB价格
    pub price_bnb: Decimal,
    /// USD价格
    pub price_usd: Decimal,
    /// 24h价格变化
    pub price_change_24h: Decimal,
    /// 24h交易量
    pub volume_24h: Decimal,
    /// 总锁仓量
    pub tvl: Decimal,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

/// 流动性数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityData {
    /// 池子地址
    pub pool: String,
    /// 总流动性
    pub total_liquidity: Decimal,
    /// 代币0流动性
    pub token0_liquidity: Decimal,
    /// 代币1流动性
    pub token1_liquidity: Decimal,
    /// 代币0未收取手续费
    pub uncollected_fees0: Decimal,
    /// 代币1未收取手续费
    pub uncollected_fees1: Decimal,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

/// 交易数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapData {
    /// 交易哈希
    pub tx_hash: String,
    /// 池子地址
    pub pool: String,
    /// 发送者地址
    pub sender: String,
    /// 接收者地址
    pub recipient: String,
    /// 代币0数量
    pub amount0: Decimal,
    /// 代币1数量
    pub amount1: Decimal,
    /// 价格
    pub price: Decimal,
    /// 手续费
    pub fee: Decimal,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

/// 农场数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FarmData {
    /// 农场ID
    pub farm_id: String,
    /// LP代币地址
    pub lp_token: String,
    /// 奖励代币地址
    pub reward_token: String,
    /// APR
    pub apr: Decimal,
    /// 总质押量
    pub total_staked: Decimal,
    /// 每区块奖励
    pub reward_per_block: Decimal,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

/// 预测市场数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionData {
    /// 回合ID
    pub round_id: u64,
    /// 开始价格
    pub start_price: Decimal,
    /// 结束价格
    pub end_price: Decimal,
    /// 看涨金额
    pub bull_amount: Decimal,
    /// 看跌金额
    pub bear_amount: Decimal,
    /// 开始时间
    pub start_time: DateTime<Utc>,
    /// 结束时间
    pub end_time: DateTime<Utc>,
} 