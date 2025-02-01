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
    /// 是否为底层代币（元池子）
    pub is_underlying: bool,
    /// 代币价格（USD）
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
    /// 虚拟价格
    pub virtual_price: Decimal,
    /// 池子地址
    pub pool_address: String,
} 