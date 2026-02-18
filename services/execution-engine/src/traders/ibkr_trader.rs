use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use ibapi::contracts::Contract;
use ibapi::orders::{Action, Order as IbOrder, OrderNotification};
use ibapi::Client;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};

use super::{AccountSummary, BrokerOrderType, BrokerPosition, OrderParams, OrderResult, Trader};

static NEXT_ORDER_ID: AtomicI32 = AtomicI32::new(1000);

fn next_order_id() -> i32 {
    NEXT_ORDER_ID.fetch_add(1, Ordering::SeqCst)
}

/// Ensure NEXT_ORDER_ID is at least `min_id`.
/// Called from main.rs after querying the DB for the highest persisted order_id.
pub fn set_min_order_id(min_id: i32) {
    loop {
        let current = NEXT_ORDER_ID.load(Ordering::SeqCst);
        if min_id <= current {
            return;
        }
        if NEXT_ORDER_ID
            .compare_exchange(current, min_id, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            info!(
                "NEXT_ORDER_ID bumped from {} to {} (DB max)",
                current, min_id
            );
            return;
        }
    }
}

/// Wrapper to allow ibapi::Client across thread boundaries.
/// ibapi::Client uses RefCell internally (not Send/Sync).
/// We guarantee exclusive access through Mutex and only use it in spawn_blocking.
struct IbClient(Client);

// SAFETY: All access to the inner Client goes through Mutex + spawn_blocking,
// ensuring single-threaded exclusive access at all times.
unsafe impl Send for IbClient {}
unsafe impl Sync for IbClient {}

#[derive(Clone)]
pub struct IBKRTrader {
    client: Arc<Mutex<IbClient>>,
}

/// Accumulated fill data from IBKR order notifications
struct FillAccumulator {
    filled_qty: f64,
    avg_price: f64,
    status: String,
}

impl FillAccumulator {
    fn new() -> Self {
        Self {
            filled_qty: 0.0,
            avg_price: 0.0,
            status: "Submitted".to_string(),
        }
    }

    /// Process an OrderNotification and accumulate fill data
    fn process(&mut self, notification: &OrderNotification) {
        match notification {
            OrderNotification::OrderStatus(os) => {
                self.status = os.status.clone();
                if os.filled > 0.0 {
                    self.filled_qty = os.filled;
                    self.avg_price = os.average_fill_price;
                }
            }
            OrderNotification::ExecutionData(exec_data) => {
                info!(
                    "IBKR execution: shares={}, price={}, exec_id={}",
                    exec_data.execution.shares,
                    exec_data.execution.price,
                    exec_data.execution.execution_id
                );
            }
            OrderNotification::CommissionReport(report) => {
                info!(
                    "IBKR commission: ${:.4} (exec_id: {})",
                    report.commission, report.execution_id
                );
            }
            _ => {}
        }
    }

    /// Whether the order has reached a terminal state (no more updates expected).
    /// Breaking on terminal status avoids holding the Mutex for the full 10-second
    /// iterator timeout, reducing lock contention from ~10s to <1s per order.
    fn is_terminal(&self) -> bool {
        matches!(
            self.status.as_str(),
            "Filled" | "Cancelled" | "Inactive" | "ApiCancelled"
        )
    }
}

impl IBKRTrader {
    /// Connect to IBKR TWS/Gateway.
    /// ibapi::Client::connect is blocking, so we run it on spawn_blocking.
    pub async fn new(host: &str, port: u32, client_id: u32) -> Result<Self> {
        let addr = format!("{}:{}", host, port);
        info!("Connecting to IBKR at {}", addr);

        let client_id = client_id as i32;
        let addr_clone = addr.clone();

        let ib_client = tokio::task::spawn_blocking(move || {
            Client::connect(&addr_clone, client_id)
                .map(IbClient)
                .map_err(|e| anyhow::anyhow!("IBKR connection failed to {}: {}", addr_clone, e))
        })
        .await??;

        info!("Connected to IBKR at {}", addr);

        // Seed NEXT_ORDER_ID from IBKR's next valid order ID (set during connection handshake)
        let ibkr_next = ib_client.0.next_order_id();
        let prev = NEXT_ORDER_ID.swap(ibkr_next, Ordering::SeqCst);
        info!("IBKR next_order_id: {} (was {})", ibkr_next, prev);

        Ok(Self {
            client: Arc::new(Mutex::new(ib_client)),
        })
    }

    fn build_contract(symbol: &str) -> Contract {
        let clean_symbol = symbol.strip_prefix("US.").unwrap_or(symbol);
        Contract::stock(clean_symbol)
    }

    /// Execute an order and collect fill data from the response iterator.
    ///
    /// **Key optimization**: breaks the iterator loop as soon as a terminal status
    /// (Filled/Cancelled/Inactive) is received, instead of waiting for the 10-second
    /// ibapi iterator timeout. This reduces Mutex hold time from ~10s to <1s per order,
    /// preventing cascading delays when multiple orders are placed concurrently.
    fn execute_order(
        client: &Arc<Mutex<IbClient>>,
        order_id: i32,
        contract: &Contract,
        order: &IbOrder,
        symbol: &str,
    ) -> Result<FillAccumulator> {
        let guard = client
            .lock()
            .map_err(|e| anyhow::anyhow!("IBKR lock poisoned: {}", e))?;

        let mut fill = FillAccumulator::new();

        match guard.0.place_order(order_id, contract, order) {
            Ok(responses) => {
                for resp in responses {
                    info!("IBKR order {} response: {:?}", order_id, resp);
                    fill.process(&resp);

                    // Break immediately on terminal status to release the Mutex.
                    // Without this, we hold the lock for the full 10-second iterator
                    // timeout after the last message, blocking all other orders.
                    if fill.is_terminal() {
                        info!(
                            "IBKR order {} reached terminal status: {} (filled={}, avg={})",
                            order_id, fill.status, fill.filled_qty, fill.avg_price
                        );
                        break;
                    }
                }
            }
            Err(e) => {
                error!("IBKR place_order failed for {}: {}", symbol, e);
                return Err(anyhow::anyhow!("IBKR place_order error: {}", e));
            }
        }

        if !fill.is_terminal() {
            warn!(
                "IBKR order {} for {} ended with non-terminal status: {} \
                 (fill will be captured by reconciliation)",
                order_id, symbol, fill.status
            );
        }

        Ok(fill)
    }
}

#[async_trait]
impl Trader for IBKRTrader {
    fn broker_name(&self) -> &str {
        "IBKR"
    }

    async fn buy(&self, symbol: &str, quantity: f64, params: &OrderParams) -> Result<OrderResult> {
        let contract = Self::build_contract(symbol);
        let order_id = next_order_id();

        let order = IbOrder {
            action: Action::Buy,
            total_quantity: quantity,
            order_type: match params.order_type {
                BrokerOrderType::Market => "MKT".to_string(),
                BrokerOrderType::Limit => "LMT".to_string(),
                BrokerOrderType::MarketOnClose => "MOC".to_string(),
            },
            limit_price: params.limit_price,
            account: params.account.clone().unwrap_or_default(),
            order_id,
            ..Default::default()
        };

        info!(
            "IBKR BUY: {} x{} ({}) order_id={} account={}",
            symbol,
            quantity,
            order.order_type,
            order_id,
            if order.account.is_empty() {
                "default"
            } else {
                &order.account
            }
        );

        let client = self.client.clone();
        let sym = symbol.to_string();

        let fill = tokio::task::spawn_blocking(move || {
            Self::execute_order(&client, order_id, &contract, &order, &sym)
        })
        .await??;

        Ok(OrderResult {
            order_id: order_id.to_string(),
            status: fill.status,
            filled_qty: fill.filled_qty,
            avg_price: fill.avg_price,
            broker: "IBKR".to_string(),
            timestamp: Utc::now(),
        })
    }

    async fn sell(&self, symbol: &str, quantity: f64, params: &OrderParams) -> Result<OrderResult> {
        let contract = Self::build_contract(symbol);
        let order_id = next_order_id();

        let order = IbOrder {
            action: Action::Sell,
            total_quantity: quantity,
            order_type: match params.order_type {
                BrokerOrderType::Market => "MKT".to_string(),
                BrokerOrderType::Limit => "LMT".to_string(),
                BrokerOrderType::MarketOnClose => "MOC".to_string(),
            },
            limit_price: params.limit_price,
            account: params.account.clone().unwrap_or_default(),
            order_id,
            ..Default::default()
        };

        info!(
            "IBKR SELL: {} x{} ({}) order_id={} account={}",
            symbol,
            quantity,
            order.order_type,
            order_id,
            if order.account.is_empty() {
                "default"
            } else {
                &order.account
            }
        );

        let client = self.client.clone();
        let sym = symbol.to_string();

        let fill = tokio::task::spawn_blocking(move || {
            Self::execute_order(&client, order_id, &contract, &order, &sym)
        })
        .await??;

        Ok(OrderResult {
            order_id: order_id.to_string(),
            status: fill.status,
            filled_qty: fill.filled_qty,
            avg_price: fill.avg_price,
            broker: "IBKR".to_string(),
            timestamp: Utc::now(),
        })
    }

    async fn cancel_order(&self, order_id: &str) -> Result<()> {
        let oid: i32 = order_id.parse()?;
        let client = self.client.clone();

        info!("IBKR cancelling order {}", order_id);

        tokio::task::spawn_blocking(move || -> Result<()> {
            let guard = client
                .lock()
                .map_err(|e| anyhow::anyhow!("IBKR lock poisoned: {}", e))?;
            match guard.0.cancel_order(oid, "") {
                Ok(responses) => {
                    for resp in responses {
                        info!("IBKR cancel response: {:?}", resp);
                    }
                }
                Err(e) => {
                    warn!("IBKR cancel_order error: {}", e);
                }
            }
            Ok(())
        })
        .await??;

        Ok(())
    }

    async fn get_positions(&self) -> Result<Vec<BrokerPosition>> {
        let client = self.client.clone();

        let positions = tokio::task::spawn_blocking(move || -> Result<Vec<BrokerPosition>> {
            let guard = client
                .lock()
                .map_err(|e| anyhow::anyhow!("IBKR lock poisoned: {}", e))?;
            let mut result = Vec::new();
            match guard.0.positions() {
                Ok(pos_iter) => {
                    for pos in pos_iter {
                        result.push(BrokerPosition {
                            symbol: pos.contract.symbol.clone(),
                            quantity: pos.position,
                            avg_cost: pos.average_cost,
                            market_value: pos.position * pos.average_cost,
                            unrealized_pnl: 0.0,
                            account: pos.account.clone(),
                        });
                    }
                }
                Err(e) => {
                    warn!("IBKR positions() error: {}", e);
                }
            }
            Ok(result)
        })
        .await??;

        Ok(positions)
    }

    async fn get_account_summary(&self) -> Result<AccountSummary> {
        let positions = self.get_positions().await?;
        let total_value: f64 = positions
            .iter()
            .map(|p| p.quantity.abs() * p.avg_cost)
            .sum();

        Ok(AccountSummary {
            net_liquidation: total_value,
            cash: 0.0,
            buying_power: 0.0,
            currency: "USD".to_string(),
        })
    }
}
