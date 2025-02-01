use crate::error::UniswapV3Error;
use crate::models::{Pool, Token, PoolData, TickData, Position};
use rust_decimal::Decimal;
use serde_json::Value;
use chrono::{DateTime, Utc};

/// Uniswap V3数据解析器
pub struct UniswapV3Parser;

impl UniswapV3Parser {
    /// 解析Graph API返回的池子数据
    pub fn parse_pool_data(data: &Value) -> Result<Pool, UniswapV3Error> {
        let pool = Pool {
            id: data["id"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing pool ID".to_string()))?
                .to_string(),
            token0: data["token0"]["id"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing token0 ID".to_string()))?
                .to_string(),
            token1: data["token1"]["id"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing token1 ID".to_string()))?
                .to_string(),
            fee_tier: data["feeTier"].as_u64()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing fee tier".to_string()))?,
            liquidity: Decimal::from_str_exact(data["liquidity"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing liquidity".to_string()))?)
                .map_err(|e| UniswapV3Error::ParseError(format!("Invalid liquidity: {}", e)))?,
            sqrt_price_x96: Decimal::from_str_exact(data["sqrtPrice"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing sqrt price".to_string()))?)
                .map_err(|e| UniswapV3Error::ParseError(format!("Invalid sqrt price: {}", e)))?,
            tick: data["tick"].as_i64()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing tick".to_string()))?,
            token0_price: Decimal::from_str_exact(data["token0Price"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing token0 price".to_string()))?)
                .map_err(|e| UniswapV3Error::ParseError(format!("Invalid token0 price: {}", e)))?,
            token1_price: Decimal::from_str_exact(data["token1Price"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing token1 price".to_string()))?)
                .map_err(|e| UniswapV3Error::ParseError(format!("Invalid token1 price: {}", e)))?,
            updated_at: DateTime::from_timestamp_millis(
                data["updatedAtTimestamp"].as_i64()
                    .ok_or_else(|| UniswapV3Error::ParseError("Missing updated timestamp".to_string()))?)
                .ok_or_else(|| UniswapV3Error::ParseError("Invalid updated timestamp".to_string()))?
                .with_timezone(&Utc),
        };

        Ok(pool)
    }

    /// 解析Graph API返回的代币数据
    pub fn parse_token_data(data: &Value) -> Result<Token, UniswapV3Error> {
        let token = Token {
            id: data["id"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing token ID".to_string()))?
                .to_string(),
            symbol: data["symbol"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing token symbol".to_string()))?
                .to_string(),
            name: data["name"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing token name".to_string()))?
                .to_string(),
            decimals: data["decimals"].as_u64()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing token decimals".to_string()))?,
            total_supply: Decimal::from_str_exact(data["totalSupply"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing total supply".to_string()))?)
                .map_err(|e| UniswapV3Error::ParseError(format!("Invalid total supply: {}", e)))?,
            volume: Decimal::from_str_exact(data["volume"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing volume".to_string()))?)
                .map_err(|e| UniswapV3Error::ParseError(format!("Invalid volume: {}", e)))?,
            tx_count: data["txCount"].as_u64()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing transaction count".to_string()))?,
        };

        Ok(token)
    }

    /// 解析合约事件数据
    pub fn parse_event_data(event_name: &str, data: &Value) -> Result<PoolData, UniswapV3Error> {
        match event_name {
            "Swap" => Self::parse_swap_event(data),
            "Mint" => Self::parse_mint_event(data),
            "Burn" => Self::parse_burn_event(data),
            "Flash" => Self::parse_flash_event(data),
            "Collect" => Self::parse_collect_event(data),
            _ => Err(UniswapV3Error::ParseError(format!("Unknown event type: {}", event_name))),
        }
    }

    /// 解析Swap事件数据
    fn parse_swap_event(data: &Value) -> Result<PoolData, UniswapV3Error> {
        let pool_data = PoolData::Swap {
            sender: data["sender"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing sender address".to_string()))?
                .to_string(),
            recipient: data["recipient"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing recipient address".to_string()))?
                .to_string(),
            amount0: Decimal::from_str_exact(data["amount0"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing amount0".to_string()))?)
                .map_err(|e| UniswapV3Error::ParseError(format!("Invalid amount0: {}", e)))?,
            amount1: Decimal::from_str_exact(data["amount1"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing amount1".to_string()))?)
                .map_err(|e| UniswapV3Error::ParseError(format!("Invalid amount1: {}", e)))?,
            sqrt_price_x96: Decimal::from_str_exact(data["sqrtPriceX96"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing sqrt price".to_string()))?)
                .map_err(|e| UniswapV3Error::ParseError(format!("Invalid sqrt price: {}", e)))?,
            liquidity: Decimal::from_str_exact(data["liquidity"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing liquidity".to_string()))?)
                .map_err(|e| UniswapV3Error::ParseError(format!("Invalid liquidity: {}", e)))?,
            tick: data["tick"].as_i64()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing tick".to_string()))?,
        };

        Ok(pool_data)
    }

    /// 解析Mint事件数据
    fn parse_mint_event(data: &Value) -> Result<PoolData, UniswapV3Error> {
        let pool_data = PoolData::Mint {
            sender: data["sender"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing sender address".to_string()))?
                .to_string(),
            owner: data["owner"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing owner address".to_string()))?
                .to_string(),
            tick_lower: data["tickLower"].as_i64()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing lower tick".to_string()))?,
            tick_upper: data["tickUpper"].as_i64()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing upper tick".to_string()))?,
            amount: Decimal::from_str_exact(data["amount"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing amount".to_string()))?)
                .map_err(|e| UniswapV3Error::ParseError(format!("Invalid amount: {}", e)))?,
            amount0: Decimal::from_str_exact(data["amount0"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing amount0".to_string()))?)
                .map_err(|e| UniswapV3Error::ParseError(format!("Invalid amount0: {}", e)))?,
            amount1: Decimal::from_str_exact(data["amount1"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing amount1".to_string()))?)
                .map_err(|e| UniswapV3Error::ParseError(format!("Invalid amount1: {}", e)))?,
        };

        Ok(pool_data)
    }

    /// 解析Burn事件数据
    fn parse_burn_event(data: &Value) -> Result<PoolData, UniswapV3Error> {
        let pool_data = PoolData::Burn {
            owner: data["owner"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing owner address".to_string()))?
                .to_string(),
            tick_lower: data["tickLower"].as_i64()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing lower tick".to_string()))?,
            tick_upper: data["tickUpper"].as_i64()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing upper tick".to_string()))?,
            amount: Decimal::from_str_exact(data["amount"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing amount".to_string()))?)
                .map_err(|e| UniswapV3Error::ParseError(format!("Invalid amount: {}", e)))?,
            amount0: Decimal::from_str_exact(data["amount0"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing amount0".to_string()))?)
                .map_err(|e| UniswapV3Error::ParseError(format!("Invalid amount0: {}", e)))?,
            amount1: Decimal::from_str_exact(data["amount1"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing amount1".to_string()))?)
                .map_err(|e| UniswapV3Error::ParseError(format!("Invalid amount1: {}", e)))?,
        };

        Ok(pool_data)
    }

    /// 解析Flash事件数据
    fn parse_flash_event(data: &Value) -> Result<PoolData, UniswapV3Error> {
        let pool_data = PoolData::Flash {
            sender: data["sender"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing sender address".to_string()))?
                .to_string(),
            recipient: data["recipient"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing recipient address".to_string()))?
                .to_string(),
            amount0: Decimal::from_str_exact(data["amount0"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing amount0".to_string()))?)
                .map_err(|e| UniswapV3Error::ParseError(format!("Invalid amount0: {}", e)))?,
            amount1: Decimal::from_str_exact(data["amount1"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing amount1".to_string()))?)
                .map_err(|e| UniswapV3Error::ParseError(format!("Invalid amount1: {}", e)))?,
            paid0: Decimal::from_str_exact(data["paid0"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing paid0".to_string()))?)
                .map_err(|e| UniswapV3Error::ParseError(format!("Invalid paid0: {}", e)))?,
            paid1: Decimal::from_str_exact(data["paid1"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing paid1".to_string()))?)
                .map_err(|e| UniswapV3Error::ParseError(format!("Invalid paid1: {}", e)))?,
        };

        Ok(pool_data)
    }

    /// 解析Collect事件数据
    fn parse_collect_event(data: &Value) -> Result<PoolData, UniswapV3Error> {
        let pool_data = PoolData::Collect {
            owner: data["owner"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing owner address".to_string()))?
                .to_string(),
            recipient: data["recipient"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing recipient address".to_string()))?
                .to_string(),
            tick_lower: data["tickLower"].as_i64()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing lower tick".to_string()))?,
            tick_upper: data["tickUpper"].as_i64()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing upper tick".to_string()))?,
            amount0: Decimal::from_str_exact(data["amount0"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing amount0".to_string()))?)
                .map_err(|e| UniswapV3Error::ParseError(format!("Invalid amount0: {}", e)))?,
            amount1: Decimal::from_str_exact(data["amount1"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing amount1".to_string()))?)
                .map_err(|e| UniswapV3Error::ParseError(format!("Invalid amount1: {}", e)))?,
        };

        Ok(pool_data)
    }

    /// 解析Tick数据
    pub fn parse_tick_data(data: &Value) -> Result<TickData, UniswapV3Error> {
        let tick_data = TickData {
            tick_idx: data["tickIdx"].as_i64()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing tick index".to_string()))?,
            liquidity_gross: Decimal::from_str_exact(data["liquidityGross"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing liquidity gross".to_string()))?)
                .map_err(|e| UniswapV3Error::ParseError(format!("Invalid liquidity gross: {}", e)))?,
            liquidity_net: Decimal::from_str_exact(data["liquidityNet"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing liquidity net".to_string()))?)
                .map_err(|e| UniswapV3Error::ParseError(format!("Invalid liquidity net: {}", e)))?,
            price0: Decimal::from_str_exact(data["price0"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing price0".to_string()))?)
                .map_err(|e| UniswapV3Error::ParseError(format!("Invalid price0: {}", e)))?,
            price1: Decimal::from_str_exact(data["price1"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("Missing price1".to_string()))?)
                .map_err(|e| UniswapV3Error::ParseError(format!("Invalid price1: {}", e)))?,
        };

        Ok(tick_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_parse_pool_data() {
        let data = serde_json::json!({
            "id": "0x1234567890abcdef",
            "token0": {
                "id": "0xtoken0"
            },
            "token1": {
                "id": "0xtoken1"
            },
            "feeTier": 3000,
            "liquidity": "1000000",
            "sqrtPrice": "1000000000000000000",
            "tick": 100,
            "token0Price": "1000.5",
            "token1Price": "0.001",
            "updatedAtTimestamp": 1677649200000
        });

        let pool = UniswapV3Parser::parse_pool_data(&data).unwrap();
        
        assert_eq!(pool.id, "0x1234567890abcdef");
        assert_eq!(pool.token0, "0xtoken0");
        assert_eq!(pool.token1, "0xtoken1");
        assert_eq!(pool.fee_tier, 3000);
        assert_eq!(pool.liquidity, dec!(1000000));
        assert_eq!(pool.token0_price, dec!(1000.5));
        assert_eq!(pool.token1_price, dec!(0.001));
    }

    #[test]
    fn test_parse_swap_event() {
        let data = serde_json::json!({
            "sender": "0xsender",
            "recipient": "0xrecipient",
            "amount0": "1000",
            "amount1": "2000",
            "sqrtPriceX96": "1000000000000000000",
            "liquidity": "5000000",
            "tick": 200
        });

        if let PoolData::Swap {
            sender,
            recipient,
            amount0,
            amount1,
            sqrt_price_x96,
            liquidity,
            tick,
        } = UniswapV3Parser::parse_event_data("Swap", &data).unwrap() {
            assert_eq!(sender, "0xsender");
            assert_eq!(recipient, "0xrecipient");
            assert_eq!(amount0, dec!(1000));
            assert_eq!(amount1, dec!(2000));
            assert_eq!(liquidity, dec!(5000000));
            assert_eq!(tick, 200);
        } else {
            panic!("Wrong event type");
        }
    }

    #[test]
    fn test_parse_mint_event() {
        let data = serde_json::json!({
            "sender": "0xsender",
            "owner": "0xowner",
            "tickLower": -100,
            "tickUpper": 100,
            "amount": "1000",
            "amount0": "500",
            "amount1": "500"
        });

        if let PoolData::Mint {
            sender,
            owner,
            tick_lower,
            tick_upper,
            amount,
            amount0,
            amount1,
        } = UniswapV3Parser::parse_event_data("Mint", &data).unwrap() {
            assert_eq!(sender, "0xsender");
            assert_eq!(owner, "0xowner");
            assert_eq!(tick_lower, -100);
            assert_eq!(tick_upper, 100);
            assert_eq!(amount, dec!(1000));
            assert_eq!(amount0, dec!(500));
            assert_eq!(amount1, dec!(500));
        } else {
            panic!("Wrong event type");
        }
    }
} 