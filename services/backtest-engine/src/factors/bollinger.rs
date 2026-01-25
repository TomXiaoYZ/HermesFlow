use crate::factors::moving_averages::MovingAverages;
use crate::vm::ops::ts_delay;
use ndarray::Array2;

/// Bollinger Bands indicator
pub struct BollingerBands;

impl BollingerBands {
    /// Calculate Bollinger Bands with standard parameters
    /// 
    /// # Arguments
    /// * `close` - Closing prices (batch_size, time_steps)
    /// 
    /// # Returns
    /// (upper_band, middle_band, lower_band)
    /// - middle_band: SMA(20)
    /// - upper_band: middle + 2 * std_dev
    /// - lower_band: middle - 2 * std_dev
    pub fn bollinger(close: &Array2<f64>) -> (Array2<f64>, Array2<f64>, Array2<f64>) {
        Self::bollinger_custom(close, 20, 2.0)
    }

    /// Calculate Bollinger Bands with custom parameters
    /// 
    /// # Arguments
    /// * `close` - Closing prices
    /// * `window` - Period for SMA and std dev calculation
    /// * `num_std` - Number of standard deviations for bands
    pub fn bollinger_custom(
        close: &Array2<f64>,
        window: usize,
        num_std: f64,
    ) -> (Array2<f64>, Array2<f64>, Array2<f64>) {
        // Middle band = SMA
        let middle_band = MovingAverages::sma(close, window);
        
        // Calculate rolling standard deviation
        let std_dev = Self::rolling_std(close, window);
        
        // Upper and lower bands
        let upper_band = &middle_band + &std_dev * num_std;
        let lower_band = &middle_band - &std_dev * num_std;
        
        (upper_band, middle_band, lower_band)
    }

    /// Calculate Bollinger Bandwidth
    /// 
    /// # Arguments
    /// * `close` - Closing prices
    /// * `window` - Period for calculation
    /// 
    /// # Returns
    /// Bandwidth = (upper - lower) / middle
    /// Measures volatility - wider bands = higher volatility
    pub fn bandwidth(close: &Array2<f64>, window: usize) -> Array2<f64> {
        let (upper, middle, lower) = Self::bollinger_custom(close, window, 2.0);
        
        (&upper - &lower) / (&middle + 1e-9)
    }

    /// Calculate %B (Percent B)
    /// 
    /// # Arguments
    /// * `close` - Closing prices
    /// * `window` - Period for calculation
    /// 
    /// # Returns
    /// %B = (close - lower) / (upper - lower)
    /// Shows where price is relative to bands
    /// - Above 1.0: price above upper band
    /// - 0.5: price at middle
    /// - Below 0.0: price below lower band
    pub fn percent_b(close: &Array2<f64>, window: usize) -> Array2<f64> {
        let (upper, _, lower) = Self::bollinger_custom(close, window, 2.0);
        
        let range = &upper - &lower;
        (close - &lower) / (&range + 1e-9)
    }

    /// Helper: Calculate rolling standard deviation
    fn rolling_std(data: &Array2<f64>, window: usize) -> Array2<f64> {
        // Calculate rolling mean
        let mean = MovingAverages::sma(data, window);
        
        // Calculate squared deviations
        let mut var_sum: Array2<f64> = Array2::zeros(data.dim());
        for i in 0..window {
            let delayed = ts_delay(data, i);
            let diff = &delayed - &mean;
            var_sum = var_sum + &diff * &diff;
        }
        
        // Variance = sum of squared deviations / window
        let variance = var_sum / (window as f64);
        
        // Standard deviation = sqrt(variance)
        variance.mapv(|v| f64::sqrt(v + 1e-9))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use ndarray::arr2;

    #[test]
    fn test_bollinger_basic() {
        // Constant price should have zero bandwidth
        let close = arr2(&[[10.0, 10.0, 10.0, 10.0, 10.0]]);
        let (upper, middle, lower) = BollingerBands::bollinger_custom(&close, 3, 2.0);
        
        // With constant price, all bands should equal the price
        // (std dev = 0)
        assert_abs_diff_eq!(middle[[0, 4]], 10.0, epsilon = 0.1);
        assert_abs_diff_eq!(upper[[0, 4]], middle[[0, 4]], epsilon = 0.1);
        assert_abs_diff_eq!(lower[[0, 4]], middle[[0, 4]], epsilon = 0.1);
    }

    #[test]
    fn test_bollinger_bands_ordering() {
        // Volatile price series
        let close = arr2(&[[10.0, 15.0, 8.0, 20.0, 5.0, 18.0]]);
        let (upper, middle, lower) = BollingerBands::bollinger_custom(&close, 3, 2.0);
        
        // Upper should be >= middle >= lower
        for t in 2..6 {
            assert!(
                upper[[0, t]] >= middle[[0, t]],
                "Upper band should be >= middle at t={}",
                t
            );
            assert!(
                middle[[0, t]] >= lower[[0, t]],
                "Middle should be >= lower at t={}",
                t
            );
        }
    }

    #[test]
    fn test_bandwidth() {
        // High volatility should give larger bandwidth
        let stable = arr2(&[[10.0, 10.1, 10.2, 10.1, 10.0]]);
        let volatile = arr2(&[[10.0, 5.0, 15.0, 8.0, 20.0]]);
        
        let bw_stable = BollingerBands::bandwidth(&stable, 3);
        let bw_volatile = BollingerBands::bandwidth(&volatile, 3);
        
        assert!(
            bw_volatile[[0, 4]] > bw_stable[[0, 4]],
            "Volatile series should have higher bandwidth"
        );
    }

    #[test]
    fn test_percent_b() {
        let close = arr2(&[[10.0, 11.0, 12.0, 13.0, 14.0]]);
        let pb = BollingerBands::percent_b(&close, 3);
        
        // %B should be between 0 and 1 for prices within bands
        // (though can exceed during high volatility)
        assert_eq!(pb.dim(), close.dim());
        
        // Should have reasonable values
        for val in pb.iter() {
            assert!(val.abs() < 10.0, "Percent B should have reasonable magnitude");
        }
    }

    #[test]
    fn test_rolling_std() {
        // Known std dev case
        let data = arr2(&[[1.0, 2.0, 3.0, 4.0, 5.0]]);
        let std = BollingerBands::rolling_std(&data, 3);
        
        // For window=3 at t=2: [1,2,3], mean=2, std=sqrt((1+0+1)/3)=0.816
        // Close enough check
        assert!(std[[0, 2]] > 0.5 && std[[0, 2]] < 1.5);
    }
}
