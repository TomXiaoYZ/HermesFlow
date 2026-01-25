use crate::factors::moving_averages::MovingAverages;
use crate::vm::ops::ts_delay;
use ndarray::Array2;

/// VWAP (Volume-Weighted Average Price)
/// Price averaged by volume - institutional benchmark
pub struct VWAP;

impl VWAP {
    /// Calculate VWAP (cumulative from start)
    /// 
    /// # Arguments
    /// * `high` - High prices
    /// * `low` - Low prices
    /// * `close` - Closing prices
    /// * `volume` - Trading volume
    /// 
    /// # Formula
    /// Typical Price = (High + Low + Close) / 3
    /// VWAP = Σ(Typical Price × Volume) / Σ(Volume)
    /// 
    /// # Returns
    /// VWAP values (cumulative)
    pub fn vwap(
        high: &Array2<f64>,
        low: &Array2<f64>,
        close: &Array2<f64>,
        volume: &Array2<f64>,
    ) -> Array2<f64> {
        // Typical Price
        let typical_price = (high + low + close) / 3.0;
        
        // Cumulative (TP * Volume)
        let pv = &typical_price * volume;
        
        let (batch, time) = close.dim();
        let mut cumsum_pv: Array2<f64> = Array2::zeros((batch, time));
        let mut cumsum_vol: Array2<f64> = Array2::zeros((batch, time));
        
        // Calculate cumulative sums
        for b in 0..batch {
            let mut sum_pv = 0.0;
            let mut sum_vol = 0.0;
            
            for t in 0..time {
                sum_pv += pv[[b, t]];
                sum_vol += volume[[b, t]];
                
                cumsum_pv[[b, t]] = sum_pv;
                cumsum_vol[[b, t]] = sum_vol;
            }
        }
        
        // VWAP = cumsum(PV) / cumsum(Volume)
        &cumsum_pv / (&cumsum_vol + 1e-9)
    }

    /// Calculate rolling VWAP over window
    /// 
    /// More responsive than cumulative VWAP
    pub fn vwap_rolling(
        high: &Array2<f64>,
        low: &Array2<f64>,
        close: &Array2<f64>,
        volume: &Array2<f64>,
        window: usize,
    ) -> Array2<f64> {
        let typical_price = (high + low + close) / 3.0;
        let pv = &typical_price * volume;
        
        // Rolling sum over window
        let mut sum_pv: Array2<f64> = Array2::zeros(close.dim());
        let mut sum_vol: Array2<f64> = Array2::zeros(close.dim());
        
        for i in 0..window {
            sum_pv = sum_pv + ts_delay(&pv, i);
            sum_vol = sum_vol + ts_delay(volume, i);
        }
        
        &sum_pv / (&sum_vol + 1e-9)
    }

    /// VWAP Deviation (price distance from VWAP)
    /// 
    /// # Returns
    /// (Close - VWAP) / VWAP
    /// Positive = price above VWAP (bullish)
    /// Negative = price below VWAP (bearish)
    pub fn vwap_deviation(
        high: &Array2<f64>,
        low: &Array2<f64>,
        close: &Array2<f64>,
        volume: &Array2<f64>,
    ) -> Array2<f64> {
        let vwap = Self::vwap(high, low, close, volume);
        (close - &vwap) / (&vwap + 1e-9)
    }

    /// VWAP Bands (standard deviation bands around VWAP)
    /// 
    /// # Returns
    /// (upper_band, vwap, lower_band)
    pub fn vwap_bands(
        high: &Array2<f64>,
        low: &Array2<f64>,
        close: &Array2<f64>,
        volume: &Array2<f64>,
        num_std: f64,
    ) -> (Array2<f64>, Array2<f64>, Array2<f64>) {
        let vwap = Self::vwap(high, low, close, volume);
        let typical_price = (high + low + close) / 3.0;
        
        // Calculate weighted variance
        let diff = &typical_price - &vwap;
        let diff_sq = &diff * &diff;
        let weighted_var = &diff_sq * volume;
        
        let (batch, time) = close.dim();
        let mut cumsum_wvar: Array2<f64> = Array2::zeros((batch, time));
        let mut cumsum_vol: Array2<f64> = Array2::zeros((batch, time));
        
        for b in 0..batch {
            let mut sum_wvar = 0.0;
            let mut sum_vol = 0.0;
            
            for t in 0..time {
                sum_wvar += weighted_var[[b, t]];
                sum_vol += volume[[b, t]];
                
                cumsum_wvar[[b, t]] = sum_wvar;
                cumsum_vol[[b, t]] = sum_vol;
            }
        }
        
        let variance = &cumsum_wvar / (&cumsum_vol + 1e-9);
        let std_dev = variance.mapv(|v| f64::sqrt(v + 1e-9));
        
        let upper_band = &vwap + &std_dev * num_std;
        let lower_band = &vwap - &std_dev * num_std;
        
        (upper_band, vwap, lower_band)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use ndarray::arr2;

    #[test]
    fn test_vwap_basic() {
        let high = arr2(&[[11.0, 12.0, 13.0]]);
        let low = arr2(&[[9.0, 10.0, 11.0]]);
        let close = arr2(&[[10.0, 11.0, 12.0]]);
        let volume = arr2(&[[100.0, 200.0, 150.0]]);
        
        let vwap = VWAP::vwap(&high, &low, &close, &volume);
        
        // VWAP should be calculated
        assert_eq!(vwap.dim(), close.dim());
        
        // First value should equal typical price (TP * V / V = TP)
        let tp0 = (11.0 + 9.0 + 10.0) / 3.0;
        assert_abs_diff_eq!(vwap[[0, 0]], tp0, epsilon = 0.01);
    }

    #[test]
    fn test_vwap_cumulative() {
        // Constant price, varying volume
        let high = arr2(&[[11.0, 11.0, 11.0]]);
        let low = arr2(&[[9.0, 9.0, 9.0]]);
        let close = arr2(&[[10.0, 10.0, 10.0]]);
        let volume = arr2(&[[100.0, 200.0, 300.0]]);
        
        let vwap = VWAP::vwap(&high, &low, &close, &volume);
        
        // With constant price, VWAP should equal typical price
        let tp = (11.0 + 9.0 + 10.0) / 3.0;
        assert_abs_diff_eq!(vwap[[0, 2]], tp, epsilon = 0.01);
    }

    #[test]
    fn test_vwap_rolling() {
        let high = arr2(&[[12.0, 13.0, 14.0, 15.0]]);
        let low = arr2(&[[10.0, 11.0, 12.0, 13.0]]);
        let close = arr2(&[[11.0, 12.0, 13.0, 14.0]]);
        let volume = arr2(&[[100.0, 200.0, 150.0, 180.0]]);
        
        let vwap_roll = VWAP::vwap_rolling(&high, &low, &close, &volume, 2);
        
        // Rolling should be smoother than price
        assert_eq!(vwap_roll.dim(), close.dim());
    }

    #[test]
    fn test_vwap_deviation() {
        let high = arr2(&[[15.0, 16.0, 14.0]]);
        let low = arr2(&[[10.0, 11.0, 9.0]]);
        let close = arr2(&[[13.0, 14.0, 10.0]]);
        let volume = arr2(&[[100.0, 100.0, 100.0]]);
        
        let dev = VWAP::vwap_deviation(&high, &low, &close, &volume);
        
        // Deviation should be reasonable
        for val in dev.iter() {
            assert!(val.abs() < 1.0, "Deviation should be reasonable");
        }
    }
}
