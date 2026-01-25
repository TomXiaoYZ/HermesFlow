use crate::factors::moving_averages::MovingAverages;
use crate::vm::ops::ts_delay;
use ndarray::Array2;

/// MFI (Money Flow Index)
/// Volume-weighted RSI - measures buying/selling pressure
pub struct MFI;

impl MFI {
    /// Calculate MFI with standard period (14)
    /// 
    /// # Arguments
    /// * `high` - High prices
    /// * `low` - Low prices
    /// * `close` - Closing prices
    /// * `volume` - Trading volume
    /// 
    /// # Returns
    /// MFI values (0-100 range)
    /// - Above 80: Overbought
    /// - Below 20: Oversold
    pub fn mfi(
        high: &Array2<f64>,
        low: &Array2<f64>,
        close: &Array2<f64>,
        volume: &Array2<f64>,
    ) -> Array2<f64> {
        Self::mfi_custom(high, low, close, volume, 14)
    }

    /// Calculate MFI with custom period
    /// 
    /// # Formula
    /// 1. Typical Price (TP) = (High + Low + Close) / 3
    /// 2. Raw Money Flow (RMF) = TP × Volume
    /// 3. Positive/Negative Money Flow based on TP direction
    /// 4. Money Flow Ratio (MFR) = Positive MF / Negative MF
    /// 5. MFI = 100 - (100 / (1 + MFR))
    pub fn mfi_custom(
        high: &Array2<f64>,
        low: &Array2<f64>,
        close: &Array2<f64>,
        volume: &Array2<f64>,
        period: usize,
    ) -> Array2<f64> {
        // 1. Calculate Typical Price
        let typical_price = (high + low + close) / 3.0;
        
        // 2. Raw Money Flow
        let raw_mf = &typical_price * volume;
        
        // 3. Separate positive and negative flows
        let tp_prev = ts_delay(&typical_price, 1);
        let tp_change = &typical_price - &tp_prev;
        
        // Positive flow when TP increases
        let positive_flow = raw_mf.clone().into_iter()
            .zip(tp_change.iter())
            .map(|(mf, &change)| if change > 0.0 { mf } else { 0.0 })
            .collect::<Vec<f64>>();
        
        // Negative flow when TP decreases
        let negative_flow = raw_mf.clone().into_iter()
            .zip(tp_change.iter())
            .map(|(mf, &change)| if change < 0.0 { mf } else { 0.0 })
            .collect::<Vec<f64>>();
        
        let (batch, time) = close.dim();
        let pos_flow_arr = Array2::from_shape_vec((batch, time), positive_flow).unwrap();
        let neg_flow_arr = Array2::from_shape_vec((batch, time), negative_flow).unwrap();
        
        // 4. Sum over period
        let mut sum_pos: Array2<f64> = Array2::zeros((batch, time));
        let mut sum_neg: Array2<f64> = Array2::zeros((batch, time));
        
        for i in 0..period {
            sum_pos = sum_pos + ts_delay(&pos_flow_arr, i);
            sum_neg = sum_neg + ts_delay(&neg_flow_arr, i);
        }
        
        // 5. MFI calculation
        let mfr = &sum_pos / (&sum_neg + 1e-9);
        let mfi = 100.0 - (100.0 / (1.0 + &mfr));
        
        mfi
    }

    /// Normalized MFI (scaled to -1 to 1)
    /// 
    /// Converts from [0, 100] to [-1, 1] for ML features
    pub fn mfi_normalized(
        high: &Array2<f64>,
        low: &Array2<f64>,
        close: &Array2<f64>,
        volume: &Array2<f64>,
        period: usize,
    ) -> Array2<f64> {
        let mfi = Self::mfi_custom(high, low, close, volume, period);
        
        // (MFI - 50) / 50 to get range [-1, 1]
        (mfi - 50.0) / 50.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::arr2;

    #[test]
    fn test_mfi_basic() {
        let high = arr2(&[[15.0, 16.0, 17.0, 18.0, 19.0]]);
        let low = arr2(&[[10.0, 11.0, 12.0, 13.0, 14.0]]);
        let close = arr2(&[[12.0, 13.0, 14.0, 15.0, 16.0]]);
        let volume = arr2(&[[100.0, 150.0, 200.0, 180.0, 220.0]]);
        
        let mfi = MFI::mfi_custom(&high, &low, &close, &volume, 3);
        
        // MFI should be in [0, 100] range
        for val in mfi.iter() {
            assert!(
                *val >= 0.0 && *val <= 100.0,
                "MFI should be in [0, 100], got {}",
                val
            );
        }
    }

    #[test]
    fn test_mfi_uptrend() {
        // Strong uptrend with high volume
        let high = arr2(&[[11.0, 12.0, 13.0, 14.0, 15.0, 16.0]]);
        let low = arr2(&[[10.0, 11.0, 12.0, 13.0, 14.0, 15.0]]);
        let close = arr2(&[[10.5, 11.5, 12.5, 13.5, 14.5, 15.5]]);
        let volume = arr2(&[[100.0, 100.0, 100.0, 100.0, 100.0, 100.0]]);
        
        let mfi = MFI::mfi_custom(&high, &low, &close, &volume, 4);
        
        // In uptrend, MFI should be high (approaching overbought)
        assert!(mfi[[0, 5]] > 50.0, "MFI should be high in uptrend");
    }

    #[test]
    fn test_mfi_downtrend() {
        // Downtrend
        let high = arr2(&[[16.0, 15.0, 14.0, 13.0, 12.0, 11.0]]);
        let low = arr2(&[[15.0, 14.0, 13.0, 12.0, 11.0, 10.0]]);
        let close = arr2(&[[15.5, 14.5, 13.5, 12.5, 11.5, 10.5]]);
        let volume = arr2(&[[100.0, 100.0, 100.0, 100.0, 100.0, 100.0]]);
        
        let mfi = MFI::mfi_custom(&high, &low, &close, &volume, 4);
        
        // In downtrend, MFI should be low (approaching oversold)
        assert!(mfi[[0, 5]] < 50.0, "MFI should be low in downtrend");
    }

    #[test]
    fn test_mfi_normalized() {
        let high = arr2(&[[15.0, 16.0, 14.0, 17.0]]);
        let low = arr2(&[[10.0, 11.0, 9.0, 12.0]]);
        let close = arr2(&[[12.0, 13.0, 11.0, 14.0]]);
        let volume = arr2(&[[100.0, 150.0, 120.0, 180.0]]);
        
        let mfi_norm = MFI::mfi_normalized(&high, &low, &close, &volume, 3);
        
        // Normalized should be in [-1, 1]
        for val in mfi_norm.iter() {
            assert!(
                *val >= -1.0 && *val <= 1.0,
                "Normalized MFI should be in [-1, 1], got {}",
                val
            );
        }
    }

    #[test]
    fn test_mfi_overbought_oversold() {
        // Create overbought scenario (strong buying pressure)
        let high = arr2(&[[20.0, 22.0, 25.0, 28.0]]);
        let low = arr2(&[[18.0, 20.0, 23.0, 26.0]]);
        let close = arr2(&[[19.0, 21.0, 24.0, 27.0]]);
        let volume = arr2(&[[100.0, 200.0, 300.0, 400.0]]); // Increasing volume
        
        let mfi = MFI::mfi(&high, &low, &close, &volume);
        
        // Should indicate strong buying (high MFI)
        assert!(mfi[[0, 3]] > 50.0, "Strong uptrend should have high MFI");
    }
}
