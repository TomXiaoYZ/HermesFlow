use backtest_engine::config::FactorConfig;
use backtest_engine::factors::engineer::FeatureEngineer;
use backtest_engine::factors::traits::OhlcvArrays;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use common::events::MarketDataUpdate;
use ndarray::{Array2, Array3, Axis};
use std::collections::HashMap;
use tracing::debug;

// Constants
const WINDOW_SIZE: usize = 1000; // Keep enough history for long windows
const DEFAULT_BAR_SECONDS: i64 = 3600; // 1-hour bars by default

/// P10F-1: OHLCV bar aggregator.
///
/// Accumulates incoming ticks into time-bounded OHLCV bars.
/// When the current bar's time window expires, it is emitted and a new bar starts.
#[derive(Debug, Clone)]
struct BarAggregator {
    /// Bar duration.
    bar_duration: ChronoDuration,
    /// Current bar being built (None if no ticks received yet).
    current_bar: Option<PartialBar>,
}

/// A bar being accumulated from ticks.
#[derive(Debug, Clone)]
struct PartialBar {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    bar_start: DateTime<Utc>,
}

/// A completed OHLCV bar ready for feature computation.
#[derive(Debug, Clone)]
struct CompletedBar {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    timestamp: DateTime<Utc>,
}

impl BarAggregator {
    fn new(bar_seconds: i64) -> Self {
        Self {
            bar_duration: ChronoDuration::seconds(bar_seconds),
            current_bar: None,
        }
    }

    /// Align a timestamp to bar boundary (floor to bar_duration interval).
    fn bar_start_for(&self, ts: DateTime<Utc>) -> DateTime<Utc> {
        let secs = self.bar_duration.num_seconds();
        if secs <= 0 {
            return ts;
        }
        let epoch_secs = ts.timestamp();
        let aligned = epoch_secs - (epoch_secs % secs);
        DateTime::from_timestamp(aligned, 0).unwrap_or(ts)
    }

    /// Process a tick. Returns a completed bar if the current bar's window has expired.
    fn on_tick(&mut self, price: f64, volume: f64, ts: DateTime<Utc>) -> Option<CompletedBar> {
        let tick_bar_start = self.bar_start_for(ts);
        let mut completed = None;

        if let Some(ref bar) = self.current_bar {
            // If this tick belongs to a new bar, emit the current bar first
            if tick_bar_start > bar.bar_start {
                completed = Some(CompletedBar {
                    open: bar.open,
                    high: bar.high,
                    low: bar.low,
                    close: bar.close,
                    volume: bar.volume,
                    timestamp: bar.bar_start + self.bar_duration,
                });
                // Start new bar
                self.current_bar = Some(PartialBar {
                    open: price,
                    high: price,
                    low: price,
                    close: price,
                    volume,
                    bar_start: tick_bar_start,
                });
            } else {
                // Same bar — update OHLCV
                let bar = self.current_bar.as_mut().unwrap();
                bar.high = bar.high.max(price);
                bar.low = bar.low.min(price);
                bar.close = price;
                bar.volume += volume;
            }
        } else {
            // First tick ever — start first bar
            self.current_bar = Some(PartialBar {
                open: price,
                high: price,
                low: price,
                close: price,
                volume,
                bar_start: tick_bar_start,
            });
        }

        completed
    }
}

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
    /// P10F-1: Bar aggregator for converting ticks to OHLCV bars.
    aggregator: BarAggregator,
}

impl SymbolBuffer {
    fn new(symbol: String, bar_seconds: i64) -> Self {
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
            aggregator: BarAggregator::new(bar_seconds),
        }
    }

    /// Push a tick through the bar aggregator. Returns true if a new bar was completed.
    fn push(&mut self, update: &MarketDataUpdate) -> bool {
        if let Some(bar) = self
            .aggregator
            .on_tick(update.price, update.volume, update.timestamp)
        {
            // Maintain window size
            if self.close.len() >= WINDOW_SIZE {
                self.remove_first();
            }

            debug!(
                "{}: bar completed O={:.2} H={:.2} L={:.2} C={:.2} V={:.0}",
                self.symbol, bar.open, bar.high, bar.low, bar.close, bar.volume
            );

            self.open.push(bar.open);
            self.high.push(bar.high);
            self.low.push(bar.low);
            self.close.push(bar.close);
            self.volume.push(bar.volume);
            self.liquidity.push(0.0);
            self.fdv.push(0.0);
            self.timestamps.push(bar.timestamp);
            true
        } else {
            false
        }
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
    /// Bar duration in seconds for OHLCV aggregation.
    bar_seconds: i64,
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
            bar_seconds: DEFAULT_BAR_SECONDS,
        }
    }

    pub fn with_factor_config(factor_config: FactorConfig, n_resolutions: usize) -> Self {
        Self {
            buffers: HashMap::new(),
            factor_config: Some(factor_config),
            n_resolutions: n_resolutions.max(1),
            bar_seconds: DEFAULT_BAR_SECONDS,
        }
    }

    /// Set bar aggregation interval (e.g., 3600 for 1h, 300 for 5m).
    pub fn with_bar_seconds(mut self, seconds: i64) -> Self {
        self.bar_seconds = seconds;
        self
    }

    pub fn on_update(&mut self, update: MarketDataUpdate) -> Option<Array3<f64>> {
        let symbol = update.symbol.clone();

        let bar_secs = self.bar_seconds;
        let buffer = self
            .buffers
            .entry(symbol.clone())
            .or_insert_with(|| SymbolBuffer::new(symbol, bar_secs));

        // P10F-1: Only compute features when a new bar completes
        if !buffer.push(&update) {
            return None; // Still accumulating ticks within current bar
        }

        // Require minimum history for moving averages (e.g. 20-period factors)
        if buffer.close.len() < 2 {
            return None;
        }

        // Generate Features
        let arrays = buffer.to_arrays()?;

        let base_features = match &self.factor_config {
            Some(config) => FeatureEngineer::compute_features_from_config(config, &arrays.as_ref()),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tick(price: f64, volume: f64, ts_secs: i64) -> MarketDataUpdate {
        MarketDataUpdate {
            symbol: "AAPL".to_string(),
            price,
            volume,
            timestamp: DateTime::from_timestamp(ts_secs, 0).unwrap(),
            source: "test".to_string(),
        }
    }

    #[test]
    fn bar_aggregator_accumulates_within_window() {
        let mut agg = BarAggregator::new(60); // 1-minute bars
                                              // All ticks within same minute
        assert!(agg
            .on_tick(100.0, 10.0, DateTime::from_timestamp(0, 0).unwrap())
            .is_none());
        assert!(agg
            .on_tick(105.0, 5.0, DateTime::from_timestamp(30, 0).unwrap())
            .is_none());
        assert!(agg
            .on_tick(98.0, 8.0, DateTime::from_timestamp(59, 0).unwrap())
            .is_none());
    }

    #[test]
    fn bar_aggregator_emits_on_new_bar() {
        let mut agg = BarAggregator::new(60); // 1-minute bars
                                              // First tick: opens bar at t=0
        assert!(agg
            .on_tick(100.0, 10.0, DateTime::from_timestamp(0, 0).unwrap())
            .is_none());
        // More ticks in same bar
        assert!(agg
            .on_tick(105.0, 5.0, DateTime::from_timestamp(30, 0).unwrap())
            .is_none());
        assert!(agg
            .on_tick(98.0, 8.0, DateTime::from_timestamp(45, 0).unwrap())
            .is_none());

        // Tick in next minute → emits completed bar
        let bar = agg.on_tick(102.0, 3.0, DateTime::from_timestamp(60, 0).unwrap());
        assert!(bar.is_some());

        let bar = bar.unwrap();
        assert!(
            (bar.open - 100.0).abs() < 1e-10,
            "open should be first tick price"
        );
        assert!((bar.high - 105.0).abs() < 1e-10, "high should be max");
        assert!((bar.low - 98.0).abs() < 1e-10, "low should be min");
        assert!(
            (bar.close - 98.0).abs() < 1e-10,
            "close should be last tick"
        );
        assert!(
            (bar.volume - 23.0).abs() < 1e-10,
            "volume should be sum: 10+5+8"
        );
    }

    #[test]
    fn bar_aggregator_multiple_bars() {
        let mut agg = BarAggregator::new(60);
        agg.on_tick(100.0, 1.0, DateTime::from_timestamp(0, 0).unwrap());
        let b1 = agg.on_tick(200.0, 2.0, DateTime::from_timestamp(60, 0).unwrap());
        assert!(b1.is_some());

        let b2 = agg.on_tick(300.0, 3.0, DateTime::from_timestamp(120, 0).unwrap());
        assert!(b2.is_some());
        let b2 = b2.unwrap();
        assert!((b2.open - 200.0).abs() < 1e-10);
        assert!((b2.close - 200.0).abs() < 1e-10);
    }

    #[test]
    fn symbol_buffer_push_returns_true_on_bar_complete() {
        let mut buf = SymbolBuffer::new("AAPL".to_string(), 60);
        let t1 = make_tick(100.0, 10.0, 0);
        let t2 = make_tick(105.0, 5.0, 30);
        let t3 = make_tick(102.0, 3.0, 60);

        assert!(!buf.push(&t1), "first tick should not complete bar");
        assert!(!buf.push(&t2), "second tick same bar");
        assert!(
            buf.push(&t3),
            "third tick in next bar should complete first bar"
        );
        assert_eq!(buf.close.len(), 1);
        assert!((buf.high[0] - 105.0).abs() < 1e-10);
        assert!((buf.low[0] - 100.0).abs() < 1e-10);
    }

    #[test]
    fn bar_aggregator_bar_alignment() {
        let agg = BarAggregator::new(3600); // 1-hour bars
                                            // Timestamp 5400 = 1.5 hours since epoch → bar starts at 3600 (1 hour)
        let ts = DateTime::from_timestamp(5400, 0).unwrap();
        let bar_start = agg.bar_start_for(ts);
        assert_eq!(bar_start, DateTime::from_timestamp(3600, 0).unwrap());
    }
}
