use crate::collectors::jupiter::client::JupiterClient;
use crate::collectors::jupiter::config::JupiterConfig;
use crate::repository::TokenRepository;
use chrono::Utc;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use std::error::Error;
use std::sync::Arc;
use tracing::{error, info, warn};

pub struct JupiterPriceCollector {
    client: JupiterClient,
    config: JupiterConfig,
}

impl JupiterPriceCollector {
    pub fn new(config: JupiterConfig) -> Self {
        let client = JupiterClient::new(config.clone());
        Self { client, config }
    }

    pub async fn connect(
        &self,
        token_repo: Arc<dyn TokenRepository>,
    ) -> Result<
        tokio::sync::mpsc::Receiver<crate::models::StandardMarketData>,
        Box<dyn Error + Send + Sync>,
    > {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let client = self.client.clone();
        let poll_interval = self.config.poll_interval_secs;

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
                    match client.get_prices(chunk).await {
                        Ok(prices) => {
                           for (id, item) in prices {
                               // in V3, price is already f64
                               let price_f64 = item.price;
                               let data = crate::models::StandardMarketData {
                                   source: crate::models::DataSourceType::Jupiter,
                                   exchange: "Jupiter".to_string(),
                                   symbol: id.clone(),
                                   asset_type: crate::models::AssetType::Spot,
                                   data_type: crate::models::MarketDataType::Ticker,
                                   price: Decimal::from_f64(price_f64).unwrap_or_default(),
                                       quantity: Decimal::ZERO,
                                       timestamp: Utc::now().timestamp_millis(),
                                       received_at: Utc::now().timestamp_millis(),
                                       bid: None,
                                       ask: None,
                                       high_24h: None,
                                       low_24h: None,
                                       volume_24h: None, // Jupiter Price API V2 doesn't give volume, only price
                                       open_interest: None,
                                       funding_rate: None,
                                       liquidity: None,
                                       fdv: None,
                                       sequence_id: None,
                                       raw_data: String::new(),
                                   };

                                   if let Err(_) = tx.send(data).await {
                                       error!("[Jupiter] Receiver dropped, exiting...");
                                       return;
                                   }
                               }
                        }
                        Err(e) => {
                            error!("[Jupiter] Fetch failed: {}", e);
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
