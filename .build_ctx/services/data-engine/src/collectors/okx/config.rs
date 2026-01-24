use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct OkxConfig {
    pub enabled: bool,
    pub api_key: String,
    pub secret_key: String,
    pub passphrase: String, // OKX specific
    #[serde(default = "default_okx_http_url")]
    pub base_url: String,
    #[serde(default = "default_okx_ws_url")]
    pub ws_url: String,
    #[serde(default)]
    pub symbols: Vec<String>,
}

fn default_okx_http_url() -> String {
    "https://www.okx.com".to_string()
}

fn default_okx_ws_url() -> String {
    "wss://ws.okx.com:8443/ws/v5/public".to_string()
}
