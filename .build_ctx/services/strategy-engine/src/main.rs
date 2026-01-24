use tracing::{info, warn, error};
use strategy_engine::event_bus::EventBus;
use strategy_engine::market_data_manager::MarketDataManager;
use common::events::{TradeSignal, OrderSide, OrderType, StrategyLog};
use strategy_engine::risk::RiskEngine;
use backtest_engine::vm::vm::StackVM;
use chrono::Utc;
use std::env;

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
    
    use std::sync::{Arc, RwLock};
    use serde::Deserialize;

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
    let formula_tokens = Arc::new(RwLock::new(vec![0, 11]));
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
        pubsub.subscribe("strategy_updates").unwrap();
        
        info!("Evolution Listener: Connected to strategy_updates channel");

        loop {
            if let Ok(msg) = pubsub.get_message() {
                if let Ok(payload) = msg.get_payload::<String>() {
                    if let Ok(update) = serde_json::from_str::<StrategyUpdate>(&payload) {
                        info!("Evolution Event: Switching to strategy '{}'", update.meta.name);
                        
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
            // Small sleep to prevent tight loop if redis spams
             std::thread::sleep(std::time::Duration::from_millis(100));
        }
    });

    // 3. Subscribe to Market Data
    let mut market_rx = event_bus.subscribe_market_data("market_data").await?;

    info!("Listening for Market Data...");

    // 4. Main Loop
    while let Some(msg) = market_rx.recv().await {
        info!("Received update for {} @ {}", msg.symbol, msg.price);
        
        // 4.1 Update Buffer / Generate Features
        if let Some(features) = market_manager.on_update(msg.clone()) {
            // info!("Buffer saturated. Running AlphaGPT VM...");
            
            // 4.2 Run VM
            // Acquire Read Lock on current formula
            let current_formula = { formula_tokens.read().unwrap().clone() };
            let strategy_name = { current_strategy_name.read().unwrap().clone() };

            if let Some(result) = vm.execute(&current_formula, &features) {
                // Get last value
                if let Some(last_val) = result.last() {
                     // Log the calculation
                     let _ = event_bus.publish_strategy_log(&StrategyLog {
                         timestamp: Utc::now(),
                         strategy_id: "alpha_gpt_param".to_string(), // Fixed ID or dynamic?
                         symbol: msg.symbol.clone(),
                         action: "Analyzing".to_string(),
                         message: format!("[{}] Value: {:.4}", strategy_name, last_val),
                     }).await;

                     // 4.3 Signal Logic (Generic Threshold > 0.5 or < -0.5 ?)
                     // For flexible logic, usually the Formula outputs a Signal Score directly.
                     // But here we had custom logic "if val > 2.0".
                     // Let's generalize: If Abs(Val) > Threshold (e.g., 0.05 for Return, 2.0 for ZScore)
                     // Since units change, we might need dynamic thresholds too.
                     // For DEMO: Just assume threshold 0.1 for now as we might use raw returns
                     
                     let threshold = 0.01; // Lower threshold to trigger more often for demo
                     
                     let side = if *last_val > threshold {
                         Some(OrderSide::Buy)
                     } else if *last_val < -threshold {
                         Some(OrderSide::Sell)
                     } else {
                        // Log rejection reasoning (optional sample)
                        if rand::random::<f64>() < 0.1 {
                             let _ = event_bus.publish_strategy_log(&StrategyLog {
                                 timestamp: Utc::now(),
                                 strategy_id: "alpha_gpt_param".to_string(),
                                 symbol: msg.symbol.clone(),
                                 action: "No Signal".to_string(),
                                 message: format!("Value {:.4} within threshold", last_val),
                             }).await;
                        }
                        None
                    };

                    if let Some(side_enum) = side {
                         // 4.4 Create Signal
                         let signal = TradeSignal {
                             id: uuid::Uuid::new_v4(),
                             strategy_id: strategy_name.clone(),
                             symbol: msg.symbol.clone(),
                             side: side_enum.clone(),
                             quantity: 1.0, 
                             price: None, 
                             order_type: OrderType::Market,
                             timestamp: Utc::now(),
                             reason: format!("{} Signal: {:.4}", strategy_name, last_val),
                         };

    // 4.6 Risk Check
                         if risk_engine.check(&signal, Some(1_000_000.0)) { 
                             info!("Signal Generated: {:?} {} @ {}", side_enum, msg.symbol, msg.price);
                             
                             // 4.7 Publish
                             if let Err(e) = event_bus.publish_signal(&signal).await {
                                 error!("Failed to publish signal: {}", e);
                             }
                         } else {
                             warn!("Signal Rejected by Risk Engine: {} {:?}", msg.symbol, side_enum);
                         }
                    }
                }
            }
        }
    }
    
    Ok(())
}

