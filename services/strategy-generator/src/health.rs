use axum::{Json, routing::get, Router};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tracing::info;

async fn health() -> Json<Value> {
    Json(json!({
        "service": "strategy-generator",
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

pub async fn start_health_server() {
    let app = Router::new().route("/health", get(health));
    
    let addr = SocketAddr::from(([0, 0, 0, 0], 8084));
    info!("Strategy Generator health endpoint listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.expect("Health server failed");
}
