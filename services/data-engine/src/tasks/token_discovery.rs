use crate::collectors::birdeye::client::BirdeyeClient;
use crate::collectors::birdeye::config::BirdeyeConfig;
use crate::collectors::BirdeyeConnector;
use crate::repository::token::{ActiveToken, TokenRepository};
use crate::repository::MarketDataRepository;

use chrono::{Duration, Utc};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use std::sync::Arc;
use tracing::{error, info, warn};

pub struct TokenDiscoveryTask {
    birdeye_client: BirdeyeClient,
    config: BirdeyeConfig,
    token_repo: Arc<dyn TokenRepository>,
    market_repo: Arc<dyn MarketDataRepository>,
    min_liquidity_usd: f64,
    min_fdv: f64,
    max_fdv: f64,
}

impl TokenDiscoveryTask {
    pub fn new(
        birdeye_config: BirdeyeConfig,
        token_repo: Arc<dyn TokenRepository>,
        market_repo: Arc<dyn MarketDataRepository>,
        min_liquidity_usd: f64,
        min_fdv: f64,
        max_fdv: f64,
    ) -> Self {
        let client = BirdeyeClient::new(birdeye_config.clone());

        Self {
            birdeye_client: client,
            config: birdeye_config,
            token_repo,
            market_repo,
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

        // Filter by liquidity and FDV
        let mut filtered_tokens = Vec::new();
        for token in all_trending {
            let liq = token.liquidity.unwrap_or(0.0);
            let fdv = token.fdv.unwrap_or(0.0);

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

        if filtered_tokens.is_empty() {
            warn!("No tokens passed the filter criteria");
            return;
        }

        // ---------------------------------------------------------
        // New Token Detection & Auto-Backfill Trigger
        // ---------------------------------------------------------
        let existing_addresses = match self.token_repo.get_active_addresses().await {
            Ok(addrs) => addrs,
            Err(e) => {
                error!("Failed to fetch existing addresses: {}", e);
                return;
            }
        };

        // Identify new tokens
        let new_tokens: Vec<_> = filtered_tokens
            .iter()
            .filter(|t| !existing_addresses.contains(&t.address))
            .collect();

        if !new_tokens.is_empty() {
            info!(
                "✨ Found {} NEW tokens. Triggering Auto-Backfill...",
                new_tokens.len()
            );

            // Spawn background task for backfill
            // We clone necessary data for the async block
            let new_token_infos: Vec<(String, String)> = new_tokens
                .iter()
                .map(|t| (t.symbol.clone().unwrap_or_default(), t.address.clone()))
                .collect();

            let config_clone = self.config.clone();
            let market_repo_clone = self.market_repo.clone();

            tokio::spawn(async move {
                let connector = BirdeyeConnector::new(config_clone);
                let to_ts = Utc::now().timestamp();
                // Backfill 365 days (1 year) - note: API might limit to 30d
                let from_ts = Utc::now()
                    .checked_sub_signed(Duration::days(365))
                    .unwrap()
                    .timestamp();

                let resolutions = vec!["15m", "1H", "4H", "1D", "1W"];

                for (symbol, _addr) in new_token_infos {
                    for res in &resolutions {
                        info!(
                            "⏳ [Auto-Backfill] Fetching history for {} ({})...",
                            symbol, res
                        );

                        match connector
                            .fetch_history_candles(&symbol, res, from_ts, to_ts)
                            .await
                        {
                            Ok(candles) => {
                                let count = candles.len();
                                let mut upserted = 0;
                                for candle in candles {
                                    if let Ok(_) = market_repo_clone.insert_candle(&candle).await {
                                        upserted += 1;
                                    }
                                }
                                info!(
                                    "✅ [Auto-Backfill] Completed for {} ({}). Saved {}/{} candles.",
                                    symbol, res, upserted, count
                                );
                            }
                            Err(e) => {
                                error!("❌ [Auto-Backfill] Failed for {} ({}): {}", symbol, res, e);
                            }
                        }

                        // Tiny sleep to be nice to API
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    }
                }
            });
        }

        // Upsert to DB
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

        if let Err(e) = self.token_repo.upsert_tokens(tokens_to_upsert).await {
            error!("Failed to upsert tokens: {}", e);
        }

        // Deactivate stale
        if let Err(e) = self.token_repo.deactivate_stale(24).await {
            warn!("Failed to deactivate stale tokens: {}", e);
        }

        info!("Token Discovery Task completed");
    }
}
