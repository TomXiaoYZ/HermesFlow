use std::error::Error;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use crate::collectors::dexscreener::client::{DexScreenerClient, Pair};
use crate::collectors::dexscreener::config::DexScreenerConfig;

pub struct DexScreenerConnector {
    client: DexScreenerClient,
}

impl DexScreenerConnector {
    pub fn new(config: DexScreenerConfig) -> Self {
        let client = DexScreenerClient::new(config);
        Self { client }
    }

    /// Fetches the most liquid pair for a token
    pub async fn fetch_best_pair(&self, token_address: &str) -> Result<Option<Pair>, Box<dyn Error + Send + Sync>> {
        let pairs = self.client.get_token_pairs(token_address).await?;
        
        // Find pair with max liquidity USD
        let best_pair = pairs.into_iter().max_by(|a, b| {
            let liq_a = a.liquidity.as_ref().and_then(|l| l.usd).unwrap_or(0.0);
            let liq_b = b.liquidity.as_ref().and_then(|l| l.usd).unwrap_or(0.0);
            liq_a.partial_cmp(&liq_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(best_pair)
    }
}
