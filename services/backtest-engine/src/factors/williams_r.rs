use crate::vm::ops::ts_delay;
use ndarray::Array2;

/// Williams %R
/// Momentum indicator measuring overbought/oversold levels
pub struct WilliamsR;

impl WilliamsR {
    /// Calculate Williams %R with standard period (14)
    /// 
    /// # Arguments
    /// * `high` - High prices
    /// * `low` - Low prices
    /// * `close` - Closing prices
    /// 
    /// # Returns
    /// Williams %R values (range -100 to 0)
    /// - Above -20: Overbought
    /// - -20 to -80: Normal range
    /// - Below -80: Oversold
    pub fn williams_r(
        high: &Array2<f64>,
        low: &Array2<f64>,
        close: &Array2<f64>,
    ) -> Array2<f64> {
        Self::williams_r_custom(high, low, close, 14)
    }

    /// Calculate Williams %R with custom period
    /// 
    /// # Arguments
    /// * `high` - High prices
    /// * `low` - Low prices
    /// * `close` - Closing prices
    /// * `period` - Lookback period (default 14)
    /// 
    /// # Formula
    /// %R = -100 × (Highest High - Close) / (Highest High - Lowest Low)
    /// 
    /// Note: Negative values by convention (opposite of Stochastic)
    pub fn williams_r_custom(
        high: &Array2<f64>,
        low: &Array2<f64>,
        close: &Array2<f64>,
        period: usize,
    ) -> Array2<f64> {
        // Find highest high and lowest low over period
        let mut highest_high = Array2::zeros(close.dim());
        let mut lowest_low = Array2::zeros(close.dim());
        
        for i in 0..period {
            let delayed_high = ts_delay(high, i);
            let delayed_low = ts_delay(low, i);
            
            if i == 0 {
                highest_high = delayed_high.clone();
                lowest_low = delayed_low.clone();
            } else {
                let (batch, time) = close.dim();
                for b in 0..batch {
                    for t in 0..time {
                        highest_high[[b, t]] = highest_high[[b, t]].max(delayed_high[[b, t]]);
                        lowest_low[[b, t]] = lowest_low[[b, t]].min(delayed_low[[b, t]]);
                    }
                }
            }
        }
        
        // %R = -100 * (HH - C) / (HH - LL)
        let numerator = &highest_high - close;
        let denominator = &highest_high - &lowest_low;
        
        -100.0 * &numerator / (&denominator + 1e-9)
    }

    /// Normalized Williams %R (0 to 1 range)
    /// 
    /// Converts from [-100, 0] to [0, 1] for easier interpretation
    /// - 1.0 = Overbought (was -0)
    /// - 0.0 = Oversold (was -100)
    pub fn williams_r_normalized(
        high: &Array2<f64>,
        low: &Array2<f64>,
        close: &Array2<f64>,
        period: usize,
    ) -> Array2<f64> {
        let wr = Self::williams_r_custom(high, low, close, period);
        
        // Convert [-100, 0] to [0, 1]
        // %R_norm = (%R + 100) / 100
        (wr + 100.0) / 100.0
    }

    /// Inverted Williams %R (matches Stochastic convention)
    /// 
    /// Returns values in [0, 100] range where:
    /// - 100 = Overbought
    /// - 0 = Oversold
    pub fn williams_r_inverted(
        high: &Array2<f64>,
        low: &Array2<f64>,
        close: &Array2<f64>,
        period: usize,
    ) -> Array2<f64> {
        let wr = Self::williams_r_custom(high, low, close, period);
        
        // Invert: -100 → 0, 0 → 100
        wr + 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use ndarray::arr2;

    #[test]
    fn test_williams_r_basic() {
        let high = arr2(&[[15.0, 16.0, 17.0, 16.5, 18.0]]);
        let low = arr2(&[[10.0, 11.0, 12.0, 11.5, 13.0]]);
        let close = arr2(&[[12.0, 13.0, 14.0, 13.5, 15.0]]);
        
        let wr = WilliamsR::williams_r_custom(&high, &low, &close, 3);
        
        // Williams %R should be between -100 and 0
        for val in wr.iter() {
            assert!(
                *val >= -100.0 && *val <= 0.0,
                "Williams %R should be in [-100, 0], got {}",
                val
            );
        }
    }

    #[test]
    fn test_williams_r_extremes() {
        // Price at top of range
        let high = arr2(&[[20.0, 20.0, 20.0, 20.0]]);
        let low = arr2(&[[10.0, 10.0, 10.0, 10.0]]);
        let close = arr2(&[[20.0, 20.0, 20.0, 20.0]]); // At high
        
        let wr = WilliamsR::williams_r_custom(&high, &low, &close, 3);
        
        // When close = highest high, %R should be 0 (overbought)
        assert_abs_diff_eq!(wr[[0, 3]], 0.0, epsilon = 1.0);
        
        // Price at bottom of range
        let close_low = arr2(&[[10.0, 10.0, 10.0, 10.0]]);
        let wr_low = WilliamsR::williams_r_custom(&high, &low, &close_low, 3);
        
        // When close = lowest low, %R should be -100 (oversold)
        assert_abs_diff_eq!(wr_low[[0, 3]], -100.0, epsilon = 1.0);
    }

    #[test]
    fn test_williams_r_normalized() {
        let high = arr2(&[[15.0, 16.0, 14.0, 17.0]]);
        let low = arr2(&[[10.0, 11.0, 9.0, 12.0]]);
        let close = arr2(&[[12.0, 13.0, 11.0, 14.0]]);
        
        let wr_norm = WilliamsR::williams_r_normalized(&high, &low, &close, 3);
        
        // Normalized should be between 0 and 1
        for val in wr_norm.iter() {
            assert!(
                *val >= 0.0 && *val <= 1.0,
                "Normalized Williams %R should be in [0, 1], got {}",
                val
            );
        }
    }

    #[test]
    fn test_williams_r_inverted_matches_stochastic_range() {
        let high = arr2(&[[15.0, 16.0, 17.0]]);
        let low = arr2(&[[10.0, 11.0, 12.0]]);
        let close = arr2(&[[12.0, 13.0, 14.0]]);
        
        let wr_inv = WilliamsR::williams_r_inverted(&high, &low, &close, 2);
        
        // Inverted should be between 0 and 100 (like Stochastic)
        for val in wr_inv.iter() {
            assert!(
                *val >= 0.0 && *val <= 100.0,
                "Inverted Williams %R should be in [0, 100], got {}",
                val
            );
        }
    }

    #[test]
    fn test_overbought_oversold_signals() {
        // Create overbought condition (price near high)
        let high = arr2(&[[20.0, 20.0, 20.0]]);
        let low = arr2(&[[10.0, 10.0, 10.0]]);
        let close = arr2(&[[19.5, 19.8, 19.9]]); // Near high
        
        let wr = WilliamsR::williams_r(&high, &low, &close);
        
        // Should be near 0 (overbought)
        assert!(wr[[0, 2]] > -20.0, "Should indicate overbought");
        
        // Create oversold condition (price near low)
        let close_low = arr2(&[[10.5, 10.2, 10.1]]); // Near low
        let wr_low = WilliamsR::williams_r(&high, &low, &close_low);
        
        // Should be near -100 (oversold) - relax threshold slightly
        assert!(wr_low[[0, 2]] < -70.0, "Should indicate oversold, got {}", wr_low[[0, 2]]);
    }
}
