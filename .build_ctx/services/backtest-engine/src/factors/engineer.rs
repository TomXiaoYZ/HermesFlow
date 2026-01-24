use ndarray::{Array2, Axis};
// use ndarray_stats::Quantile1dExt;
use crate::vm::ops::ts_delay;
use super::indicators::MemeIndicators;

pub struct FeatureEngineer;

impl FeatureEngineer {
    pub const INPUT_DIM: usize = 6;

    /// Robust normalization: (x - median) / MAD
    pub fn robust_norm(x: &Array2<f64>) -> Array2<f64> {
        let mut out = x.clone();
        
        // Compute per-row (batch item) statistics
        for mut row in out.rows_mut() {
            // Collect into vec for sorting
            let mut v: Vec<f64> = row.to_vec();
            // Sort ignoring NaNs (treat as large or small, or filter)
            // Here we assume clean data or standard sort behavior
            v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            
            let len = v.len();
            if len == 0 { continue; }
            let median = if len % 2 == 0 {
                (v[len/2 - 1] + v[len/2]) / 2.0
            } else {
                v[len/2]
            };
            
            // MAD
            let mut diffs: Vec<f64> = row.mapv(|v| (v - median).abs()).to_vec();
            diffs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let mad = if len % 2 == 0 {
                (diffs[len/2 - 1] + diffs[len/2]) / 2.0
            } else {
                diffs[len/2]
            } + 1e-6;
            
            row.mapv_inplace(|v| {
                let norm = (v - median) / mad;
                norm.clamp(-5.0, 5.0)
            });
        }
        
        out
    }

    /// Compute basic features
    /// Returns: (batch, features, time) which is flattened/permuted to fit VM input?
    /// VM expects (batch, features, time)
    pub fn compute_features(
        close: &Array2<f64>,
        open: &Array2<f64>,
        high: &Array2<f64>,
        low: &Array2<f64>,
        volume: &Array2<f64>,
        liquidity: &Array2<f64>,
        fdv: &Array2<f64>,
    ) -> ndarray::Array3<f64> {
        // 1. Log returns
        let prev_close = ts_delay(close, 1);
        let ret = (close / (&prev_close + 1e-9)).mapv(f64::ln);
        
        // 2. Liquidity Score
        let liq_score = MemeIndicators::liquidity_health(liquidity, fdv);
        
        // 3. Pressure
        let pressure = MemeIndicators::buy_sell_imbalance(close, open, high, low);
        
        // 4. FOMO
        let fomo = MemeIndicators::fomo_acceleration(volume);
        
        // 5. Deviation
        let dev = MemeIndicators::pump_deviation(close, 20); // Default window 20
        
        // 6. Log Volume
        let log_vol = volume.mapv(|v| (1.0 + v).ln());
        
        // Normalize
        let ret_norm = Self::robust_norm(&ret);
        let fomo_norm = Self::robust_norm(&fomo);
        let dev_norm = Self::robust_norm(&dev);
        let log_vol_norm = Self::robust_norm(&log_vol);
        
        // Stack into Array3 inputs
        // Shape: (batch, features, time)
        let (batch, time) = close.dim();
        let features = 6;
        let mut out = ndarray::Array3::<f64>::zeros((batch, features, time));
        
        // Assign slices
        // Axis 1 is feature dimension
        out.index_axis_mut(Axis(1), 0).assign(&ret_norm);
        out.index_axis_mut(Axis(1), 1).assign(&liq_score);
        out.index_axis_mut(Axis(1), 2).assign(&pressure);
        out.index_axis_mut(Axis(1), 3).assign(&fomo_norm);
        out.index_axis_mut(Axis(1), 4).assign(&dev_norm);
        out.index_axis_mut(Axis(1), 5).assign(&log_vol_norm);
        
        out
    }
}
