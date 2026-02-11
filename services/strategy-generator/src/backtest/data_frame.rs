use crate::backtest::CachedData;
use ndarray::Axis;
use std::collections::{BTreeSet, HashMap};
use tracing::info;

/// Struct to hold time-aligned market data for portfolio simulation
pub struct TimeDataFrame {
    /// Global sorted timestamps (Unix seconds)
    pub timestamps: Vec<i64>,
    /// Map of Symbol -> Aligned Returns (matches timestamps length)
    pub returns: HashMap<String, Vec<f64>>,
    /// Map of Symbol -> Aligned Amounts (Quote Volume) (matches timestamps length)
    pub amounts: HashMap<String, Vec<f64>>,
    /// Map of Symbol -> Aligned Liquidity (matches timestamps length)
    pub liquidity: HashMap<String, Vec<f64>>,
}

impl TimeDataFrame {
    pub fn new() -> Self {
        Self {
            timestamps: Vec::new(),
            returns: HashMap::new(),
            amounts: HashMap::new(),
            liquidity: HashMap::new(),
        }
    }

    /// Align multiple CachedData sets found in cache.
    /// Note: CachedData currently stores ndarray::Array2 (matrix) without timestamps.
    /// We need SOURCE timestamps from `Candle` objects?
    /// PROBLEM: `CachedData` loses timestamps!
    /// We must fix `CachedData` first to include `timestamps`.
    ///
    /// Assuming `CachedData` refactor is done and has `pub timestamps: Vec<i64>`.
    pub fn align(data_map: &HashMap<String, CachedData>) -> Self {
        if data_map.is_empty() {
            return Self::new();
        }

        // 1. Collect all unique timestamps union
        let mut global_timestamps = BTreeSet::new();
        for data in data_map.values() {
            for &ts in &data.timestamps {
                global_timestamps.insert(ts);
            }
        }

        // Convert to Vec
        let timestamps: Vec<i64> = global_timestamps.into_iter().collect();
        let len = timestamps.len();

        info!(
            "Aligning data for {} symbols across {} timestamps",
            data_map.len(),
            len
        );

        let mut aligned_returns = HashMap::new();
        let mut aligned_amounts = HashMap::new();
        let mut aligned_liquidity = HashMap::new();

        // 2. Reindex each symbol
        for (symbol, data) in data_map {
            let mut ret_vec = vec![0.0; len];
            let mut amt_vec = vec![0.0; len];
            let mut liq_vec = vec![0.0; len];

            // Create lookup for this symbol data
            // data.timestamps maps index -> time
            // data.returns[[0, index]] -> value
            let mut internal_map: HashMap<i64, usize> = HashMap::new();
            for (idx, &ts) in data.timestamps.iter().enumerate() {
                internal_map.insert(ts, idx);
            }

            // Fill aligned vectors
            for (i, &ts) in timestamps.iter().enumerate() {
                if let Some(&idx) = internal_map.get(&ts) {
                    // Start Index check
                    if idx < data.returns.len_of(Axis(1)) {
                        ret_vec[i] = data.returns[[0, idx]];
                        amt_vec[i] = data.amount[[0, idx]];
                        liq_vec[i] = data.liquidity[[0, idx]];
                    }
                } else {
                    // Missing Data logic (Fill 0.0 for Return, 0.0 for Amt)
                    // This is correct for Backtesting (No position if no data).
                    ret_vec[i] = 0.0;
                    amt_vec[i] = 0.0;
                    liq_vec[i] = 0.0;
                }
            }

            aligned_returns.insert(symbol.clone(), ret_vec);
            aligned_amounts.insert(symbol.clone(), amt_vec);
            aligned_liquidity.insert(symbol.clone(), liq_vec);
        }

        Self {
            timestamps,
            returns: aligned_returns,
            amounts: aligned_amounts,
            liquidity: aligned_liquidity,
        }
    }
}
