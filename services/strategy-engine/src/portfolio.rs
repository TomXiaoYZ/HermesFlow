use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use tracing::info;

// Crypto defaults (meme tokens — high volatility)
pub const DEFAULT_STOP_LOSS_PCT: f64 = -0.15;
pub const DEFAULT_TP_MOONBAG_PCT: f64 = 1.00; // +100% (2x)
pub const DEFAULT_MOONBAG_SELL_RATIO: f64 = 0.50; // Sell 50%
pub const DEFAULT_TRAILING_ACTIVATION: f64 = 0.50; // Activate after +50% gain
pub const DEFAULT_TRAILING_DROP: f64 = 0.10; // Drop 10% from high triggers sell

// Stock defaults (lower volatility)
pub const DEFAULT_STOCK_STOP_LOSS_PCT: f64 = -0.05;
pub const DEFAULT_STOCK_TP_MOONBAG_PCT: f64 = 0.15;
pub const DEFAULT_STOCK_MOONBAG_SELL_RATIO: f64 = 0.50;
pub const DEFAULT_STOCK_TRAILING_ACTIVATION: f64 = 0.08;
pub const DEFAULT_STOCK_TRAILING_DROP: f64 = 0.03;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PositionStatus {
    Active,
    Closing, // Algo decided to exit, signal sent
    Closed,  // Fully exited
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PositionDirection {
    Long,
    Short,
}

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
    pub lowest_price: f64,
    pub direction: PositionDirection,
    pub status: PositionStatus,
}

impl Position {
    pub fn pnl_pct(&self) -> f64 {
        if self.entry_price == 0.0 {
            return 0.0;
        }
        match self.direction {
            PositionDirection::Long => (self.current_price - self.entry_price) / self.entry_price,
            PositionDirection::Short => (self.entry_price - self.current_price) / self.entry_price,
        }
    }

    /// Maximum unrealized gain from entry.
    /// Long: how far price rose from entry. Short: how far price dropped from entry.
    pub fn max_gain_pct(&self) -> f64 {
        if self.entry_price == 0.0 {
            return 0.0;
        }
        match self.direction {
            PositionDirection::Long => (self.highest_price - self.entry_price) / self.entry_price,
            PositionDirection::Short => (self.entry_price - self.lowest_price) / self.entry_price,
        }
    }

    /// Drawdown from peak gain.
    /// Long: price dropped from highest. Short: price bounced up from lowest.
    pub fn drawdown_from_peak(&self) -> f64 {
        match self.direction {
            PositionDirection::Long => {
                if self.highest_price == 0.0 {
                    return 0.0;
                }
                (self.highest_price - self.current_price) / self.highest_price
            }
            PositionDirection::Short => {
                if self.lowest_price == 0.0 {
                    return 0.0;
                }
                (self.current_price - self.lowest_price) / self.lowest_price
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PortfolioConfig {
    pub stop_loss_pct: f64,
    pub tp_moonbag_pct: f64,
    pub moonbag_sell_ratio: f64,
    pub trailing_activation: f64,
    pub trailing_drop: f64,
}

impl Default for PortfolioConfig {
    fn default() -> Self {
        Self {
            stop_loss_pct: DEFAULT_STOP_LOSS_PCT,
            tp_moonbag_pct: DEFAULT_TP_MOONBAG_PCT,
            moonbag_sell_ratio: DEFAULT_MOONBAG_SELL_RATIO,
            trailing_activation: DEFAULT_TRAILING_ACTIVATION,
            trailing_drop: DEFAULT_TRAILING_DROP,
        }
    }
}

impl PortfolioConfig {
    /// Config tuned for US stocks — tighter thresholds for lower volatility
    pub fn stock_defaults() -> Self {
        Self {
            stop_loss_pct: env::var("STOCK_STOP_LOSS_PCT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_STOCK_STOP_LOSS_PCT),
            tp_moonbag_pct: env::var("STOCK_TP_MOONBAG_PCT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_STOCK_TP_MOONBAG_PCT),
            moonbag_sell_ratio: env::var("STOCK_MOONBAG_SELL_RATIO")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_STOCK_MOONBAG_SELL_RATIO),
            trailing_activation: env::var("STOCK_TRAILING_ACTIVATION")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_STOCK_TRAILING_ACTIVATION),
            trailing_drop: env::var("STOCK_TRAILING_DROP")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_STOCK_TRAILING_DROP),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExitReason {
    StopLoss(f64),          // Current PnL
    MoonbagTP(f64),         // Current PnL
    TrailingStop(f64, f64), // MaxGain, Drawdown
}

#[derive(Debug, Clone)]
pub struct ExitSignal {
    pub token_address: String,
    pub symbol: String,
    pub sell_ratio: f64, // 0.0 - 1.0
    pub reason: ExitReason,
}

#[derive(Debug)]
pub struct PortfolioManager {
    pub positions: HashMap<String, Position>,
    pub cash_balance: f64,
    pub config: PortfolioConfig,
}

impl Default for PortfolioManager {
    fn default() -> Self {
        Self {
            positions: HashMap::new(),
            cash_balance: 0.0,
            config: PortfolioConfig::default(),
        }
    }
}

impl PortfolioManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: PortfolioConfig) -> Self {
        Self {
            positions: HashMap::new(),
            cash_balance: 0.0,
            config,
        }
    }

    pub fn add_position(
        &mut self,
        token: String,
        symbol: String,
        price: f64,
        amount: f64,
        cost: f64,
        direction: PositionDirection,
    ) {
        let pos = Position {
            token_address: token.clone(),
            symbol: symbol.clone(),
            amount_held: amount,
            entry_price: price,
            current_price: price,
            cost_basis: cost,
            open_time: Utc::now(),
            is_moonbag: false,
            highest_price: price,
            lowest_price: price,
            direction,
            status: PositionStatus::Active,
        };
        self.positions.insert(token, pos);
    }

    pub fn update_price(&mut self, token: &str, price: f64) {
        if let Some(pos) = self.positions.get_mut(token) {
            pos.current_price = price;
            if price > pos.highest_price {
                pos.highest_price = price;
            }
            if price < pos.lowest_price {
                pos.lowest_price = price;
            }
        }
    }

    pub fn update_holding(&mut self, token: &str, new_amount: f64) {
        if new_amount <= 0.000001 {
            // Epsilon for float zero
            if self.positions.remove(token).is_some() {
                info!("Position {} closed fully.", token);
            }
        } else if let Some(pos) = self.positions.get_mut(token) {
            pos.amount_held = new_amount;
        }
    }

    pub fn mark_moonbag(&mut self, token: &str) {
        if let Some(pos) = self.positions.get_mut(token) {
            pos.is_moonbag = true;
        }
    }

    /// Check all active positions for exit conditions based on latest prices.
    /// Works for both long and short positions using direction-aware PnL.
    pub fn check_exits(&self) -> Vec<ExitSignal> {
        let mut signals = Vec::new();

        for pos in self.positions.values() {
            let pnl = pos.pnl_pct();

            // 1. Stop Loss (direction-aware: pnl is negative when losing for both sides)
            if pnl <= self.config.stop_loss_pct {
                signals.push(ExitSignal {
                    token_address: pos.token_address.clone(),
                    symbol: pos.symbol.clone(),
                    sell_ratio: 1.0,
                    reason: ExitReason::StopLoss(pnl),
                });
                continue;
            }

            // 2. Moonbag Take Profit (Target 1)
            if !pos.is_moonbag && pnl >= self.config.tp_moonbag_pct {
                signals.push(ExitSignal {
                    token_address: pos.token_address.clone(),
                    symbol: pos.symbol.clone(),
                    sell_ratio: self.config.moonbag_sell_ratio,
                    reason: ExitReason::MoonbagTP(pnl),
                });
                continue;
            }

            // 3. Trailing Stop — uses direction-aware peak drawdown
            let max_gain = pos.max_gain_pct();
            let dd = pos.drawdown_from_peak();

            if max_gain > self.config.trailing_activation && dd > self.config.trailing_drop {
                signals.push(ExitSignal {
                    token_address: pos.token_address.clone(),
                    symbol: pos.symbol.clone(),
                    sell_ratio: 1.0,
                    reason: ExitReason::TrailingStop(max_gain, dd),
                });
                continue;
            }
        }

        signals
    }
}
