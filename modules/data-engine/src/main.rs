use axum::{routing::get, Router};
use std::net::SocketAddr;

mod health;

/// Main entry point for the Data Engine service
/// Provides health check endpoint for monitoring
#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(health::health_check));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Data Engine listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
