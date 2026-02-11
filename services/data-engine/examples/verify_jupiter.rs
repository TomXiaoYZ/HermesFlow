use data_engine::collectors::{JupiterConfig, JupiterPriceCollector};
use data_engine::error::DataEngineError;
use data_engine::repository::{ActiveToken, TokenRepository};
use std::sync::Arc;

// Mock Token Repository
struct MockRepo;

#[async_trait::async_trait]
impl TokenRepository for MockRepo {
    async fn get_active_addresses(&self) -> Result<Vec<String>, DataEngineError> {
        // Return some known Solana tokens: SOL, USDC, RAY
        Ok(vec![
            "So11111111111111111111111111111111111111112".to_string(), // SOL
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
            "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R".to_string(), // RAY
        ])
    }

    async fn get_active_tokens(&self) -> Result<Vec<ActiveToken>, DataEngineError> {
        Ok(vec![])
    }

    async fn get_token(&self, _address: &str) -> Result<Option<ActiveToken>, DataEngineError> {
        Ok(None)
    }

    async fn upsert_token(&self, _token: &ActiveToken) -> Result<(), DataEngineError> {
        Ok(())
    }

    async fn upsert_tokens(&self, _tokens: Vec<ActiveToken>) -> Result<(), DataEngineError> {
        Ok(())
    }

    async fn deactivate_stale(&self, _threshold_seconds: i64) -> Result<usize, DataEngineError> {
        Ok(0)
    }
}

#[tokio::main]
async fn main() {
    // Setup logging
    tracing_subscriber::fmt::init();

    tracing::info!("Starting Jupiter Verification Test...");

    // Configure Jupiter Collector
    let config = JupiterConfig {
        enabled: true,
        api_url: "https://api.jup.ag/price/v3".to_string(),
        poll_interval_secs: 2, // Fast polling for test
        api_key: std::env::var("DATA_ENGINE__JUPITER__API_KEY").ok(),
    };

    let collector = JupiterPriceCollector::new(config, None);
    let repo = Arc::new(MockRepo);

    tracing::info!("Connecting to Jupiter...");
    match collector.connect(repo).await {
        Ok(mut rx) => {
            tracing::info!("✅ Connected! Waiting for price updates...");

            // Listen for a few updates
            let mut count = 0;
            while let Some(msg) = rx.recv().await {
                tracing::info!(
                    "[{}] Received Price: {} = ${} (Source: {:?})",
                    count,
                    msg.symbol,
                    msg.price,
                    msg.source
                );

                count += 1;
                if count >= 3 {
                    tracing::info!("✅ Verification Successful! Received 3 updates.");
                    println!("VERIFICATION_SUCCESS");
                    std::fs::write("verified.txt", "SUCCESS").unwrap();
                    break;
                }
            }
        }
        Err(e) => {
            tracing::error!("❌ Failed to connect: {}", e);
            std::process::exit(1);
        }
    }
}
