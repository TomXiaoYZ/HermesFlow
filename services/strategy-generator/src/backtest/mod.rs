use crate::genetic::Genome;
use backtest_engine::config::FactorConfig;
use backtest_engine::factors::engineer::FeatureEngineer;
use backtest_engine::vm::vm::StackVM;
use chrono::{DateTime, Utc};
use ndarray::{Array2, Array3};
use rust_decimal::prelude::ToPrimitive;
use sqlx::postgres::PgPool;
use sqlx::FromRow;
use std::collections::HashMap;

pub mod data_frame;
pub mod portfolio;

#[derive(Debug, FromRow)]
pub struct Candle {
    pub time: DateTime<Utc>,
    pub open: rust_decimal::Decimal,
    pub high: rust_decimal::Decimal,
    pub low: rust_decimal::Decimal,
    pub close: rust_decimal::Decimal,
    pub volume: rust_decimal::Decimal,
    pub liquidity: rust_decimal::Decimal,
    pub fdv: rust_decimal::Decimal,
    pub amount: rust_decimal::Decimal, // Added for fallback
}

#[derive(Clone)]
pub struct CachedData {
    pub features: Array3<f64>,
    pub returns: Array2<f64>,
    pub liquidity: Array2<f64>,
    pub amount: Array2<f64>,
    pub timestamps: Vec<i64>,
}

#[derive(Clone, Copy, Debug)]
pub enum OptimizationMetric {
    #[allow(dead_code)]
    Sharpe,
    IC, // Information Coefficient (Spearman Correlation)
}

pub struct Backtester {
    pool: PgPool,
    vm: StackVM,
    cache: HashMap<String, CachedData>,
    factor_config: FactorConfig,
    pub metric: OptimizationMetric,
}

impl Backtester {
    pub fn new(pool: PgPool, factor_config: FactorConfig) -> Self {
        Self {
            pool,
            vm: StackVM::from_config(&factor_config),
            cache: HashMap::new(),
            factor_config,
            metric: {
                tracing::info!("Initializing Backtester with OptimizationMetric::IC");
                OptimizationMetric::IC
            },
        }
    }

    pub async fn load_data(&mut self, symbols: &[String], days: i64) -> anyhow::Result<()> {
        let mut loaded_count = 0;

        for symbol in symbols {
            // query postgres
            // Schema assumed from data-engine: mkt_equity_candles
            // resolution '15m' or '1h'
            let rows = sqlx::query_as::<_, Candle>(
                r#"
                SELECT time, open, high, low, close, 
                       COALESCE(volume, 0) as volume,
                       COALESCE(liquidity, 0) as liquidity, 
                       COALESCE(fdv, 0) as fdv,
                       COALESCE(amount, 0) as amount
                FROM mkt_equity_candles
                WHERE exchange = 'Birdeye' AND symbol = $1 AND resolution = '15m'
                AND time > NOW() - make_interval(days := $2)
                ORDER BY time ASC
                "#,
            )
            .bind(symbol)
            .bind(days as i32)
            .fetch_all(&self.pool)
            .await?;

            if rows.len() < 50 {
                tracing::warn!("Insufficient data for {}: {} rows", symbol, rows.len());
                continue;
            }

            // Convert to Array2
            let len = rows.len();
            let mut close = Array2::<f64>::zeros((1, len));
            let mut open = Array2::<f64>::zeros((1, len));
            let mut high = Array2::<f64>::zeros((1, len));
            let mut low = Array2::<f64>::zeros((1, len));
            let mut volume = Array2::<f64>::zeros((1, len));
            let mut liq = Array2::<f64>::zeros((1, len));
            let mut fdv = Array2::<f64>::zeros((1, len));
            let mut amount = Array2::<f64>::zeros((1, len));
            let mut timestamps = Vec::with_capacity(len);

            for (i, c) in rows.iter().enumerate() {
                close[[0, i]] = c.close.to_f64().unwrap_or(0.0);
                open[[0, i]] = c.open.to_f64().unwrap_or(0.0);
                high[[0, i]] = c.high.to_f64().unwrap_or(0.0);
                low[[0, i]] = c.low.to_f64().unwrap_or(0.0);
                volume[[0, i]] = c.volume.to_f64().unwrap_or(0.0);
                liq[[0, i]] = c.liquidity.to_f64().unwrap_or(0.0);
                fdv[[0, i]] = c.fdv.to_f64().unwrap_or(0.0);
                amount[[0, i]] = c.amount.to_f64().unwrap_or(0.0);
                timestamps.push(c.time.timestamp());
            }

            // Generate Features based on Config
            let features = FeatureEngineer::compute_features_from_config(
                &self.factor_config,
                &close,
                &open,
                &high,
                &low,
                &volume,
                &liq,
                &fdv,
            );

            // Calculate Future Returns (Target)
            // For fitness, we want correlation with Next Return?
            // Or run a simulation?
            // Simulation is better.
            // But for simple fitness: Info Coefficient (IC).
            // Let's compute Next Close Return.
            // ret_t+1 = close[t+1] / close[t] - 1
            let mut max_price = -f64::INFINITY;
            let mut min_price = f64::INFINITY;
            for x in close.iter() {
                if *x > max_price {
                    max_price = *x;
                }
                if *x < min_price {
                    min_price = *x;
                }
            }
            tracing::info!(
                "Symbol {} Price Range: [{}, {}]",
                symbol,
                min_price,
                max_price
            );

            let mut future_ret = Array2::<f64>::zeros((1, len));
            let mut max_ret = -f64::INFINITY;
            for i in 0..len - 1 {
                let curr = close[[0, i]];
                let next = close[[0, i + 1]];
                let r = if curr.abs() > 1e-9 {
                    let raw_r = next / curr - 1.0;
                    // Cap return to avoid explosion from bad data (e.g. 1000% in 15m)
                    raw_r.clamp(-0.99, 10.0)
                } else {
                    0.0
                };
                if r > max_ret {
                    max_ret = r;
                }
                future_ret[[0, i]] = r;
            }
            tracing::info!("Symbol {} Max Return: {}", symbol, max_ret);

            self.cache.insert(
                symbol.clone(),
                CachedData {
                    features,
                    returns: future_ret,
                    liquidity: liq,
                    amount,
                    timestamps,
                },
            );
            loaded_count += 1;
        }

        tracing::info!("Loaded data for {} symbols", loaded_count);
        Ok(())
    }

    // Evaluate a single genome
    pub fn evaluate(&self, genome: &mut Genome) {
        if self.cache.is_empty() {
            genome.fitness = 0.0;
            return;
        }

        let mut total_score = 0.0;
        let count = self.cache.len() as f64;

        for data in self.cache.values() {
            // Run VM
            if let Some(signal) = self.vm.execute(&genome.tokens, &data.features) {
                // Debug log (only once per generation/genome? too noisy. only if best?)
                // Just log the first one to confirm metric
                // tracing::info!("Evaluating with metric: {:?}", self.metric);
                // No, genome iteration is frequent.

                let sig_slice = signal.as_slice().unwrap();
                let ret_slice = data.returns.as_slice().unwrap();

                // Align lengths
                let len = sig_slice.len().min(ret_slice.len());
                if len < 2 {
                    continue;
                }

                let s = &sig_slice[..len];
                let r = &ret_slice[..len];

                match self.metric {
                    OptimizationMetric::IC => {
                        // Information Coefficient: Spearman Correlation(Signal, Return)
                        let ic = spearman_rank_corr(s, r);
                        total_score += ic;
                    }
                    OptimizationMetric::Sharpe => {
                        // Backtest Logic (Slippage + Costs)
                        let liq_slice = data.liquidity.as_slice().unwrap();
                        let amt_slice = data.amount.as_slice().unwrap();

                        let mut strat_returns = Vec::with_capacity(len);
                        let mut trade_count = 0;
                        let mut prev_sig = 0.0;
                        let portfolio_size = 10_000.0;
                        let base_fee = 0.001;

                        for i in 0..len {
                            let mut val_s = s[i].clamp(-1.0, 1.0);
                            let val_r = r[i];
                            let liq = liq_slice[i.min(liq_slice.len() - 1)];
                            let amt = amt_slice[i.min(amt_slice.len() - 1)];

                            let capacity = if liq > 0.0 { liq } else { amt * 0.1 };
                            let trade_val = val_s.abs() * portfolio_size;
                            let impact = trade_val / (capacity + 1e-9);

                            if impact > 0.05 {
                                val_s = 0.0;
                            }

                            if (val_s - prev_sig).abs() > 0.1 {
                                trade_count += 1;
                            }

                            let turnover = (val_s - prev_sig).abs();
                            let cost = if turnover > 0.0 {
                                turnover * (base_fee + impact)
                            } else {
                                0.0
                            };

                            let pnl = val_s * val_r - cost;
                            strat_returns.push(pnl);
                            prev_sig = val_s;
                        }

                        if trade_count < 5 {
                            total_score -= 5.0; // Penalty
                            continue;
                        }

                        let n = strat_returns.len() as f64;
                        let mean_ret: f64 = strat_returns.iter().sum::<f64>() / n;
                        let var_ret: f64 = strat_returns
                            .iter()
                            .map(|x| (x - mean_ret).powi(2))
                            .sum::<f64>()
                            / (n - 1.0);
                        let std_ret = var_ret.sqrt();

                        let sharpe = if std_ret > 1e-6 {
                            mean_ret / std_ret * (252.0f64 * 96.0f64).sqrt()
                        } else {
                            0.0
                        };

                        total_score += sharpe;
                    }
                }
            } else {
                genome.fitness = -1000.0; // Invalid genome
                return;
            }
        }

        let avg_score = total_score / count;
        if avg_score.is_nan() {
            genome.fitness = -999.0;
        } else {
            genome.fitness = avg_score;
        }
    }
}

// Helper functions for IC
fn spearman_rank_corr(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len();
    if n < 2 {
        return 0.0;
    }
    let rx = rank_vector(x);
    let ry = rank_vector(y);
    pearson_corr(&rx, &ry)
}

fn rank_vector(x: &[f64]) -> Vec<f64> {
    let mut indices: Vec<usize> = (0..x.len()).collect();
    // Helper sort
    indices.sort_by(|&a, &b| x[a].partial_cmp(&x[b]).unwrap_or(std::cmp::Ordering::Equal));
    let mut ranks = vec![0.0; x.len()];
    for (i, &idx) in indices.iter().enumerate() {
        ranks[idx] = i as f64;
    }
    ranks
}

fn pearson_corr(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len() as f64;
    let mean_x = x.iter().sum::<f64>() / n;
    let mean_y = y.iter().sum::<f64>() / n;

    let mut num = 0.0;
    let mut den_x = 0.0;
    let mut den_y = 0.0;

    for i in 0..x.len() {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        num += dx * dy;
        den_x += dx * dx;
        den_y += dy * dy;
    }

    if den_x <= 1e-12 || den_y <= 1e-12 {
        return 0.0;
    }
    num / (den_x.sqrt() * den_y.sqrt())
}

impl Backtester {
    // Continuation... existing methods needs to be preserved? No, I am terminating the impl block above.
    // Actually I need to be careful with the brace matching.
    // The replace block started at `pub fn evaluate...` inside `impl Backtester`.
    // So I should close the `impl Backtester` block before defining helper functions.
    // My updated content closes the brace `}` for `evaluate` then `}` for `impl Backtester`, then defines functions.
    // But wait, the original file has `pub async fn run_detailed_simulation` AFTER `evaluate`.
    // I CANNOT close `impl Backtester` here because I am only replacing `evaluate`.
    // The helper functions must be OUTSIDE `impl Backtester` or inside.
    // `ops.rs` functions are public or module level.
    // Best to put helpers as private logic inside `evaluate`, or just put them outside.
    // BUT `run_detailed_simulation` is part of `impl Backtester` and it follows `evaluate`.
    // So if I close `}` I break the file.

    // Solution: Put helper functions *before* `evaluate` or *inside* `evaluate` (can't do nested fn easily in Rust without being closures).
    // Or put them at the end of the file? I can't target end of file easily.
    // I can put them inside `evaluate` as closures? No, too complex.
    // Use `replace_file_content` to replace `evaluate`... but I can't insert code at EOF easily.

    // I will insert helper functions *before* `evaluate`? No, I am replacing `evaluate`.

    // Let's modify the plan:
    // 1. Add helper functions at the end of the file (using `replace_file_content` targeting the last line `}` of the file? No, risky).
    // 2. Or, just implement the logic inline if it's not too long? It's ~40 lines.
    // 3. Or, define them as `fn` inside `impl Backtester` (methods)? They don't need `self`. They can be `associated functions` (static methods).
    // Yes, `fn spearman...` inside `impl Backtester` is fine.

    // Let's add them as private associated functions to `impl Backtester`.
    // I will REPLACE `evaluate` and append the new functions *after* it, but *before* `run_detailed_simulation`.

    pub async fn run_detailed_simulation(
        &mut self,
        genome: &[i32],
        symbol: &str,
        days: i64,
    ) -> anyhow::Result<serde_json::Value> {
        // 1. Ensure data is loaded
        if !self.cache.contains_key(symbol) {
            self.load_data(&[symbol.to_string()], days).await?;
        }

        if let Some(_data) = self.cache.get(symbol) {
            // Access from cache
        } else {
            return Err(anyhow::anyhow!("Data not found (logic error)"));
        }

        let data = self.cache.get(symbol).unwrap();
        let features = &data.features;
        let future_ret = &data.returns;
        let liquidity = &data.liquidity;
        let amount = &data.amount;

        // VM expects usize
        let genome_usize: Vec<usize> = genome.iter().map(|&x| x as usize).collect();

        if let Some(signal) = self.vm.execute(&genome_usize, features) {
            let sig_slice = signal.as_slice().unwrap();
            let ret_slice = future_ret.as_slice().unwrap();

            // Min length
            let len = sig_slice
                .len()
                .min(ret_slice.len())
                .min(features.shape()[2]);

            let mut equity_curve = Vec::with_capacity(len);
            let mut current_equity = 1.0;
            let mut win = 0;
            let mut total = 0;
            let mut prev_sig = 0.0;
            let portfolio_size = 10_000.0;
            let base_fee = 0.001;

            for i in 0..len {
                let mut s = sig_slice[i].clamp(-1.0, 1.0);
                let r = ret_slice[i];
                let liq = liquidity[[0, i]];
                let amt = amount[[0, i]];

                let capacity = if liq > 0.0 { liq } else { amt * 0.1 };

                // Adaptive Check
                let trade_val = s.abs() * portfolio_size;
                let impact = trade_val / (capacity + 1e-9);

                if impact > 0.05 {
                    s = 0.0;
                }

                let turnover = (s - prev_sig).abs();
                let cost = if turnover > 0.0 {
                    turnover * (base_fee + impact)
                } else {
                    0.0
                };

                let pnl = s * r - cost;

                current_equity *= 1.0 + pnl;

                // Bankruptcy check
                if current_equity <= 0.0 {
                    current_equity = 0.0;
                    break;
                }

                if turnover > 0.0 {
                    // Trade occurred
                    // Check if effective trade change
                    if (s - prev_sig).abs() > 0.1 {
                        total += 1;
                        if pnl > 0.0 {
                            win += 1;
                        }
                    }
                }

                equity_curve.push(serde_json::json!({
                    "i": i,
                    "equity": current_equity,
                    "sig": s,
                    "ret": r,
                    "cost": cost,
                    "impact": impact
                }));
                prev_sig = s;
            }

            let total_ret = current_equity - 1.0;
            let win_rate = if total > 0 {
                win as f64 / total as f64
            } else {
                0.0
            };

            return Ok(serde_json::json!({
                "symbol": symbol,
                "days": days,
                "metrics": {
                    "total_return": total_ret,
                    "final_equity": current_equity,
                    "win_rate": win_rate,
                    "total_trades": total
                },
                "equity_curve": equity_curve
            }));
        }

        Err(anyhow::anyhow!("VM execution failed"))
    }

    pub async fn run_portfolio_simulation(
        &mut self,
        genome: &[i32],
        days: i64,
    ) -> anyhow::Result<serde_json::Value> {
        // Ensure data loaded
        if self.cache.is_empty() {
            use sqlx::Row;
            let rows = sqlx::query("SELECT address FROM active_tokens WHERE is_active = true")
                .fetch_all(&self.pool)
                .await?;

            let symbols: Vec<String> = rows.into_iter().map(|r| r.get("address")).collect();
            if !symbols.is_empty() {
                self.load_data(&symbols, days).await?;
            }
        }

        let mut pb = portfolio::PortfolioBacktester::new();
        pb.run(genome, &self.cache, days).await
    }
}
