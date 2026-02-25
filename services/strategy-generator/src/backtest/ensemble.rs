use crate::backtest::{
    adaptive_lower_threshold_zscore, adaptive_threshold, adaptive_threshold_zscore, sigmoid,
    zscore_params, CachedData, StrategyMode,
};
use backtest_engine::vm::vm::StackVM;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use sqlx::Row;
use std::fmt;
use tracing::warn;

// ── Types ──────────────────────────────────────────────────────────────

/// Unique identifier for a strategy within the ensemble.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StrategyId {
    pub exchange: String,
    pub symbol: String,
    pub mode: String,
    pub generation: i32,
}

impl fmt::Display for StrategyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}:gen{}",
            self.exchange, self.symbol, self.mode, self.generation
        )
    }
}

/// A candidate strategy loaded from DB for potential inclusion in the ensemble.
#[derive(Debug, Clone)]
pub struct StrategyCandidate {
    pub id: StrategyId,
    pub genome: Vec<i32>,
    pub oos_psr: f64,
    pub is_fitness: f64,
    pub utilization: f64,
    pub walk_forward_steps: usize,
}

/// Configuration controlling ensemble candidate selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnsembleConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_min_oos_psr")]
    pub min_oos_psr: f64,
    #[serde(default = "default_min_wf_steps")]
    pub min_wf_steps: usize,
    #[serde(default = "default_min_utilization")]
    pub min_utilization: f64,
    #[serde(default = "default_max_strategies_per_symbol")]
    pub max_strategies_per_symbol: usize,
    #[serde(default = "default_max_total_strategies")]
    pub max_total_strategies: usize,
    #[serde(default = "default_correlation_lookback_bars")]
    pub correlation_lookback_bars: usize,
    #[serde(default = "default_rebalance_interval_minutes")]
    pub rebalance_interval_minutes: u64,
    #[serde(default)]
    pub dynamic_weights: DynamicWeightYamlConfig,
    /// P6a-F1: Covariance estimation method for HRP allocation.
    #[serde(default)]
    pub covariance_method: super::hrp::CovarianceMethod,
    /// P6a-F2: Turnover cost rate (fraction of portfolio value per unit turnover).
    #[serde(default = "default_turnover_cost_rate")]
    pub turnover_cost_rate: f64,
    /// P6b-F3: Enable regime-aware rebalancing.
    #[serde(default)]
    pub regime_aware: bool,
    /// P6b-F3: Annualized vol thresholds [low_upper, normal_upper].
    #[serde(default = "default_regime_thresholds")]
    pub regime_thresholds: [f64; 2],
    /// P6b-F3: Rebalance intervals (minutes) per regime [low, normal, high].
    #[serde(default = "default_regime_intervals")]
    pub regime_intervals: [u64; 3],
    /// P6b-C1: Deadzone + L1 regularization for turnover suppression.
    #[serde(default)]
    pub deadzone: DeadzoneConfig,
}

impl Default for EnsembleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_oos_psr: default_min_oos_psr(),
            min_wf_steps: default_min_wf_steps(),
            min_utilization: default_min_utilization(),
            max_strategies_per_symbol: default_max_strategies_per_symbol(),
            max_total_strategies: default_max_total_strategies(),
            correlation_lookback_bars: default_correlation_lookback_bars(),
            rebalance_interval_minutes: default_rebalance_interval_minutes(),
            dynamic_weights: DynamicWeightYamlConfig::default(),
            covariance_method: super::hrp::CovarianceMethod::default(),
            turnover_cost_rate: default_turnover_cost_rate(),
            regime_aware: false,
            regime_thresholds: default_regime_thresholds(),
            regime_intervals: default_regime_intervals(),
            deadzone: DeadzoneConfig::default(),
        }
    }
}

/// YAML sub-config for dynamic weight adjustment (deserialized separately from runtime struct).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicWeightYamlConfig {
    #[serde(default = "default_psr_reward_scale")]
    pub psr_reward_scale: f64,
    #[serde(default = "default_psr_max_reward")]
    pub psr_max_reward: f64,
    #[serde(default = "default_utilization_floor")]
    pub utilization_floor: f64,
    #[serde(default = "default_crowding_corr_threshold")]
    pub crowding_corr_threshold: f64,
    #[serde(default = "default_crowding_penalty_rate")]
    pub crowding_penalty_rate: f64,
    #[serde(default = "default_crowding_max_penalty")]
    pub crowding_max_penalty: f64,
}

impl Default for DynamicWeightYamlConfig {
    fn default() -> Self {
        Self {
            psr_reward_scale: default_psr_reward_scale(),
            psr_max_reward: default_psr_max_reward(),
            utilization_floor: default_utilization_floor(),
            crowding_corr_threshold: default_crowding_corr_threshold(),
            crowding_penalty_rate: default_crowding_penalty_rate(),
            crowding_max_penalty: default_crowding_max_penalty(),
        }
    }
}

/// P6b-C1: Deadzone + L1 regularization config for turnover suppression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadzoneConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_deadzone_threshold")]
    pub threshold: f64,
    #[serde(default = "default_deadzone_l1_lambda")]
    pub l1_lambda: f64,
    #[serde(default)]
    pub vol_adaptive: bool,
}

impl Default for DeadzoneConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold: default_deadzone_threshold(),
            l1_lambda: default_deadzone_l1_lambda(),
            vol_adaptive: false,
        }
    }
}

fn default_deadzone_threshold() -> f64 {
    0.02
}
fn default_deadzone_l1_lambda() -> f64 {
    0.01
}
fn default_enabled() -> bool {
    true
}
fn default_min_oos_psr() -> f64 {
    0.5
}
fn default_min_wf_steps() -> usize {
    2
}
fn default_min_utilization() -> f64 {
    0.10
}
fn default_max_strategies_per_symbol() -> usize {
    1
}
fn default_max_total_strategies() -> usize {
    20
}
fn default_correlation_lookback_bars() -> usize {
    500
}
fn default_rebalance_interval_minutes() -> u64 {
    30
}
fn default_turnover_cost_rate() -> f64 {
    0.0001 // 1 bps
}
fn default_regime_thresholds() -> [f64; 2] {
    [0.15, 0.30]
}
fn default_regime_intervals() -> [u64; 3] {
    [240, 60, 15]
}
fn default_psr_reward_scale() -> f64 {
    0.2
}
fn default_psr_max_reward() -> f64 {
    3.0
}
fn default_utilization_floor() -> f64 {
    0.3
}
fn default_crowding_corr_threshold() -> f64 {
    0.7
}
fn default_crowding_penalty_rate() -> f64 {
    0.3
}
fn default_crowding_max_penalty() -> f64 {
    0.8
}

// ── Regime Detection ──────────────────────────────────────────────────

/// Volatility regime for adaptive rebalance frequency (P6b-F3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum VolRegime {
    Low = 0,
    Normal = 1,
    High = 2,
}

impl VolRegime {
    /// Regime-adaptive multiplier for deadzone L1 lambda (P6b-C1).
    ///
    /// Higher in calm markets (more turnover suppression),
    /// lower in volatile markets (faster adaptation).
    pub fn deadzone_multiplier(&self) -> f64 {
        match self {
            VolRegime::Low => 2.0,
            VolRegime::Normal => 1.0,
            VolRegime::High => 0.5,
        }
    }
}

impl fmt::Display for VolRegime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VolRegime::Low => write!(f, "Low"),
            VolRegime::Normal => write!(f, "Normal"),
            VolRegime::High => write!(f, "High"),
        }
    }
}

/// Result of volatility regime detection.
#[derive(Debug, Clone, Copy)]
pub struct RegimeInfo {
    pub regime: VolRegime,
    pub annualized_vol: f64,
}

/// Detect the current volatility regime from recent portfolio returns.
///
/// Computes annualized volatility from the last `window` returns,
/// then classifies into Low/Normal/High based on thresholds.
///
/// `resolution` controls the annualization factor: "1h" uses √(252×6.5).
pub fn detect_regime(
    portfolio_returns: &[f64],
    resolution: &str,
    window: usize,
    thresholds: [f64; 2],
) -> RegimeInfo {
    let n = portfolio_returns.len();
    let start = n.saturating_sub(window);
    let recent = &portfolio_returns[start..];

    if recent.len() < 5 {
        return RegimeInfo {
            regime: VolRegime::Normal,
            annualized_vol: 0.0,
        };
    }

    let mean = recent.iter().sum::<f64>() / recent.len() as f64;
    let var = recent
        .iter()
        .map(|r| (r - mean).powi(2))
        .sum::<f64>()
        / (recent.len() as f64 - 1.0);
    let std = var.sqrt();

    // Annualize: bars_per_year depends on resolution
    let bars_per_year: f64 = match resolution {
        "1d" => 252.0,
        "4h" => 252.0 * 6.5 / 4.0,
        "1h" => 252.0 * 6.5,
        "15m" => 252.0 * 6.5 * 4.0,
        _ => 252.0 * 6.5,
    };
    let annualized_vol = std * bars_per_year.sqrt();

    let regime = if annualized_vol < thresholds[0] {
        VolRegime::Low
    } else if annualized_vol < thresholds[1] {
        VolRegime::Normal
    } else {
        VolRegime::High
    };

    RegimeInfo {
        regime,
        annualized_vol,
    }
}

// ── DB Loading ─────────────────────────────────────────────────────────

/// Load the latest-generation best strategy per (symbol, mode) from the DB.
///
/// Extracts OOS PSR, utilization, and walk-forward step count from the
/// metadata JSONB column stored by the evolution loop.
pub async fn load_candidates_from_db(
    pool: &PgPool,
    exchange: &str,
) -> anyhow::Result<Vec<StrategyCandidate>> {
    // Get the latest generation per (symbol, mode) using DISTINCT ON
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT ON (symbol, mode)
               exchange, symbol, mode, generation, fitness, best_genome, metadata
        FROM strategy_generations
        WHERE exchange = $1
        ORDER BY symbol, mode, generation DESC
        "#,
    )
    .bind(exchange)
    .fetch_all(pool)
    .await?;

    let mut candidates = Vec::with_capacity(rows.len());
    for row in &rows {
        let symbol: String = row.get("symbol");
        let mode: String = row.get("mode");
        let generation: i32 = row.get("generation");
        let is_fitness: f64 = row.get("fitness");
        let genome: Vec<i32> = row.get("best_genome");
        let metadata: serde_json::Value = row.get("metadata");

        // Extract OOS PSR from metadata -> walk_forward -> mean_psr
        let oos_psr = metadata
            .get("walk_forward")
            .and_then(|wf| wf.get("mean_psr"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        // Extract utilization from metadata -> utilization -> total_utilization
        let utilization = metadata
            .get("utilization")
            .and_then(|u| u.get("total_utilization"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        // Extract walk-forward step count from metadata -> walk_forward -> num_valid
        let walk_forward_steps = metadata
            .get("walk_forward")
            .and_then(|wf| wf.get("num_valid"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        candidates.push(StrategyCandidate {
            id: StrategyId {
                exchange: exchange.to_string(),
                symbol,
                mode,
                generation,
            },
            genome,
            oos_psr,
            is_fitness,
            utilization,
            walk_forward_steps,
        });
    }

    Ok(candidates)
}

// ── Selection ──────────────────────────────────────────────────────────

/// Filter and rank candidates according to ensemble config.
///
/// Filters: min OOS PSR, min utilization, min walk-forward steps.
/// Then keeps the top candidate per symbol (by OOS PSR desc),
/// and caps total strategies at max_total_strategies.
pub fn select_candidates(
    candidates: Vec<StrategyCandidate>,
    config: &EnsembleConfig,
) -> Vec<StrategyCandidate> {
    // Filter by quality thresholds
    let mut filtered: Vec<StrategyCandidate> = candidates
        .into_iter()
        .filter(|c| {
            c.oos_psr >= config.min_oos_psr
                && c.utilization >= config.min_utilization
                && c.walk_forward_steps >= config.min_wf_steps
        })
        .collect();

    // Sort by OOS PSR descending (best first)
    filtered.sort_by(|a, b| {
        b.oos_psr
            .partial_cmp(&a.oos_psr)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Limit per symbol: keep top N per (symbol)
    let mut per_symbol_count: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    let mut selected: Vec<StrategyCandidate> = Vec::new();

    for c in filtered {
        let count = per_symbol_count.entry(c.id.symbol.clone()).or_insert(0);
        if *count < config.max_strategies_per_symbol {
            *count += 1;
            selected.push(c);
        }
    }

    // Cap total
    selected.truncate(config.max_total_strategies);

    selected
}

// ── Return Extraction ──────────────────────────────────────────────────

/// Replay a genome's signal through the position/cost model to produce per-bar returns.
///
/// Uses the same signal→position→PnL logic as `psr_fitness` to ensure consistency
/// between evolution evaluation and ensemble allocation.
///
/// Returns `None` if the VM fails to execute the genome or data is insufficient.
#[allow(clippy::too_many_arguments)]
pub fn extract_strategy_returns(
    vm: &StackVM,
    genome: &[i32],
    data: &CachedData,
    mode: StrategyMode,
    threshold_config: &crate::backtest::ThresholdConfig,
    symbol: &str,
    exchange: &str,
    lookback: usize,
) -> Option<Vec<f64>> {
    let tokens: Vec<usize> = genome.iter().map(|&t| t as usize).collect();
    let sig_2d = vm.execute(&tokens, &data.features)?;
    let n_bars = data.returns.shape()[1];

    if n_bars < 30 {
        return None;
    }

    // Use the last `lookback` bars (or all if lookback > n_bars)
    let start = if lookback > 0 && lookback < n_bars {
        n_bars - lookback
    } else {
        0
    };
    let end = n_bars;

    // Build signal and return slices
    let sig: Vec<f64> = (start..end).map(|i| sig_2d[[0, i]]).collect();
    let ret: Vec<f64> = (start..end).map(|i| data.returns[[0, i]]).collect();

    let n = sig.len();
    if n < 30 {
        return None;
    }

    // Resolve thresholds (same logic as psr_fitness)
    let upper_params = threshold_config.resolve_upper(symbol, mode);
    let lower_params = threshold_config.resolve_lower(symbol);

    // We need to pass the original signal array for threshold computation.
    // Reconstruct a contiguous signal array for the adaptive threshold functions.
    let (upper, lower, z_mean, z_std) = match mode {
        StrategyMode::LongShort => {
            let (mean, std) = zscore_params(&sig, 0, n);
            let u = adaptive_threshold_zscore(&sig, 0, n, mean, std, &upper_params);
            let l = adaptive_lower_threshold_zscore(&sig, 0, n, mean, std, &lower_params);
            (u, l, mean, std)
        }
        StrategyMode::LongOnly => {
            let u = adaptive_threshold(&sig, 0, n, &upper_params);
            (u, 0.0, 0.0, 1.0)
        }
    };

    // Transaction fee
    let fee = if exchange == "Polygon" { 0.0001 } else { 0.001 };

    // Position simulation (mirrors psr_fitness exactly)
    let mut prev_pos = 0.0_f64;
    let mut bar_returns = Vec::with_capacity(n);

    for i in 0..n {
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

        prev_pos = pos;
    }

    if bar_returns.is_empty() {
        warn!("Empty bar returns for {}", symbol);
        return None;
    }

    Some(bar_returns)
}

// ── Deployment ────────────────────────────────────────────────────────

/// UPSERT selected strategies into deployed_strategies table.
///
/// For each strategy in the current ensemble, inserts or updates its row
/// in deployed_strategies with status='active'. Previously active strategies
/// for this exchange that are no longer in the ensemble are set to 'replaced'.
pub async fn upsert_deployed_strategies(
    pool: &PgPool,
    exchange: &str,
    candidates: &[StrategyCandidate],
    weights: &[super::ensemble_weights::WeightAdjustment],
    ensemble_version: i32,
    threshold_config: &serde_json::Value,
) -> anyhow::Result<()> {
    // Collect (symbol, mode) pairs from current ensemble
    let active_keys: std::collections::HashSet<(String, String)> = candidates
        .iter()
        .map(|c| (c.id.symbol.clone(), c.id.mode.clone()))
        .collect();

    // Mark previously active strategies as 'replaced' if no longer in ensemble
    sqlx::query(
        "UPDATE deployed_strategies \
         SET status = 'replaced', updated_at = NOW() \
         WHERE exchange = $1 AND status = 'active' \
           AND (symbol, mode) NOT IN (SELECT * FROM UNNEST($2::text[], $3::text[]))",
    )
    .bind(exchange)
    .bind(
        active_keys
            .iter()
            .map(|(s, _)| s.clone())
            .collect::<Vec<_>>(),
    )
    .bind(
        active_keys
            .iter()
            .map(|(_, m)| m.clone())
            .collect::<Vec<_>>(),
    )
    .execute(pool)
    .await?;

    // UPSERT each active strategy
    for (i, c) in candidates.iter().enumerate() {
        let final_weight = weights.get(i).map(|w| w.final_weight).unwrap_or(0.0);

        sqlx::query(
            "INSERT INTO deployed_strategies \
             (exchange, symbol, mode, genome, generation, threshold_config, \
              oos_psr, is_fitness, utilization, final_weight, ensemble_version, \
              status, deployed_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 'active', NOW(), NOW()) \
             ON CONFLICT (exchange, symbol, mode) DO UPDATE SET \
              genome = EXCLUDED.genome, \
              generation = EXCLUDED.generation, \
              threshold_config = EXCLUDED.threshold_config, \
              oos_psr = EXCLUDED.oos_psr, \
              is_fitness = EXCLUDED.is_fitness, \
              utilization = EXCLUDED.utilization, \
              final_weight = EXCLUDED.final_weight, \
              ensemble_version = EXCLUDED.ensemble_version, \
              status = 'active', \
              updated_at = NOW()",
        )
        .bind(exchange)
        .bind(&c.id.symbol)
        .bind(&c.id.mode)
        .bind(&c.genome)
        .bind(c.id.generation)
        .bind(threshold_config)
        .bind(c.oos_psr)
        .bind(c.is_fitness)
        .bind(c.utilization)
        .bind(final_weight)
        .bind(ensemble_version)
        .execute(pool)
        .await?;
    }

    Ok(())
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candidate(
        symbol: &str,
        mode: &str,
        oos_psr: f64,
        utilization: f64,
        wf_steps: usize,
    ) -> StrategyCandidate {
        StrategyCandidate {
            id: StrategyId {
                exchange: "Polygon".to_string(),
                symbol: symbol.to_string(),
                mode: mode.to_string(),
                generation: 100,
            },
            genome: vec![1, 2, 3],
            oos_psr,
            is_fitness: 1.0,
            utilization,
            walk_forward_steps: wf_steps,
        }
    }

    fn default_config() -> EnsembleConfig {
        EnsembleConfig {
            min_oos_psr: 0.5,
            min_wf_steps: 2,
            min_utilization: 0.10,
            max_strategies_per_symbol: 1,
            max_total_strategies: 20,
            ..Default::default()
        }
    }

    #[test]
    fn select_filters_low_psr() {
        let candidates = vec![
            make_candidate("AAPL", "long_only", 0.3, 0.5, 3), // below min PSR
            make_candidate("GOOG", "long_only", 0.8, 0.5, 3), // passes
        ];
        let result = select_candidates(candidates, &default_config());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id.symbol, "GOOG");
    }

    #[test]
    fn select_filters_low_utilization() {
        let candidates = vec![
            make_candidate("AAPL", "long_only", 1.0, 0.05, 3), // below min util
            make_candidate("GOOG", "long_only", 1.0, 0.5, 3),  // passes
        ];
        let result = select_candidates(candidates, &default_config());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id.symbol, "GOOG");
    }

    #[test]
    fn select_filters_insufficient_wf_steps() {
        let candidates = vec![
            make_candidate("AAPL", "long_only", 1.0, 0.5, 1), // below min steps
            make_candidate("GOOG", "long_only", 1.0, 0.5, 3), // passes
        ];
        let result = select_candidates(candidates, &default_config());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id.symbol, "GOOG");
    }

    #[test]
    fn select_per_symbol_limit() {
        let candidates = vec![
            make_candidate("AAPL", "long_only", 2.0, 0.5, 3),
            make_candidate("AAPL", "long_short", 1.5, 0.5, 3),
        ];
        let config = default_config(); // max_strategies_per_symbol = 1
        let result = select_candidates(candidates, &config);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].oos_psr, 2.0); // highest PSR kept
    }

    #[test]
    fn select_total_limit() {
        let candidates: Vec<StrategyCandidate> = (0..30)
            .map(|i| {
                make_candidate(
                    &format!("SYM{}", i),
                    "long_only",
                    1.0 + i as f64 * 0.01,
                    0.5,
                    3,
                )
            })
            .collect();
        let config = default_config(); // max_total = 20
        let result = select_candidates(candidates, &config);
        assert_eq!(result.len(), 20);
    }

    #[test]
    fn select_sort_order() {
        let candidates = vec![
            make_candidate("AAPL", "long_only", 1.0, 0.5, 3),
            make_candidate("GOOG", "long_only", 2.0, 0.5, 3),
            make_candidate("MSFT", "long_only", 1.5, 0.5, 3),
        ];
        let result = select_candidates(candidates, &default_config());
        assert_eq!(result[0].id.symbol, "GOOG");
        assert_eq!(result[1].id.symbol, "MSFT");
        assert_eq!(result[2].id.symbol, "AAPL");
    }

    #[test]
    fn select_empty_input() {
        let result = select_candidates(vec![], &default_config());
        assert!(result.is_empty());
    }

    #[test]
    fn select_all_filtered_out() {
        let candidates = vec![
            make_candidate("AAPL", "long_only", 0.1, 0.01, 0), // fails all
        ];
        let result = select_candidates(candidates, &default_config());
        assert!(result.is_empty());
    }

    #[test]
    fn strategy_id_display() {
        let id = StrategyId {
            exchange: "Polygon".to_string(),
            symbol: "AAPL".to_string(),
            mode: "long_only".to_string(),
            generation: 42,
        };
        assert_eq!(format!("{}", id), "Polygon:AAPL:long_only:gen42");
    }

    #[test]
    fn ensemble_config_defaults() {
        let config = EnsembleConfig::default();
        assert_eq!(config.min_oos_psr, 0.5);
        assert_eq!(config.min_wf_steps, 2);
        assert_eq!(config.min_utilization, 0.10);
        assert_eq!(config.max_strategies_per_symbol, 1);
        assert_eq!(config.max_total_strategies, 20);
        assert_eq!(config.correlation_lookback_bars, 500);
        assert_eq!(config.rebalance_interval_minutes, 30);
        assert!(!config.regime_aware);
        assert_eq!(config.regime_thresholds, [0.15, 0.30]);
        assert_eq!(config.regime_intervals, [240, 60, 15]);
    }

    #[test]
    fn detect_regime_low_vol() {
        // Very small returns → low annualized vol
        let returns: Vec<f64> = (0..30).map(|i| (i as f64 * 0.1).sin() * 0.0001).collect();
        let info = detect_regime(&returns, "1h", 20, [0.15, 0.30]);
        assert_eq!(info.regime, VolRegime::Low);
        assert!(info.annualized_vol < 0.15);
    }

    #[test]
    fn detect_regime_high_vol() {
        // Large alternating returns → high annualized vol
        let returns: Vec<f64> = (0..30)
            .map(|i| if i % 2 == 0 { 0.05 } else { -0.05 })
            .collect();
        let info = detect_regime(&returns, "1h", 20, [0.15, 0.30]);
        assert_eq!(info.regime, VolRegime::High);
        assert!(info.annualized_vol > 0.30);
    }

    #[test]
    fn detect_regime_insufficient_data() {
        let returns = vec![0.01, 0.02];
        let info = detect_regime(&returns, "1h", 20, [0.15, 0.30]);
        assert_eq!(info.regime, VolRegime::Normal);
        assert_eq!(info.annualized_vol, 0.0);
    }

    #[test]
    fn detect_regime_uses_window() {
        // First 20 bars: calm. Last 20 bars: volatile.
        let mut returns = vec![0.0001; 20];
        returns.extend(
            (0..20).map(|i| if i % 2 == 0 { 0.05 } else { -0.05 }),
        );
        let info = detect_regime(&returns, "1h", 20, [0.15, 0.30]);
        // Window=20 uses only the last 20 (volatile) bars
        assert_eq!(info.regime, VolRegime::High);
    }

    #[test]
    fn vol_regime_ordering() {
        assert!(VolRegime::Low < VolRegime::Normal);
        assert!(VolRegime::Normal < VolRegime::High);
        assert!(VolRegime::Low < VolRegime::High);
    }

    #[test]
    fn vol_regime_display() {
        assert_eq!(format!("{}", VolRegime::Low), "Low");
        assert_eq!(format!("{}", VolRegime::Normal), "Normal");
        assert_eq!(format!("{}", VolRegime::High), "High");
    }

    #[test]
    fn select_multiple_per_symbol_when_allowed() {
        let candidates = vec![
            make_candidate("AAPL", "long_only", 2.0, 0.5, 3),
            make_candidate("AAPL", "long_short", 1.5, 0.5, 3),
        ];
        let mut config = default_config();
        config.max_strategies_per_symbol = 2;
        let result = select_candidates(candidates, &config);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn config_yaml_deserialization() {
        let yaml = r#"
enabled: true
min_oos_psr: 0.8
min_wf_steps: 3
max_total_strategies: 10
"#;
        let config: EnsembleConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.min_oos_psr, 0.8);
        assert_eq!(config.min_wf_steps, 3);
        assert_eq!(config.max_total_strategies, 10);
        // Defaults filled in
        assert_eq!(config.min_utilization, 0.10);
        assert_eq!(config.max_strategies_per_symbol, 1);
    }
}
