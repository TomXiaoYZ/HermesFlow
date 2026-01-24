use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct FutuConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub pwd_md5: String,
    #[serde(default)]
    pub market_hksa: bool, // Hong Kong / A-Shares
    #[serde(default)]
    pub market_us: bool, // US Stocks
    #[serde(default)]
    pub symbols: Vec<String>,
}

impl Default for FutuConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: "127.0.0.1".to_string(),
            port: 11111,
            pwd_md5: String::new(),
            market_hksa: true,
            market_us: true,
            symbols: vec![],
        }
    }
}
