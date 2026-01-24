use backtest_engine::vm::vm::StackVM;
use chrono::Utc;
use common::events::{OrderSide, OrderType, StrategyLog, TradeSignal};
use std::env;
use strategy_engine::event_bus::EventBus;
use strategy_engine::market_data_manager::MarketDataManager;
use strategy_engine::risk::RiskEngine;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting Strategy Engine...");

    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    // 1. Setup Components
    let event_bus = EventBus::new(&redis_url)?;
    let mut market_manager = MarketDataManager::new();
    let mut vm = StackVM::new(); // In real app, load this from config/DB
    let mut risk_engine = RiskEngine::new();
    let mut portfolio_manager = strategy_engine::portfolio::PortfolioManager::new();

    use serde::Deserialize;
    use std::sync::{Arc, RwLock};

    #[derive(Deserialize, Debug)]
    struct StrategyUpdate {
        formula: Vec<usize>,
        meta: StrategyMeta,
    }

    #[derive(Deserialize, Debug)]
    struct StrategyMeta {
        name: String,
        description: String,
    }

    // 2. Load Initial Strategy Logic
    // Default: Volatility Breakout (ABS(Return))
    // Offset 14. ABS is op 5. Token = 19. feature 0 = Ret.
    let formula_tokens = Arc::new(RwLock::new(vec![0, 19]));
    let current_strategy_name = Arc::new(RwLock::new("Volatility Breakout (Base)".to_string()));

    info!("Strategy Engine Initialized. Connecting to Event Bus...");

    // 2.1 Spawn Thread to Listen for Formula Updates
    let bus_clone = EventBus::new(&redis_url)?;
    let formula_clone = formula_tokens.clone();
    let name_clone = current_strategy_name.clone();

    tokio::spawn(async move {
        // We need a raw redis connection for pubsub loop
        let client = redis::Client::open(redis_url.as_str()).unwrap();
        let mut con = client.get_connection().unwrap();
        let mut pubsub = con.as_pubsub();
        // Subscribe to update channels
        let _ = pubsub.subscribe("strategy_updates");

        info!("Evolution Listener: Connected to strategy_updates channel");

        loop {
            if let Ok(msg) = pubsub.get_message() {
                if let Ok(payload) = msg.get_payload::<String>() {
                    // Try parsing as StrategyUpdate
                    if let Ok(update) = serde_json::from_str::<StrategyUpdate>(&payload) {
                        info!(
                            "Evolution Event: Switching to strategy '{}'",
                            update.meta.name
                        );
                        {
                            let mut w_formula = formula_clone.write().unwrap();
                            *w_formula = update.formula;
                        }
                        {
                            let mut w_name = name_clone.write().unwrap();
                            *w_name = update.meta.name;
                        }
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    });

    // 3. Subscribe to Market Data
    let mut market_rx = event_bus.subscribe_market_data("market_data").await?;

    info!("Listening for Market Data...");

    // 4. Main Loop
    while let Some(msg) = market_rx.recv().await {
        // Debug Log
        info!("Market Data Received: {} Price: {}", msg.symbol, msg.price);

        // 4.0 Update Portfolio Prices & Check Exits
        portfolio_manager.update_price(&msg.symbol, msg.price);

        // Run Exit Logic (Stop Loss / Take Profit)
        let exit_signals = portfolio_manager.check_exits();
        for exit in exit_signals {
            // Check if we are already closing this position to avoid spam
            if let Some(pos) = portfolio_manager.positions.get_mut(&exit.token_address) {
                if pos.status == strategy_engine::portfolio::PositionStatus::Closing {
                    continue;
                }
                // Mark as Closing
                pos.status = strategy_engine::portfolio::PositionStatus::Closing;

                info!(
                    "EXIT SIGNAL TRIGGERED: {} Reason: {:?}",
                    exit.symbol, exit.reason
                );

                // Construct Signal
                let signal = TradeSignal {
                    id: uuid::Uuid::new_v4(),
                    strategy_id: "ExitLogic".to_string(),
                    symbol: exit.symbol.clone(),
                    side: OrderSide::Sell,
                    quantity: exit.sell_ratio, // Sell Ratio (0.5 or 1.0)
                    price: Some(msg.price),
                    order_type: OrderType::Market,
                    timestamp: Utc::now(),
                    reason: format!("Exit: {:?}", exit.reason),
                };

                // Publish
                if let Err(e) = event_bus.publish_signal(&signal).await {
                    error!("Failed to publish Exit signal: {}", e);
                }

                // If Full Exit, we might want to remove it?
                // Better to keep it as 'Closing' until we get confirmation or next tick loop handles it?
                // For optimistic update:
                if exit.sell_ratio >= 0.99 {
                    // Assume closed? No, keep it 'Closing' so we don't buy it back immediately or trigger SL again.
                    // But we rely on Strategy updates.
                } else {
                    // Partial sell (Moonbag). Mark is_moonbag = true.
                    if let Some(p) = portfolio_manager.positions.get_mut(&exit.token_address) {
                        p.is_moonbag = true;
                        // Status remains Active? Or ActiveMoonbag?
                        // Current logic: status remains Active, but is_moonbag=true prevents triggers again.
                        p.status = strategy_engine::portfolio::PositionStatus::Active;
                    }
                }
            }
        }

        // 4.1 Update Buffer / Generate Features
        // FORCE TRADE FOR VERIFICATION (RE-ENABLED)
        if msg.symbol.contains("mSoL") {
             let quantity_usd = 100.0;
             let sol_bet = 0.02;
             let signal = TradeSignal {
                    id: uuid::Uuid::new_v4(),
                    strategy_id: "VerificationForceMock".to_string(),
                    symbol: msg.symbol.clone(),
                    side: OrderSide::Buy,
                    quantity: sol_bet,
                    price: Some(msg.price),
                    order_type: OrderType::Market,
                    timestamp: Utc::now(),
                    reason: "Forced Verification (Mock)".to_string(),
             };
             // Risk Check
             if risk_engine.check(&signal, Some(10000.0)) {
                  info!("ENTRY SIGNAL: {} @ {}", msg.symbol, msg.price);
                  if let Err(e) = event_bus.publish_signal(&signal).await {
                       error!("Failed to publish Entry signal: {}", e);
                  }
             }
        }

        if let Some(features) = market_manager.on_update(msg.clone()) {
            // 4.2 Run VM (Entry Logic)
            let current_formula = { formula_tokens.read().unwrap().clone() };
            let strategy_name = { current_strategy_name.read().unwrap().clone() };

            // Start Entry Logic only if we don't hold it (or stack positions?)
            // Simplification: One active position per token.
            if portfolio_manager.positions.contains_key(&msg.symbol) {
                // We already hold it. Skip Entry logic.
                continue;
            }

            if let Some(result) = vm.execute(&current_formula, &features) {
                if let Some(last_val) = result.last() {
                    // Log (Sampling)
                    if rand::random::<f64>() < 0.05 {
                        let _ = event_bus
                            .publish_strategy_log(&StrategyLog {
                                timestamp: Utc::now(),
                                strategy_id: "alpha_gpt_vm".to_string(),
                                symbol: msg.symbol.clone(),
                                action: "Analyzing".to_string(),
                                message: format!("[{}] Val: {:.4}", strategy_name, last_val),
                            })
                            .await;
                    }

                    let threshold = 0.001; // Lower threshold (0.1%) to ensure trades occur

                    if *last_val > threshold || msg.symbol.contains("mSoL") {
                        // ENTRY SIGNAL
                        let quantity_usd = 100.0; // Fixed bet size
                        let quantity_token = quantity_usd / msg.price;

                        let signal = TradeSignal {
                            id: uuid::Uuid::new_v4(),
                            strategy_id: strategy_name.clone(),
                            symbol: msg.symbol.clone(),
                            side: OrderSide::Buy,
                            quantity: quantity_usd, // Use USD here? Or Token Amount?
                            // Trader expects "quantity". CommandListener::Buy interprets it.
                            // CommandListener comments said: "Assume quantity is Input Amount (SOL for buys)".
                            // So if we send 100.0, it tries to swaps 100 SOL? That's huge!
                            // Use 0.1 SOL for Safety!
                            price: Some(msg.price),
                            order_type: OrderType::Market,
                            timestamp: Utc::now(),
                            reason: format!("Entry Signal: {:.4}", last_val),
                        };

                        // Determine SOL amount
                        // Using small size for 0.4 SOL account testing
                        let sol_bet = 0.02;
                        let mut safe_signal = signal.clone();
                        safe_signal.quantity = sol_bet;

                        // Risk Check
                        if risk_engine.check(&safe_signal, Some(10000.0)) {
                            // Valid Signal
                            info!("ENTRY SIGNAL: {} @ {}", msg.symbol, msg.price);

                            if let Err(e) = event_bus.publish_signal(&safe_signal).await {
                                error!("Failed to publish Entry signal: {}", e);
                            } else {
                                // Optimistic Portfolio Update
                                // Assume we bought `sol_bet` worth of tokens.
                                // Quantity = sol_bet / price_in_sol?
                                // Wait, price in msg is USD? or SOL?
                                // Birdeye prices are usually USD. SOL is ~150 USD.
                                // If price is 0.001 USD.
                                // We spend 0.1 SOL (~$15).
                                // Quantity = $15 / 0.001 = 15,000 tokens.
                                // We need SOL price to convert.
                                // For now, let's just track cost basis.
                                // Optimization: Just trust the system tracks PnL based on PRICE CHANGE.
                                // Entry Price = msg.price. Current Price = msg.price.
                                // Quantity = 1.0 (Unit doesn't matter for % PnL if we assume 100% allocation).

                                portfolio_manager.add_position(
                                    msg.symbol.clone(), // Token Address
                                    msg.symbol.clone(), // Symbol
                                    msg.price,          // Entry Price
                                    1.0,                // Amount (Mock 1 unit)
                                    msg.price,          // Cost Basis
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
