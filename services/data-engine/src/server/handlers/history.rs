use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

use super::super::routes::AppState;

#[derive(Serialize)]
pub struct LogsResponse {
    pub logs: Vec<LogEntry>,
}

#[derive(Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String, // "INFO", "WARN", "ERROR"
    pub message: String,
}

#[derive(Serialize)]
pub struct StrategyStatusResponse {
    pub active: bool,
    pub generation: i64,
    pub fitness: f64,
    pub best_tokens: Vec<usize>,
    pub timestamp: i64,
}

/// GET /api/v1/history/logs
pub async fn get_logs(State(state): State<AppState>) -> Response {
    let redis_opt = state.redis.as_ref();
    if redis_opt.is_none() {
        return (StatusCode::SERVICE_UNAVAILABLE, "Redis unavailable").into_response();
    }

    let redis_cache = redis_opt.unwrap().read().await; // Use read lock instead of write since we just get connection
    let mut conn = redis_cache.get_connection();

    // Fetch logs from Redis List "system:logs" (0 to 99)
    let logs_json: Vec<String> = match conn.lrange("system:logs", 0, 99).await {
        Ok(logs) => logs,
        Err(e) => {
            tracing::error!("Failed to fetch logs from Redis: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Redis error").into_response();
        }
    };

    let logs: Vec<LogEntry> = logs_json
        .into_iter()
        .filter_map(|json| serde_json::from_str::<serde_json::Value>(&json).ok())
        .map(|v| LogEntry {
            timestamp: v["timestamp"].as_str().unwrap_or("").to_string(),
            level: v["action"].as_str().unwrap_or("INFO").to_string(),
            message: v["message"].as_str().unwrap_or("").to_string(),
        })
        .collect();

    (StatusCode::OK, Json(LogsResponse { logs })).into_response()
}

/// GET /api/v1/strategy/status
pub async fn get_strategy_status(State(state): State<AppState>) -> Response {
    let redis_opt = state.redis.as_ref();
    if redis_opt.is_none() {
        return (StatusCode::SERVICE_UNAVAILABLE, "Redis unavailable").into_response();
    }

    let redis_cache = redis_opt.unwrap().read().await; // Read lock
    let mut conn = redis_cache.get_connection();

    // Fetch status key "strategy:status"
    let status_json: Option<String> = match conn.get("strategy:status").await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to fetch strategy status: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Redis error").into_response();
        }
    };

    if let Some(json_str) = status_json {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&json_str) {
            let response = StrategyStatusResponse {
                active: true,
                generation: v["generation"].as_i64().unwrap_or(0),
                fitness: v["fitness"].as_f64().unwrap_or(0.0),
                best_tokens: serde_json::from_value(v["formula"].clone()).unwrap_or_default(),
                timestamp: v["timestamp"].as_i64().unwrap_or(0),
            };
            return (StatusCode::OK, Json(response)).into_response();
        }
    }

    // Default response if no status yet
    (
        StatusCode::OK,
        Json(StrategyStatusResponse {
            active: false,
            generation: 0,
            fitness: 0.0,
            best_tokens: vec![],
            timestamp: 0,
        }),
    )
        .into_response()
}
