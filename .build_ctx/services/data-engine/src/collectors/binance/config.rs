use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct BinanceConfig {
    pub enabled: bool,
    pub api_key: String,
    pub secret_key: String,
    #[serde(default = "default_binance_http_url")]
    pub base_url: String,
    #[serde(default = "default_binance_ws_url")]
    pub ws_url: String,
    #[serde(default)]
    pub symbols: Vec<String>,
}

fn default_binance_http_url() -> String {
    "https://api.binance.com".to_string()
}

fn default_binance_ws_url() -> String {
    "wss://stream.binance.com:9443/ws".to_string()
}
