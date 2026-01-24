pub mod market_data;
pub mod migration;
pub mod prediction;
pub mod social;
pub mod token;
pub mod trading;

pub use market_data::PostgresMarketDataRepository;
pub use migration::MigrationManager;
pub use prediction::PostgresPredictionRepository;
pub use social::PostgresSocialRepository;
pub use token::PostgresTokenRepository;
pub use trading::PostgresTradingRepository;

use crate::config::PostgresConfig;
use crate::error::DataEngineError;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

pub struct PostgresRepositories {
    pub pool: PgPool,
    pub market_data: Arc<PostgresMarketDataRepository>,
    pub social: Arc<PostgresSocialRepository>,
    pub trading: Arc<PostgresTradingRepository>,
    pub prediction: Arc<PostgresPredictionRepository>,
    pub migration: Arc<MigrationManager>,
    pub token: Arc<PostgresTokenRepository>,
}

impl PostgresRepositories {
    pub async fn new(config: &PostgresConfig) -> Result<Self, DataEngineError> {
        let connection_string = format!(
            "postgres://{}:{}@{}:{}/{}",
            config.username, config.password, config.host, config.port, config.database
        );

        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .acquire_timeout(Duration::from_secs(30))
            .connect(&connection_string)
            .await
            .map_err(|e| {
                error!("Failed to connect to Postgres: {}", e);
                DataEngineError::DatabaseError(format!("Postgres connection failed: {}", e))
            })?;

        info!(
            "Connected to Postgres at {}:{}/{}",
            config.host, config.port, config.database
        );

        Ok(Self {
            market_data: Arc::new(PostgresMarketDataRepository::new(pool.clone())),
            social: Arc::new(PostgresSocialRepository::new(pool.clone())),
            trading: Arc::new(PostgresTradingRepository::new(pool.clone())),
            prediction: Arc::new(PostgresPredictionRepository::new(pool.clone())),
            migration: Arc::new(MigrationManager::new(pool.clone())),
            token: Arc::new(PostgresTokenRepository::new(pool.clone())),
            pool,
        })
    }
}
