use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub token_address: String,
    pub symbol: String,
    pub amount_held: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub cost_basis: f64,
    pub open_time: DateTime<Utc>,
    pub is_moonbag: bool,
    pub highest_price: f64,
}

#[derive(Debug, Default)]
pub struct PortfolioManager {
    pub positions: HashMap<String, Position>,
    pub cash_balance: f64,
}

impl PortfolioManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_position(&mut self, token: String, symbol: String, price: f64, amount: f64, cost: f64) {
        let pos = Position {
            token_address: token.clone(),
            symbol,
            amount_held: amount,
            entry_price: price,
            current_price: price,
            cost_basis: cost,
            open_time: Utc::now(),
            is_moonbag: false,
            highest_price: price,
        };
        self.positions.insert(token, pos);
    }

    pub fn update_price(&mut self, token: &str, price: f64) {
        if let Some(pos) = self.positions.get_mut(token) {
            pos.current_price = price;
            if price > pos.highest_price {
                pos.highest_price = price;
            }
        }
    }
}
