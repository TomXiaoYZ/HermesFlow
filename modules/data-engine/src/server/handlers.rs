use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::monitoring::metrics::export_metrics;
use crate::monitoring::{DependencyStatus, HealthStatus};

use super::routes::AppState;

/// Health check response
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_secs: u64,
    pub dependencies: HealthDeps,
}

#[derive(Serialize)]
pub struct HealthDeps {
    pub redis: DependencyStatus,
    pub clickhouse: DependencyStatus,
}

/// Latest price response
#[derive(Serialize)]
pub struct LatestPriceResponse {
    pub symbol: String,
    pub price: String,
    pub timestamp: i64,
    pub source: String,
    pub bid: Option<String>,
    pub ask: Option<String>,
}

/// History query parameters
#[derive(Deserialize)]
pub struct HistoryQuery {
    pub start: Option<i64>,
    pub end: Option<i64>,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    1000
}

/// History response
#[derive(Serialize)]
pub struct HistoryResponse {
    pub symbol: String,
    pub data: Vec<MarketDataPoint>,
    pub count: usize,
}

#[derive(Serialize)]
pub struct MarketDataPoint {
    pub timestamp: i64,
    pub price: String,
    pub quantity: String,
    pub source: String,
}

/// Health check endpoint
/// GET /health
pub async fn health_check(State(state): State<AppState>) -> Response {
    let health_status = state.health_monitor.check_health().await;
    let uptime = state.health_monitor.uptime_secs();

    let redis_status = state.health_monitor.get_redis_status().await;
    let clickhouse_status = state.health_monitor.get_clickhouse_status().await;

    let response = HealthResponse {
        status: match health_status {
            HealthStatus::Healthy => "healthy".to_string(),
            HealthStatus::Degraded(msg) => format!("degraded: {}", msg),
            HealthStatus::Unhealthy => "unhealthy".to_string(),
        },
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: uptime,
        dependencies: HealthDeps {
            redis: redis_status,
            clickhouse: clickhouse_status,
        },
    };

    let status_code = match health_status {
        HealthStatus::Healthy => StatusCode::OK,
        HealthStatus::Degraded(_) => StatusCode::OK,
        HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };

    (status_code, Json(response)).into_response()
}

/// Prometheus metrics endpoint
/// GET /metrics
pub async fn metrics() -> Response {
    match export_metrics() {
        Ok(metrics_text) => (StatusCode::OK, metrics_text).into_response(),
        Err(e) => {
            tracing::error!("Failed to export metrics: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to export metrics: {}", e),
            )
                .into_response()
        }
    }
}

/// Get latest price for a symbol
/// GET /api/v1/market/:symbol/latest
pub async fn get_latest_price(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
) -> Response {
    let mut redis = state.redis.write().await;

    // Try to get from Redis (assuming BinanceSpot for now)
    // In production, you'd determine the source dynamically
    match redis.get_latest("BinanceSpot", &symbol).await {
        Ok(Some(data)) => {
            let response = LatestPriceResponse {
                symbol: data.symbol.clone(),
                price: data.price.to_string(),
                timestamp: data.timestamp,
                source: data.source.to_string(),
                bid: data.bid.map(|b| b.to_string()),
                ask: data.ask.map(|a| a.to_string()),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Symbol not found",
                "symbol": symbol
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Redis error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

/// Get historical data for a symbol
/// GET /api/v1/market/:symbol/history?start=<timestamp>&end=<timestamp>&limit=1000
pub async fn get_history(
    State(_state): State<AppState>,
    Path(symbol): Path<String>,
    Query(params): Query<HistoryQuery>,
) -> Response {
    // Validate limit
    let limit = params.limit.min(10000); // Max 10000 records

    // In a real implementation, query ClickHouse here
    // For now, return a placeholder response
    tracing::debug!(
        "History query: symbol={}, start={:?}, end={:?}, limit={}",
        symbol,
        params.start,
        params.end,
        limit
    );

    let response = HistoryResponse {
        symbol: symbol.clone(),
        data: vec![], // Would be populated from ClickHouse
        count: 0,
    };

    (StatusCode::OK, Json(response)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_limit() {
        assert_eq!(default_limit(), 1000);
    }

    #[test]
    fn test_health_response_serialization() {
        let response = HealthResponse {
            status: "healthy".to_string(),
            version: "0.1.0".to_string(),
            uptime_secs: 3600,
            dependencies: HealthDeps {
                redis: DependencyStatus::up(5.0),
                clickhouse: DependencyStatus::up(10.0),
            },
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("healthy"));
        assert!(json.contains("0.1.0"));
    }

    #[test]
    fn test_latest_price_response_serialization() {
        let response = LatestPriceResponse {
            symbol: "BTCUSDT".to_string(),
            price: "50000.0".to_string(),
            timestamp: 1234567890000,
            source: "BinanceSpot".to_string(),
            bid: Some("49999.0".to_string()),
            ask: Some("50001.0".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("BTCUSDT"));
        assert!(json.contains("50000.0"));
    }

    #[test]
    fn test_history_query_deserialization() {
        let json = r#"{"start":1000,"end":2000,"limit":500}"#;
        let query: HistoryQuery = serde_json::from_str(json).unwrap();

        assert_eq!(query.start, Some(1000));
        assert_eq!(query.end, Some(2000));
        assert_eq!(query.limit, 500);
    }

    #[test]
    fn test_history_query_default_limit() {
        let json = r#"{"start":1000}"#;
        let query: HistoryQuery = serde_json::from_str(json).unwrap();

        assert_eq!(query.limit, 1000);
    }
}
