use crate::collectors::dexscreener::config::DexScreenerConfig;
use reqwest::Client;
use serde::Deserialize;
use std::error::Error;

pub struct DexScreenerClient {
    client: Client,
    config: DexScreenerConfig,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct DexScreenerResponse {
    pub schemaVersion: Option<String>,
    pub pairs: Option<Vec<Pair>>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct Pair {
    pub chainId: String,
    pub dexId: String,
    pub url: String,
    pub pairAddress: String,
    pub baseToken: Token,
    pub quoteToken: Token,
    pub priceNative: String,
    pub priceUsd: Option<String>,
    pub txns: Txns,
    pub volume: Volume,
    pub priceChange: PriceChange,
    pub liquidity: Option<Liquidity>,
    pub fdv: Option<f64>,
    pub pairCreatedAt: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct Token {
    pub address: String,
    pub name: String,
    pub symbol: String,
}

#[derive(Debug, Deserialize)]
pub struct Txns {
    pub m5: TxnStats,
    pub h1: TxnStats,
    pub h6: TxnStats,
    pub h24: TxnStats,
}

#[derive(Debug, Deserialize)]
pub struct TxnStats {
    pub buys: i32,
    pub sells: i32,
}

#[derive(Debug, Deserialize)]
pub struct Volume {
    pub h24: f64,
    pub h6: f64,
    pub h1: f64,
    pub m5: f64,
}

#[derive(Debug, Deserialize)]
pub struct PriceChange {
    pub m5: f64,
    pub h1: f64,
    pub h6: f64,
    pub h24: f64,
}

#[derive(Debug, Deserialize)]
pub struct Liquidity {
    pub usd: Option<f64>,
    pub base: f64,
    pub quote: f64,
}

impl DexScreenerClient {
    pub fn new(config: DexScreenerConfig) -> Self {
        let client = Client::new();
        Self { client, config }
    }

    pub async fn get_token_pairs(
        &self,
        token_address: &str,
    ) -> Result<Vec<Pair>, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/tokens/{}", self.config.base_url, token_address);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        let data: DexScreenerResponse = resp
            .json()
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        Ok(data.pairs.unwrap_or_default())
    }
}
