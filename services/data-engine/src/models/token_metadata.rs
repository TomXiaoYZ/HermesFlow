use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetadata {
    pub symbol: String,
    pub liquidity: f64,
    pub volume_24h: f64,
    pub fdv: f64,
    pub last_updated_at: i64,
}

impl TokenMetadata {
    pub fn new(
        symbol: String,
        liquidity: f64,
        volume_24h: f64,
        fdv: f64,
        last_updated_at: i64,
    ) -> Self {
        Self {
            symbol,
            liquidity,
            volume_24h,
            fdv,
            last_updated_at,
        }
    }
}
