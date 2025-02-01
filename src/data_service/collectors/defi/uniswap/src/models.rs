use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

/// 池子信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolInfo {
    /// 池子地址
    pub address: String,
    /// 代币0地址
    pub token0: String,
    /// 代币1地址
    pub token1: String,
    /// 代币0精度
    pub decimals0: u8,
    /// 代币1精度
    pub decimals1: u8,
    /// 代币0余额
    pub reserve0: String,
    /// 代币1余额
    pub reserve1: String,
    /// 手续费率
    pub fee: u32,
    /// 流动性
    pub liquidity: String,
    /// 最后更新时间
    pub last_update: DateTime<Utc>,
}

/// 价格数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    /// 代币地址
    pub token: String,
    /// 价格（以ETH计价）
    pub price_eth: Decimal,
    /// 价格（以USD计价）
    pub price_usd: Decimal,
    /// 24小时价格变化
    pub price_change_24h: Decimal,
    /// 24小时交易量
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
    /// 未使用流动性
    pub uncollected_fees0: Decimal,
    /// 未使用流动性
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

/// Graph API响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphResponse<T> {
    pub data: T,
}

/// Graph API错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphError {
    pub message: String,
} 