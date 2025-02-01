use crate::error::UniswapV3Error;
use crate::models::{Pool, Token, TickData, Position, PoolData};
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref ETH_ADDRESS_REGEX: Regex = Regex::new(r"^0x[a-fA-F0-9]{40}$").unwrap();
}

/// 数据验证器
pub struct DataValidator;

impl DataValidator {
    /// 验证以太坊地址格式
    pub fn validate_eth_address(address: &str) -> Result<(), UniswapV3Error> {
        if !ETH_ADDRESS_REGEX.is_match(address) {
            return Err(UniswapV3Error::ValidationError(
                format!("无效的以太坊地址格式: {}", address)
            ));
        }
        Ok(())
    }

    /// 验证时间戳
    pub fn validate_timestamp(timestamp: DateTime<Utc>) -> Result<(), UniswapV3Error> {
        let now = Utc::now();
        if timestamp > now {
            return Err(UniswapV3Error::ValidationError(
                format!("时间戳不能大于当前时间: {}", timestamp)
            ));
        }
        Ok(())
    }

    /// 验证数值范围
    pub fn validate_decimal_range(
        value: Decimal,
        min: Option<Decimal>,
        max: Option<Decimal>,
        field_name: &str,
    ) -> Result<(), UniswapV3Error> {
        if let Some(min_value) = min {
            if value < min_value {
                return Err(UniswapV3Error::ValidationError(
                    format!("{} 不能小于 {}: 当前值 {}", field_name, min_value, value)
                ));
            }
        }
        if let Some(max_value) = max {
            if value > max_value {
                return Err(UniswapV3Error::ValidationError(
                    format!("{} 不能大于 {}: 当前值 {}", field_name, max_value, value)
                ));
            }
        }
        Ok(())
    }

    /// 验证池子数据
    pub fn validate_pool(pool: &Pool) -> Result<(), UniswapV3Error> {
        Self::validate_eth_address(&pool.id)?;
        Self::validate_eth_address(&pool.token0)?;
        Self::validate_eth_address(&pool.token1)?;
        
        // 验证费率范围（Uniswap V3支持的费率：0.01%, 0.05%, 0.3%, 1%）
        let valid_fee_tiers = vec![100, 500, 3000, 10000];
        if !valid_fee_tiers.contains(&pool.fee_tier) {
            return Err(UniswapV3Error::ValidationError(
                format!("无效的费率: {}", pool.fee_tier)
            ));
        }

        Self::validate_decimal_range(pool.liquidity, Some(Decimal::ZERO), None, "流动性")?;
        Self::validate_decimal_range(pool.token0_price, Some(Decimal::ZERO), None, "token0价格")?;
        Self::validate_decimal_range(pool.token1_price, Some(Decimal::ZERO), None, "token1价格")?;
        Self::validate_timestamp(pool.updated_at)?;

        Ok(())
    }

    /// 验证代币数据
    pub fn validate_token(token: &Token) -> Result<(), UniswapV3Error> {
        Self::validate_eth_address(&token.id)?;
        
        if token.symbol.is_empty() {
            return Err(UniswapV3Error::ValidationError("代币符号不能为空".to_string()));
        }
        if token.name.is_empty() {
            return Err(UniswapV3Error::ValidationError("代币名称不能为空".to_string()));
        }

        if token.decimals > 18 {
            return Err(UniswapV3Error::ValidationError(
                format!("代币精度不能大于18: {}", token.decimals)
            ));
        }

        Self::validate_decimal_range(token.total_supply, Some(Decimal::ZERO), None, "总供应量")?;
        Self::validate_decimal_range(token.volume, Some(Decimal::ZERO), None, "交易量")?;

        Ok(())
    }

    /// 验证Tick数据
    pub fn validate_tick_data(tick_data: &TickData) -> Result<(), UniswapV3Error> {
        // Uniswap V3的tick范围是 [-887272, 887272]
        if tick_data.tick_idx < -887272 || tick_data.tick_idx > 887272 {
            return Err(UniswapV3Error::ValidationError(
                format!("Tick索引超出范围: {}", tick_data.tick_idx)
            ));
        }

        Self::validate_decimal_range(tick_data.price0, Some(Decimal::ZERO), None, "price0")?;
        Self::validate_decimal_range(tick_data.price1, Some(Decimal::ZERO), None, "price1")?;

        Ok(())
    }

    /// 验证头寸数据
    pub fn validate_position(position: &Position) -> Result<(), UniswapV3Error> {
        Self::validate_eth_address(&position.owner)?;
        Self::validate_eth_address(&position.pool)?;

        if position.tick_lower >= position.tick_upper {
            return Err(UniswapV3Error::ValidationError(
                format!("无效的tick范围: lower {} >= upper {}", 
                    position.tick_lower, position.tick_upper)
            ));
        }

        Self::validate_decimal_range(position.liquidity, Some(Decimal::ZERO), None, "流动性")?;
        Self::validate_decimal_range(position.token0_owed, Some(Decimal::ZERO), None, "token0待收金额")?;
        Self::validate_decimal_range(position.token1_owed, Some(Decimal::ZERO), None, "token1待收金额")?;

        Ok(())
    }

    /// 验证事件数据
    pub fn validate_pool_data(pool_data: &PoolData) -> Result<(), UniswapV3Error> {
        match pool_data {
            PoolData::Swap { 
                sender, 
                recipient, 
                amount0, 
                amount1, 
                sqrt_price_x96,
                liquidity,
                tick 
            } => {
                Self::validate_eth_address(sender)?;
                Self::validate_eth_address(recipient)?;
                Self::validate_decimal_range(*liquidity, Some(Decimal::ZERO), None, "流动性")?;
                if *tick < -887272 || *tick > 887272 {
                    return Err(UniswapV3Error::ValidationError(
                        format!("Tick超出范围: {}", tick)
                    ));
                }
                Self::validate_decimal_range(*sqrt_price_x96, Some(Decimal::ZERO), None, "sqrt_price_x96")?;
            },
            PoolData::Mint { 
                sender, 
                owner, 
                tick_lower, 
                tick_upper, 
                amount,
                amount0,
                amount1 
            } => {
                Self::validate_eth_address(sender)?;
                Self::validate_eth_address(owner)?;
                if *tick_lower >= *tick_upper {
                    return Err(UniswapV3Error::ValidationError(
                        format!("无效的tick范围: lower {} >= upper {}", tick_lower, tick_upper)
                    ));
                }
                if *tick_lower < -887272 || *tick_lower > 887272 || *tick_upper < -887272 || *tick_upper > 887272 {
                    return Err(UniswapV3Error::ValidationError(
                        format!("Tick超出范围: lower {}, upper {}", tick_lower, tick_upper)
                    ));
                }
                Self::validate_decimal_range(*amount, Some(Decimal::ZERO), None, "数量")?;
                Self::validate_decimal_range(*amount0, Some(Decimal::ZERO), None, "amount0")?;
                Self::validate_decimal_range(*amount1, Some(Decimal::ZERO), None, "amount1")?;
            },
            PoolData::Burn { 
                owner, 
                tick_lower, 
                tick_upper, 
                amount,
                amount0,
                amount1 
            } => {
                Self::validate_eth_address(owner)?;
                if *tick_lower >= *tick_upper {
                    return Err(UniswapV3Error::ValidationError(
                        format!("无效的tick范围: lower {} >= upper {}", tick_lower, tick_upper)
                    ));
                }
                if *tick_lower < -887272 || *tick_lower > 887272 || *tick_upper < -887272 || *tick_upper > 887272 {
                    return Err(UniswapV3Error::ValidationError(
                        format!("Tick超出范围: lower {}, upper {}", tick_lower, tick_upper)
                    ));
                }
                Self::validate_decimal_range(*amount, Some(Decimal::ZERO), None, "数量")?;
                Self::validate_decimal_range(*amount0, Some(Decimal::ZERO), None, "amount0")?;
                Self::validate_decimal_range(*amount1, Some(Decimal::ZERO), None, "amount1")?;
            },
            PoolData::Flash { 
                sender, 
                recipient, 
                amount0, 
                amount1,
                paid0,
                paid1 
            } => {
                Self::validate_eth_address(sender)?;
                Self::validate_eth_address(recipient)?;
                Self::validate_decimal_range(*amount0, Some(Decimal::ZERO), None, "amount0")?;
                Self::validate_decimal_range(*amount1, Some(Decimal::ZERO), None, "amount1")?;
                Self::validate_decimal_range(*paid0, Some(Decimal::ZERO), None, "paid0")?;
                Self::validate_decimal_range(*paid1, Some(Decimal::ZERO), None, "paid1")?;
                
                // 确保支付金额不小于借款金额
                if *paid0 < *amount0 {
                    return Err(UniswapV3Error::ValidationError(
                        format!("token0支付金额小于借款金额: paid {} < borrowed {}", paid0, amount0)
                    ));
                }
                if *paid1 < *amount1 {
                    return Err(UniswapV3Error::ValidationError(
                        format!("token1支付金额小于借款金额: paid {} < borrowed {}", paid1, amount1)
                    ));
                }
            },
            PoolData::Collect { 
                owner, 
                recipient, 
                tick_lower, 
                tick_upper,
                amount0,
                amount1 
            } => {
                Self::validate_eth_address(owner)?;
                Self::validate_eth_address(recipient)?;
                if *tick_lower >= *tick_upper {
                    return Err(UniswapV3Error::ValidationError(
                        format!("无效的tick范围: lower {} >= upper {}", tick_lower, tick_upper)
                    ));
                }
                if *tick_lower < -887272 || *tick_lower > 887272 || *tick_upper < -887272 || *tick_upper > 887272 {
                    return Err(UniswapV3Error::ValidationError(
                        format!("Tick超出范围: lower {}, upper {}", tick_lower, tick_upper)
                    ));
                }
                Self::validate_decimal_range(*amount0, Some(Decimal::ZERO), None, "amount0")?;
                Self::validate_decimal_range(*amount1, Some(Decimal::ZERO), None, "amount1")?;
            },
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_validate_eth_address() {
        // 有效地址
        assert!(DataValidator::validate_eth_address(
            "0x1234567890123456789012345678901234567890"
        ).is_ok());

        // 无效地址
        assert!(DataValidator::validate_eth_address(
            "0x123"  // 太短
        ).is_err());
        assert!(DataValidator::validate_eth_address(
            "0xXYZ4567890123456789012345678901234567890"  // 包含非法字符
        ).is_err());
    }

    #[test]
    fn test_validate_pool() {
        let valid_pool = Pool {
            id: "0x1234567890123456789012345678901234567890".to_string(),
            token0: "0x1234567890123456789012345678901234567890".to_string(),
            token1: "0x1234567890123456789012345678901234567890".to_string(),
            fee_tier: 3000,
            liquidity: dec!(1000),
            sqrt_price_x96: dec!(1000000),
            tick: 100,
            token0_price: dec!(1.5),
            token1_price: dec!(0.5),
            updated_at: Utc::now(),
        };
        assert!(DataValidator::validate_pool(&valid_pool).is_ok());

        // 测试无效费率
        let mut invalid_pool = valid_pool.clone();
        invalid_pool.fee_tier = 1234;
        assert!(DataValidator::validate_pool(&invalid_pool).is_err());

        // 测试负流动性
        let mut invalid_pool = valid_pool.clone();
        invalid_pool.liquidity = dec!(-1000);
        assert!(DataValidator::validate_pool(&invalid_pool).is_err());
    }

    #[test]
    fn test_validate_token() {
        let valid_token = Token {
            id: "0x1234567890123456789012345678901234567890".to_string(),
            symbol: "TOKEN".to_string(),
            name: "Test Token".to_string(),
            decimals: 18,
            total_supply: dec!(1000000),
            volume: dec!(5000),
            tx_count: 100,
        };
        assert!(DataValidator::validate_token(&valid_token).is_ok());

        // 测试空符号
        let mut invalid_token = valid_token.clone();
        invalid_token.symbol = "".to_string();
        assert!(DataValidator::validate_token(&invalid_token).is_err());

        // 测试超出范围的精度
        let mut invalid_token = valid_token.clone();
        invalid_token.decimals = 19;
        assert!(DataValidator::validate_token(&invalid_token).is_err());
    }

    #[test]
    fn test_validate_tick_data() {
        let valid_tick = TickData {
            tick_idx: 100,
            liquidity_gross: dec!(1000),
            liquidity_net: dec!(500),
            price0: dec!(1.5),
            price1: dec!(0.5),
        };
        assert!(DataValidator::validate_tick_data(&valid_tick).is_ok());

        // 测试超出范围的tick
        let mut invalid_tick = valid_tick.clone();
        invalid_tick.tick_idx = 888888;
        assert!(DataValidator::validate_tick_data(&invalid_tick).is_err());

        // 测试负价格
        let mut invalid_tick = valid_tick.clone();
        invalid_tick.price0 = dec!(-1.5);
        assert!(DataValidator::validate_tick_data(&invalid_tick).is_err());
    }

    #[test]
    fn test_validate_position() {
        let valid_position = Position {
            id: "1".to_string(),
            owner: "0x1234567890123456789012345678901234567890".to_string(),
            pool: "0x1234567890123456789012345678901234567890".to_string(),
            tick_lower: -100,
            tick_upper: 100,
            liquidity: dec!(1000),
            token0_owed: dec!(10),
            token1_owed: dec!(20),
        };
        assert!(DataValidator::validate_position(&valid_position).is_ok());

        // 测试无效的tick范围
        let mut invalid_position = valid_position.clone();
        invalid_position.tick_lower = 200;
        invalid_position.tick_upper = 100;
        assert!(DataValidator::validate_position(&invalid_position).is_err());

        // 测试负流动性
        let mut invalid_position = valid_position.clone();
        invalid_position.liquidity = dec!(-1000);
        assert!(DataValidator::validate_position(&invalid_position).is_err());
    }

    #[test]
    fn test_validate_pool_data() {
        // 测试Swap事件
        let swap_event = PoolData::Swap {
            sender: "0x1234567890123456789012345678901234567890".to_string(),
            recipient: "0x1234567890123456789012345678901234567890".to_string(),
            amount0: dec!(100),
            amount1: dec!(200),
            sqrt_price_x96: dec!(1000000),
            liquidity: dec!(5000),
            tick: 100,
        };
        assert!(DataValidator::validate_pool_data(&swap_event).is_ok());

        // 测试Mint事件
        let mint_event = PoolData::Mint {
            sender: "0x1234567890123456789012345678901234567890".to_string(),
            owner: "0x1234567890123456789012345678901234567890".to_string(),
            tick_lower: -100,
            tick_upper: 100,
            amount: dec!(1000),
            amount0: dec!(500),
            amount1: dec!(500),
        };
        assert!(DataValidator::validate_pool_data(&mint_event).is_ok());
    }

    #[test]
    fn test_validate_pool_data_comprehensive() {
        let valid_address = "0x1234567890123456789012345678901234567890".to_string();

        // 测试Swap事件
        let swap_event = PoolData::Swap {
            sender: valid_address.clone(),
            recipient: valid_address.clone(),
            amount0: dec!(100),
            amount1: dec!(200),
            sqrt_price_x96: dec!(1000000),
            liquidity: dec!(5000),
            tick: 100,
        };
        assert!(DataValidator::validate_pool_data(&swap_event).is_ok());

        // 测试无效的Swap事件（tick超出范围）
        let invalid_swap = PoolData::Swap {
            sender: valid_address.clone(),
            recipient: valid_address.clone(),
            amount0: dec!(100),
            amount1: dec!(200),
            sqrt_price_x96: dec!(1000000),
            liquidity: dec!(5000),
            tick: 888888,
        };
        assert!(DataValidator::validate_pool_data(&invalid_swap).is_err());

        // 测试无效的Swap事件（负流动性）
        let invalid_swap = PoolData::Swap {
            sender: valid_address.clone(),
            recipient: valid_address.clone(),
            amount0: dec!(100),
            amount1: dec!(200),
            sqrt_price_x96: dec!(1000000),
            liquidity: dec!(-5000),
            tick: 100,
        };
        assert!(DataValidator::validate_pool_data(&invalid_swap).is_err());

        // 测试Mint事件
        let mint_event = PoolData::Mint {
            sender: valid_address.clone(),
            owner: valid_address.clone(),
            tick_lower: -100,
            tick_upper: 100,
            amount: dec!(1000),
            amount0: dec!(500),
            amount1: dec!(500),
        };
        assert!(DataValidator::validate_pool_data(&mint_event).is_ok());

        // 测试无效的Mint事件（tick范围无效）
        let invalid_mint = PoolData::Mint {
            sender: valid_address.clone(),
            owner: valid_address.clone(),
            tick_lower: 200,
            tick_upper: 100,
            amount: dec!(1000),
            amount0: dec!(500),
            amount1: dec!(500),
        };
        assert!(DataValidator::validate_pool_data(&invalid_mint).is_err());

        // 测试Burn事件
        let burn_event = PoolData::Burn {
            owner: valid_address.clone(),
            tick_lower: -100,
            tick_upper: 100,
            amount: dec!(1000),
            amount0: dec!(500),
            amount1: dec!(500),
        };
        assert!(DataValidator::validate_pool_data(&burn_event).is_ok());

        // 测试无效的Burn事件（负数量）
        let invalid_burn = PoolData::Burn {
            owner: valid_address.clone(),
            tick_lower: -100,
            tick_upper: 100,
            amount: dec!(-1000),
            amount0: dec!(500),
            amount1: dec!(500),
        };
        assert!(DataValidator::validate_pool_data(&invalid_burn).is_err());

        // 测试Flash事件
        let flash_event = PoolData::Flash {
            sender: valid_address.clone(),
            recipient: valid_address.clone(),
            amount0: dec!(100),
            amount1: dec!(200),
            paid0: dec!(110),  // 支付金额大于借款金额
            paid1: dec!(220),
        };
        assert!(DataValidator::validate_pool_data(&flash_event).is_ok());

        // 测试无效的Flash事件（支付金额小于借款金额）
        let invalid_flash = PoolData::Flash {
            sender: valid_address.clone(),
            recipient: valid_address.clone(),
            amount0: dec!(100),
            amount1: dec!(200),
            paid0: dec!(90),  // 支付金额小于借款金额
            paid1: dec!(220),
        };
        assert!(DataValidator::validate_pool_data(&invalid_flash).is_err());

        // 测试无效的Flash事件（负借款金额）
        let invalid_flash = PoolData::Flash {
            sender: valid_address.clone(),
            recipient: valid_address.clone(),
            amount0: dec!(-100),
            amount1: dec!(200),
            paid0: dec!(110),
            paid1: dec!(220),
        };
        assert!(DataValidator::validate_pool_data(&invalid_flash).is_err());

        // 测试Collect事件
        let collect_event = PoolData::Collect {
            owner: valid_address.clone(),
            recipient: valid_address.clone(),
            tick_lower: -100,
            tick_upper: 100,
            amount0: dec!(500),
            amount1: dec!(500),
        };
        assert!(DataValidator::validate_pool_data(&collect_event).is_ok());

        // 测试无效的Collect事件（tick范围无效）
        let invalid_collect = PoolData::Collect {
            owner: valid_address.clone(),
            recipient: valid_address,
            tick_lower: 200,
            tick_upper: 100,  // lower > upper
            amount0: dec!(500),
            amount1: dec!(500),
        };
        assert!(DataValidator::validate_pool_data(&invalid_collect).is_err());
    }

    #[test]
    fn test_validate_eth_address_comprehensive() {
        // 有效地址
        assert!(DataValidator::validate_eth_address(
            "0x1234567890123456789012345678901234567890"
        ).is_ok());
        assert!(DataValidator::validate_eth_address(
            "0xabcdef1234567890abcdef1234567890abcdef12"
        ).is_ok());
        assert!(DataValidator::validate_eth_address(
            "0xABCDEF1234567890ABCDEF1234567890ABCDEF12"
        ).is_ok());

        // 无效地址
        assert!(DataValidator::validate_eth_address(
            "0x123"  // 太短
        ).is_err());
        assert!(DataValidator::validate_eth_address(
            "0xXYZ4567890123456789012345678901234567890"  // 包含非法字符
        ).is_err());
        assert!(DataValidator::validate_eth_address(
            "1234567890123456789012345678901234567890"  // 缺少0x前缀
        ).is_err());
        assert!(DataValidator::validate_eth_address(
            "0x12345678901234567890123456789012345678901"  // 太长
        ).is_err());
        assert!(DataValidator::validate_eth_address(
            ""  // 空字符串
        ).is_err());
    }

    #[test]
    fn test_validate_decimal_range_comprehensive() {
        // 测试正常范围
        assert!(DataValidator::validate_decimal_range(
            dec!(100),
            Some(dec!(0)),
            Some(dec!(1000)),
            "test"
        ).is_ok());

        // 测试边界值
        assert!(DataValidator::validate_decimal_range(
            dec!(0),
            Some(dec!(0)),
            Some(dec!(1000)),
            "test"
        ).is_ok());
        assert!(DataValidator::validate_decimal_range(
            dec!(1000),
            Some(dec!(0)),
            Some(dec!(1000)),
            "test"
        ).is_ok());

        // 测试超出范围
        assert!(DataValidator::validate_decimal_range(
            dec!(-1),
            Some(dec!(0)),
            Some(dec!(1000)),
            "test"
        ).is_err());
        assert!(DataValidator::validate_decimal_range(
            dec!(1001),
            Some(dec!(0)),
            Some(dec!(1000)),
            "test"
        ).is_err());

        // 测试无上限
        assert!(DataValidator::validate_decimal_range(
            dec!(999999),
            Some(dec!(0)),
            None,
            "test"
        ).is_ok());

        // 测试无下限
        assert!(DataValidator::validate_decimal_range(
            dec!(-999999),
            None,
            Some(dec!(0)),
            "test"
        ).is_ok());

        // 测试无上下限
        assert!(DataValidator::validate_decimal_range(
            dec!(-999999),
            None,
            None,
            "test"
        ).is_ok());
    }
} 