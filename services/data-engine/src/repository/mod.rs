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
    /// store a batch of candles
    async fn insert_candles(&self, data: &[Candle]) -> Result<(), DataEngineError>;
    /// fetch distinct symbols that have data (or are configured)
    async fn get_active_symbols(&self) -> Result<Vec<String>, DataEngineError>;
    /// fetch the timestamp of the latest candle for a symbol/resolution
    async fn get_latest_candle_time(
        &self,
        exchange: &str,
        symbol: &str,
        resolution: &str,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, DataEngineError>;
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
    async fn list_markets(
        &self,
        active_only: bool,
        category: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PredictionMarket>, DataEngineError>;
    async fn get_market(&self, market_id: &str)
        -> Result<Option<PredictionMarket>, DataEngineError>;
    async fn get_outcome_history(
        &self,
        market_id: &str,
        limit: i64,
    ) -> Result<Vec<MarketOutcome>, DataEngineError>;
}
#[async_trait]
pub trait MetricsRepository: Send + Sync {
    async fn insert_api_usage(&self, provider: &str, count: i64) -> Result<(), DataEngineError>;
}

pub mod token;
pub use token::{ActiveToken, TokenRepository};
