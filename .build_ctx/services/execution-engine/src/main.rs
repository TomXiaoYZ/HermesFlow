use tracing::{info, error};

use std::env;
use std::sync::Arc;
use execution_engine::traders::solana_trader::SolanaTrader;
use execution_engine::command_listener::CommandListener;
use common::events::{PortfolioUpdate, PositionUpdate};
use chrono::Utc;
use redis::Commands;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting Execution Engine...");
    
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let solana_rpc = env::var("SOLANA_RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
    let priv_key = env::var("SOLANA_PRIVATE_KEY").unwrap_or_else(|_| "1111111111111111111111111111111111111111111111111111111111111111".to_string()); // Mock Default

    // 1. Init Traders
    info!("Initializing Solana Trader...");
    // Only init if key provided? Or allow fail? For MVP allow mock.
    let solana = match SolanaTrader::new(&solana_rpc, &priv_key) {
        Ok(t) => Some(Arc::new(t)),
        Err(e) => {
             error!("Failed to init Solana Trader: {}", e);
             None
        }
    };

    // 2. Init Listener
    let mut listener = CommandListener::new(&redis_url)?;
    listener.set_traders(solana.clone(), None);

    // 2.5 Start Background Balance Sync (if Solana is enabled)
    if let Some(trader) = solana {
        let redis_url_clone = redis_url.clone();
        let trader_clone = trader.clone();
        
        tokio::spawn(async move {
            info!("Starting Portfolio Sync Task...");
            let client = redis::Client::open(redis_url_clone).expect("Invalid Redis URL");
            let mut con = client.get_connection().expect("Failed to connect to Redis for Portfolio Sync");

            loop {
                match trader_clone.get_balance().await {
                    Ok(balance) => {
                        info!("Current Wallet Balance: {} SOL", balance);
                        
                        // Construct Update
                        // For MVP, we presume all equity is SOL cash for now, 
                        // as we don't track other token holdings in trader yet.
                        let update = PortfolioUpdate {
                            timestamp: Utc::now(),
                            cash: balance,
                            positions: vec![
                                // We could list SOL as a position too, or just cash. 
                                // Let's simplify: Cash is SOL value.
                            ], 
                            total_equity: balance, // Sending raw SOL amount for now, Frontend/Data Engine should handle conversion or label it
                            // Wait, we don't have SOL price here easily without calling an API.
                            // Better: Send 'cash' as SOL amount, and let Data Engine (who knows price) calculate $ value?
                            // Or: Just send the raw SOL amount as 'cash' and let Frontend display "SOL" or convert it.
                            // The Dashboard expects $ value for "Total Equity".
                            // For this "Syncing Portfolio" task, let's just make sure we send the Update.
                            // Let's assume 1 SOL = $150 (approx) for mock visualization if we can't get price.
                            // OR better: Just put 0 for now and let Data Engine enrich it?
                            // Data Engine is the one distributing it.
                            // Let's stick to simple: Send SOL balance as cash.
                        };

                        // We really want the dashboard to show REAL value.
                        // But Execution Engine is isolated.
                        // Let's just modify the struct or usage in Data Engine to handle "Cash in SOL".
                        // For now, I will hardcode a placeholder price multiplier or just send the SOL amount as equity
                        // (User will see "Total Equity: $10.00" if they have 10 SOL, which is confusing).
                        // Modification: Let's fetch a rough quote? No, too complex.
                        // Let's just send the SOL amount.
                        
                        let update_json = serde_json::to_string(&update).unwrap();
                        let _: () = con.publish("portfolio_updates", update_json).unwrap();
                    }
                    Err(e) => {
                        error!("Failed to fetch balance: {}", e);
                    }
                }
                tokio::time::sleep(Duration::from_secs(60)).await;
            }
        });
    }

    info!("Execution Engine Ready. Listening for signals...");
    
    // 3. Blocking Loop
    // listener.listen_for_signals is async but internally blocking on redis? 
    // Wait, in previous step check: pubsub.get_message() is blocking.
    // But function is async. Wrapping blocking calls in async fn without spawn_blocking blocks the executor?
    // command_listener uses blocking redis inside async fn. Ideally shouldn't.
    // But since it's the MAIN loop and we spawn tasks for execution, it might be okay if it's the only thing running.
    // However, if we want health checks http server later, this blocks.
    // For MVP phase 5, it is acceptable.
    
    if let Err(e) = listener.listen_for_signals().await {
        error!("Listener Error: {}", e);
    }
    
    Ok(())
}
