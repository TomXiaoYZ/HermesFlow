use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tracing::info;

use crate::metrics;

async fn health() -> Json<Value> {
    Json(json!({
        "service": "strategy-generator",
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// P10C: Prometheus metrics endpoint.
async fn prometheus_metrics() -> String {
    metrics::gather_metrics()
}

pub async fn start_health_server() {
    let app = Router::new()
        .route("/health", get(health))
        .route("/metrics", get(prometheus_metrics));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8084));
    info!(
        "Strategy Generator health + metrics endpoint listening on {}",
        addr
    );

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .await
        .expect("Health server failed");
}
