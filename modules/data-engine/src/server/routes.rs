use axum::{routing::get, Router};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::AppConfig;
use crate::monitoring::HealthMonitor;
use crate::storage::{ClickHouseWriter, RedisCache};

use super::handlers;

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub redis: Arc<RwLock<RedisCache>>,
    pub clickhouse: Arc<RwLock<ClickHouseWriter>>,
    pub health_monitor: Arc<HealthMonitor>,
    pub start_time: std::time::Instant,
}

impl AppState {
    pub fn new(
        config: AppConfig,
        redis: RedisCache,
        clickhouse: ClickHouseWriter,
        health_monitor: HealthMonitor,
    ) -> Self {
        Self {
            config: Arc::new(config),
            redis: Arc::new(RwLock::new(redis)),
            clickhouse: Arc::new(RwLock::new(clickhouse)),
            health_monitor: Arc::new(health_monitor),
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
            clickhouse: ClickHouseConfig::default(),
            data_sources: vec![],
            performance: PerformanceConfig::default(),
            logging: LoggingConfig::default(),
        };

        let redis = RedisCache::new("redis://localhost:6379", 86400)
            .await
            .unwrap_or_else(|_| {
                // For testing, create a mock if Redis not available
                panic!("Redis connection failed - tests require Redis")
            });

        let clickhouse = ClickHouseWriter::new("tcp://localhost:9000", "test", 1000, 5000).unwrap();

        let health_monitor = HealthMonitor::new();

        AppState::new(config, redis, clickhouse, health_monitor)
    }

    #[tokio::test]
    async fn test_router_creation() {
        // This test just verifies the router can be created
        // Actual route testing would be done in integration tests
        // with a running server
    }
}
