use tracing::{error, info, warn};

use chrono::Utc;
use common::events::{PortfolioUpdate, PositionUpdate};
use execution_engine::command_listener::CommandListener;
use execution_engine::traders::futu_trader::FutuTrader;
use execution_engine::traders::ibkr_trader::IBKRTrader;
use execution_engine::traders::solana_trader::SolanaTrader;
use execution_engine::traders::Trader;
use redis::Commands;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio_postgres::NoTls;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting Execution Engine...");

    // Spawn health check server
    tokio::spawn(common::health::start_health_server(
        "execution-engine",
        8083,
    ));

    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let solana_rpc = env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
    let priv_key = env::var("SOLANA_PRIVATE_KEY")
        .unwrap_or_else(|_| {
            "1111111111111111111111111111111111111111111111111111111111111111".to_string()
        })
        .trim()
        .to_string();

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
    // 1. Initialize Solana Trader
    // ========================================
    info!("Initializing Solana Trader...");
    let solana = match SolanaTrader::new(&solana_rpc, &priv_key) {
        Ok(t) => {
            info!("Solana Trader initialized");
            Some(Arc::new(t))
        }
        Err(e) => {
            warn!("Solana Trader not available: {}", e);
            None
        }
    };

    // ========================================
    // 2. Initialize IBKR Trader
    // ========================================
    let ibkr_host = env::var("IBKR_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let ibkr_port: u32 = env::var("IBKR_PORT")
        .unwrap_or_else(|_| "7497".to_string())
        .parse()
        .unwrap_or(7497);
    let ibkr_client_id: u32 = env::var("IBKR_CLIENT_ID")
        .unwrap_or_else(|_| "1".to_string())
        .parse()
        .unwrap_or(1);

    info!(
        "Initializing IBKR Trader ({}:{}, client_id={})...",
        ibkr_host, ibkr_port, ibkr_client_id
    );
    let ibkr = match IBKRTrader::new(&ibkr_host, ibkr_port, ibkr_client_id).await {
        Ok(t) => {
            info!("IBKR Trader connected");
            Some(Arc::new(t))
        }
        Err(e) => {
            warn!(
                "IBKR Trader not available (TWS/Gateway not running?): {}",
                e
            );
            None
        }
    };

    // ========================================
    // 2b. Sync order ID counter with DB max to avoid uniqueness conflicts
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
    // 3. Initialize Futu Trader
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
    // 4. Setup Command Listener
    // ========================================
    let mut listener = CommandListener::new(&redis_url, db)?;
    listener.set_traders(solana.clone(), ibkr.clone(), futu.clone());

    // Record status before moves
    let solana_on = solana.is_some();
    let ibkr_on = ibkr.is_some();
    let futu_on = futu.is_some();

    // ========================================
    // 5. Background: Solana Portfolio Sync
    // ========================================
    if let Some(trader) = solana {
        let redis_url_clone = redis_url.clone();
        let trader_clone = trader.clone();

        tokio::spawn(async move {
            info!("Starting Solana Portfolio Sync Task...");
            let client = match redis::Client::open(redis_url_clone.as_str()) {
                Ok(c) => c,
                Err(e) => {
                    error!("Failed to create Redis client for Solana sync: {}", e);
                    return;
                }
            };
            let mut con = match client.get_connection() {
                Ok(c) => c,
                Err(e) => {
                    error!("Failed to connect to Redis for Solana sync: {}", e);
                    return;
                }
            };

            loop {
                match trader_clone.get_balance().await {
                    Ok(balance) => {
                        let update = PortfolioUpdate {
                            timestamp: Utc::now(),
                            cash: balance,
                            positions: vec![],
                            total_equity: balance,
                        };
                        if let Ok(json) = serde_json::to_string(&update) {
                            let _: std::result::Result<(), _> =
                                con.publish("portfolio_updates", json);
                        }
                    }
                    Err(e) => {
                        error!("Failed to fetch Solana balance: {}", e);
                    }
                }
                tokio::time::sleep(Duration::from_secs(60)).await;
            }
        });
    }

    // ========================================
    // 6. Background: IBKR Portfolio Sync
    // ========================================
    if let Some(trader) = ibkr.clone() {
        let redis_url_clone = redis_url.clone();

        tokio::spawn(async move {
            info!("Starting IBKR Portfolio Sync Task...");
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
                };

                if let Ok(json) = serde_json::to_string(&update) {
                    let _: std::result::Result<(), _> = con.publish("portfolio_updates", json);
                }

                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        });
    }

    // ========================================
    // 6b. Background: Futu Portfolio Sync
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
                };

                if let Ok(json) = serde_json::to_string(&update) {
                    let _: std::result::Result<(), _> = con.publish("portfolio_updates", json);
                }

                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        });
    }

    // ========================================
    // 7. Background: Heartbeat
    // ========================================
    common::heartbeat::spawn_heartbeat("execution-engine", &redis_url);

    info!("Execution Engine Ready. Listening for signals...");
    info!(
        "  Solana: {}  |  IBKR: {}  |  Futu: {}",
        if solana_on { "ON" } else { "OFF" },
        if ibkr_on { "ON" } else { "OFF" },
        if futu_on { "ON" } else { "OFF" },
    );

    // ========================================
    // 8. Main Loop: Listen for trade signals
    // ========================================
    if let Err(e) = listener.listen_for_signals().await {
        error!("Listener Error: {}", e);
    }

    Ok(())
}
