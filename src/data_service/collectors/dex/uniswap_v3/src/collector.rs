use crate::error::UniswapV3Error;
use crate::models::{Pool, Token, PoolData, TickData};
use crate::conversion::DataConverter;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use chrono::{DateTime, Utc};

const UNISWAP_V3_GRAPH_URL: &str = "https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v3";

/// Uniswap V3数据采集器
pub struct UniswapV3Collector {
    client: Client,
}

impl UniswapV3Collector {
    /// 创建新的采集器实例
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// 获取池子信息
    pub async fn get_pool(&self, pool_address: &str) -> Result<Pool, UniswapV3Error> {
        let query = format!(
            r#"{{
                pool(id: "{}") {{
                    id
                    token0 {{
                        id
                    }}
                    token1 {{
                        id
                    }}
                    feeTier
                    liquidity
                    sqrtPrice
                    tick
                    token0Price
                    token1Price
                    updatedAtTimestamp
                }}
            }}"#,
            pool_address.to_lowercase()
        );

        let response = self.query_graph(&query).await?;
        let pool_data = response["data"]["pool"].clone();
        DataConverter::json_to_pool(&pool_data)
    }

    /// 获取代币信息
    pub async fn get_token(&self, token_address: &str) -> Result<Token, UniswapV3Error> {
        let query = format!(
            r#"{{
                token(id: "{}") {{
                    id
                    symbol
                    name
                    decimals
                    totalSupply
                    volume
                    txCount
                }}
            }}"#,
            token_address.to_lowercase()
        );

        let response = self.query_graph(&query).await?;
        let token_data = response["data"]["token"].clone();
        DataConverter::json_to_token(&token_data)
    }

    /// 获取最近的Swap事件
    pub async fn get_recent_swaps(&self, pool_address: &str, limit: i32) -> Result<Vec<PoolData>, UniswapV3Error> {
        let query = format!(
            r#"{{
                swaps(
                    first: {}
                    orderBy: timestamp
                    orderDirection: desc
                    where: {{ pool: "{}" }}
                ) {{
                    sender
                    recipient
                    amount0
                    amount1
                    sqrtPriceX96
                    liquidity
                    tick
                }}
            }}"#,
            limit,
            pool_address.to_lowercase()
        );

        let response = self.query_graph(&query).await?;
        let swaps = response["data"]["swaps"].as_array()
            .ok_or_else(|| UniswapV3Error::ParseError("无法解析Swap事件列表".to_string()))?;

        let mut result = Vec::new();
        for swap in swaps {
            result.push(DataConverter::json_to_pool_data("Swap", swap)?);
        }

        Ok(result)
    }

    /// 获取最近的Mint事件
    pub async fn get_recent_mints(&self, pool_address: &str, limit: i32) -> Result<Vec<PoolData>, UniswapV3Error> {
        let query = format!(
            r#"{{
                mints(
                    first: {}
                    orderBy: timestamp
                    orderDirection: desc
                    where: {{ pool: "{}" }}
                ) {{
                    sender
                    owner
                    tickLower
                    tickUpper
                    amount
                    amount0
                    amount1
                }}
            }}"#,
            limit,
            pool_address.to_lowercase()
        );

        let response = self.query_graph(&query).await?;
        let mints = response["data"]["mints"].as_array()
            .ok_or_else(|| UniswapV3Error::ParseError("无法解析Mint事件列表".to_string()))?;

        let mut result = Vec::new();
        for mint in mints {
            result.push(DataConverter::json_to_pool_data("Mint", mint)?);
        }

        Ok(result)
    }

    /// 获取最近的Burn事件
    pub async fn get_recent_burns(&self, pool_address: &str, limit: i32) -> Result<Vec<PoolData>, UniswapV3Error> {
        let query = format!(
            r#"{{
                burns(
                    first: {}
                    orderBy: timestamp
                    orderDirection: desc
                    where: {{ pool: "{}" }}
                ) {{
                    owner
                    tickLower
                    tickUpper
                    amount
                    amount0
                    amount1
                }}
            }}"#,
            limit,
            pool_address.to_lowercase()
        );

        let response = self.query_graph(&query).await?;
        let burns = response["data"]["burns"].as_array()
            .ok_or_else(|| UniswapV3Error::ParseError("无法解析Burn事件列表".to_string()))?;

        let mut result = Vec::new();
        for burn in burns {
            result.push(DataConverter::json_to_pool_data("Burn", burn)?);
        }

        Ok(result)
    }

    /// 获取历史价格数据
    pub async fn get_historical_prices(
        &self,
        pool_address: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        interval: i32, // 时间间隔（秒）
    ) -> Result<Vec<(DateTime<Utc>, f64, f64)>, UniswapV3Error> {
        let query = format!(
            r#"{{
                poolDayDatas(
                    where: {{
                        pool: "{}",
                        date_gte: {},
                        date_lte: {}
                    }}
                    orderBy: date
                    orderDirection: asc
                ) {{
                    date
                    token0Price
                    token1Price
                    volumeToken0
                    volumeToken1
                }}
            }}"#,
            pool_address.to_lowercase(),
            start_time.timestamp(),
            end_time.timestamp()
        );

        let response = self.query_graph(&query).await?;
        let price_data = response["data"]["poolDayDatas"].as_array()
            .ok_or_else(|| UniswapV3Error::ParseError("无法解析价格数据".to_string()))?;

        let mut prices = Vec::new();
        for data in price_data {
            let timestamp = DateTime::from_timestamp(
                data["date"].as_i64()
                    .ok_or_else(|| UniswapV3Error::ParseError("无法解析时间戳".to_string()))?,
                0
            ).ok_or_else(|| UniswapV3Error::ParseError("无效的时间戳".to_string()))?;

            let token0_price = data["token0Price"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("无法解析token0价格".to_string()))?
                .parse::<f64>()
                .map_err(|e| UniswapV3Error::ParseError(format!("价格转换失败: {}", e)))?;

            let token1_price = data["token1Price"].as_str()
                .ok_or_else(|| UniswapV3Error::ParseError("无法解析token1价格".to_string()))?
                .parse::<f64>()
                .map_err(|e| UniswapV3Error::ParseError(format!("价格转换失败: {}", e)))?;

            prices.push((timestamp, token0_price, token1_price));
        }

        Ok(prices)
    }

    /// 获取流动性分布数据
    pub async fn get_liquidity_distribution(
        &self,
        pool_address: &str,
        tick_lower: i32,
        tick_upper: i32,
    ) -> Result<Vec<TickData>, UniswapV3Error> {
        let query = format!(
            r#"{{
                ticks(
                    where: {{
                        poolAddress: "{}",
                        tickIdx_gte: {},
                        tickIdx_lte: {}
                    }}
                    orderBy: tickIdx
                    first: 1000
                ) {{
                    tickIdx
                    liquidityGross
                    liquidityNet
                    price0
                    price1
                }}
            }}"#,
            pool_address.to_lowercase(),
            tick_lower,
            tick_upper
        );

        let response = self.query_graph(&query).await?;
        let tick_data = response["data"]["ticks"].as_array()
            .ok_or_else(|| UniswapV3Error::ParseError("无法解析Tick数据".to_string()))?;

        let mut ticks = Vec::new();
        for data in tick_data {
            let tick = TickData {
                tick_idx: data["tickIdx"].as_i64()
                    .ok_or_else(|| UniswapV3Error::ParseError("无法解析tick索引".to_string()))?,
                liquidity_gross: DataConverter::parse_decimal(&data["liquidityGross"])?,
                liquidity_net: DataConverter::parse_decimal(&data["liquidityNet"])?,
                price0: DataConverter::parse_decimal(&data["price0"])?,
                price1: DataConverter::parse_decimal(&data["price1"])?,
            };
            ticks.push(tick);
        }

        Ok(ticks)
    }

    /// 获取池子统计数据
    pub async fn get_pool_stats(
        &self,
        pool_address: &str,
    ) -> Result<(f64, f64, i64), UniswapV3Error> {
        let query = format!(
            r#"{{
                pool(id: "{}") {{
                    volumeUSD
                    feesUSD
                    txCount
                }}
            }}"#,
            pool_address.to_lowercase()
        );

        let response = self.query_graph(&query).await?;
        let stats = &response["data"]["pool"];

        let volume = stats["volumeUSD"].as_str()
            .ok_or_else(|| UniswapV3Error::ParseError("无法解析交易量".to_string()))?
            .parse::<f64>()
            .map_err(|e| UniswapV3Error::ParseError(format!("交易量转换失败: {}", e)))?;

        let fees = stats["feesUSD"].as_str()
            .ok_or_else(|| UniswapV3Error::ParseError("无法解析手续费".to_string()))?
            .parse::<f64>()
            .map_err(|e| UniswapV3Error::ParseError(format!("手续费转换失败: {}", e)))?;

        let tx_count = stats["txCount"].as_i64()
            .ok_or_else(|| UniswapV3Error::ParseError("无法解析交易数".to_string()))?;

        Ok((volume, fees, tx_count))
    }

    /// 查询Graph API
    async fn query_graph(&self, query: &str) -> Result<Value, UniswapV3Error> {
        let response = self.client
            .post(UNISWAP_V3_GRAPH_URL)
            .json(&serde_json::json!({
                "query": query
            }))
            .send()
            .await
            .map_err(|e| UniswapV3Error::RequestError(format!("Graph API请求失败: {}", e)))?;

        let json = response.json::<Value>()
            .await
            .map_err(|e| UniswapV3Error::JsonError(e))?;

        if let Some(errors) = json.get("errors") {
            return Err(UniswapV3Error::RequestError(
                format!("Graph API返回错误: {}", errors)
            ));
        }

        Ok(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_get_pool() {
        let collector = UniswapV3Collector::new();
        // USDC/ETH池子
        let pool_address = "0x8ad599c3a0ff1de082011efddc58f1908eb6e6d8";
        
        let result = collector.get_pool(pool_address).await;
        assert!(result.is_ok());
        
        let pool = result.unwrap();
        assert_eq!(pool.id.to_lowercase(), pool_address);
        assert_eq!(pool.fee_tier, 3000); // 0.3%费率
    }

    #[tokio::test]
    async fn test_get_token() {
        let collector = UniswapV3Collector::new();
        // USDC代币
        let token_address = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48";
        
        let result = collector.get_token(token_address).await;
        assert!(result.is_ok());
        
        let token = result.unwrap();
        assert_eq!(token.id.to_lowercase(), token_address);
        assert_eq!(token.symbol, "USDC");
        assert_eq!(token.decimals, 6);
    }

    #[tokio::test]
    async fn test_get_recent_swaps() {
        let collector = UniswapV3Collector::new();
        // USDC/ETH池子
        let pool_address = "0x8ad599c3a0ff1de082011efddc58f1908eb6e6d8";
        
        let result = collector.get_recent_swaps(pool_address, 5).await;
        assert!(result.is_ok());
        
        let swaps = result.unwrap();
        assert!(!swaps.is_empty());
        assert!(swaps.len() <= 5);
    }

    #[tokio::test]
    async fn test_get_historical_prices() {
        let collector = UniswapV3Collector::new();
        let pool_address = "0x8ad599c3a0ff1de082011efddc58f1908eb6e6d8";
        
        let end_time = Utc::now();
        let start_time = end_time - Duration::days(7);
        
        let result = collector.get_historical_prices(pool_address, start_time, end_time, 86400).await;
        assert!(result.is_ok());
        
        let prices = result.unwrap();
        assert!(!prices.is_empty());
        
        // 验证时间序列是否有序
        for i in 1..prices.len() {
            assert!(prices[i].0 > prices[i-1].0);
        }
    }

    #[tokio::test]
    async fn test_get_liquidity_distribution() {
        let collector = UniswapV3Collector::new();
        let pool_address = "0x8ad599c3a0ff1de082011efddc58f1908eb6e6d8";
        
        let result = collector.get_liquidity_distribution(pool_address, -1000, 1000).await;
        assert!(result.is_ok());
        
        let ticks = result.unwrap();
        assert!(!ticks.is_empty());
        
        // 验证tick是否有序
        for i in 1..ticks.len() {
            assert!(ticks[i].tick_idx > ticks[i-1].tick_idx);
        }
    }

    #[tokio::test]
    async fn test_get_pool_stats() {
        let collector = UniswapV3Collector::new();
        let pool_address = "0x8ad599c3a0ff1de082011efddc58f1908eb6e6d8";
        
        let result = collector.get_pool_stats(pool_address).await;
        assert!(result.is_ok());
        
        let (volume, fees, tx_count) = result.unwrap();
        assert!(volume > 0.0);
        assert!(fees > 0.0);
        assert!(tx_count > 0);
    }
} 