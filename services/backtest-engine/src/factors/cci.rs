use crate::factors::moving_averages::MovingAverages;
use crate::vm::ops::ts_delay;
use ndarray::Array2;

/// CCI (Commodity Channel Index)
/// Momentum oscillator used to identify cyclical trends
pub struct CCI;

impl CCI {
    /// Calculate CCI with standard period (20)
    ///
    /// # Arguments
    /// * `high` - High prices
    /// * `low` - Low prices
    /// * `close` - Closing prices
    ///
    /// # Returns
    /// CCI values (typically range -100 to +100, but unbounded)
    pub fn cci(high: &Array2<f64>, low: &Array2<f64>, close: &Array2<f64>) -> Array2<f64> {
        Self::cci_custom(high, low, close, 20)
    }

    /// Calculate CCI with custom period
    ///
    /// # Arguments
    /// * `high` - High prices
    /// * `low` - Low prices
    /// * `close` - Closing prices
    /// * `period` - Lookback period (default 20)
    ///
    /// # Formula
    /// 1. Typical Price (TP) = (High + Low + Close) / 3
    /// 2. SMA of TP
    /// 3. Mean Deviation = Average of |TP - SMA(TP)|
    /// 4. CCI = (TP - SMA(TP)) / (0.015 × Mean Deviation)
    ///
    /// The constant 0.015 ensures ~70-80% of values fall between -100 and +100
    pub fn cci_custom(
        high: &Array2<f64>,
        low: &Array2<f64>,
        close: &Array2<f64>,
        period: usize,
    ) -> Array2<f64> {
        // 1. Calculate Typical Price
        let typical_price = (high + low + close) / 3.0;

        // 2. SMA of Typical Price
        let sma_tp = MovingAverages::sma(&typical_price, period);

        // 3. Calculate Mean Deviation
        let mut mad_sum: Array2<f64> = Array2::zeros(typical_price.dim());

        for i in 0..period {
            let delayed_tp = ts_delay(&typical_price, i);
            let deviation = (&delayed_tp - &sma_tp).mapv(f64::abs);
            mad_sum = mad_sum + deviation;
        }

        let mean_deviation = mad_sum / (period as f64);

        // 4. CCI = (TP - SMA) / (0.015 * MAD)
        let numerator = &typical_price - &sma_tp;
        let denominator = &mean_deviation * 0.015 + 1e-9;

        numerator / denominator
    }

    /// Normalized CCI (clamped to -3 to +3 standard deviations)
    ///
    /// More stable for ML features
    pub fn cci_normalized(
        high: &Array2<f64>,
        low: &Array2<f64>,
        close: &Array2<f64>,
        period: usize,
    ) -> Array2<f64> {
        let cci = Self::cci_custom(high, low, close, period);

        // Clamp to ±300 (roughly 3 std devs) and normalize to ±1
        cci.mapv(|v| (v / 300.0).clamp(-1.0, 1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::arr2;

    #[test]
    fn test_cci_basic() {
        let high = arr2(&[[15.0, 16.0, 17.0, 18.0, 19.0]]);
        let low = arr2(&[[10.0, 11.0, 12.0, 13.0, 14.0]]);
        let close = arr2(&[[12.0, 13.0, 14.0, 15.0, 16.0]]);

        let cci = CCI::cci_custom(&high, &low, &close, 3);

        // CCI should be calculated
        assert_eq!(cci.dim(), close.dim());

        // Values should be reasonable (not NaN or Inf)
        for val in cci.iter() {
            assert!(val.is_finite(), "CCI should be finite, got {}", val);
        }
    }

    #[test]
    fn test_cci_uptrend() {
        // Strong uptrend
        let high = arr2(&[[11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0]]);
        let low = arr2(&[[10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0]]);
        let close = arr2(&[[10.5, 11.5, 12.5, 13.5, 14.5, 15.5, 16.5]]);

        let cci = CCI::cci_custom(&high, &low, &close, 5);

        // In uptrend, later CCI values should be positive
        assert!(cci[[0, 6]] > 0.0, "CCI should be positive in uptrend");
    }

    #[test]
    fn test_cci_range() {
        // Most values should fall within typical range
        let high = arr2(&[[15.0, 16.0, 14.0, 17.0, 13.0, 18.0]]);
        let low = arr2(&[[10.0, 11.0, 9.0, 12.0, 8.0, 13.0]]);
        let close = arr2(&[[12.0, 13.0, 11.0, 14.0, 10.0, 15.0]]);

        let cci = CCI::cci_custom(&high, &low, &close, 4);

        // CCI is unbounded but typically ±100 for normal conditions
        // Just check they're reasonable
        for val in cci.iter() {
            assert!(val.abs() < 500.0, "CCI magnitude should be reasonable");
        }
    }

    #[test]
    fn test_cci_normalized() {
        let high = arr2(&[[20.0, 25.0, 22.0, 28.0]]);
        let low = arr2(&[[15.0, 20.0, 17.0, 23.0]]);
        let close = arr2(&[[17.0, 22.0, 19.0, 25.0]]);

        let cci_norm = CCI::cci_normalized(&high, &low, &close, 3);

        // Normalized should be between -1 and 1
        for val in cci_norm.iter() {
            assert!(
                *val >= -1.0 && *val <= 1.0,
                "Normalized CCI should be in [-1, 1], got {}",
                val
            );
        }
    }
}
