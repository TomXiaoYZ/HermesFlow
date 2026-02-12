use crate::collectors::jupiter::client::{JupiterClient, JupiterPriceItem};
use crate::collectors::jupiter::config::JupiterConfig;
use crate::error::{retry_with_backoff, DataError};
use crate::repository::TokenRepository;
use crate::traits::ConnectorStats;
use chrono::Utc;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::storage::RedisCache;

pub struct JupiterPriceCollector {
    client: JupiterClient,
    config: JupiterConfig,
    redis: Option<RedisCache>,
    stats: Arc<RwLock<ConnectorStats>>,
}

impl JupiterPriceCollector {
    pub fn new(config: JupiterConfig, redis: Option<RedisCache>) -> Self {
        let client = JupiterClient::new(config.clone());
        Self {
            client,
            config,
            redis,
            stats: Arc::new(RwLock::new(ConnectorStats::default())),
        }
    }

    /// Returns the current connector stats
    pub fn stats(&self) -> ConnectorStats {
        match self.stats.try_read() {
            Ok(guard) => guard.clone(),
            Err(_) => ConnectorStats::default(),
        }
    }

    pub async fn connect(
        &self,
        token_repo: Arc<dyn TokenRepository>,
    ) -> Result<
        tokio::sync::mpsc::Receiver<crate::models::StandardMarketData>,
        Box<dyn Error + Send + Sync>,
    > {
        let (tx, rx) = tokio::sync::mpsc::channel(10_000);
        let client = self.client.clone();
        let mut redis = self.redis.clone(); // Clone the cache (cheap)
        let poll_interval = self.config.poll_interval_secs;
        let stats = self.stats.clone();

        tokio::spawn(async move {
            let mut cached_symbols: Vec<String> = Vec::new();
            let mut last_refresh = std::time::Instant::now();
            let refresh_interval = std::time::Duration::from_secs(300); // 5 minutes

            loop {
                // 1. Refresh symbol list from database every 5 minutes
                if cached_symbols.is_empty() || last_refresh.elapsed() >= refresh_interval {
                    match token_repo.get_active_addresses().await {
                        Ok(symbols) => {
                            if symbols.len() != cached_symbols.len() {
                                info!(
                                    "🔄 [Jupiter] Refreshed active watchlist from DB: {} tokens",
                                    symbols.len()
                                );
                            }
                            // Filter valid solana addresses roughly
                            cached_symbols = symbols
                                .into_iter()
                                .filter(|s| s.len() > 30) // Basic check
                                .collect();
                            last_refresh = std::time::Instant::now();
                        }
                        Err(e) => {
                            warn!(
                                "[Jupiter] Failed to fetch active watchlist: {}. Using cached list.",
                                e
                            );
                        }
                    }
                }

                if cached_symbols.is_empty() {
                    info!("[Jupiter] Watchlist empty, waiting 30s...");
                    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                    continue;
                }

                // 2. Batch Fetch Prices (Jupiter supports up to 100 IDs per request)
                let chunk_size = 100;
                for chunk in cached_symbols.chunks(chunk_size) {
                    // Capture timestamp BEFORE the HTTP request for latency measurement
                    let batch_timestamp = Utc::now().timestamp_millis();

                    let chunk_owned: Vec<String> = chunk.to_vec();
                    let client_ref = &client;
                    let fetch_result: crate::error::Result<HashMap<String, JupiterPriceItem>> =
                        retry_with_backoff(
                            || {
                                let ids = chunk_owned.clone();
                                async move {
                                    client_ref.get_prices(&ids).await.map_err(|e| {
                                        DataError::ExchangeError(format!(
                                            "[Jupiter] Fetch failed: {}",
                                            e
                                        ))
                                    })
                                }
                            },
                            3,
                            500,
                        )
                        .await;

                    match fetch_result {
                        Ok(prices) => {
                            let batch_count = prices.len() as u64;
                            for (id, item) in prices {
                                let price_f64 = item.price;
                                // Enrich with Metadata from Redis
                                let mut liquidity = None;
                                let mut volume_24h = None;
                                let mut fdv = None;

                                if let Some(r) = &mut redis {
                                    if let Ok(Some(meta)) = r.get_token_metadata(&id).await {
                                        liquidity = Some(
                                            Decimal::from_f64(meta.liquidity).unwrap_or_default(),
                                        );
                                        volume_24h = Some(
                                            Decimal::from_f64(meta.volume_24h).unwrap_or_default(),
                                        );
                                        fdv = Some(Decimal::from_f64(meta.fdv).unwrap_or_default());
                                    }
                                }

                                let data = crate::models::StandardMarketData {
                                    source: crate::models::DataSourceType::Jupiter,
                                    exchange: "Jupiter".to_string(),
                                    symbol: id.clone(),
                                    asset_type: crate::models::AssetType::Spot,
                                    data_type: crate::models::MarketDataType::Ticker,
                                    price: Decimal::from_f64(price_f64).unwrap_or_default(),
                                    quantity: Decimal::ZERO,
                                    timestamp: batch_timestamp,
                                    received_at: Utc::now().timestamp_millis(),
                                    bid: None,
                                    ask: None,
                                    high_24h: None,
                                    low_24h: None,
                                    volume_24h,
                                    open_interest: None,
                                    funding_rate: None,
                                    liquidity,
                                    fdv,
                                    sequence_id: None,
                                    raw_data: String::new(),
                                };

                                if tx.send(data).await.is_err() {
                                    error!("[Jupiter] Receiver dropped, exiting...");
                                    return;
                                }
                            }
                            // Track stats for the batch
                            {
                                let mut s = stats.write().await;
                                s.messages_received += batch_count;
                                s.messages_processed += batch_count;
                                s.last_message_at = Some(std::time::SystemTime::now());
                            }
                        }
                        Err(e) => {
                            error!(
                                "[Jupiter] Fetch failed after retries: {}. Skipping batch.",
                                e
                            );
                            let mut s = stats.write().await;
                            s.errors += 1;
                        }
                    }
                    // Small delay between chunks to be nice
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }

                // 3. Sleep
                // info!("[Jupiter] Cycle complete. Sleeping {}s...", poll_interval);
                tokio::time::sleep(tokio::time::Duration::from_secs(poll_interval)).await;
            }
        });

        Ok(rx)
    }

    pub async fn disconnect(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }
}
