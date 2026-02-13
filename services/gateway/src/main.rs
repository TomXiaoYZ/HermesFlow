use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use clickhouse::Client as ClickHouseClient;
use futures::stream::StreamExt;
use redis::AsyncCommands;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth_handler;
mod health_checker;
mod metrics;

struct AppState {
    tx: broadcast::Sender<String>,
    ch_client: ClickHouseClient,
    http_client: reqwest::Client,
    user_management_url: String,
    pg_pool: sqlx::PgPool,
}

#[derive(Deserialize)]
struct LogQuery {
    service: String, // "data-engine", "gateway" etc. or "all"
    level: Option<String>,
    keyword: Option<String>,
    limit: Option<u64>,
}

#[derive(clickhouse::Row, serde::Serialize, serde::Deserialize)]
struct SystemLog {
    timestamp: i64, // unix timestamp
    container_name: String,
    level: String,
    message: String,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Gateway Service...");

    // Initialize Prometheus metrics
    if let Err(e) = metrics::init_gateway_metrics() {
        error!("Failed to initialize metrics: {}", e);
    }

    // Redis URL
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    // ClickHouse URL
    let ch_url =
        std::env::var("CLICKHOUSE_URL").unwrap_or_else(|_| "http://localhost:8123".to_string());
    let ch_user = std::env::var("CLICKHOUSE_USER").unwrap_or_else(|_| "default".to_string());
    let ch_pass = std::env::var("CLICKHOUSE_PASSWORD").unwrap_or_else(|_| "".to_string());

    let ch_client = ClickHouseClient::default()
        .with_url(&ch_url)
        .with_user(&ch_user)
        .with_password(&ch_pass);

    // Spawn health checker with retry logic
    let redis_url_health = redis_url.clone();
    tokio::spawn(async move {
        // Wait for Redis to be ready
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        match redis::Client::open(redis_url_health.as_str()) {
            Ok(client) => {
                info!("Health checker starting...");
                let health_checker = health_checker::HealthChecker::new();
                health_checker.start_monitoring(client).await;
            }
            Err(e) => {
                error!("Failed to connect to Redis for health checker: {}", e);
            }
        }
    });

    // Broadcast Channel: Capacity 100

    let (tx, _rx) = broadcast::channel(100);

    let pg_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:hermesflow@localhost:5432/hermesflow".to_string());

    // Connect to Postgres
    let pg_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&pg_url)
        .await
        .expect("Failed to connect to Postgres");

    let app_state = Arc::new(AppState {
        tx: tx.clone(),
        ch_client,
        http_client: reqwest::Client::new(),
        user_management_url: std::env::var("USER_MANAGEMENT_URL")
            .unwrap_or_else(|_| "http://user-management:8086".to_string()),
        pg_pool,
    });

    // Spawn Redis Subscriber Task
    let tx_clone = tx.clone();
    let redis_url_clone = redis_url.clone();

    tokio::spawn(async move {
        info!("Connecting to Redis PubSub...");
        let client = match redis::Client::open(redis_url_clone) {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to create Redis client: {}", e);
                return;
            }
        };

        let con = match client.get_async_connection().await {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to connect to Redis: {}", e);
                return;
            }
        };

        let mut pubsub = con.into_pubsub();
        // Subscribe to relevant channels
        let channels = [
            "trade_signals",
            "portfolio_updates",
            "market_data",
            "strategy_logs",
            "system_metrics",
        ];
        for ch in channels {
            if let Err(e) = pubsub.subscribe(ch).await {
                error!("Failed to subscribe to {}: {}", ch, e);
            }
        }

        info!("Redis PubSub Active. Forwarding messages to WebSocket clients.");

        let _ = pubsub.subscribe("system_heartbeat").await;

        let mut stream = pubsub.on_message();
        while let Some(msg) = stream.next().await {
            match msg.get_payload::<String>() {
                Ok(payload) => {
                    let channel_name = msg.get_channel_name();
                    let event_type = match channel_name {
                        "trade_signals" => "signal",
                        "portfolio_updates" => "portfolio",
                        "market_data" => "market",
                        "strategy_logs" => "log",
                        "system_metrics" => "metrics",
                        "system_heartbeat" => "heartbeat",
                        _ => "unknown",
                    };

                    let wrapper = json!({
                        "type": event_type,
                        "data": serde_json::from_str::<Value>(&payload).unwrap_or(Value::String(payload))
                    });

                    let _ = tx_clone.send(wrapper.to_string());
                }
                Err(e) => error!("Failed to get payload: {}", e),
            }
        }
    });

    // Build App
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(common::metrics::metrics_handler))
        .route("/ws", get(ws_handler))
        .route("/api/logs", get(get_logs))
        .route("/api/v1/strategy/status", get(get_strategy_status))
        .route("/api/v1/strategy/population", get(get_strategy_population))
        .route("/api/v1/strategy/history", get(get_strategy_history))
        .route("/api/v1/backtest/history", get(get_backtest_history))
        .route("/api/v1/backtest/run", axum::routing::post(run_backtest_proxy))
        .route("/api/v1/market/tokens", get(get_market_tokens))
        .route("/api/v1/data/*path", axum::routing::any(data_engine_proxy))
        .route("/api/v1/watchlist", axum::routing::any(watchlist_proxy))
        .route("/api/v1/jobs/*path", axum::routing::any(jobs_proxy))
        .route("/api/auth/login", axum::routing::any(auth_handler::proxy_handler))
        .layer(CorsLayer::permissive()) // Enable CORS for local dev
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("Gateway listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok", "service": "gateway" }))
}

async fn get_logs(
    State(state): State<Arc<AppState>>,
    Query(params): Query<LogQuery>,
) -> Json<Value> {
    let limit = params.limit.unwrap_or(100).min(1000);

    let mut query = "SELECT toInt64(toUnixTimestamp(timestamp)) as timestamp, container_name, level, message FROM system_logs WHERE 1=1".to_string();

    if params.service != "all" {
        query.push_str(&format!(" AND container_name ILIKE '%{}%'", params.service));
    }

    if let Some(level) = &params.level {
        if level != "ALL" {
            query.push_str(&format!(" AND level = '{}'", level));
        }
    }

    if let Some(keyword) = &params.keyword {
        let safe_keyword = keyword.replace("'", "''");
        query.push_str(&format!(" AND message ILIKE '%{}%'", safe_keyword));
    }

    query.push_str(&format!(" ORDER BY timestamp DESC LIMIT {}", limit));

    match state.ch_client.query(&query).fetch_all::<SystemLog>().await {
        Ok(logs) => Json(json!(logs)),
        Err(e) => {
            error!("ClickHouse query error: {}", e);
            Json(json!({ "error": "Failed to fetch logs" }))
        }
    }
}

async fn get_strategy_status(State(_state): State<Arc<AppState>>) -> Json<Value> {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    match redis::Client::open(redis_url) {
        Ok(client) => match client.get_async_connection().await {
            Ok(mut con) => match con.get::<_, String>("strategy:status").await {
                Ok(data) => {
                    if let Ok(json) = serde_json::from_str::<Value>(&data) {
                        return Json(json);
                    }
                    Json(json!({"error": "Invalid JSON in redis"}))
                }
                Err(_) => Json(json!({"error": "No strategy status found"})),
            },
            Err(e) => {
                error!("Redis connection error: {}", e);
                Json(json!({"error": "Redis unavailable"}))
            }
        },
        Err(e) => {
            error!("Redis client error: {}", e);
            Json(json!({"error": "Redis config error"}))
        }
    }
}

async fn get_strategy_population(State(_state): State<Arc<AppState>>) -> Json<Value> {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    match redis::Client::open(redis_url) {
        Ok(client) => match client.get_async_connection().await {
            Ok(mut con) => match con.get::<_, String>("strategy:population").await {
                Ok(data) => {
                    if let Ok(json) = serde_json::from_str::<Value>(&data) {
                        return Json(json);
                    }
                    Json(json!([]))
                }
                Err(_) => Json(json!([])),
            },
            Err(e) => {
                error!("Redis connection error: {}", e);
                Json(json!([]))
            }
        },
        Err(e) => {
            error!("Redis client error: {}", e);
            Json(json!([]))
        }
    }
}

async fn get_strategy_history(State(state): State<Arc<AppState>>) -> Json<Value> {
    use sqlx::Row;
    let query = "SELECT generation, fitness, timestamp, metadata FROM strategy_generations ORDER BY generation DESC LIMIT 1000";

    match sqlx::query(query).fetch_all(&state.pg_pool).await {
        Ok(rows) => {
            let mut results = Vec::new();
            for row in rows {
                let gen: i32 = row.get("generation");
                let fit: f64 = row.get("fitness");
                let ts: chrono::DateTime<chrono::Utc> = row.get("timestamp");
                let meta: Value = row.get("metadata");

                results.push(json!({
                    "generation": gen,
                    "fitness": fit,
                    "timestamp": ts.timestamp(),
                    "meta": meta
                }));
            }
            Json(json!(results))
        }
        Err(e) => {
            error!("Postgres query error: {}", e);
            Json(json!({"error": "Failed to fetch history"}))
        }
    }
}

async fn get_backtest_history(State(state): State<Arc<AppState>>) -> Json<Value> {
    use sqlx::Row;
    let query = "SELECT id, strategy_id, token_address, metrics, created_at FROM backtest_results ORDER BY created_at DESC LIMIT 100";

    match sqlx::query(query).fetch_all(&state.pg_pool).await {
        Ok(rows) => {
            let mut results = Vec::new();
            for row in rows {
                let id: uuid::Uuid = row.get("id");
                let sid: Option<String> = row.get("strategy_id");
                let token: String = row.get("token_address");
                let metrics: Value = row.get("metrics");
                let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");

                results.push(json!({
                    "id": id.to_string(),
                    "strategy_id": sid,
                    "token_address": token,
                    "metrics": metrics,
                    "created_at": created_at.timestamp()
                }));
            }
            Json(json!(results))
        }
        Err(e) => {
            error!("Postgres query error: {}", e);
            Json(json!({"error": "Failed to fetch backtests"}))
        }
    }
}

async fn run_backtest_proxy(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let target_url = "http://strategy-generator:8082/backtest";

    match state
        .http_client
        .post(target_url)
        .json(&payload)
        .send()
        .await
    {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or("".to_string());
            (status, body).into_response()
        }
        Err(e) => {
            error!("Failed to proxy backtest: {}", e);
            metrics::PROXY_ERRORS_TOTAL
                .with_label_values(&["strategy-generator"])
                .inc();
            (
                axum::http::StatusCode::BAD_GATEWAY,
                json!({"error": "Backtest service unavailable"}).to_string(),
            )
                .into_response()
        }
    }
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    metrics::WEBSOCKET_CONNECTIONS.inc();
    let mut rx = state.tx.subscribe();
    if socket
        .send(Message::Text(
            json!({"type": "connect", "status": "ok"}).to_string(),
        ))
        .await
        .is_err()
    {
        metrics::WEBSOCKET_CONNECTIONS.dec();
        return;
    }

    loop {
        tokio::select! {
            Ok(msg) = rx.recv() => {
                if socket.send(Message::Text(msg)).await.is_err() {
                    break;
                }
            }
            else => break,
        }
    }
    metrics::WEBSOCKET_CONNECTIONS.dec();
}

async fn get_market_tokens(State(state): State<Arc<AppState>>) -> Json<Value> {
    use sqlx::Row;
    match sqlx::query("SELECT address, symbol, name FROM active_tokens ORDER BY symbol")
        .fetch_all(&state.pg_pool)
        .await
    {
        Ok(rows) => {
            let mut results = Vec::new();
            for row in rows {
                let addr: String = row.get("address");
                let symbol: String = row.get("symbol");
                let name: String = row.get("name");
                results.push(json!({
                    "address": addr,
                    "symbol": symbol,
                    "name": name
                }));
            }
            Json(json!(results))
        }
        Err(e) => {
            error!("Failed to fetch tokens: {}", e);
            Json(json!([]))
        }
    }
}

async fn watchlist_proxy(
    State(state): State<Arc<AppState>>,
    req: axum::extract::Request,
) -> impl IntoResponse {
    let (parts, body) = req.into_parts();
    let query_string = parts
        .uri
        .query()
        .map(|q| format!("?{}", q))
        .unwrap_or_default();
    let target_url = format!("http://data-engine:8080/api/v1/watchlist{}", query_string);

    // Read body content
    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .unwrap_or_default();

    // Build new request
    let mut request_builder = state.http_client.request(parts.method, &target_url);

    // Forward headers
    for (key, value) in parts.headers.iter() {
        request_builder = request_builder.header(key, value);
    }

    // Set body
    request_builder = request_builder.body(body_bytes);

    match request_builder.send().await {
        Ok(resp) => {
            let status = resp.status();
            let headers = resp.headers().clone();
            let body = resp.bytes().await.unwrap_or_default();

            let mut response = (status, body).into_response();
            *response.headers_mut() = headers;
            response
        }
        Err(e) => {
            error!("Failed to proxy to data-engine watchlist: {}", e);
            metrics::PROXY_ERRORS_TOTAL
                .with_label_values(&["data-engine"])
                .inc();
            (
                axum::http::StatusCode::BAD_GATEWAY,
                json!({"error": "Data Engine unavailable"}).to_string(),
            )
                .into_response()
        }
    }
}

async fn data_engine_proxy(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(path): axum::extract::Path<String>,
    req: axum::extract::Request,
) -> impl IntoResponse {
    // Deconstruct request
    let (parts, body) = req.into_parts();

    // Extract query string from original URI
    let query_string = parts
        .uri
        .query()
        .map(|q| format!("?{}", q))
        .unwrap_or_default();
    let target_url = format!(
        "http://data-engine:8080/api/v1/data/{}{}",
        path, query_string
    );

    error!(
        "[GATEWAY DEBUG] Query: {:?}, Target: {}",
        parts.uri.query(),
        target_url
    );

    // Read body content
    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .unwrap_or_default();

    // Build new request
    let mut request_builder = state.http_client.request(parts.method, &target_url);

    // Forward headers
    for (key, value) in parts.headers.iter() {
        request_builder = request_builder.header(key, value);
    }

    // Set body
    request_builder = request_builder.body(body_bytes);

    match request_builder.send().await {
        Ok(resp) => {
            let status = resp.status();
            let headers = resp.headers().clone();
            let body = resp.bytes().await.unwrap_or_default();

            let mut response = (status, body).into_response();
            *response.headers_mut() = headers;
            response
        }
        Err(e) => {
            error!("Failed to proxy to data-engine: {}", e);
            metrics::PROXY_ERRORS_TOTAL
                .with_label_values(&["data-engine"])
                .inc();
            (
                axum::http::StatusCode::BAD_GATEWAY,
                json!({"error": "Data Engine unavailable"}).to_string(),
            )
                .into_response()
        }
    }
}

async fn jobs_proxy(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(path): axum::extract::Path<String>,
    req: axum::extract::Request,
) -> impl IntoResponse {
    let (parts, body) = req.into_parts();
    let query_string = parts
        .uri
        .query()
        .map(|q| format!("?{}", q))
        .unwrap_or_default();
    let target_url = format!(
        "http://data-engine:8080/api/v1/jobs/{}{}",
        path, query_string
    );

    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .unwrap_or_default();
    let mut request_builder = state.http_client.request(parts.method, &target_url);

    for (key, value) in parts.headers.iter() {
        request_builder = request_builder.header(key, value);
    }
    request_builder = request_builder.body(body_bytes);

    match request_builder.send().await {
        Ok(resp) => {
            let status = resp.status();
            let headers = resp.headers().clone();
            let body = resp.bytes().await.unwrap_or_default();
            let mut response = (status, body).into_response();
            *response.headers_mut() = headers;
            response
        }
        Err(e) => {
            error!("Failed to proxy to data-engine jobs: {}", e);
            metrics::PROXY_ERRORS_TOTAL
                .with_label_values(&["data-engine"])
                .inc();
            (
                axum::http::StatusCode::BAD_GATEWAY,
                json!({"error": "Data Engine unavailable"}).to_string(),
            )
                .into_response()
        }
    }
}
