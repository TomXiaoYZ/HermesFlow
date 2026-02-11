use anyhow::Result;
use chrono::Utc;
use common::events::{OrderStatus, OrderUpdate, TradeSignal};
use redis::Commands;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::traders::futu_trader::FutuTrader;
use crate::traders::ibkr_trader::IBKRTrader;
use crate::traders::solana_trader::SolanaTrader;
use crate::traders::{BrokerOrderType, OrderParams, TimeInForce, Trader};

/// Determines which broker should handle a given symbol
#[derive(Debug, Clone, PartialEq)]
enum BrokerRoute {
    Solana,
    Ibkr,
    Futu,
    #[allow(dead_code)]
    Unknown,
}

pub struct CommandListener {
    client: redis::Client,
    pub solana_trader: Option<Arc<SolanaTrader>>,
    pub ibkr_trader: Option<Arc<IBKRTrader>>,
    pub futu_trader: Option<Arc<FutuTrader>>,
}

impl CommandListener {
    pub fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self {
            client,
            solana_trader: None,
            ibkr_trader: None,
            futu_trader: None,
        })
    }

    pub fn set_traders(
        &mut self,
        solana: Option<Arc<SolanaTrader>>,
        ibkr: Option<Arc<IBKRTrader>>,
        futu: Option<Arc<FutuTrader>>,
    ) {
        self.solana_trader = solana;
        self.ibkr_trader = ibkr;
        self.futu_trader = futu;
    }

    /// Route a signal to the appropriate broker based on symbol format
    fn route_signal(signal: &TradeSignal) -> BrokerRoute {
        let sym = &signal.symbol;

        // Futu-style symbols: "US.AAPL", "HK.0700", "SH.600000"
        if sym.starts_with("US.")
            || sym.starts_with("HK.")
            || sym.starts_with("SH.")
            || sym.starts_with("SZ.")
        {
            return BrokerRoute::Futu;
        }

        // Solana addresses are base58, typically 32-44 chars
        if sym.len() > 30 || sym == "SOL" {
            return BrokerRoute::Solana;
        }

        // Default: US stock tickers -> IBKR
        BrokerRoute::Ibkr
    }

    fn build_order_params(signal: &TradeSignal) -> OrderParams {
        let order_type = match signal.order_type {
            common::events::OrderType::Market => BrokerOrderType::Market,
            common::events::OrderType::Limit => BrokerOrderType::Limit,
        };

        OrderParams {
            order_type,
            limit_price: signal.price,
            time_in_force: TimeInForce::Day,
        }
    }

    pub fn publish_update(&self, update: &OrderUpdate) -> Result<()> {
        let mut conn = self.client.get_connection()?;
        let json = serde_json::to_string(update)?;
        let _: () = conn.publish("order_updates", json)?;
        Ok(())
    }

    fn make_order_update(
        signal: &TradeSignal,
        result: &crate::traders::OrderResult,
    ) -> OrderUpdate {
        OrderUpdate {
            order_id: result.order_id.clone(),
            signal_id: Some(signal.id),
            symbol: signal.symbol.clone(),
            status: OrderStatus::Pending,
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
                "Received signal: {} {} {} x{} (strategy: {})",
                signal.id, signal.side, signal.symbol, signal.quantity, signal.strategy_id
            );

            let route = Self::route_signal(&signal);
            let params = Self::build_order_params(&signal);

            match route {
                BrokerRoute::Solana => {
                    if let Some(trader) = &self.solana_trader {
                        let trader = trader.clone();
                        let sig = signal.clone();
                        let redis_client = self.client.clone();

                        tokio::spawn(async move {
                            let res = match sig.side {
                                common::events::OrderSide::Buy => {
                                    trader.buy(&sig.symbol, sig.quantity, 100).await
                                }
                                common::events::OrderSide::Sell => {
                                    trader.sell(&sig.symbol, sig.quantity, 100).await
                                }
                            };
                            match res {
                                Ok(tx) => {
                                    info!("Solana execution OK: {}", tx);
                                    let update = OrderUpdate {
                                        order_id: tx.clone(),
                                        signal_id: Some(sig.id),
                                        symbol: sig.symbol.clone(),
                                        status: OrderStatus::Filled,
                                        filled_quantity: sig.quantity,
                                        filled_avg_price: 0.0,
                                        timestamp: Utc::now(),
                                        message: Some(format!(
                                            "Solana tx: {}",
                                            tx
                                        )),
                                    };
                                    if let Ok(mut conn) = redis_client.get_connection() {
                                        let json =
                                            serde_json::to_string(&update).unwrap_or_default();
                                        let _: std::result::Result<(), _> =
                                            conn.publish("order_updates", json);
                                    }
                                }
                                Err(e) => {
                                    error!("Solana execution failed: {}", e);
                                    let update =
                                        Self::make_failed_update(&sig, &e.to_string());
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
                        warn!("No Solana trader configured, dropping signal");
                    }
                }

                BrokerRoute::Ibkr => {
                    if let Some(trader) = &self.ibkr_trader {
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
                                    info!("IBKR execution OK: order_id={}", result.order_id);
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
                        warn!(
                            "No IBKR trader configured, dropping signal for {}",
                            signal.symbol
                        );
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
                                    let update =
                                        Self::make_failed_update(&sig, &e.to_string());
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
                            "No Futu trader configured, dropping signal for {}",
                            signal.symbol
                        );
                    }
                }

                BrokerRoute::Unknown => {
                    warn!("Unknown route for symbol: {}", signal.symbol);
                }
            }
        }
    }
}
