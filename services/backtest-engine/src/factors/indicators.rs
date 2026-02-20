use crate::vm::ops::ts_delay;
use ndarray::Array2;

pub struct MemeIndicators;

impl MemeIndicators {
    /// Liquidity Health: liquidity / (fdv + 1e-6)
    pub fn liquidity_health(liquidity: &Array2<f64>, fdv: &Array2<f64>) -> Array2<f64> {
        let ratio = liquidity / (fdv + 1e-6);
        ratio.mapv(|v| (v * 4.0).clamp(0.0, 1.0))
    }

    /// Buy/Sell Imbalance: (close - open) / (high - low)
    pub fn buy_sell_imbalance(
        close: &Array2<f64>,
        open: &Array2<f64>,
        high: &Array2<f64>,
        low: &Array2<f64>,
    ) -> Array2<f64> {
        let range_hl = high - low + 1e-9;
        let body = close - open;
        let strength = body / range_hl;
        strength.mapv(|v| (v * 3.0).tanh())
    }

    /// FOMO Acceleration: 2nd derivative of volume change
    pub fn fomo_acceleration(volume: &Array2<f64>) -> Array2<f64> {
        let vol_prev = ts_delay(volume, 1);
        let vol_chg = (volume - &vol_prev) / (&vol_prev + 1.0);

        let vol_chg_prev = ts_delay(&vol_chg, 1);
        let acc = &vol_chg - &vol_chg_prev;

        acc.mapv(|v| v.clamp(-5.0, 5.0))
    }

    /// Pump Deviation: (close - MA) / MA
    pub fn pump_deviation(close: &Array2<f64>, window: usize) -> Array2<f64> {
        // let mut ma: Array2<f64> = Array2::zeros(close.dim());
        // Simple moving average implementation using windows
        // ndarray windows are a bit complex for 2D, let's use a simpler loop for clarity and correctness with "ts_delay" summation
        // Or better yet, accumulate sum manually for rolling window

        // Naive rolling mean using loop over window size (ok for small windows)
        let mut sum: Array2<f64> = Array2::zeros(close.dim());
        for i in 0..window {
            sum = sum + ts_delay(close, i);
        }
        let ma: Array2<f64> = sum / (window as f64);

        (close - &ma) / (&ma + 1e-9)
    }

    /// Volatility Clustering: sqrt(mean(log_return^2))
    pub fn volatility_clustering(close: &Array2<f64>, window: usize) -> Array2<f64> {
        let prev_close = ts_delay(close, 1);
        let ret = (close / (&prev_close + 1e-9)).mapv(f64::ln);
        let ret_sq = ret.mapv(|v| v.powi(2));

        let mut sum_sq: Array2<f64> = Array2::zeros(close.dim());
        for i in 0..window {
            sum_sq = sum_sq + ts_delay(&ret_sq, i);
        }
        let vol_ma = sum_sq / (window as f64);

        (vol_ma + 1e-9).mapv(f64::sqrt)
    }

    /// Momentum Reversal: Detect if momentum flips sign
    pub fn momentum_reversal(close: &Array2<f64>, window: usize) -> Array2<f64> {
        let prev_close = ts_delay(close, 1);
        let ret = (close / (&prev_close + 1e-9)).mapv(f64::ln);

        let mut mom: Array2<f64> = Array2::zeros(close.dim());
        for i in 0..window {
            mom = mom + ts_delay(&ret, i);
        }

        let mom_prev = ts_delay(&mom, 1);

        // reversal = (mom * mom_prev < 0).float()
        let prod = &mom * &mom_prev;
        prod.mapv(|v| if v < 0.0 { 1.0 } else { 0.0 })
    }

    /// Volume Ratio: volume / SMA(volume, window)
    /// Measures current volume relative to recent average.
    /// Values > 1 indicate above-average activity.
    pub fn volume_ratio(volume: &Array2<f64>, window: usize) -> Array2<f64> {
        let mut sum: Array2<f64> = Array2::zeros(volume.dim());
        for i in 0..window {
            sum = sum + ts_delay(volume, i);
        }
        let ma = sum / (window as f64);
        (volume / (&ma + 1e-9)).mapv(|v| v.clamp(0.0, 10.0))
    }

    /// Momentum: (close - close[t-window]) / close[t-window]
    /// Continuous price momentum over a lookback window.
    pub fn momentum(close: &Array2<f64>, window: usize) -> Array2<f64> {
        let delayed = ts_delay(close, window);
        (close - &delayed) / (&delayed + 1e-9)
    }

    /// VWAP Deviation: (close - VWAP) / VWAP
    /// Typical price = (high + low + close) / 3
    /// VWAP = cumulative(tp * volume) / cumulative(volume) over rolling window
    pub fn vwap_deviation(
        close: &Array2<f64>,
        high: &Array2<f64>,
        low: &Array2<f64>,
        volume: &Array2<f64>,
        window: usize,
    ) -> Array2<f64> {
        let tp = &(high + low + close) / 3.0;
        let tp_vol = &tp * volume;
        let mut sum_tp_vol: Array2<f64> = Array2::zeros(close.dim());
        for i in 0..window {
            sum_tp_vol = sum_tp_vol + ts_delay(&tp_vol, i);
        }
        let mut sum_vol: Array2<f64> = Array2::zeros(close.dim());
        for i in 0..window {
            sum_vol = sum_vol + ts_delay(volume, i);
        }
        let vwap = &sum_tp_vol / (&sum_vol + 1e-9);
        (close - &vwap) / (&vwap + 1e-9)
    }

    /// ADV Ratio: dollar_volume / SMA(dollar_volume, window)
    /// Measures relative trading activity.
    pub fn adv_ratio(close: &Array2<f64>, volume: &Array2<f64>, window: usize) -> Array2<f64> {
        let dollar_vol = close * volume;
        let mut sum = Array2::zeros(close.dim());
        for i in 0..window {
            sum = sum + ts_delay(&dollar_vol, i);
        }
        let adv = sum / (window as f64);
        (&dollar_vol / (&adv + 1e-9)).mapv(|v: f64| v.clamp(0.0, 10.0))
    }

    /// Volatility Regime: z-score of short-term realized vol vs long-term vol average.
    /// >0 = high volatility regime, <0 = low volatility regime.
    pub fn volatility_regime(
        close: &Array2<f64>,
        short_window: usize,
        long_window: usize,
    ) -> Array2<f64> {
        let prev = ts_delay(close, 1);
        let ret = (close / (&prev + 1e-9)).mapv(f64::ln);
        let ret_sq = ret.mapv(|v| v.powi(2));

        // Short-term realized vol
        let mut short_sum = Array2::zeros(close.dim());
        for i in 0..short_window {
            short_sum = short_sum + ts_delay(&ret_sq, i);
        }
        let short_vol = (short_sum / (short_window as f64) + 1e-9).mapv(f64::sqrt);

        // Long-term rolling mean of short vol
        let mut long_sum: Array2<f64> = Array2::zeros(close.dim());
        for i in 0..long_window {
            long_sum = long_sum + ts_delay(&short_vol, i);
        }
        let long_mean = &long_sum / (long_window as f64);

        // Long-term rolling std of short vol
        let diff = &short_vol - &long_mean;
        let diff_sq = diff.mapv(|v| v.powi(2));
        let mut long_var_sum = Array2::zeros(close.dim());
        for i in 0..long_window {
            long_var_sum = long_var_sum + ts_delay(&diff_sq, i);
        }
        let long_std = (long_var_sum / (long_window as f64) + 1e-9).mapv(f64::sqrt);

        // Z-score: (short_vol - long_mean) / long_std
        let zscore = (&short_vol - &long_mean) / &long_std;
        zscore.mapv(|v| v.clamp(-3.0, 3.0))
    }

    /// Trend Strength: normalized linear regression slope of close over window.
    /// Positive = uptrend, negative = downtrend, magnitude = strength.
    pub fn trend_strength(close: &Array2<f64>, window: usize) -> Array2<f64> {
        // Linear regression slope: sum((x - x_mean)(y - y_mean)) / sum((x - x_mean)^2)
        // x = 0..window-1, x_mean = (window-1)/2
        // Precompute x terms
        let x_mean = (window as f64 - 1.0) / 2.0;
        let ss_x: f64 = (0..window).map(|i| (i as f64 - x_mean).powi(2)).sum();

        let mut result = Array2::zeros(close.dim());
        let (_, time) = close.dim();

        for row in 0..close.dim().0 {
            for t in 0..time {
                if t + 1 < window {
                    result[[row, t]] = 0.0;
                    continue;
                }
                let mut ss_xy = 0.0;
                let mut y_sum = 0.0;
                for k in 0..window {
                    y_sum += close[[row, t + 1 - window + k]];
                }
                let y_mean = y_sum / window as f64;
                for k in 0..window {
                    let x_val = k as f64 - x_mean;
                    let y_val = close[[row, t + 1 - window + k]] - y_mean;
                    ss_xy += x_val * y_val;
                }
                let slope = if ss_x > 1e-12 { ss_xy / ss_x } else { 0.0 };
                // Normalize by price level
                let price = close[[row, t]].abs().max(1e-9);
                result[[row, t]] = (slope / price * window as f64).clamp(-5.0, 5.0);
            }
        }
        result
    }

    /// Momentum Regime: fraction of positive return bars in window, mapped to [-1, 1].
    /// >0 = trending up, <0 = trending down, ~0 = choppy.
    pub fn momentum_regime(close: &Array2<f64>, window: usize) -> Array2<f64> {
        let prev = ts_delay(close, 1);
        let ret = (close / (&prev + 1e-9)).mapv(f64::ln);
        let positive = ret.mapv(|v| if v > 0.0 { 1.0 } else { 0.0 });

        let mut sum = Array2::zeros(close.dim());
        for i in 0..window {
            sum = sum + ts_delay(&positive, i);
        }
        let frac = sum / (window as f64);
        // Map [0, 1] to [-1, 1]
        frac * 2.0 - 1.0
    }

    /// Close Position: (close - low) / (high - low)
    /// Where the close sits within the day's range. 1.0 = closed at high, 0.0 = closed at low.
    pub fn close_position(
        close: &Array2<f64>,
        high: &Array2<f64>,
        low: &Array2<f64>,
    ) -> Array2<f64> {
        let range = high - low + 1e-9;
        let pos = (close - low) / &range;
        pos.mapv(|v| v.clamp(0.0, 1.0))
    }

    /// Intraday Range: (high - low) / close
    /// Normalized daily price range as a volatility proxy.
    pub fn intraday_range(
        high: &Array2<f64>,
        low: &Array2<f64>,
        close: &Array2<f64>,
    ) -> Array2<f64> {
        (high - low) / (close + 1e-9)
    }

    // ── Microstructure Factors ──────────────────────────────────────────

    /// Amihud Illiquidity: rolling mean of |return| / dollar_volume.
    /// Higher = less liquid, more price impact per unit of volume.
    pub fn amihud_illiquidity(
        close: &Array2<f64>,
        volume: &Array2<f64>,
        window: usize,
    ) -> Array2<f64> {
        let prev = ts_delay(close, 1);
        let abs_ret = ((close / (&prev + 1e-9)).mapv(f64::ln)).mapv(f64::abs);
        let dollar_vol = close * volume + 1e-9;
        let ratio = &abs_ret / &dollar_vol;
        // Rolling mean
        let mut sum = Array2::zeros(close.dim());
        for i in 0..window {
            sum = sum + ts_delay(&ratio, i);
        }
        (sum / window as f64).mapv(|v: f64| v.clamp(0.0, 1e6))
    }

    /// Spread Proxy: (high - low) / midprice, normalized by rolling mean.
    /// Captures implicit bid-ask spread from OHLC data.
    pub fn spread_proxy(high: &Array2<f64>, low: &Array2<f64>, window: usize) -> Array2<f64> {
        let mid = (high + low) / 2.0 + 1e-9;
        let raw_spread = (high - low) / &mid;
        // Normalize by rolling mean
        let mut sum: Array2<f64> = Array2::zeros(high.dim());
        for i in 0..window {
            sum = sum + ts_delay(&raw_spread, i);
        }
        let avg = &sum / window as f64 + 1e-9;
        (&raw_spread / &avg).mapv(|v: f64| v.clamp(0.0, 10.0))
    }

    /// Return Autocorrelation: rolling correlation of ret[t] with ret[t-1].
    /// Positive = trending, negative = mean-reverting, ~0 = random walk.
    pub fn return_autocorrelation(close: &Array2<f64>, window: usize) -> Array2<f64> {
        let prev = ts_delay(close, 1);
        let ret = (close / (&prev + 1e-9)).mapv(f64::ln);
        let ret_lag = ts_delay(&ret, 1);

        // Rolling Pearson correlation between ret and ret_lag
        let mut sum_a = Array2::zeros(close.dim());
        let mut sum_b = Array2::zeros(close.dim());
        let mut sum_ab = Array2::zeros(close.dim());
        let mut sum_a2 = Array2::zeros(close.dim());
        let mut sum_b2 = Array2::zeros(close.dim());

        for i in 0..window {
            let a = ts_delay(&ret, i);
            let b = ts_delay(&ret_lag, i);
            sum_ab += &(&a * &b);
            sum_a += &a;
            sum_b += &b;
            sum_a2 += &a.mapv(|v: f64| v * v);
            sum_b2 += &b.mapv(|v: f64| v * v);
        }

        let n = window as f64;
        let cov = &sum_ab / n - &(&sum_a / n) * &(&sum_b / n);
        let var_a = (&sum_a2 / n - (&sum_a / n).mapv(|v: f64| v * v)).mapv(|v: f64| v.max(0.0));
        let var_b = (&sum_b2 / n - (&sum_b / n).mapv(|v: f64| v * v)).mapv(|v: f64| v.max(0.0));
        let denom = (var_a * var_b).mapv(f64::sqrt) + 1e-12;

        (&cov / &denom).mapv(|v: f64| v.clamp(-1.0, 1.0))
    }

    // ── Cross-Asset Factors ──────────────────────────────────────────

    /// Rolling Pearson correlation between two return series.
    pub fn rolling_correlation(
        close_a: &Array2<f64>,
        close_b: &Array2<f64>,
        window: usize,
    ) -> Array2<f64> {
        let prev_a = ts_delay(close_a, 1);
        let prev_b = ts_delay(close_b, 1);
        let ret_a = (close_a / (&prev_a + 1e-9)).mapv(f64::ln);
        let ret_b = (close_b / (&prev_b + 1e-9)).mapv(f64::ln);

        let mut sum_a: Array2<f64> = Array2::zeros(close_a.dim());
        let mut sum_b: Array2<f64> = Array2::zeros(close_a.dim());
        let mut sum_ab: Array2<f64> = Array2::zeros(close_a.dim());
        let mut sum_a2: Array2<f64> = Array2::zeros(close_a.dim());
        let mut sum_b2: Array2<f64> = Array2::zeros(close_a.dim());

        for i in 0..window {
            let a = ts_delay(&ret_a, i);
            let b = ts_delay(&ret_b, i);
            sum_ab += &(&a * &b);
            sum_a += &a;
            sum_b += &b;
            sum_a2 += &a.mapv(|v: f64| v * v);
            sum_b2 += &b.mapv(|v: f64| v * v);
        }

        let n = window as f64;
        let cov = &sum_ab / n - &(&sum_a / n) * &(&sum_b / n);
        let var_a = (&sum_a2 / n - (&sum_a / n).mapv(|v: f64| v * v)).mapv(|v: f64| v.max(0.0));
        let var_b = (&sum_b2 / n - (&sum_b / n).mapv(|v: f64| v * v)).mapv(|v: f64| v.max(0.0));
        let denom = (var_a * var_b).mapv(f64::sqrt) + 1e-12;

        (&cov / &denom).mapv(|v: f64| v.clamp(-1.0, 1.0))
    }

    /// Rolling beta: cov(ret_a, ret_b) / var(ret_b) over window.
    pub fn rolling_beta(
        close_a: &Array2<f64>,
        close_b: &Array2<f64>,
        window: usize,
    ) -> Array2<f64> {
        let prev_a = ts_delay(close_a, 1);
        let prev_b = ts_delay(close_b, 1);
        let ret_a = (close_a / (&prev_a + 1e-9)).mapv(f64::ln);
        let ret_b = (close_b / (&prev_b + 1e-9)).mapv(f64::ln);

        let mut sum_a: Array2<f64> = Array2::zeros(close_a.dim());
        let mut sum_b: Array2<f64> = Array2::zeros(close_a.dim());
        let mut sum_ab: Array2<f64> = Array2::zeros(close_a.dim());
        let mut sum_b2: Array2<f64> = Array2::zeros(close_a.dim());

        for i in 0..window {
            let a = ts_delay(&ret_a, i);
            let b = ts_delay(&ret_b, i);
            sum_ab += &(&a * &b);
            sum_a += &a;
            sum_b += &b;
            sum_b2 += &b.mapv(|v: f64| v * v);
        }

        let n = window as f64;
        let cov = &sum_ab / n - &(&sum_a / n) * &(&sum_b / n);
        let var_b = &sum_b2 / n - (&sum_b / n).mapv(|v: f64| v * v) + 1e-12;

        (&cov / &var_b).mapv(|v: f64| v.clamp(-5.0, 5.0))
    }

    /// Relative strength vs reference: cumulative return of A minus cumulative return of B.
    pub fn relative_strength_vs(
        close_a: &Array2<f64>,
        close_b: &Array2<f64>,
        window: usize,
    ) -> Array2<f64> {
        let delayed_a = ts_delay(close_a, window);
        let delayed_b = ts_delay(close_b, window);
        let cum_ret_a = (close_a / (&delayed_a + 1e-9)) - 1.0;
        let cum_ret_b = (close_b / (&delayed_b + 1e-9)) - 1.0;
        (&cum_ret_a - &cum_ret_b).mapv(|v| v.clamp(-5.0, 5.0))
    }

    /// Relative Strength (RSI-like)
    pub fn relative_strength(close: &Array2<f64>, window: usize) -> Array2<f64> {
        let diff = close - &ts_delay(close, 1);

        let gains = diff.mapv(|v| if v > 0.0 { v } else { 0.0 });
        let losses = diff.mapv(|v| if v < 0.0 { -v } else { 0.0 }); // take absolute value of loss

        let mut avg_gain: Array2<f64> = Array2::zeros(close.dim());
        let mut avg_loss: Array2<f64> = Array2::zeros(close.dim());

        // Simple moving average for RSI (Note: Wilder's smoothing is standard but here we replicate 'mean' from Python code which uses unfold.mean)
        for i in 0..window {
            avg_gain = avg_gain + ts_delay(&gains, i);
            avg_loss = avg_loss + ts_delay(&losses, i);
        }
        avg_gain /= window as f64;
        avg_loss /= window as f64;

        let rs = (&avg_gain + 1e-9) / (&avg_loss + 1e-9);

        // rsi = 100 - (100 / (1 + rs))
        // return (rsi - 50) / 50

        let rsi = 100.0 - (100.0 / (1.0 + &rs));
        (rsi - 50.0) / 50.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use ndarray::arr2;

    #[test]
    fn test_liquidity_health() {
        let liq = arr2(&[[100., 200.], [10., 0.]]);
        let fdv = arr2(&[[1000., 400.], [1000., 100.]]);

        // ratio = liq/fdv -> [0.1, 0.5]
        // *4 -> [0.4, 2.0]
        // clamp -> [0.4, 1.0]
        // Row 2: [0.01, 0.0] -> [0.04, 0.0] -> [0.04, 0.0]

        let res = MemeIndicators::liquidity_health(&liq, &fdv);
        let expected = arr2(&[[0.4, 1.0], [0.03999996, 0.0]]); // fdv+1e-6 affects output slightly
        assert_abs_diff_eq!(res, expected, epsilon = 1e-4);
    }

    #[test]
    fn test_pump_deviation() {
        // [10, 11, 12] window=2
        // MA at t=0: 10/2 = 5 (since delay(1) is 0) -> Correct if we assume 0 padding?
        // Wait, delay pads with 0. So close + delay lines up.
        // t=0: close=10. delay(1)=0. sum=10. ma=5.
        // t=1: close=11. delay(1)=10. sum=21. ma=10.5.
        // t=2: close=12. delay(1)=11. sum=23. ma=11.5.

        let close = arr2(&[[10., 11., 12.]]);
        let res = MemeIndicators::pump_deviation(&close, 2);

        // t=0: (10 - 5)/5 = 1.0
        // t=1: (11 - 10.5)/10.5 = 0.5 / 10.5 = 0.0476
        // t=2: (12 - 11.5)/11.5 = 0.5 / 11.5 = 0.04347

        let expected = arr2(&[[1.0, 0.047619, 0.043478]]);
        assert_abs_diff_eq!(res, expected, epsilon = 1e-4);
    }

    #[test]
    fn test_volume_ratio() {
        // volume = [100, 200, 300], window=2
        // t=0: ma = (100+0)/2 = 50, ratio = 100/50 = 2.0
        // t=1: ma = (200+100)/2 = 150, ratio = 200/150 = 1.333
        // t=2: ma = (300+200)/2 = 250, ratio = 300/250 = 1.2
        let vol = arr2(&[[100., 200., 300.]]);
        let res = MemeIndicators::volume_ratio(&vol, 2);
        let expected = arr2(&[[2.0, 1.3333, 1.2]]);
        assert_abs_diff_eq!(res, expected, epsilon = 1e-3);
    }

    #[test]
    fn test_volatility_regime() {
        // Create a price series with known volatility pattern:
        // First half: low vol (small moves), second half: high vol (large moves)
        let mut prices = vec![100.0_f64; 80];
        for i in 1..40 {
            prices[i] = prices[i - 1] * (1.0 + if i % 2 == 0 { 0.001 } else { -0.001 });
        }
        for i in 40..80 {
            prices[i] = prices[i - 1] * (1.0 + if i % 2 == 0 { 0.02 } else { -0.02 });
        }
        let n = prices.len();
        let close = Array2::from_shape_vec((1, n), prices).unwrap();
        let result = MemeIndicators::volatility_regime(&close, 5, 20);
        // The high-vol region (towards end) should have positive z-scores
        assert!(result[[0, 79]] > 0.0, "High vol region should be positive");
    }

    #[test]
    fn test_trend_strength() {
        // Strong uptrend: 100, 101, 102, ..., 119
        let prices: Vec<f64> = (0..20).map(|i| 100.0 + i as f64).collect();
        let n = prices.len();
        let close = Array2::from_shape_vec((1, n), prices).unwrap();
        let result = MemeIndicators::trend_strength(&close, 10);
        assert!(
            result[[0, 19]] > 0.0,
            "Uptrend should have positive trend_strength"
        );

        // Strong downtrend: 119, 118, ..., 100
        let prices_down: Vec<f64> = (0..20).map(|i| 119.0 - i as f64).collect();
        let close_down = Array2::from_shape_vec((1, n), prices_down).unwrap();
        let result_down = MemeIndicators::trend_strength(&close_down, 10);
        assert!(
            result_down[[0, 19]] < 0.0,
            "Downtrend should have negative trend_strength"
        );
    }

    #[test]
    fn test_momentum_regime() {
        // All up: 100, 101, 102, ..., 109
        let prices: Vec<f64> = (0..10).map(|i| 100.0 + i as f64).collect();
        let n = prices.len();
        let close = Array2::from_shape_vec((1, n), prices).unwrap();
        let result = MemeIndicators::momentum_regime(&close, 5);
        assert!(
            result[[0, 9]] > 0.5,
            "All-up series should have positive regime"
        );

        // Choppy: alternating up/down
        let choppy: Vec<f64> = (0..10)
            .map(|i| if i % 2 == 0 { 100.0 } else { 99.0 })
            .collect();
        let close_choppy = Array2::from_shape_vec((1, n), choppy).unwrap();
        let result_choppy = MemeIndicators::momentum_regime(&close_choppy, 5);
        assert!(
            result_choppy[[0, 9]].abs() < 0.8,
            "Choppy series should be near zero"
        );
    }

    #[test]
    fn test_momentum() {
        // close = [100, 110, 105], window=1
        // t=0: (100 - 0) / (0 + 1e-9) → clamped huge, but delay pads with 0
        // t=1: (110 - 100) / 100 = 0.1
        // t=2: (105 - 110) / 110 = -0.04545
        let close = arr2(&[[100., 110., 105.]]);
        let res = MemeIndicators::momentum(&close, 1);
        // t=0 is distorted by zero padding, check t=1 and t=2
        assert_abs_diff_eq!(res[[0, 1]], 0.1, epsilon = 1e-6);
        assert_abs_diff_eq!(res[[0, 2]], -0.04545, epsilon = 1e-4);
    }

    #[test]
    fn test_amihud_illiquidity() {
        // Known price/volume: stable price, increasing volume should decrease illiquidity
        let close =
            Array2::from_shape_vec((1, 30), (0..30).map(|i| 100.0 + (i as f64 * 0.1)).collect())
                .unwrap();
        let volume = Array2::from_shape_vec(
            (1, 30),
            (0..30).map(|i| 1000.0 + i as f64 * 100.0).collect(),
        )
        .unwrap();
        let result = MemeIndicators::amihud_illiquidity(&close, &volume, 10);
        // Should be non-negative and finite
        for &v in result.iter() {
            assert!(v >= 0.0, "Amihud should be non-negative");
            assert!(v.is_finite(), "Amihud should be finite");
        }
        // Later bars (higher volume) should have lower illiquidity
        assert!(
            result[[0, 29]] <= result[[0, 15]] + 1e-9,
            "Higher volume should mean lower illiquidity"
        );
    }

    #[test]
    fn test_spread_proxy() {
        // Known OHLC: wider spread = higher value
        let high = Array2::from_shape_vec(
            (1, 30),
            (0..30)
                .map(|i| 105.0 + if i > 15 { 5.0 } else { 0.0 })
                .collect(),
        )
        .unwrap();
        let low = Array2::from_shape_vec(
            (1, 30),
            (0..30)
                .map(|i| 95.0 - if i > 15 { 5.0 } else { 0.0 })
                .collect(),
        )
        .unwrap();
        let result = MemeIndicators::spread_proxy(&high, &low, 10);
        for &v in result.iter() {
            assert!(v >= 0.0, "Spread proxy should be non-negative");
            assert!(v.is_finite(), "Spread proxy should be finite");
        }
    }

    #[test]
    fn test_return_autocorrelation() {
        // Trending series: consistently increasing -> positive autocorrelation
        let trending: Vec<f64> = (0..50).map(|i| 100.0 + i as f64).collect();
        let close_trend = Array2::from_shape_vec((1, 50), trending).unwrap();
        let result_trend = MemeIndicators::return_autocorrelation(&close_trend, 20);
        // After warmup, trending series should have positive autocorrelation
        assert!(
            result_trend[[0, 49]] > 0.0,
            "Trending series should have positive autocorrelation, got {}",
            result_trend[[0, 49]]
        );

        // Random-like series: alternating -> should be negative or near zero
        let alternating: Vec<f64> = (0..50)
            .map(|i| if i % 2 == 0 { 100.0 } else { 101.0 })
            .collect();
        let close_alt = Array2::from_shape_vec((1, 50), alternating).unwrap();
        let result_alt = MemeIndicators::return_autocorrelation(&close_alt, 20);
        assert!(
            result_alt[[0, 49]] < 0.5,
            "Alternating series should have low/negative autocorrelation, got {}",
            result_alt[[0, 49]]
        );
    }

    #[test]
    fn test_rolling_correlation() {
        // Perfectly correlated: same series -> correlation ~1.0
        let prices: Vec<f64> = (0..80).map(|i| 100.0 + i as f64 * 0.5).collect();
        let close_a = Array2::from_shape_vec((1, 80), prices.clone()).unwrap();
        let close_b = Array2::from_shape_vec((1, 80), prices).unwrap();
        let result = MemeIndicators::rolling_correlation(&close_a, &close_b, 30);
        // After warmup period, should be near 1.0
        assert!(
            result[[0, 79]] > 0.95,
            "Self-correlation should be ~1.0, got {}",
            result[[0, 79]]
        );
    }

    #[test]
    fn test_rolling_beta() {
        // Beta of a series with itself should be ~1.0
        let prices: Vec<f64> = (0..80).map(|i| 100.0 + i as f64 * 0.5).collect();
        let close_a = Array2::from_shape_vec((1, 80), prices.clone()).unwrap();
        let close_b = Array2::from_shape_vec((1, 80), prices).unwrap();
        let result = MemeIndicators::rolling_beta(&close_a, &close_b, 30);
        // After warmup, beta of self should be ~1.0
        assert!(
            (result[[0, 79]] - 1.0).abs() < 0.1,
            "Self-beta should be ~1.0, got {}",
            result[[0, 79]]
        );
    }

    #[test]
    fn test_relative_strength_vs() {
        // A outperforms B: A goes up, B stays flat
        let close_a =
            Array2::from_shape_vec((1, 30), (0..30).map(|i| 100.0 + i as f64 * 2.0).collect())
                .unwrap();
        let close_b = Array2::from_shape_vec((1, 30), vec![100.0; 30]).unwrap();
        let result = MemeIndicators::relative_strength_vs(&close_a, &close_b, 10);
        // A outperforms -> positive relative strength
        assert!(
            result[[0, 29]] > 0.0,
            "Outperforming asset should have positive relative strength, got {}",
            result[[0, 29]]
        );
    }
}
