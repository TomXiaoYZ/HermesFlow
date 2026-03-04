use backtest_engine::config::FactorConfig;
use backtest_engine::factors::engineer::FeatureEngineer;
use backtest_engine::factors::traits::OhlcvArrays;
use chrono::{DateTime, Utc};
use common::events::MarketDataUpdate;
use ndarray::{Array2, Array3, Axis};
use std::collections::HashMap;

// Constants
const WINDOW_SIZE: usize = 1000; // Keep enough history for long windows

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct SymbolBuffer {
    symbol: String,
    // Columns (using Vec for easy append, convert to Array2 for feature engineering)
    close: Vec<f64>,
    open: Vec<f64>,
    high: Vec<f64>,
    low: Vec<f64>,
    volume: Vec<f64>,
    liquidity: Vec<f64>,
    fdv: Vec<f64>,
    timestamps: Vec<DateTime<Utc>>,
}

impl SymbolBuffer {
    fn new(symbol: String) -> Self {
        Self {
            symbol,
            close: Vec::with_capacity(WINDOW_SIZE),
            open: Vec::with_capacity(WINDOW_SIZE),
            high: Vec::with_capacity(WINDOW_SIZE),
            low: Vec::with_capacity(WINDOW_SIZE),
            volume: Vec::with_capacity(WINDOW_SIZE),
            liquidity: Vec::with_capacity(WINDOW_SIZE),
            fdv: Vec::with_capacity(WINDOW_SIZE),
            timestamps: Vec::with_capacity(WINDOW_SIZE),
        }
    }

    fn push(&mut self, update: &MarketDataUpdate) {
        // Maintain window size
        if self.close.len() >= WINDOW_SIZE {
            self.remove_first();
        }

        self.close.push(update.price);
        // Approximation: For minimal updates, if we don't have OHLC, assume C=O=H=L
        // But MarketDataUpdate is a single tick.
        // Real system should aggregate candles.
        // For Phase 5 "Live Update" demo, we can treat each update as a "step" or "candle"
        // to drive the VM, or we need a proper candle aggregator.
        // Let's assume the Update IS the candle Close, and we just use price for everything if other fields missing.
        // But update has NO open/high/low.

        // CRITICAL GAP: Redis sends Ticks (price), VM needs OHLCV.
        // Solution for MVP: Use Price for Open/High/Low/Close.
        // Or if volume provided use it.

        self.open.push(update.price);
        self.high.push(update.price);
        self.low.push(update.price);
        self.volume.push(update.volume);

        // Metadata might be missing in Update, assume 0 or last known?
        // Update struct: price, volume, source, timestamp. No liquidity/fdv.
        // We'll insert 0.0 or defaults for now.
        self.liquidity.push(0.0);
        self.fdv.push(0.0);

        self.timestamps.push(update.timestamp);
    }

    fn remove_first(&mut self) {
        if !self.close.is_empty() {
            self.close.remove(0);
            self.open.remove(0);
            self.high.remove(0);
            self.low.remove(0);
            self.volume.remove(0);
            self.liquidity.remove(0);
            self.fdv.remove(0);
            self.timestamps.remove(0);
        }
    }

    fn to_arrays(&self) -> Option<OhlcvArrays> {
        let t = self.close.len();
        if t == 0 {
            return None;
        }
        let shape = (1, t);

        Some(OhlcvArrays {
            close: Array2::from_shape_vec(shape, self.close.clone()).ok()?,
            open: Array2::from_shape_vec(shape, self.open.clone()).ok()?,
            high: Array2::from_shape_vec(shape, self.high.clone()).ok()?,
            low: Array2::from_shape_vec(shape, self.low.clone()).ok()?,
            volume: Array2::from_shape_vec(shape, self.volume.clone()).ok()?,
            liquidity: Array2::from_shape_vec(shape, self.liquidity.clone()).ok()?,
            fdv: Array2::from_shape_vec(shape, self.fdv.clone()).ok()?,
        })
    }
}

pub struct MarketDataManager {
    buffers: HashMap<String, SymbolBuffer>,
    factor_config: Option<FactorConfig>,
    /// Number of timeframe resolutions for multi-timeframe stacking.
    /// When > 1, the base features are replicated to fill all resolution slots.
    n_resolutions: usize,
}

impl Default for MarketDataManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MarketDataManager {
    pub fn new() -> Self {
        Self {
            buffers: HashMap::new(),
            factor_config: None,
            n_resolutions: 1,
        }
    }

    pub fn with_factor_config(factor_config: FactorConfig, n_resolutions: usize) -> Self {
        Self {
            buffers: HashMap::new(),
            factor_config: Some(factor_config),
            n_resolutions: n_resolutions.max(1),
        }
    }

    pub fn on_update(&mut self, update: MarketDataUpdate) -> Option<Array3<f64>> {
        let symbol = update.symbol.clone();

        let buffer = self
            .buffers
            .entry(symbol.clone())
            .or_insert_with(|| SymbolBuffer::new(symbol));
        buffer.push(&update);

        // Require minimum history for moving averages (e.g. 20-period factors)
        if buffer.close.len() < 2 {
            return None;
        }

        // Generate Features
        let arrays = buffer.to_arrays()?;

        let base_features = match &self.factor_config {
            Some(config) => {
                FeatureEngineer::compute_features_from_config(config, &arrays.as_ref())
            }
            None => FeatureEngineer::compute_features(&arrays.as_ref()),
        };

        // Multi-timeframe stacking: replicate base features across resolution slots.
        // In production, each slot would have its own candle aggregation (1h/4h/1d).
        // For tick-driven live trading, we replicate the base features as a proxy.
        if self.n_resolutions > 1 {
            let views: Vec<_> = (0..self.n_resolutions)
                .map(|_| base_features.view())
                .collect();
            Some(ndarray::concatenate(Axis(1), &views).ok()?)
        } else {
            Some(base_features)
        }
    }
}
