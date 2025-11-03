use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::storage::RedisCache;

/// Overall health status of the service
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    /// All systems operational
    Healthy,
    /// Some non-critical systems degraded
    Degraded(&'static str),
    /// Critical systems down
    Unhealthy,
}

/// Status of a dependency
#[derive(Debug, Clone, serde::Serialize)]
pub struct DependencyStatus {
    pub status: String, // "up" | "down"
    pub latency_ms: Option<f64>,
    pub last_check: String, // ISO 8601 timestamp
    pub message: Option<String>,
}

impl DependencyStatus {
    pub fn up(latency_ms: f64) -> Self {
        Self {
            status: "up".to_string(),
            latency_ms: Some(latency_ms),
            last_check: chrono::Utc::now().to_rfc3339(),
            message: None,
        }
    }

    pub fn down(message: String) -> Self {
        Self {
            status: "down".to_string(),
            latency_ms: None,
            last_check: chrono::Utc::now().to_rfc3339(),
            message: Some(message),
        }
    }
}

/// Health monitor for tracking service health
pub struct HealthMonitor {
    last_message: Arc<RwLock<Option<Instant>>>,
    redis_status: Arc<RwLock<DependencyStatus>>,
    clickhouse_status: Arc<RwLock<DependencyStatus>>,
    start_time: Instant,
}

impl HealthMonitor {
    /// Creates a new health monitor
    pub fn new() -> Self {
        Self {
            last_message: Arc::new(RwLock::new(None)),
            redis_status: Arc::new(RwLock::new(DependencyStatus::down(
                "Not checked yet".to_string(),
            ))),
            clickhouse_status: Arc::new(RwLock::new(DependencyStatus::down(
                "Not checked yet".to_string(),
            ))),
            start_time: Instant::now(),
        }
    }

    /// Records that a message was received
    pub async fn record_message(&self) {
        let mut last = self.last_message.write().await;
        *last = Some(Instant::now());
    }

    /// Checks overall health status
    pub async fn check_health(&self) -> HealthStatus {
        let redis = self.redis_status.read().await;
        let clickhouse = self.clickhouse_status.read().await;
        let last_msg = self.last_message.read().await;

        let redis_ok = redis.status == "up";
        let clickhouse_ok = clickhouse.status == "up";

        // Check if we've received data recently (within 60 seconds)
        let recent_data = last_msg
            .as_ref()
            .map(|t| t.elapsed() < Duration::from_secs(60))
            .unwrap_or(false);

        match (redis_ok, clickhouse_ok, recent_data) {
            (true, true, true) => HealthStatus::Healthy,
            (true, true, false) => HealthStatus::Degraded("No recent data"),
            (false, true, _) => HealthStatus::Degraded("Redis unavailable"),
            (true, false, _) => HealthStatus::Unhealthy, // ClickHouse critical
            (false, false, _) => HealthStatus::Unhealthy,
        }
    }

    /// Checks Redis health
    pub async fn check_redis(&self, redis: &mut RedisCache) -> DependencyStatus {
        let start = Instant::now();

        match redis.check_health().await {
            Ok(true) => {
                let latency = start.elapsed().as_secs_f64() * 1000.0;
                let status = DependencyStatus::up(latency);
                *self.redis_status.write().await = status.clone();
                status
            }
            Ok(false) | Err(_) => {
                let status = DependencyStatus::down("Health check failed".to_string());
                *self.redis_status.write().await = status.clone();
                status
            }
        }
    }

    /// Checks ClickHouse health
    pub async fn check_clickhouse(&self) -> DependencyStatus {
        // In a real implementation, we'd ping ClickHouse
        // For now, return a mock status
        let status = DependencyStatus::up(5.0);
        *self.clickhouse_status.write().await = status.clone();
        status
    }

    /// Returns uptime in seconds
    pub fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Gets current Redis status
    pub async fn get_redis_status(&self) -> DependencyStatus {
        self.redis_status.read().await.clone()
    }

    /// Gets current ClickHouse status
    pub async fn get_clickhouse_status(&self) -> DependencyStatus {
        self.clickhouse_status.read().await.clone()
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_status_up() {
        let status = DependencyStatus::up(5.5);
        assert_eq!(status.status, "up");
        assert_eq!(status.latency_ms, Some(5.5));
        assert!(status.message.is_none());
    }

    #[test]
    fn test_dependency_status_down() {
        let status = DependencyStatus::down("Connection failed".to_string());
        assert_eq!(status.status, "down");
        assert!(status.latency_ms.is_none());
        assert_eq!(status.message, Some("Connection failed".to_string()));
    }

    #[tokio::test]
    async fn test_health_monitor_creation() {
        let monitor = HealthMonitor::new();
        assert!(monitor.uptime_secs() < 1);
    }

    #[tokio::test]
    async fn test_health_monitor_record_message() {
        let monitor = HealthMonitor::new();

        monitor.record_message().await;

        let last_msg = monitor.last_message.read().await;
        assert!(last_msg.is_some());
    }

    #[tokio::test]
    async fn test_health_status_unhealthy_by_default() {
        let monitor = HealthMonitor::new();
        let status = monitor.check_health().await;

        // Should be unhealthy initially since dependencies not checked
        assert_eq!(status, HealthStatus::Unhealthy);
    }
}
