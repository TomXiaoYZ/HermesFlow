use std::error::Error;
use crate::collectors::helius::config::HeliusConfig;
use tracing::info;

#[derive(Clone)]
pub struct HeliusClient {
    pub config: HeliusConfig,
    client: reqwest::Client,
}

impl HeliusClient {
    pub fn new(config: HeliusConfig) -> Self {
        info!("Initializing HeliusClient...");
        let client = reqwest::Client::new();
        Self { config, client }
    }
    
    // Placeholder for future REST API calls (e.g. historical data or enhanced transactions)
}
