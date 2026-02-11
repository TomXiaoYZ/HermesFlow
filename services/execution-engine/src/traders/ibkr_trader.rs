use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use ibapi::contracts::Contract;
use ibapi::orders::{Action, Order as IbOrder};
use ibapi::Client;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};

use super::{AccountSummary, BrokerOrderType, BrokerPosition, OrderParams, OrderResult, Trader};

static NEXT_ORDER_ID: AtomicI32 = AtomicI32::new(1000);

fn next_order_id() -> i32 {
    NEXT_ORDER_ID.fetch_add(1, Ordering::SeqCst)
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

        Ok(Self {
            client: Arc::new(Mutex::new(ib_client)),
        })
    }

    fn build_contract(symbol: &str) -> Contract {
        let clean_symbol = if symbol.starts_with("US.") {
            &symbol[3..]
        } else {
            symbol
        };
        Contract::stock(clean_symbol)
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

        let mut order = IbOrder::default();
        order.action = Action::Buy;
        order.total_quantity = quantity;
        order.order_type = match params.order_type {
            BrokerOrderType::Market => "MKT".to_string(),
            BrokerOrderType::Limit => "LMT".to_string(),
            BrokerOrderType::MarketOnClose => "MOC".to_string(),
        };
        if let Some(price) = params.limit_price {
            order.limit_price = Some(price);
        }
        order.order_id = order_id;

        info!(
            "IBKR BUY: {} x{} ({}) order_id={}",
            symbol, quantity, order.order_type, order_id
        );

        let client = self.client.clone();
        let sym = symbol.to_string();

        tokio::task::spawn_blocking(move || -> Result<()> {
            let guard = client
                .lock()
                .map_err(|e| anyhow::anyhow!("IBKR lock poisoned: {}", e))?;
            match guard.0.place_order(order_id, &contract, &order) {
                Ok(responses) => {
                    for resp in responses {
                        info!("IBKR order response: {:?}", resp);
                    }
                }
                Err(e) => {
                    error!("IBKR place_order failed for {}: {}", sym, e);
                    return Err(anyhow::anyhow!("IBKR place_order error: {}", e));
                }
            }
            Ok(())
        })
        .await??;

        Ok(OrderResult {
            order_id: order_id.to_string(),
            status: "Submitted".to_string(),
            filled_qty: 0.0,
            avg_price: 0.0,
            broker: "IBKR".to_string(),
            timestamp: Utc::now(),
        })
    }

    async fn sell(&self, symbol: &str, quantity: f64, params: &OrderParams) -> Result<OrderResult> {
        let contract = Self::build_contract(symbol);
        let order_id = next_order_id();

        let mut order = IbOrder::default();
        order.action = Action::Sell;
        order.total_quantity = quantity;
        order.order_type = match params.order_type {
            BrokerOrderType::Market => "MKT".to_string(),
            BrokerOrderType::Limit => "LMT".to_string(),
            BrokerOrderType::MarketOnClose => "MOC".to_string(),
        };
        if let Some(price) = params.limit_price {
            order.limit_price = Some(price);
        }
        order.order_id = order_id;

        info!(
            "IBKR SELL: {} x{} ({}) order_id={}",
            symbol, quantity, order.order_type, order_id
        );

        let client = self.client.clone();
        let sym = symbol.to_string();

        tokio::task::spawn_blocking(move || -> Result<()> {
            let guard = client
                .lock()
                .map_err(|e| anyhow::anyhow!("IBKR lock poisoned: {}", e))?;
            match guard.0.place_order(order_id, &contract, &order) {
                Ok(responses) => {
                    for resp in responses {
                        info!("IBKR order response: {:?}", resp);
                    }
                }
                Err(e) => {
                    error!("IBKR place_order(sell) failed for {}: {}", sym, e);
                    return Err(anyhow::anyhow!("IBKR place_order error: {}", e));
                }
            }
            Ok(())
        })
        .await??;

        Ok(OrderResult {
            order_id: order_id.to_string(),
            status: "Submitted".to_string(),
            filled_qty: 0.0,
            avg_price: 0.0,
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
        // ibapi 0.1 does not expose account_summary; estimate from positions
        let positions = self.get_positions().await?;
        let total_value: f64 = positions.iter().map(|p| p.market_value).sum();

        Ok(AccountSummary {
            net_liquidation: total_value,
            cash: 0.0,
            buying_power: 0.0,
            currency: "USD".to_string(),
        })
    }
}
