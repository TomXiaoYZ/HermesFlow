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
    exchange: String,
    resolution: String,
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
    fn base_fee(&self) -> f64 {
        if self.exchange == "Polygon" {
            0.0001
        } else {
            0.001
        }
    }

    /// Estimate trade capacity.
    fn capacity(&self, liquidity: f64, amount: f64) -> f64 {
        if self.exchange == "Polygon" {
            amount.max(1e6)
        } else if liquidity > 0.0 {
            liquidity
        } else {
            amount * 0.1
        }
    }

    /// Annualization factor for Sharpe ratio.
    fn annualization_factor(&self) -> f64 {
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
