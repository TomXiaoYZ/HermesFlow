use crate::error::UniswapV3Error;
use crate::models::{Pool, Token, TickData, Position, PoolData};
use rust_decimal::Decimal;
use serde_json::{Value, json};
use std::str::FromStr;
use chrono::{DateTime, Utc};

/// 数据转换器
pub struct DataConverter;

impl DataConverter {
    /// 将wei转换为ether（除以10^18）
    pub fn wei_to_eth(wei: &Decimal) -> Result<Decimal, UniswapV3Error> {
        let eth = wei.checked_div(Decimal::from(10u64.pow(18)))
            .ok_or_else(|| UniswapV3Error::ConversionError("Wei转换Ether失败".to_string()))?;
        Ok(eth)
    }

    /// 将ether转换为wei（乘以10^18）
    pub fn eth_to_wei(eth: &Decimal) -> Result<Decimal, UniswapV3Error> {
        let wei = eth.checked_mul(Decimal::from(10u64.pow(18)))
            .ok_or_else(|| UniswapV3Error::ConversionError("Ether转换Wei失败".to_string()))?;
        Ok(wei)
    }

    /// 根据精度转换代币数量
    pub fn convert_token_amount(amount: &Decimal, decimals: u32) -> Result<Decimal, UniswapV3Error> {
        let scale = Decimal::from(10u64.pow(decimals));
        amount.checked_div(scale)
            .ok_or_else(|| UniswapV3Error::ConversionError(
                format!("代币数量转换失败，精度: {}", decimals)
            ))
    }

    /// 将JSON转换为Pool结构体
    pub fn json_to_pool(value: &Value) -> Result<Pool, UniswapV3Error> {
        let pool = Pool {
            id: value["id"].as_str()
                .ok_or_else(|| UniswapV3Error::ConversionError("缺少池子ID".to_string()))?
                .to_string(),
            token0: value["token0"]["id"].as_str()
                .ok_or_else(|| UniswapV3Error::ConversionError("缺少token0 ID".to_string()))?
                .to_string(),
            token1: value["token1"]["id"].as_str()
                .ok_or_else(|| UniswapV3Error::ConversionError("缺少token1 ID".to_string()))?
                .to_string(),
            fee_tier: value["feeTier"].as_u64()
                .ok_or_else(|| UniswapV3Error::ConversionError("缺少费率".to_string()))?,
            liquidity: Decimal::from_str(value["liquidity"].as_str()
                .ok_or_else(|| UniswapV3Error::ConversionError("缺少流动性".to_string()))?)
                .map_err(|e| UniswapV3Error::ConversionError(format!("流动性转换失败: {}", e)))?,
            sqrt_price_x96: Decimal::from_str(value["sqrtPrice"].as_str()
                .ok_or_else(|| UniswapV3Error::ConversionError("缺少价格".to_string()))?)
                .map_err(|e| UniswapV3Error::ConversionError(format!("价格转换失败: {}", e)))?,
            tick: value["tick"].as_i64()
                .ok_or_else(|| UniswapV3Error::ConversionError("缺少tick".to_string()))?,
            token0_price: Decimal::from_str(value["token0Price"].as_str()
                .ok_or_else(|| UniswapV3Error::ConversionError("缺少token0价格".to_string()))?)
                .map_err(|e| UniswapV3Error::ConversionError(format!("token0价格转换失败: {}", e)))?,
            token1_price: Decimal::from_str(value["token1Price"].as_str()
                .ok_or_else(|| UniswapV3Error::ConversionError("缺少token1价格".to_string()))?)
                .map_err(|e| UniswapV3Error::ConversionError(format!("token1价格转换失败: {}", e)))?,
            updated_at: DateTime::from_timestamp_millis(
                value["updatedAtTimestamp"].as_i64()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少更新时间".to_string()))?)
                .ok_or_else(|| UniswapV3Error::ConversionError("时间戳转换失败".to_string()))?
                .with_timezone(&Utc),
        };
        Ok(pool)
    }

    /// 将Pool结构体转换为JSON
    pub fn pool_to_json(pool: &Pool) -> Value {
        json!({
            "id": pool.id,
            "token0": {
                "id": pool.token0
            },
            "token1": {
                "id": pool.token1
            },
            "feeTier": pool.fee_tier,
            "liquidity": pool.liquidity.to_string(),
            "sqrtPrice": pool.sqrt_price_x96.to_string(),
            "tick": pool.tick,
            "token0Price": pool.token0_price.to_string(),
            "token1Price": pool.token1_price.to_string(),
            "updatedAtTimestamp": pool.updated_at.timestamp_millis()
        })
    }

    /// 将JSON转换为Token结构体
    pub fn json_to_token(value: &Value) -> Result<Token, UniswapV3Error> {
        let token = Token {
            id: value["id"].as_str()
                .ok_or_else(|| UniswapV3Error::ConversionError("缺少代币ID".to_string()))?
                .to_string(),
            symbol: value["symbol"].as_str()
                .ok_or_else(|| UniswapV3Error::ConversionError("缺少代币符号".to_string()))?
                .to_string(),
            name: value["name"].as_str()
                .ok_or_else(|| UniswapV3Error::ConversionError("缺少代币名称".to_string()))?
                .to_string(),
            decimals: value["decimals"].as_u64()
                .ok_or_else(|| UniswapV3Error::ConversionError("缺少代币精度".to_string()))?,
            total_supply: Decimal::from_str(value["totalSupply"].as_str()
                .ok_or_else(|| UniswapV3Error::ConversionError("缺少总供应量".to_string()))?)
                .map_err(|e| UniswapV3Error::ConversionError(format!("总供应量转换失败: {}", e)))?,
            volume: Decimal::from_str(value["volume"].as_str()
                .ok_or_else(|| UniswapV3Error::ConversionError("缺少交易量".to_string()))?)
                .map_err(|e| UniswapV3Error::ConversionError(format!("交易量转换失败: {}", e)))?,
            tx_count: value["txCount"].as_u64()
                .ok_or_else(|| UniswapV3Error::ConversionError("缺少交易数".to_string()))?,
        };
        Ok(token)
    }

    /// 将Token结构体转换为JSON
    pub fn token_to_json(token: &Token) -> Value {
        json!({
            "id": token.id,
            "symbol": token.symbol,
            "name": token.name,
            "decimals": token.decimals,
            "totalSupply": token.total_supply.to_string(),
            "volume": token.volume.to_string(),
            "txCount": token.tx_count
        })
    }

    /// 将JSON转换为PoolData枚举
    pub fn json_to_pool_data(event_type: &str, value: &Value) -> Result<PoolData, UniswapV3Error> {
        match event_type {
            "Swap" => Ok(PoolData::Swap {
                sender: value["sender"].as_str()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少发送者地址".to_string()))?
                    .to_string(),
                recipient: value["recipient"].as_str()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少接收者地址".to_string()))?
                    .to_string(),
                amount0: Decimal::from_str(value["amount0"].as_str()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少amount0".to_string()))?)
                    .map_err(|e| UniswapV3Error::ConversionError(format!("amount0转换失败: {}", e)))?,
                amount1: Decimal::from_str(value["amount1"].as_str()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少amount1".to_string()))?)
                    .map_err(|e| UniswapV3Error::ConversionError(format!("amount1转换失败: {}", e)))?,
                sqrt_price_x96: Decimal::from_str(value["sqrtPriceX96"].as_str()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少价格".to_string()))?)
                    .map_err(|e| UniswapV3Error::ConversionError(format!("价格转换失败: {}", e)))?,
                liquidity: Decimal::from_str(value["liquidity"].as_str()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少流动性".to_string()))?)
                    .map_err(|e| UniswapV3Error::ConversionError(format!("流动性转换失败: {}", e)))?,
                tick: value["tick"].as_i64()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少tick".to_string()))?,
            }),
            "Mint" => Ok(PoolData::Mint {
                sender: value["sender"].as_str()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少发送者地址".to_string()))?
                    .to_string(),
                owner: value["owner"].as_str()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少所有者地址".to_string()))?
                    .to_string(),
                tick_lower: value["tickLower"].as_i64()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少下限tick".to_string()))?,
                tick_upper: value["tickUpper"].as_i64()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少上限tick".to_string()))?,
                amount: Decimal::from_str(value["amount"].as_str()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少数量".to_string()))?)
                    .map_err(|e| UniswapV3Error::ConversionError(format!("数量转换失败: {}", e)))?,
                amount0: Decimal::from_str(value["amount0"].as_str()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少amount0".to_string()))?)
                    .map_err(|e| UniswapV3Error::ConversionError(format!("amount0转换失败: {}", e)))?,
                amount1: Decimal::from_str(value["amount1"].as_str()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少amount1".to_string()))?)
                    .map_err(|e| UniswapV3Error::ConversionError(format!("amount1转换失败: {}", e)))?,
            }),
            "Burn" => Ok(PoolData::Burn {
                owner: value["owner"].as_str()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少所有者地址".to_string()))?
                    .to_string(),
                tick_lower: value["tickLower"].as_i64()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少下限tick".to_string()))?,
                tick_upper: value["tickUpper"].as_i64()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少上限tick".to_string()))?,
                amount: Decimal::from_str(value["amount"].as_str()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少数量".to_string()))?)
                    .map_err(|e| UniswapV3Error::ConversionError(format!("数量转换失败: {}", e)))?,
                amount0: Decimal::from_str(value["amount0"].as_str()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少amount0".to_string()))?)
                    .map_err(|e| UniswapV3Error::ConversionError(format!("amount0转换失败: {}", e)))?,
                amount1: Decimal::from_str(value["amount1"].as_str()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少amount1".to_string()))?)
                    .map_err(|e| UniswapV3Error::ConversionError(format!("amount1转换失败: {}", e)))?,
            }),
            "Flash" => Ok(PoolData::Flash {
                sender: value["sender"].as_str()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少发送者地址".to_string()))?
                    .to_string(),
                recipient: value["recipient"].as_str()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少接收者地址".to_string()))?
                    .to_string(),
                amount0: Decimal::from_str(value["amount0"].as_str()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少amount0".to_string()))?)
                    .map_err(|e| UniswapV3Error::ConversionError(format!("amount0转换失败: {}", e)))?,
                amount1: Decimal::from_str(value["amount1"].as_str()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少amount1".to_string()))?)
                    .map_err(|e| UniswapV3Error::ConversionError(format!("amount1转换失败: {}", e)))?,
                paid0: Decimal::from_str(value["paid0"].as_str()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少paid0".to_string()))?)
                    .map_err(|e| UniswapV3Error::ConversionError(format!("paid0转换失败: {}", e)))?,
                paid1: Decimal::from_str(value["paid1"].as_str()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少paid1".to_string()))?)
                    .map_err(|e| UniswapV3Error::ConversionError(format!("paid1转换失败: {}", e)))?,
            }),
            "Collect" => Ok(PoolData::Collect {
                owner: value["owner"].as_str()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少所有者地址".to_string()))?
                    .to_string(),
                recipient: value["recipient"].as_str()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少接收者地址".to_string()))?
                    .to_string(),
                tick_lower: value["tickLower"].as_i64()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少下限tick".to_string()))?,
                tick_upper: value["tickUpper"].as_i64()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少上限tick".to_string()))?,
                amount0: Decimal::from_str(value["amount0"].as_str()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少amount0".to_string()))?)
                    .map_err(|e| UniswapV3Error::ConversionError(format!("amount0转换失败: {}", e)))?,
                amount1: Decimal::from_str(value["amount1"].as_str()
                    .ok_or_else(|| UniswapV3Error::ConversionError("缺少amount1".to_string()))?)
                    .map_err(|e| UniswapV3Error::ConversionError(format!("amount1转换失败: {}", e)))?,
            }),
            _ => Err(UniswapV3Error::ConversionError(format!("未知的事件类型: {}", event_type)))
        }
    }

    /// 将PoolData枚举转换为JSON
    pub fn pool_data_to_json(pool_data: &PoolData) -> Value {
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
                json!({
                    "type": "Swap",
                    "sender": sender,
                    "recipient": recipient,
                    "amount0": amount0.to_string(),
                    "amount1": amount1.to_string(),
                    "sqrtPriceX96": sqrt_price_x96.to_string(),
                    "liquidity": liquidity.to_string(),
                    "tick": tick
                })
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
                json!({
                    "type": "Mint",
                    "sender": sender,
                    "owner": owner,
                    "tickLower": tick_lower,
                    "tickUpper": tick_upper,
                    "amount": amount.to_string(),
                    "amount0": amount0.to_string(),
                    "amount1": amount1.to_string()
                })
            },
            PoolData::Burn {
                owner,
                tick_lower,
                tick_upper,
                amount,
                amount0,
                amount1
            } => {
                json!({
                    "type": "Burn",
                    "owner": owner,
                    "tickLower": tick_lower,
                    "tickUpper": tick_upper,
                    "amount": amount.to_string(),
                    "amount0": amount0.to_string(),
                    "amount1": amount1.to_string()
                })
            },
            PoolData::Flash {
                sender,
                recipient,
                amount0,
                amount1,
                paid0,
                paid1
            } => {
                json!({
                    "type": "Flash",
                    "sender": sender,
                    "recipient": recipient,
                    "amount0": amount0.to_string(),
                    "amount1": amount1.to_string(),
                    "paid0": paid0.to_string(),
                    "paid1": paid1.to_string()
                })
            },
            PoolData::Collect {
                owner,
                recipient,
                tick_lower,
                tick_upper,
                amount0,
                amount1
            } => {
                json!({
                    "type": "Collect",
                    "owner": owner,
                    "recipient": recipient,
                    "tickLower": tick_lower,
                    "tickUpper": tick_upper,
                    "amount0": amount0.to_string(),
                    "amount1": amount1.to_string()
                })
            }
        }
    }

    /// 从JSON值解析Decimal
    pub fn parse_decimal(value: &Value) -> Result<Decimal, UniswapV3Error> {
        let str_value = value.as_str()
            .ok_or_else(|| UniswapV3Error::ConversionError("无法解析为字符串".to_string()))?;
        
        Decimal::from_str(str_value)
            .map_err(|e| UniswapV3Error::ConversionError(format!("Decimal转换失败: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_wei_to_eth_conversion() {
        let wei = dec!(1000000000000000000); // 1 ETH in wei
        let eth = DataConverter::wei_to_eth(&wei).unwrap();
        assert_eq!(eth, dec!(1));

        let wei = dec!(500000000000000000); // 0.5 ETH in wei
        let eth = DataConverter::wei_to_eth(&wei).unwrap();
        assert_eq!(eth, dec!(0.5));
    }

    #[test]
    fn test_eth_to_wei_conversion() {
        let eth = dec!(1);
        let wei = DataConverter::eth_to_wei(&eth).unwrap();
        assert_eq!(wei, dec!(1000000000000000000));

        let eth = dec!(0.5);
        let wei = DataConverter::eth_to_wei(&eth).unwrap();
        assert_eq!(wei, dec!(500000000000000000));
    }

    #[test]
    fn test_token_amount_conversion() {
        let amount = dec!(1000000); // 1 USDC (6 decimals)
        let converted = DataConverter::convert_token_amount(&amount, 6).unwrap();
        assert_eq!(converted, dec!(1));

        let amount = dec!(500000000000000000); // 0.5 WETH (18 decimals)
        let converted = DataConverter::convert_token_amount(&amount, 18).unwrap();
        assert_eq!(converted, dec!(0.5));
    }

    #[test]
    fn test_json_pool_conversion() {
        let json_data = json!({
            "id": "0x1234567890123456789012345678901234567890",
            "token0": {
                "id": "0x1234567890123456789012345678901234567890"
            },
            "token1": {
                "id": "0x1234567890123456789012345678901234567890"
            },
            "feeTier": 3000,
            "liquidity": "1000000",
            "sqrtPrice": "1000000000000000000",
            "tick": 100,
            "token0Price": "1000.5",
            "token1Price": "0.001",
            "updatedAtTimestamp": 1677649200000
        });

        let pool = DataConverter::json_to_pool(&json_data).unwrap();
        assert_eq!(pool.fee_tier, 3000);
        assert_eq!(pool.liquidity, dec!(1000000));
        assert_eq!(pool.token0_price, dec!(1000.5));

        let json = DataConverter::pool_to_json(&pool);
        assert_eq!(json["feeTier"], 3000);
        assert_eq!(json["liquidity"], "1000000");
        assert_eq!(json["token0Price"], "1000.5");
    }

    #[test]
    fn test_json_token_conversion() {
        let json_data = json!({
            "id": "0x1234567890123456789012345678901234567890",
            "symbol": "TOKEN",
            "name": "Test Token",
            "decimals": 18,
            "totalSupply": "1000000000000000000000000",
            "volume": "5000000000000000000000",
            "txCount": 1000
        });

        let token = DataConverter::json_to_token(&json_data).unwrap();
        assert_eq!(token.symbol, "TOKEN");
        assert_eq!(token.decimals, 18);
        assert_eq!(token.tx_count, 1000);

        let json = DataConverter::token_to_json(&token);
        assert_eq!(json["symbol"], "TOKEN");
        assert_eq!(json["decimals"], 18);
        assert_eq!(json["txCount"], 1000);
    }

    #[test]
    fn test_json_pool_data_conversion() {
        // 测试Swap事件
        let swap_data = json!({
            "sender": "0x1234567890123456789012345678901234567890",
            "recipient": "0x1234567890123456789012345678901234567890",
            "amount0": "1000000000000000000",
            "amount1": "2000000000000000000",
            "sqrtPriceX96": "1000000000000000000",
            "liquidity": "5000000000000000000",
            "tick": 100
        });

        let pool_data = DataConverter::json_to_pool_data("Swap", &swap_data).unwrap();
        if let PoolData::Swap { amount0, amount1, tick, .. } = pool_data {
            assert_eq!(amount0, dec!(1000000000000000000));
            assert_eq!(amount1, dec!(2000000000000000000));
            assert_eq!(tick, 100);
        } else {
            panic!("Wrong event type");
        }

        // 测试Mint事件
        let mint_data = json!({
            "sender": "0x1234567890123456789012345678901234567890",
            "owner": "0x1234567890123456789012345678901234567890",
            "tickLower": -100,
            "tickUpper": 100,
            "amount": "1000000000000000000",
            "amount0": "500000000000000000",
            "amount1": "500000000000000000"
        });

        let pool_data = DataConverter::json_to_pool_data("Mint", &mint_data).unwrap();
        if let PoolData::Mint { tick_lower, tick_upper, amount, .. } = pool_data {
            assert_eq!(tick_lower, -100);
            assert_eq!(tick_upper, 100);
            assert_eq!(amount, dec!(1000000000000000000));
        } else {
            panic!("Wrong event type");
        }

        // 测试Burn事件
        let burn_data = json!({
            "owner": "0x1234567890123456789012345678901234567890",
            "tickLower": -100,
            "tickUpper": 100,
            "amount": "1000000000000000000",
            "amount0": "500000000000000000",
            "amount1": "500000000000000000"
        });

        let pool_data = DataConverter::json_to_pool_data("Burn", &burn_data).unwrap();
        if let PoolData::Burn { tick_lower, tick_upper, amount, .. } = pool_data {
            assert_eq!(tick_lower, -100);
            assert_eq!(tick_upper, 100);
            assert_eq!(amount, dec!(1000000000000000000));
        } else {
            panic!("Wrong event type");
        }

        // 测试Flash事件
        let flash_data = json!({
            "sender": "0x1234567890123456789012345678901234567890",
            "recipient": "0x1234567890123456789012345678901234567890",
            "amount0": "1000000000000000000",
            "amount1": "2000000000000000000",
            "paid0": "1100000000000000000",
            "paid1": "2200000000000000000"
        });

        let pool_data = DataConverter::json_to_pool_data("Flash", &flash_data).unwrap();
        if let PoolData::Flash { amount0, amount1, paid0, paid1, .. } = pool_data {
            assert_eq!(amount0, dec!(1000000000000000000));
            assert_eq!(amount1, dec!(2000000000000000000));
            assert_eq!(paid0, dec!(1100000000000000000));
            assert_eq!(paid1, dec!(2200000000000000000));
        } else {
            panic!("Wrong event type");
        }

        // 测试Collect事件
        let collect_data = json!({
            "owner": "0x1234567890123456789012345678901234567890",
            "recipient": "0x1234567890123456789012345678901234567890",
            "tickLower": -100,
            "tickUpper": 100,
            "amount0": "500000000000000000",
            "amount1": "500000000000000000"
        });

        let pool_data = DataConverter::json_to_pool_data("Collect", &collect_data).unwrap();
        if let PoolData::Collect { tick_lower, tick_upper, amount0, amount1, .. } = pool_data {
            assert_eq!(tick_lower, -100);
            assert_eq!(tick_upper, 100);
            assert_eq!(amount0, dec!(500000000000000000));
            assert_eq!(amount1, dec!(500000000000000000));
        } else {
            panic!("Wrong event type");
        }

        // 测试未知事件类型
        assert!(DataConverter::json_to_pool_data("Unknown", &json!({})).is_err());
    }

    #[test]
    fn test_pool_data_json_conversion() {
        // 测试Swap事件
        let swap_event = PoolData::Swap {
            sender: "0x1234567890123456789012345678901234567890".to_string(),
            recipient: "0x1234567890123456789012345678901234567890".to_string(),
            amount0: dec!(1000000000000000000),
            amount1: dec!(2000000000000000000),
            sqrt_price_x96: dec!(1000000000000000000),
            liquidity: dec!(5000000000000000000),
            tick: 100,
        };

        let json = DataConverter::pool_data_to_json(&swap_event);
        assert_eq!(json["type"], "Swap");
        assert_eq!(json["amount0"], "1000000000000000000");
        assert_eq!(json["amount1"], "2000000000000000000");
        assert_eq!(json["tick"], 100);

        // 测试Mint事件
        let mint_event = PoolData::Mint {
            sender: "0x1234567890123456789012345678901234567890".to_string(),
            owner: "0x1234567890123456789012345678901234567890".to_string(),
            tick_lower: -100,
            tick_upper: 100,
            amount: dec!(1000000000000000000),
            amount0: dec!(500000000000000000),
            amount1: dec!(500000000000000000),
        };

        let json = DataConverter::pool_data_to_json(&mint_event);
        assert_eq!(json["type"], "Mint");
        assert_eq!(json["tickLower"], -100);
        assert_eq!(json["tickUpper"], 100);
        assert_eq!(json["amount"], "1000000000000000000");
    }
} 