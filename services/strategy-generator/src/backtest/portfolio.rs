use crate::backtest::data_frame::TimeDataFrame;
use crate::backtest::CachedData;
use anyhow;
use backtest_engine::config::FactorConfig;
use backtest_engine::vm::vm::StackVM;
use serde_json::json;
use std::collections::HashMap;
use tracing::{info, warn};

pub struct PortfolioBacktester {
    vm: StackVM,
    pub(crate) exchange: String,
    pub(crate) resolution: String,
}

impl PortfolioBacktester {
    pub fn new(factor_config: &FactorConfig, exchange: String, resolution: String) -> Self {
        Self {
            vm: StackVM::from_config(factor_config),
            exchange,
            resolution,
        }
    }

    /// Base transaction fee for the exchange.
    pub(crate) fn base_fee(&self) -> f64 {
        if self.exchange == "Polygon" {
            0.0001
        } else {
            0.001
        }
    }

    /// Estimate trade capacity.
    pub(crate) fn capacity(&self, liquidity: f64, amount: f64) -> f64 {
        if self.exchange == "Polygon" {
            amount.max(1e6)
        } else if liquidity > 0.0 {
            liquidity
        } else {
            amount * 0.1
        }
    }

    /// Annualization factor for Sharpe ratio.
    pub(crate) fn annualization_factor(&self) -> f64 {
        match self.resolution.as_str() {
            "1d" => 252.0_f64.sqrt(),
            "1h" => {
                if self.exchange == "Polygon" {
                    (252.0_f64 * 6.5).sqrt()
                } else {
                    (365.0_f64 * 24.0).sqrt()
                }
            }
            "15m" => {
                if self.exchange == "Polygon" {
                    (252.0_f64 * 6.5 * 4.0).sqrt()
                } else {
                    (365.0_f64 * 96.0).sqrt()
                }
            }
            _ => (252.0_f64 * 96.0).sqrt(),
        }
    }

    /// Run a portfolio simulation across ALL cached symbols.
    pub async fn run(
        &mut self,
        genome: &[i32],
        cache: &HashMap<String, CachedData>,
        days: i64,
    ) -> anyhow::Result<serde_json::Value> {
        info!(
            "Starting Portfolio Simulation for {} symbols...",
            cache.len()
        );

        // 1. Time Alignment
        let df = TimeDataFrame::align(cache);
        let len = df.timestamps.len();

        if len == 0 {
            return Err(anyhow::anyhow!(
                "No aligned data found for portfolio simulation"
            ));
        }

        // 2. Simulation State
        let mut equity_curve = Vec::with_capacity(len);
        let mut current_equity = 1.0;
        let portfolio_size = 10_000.0;
        let fee = self.base_fee();

        // Pre-convert genome for VM
        let genome_usize: Vec<usize> = genome.iter().map(|&x| x as usize).collect();

        // Map Symbol -> Previous Weight (for turnover)
        let mut prev_weights: HashMap<String, f64> = HashMap::new();
        for sym in df.returns.keys() {
            prev_weights.insert(sym.clone(), 0.0);
        }

        // Pre-calculate Signals for all symbols
        let mut all_signals: HashMap<String, Vec<f64>> = HashMap::new();

        for (symbol, data) in cache {
            if let Some(signal) = self.vm.execute(&genome_usize, &data.features) {
                all_signals.insert(symbol.clone(), signal.into_raw_vec());
            } else {
                warn!("VM execution failed for {}", symbol);
            }
        }

        // Align signals to global timestamps
        let mut aligned_signals: HashMap<String, Vec<f64>> = HashMap::new();

        for (symbol, signal_vec) in &all_signals {
            let data = cache.get(symbol).unwrap();
            let mut aligned = vec![0.0; len];

            let mut ts_to_val: HashMap<i64, f64> = HashMap::new();
            for (i, &ts) in data.timestamps.iter().enumerate() {
                if i < signal_vec.len() {
                    ts_to_val.insert(ts, signal_vec[i]);
                }
            }

            for (i, &ts) in df.timestamps.iter().enumerate() {
                aligned[i] = *ts_to_val.get(&ts).unwrap_or(&0.0);
            }
            aligned_signals.insert(symbol.clone(), aligned);
        }

        // Loop over time
        for t in 0..len {
            let mut total_abs = 0.0;

            for sigs in aligned_signals.values() {
                let s = sigs[t].clamp(-1.0, 1.0);
                if s.abs() > 0.1 {
                    total_abs += s.abs();
                }
            }

            let scaler = 1.0f64.max(total_abs);

            let mut step_pnl = 0.0;
            let mut _step_turnover = 0.0;
            let mut step_cost = 0.0;

            for (sym, sigs) in &aligned_signals {
                let raw_s = sigs[t].clamp(-1.0, 1.0);
                let w = if raw_s.abs() > 0.1 {
                    raw_s / scaler
                } else {
                    0.0
                };

                let prev_w = *prev_weights.get(sym).unwrap_or(&0.0);
                let turnover = (w - prev_w).abs();

                let liq_vec = df.liquidity.get(sym).unwrap();
                let amt_vec = df.amounts.get(sym).unwrap();
                let liq = liq_vec[t];
                let amt = amt_vec[t];
                let cap = self.capacity(liq, amt);

                let trade_val = turnover * portfolio_size;
                let impact = trade_val / (cap + 1e-9);

                let cost = if turnover > 0.0 {
                    turnover * (fee + impact)
                } else {
                    0.0
                };

                let ret_vec = df.returns.get(sym).unwrap();
                let r = ret_vec[t];

                let contrib = w * r - cost;

                step_pnl += contrib;
                step_cost += cost;
                _step_turnover += turnover;

                prev_weights.insert(sym.clone(), w);
            }

            current_equity *= 1.0 + step_pnl;

            if current_equity <= 0.0 {
                current_equity = 0.0;
                equity_curve.push(json!({
                    "t": df.timestamps[t],
                    "equity": 0.0,
                    "pnl": step_pnl,
                    "cost": step_cost,
                    "active_assets": total_abs,
                }));
                break;
            }

            equity_curve.push(json!({
                "t": df.timestamps[t],
                "equity": current_equity,
                "pnl": step_pnl,
                "cost": step_cost,
                "active_assets": total_abs,
            }));
        }

        // Metrics Calculation
        let total_ret = current_equity - 1.0;

        let mut max_equity = 1.0;
        let mut max_drawdown = 0.0;
        let mut returns = Vec::with_capacity(equity_curve.len());

        for i in 1..equity_curve.len() {
            let eq_curr = equity_curve[i]["equity"].as_f64().unwrap_or(1.0);
            let eq_prev = equity_curve[i - 1]["equity"].as_f64().unwrap_or(1.0);
            let r = eq_curr / eq_prev - 1.0;
            returns.push(r);

            if eq_curr > max_equity {
                max_equity = eq_curr;
            }
            let dd = (max_equity - eq_curr) / max_equity;
            if dd > max_drawdown {
                max_drawdown = dd;
            }
        }

        // Sharpe
        let n = returns.len() as f64;
        let mean_ret = if n > 0.0 {
            returns.iter().sum::<f64>() / n
        } else {
            0.0
        };
        let var_ret = if n > 1.0 {
            returns.iter().map(|&x| (x - mean_ret).powi(2)).sum::<f64>() / (n - 1.0)
        } else {
            0.0
        };
        let std_ret = var_ret.sqrt();

        let sharpe = if std_ret > 1e-9 {
            mean_ret / std_ret * self.annualization_factor()
        } else {
            0.0
        };

        Ok(json!({
            "symbol": "UNIVERSAL_PORTFOLIO",
            "days": days,
            "metrics": {
                "total_return": total_ret,
                "final_equity": current_equity,
                "sharpe_ratio": sharpe,
                "max_drawdown": max_drawdown,
                "total_trades": len,
                "win_rate": 0.0
            },
            "equity_curve": equity_curve
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a PortfolioBacktester for testing (needs a minimal FactorConfig).
    fn make_backtester(exchange: &str, resolution: &str) -> PortfolioBacktester {
        use backtest_engine::config::{FactorConfig, FactorDefinition, NormalizationType};
        let config = FactorConfig {
            active_factors: vec![FactorDefinition {
                id: 0,
                name: "return".to_string(),
                description: "test".to_string(),
                normalization: NormalizationType::None,
            }],
        };
        PortfolioBacktester::new(&config, exchange.to_string(), resolution.to_string())
    }

    // ── base_fee ───────────────────────────────────────────────────────

    #[test]
    fn base_fee_polygon() {
        let bt = make_backtester("Polygon", "1h");
        assert!((bt.base_fee() - 0.0001).abs() < 1e-10);
    }

    #[test]
    fn base_fee_crypto() {
        let bt = make_backtester("Binance", "1h");
        assert!((bt.base_fee() - 0.001).abs() < 1e-10);
    }

    #[test]
    fn base_fee_other_exchange() {
        let bt = make_backtester("OKX", "1h");
        assert!((bt.base_fee() - 0.001).abs() < 1e-10);
    }

    // ── capacity ───────────────────────────────────────────────────────

    #[test]
    fn capacity_polygon_always_at_least_1m() {
        let bt = make_backtester("Polygon", "1h");
        // Polygon: amount.max(1e6)
        assert!((bt.capacity(0.0, 500.0) - 1e6).abs() < 1.0);
        assert!((bt.capacity(1e9, 2e6) - 2e6).abs() < 1.0);
    }

    #[test]
    fn capacity_crypto_uses_liquidity() {
        let bt = make_backtester("Binance", "1h");
        assert!((bt.capacity(5e5, 1e4) - 5e5).abs() < 1.0);
    }

    #[test]
    fn capacity_crypto_zero_liquidity_fallback() {
        let bt = make_backtester("Binance", "1h");
        // Zero liquidity → amount * 0.1
        assert!((bt.capacity(0.0, 1e4) - 1e3).abs() < 1.0);
    }

    // ── annualization_factor ───────────────────────────────────────────

    #[test]
    fn annualization_daily() {
        let bt = make_backtester("Polygon", "1d");
        let expected = 252.0_f64.sqrt();
        assert!(
            (bt.annualization_factor() - expected).abs() < 1e-6,
            "daily annualization: expected {}, got {}",
            expected,
            bt.annualization_factor()
        );
    }

    #[test]
    fn annualization_hourly_polygon() {
        let bt = make_backtester("Polygon", "1h");
        let expected = (252.0_f64 * 6.5).sqrt();
        assert!(
            (bt.annualization_factor() - expected).abs() < 1e-6,
            "1h Polygon: expected {}, got {}",
            expected,
            bt.annualization_factor()
        );
    }

    #[test]
    fn annualization_hourly_crypto() {
        let bt = make_backtester("Binance", "1h");
        let expected = (365.0_f64 * 24.0).sqrt();
        assert!(
            (bt.annualization_factor() - expected).abs() < 1e-6,
            "1h Binance: expected {}, got {}",
            expected,
            bt.annualization_factor()
        );
    }

    #[test]
    fn annualization_15m_polygon() {
        let bt = make_backtester("Polygon", "15m");
        let expected = (252.0_f64 * 6.5 * 4.0).sqrt();
        assert!(
            (bt.annualization_factor() - expected).abs() < 1e-6,
            "15m Polygon: expected {}, got {}",
            expected,
            bt.annualization_factor()
        );
    }

    #[test]
    fn annualization_15m_crypto() {
        let bt = make_backtester("Binance", "15m");
        let expected = (365.0_f64 * 96.0).sqrt();
        assert!(
            (bt.annualization_factor() - expected).abs() < 1e-6,
            "15m Binance: expected {}, got {}",
            expected,
            bt.annualization_factor()
        );
    }

    // ── Weight normalization logic (extracted from run loop) ───────────

    #[test]
    fn weight_normalization_scaling() {
        // Simulate the signal → weight logic from the run() loop
        let signals: Vec<f64> = vec![0.8, -0.5, 0.3, 0.05]; // 0.05 < 0.1 → filtered out
        let mut total_abs = 0.0_f64;
        for &s in &signals {
            let clamped = s.clamp(-1.0, 1.0);
            if clamped.abs() > 0.1 {
                total_abs += clamped.abs();
            }
        }
        // total_abs = 0.8 + 0.5 + 0.3 = 1.6
        assert!((total_abs - 1.6).abs() < 1e-10);

        let scaler = 1.0_f64.max(total_abs); // 1.6
        let weights: Vec<f64> = signals
            .iter()
            .map(|&s| {
                let clamped: f64 = s.clamp(-1.0, 1.0);
                if clamped.abs() > 0.1 {
                    clamped / scaler
                } else {
                    0.0
                }
            })
            .collect();

        // Verify weights sum to <= 1.0
        let abs_sum: f64 = weights.iter().map(|w: &f64| w.abs()).sum();
        assert!(
            abs_sum <= 1.0 + 1e-10,
            "abs weights should sum to <= 1.0, got {}",
            abs_sum
        );

        // Verify filtered signal (0.05) has zero weight
        assert_eq!(weights[3], 0.0);

        // Verify proportions
        assert!((weights[0] - 0.8 / 1.6).abs() < 1e-10); // 0.5
        assert!((weights[1] - (-0.5 / 1.6)).abs() < 1e-10); // -0.3125
        assert!((weights[2] - 0.3 / 1.6).abs() < 1e-10); // 0.1875
    }

    #[test]
    fn weight_normalization_no_scaling_needed() {
        // If total_abs <= 1.0, scaler = 1.0 (no scaling)
        let signals: Vec<f64> = vec![0.3, -0.2];
        let total_abs: f64 = signals
            .iter()
            .map(|&s| s.clamp(-1.0_f64, 1.0).abs())
            .filter(|&a| a > 0.1)
            .sum();
        assert!((total_abs - 0.5).abs() < 1e-10);
        let scaler = 1.0_f64.max(total_abs);
        assert!((scaler - 1.0).abs() < 1e-10, "no scaling needed");
    }

    // ── Equity curve / drawdown calculation ────────────────────────────

    #[test]
    fn drawdown_calculation() {
        // Simulate equity: [1.0, 1.1, 1.05, 1.2, 0.9]
        let equity: Vec<f64> = vec![1.0, 1.1, 1.05, 1.2, 0.9];

        let mut max_eq = equity[0];
        let mut max_dd = 0.0_f64;
        for &eq in &equity[1..] {
            if eq > max_eq {
                max_eq = eq;
            }
            let dd = (max_eq - eq) / max_eq;
            if dd > max_dd {
                max_dd = dd;
            }
        }

        // Max equity reaches 1.2 at index 3, drops to 0.9
        // DD = (1.2 - 0.9) / 1.2 = 0.25
        assert!(
            (max_dd - 0.25).abs() < 1e-10,
            "max drawdown should be 0.25, got {}",
            max_dd
        );
    }

    #[test]
    fn sharpe_ratio_calculation() {
        // Period returns: [0.01, 0.02, -0.01, 0.03, 0.01]
        let returns = vec![0.01, 0.02, -0.01, 0.03, 0.01];
        let n = returns.len() as f64;
        let mean_ret = returns.iter().sum::<f64>() / n; // 0.012
        let var_ret = returns.iter().map(|&x| (x - mean_ret).powi(2)).sum::<f64>() / (n - 1.0);
        let std_ret = var_ret.sqrt();

        // Polygon 1h annualization factor
        let ann_factor = (252.0_f64 * 6.5).sqrt();
        let sharpe = mean_ret / std_ret * ann_factor;

        assert!(sharpe > 0.0, "sharpe should be positive for positive mean");
        assert!(sharpe.is_finite());
        assert!((mean_ret - 0.012).abs() < 1e-10);
    }

    #[test]
    fn equity_zero_halts() {
        // If equity goes to 0 or negative, simulation should stop
        let mut current_equity = 1.0_f64;
        let pnls: Vec<f64> = vec![0.01, -1.5]; // loss > 100%
        let mut stopped = false;
        for &pnl in &pnls {
            current_equity *= 1.0 + pnl;
            if current_equity <= 0.0 {
                current_equity = 0.0;
                stopped = true;
                break;
            }
        }
        assert!(stopped, "should halt on equity <= 0");
        assert_eq!(current_equity, 0.0);
    }

    // ── Cost model ─────────────────────────────────────────────────────

    #[test]
    fn turnover_cost_model() {
        let fee = 0.0001; // Polygon base fee
        let portfolio_size = 10_000.0;

        // Turnover from 0 to 0.5 weight
        let turnover = 0.5;
        let trade_val = turnover * portfolio_size; // 5000
        let cap = 1e6; // Polygon default
        let impact = trade_val / cap; // 0.005
        let cost = turnover * (fee + impact);

        assert!((cost - 0.5_f64 * (0.0001 + 0.005)).abs() < 1e-10);
        // Cost is small relative to portfolio
        assert!(cost < 0.01, "cost should be small fraction of portfolio");
    }
}
