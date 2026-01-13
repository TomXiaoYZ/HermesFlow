use axum::{
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::AppConfig;
use crate::monitoring::HealthMonitor;
use crate::repository::postgres::PostgresRepositories;
use crate::storage::{ClickHouseWriter, RedisCache};
use crate::trading::ibkr_trader::IBKRTrader;

use super::handlers;

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub redis: Option<Arc<RwLock<RedisCache>>>,
    pub clickhouse: Option<Arc<RwLock<ClickHouseWriter>>>,
    pub postgres: Arc<PostgresRepositories>,
    pub health_monitor: Arc<HealthMonitor>,
    pub ibkr_trader: Option<Arc<IBKRTrader>>,
    pub start_time: std::time::Instant,
}

impl AppState {
    pub fn new(
        config: AppConfig,
        redis: Option<Arc<RwLock<RedisCache>>>,
        clickhouse: Option<Arc<RwLock<ClickHouseWriter>>>,
        postgres: Arc<PostgresRepositories>,
        health_monitor: HealthMonitor,
        ibkr_trader: Option<IBKRTrader>,
    ) -> Self {
        Self {
            config: Arc::new(config),
            redis,
            clickhouse,
            postgres,
            health_monitor: Arc::new(health_monitor),
            ibkr_trader: ibkr_trader.map(Arc::new),
            start_time: std::time::Instant::now(),
        }
    }
}

/// Creates the Axum router with all routes configured
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(handlers::health_check))
        .route("/metrics", get(handlers::metrics))
        .route(
            "/api/v1/market/:symbol/latest",
            get(handlers::get_latest_price),
        )
        .route("/api/v1/market/:symbol/history", get(handlers::get_history))
        .route("/api/v1/orders", post(handlers::trading::place_order))
        .route(
            "/api/v1/orders/:id",
            delete(handlers::trading::cancel_order),
        )
        .route("/api/v1/positions", get(handlers::trading::get_positions))
        .route("/api/v1/account", get(handlers::trading::get_account_summary))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;
    use crate::monitoring::HealthMonitor;

    #[allow(dead_code)]
    async fn create_test_state() -> AppState {
        let config = AppConfig {
            server: ServerConfig::default(),
            redis: RedisConfig::default(),
            postgres: PostgresConfig::default(),
            clickhouse: ClickHouseConfig::default(),
            data_sources: vec![],
            twitter: None,
            polymarket: None,
            performance: PerformanceConfig::default(),
            logging: LoggingConfig::default(),
        };

        // For tests, we can skip Redis/ClickHouse connections
        AppState::new(config, None, None, Arc::new(postgres), health_monitor, None)
    }

    #[tokio::test]
    async fn test_router_creation() {
        // This test just verifies the router can be created
        // Actual route testing would be done in integration tests
        // with a running server
    }
}
