use backtest_engine::vm::vm::StackVM;
use chrono::Utc;
use common::events::{OrderSide, OrderStatus, OrderType, TradeSignal};
use std::collections::HashMap;
use std::env;
use strategy_engine::event_bus::EventBus;
use strategy_engine::market_data_manager::MarketDataManager;
use strategy_engine::portfolio::PortfolioConfig;
use strategy_engine::risk::{is_stock_symbol, RiskEngine};
use tracing::{error, info, warn};

use serde::Deserialize;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
struct SymbolStrategy {
    formula: Vec<usize>,
    mode: String,
    strategy_id: String,
}

#[derive(Deserialize, Debug)]
struct StrategyUpdate {
    formula: Vec<usize>,
    #[serde(default)]
    symbol: Option<String>,
    #[serde(default)]
    mode: Option<String>,
    meta: StrategyMeta,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct StrategyMeta {
    name: String,
    description: String,
}

#[tokio::main]
#[allow(unreachable_code)]
async fn main() -> anyhow::Result<()> {
    if !common::telemetry::try_init_telemetry("strategy-engine") {
        tracing_subscriber::fmt::init();
    }
    info!("Starting Strategy Engine...");

    if let Err(e) = common::metrics::init_metrics("strategy-engine") {
        error!("Failed to initialize metrics: {}", e);
    }
    if let Err(e) = strategy_engine::metrics::init_strategy_metrics() {
        error!("Failed to initialize strategy metrics: {}", e);
    }

    tokio::spawn(common::health::start_health_server("strategy-engine", 8082));

    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let signal_threshold: f64 = env::var("SIGNAL_THRESHOLD")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.52);

    // 1. Setup Components
    let event_bus = EventBus::new(&redis_url)?;
    let mut market_manager = MarketDataManager::new();
    let vm = StackVM::new();
    let mut risk_engine = RiskEngine::new();

    // Two portfolio managers: crypto (default thresholds) and stock (tighter)
    let mut crypto_portfolio = strategy_engine::portfolio::PortfolioManager::new();
    let mut stock_portfolio = strategy_engine::portfolio::PortfolioManager::with_config(
        PortfolioConfig::stock_defaults(),
    );

    // Per-symbol strategy map (populated by evolved formulas from strategy-generator)
    let strategies: Arc<RwLock<HashMap<String, SymbolStrategy>>> =
        Arc::new(RwLock::new(HashMap::new()));

    // Fallback formula for symbols without an evolved strategy
    let fallback_formula: Vec<usize> = vec![0, 19]; // Volatility Breakout

    info!("Strategy Engine Initialized. Connecting to Event Bus...");

    // 2. Spawn Thread to Listen for Per-Symbol Formula Updates via Pattern Subscribe
    let formula_strategies = strategies.clone();
    let redis_url_clone = redis_url.clone();

    tokio::spawn(async move {
        let client = match redis::Client::open(redis_url_clone.as_str()) {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to create Redis client for strategy updates: {}", e);
                return;
            }
        };
        let mut con = match client.get_connection() {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to connect to Redis for strategy updates: {}", e);
                return;
            }
        };
        let mut pubsub = con.as_pubsub();

        // Pattern subscribe: strategy_updates:{exchange}:{symbol}:{mode}
        if let Err(e) = pubsub.psubscribe("strategy_updates:polygon:*:*") {
            error!("Failed to pattern subscribe: {}", e);
            return;
        }
        // Also subscribe to the legacy channel for backward compatibility
        if let Err(e) = pubsub.subscribe("strategy_updates") {
            warn!("Failed to subscribe to legacy strategy_updates: {}", e);
        }

        info!("Evolution Listener: Subscribed to strategy_updates:polygon:*:* pattern");

        common::heartbeat::spawn_heartbeat("strategy-engine", &redis_url_clone);

        loop {
            if let Ok(msg) = pubsub.get_message() {
                let channel: String = msg.get_channel_name().to_string();
                if let Ok(payload) = msg.get_payload::<String>() {
                    if let Ok(update) = serde_json::from_str::<StrategyUpdate>(&payload) {
                        // Parse symbol and mode from channel name or payload
                        let (symbol, mode) = parse_channel_or_payload(&channel, &update);

                        if let Some(sym) = symbol {
                            let strategy_id = update.meta.name.clone();
                            info!(
                                "Evolution Event: {} -> strategy '{}' (mode: {})",
                                sym,
                                strategy_id,
                                mode.as_deref().unwrap_or("unknown")
                            );

                            let mut w = formula_strategies.write().unwrap();
                            w.insert(
                                sym,
                                SymbolStrategy {
                                    formula: update.formula,
                                    mode: mode.unwrap_or_else(|| "long_only".to_string()),
                                    strategy_id,
                                },
                            );
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
    let mut order_rx = event_bus
        .subscribe_order_updates("order_updates")
        .await?;

    info!("Listening for Market Data, Portfolio Updates, and Order Updates...");

    // 4. Main Loop
    loop {
        tokio::select! {
            Some(update) = portfolio_rx.recv() => {
                info!("Portfolio Update: Cash={:.4}, Total={:.4}", update.cash, update.total_equity);
                risk_engine.update_equity(update.total_equity);
            }

            Some(order) = order_rx.recv() => {
                match order.status {
                    OrderStatus::Failed | OrderStatus::Cancelled | OrderStatus::Rejected => {
                        let is_stock = is_stock_symbol(&order.symbol);
                        let portfolio = if is_stock { &mut stock_portfolio } else { &mut crypto_portfolio };

                        if portfolio.positions.remove(&order.symbol).is_some() {
                            warn!(
                                "Order {} for {} {}: removed phantom position (open_positions: {})",
                                order.status, order.symbol,
                                order.message.as_deref().unwrap_or(""),
                                portfolio.positions.len()
                            );
                            strategy_engine::metrics::ACTIVE_POSITIONS.set(
                                (crypto_portfolio.positions.len()
                                    + stock_portfolio.positions.len()) as i64,
                            );
                        }
                    }
                    OrderStatus::Filled => {
                        info!(
                            "Order Filled: {} qty={:.4} avg_price={:.4}",
                            order.symbol, order.filled_quantity, order.filled_avg_price
                        );
                    }
                    _ => {}
                }
            }

            Some(msg) = market_rx.recv() => {
                strategy_engine::metrics::MARKET_DATA_CONSUMED.inc();
                let lag_secs = (Utc::now() - msg.timestamp).num_milliseconds() as f64 / 1000.0;
                if lag_secs >= 0.0 {
                    strategy_engine::metrics::MARKET_DATA_LAG_SECONDS.observe(lag_secs);
                }

                let is_stock = is_stock_symbol(&msg.symbol);
                let portfolio = if is_stock { &mut stock_portfolio } else { &mut crypto_portfolio };

                // 4.0 Update Portfolio Prices & Check Exits
                portfolio.update_price(&msg.symbol, msg.price);

                let exit_signals = portfolio.check_exits();
                for exit in exit_signals {
                    if let Some(pos) = portfolio.positions.get_mut(&exit.token_address) {
                        if pos.status == strategy_engine::portfolio::PositionStatus::Closing {
                            continue;
                        }
                        pos.status = strategy_engine::portfolio::PositionStatus::Closing;

                        info!(
                            "EXIT SIGNAL TRIGGERED: {} Reason: {:?}",
                            exit.symbol, exit.reason
                        );

                        strategy_engine::metrics::SIGNALS_GENERATED_TOTAL
                            .with_label_values(&["ExitLogic", "sell"])
                            .inc();

                        let signal = TradeSignal {
                            id: uuid::Uuid::new_v4(),
                            strategy_id: "ExitLogic".to_string(),
                            symbol: exit.symbol.clone(),
                            side: OrderSide::Sell,
                            quantity: exit.sell_ratio,
                            price: Some(msg.price),
                            order_type: OrderType::Market,
                            timestamp: Utc::now(),
                            reason: format!("Exit: {:?}", exit.reason),
                            exchange: if is_stock { Some("polygon".to_string()) } else { None },
                        };

                        if let Err(e) = event_bus.publish_signal(&signal).await {
                            error!("Failed to publish Exit signal: {}", e);
                        }

                        if exit.sell_ratio < 0.99 {
                            if let Some(p) = portfolio.positions.get_mut(&exit.token_address) {
                                p.is_moonbag = true;
                                p.status = strategy_engine::portfolio::PositionStatus::Active;
                            }
                        }
                    }
                }

                // 4.1 Update Buffer / Generate Features
                if let Some(features) = market_manager.on_update(msg.clone()) {
                    // 4.2 Look up per-symbol formula, fall back to default
                    let (current_formula, strategy_name, _mode) = {
                        let strats = strategies.read().unwrap();
                        if let Some(ss) = strats.get(&msg.symbol) {
                            (ss.formula.clone(), ss.strategy_id.clone(), ss.mode.clone())
                        } else {
                            (fallback_formula.clone(), "Fallback".to_string(), "long_only".to_string())
                        }
                    };

                    // Skip entry if we already hold this symbol
                    if portfolio.positions.contains_key(&msg.symbol) {
                        continue;
                    }

                    if let Some(result) = vm.execute(&current_formula, &features) {
                        if let Some(last_val) = result.last() {
                            // Use sigmoid threshold for evolved strategies, raw threshold for fallback
                            let threshold = if strategy_name == "Fallback" {
                                0.001
                            } else {
                                signal_threshold
                            };

                            // Apply sigmoid for evolved strategies to normalize output to [0, 1]
                            let signal_score = if strategy_name != "Fallback" {
                                1.0 / (1.0 + (-last_val).exp())
                            } else {
                                *last_val
                            };

                            if signal_score > threshold {
                                // Calculate position size
                                let (shares, exchange) = if is_stock {
                                    let s = risk_engine.calculate_stock_entry_shares(msg.price);
                                    if s <= 0.0 {
                                        warn!("Insufficient price for stock entry: {}", msg.symbol);
                                        continue;
                                    }
                                    (s, Some("polygon".to_string()))
                                } else {
                                    let amount_sol = risk_engine.calculate_entry_size();
                                    if amount_sol <= 0.0 {
                                        warn!("Insufficient equity/size for entry.");
                                        continue;
                                    }
                                    (amount_sol, None)
                                };

                                let signal = TradeSignal {
                                    id: uuid::Uuid::new_v4(),
                                    strategy_id: strategy_name.clone(),
                                    symbol: msg.symbol.clone(),
                                    side: OrderSide::Buy,
                                    quantity: shares,
                                    price: Some(msg.price),
                                    order_type: OrderType::Market,
                                    timestamp: Utc::now(),
                                    reason: format!("Entry Signal: {:.4} (thr: {:.4})", signal_score, threshold),
                                    exchange,
                                };

                                // Update open stock position count for risk engine
                                if is_stock {
                                    risk_engine.set_open_stock_positions(
                                        portfolio.positions.len(),
                                    );
                                }

                                let risk_approved = risk_engine.check(&signal, Some(10000.0)).await;
                                strategy_engine::metrics::RISK_CHECKS_TOTAL
                                    .with_label_values(&[if risk_approved { "approved" } else { "rejected" }])
                                    .inc();

                                if risk_approved {
                                    info!(
                                        "ENTRY SIGNAL: {} @ {} (Qty: {}, Strategy: {})",
                                        msg.symbol, msg.price, shares, strategy_name
                                    );

                                    strategy_engine::metrics::SIGNALS_GENERATED_TOTAL
                                        .with_label_values(&[&strategy_name, "buy"])
                                        .inc();

                                    if let Err(e) = event_bus.publish_signal(&signal).await {
                                        error!("Failed to publish Entry signal: {}", e);
                                    } else {
                                        let amount_held = if is_stock {
                                            shares
                                        } else {
                                            shares / msg.price
                                        };
                                        portfolio.add_position(
                                            msg.symbol.clone(),
                                            msg.symbol.clone(),
                                            msg.price,
                                            amount_held,
                                            msg.price,
                                        );
                                        strategy_engine::metrics::ACTIVE_POSITIONS.set(
                                            (crypto_portfolio.positions.len()
                                                + stock_portfolio.positions.len())
                                                as i64,
                                        );
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

/// Parse symbol and mode from Redis channel name (e.g., "strategy_updates:polygon:AAPL:long_only")
/// or fall back to payload fields.
fn parse_channel_or_payload(
    channel: &str,
    update: &StrategyUpdate,
) -> (Option<String>, Option<String>) {
    let parts: Vec<&str> = channel.split(':').collect();
    if parts.len() == 4 {
        // strategy_updates:exchange:symbol:mode
        let symbol = parts[2].to_string();
        let mode = parts[3].to_string();
        return (Some(symbol), Some(mode));
    }

    // Fall back to payload fields
    (update.symbol.clone(), update.mode.clone())
}
