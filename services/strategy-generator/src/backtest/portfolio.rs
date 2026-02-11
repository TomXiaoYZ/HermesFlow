use crate::backtest::data_frame::TimeDataFrame;
use crate::backtest::CachedData;
use anyhow;
use backtest_engine::vm::vm::StackVM;
use serde_json::json;
use std::collections::HashMap;
use tracing::{info, warn}; // Explicit import

pub struct PortfolioBacktester {
    vm: StackVM,
}

impl PortfolioBacktester {
    pub fn new() -> Self {
        Self { vm: StackVM::new() }
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
        let portfolio_size = 10_000.0; // Base capital
        let base_fee = 0.001; // 0.1%

        // Pre-convert genome for VM
        let genome_usize: Vec<usize> = genome.iter().map(|&x| x as usize).collect();

        // Map Symbol -> Previous Weight (for turnover)
        let mut prev_weights: HashMap<String, f64> = HashMap::new();
        for sym in df.returns.keys() {
            prev_weights.insert(sym.clone(), 0.0);
        }

        // 4. Restarting Logic Structure
        // Step 1: Pre-calculate Signals for all symbols
        let mut all_signals: HashMap<String, Vec<f64>> = HashMap::new();

        for (symbol, data) in cache {
            if let Some(signal) = self.vm.execute(&genome_usize, &data.features) {
                // Convert Array to Vec
                all_signals.insert(symbol.clone(), signal.into_raw_vec());
            } else {
                warn!("VM execution failed for {}", symbol);
            }
        }

        // Step 2: Align SIGNALS (not just returns)
        // We reuse TimeDataFrame logic but extended to signals.
        // Actually `TimeDataFrame` struct has `returns`, `amounts`.
        // We can add `signals` to it?
        // Or create a new helper `align_signals`.
        // Let's just do it inline here for now using `df` timestamps.

        let mut aligned_signals: HashMap<String, Vec<f64>> = HashMap::new();

        for (symbol, signal_vec) in &all_signals {
            let data = cache.get(symbol).unwrap();
            let mut aligned = vec![0.0; len];

            // Map: timestamp -> value
            // source indicies
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

        // Step 3: Loop Time (Now we have aligned signals & returns)
        for t in 0..len {
            // A. Calculate Allocation Weights
            let mut total_abs = 0.0;

            // Collect active signals
            for sigs in aligned_signals.values() {
                let s = sigs[t].clamp(-1.0, 1.0);
                if s.abs() > 0.1 {
                    // Threshold
                    total_abs += s.abs();
                }
            }

            // Normalize
            // If total_abs > 1.0, scale down.
            // If total_abs <= 1.0, keep as is? (Leverage < 1).
            // Strategy: Alloc = Signal / Max(1.0, Total_Abs).
            // This ensures Sum(|Alloc|) <= 1.0.
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

                // Cost
                // Estimate impact? (Need aligned liquidity).
                // `TimeDataFrame` has `df.liquidity`.
                let liq_vec = df.liquidity.get(sym).unwrap();
                let amt_vec = df.amounts.get(sym).unwrap();
                let liq = liq_vec[t];
                let amt = amt_vec[t];
                let capacity = if liq > 0.0 { liq } else { amt * 0.1 };

                let trade_val = turnover * portfolio_size;
                let impact = trade_val / (capacity + 1e-9);

                let cost = if turnover > 0.0 {
                    turnover * (base_fee + impact)
                } else {
                    0.0
                };

                // Return
                let ret_vec = df.returns.get(sym).unwrap();
                let r = ret_vec[t];

                // PnL contribution
                // w * r - cost
                let contrib = w * r - cost;

                step_pnl += contrib;
                step_cost += cost;
                _step_turnover += turnover;

                prev_weights.insert(sym.clone(), w);
            }

            current_equity *= 1.0 + step_pnl;

            // Bankruptcy check
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
                "active_assets": total_abs, // Rough count proxy
            }));
        }

        // Metrics Calculation
        let total_ret = current_equity - 1.0;

        // Calculate Drawdown & Sharpe
        let mut max_equity = 1.0;
        let mut max_drawdown = 0.0;
        let mut returns = Vec::with_capacity(equity_curve.len());

        // Skip first element (no return)
        // Use actual equity_curve length (may be shorter if bankruptcy occurred)
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
            mean_ret / std_ret * (252.0f64 * 96.0f64).sqrt() // 96 bars per day (15m)
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
                "total_trades": len, // Treat every step as potential rebalance
                "win_rate": 0.0 // Hard to define for portfolio rebalancing
            },
            "equity_curve": equity_curve
        }))
    }
}
