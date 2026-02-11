use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{Column, Pool, Postgres, Row, TypeInfo};

use crate::server::routes::AppState;

/// Data Quality Metrics Response
#[derive(Serialize)]
pub struct DataQualityResponse {
    pub snapshots: MetricStatus,
    pub candles_1m: MetricStatus,
    pub candles_15m: MetricStatus,
    pub candles_1h: MetricStatus,
    pub candles_4h: MetricStatus,
    pub candles_1d: MetricStatus,
    pub token_discovery: MetricStatus,
    pub active_tokens: i64,
}

#[derive(Serialize)]
pub struct MetricStatus {
    pub latest: Option<String>,
    pub lag_seconds: Option<i64>,
    pub status: String, // "healthy", "degraded", "stale", "empty"
}

/// SQL Query Request
#[derive(Deserialize)]
pub struct SqlQueryRequest {
    pub query: String,
}

/// List Tables Request
#[derive(Serialize)]
pub struct TableListResponse {
    pub tables: Vec<TableInfo>,
}

#[derive(Serialize)]
pub struct TableInfo {
    pub name: String,
    pub size: Option<String>,
}

async fn fetch_metric(pool: &Pool<Postgres>, resolution: &str, threshold: i64) -> MetricStatus {
    let res: Result<Option<chrono::DateTime<Utc>>, _> =
        sqlx::query_scalar("SELECT MAX(time) FROM mkt_equity_candles WHERE resolution = $1")
            .bind(resolution)
            .fetch_one(pool)
            .await;
    calc_metric(res, threshold)
}

/// GET /api/v1/data/quality
pub async fn get_data_quality(State(state): State<AppState>) -> Response {
    let pool = &state.postgres.pool;

    // 1. Snapshots
    let snap_res: Result<Option<chrono::DateTime<Utc>>, _> =
        sqlx::query_scalar("SELECT MAX(timestamp) FROM mkt_equity_snapshots")
            .fetch_one(pool)
            .await;
    let snap_metric = calc_metric(snap_res, 30); // Healthy if < 30s

    // 2. Candles (Various Resolutions)
    let candles_1m = fetch_metric(pool, "1m", 120).await;
    let candles_15m = fetch_metric(pool, "15m", 15 * 60 + 300).await;
    let candles_1h = fetch_metric(pool, "1h", 3600 + 600).await;
    let candles_4h = fetch_metric(pool, "4h", 4 * 3600 + 1200).await;
    let candles_1d = fetch_metric(pool, "1d", 24 * 3600 + 3600).await;

    // 3. Token Discovery
    let token_res: Result<Option<chrono::DateTime<Utc>>, _> =
        sqlx::query_scalar("SELECT MAX(last_updated) FROM active_tokens")
            .fetch_one(pool)
            .await;
    let token_metric = calc_metric(token_res, 3600 + 600); // 1h + 10m buffer

    // 4. Active Count
    let count_res: Result<i64, _> =
        sqlx::query_scalar("SELECT count(*) FROM active_tokens WHERE is_active = true")
            .fetch_one(pool)
            .await;

    let response = DataQualityResponse {
        snapshots: snap_metric,
        candles_1m,
        candles_15m,
        candles_1h,
        candles_4h,
        candles_1d,
        token_discovery: token_metric,
        active_tokens: count_res.unwrap_or(0),
    };

    (StatusCode::OK, Json(response)).into_response()
}

fn calc_metric(
    res: Result<Option<chrono::DateTime<Utc>>, sqlx::Error>,
    threshold_sec: i64,
) -> MetricStatus {
    match res {
        Ok(Some(ts)) => {
            let now = Utc::now();
            let lag = (now - ts).num_seconds();
            let status = if lag <= threshold_sec {
                "healthy"
            } else {
                "stale"
            };
            MetricStatus {
                latest: Some(ts.to_rfc3339()),
                lag_seconds: Some(lag),
                status: status.to_string(),
            }
        }
        Ok(None) => MetricStatus {
            latest: None,
            lag_seconds: None,
            status: "empty".to_string(),
        },
        Err(e) => {
            tracing::error!("Metric DB Error: {}", e);
            MetricStatus {
                latest: None,
                lag_seconds: None,
                status: "error".to_string(),
            }
        }
    }
}

/// GET /api/v1/data/tables
pub async fn get_tables(State(state): State<AppState>) -> Response {
    let pool = &state.postgres.pool;

    // Simple query to get public tables
    let query = "
        SELECT table_name 
        FROM information_schema.tables 
        WHERE table_schema = 'public' 
        ORDER BY table_name
    ";

    let rows_res = sqlx::query(query).fetch_all(pool).await;

    let restricted = [
        "users",
        "users_roles",
        "permissions",
        "roles",
        "_sqlx_migrations",
        "schema_migrations",
    ];

    match rows_res {
        Ok(rows) => {
            let tables: Vec<TableInfo> = rows
                .into_iter()
                .filter_map(|r| {
                    let name: String = r.get("table_name");
                    if restricted.contains(&name.as_str()) {
                        None
                    } else {
                        Some(TableInfo { name, size: None })
                    }
                })
                .collect();
            (StatusCode::OK, Json(TableListResponse { tables })).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// POST /api/v1/data/query
pub async fn query_data(
    State(state): State<AppState>,
    Json(payload): Json<SqlQueryRequest>,
) -> Response {
    let pool = &state.postgres.pool;
    let query_str = payload.query.trim();

    // Safety Check: Forbidden Keywords
    let forbidden_keywords = [
        "DROP", "DELETE", "UPDATE", "INSERT", "ALTER", "TRUNCATE", "GRANT", "REVOKE",
    ];
    let upper_query = query_str.to_uppercase();
    for kw in forbidden_keywords {
        if upper_query.contains(kw) {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({"error": format!("Query contains forbidden keyword: {}", kw)})),
            )
                .into_response();
        }
    }

    // Safety Check: Restricted Tables
    let restricted_tables = ["users", "users_roles", "permissions", "roles"];
    // Naive check - if table name appears in query, block it.
    // Ideally use a SQL parser, but for this simpler protection, string matching is filter-first.
    // We check if the table name exists as a whole word to avoid partial matches (like 'users_stats' matching 'users')
    // Actually simple contains is risky. Let's stick to simple contains but check for boundaries or just be strict.
    // Being strict is safer for now.
    for table in restricted_tables {
        if upper_query.contains(&table.to_uppercase()) {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({"error": format!("Access to table '{}' is restricted", table)})),
            )
                .into_response();
        }
    }

    // Execute
    let res = sqlx::query(query_str).fetch_all(pool).await;

    match res {
        Ok(rows) => {
            if rows.is_empty() {
                return (StatusCode::OK, Json(json!({"columns": [], "rows": []}))).into_response();
            }

            // Extract Columns
            let columns: Vec<String> = rows[0]
                .columns()
                .iter()
                .map(|c| c.name().to_string())
                .collect();

            // Extract Rows
            let mut result_rows = Vec::new();
            for row in rows {
                let mut row_values = Vec::new();
                for (i, col) in row.columns().iter().enumerate() {
                    let type_name = col.type_info().name();

                    let val: Value =
                        if type_name == "VARCHAR" || type_name == "TEXT" || type_name == "CHAR" {
                            let v: Option<String> = row.try_get(i).ok();
                            json!(v)
                        } else if type_name == "INT4"
                            || type_name == "INT8"
                            || type_name == "BIGINT"
                            || type_name == "INTEGER"
                        {
                            let v: Option<i64> = row.try_get(i).ok();
                            json!(v)
                        } else if type_name == "FLOAT4" || type_name == "FLOAT8" {
                            let v: Option<f64> = row.try_get(i).ok();
                            json!(v)
                        } else if type_name == "NUMERIC" || type_name == "DECIMAL" {
                            // Use rust_decimal for numeric types
                            let v: Option<rust_decimal::Decimal> = row.try_get(i).ok();
                            json!(v)
                        } else if type_name == "TIMESTAMPTZ" || type_name == "TIMESTAMP" {
                            let v: Option<chrono::DateTime<Utc>> = row.try_get(i).ok();
                            json!(v)
                        } else if type_name == "BOOL" || type_name == "BOOLEAN" {
                            let v: Option<bool> = row.try_get(i).ok();
                            json!(v)
                        } else {
                            // Default fallback: Try String, if fail, null
                            let v: Option<String> = row.try_get(i).ok();
                            json!(v)
                        };

                    row_values.push(val);
                }
                result_rows.push(row_values);
            }

            (
                StatusCode::OK,
                Json(json!({
                    "columns": columns,
                    "rows": result_rows
                })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// POST /api/v1/data/tasks/discovery
pub async fn trigger_token_discovery(State(state): State<AppState>) -> Response {
    if let Some(tm) = &state.task_manager {
        match tm.trigger_discovery().await {
            Ok(_) => (StatusCode::ACCEPTED, Json(json!({"status": "triggered"}))).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response(),
        }
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "Task Manager not available"})),
        )
            .into_response()
    }
}

/// POST /api/v1/data/tasks/aggregation
pub async fn trigger_aggregation(State(state): State<AppState>) -> Response {
    if let Some(tm) = &state.task_manager {
        match tm.trigger_aggregation().await {
            Ok(_) => (StatusCode::ACCEPTED, Json(json!({"status": "triggered"}))).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response(),
        }
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "Task Manager not available"})),
        )
            .into_response()
    }
}
