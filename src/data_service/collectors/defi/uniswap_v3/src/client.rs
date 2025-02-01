use std::sync::Arc;
use ethers::{
    providers::{Provider, Http, Ws, Middleware},
    types::{Address, U256, H256, Log},
    contract::Contract,
};
use reqwest::Client as HttpClient;
use serde_json::json;
use tokio::sync::mpsc;
use futures::StreamExt;

use crate::{
    error::UniswapV3Error,
    model::{PoolInfo, TokenInfo, SwapData, Position, TickData, FactoryData},
    constants::*,
};

pub struct UniswapV3Client {
    provider: Arc<Provider<Http>>,
    ws_provider: Option<Provider<Ws>>,
    http_client: HttpClient,
    graph_url: String,
    chain_id: u64,
}

impl UniswapV3Client {
    pub fn new(
        provider: Provider<Http>,
        ws_url: Option<&str>,
        graph_url: Option<&str>,
        chain_id: u64,
    ) -> Self {
        let ws_provider = ws_url
            .map(|url| Provider::<Ws>::connect(url))
            .transpose()
            .ok()
            .flatten();

        let graph_url = graph_url.map(|url| url.to_string()).unwrap_or_else(|| {
            match chain_id {
                1 => UNISWAP_V3_SUBGRAPH_URL,
                42161 => ARBITRUM_SUBGRAPH_URL,
                10 => OPTIMISM_SUBGRAPH_URL,
                _ => UNISWAP_V3_SUBGRAPH_URL,
            }.to_string()
        });

        Self {
            provider: Arc::new(provider),
            ws_provider,
            http_client: HttpClient::new(),
            graph_url,
            chain_id,
        }
    }

    pub async fn get_pool_info(&self, pool_address: &str) -> Result<PoolInfo, UniswapV3Error> {
        let query = json!({
            "query": r#"
            query getPoolInfo($address: String!) {
                pool(id: $address) {
                    id
                    token0 { id }
                    token1 { id }
                    feeTier
                    tickSpacing
                    liquidity
                    sqrtPrice
                    tick
                    observationIndex
                    observationCardinality
                    observationCardinalityNext
                    feeProtocol
                    unlocked
                }
            }
            "#,
            "variables": {
                "address": pool_address.to_lowercase()
            }
        });

        let response = self.http_client
            .post(&self.graph_url)
            .json(&query)
            .send()
            .await?
            .json()
            .await?;

        // TODO: Parse response into PoolInfo
        todo!()
    }

    pub async fn get_token_info(&self, token_address: &str) -> Result<TokenInfo, UniswapV3Error> {
        let query = json!({
            "query": r#"
            query getTokenInfo($address: String!) {
                token(id: $address) {
                    id
                    symbol
                    name
                    decimals
                    totalSupply
                    volume
                    volumeUSD
                    txCount
                    poolCount
                    totalValueLocked
                    totalValueLockedUSD
                    priceUSD
                    feesUSD
                }
            }
            "#,
            "variables": {
                "address": token_address.to_lowercase()
            }
        });

        let response = self.http_client
            .post(&self.graph_url)
            .json(&query)
            .send()
            .await?
            .json()
            .await?;

        // TODO: Parse response into TokenInfo
        todo!()
    }

    pub async fn subscribe_events(
        &self,
        pool_address: &str,
    ) -> Result<mpsc::Receiver<SwapData>, UniswapV3Error> {
        let ws_provider = self.ws_provider.as_ref()
            .ok_or_else(|| UniswapV3Error::Config("WebSocket provider not configured".to_string()))?;

        let pool_address = pool_address.parse::<Address>()
            .map_err(|e| UniswapV3Error::Other(e.to_string()))?;

        // TODO: Implement event subscription
        todo!()
    }

    pub async fn get_position(
        &self,
        token_id: u128,
    ) -> Result<Position, UniswapV3Error> {
        let query = json!({
            "query": r#"
            query getPosition($tokenId: String!) {
                position(id: $tokenId) {
                    id
                    owner
                    pool { id }
                    token0 { id }
                    token1 { id }
                    tickLower { tickIdx }
                    tickUpper { tickIdx }
                    liquidity
                    feeGrowthInside0LastX128
                    feeGrowthInside1LastX128
                    tokensOwed0
                    tokensOwed1
                }
            }
            "#,
            "variables": {
                "tokenId": token_id.to_string()
            }
        });

        let response = self.http_client
            .post(&self.graph_url)
            .json(&query)
            .send()
            .await?
            .json()
            .await?;

        // TODO: Parse response into Position
        todo!()
    }

    pub async fn get_tick_data(
        &self,
        pool_address: &str,
        tick: i32,
    ) -> Result<TickData, UniswapV3Error> {
        let query = json!({
            "query": r#"
            query getTickData($poolAddress: String!, $tickIdx: Int!) {
                ticks(
                    where: {
                        pool: $poolAddress,
                        tickIdx: $tickIdx
                    }
                ) {
                    tickIdx
                    liquidityGross
                    liquidityNet
                    price0
                    price1
                    feeGrowthOutside0X128
                    feeGrowthOutside1X128
                }
            }
            "#,
            "variables": {
                "poolAddress": pool_address.to_lowercase(),
                "tickIdx": tick
            }
        });

        let response = self.http_client
            .post(&self.graph_url)
            .json(&query)
            .send()
            .await?
            .json()
            .await?;

        // TODO: Parse response into TickData
        todo!()
    }

    pub async fn get_factory_data(&self) -> Result<FactoryData, UniswapV3Error> {
        let query = json!({
            "query": r#"
            {
                factory(id: "factory") {
                    poolCount
                    totalVolumeUSD
                    totalFeesUSD
                    totalValueLockedUSD
                    txCount
                }
            }
            "#
        });

        let response = self.http_client
            .post(&self.graph_url)
            .json(&query)
            .send()
            .await?
            .json()
            .await?;

        // TODO: Parse response into FactoryData
        todo!()
    }
} 