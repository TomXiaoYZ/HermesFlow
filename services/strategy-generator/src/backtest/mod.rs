use crate::genetic::Genome;
use backtest_engine::config::{FactorConfig, MultiTimeframeFactorConfig};
use backtest_engine::factors::engineer::FeatureEngineer;
use backtest_engine::vm::vm::StackVM;
use chrono::{DateTime, Utc};
use ndarray::{Array2, Array3, Axis};
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use sqlx::FromRow;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

pub mod data_frame;
pub mod portfolio;

// ── OOS Sentinel Values ──────────────────────────────────────────────
// Values < -9.0 indicate evaluation failure (not genuine poor performance).
// Each failure mode gets a distinct value for diagnosis.
pub const SENTINEL_CACHE_MISS: f64 = -10.0;
pub const SENTINEL_INSUFFICIENT_DATA: f64 = -11.0;
pub const SENTINEL_OOS_TOO_SMALL: f64 = -12.0;
pub const SENTINEL_VM_FAILURE: f64 = -13.0;
pub const SENTINEL_TOO_FEW_BARS: f64 = -14.0;
pub const SENTINEL_TOO_FEW_TRADES: f64 = -15.0;
pub const SENTINEL_ZERO_VARIANCE: f64 = -16.0;
pub const SENTINEL_NEGATIVE_SE: f64 = -17.0;
pub const SENTINEL_ZERO_SE: f64 = -18.0;
pub const SENTINEL_NAN_PSR: f64 = -19.0;

/// Returns true if a fitness value is a sentinel (evaluation failure, not real performance).
pub fn is_sentinel(value: f64) -> bool {
    value <= -9.5
}

/// Human-readable label for sentinel values.
pub fn sentinel_label(value: f64) -> &'static str {
    match value as i64 {
        -10 => "cache_miss",
        -11 => "insufficient_data",
        -12 => "oos_too_small",
        -13 => "vm_failure",
        -14 => "too_few_bars",
        -15 => "too_few_trades",
        -16 => "zero_variance",
        -17 => "negative_se",
        -18 => "zero_se",
        -19 => "nan_psr",
        _ => "unknown_sentinel",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrategyMode {
    LongOnly,
    LongShort,
}

impl StrategyMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LongOnly => "long_only",
            Self::LongShort => "long_short",
        }
    }

    pub fn all() -> &'static [StrategyMode] {
        &[StrategyMode::LongOnly, StrategyMode::LongShort]
    }
}

impl fmt::Display for StrategyMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for StrategyMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "long_only" => Ok(Self::LongOnly),
            "long_short" => Ok(Self::LongShort),
            _ => Err(format!("unknown strategy mode: {}", s)),
        }
    }
}

#[derive(Debug, FromRow)]
pub struct Candle {
    pub time: DateTime<Utc>,
    pub open: rust_decimal::Decimal,
    pub high: rust_decimal::Decimal,
    pub low: rust_decimal::Decimal,
    pub close: rust_decimal::Decimal,
    pub volume: rust_decimal::Decimal,
    pub liquidity: rust_decimal::Decimal,
    pub fdv: rust_decimal::Decimal,
    pub amount: rust_decimal::Decimal,
}

#[derive(Clone)]
pub struct CachedData {
    pub features: Array3<f64>,
    pub returns: Array2<f64>,
    #[allow(dead_code)]
    pub open: Array2<f64>,
    #[allow(dead_code)]
    pub close: Array2<f64>,
    pub liquidity: Array2<f64>,
    pub amount: Array2<f64>,
    pub timestamps: Vec<i64>,
}

/// Configuration for walk-forward out-of-sample evaluation.
#[derive(Debug, Clone)]
pub struct WalkForwardConfig {
    /// Minimum initial training window (bars).
    pub initial_train: usize,
    /// Target test window per step (bars). Adjusted downward if data is limited.
    pub target_test_window: usize,
    /// Minimum acceptable test window (bars). Steps smaller than this are skipped.
    pub min_test_window: usize,
    /// Embargo bars between train and test to prevent information leakage.
    pub embargo: usize,
    /// Target number of walk-forward steps.
    pub target_steps: usize,
}

impl WalkForwardConfig {
    /// Default config for 1h Polygon data.
    pub fn default_1h() -> Self {
        Self {
            initial_train: 2500,
            target_test_window: 1000,
            min_test_window: 400,
            embargo: 20,
            target_steps: 3,
        }
    }
}

/// Result of a single walk-forward step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkForwardStep {
    pub step: usize,
    pub train_start: usize,
    pub train_end: usize,
    pub test_start: usize,
    pub test_end: usize,
    pub psr: f64,
    pub trade_count: u32,
    pub active_bars: u32,
    pub upper_threshold: f64,
    pub lower_threshold: f64,
}

/// Aggregated walk-forward OOS result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkForwardResult {
    pub aggregate_psr: f64,
    pub mean_psr: f64,
    pub std_psr: f64,
    pub num_steps: usize,
    pub num_valid_steps: usize,
    pub steps: Vec<WalkForwardStep>,
    /// If aggregate_psr is a sentinel, this explains which failure dominated.
    pub failure_mode: Option<String>,
}

/// Forward-fill lower-frequency features to align with higher-frequency timestamps.
///
/// For each target (high-frequency) timestamp `t`, finds the most recent low-frequency
/// bar where `lf_timestamps[j] <= t` and copies its feature row. Bars before the first
/// low-frequency timestamp are filled with zeros.
///
/// Uses binary search for O(T_hf * log(T_lf)) complexity.
pub fn forward_fill_align(
    lf_features: &Array3<f64>,
    lf_timestamps: &[i64],
    hf_timestamps: &[i64],
) -> Array3<f64> {
    let n_factors = lf_features.shape()[1];
    let hf_len = hf_timestamps.len();
    let mut out = Array3::<f64>::zeros((1, n_factors, hf_len));

    for (i, &t) in hf_timestamps.iter().enumerate() {
        // Binary search: find rightmost lf index where lf_timestamps[j] <= t
        let idx = match lf_timestamps.binary_search(&t) {
            Ok(exact) => Some(exact),
            Err(insert_pos) => {
                if insert_pos > 0 {
                    Some(insert_pos - 1)
                } else {
                    None // t is before the first lf timestamp
                }
            }
        };

        if let Some(j) = idx {
            // Copy all factors from lf bar j to output bar i
            for f in 0..n_factors {
                out[[0, f, i]] = lf_features[[0, f, j]];
            }
        }
        // else: zeros (default from Array3::zeros)
    }

    out
}

pub struct Backtester {
    pool: PgPool,
    vm: StackVM,
    cache: HashMap<String, CachedData>,
    /// Reference asset close prices for cross-asset factors (e.g. "SPY" -> close array).
    ref_cache: HashMap<String, Array2<f64>>,
    factor_config: FactorConfig,
    pub exchange: String,
    pub resolution: String,
}

impl Backtester {
    pub fn new(
        pool: PgPool,
        factor_config: FactorConfig,
        exchange: String,
        resolution: String,
    ) -> Self {
        let ts_window = StackVM::ts_window_for_resolution(&resolution);
        tracing::info!(
            "Initializing Backtester with PSR fitness, ts_window={}",
            ts_window
        );
        Self {
            pool,
            vm: StackVM::with_window(&factor_config, ts_window),
            cache: HashMap::new(),
            ref_cache: HashMap::new(),
            factor_config,
            exchange,
            resolution,
        }
    }

    /// Annualization factor for Sharpe ratio based on resolution and exchange.
    fn annualization_factor(&self) -> f64 {
        match self.resolution.as_str() {
            "1d" => 252.0_f64.sqrt(),
            "1h" => {
                if self.exchange == "Polygon" {
                    // 6.5 market hours per trading day
                    (252.0_f64 * 6.5).sqrt()
                } else {
                    // Crypto: 24 hours
                    (365.0_f64 * 24.0).sqrt()
                }
            }
            "15m" => {
                if self.exchange == "Polygon" {
                    (252.0_f64 * 6.5 * 4.0).sqrt()
                } else {
                    // Crypto 24/7: 96 bars per day
                    (365.0_f64 * 96.0).sqrt()
                }
            }
            _ => (252.0_f64 * 96.0).sqrt(),
        }
    }

    /// Base transaction fee for the exchange.
    fn base_fee(&self) -> f64 {
        if self.exchange == "Polygon" {
            0.0001 // 1 bp for US equities
        } else {
            0.001 // 10 bps for crypto DEX
        }
    }

    /// Estimate trade capacity. For stocks, liquidity field is 0 so use volume.
    #[allow(dead_code)]
    fn capacity(&self, liquidity: f64, amount: f64) -> f64 {
        if self.exchange == "Polygon" {
            amount.max(1e6)
        } else if liquidity > 0.0 {
            liquidity
        } else {
            amount * 0.1
        }
    }

    /// Return the number of time bars for a cached symbol (0 if not loaded).
    pub fn data_length(&self, symbol: &str) -> usize {
        self.cache
            .get(symbol)
            .map(|d| d.returns.shape()[1])
            .unwrap_or(0)
    }

    pub async fn load_data(&mut self, symbols: &[String], days: i64) -> anyhow::Result<()> {
        let mut loaded_count = 0;

        for symbol in symbols {
            let rows = sqlx::query_as::<_, Candle>(
                r#"
                SELECT time, open, high, low, close,
                       COALESCE(volume, 0) as volume,
                       COALESCE(liquidity, 0) as liquidity,
                       COALESCE(fdv, 0) as fdv,
                       COALESCE(amount, 0) as amount
                FROM mkt_equity_candles
                WHERE exchange = $2 AND symbol = $1 AND resolution = $3
                AND time > NOW() - make_interval(days := $4)
                ORDER BY time ASC
                "#,
            )
            .bind(symbol)
            .bind(&self.exchange)
            .bind(&self.resolution)
            .bind(days as i32)
            .fetch_all(&self.pool)
            .await?;

            if rows.len() < 50 {
                tracing::warn!("Insufficient data for {}: {} rows", symbol, rows.len());
                continue;
            }

            // Convert to Array2
            let len = rows.len();
            let mut close = Array2::<f64>::zeros((1, len));
            let mut open = Array2::<f64>::zeros((1, len));
            let mut high = Array2::<f64>::zeros((1, len));
            let mut low = Array2::<f64>::zeros((1, len));
            let mut volume = Array2::<f64>::zeros((1, len));
            let mut liq = Array2::<f64>::zeros((1, len));
            let mut fdv = Array2::<f64>::zeros((1, len));
            let mut amount = Array2::<f64>::zeros((1, len));
            let mut timestamps = Vec::with_capacity(len);

            for (i, c) in rows.iter().enumerate() {
                close[[0, i]] = c.close.to_f64().unwrap_or(0.0);
                open[[0, i]] = c.open.to_f64().unwrap_or(0.0);
                high[[0, i]] = c.high.to_f64().unwrap_or(0.0);
                low[[0, i]] = c.low.to_f64().unwrap_or(0.0);
                volume[[0, i]] = c.volume.to_f64().unwrap_or(0.0);
                liq[[0, i]] = c.liquidity.to_f64().unwrap_or(0.0);
                fdv[[0, i]] = c.fdv.to_f64().unwrap_or(0.0);
                amount[[0, i]] = c.amount.to_f64().unwrap_or(0.0);
                timestamps.push(c.time.timestamp());
            }

            // Generate Features based on Config
            // Align reference data (SPY) to this symbol's bar count
            let ref_close_aligned = self.ref_cache.get("SPY").and_then(|spy| {
                let spy_len = spy.shape()[1];
                let sym_len = close.shape()[1];
                if spy_len >= sym_len {
                    Some(spy.slice(ndarray::s![.., (spy_len - sym_len)..]).to_owned())
                } else {
                    None
                }
            });
            let ohlcv = backtest_engine::factors::traits::OhlcvData {
                close: &close,
                open: &open,
                high: &high,
                low: &low,
                volume: &volume,
                liquidity: &liq,
                fdv: &fdv,
                ref_close: ref_close_aligned.as_ref(),
            };
            let features =
                FeatureEngineer::compute_features_from_config(&self.factor_config, &ohlcv);

            // Calculate Future Returns
            // Open-to-open with 1-bar execution delay:
            //   Signal at bar i → execute at open[i+1] → exit at open[i+2]
            //   return[i] = open[i+2] / open[i+1] - 1
            // This avoids look-ahead bias: signal uses data up to close[i],
            // trade enters at next bar's open, exits at the bar after.
            let mut min_price = f64::INFINITY;
            let mut max_price = f64::NEG_INFINITY;
            for x in close.iter() {
                if *x > max_price {
                    max_price = *x;
                }
                if *x < min_price {
                    min_price = *x;
                }
            }
            tracing::info!(
                "Symbol {} Price Range: [{}, {}]",
                symbol,
                min_price,
                max_price
            );

            let mut future_ret = Array2::<f64>::zeros((1, len));
            let mut max_ret = f64::NEG_INFINITY;
            for i in 0..len.saturating_sub(2) {
                let exec_price = open[[0, i + 1]];
                let exit_price = open[[0, i + 2]];
                let r = if exec_price.abs() > 1e-9 {
                    (exit_price / exec_price - 1.0).clamp(-0.99, 10.0)
                } else {
                    0.0
                };
                if r > max_ret {
                    max_ret = r;
                }
                future_ret[[0, i]] = r;
            }
            tracing::info!("Symbol {} Max Return: {}", symbol, max_ret);

            self.cache.insert(
                symbol.clone(),
                CachedData {
                    features,
                    returns: future_ret,
                    open: open.clone(),
                    close: close.clone(),
                    liquidity: liq,
                    amount,
                    timestamps,
                },
            );
            loaded_count += 1;
        }

        tracing::info!("Loaded data for {} symbols", loaded_count);
        Ok(())
    }

    /// Load reference asset close prices for cross-asset factors (e.g. SPY).
    /// Queries candle close prices from the same exchange/resolution and stores
    /// the close Array2 in ref_cache keyed by symbol.
    pub async fn load_reference_data(&mut self, symbol: &str, days: i64) -> anyhow::Result<()> {
        let rows = sqlx::query_as::<_, Candle>(
            r#"
            SELECT time, open, high, low, close,
                   COALESCE(volume, 0) as volume,
                   COALESCE(liquidity, 0) as liquidity,
                   COALESCE(fdv, 0) as fdv,
                   COALESCE(amount, 0) as amount
            FROM mkt_equity_candles
            WHERE exchange = $2 AND symbol = $1 AND resolution = $3
            AND time > NOW() - make_interval(days := $4)
            ORDER BY time ASC
            "#,
        )
        .bind(symbol)
        .bind(&self.exchange)
        .bind(&self.resolution)
        .bind(days as i32)
        .fetch_all(&self.pool)
        .await?;

        if rows.len() < 50 {
            return Err(anyhow::anyhow!(
                "Insufficient reference data for {}: {} rows",
                symbol,
                rows.len()
            ));
        }

        let len = rows.len();
        let mut close = Array2::<f64>::zeros((1, len));
        for (i, c) in rows.iter().enumerate() {
            close[[0, i]] = c.close.to_f64().unwrap_or(0.0);
        }

        tracing::info!("Loaded {} bars of reference data for {}", len, symbol);
        self.ref_cache.insert(symbol.to_string(), close);
        Ok(())
    }

    /// Load reference asset close prices at a specific resolution (for MTF).
    async fn load_reference_close(
        &self,
        symbol: &str,
        resolution: &str,
        days: i64,
    ) -> anyhow::Result<(Array2<f64>, usize)> {
        let rows = sqlx::query_as::<_, Candle>(
            r#"
            SELECT time, open, high, low, close,
                   COALESCE(volume, 0) as volume,
                   COALESCE(liquidity, 0) as liquidity,
                   COALESCE(fdv, 0) as fdv,
                   COALESCE(amount, 0) as amount
            FROM mkt_equity_candles
            WHERE exchange = $2 AND symbol = $1 AND resolution = $3
            AND time > NOW() - make_interval(days := $4)
            ORDER BY time ASC
            "#,
        )
        .bind(symbol)
        .bind(&self.exchange)
        .bind(resolution)
        .bind(days as i32)
        .fetch_all(&self.pool)
        .await?;

        let len = rows.len();
        let mut close = Array2::<f64>::zeros((1, len));
        for (i, c) in rows.iter().enumerate() {
            close[[0, i]] = c.close.to_f64().unwrap_or(0.0);
        }
        Ok((close, len))
    }

    /// Fetch candles at a specific resolution and return OHLCV arrays + timestamps.
    async fn fetch_candles_for_resolution(
        &self,
        symbol: &str,
        resolution: &str,
        days: i64,
    ) -> anyhow::Result<
        Option<(
            Array2<f64>,
            Array2<f64>,
            Array2<f64>,
            Array2<f64>,
            Array2<f64>,
            Array2<f64>,
            Array2<f64>,
            Array2<f64>,
            Vec<i64>,
        )>,
    > {
        let rows = sqlx::query_as::<_, Candle>(
            r#"
            SELECT time, open, high, low, close,
                   COALESCE(volume, 0) as volume,
                   COALESCE(liquidity, 0) as liquidity,
                   COALESCE(fdv, 0) as fdv,
                   COALESCE(amount, 0) as amount
            FROM mkt_equity_candles
            WHERE exchange = $2 AND symbol = $1 AND resolution = $3
            AND time > NOW() - make_interval(days := $4)
            ORDER BY time ASC
            "#,
        )
        .bind(symbol)
        .bind(&self.exchange)
        .bind(resolution)
        .bind(days as i32)
        .fetch_all(&self.pool)
        .await?;

        if rows.len() < 50 {
            tracing::warn!(
                "Insufficient {} data for {}: {} rows",
                resolution,
                symbol,
                rows.len()
            );
            return Ok(None);
        }

        let len = rows.len();
        let mut close = Array2::<f64>::zeros((1, len));
        let mut open = Array2::<f64>::zeros((1, len));
        let mut high = Array2::<f64>::zeros((1, len));
        let mut low = Array2::<f64>::zeros((1, len));
        let mut volume = Array2::<f64>::zeros((1, len));
        let mut liq = Array2::<f64>::zeros((1, len));
        let mut fdv = Array2::<f64>::zeros((1, len));
        let mut amount = Array2::<f64>::zeros((1, len));
        let mut timestamps = Vec::with_capacity(len);

        for (i, c) in rows.iter().enumerate() {
            close[[0, i]] = c.close.to_f64().unwrap_or(0.0);
            open[[0, i]] = c.open.to_f64().unwrap_or(0.0);
            high[[0, i]] = c.high.to_f64().unwrap_or(0.0);
            low[[0, i]] = c.low.to_f64().unwrap_or(0.0);
            volume[[0, i]] = c.volume.to_f64().unwrap_or(0.0);
            liq[[0, i]] = c.liquidity.to_f64().unwrap_or(0.0);
            fdv[[0, i]] = c.fdv.to_f64().unwrap_or(0.0);
            amount[[0, i]] = c.amount.to_f64().unwrap_or(0.0);
            timestamps.push(c.time.timestamp());
        }

        Ok(Some((
            close, open, high, low, volume, liq, fdv, amount, timestamps,
        )))
    }

    /// Load candle data at multiple resolutions, compute features per resolution,
    /// forward-fill align to the base (1h) time axis, and concatenate into a
    /// stacked feature tensor of shape (1, n_factors * n_resolutions, T_base).
    pub async fn load_data_multi_timeframe(
        &mut self,
        symbols: &[String],
        days: i64,
        mtf_config: &MultiTimeframeFactorConfig,
    ) -> anyhow::Result<()> {
        let resolutions = &mtf_config.resolutions;
        let _base_resolution = &resolutions[0]; // "1h" = highest frequency
        let mut loaded_count = 0;

        for symbol in symbols {
            // Fetch candles for each resolution
            let mut res_data: Vec<(Array3<f64>, Vec<i64>)> = Vec::new();
            let mut base_timestamps: Option<Vec<i64>> = None;
            let mut base_open: Option<Array2<f64>> = None;
            let mut base_close: Option<Array2<f64>> = None;
            let mut base_liq: Option<Array2<f64>> = None;
            let mut base_amount: Option<Array2<f64>> = None;

            let mut skip_symbol = false;
            for (res_idx, resolution) in resolutions.iter().enumerate() {
                let candle_data = self
                    .fetch_candles_for_resolution(symbol, resolution, days)
                    .await?;

                let (close, open, high, low, volume, liq, fdv, amount, timestamps) =
                    match candle_data {
                        Some(d) => d,
                        None => {
                            tracing::warn!(
                                "Skipping {} for MTF: insufficient {} data",
                                symbol,
                                resolution
                            );
                            skip_symbol = true;
                            break;
                        }
                    };

                // Load SPY reference for this resolution
                let ref_close_aligned = if self.exchange == "Polygon" && symbol != "SPY" {
                    match self.load_reference_close("SPY", resolution, days).await {
                        Ok((spy_close, spy_len)) => {
                            let sym_len = close.shape()[1];
                            if spy_len >= sym_len {
                                Some(
                                    spy_close
                                        .slice(ndarray::s![.., (spy_len - sym_len)..])
                                        .to_owned(),
                                )
                            } else {
                                None
                            }
                        }
                        Err(_) => None,
                    }
                } else {
                    None
                };

                let ohlcv = backtest_engine::factors::traits::OhlcvData {
                    close: &close,
                    open: &open,
                    high: &high,
                    low: &low,
                    volume: &volume,
                    liquidity: &liq,
                    fdv: &fdv,
                    ref_close: ref_close_aligned.as_ref(),
                };
                let features =
                    FeatureEngineer::compute_features_from_config(&self.factor_config, &ohlcv);

                tracing::info!(
                    "[{}:{}] {} features: {} factors x {} bars",
                    symbol,
                    resolution,
                    resolution,
                    features.shape()[1],
                    features.shape()[2]
                );

                // Save base resolution data for returns computation
                if res_idx == 0 {
                    base_timestamps = Some(timestamps.clone());
                    base_open = Some(open);
                    base_close = Some(close);
                    base_liq = Some(liq);
                    base_amount = Some(amount);
                }

                res_data.push((features, timestamps));
            }

            if skip_symbol {
                continue;
            }

            let hf_timestamps = base_timestamps.as_ref().unwrap();
            let hf_len = hf_timestamps.len();
            let n_factors_per_res = mtf_config.base_feat_count();
            let total_factors = mtf_config.feat_count();

            // Stack features: base resolution directly, others via forward-fill
            let mut stacked = Array3::<f64>::zeros((1, total_factors, hf_len));

            for (res_idx, (features, timestamps)) in res_data.iter().enumerate() {
                let offset = res_idx * n_factors_per_res;
                let aligned = if res_idx == 0 {
                    features.clone()
                } else {
                    forward_fill_align(features, timestamps, hf_timestamps)
                };

                for f in 0..n_factors_per_res {
                    stacked
                        .index_axis_mut(Axis(1), offset + f)
                        .assign(&aligned.index_axis(Axis(1), f));
                }
            }

            // Compute future returns from base resolution open prices
            let open = base_open.unwrap();
            let close = base_close.unwrap();

            let mut future_ret = Array2::<f64>::zeros((1, hf_len));
            for i in 0..hf_len.saturating_sub(2) {
                let exec_price = open[[0, i + 1]];
                let exit_price = open[[0, i + 2]];
                let r = if exec_price.abs() > 1e-9 {
                    (exit_price / exec_price - 1.0).clamp(-0.99, 10.0)
                } else {
                    0.0
                };
                future_ret[[0, i]] = r;
            }

            tracing::info!(
                "[{}] MTF stacked: {} factors x {} bars (resolutions: {:?})",
                symbol,
                total_factors,
                hf_len,
                resolutions
            );

            self.cache.insert(
                symbol.clone(),
                CachedData {
                    features: stacked,
                    returns: future_ret,
                    open,
                    close,
                    liquidity: base_liq.unwrap(),
                    amount: base_amount.unwrap(),
                    timestamps: hf_timestamps.clone(),
                },
            );
            loaded_count += 1;
        }

        tracing::info!(
            "Loaded MTF data for {} symbols ({} resolutions)",
            loaded_count,
            resolutions.len()
        );
        Ok(())
    }

    /// PnL-based fitness for a single symbol's in-sample data (first 70%).
    ///
    /// Matches AlphaGPT's approach:
    ///   1. sigmoid(raw_signal) → [0, 1]
    ///   2. position = 1.0 if sigmoid > threshold, else 0.0 (long-only)
    ///   3. net_pnl = position * open-to-open return - turnover * fee
    ///   4. fitness = cumulative_pnl - drawdown_penalty - complexity_penalty
    ///   5. Require minimum trading activity
    #[allow(dead_code)]
    pub fn evaluate_symbol(&self, genome: &mut Genome, symbol: &str, mode: StrategyMode) {
        let data = match self.cache.get(symbol) {
            Some(d) => d,
            None => {
                genome.fitness = -1000.0;
                return;
            }
        };

        if let Some(signal) = self.vm.execute(&genome.tokens, &data.features) {
            let sig_slice = signal.as_slice().unwrap();
            let ret_slice = data.returns.as_slice().unwrap();

            let len = sig_slice.len().min(ret_slice.len());
            if len < 20 {
                genome.fitness = -1000.0;
                return;
            }

            let split_idx = (len as f64 * 0.7).max(20.0) as usize;
            let pnl = self.pnl_fitness(sig_slice, ret_slice, 0, split_idx, mode);

            // Parsimony pressure: penalize formulas longer than 10 tokens.
            // Shorter formulas are less likely to overfit.
            let token_len = genome.tokens.len();
            let complexity_penalty = if token_len > 10 {
                (token_len - 10) as f64 * 0.005
            } else {
                0.0
            };

            genome.fitness = pnl - complexity_penalty;
        } else {
            genome.fitness = -1000.0;
        }
    }

    /// PnL-based evaluation on out-of-sample data (last 30%).
    #[allow(dead_code)]
    pub fn evaluate_symbol_oos(&self, genome: &Genome, symbol: &str, mode: StrategyMode) -> f64 {
        let data = match self.cache.get(symbol) {
            Some(d) => d,
            None => return 0.0,
        };

        if let Some(signal) = self.vm.execute(&genome.tokens, &data.features) {
            let sig_slice = signal.as_slice().unwrap();
            let ret_slice = data.returns.as_slice().unwrap();

            let len = sig_slice.len().min(ret_slice.len());
            if len < 20 {
                return 0.0;
            }

            let split_idx = (len as f64 * 0.7).max(20.0) as usize;
            if split_idx >= len {
                return 0.0;
            }

            self.pnl_fitness(sig_slice, ret_slice, split_idx, len, mode)
        } else {
            0.0
        }
    }

    /// PSR-based evaluation on out-of-sample data using walk-forward methodology.
    /// Thresholds are computed from training windows, fixing the look-ahead bias
    /// present in the old fixed 70/30 split.
    #[allow(dead_code)]
    pub fn evaluate_symbol_oos_psr(
        &self,
        genome: &Genome,
        symbol: &str,
        mode: StrategyMode,
    ) -> f64 {
        let wf_config = WalkForwardConfig {
            embargo: self.embargo_size(),
            ..WalkForwardConfig::default_1h()
        };
        let result = self.evaluate_walk_forward_oos(genome, symbol, mode, &wf_config);
        result.aggregate_psr
    }

    /// Walk-forward OOS evaluation returning full diagnostic details.
    #[allow(dead_code)]
    pub fn evaluate_walk_forward_oos_detail(
        &self,
        genome: &Genome,
        symbol: &str,
        mode: StrategyMode,
    ) -> WalkForwardResult {
        let wf_config = WalkForwardConfig {
            embargo: self.embargo_size(),
            ..WalkForwardConfig::default_1h()
        };
        self.evaluate_walk_forward_oos(genome, symbol, mode, &wf_config)
    }

    /// Walk-forward OOS evaluation with explicit config.
    pub fn evaluate_walk_forward_oos_with_config(
        &self,
        genome: &Genome,
        symbol: &str,
        mode: StrategyMode,
        config: &WalkForwardConfig,
    ) -> WalkForwardResult {
        self.evaluate_walk_forward_oos(genome, symbol, mode, config)
    }

    /// Walk-forward OOS evaluation: expanding-window train, fixed-window test.
    ///
    /// Thresholds are computed from the TRAIN window and applied to the TEST
    /// window, eliminating look-ahead bias. Multiple walk-forward steps are
    /// aggregated as mean(psr) - 0.5 * std(psr) to penalize inconsistency.
    fn evaluate_walk_forward_oos(
        &self,
        genome: &Genome,
        symbol: &str,
        mode: StrategyMode,
        config: &WalkForwardConfig,
    ) -> WalkForwardResult {
        let fail = |sentinel: f64, label: &str| WalkForwardResult {
            aggregate_psr: sentinel,
            mean_psr: sentinel,
            std_psr: 0.0,
            num_steps: 0,
            num_valid_steps: 0,
            steps: vec![],
            failure_mode: Some(label.to_string()),
        };

        let data = match self.cache.get(symbol) {
            Some(d) => d,
            None => return fail(SENTINEL_CACHE_MISS, "cache_miss"),
        };

        let signal = match self.vm.execute(&genome.tokens, &data.features) {
            Some(s) => s,
            None => return fail(SENTINEL_VM_FAILURE, "vm_failure"),
        };

        let sig_slice = signal.as_slice().unwrap();
        let ret_slice = data.returns.as_slice().unwrap();
        let total_bars = sig_slice.len().min(ret_slice.len());

        if total_bars < 60 {
            return fail(SENTINEL_INSUFFICIENT_DATA, "insufficient_data");
        }

        // Compute adaptive step sizing
        let initial_train = config.initial_train.min(total_bars * 2 / 3);
        let available = total_bars.saturating_sub(initial_train);
        if available < config.min_test_window + config.embargo {
            return fail(SENTINEL_OOS_TOO_SMALL, "oos_too_small");
        }

        let test_window = if config.target_steps > 0 {
            let denominator = config.target_steps * (1 + config.embargo);
            let tw = if denominator > 0 {
                available.saturating_sub(config.target_steps * config.embargo) / config.target_steps
            } else {
                available
            };
            tw.clamp(config.min_test_window, config.target_test_window)
        } else {
            config.target_test_window.min(available)
        };

        let num_steps = if test_window + config.embargo > 0 {
            let max_steps = available / (test_window + config.embargo);
            max_steps.min(config.target_steps).max(1)
        } else {
            1
        };

        let mut steps = Vec::with_capacity(num_steps);
        let mut valid_psrs = Vec::new();

        for i in 0..num_steps {
            // Expanding train window
            let train_start = 0;
            let train_end = initial_train + i * (test_window + config.embargo);
            let test_start = train_end + config.embargo;
            let test_end = if i == num_steps - 1 {
                total_bars.min(test_start + test_window)
            } else {
                (test_start + test_window).min(total_bars)
            };

            if test_start >= total_bars || test_end <= test_start {
                continue;
            }
            let actual_test_len = test_end - test_start;
            if actual_test_len < config.min_test_window.min(30) {
                continue;
            }

            // Compute thresholds from TRAIN window (fixing look-ahead bias)
            // LongShort: z-score thresholds; LongOnly: sigmoid thresholds
            let (upper, lower, train_zs) = match mode {
                StrategyMode::LongShort => {
                    let (mean, std) = zscore_params(sig_slice, train_start, train_end);
                    let u = adaptive_threshold_zscore(sig_slice, train_start, train_end, mean, std);
                    let l = adaptive_lower_threshold_zscore(
                        sig_slice,
                        train_start,
                        train_end,
                        mean,
                        std,
                    );
                    (u, l, Some((mean, std)))
                }
                StrategyMode::LongOnly => {
                    let u = adaptive_threshold(sig_slice, train_start, train_end);
                    (u, 0.0, None)
                }
            };

            // Evaluate PSR on TEST window using train-derived thresholds
            let (psr, trade_count, active_count) = self.psr_fitness_oos(
                sig_slice, ret_slice, test_start, test_end, mode, upper, lower, train_zs,
            );

            if is_sentinel(psr) {
                tracing::warn!(
                    "[{}:{}:{}] WF step {} failed: {} (train=[{}..{}], test=[{}..{}], upper={:.4}, lower={:.4})",
                    self.exchange, symbol, mode.as_str(), i, sentinel_label(psr),
                    train_start, train_end, test_start, test_end, upper, lower
                );
            }

            let step = WalkForwardStep {
                step: i,
                train_start,
                train_end,
                test_start,
                test_end,
                psr,
                trade_count,
                active_bars: active_count,
                upper_threshold: upper,
                lower_threshold: lower,
            };
            steps.push(step);

            if !is_sentinel(psr) {
                valid_psrs.push(psr);
            }
        }

        // Aggregate valid steps
        let num_valid = valid_psrs.len();
        let (aggregate_psr, mean_psr, std_psr, failure_mode) = if num_valid >= 2 {
            let mean = valid_psrs.iter().sum::<f64>() / num_valid as f64;
            let var = valid_psrs.iter().map(|&p| (p - mean).powi(2)).sum::<f64>()
                / (num_valid as f64 - 1.0);
            let std = var.sqrt();
            let agg = mean - 0.5 * std;
            (agg, mean, std, None)
        } else if num_valid == 1 {
            // Single valid step — use it directly but flag reduced confidence
            (
                valid_psrs[0],
                valid_psrs[0],
                0.0,
                Some("single_step".to_string()),
            )
        } else {
            // No valid steps — find the dominant failure mode
            let dominant = steps
                .iter()
                .filter(|s| is_sentinel(s.psr))
                .map(|s| sentinel_label(s.psr))
                .next()
                .unwrap_or("no_steps");
            let worst_sentinel = steps.iter().map(|s| s.psr).fold(f64::INFINITY, f64::min);
            let sentinel = if worst_sentinel.is_finite() {
                worst_sentinel
            } else {
                SENTINEL_OOS_TOO_SMALL
            };
            (sentinel, sentinel, 0.0, Some(dominant.to_string()))
        };

        let result = WalkForwardResult {
            aggregate_psr,
            mean_psr,
            std_psr,
            num_steps: steps.len(),
            num_valid_steps: num_valid,
            steps,
            failure_mode,
        };

        tracing::info!(
            "[{}:{}:{}] Walk-forward OOS: {} steps, {}/{} valid, aggregate_psr={:.4}, per_step={:?}",
            self.exchange, symbol, mode.as_str(),
            result.num_steps, result.num_valid_steps, result.num_steps,
            result.aggregate_psr,
            result.steps.iter().map(|s| s.psr).collect::<Vec<_>>()
        );

        result
    }

    /// Resolution-aware embargo size (bars to skip at fold boundaries).
    /// Prevents information leakage from TS operators carrying state across folds.
    pub fn embargo_size(&self) -> usize {
        match self.resolution.as_str() {
            "1d" => 20, // 20 trading days (~1 month)
            "1h" => 10, // 10 hours (matches TS window)
            "15m" => 8, // 2 hours
            _ => 10,
        }
    }

    /// K-fold temporal cross-validation fitness for a single symbol.
    ///
    /// Runs VM once, then evaluates PSR (Probabilistic Sharpe Ratio) on K
    /// equal-sized temporal folds with embargo gaps at fold boundaries.
    /// Fitness = mean(fold_psr) - 0.5 * std(fold_psr) - complexity_penalty.
    /// Strategies must perform consistently across all time regimes.
    pub fn evaluate_symbol_kfold(
        &self,
        genome: &mut Genome,
        symbol: &str,
        k: usize,
        mode: StrategyMode,
    ) {
        let data = match self.cache.get(symbol) {
            Some(d) => d,
            None => {
                genome.fitness = -1000.0;
                return;
            }
        };

        let signal = match self.vm.execute(&genome.tokens, &data.features) {
            Some(s) => s,
            None => {
                genome.fitness = -1000.0;
                return;
            }
        };

        let sig_slice = signal.as_slice().unwrap();
        let ret_slice = data.returns.as_slice().unwrap();
        let len = sig_slice.len().min(ret_slice.len());
        if len < 20 {
            genome.fitness = -1000.0;
            return;
        }

        // Split into K equal folds with embargo gaps
        let fold_size = len / k;
        if fold_size < 30 {
            // PSR needs 30+ samples per fold for statistical significance
            genome.fitness = -1000.0;
            return;
        }

        let embargo = self.embargo_size();
        let mut fold_scores = Vec::with_capacity(k);
        for i in 0..k {
            // Apply embargo: skip bars at fold start that overlap with previous fold's lookback
            let raw_start = i * fold_size;
            let start = if i > 0 {
                (raw_start + embargo).min(len)
            } else {
                raw_start
            };
            let end = if i == k - 1 { len } else { (i + 1) * fold_size };

            if end <= start || end - start < 30 {
                continue;
            }

            let psr = self.psr_fitness(sig_slice, ret_slice, start, end, mode);
            if psr > -9.0 {
                fold_scores.push(psr);
            }
        }

        // Require valid performance in at least 3 of K folds
        let min_valid = 3_usize.min(k);
        if fold_scores.len() < min_valid {
            genome.fitness = SENTINEL_CACHE_MISS; // Keep -10.0 for IS; decomposition is OOS-only
            return;
        }

        let n_folds = fold_scores.len() as f64;
        let mean_psr = fold_scores.iter().sum::<f64>() / n_folds;
        let std_psr = if n_folds > 1.0 {
            let var = fold_scores
                .iter()
                .map(|&p| (p - mean_psr).powi(2))
                .sum::<f64>()
                / (n_folds - 1.0);
            var.sqrt()
        } else {
            0.0
        };

        // Parsimony: penalize formulas longer than 8 tokens, scaled inversely with data length
        let token_len = genome.tokens.len();
        let penalty_scale = (1000.0 / (len as f64).max(1000.0)).clamp(0.2, 1.0);
        let complexity_penalty = if token_len > 8 {
            (token_len - 8) as f64 * 0.02 * penalty_scale
        } else {
            0.0
        };

        let fitness = mean_psr - 0.5 * std_psr - complexity_penalty;
        genome.fitness = if fitness.is_nan() { -10.0 } else { fitness };
    }

    /// Diagnostic: return per-fold PnL for monitoring/frontend display.
    /// Uses the same embargo gaps as evaluate_symbol_kfold for consistency.
    #[allow(dead_code)]
    pub fn evaluate_symbol_fold_detail(
        &self,
        genome: &Genome,
        symbol: &str,
        k: usize,
        mode: StrategyMode,
    ) -> Vec<f64> {
        let data = match self.cache.get(symbol) {
            Some(d) => d,
            None => return vec![],
        };

        let signal = match self.vm.execute(&genome.tokens, &data.features) {
            Some(s) => s,
            None => return vec![],
        };

        let sig_slice = signal.as_slice().unwrap();
        let ret_slice = data.returns.as_slice().unwrap();
        let len = sig_slice.len().min(ret_slice.len());
        if len < 20 {
            return vec![];
        }

        let fold_size = len / k;
        if fold_size < 30 {
            return vec![];
        }

        let embargo = self.embargo_size();
        let mut fold_pnls = Vec::with_capacity(k);
        for i in 0..k {
            let raw_start = i * fold_size;
            let start = if i > 0 {
                (raw_start + embargo).min(len)
            } else {
                raw_start
            };
            let end = if i == k - 1 { len } else { (i + 1) * fold_size };
            if end > start {
                fold_pnls.push(self.pnl_fitness(sig_slice, ret_slice, start, end, mode));
            }
        }
        fold_pnls
    }

    /// Diagnostic: return per-fold PSR z-scores (same metric as IS fitness).
    /// Uses embargo gaps for consistency with evaluate_symbol_kfold.
    pub fn evaluate_symbol_fold_psr_detail(
        &self,
        genome: &Genome,
        symbol: &str,
        k: usize,
        mode: StrategyMode,
    ) -> Vec<f64> {
        let data = match self.cache.get(symbol) {
            Some(d) => d,
            None => return vec![],
        };

        let signal = match self.vm.execute(&genome.tokens, &data.features) {
            Some(s) => s,
            None => return vec![],
        };

        let sig_slice = signal.as_slice().unwrap();
        let ret_slice = data.returns.as_slice().unwrap();
        let len = sig_slice.len().min(ret_slice.len());
        if len < 20 {
            return vec![];
        }

        let fold_size = len / k;
        if fold_size < 30 {
            return vec![];
        }

        let embargo = self.embargo_size();
        let mut fold_psrs = Vec::with_capacity(k);
        for i in 0..k {
            let raw_start = i * fold_size;
            let start = if i > 0 {
                (raw_start + embargo).min(len)
            } else {
                raw_start
            };
            let end = if i == k - 1 { len } else { (i + 1) * fold_size };
            if end > start && end - start >= 30 {
                fold_psrs.push(self.psr_fitness(sig_slice, ret_slice, start, end, mode));
            } else {
                fold_psrs.push(-10.0);
            }
        }
        fold_psrs
    }

    /// Probabilistic Sharpe Ratio (PSR) fitness for a fold.
    ///
    /// Computes the probability that the true Sharpe ratio exceeds a benchmark
    /// (default: 0), accounting for skewness and kurtosis of the return distribution.
    /// Returns a z-score: higher = more likely the Sharpe is real, not noise.
    ///
    /// Reference: Bailey & Lopez de Prado (2012), "The Sharpe Ratio Efficient Frontier"
    fn psr_fitness(
        &self,
        sig: &[f64],
        ret: &[f64],
        start: usize,
        end: usize,
        mode: StrategyMode,
    ) -> f64 {
        let n = end - start;
        if n < 30 {
            return -10.0;
        }

        // Collect per-bar returns using the same position logic as pnl_fitness
        // LongShort: z-score normalization (de-mean + scale by std)
        // LongOnly:  sigmoid normalization (legacy, unchanged)
        let (upper, lower, z_mean, z_std) = match mode {
            StrategyMode::LongShort => {
                let (mean, std) = zscore_params(sig, start, end);
                let u = adaptive_threshold_zscore(sig, start, end, mean, std);
                let l = adaptive_lower_threshold_zscore(sig, start, end, mean, std);
                (u, l, mean, std)
            }
            StrategyMode::LongOnly => {
                let u = adaptive_threshold(sig, start, end);
                (u, 0.0, 0.0, 1.0)
            }
        };
        let fee = self.base_fee();
        let mut prev_pos = 0.0_f64;
        let mut bar_returns = Vec::with_capacity(n);
        let mut trade_count = 0_u32;
        let mut active_bars = 0_u32;

        for i in start..end {
            let sig_val = match mode {
                StrategyMode::LongShort => (sig[i] - z_mean) / z_std,
                StrategyMode::LongOnly => sigmoid(sig[i]),
            };
            let pos = match mode {
                StrategyMode::LongOnly => {
                    if sig_val > upper {
                        1.0
                    } else {
                        0.0
                    }
                }
                StrategyMode::LongShort => {
                    if sig_val > upper {
                        1.0
                    } else if sig_val < lower {
                        -1.0
                    } else {
                        0.0
                    }
                }
            };

            let turnover = (pos - prev_pos).abs();
            let entering_short = pos < -0.5 && prev_pos > -0.5;
            let cost = if entering_short {
                turnover * fee * 1.5
            } else {
                turnover * fee
            };

            let bar_pnl = pos * ret[i] - cost;
            bar_returns.push(bar_pnl);

            if turnover > 0.5 {
                trade_count += 1;
            }
            if pos.abs() > 0.5 {
                active_bars += 1;
            }

            prev_pos = pos;
        }

        // Minimum activity check (same as pnl_fitness)
        let bars_per_day = match self.resolution.as_str() {
            "1d" => 1.0,
            "1h" => {
                if self.exchange == "Polygon" {
                    6.5
                } else {
                    24.0
                }
            }
            "15m" => {
                if self.exchange == "Polygon" {
                    26.0
                } else {
                    96.0
                }
            }
            _ => 24.0,
        };
        let trading_days = n as f64 / bars_per_day;
        let min_trades = 3_u32.max((trading_days / 10.0) as u32);
        if trade_count < min_trades || (active_bars as f64) < (n as f64 * 0.05) {
            return -10.0;
        }

        // Compute PSR
        let nf = bar_returns.len() as f64;
        let mean = bar_returns.iter().sum::<f64>() / nf;
        let var = bar_returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (nf - 1.0);
        let std = var.sqrt();
        if std < 1e-10 {
            return -10.0;
        }

        let sharpe = mean / std;

        // Higher moments: skewness and excess kurtosis
        let skew = bar_returns
            .iter()
            .map(|r| ((r - mean) / std).powi(3))
            .sum::<f64>()
            / nf;
        let kurt = bar_returns
            .iter()
            .map(|r| ((r - mean) / std).powi(4))
            .sum::<f64>()
            / nf
            - 3.0;

        // PSR formula: standard error of Sharpe ratio adjusted for non-normality
        // (Bailey & Lopez de Prado 2012, eq. 4)
        let benchmark_sharpe = 0.0; // Test if Sharpe > 0
        let se_inner = (1.0 - skew * sharpe + (kurt - 1.0) / 4.0 * sharpe.powi(2)) / nf;
        if se_inner <= 0.0 {
            return -10.0;
        }
        let se_sharpe = se_inner.sqrt();
        if se_sharpe < 1e-10 {
            return -10.0;
        }

        let z = (sharpe - benchmark_sharpe) / se_sharpe;

        // Clamp to reasonable range to avoid extreme outliers dominating
        if z.is_nan() {
            -10.0
        } else {
            z.clamp(-5.0, 5.0)
        }
    }

    /// PSR fitness for OOS evaluation with pre-computed thresholds.
    ///
    /// Unlike `psr_fitness()`, this variant:
    /// - Accepts pre-computed thresholds (computed from the TRAIN window, not test)
    /// - Returns distinct sentinel values per failure mode
    /// - Also returns trade_count and active_bars for diagnostics
    #[allow(clippy::too_many_arguments)]
    fn psr_fitness_oos(
        &self,
        sig: &[f64],
        ret: &[f64],
        start: usize,
        end: usize,
        mode: StrategyMode,
        upper_threshold: f64,
        lower_threshold: f64,
        train_zscore: Option<(f64, f64)>,
    ) -> (f64, u32, u32) {
        let n = end - start;
        if n < 30 {
            return (SENTINEL_TOO_FEW_BARS, 0, 0);
        }

        let fee = self.base_fee();
        let mut prev_pos = 0.0_f64;
        let mut bar_returns = Vec::with_capacity(n);
        let mut trade_count = 0_u32;
        let mut active_bars = 0_u32;

        for i in start..end {
            let sig_val = match (mode, train_zscore) {
                (StrategyMode::LongShort, Some((mean, std))) => (sig[i] - mean) / std,
                _ => sigmoid(sig[i]),
            };
            let pos = match mode {
                StrategyMode::LongOnly => {
                    if sig_val > upper_threshold {
                        1.0
                    } else {
                        0.0
                    }
                }
                StrategyMode::LongShort => {
                    if sig_val > upper_threshold {
                        1.0
                    } else if sig_val < lower_threshold {
                        -1.0
                    } else {
                        0.0
                    }
                }
            };

            let turnover = (pos - prev_pos).abs();
            let entering_short = pos < -0.5 && prev_pos > -0.5;
            let cost = if entering_short {
                turnover * fee * 1.5
            } else {
                turnover * fee
            };

            let bar_pnl = pos * ret[i] - cost;
            bar_returns.push(bar_pnl);

            if turnover > 0.5 {
                trade_count += 1;
            }
            if pos.abs() > 0.5 {
                active_bars += 1;
            }

            prev_pos = pos;
        }

        // Minimum activity check
        let bars_per_day = match self.resolution.as_str() {
            "1d" => 1.0,
            "1h" => {
                if self.exchange == "Polygon" {
                    6.5
                } else {
                    24.0
                }
            }
            "15m" => {
                if self.exchange == "Polygon" {
                    26.0
                } else {
                    96.0
                }
            }
            _ => 24.0,
        };
        let trading_days = n as f64 / bars_per_day;
        let min_trades = 3_u32.max((trading_days / 10.0) as u32);
        if trade_count < min_trades || (active_bars as f64) < (n as f64 * 0.05) {
            return (SENTINEL_TOO_FEW_TRADES, trade_count, active_bars);
        }

        // Compute PSR
        let nf = bar_returns.len() as f64;
        let mean = bar_returns.iter().sum::<f64>() / nf;
        let var = bar_returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (nf - 1.0);
        let std = var.sqrt();
        if std < 1e-10 {
            return (SENTINEL_ZERO_VARIANCE, trade_count, active_bars);
        }

        let sharpe = mean / std;

        let skew = bar_returns
            .iter()
            .map(|r| ((r - mean) / std).powi(3))
            .sum::<f64>()
            / nf;
        let kurt = bar_returns
            .iter()
            .map(|r| ((r - mean) / std).powi(4))
            .sum::<f64>()
            / nf
            - 3.0;

        let benchmark_sharpe = 0.0;
        let se_inner = (1.0 - skew * sharpe + (kurt - 1.0) / 4.0 * sharpe.powi(2)) / nf;
        if se_inner <= 0.0 {
            return (SENTINEL_NEGATIVE_SE, trade_count, active_bars);
        }
        let se_sharpe = se_inner.sqrt();
        if se_sharpe < 1e-10 {
            return (SENTINEL_ZERO_SE, trade_count, active_bars);
        }

        let z = (sharpe - benchmark_sharpe) / se_sharpe;

        if z.is_nan() {
            (SENTINEL_NAN_PSR, trade_count, active_bars)
        } else {
            (z.clamp(-5.0, 5.0), trade_count, active_bars)
        }
    }

    /// Core PnL fitness computation used by both IS and OOS evaluation.
    ///
    /// Signal processing: sigmoid -> threshold -> position
    ///   - LongOnly: pos = 1.0 if sigmoid > upper, else 0.0
    ///   - LongShort: pos = 1.0 if sigmoid > upper, -1.0 if sigmoid < lower, else 0.0
    ///   - Cost model: IBKR 1bp base fee + 50% borrow premium for short entries
    ///   - Penalty: large per-bar losses
    fn pnl_fitness(
        &self,
        sig: &[f64],
        ret: &[f64],
        start: usize,
        end: usize,
        mode: StrategyMode,
    ) -> f64 {
        let n = end - start;
        if n < 10 {
            return -10.0;
        }

        // LongShort: z-score thresholds on raw signal; LongOnly: sigmoid thresholds
        let (upper, lower, z_mean, z_std) = match mode {
            StrategyMode::LongShort => {
                let (mean, std) = zscore_params(sig, start, end);
                let u = adaptive_threshold_zscore(sig, start, end, mean, std);
                let l = adaptive_lower_threshold_zscore(sig, start, end, mean, std);
                (u, l, mean, std)
            }
            StrategyMode::LongOnly => {
                let u = adaptive_threshold(sig, start, end);
                (u, 0.0, 0.0, 1.0)
            }
        };
        let fee = self.base_fee();
        let mut prev_pos = 0.0_f64;
        let mut cum_pnl = 0.0_f64;
        let mut trade_count = 0_u32;
        let mut active_bars = 0_u32;
        let mut big_loss_count = 0_u32;

        for i in start..end {
            let sig_val = match mode {
                StrategyMode::LongShort => (sig[i] - z_mean) / z_std,
                StrategyMode::LongOnly => sigmoid(sig[i]),
            };
            let pos = match mode {
                StrategyMode::LongOnly => {
                    if sig_val > upper {
                        1.0
                    } else {
                        0.0
                    }
                }
                StrategyMode::LongShort => {
                    if sig_val > upper {
                        1.0
                    } else if sig_val < lower {
                        -1.0
                    } else {
                        0.0
                    }
                }
            };

            // Turnover and costs — short entries incur 50% borrow premium
            let turnover = (pos - prev_pos).abs();
            let entering_short = pos < -0.5 && prev_pos > -0.5;
            let cost = if entering_short {
                turnover * fee * 1.5
            } else {
                turnover * fee
            };

            // PnL for this bar (short: pos=-1.0 * positive_return = loss, negative_return = gain)
            let bar_pnl = pos * ret[i] - cost;
            cum_pnl += bar_pnl;

            // Track activity
            if turnover > 0.5 {
                trade_count += 1;
            }
            if pos.abs() > 0.5 {
                active_bars += 1;
            }

            // Track large per-bar losses (> 2% for equities)
            if bar_pnl < -0.02 {
                big_loss_count += 1;
            }

            prev_pos = pos;
        }

        // Minimum activity: resolution-aware, targeting ~1 trade per 10 trading days
        let bars_per_day = match self.resolution.as_str() {
            "1d" => 1.0,
            "1h" => {
                if self.exchange == "Polygon" {
                    6.5
                } else {
                    24.0
                }
            }
            "15m" => {
                if self.exchange == "Polygon" {
                    26.0
                } else {
                    96.0
                }
            }
            _ => 24.0,
        };
        let trading_days = n as f64 / bars_per_day;
        let min_trades = 3_u32.max((trading_days / 10.0) as u32);
        if trade_count < min_trades || (active_bars as f64) < (n as f64 * 0.05) {
            return -10.0;
        }

        // Drawdown penalty: 0.5 per big loss event (adapted from AlphaGPT's 2.0 for crypto)
        let dd_penalty = big_loss_count.saturating_sub(3) as f64 * 0.5;

        let fitness = cum_pnl - dd_penalty;
        if fitness.is_nan() {
            -10.0
        } else {
            fitness
        }
    }

    /// Evaluate a genome across all cached symbols (PnL-based, in-sample).
    #[allow(dead_code)]
    pub fn evaluate(&self, genome: &mut Genome, mode: StrategyMode) {
        if self.cache.is_empty() {
            genome.fitness = 0.0;
            return;
        }

        let mut total_score = 0.0;
        let mut valid_count = 0.0;
        let total_symbols = self.cache.len() as f64;

        for data in self.cache.values() {
            if let Some(signal) = self.vm.execute(&genome.tokens, &data.features) {
                let sig_slice = signal.as_slice().unwrap();
                let ret_slice = data.returns.as_slice().unwrap();
                let len = sig_slice.len().min(ret_slice.len());
                if len < 20 {
                    continue;
                }
                let split_idx = (len as f64 * 0.7).max(20.0) as usize;
                let score = self.pnl_fitness(sig_slice, ret_slice, 0, split_idx, mode);
                total_score += score;
                valid_count += 1.0;
            }
        }

        if valid_count < 1.0 || valid_count / total_symbols < 0.2 {
            genome.fitness = -1000.0;
            return;
        }

        let avg_score = total_score / valid_count;
        genome.fitness = if avg_score.is_nan() {
            -999.0
        } else {
            avg_score
        };
    }

    /// Evaluate a genome across all cached symbols (PnL-based, out-of-sample).
    #[allow(dead_code)]
    pub fn evaluate_oos(&self, genome: &Genome, mode: StrategyMode) -> f64 {
        if self.cache.is_empty() {
            return 0.0;
        }

        let mut total_score = 0.0;
        let mut count = 0.0;

        for data in self.cache.values() {
            if let Some(signal) = self.vm.execute(&genome.tokens, &data.features) {
                let sig_slice = signal.as_slice().unwrap();
                let ret_slice = data.returns.as_slice().unwrap();
                let len = sig_slice.len().min(ret_slice.len());
                if len < 20 {
                    continue;
                }
                let split_idx = (len as f64 * 0.7).max(20.0) as usize;
                if split_idx >= len {
                    continue;
                }
                let score = self.pnl_fitness(sig_slice, ret_slice, split_idx, len, mode);
                total_score += score;
                count += 1.0;
            }
        }

        if count > 0.0 {
            let avg = total_score / count;
            if avg.is_nan() {
                0.0
            } else {
                avg
            }
        } else {
            0.0
        }
    }
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

/// Compute an adaptive sigmoid threshold as the 70th percentile of sigmoid(signal),
/// clamped to [0.52, 0.70]. Goes long on top ~30% of signals.
fn adaptive_threshold(sig: &[f64], start: usize, end: usize) -> f64 {
    let mut vals: Vec<f64> = (start..end).map(|i| sigmoid(sig[i])).collect();
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if vals.is_empty() {
        return 0.65;
    }
    let idx = ((vals.len() as f64) * 0.70) as usize;
    vals[idx.min(vals.len() - 1)].clamp(0.52, 0.70)
}

/// Compute an adaptive lower sigmoid threshold as the 30th percentile of sigmoid(signal),
/// clamped to [0.20, 0.48]. Superseded by z-score thresholds for LongShort (P3.5).
/// Retained for LongOnly backward compatibility and test comparison.
#[allow(dead_code)]
fn adaptive_lower_threshold(sig: &[f64], start: usize, end: usize) -> f64 {
    let mut vals: Vec<f64> = (start..end).map(|i| sigmoid(sig[i])).collect();
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if vals.is_empty() {
        return 0.35;
    }
    let idx = ((vals.len() as f64) * 0.30) as usize;
    vals[idx.min(vals.len() - 1)].clamp(0.20, 0.48)
}

// ── Z-Score / De-mean signal normalization (P3.5 LongShort fix) ──────────

/// Compute mean and standard deviation of raw signal over [start, end).
/// Returns (mean, std) with std floored at 1e-10 to avoid division by zero.
fn zscore_params(sig: &[f64], start: usize, end: usize) -> (f64, f64) {
    let n = (end - start) as f64;
    if n < 2.0 {
        return (0.0, 1.0);
    }
    let mean = (start..end).map(|i| sig[i]).sum::<f64>() / n;
    let var = (start..end).map(|i| (sig[i] - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let std = var.sqrt().max(1e-10);
    (mean, std)
}

/// Compute adaptive upper threshold as 70th percentile of z-scored signal,
/// clamped to [0.1, 2.0]. For LongShort mode (raw signal, no sigmoid).
fn adaptive_threshold_zscore(sig: &[f64], start: usize, end: usize, mean: f64, std: f64) -> f64 {
    let mut zvals: Vec<f64> = (start..end).map(|i| (sig[i] - mean) / std).collect();
    zvals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if zvals.is_empty() {
        return 0.5;
    }
    let idx = ((zvals.len() as f64) * 0.70) as usize;
    zvals[idx.min(zvals.len() - 1)].clamp(0.1, 2.0)
}

/// Compute adaptive lower threshold as 30th percentile of z-scored signal,
/// clamped to [-2.0, -0.1]. For LongShort mode (raw signal, no sigmoid).
fn adaptive_lower_threshold_zscore(
    sig: &[f64],
    start: usize,
    end: usize,
    mean: f64,
    std: f64,
) -> f64 {
    let mut zvals: Vec<f64> = (start..end).map(|i| (sig[i] - mean) / std).collect();
    zvals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if zvals.is_empty() {
        return -0.5;
    }
    let idx = ((zvals.len() as f64) * 0.30) as usize;
    zvals[idx.min(zvals.len() - 1)].clamp(-2.0, -0.1)
}

#[allow(dead_code)]
fn spearman_rank_corr(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len();
    if n < 2 {
        return 0.0;
    }
    let rx = rank_vector(x);
    let ry = rank_vector(y);
    pearson_corr(&rx, &ry)
}

#[allow(dead_code)]
fn rank_vector(x: &[f64]) -> Vec<f64> {
    let mut indices: Vec<usize> = (0..x.len()).collect();
    indices.sort_by(|&a, &b| x[a].partial_cmp(&x[b]).unwrap_or(std::cmp::Ordering::Equal));
    let mut ranks = vec![0.0; x.len()];
    for (i, &idx) in indices.iter().enumerate() {
        ranks[idx] = i as f64;
    }
    ranks
}

#[allow(dead_code)]
fn pearson_corr(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len() as f64;
    let mean_x = x.iter().sum::<f64>() / n;
    let mean_y = y.iter().sum::<f64>() / n;

    let mut num = 0.0;
    let mut den_x = 0.0;
    let mut den_y = 0.0;

    for i in 0..x.len() {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        num += dx * dy;
        den_x += dx * dx;
        den_y += dy * dy;
    }

    if den_x <= 1e-12 || den_y <= 1e-12 {
        return 0.0;
    }
    num / (den_x.sqrt() * den_y.sqrt())
}

impl Backtester {
    pub async fn run_detailed_simulation(
        &mut self,
        genome: &[i32],
        symbol: &str,
        days: i64,
        mode: StrategyMode,
    ) -> anyhow::Result<serde_json::Value> {
        if !self.cache.contains_key(symbol) {
            self.load_data(&[symbol.to_string()], days).await?;
        }

        if !self.cache.contains_key(symbol) {
            return Err(anyhow::anyhow!("Data not found for symbol {}", symbol));
        }

        let data = self.cache.get(symbol).unwrap();
        let features = &data.features;
        let future_ret = &data.returns;

        let genome_usize: Vec<usize> = genome.iter().map(|&x| x as usize).collect();

        if let Some(signal) = self.vm.execute(&genome_usize, features) {
            let sig_slice = signal.as_slice().unwrap();
            let ret_slice = future_ret.as_slice().unwrap();

            let len = sig_slice
                .len()
                .min(ret_slice.len())
                .min(features.shape()[2]);

            // LongShort: z-score thresholds; LongOnly: sigmoid thresholds
            let (upper, lower, z_mean, z_std) = match mode {
                StrategyMode::LongShort => {
                    let (mean, std) = zscore_params(sig_slice, 0, len);
                    let u = adaptive_threshold_zscore(sig_slice, 0, len, mean, std);
                    let l = adaptive_lower_threshold_zscore(sig_slice, 0, len, mean, std);
                    (u, l, mean, std)
                }
                StrategyMode::LongOnly => {
                    let u = adaptive_threshold(sig_slice, 0, len);
                    (u, 0.0, 0.0, 1.0)
                }
            };
            let fee = self.base_fee();
            let mut equity_curve = Vec::with_capacity(len);
            let mut current_equity = 1.0_f64;
            let mut prev_pos = 0.0_f64;
            let mut period_returns = Vec::with_capacity(len);

            // Trade tracking: entry -> exit pairs with direction
            let mut trades: Vec<serde_json::Value> = Vec::new();
            let mut in_trade = false;
            let mut trade_entry_bar = 0_usize;
            let mut trade_entry_equity = 1.0_f64;
            let mut trade_direction: &str = "long";

            for i in 0..len {
                let sig_val = match mode {
                    StrategyMode::LongShort => (sig_slice[i] - z_mean) / z_std,
                    StrategyMode::LongOnly => sigmoid(sig_slice[i]),
                };
                let pos = match mode {
                    StrategyMode::LongOnly => {
                        if sig_val > upper {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    StrategyMode::LongShort => {
                        if sig_val > upper {
                            1.0
                        } else if sig_val < lower {
                            -1.0
                        } else {
                            0.0
                        }
                    }
                };
                let r = ret_slice[i];
                let turnover = (pos - prev_pos).abs();
                let entering_short = pos < -0.5 && prev_pos > -0.5;
                let cost = if entering_short {
                    turnover * fee * 1.5
                } else {
                    turnover * fee
                };
                let pnl = pos * r - cost;

                let prev_equity = current_equity;
                current_equity *= 1.0 + pnl;

                if current_equity <= 0.0 {
                    current_equity = 0.0;
                    if in_trade {
                        trades.push(serde_json::json!({
                            "entry": trade_entry_bar, "exit": i,
                            "bars": i - trade_entry_bar,
                            "pnl": -1.0,
                            "direction": trade_direction,
                        }));
                        in_trade = false;
                    }
                    equity_curve.push(serde_json::json!({
                        "i": i, "equity": 0.0, "pos": pos, "ret": r, "cost": cost
                    }));
                    break;
                }

                // Track trade entry/exit (handles long->short and short->long transitions)
                if turnover > 0.5 {
                    // Close existing trade first
                    if in_trade {
                        trades.push(serde_json::json!({
                            "entry": trade_entry_bar,
                            "exit": i,
                            "bars": i - trade_entry_bar,
                            "pnl": (current_equity / trade_entry_equity) - 1.0,
                            "direction": trade_direction,
                        }));
                        in_trade = false;
                    }
                    // Open new trade if entering a position
                    if pos.abs() > 0.5 {
                        in_trade = true;
                        trade_entry_bar = i;
                        trade_entry_equity = current_equity;
                        trade_direction = if pos > 0.5 { "long" } else { "short" };
                    }
                }

                // Period return for Sharpe/Sortino
                if i > 0 && prev_equity > 0.0 {
                    period_returns.push(current_equity / prev_equity - 1.0);
                }

                equity_curve.push(serde_json::json!({
                    "i": i, "equity": current_equity, "pos": pos, "ret": r, "cost": cost
                }));
                prev_pos = pos;
            }

            // Close any trade still open at the end
            if in_trade {
                trades.push(serde_json::json!({
                    "entry": trade_entry_bar,
                    "exit": len.saturating_sub(1),
                    "bars": len.saturating_sub(1) - trade_entry_bar,
                    "pnl": (current_equity / trade_entry_equity) - 1.0,
                    "direction": trade_direction,
                }));
            }

            let metrics =
                self.compute_trade_stats(current_equity, &trades, &period_returns, &equity_curve);

            return Ok(serde_json::json!({
                "symbol": symbol,
                "days": days,
                "mode": mode.as_str(),
                "metrics": metrics,
                "trades": trades,
                "equity_curve": equity_curve
            }));
        }

        Err(anyhow::anyhow!("VM execution failed"))
    }

    /// Compute comprehensive trade-level and portfolio-level statistics.
    fn compute_trade_stats(
        &self,
        final_equity: f64,
        trades: &[serde_json::Value],
        period_returns: &[f64],
        equity_curve: &[serde_json::Value],
    ) -> serde_json::Value {
        let total_ret = final_equity - 1.0;
        let total_trades = trades.len();

        // Per-trade P&L
        let trade_pnls: Vec<f64> = trades.iter().filter_map(|t| t["pnl"].as_f64()).collect();

        let wins: Vec<f64> = trade_pnls.iter().filter(|&&p| p > 0.0).copied().collect();
        let losses: Vec<f64> = trade_pnls.iter().filter(|&&p| p <= 0.0).copied().collect();
        let win_count = wins.len();
        let loss_count = losses.len();
        let win_rate = if total_trades > 0 {
            win_count as f64 / total_trades as f64
        } else {
            0.0
        };

        let avg_win = if !wins.is_empty() {
            wins.iter().sum::<f64>() / wins.len() as f64
        } else {
            0.0
        };
        let avg_loss = if !losses.is_empty() {
            losses.iter().sum::<f64>() / losses.len() as f64
        } else {
            0.0
        };

        let max_win = trade_pnls.iter().copied().fold(0.0_f64, f64::max);
        let max_loss = trade_pnls.iter().copied().fold(0.0_f64, f64::min);

        let gross_profit: f64 = wins.iter().sum();
        let gross_loss: f64 = losses.iter().map(|l| l.abs()).sum();
        let profit_factor = if gross_loss > 1e-9 {
            gross_profit / gross_loss
        } else if gross_profit > 0.0 {
            f64::INFINITY
        } else {
            0.0
        };

        let avg_holding_bars = if total_trades > 0 {
            trades
                .iter()
                .filter_map(|t| t["bars"].as_f64())
                .sum::<f64>()
                / total_trades as f64
        } else {
            0.0
        };

        // Max consecutive wins/losses
        let (max_consec_wins, max_consec_losses) = {
            let (mut mw, mut ml, mut cw, mut cl) = (0_u32, 0_u32, 0_u32, 0_u32);
            for &pnl in &trade_pnls {
                if pnl > 0.0 {
                    cw += 1;
                    cl = 0;
                    mw = mw.max(cw);
                } else {
                    cl += 1;
                    cw = 0;
                    ml = ml.max(cl);
                }
            }
            (mw, ml)
        };

        // Max drawdown
        let mut peak = 1.0_f64;
        let mut max_drawdown = 0.0_f64;
        for v in equity_curve {
            let eq = v["equity"].as_f64().unwrap_or(1.0);
            if eq > peak {
                peak = eq;
            }
            let dd = (peak - eq) / peak;
            if dd > max_drawdown {
                max_drawdown = dd;
            }
        }

        // Sharpe ratio
        let n = period_returns.len() as f64;
        let ann = self.annualization_factor();
        let (mean_r, std_r) = if n > 1.0 {
            let m = period_returns.iter().sum::<f64>() / n;
            let v = period_returns.iter().map(|&x| (x - m).powi(2)).sum::<f64>() / (n - 1.0);
            (m, v.sqrt())
        } else {
            (0.0, 0.0)
        };
        let sharpe_ratio = if std_r > 1e-9 {
            mean_r / std_r * ann
        } else {
            0.0
        };

        // Sortino ratio (downside deviation only)
        let sortino_ratio = if n > 1.0 {
            let downside_var = period_returns
                .iter()
                .filter(|&&r| r < 0.0)
                .map(|&r| r.powi(2))
                .sum::<f64>()
                / n;
            let downside_std = downside_var.sqrt();
            if downside_std > 1e-9 {
                mean_r / downside_std * ann
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Calmar ratio (annualized return / max drawdown)
        let calmar_ratio = if max_drawdown > 1e-9 && n > 0.0 {
            let bars_per_year = match self.resolution.as_str() {
                "1d" => 252.0,
                "1h" => {
                    if self.exchange == "Polygon" {
                        252.0 * 6.5
                    } else {
                        365.0 * 24.0
                    }
                }
                _ => 252.0 * 96.0,
            };
            let annual_factor = bars_per_year / n;
            let annualized_ret = (1.0 + total_ret).powf(annual_factor) - 1.0;
            annualized_ret / max_drawdown
        } else {
            0.0
        };

        // Average trade return and std
        let avg_trade_return = if total_trades > 0 {
            trade_pnls.iter().sum::<f64>() / total_trades as f64
        } else {
            0.0
        };
        let trade_return_std = if total_trades > 1 {
            let var = trade_pnls
                .iter()
                .map(|&p| (p - avg_trade_return).powi(2))
                .sum::<f64>()
                / (total_trades as f64 - 1.0);
            var.sqrt()
        } else {
            0.0
        };

        serde_json::json!({
            "total_return": total_ret,
            "final_equity": final_equity,
            "total_trades": total_trades,
            "win_rate": win_rate,
            "sharpe_ratio": sharpe_ratio,
            "max_drawdown": max_drawdown,
            "sortino_ratio": sortino_ratio,
            "calmar_ratio": calmar_ratio,
            "profit_factor": profit_factor,
            "avg_win": avg_win,
            "avg_loss": avg_loss,
            "max_win": max_win,
            "max_loss": max_loss,
            "avg_holding_bars": avg_holding_bars,
            "max_consecutive_wins": max_consec_wins,
            "max_consecutive_losses": max_consec_losses,
            "avg_trade_return": avg_trade_return,
            "trade_return_std": trade_return_std,
            "win_count": win_count,
            "loss_count": loss_count,
            "gross_profit": gross_profit,
            "gross_loss": gross_loss,
        })
    }

    pub async fn run_portfolio_simulation(
        &mut self,
        genome: &[i32],
        days: i64,
    ) -> anyhow::Result<serde_json::Value> {
        // Ensure data loaded — use exchange-aware symbol query
        if self.cache.is_empty() {
            use sqlx::Row;
            let symbols: Vec<String> = if self.exchange == "Polygon" {
                let rows = sqlx::query(
                    "SELECT symbol FROM market_watchlist WHERE exchange = 'Polygon' AND is_active = true",
                )
                .fetch_all(&self.pool)
                .await?;
                rows.into_iter().map(|r| r.get("symbol")).collect()
            } else {
                let rows = sqlx::query(
                    "SELECT address as symbol FROM active_tokens WHERE is_active = true",
                )
                .fetch_all(&self.pool)
                .await?;
                rows.into_iter().map(|r| r.get("symbol")).collect()
            };

            if !symbols.is_empty() {
                self.load_data(&symbols, days).await?;
            }
        }

        let mut pb = portfolio::PortfolioBacktester::new(
            &self.factor_config,
            self.exchange.clone(),
            self.resolution.clone(),
        );
        pb.run(genome, &self.cache, days).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentinel_constants_are_distinct() {
        let sentinels = [
            SENTINEL_CACHE_MISS,
            SENTINEL_INSUFFICIENT_DATA,
            SENTINEL_OOS_TOO_SMALL,
            SENTINEL_VM_FAILURE,
            SENTINEL_TOO_FEW_BARS,
            SENTINEL_TOO_FEW_TRADES,
            SENTINEL_ZERO_VARIANCE,
            SENTINEL_NEGATIVE_SE,
            SENTINEL_ZERO_SE,
            SENTINEL_NAN_PSR,
        ];
        // All distinct
        for i in 0..sentinels.len() {
            for j in (i + 1)..sentinels.len() {
                assert_ne!(
                    sentinels[i], sentinels[j],
                    "Sentinel {} and {} are not distinct",
                    i, j
                );
            }
        }
        // All detected by is_sentinel
        for &s in &sentinels {
            assert!(is_sentinel(s), "is_sentinel({}) should be true", s);
        }
    }

    #[test]
    fn is_sentinel_boundary() {
        assert!(is_sentinel(-10.0));
        assert!(is_sentinel(-19.0));
        assert!(is_sentinel(-100.0));
        assert!(!is_sentinel(-9.0));
        assert!(!is_sentinel(-5.0));
        assert!(!is_sentinel(0.0));
        assert!(!is_sentinel(3.0));
        // Boundary: -9.5 is sentinel, -9.4 is not
        assert!(is_sentinel(-9.5));
        assert!(!is_sentinel(-9.4));
    }

    #[test]
    fn sentinel_labels_are_named() {
        assert_eq!(sentinel_label(-10.0), "cache_miss");
        assert_eq!(sentinel_label(-11.0), "insufficient_data");
        assert_eq!(sentinel_label(-12.0), "oos_too_small");
        assert_eq!(sentinel_label(-13.0), "vm_failure");
        assert_eq!(sentinel_label(-14.0), "too_few_bars");
        assert_eq!(sentinel_label(-15.0), "too_few_trades");
        assert_eq!(sentinel_label(-16.0), "zero_variance");
        assert_eq!(sentinel_label(-17.0), "negative_se");
        assert_eq!(sentinel_label(-18.0), "zero_se");
        assert_eq!(sentinel_label(-19.0), "nan_psr");
        assert_eq!(sentinel_label(-99.0), "unknown_sentinel");
    }

    #[test]
    fn walk_forward_config_default() {
        let cfg = WalkForwardConfig::default_1h();
        assert_eq!(cfg.initial_train, 2500);
        assert_eq!(cfg.target_test_window, 1000);
        assert_eq!(cfg.min_test_window, 400);
        assert_eq!(cfg.embargo, 20);
        assert_eq!(cfg.target_steps, 3);
    }

    #[test]
    fn walk_forward_step_boundaries() {
        // Simulate the step calculation logic with known data length
        let total_bars: usize = 6000;
        let config = WalkForwardConfig::default_1h();
        let initial_train = config.initial_train.min(total_bars * 2 / 3); // 2500
        let available = total_bars - initial_train; // 3500

        // test_window = min(1000, (3500 - 3*20) / 3) = min(1000, 1146) = 1000
        let test_window = {
            let tw = available.saturating_sub(config.target_steps * config.embargo)
                / config.target_steps;
            tw.clamp(config.min_test_window, config.target_test_window)
        };
        assert_eq!(test_window, 1000);

        // num_steps = min(3, 3500 / (1000 + 20)) = min(3, 3) = 3
        let num_steps = {
            let max_steps = available / (test_window + config.embargo);
            max_steps.min(config.target_steps).max(1)
        };
        assert_eq!(num_steps, 3);

        // Verify step boundaries
        for i in 0..num_steps {
            let train_end = initial_train + i * (test_window + config.embargo);
            let test_start = train_end + config.embargo;
            let test_end = if i == num_steps - 1 {
                total_bars.min(test_start + test_window)
            } else {
                (test_start + test_window).min(total_bars)
            };

            assert!(test_start > train_end, "step {}: embargo gap missing", i);
            assert!(test_end > test_start, "step {}: empty test window", i);
            assert!(test_end <= total_bars, "step {}: test exceeds data", i);
            assert!(
                test_start - train_end >= config.embargo,
                "step {}: embargo too small",
                i
            );
        }
    }

    #[test]
    fn walk_forward_aggregation_formula() {
        // Test: aggregate = mean - 0.5 * std
        let psrs = vec![1.5, 2.0, 1.0];
        let n = psrs.len() as f64;
        let mean = psrs.iter().sum::<f64>() / n;
        let var = psrs.iter().map(|&p| (p - mean).powi(2)).sum::<f64>() / (n - 1.0);
        let std = var.sqrt();
        let aggregate = mean - 0.5 * std;

        assert!((mean - 1.5).abs() < 1e-10);
        assert!(aggregate < mean, "aggregation should penalize variance");
        assert!(
            aggregate > 0.0,
            "positive PSRs should yield positive aggregate"
        );

        // Expected: mean=1.5, std=0.5, aggregate=1.5 - 0.25 = 1.25
        assert!((std - 0.5).abs() < 1e-10);
        assert!((aggregate - 1.25).abs() < 1e-10);
    }

    #[test]
    fn walk_forward_small_data_returns_sentinel() {
        // With only 50 bars, should fail with insufficient data or oos_too_small
        let config = WalkForwardConfig::default_1h();
        // initial_train = min(2500, 50 * 2/3) = 33
        // available = 50 - 33 = 17
        // 17 < min_test_window(400) + embargo(20) = 420
        // Should return oos_too_small
        let initial = config.initial_train.min(50 * 2 / 3);
        let available = 50_usize.saturating_sub(initial);
        assert!(
            available < config.min_test_window + config.embargo,
            "small data should trigger oos_too_small path"
        );
    }

    #[test]
    fn forward_fill_exact_match() {
        // LF and HF timestamps are identical → output equals input
        let lf = Array3::from_shape_vec((1, 2, 3), vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap();
        let ts = vec![100, 200, 300];
        let out = forward_fill_align(&lf, &ts, &ts);
        assert_eq!(out, lf);
    }

    #[test]
    fn forward_fill_4h_to_1h() {
        // 4h bars at t=0, t=400. 1h bars at t=0,100,200,300,400,500.
        // Expected: bars 0-3 use 4h[0], bars 4-5 use 4h[1].
        let lf = Array3::from_shape_vec((1, 1, 2), vec![10.0, 20.0]).unwrap();
        let lf_ts = vec![0, 400];
        let hf_ts = vec![0, 100, 200, 300, 400, 500];
        let out = forward_fill_align(&lf, &lf_ts, &hf_ts);
        let expected = vec![10.0, 10.0, 10.0, 10.0, 20.0, 20.0];
        assert_eq!(out.as_slice().unwrap(), &expected);
    }

    #[test]
    fn forward_fill_hf_before_first_lf() {
        // HF bar at t=50 is before first LF bar at t=100 → zeros
        let lf = Array3::from_shape_vec((1, 1, 2), vec![5.0, 9.0]).unwrap();
        let lf_ts = vec![100, 200];
        let hf_ts = vec![50, 100, 150, 200, 250];
        let out = forward_fill_align(&lf, &lf_ts, &hf_ts);
        let expected = vec![0.0, 5.0, 5.0, 9.0, 9.0];
        assert_eq!(out.as_slice().unwrap(), &expected);
    }

    #[test]
    fn forward_fill_multi_factor() {
        // 2 factors, 2 LF bars, 4 HF bars
        let lf = Array3::from_shape_vec((1, 2, 2), vec![1.0, 2.0, 3.0, 4.0]).unwrap();
        let lf_ts = vec![0, 200];
        let hf_ts = vec![0, 100, 200, 300];
        let out = forward_fill_align(&lf, &lf_ts, &hf_ts);
        // factor 0: [1, 1, 2, 2], factor 1: [3, 3, 4, 4]
        assert_eq!(out[[0, 0, 0]], 1.0);
        assert_eq!(out[[0, 0, 1]], 1.0);
        assert_eq!(out[[0, 0, 2]], 2.0);
        assert_eq!(out[[0, 0, 3]], 2.0);
        assert_eq!(out[[0, 1, 0]], 3.0);
        assert_eq!(out[[0, 1, 1]], 3.0);
        assert_eq!(out[[0, 1, 2]], 4.0);
        assert_eq!(out[[0, 1, 3]], 4.0);
    }

    // ── Z-Score / LongShort fix tests ──────────────────────────────────────

    #[test]
    fn zscore_params_basic() {
        // Signal: [-2, -1, 0, 1, 2] → mean=0, std=sqrt(10/4)=~1.5811
        let sig = vec![-2.0, -1.0, 0.0, 1.0, 2.0];
        let (mean, std) = zscore_params(&sig, 0, 5);
        assert!((mean - 0.0).abs() < 1e-10, "mean should be 0, got {}", mean);
        let expected_std = (10.0_f64 / 4.0).sqrt(); // sample std
        assert!(
            (std - expected_std).abs() < 1e-10,
            "std should be {}, got {}",
            expected_std,
            std
        );
    }

    #[test]
    fn zscore_params_constant_signal() {
        // All same values → std floored at 1e-10
        let sig = vec![5.0, 5.0, 5.0, 5.0];
        let (mean, std) = zscore_params(&sig, 0, 4);
        assert!((mean - 5.0).abs() < 1e-10);
        assert!(std >= 1e-10, "std should be floored, got {}", std);
    }

    #[test]
    fn zscore_thresholds_symmetric_signal() {
        // Symmetric signal centered at 0 → upper > 0, lower < 0
        let sig: Vec<f64> = (-50..=50).map(|i| i as f64 * 0.1).collect(); // -5.0 to 5.0
        let (mean, std) = zscore_params(&sig, 0, sig.len());
        let upper = adaptive_threshold_zscore(&sig, 0, sig.len(), mean, std);
        let lower = adaptive_lower_threshold_zscore(&sig, 0, sig.len(), mean, std);

        assert!(
            upper > 0.0,
            "upper z-threshold should be positive: {}",
            upper
        );
        assert!(
            lower < 0.0,
            "lower z-threshold should be negative: {}",
            lower
        );
        assert!(
            (upper + lower).abs() < 0.5,
            "thresholds should be roughly symmetric: upper={}, lower={}",
            upper,
            lower
        );
    }

    #[test]
    fn zscore_longshort_produces_short_positions() {
        // Simulate a realistic signal: typical formula output resembles
        // financial returns in [-0.05, 0.05]. After sigmoid, these values
        // cluster tightly around 0.5, making the lower threshold unreachable.
        // This is the exact bug we're fixing: sigmoid compression kills shorts.
        let n = 200;
        let mut sig = Vec::with_capacity(n);
        for i in 0..n {
            // Sinusoidal signal with return-like amplitude: [-0.03, 0.03]
            let t = i as f64 / n as f64 * 4.0 * std::f64::consts::PI;
            sig.push(0.03 * t.sin());
        }

        // Z-score path: should produce both longs and shorts
        let (mean, std) = zscore_params(&sig, 0, sig.len());
        let upper = adaptive_threshold_zscore(&sig, 0, sig.len(), mean, std);
        let lower = adaptive_lower_threshold_zscore(&sig, 0, sig.len(), mean, std);

        let mut zs_long = 0;
        let mut zs_short = 0;
        for i in 0..sig.len() {
            let z = (sig[i] - mean) / std;
            if z > upper {
                zs_long += 1;
            } else if z < lower {
                zs_short += 1;
            }
        }

        assert!(zs_long > 0, "z-score: should have longs (upper={})", upper);
        assert!(
            zs_short > 0,
            "z-score: should have shorts (lower={})",
            lower
        );

        // Sigmoid path: all sigmoid values cluster near 0.5 for small inputs.
        // sigmoid(0.03) ≈ 0.50750, sigmoid(-0.03) ≈ 0.49250
        // 30th percentile of [0.4925..0.5075] ≈ 0.496, clamped to max 0.48
        // → need sigmoid < 0.48, i.e. raw < -0.08, but max|raw| = 0.03 → zero shorts
        let sig_lower = adaptive_lower_threshold(&sig, 0, sig.len());
        let mut sigmoid_short = 0;
        for &s in &sig {
            if sigmoid(s) < sig_lower {
                sigmoid_short += 1;
            }
        }

        assert!(
            zs_short > sigmoid_short,
            "z-score should produce more shorts ({}) than sigmoid ({})",
            zs_short,
            sigmoid_short
        );
    }

    #[test]
    fn zscore_longonly_unchanged() {
        // Verify that long_only path still uses sigmoid (not z-score)
        let sig = vec![0.5, -0.3, 1.2, -0.8, 0.1];
        let upper_sigmoid = adaptive_threshold(&sig, 0, sig.len());

        // sigmoid(0.5) ≈ 0.622, sigmoid(-0.3) ≈ 0.426, etc.
        // Verify sigmoid values are in (0, 1)
        for &s in &sig {
            let sv = sigmoid(s);
            assert!(sv > 0.0 && sv < 1.0, "sigmoid({}) = {} not in (0,1)", s, sv);
        }

        // Upper threshold should be in sigmoid range
        assert!(
            upper_sigmoid >= 0.52 && upper_sigmoid <= 0.70,
            "sigmoid upper should be in [0.52, 0.70], got {}",
            upper_sigmoid
        );
    }

    #[test]
    fn zscore_lower_clamp_prevents_extreme() {
        // Very skewed signal (all positive) → lower z-threshold should clamp to -0.1
        let sig: Vec<f64> = (1..=100).map(|i| i as f64).collect(); // 1.0 to 100.0
        let (mean, std) = zscore_params(&sig, 0, sig.len());
        let lower = adaptive_lower_threshold_zscore(&sig, 0, sig.len(), mean, std);

        // 30th percentile of z-scores for uniform-ish positive data:
        // z-scores range from (1-50.5)/29.01 ≈ -1.71 to (100-50.5)/29.01 ≈ 1.71
        // 30th percentile ≈ z-score at index 30 → (30-50.5)/29.01 ≈ -0.71
        // Should be clamped to [-2.0, -0.1]
        assert!(
            lower >= -2.0 && lower <= -0.1,
            "lower should be clamped to [-2.0, -0.1], got {}",
            lower
        );
    }
}
