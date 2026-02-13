use axum::{routing::get, Json, Router};
use std::net::SocketAddr;
use tracing::info;

/// Start a lightweight health-check HTTP server.
///
/// Every service calls this with its own name and port to expose `/health`.
/// When the `metrics` feature is enabled, also exposes `/metrics` for Prometheus scraping.
pub async fn start_health_server(service_name: &str, port: u16) {
    let name = service_name.to_owned();
    let handler = move || {
        let name = name.clone();
        async move {
            Json(serde_json::json!({
                "service": name,
                "status": "healthy",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        }
    };

    let mut app = Router::new().route("/health", get(handler));

    #[cfg(feature = "metrics")]
    {
        app = app.route("/metrics", get(crate::metrics::metrics_handler));
    }

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("{} health endpoint listening on {}", service_name, addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .await
        .expect("Health server failed");
}
