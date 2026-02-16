use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::monitoring::metrics::export_metrics;
use crate::monitoring::metrics::{ACTIVE_SYMBOLS_COUNT, BIRDEYE_API_REQUESTS_TOTAL};
use crate::monitoring::{CollectorHealth, DependencyStatus, HealthStatus};
use std::time::Duration;

pub mod agent;
pub mod config;
pub mod data;
pub mod history;
pub mod jobs;
pub mod prediction;
pub mod trading;

// ... metrics background updater ...
pub async fn spawn_metrics_updater(state: AppState) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10)); // Increased frequency to 10s
        let mut _ticks: u64 = 0;
        let mut last_birdeye_reqs = BIRDEYE_API_REQUESTS_TOTAL.get();

        loop {
            interval.tick().await;
            _ticks += 1;

            // Query DB count
            let pool = &state.postgres.pool;
            let count_res: Result<i64, _> =
                sqlx::query_scalar("SELECT count(*) FROM active_tokens WHERE is_active = true")
                    .fetch_one(pool)
                    .await;

            if let Ok(count) = count_res {
                ACTIVE_SYMBOLS_COUNT.set(count);

                // Publish to Redis
                if let Some(redis_lock) = &state.redis {
                    let redis = redis_lock.read().await;
                    let mut conn = redis.get_connection();
                    use redis::AsyncCommands;

                    let birdeye_reqs = BIRDEYE_API_REQUESTS_TOTAL.get();

                    let payload = serde_json::json!({
                        "active_tokens": count,
                        "birdeye_requests": birdeye_reqs,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    })
                    .to_string();

                    if let Err(e) = conn.publish::<_, _, ()>("system_metrics", payload).await {
                        tracing::error!("Failed to publish system_metrics: {}", e);
                    }
                }
            }

            // Persist to DB every 60 seconds (6 ticks)
            if _ticks.is_multiple_of(6) {
                let current_birdeye_reqs = BIRDEYE_API_REQUESTS_TOTAL.get();
                let delta = if current_birdeye_reqs >= last_birdeye_reqs {
                    current_birdeye_reqs - last_birdeye_reqs
                } else {
                    current_birdeye_reqs // Should not happen in same process, but fail-safe
                };

                if delta > 0.0 {
                    use crate::repository::MetricsRepository;
                    if let Err(e) = state
                        .postgres
                        .metrics
                        .insert_api_usage("BirdEye", delta as i64)
                        .await
                    {
                        tracing::error!("Failed to persist API usage: {}", e);
                    } else {
                        last_birdeye_reqs = current_birdeye_reqs;
                    }
                }
            }
        }
    });
}

use super::routes::AppState;

/// Health check response
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_secs: u64,
    pub dependencies: HealthDeps,
    pub collectors: Vec<CollectorHealth>,
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
    pub resolution: Option<String>,
    pub exchange: Option<String>,
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
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Active Token Summary
#[derive(Serialize)]
pub struct TokenSummary {
    pub address: String,
    pub symbol: String,
    pub name: Option<String>,
    pub price: Option<f64>,
    pub volume_24h: Option<f64>,
    pub change_24h: Option<f64>,
    pub token_type: Option<String>,
}

#[derive(Serialize)]
pub struct TokenListResponse {
    pub tokens: Vec<TokenSummary>,
    pub count: usize,
}

/// Health check endpoint
/// GET /health
pub async fn health_check(State(state): State<AppState>) -> Response {
    let health_status = state.health_monitor.check_health().await;
    let uptime = state.health_monitor.uptime_secs();

    let redis_status = state.health_monitor.get_redis_status().await;
    let clickhouse_status = state.health_monitor.get_clickhouse_status().await;
    let collectors = state.health_monitor.collector_statuses().await;

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
        collectors,
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
    let redis_opt = state.redis.as_ref();

    if redis_opt.is_none() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "Redis unavailable",
                "symbol": symbol
            })),
        )
            .into_response();
    }

    let mut redis = redis_opt.unwrap().write().await;

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
/// GET /api/v1/market/:symbol/history?start=<timestamp>&end=<timestamp>&limit=1000&resolution=1m
pub async fn get_history(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
    Query(params): Query<HistoryQuery>,
) -> Response {
    let pool = &state.postgres.pool;
    let resolution = params.resolution.unwrap_or_else(|| "1h".to_string());
    let limit = params.limit.min(10000) as i64;

    // Default to last 24h if no start
    let end_ts = params
        .end
        .map(|ts| chrono::DateTime::from_timestamp(ts / 1000, 0).unwrap_or(chrono::Utc::now()))
        .unwrap_or(chrono::Utc::now());
    let start_ts = params
        .start
        .map(|ts| {
            chrono::DateTime::from_timestamp(ts / 1000, 0)
                .unwrap_or(end_ts - chrono::Duration::hours(24))
        })
        .unwrap_or(end_ts - chrono::Duration::days(7)); // Default 7 days if not provided

    let exchange = params
        .exchange
        .clone()
        .unwrap_or_else(|| "Birdeye".to_string());

    let query = "
        SELECT time,
               open::float8 AS open, high::float8 AS high,
               low::float8 AS low, close::float8 AS close,
               volume::float8 AS volume
        FROM mkt_equity_candles
        WHERE exchange = $5 AND symbol = $1 AND resolution = $2 AND time >= $3 AND time <= $4
        ORDER BY time ASC
        LIMIT $6
    ";

    tracing::debug!(
        "[History API] symbol={}, resolution={}, start={}, end={}, exchange={}, limit={}",
        symbol,
        resolution,
        start_ts,
        end_ts,
        exchange,
        limit
    );

    let rows_res = sqlx::query(query)
        .bind(&symbol)
        .bind(&resolution)
        .bind(start_ts)
        .bind(end_ts)
        .bind(&exchange)
        .bind(limit)
        .fetch_all(pool)
        .await;

    match rows_res {
        Ok(rows) => {
            let data: Vec<MarketDataPoint> = rows
                .into_iter()
                .map(|row| {
                    let time: chrono::DateTime<chrono::Utc> = row.get("time");

                    MarketDataPoint {
                        timestamp: time.timestamp_millis(),
                        open: row.get("open"),
                        high: row.get("high"),
                        low: row.get("low"),
                        close: row.get("close"),
                        volume: row.get("volume"),
                    }
                })
                .collect();

            (
                StatusCode::OK,
                Json(HistoryResponse {
                    symbol,
                    count: data.len(),
                    data,
                }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to fetch history: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

/// Get active tokens list
/// GET /api/v1/market/tokens
pub async fn get_active_tokens(State(state): State<AppState>) -> Response {
    let pool = &state.postgres.pool;

    // 1. Fetch data from Postgres (Crypto with prices, Stocks with 0.0 placeholders)
    // We cannot join with ClickHouse tables in Postgres, so we fetch standard data here
    let query = "
        WITH LatestPrices AS (
            SELECT DISTINCT ON (symbol) symbol, price, volume, time
            FROM mkt_equity_snapshots
            WHERE time > NOW() - INTERVAL '2 hours'
            ORDER BY symbol, time DESC
        )
        SELECT 
            t.address, 
            t.symbol, 
            t.name, 
            COALESCE(lp.price, 0.0) as price, 
            COALESCE(lp.volume, t.volume_24h) as volume_24h, 
            t.price_change_24h,
            'crypto' as asset_type
        FROM active_tokens t
        LEFT JOIN LatestPrices lp ON t.address = lp.symbol
        WHERE t.is_active = true 
        
        UNION ALL

        SELECT
            w.symbol as address,
            w.symbol,
            w.name as name,
            COALESCE(c.close, 0.0) as price,
            COALESCE(c.volume, 0.0) as volume_24h,
            CASE
                WHEN c.close IS NOT NULL AND prev.close IS NOT NULL AND prev.close > 0
                THEN ((c.close - prev.close) / prev.close * 100)
                ELSE 0.0
            END as price_change_24h,
            'stock' as asset_type
        FROM market_watchlist w
        LEFT JOIN LATERAL (
            SELECT close, volume, time
            FROM mkt_equity_candles
            WHERE symbol = w.symbol AND exchange = 'Polygon'
            ORDER BY time DESC LIMIT 1
        ) c ON true
        LEFT JOIN LATERAL (
            SELECT close
            FROM mkt_equity_candles
            WHERE symbol = w.symbol AND exchange = 'Polygon'
              AND time <= c.time - INTERVAL '24 hours'
            ORDER BY time DESC LIMIT 1
        ) prev ON true
        WHERE w.is_active = true
        
        ORDER BY asset_type DESC, volume_24h DESC 
        LIMIT 200
    ";

    let rows_res = sqlx::query(query).fetch_all(pool).await;

    match rows_res {
        Ok(rows) => {
            let tokens: Vec<TokenSummary> = rows
                .into_iter()
                .map(|row| {
                    use rust_decimal::prelude::ToPrimitive;
                    let price: Option<rust_decimal::Decimal> = row.try_get("price").ok();
                    let vol: Option<rust_decimal::Decimal> = row.try_get("volume_24h").ok();
                    let change: Option<rust_decimal::Decimal> =
                        row.try_get("price_change_24h").ok();

                    TokenSummary {
                        address: row.get("address"),
                        symbol: row.get("symbol"),
                        name: row.try_get("name").ok(),
                        price: price.and_then(|d| d.to_f64()),
                        volume_24h: vol.and_then(|d| d.to_f64()),
                        change_24h: change.and_then(|d| d.to_f64()),
                        token_type: row.try_get("asset_type").ok(),
                    }
                })
                .collect();

            (
                StatusCode::OK,
                Json(TokenListResponse {
                    count: tokens.len(),
                    tokens,
                }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to fetch tokens: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

/// WebSocket handler for real-time market data
/// GET /ws
pub async fn ws_handler(State(state): State<AppState>, ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut rx = state.broadcast_tx.subscribe();

    // Only send data to client, we ignore incoming messages for now
    // Or we could handle subscription requests

    loop {
        match rx.recv().await {
            Ok(msg) => {
                if let Err(_e) = socket.send(Message::Text(msg)).await {
                    // Client disconnected
                    break;
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                // Client lagged, skip messages
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                break;
            }
        }
    }
}

/// Get latest strategy population (leaderboard)
/// GET /api/v1/strategy/population
pub async fn get_strategy_population(State(state): State<AppState>) -> Response {
    let redis_opt = state.redis.as_ref();

    if redis_opt.is_none() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "Redis unavailable"
            })),
        )
            .into_response();
    }

    let redis = redis_opt.unwrap().read().await;
    let mut conn = redis.get_connection();

    // Fetch raw JSON string from Redis
    // We import AsyncCommands at the top of file to use .get()
    use redis::AsyncCommands;

    let result: Result<Option<String>, _> = conn.get("strategy:population").await;

    match result {
        Ok(Some(json_str)) => {
            // It's already JSON string, return it directly with application/json content type
            // But Axum Json() expects a serializable object.
            // We can deserialize to Value and re-serialize, OR just return string with header.
            // Let's deserialize to Value to be safe and standard.
            let val: serde_json::Value =
                serde_json::from_str(&json_str).unwrap_or(serde_json::json!([]));
            (StatusCode::OK, Json(val)).into_response()
        }
        Ok(None) => (StatusCode::OK, Json(serde_json::json!([]))).into_response(),
        Err(e) => {
            tracing::error!("Redis error fetching population: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error"
                })),
            )
                .into_response()
        }
    }
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
            collectors: vec![CollectorHealth {
                name: "binance".to_string(),
                connected: true,
                messages_per_min: 120.0,
                last_message_at: Some(1700000000),
                consecutive_errors: 0,
            }],
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("healthy"));
        assert!(json.contains("0.1.0"));
        assert!(json.contains("binance"));
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
        let json = r#"{"start":1000,"end":2000,"limit":500, "resolution": "15m"}"#;
        let query: HistoryQuery = serde_json::from_str(json).unwrap();

        assert_eq!(query.start, Some(1000));
        assert_eq!(query.end, Some(2000));
        assert_eq!(query.limit, 500);
        assert_eq!(query.resolution, Some("15m".to_string()));
    }
}
