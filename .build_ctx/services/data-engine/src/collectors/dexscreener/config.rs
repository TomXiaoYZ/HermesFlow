use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct DexScreenerConfig {
    pub enabled: bool,
    pub base_url: String,
}

impl Default for DexScreenerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_url: "https://api.dexscreener.com/latest/dex".to_string(),
        }
    }
}
