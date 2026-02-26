pub mod futu_trader;
pub mod ibkr_trader;
pub mod paper_trader;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unified order parameters for all broker types
#[derive(Debug, Clone)]
pub struct OrderParams {
    pub order_type: BrokerOrderType,
    pub limit_price: Option<f64>,
    pub time_in_force: TimeInForce,
    /// IBKR account ID to route the order to (for FA/sub-account setups).
    /// When set, the order's `account` field is populated so IBKR routes it
    /// to the correct sub-account. Ignored by non-IBKR brokers.
    pub account: Option<String>,
}

#[derive(Debug, Clone)]
pub enum BrokerOrderType {
    Market,
    Limit,
    MarketOnClose,
}

#[derive(Debug, Clone)]
pub enum TimeInForce {
    Day,
    GoodTilCancel,
    ImmediateOrCancel,
}

/// Result of a trade execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResult {
    pub order_id: String,
    pub status: String,
    pub filled_qty: f64,
    pub avg_price: f64,
    pub broker: String,
    pub timestamp: DateTime<Utc>,
}

/// Account summary from a broker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountSummary {
    pub net_liquidation: f64,
    pub cash: f64,
    pub buying_power: f64,
    pub currency: String,
}

/// A position held at a broker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerPosition {
    pub symbol: String,
    pub quantity: f64,
    pub avg_cost: f64,
    pub market_value: f64,
    pub unrealized_pnl: f64,
    /// IBKR account that holds this position (empty for non-IBKR brokers).
    #[serde(default)]
    pub account: String,
}

/// Unified trader trait for all brokers (IBKR, Futu, Paper, etc.)
#[async_trait]
pub trait Trader: Send + Sync {
    fn broker_name(&self) -> &str;

    async fn buy(&self, symbol: &str, quantity: f64, params: &OrderParams) -> Result<OrderResult>;
    async fn sell(&self, symbol: &str, quantity: f64, params: &OrderParams) -> Result<OrderResult>;
    async fn cancel_order(&self, order_id: &str) -> Result<()>;
    async fn get_positions(&self) -> Result<Vec<BrokerPosition>>;
    async fn get_account_summary(&self) -> Result<AccountSummary>;

    /// Returns per-account summaries keyed by IBKR account ID (e.g. "DUxxxxxxx").
    /// Default implementation returns a single entry with empty key.
    async fn get_account_summaries(&self) -> Result<HashMap<String, AccountSummary>> {
        let summary = self.get_account_summary().await?;
        let mut map = HashMap::new();
        map.insert(String::new(), summary);
        Ok(map)
    }
}

/// P6-2C: Execution quality metrics for a single fill.
///
/// Captures slippage, fill rate, and latency for every executed order.
/// Used for:
/// - Promotion criteria (max acceptable slippage guard)
/// - Ongoing execution quality monitoring
/// - Broker comparison and routing optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionQuality {
    pub order_id: String,
    pub exchange: String,
    pub symbol: String,
    pub side: String,
    pub expected_price: f64,
    pub fill_price: f64,
    /// Slippage in basis points: (fill - expected) / expected * 10000
    /// Positive = worse than expected (paid more / received less)
    pub slippage_bps: f64,
    pub quantity: f64,
    pub fill_rate: f64,
    pub latency_ms: i64,
    pub broker: String,
    pub account_id: Option<String>,
    pub mode: Option<String>,
}

impl ExecutionQuality {
    /// Compute execution quality from order parameters and result.
    #[allow(clippy::too_many_arguments)]
    pub fn from_fill(
        order_id: &str,
        exchange: &str,
        symbol: &str,
        side: &str,
        expected_price: f64,
        result: &OrderResult,
        requested_qty: f64,
        order_submitted_at: DateTime<Utc>,
        account_id: Option<&str>,
        mode: Option<&str>,
    ) -> Self {
        let slippage_bps = if expected_price.abs() > 1e-12 {
            match side {
                "buy" => (result.avg_price - expected_price) / expected_price * 10_000.0,
                "sell" => (expected_price - result.avg_price) / expected_price * 10_000.0,
                _ => 0.0,
            }
        } else {
            0.0
        };

        let fill_rate = if requested_qty.abs() > 1e-12 {
            (result.filled_qty / requested_qty).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let latency_ms = (result.timestamp - order_submitted_at).num_milliseconds();

        Self {
            order_id: order_id.to_string(),
            exchange: exchange.to_string(),
            symbol: symbol.to_string(),
            side: side.to_string(),
            expected_price,
            fill_price: result.avg_price,
            slippage_bps,
            quantity: result.filled_qty,
            fill_rate,
            latency_ms,
            broker: result.broker.clone(),
            account_id: account_id.map(|s| s.to_string()),
            mode: mode.map(|s| s.to_string()),
        }
    }
}

impl Default for OrderParams {
    fn default() -> Self {
        Self {
            order_type: BrokerOrderType::Market,
            limit_price: None,
            time_in_force: TimeInForce::Day,
            account: None,
        }
    }
}
