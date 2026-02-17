use backtest_engine::vm::vm::StackVM;
use chrono::Utc;
use common::events::{OrderSide, OrderStatus, OrderType, TradeSignal};
use std::collections::HashMap;
use std::env;
use strategy_engine::event_bus::EventBus;
use strategy_engine::market_data_manager::MarketDataManager;
use strategy_engine::portfolio::{
    PortfolioConfig, PortfolioManager, PositionDirection, PositionStatus,
};
use strategy_engine::risk::{is_stock_symbol, RiskEngine};
use strategy_engine::signal_buffer::SignalBuffer;
use tracing::{error, info, warn};

use serde::Deserialize;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
struct SymbolStrategy {
    formula: Vec<usize>,
    #[allow(dead_code)]
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

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

/// Count total positions across all portfolios.
fn total_positions(
    crypto: &PortfolioManager,
    stock_lo: &PortfolioManager,
    stock_ls: &PortfolioManager,
) -> i64 {
    (crypto.positions.len() + stock_lo.positions.len() + stock_ls.positions.len()) as i64
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
    let signal_lower_threshold: f64 = env::var("SIGNAL_LOWER_THRESHOLD")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.35);

    // 1. Setup Components
    let event_bus = EventBus::new(&redis_url)?;
    let mut market_manager = MarketDataManager::new();
    let vm = StackVM::new();
    let mut risk_engine = RiskEngine::new();

    // Three portfolio managers: crypto, stock long_only, stock long_short
    let mut crypto_portfolio = PortfolioManager::new();
    let mut stock_portfolio_long_only =
        PortfolioManager::with_config(PortfolioConfig::stock_defaults());
    let mut stock_portfolio_long_short =
        PortfolioManager::with_config(PortfolioConfig::stock_defaults());

    // Per-(symbol, mode) strategy map (populated by evolved formulas from strategy-generator)
    let strategies: Arc<RwLock<HashMap<(String, String), SymbolStrategy>>> =
        Arc::new(RwLock::new(HashMap::new()));

    // Fallback formula for symbols without an evolved strategy (long_only only)
    let fallback_formula: Vec<usize> = vec![0, 19]; // Volatility Breakout

    // Rolling sigmoid buffer for adaptive thresholds
    let mut signal_buffer = SignalBuffer::new();

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
                        let (symbol, mode) = parse_channel_or_payload(&channel, &update);

                        if let Some(sym) = symbol {
                            let mode_str = mode.unwrap_or_else(|| "long_only".to_string());
                            let strategy_id = update.meta.name.clone();
                            info!(
                                "Evolution Event: {} -> strategy '{}' (mode: {})",
                                sym, strategy_id, mode_str
                            );

                            let mut w = formula_strategies.write().unwrap();
                            w.insert(
                                (sym, mode_str.clone()),
                                SymbolStrategy {
                                    formula: update.formula,
                                    mode: mode_str,
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
    let mut order_rx = event_bus.subscribe_order_updates("order_updates").await?;

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
                        let removed = if is_stock {
                            let r1 = stock_portfolio_long_only.positions.remove(&order.symbol).is_some();
                            let r2 = stock_portfolio_long_short.positions.remove(&order.symbol).is_some();
                            r1 || r2
                        } else {
                            crypto_portfolio.positions.remove(&order.symbol).is_some()
                        };

                        if removed {
                            warn!(
                                "Order {} for {} {}: removed phantom position",
                                order.status, order.symbol,
                                order.message.as_deref().unwrap_or(""),
                            );
                            strategy_engine::metrics::ACTIVE_POSITIONS.set(
                                total_positions(&crypto_portfolio, &stock_portfolio_long_only, &stock_portfolio_long_short),
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

                // 4.0 Update prices on ALL relevant portfolios
                if is_stock {
                    stock_portfolio_long_only.update_price(&msg.symbol, msg.price);
                    stock_portfolio_long_short.update_price(&msg.symbol, msg.price);
                } else {
                    crypto_portfolio.update_price(&msg.symbol, msg.price);
                }

                // 4.0b Check exits on each portfolio separately to avoid borrow conflicts
                process_exits(
                    &mut stock_portfolio_long_only, "long_only", is_stock,
                    &msg.symbol, msg.price, &event_bus, true,
                ).await;
                process_exits(
                    &mut stock_portfolio_long_short, "long_short", is_stock,
                    &msg.symbol, msg.price, &event_bus, true,
                ).await;
                if !is_stock {
                    process_exits(
                        &mut crypto_portfolio, "long_only", is_stock,
                        &msg.symbol, msg.price, &event_bus, false,
                    ).await;
                }

                // 4.1 Update Buffer / Generate Features (once per symbol, shared by both modes)
                if let Some(features) = market_manager.on_update(msg.clone()) {
                    // Collect strategies we need, then drop the read lock
                    let mode_formulas: Vec<(String, Vec<usize>, String)> = {
                        let strats = strategies.read().unwrap();
                        let mut collected = Vec::new();
                        for mode_str in &["long_only", "long_short"] {
                            let key = (msg.symbol.clone(), mode_str.to_string());
                            match strats.get(&key) {
                                Some(ss) => {
                                    collected.push((
                                        mode_str.to_string(),
                                        ss.formula.clone(),
                                        ss.strategy_id.clone(),
                                    ));
                                }
                                None => {
                                    if *mode_str == "long_only" {
                                        collected.push((
                                            mode_str.to_string(),
                                            fallback_formula.clone(),
                                            "Fallback".to_string(),
                                        ));
                                    }
                                    // No fallback for long_short — skip
                                }
                            }
                        }
                        collected
                    }; // strats read lock dropped here

                    // 4.2 Evaluate each mode
                    for (mode_str, current_formula, strategy_name) in &mode_formulas {
                        // Compute total positions before taking the mutable portfolio borrow
                        let total_pos = total_positions(
                            &crypto_portfolio,
                            &stock_portfolio_long_only,
                            &stock_portfolio_long_short,
                        );

                        let portfolio: &mut PortfolioManager = if is_stock {
                            if mode_str == "long_short" {
                                &mut stock_portfolio_long_short
                            } else {
                                &mut stock_portfolio_long_only
                            }
                        } else {
                            &mut crypto_portfolio
                        };

                        // Skip if already holding this symbol in this mode's portfolio
                        if portfolio.positions.contains_key(&msg.symbol) {
                            continue;
                        }

                        if let Some(result) = vm.execute(current_formula, &features) {
                            if let Some(last_val) = result.last() {
                                let is_fallback = strategy_name == "Fallback";
                                let signal_score = if is_fallback {
                                    *last_val
                                } else {
                                    sigmoid(*last_val)
                                };

                                // Push to adaptive buffer (only for evolved strategies)
                                if !is_fallback {
                                    signal_buffer.push(&msg.symbol, mode_str, signal_score);
                                }

                                // --- LONG entry ---
                                let upper = if is_fallback {
                                    0.001
                                } else {
                                    signal_buffer
                                        .upper_threshold(&msg.symbol, mode_str)
                                        .unwrap_or(signal_threshold)
                                };

                                if signal_score > upper {
                                    try_entry(
                                        &event_bus,
                                        &mut risk_engine,
                                        portfolio,
                                        total_pos,
                                        &msg.symbol,
                                        msg.price,
                                        is_stock,
                                        mode_str,
                                        strategy_name,
                                        OrderSide::Buy,
                                        PositionDirection::Long,
                                        signal_score,
                                        upper,
                                    )
                                    .await;
                                }

                                // --- SHORT entry (long_short mode only, non-fallback) ---
                                if mode_str == "long_short" && !is_fallback {
                                    let lower = signal_buffer
                                        .lower_threshold(&msg.symbol, mode_str)
                                        .unwrap_or(signal_lower_threshold);
                                    if signal_score < lower {
                                        try_entry(
                                            &event_bus,
                                            &mut risk_engine,
                                            portfolio,
                                            total_pos,
                                            &msg.symbol,
                                            msg.price,
                                            is_stock,
                                            mode_str,
                                            strategy_name,
                                            OrderSide::Sell,
                                            PositionDirection::Short,
                                            signal_score,
                                            lower,
                                        )
                                        .await;
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

/// Check exits for a single portfolio and publish exit signals.
#[allow(clippy::too_many_arguments)]
async fn process_exits(
    portfolio: &mut PortfolioManager,
    mode_str: &str,
    is_stock: bool,
    _symbol: &str,
    price: f64,
    event_bus: &EventBus,
    is_stock_portfolio: bool,
) {
    // Only process stock portfolios when is_stock, and crypto portfolio when !is_stock
    if is_stock_portfolio && !is_stock {
        return;
    }

    let exit_signals = portfolio.check_exits();
    for exit in exit_signals {
        if let Some(pos) = portfolio.positions.get_mut(&exit.token_address) {
            if pos.status == PositionStatus::Closing {
                continue;
            }
            pos.status = PositionStatus::Closing;

            // For short positions, exit side is Buy (cover); for long, Sell
            let exit_side = match pos.direction {
                PositionDirection::Long => OrderSide::Sell,
                PositionDirection::Short => OrderSide::Buy,
            };
            let direction_label = match exit_side {
                OrderSide::Buy => "buy",
                OrderSide::Sell => "sell",
            };

            info!(
                "EXIT SIGNAL TRIGGERED: {} Reason: {:?} (mode: {}, dir: {:?})",
                exit.symbol, exit.reason, mode_str, pos.direction
            );

            strategy_engine::metrics::SIGNALS_GENERATED_TOTAL
                .with_label_values(&["ExitLogic", direction_label, mode_str])
                .inc();

            let signal = TradeSignal {
                id: uuid::Uuid::new_v4(),
                strategy_id: "ExitLogic".to_string(),
                symbol: exit.symbol.clone(),
                side: exit_side,
                quantity: exit.sell_ratio,
                price: Some(price),
                order_type: OrderType::Market,
                timestamp: Utc::now(),
                reason: format!("Exit: {:?}", exit.reason),
                exchange: if is_stock {
                    Some("polygon".to_string())
                } else {
                    None
                },
                mode: Some(mode_str.to_string()),
            };

            if let Err(e) = event_bus.publish_signal(&signal).await {
                error!("Failed to publish Exit signal: {}", e);
            }

            if exit.sell_ratio < 0.99 {
                if let Some(p) = portfolio.positions.get_mut(&exit.token_address) {
                    p.is_moonbag = true;
                    p.status = PositionStatus::Active;
                }
            }
        }
    }
}

/// Attempt an entry (long or short), performing risk checks and publishing the signal.
#[allow(clippy::too_many_arguments)]
async fn try_entry(
    event_bus: &EventBus,
    risk_engine: &mut RiskEngine,
    portfolio: &mut PortfolioManager,
    current_total_positions: i64,
    symbol: &str,
    price: f64,
    is_stock: bool,
    mode_str: &str,
    strategy_name: &str,
    side: OrderSide,
    direction: PositionDirection,
    signal_score: f64,
    threshold: f64,
) {
    let (shares, exchange) = if is_stock {
        let s = risk_engine.calculate_stock_entry_shares(price);
        if s <= 0.0 {
            warn!("Insufficient price for stock entry: {}", symbol);
            return;
        }
        (s, Some("polygon".to_string()))
    } else {
        let amount_sol = risk_engine.calculate_entry_size();
        if amount_sol <= 0.0 {
            warn!("Insufficient equity/size for entry.");
            return;
        }
        (amount_sol, None)
    };

    let direction_label = match side {
        OrderSide::Buy => "buy",
        OrderSide::Sell => "sell",
    };

    let signal = TradeSignal {
        id: uuid::Uuid::new_v4(),
        strategy_id: strategy_name.to_string(),
        symbol: symbol.to_string(),
        side,
        quantity: shares,
        price: Some(price),
        order_type: OrderType::Market,
        timestamp: Utc::now(),
        reason: format!(
            "Entry Signal: {:.4} (thr: {:.4}, mode: {})",
            signal_score, threshold, mode_str
        ),
        exchange,
        mode: Some(mode_str.to_string()),
    };

    // Update open stock position count for risk engine
    if is_stock {
        risk_engine.set_open_stock_positions(mode_str, portfolio.positions.len());
    }

    let risk_approved = risk_engine.check(&signal, Some(10000.0)).await;
    strategy_engine::metrics::RISK_CHECKS_TOTAL
        .with_label_values(&[if risk_approved {
            "approved"
        } else {
            "rejected"
        }])
        .inc();

    if risk_approved {
        info!(
            "ENTRY SIGNAL: {} @ {} (Qty: {}, Strategy: {}, mode: {}, side: {})",
            symbol, price, shares, strategy_name, mode_str, direction_label
        );

        strategy_engine::metrics::SIGNALS_GENERATED_TOTAL
            .with_label_values(&[strategy_name, direction_label, mode_str])
            .inc();

        if let Err(e) = event_bus.publish_signal(&signal).await {
            error!("Failed to publish Entry signal: {}", e);
        } else {
            let amount_held = if is_stock { shares } else { shares / price };
            portfolio.add_position(
                symbol.to_string(),
                symbol.to_string(),
                price,
                amount_held,
                price,
                direction,
            );
            strategy_engine::metrics::ACTIVE_POSITIONS.set(current_total_positions + 1);
        }
    }
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
