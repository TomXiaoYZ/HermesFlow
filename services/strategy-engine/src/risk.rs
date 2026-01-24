use common::events::{OrderSide, TradeSignal};
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct RiskConfig {
    pub min_liquidity_usd: f64,
    pub max_position_size_portion: f64, // e.g., 0.1 (10%)
    pub max_drawdown_limit: f64,        // e.g. 0.05 (5% daily)
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            min_liquidity_usd: 1.0, // Effectively disabled for testing
            max_position_size_portion: 0.5, // Allow up to 50% allocation for small account testing
            max_drawdown_limit: 0.20, // Looser drawdown limit
        }
    }
}

pub struct RiskEngine {
    config: RiskConfig,
    current_equity: f64, // Mock current equity
    daily_start_equity: f64,
}

impl RiskEngine {
    pub fn new() -> Self {
        Self {
            config: RiskConfig::default(),
            current_equity: 60.0, // Aligned with ~0.4 SOL
            daily_start_equity: 60.0,
        }
    }

    // In a real system, we'd sync equity from Account Service or Portfolio Manager
    pub fn update_equity(&mut self, equity: f64) {
        self.current_equity = equity;
    }

    pub fn check(&self, signal: &TradeSignal, liquidity: Option<f64>) -> bool {
        // 1. Check Liquidity
        if let Some(liq) = liquidity {
            if liq < self.config.min_liquidity_usd {
                warn!(
                    "Risk Reject: Liquidity ${} < Min ${}",
                    liq, self.config.min_liquidity_usd
                );
                return false;
            }
        }

        // 2. Check Drawdown
        let dd = (self.daily_start_equity - self.current_equity) / self.daily_start_equity;
        if dd > self.config.max_drawdown_limit {
            warn!(
                "Risk Reject: Daily Drawdown {:.2}% > Limit {:.2}%",
                dd * 100.0,
                self.config.max_drawdown_limit * 100.0
            );
            return false;
        }

        // 3. Position Sizing (Check if signal quantity is within limits)
        let max_pos_usd = self.current_equity * self.config.max_position_size_portion;

        // Approx check: if buy, check cost. If sell, assume we hold it (PortfolioManager check should happen before or here).
        // Since we don't have price in signal (Market Order), we can't strictly check USD size here without price.
        // We'll pass for now in MVP.

        true
    }
}
