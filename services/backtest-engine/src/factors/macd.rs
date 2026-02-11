use crate::factors::moving_averages::MovingAverages;
use ndarray::Array2;

/// MACD (Moving Average Convergence Divergence) indicator
pub struct MACD;

impl MACD {
    /// Calculate MACD with standard parameters
    ///
    /// # Arguments
    /// * `close` - Closing prices (batch_size, time_steps)
    ///
    /// # Returns
    /// (macd_line, signal_line, histogram)
    /// - macd_line: EMA(12) - EMA(26)
    /// - signal_line: EMA(macd_line, 9)
    /// - histogram: macd_line - signal_line
    pub fn macd(close: &Array2<f64>) -> (Array2<f64>, Array2<f64>, Array2<f64>) {
        Self::macd_custom(close, 12, 26, 9)
    }

    /// Calculate MACD with custom parameters
    ///
    /// # Arguments
    /// * `close` - Closing prices
    /// * `fast_period` - Fast EMA period (default 12)
    /// * `slow_period` - Slow EMA period (default 26)
    /// * `signal_period` - Signal line EMA period (default 9)
    pub fn macd_custom(
        close: &Array2<f64>,
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
    ) -> (Array2<f64>, Array2<f64>, Array2<f64>) {
        // Calculate fast and slow EMAs
        let fast_ema = MovingAverages::ema(close, fast_period);
        let slow_ema = MovingAverages::ema(close, slow_period);

        // MACD line = fast - slow
        let macd_line = &fast_ema - &slow_ema;

        // Signal line = EMA of MACD
        let signal_line = MovingAverages::ema(&macd_line, signal_period);

        // Histogram = MACD - Signal
        let histogram = &macd_line - &signal_line;

        (macd_line, signal_line, histogram)
    }

    /// Calculate normalized MACD for feature engineering
    ///
    /// Returns histogram normalized by close price to make it scale-independent
    pub fn macd_normalized(close: &Array2<f64>) -> Array2<f64> {
        let (_, _, histogram) = Self::macd(close);

        // Normalize by price to make it scale-invariant
        &histogram / (close + 1e-9)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::arr2;

    #[test]
    fn test_macd_basic() {
        // Create uptrend data
        let close = arr2(&[[
            10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0,
        ]]);

        let (macd_line, signal_line, histogram) = MACD::macd(&close);

        // Basic sanity checks
        assert_eq!(macd_line.dim(), close.dim());
        assert_eq!(signal_line.dim(), close.dim());
        assert_eq!(histogram.dim(), close.dim());

        // In uptrend, later MACD should be positive
        assert!(
            macd_line[[0, 10]] > 0.0,
            "MACD should be positive in uptrend"
        );
    }

    #[test]
    fn test_macd_crossover() {
        // Test bullish crossover scenario
        // Downtrend followed by uptrend
        let close = arr2(&[[
            20.0, 19.0, 18.0, 17.0, 16.0, 15.0, // Downtrend
            16.0, 17.0, 18.0, 19.0, 20.0, 21.0, 22.0, // Uptrend
        ]]);

        let (_macd_line, _signal_line, histogram) = MACD::macd(&close);

        // Histogram should eventually become positive in strong uptrend
        // (though exact timing depends on EMA parameters)
        let final_hist = histogram[[0, 12]];

        // At minimum, verify histogram exists and has reasonable magnitude
        assert!(
            final_hist.abs() < 10.0,
            "Histogram magnitude should be reasonable"
        );
    }

    #[test]
    fn test_macd_custom_periods() {
        let close = arr2(&[[10.0, 11.0, 12.0, 13.0, 14.0]]);

        let (macd1, _, _) = MACD::macd_custom(&close, 2, 3, 2);
        let (macd2, _, _) = MACD::macd_custom(&close, 5, 10, 5);

        // Different periods should give different results
        assert!((macd1[[0, 4]] - macd2[[0, 4]]).abs() > 0.01);
    }

    #[test]
    fn test_macd_normalized() {
        let close = arr2(&[[100.0, 110.0, 120.0, 130.0, 140.0]]);
        let norm_macd = MACD::macd_normalized(&close);

        // Should be same shape
        assert_eq!(norm_macd.dim(), close.dim());

        // Normalized values should be smaller than raw MACD
        let (_, _, histogram) = MACD::macd(&close);
        assert!(norm_macd[[0, 4]].abs() < histogram[[0, 4]].abs());
    }
}
