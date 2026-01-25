use crate::vm::ops::ts_delay;
use ndarray::Array2;

/// OBV (On-Balance Volume)
/// Cumulative volume indicator based on price direction
pub struct OBV;

impl OBV {
    /// Calculate On-Balance Volume
    /// 
    /// # Arguments
    /// * `close` - Closing prices
    /// * `volume` - Trading volume
    /// 
    /// # Formula
    /// If Close > Close_prev: OBV = OBV_prev + Volume
    /// If Close < Close_prev: OBV = OBV_prev - Volume
    /// If Close = Close_prev: OBV = OBV_prev
    /// 
    /// # Returns
    /// Cumulative OBV values
    pub fn obv(close: &Array2<f64>, volume: &Array2<f64>) -> Array2<f64> {
        let (batch, time) = close.dim();
        let mut obv: Array2<f64> = Array2::zeros((batch, time));
        
        for b in 0..batch {
            let mut cumulative_obv = 0.0;
            
            for t in 0..time {
                if t == 0 {
                    // Initialize with first volume
                    cumulative_obv = volume[[b, t]];
                } else {
                    let curr_close = close[[b, t]];
                    let prev_close = close[[b, t - 1]];
                    
                    if curr_close > prev_close {
                        cumulative_obv += volume[[b, t]];
                    } else if curr_close < prev_close {
                        cumulative_obv -= volume[[b, t]];
                    }
                    // If equal, OBV unchanged
                }
                
                obv[[b, t]] = cumulative_obv;
            }
        }
        
        obv
    }

    /// OBV Change (momentum)
    /// 
    /// # Returns
    /// OBV[t] - OBV[t-1]
    pub fn obv_change(close: &Array2<f64>, volume: &Array2<f64>) -> Array2<f64> {
        let obv = Self::obv(close, volume);
        let obv_prev = ts_delay(&obv, 1);
        &obv - &obv_prev
    }

    /// OBV Percentage Change
    /// 
    /// # Returns
    /// (OBV[t] - OBV[t-1]) / |OBV[t-1]|
    pub fn obv_pct_change(close: &Array2<f64>, volume: &Array2<f64>) -> Array2<f64> {
        let obv = Self::obv(close, volume);
        let obv_prev = ts_delay(&obv, 1);
        
        let change = &obv - &obv_prev;
        let abs_prev = obv_prev.mapv(f64::abs);
        
        &change / (&abs_prev + 1e-9)
    }

    /// OBV Oscillator (OBV EMA difference)
    /// 
    /// # Arguments
    /// * `close` - Closing prices
    /// * `volume` - Trading volume
    /// * `fast_period` - Fast EMA period
    /// * `slow_period` - Slow EMA period
    /// 
    /// # Returns
    /// EMA(OBV, fast) - EMA(OBV, slow)
    pub fn obv_oscillator(
        close: &Array2<f64>,
        volume: &Array2<f64>,
        fast_period: usize,
        slow_period: usize,
    ) -> Array2<f64> {
        use crate::factors::moving_averages::MovingAverages;
        
        let obv = Self::obv(close, volume);
        let fast_ema = MovingAverages::ema(&obv, fast_period);
        let slow_ema = MovingAverages::ema(&obv, slow_period);
        
        &fast_ema - &slow_ema
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::arr2;

    #[test]
    fn test_obv_basic() {
        // Rising prices
        let close = arr2(&[[10.0, 11.0, 12.0, 13.0]]);
        let volume = arr2(&[[100.0, 150.0, 200.0, 180.0]]);
        
        let obv = OBV::obv(&close, &volume);
        
        // OBV should be increasing with rising prices
        assert!(obv[[0, 3]] > obv[[0, 0]], "OBV should increase with price");
    }

    #[test]
    fn test_obv_direction() {
        // Up, down, up pattern
        let close = arr2(&[[10.0, 12.0, 11.0, 13.0]]);
        let volume = arr2(&[[100.0, 100.0, 100.0, 100.0]]);
        
        let obv = OBV::obv(&close, &volume);
        
        // t=0: 100
        // t=1: 100 + 100 = 200 (up)
        // t=2: 200 - 100 = 100 (down)
        // t=3: 100 + 100 = 200 (up)
        
        assert_eq!(obv[[0, 0]], 100.0);
        assert_eq!(obv[[0, 1]], 200.0);
        assert_eq!(obv[[0, 2]], 100.0);
        assert_eq!(obv[[0, 3]], 200.0);
    }

    #[test]
    fn test_obv_unchanged_price() {
        // Flat price
        let close = arr2(&[[10.0, 10.0, 10.0]]);
        let volume = arr2(&[[100.0, 150.0, 200.0]]);
        
        let obv = OBV::obv(&close, &volume);
        
        // OBV should stay constant when price unchanged
        assert_eq!(obv[[0, 1]], obv[[0, 0]]);
        assert_eq!(obv[[0, 2]], obv[[0, 1]]);
    }

    #[test]
    fn test_obv_change() {
        let close = arr2(&[[10.0, 11.0, 12.0]]);
        let volume = arr2(&[[100.0, 150.0, 200.0]]);
        
        let change = OBV::obv_change(&close, &volume);
        
        // Should have same dimensions
        assert_eq!(change.dim(), close.dim());
    }

    #[test]
    fn test_obv_oscillator() {
        let close = arr2(&[[10.0, 11.0, 12.0, 13.0, 14.0, 15.0]]);
        let volume = arr2(&[[100.0, 100.0, 100.0, 100.0, 100.0, 100.0]]);
        
        let osc = OBV::obv_oscillator(&close, &volume, 2, 4);
        
        // Should be calculated
        assert_eq!(osc.dim(), close.dim());
    }
}
