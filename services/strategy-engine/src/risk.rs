use common::events::TradeSignal;
use std::env;
use tracing::warn;

#[derive(Debug, Clone)]
pub struct RiskConfig {
    pub min_liquidity_usd: f64,
    pub max_drawdown_limit: f64, // e.g. 0.20 (20% daily)
    pub trade_size_pct: f64,     // % of equity per stock trade (e.g. 0.005 = 0.5%)
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            min_liquidity_usd: 1.0,
            max_drawdown_limit: 0.20,
            trade_size_pct: env::var("TRADE_SIZE_PCT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.005),
        }
    }
}

pub struct RiskEngine {
    config: RiskConfig,
    current_equity: f64,
    daily_start_equity: f64,
}

/// Check if a symbol looks like a US stock ticker.
/// Matches 1-5 uppercase ASCII letters, optionally followed by a dot and 1-2
/// uppercase letters (e.g. AAPL, A, BRK.A, BRK.B, PBR.A).
pub fn is_stock_symbol(symbol: &str) -> bool {
    if symbol.is_empty() || symbol.len() > 7 {
        return false;
    }
    let parts: Vec<&str> = symbol.splitn(2, '.').collect();
    let base = parts[0];
    if base.is_empty() || base.len() > 5 || !base.chars().all(|c| c.is_ascii_uppercase()) {
        return false;
    }
    if let Some(suffix) = parts.get(1) {
        !suffix.is_empty() && suffix.len() <= 2 && suffix.chars().all(|c| c.is_ascii_uppercase())
    } else {
        true
    }
}

impl Default for RiskEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RiskEngine {
    pub fn new() -> Self {
        Self {
            config: RiskConfig::default(),
            current_equity: 0.0,
            daily_start_equity: 0.0,
        }
    }

    pub fn update_equity(&mut self, equity: f64) {
        self.current_equity = equity;
    }

    /// Calculate stock position size in shares as a percentage of account equity.
    /// trade_size_pct (e.g. 0.005 = 0.5%) determines the USD value per trade,
    /// then we floor-divide by price to get whole shares.
    pub fn calculate_stock_entry_shares(&self, current_price: f64) -> f64 {
        if current_price <= 0.0 || self.current_equity <= 0.0 {
            return 0.0;
        }
        let trade_value = self.current_equity * self.config.trade_size_pct;
        (trade_value / current_price).floor()
    }

    /// Async check for signal validity
    pub async fn check(&self, signal: &TradeSignal, liquidity: Option<f64>) -> bool {
        let is_stock = is_stock_symbol(&signal.symbol);

        // 1. Check Liquidity (skip for stocks — they have exchange-level liquidity)
        if !is_stock {
            if let Some(liq) = liquidity {
                if liq < self.config.min_liquidity_usd {
                    warn!(
                        "Risk Reject: Liquidity ${} < Min ${}",
                        liq, self.config.min_liquidity_usd
                    );
                    return false;
                }
            }
        }

        // 2. Check Drawdown
        if self.daily_start_equity > 0.0 {
            let dd = (self.daily_start_equity - self.current_equity) / self.daily_start_equity;
            if dd > self.config.max_drawdown_limit {
                warn!("Risk Reject: Daily Drawdown {:.2}% > Limit", dd * 100.0);
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use common::events::{OrderSide, OrderType};
    use uuid::Uuid;

    #[test]
    fn test_is_stock_symbol() {
        assert!(is_stock_symbol("AAPL"));
        assert!(is_stock_symbol("MSFT"));
        assert!(is_stock_symbol("A"));
        // Dot-suffix share classes (e.g. BRK.A, BRK.B, PBR.A)
        assert!(is_stock_symbol("BRK.A"));
        assert!(is_stock_symbol("BRK.B"));
        assert!(is_stock_symbol("PBR.A"));
        // Negative cases
        assert!(!is_stock_symbol(
            "So11111111111111111111111111111111111111112"
        ));
        assert!(!is_stock_symbol("sol"));
        assert!(!is_stock_symbol(""));
        assert!(!is_stock_symbol("TOOLONGSYMBOL"));
        assert!(!is_stock_symbol("A.BCD")); // suffix too long
        assert!(!is_stock_symbol(".A")); // empty base
    }

    #[test]
    fn test_stock_entry_shares_pct_based() {
        let mut engine = RiskEngine::new();
        // Default trade_size_pct = 0.005 (0.5%)
        // Equity = $1,000,000 → trade_value = $5,000
        engine.update_equity(1_000_000.0);
        let shares = engine.calculate_stock_entry_shares(180.0);
        assert_eq!(shares, 27.0); // floor(5000/180) = 27

        // Equity = $100,000 → trade_value = $500
        engine.update_equity(100_000.0);
        let shares = engine.calculate_stock_entry_shares(180.0);
        assert_eq!(shares, 2.0); // floor(500/180) = 2

        // Zero equity → 0 shares
        engine.update_equity(0.0);
        let shares = engine.calculate_stock_entry_shares(180.0);
        assert_eq!(shares, 0.0);

        // Zero price → 0 shares
        engine.update_equity(1_000_000.0);
        let shares = engine.calculate_stock_entry_shares(0.0);
        assert_eq!(shares, 0.0);
    }

    #[tokio::test]
    async fn test_risk_check_liquidity() {
        let engine = RiskEngine::new();

        let signal = TradeSignal {
            id: Uuid::new_v4(),
            strategy_id: "test".to_string(),
            symbol: "TEST_TOKEN_LONG_NAME_XXXX".to_string(),
            side: OrderSide::Buy,
            quantity: 1.0,
            price: Some(1.0),
            order_type: OrderType::Market,
            timestamp: Utc::now(),
            reason: "Testing".to_string(),
            exchange: None,
            mode: None,
        };

        assert!(engine.check(&signal, Some(1000.0)).await);
        assert!(!engine.check(&signal, Some(0.5)).await);
    }

    #[tokio::test]
    async fn test_stock_risk_checks() {
        let mut engine = RiskEngine::new();
        engine.update_equity(1_000_000.0);

        // Stock signal — should skip liquidity checks
        let signal = TradeSignal {
            id: Uuid::new_v4(),
            strategy_id: "test".to_string(),
            symbol: "AAPL".to_string(),
            side: OrderSide::Buy,
            quantity: 10.0,
            price: Some(180.0),
            order_type: OrderType::Market,
            timestamp: Utc::now(),
            reason: "Testing".to_string(),
            exchange: Some("polygon".to_string()),
            mode: Some("long_only".to_string()),
        };

        // Stock entry with equity → approved (no fixed limits)
        assert!(engine.check(&signal, None).await);
    }
}
