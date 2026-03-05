use anyhow::Result;
use chrono::Utc;
use common::events::{OrderStatus, OrderUpdate, TradeSignal};
use redis::Commands;
use std::env;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::risk::StockRiskEngine;
use crate::traders::futu_trader::FutuTrader;
use crate::traders::ibkr_trader::IBKRTrader;
use crate::traders::{BrokerOrderType, OrderParams, OrderResult, TimeInForce, Trader};
use tokio_postgres::Client as PgClient;

/// Determines which broker should handle a given symbol
#[derive(Debug, Clone, PartialEq)]
enum BrokerRoute {
    Ibkr,
    Futu,
}

pub struct CommandListener {
    client: redis::Client,
    db: Option<Arc<PgClient>>,
    risk: StockRiskEngine,
    /// IBKRTrader for long_only mode (client_id from IBKR_CLIENT_ID_LONG_ONLY)
    pub ibkr_long_only_trader: Option<Arc<IBKRTrader>>,
    /// IBKRTrader for long_short mode (client_id from IBKR_CLIENT_ID_LONG_SHORT)
    pub ibkr_long_short_trader: Option<Arc<IBKRTrader>>,
    pub futu_trader: Option<Arc<FutuTrader>>,
    /// IBKR sub-account ID for long_only mode (env: IBKR_ACCOUNT_LONG_ONLY)
    ibkr_account_long_only: Option<String>,
    /// IBKR sub-account ID for long_short mode (env: IBKR_ACCOUNT_LONG_SHORT)
    ibkr_account_long_short: Option<String>,
}

impl CommandListener {
    pub fn new(redis_url: &str, db: Option<Arc<PgClient>>) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let acct_lo = env::var("IBKR_ACCOUNT_LONG_ONLY")
            .ok()
            .filter(|s| !s.is_empty());
        let acct_ls = env::var("IBKR_ACCOUNT_LONG_SHORT")
            .ok()
            .filter(|s| !s.is_empty());

        info!(
            "IBKR account routing: long_only={}, long_short={}",
            acct_lo.as_deref().unwrap_or("(default)"),
            acct_ls.as_deref().unwrap_or("(default)")
        );

        Ok(Self {
            client,
            db,
            risk: StockRiskEngine::new(),
            ibkr_long_only_trader: None,
            ibkr_long_short_trader: None,
            futu_trader: None,
            ibkr_account_long_only: acct_lo,
            ibkr_account_long_short: acct_ls,
        })
    }

    pub fn set_traders(
        &mut self,
        ibkr_long_only: Option<Arc<IBKRTrader>>,
        ibkr_long_short: Option<Arc<IBKRTrader>>,
        futu: Option<Arc<FutuTrader>>,
    ) {
        self.ibkr_long_only_trader = ibkr_long_only;
        self.ibkr_long_short_trader = ibkr_long_short;
        self.futu_trader = futu;
    }

    /// Route a signal to the appropriate broker based on symbol format and exchange field
    fn route_signal(signal: &TradeSignal) -> BrokerRoute {
        // If exchange is explicitly set to "polygon", route to IBKR
        if let Some(ref exchange) = signal.exchange {
            if exchange == "polygon" {
                return BrokerRoute::Ibkr;
            }
        }

        let sym = &signal.symbol;

        // Futu-style symbols: "US.AAPL", "HK.0700", "SH.600000"
        if sym.starts_with("US.")
            || sym.starts_with("HK.")
            || sym.starts_with("SH.")
            || sym.starts_with("SZ.")
        {
            return BrokerRoute::Futu;
        }

        // Default: US stock tickers -> IBKR
        BrokerRoute::Ibkr
    }

    fn build_order_params(&self, signal: &TradeSignal) -> OrderParams {
        let order_type = match signal.order_type {
            common::events::OrderType::Market => BrokerOrderType::Market,
            common::events::OrderType::Limit => BrokerOrderType::Limit,
        };

        let account = match signal.mode.as_deref() {
            Some("long_only") => self.ibkr_account_long_only.clone(),
            Some("long_short") => self.ibkr_account_long_short.clone(),
            _ => None,
        };

        OrderParams {
            order_type,
            limit_price: signal.price,
            time_in_force: TimeInForce::Day,
            account,
        }
    }

    pub fn publish_update(&self, update: &OrderUpdate) -> Result<()> {
        let mut conn = self.client.get_connection()?;
        let json = serde_json::to_string(update)?;
        let _: () = conn.publish("order_updates", json)?;
        Ok(())
    }

    fn make_order_update(signal: &TradeSignal, result: &OrderResult) -> OrderUpdate {
        let status = match result.status.as_str() {
            "Filled" => OrderStatus::Filled,
            "Cancelled" | "Canceled" => OrderStatus::Cancelled,
            _ => OrderStatus::Pending,
        };
        OrderUpdate {
            order_id: result.order_id.clone(),
            signal_id: Some(signal.id),
            symbol: signal.symbol.clone(),
            status,
            filled_quantity: result.filled_qty,
            filled_avg_price: result.avg_price,
            timestamp: Utc::now(),
            message: Some(format!("Routed to {}", result.broker)),
        }
    }

    fn make_failed_update(signal: &TradeSignal, err: &str) -> OrderUpdate {
        OrderUpdate {
            order_id: String::new(),
            signal_id: Some(signal.id),
            symbol: signal.symbol.clone(),
            status: OrderStatus::Failed,
            filled_quantity: 0.0,
            filled_avg_price: 0.0,
            timestamp: Utc::now(),
            message: Some(err.to_string()),
        }
    }

    /// Determine asset type from exchange/routing context
    fn asset_type_for(signal: &TradeSignal) -> &'static str {
        match signal.exchange.as_deref() {
            Some("polygon") => "STK",
            Some("binance") | Some("okx") | Some("bybit") => "STK",
            _ => "STK",
        }
    }

    /// Persist an order to the trade_orders table.
    /// Uses float8 casts so tokio_postgres can serialize f64 natively;
    /// PostgreSQL implicitly converts float8 → numeric for the column.
    async fn persist_order(
        db: &PgClient,
        signal: &TradeSignal,
        result: &crate::traders::OrderResult,
    ) {
        let exchange = signal.exchange.as_deref().unwrap_or("IBKR");
        let side = signal.side.to_string();
        let order_type = signal.order_type.to_string();
        let asset_type = Self::asset_type_for(signal);
        let limit_price = signal.price;
        let account_id = match signal.mode.as_deref() {
            Some(m) => format!("ibkr_{}", m),
            None => "default".to_string(),
        };

        let mode = signal.mode.as_deref();

        let res = db
            .execute(
                "INSERT INTO trade_orders (order_id, exchange, symbol, asset_type, side, order_type, quantity, filled_qty, price, avg_price, status, strategy_id, account_id, mode, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7::float8, $8::float8, $9::float8, $10::float8, $11, $12, $13, $14, NOW(), NOW())",
                &[
                    &result.order_id,
                    &exchange,
                    &signal.symbol,
                    &asset_type,
                    &side,
                    &order_type,
                    &signal.quantity,
                    &result.filled_qty,
                    &limit_price,
                    &result.avg_price,
                    &result.status,
                    &signal.strategy_id,
                    &account_id,
                    &mode,
                ],
            )
            .await;

        if let Err(e) = res {
            warn!("Failed to persist order: {}", e);
        } else {
            info!("Persisted order {} to DB", result.order_id);
        }
    }

    /// Persist fill/execution data
    async fn persist_execution(db: &PgClient, result: &crate::traders::OrderResult) {
        if result.filled_qty <= 0.0 {
            return;
        }

        let execution_id = format!("exec_{}", result.order_id);
        let res = db
            .execute(
                "INSERT INTO trade_executions (execution_id, order_id, price, quantity, trade_time)
                 VALUES ($1, $2, $3::float8, $4::float8, NOW())",
                &[
                    &execution_id,
                    &result.order_id,
                    &result.avg_price,
                    &result.filled_qty,
                ],
            )
            .await;

        if let Err(e) = res {
            warn!("Failed to persist execution: {}", e);
        }
    }

    /// Upsert position after a fill
    async fn update_position(
        db: &PgClient,
        symbol: &str,
        exchange: &str,
        quantity: f64,
        price: f64,
        side: &str,
        mode: Option<&str>,
    ) {
        let signed_qty = if side == "Buy" { quantity } else { -quantity };
        let account_id = match mode {
            Some(m) => format!("ibkr_{}", m),
            None => "default".to_string(),
        };

        let res = db
            .execute(
                "INSERT INTO trade_positions (account_id, exchange, symbol, quantity, avg_price, updated_at)
                 VALUES ($1, $2, $3, $4::float8, $5::float8, NOW())
                 ON CONFLICT (account_id, exchange, symbol) DO UPDATE
                 SET quantity = trade_positions.quantity + $4::float8,
                     avg_price = CASE WHEN $4::float8 > 0 THEN
                         (trade_positions.quantity * trade_positions.avg_price + $4::float8 * $5::float8) / NULLIF(trade_positions.quantity + $4::float8, 0)
                     ELSE trade_positions.avg_price END,
                     updated_at = NOW()",
                &[&account_id, &exchange, &symbol, &signed_qty, &price],
            )
            .await;

        if let Err(e) = res {
            warn!("Failed to update position: {}", e);
        }
    }

    pub async fn listen_for_signals(&self) -> Result<()> {
        let mut conn = self.client.get_connection()?;
        let mut pubsub = conn.as_pubsub();
        pubsub.subscribe("trade_signals")?;

        info!("CommandListener: Subscribed to trade_signals");

        loop {
            let msg = pubsub.get_message()?;
            let payload: String = msg.get_payload()?;

            let signal = match serde_json::from_str::<TradeSignal>(&payload) {
                Ok(s) => s,
                Err(e) => {
                    warn!("Failed to parse trade signal: {}", e);
                    continue;
                }
            };

            info!(
                "Received signal: {} {} {} x{} (strategy: {}, exchange: {:?}, mode: {:?})",
                signal.id,
                signal.side,
                signal.symbol,
                signal.quantity,
                signal.strategy_id,
                signal.exchange,
                signal.mode
            );

            let route = Self::route_signal(&signal);
            let params = self.build_order_params(&signal);

            // Pre-trade risk check for IBKR orders
            if route == BrokerRoute::Ibkr {
                let risk_result = self.risk.check_pre_trade(&signal, &self.db).await;
                if !risk_result.approved {
                    warn!("Risk rejected signal {}: {}", signal.id, risk_result.reason);
                    let update = Self::make_failed_update(&signal, &risk_result.reason);
                    if let Err(e) = self.publish_update(&update) {
                        error!("Failed to publish risk rejection: {}", e);
                    }
                    continue;
                }
            }

            match route {
                BrokerRoute::Ibkr => {
                    let ibkr_trader = match signal.mode.as_deref() {
                        Some("long_short") => self.ibkr_long_short_trader.as_ref(),
                        _ => self.ibkr_long_only_trader.as_ref(),
                    };
                    if let Some(trader) = ibkr_trader {
                        // Skip pre-execution is_alive() gate — ibapi may have
                        // internally reconnected even when is_connected() returns
                        // false.  Let the actual buy/sell call determine liveness;
                        // a real connection failure will surface as an execution
                        // error handled below.
                        let trader = trader.clone();
                        let sig = signal.clone();
                        let redis_client = self.client.clone();
                        let db = self.db.clone();

                        tokio::spawn(async move {
                            let res = match sig.side {
                                common::events::OrderSide::Buy => {
                                    trader.buy(&sig.symbol, sig.quantity, &params).await
                                }
                                common::events::OrderSide::Sell => {
                                    trader.sell(&sig.symbol, sig.quantity, &params).await
                                }
                            };

                            match res {
                                Ok(result) => {
                                    info!(
                                        "IBKR execution OK: order_id={}, status={}, filled={}, avg_price={}",
                                        result.order_id, result.status, result.filled_qty, result.avg_price
                                    );

                                    // Persist to DB
                                    if let Some(ref db_client) = db {
                                        Self::persist_order(db_client, &sig, &result).await;
                                        Self::persist_execution(db_client, &result).await;
                                        if result.filled_qty > 0.0 {
                                            let exchange =
                                                sig.exchange.as_deref().unwrap_or("IBKR");
                                            Self::update_position(
                                                db_client,
                                                &sig.symbol,
                                                exchange,
                                                result.filled_qty,
                                                result.avg_price,
                                                &sig.side.to_string(),
                                                sig.mode.as_deref(),
                                            )
                                            .await;
                                        }
                                    }

                                    let update = Self::make_order_update(&sig, &result);
                                    if let Ok(mut conn) = redis_client.get_connection() {
                                        let json =
                                            serde_json::to_string(&update).unwrap_or_default();
                                        let _: std::result::Result<(), _> =
                                            conn.publish("order_updates", json);
                                    }
                                }
                                Err(e) => {
                                    error!("IBKR execution failed: {}", e);
                                    let update = Self::make_failed_update(&sig, &e.to_string());
                                    if let Ok(mut conn) = redis_client.get_connection() {
                                        let json =
                                            serde_json::to_string(&update).unwrap_or_default();
                                        let _: std::result::Result<(), _> =
                                            conn.publish("order_updates", json);
                                    }
                                }
                            }
                        });
                    } else {
                        let mode = signal.mode.as_deref().unwrap_or("long_only");
                        warn!(
                            "No IBKR trader configured for mode '{}', rejecting signal for {}",
                            mode, signal.symbol
                        );
                        let msg = format!("No IBKR trader configured for mode '{}'", mode);
                        let update = Self::make_failed_update(&signal, &msg);
                        if let Err(e) = self.publish_update(&update) {
                            error!("Failed to publish no-trader rejection: {}", e);
                        }
                    }
                }

                BrokerRoute::Futu => {
                    if let Some(trader) = &self.futu_trader {
                        let trader = trader.clone();
                        let sig = signal.clone();
                        let redis_client = self.client.clone();

                        tokio::spawn(async move {
                            let res = match sig.side {
                                common::events::OrderSide::Buy => {
                                    trader.buy(&sig.symbol, sig.quantity, &params).await
                                }
                                common::events::OrderSide::Sell => {
                                    trader.sell(&sig.symbol, sig.quantity, &params).await
                                }
                            };

                            match res {
                                Ok(result) => {
                                    info!("Futu execution OK: order_id={}", result.order_id);
                                    let update = Self::make_order_update(&sig, &result);
                                    if let Ok(mut conn) = redis_client.get_connection() {
                                        let json =
                                            serde_json::to_string(&update).unwrap_or_default();
                                        let _: std::result::Result<(), _> =
                                            conn.publish("order_updates", json);
                                    }
                                }
                                Err(e) => {
                                    error!("Futu execution failed: {}", e);
                                    let update = Self::make_failed_update(&sig, &e.to_string());
                                    if let Ok(mut conn) = redis_client.get_connection() {
                                        let json =
                                            serde_json::to_string(&update).unwrap_or_default();
                                        let _: std::result::Result<(), _> =
                                            conn.publish("order_updates", json);
                                    }
                                }
                            }
                        });
                    } else {
                        warn!(
                            "No Futu trader configured, rejecting signal for {}",
                            signal.symbol
                        );
                        let update = Self::make_failed_update(&signal, "No Futu trader configured");
                        if let Err(e) = self.publish_update(&update) {
                            error!("Failed to publish no-trader rejection: {}", e);
                        }
                    }
                }
            }
        }
    }
}
