use backtest_engine::vm::vm::StackVM;
use chrono::Utc;
use common::events::{OrderSide, OrderType, TradeSignal};
use std::env;
use strategy_engine::event_bus::EventBus;
use strategy_engine::market_data_manager::MarketDataManager;
use strategy_engine::risk::RiskEngine;
use tracing::{error, info, warn};

#[tokio::main]
#[allow(unreachable_code)]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing (with OpenTelemetry if OTEL_EXPORTER_OTLP_ENDPOINT is set)
    if !common::telemetry::try_init_telemetry("strategy-engine") {
        tracing_subscriber::fmt::init();
    }
    info!("Starting Strategy Engine...");

    // Initialize Prometheus metrics
    if let Err(e) = common::metrics::init_metrics("strategy-engine") {
        error!("Failed to initialize metrics: {}", e);
    }
    if let Err(e) = strategy_engine::metrics::init_strategy_metrics() {
        error!("Failed to initialize strategy metrics: {}", e);
    }

    // Spawn health check server (also serves /metrics when metrics feature enabled)
    tokio::spawn(common::health::start_health_server("strategy-engine", 8082));

    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    // 1. Setup Components
    let event_bus = EventBus::new(&redis_url)?;
    let mut market_manager = MarketDataManager::new();
    let vm = StackVM::new(); // In real app, load this from config/DB
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
    #[allow(dead_code)]
    struct StrategyMeta {
        name: String,
        description: String,
    }

    // 2. Load Initial Strategy Logic
    // Try to load from file, otherwise use default
    let strategy_file = "best_meme_strategy.json";
    let (initial_formula, initial_name) = match std::fs::read_to_string(strategy_file) {
        Ok(content) => match serde_json::from_str::<StrategyUpdate>(&content) {
            Ok(update) => {
                info!("Loaded strategy from file: {}", update.meta.name);
                (update.formula, update.meta.name)
            }
            Err(e) => {
                error!("Failed to parse strategy file: {}", e);
                (vec![0, 19], "Volatility Breakout (Fallback)".to_string())
            }
        },
        Err(_) => {
            warn!("Strategy file not found, using default.");
            (vec![0, 19], "Volatility Breakout (Base)".to_string())
        }
    };

    let formula_tokens = Arc::new(RwLock::new(initial_formula));
    let current_strategy_name = Arc::new(RwLock::new(initial_name));

    info!("Strategy Engine Initialized. Connecting to Event Bus...");

    // 2.1 Spawn Thread to Listen for Formula Updates
    let _bus_clone = EventBus::new(&redis_url)?;
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

        // Spawn Heartbeat Loop
        common::heartbeat::spawn_heartbeat("strategy-engine", &redis_url);

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
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    });

    // 3. Subscribe to Market Data & Portfolio Updates
    let mut market_rx = event_bus.subscribe_market_data("market_data").await?;
    let mut portfolio_rx = event_bus
        .subscribe_portfolio_updates("portfolio_updates")
        .await?;

    info!("Listening for Market Data and Portfolio Updates...");

    // 4. Main Loop
    loop {
        tokio::select! {
            Some(update) = portfolio_rx.recv() => {
                info!("Portfolio Update: Cash={:.4}, Total={:.4}", update.cash, update.total_equity);
                risk_engine.update_equity(update.total_equity);
                // Also update portfolio manager cash if needed, but RiskEngine is the gatekeeper here.
            }

            Some(msg) = market_rx.recv() => {
                // Debug Log
                info!("Market Data Received: {} Price: {}", msg.symbol, msg.price);

                strategy_engine::metrics::MARKET_DATA_CONSUMED.inc();
                let lag_secs = (Utc::now() - msg.timestamp).num_milliseconds() as f64 / 1000.0;
                if lag_secs >= 0.0 {
                    strategy_engine::metrics::MARKET_DATA_LAG_SECONDS.observe(lag_secs);
                }

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

                        strategy_engine::metrics::SIGNALS_GENERATED_TOTAL
                            .with_label_values(&["ExitLogic", "sell"])
                            .inc();

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

                        if exit.sell_ratio < 0.99 {
                            // Partial sell (Moonbag). Mark is_moonbag = true.
                            if let Some(p) = portfolio_manager.positions.get_mut(&exit.token_address) {
                                p.is_moonbag = true;
                                p.status = strategy_engine::portfolio::PositionStatus::Active;
                            }
                        }
                    }
                }

                // 4.1 Update Buffer / Generate Features
                if let Some(features) = market_manager.on_update(msg.clone()) {
                    // 4.2 Run VM (Entry Logic)
                    let current_formula = { formula_tokens.read().unwrap().clone() };
                    let strategy_name = { current_strategy_name.read().unwrap().clone() };

                    // Start Entry Logic only if we don't hold it
                    if portfolio_manager.positions.contains_key(&msg.symbol) {
                        continue;
                    }

                    if let Some(result) = vm.execute(&current_formula, &features) {
                        if let Some(last_val) = result.last() {
                            let threshold = 0.001;

                            if *last_val > threshold {
                                // ENTRY SIGNAL
                                // Calculate Size
                                let amount_sol = risk_engine.calculate_entry_size();
                                if amount_sol <= 0.0 {
                                    warn!("Insufficient equity/size for entry.");
                                    continue;
                                }

                                // Construct Signal
                                let signal = TradeSignal {
                                    id: uuid::Uuid::new_v4(),
                                    strategy_id: strategy_name.clone(),
                                    symbol: msg.symbol.clone(),
                                    side: OrderSide::Buy,
                                    quantity: amount_sol,
                                    price: Some(msg.price),
                                    order_type: OrderType::Market,
                                    timestamp: Utc::now(),
                                    reason: format!("Entry Signal: {:.4}", last_val),
                                };

                                // Async Risk Check (with Honeypot)
                                let risk_approved = risk_engine.check(&signal, Some(10000.0)).await;
                                strategy_engine::metrics::RISK_CHECKS_TOTAL
                                    .with_label_values(&[if risk_approved { "approved" } else { "rejected" }])
                                    .inc();
                                if risk_approved {
                                    // Valid Signal
                                    info!("ENTRY SIGNAL: {} @ {} (Amt: {} SOL)", msg.symbol, msg.price, amount_sol);

                                    strategy_engine::metrics::SIGNALS_GENERATED_TOTAL
                                        .with_label_values(&[&strategy_name, "buy"])
                                        .inc();

                                    if let Err(e) = event_bus.publish_signal(&signal).await {
                                        error!("Failed to publish Entry signal: {}", e);
                                    } else {
                                        // Optimistic Portfolio Update
                                        portfolio_manager.add_position(
                                            msg.symbol.clone(),
                                            msg.symbol.clone(),
                                            msg.price,
                                            amount_sol / msg.price,
                                            msg.price,
                                        );
                                        strategy_engine::metrics::ACTIVE_POSITIONS
                                            .set(portfolio_manager.positions.len() as i64);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
