use crate::genetic::Genome;
use backtest_engine::config::FactorConfig;
use backtest_engine::factors::engineer::FeatureEngineer;
use backtest_engine::vm::vm::StackVM;
use chrono::{DateTime, Utc};
use ndarray::{Array2, Array3};
use rust_decimal::prelude::ToPrimitive;
use sqlx::postgres::PgPool;
use sqlx::FromRow;
use std::collections::HashMap;

pub mod data_frame;
pub mod portfolio;

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
    pub liquidity: Array2<f64>,
    pub amount: Array2<f64>,
    pub timestamps: Vec<i64>,
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum OptimizationMetric {
    Sharpe,
    IC,
}

pub struct Backtester {
    pool: PgPool,
    vm: StackVM,
    cache: HashMap<String, CachedData>,
    factor_config: FactorConfig,
    #[allow(dead_code)]
    pub metric: OptimizationMetric,
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
        Self {
            pool,
            vm: StackVM::from_config(&factor_config),
            cache: HashMap::new(),
            factor_config,
            metric: {
                tracing::info!("Initializing Backtester with OptimizationMetric::IC");
                OptimizationMetric::IC
            },
            exchange,
            resolution,
        }
    }

    /// Annualization factor for Sharpe ratio based on resolution and exchange.
    #[allow(dead_code)]
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
    fn capacity(&self, liquidity: f64, amount: f64) -> f64 {
        if self.exchange == "Polygon" {
            // Stocks: use amount (≈ volume * price) as capacity proxy, $1M floor for liquid stocks
            amount.max(1e6)
        } else if liquidity > 0.0 {
            liquidity
        } else {
            amount * 0.1
        }
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
            let mut max_price = -f64::INFINITY;
            let mut min_price = f64::INFINITY;
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
            let mut max_ret = -f64::INFINITY;
            for i in 0..len - 1 {
                let curr = close[[0, i]];
                let next = close[[0, i + 1]];
                let r = if curr.abs() > 1e-9 {
                    let raw_r = next / curr - 1.0;
                    raw_r.clamp(-0.99, 10.0)
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

    /// Evaluate a genome on a single symbol's in-sample data (first 70%).
    pub fn evaluate_symbol(&self, genome: &mut Genome, symbol: &str) {
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
            if len < 2 {
                genome.fitness = -1000.0;
                return;
            }

            let split_idx = (len as f64 * 0.7) as usize;
            let split_idx = split_idx.max(2);

            let s = &sig_slice[..split_idx];
            let r = &ret_slice[..split_idx];

            let ic = spearman_rank_corr(s, r);
            genome.fitness = if ic.is_nan() { -999.0 } else { ic };
        } else {
            genome.fitness = -1000.0;
        }
    }

    /// Evaluate a genome on a single symbol's out-of-sample data (last 30%).
    pub fn evaluate_symbol_oos(&self, genome: &Genome, symbol: &str) -> f64 {
        let data = match self.cache.get(symbol) {
            Some(d) => d,
            None => return 0.0,
        };

        if let Some(signal) = self.vm.execute(&genome.tokens, &data.features) {
            let sig_slice = signal.as_slice().unwrap();
            let ret_slice = data.returns.as_slice().unwrap();

            let len = sig_slice.len().min(ret_slice.len());
            if len < 4 {
                return 0.0;
            }

            let split_idx = (len as f64 * 0.7) as usize;
            let split_idx = split_idx.max(2);
            if split_idx >= len {
                return 0.0;
            }

            let s = &sig_slice[split_idx..len];
            let r = &ret_slice[split_idx..len];

            if s.len() < 2 {
                return 0.0;
            }

            let ic = spearman_rank_corr(s, r);
            if ic.is_nan() {
                0.0
            } else {
                ic
            }
        } else {
            0.0
        }
    }

    /// Evaluate a genome using in-sample data (first 70%) across all cached symbols.
    #[allow(dead_code)]
    pub fn evaluate(&self, genome: &mut Genome) {
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
                if len < 2 {
                    continue;
                }

                // Train/test split: use first 70% for fitness (in-sample)
                let split_idx = (len as f64 * 0.7) as usize;
                let split_idx = split_idx.max(2); // Ensure at least 2 data points

                let s = &sig_slice[..split_idx];
                let r = &ret_slice[..split_idx];

                match self.metric {
                    OptimizationMetric::IC => {
                        let ic = spearman_rank_corr(s, r);
                        total_score += ic;
                        valid_count += 1.0;
                    }
                    OptimizationMetric::Sharpe => {
                        let liq_slice = data.liquidity.as_slice().unwrap();
                        let amt_slice = data.amount.as_slice().unwrap();

                        let mut strat_returns = Vec::with_capacity(split_idx);
                        let mut trade_count = 0;
                        let mut prev_sig = 0.0;
                        let portfolio_size = 10_000.0;
                        let fee = self.base_fee();

                        for i in 0..split_idx {
                            let mut val_s = s[i].clamp(-1.0, 1.0);
                            let val_r = r[i];
                            let liq = liq_slice[i.min(liq_slice.len() - 1)];
                            let amt = amt_slice[i.min(amt_slice.len() - 1)];

                            let cap = self.capacity(liq, amt);
                            let trade_val = val_s.abs() * portfolio_size;
                            let impact = trade_val / (cap + 1e-9);

                            if impact > 0.05 {
                                val_s = 0.0;
                            }

                            if (val_s - prev_sig).abs() > 0.1 {
                                trade_count += 1;
                            }

                            let turnover = (val_s - prev_sig).abs();
                            let cost = if turnover > 0.0 {
                                turnover * (fee + impact)
                            } else {
                                0.0
                            };

                            let pnl = val_s * val_r - cost;
                            strat_returns.push(pnl);
                            prev_sig = val_s;
                        }

                        if trade_count < 5 {
                            total_score -= 5.0;
                            valid_count += 1.0;
                            continue;
                        }

                        let n = strat_returns.len() as f64;
                        let mean_ret: f64 = strat_returns.iter().sum::<f64>() / n;
                        let var_ret: f64 = strat_returns
                            .iter()
                            .map(|x| (x - mean_ret).powi(2))
                            .sum::<f64>()
                            / (n - 1.0);
                        let std_ret = var_ret.sqrt();

                        let sharpe = if std_ret > 1e-6 {
                            mean_ret / std_ret * self.annualization_factor()
                        } else {
                            0.0
                        };

                        total_score += sharpe;
                        valid_count += 1.0;
                    }
                }
            }
            // VM failure: skip symbol instead of aborting entire evaluation
        }

        // Require at least 20% of symbols to produce valid signals
        if valid_count < 1.0 || valid_count / total_symbols < 0.2 {
            genome.fitness = -1000.0;
            return;
        }

        let avg_score = total_score / valid_count;
        if avg_score.is_nan() {
            genome.fitness = -999.0;
        } else {
            genome.fitness = avg_score;
        }
    }

    /// Evaluate a genome on out-of-sample data (last 30%) across all cached symbols.
    #[allow(dead_code)]
    pub fn evaluate_oos(&self, genome: &Genome) -> f64 {
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
                if len < 4 {
                    continue;
                }

                let split_idx = (len as f64 * 0.7) as usize;
                let split_idx = split_idx.max(2);
                if split_idx >= len {
                    continue;
                }

                let s = &sig_slice[split_idx..len];
                let r = &ret_slice[split_idx..len];

                if s.len() < 2 {
                    continue;
                }

                let ic = spearman_rank_corr(s, r);
                total_score += ic;
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

fn spearman_rank_corr(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len();
    if n < 2 {
        return 0.0;
    }
    let rx = rank_vector(x);
    let ry = rank_vector(y);
    pearson_corr(&rx, &ry)
}

fn rank_vector(x: &[f64]) -> Vec<f64> {
    let mut indices: Vec<usize> = (0..x.len()).collect();
    indices.sort_by(|&a, &b| x[a].partial_cmp(&x[b]).unwrap_or(std::cmp::Ordering::Equal));
    let mut ranks = vec![0.0; x.len()];
    for (i, &idx) in indices.iter().enumerate() {
        ranks[idx] = i as f64;
    }
    ranks
}

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
        let liquidity = &data.liquidity;
        let amount = &data.amount;

        let genome_usize: Vec<usize> = genome.iter().map(|&x| x as usize).collect();

        if let Some(signal) = self.vm.execute(&genome_usize, features) {
            let sig_slice = signal.as_slice().unwrap();
            let ret_slice = future_ret.as_slice().unwrap();

            let len = sig_slice
                .len()
                .min(ret_slice.len())
                .min(features.shape()[2]);

            let mut equity_curve = Vec::with_capacity(len);
            let mut current_equity = 1.0;
            let mut win = 0;
            let mut total = 0;
            let mut prev_sig = 0.0;
            let portfolio_size = 10_000.0;
            let fee = self.base_fee();

            for i in 0..len {
                let mut s = sig_slice[i].clamp(-1.0, 1.0);
                let r = ret_slice[i];
                let liq = liquidity[[0, i]];
                let amt = amount[[0, i]];

                let cap = self.capacity(liq, amt);

                let trade_val = s.abs() * portfolio_size;
                let impact = trade_val / (cap + 1e-9);

                if impact > 0.05 {
                    s = 0.0;
                }

                let turnover = (s - prev_sig).abs();
                let cost = if turnover > 0.0 {
                    turnover * (fee + impact)
                } else {
                    0.0
                };

                let pnl = s * r - cost;

                current_equity *= 1.0 + pnl;

                if current_equity <= 0.0 {
                    current_equity = 0.0;
                    break;
                }

                if turnover > 0.0 && (s - prev_sig).abs() > 0.1 {
                    total += 1;
                    if pnl > 0.0 {
                        win += 1;
                    }
                }

                equity_curve.push(serde_json::json!({
                    "i": i,
                    "equity": current_equity,
                    "sig": s,
                    "ret": r,
                    "cost": cost,
                    "impact": impact
                }));
                prev_sig = s;
            }

            let total_ret = current_equity - 1.0;
            let win_rate = if total > 0 {
                win as f64 / total as f64
            } else {
                0.0
            };

            // Compute Sharpe ratio and max drawdown from equity curve
            let mut max_equity = 1.0;
            let mut max_drawdown = 0.0;
            let mut period_returns = Vec::with_capacity(equity_curve.len());

            for i in 0..equity_curve.len() {
                let eq = equity_curve[i]["equity"].as_f64().unwrap_or(1.0);
                if eq > max_equity {
                    max_equity = eq;
                }
                let dd = (max_equity - eq) / max_equity;
                if dd > max_drawdown {
                    max_drawdown = dd;
                }
                if i > 0 {
                    let prev_eq = equity_curve[i - 1]["equity"].as_f64().unwrap_or(1.0);
                    if prev_eq > 0.0 {
                        period_returns.push(eq / prev_eq - 1.0);
                    }
                }
            }

            let n = period_returns.len() as f64;
            let sharpe_ratio = if n > 1.0 {
                let mean_r = period_returns.iter().sum::<f64>() / n;
                let var_r = period_returns
                    .iter()
                    .map(|&x| (x - mean_r).powi(2))
                    .sum::<f64>()
                    / (n - 1.0);
                let std_r = var_r.sqrt();
                if std_r > 1e-9 {
                    mean_r / std_r * 252.0_f64.sqrt()
                } else {
                    0.0
                }
            } else {
                0.0
            };

            return Ok(serde_json::json!({
                "symbol": symbol,
                "days": days,
                "metrics": {
                    "total_return": total_ret,
                    "final_equity": current_equity,
                    "win_rate": win_rate,
                    "total_trades": total,
                    "sharpe_ratio": sharpe_ratio,
                    "max_drawdown": max_drawdown
                },
                "equity_curve": equity_curve
            }));
        }

        Err(anyhow::anyhow!("VM execution failed"))
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
