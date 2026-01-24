use backtest_engine::factors::engineer::FeatureEngineer;
use chrono::{DateTime, Utc};
use common::events::MarketDataUpdate;
use ndarray::{Array2, Array3};
use std::collections::HashMap;

// Constants
const WINDOW_SIZE: usize = 1000; // Keep enough history for long windows

#[derive(Debug, Clone)]
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

    fn to_arrays(
        &self,
    ) -> (
        Array2<f64>,
        Array2<f64>,
        Array2<f64>,
        Array2<f64>,
        Array2<f64>,
        Array2<f64>,
        Array2<f64>,
    ) {
        // Convert Vec<f64> to Array2<f64> of shape (1, T) - Single batch, T time steps
        // Actually FeatureEngineer expects (batch, time) per asset?
        // No, compute_features takes &Array2<f64> which is usually (batch, time).
        // Since we are processing 1 symbol, batch=1.

        let t = self.close.len();
        let shape = (1, t);

        (
            Array2::from_shape_vec(shape, self.close.clone()).unwrap(),
            Array2::from_shape_vec(shape, self.open.clone()).unwrap(),
            Array2::from_shape_vec(shape, self.high.clone()).unwrap(),
            Array2::from_shape_vec(shape, self.low.clone()).unwrap(),
            Array2::from_shape_vec(shape, self.volume.clone()).unwrap(),
            Array2::from_shape_vec(shape, self.liquidity.clone()).unwrap(),
            Array2::from_shape_vec(shape, self.fdv.clone()).unwrap(),
        )
    }
}

pub struct MarketDataManager {
    buffers: HashMap<String, SymbolBuffer>,
}

impl MarketDataManager {
    pub fn new() -> Self {
        Self {
            buffers: HashMap::new(),
        }
    }

    pub fn on_update(&mut self, update: MarketDataUpdate) -> Option<Array3<f64>> {
        let symbol = update.symbol.clone();

        let buffer = self
            .buffers
            .entry(symbol.clone())
            .or_insert_with(|| SymbolBuffer::new(symbol));
        buffer.push(&update);

        // Require minimum history to run VM? e.g. 20 for moving averages
        // For Verification/Demo: Reduce to 2 to see logs immediately
        if buffer.close.len() < 2 {
            return None;
        }

        // Generate Features
        let (c, o, h, l, v, liq, fdv) = buffer.to_arrays();

        let features = FeatureEngineer::compute_features(&c, &o, &h, &l, &v, &liq, &fdv);
        Some(features)
    }
}
