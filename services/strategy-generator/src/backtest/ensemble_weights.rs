//! Dynamic weight adjustment for portfolio ensemble.
//!
//! Modifies HRP base weights using PSR reward, utilization decay, and
//! crowding penalty factors. All adjustments are multiplicative — they
//! tilt the HRP allocation without destroying its diversification structure.

use crate::backtest::ensemble::{DynamicWeightYamlConfig, StrategyCandidate};
use ndarray::Array2;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Per-strategy weight adjustment breakdown.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WeightAdjustment {
    pub strategy_idx: usize,
    pub hrp_weight: f64,
    pub psr_factor: f64,
    pub utilization_factor: f64,
    pub crowding_penalty: f64,
    pub final_weight: f64,
}

/// Runtime config for dynamic weight adjustment (converted from YAML config).
#[derive(Debug, Clone)]
pub struct DynamicWeightConfig {
    pub psr_reward_scale: f64,
    pub psr_max_reward: f64,
    pub utilization_floor: f64,
    pub crowding_corr_threshold: f64,
    pub crowding_penalty_rate: f64,
    pub crowding_max_penalty: f64,
}

impl DynamicWeightConfig {
    pub fn from_yaml(yaml: &DynamicWeightYamlConfig) -> Self {
        Self {
            psr_reward_scale: yaml.psr_reward_scale,
            psr_max_reward: yaml.psr_max_reward,
            utilization_floor: yaml.utilization_floor,
            crowding_corr_threshold: yaml.crowding_corr_threshold,
            crowding_penalty_rate: yaml.crowding_penalty_rate,
            crowding_max_penalty: yaml.crowding_max_penalty,
        }
    }
}

impl Default for DynamicWeightConfig {
    fn default() -> Self {
        Self {
            psr_reward_scale: 0.2,
            psr_max_reward: 3.0,
            utilization_floor: 0.3,
            crowding_corr_threshold: 0.7,
            crowding_penalty_rate: 0.3,
            crowding_max_penalty: 0.8,
        }
    }
}

/// A detected crowding pair: two strategies with high correlation.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CrowdingPair {
    pub idx_i: usize,
    pub idx_j: usize,
    pub correlation: f64,
}

// ── Crowding Detection ─────────────────────────────────────────────────

/// Detect pairs of strategies whose correlation exceeds the threshold.
///
/// Returns all (i, j) pairs where corr[i,j] > threshold, with i < j.
pub fn detect_crowding(corr_matrix: &Array2<f64>, threshold: f64) -> Vec<CrowdingPair> {
    let n = corr_matrix.nrows();
    let mut pairs = Vec::new();

    for i in 0..n {
        for j in (i + 1)..n {
            if corr_matrix[[i, j]] > threshold {
                pairs.push(CrowdingPair {
                    idx_i: i,
                    idx_j: j,
                    correlation: corr_matrix[[i, j]],
                });
            }
        }
    }

    pairs
}

// ── Weight Adjustment ──────────────────────────────────────────────────

/// Apply PSR reward, utilization decay, and crowding penalty to HRP weights.
///
/// Each factor is a multiplier on the base HRP weight:
/// - **PSR reward**: `1 + scale * clamp(oos_psr, 0, max)` — higher PSR → more weight
/// - **Utilization decay**: `max(floor, utilization)` — low util → reduced weight
/// - **Crowding penalty**: `1 - penalty` — penalize the weaker strategy in correlated pairs
///
/// Final weights are renormalized to sum to 1.0.
pub fn adjust_weights(
    hrp_weights: &[f64],
    candidates: &[StrategyCandidate],
    corr_matrix: &Array2<f64>,
    config: &DynamicWeightConfig,
) -> Vec<WeightAdjustment> {
    let n = hrp_weights.len();
    assert_eq!(n, candidates.len());

    // P8-4B: Convert to Decimal for financial-grade precision
    let d_psr_scale = Decimal::from_f64_retain(config.psr_reward_scale).unwrap_or(dec!(0.2));
    let d_psr_max = Decimal::from_f64_retain(config.psr_max_reward).unwrap_or(dec!(3.0));
    let d_util_floor = Decimal::from_f64_retain(config.utilization_floor).unwrap_or(dec!(0.3));
    let d_penalty_rate =
        Decimal::from_f64_retain(config.crowding_penalty_rate).unwrap_or(dec!(0.3));
    let d_penalty_max =
        Decimal::from_f64_retain(config.crowding_max_penalty).unwrap_or(dec!(0.8));

    let mut adjustments = Vec::with_capacity(n);
    let mut d_weights = vec![Decimal::ZERO; n];

    // Step 1: Compute per-strategy PSR and utilization factors (Decimal)
    let mut d_psr_factors = vec![Decimal::ONE; n];
    let mut d_util_factors = vec![Decimal::ONE; n];

    for i in 0..n {
        let d_oos_psr =
            Decimal::from_f64_retain(candidates[i].oos_psr).unwrap_or(Decimal::ZERO);
        let clamped = d_oos_psr.max(Decimal::ZERO).min(d_psr_max);
        d_psr_factors[i] = Decimal::ONE + d_psr_scale * clamped;

        let d_util =
            Decimal::from_f64_retain(candidates[i].utilization).unwrap_or(Decimal::ZERO);
        d_util_factors[i] = d_util.max(d_util_floor);
    }

    // Step 2: Compute crowding penalties (Decimal)
    let crowding_pairs = detect_crowding(corr_matrix, config.crowding_corr_threshold);
    let mut d_crowding = vec![Decimal::ZERO; n];

    for pair in &crowding_pairs {
        let (weaker, _stronger) = if candidates[pair.idx_i].oos_psr < candidates[pair.idx_j].oos_psr
        {
            (pair.idx_i, pair.idx_j)
        } else {
            (pair.idx_j, pair.idx_i)
        };
        d_crowding[weaker] = (d_crowding[weaker] + d_penalty_rate).min(d_penalty_max);
    }

    // Step 3: Apply all factors multiplicatively
    for i in 0..n {
        let d_hrp = Decimal::from_f64_retain(hrp_weights[i]).unwrap_or(Decimal::ZERO);
        d_weights[i] = d_hrp * d_psr_factors[i] * d_util_factors[i] * (Decimal::ONE - d_crowding[i]);
    }

    // Step 4: Renormalize to sum = 1.0
    let total: Decimal = d_weights.iter().copied().sum();
    if total > dec!(0.000000000001) {
        for w in &mut d_weights {
            *w /= total;
        }
    }

    // Build adjustment records (convert back to f64 for ndarray compatibility)
    for i in 0..n {
        adjustments.push(WeightAdjustment {
            strategy_idx: i,
            hrp_weight: hrp_weights[i],
            psr_factor: d_psr_factors[i].to_f64().unwrap_or(1.0),
            utilization_factor: d_util_factors[i].to_f64().unwrap_or(1.0),
            crowding_penalty: d_crowding[i].to_f64().unwrap_or(0.0),
            final_weight: d_weights[i].to_f64().unwrap_or(0.0),
        });
    }

    adjustments
}

// ── Turnover Cost (P6a-F2) ────────────────────────────────────────────

/// Compute portfolio turnover between two weight vectors.
///
/// Turnover = 0.5 * sum(|new_i - old_i|). Range [0.0, 1.0].
/// 0.0 means no change; 1.0 means complete rebalance.
/// Old and new vectors may have different lengths (strategies added/removed).
/// Uses strategy symbol+mode as key for matching.
pub fn compute_turnover(old_weights: &[(String, f64)], new_weights: &[(String, f64)]) -> f64 {
    // P8-4B: Decimal precision for turnover affecting real money
    let old_map: std::collections::HashMap<&str, Decimal> = old_weights
        .iter()
        .map(|(k, v)| (k.as_str(), Decimal::from_f64_retain(*v).unwrap_or(Decimal::ZERO)))
        .collect();
    let new_map: std::collections::HashMap<&str, Decimal> = new_weights
        .iter()
        .map(|(k, v)| (k.as_str(), Decimal::from_f64_retain(*v).unwrap_or(Decimal::ZERO)))
        .collect();

    let mut all_keys: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for (k, _) in old_weights {
        all_keys.insert(k.as_str());
    }
    for (k, _) in new_weights {
        all_keys.insert(k.as_str());
    }

    let total_change: Decimal = all_keys
        .iter()
        .map(|k| {
            let old_w = old_map.get(k).copied().unwrap_or(Decimal::ZERO);
            let new_w = new_map.get(k).copied().unwrap_or(Decimal::ZERO);
            (new_w - old_w).abs()
        })
        .sum();

    let half = dec!(0.5);
    (half * total_change).to_f64().unwrap_or(0.0)
}

/// Compute turnover cost: turnover * cost_rate.
/// Cost rate is per-exchange (e.g., IBKR=0.0001, Binance=0.001).
pub fn turnover_cost(turnover: f64, cost_rate: f64) -> f64 {
    // P8-4B: Decimal precision for cost calculation
    let d_turnover = Decimal::from_f64_retain(turnover).unwrap_or(Decimal::ZERO);
    let d_rate = Decimal::from_f64_retain(cost_rate).unwrap_or(Decimal::ZERO);
    (d_turnover * d_rate).to_f64().unwrap_or(0.0)
}

// ── Deadzone + L1 Regularization (P6b-C1) ────────────────────────────

/// Apply deadzone and L1 regularization to suppress unnecessary turnover.
///
/// 1. **Deadzone**: If `|new_w - old_w| < threshold` for a strategy, revert to old weight.
/// 2. **L1 regularization**: Shrink weight deltas toward zero:
///    `adjusted = old + (new - old) * (1 - effective_lambda)`.
/// 3. **Renormalize** to sum = 1.0.
///
/// `regime_multiplier` adapts lambda: >1.0 in calm markets (more suppression),
/// <1.0 in volatile markets (faster adaptation).
pub fn apply_deadzone_l1(
    old_weights: &[(String, f64)],
    new_weights: &mut [(String, f64)],
    threshold: f64,
    l1_lambda: f64,
    regime_multiplier: f64,
) {
    // P8-4B: Decimal precision for weight adjustment affecting real money
    let old_map: std::collections::HashMap<&str, Decimal> = old_weights
        .iter()
        .map(|(k, v)| (k.as_str(), Decimal::from_f64_retain(*v).unwrap_or(Decimal::ZERO)))
        .collect();

    let d_threshold = Decimal::from_f64_retain(threshold).unwrap_or(Decimal::ZERO);
    let d_lambda = Decimal::from_f64_retain(l1_lambda).unwrap_or(Decimal::ZERO);
    let d_multiplier = Decimal::from_f64_retain(regime_multiplier).unwrap_or(Decimal::ONE);
    let effective_lambda = (d_lambda * d_multiplier).min(dec!(0.5));

    for (key, weight) in new_weights.iter_mut() {
        if let Some(&old_w) = old_map.get(key.as_str()) {
            let d_new = Decimal::from_f64_retain(*weight).unwrap_or(Decimal::ZERO);
            let delta = d_new - old_w;

            if delta.abs() < d_threshold {
                *weight = old_w.to_f64().unwrap_or(*weight);
                continue;
            }

            let adjusted = old_w + delta * (Decimal::ONE - effective_lambda);
            *weight = adjusted.to_f64().unwrap_or(*weight);
        }
    }

    // Renormalize
    let total: f64 = new_weights.iter().map(|(_, w)| *w).sum();
    if total > 1e-12 {
        for (_, w) in new_weights.iter_mut() {
            *w /= total;
        }
    }
}

// ── Hysteresis Dead-Zone (P6-2A) ─────────────────────────────────────
// Not yet integrated into ensemble routing pipeline.

/// P6-2A: Per-asset hysteresis dead-zone parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AssetDeadzone {
    /// Strategy key (symbol:mode)
    pub key: String,
    /// Annualized volatility of the strategy's return series
    pub volatility: f64,
    /// Transaction fee rate (one-way, as fraction)
    pub fee_rate: f64,
    /// Bid-ask spread as fraction of price (e.g., 0.0002 = 2 bps)
    pub spread: f64,
}

/// P6-2A: Compute per-asset no-trade threshold.
///
/// The threshold is proportional to the expected cost of a round-trip:
/// `δ_i = base_threshold + fee_multiplier * (2 * fee_rate + spread) * sqrt(vol_i)`
///
/// Higher volatility + higher costs → wider dead-zone (fewer trades).
#[allow(dead_code)]
pub fn compute_asset_threshold(
    asset: &AssetDeadzone,
    base_threshold: f64,
    fee_multiplier: f64,
) -> f64 {
    let cost = 2.0 * asset.fee_rate + asset.spread;
    base_threshold + fee_multiplier * cost * asset.volatility.sqrt().max(0.01)
}

/// P6-2A: Apply per-asset hysteresis dead-zone with partial rebalancing.
///
/// For each asset:
/// 1. Compute per-asset no-trade threshold δ_i
/// 2. If |w_target - w_current| <= δ_i → keep current weight
/// 3. If |w_target - w_current| > δ_i → trade to dead-zone BOUNDARY, not exact target
///    New weight = w_current + (delta - δ_i * sign(delta))
///
/// This avoids micro-rebalancing that destroys alpha through transaction costs.
///
/// Returns (adjusted_weights, deadzone_metadata) where metadata includes
/// per-asset threshold and whether a trade was triggered.
#[allow(dead_code)] // P7-3D validated; wired into ensemble rebalance in P8
pub fn apply_hysteresis_deadzone(
    old_weights: &[(String, f64)],
    new_weights: &mut [(String, f64)],
    asset_params: &[AssetDeadzone],
    base_threshold: f64,
    fee_multiplier: f64,
) -> Vec<DeadzoneMetadata> {
    // P8-4B: Decimal precision for weight adjustment affecting real money
    let old_map: std::collections::HashMap<&str, Decimal> = old_weights
        .iter()
        .map(|(k, v)| (k.as_str(), Decimal::from_f64_retain(*v).unwrap_or(Decimal::ZERO)))
        .collect();

    let param_map: std::collections::HashMap<&str, &AssetDeadzone> =
        asset_params.iter().map(|a| (a.key.as_str(), a)).collect();

    let mut metadata = Vec::with_capacity(new_weights.len());

    for (key, weight) in new_weights.iter_mut() {
        let d_old = old_map.get(key.as_str()).copied().unwrap_or(Decimal::ZERO);
        let d_new = Decimal::from_f64_retain(*weight).unwrap_or(Decimal::ZERO);
        let d_delta = d_new - d_old;

        // Compute per-asset threshold (stays f64 — uses sqrt which Decimal lacks)
        let threshold = if let Some(params) = param_map.get(key.as_str()) {
            compute_asset_threshold(params, base_threshold, fee_multiplier)
        } else {
            base_threshold
        };
        let d_threshold = Decimal::from_f64_retain(threshold).unwrap_or(Decimal::ZERO);

        let triggered = d_delta.abs() > d_threshold;
        let delta_f64 = d_delta.to_f64().unwrap_or(0.0);

        if !triggered {
            tracing::debug!(
                asset = key.as_str(), %threshold, delta = %delta_f64,
                "dead-zone: suppressed (|delta| <= threshold)"
            );
            *weight = d_old.to_f64().unwrap_or(*weight);
            metadata.push(DeadzoneMetadata {
                key: key.clone(),
                threshold,
                delta_before: delta_f64,
                delta_after: 0.0,
                triggered: false,
            });
        } else {
            let d_sign = if d_delta > Decimal::ZERO {
                Decimal::ONE
            } else {
                -Decimal::ONE
            };
            let boundary_delta = d_delta - d_threshold * d_sign;
            let boundary_f64 = boundary_delta.to_f64().unwrap_or(0.0);
            tracing::debug!(
                asset = key.as_str(), %threshold, delta = %delta_f64,
                boundary_delta = %boundary_f64,
                "dead-zone: triggered, partial rebalance to boundary"
            );
            *weight = (d_old + boundary_delta).to_f64().unwrap_or(*weight);
            metadata.push(DeadzoneMetadata {
                key: key.clone(),
                threshold,
                delta_before: delta_f64,
                delta_after: boundary_f64,
                triggered: true,
            });
        }
    }

    // Renormalize
    let total: f64 = new_weights.iter().map(|(_, w)| *w).sum();
    if total > 1e-12 {
        for (_, w) in new_weights.iter_mut() {
            *w /= total;
        }
    }

    metadata
}

/// P6-2A: Per-asset dead-zone metadata for monitoring and Redis publication.
#[allow(dead_code)] // P7-3D validated; used in tests
#[derive(Debug, Clone)]
pub struct DeadzoneMetadata {
    pub key: String,
    pub threshold: f64,
    pub delta_before: f64,
    pub delta_after: f64,
    pub triggered: bool,
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtest::ensemble::{StrategyCandidate, StrategyId};

    fn approx_eq(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() < tol
    }

    fn make_candidate(symbol: &str, oos_psr: f64, utilization: f64) -> StrategyCandidate {
        StrategyCandidate {
            id: StrategyId {
                exchange: "Polygon".to_string(),
                symbol: symbol.to_string(),
                mode: "long_only".to_string(),
                generation: 100,
            },
            genome: vec![1, 2, 3],
            oos_psr,
            is_fitness: 1.0,
            utilization,
            walk_forward_steps: 3,
        }
    }

    fn neutral_corr(n: usize) -> Array2<f64> {
        Array2::<f64>::eye(n) // no correlation between strategies
    }

    #[test]
    fn no_modification_with_neutral_inputs() {
        let hrp = vec![0.5, 0.5];
        let candidates = vec![
            make_candidate("AAPL", 0.0, 1.0), // PSR=0 → factor=1.0, util=1.0
            make_candidate("GOOG", 0.0, 1.0),
        ];
        let corr = neutral_corr(2);
        let config = DynamicWeightConfig::default();
        let adj = adjust_weights(&hrp, &candidates, &corr, &config);
        assert!(approx_eq(adj[0].final_weight, 0.5, 0.01));
        assert!(approx_eq(adj[1].final_weight, 0.5, 0.01));
    }

    #[test]
    fn psr_reward_increases_weight() {
        let hrp = vec![0.5, 0.5];
        let candidates = vec![
            make_candidate("AAPL", 2.0, 1.0), // high PSR
            make_candidate("GOOG", 0.0, 1.0), // zero PSR
        ];
        let corr = neutral_corr(2);
        let config = DynamicWeightConfig::default();
        let adj = adjust_weights(&hrp, &candidates, &corr, &config);
        assert!(adj[0].final_weight > adj[1].final_weight);
    }

    #[test]
    fn utilization_decay_reduces_weight() {
        let hrp = vec![0.5, 0.5];
        let candidates = vec![
            make_candidate("AAPL", 1.0, 0.05), // very low util → clamped to floor
            make_candidate("GOOG", 1.0, 1.0),  // full util
        ];
        let corr = neutral_corr(2);
        let config = DynamicWeightConfig::default();
        let adj = adjust_weights(&hrp, &candidates, &corr, &config);
        assert!(adj[0].final_weight < adj[1].final_weight);
    }

    #[test]
    fn crowding_penalty_on_weaker_strategy() {
        let hrp = vec![0.5, 0.5];
        let candidates = vec![
            make_candidate("AAPL", 1.0, 0.5), // weaker PSR
            make_candidate("GOOG", 2.0, 0.5), // stronger PSR
        ];
        // High correlation → crowding
        let mut corr = Array2::<f64>::eye(2);
        corr[[0, 1]] = 0.9;
        corr[[1, 0]] = 0.9;
        let config = DynamicWeightConfig::default();
        let adj = adjust_weights(&hrp, &candidates, &corr, &config);
        // Weaker (AAPL) should be penalized
        assert!(adj[0].crowding_penalty > 0.0);
        assert!(approx_eq(adj[1].crowding_penalty, 0.0, 1e-10));
        assert!(adj[0].final_weight < adj[1].final_weight);
    }

    #[test]
    fn weights_always_sum_to_one() {
        let hrp = vec![0.4, 0.35, 0.25];
        let candidates = vec![
            make_candidate("AAPL", 1.5, 0.6),
            make_candidate("GOOG", 2.0, 0.8),
            make_candidate("MSFT", 0.5, 0.3),
        ];
        let corr = neutral_corr(3);
        let config = DynamicWeightConfig::default();
        let adj = adjust_weights(&hrp, &candidates, &corr, &config);
        let sum: f64 = adj.iter().map(|a| a.final_weight).sum();
        assert!(approx_eq(sum, 1.0, 1e-10));
    }

    #[test]
    fn no_crowding_below_threshold() {
        let mut corr = Array2::<f64>::eye(2);
        corr[[0, 1]] = 0.5; // below default 0.7 threshold
        corr[[1, 0]] = 0.5;
        let pairs = detect_crowding(&corr, 0.7);
        assert!(pairs.is_empty());
    }

    #[test]
    fn crowding_detected_above_threshold() {
        let mut corr = Array2::<f64>::eye(2);
        corr[[0, 1]] = 0.85;
        corr[[1, 0]] = 0.85;
        let pairs = detect_crowding(&corr, 0.7);
        assert_eq!(pairs.len(), 1);
        assert!(approx_eq(pairs[0].correlation, 0.85, 1e-10));
    }

    #[test]
    fn max_penalty_cap() {
        let hrp = vec![1.0 / 3.0; 3];
        let candidates = vec![
            make_candidate("AAPL", 0.5, 0.5), // weakest PSR
            make_candidate("GOOG", 1.0, 0.5),
            make_candidate("MSFT", 2.0, 0.5),
        ];
        // AAPL is correlated with both GOOG and MSFT → double penalty
        let mut corr = Array2::<f64>::eye(3);
        corr[[0, 1]] = 0.9;
        corr[[1, 0]] = 0.9;
        corr[[0, 2]] = 0.9;
        corr[[2, 0]] = 0.9;
        let config = DynamicWeightConfig {
            crowding_penalty_rate: 0.5,
            crowding_max_penalty: 0.8,
            ..Default::default()
        };
        let adj = adjust_weights(&hrp, &candidates, &corr, &config);
        // Penalty should be capped at 0.8, not 1.0
        assert!(approx_eq(adj[0].crowding_penalty, 0.8, 1e-10));
        assert!(adj[0].final_weight > 0.0); // not completely zeroed
    }

    // ── Turnover cost tests (P6a-F2) ──────────────────────────────

    #[test]
    fn turnover_identical_weights_is_zero() {
        let old = vec![("SPY_lo".to_string(), 0.5), ("GLD_lo".to_string(), 0.5)];
        let new = vec![("SPY_lo".to_string(), 0.5), ("GLD_lo".to_string(), 0.5)];
        assert!(approx_eq(compute_turnover(&old, &new), 0.0, 1e-10));
    }

    #[test]
    fn turnover_complete_rebalance() {
        let old = vec![("SPY_lo".to_string(), 1.0), ("GLD_lo".to_string(), 0.0)];
        let new = vec![("SPY_lo".to_string(), 0.0), ("GLD_lo".to_string(), 1.0)];
        assert!(approx_eq(compute_turnover(&old, &new), 1.0, 1e-10));
    }

    #[test]
    fn turnover_new_strategy_added() {
        let old = vec![("SPY_lo".to_string(), 1.0)];
        let new = vec![("SPY_lo".to_string(), 0.6), ("GLD_lo".to_string(), 0.4)];
        // |0.6-1.0| + |0.4-0.0| = 0.4 + 0.4 = 0.8, turnover = 0.4
        assert!(approx_eq(compute_turnover(&old, &new), 0.4, 1e-10));
    }

    #[test]
    fn turnover_strategy_removed() {
        let old = vec![("SPY_lo".to_string(), 0.6), ("GLD_lo".to_string(), 0.4)];
        let new = vec![("SPY_lo".to_string(), 1.0)];
        assert!(approx_eq(compute_turnover(&old, &new), 0.4, 1e-10));
    }

    #[test]
    fn turnover_cost_calculation() {
        assert!(approx_eq(turnover_cost(0.3, 0.001), 0.0003, 1e-10));
        assert!(approx_eq(turnover_cost(0.0, 0.001), 0.0, 1e-10));
    }

    // ── Deadzone + L1 tests (P6b-C1) ──────────────────────────────

    #[test]
    fn deadzone_suppresses_small_changes() {
        let old = vec![("SPY_lo".to_string(), 0.5), ("GLD_lo".to_string(), 0.5)];
        let mut new = vec![("SPY_lo".to_string(), 0.51), ("GLD_lo".to_string(), 0.49)];
        // threshold=0.02, delta=0.01 < 0.02 → reverted to old
        apply_deadzone_l1(&old, &mut new, 0.02, 0.0, 1.0);
        assert!(approx_eq(new[0].1, 0.5, 1e-10));
        assert!(approx_eq(new[1].1, 0.5, 1e-10));
    }

    #[test]
    fn deadzone_allows_large_changes() {
        let old = vec![("SPY_lo".to_string(), 0.5), ("GLD_lo".to_string(), 0.5)];
        let mut new = vec![("SPY_lo".to_string(), 0.7), ("GLD_lo".to_string(), 0.3)];
        // threshold=0.02, delta=0.2 > 0.02 → not reverted
        apply_deadzone_l1(&old, &mut new, 0.02, 0.0, 1.0);
        // With lambda=0, no L1 shrinkage, just renormalized
        let sum: f64 = new.iter().map(|(_, w)| *w).sum();
        assert!(approx_eq(sum, 1.0, 1e-10));
        assert!(new[0].1 > new[1].1); // SPY still has more weight
    }

    #[test]
    fn l1_shrinks_deltas() {
        let old = vec![("SPY_lo".to_string(), 0.5), ("GLD_lo".to_string(), 0.5)];
        let mut new = vec![("SPY_lo".to_string(), 0.7), ("GLD_lo".to_string(), 0.3)];
        // lambda=0.5 → delta shrunk by 50%: 0.5 + 0.2*0.5 = 0.6, 0.5 + (-0.2)*0.5 = 0.4
        apply_deadzone_l1(&old, &mut new, 0.0, 0.5, 1.0);
        // After renormalization: 0.6/(0.6+0.4)=0.6, 0.4/(0.6+0.4)=0.4
        assert!(approx_eq(new[0].1, 0.6, 1e-10));
        assert!(approx_eq(new[1].1, 0.4, 1e-10));
    }

    #[test]
    fn regime_multiplier_scales_lambda() {
        let old = vec![("SPY_lo".to_string(), 0.5), ("GLD_lo".to_string(), 0.5)];
        let mut new_calm = vec![("SPY_lo".to_string(), 0.7), ("GLD_lo".to_string(), 0.3)];
        let mut new_volatile = vec![("SPY_lo".to_string(), 0.7), ("GLD_lo".to_string(), 0.3)];

        // Calm: multiplier=2.0, effective_lambda=0.1*2.0=0.2
        apply_deadzone_l1(&old, &mut new_calm, 0.0, 0.1, 2.0);
        // Volatile: multiplier=0.5, effective_lambda=0.1*0.5=0.05
        apply_deadzone_l1(&old, &mut new_volatile, 0.0, 0.1, 0.5);

        // Calm should have weights closer to old (more suppression)
        let calm_delta = (new_calm[0].1 - 0.5).abs();
        let vol_delta = (new_volatile[0].1 - 0.5).abs();
        assert!(calm_delta < vol_delta);
    }

    #[test]
    fn deadzone_new_strategy_passes_through() {
        let old = vec![("SPY_lo".to_string(), 1.0)];
        let mut new = vec![("SPY_lo".to_string(), 0.6), ("GLD_lo".to_string(), 0.4)];
        // GLD is new (not in old), should keep its weight
        apply_deadzone_l1(&old, &mut new, 0.02, 0.1, 1.0);
        let sum: f64 = new.iter().map(|(_, w)| *w).sum();
        assert!(approx_eq(sum, 1.0, 1e-10));
        assert!(new[1].1 > 0.0); // GLD has weight
    }

    #[test]
    fn lambda_capped_at_half() {
        let old = vec![("SPY_lo".to_string(), 0.5), ("GLD_lo".to_string(), 0.5)];
        let mut new = vec![("SPY_lo".to_string(), 0.7), ("GLD_lo".to_string(), 0.3)];
        // lambda=0.4, multiplier=2.0 → 0.8, but capped at 0.5
        apply_deadzone_l1(&old, &mut new, 0.0, 0.4, 2.0);
        // delta=0.2 * (1-0.5) = 0.1 → new=0.6, renormalized to 0.6
        assert!(approx_eq(new[0].1, 0.6, 1e-10));
    }

    #[test]
    fn psr_reward_capped_at_max() {
        let hrp = vec![0.5, 0.5];
        let candidates = vec![
            make_candidate("AAPL", 10.0, 1.0), // very high PSR
            make_candidate("GOOG", 10.0, 1.0),
        ];
        let corr = neutral_corr(2);
        let config = DynamicWeightConfig::default(); // max_reward = 3.0
        let adj = adjust_weights(&hrp, &candidates, &corr, &config);
        // Both capped identically → still 50/50
        assert!(approx_eq(adj[0].psr_factor, adj[1].psr_factor, 1e-10));
        // Factor should be 1 + 0.2 * 3.0 = 1.6
        assert!(approx_eq(adj[0].psr_factor, 1.6, 1e-10));
    }

    // ── Hysteresis dead-zone tests (P6-2A) ──────────────────────────

    fn make_asset(key: &str, vol: f64) -> AssetDeadzone {
        AssetDeadzone {
            key: key.to_string(),
            volatility: vol,
            fee_rate: 0.0001,  // 1 bps
            spread: 0.0002,    // 2 bps
        }
    }

    #[test]
    fn asset_threshold_proportional_to_vol() {
        let low_vol = make_asset("SPY", 0.10);
        let high_vol = make_asset("TSLA", 0.50);
        let t_low = compute_asset_threshold(&low_vol, 0.005, 2.0);
        let t_high = compute_asset_threshold(&high_vol, 0.005, 2.0);
        assert!(t_high > t_low, "Higher vol should have wider dead-zone");
    }

    #[test]
    fn hysteresis_suppresses_small_delta() {
        let old = vec![("SPY_lo".to_string(), 0.5), ("GLD_lo".to_string(), 0.5)];
        let mut new = vec![("SPY_lo".to_string(), 0.51), ("GLD_lo".to_string(), 0.49)];
        let assets = vec![make_asset("SPY_lo", 0.15), make_asset("GLD_lo", 0.12)];

        let meta = apply_hysteresis_deadzone(&old, &mut new, &assets, 0.02, 2.0);

        // Delta = 0.01 < threshold (~0.02+), should NOT trigger
        assert!(!meta[0].triggered);
        assert!(!meta[1].triggered);
    }

    #[test]
    fn hysteresis_trades_to_boundary() {
        let old = vec![("SPY_lo".to_string(), 0.5), ("GLD_lo".to_string(), 0.5)];
        let mut new = vec![("SPY_lo".to_string(), 0.7), ("GLD_lo".to_string(), 0.3)];
        let assets = vec![make_asset("SPY_lo", 0.15), make_asset("GLD_lo", 0.15)];

        let meta = apply_hysteresis_deadzone(&old, &mut new, &assets, 0.005, 2.0);

        // Delta = 0.2 > threshold, should trigger
        assert!(meta[0].triggered);
        // The delta_after should be less than delta_before (partial rebalance)
        assert!(meta[0].delta_after.abs() < meta[0].delta_before.abs());
    }

    #[test]
    fn hysteresis_weights_renormalize() {
        let old = vec![("SPY_lo".to_string(), 0.5), ("GLD_lo".to_string(), 0.5)];
        let mut new = vec![("SPY_lo".to_string(), 0.7), ("GLD_lo".to_string(), 0.3)];
        let assets = vec![make_asset("SPY_lo", 0.15), make_asset("GLD_lo", 0.15)];

        apply_hysteresis_deadzone(&old, &mut new, &assets, 0.005, 2.0);

        let sum: f64 = new.iter().map(|(_, w)| *w).sum();
        assert!(approx_eq(sum, 1.0, 1e-10));
    }
}
