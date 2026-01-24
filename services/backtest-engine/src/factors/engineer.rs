use ndarray::{Array2, Axis};
// use ndarray_stats::Quantile1dExt;
use super::indicators::MemeIndicators;
use crate::vm::ops::ts_delay;

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
            if len == 0 {
                continue;
            }
            let median = if len % 2 == 0 {
                (v[len / 2 - 1] + v[len / 2]) / 2.0
            } else {
                v[len / 2]
            };

            // MAD
            let mut diffs: Vec<f64> = row.mapv(|v| (v - median).abs()).to_vec();
            diffs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let mad = if len % 2 == 0 {
                (diffs[len / 2 - 1] + diffs[len / 2]) / 2.0
            } else {
                diffs[len / 2]
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
        let log_vol = volume.mapv(|v| (v + 1.0).ln());

        // 7. Volatility Clustering
        let vol_cluster = MemeIndicators::volatility_clustering(&ret, 20);

        // 8. Momentum Reversal
        let mom_rev = MemeIndicators::momentum_reversal(&close, 20);

        // 9. RSI (Relative Strength)
        let rsi = MemeIndicators::relative_strength(&close, 14);

        // Normalize
        let ret_norm = Self::robust_norm(&ret);
        let fomo_norm = Self::robust_norm(&fomo);
        let dev_norm = Self::robust_norm(&dev);
        let log_vol_norm = Self::robust_norm(&log_vol);
        let vol_cluster_norm = Self::robust_norm(&vol_cluster);
        // mom_rev is 0/1, no need to normalize? Or robust_norm it to center?
        // It's a binary flag 0.0 or 1.0.
        // Robust norm might make it weird if only 0s exist.
        // Let's keep it raw or map to -1, 1?
        // AlphaGPT usually normalizes everything.
        // But binary features are usually left as is or centered.
        // robustness check: if all 0, median=0, mad=1e-6 -> 0.
        let mom_rev_norm = Self::robust_norm(&mom_rev);

        let rsi_norm = Self::robust_norm(&rsi); // RSI is -1 to 1 centered roughly, but robust norm helps outliers.

        // Stack into Array3 inputs
        // Shape: (batch, features, time)
        let (batch, time) = close.dim();
        let features = 14;

        // 0: Ret
        // 1: Liq
        // 2: Pressure
        // 3: FOMO
        // 4: Dev
        // 5: LogVol
        // 6: VolCluster
        // 7: MomRev
        // 8: RSI
        // 9: lnOpen
        // 10: lnHigh
        // 11: lnLow
        // 12: lnClose
        // 13: lnVolRaw

        let mut out = ndarray::Array3::<f64>::zeros((batch, features, time));

        // Assign slices
        // Axis 1 is feature dimension
        // 0-8: Expert Features (Normalized)
        out.index_axis_mut(Axis(1), 0).assign(&ret_norm);
        out.index_axis_mut(Axis(1), 1).assign(&liq_score);
        out.index_axis_mut(Axis(1), 2).assign(&pressure);
        out.index_axis_mut(Axis(1), 3).assign(&fomo_norm);
        out.index_axis_mut(Axis(1), 4).assign(&dev_norm);
        out.index_axis_mut(Axis(1), 5).assign(&log_vol_norm);
        out.index_axis_mut(Axis(1), 6).assign(&vol_cluster_norm);
        out.index_axis_mut(Axis(1), 7).assign(&mom_rev_norm);
        out.index_axis_mut(Axis(1), 8).assign(&rsi_norm);

        // 9-13: Raw Log Inputs (for VM composition)
        let log_open = open.mapv(|v| (v + 1e-9).ln());
        let log_high = high.mapv(|v| (v + 1e-9).ln());
        let log_low = low.mapv(|v| (v + 1e-9).ln());
        let log_close = close.mapv(|v| (v + 1e-9).ln());
        let log_volume_raw = volume.mapv(|v| (v + 1e-9).ln());

        out.index_axis_mut(Axis(1), 9).assign(&log_open);
        out.index_axis_mut(Axis(1), 10).assign(&log_high);
        out.index_axis_mut(Axis(1), 11).assign(&log_low);
        out.index_axis_mut(Axis(1), 12).assign(&log_close);
        out.index_axis_mut(Axis(1), 13).assign(&log_volume_raw);

        out
    }
}
