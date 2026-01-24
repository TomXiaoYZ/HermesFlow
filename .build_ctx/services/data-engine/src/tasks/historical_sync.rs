use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};
use chrono::{Utc, TimeZone};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use crate::collectors::birdeye::client::BirdeyeClient;
use crate::collectors::birdeye::config::BirdeyeConfig;
use crate::repository::TokenRepository;
use crate::repository::MarketDataRepository;
use crate::repository::postgres::PostgresRepositories;
use crate::models::Candle;

pub struct HistoricalSyncTask {
    birdeye_client: BirdeyeClient,
    repos: Arc<PostgresRepositories>,
    token_repo: Arc<dyn TokenRepository>,
}

impl HistoricalSyncTask {
    pub fn new(config: BirdeyeConfig, repos: Arc<PostgresRepositories>) -> Self {
        let client = BirdeyeClient::new(config);
        let token_repo = repos.token.clone();
        
        Self {
            birdeye_client: client,
            repos,
            token_repo,
        }
    }
    
    pub async fn run(&self) {
        // Fetch active symbols from database instead of config
        let symbols = match self.token_repo.get_active_addresses().await {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to fetch active symbols for historical sync: {}", e);
                return;
            }
        };
        
        info!("Starting Historical Data Backfill for {} symbols...", symbols.len());
        
        if symbols.is_empty() {
            info!("No active symbols to backfill");
            return;
        }
        
        let end_time = Utc::now().timestamp();
        let start_time = end_time - (30 * 24 * 60 * 60); // 30 days
        
        for symbol in &symbols {
            info!("Fetching history for symbol: {}", symbol);
            
            let mut attempts = 0;
            let max_attempts = 3;
            
            while attempts < max_attempts {
                match self.birdeye_client.get_history(symbol, start_time, end_time, "1h").await {
                    Ok(items) => {
                        let count = items.len();
                        info!("Fetched {} candles for {}. Inserting into DB...", count, symbol);
                        
                        for item in items {
                            let candle = Candle {
                                exchange: "birdeye".to_string(),
                                symbol: symbol.clone(),
                                resolution: "1h".to_string(),
                                open: Decimal::from_f64(item.open).unwrap_or(Decimal::ZERO),
                                high: Decimal::from_f64(item.high).unwrap_or(Decimal::ZERO),
                                low: Decimal::from_f64(item.low).unwrap_or(Decimal::ZERO),
                                close: Decimal::from_f64(item.close).unwrap_or(Decimal::ZERO),
                                volume: Decimal::from_f64(item.volume).unwrap_or(Decimal::ZERO),
                                amount: None,
                                liquidity: None,
                                fdv: None,
                                metadata: None,
                                time: Utc.timestamp_opt(item.unix_time, 0).unwrap(),
                            };
                            
                            if let Err(e) = self.repos.market_data.insert_candle(&candle).await {
                                warn!("Failed to insert candle for {}: {}", symbol, e);
                            }
                        }
                        info!("Successfully backfilled {} for last 30 days.", symbol);
                        break;
                    }
                    Err(e) => {
                        attempts += 1;
                        warn!("Failed to fetch history for {} (Attempt {}/{}): {}", symbol, attempts, max_attempts, e);
                        sleep(Duration::from_secs(2)).await;
                    }
                }
            }
            
            sleep(Duration::from_millis(500)).await;
        }
        
        info!("Historical Data Backfill Completed.");
    }
}
