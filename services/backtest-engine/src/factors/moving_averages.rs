use crate::vm::ops::ts_delay;
use ndarray::Array2;

/// Moving Average calculations
pub struct MovingAverages;

impl MovingAverages {
    /// Simple Moving Average
    ///
    /// # Arguments
    /// * `data` - Input time series (batch_size, time_steps)
    /// * `window` - Number of periods for averaging
    ///
    /// # Returns
    /// SMA values with same shape as input
    pub fn sma(data: &Array2<f64>, window: usize) -> Array2<f64> {
        let mut sum = Array2::zeros(data.dim());

        // Sum over window using ts_delay
        for i in 0..window {
            sum = sum + ts_delay(data, i);
        }

        sum / (window as f64)
    }

    /// Exponential Moving Average
    ///
    /// # Arguments
    /// * `data` - Input time series
    /// * `window` - Number of periods (determines alpha)
    ///
    /// # Formula
    /// alpha = 2 / (window + 1)
    /// EMA[t] = alpha * price[t] + (1 - alpha) * EMA[t-1]
    ///
    /// # Returns
    /// EMA values with same shape as input
    pub fn ema(data: &Array2<f64>, window: usize) -> Array2<f64> {
        let alpha = 2.0 / (window as f64 + 1.0);
        let (batch_size, time_steps) = data.dim();

        let mut ema = Array2::zeros((batch_size, time_steps));

        // Initialize with first value (or could use SMA as seed)
        for b in 0..batch_size {
            ema[[b, 0]] = data[[b, 0]];

            // Iteratively compute EMA
            for t in 1..time_steps {
                ema[[b, t]] = alpha * data[[b, t]] + (1.0 - alpha) * ema[[b, t - 1]];
            }
        }

        ema
    }

    /// Weighted Moving Average
    ///
    /// # Arguments
    /// * `data` - Input time series
    /// * `window` - Number of periods
    ///
    /// # Formula
    /// WMA = (n*P1 + (n-1)*P2 + ... + 1*Pn) / (n + (n-1) + ... + 1)
    /// where n = window, P1 = most recent price
    ///
    /// # Returns
    /// WMA values with same shape as input
    pub fn wma(data: &Array2<f64>, window: usize) -> Array2<f64> {
        let mut weighted_sum: Array2<f64> = Array2::zeros(data.dim());

        // Weight sum: 1 + 2 + ... + n = n*(n+1)/2
        let weight_divisor = (window * (window + 1)) as f64 / 2.0;

        // Most recent gets highest weight
        for i in 0..window {
            // i=0 (most recent) gets weight=window, i=1 gets window-1, etc
            let weight = (window - i) as f64;
            weighted_sum = weighted_sum + ts_delay(data, i) * weight;
        }

        weighted_sum / weight_divisor
    }

    /// Double Exponential Moving Average (DEMA)
    ///
    /// # Arguments
    /// * `data` - Input time series
    /// * `window` - Number of periods
    ///
    /// # Formula
    /// DEMA = 2 * EMA - EMA(EMA)
    ///
    /// # Returns
    /// DEMA values (less lag than regular EMA)
    pub fn dema(data: &Array2<f64>, window: usize) -> Array2<f64> {
        let ema1 = Self::ema(data, window);
        let ema2 = Self::ema(&ema1, window);

        2.0 * &ema1 - &ema2
    }

    /// Triple Exponential Moving Average (TEMA)
    ///
    /// # Arguments
    /// * `data` - Input time series
    /// * `window` - Number of periods
    ///
    /// # Formula
    /// TEMA = 3*EMA1 - 3*EMA2 + EMA3
    /// where EMA2 = EMA(EMA1), EMA3 = EMA(EMA2)
    ///
    /// # Returns
    /// TEMA values (even less lag)
    pub fn tema(data: &Array2<f64>, window: usize) -> Array2<f64> {
        let ema1 = Self::ema(data, window);
        let ema2 = Self::ema(&ema1, window);
        let ema3 = Self::ema(&ema2, window);

        3.0 * &ema1 - 3.0 * &ema2 + &ema3
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use ndarray::arr2;

    #[test]
    fn test_sma_basic() {
        // Test data: [10, 20, 30, 40]
        let data = arr2(&[[10.0, 20.0, 30.0, 40.0]]);
        let result = MovingAverages::sma(&data, 2);

        // t=0: (0 + 10) / 2 = 5
        // t=1: (10 + 20) / 2 = 15
        // t=2: (20 + 30) / 2 = 25
        // t=3: (30 + 40) / 2 = 35
        let expected = arr2(&[[5.0, 15.0, 25.0, 35.0]]);
        assert_abs_diff_eq!(result, expected, epsilon = 1e-6);
    }

    #[test]
    fn test_ema_convergence() {
        // Test EMA with constant value (should converge to value)
        let data = arr2(&[[10.0, 10.0, 10.0, 10.0, 10.0]]);
        let result = MovingAverages::ema(&data, 3);

        // With constant input, EMA should converge to that value
        // Last value should be very close to 10
        assert!((result[[0, 4]] - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_wma_weights() {
        // Test WMA with simple sequence
        let data = arr2(&[[1.0, 2.0, 3.0]]);
        let result = MovingAverages::wma(&data, 2);

        // t=0: (2*0 + 1*1) / 3 = 1/3
        // t=1: (2*1 + 1*2) / 3 = 4/3
        // t=2: (2*2 + 1*3) / 3 = 7/3
        let expected = arr2(&[[1.0 / 3.0, 4.0 / 3.0, 7.0 / 3.0]]);
        assert_abs_diff_eq!(result, expected, epsilon = 1e-6);
    }

    #[test]
    fn test_dema_less_lag() {
        // DEMA should respond faster than regular EMA
        let data = arr2(&[[1.0, 1.0, 1.0, 10.0, 10.0, 10.0]]);

        let ema = MovingAverages::ema(&data, 3);
        let dema = MovingAverages::dema(&data, 3);

        // At t=4, DEMA should be closer to 10 than EMA
        assert!(dema[[0, 4]] > ema[[0, 4]]);
    }
}
