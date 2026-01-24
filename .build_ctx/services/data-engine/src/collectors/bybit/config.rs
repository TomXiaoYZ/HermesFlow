use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct BybitConfig {
    pub enabled: bool,
    pub api_key: String,
    pub secret_key: String,
    #[serde(default = "default_bybit_http_url")]
    pub base_url: String,
    #[serde(default = "default_bybit_ws_url")]
    pub ws_url: String,
    #[serde(default)]
    pub symbols: Vec<String>,
}

fn default_bybit_http_url() -> String {
    "https://api.bybit.com".to_string()
}

fn default_bybit_ws_url() -> String {
    "wss://stream.bybit.com/v5/public/linear".to_string()
}
