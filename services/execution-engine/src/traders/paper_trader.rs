//! P6b-B2: Paper trading account for ensemble strategy validation.
//!
//! Simulates order execution with configurable slippage and commission.
//! Tracks positions and cash in-memory, persists to `paper_trade_*` tables.
//! Used to validate deployed strategies before live trading promotion.

use super::{AccountSummary, BrokerOrderType, BrokerPosition, OrderParams, OrderResult, Trader};
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_postgres::Client;
use tracing::{info, warn};

static PAPER_ORDER_ID: AtomicU64 = AtomicU64::new(1);

/// In-memory position for paper trading.
#[derive(Debug, Clone)]
struct PaperPosition {
    quantity: f64,
    avg_cost: f64,
    realized_pnl: f64,
}

/// Paper trading configuration.
#[derive(Debug, Clone)]
pub struct PaperTraderConfig {
    pub initial_cash: f64,
    pub slippage_bps: f64,
    pub commission_per_share: f64,
    pub exchange: String,
}

impl Default for PaperTraderConfig {
    fn default() -> Self {
        Self {
            initial_cash: 100_000.0,
            slippage_bps: 5.0,
            commission_per_share: 0.005,
            exchange: "Paper".to_string(),
        }
    }
}

/// Simulated trader for paper trading deployed strategies.
pub struct PaperTrader {
    config: PaperTraderConfig,
    positions: Arc<RwLock<HashMap<String, PaperPosition>>>,
    cash: Arc<RwLock<f64>>,
    total_commission: Arc<RwLock<f64>>,
    db: Option<Arc<Client>>,
}

impl PaperTrader {
    pub fn new(config: PaperTraderConfig, db: Option<Arc<Client>>) -> Self {
        let cash = config.initial_cash;
        Self {
            config,
            positions: Arc::new(RwLock::new(HashMap::new())),
            cash: Arc::new(RwLock::new(cash)),
            total_commission: Arc::new(RwLock::new(0.0)),
            db,
        }
    }

    /// Simulate fill price with slippage.
    fn simulate_fill_price(&self, base_price: f64, is_buy: bool) -> f64 {
        let slippage = base_price * self.config.slippage_bps / 10_000.0;
        if is_buy {
            base_price + slippage
        } else {
            base_price - slippage
        }
    }

    /// Execute a simulated order (buy or sell).
    async fn execute_order(
        &self,
        symbol: &str,
        quantity: f64,
        is_buy: bool,
        params: &OrderParams,
    ) -> Result<OrderResult> {
        let order_id = format!("PAPER-{}", PAPER_ORDER_ID.fetch_add(1, Ordering::Relaxed));

        // Determine base price (use limit price if available, otherwise use a placeholder)
        let base_price = match params.order_type {
            BrokerOrderType::Limit => params.limit_price.unwrap_or(0.0),
            _ => params.limit_price.unwrap_or(0.0),
        };

        if base_price <= 0.0 {
            warn!(
                "[Paper] Order {} for {} {} — no price available, using 0",
                order_id, symbol, quantity
            );
            return Ok(OrderResult {
                order_id,
                status: "rejected".to_string(),
                filled_qty: 0.0,
                avg_price: 0.0,
                broker: "paper".to_string(),
                timestamp: Utc::now(),
            });
        }

        let fill_price = self.simulate_fill_price(base_price, is_buy);
        let commission = quantity.abs() * self.config.commission_per_share;
        let total_cost = if is_buy {
            quantity * fill_price + commission
        } else {
            -(quantity * fill_price) + commission
        };

        // Update cash
        {
            let mut cash = self.cash.write().await;
            if is_buy && *cash < total_cost {
                return Ok(OrderResult {
                    order_id,
                    status: "rejected_insufficient_funds".to_string(),
                    filled_qty: 0.0,
                    avg_price: 0.0,
                    broker: "paper".to_string(),
                    timestamp: Utc::now(),
                });
            }
            if is_buy {
                *cash -= total_cost;
            } else {
                *cash += quantity * fill_price - commission;
            }
        }

        // Update commission tracker
        {
            let mut total_comm = self.total_commission.write().await;
            *total_comm += commission;
        }

        // Update positions
        {
            let mut positions = self.positions.write().await;
            let pos = positions
                .entry(symbol.to_string())
                .or_insert(PaperPosition {
                    quantity: 0.0,
                    avg_cost: 0.0,
                    realized_pnl: 0.0,
                });

            if is_buy {
                let new_qty = pos.quantity + quantity;
                if new_qty.abs() > 1e-10 {
                    pos.avg_cost = (pos.avg_cost * pos.quantity + fill_price * quantity) / new_qty;
                }
                pos.quantity = new_qty;
            } else {
                // Selling: realize PnL
                let sell_qty = quantity.min(pos.quantity);
                if sell_qty > 0.0 {
                    pos.realized_pnl += (fill_price - pos.avg_cost) * sell_qty;
                }
                pos.quantity -= quantity;
                // If position flipped to short, reset avg_cost
                if pos.quantity < 0.0 {
                    pos.avg_cost = fill_price;
                }
            }

            // Remove zero positions
            if pos.quantity.abs() < 1e-10 {
                positions.remove(symbol);
            }
        }

        // Persist to DB if available
        if let Some(db) = &self.db {
            let side = if is_buy { "buy" } else { "sell" };
            let _ = db
                .execute(
                    "INSERT INTO paper_trade_orders \
                     (order_id, exchange, symbol, side, quantity, order_type, \
                      limit_price, filled_qty, avg_price, status) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'filled')",
                    &[
                        &order_id,
                        &self.config.exchange.as_str(),
                        &symbol,
                        &side,
                        &quantity,
                        &"market",
                        &fill_price,
                        &quantity,
                        &fill_price,
                    ],
                )
                .await
                .map_err(|e| warn!("[Paper] Failed to persist order: {}", e));

            let exec_id = format!("{}-exec", order_id);
            let _ = db
                .execute(
                    "INSERT INTO paper_trade_executions \
                     (order_id, execution_id, price, quantity, commission) \
                     VALUES ($1, $2, $3, $4, $5)",
                    &[&order_id, &exec_id, &fill_price, &quantity, &commission],
                )
                .await
                .map_err(|e| warn!("[Paper] Failed to persist execution: {}", e));
        }

        info!(
            "[Paper] {} {} {} @ {:.4} (commission={:.4})",
            if is_buy { "BUY" } else { "SELL" },
            quantity,
            symbol,
            fill_price,
            commission
        );

        Ok(OrderResult {
            order_id,
            status: "filled".to_string(),
            filled_qty: quantity,
            avg_price: fill_price,
            broker: "paper".to_string(),
            timestamp: Utc::now(),
        })
    }

    /// Persist current positions to DB.
    pub async fn sync_positions_to_db(&self) -> Result<()> {
        let db = match &self.db {
            Some(db) => db,
            None => return Ok(()),
        };

        let positions = self.positions.read().await;
        for (symbol, pos) in positions.iter() {
            let _ = db
                .execute(
                    "INSERT INTO paper_positions (exchange, symbol, quantity, avg_cost, updated_at) \
                     VALUES ($1, $2, $3, $4, NOW()) \
                     ON CONFLICT (exchange, symbol) DO UPDATE SET \
                      quantity = EXCLUDED.quantity, \
                      avg_cost = EXCLUDED.avg_cost, \
                      updated_at = NOW()",
                    &[
                        &self.config.exchange.as_str(),
                        &symbol.as_str(),
                        &pos.quantity,
                        &pos.avg_cost,
                    ],
                )
                .await
                .map_err(|e| warn!("[Paper] Failed to sync position {}: {}", symbol, e));
        }

        Ok(())
    }
}

#[async_trait]
impl Trader for PaperTrader {
    fn broker_name(&self) -> &str {
        "paper"
    }

    async fn buy(&self, symbol: &str, quantity: f64, params: &OrderParams) -> Result<OrderResult> {
        self.execute_order(symbol, quantity, true, params).await
    }

    async fn sell(&self, symbol: &str, quantity: f64, params: &OrderParams) -> Result<OrderResult> {
        self.execute_order(symbol, quantity, false, params).await
    }

    async fn cancel_order(&self, order_id: &str) -> Result<()> {
        info!("[Paper] Cancel order {} (no-op for paper)", order_id);
        Ok(())
    }

    async fn get_positions(&self) -> Result<Vec<BrokerPosition>> {
        let positions = self.positions.read().await;
        Ok(positions
            .iter()
            .map(|(symbol, pos)| BrokerPosition {
                symbol: symbol.clone(),
                quantity: pos.quantity,
                avg_cost: pos.avg_cost,
                market_value: 0.0, // No live market data in paper mode
                unrealized_pnl: 0.0,
                account: "paper".to_string(),
            })
            .collect())
    }

    async fn get_account_summary(&self) -> Result<AccountSummary> {
        let cash = *self.cash.read().await;
        let positions = self.positions.read().await;

        // Net liquidation = cash + sum of position values (at avg_cost, since no live data)
        let position_value: f64 = positions.values().map(|p| p.quantity * p.avg_cost).sum();

        Ok(AccountSummary {
            net_liquidation: cash + position_value,
            cash,
            buying_power: cash,
            currency: "USD".to_string(),
        })
    }
}
