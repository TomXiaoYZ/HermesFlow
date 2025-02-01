use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

/// 代币信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    /// 代币地址
    pub address: String,
    /// 代币符号
    pub symbol: String,
    /// 代币名称
    pub name: String,
    /// 代币精度
    pub decimals: u8,
    /// 代币总供应量
    pub total_supply: Decimal,
    /// 交易量（USD）
    pub volume_usd: Decimal,
    /// 交易次数
    pub tx_count: u64,
    /// 当前价格（USD）
    pub price_usd: Option<Decimal>,
}

/// LP代币信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LPToken {
    /// 代币地址
    pub address: String,
    /// 代币符号
    pub symbol: String,
    /// 代币名称
    pub name: String,
    /// 代币精度
    pub decimals: u8,
    /// 代币总供应量
    pub total_supply: Decimal,
    /// 交易对地址
    pub pair_address: String,
    /// token0地址
    pub token0: String,
    /// token1地址
    pub token1: String,
    /// token0储备量
    pub reserve0: Decimal,
    /// token1储备量
    pub reserve1: Decimal,
} 