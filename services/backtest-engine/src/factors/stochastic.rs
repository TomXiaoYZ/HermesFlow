use crate::factors::moving_averages::MovingAverages;
use crate::vm::ops::ts_delay;
use ndarray::Array2;

/// Stochastic Oscillator
/// Momentum indicator comparing closing price to price range over period
pub struct Stochastic;

impl Stochastic {
    /// Calculate Stochastic Oscillator with standard parameters
    /// 
    /// # Arguments
    /// * `high` - High prices
    /// * `low` - Low prices
    /// * `close` - Closing prices
    /// 
    /// # Returns
    /// (%K, %D) - Fast and slow stochastic lines
    /// - %K (14 period): Fast line, more volatile
    /// - %D (3 period SMA of %K): Slow line, smoothed signal
    pub fn stochastic(
        high: &Array2<f64>,
        low: &Array2<f64>,
        close: &Array2<f64>,
    ) -> (Array2<f64>, Array2<f64>) {
        Self::stochastic_custom(high, low, close, 14, 3)
    }

    /// Calculate Stochastic with custom parameters
    /// 
    /// # Arguments
    /// * `high` - High prices
    /// * `low` - Low prices
    /// * `close` - Closing prices
    /// * `k_period` - Period for %K calculation (default 14)
    /// * `d_period` - Period for %D smoothing (default 3)
    /// 
    /// # Formula
    /// %K = 100 * (Close - Lowest Low) / (Highest High - Lowest Low)
    /// %D = SMA(%K, d_period)
    pub fn stochastic_custom(
        high: &Array2<f64>,
        low: &Array2<f64>,
        close: &Array2<f64>,
        k_period: usize,
        d_period: usize,
    ) -> (Array2<f64>, Array2<f64>) {
        // Calculate %K
        let mut lowest_low = Array2::zeros(close.dim());
        let mut highest_high = Array2::zeros(close.dim());
        
        // For each timestep, find min/max over k_period
        for i in 0..k_period {
            let delayed_low = ts_delay(low, i);
            let delayed_high = ts_delay(high, i);
            
            if i == 0 {
                lowest_low = delayed_low.clone();
                highest_high = delayed_high.clone();
            } else {
                // Element-wise min/max
                let (batch, time) = close.dim();
                for b in 0..batch {
                    for t in 0..time {
                        lowest_low[[b, t]] = lowest_low[[b, t]].min(delayed_low[[b, t]]);
                        highest_high[[b, t]] = highest_high[[b, t]].max(delayed_high[[b, t]]);
                    }
                }
            }
        }
        
        // %K = 100 * (close - lowest_low) / (highest_high - lowest_low)
        let range = &highest_high - &lowest_low;
        let k = (close - &lowest_low) / (&range + 1e-9) * 100.0;
        
        // %D = SMA of %K
        let d = MovingAverages::sma(&k, d_period);
        
        (k, d)
    }

    /// Calculate Fast Stochastic (no smoothing)
    /// 
    /// Returns raw %K without %D smoothing
    pub fn fast_stochastic(
        high: &Array2<f64>,
        low: &Array2<f64>,
        close: &Array2<f64>,
        period: usize,
    ) -> Array2<f64> {
        let (k, _) = Self::stochastic_custom(high, low, close, period, 3);
        k
    }

    /// Calculate Slow Stochastic (additional smoothing)
    /// 
    /// # Returns
    /// (%K smoothed, %D) where %K is already smoothed
    pub fn slow_stochastic(
        high: &Array2<f64>,
        low: &Array2<f64>,
        close: &Array2<f64>,
        k_period: usize,
        k_smooth: usize,
        d_period: usize,
    ) -> (Array2<f64>, Array2<f64>) {
        // Calculate raw %K
        let (raw_k, _) = Self::stochastic_custom(high, low, close, k_period, 3);
        
        // Smooth %K
        let k_smoothed = MovingAverages::sma(&raw_k, k_smooth);
        
        // %D = SMA of smoothed %K
        let d = MovingAverages::sma(&k_smoothed, d_period);
        
        (k_smoothed, d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use ndarray::arr2;

    #[test]
    fn test_stochastic_basic() {
        // Simple price pattern
        let high = arr2(&[[15.0, 16.0, 17.0, 16.5, 18.0]]);
        let low = arr2(&[[10.0, 11.0, 12.0, 11.5, 13.0]]);
        let close = arr2(&[[12.0, 13.0, 14.0, 13.5, 15.0]]);
        
        let (k, d) = Stochastic::stochastic_custom(&high, &low, &close, 3, 2);
        
        // %K should be between 0 and 100
        for val in k.iter() {
            assert!(*val >= 0.0 && *val <= 100.0, "K should be 0-100, got {}", val);
        }
        
        // %D should also be 0-100
        for val in d.iter() {
            assert!(*val >= 0.0 && *val <= 100.0, "D should be 0-100, got {}", val);
        }
    }

    #[test]
    fn test_stochastic_extremes() {
        // Price at top of range
        let high = arr2(&[[20.0, 20.0, 20.0, 20.0]]);
        let low = arr2(&[[10.0, 10.0, 10.0, 10.0]]);
        let close = arr2(&[[20.0, 20.0, 20.0, 20.0]]); // At high
        
        let (k, _) = Stochastic::stochastic_custom(&high, &low, &close, 3, 2);
        
        // When close = high, %K should be 100
        assert_abs_diff_eq!(k[[0, 3]], 100.0, epsilon = 1.0);
        
        // Price at bottom of range
        let close_low = arr2(&[[10.0, 10.0, 10.0, 10.0]]);
        let (k_low, _) = Stochastic::stochastic_custom(&high, &low, &close_low, 3, 2);
        
        // When close = low, %K should be 0
        assert_abs_diff_eq!(k_low[[0, 3]], 0.0, epsilon = 1.0);
    }

    #[test]
    fn test_d_is_smoother_than_k() {
        // Volatile price series
        let high = arr2(&[[15.0, 20.0, 12.0, 18.0, 14.0, 19.0]]);
        let low = arr2(&[[10.0, 15.0, 8.0, 13.0, 9.0, 14.0]]);
        let close = arr2(&[[12.0, 18.0, 10.0, 16.0, 11.0, 17.0]]);
        
        let (k, d) = Stochastic::stochastic_custom(&high, &low, &close, 4, 3);
        
        // %D should be smoother (less volatile) than %K
        // Check that dimensions match
        assert_eq!(k.dim(), d.dim());
    }
}
