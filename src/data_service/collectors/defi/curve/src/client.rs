use std::sync::Arc;
use ethers::{
    providers::{Provider, Http, Ws, Middleware},
    types::{Address, U256},
};
use reqwest::Client as HttpClient;
use serde_json::json;
use tokio::sync::mpsc;
use futures::StreamExt;

use crate::{
    error::CurveError,
    model::{PoolInfo, PriceData, SwapData, GaugeData, VotingEscrowData, FactoryData},
};

const CURVE_SUBGRAPH_URL: &str = "https://api.thegraph.com/subgraphs/name/convex-community/curve-mainnet";

pub struct CurveClient {
    provider: Arc<Provider<Http>>,
    ws_provider: Option<Provider<Ws>>,
    http_client: HttpClient,
    graph_url: String,
}

impl CurveClient {
    pub fn new(
        provider: Provider<Http>,
        ws_url: Option<&str>,
        graph_url: Option<&str>,
    ) -> Self {
        let ws_provider = ws_url
            .map(|url| Provider::<Ws>::connect(url))
            .transpose()
            .ok()
            .flatten();

        Self {
            provider: Arc::new(provider),
            ws_provider,
            http_client: HttpClient::new(),
            graph_url: graph_url.unwrap_or(CURVE_SUBGRAPH_URL).to_string(),
        }
    }

    pub async fn get_pool_info(&self, pool_address: &str) -> Result<PoolInfo, CurveError> {
        let query = json!({
            "query": r#"
            query getPoolInfo($address: String!) {
                pool(id: $address) {
                    address
                    name
                    coins
                    underlyingCoins
                    balances
                    a
                    fee
                    adminFee
                    virtualPrice
                    totalSupply
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

    pub async fn get_price_data(&self, token_address: &str) -> Result<PriceData, CurveError> {
        let query = json!({
            "query": r#"
            query getTokenPrice($address: String!) {
                token(id: $address) {
                    priceUSD
                    volume24h
                    totalValueLockedUSD
                    lastUpdateTimestamp
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

        // TODO: Parse response into PriceData
        todo!()
    }

    pub async fn subscribe_events(
        &self,
        pool_address: &str,
    ) -> Result<mpsc::Receiver<SwapData>, CurveError> {
        let ws_provider = self.ws_provider.as_ref()
            .ok_or_else(|| CurveError::Config("WebSocket provider not configured".to_string()))?;

        let pool_address = pool_address.parse::<Address>()
            .map_err(|e| CurveError::Other(e.to_string()))?;

        // TODO: Implement event subscription
        todo!()
    }

    pub async fn get_gauge_data(&self, gauge_address: &str) -> Result<GaugeData, CurveError> {
        let query = json!({
            "query": r#"
            query getGaugeInfo($address: String!) {
                liquidityGauge(id: $address) {
                    address
                    pool { id }
                    totalSupply
                    workingSupply
                    relativeWeight
                    inflationRate
                    rewardTokens
                    rewardRates
                }
            }
            "#,
            "variables": {
                "address": gauge_address.to_lowercase()
            }
        });

        let response = self.http_client
            .post(&self.graph_url)
            .json(&query)
            .send()
            .await?
            .json()
            .await?;

        // TODO: Parse response into GaugeData
        todo!()
    }

    pub async fn get_voting_escrow_data(
        &self,
        user_address: &str,
    ) -> Result<VotingEscrowData, CurveError> {
        let query = json!({
            "query": r#"
            query getVotingEscrow($address: String!) {
                votingEscrow(id: $address) {
                    lockedAmount
                    unlockTime
                    votingPower
                }
            }
            "#,
            "variables": {
                "address": user_address.to_lowercase()
            }
        });

        let response = self.http_client
            .post(&self.graph_url)
            .json(&query)
            .send()
            .await?
            .json()
            .await?;

        // TODO: Parse response into VotingEscrowData
        todo!()
    }

    pub async fn get_factory_data(&self) -> Result<FactoryData, CurveError> {
        let query = json!({
            "query": r#"
            {
                factory(id: "factory") {
                    implementation
                    poolCount
                    lastPoolAddress
                    lastPoolTimestamp
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