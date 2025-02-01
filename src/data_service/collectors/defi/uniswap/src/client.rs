use std::sync::Arc;
use ethers::{
    providers::{Provider, Http, Middleware, StreamExt, Ws},
    contract::Contract,
    types::{Address, Filter, H256},
};
use reqwest::Client as HttpClient;
use serde_json::json;
use tracing::{debug, error, info};
use tokio::sync::mpsc;

use crate::error::UniswapError;
use crate::models::{PoolInfo, PriceData, LiquidityData, SwapData, GraphResponse};

const POOL_INFO_QUERY: &str = r#"
query poolInfo($poolAddress: String!) {
    pool(id: $poolAddress) {
        id
        token0 { id decimals }
        token1 { id decimals }
        feeTier
        liquidity
        token0Price
        token1Price
        totalValueLockedToken0
        totalValueLockedToken1
    }
}"#;

const PRICE_DATA_QUERY: &str = r#"
query tokenPrices($tokenAddress: String!) {
    token(id: $tokenAddress) {
        id
        derivedETH
        totalValueLocked
        volume
        volumeUSD
        priceUSD
    }
}"#;

const SWAP_DATA_QUERY: &str = r#"
query swapData($poolAddress: String!, $limit: Int!) {
    swaps(
        first: $limit,
        orderBy: timestamp,
        orderDirection: desc,
        where: { pool: $poolAddress }
    ) {
        id
        timestamp
        transaction { id }
        sender
        recipient
        amount0
        amount1
        amountUSD
        sqrtPriceX96
    }
}"#;

const SWAP_EVENT_SIGNATURE: &str = "Swap(address,address,int256,int256,uint160,uint128,int24)";

/// Uniswap客户端
pub struct UniswapClient {
    /// 以太坊节点客户端
    provider: Provider<Http>,
    /// WebSocket客户端（用于事件订阅）
    ws_provider: Option<Provider<Ws>>,
    /// HTTP客户端
    http_client: HttpClient,
    /// Graph API地址
    graph_url: String,
}

impl UniswapClient {
    /// 创建新的Uniswap客户端
    pub fn new(provider: Provider<Http>, ws_url: Option<&str>, graph_url: &str) -> Self {
        let ws_provider = ws_url.map(|url| {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async {
                    Provider::<Ws>::connect(url).await.unwrap()
                })
        });

        Self {
            provider,
            ws_provider,
            http_client: HttpClient::new(),
            graph_url: graph_url.to_string(),
        }
    }

    /// 获取池子信息
    pub async fn get_pool_info(&self, pool_address: &str) -> Result<PoolInfo, UniswapError> {
        let query = json!({
            "query": POOL_INFO_QUERY,
            "variables": {
                "poolAddress": pool_address.to_lowercase()
            }
        });

        let response = self.http_client
            .post(&self.graph_url)
            .json(&query)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(UniswapError::GraphError(format!(
                "Failed to get pool info: {}",
                error_text
            )));
        }

        let graph_response: GraphResponse<serde_json::Value> = response.json().await?;
        let pool_data = graph_response.data.get("pool")
            .ok_or_else(|| UniswapError::GraphError("Pool data not found".to_string()))?;

        // 解析池子信息
        let pool_info = PoolInfo {
            address: pool_address.to_string(),
            token0: pool_data["token0"]["id"].as_str()
                .ok_or_else(|| UniswapError::ParseError("token0 id not found".to_string()))?
                .to_string(),
            token1: pool_data["token1"]["id"].as_str()
                .ok_or_else(|| UniswapError::ParseError("token1 id not found".to_string()))?
                .to_string(),
            decimals0: pool_data["token0"]["decimals"].as_u64()
                .ok_or_else(|| UniswapError::ParseError("token0 decimals not found".to_string()))? as u8,
            decimals1: pool_data["token1"]["decimals"].as_u64()
                .ok_or_else(|| UniswapError::ParseError("token1 decimals not found".to_string()))? as u8,
            reserve0: pool_data["totalValueLockedToken0"].as_str()
                .ok_or_else(|| UniswapError::ParseError("reserve0 not found".to_string()))?
                .to_string(),
            reserve1: pool_data["totalValueLockedToken1"].as_str()
                .ok_or_else(|| UniswapError::ParseError("reserve1 not found".to_string()))?
                .to_string(),
            fee: pool_data["feeTier"].as_u64()
                .ok_or_else(|| UniswapError::ParseError("fee not found".to_string()))? as u32,
            liquidity: pool_data["liquidity"].as_str()
                .ok_or_else(|| UniswapError::ParseError("liquidity not found".to_string()))?
                .to_string(),
            last_update: chrono::Utc::now(),
        };

        Ok(pool_info)
    }

    /// 获取价格数据
    pub async fn get_price_data(&self, token_address: &str) -> Result<PriceData, UniswapError> {
        let query = json!({
            "query": PRICE_DATA_QUERY,
            "variables": {
                "tokenAddress": token_address.to_lowercase()
            }
        });

        let response = self.http_client
            .post(&self.graph_url)
            .json(&query)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(UniswapError::GraphError(format!(
                "Failed to get price data: {}",
                error_text
            )));
        }

        let graph_response: GraphResponse<serde_json::Value> = response.json().await?;
        let token_data = graph_response.data.get("token")
            .ok_or_else(|| UniswapError::GraphError("Token data not found".to_string()))?;

        // 解析价格数据
        let price_data = PriceData {
            token: token_address.to_string(),
            price_eth: token_data["derivedETH"].as_str()
                .ok_or_else(|| UniswapError::ParseError("ETH price not found".to_string()))?
                .parse()
                .map_err(|e| UniswapError::ParseError(format!("Failed to parse ETH price: {}", e)))?,
            price_usd: token_data["priceUSD"].as_str()
                .ok_or_else(|| UniswapError::ParseError("USD price not found".to_string()))?
                .parse()
                .map_err(|e| UniswapError::ParseError(format!("Failed to parse USD price: {}", e)))?,
            price_change_24h: "0".parse().unwrap(), // TODO: 实现24h价格变化计算
            volume_24h: token_data["volumeUSD"].as_str()
                .ok_or_else(|| UniswapError::ParseError("Volume not found".to_string()))?
                .parse()
                .map_err(|e| UniswapError::ParseError(format!("Failed to parse volume: {}", e)))?,
            tvl: token_data["totalValueLocked"].as_str()
                .ok_or_else(|| UniswapError::ParseError("TVL not found".to_string()))?
                .parse()
                .map_err(|e| UniswapError::ParseError(format!("Failed to parse TVL: {}", e)))?,
            timestamp: chrono::Utc::now(),
        };

        Ok(price_data)
    }

    /// 获取流动性数据
    pub async fn get_liquidity_data(&self, pool_address: &str) -> Result<LiquidityData, UniswapError> {
        let pool_info = self.get_pool_info(pool_address).await?;

        let liquidity_data = LiquidityData {
            pool: pool_address.to_string(),
            total_liquidity: pool_info.liquidity.parse()
                .map_err(|e| UniswapError::ParseError(format!("Failed to parse liquidity: {}", e)))?,
            token0_liquidity: pool_info.reserve0.parse()
                .map_err(|e| UniswapError::ParseError(format!("Failed to parse token0 liquidity: {}", e)))?,
            token1_liquidity: pool_info.reserve1.parse()
                .map_err(|e| UniswapError::ParseError(format!("Failed to parse token1 liquidity: {}", e)))?,
            uncollected_fees0: "0".parse().unwrap(), // TODO: 实现未收取手续费计算
            uncollected_fees1: "0".parse().unwrap(),
            timestamp: chrono::Utc::now(),
        };

        Ok(liquidity_data)
    }

    /// 获取交易数据
    pub async fn get_swap_data(&self, pool_address: &str) -> Result<Vec<SwapData>, UniswapError> {
        let query = json!({
            "query": SWAP_DATA_QUERY,
            "variables": {
                "poolAddress": pool_address.to_lowercase(),
                "limit": 100  // 默认获取最近100条交易
            }
        });

        let response = self.http_client
            .post(&self.graph_url)
            .json(&query)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(UniswapError::GraphError(format!(
                "Failed to get swap data: {}",
                error_text
            )));
        }

        let graph_response: GraphResponse<serde_json::Value> = response.json().await?;
        let swaps = graph_response.data.get("swaps")
            .ok_or_else(|| UniswapError::GraphError("Swaps data not found".to_string()))?
            .as_array()
            .ok_or_else(|| UniswapError::GraphError("Swaps data is not an array".to_string()))?;

        let mut swap_data = Vec::new();
        for swap in swaps {
            let timestamp = swap["timestamp"].as_str()
                .ok_or_else(|| UniswapError::ParseError("timestamp not found".to_string()))?
                .parse::<i64>()
                .map_err(|e| UniswapError::ParseError(format!("Failed to parse timestamp: {}", e)))?;

            let amount0: rust_decimal::Decimal = swap["amount0"].as_str()
                .ok_or_else(|| UniswapError::ParseError("amount0 not found".to_string()))?
                .parse()
                .map_err(|e| UniswapError::ParseError(format!("Failed to parse amount0: {}", e)))?;

            let amount1: rust_decimal::Decimal = swap["amount1"].as_str()
                .ok_or_else(|| UniswapError::ParseError("amount1 not found".to_string()))?
                .parse()
                .map_err(|e| UniswapError::ParseError(format!("Failed to parse amount1: {}", e)))?;

            let amount_usd: rust_decimal::Decimal = swap["amountUSD"].as_str()
                .ok_or_else(|| UniswapError::ParseError("amountUSD not found".to_string()))?
                .parse()
                .map_err(|e| UniswapError::ParseError(format!("Failed to parse amountUSD: {}", e)))?;

            // 计算价格（使用USD金额除以代币数量）
            let price = if amount0.abs() > rust_decimal::Decimal::ZERO {
                amount_usd / amount0.abs()
            } else {
                amount_usd / amount1.abs()
            };

            swap_data.push(SwapData {
                tx_hash: swap["transaction"]["id"].as_str()
                    .ok_or_else(|| UniswapError::ParseError("transaction id not found".to_string()))?
                    .to_string(),
                pool: pool_address.to_string(),
                sender: swap["sender"].as_str()
                    .ok_or_else(|| UniswapError::ParseError("sender not found".to_string()))?
                    .to_string(),
                recipient: swap["recipient"].as_str()
                    .ok_or_else(|| UniswapError::ParseError("recipient not found".to_string()))?
                    .to_string(),
                amount0,
                amount1,
                price,
                fee: (amount_usd * rust_decimal::Decimal::new(3, 3)), // 0.3% 手续费
                timestamp: chrono::DateTime::from_timestamp(timestamp, 0)
                    .ok_or_else(|| UniswapError::ParseError("Invalid timestamp".to_string()))?,
            });
        }

        Ok(swap_data)
    }

    /// 订阅事件
    pub async fn subscribe_events(&self, pool_address: &str) -> Result<mpsc::Receiver<SwapData>, UniswapError> {
        let ws_provider = self.ws_provider.as_ref()
            .ok_or_else(|| UniswapError::ConfigError("WebSocket provider not configured".to_string()))?;

        let pool_address = pool_address.parse::<Address>()
            .map_err(|e| UniswapError::ParseError(format!("Invalid pool address: {}", e)))?;

        // 创建事件过滤器
        let filter = Filter::new()
            .address(pool_address)
            .event(SWAP_EVENT_SIGNATURE);

        // 创建事件流
        let mut event_stream = ws_provider.subscribe_logs(&filter).await
            .map_err(|e| UniswapError::EventError(format!("Failed to subscribe to events: {}", e)))?;

        // 创建通道
        let (tx, rx) = mpsc::channel(100);
        let provider = ws_provider.clone();

        // 启动事件处理任务
        tokio::spawn(async move {
            while let Some(log) = event_stream.next().await {
                match Self::parse_swap_event(&log) {
                    Ok(swap_data) => {
                        if tx.send(swap_data).await.is_err() {
                            error!("Failed to send swap data through channel");
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse swap event: {}", e);
                    }
                }
            }
        });

        Ok(rx)
    }

    /// 解析Swap事件
    fn parse_swap_event(log: &ethers::types::Log) -> Result<SwapData, UniswapError> {
        let topics = &log.topics;
        if topics.len() != 3 {
            return Err(UniswapError::EventError("Invalid number of topics".to_string()));
        }

        // 解析事件数据
        let data = log.data.to_vec();
        if data.len() < 128 {  // 4 * 32 bytes
            return Err(UniswapError::EventError("Invalid data length".to_string()));
        }

        // 解析数据字段
        let amount0: i128 = ethers::abi::decode(
            &[ethers::abi::ParamType::Int(256)],
            &data[0..32]
        )
        .map_err(|e| UniswapError::ParseError(format!("Failed to decode amount0: {}", e)))?
        .pop()
        .ok_or_else(|| UniswapError::ParseError("No amount0 data".to_string()))?
        .into_int()
        .ok_or_else(|| UniswapError::ParseError("Invalid amount0 type".to_string()))?
        .as_i128();

        let amount1: i128 = ethers::abi::decode(
            &[ethers::abi::ParamType::Int(256)],
            &data[32..64]
        )
        .map_err(|e| UniswapError::ParseError(format!("Failed to decode amount1: {}", e)))?
        .pop()
        .ok_or_else(|| UniswapError::ParseError("No amount1 data".to_string()))?
        .into_int()
        .ok_or_else(|| UniswapError::ParseError("Invalid amount1 type".to_string()))?
        .as_i128();

        let sqrt_price_x96: u128 = ethers::abi::decode(
            &[ethers::abi::ParamType::Uint(160)],
            &data[64..96]
        )
        .map_err(|e| UniswapError::ParseError(format!("Failed to decode sqrtPriceX96: {}", e)))?
        .pop()
        .ok_or_else(|| UniswapError::ParseError("No sqrtPriceX96 data".to_string()))?
        .into_uint()
        .ok_or_else(|| UniswapError::ParseError("Invalid sqrtPriceX96 type".to_string()))?
        .as_u128();

        // 计算价格
        let price = Self::calculate_price_from_sqrt_x96(sqrt_price_x96)?;

        // 构造SwapData
        Ok(SwapData {
            tx_hash: log.transaction_hash
                .ok_or_else(|| UniswapError::ParseError("No transaction hash".to_string()))?
                .to_string(),
            pool: log.address.to_string(),
            sender: format!("0x{:x}", topics[1]),
            recipient: format!("0x{:x}", topics[2]),
            amount0: rust_decimal::Decimal::from_i128(amount0)
                .ok_or_else(|| UniswapError::ParseError("Failed to convert amount0".to_string()))?,
            amount1: rust_decimal::Decimal::from_i128(amount1)
                .ok_or_else(|| UniswapError::ParseError("Failed to convert amount1".to_string()))?,
            price,
            fee: rust_decimal::Decimal::new(3, 3), // 0.3% 手续费
            timestamp: chrono::Utc::now(), // 使用当前时间作为事件时间
        })
    }

    /// 从sqrt_price_x96计算价格
    fn calculate_price_from_sqrt_x96(sqrt_price_x96: u128) -> Result<rust_decimal::Decimal, UniswapError> {
        // price = (sqrtPriceX96 * sqrtPriceX96 * (10^decimals)) >> (96 * 2)
        let price_u128 = (sqrt_price_x96 as u128)
            .checked_mul(sqrt_price_x96 as u128)
            .ok_or_else(|| UniswapError::ParseError("Price overflow".to_string()))?
            .checked_mul(10u128.pow(18))
            .ok_or_else(|| UniswapError::ParseError("Price overflow".to_string()))?
            .checked_shr(192)
            .ok_or_else(|| UniswapError::ParseError("Price overflow".to_string()))?;

        rust_decimal::Decimal::from_u128(price_u128)
            .ok_or_else(|| UniswapError::ParseError("Failed to convert price".to_string()))
    }
} 