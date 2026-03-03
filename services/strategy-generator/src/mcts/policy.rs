//! Policy network for MCTS action selection.
//!
//! Provides prior probability distributions over legal actions.
//! - `UniformPolicy`: equal probability for all legal actions (baseline).
//! - `HeuristicPolicy`: feature-biased prior (simple heuristic).
//! - `LlmCachedPolicy`: LLM-generated priors with HashMap cache + uniform fallback.

use std::collections::HashMap;

#[cfg(test)]
use super::state::ActionSpace;
use crate::backtest::factor_importance::FactorImportance;
use crate::LlmMctsPriorConfig;

/// Policy trait: given a partial RPN state, return prior probabilities for each legal action.
pub trait Policy: Send + Sync {
    /// Compute prior probabilities for each legal action.
    /// Returns Vec of (action, prior) pairs. Priors sum to 1.0.
    fn prior(&self, legal_actions: &[u32], stack_depth: u32, current_tokens: &[u32]) -> Vec<f64>;
}

/// Uniform prior: equal probability for all legal actions.
pub struct UniformPolicy;

impl Policy for UniformPolicy {
    fn prior(&self, legal_actions: &[u32], _stack_depth: u32, _current_tokens: &[u32]) -> Vec<f64> {
        let n = legal_actions.len();
        if n == 0 {
            return Vec::new();
        }
        let p = 1.0 / n as f64;
        vec![p; n]
    }
}

/// Feature-biased policy: slightly favors features over operators in early positions,
/// and binary operators when stack is deep. A simple heuristic before LLM prior.
/// Currently used in tests; planned as LlmCachedPolicy fallback in future phases.
#[cfg(test)]
pub struct HeuristicPolicy {
    feat_offset: usize,
}

#[cfg(test)]
impl HeuristicPolicy {
    pub fn new(action_space: &ActionSpace) -> Self {
        Self {
            feat_offset: action_space.feat_offset,
        }
    }
}

#[cfg(test)]
impl Policy for HeuristicPolicy {
    fn prior(&self, legal_actions: &[u32], stack_depth: u32, current_tokens: &[u32]) -> Vec<f64> {
        let n = legal_actions.len();
        if n == 0 {
            return Vec::new();
        }

        let mut weights: Vec<f64> = Vec::with_capacity(n);
        let depth = current_tokens.len();

        for &action in legal_actions {
            let w = if (action as usize) < self.feat_offset {
                // Feature: prefer in early positions
                if depth < 3 {
                    2.0
                } else {
                    1.0
                }
            } else {
                // Operator: prefer when stack is deep (need to collapse)
                if stack_depth >= 3 {
                    1.5
                } else {
                    1.0
                }
            };
            weights.push(w);
        }

        // Normalize to probabilities
        let sum: f64 = weights.iter().sum();
        if sum > 0.0 {
            weights.iter_mut().for_each(|w| *w /= sum);
        }

        weights
    }
}

/// P8-0C: Canonicalize RPN tokens by sorting commutative operator operands.
///
/// For commutative binary ops (ADD, MUL, TS_CORR), ensures the left operand
/// subexpression is lexicographically <= right operand subexpression.
/// This makes mathematically equivalent expressions (e.g. `A B ADD` and `B A ADD`)
/// produce identical hash values, improving LlmCachedPolicy cache hit rate.
fn canonicalize_rpn(tokens: &[u32], feat_offset: u32) -> Vec<u32> {
    // Commutative operator offsets: ADD=0, MUL=2, TS_CORR=16
    let commutative_offsets: &[u32] = &[0, 2, 16];

    let mut result = tokens.to_vec();
    let len = result.len();
    if len < 3 {
        return result;
    }

    // Compute the arity and subexpression spans for each token.
    // For each position, compute how many tokens its subexpression spans.
    let mut subtree_size = vec![1usize; len];
    let mut stack_effect = vec![0i32; len]; // +1 for push, -1 for binary, 0 for unary

    for i in 0..len {
        let t = result[i];
        if t < feat_offset {
            // Feature: pushes 1
            stack_effect[i] = 1;
            subtree_size[i] = 1;
        } else {
            let op_idx = t - feat_offset;
            // Binary ops: ADD(0), SUB(1), MUL(2), DIV(3), TS_CORR(16)
            // Unary ops: all others
            let is_binary = matches!(op_idx, 0..=3 | 16);
            if is_binary {
                stack_effect[i] = -1; // consume 2, push 1 → net -1
                                      // Find the two operand subtrees by walking backward
                let mut depth = 0i32;
                let mut right_start = i;
                // Walk backward to find right operand start
                for j in (0..i).rev() {
                    depth += stack_effect[j];
                    if depth == 1 {
                        right_start = j;
                        break;
                    }
                }
                // Left operand is everything from some earlier point to right_start-1
                let right_slice = &result[right_start..i];
                let left_end = right_start;
                // Walk further back for left operand
                depth = 0;
                let mut left_start = 0;
                for j in (0..left_end).rev() {
                    depth += stack_effect[j];
                    if depth == 1 {
                        left_start = j;
                        break;
                    }
                }
                let left_slice = &result[left_start..left_end];

                subtree_size[i] = 1 + left_slice.len() + right_slice.len();

                // If commutative and left > right lexicographically, swap
                if commutative_offsets.contains(&op_idx) && left_slice > right_slice {
                    let mut swapped = Vec::with_capacity(subtree_size[i]);
                    swapped.extend_from_slice(right_slice);
                    swapped.extend_from_slice(left_slice);
                    swapped.push(result[i]); // the operator

                    // Replace in result
                    result.splice(left_start..=i, swapped);

                    // Recompute stack_effect for swapped region
                    for k in left_start..=i.min(result.len() - 1) {
                        let tk = result[k];
                        if tk < feat_offset {
                            stack_effect[k] = 1;
                        } else {
                            let ok = tk - feat_offset;
                            stack_effect[k] = if matches!(ok, 0..=3 | 16) { -1 } else { 0 };
                        }
                    }
                }
            } else {
                // Unary: consume 1, push 1 → net 0
                stack_effect[i] = 0;
                subtree_size[i] = 1 + subtree_size.get(i.wrapping_sub(1)).copied().unwrap_or(1);
            }
        }
    }

    result
}

/// LLM-cached policy: looks up pre-computed prior distributions from a cache.
///
/// Before running MCTS, call `insert()` to populate the cache with LLM-generated
/// probability distributions for known partial RPN states. During MCTS rollouts,
/// cache hits return the LLM prior; misses fall back to uniform distribution.
///
/// Cache key: hash of `(current_tokens, stack_depth)` tuple.
/// This avoids calling the LLM synchronously during tight MCTS loops.
pub struct LlmCachedPolicy {
    /// Cached priors: key = state hash, value = Vec of per-action weights (unnormalized).
    /// The weights cover the full vocabulary (indices 0..vocab_size).
    cache: HashMap<u64, Vec<f64>>,
    /// Feature offset for canonical hash computation.
    feat_offset: u32,
}

impl LlmCachedPolicy {
    /// Create an empty LLM policy cache.
    pub fn new(_vocab_size: usize, feat_offset: usize) -> Self {
        Self {
            cache: HashMap::new(),
            feat_offset: feat_offset as u32,
        }
    }

    /// Insert a prior distribution for a given partial state.
    /// `weights` should have length `vocab_size`, with unnormalized weights per token.
    pub fn insert(&mut self, tokens: &[u32], stack_depth: u32, weights: Vec<f64>) {
        let key = Self::hash_state(tokens, stack_depth, self.feat_offset);
        self.cache.insert(key, weights);
    }

    /// Number of cached entries.
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Hash a partial RPN state to a cache key.
    /// P8-0C: Canonicalizes tokens before hashing so that commutative-equivalent
    /// expressions (e.g. `A B ADD` and `B A ADD`) produce the same key.
    fn hash_state(tokens: &[u32], stack_depth: u32, feat_offset: u32) -> u64 {
        let canonical = canonicalize_rpn(tokens, feat_offset);
        let mut hash: u64 = 0xcbf29ce484222325; // FNV-1a offset basis
                                                // Include stack depth
        hash ^= stack_depth as u64;
        hash = hash.wrapping_mul(0x100000001b3);
        // Include each token
        for &t in &canonical {
            hash ^= t as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        // Include length to distinguish [0,1] from [0,1,X]
        hash ^= canonical.len() as u64;
        hash = hash.wrapping_mul(0x100000001b3);
        hash
    }
}

impl Policy for LlmCachedPolicy {
    fn prior(&self, legal_actions: &[u32], stack_depth: u32, current_tokens: &[u32]) -> Vec<f64> {
        let n = legal_actions.len();
        if n == 0 {
            return Vec::new();
        }

        let key = Self::hash_state(current_tokens, stack_depth, self.feat_offset);
        if let Some(weights) = self.cache.get(&key) {
            // Extract weights for legal actions only, then normalize
            let mut priors: Vec<f64> = Vec::with_capacity(n);
            for &action in legal_actions {
                let idx = action as usize;
                let w = if idx < weights.len() {
                    weights[idx].max(1e-8) // Ensure non-zero
                } else {
                    1e-8
                };
                priors.push(w);
            }
            let sum: f64 = priors.iter().sum();
            if sum > 0.0 {
                priors.iter_mut().for_each(|p| *p /= sum);
            }
            priors
        } else {
            // Fallback to uniform
            let p = 1.0 / n as f64;
            vec![p; n]
        }
    }
}

/// Build prior weights for LlmCachedPolicy from factor importance and elite tokens.
///
/// Weight construction:
/// 1. Base weight = 1.0 for all tokens
/// 2. Important factors: weight += importance * boost_scale
/// 3. Elite operators: weight *= 1.0 + op_boost_scale (per occurrence, capped at 5)
/// 4. Factors below min_threshold: weight unchanged (stays at 1.0)
pub fn build_llm_prior_weights(
    importance: &[FactorImportance],
    elite_tokens: &[Vec<usize>],
    vocab_size: usize,
    feat_offset: usize,
    config: &LlmMctsPriorConfig,
) -> Vec<f64> {
    let mut weights = vec![1.0; vocab_size];

    // Boost important factors
    for fi in importance {
        if fi.importance >= config.min_importance_threshold && fi.factor_index < feat_offset {
            weights[fi.factor_index] += fi.importance * config.importance_boost_scale;
        }
    }

    // Boost operators from elite formulas
    let mut op_counts = vec![0usize; vocab_size];
    for tokens in elite_tokens {
        for &t in tokens {
            if t >= feat_offset && t < vocab_size {
                op_counts[t] += 1;
            }
        }
    }
    for (i, &count) in op_counts.iter().enumerate() {
        if count > 0 {
            weights[i] *= 1.0 + config.operator_boost_scale * (count as f64).min(5.0);
        }
    }

    weights
}

/// Populate an LlmCachedPolicy cache by generating entries for each prefix
/// of each elite genome.
pub fn populate_policy_cache(
    policy: &mut LlmCachedPolicy,
    importance: &[FactorImportance],
    elite_tokens: &[Vec<usize>],
    vocab_size: usize,
    feat_offset: usize,
    config: &LlmMctsPriorConfig,
) {
    let weights =
        build_llm_prior_weights(importance, elite_tokens, vocab_size, feat_offset, config);

    // For each elite, insert a cache entry for each prefix (including empty)
    for tokens in elite_tokens {
        // Compute stack depth for each prefix
        let mut stack_depth: u32 = 0;
        // Empty prefix
        policy.insert(&[], stack_depth, weights.clone());

        for (i, &t) in tokens.iter().enumerate() {
            if t < feat_offset {
                stack_depth += 1;
            } else {
                let op_idx = t - feat_offset;
                let is_binary = matches!(op_idx, 0..=3 | 16);
                if is_binary && stack_depth >= 2 {
                    stack_depth -= 1; // consume 2, push 1
                }
                // unary: stack_depth stays same (consume 1, push 1)
            }

            let prefix: Vec<u32> = tokens[..=i].iter().map(|&x| x as u32).collect();
            policy.insert(&prefix, stack_depth, weights.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniform_prior_sums_to_one() {
        let policy = UniformPolicy;
        let actions = vec![0, 1, 2, 25, 26];
        let priors = policy.prior(&actions, 1, &[0]);
        let sum: f64 = priors.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
        assert!(priors.iter().all(|&p| (p - 0.2).abs() < 1e-10));
    }

    #[test]
    fn test_uniform_prior_empty() {
        let policy = UniformPolicy;
        let priors = policy.prior(&[], 0, &[]);
        assert!(priors.is_empty());
    }

    #[test]
    fn test_heuristic_prior_sums_to_one() {
        let space = ActionSpace::new(25);
        let policy = HeuristicPolicy::new(&space);
        let actions = vec![0, 1, 25, 30];
        let priors = policy.prior(&actions, 2, &[0, 1]);
        let sum: f64 = priors.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_heuristic_favors_features_early() {
        let space = ActionSpace::new(3);
        let policy = HeuristicPolicy::new(&space);
        // Early position (depth=0): features should have higher prior
        let actions = vec![0, 1, 2, 3, 4]; // 0-2 are features, 3-4 are operators
        let priors = policy.prior(&actions, 1, &[]);
        // Features (indices 0-2) should have higher prior than operators (3-4)
        let feat_prior: f64 = priors[0..3].iter().sum();
        let op_prior: f64 = priors[3..5].iter().sum();
        assert!(feat_prior > op_prior, "Features should be favored early");
    }

    #[test]
    fn test_llm_cached_policy_cache_hit() {
        let vocab_size = 10; // 3 features + 7 operators
        let mut policy = LlmCachedPolicy::new(vocab_size, 3);

        // Insert a prior: heavily favor action 0 (feature 0)
        let mut weights = vec![1.0; vocab_size];
        weights[0] = 10.0; // Strongly prefer feature 0
        policy.insert(&[0], 1, weights);

        let actions = vec![0, 1, 2, 3]; // features 0-2, operator 3
        let priors = policy.prior(&actions, 1, &[0]);

        // Priors should sum to 1
        let sum: f64 = priors.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);

        // Action 0 should have the highest prior
        assert!(
            priors[0] > priors[1],
            "Action 0 should be favored: {:?}",
            priors
        );
    }

    #[test]
    fn test_llm_cached_policy_cache_miss_uniform() {
        let policy = LlmCachedPolicy::new(10, 3);

        // No cache entry for this state → uniform fallback
        let actions = vec![0, 1, 2];
        let priors = policy.prior(&actions, 0, &[]);
        let expected = 1.0 / 3.0;
        for &p in &priors {
            assert!(
                (p - expected).abs() < 1e-10,
                "Expected uniform, got {:?}",
                priors
            );
        }
    }

    #[test]
    fn test_llm_cached_policy_different_states_different_keys() {
        let mut policy = LlmCachedPolicy::new(5, 3);

        // Insert different priors for different states
        let mut w1 = vec![1.0; 5];
        w1[0] = 10.0; // State [0] → favor action 0
        policy.insert(&[0], 1, w1);

        let mut w2 = vec![1.0; 5];
        w2[1] = 10.0; // State [1] → favor action 1
        policy.insert(&[1], 1, w2);

        // Query state [0]: should favor action 0
        let actions = vec![0, 1];
        let priors_s0 = policy.prior(&actions, 1, &[0]);
        assert!(priors_s0[0] > priors_s0[1]);

        // Query state [1]: should favor action 1
        let priors_s1 = policy.prior(&actions, 1, &[1]);
        assert!(priors_s1[1] > priors_s1[0]);
    }

    #[test]
    fn test_llm_cached_policy_cache_size() {
        let mut policy = LlmCachedPolicy::new(5, 3);
        assert_eq!(policy.cache_size(), 0);

        policy.insert(&[0], 1, vec![1.0; 5]);
        assert_eq!(policy.cache_size(), 1);

        policy.insert(&[1], 1, vec![1.0; 5]);
        assert_eq!(policy.cache_size(), 2);

        // Same key overwrites
        policy.insert(&[0], 1, vec![2.0; 5]);
        assert_eq!(policy.cache_size(), 2);
    }

    #[test]
    fn test_canonicalize_rpn_commutative_swap() {
        // feat_offset=3: tokens 0-2 are features, 3+ are operators
        // ADD=3 (offset 0), MUL=5 (offset 2)
        let feat_offset = 3u32;

        // A=1, B=0, ADD=3 → "B A ADD" should canonicalize to "A B ADD" since [0] < [1]
        let tokens = vec![1, 0, 3]; // 1 0 ADD
        let canonical = canonicalize_rpn(&tokens, feat_offset);
        assert_eq!(canonical, vec![0, 1, 3], "Should swap to 0 1 ADD");

        // Already canonical: A=0, B=1, ADD=3
        let tokens = vec![0, 1, 3];
        let canonical = canonicalize_rpn(&tokens, feat_offset);
        assert_eq!(canonical, vec![0, 1, 3], "Already canonical, no swap");
    }

    #[test]
    fn test_canonicalize_rpn_non_commutative_no_swap() {
        // SUB is not commutative (offset 1)
        let feat_offset = 3u32;
        let tokens = vec![1, 0, 4]; // 1 0 SUB — SUB = feat_offset + 1 = 4
        let canonical = canonicalize_rpn(&tokens, feat_offset);
        assert_eq!(canonical, vec![1, 0, 4], "SUB is non-commutative, no swap");
    }

    #[test]
    fn test_canonicalize_rpn_commutative_hash_equivalence() {
        // "A B ADD" and "B A ADD" should produce the same hash
        let feat_offset = 3u32;
        let t1 = vec![0u32, 1, 3]; // A B ADD
        let t2 = vec![1u32, 0, 3]; // B A ADD

        let h1 = LlmCachedPolicy::hash_state(&t1, 1, feat_offset);
        let h2 = LlmCachedPolicy::hash_state(&t2, 1, feat_offset);
        assert_eq!(h1, h2, "Commutative expressions should hash identically");
    }

    #[test]
    fn test_build_llm_prior_weights_boosts_important_factors() {
        let importance = vec![
            FactorImportance {
                factor_index: 0,
                factor_name: "f0".to_string(),
                importance: 0.5,
            },
            FactorImportance {
                factor_index: 1,
                factor_name: "f1".to_string(),
                importance: 0.02,
            }, // below threshold
        ];
        let config = LlmMctsPriorConfig {
            enabled: true,
            importance_recompute_interval: 500,
            importance_boost_scale: 0.5,
            operator_boost_scale: 0.2,
            min_importance_threshold: 0.05,
        };

        let weights = build_llm_prior_weights(&importance, &[], 5, 3, &config);
        // f0: 1.0 + 0.5 * 0.5 = 1.25
        assert!((weights[0] - 1.25).abs() < 1e-10);
        // f1: below threshold, stays at 1.0
        assert!((weights[1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_build_llm_prior_weights_boosts_elite_operators() {
        let config = LlmMctsPriorConfig::default();
        // feat_offset=3, operator ADD=token 3 (offset 0)
        let elite_tokens = vec![vec![0, 1, 3], vec![0, 2, 3]]; // both use ADD(3)

        let weights = build_llm_prior_weights(&[], &elite_tokens, 6, 3, &config);
        // Token 3 (ADD) used 2 times: 1.0 * (1.0 + 0.2 * 2) = 1.4
        assert!((weights[3] - 1.4).abs() < 1e-10);
        // Token 4 (SUB) unused: stays 1.0
        assert!((weights[4] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_populate_policy_cache_creates_entries() {
        let config = LlmMctsPriorConfig::default();
        let mut policy = LlmCachedPolicy::new(6, 3);
        let elite_tokens = vec![vec![0, 1, 3]]; // f0 f1 ADD

        populate_policy_cache(&mut policy, &[], &elite_tokens, 6, 3, &config);
        // Should have entries for: empty prefix, [0], [0,1], [0,1,3]
        assert!(policy.cache_size() >= 4);
    }
}
