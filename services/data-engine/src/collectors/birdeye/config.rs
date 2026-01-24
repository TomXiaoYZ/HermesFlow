use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct BirdeyeConfig {
    pub enabled: bool,
    pub api_key: String,
    pub base_url: String,
    pub chain: String,
    pub poll_interval_secs: u64,
    // Symbols are now dynamically loaded from database (active_tokens table)
}

impl Default for BirdeyeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key: String::new(),
            base_url: "https://public-api.birdeye.so".to_string(),
            chain: "solana".to_string(),
            poll_interval_secs: 10,
        }
    }
}
