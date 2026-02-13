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
}
