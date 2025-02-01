use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};

/// Curve池子类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PoolType {
    /// 普通池子
    Plain,
    /// 元池子
    Meta,
    /// 加密货币池子
    Crypto,
    /// 工厂池子
    Factory,
}

/// Curve池子信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pool {
    /// 池子地址
    pub address: String,
    /// 池子名称
    pub name: String,
    /// 池子类型
    pub pool_type: PoolType,
    /// 代币地址列表
    pub coins: Vec<String>,
    /// 底层代币地址列表（元池子）
    pub underlying_coins: Option<Vec<String>>,
    /// 代币余额列表
    pub balances: Vec<Decimal>,
    /// 底层代币余额列表（元池子）
    pub underlying_balances: Option<Vec<Decimal>>,
    /// 虚拟价格
    pub virtual_price: Decimal,
    /// A系数
    pub A: u64,
    /// 费率
    pub fee: Decimal,
    /// 管理费率
    pub admin_fee: Decimal,
    /// 最后更新时间
    pub last_updated: DateTime<Utc>,
}

/// 池子状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolState {
    /// 池子地址
    pub address: String,
    /// 代币余额列表
    pub balances: Vec<Decimal>,
    /// 虚拟价格
    pub virtual_price: Decimal,
    /// A系数
    pub A: u64,
    /// 最后更新时间
    pub timestamp: DateTime<Utc>,
}

/// 交易事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    /// 池子地址
    pub pool_address: String,
    /// 交易发起者
    pub trader: String,
    /// 输入代币索引
    pub token_in_index: u8,
    /// 输出代币索引
    pub token_out_index: u8,
    /// 输入金额
    pub amount_in: Decimal,
    /// 输出金额
    pub amount_out: Decimal,
    /// 交易费用
    pub fee: Decimal,
    /// 交易时间
    pub timestamp: DateTime<Utc>,
}

/// 添加流动性事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddLiquidity {
    /// 池子地址
    pub pool_address: String,
    /// 提供者地址
    pub provider: String,
    /// 代币金额列表
    pub token_amounts: Vec<Decimal>,
    /// 获得的LP代币数量
    pub lp_token_amount: Decimal,
    /// 费用
    pub fee: Decimal,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

/// 移除流动性事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveLiquidity {
    /// 池子地址
    pub pool_address: String,
    /// 提供者地址
    pub provider: String,
    /// 代币金额列表
    pub token_amounts: Vec<Decimal>,
    /// 燃烧的LP代币数量
    pub lp_token_amount: Decimal,
    /// 费用
    pub fee: Decimal,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
} 