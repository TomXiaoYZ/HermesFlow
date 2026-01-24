use crate::genetic::Genome;
use backtest_engine::factors::engineer::FeatureEngineer;
use backtest_engine::vm::vm::StackVM;
use chrono::{DateTime, Utc};
use ndarray::{Array2, Array3, Axis};
use sqlx::postgres::PgPool;
use sqlx::FromRow;
use std::collections::HashMap;

#[derive(Debug, FromRow)]
pub struct Candle {
    pub time: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub liquidity: f64,
    pub fdv: f64,
}

pub struct Backtester {
    pool: PgPool,
    vm: StackVM,
    // Cached data: Symbol -> (Features, Returns)
    // Features: (TestBatchSize, FeatDim, TimeSteps) ? No, per symbol.
    // VM takes (Batch, Feat, Time). If we test 1 symbol, Batch=1.
    // We can test multiple symbols in parallel or batch them.
    // For simplicity: Cache per symbol.
    cache: HashMap<String, (Array3<f64>, Array2<f64>)>, // Features, FutureReturns
}

impl Backtester {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            vm: StackVM::new(),
            cache: HashMap::new(),
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
                       COALESCE(fdv, 0) as fdv
                FROM mkt_equity_candles
                WHERE symbol = $1 AND resolution = '15m'
                AND time > NOW() - make_interval(days := $2)
                ORDER BY time ASC
                "#,
            )
            .bind(symbol)
            .bind(days as i32)
            .fetch_all(&self.pool)
            .await?;

            if rows.len() < 100 {
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

            for (i, c) in rows.iter().enumerate() {
                close[[0, i]] = c.close;
                open[[0, i]] = c.open;
                high[[0, i]] = c.high;
                low[[0, i]] = c.low;
                volume[[0, i]] = c.volume;
                liq[[0, i]] = c.liquidity;
                fdv[[0, i]] = c.fdv;
            }

            // Generate Features (12 dims)
            let features =
                FeatureEngineer::compute_features(&close, &open, &high, &low, &volume, &liq, &fdv);

            // Calculate Future Returns (Target)
            // For fitness, we want correlation with Next Return?
            // Or run a simulation?
            // Simulation is better.
            // But for simple fitness: Info Coefficient (IC).
            // Let's compute Next Close Return.
            // ret_t+1 = close[t+1] / close[t] - 1
            let mut future_ret = Array2::<f64>::zeros((1, len));
            for i in 0..len - 1 {
                let r = rows[i + 1].close / rows[i].close - 1.0;
                future_ret[[0, i]] = r;
            }

            self.cache.insert(symbol.clone(), (features, future_ret));
            loaded_count += 1;
        }

        tracing::info!("Loaded data for {} symbols", loaded_count);
        Ok(())
    }

    // Evaluate a single genome
    pub fn evaluate(&self, genome: &mut Genome) {
        if self.cache.is_empty() {
            // No data available yet. Return 0.0 (neutral) instead of -999 (error)
            // so the UI doesn't look broken.
            genome.fitness = 0.0;
            return;
        }

        let mut total_sharpe = 0.0;
        let mut total_ic = 0.0;
        let count = self.cache.len() as f64;

        for (_sym, (features, future_ret)) in &self.cache {
            // Run VM
            if let Some(signal) = self.vm.execute(&genome.tokens, features) {
                // signal is (1, time)
                // align with future_ret

                // 1. Calculate IC (Simple correlation)
                // Need to flatten
                let sig_slice = signal.as_slice().unwrap();
                let ret_slice = future_ret.as_slice().unwrap();

                // Truncate to match valid range (last point has no future return)
                let len = sig_slice.len() - 1;
                if len == 0 {
                    continue;
                }

                let mut sum_prod = 0.0;
                let mut sum_sig = 0.0;
                let mut sum_ret = 0.0;
                let mut sum_sig_sq = 0.0;
                let mut sum_ret_sq = 0.0;

                for i in 0..len {
                    let s = sig_slice[i];
                    let r = ret_slice[i];

                    // Filter NaNs
                    if s.is_nan() || r.is_nan() {
                        continue;
                    }

                    sum_prod += s * r;
                    sum_sig += s;
                    sum_ret += r;
                    sum_sig_sq += s * s;
                    sum_ret_sq += r * r;
                }

                let n = len as f64;
                let cov = n * sum_prod - sum_sig * sum_ret;
                let sig_std = (n * sum_sig_sq - sum_sig * sum_sig).sqrt();
                let ret_std = (n * sum_ret_sq - sum_ret * sum_ret).sqrt();

                let ic = if sig_std > 0.0 && ret_std > 0.0 {
                    cov / (sig_std * ret_std)
                } else {
                    0.0
                };

                // 2. Calculate PnL / Sharpe
                // Strategy Return = Signal * Future Return (Assuming position sizing = signal)
                let mut strat_returns = Vec::with_capacity(len);
                for i in 0..len {
                    let s = sig_slice[i].clamp(-1.0, 1.0); // Cap leverage
                    let r = ret_slice[i];
                    // Transaction costs? Ignore for now
                    strat_returns.push(s * r);
                }

                // Mean / Std
                let mean_ret: f64 = strat_returns.iter().sum::<f64>() / n;
                let var_ret: f64 = strat_returns
                    .iter()
                    .map(|x| (x - mean_ret).powi(2))
                    .sum::<f64>()
                    / n;
                let std_ret = var_ret.sqrt();

                let sharpe = if std_ret > 1e-6 {
                    mean_ret / std_ret * (252.0_f64 * 96.0_f64).sqrt() // Annualized (15m bars = 96/day)
                } else {
                    0.0
                };

                total_sharpe += sharpe;
                total_ic += ic;
            } else {
                // VM Invalid (e.g. stack underflow)
                genome.fitness = -1000.0; // Penalize heavy
                return;
            }
        }

        // Fitness Score
        // Combo of IC and Sharpe
        // Avoid overfitting to one asset by averaging
        let avg_sharpe = total_sharpe / count;
        // let avg_ic = total_ic / count;

        // Simple fitness: Sharpe
        // Check nan
        if avg_sharpe.is_nan() {
            genome.fitness = -999.0;
        } else {
            genome.fitness = avg_sharpe;
        }
    }
}
