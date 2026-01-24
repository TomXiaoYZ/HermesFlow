use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct HeliusConfig {
    pub enabled: bool,
    pub api_key: String,
    pub rpc_url: String, // e.g. https://mainnet.helius-rpc.com/?api-key=...
    pub ws_url: String, // e.g. wss://mainnet.helius-rpc.com/?api-key=...
}

impl Default for HeliusConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key: String::new(),
            rpc_url: "https://mainnet.helius-rpc.com".to_string(),
            ws_url: "wss://mainnet.helius-rpc.com".to_string(),
        }
    }
}
