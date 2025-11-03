use data_engine::{
    config::AppConfig,
    monitoring::{init_metrics, logging::init_logging, HealthMonitor},
    server::{create_router, AppState},
    storage::{ClickHouseWriter, RedisCache},
};
use std::net::SocketAddr;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = AppConfig::load().unwrap_or_else(|e| {
        eprintln!("Failed to load configuration: {}", e);
        eprintln!("Using default configuration");
        AppConfig {
            server: Default::default(),
            redis: Default::default(),
            clickhouse: Default::default(),
            data_sources: vec![],
            performance: Default::default(),
            logging: Default::default(),
        }
    });

    // Initialize logging
    init_logging(&config.logging);
    tracing::info!("Data Engine starting...");
    tracing::info!("Version: {}", env!("CARGO_PKG_VERSION"));

    // Initialize Prometheus metrics
    if let Err(e) = init_metrics() {
        tracing::error!("Failed to initialize metrics: {}", e);
    }

    // Initialize Redis
    tracing::info!("Connecting to Redis at {}", config.redis.url);
    let redis = RedisCache::new(&config.redis.url, config.redis.ttl_secs)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(
                "Failed to connect to Redis: {}. Continuing without Redis.",
                e
            );
            // In production, you might want to fail here
            // For framework-only mode, we continue
            panic!("Redis connection required");
        });

    // Initialize ClickHouse
    tracing::info!(
        "Connecting to ClickHouse at {} (database: {})",
        config.clickhouse.url,
        config.clickhouse.database
    );
    let clickhouse = ClickHouseWriter::new(
        &config.clickhouse.url,
        &config.clickhouse.database,
        config.clickhouse.batch_size,
        config.clickhouse.flush_interval_ms,
    )?;

    // Create ClickHouse schema
    tracing::info!("Creating ClickHouse schema if not exists...");
    if let Err(e) = clickhouse.create_schema().await {
        tracing::warn!("Failed to create ClickHouse schema: {}. Continuing...", e);
    }

    // Initialize health monitor
    let health_monitor = HealthMonitor::new();
    tracing::info!("Health monitor initialized");

    // Create application state
    let app_state = AppState::new(config.clone(), redis, clickhouse, health_monitor);

    // Create router
    let app = create_router(app_state);

    // Bind server
    let addr = SocketAddr::from((
        config.server.host.parse::<std::net::IpAddr>()?,
        config.server.port,
    ));

    tracing::info!("Data Engine listening on {}", addr);
    println!(
        "🚀 Data Engine v{} listening on {}",
        env!("CARGO_PKG_VERSION"),
        addr
    );
    println!("📊 Metrics: http://{}/metrics", addr);
    println!("💚 Health: http://{}/health", addr);
    println!("📈 API: http://{}/api/v1/market/{{symbol}}/latest", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Run server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Data Engine shut down gracefully");
    Ok(())
}

/// Graceful shutdown handler
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, shutting down...");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM, shutting down...");
        },
    }
}
