pub mod collectors;
pub mod config;
pub mod error;
pub mod health;
pub mod models;
pub mod monitoring;
pub mod registry;
pub mod repository;
pub mod server;
pub mod storage;
pub mod traits;
pub mod trading;
pub mod utils;

// Re-export commonly used types
pub use collectors::{PolymarketCollector, TwitterCollector, IBKRCollector};
pub use config::AppConfig;
pub use error::{DataEngineError, DataError, Result};
pub use models::{
    AssetType, DataSourceType, MarketDataType, MarketOutcome, PredictionMarket, SocialData,
    StandardMarketData,
};
pub use monitoring::{HealthMonitor, HealthStatus};
pub use registry::ParserRegistry;
pub use repository::{
    postgres::PostgresRepositories,
    MarketDataRepository, SocialRepository, TradingRepository, PredictionRepository,
};
pub use server::{create_router, AppState};
pub use storage::{ClickHouseWriter, RedisCache};
pub use traits::{ConnectorStats, DataSourceConnector, MessageParser};
