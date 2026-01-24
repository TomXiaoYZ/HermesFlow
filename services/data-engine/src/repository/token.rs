use crate::error::DataEngineError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ActiveToken {
    pub address: String,
    pub symbol: String,
    pub name: Option<String>,
    pub decimals: i32,
    pub chain: String,
    pub liquidity_usd: Option<Decimal>,
    pub fdv: Option<Decimal>,
    pub market_cap: Option<Decimal>,
    pub volume_24h: Option<Decimal>,
    pub price_change_24h: Option<Decimal>,
    pub first_discovered: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub is_active: bool,
    pub metadata: Option<Value>,
}

#[async_trait]
pub trait TokenRepository: Send + Sync {
    /// Get all active token addresses
    async fn get_active_addresses(&self) -> Result<Vec<String>, DataEngineError>;

    /// Get all active tokens with full details
    async fn get_active_tokens(&self) -> Result<Vec<ActiveToken>, DataEngineError>;

    /// Upsert a single token
    async fn upsert_token(&self, token: &ActiveToken) -> Result<(), DataEngineError>;

    /// Upsert multiple tokens in a batch
    async fn upsert_tokens(&self, tokens: Vec<ActiveToken>) -> Result<(), DataEngineError>;

    /// Mark tokens as inactive if not updated within duration
    async fn deactivate_stale(&self, hours: i64) -> Result<usize, DataEngineError>;

    /// Get token by address
    async fn get_token(&self, address: &str) -> Result<Option<ActiveToken>, DataEngineError>;
}
