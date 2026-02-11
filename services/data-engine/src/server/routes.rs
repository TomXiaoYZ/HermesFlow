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
use crate::tasks::TaskManager;
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
    pub task_manager: Option<Arc<TaskManager>>,
    pub broadcast_tx: tokio::sync::broadcast::Sender<String>,
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
        task_manager: Option<TaskManager>,
        broadcast_tx: tokio::sync::broadcast::Sender<String>,
    ) -> Self {
        Self {
            config: Arc::new(config),
            redis,
            clickhouse,
            postgres,
            health_monitor: Arc::new(health_monitor),
            ibkr_trader: ibkr_trader.map(Arc::new),
            task_manager: task_manager.map(Arc::new),
            broadcast_tx,
            start_time: std::time::Instant::now(),
        }
    }
}

/// Creates the Axum router with all routes configured
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(handlers::health_check))
        .route("/metrics", get(handlers::metrics))
        .route("/ws", get(handlers::ws_handler))
        .route("/api/v1/data/market/:symbol/latest", get(handlers::get_latest_price))
        .route("/api/v1/data/market/tokens", get(handlers::get_active_tokens))
        .route("/api/v1/data/market/:symbol/history", get(handlers::get_history))
        .route(
            "/api/v1/jobs/backfill",
            post(handlers::jobs::trigger_backfill_job),
        )
        .route(
            "/api/v1/agent/monitoring/start",
            post(handlers::agent::start_agent_monitoring),
        )
        .route("/api/v1/orders", post(handlers::trading::place_order))
        .route(
            "/api/v1/orders/:id",
            delete(handlers::trading::cancel_order),
        )
        .route("/api/v1/positions", get(handlers::trading::get_positions))
        .route(
            "/api/v1/account",
            get(handlers::trading::get_account_summary),
        )
        // History & Status APIs
        .route("/api/v1/history/logs", get(handlers::history::get_logs))
        .route(
            "/api/v1/strategy/status",
            get(handlers::history::get_strategy_status),
        )
        .route(
            "/api/v1/strategy/population",
            get(handlers::get_strategy_population),
        )
        // Data Discovery APIs
        .route("/api/v1/data/quality", get(handlers::data::get_data_quality))
        .route("/api/v1/data/tables", get(handlers::data::get_tables))
        .route("/api/v1/data/query", post(handlers::data::query_data))
        .route("/api/v1/data/tasks/discovery", post(handlers::data::trigger_token_discovery))
        .route("/api/v1/data/tasks/aggregation", post(handlers::data::trigger_aggregation))
        // Config APIs
        .route("/api/v1/config/exchanges", get(handlers::config::get_exchange_config).post(handlers::config::update_exchange_config))
        .route("/api/v1/watchlist", get(handlers::config::get_watchlist).post(handlers::config::add_to_watchlist).delete(handlers::config::remove_from_watchlist))
        // Prediction Market APIs
        .route("/api/v1/prediction/markets", get(handlers::prediction::list_prediction_markets))
        .route("/api/v1/prediction/markets/:id", get(handlers::prediction::get_prediction_market))
        .route("/api/v1/prediction/markets/:id/history", get(handlers::prediction::get_prediction_market_history))
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
        let (tx, _) = tokio::sync::broadcast::channel(100);
        AppState::new(
            config,
            None,
            None,
            Arc::new(postgres),
            health_monitor,
            None,
            None,
            tx,
        )
    }

    #[tokio::test]
    async fn test_router_creation() {
        // This test just verifies the router can be created
        // Actual route testing would be done in integration tests
        // with a running server
    }
}
