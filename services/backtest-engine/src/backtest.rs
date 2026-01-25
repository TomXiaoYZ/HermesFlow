use ndarray::{Array2, Axis};
use tracing::{info, warn};

pub struct BacktestConfig {
    pub trade_size_usd: f64,
    pub min_liquidity_usd: f64,
    pub base_fee_pct: f64, // e.g. 0.0060 (0.6%)
    pub impact_slippage_max: f64, // e.g. 0.05 (5%)
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            trade_size_usd: 1000.0,
            min_liquidity_usd: 500_000.0,
            base_fee_pct: 0.0060,
            impact_slippage_max: 0.05,
        }
    }
}

pub struct BacktestRunner {
    pub config: BacktestConfig,
}

impl BacktestRunner {
    pub fn new(config: BacktestConfig) -> Self {
        Self { config }
    }

    /// Evaluate strategy performance
    /// factors: (batch, time) - Raw logits from VM (before sigmoid)
    /// liquidity: (batch, time) - Liquidity timeline
    /// target_ret: (batch, time) - Future returns (e.g. next tick return) to calculate PnL against
    pub fn evaluate(
        &self,
        factors: &Array2<f64>,
        liquidity: &Array2<f64>,
        target_ret: &Array2<f64>,
    ) -> (f64, f64) {
        // 1. Signal Generation
        // Sigmoid
        let signal = factors.mapv(|x| 1.0 / (1.0 + (-x).exp())); // sigmoid

        // 2. Safety Mask
        // is_safe = (liquidity > min_liq)
        let is_safe = liquidity.mapv(|l| if l > self.config.min_liquidity_usd { 1.0 } else { 0.0 });
        
        // 3. Position Logic
        // position = (signal > 0.85) * is_safe
        // Note: Python used float math. We do the same.
        let position = ndarray::Zip::from(&signal)
            .and(&is_safe)
            .map_collect(|&s, &safe| {
                if s > 0.85 && safe > 0.0 {
                    1.0
                } else {
                    0.0
                }
            });

        // 4. Transaction Costs
        // Impact = trade_size / (liquidity + epsilon)
        // Clamped to max
        let impact = liquidity.mapv(|l| {
            let i = self.config.trade_size_usd / (l + 1e-9);
            i.min(self.config.impact_slippage_max).max(0.0)
        });

        let total_slippage = self.config.base_fee_pct + &impact;

        // Turnover = abs(pos - prev_pos)
        // Need to shift position manually since no roll op in ndarray easy
        // We can iterate columns or just use a loop.
        // For array2 (batch, time), we iterate time.
        
        let mut turnover = Array2::zeros(position.dim());
        // prev_pos starts at 0
        let (batch, time) = position.dim();
        
        // Col 0: turnover = abs(pos[0] - 0) = pos[0]
        // Col t: turnover = abs(pos[t] - pos[t-1])
        
        for t in 0..time {
            if t == 0 {
                let col = position.index_axis(Axis(1), t);
                let mut out_col = turnover.index_axis_mut(Axis(1), t);
                out_col.assign(&col); // abs(pos - 0) is pos (since 0 or 1)
            } else {
                let col = position.index_axis(Axis(1), t);
                let prev = position.index_axis(Axis(1), t - 1);
                
                let mut out_col = turnover.index_axis_mut(Axis(1), t);
                // abs(col - prev)
                // ndarray doesn't support easy abs diff on views directly without creating new array usually,
                // but we can zip
                ndarray::Zip::from(&mut out_col)
                    .and(&col)
                    .and(&prev)
                    .for_each(|o, &c, &p| {
                         let diff: f64 = c - p;
                         *o = diff.abs();
                    });
            }
        }

        let tx_cost = &turnover * &total_slippage;

        // 5. PnL
        // Gross PnL = position * target_ret
        // Note: target_ret should be the return achieved by holding this period.
        // Usually target_ret is shifted (Next Return).
        // Assuming caller provides aligned return.
        let gross_pnl = &position * target_ret;
        let net_pnl = &gross_pnl - &tx_cost;

        // 6. Aggregation
        // Sum across time -> Cum Return per asset
        let cum_ret = net_pnl.sum_axis(Axis(1));

        // Drawdowns: count times net_pnl < -0.05
        // (big_drawdowns = (net_pnl < -0.05).float().sum(dim=1))
        let big_drawdowns = net_pnl.map_axis(Axis(1), |row| {
             row.iter().filter(|&&r| r < -0.05).count() as f64
        });

        // Score = CumRet - (BigDD * 2.0)
        let score = &cum_ret - &(&big_drawdowns * 2.0);

        // Activity Check
        // if activity < 5: score = -10.0
        let activity = position.sum_axis(Axis(1));
        
        let final_scores = ndarray::Zip::from(&score)
            .and(&activity)
            .map_collect(|&s, &act| {
                if act < 5.0 {
                    -10.0
                } else {
                    s
                }
            });

        // Median Score
        // Sort and pick median
        let mut scores_vec: Vec<f64> = final_scores.to_vec();
        scores_vec.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        let median_score = if scores_vec.is_empty() {
            0.0
        } else {
            let mid = scores_vec.len() / 2;
             if scores_vec.len() % 2 == 0 {
                (scores_vec[mid - 1] + scores_vec[mid]) / 2.0
            } else {
                scores_vec[mid]
            }
        };

        let mean_ret = cum_ret.mean().unwrap_or(0.0);

        (median_score, mean_ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::arr2;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_backtest_logic() {
        let config = BacktestConfig {
            trade_size_usd: 100.0,
            min_liquidity_usd: 50.0,
            base_fee_pct: 0.01,
            impact_slippage_max: 0.1,
        };
        let runner = BacktestRunner::new(config);
        
        // Factors: [5.0, -5.0, 5.0]
        // Liquidity: [100.0, 100.0, 100.0]
        // TargetRet: [0.10, -0.05, 0.20]
        let factors = arr2(&[[5.0, -5.0, 5.0]]); 
        let liquidity = arr2(&[[100.0, 100.0, 100.0]]);
        let target_ret = arr2(&[[0.10, -0.05, 0.20]]);

        let (score, ret) = runner.evaluate(&factors, &liquidity, &target_ret);
        
        println!("Score: {}, Ret: {}", score, ret);
        assert_abs_diff_eq!(score, -10.0, epsilon=1e-6);
        assert_abs_diff_eq!(ret, -0.03, epsilon=1e-6);
    }
}
