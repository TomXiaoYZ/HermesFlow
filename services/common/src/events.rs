use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataUpdate {
    pub symbol: String,
    pub price: f64,
    pub volume: f64,
    pub timestamp: DateTime<Utc>,
    pub source: String, // e.g., "binance", "ibkr"
}

#[derive(Debug, Clone, Serialize, Deserialize, Display, EnumString, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize, Display, EnumString, PartialEq)]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeSignal {
    pub id: Uuid,
    pub symbol: String,
    pub side: OrderSide,
    pub quantity: f64, // For tokens: amount of tokens. For crypto buy with quote: maybe amount in quote currency?
    // Let's assume quantity in BASE asset for now (e.g., 100 AAPL, 10 SOL)
    pub price: Option<f64>, // For Limit orders
    pub order_type: OrderType,
    pub timestamp: DateTime<Utc>,
    pub reason: String,
    pub strategy_id: String,
    #[serde(default)]
    pub exchange: Option<String>, // "polygon", "binance", etc. None for legacy Solana signals
}

#[derive(Debug, Clone, Serialize, Deserialize, Display, EnumString, PartialEq)]
pub enum OrderStatus {
    Pending,
    Filled,
    PartiallyFilled,
    Cancelled,
    Rejected,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderUpdate {
    pub order_id: String,        // Exchange Order ID or Internal ID
    pub signal_id: Option<Uuid>, // Correlation ID
    pub symbol: String,
    pub status: OrderStatus,
    pub filled_quantity: f64,
    pub filled_avg_price: f64,
    pub timestamp: DateTime<Utc>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionUpdate {
    pub symbol: String,
    pub quantity: f64,
    pub market_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioUpdate {
    pub timestamp: DateTime<Utc>,
    pub cash: f64,
    pub positions: Vec<PositionUpdate>,
    pub total_equity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyLog {
    pub timestamp: DateTime<Utc>,
    pub strategy_id: String,
    pub symbol: String,
    pub action: String,  // e.g., "Analyzing", "Signal Generated", "Rejected"
    pub message: String, // e.g., "Z-Score: 1.5 < Threshold 2.0"
}
