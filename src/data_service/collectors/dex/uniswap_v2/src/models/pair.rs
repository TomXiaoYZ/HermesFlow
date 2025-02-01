use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};

/// Uniswap V2交易对信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pair {
    /// 交易对地址
    pub address: String,
    /// token0地址
    pub token0: String,
    /// token1地址
    pub token1: String,
    /// token0储备量
    pub reserve0: Decimal,
    /// token1储备量
    pub reserve1: Decimal,
    /// 总流动性
    pub total_supply: Decimal,
    /// 累计交易量（USD）
    pub volume_usd: Decimal,
    /// 累计手续费（USD）
    pub fees_usd: Decimal,
    /// 交易对创建时间
    pub created_at: DateTime<Utc>,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
}

/// 交易对状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairState {
    /// 交易对地址
    pub address: String,
    /// token0储备量
    pub reserve0: Decimal,
    /// token1储备量
    pub reserve1: Decimal,
    /// 最后更新时间
    pub timestamp: DateTime<Utc>,
}

/// 交易事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Swap {
    /// 交易对地址
    pub pair_address: String,
    /// 交易发起者
    pub sender: String,
    /// 接收者
    pub to: String,
    /// token0数量（正数表示输入，负数表示输出）
    pub amount0: Decimal,
    /// token1数量（正数表示输入，负数表示输出）
    pub amount1: Decimal,
    /// 交易时间
    pub timestamp: DateTime<Utc>,
}

/// 添加流动性事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mint {
    /// 交易对地址
    pub pair_address: String,
    /// 流动性提供者
    pub sender: String,
    /// token0数量
    pub amount0: Decimal,
    /// token1数量
    pub amount1: Decimal,
    /// 获得的LP代币数量
    pub liquidity: Decimal,
    /// 交易时间
    pub timestamp: DateTime<Utc>,
}

/// 移除流动性事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Burn {
    /// 交易对地址
    pub pair_address: String,
    /// 流动性提供者
    pub sender: String,
    /// 接收者
    pub to: String,
    /// token0数量
    pub amount0: Decimal,
    /// token1数量
    pub amount1: Decimal,
    /// 燃烧的LP代币数量
    pub liquidity: Decimal,
    /// 交易时间
    pub timestamp: DateTime<Utc>,
} 