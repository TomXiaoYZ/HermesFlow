use crate::error::CurveError;
use crate::models::{Pool, Token, Trade, PoolType};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use std::str::FromStr;

const CURVE_API_URL: &str = "https://api.curve.fi/api/";
const GRAPH_API_URL: &str = "https://api.thegraph.com/subgraphs/name/curvefi/curve";

/// Curve API客户端
pub struct CurveApiClient {
    client: Client,
}

impl CurveApiClient {
    /// 创建新的API客户端实例
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// 获取所有池子列表
    pub async fn get_pools(&self) -> Result<Vec<Pool>, CurveError> {
        let response = self.client
            .get(format!("{}getPools", CURVE_API_URL))
            .send()
            .await
            .map_err(|e| CurveError::RequestError(format!("获取池子列表失败: {}", e)))?;

        let data: Value = response.json()
            .await
            .map_err(|e| CurveError::JsonError(e))?;

        let pools = data["data"]["poolData"].as_array()
            .ok_or_else(|| CurveError::ParseError("无法解析池子列表".to_string()))?;

        let mut result = Vec::new();
        for pool in pools {
            result.push(self.parse_pool(pool)?);
        }

        Ok(result)
    }

    /// 获取池子详情
    pub async fn get_pool(&self, address: &str) -> Result<Pool, CurveError> {
        let response = self.client
            .get(format!("{}getPool/{}", CURVE_API_URL, address))
            .send()
            .await
            .map_err(|e| CurveError::RequestError(format!("获取池子详情失败: {}", e)))?;

        let data: Value = response.json()
            .await
            .map_err(|e| CurveError::JsonError(e))?;

        let pool_data = data["data"]["poolData"]
            .as_object()
            .ok_or_else(|| CurveError::ParseError("无法解析池子数据".to_string()))?;

        self.parse_pool(pool_data)
    }

    /// 获取代币信息
    pub async fn get_token(&self, address: &str) -> Result<Token, CurveError> {
        let query = format!(
            r#"{{
                token(id: "{}") {{
                    id
                    symbol
                    name
                    decimals
                    totalSupply
                }}
            }}"#,
            address.to_lowercase()
        );

        let response = self.client
            .post(GRAPH_API_URL)
            .json(&serde_json::json!({
                "query": query
            }))
            .send()
            .await
            .map_err(|e| CurveError::RequestError(format!("获取代币信息失败: {}", e)))?;

        let data: Value = response.json()
            .await
            .map_err(|e| CurveError::JsonError(e))?;

        let token_data = data["data"]["token"]
            .as_object()
            .ok_or_else(|| CurveError::ParseError("无法解析代币数据".to_string()))?;

        self.parse_token(token_data)
    }

    /// 获取最近的交易
    pub async fn get_recent_trades(&self, pool_address: &str, limit: i32) -> Result<Vec<Trade>, CurveError> {
        let query = format!(
            r#"{{
                swaps(
                    first: {}
                    orderBy: timestamp
                    orderDirection: desc
                    where: {{ pool: "{}" }}
                ) {{
                    pool {{
                        id
                    }}
                    buyer
                    tokenIn
                    tokenOut
                    amountIn
                    amountOut
                    timestamp
                }}
            }}"#,
            limit,
            pool_address.to_lowercase()
        );

        let response = self.client
            .post(GRAPH_API_URL)
            .json(&serde_json::json!({
                "query": query
            }))
            .send()
            .await
            .map_err(|e| CurveError::RequestError(format!("获取交易列表失败: {}", e)))?;

        let data: Value = response.json()
            .await
            .map_err(|e| CurveError::JsonError(e))?;

        let swaps = data["data"]["swaps"]
            .as_array()
            .ok_or_else(|| CurveError::ParseError("无法解析交易列表".to_string()))?;

        let mut result = Vec::new();
        for swap in swaps {
            result.push(self.parse_trade(swap)?);
        }

        Ok(result)
    }

    // 内部辅助方法，解析池子数据
    fn parse_pool(&self, data: &Value) -> Result<Pool, CurveError> {
        let address = data["address"]
            .as_str()
            .ok_or_else(|| CurveError::ParseError("无法解析池子地址".to_string()))?
            .to_string();

        let name = data["name"]
            .as_str()
            .ok_or_else(|| CurveError::ParseError("无法解析池子名称".to_string()))?
            .to_string();

        let pool_type = match data["type"].as_str() {
            Some("plain") => PoolType::Plain,
            Some("meta") => PoolType::Meta,
            Some("crypto") => PoolType::Crypto,
            Some("factory") => PoolType::Factory,
            _ => return Err(CurveError::ParseError("无效的池子类型".to_string())),
        };

        let coins = data["coins"]
            .as_array()
            .ok_or_else(|| CurveError::ParseError("无法解析代币列表".to_string()))?
            .iter()
            .map(|coin| {
                coin["address"]
                    .as_str()
                    .ok_or_else(|| CurveError::ParseError("无法解析代币地址".to_string()))
                    .map(|s| s.to_string())
            })
            .collect::<Result<Vec<String>, CurveError>>()?;

        let underlying_coins = if data["underlying_coins"].is_array() {
            Some(
                data["underlying_coins"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|coin| {
                        coin["address"]
                            .as_str()
                            .ok_or_else(|| CurveError::ParseError("无法解析底层代币地址".to_string()))
                            .map(|s| s.to_string())
                    })
                    .collect::<Result<Vec<String>, CurveError>>()?,
            )
        } else {
            None
        };

        let balances = data["balances"]
            .as_array()
            .ok_or_else(|| CurveError::ParseError("无法解析余额列表".to_string()))?
            .iter()
            .map(|balance| {
                Decimal::from_str(
                    balance
                        .as_str()
                        .ok_or_else(|| CurveError::ParseError("无法解析余额".to_string()))?,
                )
                .map_err(|e| CurveError::ParseError(format!("余额转换失败: {}", e)))
            })
            .collect::<Result<Vec<Decimal>, CurveError>>()?;

        let underlying_balances = if data["underlying_balances"].is_array() {
            Some(
                data["underlying_balances"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|balance| {
                        Decimal::from_str(
                            balance
                                .as_str()
                                .ok_or_else(|| CurveError::ParseError("无法解析底层余额".to_string()))?,
                        )
                        .map_err(|e| CurveError::ParseError(format!("底层余额转换失败: {}", e)))
                    })
                    .collect::<Result<Vec<Decimal>, CurveError>>()?,
            )
        } else {
            None
        };

        let virtual_price = Decimal::from_str(
            data["virtual_price"]
                .as_str()
                .ok_or_else(|| CurveError::ParseError("无法解析虚拟价格".to_string()))?,
        )
        .map_err(|e| CurveError::ParseError(format!("虚拟价格转换失败: {}", e)))?;

        let a = data["A"]
            .as_u64()
            .ok_or_else(|| CurveError::ParseError("无法解析A系数".to_string()))?;

        let fee = Decimal::from_str(
            data["fee"]
                .as_str()
                .ok_or_else(|| CurveError::ParseError("无法解析费率".to_string()))?,
        )
        .map_err(|e| CurveError::ParseError(format!("费率转换失败: {}", e)))?;

        let admin_fee = Decimal::from_str(
            data["admin_fee"]
                .as_str()
                .ok_or_else(|| CurveError::ParseError("无法解析管理费率".to_string()))?,
        )
        .map_err(|e| CurveError::ParseError(format!("管理费率转换失败: {}", e)))?;

        let last_updated = DateTime::from_timestamp(
            data["last_updated"]
                .as_i64()
                .ok_or_else(|| CurveError::ParseError("无法解析更新时间".to_string()))?,
            0,
        )
        .ok_or_else(|| CurveError::ParseError("无效的时间戳".to_string()))?;

        Ok(Pool {
            address,
            name,
            pool_type,
            coins,
            underlying_coins,
            balances,
            underlying_balances,
            virtual_price,
            A: a,
            fee,
            admin_fee,
            last_updated,
        })
    }

    // 内部辅助方法，解析代币数据
    fn parse_token(&self, data: &Value) -> Result<Token, CurveError> {
        let address = data["id"]
            .as_str()
            .ok_or_else(|| CurveError::ParseError("无法解析代币地址".to_string()))?
            .to_string();

        let symbol = data["symbol"]
            .as_str()
            .ok_or_else(|| CurveError::ParseError("无法解析代币符号".to_string()))?
            .to_string();

        let name = data["name"]
            .as_str()
            .ok_or_else(|| CurveError::ParseError("无法解析代币名称".to_string()))?
            .to_string();

        let decimals = data["decimals"]
            .as_u64()
            .ok_or_else(|| CurveError::ParseError("无法解析代币精度".to_string()))? as u8;

        let total_supply = Decimal::from_str(
            data["totalSupply"]
                .as_str()
                .ok_or_else(|| CurveError::ParseError("无法解析总供应量".to_string()))?,
        )
        .map_err(|e| CurveError::ParseError(format!("总供应量转换失败: {}", e)))?;

        let is_underlying = data["is_underlying"]
            .as_bool()
            .unwrap_or(false);

        let price_usd = if let Some(price) = data["priceUSD"].as_str() {
            Some(
                Decimal::from_str(price)
                    .map_err(|e| CurveError::ParseError(format!("价格转换失败: {}", e)))?,
            )
        } else {
            None
        };

        Ok(Token {
            address,
            symbol,
            name,
            decimals,
            total_supply,
            is_underlying,
            price_usd,
        })
    }

    // 内部辅助方法，解析交易数据
    fn parse_trade(&self, data: &Value) -> Result<Trade, CurveError> {
        let pool_address = data["pool"]["id"]
            .as_str()
            .ok_or_else(|| CurveError::ParseError("无法解析池子地址".to_string()))?
            .to_string();

        let trader = data["buyer"]
            .as_str()
            .ok_or_else(|| CurveError::ParseError("无法解析交易者地址".to_string()))?
            .to_string();

        let token_in_index = data["tokenIn"]
            .as_u64()
            .ok_or_else(|| CurveError::ParseError("无法解析输入代币索引".to_string()))? as u8;

        let token_out_index = data["tokenOut"]
            .as_u64()
            .ok_or_else(|| CurveError::ParseError("无法解析输出代币索引".to_string()))? as u8;

        let amount_in = Decimal::from_str(
            data["amountIn"]
                .as_str()
                .ok_or_else(|| CurveError::ParseError("无法解析输入金额".to_string()))?,
        )
        .map_err(|e| CurveError::ParseError(format!("输入金额转换失败: {}", e)))?;

        let amount_out = Decimal::from_str(
            data["amountOut"]
                .as_str()
                .ok_or_else(|| CurveError::ParseError("无法解析输出金额".to_string()))?,
        )
        .map_err(|e| CurveError::ParseError(format!("输出金额转换失败: {}", e)))?;

        let fee = if let Some(fee_str) = data["fee"].as_str() {
            Decimal::from_str(fee_str)
                .map_err(|e| CurveError::ParseError(format!("费用转换失败: {}", e)))?
        } else {
            Decimal::from(0)
        };

        let timestamp = DateTime::from_timestamp(
            data["timestamp"]
                .as_i64()
                .ok_or_else(|| CurveError::ParseError("无法解析交易时间".to_string()))?,
            0,
        )
        .ok_or_else(|| CurveError::ParseError("无效的时间戳".to_string()))?;

        Ok(Trade {
            pool_address,
            trader,
            token_in_index,
            token_out_index,
            amount_in,
            amount_out,
            fee,
            timestamp,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_pools() {
        let client = CurveApiClient::new();
        let result = client.get_pools().await;
        assert!(result.is_ok());
        let pools = result.unwrap();
        assert!(!pools.is_empty());
    }

    #[tokio::test]
    async fn test_get_pool() {
        let client = CurveApiClient::new();
        // 3pool地址
        let pool_address = "0xbebc44782c7db0a1a60cb6fe97d0b483032ff1c7";
        let result = client.get_pool(pool_address).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_token() {
        let client = CurveApiClient::new();
        // DAI地址
        let token_address = "0x6b175474e89094c44da98b954eedeac495271d0f";
        let result = client.get_token(token_address).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_recent_trades() {
        let client = CurveApiClient::new();
        // 3pool地址
        let pool_address = "0xbebc44782c7db0a1a60cb6fe97d0b483032ff1c7";
        let result = client.get_recent_trades(pool_address, 5).await;
        assert!(result.is_ok());
        let trades = result.unwrap();
        assert!(!trades.is_empty());
        assert!(trades.len() <= 5);
    }
} 