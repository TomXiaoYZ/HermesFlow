use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State, Query},
    response::IntoResponse,
    routing::get,
    Router,
    Json,
};
use clickhouse::Client as ClickHouseClient;
use futures::stream::StreamExt;
use redis::AsyncCommands;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::broadcast;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tower_http::cors::CorsLayer;

mod health_checker;
mod auth_handler;


struct AppState {
    tx: broadcast::Sender<String>,
    ch_client: ClickHouseClient,
    http_client: reqwest::Client,
    user_management_url: String,
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

    // Redis URL
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    // ClickHouse URL
    let ch_url = std::env::var("CLICKHOUSE_URL").unwrap_or_else(|_| "http://localhost:8123".to_string());
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
    // Determine internal/external connection (simple hack: try to connect to check health? No, just trust env)
    
    let app_state = Arc::new(AppState { 
        tx: tx.clone(),
        ch_client,
        http_client: reqwest::Client::new(),
        user_management_url: std::env::var("USER_MANAGEMENT_URL").unwrap_or_else(|_| "http://user-management:8086".to_string()),
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

        let mut con = match client.get_async_connection().await {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to connect to Redis: {}", e);
                return;
            }
        };

        let mut pubsub = con.into_pubsub();
        // Subscribe to relevant channels
        let channels = ["trade_signals", "portfolio_updates", "market_data", "strategy_logs", "system_metrics"];
        for ch in channels {
             if let Err(e) = pubsub.subscribe(ch).await {
                 error!("Failed to subscribe to {}: {}", ch, e);
             }
        }

        info!("Redis PubSub Active. Forwarding messages to WebSocket clients.");

        let _ = pubsub.subscribe("system_heartbeat").await;
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
                        _ => "unknown"
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
        .route("/ws", get(ws_handler))
        .route("/api/logs", get(get_logs))
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
        // Simple mapping: "data-engine" -> "hermesflow-data-engine-1" usually, but user might pass partial match
        // Let's use ILIKE for flexibility
        query.push_str(&format!(" AND container_name ILIKE '%{}%'", params.service));
    }
    
    if let Some(level) = &params.level {
        if level != "ALL" {
             query.push_str(&format!(" AND level = '{}'", level));
        }
    }
    
    if let Some(keyword) = &params.keyword {
        // ClickHouse SQL injection risk minimal here as this is internal tool, but bind params better.
        // For simplicity using format! but in prod use bind.
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

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.tx.subscribe();
     // Send initial verify message
    if socket.send(Message::Text(json!({"type": "connect", "status": "ok"}).to_string())).await.is_err() {
        return;
    }

    loop {
        tokio::select! {
            // Send broadcast messages to client
            Ok(msg) = rx.recv() => {
                if socket.send(Message::Text(msg)).await.is_err() {
                    break;
                }
            }
            // Client disconnect or message (we ignore client messages for now)
            else => break,
        }
    }
}
