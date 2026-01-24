use crate::error::DataEngineError;
use crate::models::{
    Candle, MarketOutcome, Order, PredictionMarket, SocialData, StandardMarketData, Trade,
};
use async_trait::async_trait;
use uuid::Uuid;

pub mod postgres;

#[async_trait]
pub trait MarketDataRepository: Send + Sync {
    /// store a real-time snapshot (ticker)
    async fn insert_snapshot(&self, data: &StandardMarketData) -> Result<(), DataEngineError>;
    /// store a historical/aggregated candle
    async fn insert_candle(&self, data: &Candle) -> Result<(), DataEngineError>;
    /// fetch distinct symbols that have data (or are configured)
    async fn get_active_symbols(&self) -> Result<Vec<String>, DataEngineError>;
}

#[async_trait]
pub trait SocialRepository: Send + Sync {
    async fn insert_tweet(&self, data: &SocialData) -> Result<(), DataEngineError>;
    async fn insert_collection_run(
        &self,
        target: &str,
        scraped: i32,
        upserted: i32,
        error: Option<&str>,
    ) -> Result<(), DataEngineError>;
}

#[async_trait]
pub trait TradingRepository: Send + Sync {
    async fn insert_order(&self, order: &Order) -> Result<Uuid, DataEngineError>;
    async fn insert_trade(&self, trade: &Trade) -> Result<Uuid, DataEngineError>;
}

#[async_trait]
pub trait PredictionRepository: Send + Sync {
    async fn upsert_market(&self, market: &PredictionMarket) -> Result<(), DataEngineError>;
    async fn insert_outcome(
        &self,
        market_id: &str,
        outcome: &MarketOutcome,
    ) -> Result<(), DataEngineError>;
}
pub mod token;
pub use token::{ActiveToken, TokenRepository};
