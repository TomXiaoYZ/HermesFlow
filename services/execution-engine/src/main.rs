use tracing::{error, info, warn};

use chrono::Utc;
use common::events::{PortfolioUpdate, PositionUpdate};
use execution_engine::command_listener::CommandListener;
use execution_engine::reconciliation;
use execution_engine::traders::futu_trader::FutuTrader;
use execution_engine::traders::ibkr_trader::IBKRTrader;
use execution_engine::traders::Trader;
use redis::Commands;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant;
use tokio_postgres::NoTls;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting Execution Engine...");

    // P7-3C: Check holiday coverage at startup
    execution_engine::shadow::check_holiday_coverage();

    // P10D-3: Initialize Prometheus metrics
    common::metrics::init_metrics("execution-engine").expect("metrics init failed");
    execution_engine::metrics::register_metrics();

    // Spawn health check server (now includes /metrics endpoint via common::metrics feature)
    tokio::spawn(common::health::start_health_server(
        "execution-engine",
        8083,
    ));

    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    // ========================================
    // 0. Initialize Database Connection (optional)
    // ========================================
    let db = match env::var("DATABASE_URL") {
        Ok(url) => {
            match tokio_postgres::connect(&url, NoTls).await {
                Ok((client, connection)) => {
                    // Spawn the connection handler in background
                    tokio::spawn(async move {
                        if let Err(e) = connection.await {
                            error!("PostgreSQL connection error: {}", e);
                        }
                    });
                    info!("Connected to PostgreSQL for trade persistence");
                    Some(Arc::new(client))
                }
                Err(e) => {
                    warn!("DB not available, trades will not be persisted: {}", e);
                    None
                }
            }
        }
        Err(_) => {
            warn!("DATABASE_URL not set, trade persistence disabled");
            None
        }
    };

    // ========================================
    // 1. Initialize IBKR Traders (one per mode)
    // ========================================
    let ibkr_host = env::var("IBKR_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let ibkr_port: u32 = env::var("IBKR_PORT")
        .unwrap_or_else(|_| "7497".to_string())
        .parse()
        .unwrap_or(7497);
    // long_short can connect to a separate gateway (IBKR_HOST_LS/IBKR_PORT_LS)
    let ibkr_host_ls = env::var("IBKR_HOST_LS").unwrap_or_else(|_| ibkr_host.clone());
    let ibkr_port_ls: u32 = env::var("IBKR_PORT_LS")
        .unwrap_or_else(|_| ibkr_port.to_string())
        .parse()
        .unwrap_or(ibkr_port);
    let ibkr_client_id_lo: u32 = env::var("IBKR_CLIENT_ID_LONG_ONLY")
        .unwrap_or_else(|_| "1".to_string())
        .parse()
        .unwrap_or(1);
    let ibkr_client_id_ls: u32 = env::var("IBKR_CLIENT_ID_LONG_SHORT")
        .unwrap_or_else(|_| "2".to_string())
        .parse()
        .unwrap_or(2);

    info!(
        "Initializing IBKR Trader long_only ({}:{}, client_id={})...",
        ibkr_host, ibkr_port, ibkr_client_id_lo
    );
    let ibkr_long_only = match IBKRTrader::new(&ibkr_host, ibkr_port, ibkr_client_id_lo).await {
        Ok(t) => {
            info!(
                "IBKR Trader long_only connected (client_id={})",
                ibkr_client_id_lo
            );
            Some(Arc::new(t))
        }
        Err(e) => {
            warn!(
                "IBKR Trader long_only not available (client_id={}): {}",
                ibkr_client_id_lo, e
            );
            None
        }
    };

    info!(
        "Initializing IBKR Trader long_short ({}:{}, client_id={})...",
        ibkr_host_ls, ibkr_port_ls, ibkr_client_id_ls
    );
    let ibkr_long_short =
        match IBKRTrader::new(&ibkr_host_ls, ibkr_port_ls, ibkr_client_id_ls).await {
            Ok(t) => {
                info!(
                    "IBKR Trader long_short connected (client_id={})",
                    ibkr_client_id_ls
                );
                Some(Arc::new(t))
            }
            Err(e) => {
                warn!(
                    "IBKR Trader long_short not available (client_id={}): {}",
                    ibkr_client_id_ls, e
                );
                None
            }
        };

    // ========================================
    // 1b. Sync order ID counter with DB max to avoid uniqueness conflicts
    // ========================================
    if let Some(ref db_client) = db {
        match db_client
            .query_one(
                "SELECT COALESCE(MAX(order_id::bigint), 0)::bigint FROM trade_orders WHERE order_id ~ '^\\d+$'",
                &[],
            )
            .await
        {
            Ok(row) => {
                let max_id: i64 = row.get(0);
                if max_id > 0 {
                    execution_engine::traders::ibkr_trader::set_min_order_id((max_id + 1) as i32);
                }
            }
            Err(e) => warn!("Failed to query max order_id from DB: {}", e),
        }
    }

    // ========================================
    // 2. Initialize Futu Trader
    // ========================================
    let futu_bridge_url =
        env::var("FUTU_BRIDGE_URL").unwrap_or_else(|_| "http://127.0.0.1:8088".to_string());

    info!("Initializing Futu Trader (bridge: {})...", futu_bridge_url);
    let futu = match FutuTrader::new(&futu_bridge_url).await {
        Ok(t) => {
            info!("Futu Trader initialized");
            Some(Arc::new(t))
        }
        Err(e) => {
            warn!("Futu Trader not available: {}", e);
            None
        }
    };

    // ========================================
    // 3. Setup Command Listener
    // ========================================
    let mut listener = CommandListener::new(&redis_url, db.clone())?;
    listener.set_traders(
        ibkr_long_only.clone(),
        ibkr_long_short.clone(),
        futu.clone(),
    );

    // Record status before moves
    let ibkr_lo_on = ibkr_long_only.is_some();
    let ibkr_ls_on = ibkr_long_short.is_some();
    let futu_on = futu.is_some();

    // ========================================
    // 4. Background: IBKR Portfolio Sync (queries both traders for per-account data)
    // ========================================
    {
        // Collect all available IBKR traders for account summary queries.
        // Each client connection only sees its own sub-account.
        let mut traders: Vec<Arc<IBKRTrader>> = Vec::new();
        let mut trader_labels: Vec<&'static str> = Vec::new();
        if let Some(ref t) = ibkr_long_only {
            traders.push(t.clone());
            trader_labels.push("long_only");
        }
        if let Some(ref t) = ibkr_long_short {
            traders.push(t.clone());
            trader_labels.push("long_short");
        }

        if !traders.is_empty() {
            let redis_url_clone = redis_url.clone();
            let db_for_sync = db.clone();

            tokio::spawn(async move {
                info!(
                    "Starting IBKR Portfolio Sync Task ({} traders)...",
                    traders.len()
                );
                let client = match redis::Client::open(redis_url_clone.as_str()) {
                    Ok(c) => c,
                    Err(e) => {
                        error!("Failed to create Redis client for IBKR sync: {}", e);
                        return;
                    }
                };
                let mut con = match client.get_connection() {
                    Ok(c) => c,
                    Err(e) => {
                        error!("Failed to connect to Redis for IBKR sync: {}", e);
                        return;
                    }
                };

                // Per-trader health state for exponential backoff reconnect
                struct TraderHealth {
                    consecutive_failures: u32,
                    last_attempt: Instant,
                }

                impl TraderHealth {
                    fn new() -> Self {
                        Self {
                            consecutive_failures: 0,
                            last_attempt: Instant::now(),
                        }
                    }

                    fn should_attempt(&self) -> bool {
                        let backoff =
                            Duration::from_secs(30 * 2u64.pow(self.consecutive_failures.min(6)));
                        self.last_attempt.elapsed() >= backoff
                    }
                }

                let mut health: Vec<TraderHealth> =
                    traders.iter().map(|_| TraderHealth::new()).collect();

                let mut sync_count: u32 = 0;
                loop {
                    // ── Health check + reconnect with exponential backoff ──
                    for (i, trader) in traders.iter().enumerate() {
                        let label = trader_labels[i];
                        if !trader.is_alive().await {
                            execution_engine::metrics::IBKR_CONNECTED
                                .with_label_values(&[label])
                                .set(0.0);
                            if health[i].should_attempt() {
                                health[i].last_attempt = Instant::now();
                                execution_engine::metrics::IBKR_RECONNECT_TOTAL
                                    .with_label_values(&[label])
                                    .inc();
                                match trader.reconnect().await {
                                    Ok(()) => {
                                        info!("IBKR trader {} reconnected successfully", label);
                                        health[i].consecutive_failures = 0;
                                        execution_engine::metrics::IBKR_CONNECTED
                                            .with_label_values(&[label])
                                            .set(1.0);
                                    }
                                    Err(e) => {
                                        health[i].consecutive_failures += 1;
                                        error!(
                                            "IBKR trader {} reconnect failed (attempt {}): {}",
                                            label, health[i].consecutive_failures, e
                                        );
                                    }
                                }
                            }
                        } else {
                            health[i].consecutive_failures = 0;
                            execution_engine::metrics::IBKR_CONNECTED
                                .with_label_values(&[label])
                                .set(1.0);
                        }
                    }

                    // Query each trader for account summary + positions (each sees its own sub-account)
                    let mut all_summaries = std::collections::HashMap::new();
                    let mut positions = Vec::new();
                    for (i, trader) in traders.iter().enumerate() {
                        let sync_start = Instant::now();
                        let label = trader_labels[i];
                        match trader.get_account_summaries().await {
                            Ok(sums) => {
                                for (acct, summary) in sums {
                                    all_summaries.insert(acct, summary);
                                }
                            }
                            Err(e) => {
                                warn!("IBKR account_summaries failed: {}", e);
                                if e.to_string().contains("timed out") {
                                    execution_engine::metrics::IBKR_API_TIMEOUT_TOTAL
                                        .with_label_values(&[label, "get_account_summaries"])
                                        .inc();
                                }
                            }
                        }
                        match trader.get_positions().await {
                            Ok(pos) => positions.extend(pos),
                            Err(e) => {
                                warn!("IBKR get_positions failed: {}", e);
                                if e.to_string().contains("timed out") {
                                    execution_engine::metrics::IBKR_API_TIMEOUT_TOTAL
                                        .with_label_values(&[label, "get_positions"])
                                        .inc();
                                }
                            }
                        }
                        execution_engine::metrics::IBKR_SYNC_DURATION
                            .with_label_values(&[label])
                            .set(sync_start.elapsed().as_secs_f64());
                    }

                    // Publish per-account portfolio updates with mode tag so
                    // strategy-engine can size signals using the target account's equity.
                    let account_mode_map = crate::reconciliation::build_account_map_public();
                    let now = Utc::now();
                    let mut published_modes = std::collections::HashSet::new();
                    for (ibkr_acct, summary) in &all_summaries {
                        let mode = account_mode_map.get(ibkr_acct).cloned();
                        let acct_positions: Vec<PositionUpdate> = positions
                            .iter()
                            .filter(|p| p.account == *ibkr_acct)
                            .map(|p| PositionUpdate {
                                symbol: p.symbol.clone(),
                                quantity: p.quantity,
                                market_value: p.market_value,
                            })
                            .collect();
                        let update = PortfolioUpdate {
                            timestamp: now,
                            cash: summary.cash,
                            positions: acct_positions,
                            total_equity: summary.net_liquidation,
                            mode: mode.clone(),
                        };
                        if let Ok(json) = serde_json::to_string(&update) {
                            let _: std::result::Result<(), _> =
                                con.publish("portfolio_updates", json);
                        }
                        if let Some(m) = mode {
                            published_modes.insert(m);
                        }
                    }

                    // Also publish aggregate for backward compatibility
                    let (total_cash, total_equity) = all_summaries
                        .values()
                        .fold((0.0, 0.0), |(c, e), s| (c + s.cash, e + s.net_liquidation));
                    let agg_update = PortfolioUpdate {
                        timestamp: now,
                        cash: total_cash,
                        positions: positions
                            .iter()
                            .map(|p| PositionUpdate {
                                symbol: p.symbol.clone(),
                                quantity: p.quantity,
                                market_value: p.market_value,
                            })
                            .collect(),
                        total_equity,
                        mode: None,
                    };
                    if let Ok(json) = serde_json::to_string(&agg_update) {
                        let _: std::result::Result<(), _> = con.publish("portfolio_updates", json);
                    }

                    // Write per-account cached broker data to trading_accounts table
                    if let Some(ref db_client) = db_for_sync {
                        info!(
                            "IBKR sync: {} accounts: {:?}",
                            all_summaries.len(),
                            all_summaries.keys().collect::<Vec<_>>()
                        );
                        for (acct_id, summary) in &all_summaries {
                            info!(
                                "  {} => net_liq={:.2}, cash={:.2}, buying_power={:.2}",
                                acct_id,
                                summary.net_liquidation,
                                summary.cash,
                                summary.buying_power
                            );
                            let res = db_client
                                .execute(
                                    "UPDATE trading_accounts \
                                     SET cached_net_liq = $1::float8, \
                                         cached_cash = $2::float8, \
                                         cached_buying_power = $3::float8, \
                                         cache_updated_at = NOW() \
                                     WHERE broker_account = $4",
                                    &[
                                        &summary.net_liquidation,
                                        &summary.cash,
                                        &summary.buying_power,
                                        acct_id,
                                    ],
                                )
                                .await;
                            if let Err(e) = res {
                                warn!("Failed to update cached broker data for {}: {}", acct_id, e);
                            }

                            // Upsert daily net-liq snapshot for day-over-day PnL
                            let snap_res = db_client
                                .execute(
                                    "INSERT INTO account_daily_snapshots \
                                         (account_id, snapshot_date, net_liquidation, cash_balance, buying_power) \
                                     SELECT account_id, CURRENT_DATE, $1::float8, $2::float8, $3::float8 \
                                     FROM trading_accounts WHERE broker_account = $4 \
                                     ON CONFLICT (account_id, snapshot_date) \
                                     DO UPDATE SET net_liquidation = EXCLUDED.net_liquidation, \
                                                   cash_balance = EXCLUDED.cash_balance, \
                                                   buying_power = EXCLUDED.buying_power",
                                    &[
                                        &summary.net_liquidation,
                                        &summary.cash,
                                        &summary.buying_power,
                                        acct_id,
                                    ],
                                )
                                .await;
                            if let Err(e) = snap_res {
                                warn!("Failed to upsert daily snapshot for {}: {}", acct_id, e);
                            }
                        }

                        // ── Position sync: write IBKR positions → trade_positions DB ──
                        reconciliation::sync_positions(&positions, db_client).await;

                        // Mark stale orders every ~5 min (10 cycles × 30s)
                        if sync_count.is_multiple_of(10) {
                            reconciliation::mark_stale_orders(db_client, Duration::from_secs(300))
                                .await;
                        }
                    }

                    sync_count += 1;
                    tokio::time::sleep(Duration::from_secs(30)).await;
                }
            });
        }
    }

    // ========================================
    // 5. Background: Futu Portfolio Sync
    // ========================================
    if let Some(trader) = futu.clone() {
        let redis_url_clone = redis_url.clone();

        tokio::spawn(async move {
            info!("Starting Futu Portfolio Sync Task...");
            let client = match redis::Client::open(redis_url_clone.as_str()) {
                Ok(c) => c,
                Err(e) => {
                    error!("Failed to create Redis client for Futu sync: {}", e);
                    return;
                }
            };
            let mut con = match client.get_connection() {
                Ok(c) => c,
                Err(e) => {
                    error!("Failed to connect to Redis for Futu sync: {}", e);
                    return;
                }
            };

            loop {
                let account = trader.get_account_summary().await.unwrap_or_default();
                let positions = trader.get_positions().await.unwrap_or_default();

                let update = PortfolioUpdate {
                    timestamp: Utc::now(),
                    cash: account.cash,
                    positions: positions
                        .iter()
                        .map(|p| PositionUpdate {
                            symbol: p.symbol.clone(),
                            quantity: p.quantity,
                            market_value: p.market_value,
                        })
                        .collect(),
                    total_equity: account.net_liquidation,
                    mode: None,
                };

                if let Ok(json) = serde_json::to_string(&update) {
                    let _: std::result::Result<(), _> = con.publish("portfolio_updates", json);
                }

                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        });
    }

    // ========================================
    // 6. Background: Heartbeat
    // ========================================
    common::heartbeat::spawn_heartbeat("execution-engine", &redis_url);

    info!("Execution Engine Ready. Listening for signals...");
    info!(
        "  IBKR long_only: {}  |  IBKR long_short: {}  |  Futu: {}",
        if ibkr_lo_on { "ON" } else { "OFF" },
        if ibkr_ls_on { "ON" } else { "OFF" },
        if futu_on { "ON" } else { "OFF" },
    );

    // ========================================
    // 7. Main Loop: Listen for trade signals
    // ========================================
    if let Err(e) = listener.listen_for_signals().await {
        error!("Listener Error: {}", e);
    }

    Ok(())
}
