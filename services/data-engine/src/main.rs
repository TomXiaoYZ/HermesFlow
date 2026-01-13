use data_engine::{
    collectors::{PolymarketCollector, TwitterCollector, IBKRCollector, AkShareCollector},
    trading::ibkr_trader::IBKRTrader,
    config::{AppConfig, LoggingConfig},
    monitoring::{init_metrics, logging::init_logging, HealthMonitor},
    server::{create_router, AppState},
    storage::{ClickHouseWriter, RedisCache},
    // traits::DataSourceConnector, // Not strictly needed if we iterate concrete types, but helpful
    repository::{
        postgres::PostgresRepositories,
        MarketDataRepository, SocialRepository, // Traits
    },
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize standard logging first (with defaults)
    init_logging(&LoggingConfig::default());
    tracing::info!("Data Engine starting (Repository Refactor)...");

    // Load configuration
    let config_result = AppConfig::load();
    let config = match &config_result {
        Ok(c) => {
            tracing::info!("Loaded Config: {:?}", c);
            c.clone()
        },
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
            tracing::warn!("Failed to connect to Redis: {}. Continuing without Redis.", e);
            None
        }
    };

    // Initialize ClickHouse
    let clickhouse = match ClickHouseWriter::new(
        &config.clickhouse.url,
        &config.clickhouse.database,
        config.clickhouse.batch_size,
        config.clickhouse.flush_interval_ms,
    ) {
        Ok(c) => Some(Arc::new(RwLock::new(c))),
        Err(e) => {
            tracing::warn!("Failed to connect to ClickHouse: {}. Continuing without ClickHouse.", e);
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
    let ibkr_trader = if let Some(ibkr_config) = config.ibkr.clone() {
        if ibkr_config.enabled {
            match IBKRTrader::new(&ibkr_config).await {
                Ok(trader) => {
                    tracing::info!("IBKR Trader initialized successfully");
                    Some(trader)
                }
                Err(e) => {
                    tracing::error!("Failed to initialize IBKR Trader: {}", e);
                    None
                }
            }
        } else {
            None
        }
    } else {
        None
    };

    // Create application state
    let app_state = AppState::new(
        config.clone(), 
        redis, 
        clickhouse, 
        postgres_repos.clone(), 
        health_monitor, 
        ibkr_trader
    );

    // Create broadcast channel for shutdown signal
    let (shutdown_tx, _) = tokio::sync::broadcast::channel::<()>(1);

    // Start Twitter collector if configured
    if let Some(twitter_config) = config.twitter.clone() {
        tracing::info!("Starting Twitter collector");
        let twitter_cfg = twitter_config.clone();
        let twitter_collector = Arc::new(TwitterCollector::new(twitter_config));
        let shutdown_rx = shutdown_tx.subscribe();
        let repos = Arc::clone(&postgres_repos);
        
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_secs(twitter_cfg.poll_interval_secs));
            let mut shutdown = shutdown_rx;
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let mut targets = twitter_cfg.targets.clone();
                        if targets.is_empty() && !twitter_cfg.username.is_empty() {
                            targets.push(twitter_cfg.username.clone());
                        }

                        let max_tweets = twitter_cfg.max_tweets_per_session.min(200);

                        // Run timeline scrapes
                        for target in targets {
                            let job = format!("user:{}", target);
                            match twitter_collector.scrape_user_timeline(&target, max_tweets as i32).await {
                                Ok(tweets) => {
                                    let scraped = tweets.len() as i32;
                                    let mut upserted = 0i32;

                                    for t in tweets {
                                        let res = repos.social.insert_tweet(&t).await;
                                        if res.is_ok() { upserted += 1; }
                                    }

                                    let _ = repos.social
                                        .insert_collection_run(&job, scraped, upserted, None)
                                        .await;
                                }
                                Err(e) => {
                                    tracing::warn!(target = %target, err = %e, "Twitter user scrape failed");
                                    let _ = repos.social
                                        .insert_collection_run(&job, 0, 0, Some(&e.to_string()))
                                        .await;
                                }
                            }
                        }
                    }
                    _ = shutdown.recv() => break,
                }
            }
        });
    }

    // Start Polymarket collector (if any) - simplified for brevity, check original code if needed
    if let Some(polymarket_config) = config.polymarket.clone() {
        let pc = Arc::new(PolymarketCollector::new(polymarket_config));
        let s_rx = shutdown_tx.subscribe();
        tokio::spawn(async move { let _ = pc.start(s_rx).await; });
    }

    // Start IBKR collector if configured
    if let Some(ibkr_config) = config.ibkr.clone() {
        if ibkr_config.enabled {
            tracing::info!("Starting IBKR collector");
            // Inject MarketDataRepository (coerced from Arc<PostgresMarketDataRepository>)
            let mut ibkr_collector = IBKRCollector::new(ibkr_config, repos.market_data.clone());
            let mut shutdown_rx = shutdown_tx.subscribe();
            let repos = Arc::clone(&postgres_repos);

            tokio::spawn(async move {
                loop {
                    match ibkr_collector.connect().await {
                        Ok(mut rx) => {
                            tracing::info!("IBKR collector connection established");
                            loop {
                                tokio::select! {
                                    Some(msg) = rx.recv() => {
                                        // Store Generic Snapshot
                                        if let Err(e) = repos.market_data.insert_snapshot(&msg).await {
                                            tracing::warn!("Failed to store IBKR snapshot: {}", e);
                                        }
                                    }
                                    _ = shutdown_rx.recv() => {
                                        let _ = ibkr_collector.disconnect().await;
                                        return;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("IBKR Connect failed: {}. Retrying...", e);
                            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                        }
                    }
                }
            });
        }
    }

    // Start AkShare collector if configured
    if let Some(ak_config) = config.akshare.clone() {
        if ak_config.enabled {
            tracing::info!("Starting AkShare collector");
            let mut ak_collector = AkShareCollector::new(ak_config);
            let mut shutdown_rx = shutdown_tx.subscribe();
            let repos = Arc::clone(&postgres_repos);

            tokio::spawn(async move {
                match ak_collector.connect().await {
                    Ok(mut rx) => {
                        loop {
                            tokio::select! {
                                Some(msg) = rx.recv() => {
                                    if let Err(e) = repos.market_data.insert_snapshot(&msg).await {
                                        tracing::warn!("Failed to store AkShare snapshot: {}", e);
                                    }
                                }
                                _ = shutdown_rx.recv() => break,
                            }
                        }
                    }
                    Err(e) => tracing::error!("AkShare Connect failed: {}", e),
                }
            });
        }
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
        signal::ctrl_c().await.expect("Failed to install Ctrl+C handler");
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
