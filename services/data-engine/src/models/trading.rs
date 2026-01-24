use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub symbol: String,
    pub action: String, // BUY, SELL
    pub quantity: f64,
    pub order_type: String, // MKT, LMT
    pub price: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub id: String, // Internal ID
    pub ib_order_id: i32,
    pub status: String, // Pending, Filled, Cancelled
    pub filled_quantity: f64,
    pub avg_fill_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub quantity: f64,
    pub avg_cost: f64,
    pub market_price: f64,
    pub market_value: f64,
    pub unrealized_pnl: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSummary {
    pub net_liquidation: f64,
    pub total_cash: f64,
    pub buying_power: f64,
    pub currency: String,
}

/// Order entity (Database)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub ib_order_id: i32,
    pub symbol: String,
    pub action: String, // BUY, SELL
    pub quantity: Decimal,
    pub order_type: String, // MKT, LMT
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Trade entity (Database)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: Uuid,
    pub order_id: Option<Uuid>,
    pub symbol: String,
    pub quantity: Decimal,
    pub price: Decimal,
    pub commission: Option<Decimal>,
    pub executed_at: DateTime<Utc>,
}
