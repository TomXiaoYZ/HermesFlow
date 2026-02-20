use crate::genetic::Genome;
use backtest_engine::config::FactorConfig;
use backtest_engine::factors::engineer::FeatureEngineer;
use backtest_engine::vm::vm::StackVM;
use chrono::{DateTime, Utc};
use ndarray::{Array2, Array3};
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use sqlx::FromRow;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

pub mod data_frame;
pub mod portfolio;

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

pub struct Backtester {
    pool: PgPool,
    vm: StackVM,
    cache: HashMap<String, CachedData>,
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
            let ohlcv = backtest_engine::factors::traits::OhlcvData {
                close: &close,
                open: &open,
                high: &high,
                low: &low,
                volume: &volume,
                liquidity: &liq,
                fdv: &fdv,
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

    /// PSR-based evaluation on out-of-sample data (last 30%).
    /// Uses the same PSR metric as K-fold IS fitness for apples-to-apples comparison.
    pub fn evaluate_symbol_oos_psr(
        &self,
        genome: &Genome,
        symbol: &str,
        mode: StrategyMode,
    ) -> f64 {
        let data = match self.cache.get(symbol) {
            Some(d) => d,
            None => return -10.0,
        };

        if let Some(signal) = self.vm.execute(&genome.tokens, &data.features) {
            let sig_slice = signal.as_slice().unwrap();
            let ret_slice = data.returns.as_slice().unwrap();

            let len = sig_slice.len().min(ret_slice.len());
            if len < 60 {
                return -10.0;
            }

            let split_idx = (len as f64 * 0.7).max(30.0) as usize;
            if split_idx >= len || len - split_idx < 30 {
                return -10.0;
            }

            self.psr_fitness(sig_slice, ret_slice, split_idx, len, mode)
        } else {
            -10.0
        }
    }

    /// Resolution-aware embargo size (bars to skip at fold boundaries).
    /// Prevents information leakage from TS operators carrying state across folds.
    fn embargo_size(&self) -> usize {
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
            genome.fitness = -10.0;
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
        let upper = adaptive_threshold(sig, start, end);
        let lower = match mode {
            StrategyMode::LongShort => adaptive_lower_threshold(sig, start, end),
            StrategyMode::LongOnly => 0.0,
        };
        let fee = self.base_fee();
        let mut prev_pos = 0.0_f64;
        let mut bar_returns = Vec::with_capacity(n);
        let mut trade_count = 0_u32;
        let mut active_bars = 0_u32;

        for i in start..end {
            let raw = sig[i];
            let sig_val = sigmoid(raw);
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

        let upper = adaptive_threshold(sig, start, end);
        let lower = match mode {
            StrategyMode::LongShort => adaptive_lower_threshold(sig, start, end),
            StrategyMode::LongOnly => 0.0, // unused
        };
        let fee = self.base_fee();
        let mut prev_pos = 0.0_f64;
        let mut cum_pnl = 0.0_f64;
        let mut trade_count = 0_u32;
        let mut active_bars = 0_u32;
        let mut big_loss_count = 0_u32;

        for i in start..end {
            let raw = sig[i];
            let sig_val = sigmoid(raw);
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
/// clamped to [0.52, 0.80]. Goes long on top ~30% of signals.
fn adaptive_threshold(sig: &[f64], start: usize, end: usize) -> f64 {
    let mut vals: Vec<f64> = (start..end).map(|i| sigmoid(sig[i])).collect();
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if vals.is_empty() {
        return 0.65;
    }
    let idx = ((vals.len() as f64) * 0.70) as usize;
    vals[idx.min(vals.len() - 1)].clamp(0.52, 0.80)
}

/// Compute an adaptive lower sigmoid threshold as the 30th percentile of sigmoid(signal),
/// clamped to [0.20, 0.48]. Goes short on bottom ~30% of signals.
fn adaptive_lower_threshold(sig: &[f64], start: usize, end: usize) -> f64 {
    let mut vals: Vec<f64> = (start..end).map(|i| sigmoid(sig[i])).collect();
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if vals.is_empty() {
        return 0.35;
    }
    let idx = ((vals.len() as f64) * 0.30) as usize;
    vals[idx.min(vals.len() - 1)].clamp(0.20, 0.48)
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

            let upper = adaptive_threshold(sig_slice, 0, len);
            let lower = match mode {
                StrategyMode::LongShort => adaptive_lower_threshold(sig_slice, 0, len),
                StrategyMode::LongOnly => 0.0,
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
                let sig_val = sigmoid(sig_slice[i]);
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
