mod collector_spawner;

use chrono::Utc;
use data_engine::{
    config::{AppConfig, LoggingConfig},
    monitoring::{init_metrics, logging::init_logging, HealthMonitor},
    repository::postgres::PostgresRepositories,
    server::{create_router, routes::AppStateParams, AppState},
    storage::{ClickHouseWriter, RedisCache},
    tasks::TaskManager,
};
use futures::StreamExt;
use redis::Commands;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::RwLock;

use collector_spawner::CollectorDeps;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env
    dotenvy::dotenv().ok();

    // Initialize standard logging first (with defaults)
    init_logging(&LoggingConfig::default());
    tracing::info!("Data Engine starting (Repository Refactor)...");

    // Load configuration
    let config_result = AppConfig::load();
    let config = match &config_result {
        Ok(c) => {
            tracing::info!("Loaded Config: {:?}", c);
            c.clone()
        }
        Err(e) => {
            tracing::error!("CRITICAL: Failed to load configuration: {}", e);
            AppConfig::default()
        }
    };

    // Initialize Prometheus metrics
    if let Err(e) = init_metrics() {
        tracing::error!("Failed to initialize metrics: {}", e);
    }

    // Initialize Redis
    tracing::info!("Connecting to Redis at {}", config.redis.url);
    let redis = match RedisCache::new(&config.redis.url, config.redis.ttl_secs).await {
        Ok(r) => Some(Arc::new(RwLock::new(r))),
        Err(e) => {
            tracing::warn!(
                "Failed to connect to Redis: {}. Continuing without Redis.",
                e
            );
            None
        }
    };

    // Initialize ClickHouse
    let clickhouse = match ClickHouseWriter::new_with_auth(
        &config.clickhouse.url,
        &config.clickhouse.database,
        &config.clickhouse.username,
        &config.clickhouse.password,
        config.clickhouse.batch_size,
        config.clickhouse.flush_interval_ms,
    ) {
        Ok(c) => Some(Arc::new(RwLock::new(c))),
        Err(e) => {
            tracing::warn!(
                "Failed to connect to ClickHouse: {}. Continuing without ClickHouse.",
                e
            );
            None
        }
    };

    if let Some(ch) = &clickhouse {
        let ch_guard = ch.read().await;
        if let Err(e) = ch_guard.create_schema().await {
            tracing::warn!("Failed to create ClickHouse schema: {}. Continuing...", e);
        }
    }

    // Initialize Postgres Repositories
    tracing::info!(
        "Connecting to Postgres at {}:{}/{}",
        config.postgres.host,
        config.postgres.port,
        config.postgres.database
    );
    let postgres_repos = PostgresRepositories::new(&config.postgres).await?;
    let postgres_repos = Arc::new(postgres_repos);

    // Run Postgres Migrations
    tracing::info!("Running Postgres migrations...");
    if let Err(e) = postgres_repos.migration.run_migrations().await {
        tracing::warn!("Failed to run Postgres migrations: {}. Continuing...", e);
    }

    // Initialize health monitor
    let health_monitor = HealthMonitor::new();

    // Initialize IBKR Trader if enabled
    let ibkr_trader = None;

    // Initialize Task Manager
    tracing::info!("Initializing Task Manager...");
    let task_manager = match TaskManager::new(config.clone(), postgres_repos.clone()).await {
        Ok(tm) => {
            if let Err(e) = tm.start().await {
                tracing::warn!("Failed to start Task Scheduler: {}", e);
            }
            if let Err(e) = tm.register_eod_job().await {
                tracing::warn!("Failed to register EOD job: {}", e);
            }
            if let Err(e) = tm.register_token_discovery_job().await {
                tracing::warn!("Failed to register token discovery job: {}", e);
            }
            if let Err(e) = tm.register_data_quality_job().await {
                tracing::warn!("Failed to register data quality job: {}", e);
            }
            if let Err(e) = tm.register_candle_aggregation_job().await {
                tracing::warn!("Failed to register candle aggregation job: {}", e);
            }
            if let Err(e) = tm.register_polymarket_job().await {
                tracing::warn!("Failed to register Polymarket job: {}", e);
            }
            Some(tm)
        }
        Err(e) => {
            tracing::error!("Failed to create Task Manager: {}", e);
            None
        }
    };

    // Create broadcast channel for WebSocket
    let (broadcast_tx, _) = tokio::sync::broadcast::channel(100);

    // Create application state
    let app_state = AppState::new(AppStateParams {
        config: config.clone(),
        redis: redis.clone(),
        clickhouse,
        postgres: postgres_repos.clone(),
        health_monitor,
        ibkr_trader,
        task_manager,
        broadcast_tx: broadcast_tx.clone(),
    });

    // Start Metrics Updater (Background Task)
    data_engine::server::handlers::spawn_metrics_updater(app_state.clone()).await;

    // Start Portfolio Update Listener (Redis Subs)
    if let Some(_r) = &redis {
        tracing::info!("Starting Portfolio Update Listener...");
        let redis_url = config.redis.url.clone();
        let tx_clone = broadcast_tx.clone();

        tokio::spawn(async move {
            let client = redis::Client::open(redis_url).expect("Invalid Redis URL");
            let con = client
                .get_async_connection()
                .await
                .expect("Failed to connect to Redis PubSub");
            let mut pubsub = con.into_pubsub();

            if let Err(e) = pubsub.subscribe("portfolio_updates").await {
                tracing::error!("Failed to subscribe to portfolio_updates: {}", e);
            }
            if let Err(e) = pubsub.subscribe("strategy_logs").await {
                tracing::error!("Failed to subscribe to strategy_logs: {}", e);
            }

            let mut stream = pubsub.on_message();

            while let Some(msg) = stream.next().await {
                if let Ok(payload) = msg.get_payload::<String>() {
                    tracing::info!("Received Redis Msg (Forwarding to WS): {}", payload);
                    let _ = tx_clone.send(payload);
                }
            }
        });
    }

    // Start Heartbeat Task
    tracing::info!("Starting Heartbeat Task...");
    let redis_url_hb = config.redis.url.clone();
    tokio::spawn(async move {
        if let Ok(client) = redis::Client::open(redis_url_hb) {
            if let Ok(mut con) = client.get_connection() {
                loop {
                    let hb = serde_json::json!({
                        "service": "data-engine",
                        "status": "online",
                        "timestamp": Utc::now().timestamp_millis()
                    });
                    let _: redis::RedisResult<()> = con.publish("system_heartbeat", hb.to_string());
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            } else {
                tracing::error!("Failed to connect to Redis for Heartbeat");
            }
        }
    });

    // Create broadcast channel for shutdown signal
    let (shutdown_tx, _) = tokio::sync::broadcast::channel::<()>(1);

    // Spawn all market data collectors
    let collector_deps = CollectorDeps {
        config: config.clone(),
        postgres_repos: postgres_repos.clone(),
        redis: redis.clone(),
        broadcast_tx: broadcast_tx.clone(),
        shutdown_tx: shutdown_tx.clone(),
    };
    collector_spawner::spawn_all_collectors(&collector_deps).await;

    // Start Health Check Loop
    {
        let health_monitor = app_state.health_monitor.clone();
        let redis = app_state.redis.clone();
        let clickhouse = app_state.clickhouse.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
            loop {
                interval.tick().await;

                if let Some(r) = &redis {
                    let mut r_guard = r.write().await;
                    health_monitor.check_redis(&mut r_guard).await;
                }

                if let Some(ch) = &clickhouse {
                    let ch_guard = ch.read().await;
                    health_monitor.check_clickhouse(&ch_guard).await;
                }
            }
        });
    }

    // Create router
    let app = create_router(app_state);

    // Bind server
    let addr = SocketAddr::from((
        config.server.host.parse::<std::net::IpAddr>()?,
        config.server.port,
    ));

    tracing::info!("Data Engine listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal_with_broadcast(shutdown_tx))
        .await?;

    tracing::info!("Data Engine shut down gracefully");
    Ok(())
}

async fn shutdown_signal_with_broadcast(shutdown_tx: tokio::sync::broadcast::Sender<()>) {
    shutdown_signal().await;
    let _ = shutdown_tx.send(());
}

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
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
