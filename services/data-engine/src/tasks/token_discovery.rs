use crate::collectors::birdeye::client::BirdeyeClient;
use crate::collectors::birdeye::config::BirdeyeConfig;
use crate::repository::token::ActiveToken;
use crate::repository::TokenRepository;
use chrono::Utc;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

pub struct TokenDiscoveryTask {
    birdeye_client: BirdeyeClient,
    token_repo: Arc<dyn TokenRepository>,
    min_liquidity_usd: f64,
    min_fdv: f64,
    max_fdv: f64,
}

impl TokenDiscoveryTask {
    pub fn new(
        birdeye_config: BirdeyeConfig,
        token_repo: Arc<dyn TokenRepository>,
        min_liquidity_usd: f64,
        min_fdv: f64,
        max_fdv: f64,
    ) -> Self {
        let client = BirdeyeClient::new(birdeye_config);

        Self {
            birdeye_client: client,
            token_repo,
            min_liquidity_usd,
            min_fdv,
            max_fdv,
        }
    }

    pub async fn run(&self) {
        info!("🔍 Starting Token Discovery Task...");

        // AlphaGPT fetches 500 tokens via pagination
        // Birdeye API: limit max 20, so we need 25 pages (20 × 25 = 500)
        let page_size = 20;
        let total_target = 500; // Match AlphaGPT
        let num_pages = total_target / page_size;

        let mut all_trending = Vec::new();

        // Paginate through trending tokens
        for page in 0..num_pages {
            let offset = page * page_size;

            match self
                .birdeye_client
                .get_trending_tokens(page_size, offset)
                .await
            {
                Ok(mut tokens) => {
                    info!(
                        "📄 Page {}/{}: {} tokens (offset={})",
                        page + 1,
                        num_pages,
                        tokens.len(),
                        offset
                    );
                    all_trending.append(&mut tokens);

                    // Rate limiting: sleep between requests (200ms)
                    if page < num_pages - 1 {
                        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to fetch page {} (offset={}): {}",
                        page + 1,
                        offset,
                        e
                    );
                    // Continue with what we have so far
                    break;
                }
            }
        }

        info!(
            "Fetched {} total trending tokens from Birdeye",
            all_trending.len()
        );

        // Filter by liquidity and FDV (AlphaGPT exact logic)
        let mut filtered_tokens = Vec::new();
        for token in all_trending {
            let liq = token.liquidity.unwrap_or(0.0);
            let fdv = token.fdv.unwrap_or(0.0);

            // AlphaGPT filtering thresholds
            if liq < self.min_liquidity_usd {
                continue;
            }
            if fdv < self.min_fdv {
                continue;
            }
            if fdv > self.max_fdv {
                continue;
            }

            filtered_tokens.push(token);
        }

        info!(
            "Filtered {} tokens (liq >${}, fdv ${}-${})",
            filtered_tokens.len(),
            self.min_liquidity_usd,
            self.min_fdv,
            if self.max_fdv.is_infinite() {
                "inf".to_string()
            } else {
                format!("{}", self.max_fdv)
            }
        );

        if filtered_tokens.is_empty() {
            warn!("No tokens passed the filter criteria");
            return;
        }

        // Map to ActiveToken struct
        let tokens_to_upsert: Vec<ActiveToken> = filtered_tokens
            .iter()
            .map(|t| ActiveToken {
                address: t.address.clone(),
                symbol: t.symbol.clone().unwrap_or_default(),
                name: t.name.clone(),
                decimals: t.decimals.unwrap_or(6) as i32,
                chain: "solana".to_string(),
                liquidity_usd: Decimal::from_f64(t.liquidity.unwrap_or(0.0)),
                fdv: Decimal::from_f64(t.fdv.unwrap_or(0.0)),
                market_cap: Decimal::from_f64(t.market_cap.unwrap_or(0.0)),
                volume_24h: Decimal::from_f64(t.volume_24h.unwrap_or(0.0)),
                price_change_24h: Decimal::from_f64(t.price_change_24h.unwrap_or(0.0)),
                first_discovered: Utc::now(),
                last_updated: Utc::now(),
                is_active: true,
                metadata: None,
            })
            .collect();

        match self.token_repo.upsert_tokens(tokens_to_upsert).await {
            Ok(_) => {
                info!(
                    "✅ Successfully upserted {} active tokens to DB",
                    filtered_tokens.len()
                );
            }
            Err(e) => {
                error!("Failed to upsert tokens: {}", e);
            }
        }

        // Deactivate stale tokens (not updated in 24 hours)
        match self.token_repo.deactivate_stale(24).await {
            Ok(count) => {
                info!("Deactivated {} stale tokens (>24h old)", count);
            }
            Err(e) => {
                warn!("Failed to deactivate stale tokens: {}", e);
            }
        }

        info!("Token Discovery Task completed");
    }
}
