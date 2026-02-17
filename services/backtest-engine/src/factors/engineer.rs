use crate::config::{FactorConfig, NormalizationType};
use crate::factors::atr::ATR;
use crate::factors::bollinger::BollingerBands;
use crate::factors::cci::CCI;
use crate::factors::indicators::MemeIndicators;
use crate::factors::macd::MACD;
use crate::factors::mfi::MFI;
use crate::factors::moving_averages::MovingAverages;
use crate::factors::obv::OBV;
use crate::factors::stochastic::Stochastic;
use crate::factors::vwap::VWAP;
use crate::factors::williams_r::WilliamsR;
use crate::vm::ops::ts_delay;
use ndarray::{Array2, Array3, Axis};
use tracing::info;

pub struct FeatureEngineer;

impl FeatureEngineer {
    pub const INPUT_DIM: usize = 6;
    pub const BASIC_DIM: usize = 9;
    pub const EXTENDED_DIM: usize = 33;

    pub fn compute_features_from_config(
        config: &FactorConfig,
        ohlcv: &crate::factors::traits::OhlcvData<'_>,
    ) -> Array3<f64> {
        let (batch, time) = ohlcv.close.dim();
        let n_factors = config.feat_count();
        let mut out = Array3::<f64>::zeros((batch, n_factors, time));

        for (idx, factor_def) in config.active_factors.iter().enumerate() {
            let raw_values = Self::compute_single_factor(&factor_def.name, ohlcv);

            let processed = match factor_def.normalization {
                NormalizationType::Robust => Self::robust_norm(&raw_values),
                NormalizationType::ZScore => Self::zscore_norm(&raw_values),
                NormalizationType::None => raw_values,
            };

            out.index_axis_mut(Axis(1), idx).assign(&processed);
        }

        out
    }

    fn compute_single_factor(name: &str, d: &crate::factors::traits::OhlcvData<'_>) -> Array2<f64> {
        match name {
            // Core factors (shared between crypto and equity)
            "return" => {
                let prev = ts_delay(d.close, 1);
                (d.close / (&prev + 1e-9)).mapv(f64::ln)
            }
            "volume_ratio" => MemeIndicators::volume_ratio(d.volume, 20),
            "momentum" => MemeIndicators::momentum(d.close, 20),
            "relative_strength" => MemeIndicators::relative_strength(d.close, 14),

            // Equity-specific factors
            "vwap_deviation" => {
                MemeIndicators::vwap_deviation(d.close, d.high, d.low, d.volume, 20)
            }
            "mean_reversion" => MemeIndicators::pump_deviation(d.close, 20),
            "adv_ratio" => MemeIndicators::adv_ratio(d.close, d.volume, 20),
            "volatility" => {
                let prev = ts_delay(d.close, 1);
                let ret = (d.close / (&prev + 1e-9)).mapv(f64::ln);
                MemeIndicators::volatility_clustering(&ret, 20)
            }
            "close_position" => MemeIndicators::close_position(d.close, d.high, d.low),
            "intraday_range" => MemeIndicators::intraday_range(d.high, d.low, d.close),

            // Regime factors (for GATE-based conditional strategies)
            "vol_regime" => MemeIndicators::volatility_regime(d.close, 20, 60),
            "trend_strength" => MemeIndicators::trend_strength(d.close, 20),
            "momentum_regime" => MemeIndicators::momentum_regime(d.close, 20),

            // Legacy crypto factors (kept for backward compat with old configs)
            "liquidity_health" => MemeIndicators::liquidity_health(d.liquidity, d.fdv),
            "buy_sell_pressure" => {
                MemeIndicators::buy_sell_imbalance(d.close, d.open, d.high, d.low)
            }
            "fomo_acceleration" => MemeIndicators::fomo_acceleration(d.volume),
            "pump_deviation" => MemeIndicators::pump_deviation(d.close, 20),
            "log_volume" => d.volume.mapv(|v| (v + 1.0).ln()),
            "volatility_clustering" => {
                let prev = ts_delay(d.close, 1);
                let ret = (d.close / (&prev + 1e-9)).mapv(f64::ln);
                MemeIndicators::volatility_clustering(&ret, 20)
            }
            "momentum_reversal" => MemeIndicators::momentum_reversal(d.close, 20),

            _ => {
                info!("Warning: Unknown factor '{}', returning zeros", name);
                Array2::zeros(d.close.dim())
            }
        }
    }

    /// Z-Score normalization (mean=0, std=1)
    pub fn zscore_norm(x: &Array2<f64>) -> Array2<f64> {
        let mut out = x.clone();
        for mut row in out.rows_mut() {
            let mean = row.mean().unwrap_or(0.0);
            let std = row.std(0.0) + 1e-6;
            row.mapv_inplace(|v| (v - mean) / std);
        }
        out
    }

    /// Robust normalization: (x - median) / MAD
    pub fn robust_norm(x: &Array2<f64>) -> Array2<f64> {
        let mut out = x.clone();
        for mut row in out.rows_mut() {
            let mut v: Vec<f64> = row.to_vec();
            v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            let len = v.len();
            if len == 0 {
                continue;
            }

            let median = if len.is_multiple_of(2) {
                (v[len / 2 - 1] + v[len / 2]) / 2.0
            } else {
                v[len / 2]
            };

            let mut diffs: Vec<f64> = row.mapv(|v| (v - median).abs()).to_vec();
            diffs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let mad = if len.is_multiple_of(2) {
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

    /// AlphaGPT-compatible feature computation (6 dimensions)
    /// Matches model_core/factors.py::FeatureEngineer.compute_features exactly
    pub fn compute_features(d: &crate::factors::traits::OhlcvData<'_>) -> Array3<f64> {
        let prev_close = ts_delay(d.close, 1);
        let ret = (d.close / (&prev_close + 1e-9)).mapv(f64::ln);

        // AlphaGPT 6 factors in exact order
        let liq_score = MemeIndicators::liquidity_health(d.liquidity, d.fdv);
        let pressure = MemeIndicators::buy_sell_imbalance(d.close, d.open, d.high, d.low);
        let fomo = MemeIndicators::fomo_acceleration(d.volume);
        let dev = MemeIndicators::pump_deviation(d.close, 20);
        let log_vol = d.volume.mapv(|v| (v + 1.0).ln());

        // Normalize specific factors (matching AlphaGPT)
        let ret_norm = Self::robust_norm(&ret);
        let fomo_norm = Self::robust_norm(&fomo);
        let dev_norm = Self::robust_norm(&dev);
        let log_vol_norm = Self::robust_norm(&log_vol);

        let (batch, time) = d.close.dim();
        let mut out = Array3::<f64>::zeros((batch, Self::INPUT_DIM, time));

        // Stack in AlphaGPT order
        out.index_axis_mut(Axis(1), 0).assign(&ret_norm);
        out.index_axis_mut(Axis(1), 1).assign(&liq_score); // NOT normalized
        out.index_axis_mut(Axis(1), 2).assign(&pressure); // NOT normalized
        out.index_axis_mut(Axis(1), 3).assign(&fomo_norm);
        out.index_axis_mut(Axis(1), 4).assign(&dev_norm);
        out.index_axis_mut(Axis(1), 5).assign(&log_vol_norm);

        out
    }

    /// Compute basic 9-dimensional meme-focused features (backward compatible)
    pub fn compute_basic_features(d: &crate::factors::traits::OhlcvData<'_>) -> Array3<f64> {
        let prev_close = ts_delay(d.close, 1);
        let ret = (d.close / (&prev_close + 1e-9)).mapv(f64::ln);

        let liq_score = MemeIndicators::liquidity_health(d.liquidity, d.fdv);
        let pressure = MemeIndicators::buy_sell_imbalance(d.close, d.open, d.high, d.low);
        let fomo = MemeIndicators::fomo_acceleration(d.volume);
        let dev = MemeIndicators::pump_deviation(d.close, 20);
        let log_vol = d.volume.mapv(|v| (v + 1.0).ln());
        let vol_cluster = MemeIndicators::volatility_clustering(&ret, 20);
        let mom_rev = MemeIndicators::momentum_reversal(d.close, 20);
        let rsi = MemeIndicators::relative_strength(d.close, 14);

        let ret_norm = Self::robust_norm(&ret);
        let fomo_norm = Self::robust_norm(&fomo);
        let dev_norm = Self::robust_norm(&dev);
        let log_vol_norm = Self::robust_norm(&log_vol);
        let vol_cluster_norm = Self::robust_norm(&vol_cluster);
        let mom_rev_norm = Self::robust_norm(&mom_rev);
        let rsi_norm = Self::robust_norm(&rsi);

        let (batch, time) = d.close.dim();
        let mut out = Array3::<f64>::zeros((batch, Self::BASIC_DIM, time));

        out.index_axis_mut(Axis(1), 0).assign(&ret_norm);
        out.index_axis_mut(Axis(1), 1).assign(&liq_score);
        out.index_axis_mut(Axis(1), 2).assign(&pressure);
        out.index_axis_mut(Axis(1), 3).assign(&fomo_norm);
        out.index_axis_mut(Axis(1), 4).assign(&dev_norm);
        out.index_axis_mut(Axis(1), 5).assign(&log_vol_norm);
        out.index_axis_mut(Axis(1), 6).assign(&vol_cluster_norm);
        out.index_axis_mut(Axis(1), 7).assign(&mom_rev_norm);
        out.index_axis_mut(Axis(1), 8).assign(&rsi_norm);

        out
    }

    /// Compute extended 33-dimensional feature set with ALL Tier 1-3 indicators
    pub fn compute_extended_features(
        close: &Array2<f64>,
        open: &Array2<f64>,
        high: &Array2<f64>,
        low: &Array2<f64>,
        volume: &Array2<f64>,
        liquidity: &Array2<f64>,
        fdv: &Array2<f64>,
    ) -> Array3<f64> {
        // Meme indicators (9)
        let prev_close = ts_delay(close, 1);
        let ret = (close / (&prev_close + 1e-9)).mapv(f64::ln);
        let liq_score = MemeIndicators::liquidity_health(liquidity, fdv);
        let pressure = MemeIndicators::buy_sell_imbalance(close, open, high, low);
        let fomo = MemeIndicators::fomo_acceleration(volume);
        let dev = MemeIndicators::pump_deviation(close, 20);
        let log_vol = volume.mapv(|v| (v + 1.0).ln());
        let vol_cluster = MemeIndicators::volatility_clustering(&ret, 20);
        let mom_rev = MemeIndicators::momentum_reversal(close, 20);
        let rsi = MemeIndicators::relative_strength(close, 14);

        // Moving averages (4)
        let ema_12_diff = (close - &MovingAverages::ema(close, 12)) / (close + 1e-9);
        let ema_26_diff = (close - &MovingAverages::ema(close, 26)) / (close + 1e-9);
        let ema_50_diff = (close - &MovingAverages::ema(close, 50)) / (close + 1e-9);
        let sma_200_diff = (close - &MovingAverages::sma(close, 200)) / (close + 1e-9);

        // MACD (3)
        let (macd_line, macd_signal, macd_hist) = MACD::macd(close);
        let macd_line_norm = &macd_line / (close + 1e-9);
        let macd_signal_norm = &macd_signal / (close + 1e-9);
        let macd_hist_norm = &macd_hist / (close + 1e-9);

        // Bollinger Bands (3)
        let bb_bandwidth = BollingerBands::bandwidth(close, 20);
        let bb_percent_b = BollingerBands::percent_b(close, 20);
        let (_, bb_middle, _) = BollingerBands::bollinger(close);
        let bb_position = (close - &bb_middle) / (&bb_middle + 1e-9);

        // ATR (1)
        let atr_pct = ATR::atr_percent(high, low, close);

        // Stochastic (2)
        let (stoch_k, stoch_d) = Stochastic::stochastic(high, low, close);
        let stoch_k_norm = (&stoch_k - 50.0) / 50.0;
        let stoch_d_norm = (&stoch_d - 50.0) / 50.0;

        // CCI (1)
        let cci_norm = CCI::cci_normalized(high, low, close, 20);

        // Williams %R (1)
        let williams_norm = WilliamsR::williams_r_normalized(high, low, close, 14);

        // VWAP (2)
        let vwap_dev = VWAP::vwap_deviation(high, low, close, volume);
        let vwap_roll = VWAP::vwap_rolling(high, low, close, volume, 20);
        let vwap_roll_dev = (close - &vwap_roll) / (&vwap_roll + 1e-9);

        // OBV (1)
        let obv_pct = OBV::obv_pct_change(close, volume);

        // MFI (1)
        let mfi_norm = MFI::mfi_normalized(high, low, close, volume, 14);

        // Additional (5)
        let hl_range = (high - low) / (close + 1e-9);
        let close_pos = (close - low) / (high - low + 1e-9);
        let vol_trend = (volume - &ts_delay(volume, 1)) / (&ts_delay(volume, 1) + 1.0);
        let momentum_10 = (close - &ts_delay(close, 10)) / (&ts_delay(close, 10) + 1e-9);
        let momentum_20 = (close - &ts_delay(close, 20)) / (&ts_delay(close, 20) + 1e-9);

        // Stack all 33 features
        let (batch, time) = close.dim();
        let mut features = Array3::<f64>::zeros((batch, Self::EXTENDED_DIM, time));

        let mut idx = 0;
        features
            .index_axis_mut(Axis(1), idx)
            .assign(&Self::robust_norm(&ret));
        idx += 1;
        features.index_axis_mut(Axis(1), idx).assign(&liq_score);
        idx += 1;
        features.index_axis_mut(Axis(1), idx).assign(&pressure);
        idx += 1;
        features
            .index_axis_mut(Axis(1), idx)
            .assign(&Self::robust_norm(&fomo));
        idx += 1;
        features
            .index_axis_mut(Axis(1), idx)
            .assign(&Self::robust_norm(&dev));
        idx += 1;
        features
            .index_axis_mut(Axis(1), idx)
            .assign(&Self::robust_norm(&log_vol));
        idx += 1;
        features
            .index_axis_mut(Axis(1), idx)
            .assign(&Self::robust_norm(&vol_cluster));
        idx += 1;
        features
            .index_axis_mut(Axis(1), idx)
            .assign(&Self::robust_norm(&mom_rev));
        idx += 1;
        features
            .index_axis_mut(Axis(1), idx)
            .assign(&Self::robust_norm(&rsi));
        idx += 1;
        features
            .index_axis_mut(Axis(1), idx)
            .assign(&Self::robust_norm(&ema_12_diff));
        idx += 1;
        features
            .index_axis_mut(Axis(1), idx)
            .assign(&Self::robust_norm(&ema_26_diff));
        idx += 1;
        features
            .index_axis_mut(Axis(1), idx)
            .assign(&Self::robust_norm(&ema_50_diff));
        idx += 1;
        features
            .index_axis_mut(Axis(1), idx)
            .assign(&Self::robust_norm(&sma_200_diff));
        idx += 1;
        features
            .index_axis_mut(Axis(1), idx)
            .assign(&Self::robust_norm(&macd_line_norm));
        idx += 1;
        features
            .index_axis_mut(Axis(1), idx)
            .assign(&Self::robust_norm(&macd_signal_norm));
        idx += 1;
        features
            .index_axis_mut(Axis(1), idx)
            .assign(&Self::robust_norm(&macd_hist_norm));
        idx += 1;
        features
            .index_axis_mut(Axis(1), idx)
            .assign(&Self::robust_norm(&bb_bandwidth));
        idx += 1;
        features
            .index_axis_mut(Axis(1), idx)
            .assign(&bb_percent_b.mapv(|v| v.clamp(0.0, 1.0)));
        idx += 1;
        features
            .index_axis_mut(Axis(1), idx)
            .assign(&Self::robust_norm(&bb_position));
        idx += 1;
        features
            .index_axis_mut(Axis(1), idx)
            .assign(&Self::robust_norm(&atr_pct));
        idx += 1;
        features.index_axis_mut(Axis(1), idx).assign(&stoch_k_norm);
        idx += 1;
        features.index_axis_mut(Axis(1), idx).assign(&stoch_d_norm);
        idx += 1;
        features.index_axis_mut(Axis(1), idx).assign(&cci_norm);
        idx += 1;
        features.index_axis_mut(Axis(1), idx).assign(&williams_norm);
        idx += 1;
        features
            .index_axis_mut(Axis(1), idx)
            .assign(&Self::robust_norm(&vwap_dev));
        idx += 1;
        features
            .index_axis_mut(Axis(1), idx)
            .assign(&Self::robust_norm(&vwap_roll_dev));
        idx += 1;
        features
            .index_axis_mut(Axis(1), idx)
            .assign(&Self::robust_norm(&obv_pct));
        idx += 1;
        features.index_axis_mut(Axis(1), idx).assign(&mfi_norm);
        idx += 1;
        features
            .index_axis_mut(Axis(1), idx)
            .assign(&Self::robust_norm(&hl_range));
        idx += 1;
        features
            .index_axis_mut(Axis(1), idx)
            .assign(&close_pos.mapv(|v| v.clamp(0.0, 1.0)));
        idx += 1;
        features
            .index_axis_mut(Axis(1), idx)
            .assign(&Self::robust_norm(&vol_trend));
        idx += 1;
        features
            .index_axis_mut(Axis(1), idx)
            .assign(&Self::robust_norm(&momentum_10));
        idx += 1;
        features
            .index_axis_mut(Axis(1), idx)
            .assign(&Self::robust_norm(&momentum_20));

        features
    }
}
