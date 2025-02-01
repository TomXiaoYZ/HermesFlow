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

use crate::error::PancakeSwapError;
use crate::models::{PoolInfo, PriceData, LiquidityData, SwapData, FarmData, PredictionData, GraphResponse};

const POOL_INFO_QUERY: &str = r#"
query poolInfo($poolAddress: String!) {
    pair(id: $poolAddress) {
        id
        token0 { id symbol decimals }
        token1 { id symbol decimals }
        reserveUSD
        reserve0
        reserve1
        token0Price
        token1Price
    }
}"#;

const PRICE_DATA_QUERY: &str = r#"
query tokenPrices($tokenAddress: String!) {
    token(id: $tokenAddress) {
        id
        derivedBNB
        derivedUSD
        totalLiquidity
        tradeVolume
        tradeVolumeUSD
    }
}"#;

const FARM_DATA_QUERY: &str = r#"
query farmInfo($farmId: String!) {
    farm(id: $farmId) {
        id
        lpToken
        rewardToken
        apr
        totalStaked
        rewardPerBlock
    }
}"#;

const PREDICTION_DATA_QUERY: &str = r#"
query predictionRound($roundId: String!) {
    round(id: $roundId) {
        id
        epoch
        startPrice
        closePrice
        totalAmount
        bullAmount
        bearAmount
        startTimestamp
        closeTimestamp
    }
}"#;

const SWAP_EVENT_SIGNATURE: &str = "Swap(address,address,uint256,uint256,uint256,uint256)";

/// PancakeSwap客户端
pub struct PancakeSwapClient {
    /// BSC节点客户端
    provider: Provider<Http>,
    /// WebSocket客户端
    ws_provider: Option<Provider<Ws>>,
    /// HTTP客户端
    http_client: HttpClient,
    /// Graph API地址
    graph_url: String,
}

impl PancakeSwapClient {
    /// 创建新的PancakeSwap客户端
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
    pub async fn get_pool_info(&self, pool_address: &str) -> Result<PoolInfo, PancakeSwapError> {
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
            return Err(PancakeSwapError::GraphError(format!(
                "Failed to get pool info: {}",
                error_text
            )));
        }

        let graph_response: GraphResponse<serde_json::Value> = response.json().await?;
        let pair = graph_response.data.get("pair")
            .ok_or_else(|| PancakeSwapError::GraphError("Pair data not found".to_string()))?;

        Ok(PoolInfo {
            address: pool_address.to_string(),
            token0: pair["token0"]["id"].as_str()
                .ok_or_else(|| PancakeSwapError::ParseError("token0 id not found".to_string()))?
                .to_string(),
            token1: pair["token1"]["id"].as_str()
                .ok_or_else(|| PancakeSwapError::ParseError("token1 id not found".to_string()))?
                .to_string(),
            fee_tier: "25".to_string(), // PancakeSwap v2 固定0.25%手续费
            liquidity: pair["reserveUSD"].as_str()
                .ok_or_else(|| PancakeSwapError::ParseError("liquidity not found".to_string()))?
                .to_string(),
            token0_price: pair["token0Price"].as_str()
                .ok_or_else(|| PancakeSwapError::ParseError("token0 price not found".to_string()))?
                .to_string(),
            token1_price: pair["token1Price"].as_str()
                .ok_or_else(|| PancakeSwapError::ParseError("token1 price not found".to_string()))?
                .to_string(),
            reserve0: pair["reserve0"].as_str()
                .ok_or_else(|| PancakeSwapError::ParseError("reserve0 not found".to_string()))?
                .to_string(),
            reserve1: pair["reserve1"].as_str()
                .ok_or_else(|| PancakeSwapError::ParseError("reserve1 not found".to_string()))?
                .to_string(),
        })
    }

    /// 获取价格数据
    pub async fn get_price_data(&self, token_address: &str) -> Result<PriceData, PancakeSwapError> {
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
            return Err(PancakeSwapError::GraphError(format!(
                "Failed to get price data: {}",
                error_text
            )));
        }

        let graph_response: GraphResponse<serde_json::Value> = response.json().await?;
        let token_data = graph_response.data.get("token")
            .ok_or_else(|| PancakeSwapError::GraphError("Token data not found".to_string()))?;

        Ok(PriceData {
            token: token_address.to_string(),
            price_bnb: token_data["derivedBNB"].as_str()
                .ok_or_else(|| PancakeSwapError::ParseError("BNB price not found".to_string()))?
                .parse()
                .map_err(|e| PancakeSwapError::ParseError(format!("Failed to parse BNB price: {}", e)))?,
            price_usd: token_data["derivedUSD"].as_str()
                .ok_or_else(|| PancakeSwapError::ParseError("USD price not found".to_string()))?
                .parse()
                .map_err(|e| PancakeSwapError::ParseError(format!("Failed to parse USD price: {}", e)))?,
            price_change_24h: "0".parse().unwrap(), // TODO: 实现24h价格变化计算
            volume_24h: token_data["tradeVolumeUSD"].as_str()
                .ok_or_else(|| PancakeSwapError::ParseError("Volume not found".to_string()))?
                .parse()
                .map_err(|e| PancakeSwapError::ParseError(format!("Failed to parse volume: {}", e)))?,
            tvl: token_data["totalLiquidity"].as_str()
                .ok_or_else(|| PancakeSwapError::ParseError("TVL not found".to_string()))?
                .parse()
                .map_err(|e| PancakeSwapError::ParseError(format!("Failed to parse TVL: {}", e)))?,
            timestamp: chrono::Utc::now(),
        })
    }

    /// 获取农场数据
    pub async fn get_farm_data(&self, farm_id: &str) -> Result<FarmData, PancakeSwapError> {
        let query = json!({
            "query": FARM_DATA_QUERY,
            "variables": {
                "farmId": farm_id
            }
        });

        let response = self.http_client
            .post(&self.graph_url)
            .json(&query)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(PancakeSwapError::GraphError(format!(
                "Failed to get farm data: {}",
                error_text
            )));
        }

        let graph_response: GraphResponse<serde_json::Value> = response.json().await?;
        let farm_data = graph_response.data.get("farm")
            .ok_or_else(|| PancakeSwapError::GraphError("Farm data not found".to_string()))?;

        Ok(FarmData {
            farm_id: farm_id.to_string(),
            lp_token: farm_data["lpToken"].as_str()
                .ok_or_else(|| PancakeSwapError::ParseError("LP token not found".to_string()))?
                .to_string(),
            reward_token: farm_data["rewardToken"].as_str()
                .ok_or_else(|| PancakeSwapError::ParseError("Reward token not found".to_string()))?
                .to_string(),
            apr: farm_data["apr"].as_str()
                .ok_or_else(|| PancakeSwapError::ParseError("APR not found".to_string()))?
                .parse()
                .map_err(|e| PancakeSwapError::ParseError(format!("Failed to parse APR: {}", e)))?,
            total_staked: farm_data["totalStaked"].as_str()
                .ok_or_else(|| PancakeSwapError::ParseError("Total staked not found".to_string()))?
                .parse()
                .map_err(|e| PancakeSwapError::ParseError(format!("Failed to parse total staked: {}", e)))?,
            reward_per_block: farm_data["rewardPerBlock"].as_str()
                .ok_or_else(|| PancakeSwapError::ParseError("Reward per block not found".to_string()))?
                .parse()
                .map_err(|e| PancakeSwapError::ParseError(format!("Failed to parse reward per block: {}", e)))?,
            timestamp: chrono::Utc::now(),
        })
    }

    /// 获取预测市场数据
    pub async fn get_prediction_data(&self, round_id: u64) -> Result<PredictionData, PancakeSwapError> {
        let query = json!({
            "query": PREDICTION_DATA_QUERY,
            "variables": {
                "roundId": round_id.to_string()
            }
        });

        let response = self.http_client
            .post(&self.graph_url)
            .json(&query)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(PancakeSwapError::GraphError(format!(
                "Failed to get prediction data: {}",
                error_text
            )));
        }

        let graph_response: GraphResponse<serde_json::Value> = response.json().await?;
        let round_data = graph_response.data.get("round")
            .ok_or_else(|| PancakeSwapError::GraphError("Round data not found".to_string()))?;

        Ok(PredictionData {
            round_id,
            start_price: round_data["startPrice"].as_str()
                .ok_or_else(|| PancakeSwapError::ParseError("Start price not found".to_string()))?
                .parse()
                .map_err(|e| PancakeSwapError::ParseError(format!("Failed to parse start price: {}", e)))?,
            end_price: round_data["closePrice"].as_str()
                .ok_or_else(|| PancakeSwapError::ParseError("End price not found".to_string()))?
                .parse()
                .map_err(|e| PancakeSwapError::ParseError(format!("Failed to parse end price: {}", e)))?,
            bull_amount: round_data["bullAmount"].as_str()
                .ok_or_else(|| PancakeSwapError::ParseError("Bull amount not found".to_string()))?
                .parse()
                .map_err(|e| PancakeSwapError::ParseError(format!("Failed to parse bull amount: {}", e)))?,
            bear_amount: round_data["bearAmount"].as_str()
                .ok_or_else(|| PancakeSwapError::ParseError("Bear amount not found".to_string()))?
                .parse()
                .map_err(|e| PancakeSwapError::ParseError(format!("Failed to parse bear amount: {}", e)))?,
            start_time: chrono::DateTime::from_timestamp(
                round_data["startTimestamp"].as_str()
                    .ok_or_else(|| PancakeSwapError::ParseError("Start timestamp not found".to_string()))?
                    .parse::<i64>()
                    .map_err(|e| PancakeSwapError::ParseError(format!("Failed to parse start timestamp: {}", e)))?,
                0
            ).ok_or_else(|| PancakeSwapError::ParseError("Invalid start timestamp".to_string()))?,
            end_time: chrono::DateTime::from_timestamp(
                round_data["closeTimestamp"].as_str()
                    .ok_or_else(|| PancakeSwapError::ParseError("End timestamp not found".to_string()))?
                    .parse::<i64>()
                    .map_err(|e| PancakeSwapError::ParseError(format!("Failed to parse end timestamp: {}", e)))?,
                0
            ).ok_or_else(|| PancakeSwapError::ParseError("Invalid end timestamp".to_string()))?,
        })
    }

    /// 订阅交易事件
    pub async fn subscribe_events(&self, pool_address: &str) -> Result<mpsc::Receiver<SwapData>, PancakeSwapError> {
        let ws_provider = self.ws_provider.as_ref()
            .ok_or_else(|| PancakeSwapError::ConfigError("WebSocket provider not configured".to_string()))?;

        let pool_address = pool_address.parse::<Address>()
            .map_err(|e| PancakeSwapError::ParseError(format!("Invalid pool address: {}", e)))?;

        // 创建事件过滤器
        let filter = Filter::new()
            .address(pool_address)
            .event(SWAP_EVENT_SIGNATURE);

        // 创建事件流
        let mut event_stream = ws_provider.subscribe_logs(&filter).await
            .map_err(|e| PancakeSwapError::EventError(format!("Failed to subscribe to events: {}", e)))?;

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

    /// 解析交易事件
    fn parse_swap_event(log: &ethers::types::Log) -> Result<SwapData, PancakeSwapError> {
        let topics = &log.topics;
        if topics.len() != 3 {
            return Err(PancakeSwapError::EventError("Invalid number of topics".to_string()));
        }

        // 解析事件数据
        let data = log.data.to_vec();
        if data.len() < 128 {  // 4 * 32 bytes
            return Err(PancakeSwapError::EventError("Invalid data length".to_string()));
        }

        // 解析数据字段
        let amount0_in: u128 = ethers::abi::decode(
            &[ethers::abi::ParamType::Uint(256)],
            &data[0..32]
        )
        .map_err(|e| PancakeSwapError::ParseError(format!("Failed to decode amount0_in: {}", e)))?
        .pop()
        .ok_or_else(|| PancakeSwapError::ParseError("No amount0_in data".to_string()))?
        .into_uint()
        .ok_or_else(|| PancakeSwapError::ParseError("Invalid amount0_in type".to_string()))?
        .as_u128();

        let amount1_in: u128 = ethers::abi::decode(
            &[ethers::abi::ParamType::Uint(256)],
            &data[32..64]
        )
        .map_err(|e| PancakeSwapError::ParseError(format!("Failed to decode amount1_in: {}", e)))?
        .pop()
        .ok_or_else(|| PancakeSwapError::ParseError("No amount1_in data".to_string()))?
        .into_uint()
        .ok_or_else(|| PancakeSwapError::ParseError("Invalid amount1_in type".to_string()))?
        .as_u128();

        let amount0_out: u128 = ethers::abi::decode(
            &[ethers::abi::ParamType::Uint(256)],
            &data[64..96]
        )
        .map_err(|e| PancakeSwapError::ParseError(format!("Failed to decode amount0_out: {}", e)))?
        .pop()
        .ok_or_else(|| PancakeSwapError::ParseError("No amount0_out data".to_string()))?
        .into_uint()
        .ok_or_else(|| PancakeSwapError::ParseError("Invalid amount0_out type".to_string()))?
        .as_u128();

        let amount1_out: u128 = ethers::abi::decode(
            &[ethers::abi::ParamType::Uint(256)],
            &data[96..128]
        )
        .map_err(|e| PancakeSwapError::ParseError(format!("Failed to decode amount1_out: {}", e)))?
        .pop()
        .ok_or_else(|| PancakeSwapError::ParseError("No amount1_out data".to_string()))?
        .into_uint()
        .ok_or_else(|| PancakeSwapError::ParseError("Invalid amount1_out type".to_string()))?
        .as_u128();

        // 计算净额
        let amount0 = (amount0_out as i128) - (amount0_in as i128);
        let amount1 = (amount1_out as i128) - (amount1_in as i128);

        // 计算价格（使用较大的金额作为基准）
        let price = if amount0.abs() > amount1.abs() {
            rust_decimal::Decimal::from(amount1.abs()) / rust_decimal::Decimal::from(amount0.abs())
        } else {
            rust_decimal::Decimal::from(amount0.abs()) / rust_decimal::Decimal::from(amount1.abs())
        };

        // 构造SwapData
        Ok(SwapData {
            tx_hash: log.transaction_hash
                .ok_or_else(|| PancakeSwapError::ParseError("No transaction hash".to_string()))?
                .to_string(),
            pool: log.address.to_string(),
            sender: format!("0x{:x}", topics[1]),
            recipient: format!("0x{:x}", topics[2]),
            amount0: rust_decimal::Decimal::from_i128(amount0)
                .ok_or_else(|| PancakeSwapError::ParseError("Failed to convert amount0".to_string()))?,
            amount1: rust_decimal::Decimal::from_i128(amount1)
                .ok_or_else(|| PancakeSwapError::ParseError("Failed to convert amount1".to_string()))?,
            price,
            fee: rust_decimal::Decimal::new(25, 4), // 0.25% 手续费
            timestamp: chrono::Utc::now(),
        })
    }

    // 其他方法将在后续实现...
} 