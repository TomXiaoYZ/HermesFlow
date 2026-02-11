use crate::collectors::birdeye::client::BirdeyeClient;
use crate::collectors::birdeye::config::BirdeyeConfig;
use crate::models::TokenMetadata;
use crate::repository::TokenRepository;
use crate::storage::RedisCache;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

pub struct BirdeyeMetaCollector {
    client: BirdeyeClient,
    redis: Arc<RwLock<RedisCache>>,
    token_repo: Arc<dyn TokenRepository>,
}

impl BirdeyeMetaCollector {
    pub fn new(
        config: BirdeyeConfig,
        redis: Arc<RwLock<RedisCache>>,
        token_repo: Arc<dyn TokenRepository>,
    ) -> Self {
        let client = BirdeyeClient::new(config);
        Self {
            client,
            redis,
            token_repo,
        }
    }

    pub async fn run(self) {
        let batch_size = 50;
        let mut current_index = 0;

        loop {
            // 1. Fetch active tokens
            let tokens = match self.token_repo.get_active_addresses().await {
                Ok(t) => t,
                Err(e) => {
                    error!("[Meta] Failed to fetch active tokens: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                    continue;
                }
            };

            if tokens.is_empty() {
                info!("[Meta] No active tokens. Sleeping...");
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                continue;
            }

            // 2. Select Batch
            if current_index >= tokens.len() {
                current_index = 0;
            }
            let end_index = (current_index + batch_size).min(tokens.len());
            let batch = &tokens[current_index..end_index];

            info!(
                "[Meta] Processing batch {}..{} (Total {})",
                current_index,
                end_index,
                tokens.len()
            );

            for address in batch {
                // Fetch Overview
                match self.client.get_token_overview(address).await {
                    Ok(overview) => {
                        // Store to Redis
                        let meta = TokenMetadata::new(
                            address.clone(),
                            overview.liquidity.unwrap_or(0.0),
                            overview.volume_24h.unwrap_or(0.0),
                            market_cap_to_fdv(overview.market_cap), // Using MC as FDV proxy if needed
                            chrono::Utc::now().timestamp(),
                        );

                        let mut redis = self.redis.write().await;
                        if let Err(e) = redis.store_token_metadata(&meta).await {
                            warn!("[Meta] Failed to cache metadata for {}: {}", address, e);
                        }
                    }
                    Err(e) => {
                        warn!("[Meta] Failed to fetch overview for {}: {}", address, e);
                    }
                }
                // Rate Limit sleep (1s per req to allow ~60 req/min)
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }

            current_index = end_index;
            if current_index >= tokens.len() {
                current_index = 0;
            }

            // Sleep 10 minutes (600 seconds) between batches to reduce API usage.
            // With 50 tokens per batch, this updates each token roughly every (num_batches * 10) minutes.
            // E.g., 500 tokens = 10 batches. Total cycle = 100 minutes.
            tokio::time::sleep(tokio::time::Duration::from_secs(600)).await;
        }
    }
}

fn market_cap_to_fdv(mc: Option<f64>) -> f64 {
    mc.unwrap_or(0.0)
}
