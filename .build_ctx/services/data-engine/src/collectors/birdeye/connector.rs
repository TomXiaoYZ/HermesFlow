use std::error::Error;
use std::sync::Arc;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use chrono::{TimeZone, Utc};
use crate::models::Candle;
use crate::collectors::birdeye::client::BirdeyeClient;
use crate::collectors::birdeye::config::BirdeyeConfig;
use crate::repository::TokenRepository;
use tracing::{info, warn, error};

pub struct BirdeyeConnector {
    client: BirdeyeClient,
}

impl BirdeyeConnector {
    pub fn new(config: BirdeyeConfig) -> Self {
        let client = BirdeyeClient::new(config);
        Self { client }
    }

    pub async fn fetch_history_candles(
        &self,
        address: &str,
        resolution: &str,
        from_ts: i64,
        to_ts: i64,
    ) -> Result<Vec<Candle>, Box<dyn Error + Send + Sync>> {
        let items = self.client.get_history(address, from_ts, to_ts, resolution).await?;
        
        let mut candles = Vec::new();
        for item in items {
            let open = Decimal::from_f64(item.open).unwrap_or_default();
            let high = Decimal::from_f64(item.high).unwrap_or_default();
            let low = Decimal::from_f64(item.low).unwrap_or_default();
            let close = Decimal::from_f64(item.close).unwrap_or_default();
            let volume = Decimal::from_f64(item.volume).unwrap_or_default();
            
            let time = Utc.timestamp_opt(item.unix_time, 0).unwrap();

            let candle = Candle::new(
                "Birdeye".to_string(),
                address.to_string(),
                resolution.to_string(),
                open,
                high,
                low,
                close,
                volume,
                time,
            );
            
            candles.push(candle);
        }

        Ok(candles)
    }

    /// Dynamic connector that queries database for active symbols
    /// Refreshes symbol list every 5 minutes from active_tokens table
    pub async fn connect(
        &self,
        token_repo: Arc<dyn TokenRepository>,
    ) -> Result<tokio::sync::mpsc::Receiver<crate::models::StandardMarketData>, Box<dyn Error + Send + Sync>> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let client = self.client.clone();
        
        tokio::spawn(async move {
            let mut cached_symbols: Vec<String> = Vec::new();
            let mut last_refresh = std::time::Instant::now();
            let refresh_interval = std::time::Duration::from_secs(300); // 5 minutes
            
            loop {
                // Refresh symbol list from database every 5 minutes
                if cached_symbols.is_empty() || last_refresh.elapsed() >= refresh_interval {
                    match token_repo.get_active_addresses().await {
                        Ok(symbols) => {
                            if symbols.len() != cached_symbols.len() {
                                info!("🔄 Refreshed active symbols from DB: {} tokens", symbols.len());
                            }
                            cached_symbols = symbols;
                            last_refresh = std::time::Instant::now();
                        }
                        Err(e) => {
                            warn!("Failed to fetch active symbols from DB: {}. Using cached list.", e);
                        }
                    }
                }
                
                if cached_symbols.is_empty() {
                    info!("No active symbols in database, waiting 30s before retry...");
                    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                    continue;
                }
                
                // Poll each active symbol
                for symbol in &cached_symbols {
                    match client.get_token_overview(symbol).await {
                        Ok(overview) => {
                            let price = overview.price.unwrap_or(0.0);
                            let liquidity = overview.liquidity.and_then(|v| Decimal::from_f64(v));
                            let fdv = overview.market_cap.and_then(|v| Decimal::from_f64(v));
                            
                            let data = crate::models::StandardMarketData {
                                source: crate::models::DataSourceType::Birdeye,
                                exchange: "Birdeye".to_string(),
                                symbol: symbol.clone(),
                                asset_type: crate::models::AssetType::Spot,
                                data_type: crate::models::MarketDataType::Ticker,
                                price: Decimal::from_f64(price).unwrap_or_default(),
                                quantity: Decimal::ZERO,
                                timestamp: Utc::now().timestamp_millis(),
                                received_at: Utc::now().timestamp_millis(),
                                bid: None,
                                ask: None,
                                high_24h: None,
                                low_24h: None,
                                volume_24h: overview.volume_24h.and_then(|v| Decimal::from_f64(v)),
                                open_interest: None,
                                funding_rate: None,
                                liquidity,
                                fdv,
                                sequence_id: None,
                                raw_data: String::new(),
                            };
                            
                            if let Err(_) = tx.send(data).await {
                                error!("Birdeye connector: receiver dropped, exiting...");
                                return;
                            }
                        }
                        Err(e) => {
                            if e.to_string().contains("429") {
                                warn!("Birdeye rate limit hit for {}, backing off...", symbol);
                                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                            }
                        }
                    }
                }
                
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            }
        });
        
        Ok(rx)
    }

    pub async fn disconnect(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }

    pub async fn fetch_token_overview(&self, address: &str) -> Result<crate::collectors::birdeye::client::TokenOverview, Box<dyn Error + Send + Sync>> {
        self.client.get_token_overview(address).await
    }
}
