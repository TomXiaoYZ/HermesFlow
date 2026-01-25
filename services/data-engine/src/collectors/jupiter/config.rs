use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct JupiterConfig {
    /// Whether this data source is enabled
    pub enabled: bool,
    /// Jupiter V2 Price API Base URL
    #[serde(default = "default_api_url")]
    pub api_url: String,
    /// Poll interval in seconds (default: 10)
    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: u64,
    /// Optional API Key for Jupiter V2 (x-api-key)
    pub api_key: Option<String>,
}

fn default_api_url() -> String {
    "https://api.jup.ag/price/v3".to_string()
}

fn default_poll_interval() -> u64 {
    10
}

impl Default for JupiterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_url: default_api_url(),
            poll_interval_secs: default_poll_interval(),
            api_key: None,
        }
    }
}
