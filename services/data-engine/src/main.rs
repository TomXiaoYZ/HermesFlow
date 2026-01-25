use chrono::{TimeZone, Utc};
use common::events::MarketDataUpdate;
use redis::Commands;
use data_engine::{
    collectors::{
        AkShareCollector, BinanceConnector, BybitConnector, FutuConnector, IBKRCollector,
        MassiveConnector, OkxConnector, PolymarketCollector, TwitterCollector,
    },
    config::{AppConfig, LoggingConfig},
    monitoring::{init_metrics, logging::init_logging, HealthMonitor},
    repository::{
        postgres::PostgresRepositories,
        MarketDataRepository,
        SocialRepository, // Traits
    },
    server::{create_router, AppState},
    storage::{ClickHouseWriter, RedisCache},
    tasks::TaskManager,
    trading::ibkr_trader::IBKRTrader,
    traits::DataSourceConnector,
};
use futures::StreamExt;
use rust_decimal::prelude::ToPrimitive;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::RwLock;

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
    let clickhouse = match ClickHouseWriter::new(
        &config.clickhouse.url,
        &config.clickhouse.database,
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
    /*
    if let Some(ibkr_config) = config.ibkr.clone() {
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
    */

    // Initialize Task Manager
    tracing::info!("Initializing Task Manager...");
    let task_manager = match TaskManager::new(config.clone(), postgres_repos.clone()).await {
        Ok(tm) => {
            if let Err(e) = tm.start().await {
                tracing::warn!("Failed to start Task Scheduler: {}", e);
            }
            // Register EOD job
            if let Err(e) = tm.register_eod_job().await {
                tracing::warn!("Failed to register EOD job: {}", e);
            }
            // Register Token Discovery job
            if let Err(e) = tm.register_token_discovery_job().await {
                tracing::warn!("Failed to register token discovery job: {}", e);
            }
            // Register Data Quality job
            if let Err(e) = tm.register_data_quality_job().await {
                tracing::warn!("Failed to register data quality job: {}", e);
            }
            // Register Candle Aggregation job
            if let Err(e) = tm.register_candle_aggregation_job().await {
                tracing::warn!("Failed to register candle aggregation job: {}", e);
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
    let app_state = AppState::new(
        config.clone(),
        redis.clone(),
        clickhouse,
        postgres_repos.clone(),
        health_monitor,
        ibkr_trader,
        task_manager,
        broadcast_tx.clone(),
    );

    // Start Metrics Updater (Background Task)
    data_engine::server::handlers::spawn_metrics_updater(app_state.clone()).await;

    // Start Portfolio Update Listener (Redis Subs)
    if let Some(r) = &redis {
        tracing::info!("Starting Portfolio Update Listener...");
        let redis_url = config.redis.url.clone();
        let tx_clone = broadcast_tx.clone();

        tokio::spawn(async move {
            let client = redis::Client::open(redis_url).expect("Invalid Redis URL");
            let mut con = client
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

            loop {
                match stream.next().await {
                    Some(msg) => {
                        if let Ok(payload) = msg.get_payload::<String>() {
                            // Check channel name if needed, but for now we just forward everything to WS
                            tracing::info!("Received Redis Msg (Forwarding to WS): {}", payload);
                            let _ = tx_clone.send(payload);
                        }
                    }
                    None => break,
                }
            }
        });
    }

            // Start Heartbeat Task
    tracing::info!("Starting Heartbeat Task...");
    let redis_url_hb = config.redis.url.clone();
    tokio::spawn(async move {
        // Simple dedicated connection for heartbeat
        if let Ok(client) = redis::Client::open(redis_url_hb) {
            if let Ok(mut con) = client.get_connection() {
                loop {
                    let hb = serde_json::json!({
                        "service": "data-engine",
                        "status": "online",
                        "timestamp": Utc::now().timestamp_millis()
                    });
                    // Fire and forget
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

    // Start Twitter collector if configured
    if let Some(twitter_config) = config.twitter.clone() {
        tracing::info!("Starting Twitter collector");
        let twitter_cfg = twitter_config.clone();
        let twitter_collector = Arc::new(TwitterCollector::new(twitter_config));
        let shutdown_rx = shutdown_tx.subscribe();
        let repos = Arc::clone(&postgres_repos);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
                twitter_cfg.poll_interval_secs,
            ));
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
    // Start Polymarket collector (if any) - simplified for brevity, check original code if needed
    /*
    if let Some(polymarket_config) = config.polymarket.clone() {
        let pc = Arc::new(PolymarketCollector::new(polymarket_config));
        let s_rx = shutdown_tx.subscribe();
        tokio::spawn(async move { let _ = pc.start(s_rx).await; });
    }
    */

    // Start IBKR collector if configured
    /*
    if let Some(ibkr_config) = config.ibkr.clone() {
        if ibkr_config.enabled {
            tracing::info!("Starting IBKR collector");
            // Inject MarketDataRepository (coerced from Arc<PostgresMarketDataRepository>)
            let mut ibkr_collector = IBKRCollector::new(ibkr_config, postgres_repos.market_data.clone());
            let mut shutdown_rx = shutdown_tx.subscribe();
            let repos = Arc::clone(&postgres_repos);

            let redis_publisher = if let Some(r) = &redis {
                Some(r.read().await.clone())
            } else {
                None
            };

            let broadcast_tx_ibkr = broadcast_tx.clone();

            tokio::spawn(async move {
                loop {
                    match ibkr_collector.connect().await {
                        Ok(mut rx) => {
                            tracing::info!("IBKR collector connection established");
                            loop {
                                tokio::select! {
                                    Some(msg) = rx.recv() => {
                                        // 1. Store Generic Snapshot
                                        if let Err(e) = repos.market_data.insert_snapshot(&msg).await {
                                            tracing::warn!("Failed to store IBKR snapshot: {}", e);
                                        }

                                        // 2. Publish to Redis for Strategy Consumer
                                        // Also broadcast internally for WebSocket
                                        if let Some(publisher) = &redis_publisher {
                                            let update = MarketDataUpdate {
                                                symbol: msg.symbol.clone(),
                                                price: msg.price.to_f64().unwrap_or_default(),
                                                volume: msg.quantity.to_f64().unwrap_or_default(),
                                                timestamp: Utc.timestamp_millis_opt(msg.timestamp).unwrap(),
                                                source: "ibkr".to_string(),
                                            };

                                            if let Ok(json) = serde_json::to_string(&update) {
                                                // Send to Redis
                                                if let Err(e) = publisher.publish(&"market_data", &json).await {
                                                     tracing::warn!("Failed to publish IBKR update to Redis: {}", e);
                                                }
                                                // Send to Internal WS Broadcast
                                                let _ = broadcast_tx_ibkr.send(json);
                                            }
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
    */
    // Start Birdeye collector if configured
    if let Some(birdeye_config) = config.birdeye.clone() {
        if birdeye_config.enabled {
            tracing::info!("Starting Birdeye collector");
            use data_engine::collectors::BirdeyeConnector;
            let mut birdeye_collector = BirdeyeConnector::new(birdeye_config);
            let mut shutdown_rx = shutdown_tx.subscribe();
            let repos = Arc::clone(&postgres_repos);

            // Re-use redis publisher logic if possible
            let redis_publisher = if let Some(r) = &redis {
                Some(r.read().await.clone())
            } else {
                None
            };
            let tx_clone = broadcast_tx.clone();

            tokio::spawn(async move {
                loop {
                    match birdeye_collector.connect(repos.token.clone()).await {
                        Ok(mut rx) => {
                            tracing::info!("Birdeye collector connection established");
                            loop {
                                tokio::select! {
                                    Some(msg) = rx.recv() => {
                                         if let Err(e) = repos.market_data.insert_snapshot(&msg).await {
                                             tracing::warn!("Failed to store Birdeye snapshot: {}", e);
                                         }
                                         // Map to Standard Event
                                         let update = MarketDataUpdate {
                                             symbol: msg.symbol.clone(),
                                             price: msg.price.to_f64().unwrap_or_default(),
                                             volume: msg.quantity.to_f64().unwrap_or_default(),
                                             timestamp: Utc.timestamp_millis_opt(msg.timestamp).unwrap(),
                                             source: "birdeye".to_string(),
                                         };

                                         if let Ok(json) = serde_json::to_string(&update) {
                                              // Send to WebSocket
                                              let _ = tx_clone.send(json.clone());

                                              let channel = "market_data";
                                              if let Some(publisher) = &redis_publisher {
                                                  if let Err(e) = publisher.publish(&channel, &json).await {
                                                      tracing::warn!("Failed to publish to Redis: {}", e);
                                                  }
                                              }
                                         }
                                    }
                                    _ = shutdown_rx.recv() => {
                                        let _ = birdeye_collector.disconnect().await;
                                        return;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Birdeye Connect failed: {}. Retrying...", e);
                            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                        }
                    }
                }
            });
        }
    }

    // Start Helius collector if configured
    if let Some(helius_config) = config.helius.clone() {
        if helius_config.enabled {
            tracing::info!("Starting Helius collector");
            use data_engine::collectors::HeliusConnector;
            let helius_collector = HeliusConnector::new(helius_config);
            let mut shutdown_rx = shutdown_tx.subscribe();
            let repos = Arc::clone(&postgres_repos);

            let redis_publisher = if let Some(r) = &redis {
                Some(r.read().await.clone())
            } else {
                None
            };
            let tx_clone = broadcast_tx.clone();

            tokio::spawn(async move {
                loop {
                    match helius_collector.connect().await {
                        Ok(mut rx) => {
                            tracing::info!("Helius collector connection established");
                            loop {
                                tokio::select! {
                                    Some(msg) = rx.recv() => {
                                         // Helius is high frequency, maybe log less or batch insert?
                                         // For now, treat same as others.
                                         if let Err(e) = repos.market_data.insert_snapshot(&msg).await {
                                             tracing::warn!("Failed to store Helius snapshot: {}", e);
                                         }

                                         let update = MarketDataUpdate {
                                             symbol: msg.symbol.clone(),
                                             price: msg.price.to_f64().unwrap_or_default(),
                                             volume: msg.quantity.to_f64().unwrap_or_default(),
                                             timestamp: Utc.timestamp_millis_opt(msg.timestamp).unwrap(),
                                             source: "helius".to_string(),
                                         };

                                         if let Ok(json) = serde_json::to_string(&update) {
                                              let _ = tx_clone.send(json.clone());
                                              let channel = "market_data";
                                              if let Some(publisher) = &redis_publisher {
                                                  if let Err(e) = publisher.publish(&channel, &json).await {
                                                      tracing::warn!("Failed to publish to Redis: {}", e);
                                                  }
                                              }
                                         }
                                    }
                                    _ = shutdown_rx.recv() => {
                                        let _ = helius_collector.disconnect().await;
                                        return;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Helius Connect failed: {}. Retrying in 5s...", e);
                            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        }
                    }
                }
            });
        }
    }

    // Start Jupiter collector if configured (New Optimization)
    if let Some(jupiter_config) = config.jupiter.clone() {
        if jupiter_config.enabled {
            tracing::info!("Starting Jupiter Price collector");
            use data_engine::collectors::JupiterPriceCollector;
            let jupiter_collector = JupiterPriceCollector::new(jupiter_config);
            let mut shutdown_rx = shutdown_tx.subscribe();
            let repos = Arc::clone(&postgres_repos);

            let redis_publisher = if let Some(r) = &redis {
                Some(r.read().await.clone())
            } else {
                None
            };
            let tx_clone = broadcast_tx.clone();

            tokio::spawn(async move {
                loop {
                    match jupiter_collector.connect(repos.token.clone()).await {
                        Ok(mut rx) => {
                            tracing::info!("Jupiter Price collector connection established");
                            loop {
                                tokio::select! {
                                    Some(msg) = rx.recv() => {
                                         if let Err(e) = repos.market_data.insert_snapshot(&msg).await {
                                             tracing::warn!("Failed to store Jupiter snapshot: {}", e);
                                         }
                                         
                                         // Map to Standard Event
                                         let update = MarketDataUpdate {
                                             symbol: msg.symbol.clone(),
                                             price: msg.price.to_f64().unwrap_or_default(),
                                             volume: msg.quantity.to_f64().unwrap_or_default(),
                                             timestamp: Utc.timestamp_millis_opt(msg.timestamp).unwrap(),
                                             source: "jupiter".to_string(),
                                         };

                                         if let Ok(json) = serde_json::to_string(&update) {
                                              let _ = tx_clone.send(json.clone());
                                              let channel = "market_data";
                                              if let Some(publisher) = &redis_publisher {
                                                  if let Err(e) = publisher.publish(&channel, &json).await {
                                                      tracing::warn!("Failed to publish to Redis: {}", e);
                                                  }
                                              }
                                         }
                                    }
                                    _ = shutdown_rx.recv() => {
                                        let _ = jupiter_collector.disconnect().await;
                                        return;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Jupiter Connect failed: {}. Retrying in 5s...", e);
                            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
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
                    Ok(mut rx) => loop {
                        tokio::select! {
                            Some(msg) = rx.recv() => {
                                if let Err(e) = repos.market_data.insert_snapshot(&msg).await {
                                    tracing::warn!("Failed to store AkShare snapshot: {}", e);
                                }
                            }
                            _ = shutdown_rx.recv() => break,
                        }
                    },
                    Err(e) => tracing::error!("AkShare Connect failed: {}", e),
                }
            });
        }
    }

    // Start Massive (Polygon.io) collector if configured
    if let Some(massive_config) = config.massive.clone() {
        if !massive_config.api_key.is_empty() {
            tracing::info!("Starting Massive (Polygon) collector");
            let mut massive_collector = MassiveConnector::new(massive_config.clone());
            let mut shutdown_rx = shutdown_tx.subscribe();
            let repos = Arc::clone(&postgres_repos);

            let redis_publisher = if let Some(r) = &redis {
                Some(r.read().await.clone())
            } else {
                None
            };
            let tx_clone = broadcast_tx.clone();

            tokio::spawn(async move {
                // Retry loop for connection
                loop {
                    match massive_collector.connect().await {
                        Ok(mut rx) => {
                            tracing::info!("Massive/Polygon collector connection established");
                            loop {
                                tokio::select! {
                                    Some(msg) = rx.recv() => {
                                        if let Err(e) = repos.market_data.insert_snapshot(&msg).await {
                                            tracing::warn!("Failed to store Massive snapshot: {}", e);
                                        }

                                        // Map to Standard Event
                                        let update = MarketDataUpdate {
                                            symbol: msg.symbol.clone(),
                                            price: msg.price.to_f64().unwrap_or_default(),
                                            volume: msg.quantity.to_f64().unwrap_or_default(),
                                            timestamp: Utc.timestamp_millis_opt(msg.timestamp).unwrap(),
                                            source: "massive".to_string(),
                                        };

                                        if let Ok(json) = serde_json::to_string(&update) {
                                             // Send to WebSocket
                                             let _ = tx_clone.send(json.clone());

                                             let channel = "market_data";
                                             if let Some(publisher) = &redis_publisher {
                                                 if let Err(e) = publisher.publish(&channel, &json).await {
                                                     tracing::warn!("Failed to publish to Redis: {}", e);
                                                 }
                                             }
                                        }
                                    }
                                    _ = shutdown_rx.recv() => {
                                        let _ = massive_collector.disconnect().await;
                                        return;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Massive Connect failed: {}. Retrying in 10s...", e);
                            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                        }
                    }
                }
            });
        }
    }

    // Start Binance collector if configured
    // Start Binance collector if configured
    /*
    if let Some(binance_config) = config.binance.clone() {
        if binance_config.enabled {
            tracing::info!("Starting Binance collector");
            let mut binance_collector = BinanceConnector::new(binance_config);
            let mut shutdown_rx = shutdown_tx.subscribe();
            let repos = Arc::clone(&postgres_repos);

            let redis_publisher = if let Some(r) = &redis {
                 Some(r.read().await.clone())
            } else {
                 None
            };
            // Note: Binance block was missing tx_clone, need to add it or it won't broadcast to WS
            let tx_clone = broadcast_tx.clone();

            tokio::spawn(async move {
                loop {
                    match binance_collector.connect().await {
                         Ok(mut rx) => {
                             tracing::info!("Binance collector connection established");
                             loop {
                                 tokio::select! {
                                     Some(msg) = rx.recv() => {
                                          if let Err(e) = repos.market_data.insert_snapshot(&msg).await {
                                              tracing::warn!("Failed to store Binance snapshot: {}", e);
                                          }

                                          // Map to Standard Event
                                          let update = MarketDataUpdate {
                                              symbol: msg.symbol.clone(),
                                              price: msg.price.to_f64().unwrap_or_default(),
                                              volume: msg.quantity.to_f64().unwrap_or_default(),
                                              timestamp: Utc.timestamp_millis_opt(msg.timestamp).unwrap(),
                                              source: "binance".to_string(),
                                          };

                                          if let Ok(json) = serde_json::to_string(&update) {
                                               // Send to WebSocket
                                               let _ = tx_clone.send(json.clone());

                                               let channel = "market_data";
                                               if let Some(publisher) = &redis_publisher {
                                                   if let Err(e) = publisher.publish(&channel, &json).await {
                                                       tracing::warn!("Failed to publish to Redis: {}", e);
                                                   }
                                               }
                                          }
                                     }
                                     _ = shutdown_rx.recv() => {
                                         let _ = binance_collector.disconnect().await;
                                         return;
                                     }
                                 }
                             }
                         }
                         Err(e) => {
                             tracing::error!("Binance Connect failed: {}. Retrying...", e);
                             tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                         }
                    }
                }
            });
        }
    }
    */

    // Start OKX collector if configured
    if let Some(okx_config) = config.okx.clone() {
        if okx_config.enabled {
            tracing::info!("Starting OKX collector");
            let mut okx_collector = OkxConnector::new(okx_config);
            let mut shutdown_rx = shutdown_tx.subscribe();
            let repos = Arc::clone(&postgres_repos);

            let redis_publisher = if let Some(r) = &redis {
                Some(r.read().await.clone())
            } else {
                None
            };
            let tx_clone = broadcast_tx.clone();

            tokio::spawn(async move {
                loop {
                    match okx_collector.connect().await {
                        Ok(mut rx) => {
                            tracing::info!("OKX collector connection established");
                            loop {
                                tokio::select! {
                                    Some(msg) = rx.recv() => {
                                         if let Err(e) = repos.market_data.insert_snapshot(&msg).await {
                                             tracing::warn!("Failed to store OKX snapshot: {}", e);
                                         }

                                         // Map to Standard Event
                                         let update = MarketDataUpdate {
                                             symbol: msg.symbol.clone(),
                                             price: msg.price.to_f64().unwrap_or_default(),
                                             volume: msg.quantity.to_f64().unwrap_or_default(),
                                             timestamp: Utc.timestamp_millis_opt(msg.timestamp).unwrap(),
                                             source: "okx".to_string(),
                                         };

                                         if let Ok(json) = serde_json::to_string(&update) {
                                              // Send to WebSocket
                                              let _ = tx_clone.send(json.clone());

                                              let channel = "market_data";
                                              if let Some(publisher) = &redis_publisher {
                                                  if let Err(e) = publisher.publish(&channel, &json).await {
                                                      tracing::warn!("Failed to publish to Redis: {}", e);
                                                  }
                                              }
                                         }
                                    }
                                    _ = shutdown_rx.recv() => {
                                        let _ = okx_collector.disconnect().await;
                                        return;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("OKX Connect failed: {}. Retrying...", e);
                            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        }
                    }
                }
            });
        }
    }

    // Start Bybit collector if configured
    if let Some(bybit_config) = config.bybit.clone() {
        if bybit_config.enabled {
            tracing::info!("Starting Bybit collector");
            let mut bybit_collector = BybitConnector::new(bybit_config);
            let mut shutdown_rx = shutdown_tx.subscribe();
            let repos = Arc::clone(&postgres_repos);

            let redis_publisher = if let Some(r) = &redis {
                Some(r.read().await.clone())
            } else {
                None
            };
            let tx_clone = broadcast_tx.clone();

            tokio::spawn(async move {
                loop {
                    match bybit_collector.connect().await {
                        Ok(mut rx) => {
                            tracing::info!("Bybit collector connection established");
                            loop {
                                tokio::select! {
                                    Some(msg) = rx.recv() => {
                                         if let Err(e) = repos.market_data.insert_snapshot(&msg).await {
                                             tracing::warn!("Failed to store Bybit snapshot: {}", e);
                                         }

                                         // Map to Standard Event
                                         let update = MarketDataUpdate {
                                             symbol: msg.symbol.clone(),
                                             price: msg.price.to_f64().unwrap_or_default(),
                                             volume: msg.quantity.to_f64().unwrap_or_default(),
                                             timestamp: Utc.timestamp_millis_opt(msg.timestamp).unwrap(),
                                             source: "bybit".to_string(),
                                         };

                                         if let Ok(json) = serde_json::to_string(&update) {
                                              // Send to WebSocket
                                              let _ = tx_clone.send(json.clone());

                                              let channel = "market_data";
                                              if let Some(publisher) = &redis_publisher {
                                                  if let Err(e) = publisher.publish(&channel, &json).await {
                                                      tracing::warn!("Failed to publish to Redis: {}", e);
                                                  }
                                              }
                                         }
                                    }
                                    _ = shutdown_rx.recv() => {
                                        let _ = bybit_collector.disconnect().await;
                                        return;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Bybit Connect failed: {}. Retrying...", e);
                            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        }
                    }
                }
            });
        }
    }

    // Start Futu collector if configured
    if let Some(futu_config) = config.futu.clone() {
        if futu_config.enabled {
            tracing::info!("Starting Futu collector");
            let mut futu_collector = FutuConnector::new(futu_config);
            let mut shutdown_rx = shutdown_tx.subscribe();
            let repos = Arc::clone(&postgres_repos);

            let redis_publisher = if let Some(r) = &redis {
                Some(r.read().await.clone())
            } else {
                None
            };
            let tx_clone = broadcast_tx.clone();

            tokio::spawn(async move {
                loop {
                    match futu_collector.connect().await {
                        Ok(mut rx) => {
                            tracing::info!("Futu collector connection established (Placeholder)");
                            loop {
                                tokio::select! {
                                    Some(msg) = rx.recv() => {
                                         if let Err(e) = repos.market_data.insert_snapshot(&msg).await {
                                             tracing::warn!("Failed to store Futu snapshot: {}", e);
                                         }

                                         // Map to Standard Event
                                         let update = MarketDataUpdate {
                                             symbol: msg.symbol.clone(),
                                             price: msg.price.to_f64().unwrap_or_default(),
                                             volume: msg.quantity.to_f64().unwrap_or_default(),
                                             timestamp: Utc.timestamp_millis_opt(msg.timestamp).unwrap(),
                                             source: "futu".to_string(),
                                         };

                                         if let Ok(json) = serde_json::to_string(&update) {
                                              // Send to WebSocket
                                              let _ = tx_clone.send(json.clone());

                                              let channel = "market_data";
                                              if let Some(publisher) = &redis_publisher {
                                                  if let Err(e) = publisher.publish(&channel, &json).await {
                                                      tracing::warn!("Failed to publish to Redis: {}", e);
                                                  }
                                              }
                                         }
                                    }
                                    _ = shutdown_rx.recv() => {
                                        let _ = futu_collector.disconnect().await;
                                        return;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Futu Connect failed: {}. Retrying...", e);
                            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                        }
                    }
                }
            });
        }
    }

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
                    health_monitor.check_redis(&mut *r_guard).await;
                }

                if let Some(_) = &clickhouse {
                    health_monitor.check_clickhouse().await;
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
