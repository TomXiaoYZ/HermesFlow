use crate::factors::moving_averages::MovingAverages;
use crate::vm::ops::ts_delay;
use ndarray::Array2;

/// Average True Range (ATR) - Volatility indicator
pub struct ATR;

impl ATR {
    /// Calculate True Range
    /// 
    /// # Arguments
    /// * `high` - High prices
    /// * `low` - Low prices  
    /// * `close` - Closing prices
    /// 
    /// # Formula
    /// TR = max(high - low, |high - prev_close|, |low - prev_close|)
    /// 
    /// # Returns
    /// True Range values
    pub fn true_range(
        high: &Array2<f64>,
        low: &Array2<f64>,
        close: &Array2<f64>,
    ) -> Array2<f64> {
        let prev_close = ts_delay(close, 1);
        
        // Calculate three components
        let hl = high - low;
        let hc = (high - &prev_close).mapv(f64::abs);
        let lc = (low - &prev_close).mapv(f64::abs);
        
        // TR = max of the three
        let (batch_size, time_steps) = high.dim();
        let mut tr = Array2::zeros((batch_size, time_steps));
        
        for b in 0..batch_size {
            for t in 0..time_steps {
                tr[[b, t]] = hl[[b, t]]
                    .max(hc[[b, t]])
                    .max(lc[[b, t]]);
            }
        }
        
        tr
    }

    /// Calculate Average True Range with standard period (14)
    /// 
    /// # Arguments
    /// * `high` - High prices
    /// * `low` - Low prices
    /// * `close` - Closing prices
    /// 
    /// # Returns
    /// ATR values (Wilder's smoothed average)
    pub fn atr(
        high: &Array2<f64>,
        low: &Array2<f64>,
        close: &Array2<f64>,
    ) -> Array2<f64> {
        Self::atr_custom(high, low, close, 14)
    }

    /// Calculate ATR with custom period
    /// 
    /// # Arguments
    /// * `high` - High prices
    /// * `low` - Low prices
    /// * `close` - Closing prices  
    /// * `period` - Smoothing period (default 14)
    /// 
    /// # Uses Wilder's smoothing
    /// ATR[t] = ((period-1) * ATR[t-1] + TR[t]) / period
    pub fn atr_custom(
        high: &Array2<f64>,
        low: &Array2<f64>,
        close: &Array2<f64>,
        period: usize,
    ) -> Array2<f64> {
        let tr = Self::true_range(high, low, close);
        
        // Wilder's smoothing is similar to EMA but with different alpha
        // alpha = 1 / period (vs 2/(period+1) for regular EMA)
        let alpha = 1.0 / period as f64;
        
        let (batch_size, time_steps) = tr.dim();
        let mut atr = Array2::zeros((batch_size, time_steps));
        
        // Initialize with first TR value
        for b in 0..batch_size {
            atr[[b, 0]] = tr[[b, 0]];
            
            // Apply Wilder's smoothing
            for t in 1..time_steps {
                atr[[b, t]] = alpha * tr[[b, t]] + (1.0 - alpha) * atr[[b, t - 1]];
            }
        }
        
        atr
    }

    /// Calculate normalized ATR (ATR as percentage of price)
    /// 
    /// # Arguments
    /// * `high`, `low`, `close` - Price data
    /// 
    /// # Returns
    /// ATR / close (volatility as percentage)
    pub fn atr_percent(
        high: &Array2<f64>,
        low: &Array2<f64>,
        close: &Array2<f64>,
    ) -> Array2<f64> {
        let atr = Self::atr(high, low, close);
        
        // Normalize by close price
        &atr / (close + 1e-9)
    }

    /// Calculate ATR Ratio (current ATR vs its SMA)
    /// 
    /// Identifies when volatility is expanding or contracting
    /// 
    /// # Returns
    /// ATR / SMA(ATR, window)
    /// > 1.0: volatility expanding
    /// < 1.0: volatility contracting
    pub fn atr_ratio(
        high: &Array2<f64>,
        low: &Array2<f64>,
        close: &Array2<f64>,
        window: usize,
    ) -> Array2<f64> {
        let atr = Self::atr(high, low, close);
        let atr_sma = MovingAverages::sma(&atr, window);
        
        &atr / (&atr_sma + 1e-9)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use ndarray::arr2;

    #[test]
    fn test_true_range_basic() {
        // Simple case: high-low is largest
        let high = arr2(&[[15.0, 16.0, 17.0]]);
        let low = arr2(&[[10.0, 11.0, 12.0]]);
        let close = arr2(&[[12.0, 13.0, 14.0]]);
        
        let tr = ATR::true_range(&high, &low, &close);
        
        // t=0: max(5, |15-0|, |10-0|) = 15 (prev_close=0 from padding)
        // t=1: max(5, |16-12|, |11-12|) = max(5, 4, 1) = 5
        // t=2: max(5, |17-13|, |12-13|) = max(5, 4, 1) = 5
        
        assert_eq!(tr[[0, 1]], 5.0);
        assert_eq!(tr[[0, 2]], 5.0);
    }

    #[test]
    fn test_true_range_gap_up() {
        // Gap up: high - prev_close is largest
        let high = arr2(&[[12.0, 20.0]]);
        let low = arr2(&[[10.0, 18.0]]);
        let close = arr2(&[[11.0, 19.0]]);
        
        let tr = ATR::true_range(&high, &low, &close);
        
        // t=1: max(2, |20-11|, |18-11|) = max(2, 9, 7) = 9
        assert_eq!(tr[[0, 1]], 9.0);
    }

    #[test]
    fn test_true_range_gap_down() {
        // Gap down: prev_close - low is largest
        let high = arr2(&[[20.0, 12.0]]);
        let low = arr2(&[[18.0, 10.0]]);
        let close = arr2(&[[19.0, 11.0]]);
        
        let tr = ATR::true_range(&high, &low, &close);
        
        // t=1: max(2, |12-19|, |10-19|) = max(2, 7, 9) = 9
        assert_eq!(tr[[0, 1]], 9.0);
    }

    #[test]
    fn test_atr_smoothing() {
        let high = arr2(&[[15.0, 16.0, 17.0, 18.0, 19.0]]);
        let low = arr2(&[[10.0, 11.0, 12.0, 13.0, 14.0]]);
        let close = arr2(&[[12.0, 13.0, 14.0, 15.0, 16.0]]);
        
        let atr = ATR::atr_custom(&high, &low, &close, 3);
        
        // ATR should smooth out the TR values
        assert_eq!(atr.dim(), high.dim());
        
        // Later values should be smoother
        for val in atr.iter() {
            assert!(*val >= 0.0, "ATR should be non-negative");
        }
    }

    #[test]
    fn test_atr_percent() {
        let high = arr2(&[[110.0, 120.0, 130.0]]);
        let low = arr2(&[[90.0, 100.0, 110.0]]);
        let close = arr2(&[[100.0, 110.0, 120.0]]);
        
        let atr_pct = ATR::atr_percent(&high, &low, &close);
        
        // Should be normalized (smaller than raw ATR)
        let raw_atr = ATR::atr(&high, &low, &close);
        
        assert!(atr_pct[[0, 2]] < raw_atr[[0, 2]]);
        assert!(atr_pct[[0, 2]] > 0.0 && atr_pct[[0, 2]] < 1.0);
    }

    #[test]
    fn test_atr_ratio() {
        // Increasing volatility
        let high = arr2(&[[11.0, 12.0, 15.0, 20.0, 30.0]]);
        let low = arr2(&[[9.0, 10.0, 13.0, 18.0, 25.0]]);
        let close = arr2(&[[10.0, 11.0, 14.0, 19.0, 27.0]]);
        
        let ratio = ATR::atr_ratio(&high, &low, &close, 2);
        
        // Should show expanding volatility (ratio > 1) in later periods
        assert_eq!(ratio.dim(), high.dim());
    }
}
