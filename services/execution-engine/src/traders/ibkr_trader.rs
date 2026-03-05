use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use ibapi::accounts::types::AccountGroup;
use ibapi::accounts::{AccountSummaryResult, AccountSummaryTags, PositionUpdate};
use ibapi::client::sync::Client;
use ibapi::contracts::Contract;
use ibapi::messages::Notice;
use ibapi::orders::{Action, CancelOrder, Order as IbOrder, PlaceOrder};
use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
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

/// Wrapper to allow ibapi::client::sync::Client across thread boundaries.
/// ibapi sync Client may use RefCell internally (not Send/Sync).
/// We guarantee exclusive access through Mutex and only use it in spawn_blocking.
struct IbClient(Client);

// SAFETY: All access to the inner Client goes through Mutex + spawn_blocking,
// ensuring single-threaded exclusive access at all times.
unsafe impl Send for IbClient {}
unsafe impl Sync for IbClient {}

#[derive(Clone)]
pub struct IBKRTrader {
    client: Arc<Mutex<IbClient>>,
    host: String,
    port: u32,
    client_id: u32,
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

    /// Process a PlaceOrder response and accumulate fill data
    fn process(&mut self, notification: &PlaceOrder) {
        match notification {
            PlaceOrder::OrderStatus(os) => {
                self.status = os.status.clone();
                if os.filled > 0.0 {
                    self.filled_qty = os.filled;
                    self.avg_price = os.average_fill_price;
                }
            }
            PlaceOrder::ExecutionData(exec_data) => {
                info!(
                    "IBKR execution: shares={}, price={}, exec_id={}",
                    exec_data.execution.shares,
                    exec_data.execution.price,
                    exec_data.execution.execution_id
                );
            }
            PlaceOrder::CommissionReport(report) => {
                info!(
                    "IBKR commission: ${:.4} (exec_id: {})",
                    report.commission, report.execution_id
                );
            }
            PlaceOrder::OpenOrder(_) => {}
            PlaceOrder::Message(notice) => {
                log_notice("place_order", notice);
            }
        }
    }

    /// Whether the order has reached a terminal state (no more updates expected).
    /// Breaking on terminal status avoids holding the Mutex for the full iterator
    /// timeout, reducing lock contention from ~10s to <1s per order.
    fn is_terminal(&self) -> bool {
        matches!(
            self.status.as_str(),
            "Filled" | "Cancelled" | "Inactive" | "ApiCancelled"
        )
    }
}

/// Log a Notice at the appropriate level based on its code.
fn log_notice(context: &str, notice: &Notice) {
    // IBKR notice codes >= 2000 are typically warnings/errors
    if notice.code >= 2000 {
        warn!(
            "IBKR {} notice [{}]: {}",
            context, notice.code, notice.message
        );
    } else {
        info!(
            "IBKR {} notice [{}]: {}",
            context, notice.code, notice.message
        );
    }
}

impl IBKRTrader {
    /// Connect to IBKR TWS/Gateway.
    /// ibapi::client::sync::Client::connect is blocking, so we run it on spawn_blocking.
    pub async fn new(host: &str, port: u32, client_id: u32) -> Result<Self> {
        let addr = format!("{}:{}", host, port);
        info!("Connecting to IBKR at {}", addr);

        let client_id_i32 = client_id as i32;
        let addr_clone = addr.clone();

        let ib_client = tokio::task::spawn_blocking(move || {
            Client::connect(&addr_clone, client_id_i32)
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
            host: host.to_string(),
            port,
            client_id,
        })
    }

    /// Check whether the underlying IBKR connection is still alive.
    pub async fn is_alive(&self) -> bool {
        let client = self.client.clone();
        Self::with_ibkr_timeout("is_alive", Duration::from_secs(5), move || {
            Ok(match client.lock() {
                Ok(guard) => guard.0.is_connected(),
                Err(_) => false,
            })
        })
        .await
        .unwrap_or(false)
    }

    /// Drop the old connection and establish a new one, swapping the Mutex contents.
    /// Uses `fetch_max` on NEXT_ORDER_ID so the counter only increases.
    pub async fn reconnect(&self) -> Result<()> {
        let addr = format!("{}:{}", self.host, self.port);
        let client_id = self.client_id as i32;
        let addr_clone = addr.clone();

        let new_client = Self::with_ibkr_timeout("reconnect", Duration::from_secs(30), move || {
            Client::connect(&addr_clone, client_id)
                .map(IbClient)
                .map_err(|e| anyhow::anyhow!("IBKR reconnect failed to {}: {}", addr_clone, e))
        })
        .await?;

        let ibkr_next = new_client.0.next_order_id();
        NEXT_ORDER_ID.fetch_max(ibkr_next, Ordering::SeqCst);
        info!(
            "IBKR reconnected to {}, next_order_id synced to {}",
            addr, ibkr_next
        );

        let mut guard = self
            .client
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned during reconnect: {}", e))?;
        *guard = new_client;
        Ok(())
    }

    /// Verify the IBKR connection is alive; reconnect if needed.
    /// Call before every trade to minimize stale-connection failures.
    pub async fn ensure_connected(&self) -> Result<()> {
        if !self.is_alive().await {
            warn!("IBKR connection down, attempting pre-trade reconnect...");
            self.reconnect().await
        } else {
            Ok(())
        }
    }

    fn build_contract(symbol: &str) -> Contract {
        let clean_symbol = symbol.strip_prefix("US.").unwrap_or(symbol);
        Contract::stock(clean_symbol).build()
    }

    /// P10D: Wrap a spawn_blocking call with a timeout.
    ///
    /// If the blocking task doesn't complete within `timeout`, returns an error
    /// instead of hanging indefinitely. This prevents IBKR API freezes from
    /// blocking the entire sync loop.
    async fn with_ibkr_timeout<F, T>(op_name: &str, timeout: Duration, f: F) -> Result<T>
    where
        F: FnOnce() -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        match tokio::time::timeout(timeout, tokio::task::spawn_blocking(f)).await {
            Ok(Ok(result)) => result,
            Ok(Err(join_err)) => Err(anyhow::anyhow!(
                "IBKR {} task panicked: {}",
                op_name,
                join_err
            )),
            Err(_elapsed) => {
                warn!("IBKR {} timed out after {:?}", op_name, timeout);
                Err(anyhow::anyhow!(
                    "IBKR {} timed out after {:?}",
                    op_name,
                    timeout
                ))
            }
        }
    }

    /// Execute an order and collect fill data from the response subscription.
    ///
    /// **Key optimization**: breaks the subscription loop as soon as a terminal status
    /// (Filled/Cancelled/Inactive) is received, instead of waiting for the iterator
    /// timeout. This reduces Mutex hold time from ~10s to <1s per order,
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
            Ok(subscription) => {
                while let Some(resp) = subscription.next() {
                    info!("IBKR order {} response: {:?}", order_id, resp);
                    fill.process(&resp);

                    // Break immediately on terminal status to release the Mutex.
                    // Without this, we hold the lock for the full iterator
                    // timeout after the last message, blocking all other orders.
                    if fill.is_terminal() {
                        info!(
                            "IBKR order {} reached terminal status: {} (filled={}, avg={})",
                            order_id, fill.status, fill.filled_qty, fill.avg_price
                        );
                        subscription.cancel();
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

        let fill = Self::with_ibkr_timeout("buy", Duration::from_secs(30), move || {
            Self::execute_order(&client, order_id, &contract, &order, &sym)
        })
        .await?;

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

        let fill = Self::with_ibkr_timeout("sell", Duration::from_secs(30), move || {
            Self::execute_order(&client, order_id, &contract, &order, &sym)
        })
        .await?;

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

        Self::with_ibkr_timeout("cancel_order", Duration::from_secs(10), move || {
            let guard = client
                .lock()
                .map_err(|e| anyhow::anyhow!("IBKR lock poisoned: {}", e))?;
            match guard.0.cancel_order(oid, "") {
                Ok(subscription) => {
                    while let Some(resp) = subscription.next() {
                        match &resp {
                            CancelOrder::OrderStatus(os) => {
                                info!("IBKR cancel order {} status: {}", oid, os.status);
                                if matches!(
                                    os.status.as_str(),
                                    "Cancelled" | "ApiCancelled" | "Inactive"
                                ) {
                                    subscription.cancel();
                                    break;
                                }
                            }
                            CancelOrder::Notice(notice) => {
                                log_notice("cancel_order", notice);
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("IBKR cancel_order error: {}", e);
                }
            }
            Ok(())
        })
        .await
    }

    async fn get_positions(&self) -> Result<Vec<BrokerPosition>> {
        let client = self.client.clone();

        Self::with_ibkr_timeout("get_positions", Duration::from_secs(15), move || {
            let guard = client
                .lock()
                .map_err(|e| anyhow::anyhow!("IBKR lock poisoned: {}", e))?;
            let mut result = Vec::new();
            match guard.0.positions() {
                Ok(subscription) => {
                    while let Some(update) = subscription.next() {
                        match update {
                            PositionUpdate::Position(pos) => {
                                result.push(BrokerPosition {
                                    symbol: pos.contract.symbol.to_string(),
                                    quantity: pos.position,
                                    avg_cost: pos.average_cost,
                                    market_value: pos.position * pos.average_cost,
                                    unrealized_pnl: 0.0,
                                    account: pos.account.clone(),
                                });
                            }
                            PositionUpdate::PositionEnd => {
                                subscription.cancel();
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("IBKR positions() error: {}", e);
                }
            }
            Ok(result)
        })
        .await
    }

    async fn get_account_summary(&self) -> Result<AccountSummary> {
        let client = self.client.clone();

        Self::with_ibkr_timeout("get_account_summary", Duration::from_secs(15), move || {
            let guard = client
                .lock()
                .map_err(|e| anyhow::anyhow!("IBKR lock poisoned: {}", e))?;

            let tags = &[
                AccountSummaryTags::NET_LIQUIDATION,
                AccountSummaryTags::TOTAL_CASH_VALUE,
                AccountSummaryTags::BUYING_POWER,
            ];
            let group = AccountGroup("All".to_string());
            let subscription = guard
                .0
                .account_summary(&group, tags)
                .map_err(|e| anyhow::anyhow!("IBKR account_summary: {}", e))?;

            let mut summary = AccountSummary {
                currency: "USD".to_string(),
                ..Default::default()
            };

            while let Some(update) = subscription.next() {
                match update {
                    AccountSummaryResult::Summary(s) => match s.tag.as_str() {
                        "NetLiquidation" => {
                            summary.net_liquidation = s.value.parse().unwrap_or(0.0);
                            if !s.currency.is_empty() {
                                summary.currency = s.currency.clone();
                            }
                        }
                        "TotalCashValue" => {
                            summary.cash = s.value.parse().unwrap_or(0.0);
                        }
                        "BuyingPower" => {
                            summary.buying_power = s.value.parse().unwrap_or(0.0);
                        }
                        _ => {}
                    },
                    AccountSummaryResult::End => {
                        subscription.cancel();
                        break;
                    }
                }
            }

            Ok(summary)
        })
        .await
    }

    async fn get_account_summaries(&self) -> Result<HashMap<String, AccountSummary>> {
        let client = self.client.clone();

        Self::with_ibkr_timeout(
            "get_account_summaries",
            Duration::from_secs(15),
            move || {
                let guard = client
                    .lock()
                    .map_err(|e| anyhow::anyhow!("IBKR lock poisoned: {}", e))?;

                let tags = &[
                    AccountSummaryTags::NET_LIQUIDATION,
                    AccountSummaryTags::TOTAL_CASH_VALUE,
                    AccountSummaryTags::BUYING_POWER,
                ];
                let group = AccountGroup("All".to_string());
                let subscription = guard
                    .0
                    .account_summary(&group, tags)
                    .map_err(|e| anyhow::anyhow!("IBKR account_summaries: {}", e))?;

                let mut summaries: HashMap<String, AccountSummary> = HashMap::new();

                while let Some(update) = subscription.next() {
                    match update {
                        AccountSummaryResult::Summary(s) => {
                            let entry = summaries.entry(s.account.clone()).or_insert_with(|| {
                                AccountSummary {
                                    currency: if s.currency.is_empty() {
                                        "USD".to_string()
                                    } else {
                                        s.currency.clone()
                                    },
                                    ..Default::default()
                                }
                            });
                            match s.tag.as_str() {
                                "NetLiquidation" => {
                                    entry.net_liquidation = s.value.parse().unwrap_or(0.0);
                                }
                                "TotalCashValue" => {
                                    entry.cash = s.value.parse().unwrap_or(0.0);
                                }
                                "BuyingPower" => {
                                    entry.buying_power = s.value.parse().unwrap_or(0.0);
                                }
                                _ => {}
                            }
                        }
                        AccountSummaryResult::End => {
                            subscription.cancel();
                            break;
                        }
                    }
                }

                Ok(summaries)
            },
        )
        .await
    }
}
