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
    // Initialize tracing (with OpenTelemetry if OTEL_EXPORTER_OTLP_ENDPOINT is set)
    if !common::telemetry::try_init_telemetry("gateway") {
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new(
                std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
            ))
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

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
            "order_updates",
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
                        "order_updates" => "order_update",
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
        .route("/api/v1/evolution/*path", get(evolution_proxy))
        .route("/api/v1/trades/history", get(get_trade_history))
        .route("/api/v1/trades/positions", get(get_trade_positions))
        .route("/api/v1/trades/strategy/:strategy_id", get(get_trade_strategy))
        .route("/api/v1/trades/account-summary", get(get_account_summary))
        .route("/api/v1/config/accounts", get(get_trading_accounts))
        .route(
            "/api/v1/config/accounts/:account_id",
            axum::routing::put(update_trading_account),
        )
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
    let start = std::time::Instant::now();

    match state
        .http_client
        .post(target_url)
        .json(&payload)
        .send()
        .await
    {
        Ok(resp) => {
            let elapsed = start.elapsed().as_secs_f64();
            metrics::UPSTREAM_LATENCY_SECONDS
                .with_label_values(&["strategy-generator"])
                .observe(elapsed);
            metrics::UPSTREAM_HEALTH
                .with_label_values(&["strategy-generator"])
                .set(1);
            let status = resp.status();
            let body = resp.text().await.unwrap_or("".to_string());
            (status, body).into_response()
        }
        Err(e) => {
            let elapsed = start.elapsed().as_secs_f64();
            metrics::UPSTREAM_LATENCY_SECONDS
                .with_label_values(&["strategy-generator"])
                .observe(elapsed);
            metrics::UPSTREAM_HEALTH
                .with_label_values(&["strategy-generator"])
                .set(0);
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
    let start = std::time::Instant::now();
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
            let elapsed = start.elapsed().as_secs_f64();
            metrics::UPSTREAM_LATENCY_SECONDS
                .with_label_values(&["data-engine"])
                .observe(elapsed);
            metrics::UPSTREAM_HEALTH
                .with_label_values(&["data-engine"])
                .set(1);
            response
        }
        Err(e) => {
            let elapsed = start.elapsed().as_secs_f64();
            metrics::UPSTREAM_LATENCY_SECONDS
                .with_label_values(&["data-engine"])
                .observe(elapsed);
            metrics::UPSTREAM_HEALTH
                .with_label_values(&["data-engine"])
                .set(0);
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
    let start = std::time::Instant::now();
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
            let elapsed = start.elapsed().as_secs_f64();
            metrics::UPSTREAM_LATENCY_SECONDS
                .with_label_values(&["data-engine"])
                .observe(elapsed);
            metrics::UPSTREAM_HEALTH
                .with_label_values(&["data-engine"])
                .set(1);
            response
        }
        Err(e) => {
            let elapsed = start.elapsed().as_secs_f64();
            metrics::UPSTREAM_LATENCY_SECONDS
                .with_label_values(&["data-engine"])
                .observe(elapsed);
            metrics::UPSTREAM_HEALTH
                .with_label_values(&["data-engine"])
                .set(0);
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
    let start = std::time::Instant::now();
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
            let elapsed = start.elapsed().as_secs_f64();
            metrics::UPSTREAM_LATENCY_SECONDS
                .with_label_values(&["data-engine"])
                .observe(elapsed);
            metrics::UPSTREAM_HEALTH
                .with_label_values(&["data-engine"])
                .set(1);
            response
        }
        Err(e) => {
            let elapsed = start.elapsed().as_secs_f64();
            metrics::UPSTREAM_LATENCY_SECONDS
                .with_label_values(&["data-engine"])
                .observe(elapsed);
            metrics::UPSTREAM_HEALTH
                .with_label_values(&["data-engine"])
                .set(0);
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

async fn evolution_proxy(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(path): axum::extract::Path<String>,
    req: axum::extract::Request,
) -> impl IntoResponse {
    let start = std::time::Instant::now();
    let (parts, _body) = req.into_parts();
    let query_string = parts
        .uri
        .query()
        .map(|q| format!("?{}", q))
        .unwrap_or_default();
    let target_url = format!("http://strategy-generator:8082/{}{}", path, query_string);

    match state.http_client.get(&target_url).send().await {
        Ok(resp) => {
            let status = resp.status();
            let headers = resp.headers().clone();
            let body = resp.bytes().await.unwrap_or_default();
            let mut response = (status, body).into_response();
            *response.headers_mut() = headers;
            let elapsed = start.elapsed().as_secs_f64();
            metrics::UPSTREAM_LATENCY_SECONDS
                .with_label_values(&["strategy-generator"])
                .observe(elapsed);
            metrics::UPSTREAM_HEALTH
                .with_label_values(&["strategy-generator"])
                .set(1);
            response
        }
        Err(e) => {
            let elapsed = start.elapsed().as_secs_f64();
            metrics::UPSTREAM_LATENCY_SECONDS
                .with_label_values(&["strategy-generator"])
                .observe(elapsed);
            metrics::UPSTREAM_HEALTH
                .with_label_values(&["strategy-generator"])
                .set(0);
            error!("Failed to proxy to strategy-generator: {}", e);
            metrics::PROXY_ERRORS_TOTAL
                .with_label_values(&["strategy-generator"])
                .inc();
            (
                axum::http::StatusCode::BAD_GATEWAY,
                json!({"error": "Strategy generator unavailable"}).to_string(),
            )
                .into_response()
        }
    }
}

// --- Trade Query Types ---

#[derive(Deserialize)]
struct TradeHistoryQuery {
    mode: Option<String>,
    symbol: Option<String>,
    status: Option<String>,
    limit: Option<i64>,
}

// --- Trade Query Endpoints ---

async fn get_trade_history(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TradeHistoryQuery>,
) -> Json<Value> {
    use sqlx::Row;
    let limit = params.limit.unwrap_or(100).min(500);

    let mut query = String::from(
        "SELECT order_id, symbol, side, quantity, filled_qty, avg_price, status, strategy_id, mode, account_id, created_at FROM trade_orders WHERE 1=1"
    );
    let mut bind_idx = 0u32;
    let mut binds_mode = None;
    let mut binds_symbol = None;
    let mut binds_status = None;

    if let Some(ref mode) = params.mode {
        bind_idx += 1;
        query.push_str(&format!(" AND mode = ${}", bind_idx));
        binds_mode = Some(mode.clone());
    }
    if let Some(ref symbol) = params.symbol {
        bind_idx += 1;
        query.push_str(&format!(" AND symbol = ${}", bind_idx));
        binds_symbol = Some(symbol.clone());
    }
    if let Some(ref status) = params.status {
        bind_idx += 1;
        query.push_str(&format!(" AND status = ${}", bind_idx));
        binds_status = Some(status.clone());
    }

    query.push_str(&format!(" ORDER BY created_at DESC LIMIT {}", limit));

    let mut q = sqlx::query(&query);
    if let Some(ref m) = binds_mode {
        q = q.bind(m);
    }
    if let Some(ref s) = binds_symbol {
        q = q.bind(s);
    }
    if let Some(ref st) = binds_status {
        q = q.bind(st);
    }

    match q.fetch_all(&state.pg_pool).await {
        Ok(rows) => {
            let results: Vec<Value> = rows
                .iter()
                .map(|row| {
                    json!({
                        "order_id": row.get::<String, _>("order_id"),
                        "symbol": row.get::<String, _>("symbol"),
                        "side": row.get::<String, _>("side"),
                        "quantity": row.get::<sqlx::types::Decimal, _>("quantity").to_string(),
                        "filled_qty": row.get::<Option<sqlx::types::Decimal>, _>("filled_qty").map(|v| v.to_string()),
                        "avg_price": row.get::<Option<sqlx::types::Decimal>, _>("avg_price").map(|v| v.to_string()),
                        "status": row.get::<String, _>("status"),
                        "strategy_id": row.get::<Option<String>, _>("strategy_id"),
                        "mode": row.get::<Option<String>, _>("mode"),
                        "account_id": row.get::<Option<String>, _>("account_id"),
                        "created_at": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("created_at").map(|t| t.to_rfc3339()),
                    })
                })
                .collect();
            Json(json!(results))
        }
        Err(e) => {
            error!("Failed to fetch trade history: {}", e);
            Json(json!({"error": "Failed to fetch trade history"}))
        }
    }
}

async fn get_trade_positions(State(state): State<Arc<AppState>>) -> Json<Value> {
    use sqlx::Row;
    let query = "\
        SELECT p.account_id, p.exchange, p.symbol, p.quantity, p.avg_price, \
               p.current_price, p.unrealized_pnl, p.updated_at, \
               latest.close AS last_price, latest.time AS price_time \
        FROM trade_positions p \
        LEFT JOIN LATERAL ( \
            SELECT close, time FROM mkt_equity_candles \
            WHERE symbol = p.symbol AND exchange = 'Polygon' \
            ORDER BY time DESC LIMIT 1 \
        ) latest ON true \
        WHERE p.quantity != 0 \
        ORDER BY p.account_id, p.symbol";

    match sqlx::query(query).fetch_all(&state.pg_pool).await {
        Ok(rows) => {
            let results: Vec<Value> = rows
                .iter()
                .map(|row| {
                    let current_price = row
                        .get::<Option<sqlx::types::Decimal>, _>("current_price")
                        .or_else(|| row.get::<Option<sqlx::types::Decimal>, _>("last_price"));
                    let qty = row.get::<sqlx::types::Decimal, _>("quantity");
                    let avg = row.get::<sqlx::types::Decimal, _>("avg_price");
                    let abs_qty = if qty < sqlx::types::Decimal::ZERO { -qty } else { qty };
                    let cost_basis = abs_qty * avg;
                    let market_value = current_price.map(|cp| abs_qty * cp);
                    let unrealized_pnl = row
                        .get::<Option<sqlx::types::Decimal>, _>("unrealized_pnl")
                        .or_else(|| market_value.map(|mv| mv - qty * avg));
                    json!({
                        "account_id": row.get::<String, _>("account_id"),
                        "exchange": row.get::<String, _>("exchange"),
                        "symbol": row.get::<String, _>("symbol"),
                        "quantity": qty.to_string(),
                        "avg_price": avg.to_string(),
                        "current_price": current_price.map(|v| v.to_string()),
                        "cost_basis": cost_basis.to_string(),
                        "market_value": market_value.map(|v| v.to_string()),
                        "unrealized_pnl": unrealized_pnl.map(|v| v.to_string()),
                        "updated_at": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("updated_at").map(|t| t.to_rfc3339()),
                        "price_time": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("price_time").map(|t| t.to_rfc3339()),
                    })
                })
                .collect();
            Json(json!(results))
        }
        Err(e) => {
            error!("Failed to fetch trade positions: {}", e);
            Json(json!({"error": "Failed to fetch trade positions"}))
        }
    }
}

async fn get_trade_strategy(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(strategy_id): axum::extract::Path<String>,
) -> Json<Value> {
    use sqlx::Row;

    // Fetch orders for this strategy
    let orders_query = "SELECT order_id, symbol, side, quantity, filled_qty, avg_price, status, mode, account_id, created_at FROM trade_orders WHERE strategy_id = $1 ORDER BY created_at DESC";

    let orders = match sqlx::query(orders_query)
        .bind(&strategy_id)
        .fetch_all(&state.pg_pool)
        .await
    {
        Ok(rows) => rows
            .iter()
            .map(|row| {
                json!({
                    "order_id": row.get::<String, _>("order_id"),
                    "symbol": row.get::<String, _>("symbol"),
                    "side": row.get::<String, _>("side"),
                    "quantity": row.get::<sqlx::types::Decimal, _>("quantity").to_string(),
                    "filled_qty": row.get::<Option<sqlx::types::Decimal>, _>("filled_qty").map(|v| v.to_string()),
                    "avg_price": row.get::<Option<sqlx::types::Decimal>, _>("avg_price").map(|v| v.to_string()),
                    "status": row.get::<String, _>("status"),
                    "mode": row.get::<Option<String>, _>("mode"),
                    "account_id": row.get::<Option<String>, _>("account_id"),
                    "created_at": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("created_at").map(|t| t.to_rfc3339()),
                })
            })
            .collect::<Vec<Value>>(),
        Err(e) => {
            error!("Failed to fetch orders for strategy {}: {}", strategy_id, e);
            return Json(json!({"error": "Failed to fetch strategy orders"}));
        }
    };

    // Fetch strategy generation metadata
    let gen_query = "SELECT exchange, symbol, mode, generation, fitness, metadata, timestamp FROM strategy_generations WHERE strategy_id = $1 LIMIT 1";

    let generation = match sqlx::query(gen_query)
        .bind(&strategy_id)
        .fetch_optional(&state.pg_pool)
        .await
    {
        Ok(Some(row)) => {
            json!({
                "exchange": row.get::<String, _>("exchange"),
                "symbol": row.get::<String, _>("symbol"),
                "mode": row.get::<String, _>("mode"),
                "generation": row.get::<i32, _>("generation"),
                "fitness": row.get::<f64, _>("fitness"),
                "metadata": row.get::<Value, _>("metadata"),
                "timestamp": row.get::<chrono::DateTime<chrono::Utc>, _>("timestamp").to_rfc3339(),
            })
        }
        Ok(None) => Value::Null,
        Err(e) => {
            error!(
                "Failed to fetch generation for strategy {}: {}",
                strategy_id, e
            );
            Value::Null
        }
    };

    Json(json!({
        "strategy_id": strategy_id,
        "orders": orders,
        "generation": generation,
    }))
}

// --- Trading Account Endpoints ---

async fn get_trading_accounts(State(state): State<Arc<AppState>>) -> Json<Value> {
    use sqlx::Row;
    let query = "SELECT account_id, label, broker, broker_account, mode, is_enabled, \
                 max_order_value, max_positions, max_daily_loss, updated_at \
                 FROM trading_accounts ORDER BY account_id";

    match sqlx::query(query).fetch_all(&state.pg_pool).await {
        Ok(rows) => {
            let results: Vec<Value> = rows
                .iter()
                .map(|row| {
                    json!({
                        "account_id": row.get::<String, _>("account_id"),
                        "label": row.get::<String, _>("label"),
                        "broker": row.get::<String, _>("broker"),
                        "broker_account": row.get::<Option<String>, _>("broker_account"),
                        "mode": row.get::<String, _>("mode"),
                        "is_enabled": row.get::<bool, _>("is_enabled"),
                        "max_order_value": row.get::<sqlx::types::Decimal, _>("max_order_value").to_string(),
                        "max_positions": row.get::<i32, _>("max_positions"),
                        "max_daily_loss": row.get::<sqlx::types::Decimal, _>("max_daily_loss").to_string(),
                        "updated_at": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("updated_at").map(|t| t.to_rfc3339()),
                    })
                })
                .collect();
            Json(json!(results))
        }
        Err(e) => {
            error!("Failed to fetch trading accounts: {}", e);
            Json(json!({"error": "Failed to fetch trading accounts"}))
        }
    }
}

#[derive(Deserialize)]
struct UpdateAccountBody {
    label: Option<String>,
    broker_account: Option<String>,
    is_enabled: Option<bool>,
    max_order_value: Option<f64>,
    max_positions: Option<i32>,
    max_daily_loss: Option<f64>,
}

async fn update_trading_account(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(account_id): axum::extract::Path<String>,
    Json(body): Json<UpdateAccountBody>,
) -> Json<Value> {
    let mut set_clauses = Vec::new();
    let mut bind_idx = 1u32;

    if body.label.is_some() {
        bind_idx += 1;
        set_clauses.push(format!("label = ${}", bind_idx));
    }
    if body.broker_account.is_some() {
        bind_idx += 1;
        set_clauses.push(format!("broker_account = ${}", bind_idx));
    }
    if body.is_enabled.is_some() {
        bind_idx += 1;
        set_clauses.push(format!("is_enabled = ${}", bind_idx));
    }
    if body.max_order_value.is_some() {
        bind_idx += 1;
        set_clauses.push(format!("max_order_value = ${}", bind_idx));
    }
    if body.max_positions.is_some() {
        bind_idx += 1;
        set_clauses.push(format!("max_positions = ${}", bind_idx));
    }
    if body.max_daily_loss.is_some() {
        bind_idx += 1;
        set_clauses.push(format!("max_daily_loss = ${}", bind_idx));
    }

    if set_clauses.is_empty() {
        return Json(json!({"error": "No fields to update"}));
    }

    set_clauses.push("updated_at = NOW()".to_string());
    let query = format!(
        "UPDATE trading_accounts SET {} WHERE account_id = $1",
        set_clauses.join(", ")
    );

    // Build the query with dynamic binds
    let mut q = sqlx::query(&query).bind(&account_id);

    if let Some(ref label) = body.label {
        q = q.bind(label);
    }
    if let Some(ref broker_account) = body.broker_account {
        q = q.bind(broker_account);
    }
    if let Some(is_enabled) = body.is_enabled {
        q = q.bind(is_enabled);
    }
    if let Some(max_order_value) = body.max_order_value {
        q = q.bind(sqlx::types::Decimal::try_from(max_order_value).unwrap_or_default());
    }
    if let Some(max_positions) = body.max_positions {
        q = q.bind(max_positions);
    }
    if let Some(max_daily_loss) = body.max_daily_loss {
        q = q.bind(sqlx::types::Decimal::try_from(max_daily_loss).unwrap_or_default());
    }

    match q.execute(&state.pg_pool).await {
        Ok(result) => {
            if result.rows_affected() == 0 {
                Json(json!({"error": "Account not found"}))
            } else {
                Json(json!({"status": "ok", "account_id": account_id}))
            }
        }
        Err(e) => {
            error!("Failed to update trading account {}: {}", account_id, e);
            Json(json!({"error": "Failed to update trading account"}))
        }
    }
}

async fn get_account_summary(State(state): State<Arc<AppState>>) -> Json<Value> {
    use sqlx::Row;
    // Uses cached IBKR values (cached_cash, cached_net_liq) when available,
    // falling back to computed values from initial_capital + trade_executions.
    let query = "\
        WITH trade_cash AS ( \
            SELECT o.account_id, \
                   SUM(COALESCE(e.commission, 0)) as total_commissions, \
                   COUNT(DISTINCT e.execution_id)::INTEGER as total_trades \
            FROM trade_executions e \
            JOIN trade_orders o ON e.order_id = o.order_id \
            WHERE o.account_id IS NOT NULL \
            GROUP BY o.account_id \
        ), \
        position_stats AS ( \
            SELECT p.account_id, \
                   COUNT(*)::INTEGER as position_count, \
                   SUM(ABS(p.quantity) * p.avg_price) as total_cost_basis, \
                   SUM(ABS(p.quantity) * COALESCE(c.close, p.current_price, p.avg_price)) as total_market_value, \
                   SUM(p.quantity * (COALESCE(c.close, p.current_price, p.avg_price) - p.avg_price)) as unrealized_pnl \
            FROM trade_positions p \
            LEFT JOIN LATERAL ( \
                SELECT close FROM mkt_equity_candles \
                WHERE symbol = p.symbol AND exchange = 'Polygon' \
                ORDER BY time DESC LIMIT 1 \
            ) c ON true \
            WHERE p.quantity != 0 \
            GROUP BY p.account_id \
        ), \
        realized_pnl AS ( \
            SELECT o.account_id, \
                   SUM(CASE WHEN o.side = 'Sell' THEN e.filled_qty * e.avg_price \
                            ELSE -e.filled_qty * e.avg_price END) - SUM(COALESCE(e.commission, 0)) as realized_pnl \
            FROM trade_executions e \
            JOIN trade_orders o ON e.order_id = o.order_id \
            WHERE o.account_id IS NOT NULL AND o.status = 'Filled' \
            GROUP BY o.account_id \
        ) \
        SELECT ta.account_id, ta.label, ta.broker_account, ta.mode, ta.is_enabled, \
               ta.max_order_value, ta.max_positions, ta.max_daily_loss, \
               ta.initial_capital, \
               COALESCE(ps.position_count, 0) as position_count, \
               COALESCE(ps.total_cost_basis, 0) as total_cost_basis, \
               COALESCE(ps.total_market_value, 0) as total_market_value, \
               COALESCE(ps.unrealized_pnl, 0) as unrealized_pnl, \
               COALESCE(ta.cached_cash, 0) as cash_balance, \
               COALESCE(ta.cached_net_liq, 0) as net_liquidation, \
               COALESCE(ta.cached_buying_power, 0) as buying_power, \
               ta.cache_updated_at, \
               COALESCE(tc.total_commissions, 0) as total_commissions, \
               COALESCE(tc.total_trades, 0) as total_trades, \
               COALESCE(rp.realized_pnl, 0) as realized_pnl \
        FROM trading_accounts ta \
        LEFT JOIN trade_cash tc ON tc.account_id = ta.account_id \
        LEFT JOIN position_stats ps ON ps.account_id = ta.account_id \
        LEFT JOIN realized_pnl rp ON rp.account_id = ta.account_id \
        ORDER BY ta.account_id";

    match sqlx::query(query).fetch_all(&state.pg_pool).await {
        Ok(rows) => {
            let results: Vec<Value> = rows
                .iter()
                .map(|row| {
                    json!({
                        "account_id": row.get::<String, _>("account_id"),
                        "label": row.get::<String, _>("label"),
                        "broker_account": row.get::<Option<String>, _>("broker_account"),
                        "mode": row.get::<String, _>("mode"),
                        "is_enabled": row.get::<bool, _>("is_enabled"),
                        "max_order_value": row.get::<sqlx::types::Decimal, _>("max_order_value").to_string(),
                        "max_positions": row.get::<i32, _>("max_positions"),
                        "max_daily_loss": row.get::<sqlx::types::Decimal, _>("max_daily_loss").to_string(),
                        "initial_capital": row.get::<sqlx::types::Decimal, _>("initial_capital").to_string(),
                        "position_count": row.get::<i32, _>("position_count"),
                        "total_cost_basis": row.get::<sqlx::types::Decimal, _>("total_cost_basis").to_string(),
                        "total_market_value": row.get::<sqlx::types::Decimal, _>("total_market_value").to_string(),
                        "unrealized_pnl": row.get::<sqlx::types::Decimal, _>("unrealized_pnl").to_string(),
                        "cash_balance": row.get::<sqlx::types::Decimal, _>("cash_balance").to_string(),
                        "net_liquidation": row.get::<sqlx::types::Decimal, _>("net_liquidation").to_string(),
                        "buying_power": row.get::<sqlx::types::Decimal, _>("buying_power").to_string(),
                        "cache_updated_at": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("cache_updated_at").map(|t| t.to_rfc3339()),
                        "total_commissions": row.get::<sqlx::types::Decimal, _>("total_commissions").to_string(),
                        "total_trades": row.get::<i32, _>("total_trades"),
                        "realized_pnl": row.get::<sqlx::types::Decimal, _>("realized_pnl").to_string(),
                    })
                })
                .collect();
            Json(json!(results))
        }
        Err(e) => {
            error!("Failed to fetch account summary: {}", e);
            Json(json!({"error": "Failed to fetch account summary"}))
        }
    }
}
