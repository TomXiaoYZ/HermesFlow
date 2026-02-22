//! Dynamic weight adjustment for portfolio ensemble.
//!
//! Modifies HRP base weights using PSR reward, utilization decay, and
//! crowding penalty factors. All adjustments are multiplicative — they
//! tilt the HRP allocation without destroying its diversification structure.

use crate::backtest::ensemble::{DynamicWeightYamlConfig, StrategyCandidate};
use ndarray::Array2;

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

    let mut adjustments = Vec::with_capacity(n);
    let mut raw_weights = vec![0.0_f64; n];

    // Step 1: Compute per-strategy PSR and utilization factors
    let mut psr_factors = vec![1.0_f64; n];
    let mut util_factors = vec![1.0_f64; n];

    for i in 0..n {
        // PSR reward: boost proportional to OOS PSR (clamped to [0, max])
        let clamped_psr = candidates[i].oos_psr.max(0.0).min(config.psr_max_reward);
        psr_factors[i] = 1.0 + config.psr_reward_scale * clamped_psr;

        // Utilization decay: penalize low-utilization strategies
        util_factors[i] = candidates[i].utilization.max(config.utilization_floor);
    }

    // Step 2: Compute crowding penalties
    let crowding_pairs = detect_crowding(corr_matrix, config.crowding_corr_threshold);
    let mut crowding_penalties = vec![0.0_f64; n];

    for pair in &crowding_pairs {
        // Penalize the strategy with lower OOS PSR in the pair
        let (weaker, _stronger) = if candidates[pair.idx_i].oos_psr < candidates[pair.idx_j].oos_psr
        {
            (pair.idx_i, pair.idx_j)
        } else {
            (pair.idx_j, pair.idx_i)
        };

        // Accumulate penalty (capped at max)
        crowding_penalties[weaker] = (crowding_penalties[weaker] + config.crowding_penalty_rate)
            .min(config.crowding_max_penalty);
    }

    // Step 3: Apply all factors multiplicatively
    for i in 0..n {
        raw_weights[i] =
            hrp_weights[i] * psr_factors[i] * util_factors[i] * (1.0 - crowding_penalties[i]);
    }

    // Step 4: Renormalize to sum = 1.0
    let total: f64 = raw_weights.iter().sum();
    if total > 1e-12 {
        for w in &mut raw_weights {
            *w /= total;
        }
    }

    // Build adjustment records
    for i in 0..n {
        adjustments.push(WeightAdjustment {
            strategy_idx: i,
            hrp_weight: hrp_weights[i],
            psr_factor: psr_factors[i],
            utilization_factor: util_factors[i],
            crowding_penalty: crowding_penalties[i],
            final_weight: raw_weights[i],
        });
    }

    adjustments
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
}
