pub mod futu_trader;
pub mod ibkr_trader;
pub mod raydium_trader;
pub mod solana_trader;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

/// Unified trader trait for all brokers (IBKR, Futu, Solana, etc.)
#[async_trait]
pub trait Trader: Send + Sync {
    fn broker_name(&self) -> &str;

    async fn buy(&self, symbol: &str, quantity: f64, params: &OrderParams) -> Result<OrderResult>;
    async fn sell(&self, symbol: &str, quantity: f64, params: &OrderParams) -> Result<OrderResult>;
    async fn cancel_order(&self, order_id: &str) -> Result<()>;
    async fn get_positions(&self) -> Result<Vec<BrokerPosition>>;
    async fn get_account_summary(&self) -> Result<AccountSummary>;
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
