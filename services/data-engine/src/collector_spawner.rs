//! Collector spawning logic extracted from main.rs.
//!
//! This module handles starting all market data collectors (Twitter, Polymarket,
//! Birdeye, Helius, AkShare, Massive/Polygon, OKX, Bybit, Futu) as
//! background tasks with proper shutdown handling.

use chrono::{TimeZone, Utc};
use common::events::MarketDataUpdate;
use rust_decimal::prelude::ToPrimitive;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use data_engine::{
    collectors::{
        birdeye::meta_collector::BirdeyeMetaCollector, circuit_breaker::CircuitBreaker,
        AkShareCollector, BinanceConnector, BirdeyeConnector, BybitConnector, FutuConnector,
        HeliusConnector, MassiveConnector, OkxConnector, PolymarketCollector, TwitterCollector,
    },
    config::AppConfig,
    models::StandardMarketData,
    monitoring::metrics::{
        CIRCUIT_BREAKER_STATE, CIRCUIT_BREAKER_TRIPS, COLLECTOR_LAST_MESSAGE_TS,
        DATA_E2E_FRESHNESS_SECONDS, DATA_ERRORS_BY_SOURCE, DATA_MESSAGES_BY_SOURCE,
        VALIDATION_FAILURES,
    },
    repository::{postgres::PostgresRepositories, MarketDataRepository, SocialRepository},
    storage::RedisCache,
    traits::DataSourceConnector,
};

/// Shared circuit breaker registry, keyed by source name.
pub type CircuitBreakerMap = Arc<HashMap<String, Arc<CircuitBreaker>>>;

/// Build the default set of circuit breakers (one per data source).
pub fn build_circuit_breakers() -> CircuitBreakerMap {
    let sources = [
        "birdeye", "helius", "akshare", "massive", "okx", "bybit", "futu", "binance",
    ];
    let mut map = HashMap::new();
    for source in sources {
        map.insert(
            source.to_string(),
            Arc::new(CircuitBreaker::new(source, 5, Duration::from_secs(60))),
        );
    }
    Arc::new(map)
}

/// Record a connection failure on the circuit breaker and update Prometheus metrics.
async fn cb_record_failure(cb: &CircuitBreaker, source: &str) {
    let before = cb.state_value();
    cb.record_failure().await;
    let after = cb.state_value();
    CIRCUIT_BREAKER_STATE
        .with_label_values(&[source])
        .set(after as i64);
    // Detect Closed→Open transition (trip)
    if before != 1 && after == 1 {
        CIRCUIT_BREAKER_TRIPS.with_label_values(&[source]).inc();
    }
}

/// Record a successful connection and update Prometheus metrics.
fn cb_record_success(cb: &CircuitBreaker, source: &str) {
    cb.record_success();
    CIRCUIT_BREAKER_STATE
        .with_label_values(&[source])
        .set(cb.state_value() as i64);
}

/// Dependencies required by all collectors.
pub struct CollectorDeps {
    pub config: AppConfig,
    pub postgres_repos: Arc<PostgresRepositories>,
    pub redis: Option<Arc<RwLock<RedisCache>>>,
    pub broadcast_tx: tokio::sync::broadcast::Sender<String>,
    pub shutdown_tx: tokio::sync::broadcast::Sender<()>,
    pub circuit_breakers: CircuitBreakerMap,
}

/// Spawn all configured collectors as background tasks.
///
/// Each collector respects the shutdown signal and publishes market data updates
/// to both Redis (for strategy consumers) and the WebSocket broadcast channel.
pub async fn spawn_all_collectors(deps: &CollectorDeps) {
    spawn_twitter_collector(deps).await;
    spawn_polymarket_collector(deps).await;
    spawn_birdeye_collector(deps).await;
    spawn_helius_collector(deps).await;
    spawn_akshare_collector(deps).await;
    spawn_massive_collector(deps).await;
    spawn_okx_collector(deps).await;
    spawn_bybit_collector(deps).await;
    spawn_futu_collector(deps).await;
    spawn_binance_collector(deps).await;
}

async fn spawn_twitter_collector(deps: &CollectorDeps) {
    let Some(twitter_config) = deps.config.twitter.clone() else {
        return;
    };

    tracing::info!("Starting Twitter collector");
    let twitter_cfg = twitter_config.clone();
    let twitter_collector = Arc::new(TwitterCollector::new(twitter_config));
    let shutdown_rx = deps.shutdown_tx.subscribe();
    let repos = Arc::clone(&deps.postgres_repos);

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

async fn spawn_polymarket_collector(deps: &CollectorDeps) {
    let Some(polymarket_config) = deps.config.polymarket.clone() else {
        return;
    };

    tracing::info!("Starting Polymarket collector");
    let pc = Arc::new(PolymarketCollector::new(
        polymarket_config,
        deps.postgres_repos.prediction.clone(),
    ));
    let s_rx = deps.shutdown_tx.subscribe();
    tokio::spawn(async move {
        if let Err(e) = pc.start(s_rx).await {
            tracing::error!("Polymarket collector error: {}", e);
        }
    });
}

async fn spawn_birdeye_collector(deps: &CollectorDeps) {
    let Some(birdeye_config) = deps.config.birdeye.clone() else {
        return;
    };
    if !birdeye_config.enabled {
        return;
    }

    tracing::info!("Starting Birdeye collector");
    let birdeye_collector = BirdeyeConnector::new(birdeye_config.clone());
    let mut shutdown_rx = deps.shutdown_tx.subscribe();
    let repos = Arc::clone(&deps.postgres_repos);

    let redis_publisher = get_redis_publisher(&deps.redis).await;
    let tx_clone = deps.broadcast_tx.clone();

    // Spawn Meta Collector if Redis is available
    if let Some(r) = &deps.redis {
        let meta_collector =
            BirdeyeMetaCollector::new(birdeye_config, r.clone(), repos.token.clone());
        tokio::spawn(async move {
            meta_collector.run().await;
        });
        tracing::info!("Started Birdeye Meta Collector");
    }

    let cb = deps.circuit_breakers.get("birdeye").cloned();
    tokio::spawn(async move {
        loop {
            if let Some(ref cb) = cb {
                if !cb.allow_request().await {
                    tracing::warn!(source = "birdeye", "Circuit breaker open, skipping connect");
                    CIRCUIT_BREAKER_STATE
                        .with_label_values(&["birdeye"])
                        .set(cb.state_value() as i64);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }
            }
            match birdeye_collector.connect(repos.token.clone()).await {
                Ok(mut rx) => {
                    if let Some(ref cb) = cb {
                        cb_record_success(cb, "birdeye");
                    }
                    tracing::info!("Birdeye collector connection established");
                    loop {
                        tokio::select! {
                            Some(msg) = rx.recv() => {
                                if !validate_market_data(&msg, "birdeye") {
                                    continue;
                                }
                                insert_with_retry(&repos, &msg, "birdeye").await;
                                publish_market_update(&msg, "Birdeye", &tx_clone, &redis_publisher).await;
                            }
                            _ = shutdown_rx.recv() => {
                                let _ = birdeye_collector.disconnect().await;
                                return;
                            }
                        }
                    }
                }
                Err(e) => {
                    if let Some(ref cb) = cb {
                        cb_record_failure(cb, "birdeye").await;
                    }
                    tracing::error!("Birdeye Connect failed: {}. Retrying...", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                }
            }
        }
    });
}

async fn spawn_helius_collector(deps: &CollectorDeps) {
    let Some(helius_config) = deps.config.helius.clone() else {
        return;
    };
    if !helius_config.enabled {
        return;
    }

    tracing::info!("Starting Helius collector");
    let helius_collector = HeliusConnector::new(helius_config);
    let mut shutdown_rx = deps.shutdown_tx.subscribe();
    let repos = Arc::clone(&deps.postgres_repos);

    let redis_publisher = get_redis_publisher(&deps.redis).await;
    let tx_clone = deps.broadcast_tx.clone();

    let cb = deps.circuit_breakers.get("helius").cloned();
    tokio::spawn(async move {
        loop {
            if let Some(ref cb) = cb {
                if !cb.allow_request().await {
                    tracing::warn!(source = "helius", "Circuit breaker open, skipping connect");
                    CIRCUIT_BREAKER_STATE
                        .with_label_values(&["helius"])
                        .set(cb.state_value() as i64);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }
            }
            match helius_collector.connect().await {
                Ok(mut rx) => {
                    if let Some(ref cb) = cb {
                        cb_record_success(cb, "helius");
                    }
                    tracing::info!("Helius collector connection established");
                    loop {
                        tokio::select! {
                            Some(msg) = rx.recv() => {
                                if !validate_market_data(&msg, "helius") {
                                    continue;
                                }
                                insert_with_retry(&repos, &msg, "helius").await;
                                publish_market_update(&msg, "Helius", &tx_clone, &redis_publisher).await;
                            }
                            _ = shutdown_rx.recv() => {
                                let _ = helius_collector.disconnect().await;
                                return;
                            }
                        }
                    }
                }
                Err(e) => {
                    if let Some(ref cb) = cb {
                        cb_record_failure(cb, "helius").await;
                    }
                    tracing::error!("Helius Connect failed: {}. Retrying in 5s...", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    });
}

async fn spawn_akshare_collector(deps: &CollectorDeps) {
    let Some(ak_config) = deps.config.akshare.clone() else {
        return;
    };
    if !ak_config.enabled {
        return;
    }

    tracing::info!("Starting AkShare collector");
    let mut ak_collector = AkShareCollector::new(ak_config);
    let mut shutdown_rx = deps.shutdown_tx.subscribe();
    let repos = Arc::clone(&deps.postgres_repos);

    let redis_publisher = get_redis_publisher(&deps.redis).await;
    let tx_clone = deps.broadcast_tx.clone();

    let cb = deps.circuit_breakers.get("akshare").cloned();
    tokio::spawn(async move {
        match ak_collector.connect().await {
            Ok(mut rx) => {
                if let Some(ref cb) = cb {
                    cb_record_success(cb, "akshare");
                }
                loop {
                    tokio::select! {
                        Some(msg) = rx.recv() => {
                            if !validate_market_data(&msg, "akshare") {
                                continue;
                            }
                            insert_with_retry(&repos, &msg, "akshare").await;
                            publish_market_update(&msg, "akshare", &tx_clone, &redis_publisher).await;
                        }
                        _ = shutdown_rx.recv() => break,
                    }
                }
            }
            Err(e) => {
                if let Some(ref cb) = cb {
                    cb_record_failure(cb, "akshare").await;
                }
                tracing::error!("AkShare Connect failed: {}", e);
            }
        }
    });
}

async fn spawn_massive_collector(deps: &CollectorDeps) {
    let Some(massive_config) = deps.config.massive.clone() else {
        return;
    };
    if massive_config.api_key.is_empty() {
        return;
    }

    let symbols = match deps
        .postgres_repos
        .market_data
        .get_watchlist_symbols()
        .await
    {
        Ok(s) => {
            tracing::info!("Polygon WS: subscribing to {} watchlist symbols", s.len());
            s
        }
        Err(e) => {
            tracing::warn!(
                "Failed to load watchlist for Polygon WS, falling back to A.*: {}",
                e
            );
            vec![]
        }
    };

    tracing::info!("Starting Massive (Polygon) collector");
    let mut massive_collector = MassiveConnector::new(massive_config, symbols);
    let mut shutdown_rx = deps.shutdown_tx.subscribe();
    let repos = Arc::clone(&deps.postgres_repos);

    let redis_publisher = get_redis_publisher(&deps.redis).await;
    let tx_clone = deps.broadcast_tx.clone();

    let cb = deps.circuit_breakers.get("massive").cloned();
    tokio::spawn(async move {
        loop {
            if let Some(ref cb) = cb {
                if !cb.allow_request().await {
                    tracing::warn!(source = "massive", "Circuit breaker open, skipping connect");
                    CIRCUIT_BREAKER_STATE
                        .with_label_values(&["massive"])
                        .set(cb.state_value() as i64);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }
            }
            match massive_collector.connect().await {
                Ok(mut rx) => {
                    if let Some(ref cb) = cb {
                        cb_record_success(cb, "massive");
                    }
                    tracing::info!("Massive/Polygon collector connection established");
                    loop {
                        tokio::select! {
                            Some(msg) = rx.recv() => {
                                if !validate_market_data(&msg, "massive") {
                                    continue;
                                }
                                insert_with_retry(&repos, &msg, "massive").await;
                                publish_market_update(&msg, "massive", &tx_clone, &redis_publisher).await;
                            }
                            _ = shutdown_rx.recv() => {
                                let _ = massive_collector.disconnect().await;
                                return;
                            }
                        }
                    }
                }
                Err(e) => {
                    if let Some(ref cb) = cb {
                        cb_record_failure(cb, "massive").await;
                    }
                    tracing::error!("Massive Connect failed: {}. Retrying in 10s...", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                }
            }
        }
    });
}

async fn spawn_okx_collector(deps: &CollectorDeps) {
    let Some(okx_config) = deps.config.okx.clone() else {
        return;
    };
    if !okx_config.enabled {
        return;
    }

    tracing::info!("Starting OKX collector");
    let mut okx_collector = OkxConnector::new(okx_config);
    let mut shutdown_rx = deps.shutdown_tx.subscribe();
    let repos = Arc::clone(&deps.postgres_repos);

    let redis_publisher = get_redis_publisher(&deps.redis).await;
    let tx_clone = deps.broadcast_tx.clone();

    let cb = deps.circuit_breakers.get("okx").cloned();
    tokio::spawn(async move {
        loop {
            if let Some(ref cb) = cb {
                if !cb.allow_request().await {
                    tracing::warn!(source = "okx", "Circuit breaker open, skipping connect");
                    CIRCUIT_BREAKER_STATE
                        .with_label_values(&["okx"])
                        .set(cb.state_value() as i64);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }
            }
            match okx_collector.connect().await {
                Ok(mut rx) => {
                    if let Some(ref cb) = cb {
                        cb_record_success(cb, "okx");
                    }
                    tracing::info!("OKX collector connection established");
                    loop {
                        tokio::select! {
                            Some(msg) = rx.recv() => {
                                if !validate_market_data(&msg, "okx") {
                                    continue;
                                }
                                insert_with_retry(&repos, &msg, "okx").await;
                                publish_market_update(&msg, "okx", &tx_clone, &redis_publisher).await;
                            }
                            _ = shutdown_rx.recv() => {
                                let _ = okx_collector.disconnect().await;
                                return;
                            }
                        }
                    }
                }
                Err(e) => {
                    if let Some(ref cb) = cb {
                        cb_record_failure(cb, "okx").await;
                    }
                    tracing::error!("OKX Connect failed: {}. Retrying...", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    });
}

async fn spawn_bybit_collector(deps: &CollectorDeps) {
    let Some(bybit_config) = deps.config.bybit.clone() else {
        return;
    };
    if !bybit_config.enabled {
        return;
    }

    tracing::info!("Starting Bybit collector");
    let mut bybit_collector = BybitConnector::new(bybit_config);
    let mut shutdown_rx = deps.shutdown_tx.subscribe();
    let repos = Arc::clone(&deps.postgres_repos);

    let redis_publisher = get_redis_publisher(&deps.redis).await;
    let tx_clone = deps.broadcast_tx.clone();

    let cb = deps.circuit_breakers.get("bybit").cloned();
    tokio::spawn(async move {
        loop {
            if let Some(ref cb) = cb {
                if !cb.allow_request().await {
                    tracing::warn!(source = "bybit", "Circuit breaker open, skipping connect");
                    CIRCUIT_BREAKER_STATE
                        .with_label_values(&["bybit"])
                        .set(cb.state_value() as i64);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }
            }
            match bybit_collector.connect().await {
                Ok(mut rx) => {
                    if let Some(ref cb) = cb {
                        cb_record_success(cb, "bybit");
                    }
                    tracing::info!("Bybit collector connection established");
                    loop {
                        tokio::select! {
                            Some(msg) = rx.recv() => {
                                if !validate_market_data(&msg, "bybit") {
                                    continue;
                                }
                                insert_with_retry(&repos, &msg, "bybit").await;
                                publish_market_update(&msg, "bybit", &tx_clone, &redis_publisher).await;
                            }
                            _ = shutdown_rx.recv() => {
                                let _ = bybit_collector.disconnect().await;
                                return;
                            }
                        }
                    }
                }
                Err(e) => {
                    if let Some(ref cb) = cb {
                        cb_record_failure(cb, "bybit").await;
                    }
                    tracing::error!("Bybit Connect failed: {}. Retrying...", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    });
}

async fn spawn_futu_collector(deps: &CollectorDeps) {
    let Some(futu_config) = deps.config.futu.clone() else {
        return;
    };
    if !futu_config.enabled {
        return;
    }

    tracing::info!("Starting Futu collector");
    let mut futu_collector = FutuConnector::new(futu_config);
    let mut shutdown_rx = deps.shutdown_tx.subscribe();
    let repos = Arc::clone(&deps.postgres_repos);

    let redis_publisher = get_redis_publisher(&deps.redis).await;
    let tx_clone = deps.broadcast_tx.clone();

    let cb = deps.circuit_breakers.get("futu").cloned();
    tokio::spawn(async move {
        loop {
            if let Some(ref cb) = cb {
                if !cb.allow_request().await {
                    tracing::warn!(source = "futu", "Circuit breaker open, skipping connect");
                    CIRCUIT_BREAKER_STATE
                        .with_label_values(&["futu"])
                        .set(cb.state_value() as i64);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }
            }
            match futu_collector.connect().await {
                Ok(mut rx) => {
                    if let Some(ref cb) = cb {
                        cb_record_success(cb, "futu");
                    }
                    tracing::info!("Futu collector connection established (Placeholder)");
                    loop {
                        tokio::select! {
                            Some(msg) = rx.recv() => {
                                if !validate_market_data(&msg, "futu") {
                                    continue;
                                }
                                insert_with_retry(&repos, &msg, "futu").await;
                                publish_market_update(&msg, "futu", &tx_clone, &redis_publisher).await;
                            }
                            _ = shutdown_rx.recv() => {
                                let _ = futu_collector.disconnect().await;
                                return;
                            }
                        }
                    }
                }
                Err(e) => {
                    if let Some(ref cb) = cb {
                        cb_record_failure(cb, "futu").await;
                    }
                    tracing::error!("Futu Connect failed: {}. Retrying...", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                }
            }
        }
    });
}

async fn spawn_binance_collector(deps: &CollectorDeps) {
    let Some(binance_config) = deps.config.binance.clone() else {
        return;
    };
    if !binance_config.enabled {
        return;
    }
    if binance_config.symbols.is_empty() {
        tracing::warn!("Binance collector enabled but no symbols configured, skipping");
        return;
    }

    tracing::info!("Starting Binance collector");
    let mut binance_collector = BinanceConnector::new(binance_config);
    let mut shutdown_rx = deps.shutdown_tx.subscribe();
    let repos = Arc::clone(&deps.postgres_repos);

    let redis_publisher = get_redis_publisher(&deps.redis).await;
    let tx_clone = deps.broadcast_tx.clone();

    let cb = deps.circuit_breakers.get("binance").cloned();
    tokio::spawn(async move {
        loop {
            if let Some(ref cb) = cb {
                if !cb.allow_request().await {
                    tracing::warn!(source = "binance", "Circuit breaker open, skipping connect");
                    CIRCUIT_BREAKER_STATE
                        .with_label_values(&["binance"])
                        .set(cb.state_value() as i64);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }
            }
            match binance_collector.connect().await {
                Ok(mut rx) => {
                    if let Some(ref cb) = cb {
                        cb_record_success(cb, "binance");
                    }
                    tracing::info!("Binance collector connection established");
                    loop {
                        tokio::select! {
                            Some(msg) = rx.recv() => {
                                if !validate_market_data(&msg, "binance") {
                                    continue;
                                }
                                insert_with_retry(&repos, &msg, "binance").await;
                                publish_market_update(&msg, "Binance", &tx_clone, &redis_publisher).await;
                            }
                            _ = shutdown_rx.recv() => {
                                let _ = binance_collector.disconnect().await;
                                return;
                            }
                        }
                    }
                }
                Err(e) => {
                    if let Some(ref cb) = cb {
                        cb_record_failure(cb, "binance").await;
                    }
                    tracing::error!("Binance Connect failed: {}. Retrying in 5s...", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    });
}

const INSERT_MAX_RETRIES: u32 = 3;
const INSERT_INITIAL_DELAY_MS: u64 = 200;

/// Insert a snapshot with retry and dead-letter fallback.
///
/// Retries up to 3 times with exponential backoff (200ms -> 400ms -> 800ms).
/// If all retries fail, logs a dead-letter record for later investigation.
async fn insert_with_retry(repos: &PostgresRepositories, msg: &StandardMarketData, source: &str) {
    let mut delay_ms = INSERT_INITIAL_DELAY_MS;
    for attempt in 1..=INSERT_MAX_RETRIES {
        match repos.market_data.insert_snapshot(msg).await {
            Ok(()) => return,
            Err(e) if attempt == INSERT_MAX_RETRIES => {
                tracing::error!(
                    source = source,
                    symbol = %msg.symbol,
                    attempt = attempt,
                    err = %e,
                    "Insert failed after all retries"
                );
                data_engine::monitoring::dead_letter::log_dead_letter(
                    msg,
                    &e.to_string(),
                    "postgres",
                );
                return;
            }
            Err(e) => {
                tracing::warn!(
                    source = source,
                    symbol = %msg.symbol,
                    attempt = attempt,
                    err = %e,
                    "Insert failed, retrying in {}ms",
                    delay_ms
                );
                DATA_ERRORS_BY_SOURCE.with_label_values(&[source]).inc();
                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                delay_ms *= 2;
            }
        }
    }
}

/// Validates market data before storage. Returns `true` if valid.
/// Invalid data is logged and counted in `VALIDATION_FAILURES` metric.
fn validate_market_data(msg: &StandardMarketData, source: &str) -> bool {
    match msg.validate() {
        Ok(()) => true,
        Err(reason) => {
            tracing::warn!(
                source = source,
                symbol = %msg.symbol,
                reason = %reason,
                "Rejected invalid market data"
            );
            VALIDATION_FAILURES.inc();
            DATA_ERRORS_BY_SOURCE.with_label_values(&[source]).inc();
            false
        }
    }
}

/// Helper: get a cloned RedisCache for publishing if Redis is available.
async fn get_redis_publisher(redis: &Option<Arc<RwLock<RedisCache>>>) -> Option<RedisCache> {
    if let Some(r) = redis {
        Some(r.read().await.clone())
    } else {
        None
    }
}

/// Helper: map a StandardMarketData snapshot to a MarketDataUpdate event and
/// publish it to both the WebSocket broadcast channel and Redis.
async fn publish_market_update(
    msg: &data_engine::models::StandardMarketData,
    source: &str,
    broadcast_tx: &tokio::sync::broadcast::Sender<String>,
    redis_publisher: &Option<RedisCache>,
) {
    let update = MarketDataUpdate {
        symbol: msg.symbol.clone(),
        price: msg.price.to_f64().unwrap_or_default(),
        volume: msg.quantity.to_f64().unwrap_or_default(),
        timestamp: Utc.timestamp_millis_opt(msg.timestamp).unwrap(),
        source: source.to_string(),
    };

    // Track per-source message count
    DATA_MESSAGES_BY_SOURCE.with_label_values(&[source]).inc();

    // Track last message timestamp per source (unix epoch seconds)
    let now_epoch = Utc::now().timestamp() as f64;
    COLLECTOR_LAST_MESSAGE_TS
        .with_label_values(&[&source.to_lowercase()])
        .set(now_epoch);

    // Track end-to-end data freshness (source timestamp to now)
    let now_ms = Utc::now().timestamp_millis();
    let lag_seconds = (now_ms - msg.timestamp) as f64 / 1000.0;
    if lag_seconds >= 0.0 {
        let source_lower = source.to_lowercase();
        DATA_E2E_FRESHNESS_SECONDS
            .with_label_values(&[&source_lower])
            .set(lag_seconds);
    }

    if let Ok(json) = serde_json::to_string(&update) {
        let _ = broadcast_tx.send(json.clone());

        if let Some(publisher) = redis_publisher {
            if let Err(e) = publisher.publish("market_data", &json).await {
                DATA_ERRORS_BY_SOURCE.with_label_values(&[source]).inc();
                tracing::warn!("Failed to publish {} update to Redis: {}", source, e);
            }
        }
    }
}
