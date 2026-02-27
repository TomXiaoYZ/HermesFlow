#![allow(dead_code)] // P6-4A: not yet integrated into ALPS evolution loop
//! Policy network for MCTS action selection.
//!
//! Provides prior probability distributions over legal actions.
//! - `UniformPolicy`: equal probability for all legal actions (baseline).
//! - `HeuristicPolicy`: feature-biased prior (simple heuristic).
//! - `LlmCachedPolicy`: LLM-generated priors with HashMap cache + uniform fallback.

use std::collections::HashMap;

use super::state::ActionSpace;

/// Policy trait: given a partial RPN state, return prior probabilities for each legal action.
pub trait Policy {
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
pub struct HeuristicPolicy {
    feat_offset: usize,
}

impl HeuristicPolicy {
    pub fn new(action_space: &ActionSpace) -> Self {
        Self {
            feat_offset: action_space.feat_offset,
        }
    }
}

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
                if depth < 3 { 2.0 } else { 1.0 }
            } else {
                // Operator: prefer when stack is deep (need to collapse)
                if stack_depth >= 3 { 1.5 } else { 1.0 }
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
    /// Vocabulary size for bounds checking.
    #[allow(dead_code)]
    vocab_size: usize,
}

impl LlmCachedPolicy {
    /// Create an empty LLM policy cache.
    pub fn new(vocab_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            vocab_size,
        }
    }

    /// Insert a prior distribution for a given partial state.
    /// `weights` should have length `vocab_size`, with unnormalized weights per token.
    pub fn insert(&mut self, tokens: &[u32], stack_depth: u32, weights: Vec<f64>) {
        let key = Self::hash_state(tokens, stack_depth);
        self.cache.insert(key, weights);
    }

    /// Number of cached entries.
    #[allow(dead_code)]
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Hash a partial RPN state to a cache key.
    fn hash_state(tokens: &[u32], stack_depth: u32) -> u64 {
        let mut hash: u64 = 0xcbf29ce484222325; // FNV-1a offset basis
        // Include stack depth
        hash ^= stack_depth as u64;
        hash = hash.wrapping_mul(0x100000001b3);
        // Include each token
        for &t in tokens {
            hash ^= t as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        // Include length to distinguish [0,1] from [0,1,X]
        hash ^= tokens.len() as u64;
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

        let key = Self::hash_state(current_tokens, stack_depth);
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
        let mut policy = LlmCachedPolicy::new(vocab_size);

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
        let policy = LlmCachedPolicy::new(10);

        // No cache entry for this state → uniform fallback
        let actions = vec![0, 1, 2];
        let priors = policy.prior(&actions, 0, &[]);
        let expected = 1.0 / 3.0;
        for &p in &priors {
            assert!((p - expected).abs() < 1e-10, "Expected uniform, got {:?}", priors);
        }
    }

    #[test]
    fn test_llm_cached_policy_different_states_different_keys() {
        let mut policy = LlmCachedPolicy::new(5);

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
        let mut policy = LlmCachedPolicy::new(5);
        assert_eq!(policy.cache_size(), 0);

        policy.insert(&[0], 1, vec![1.0; 5]);
        assert_eq!(policy.cache_size(), 1);

        policy.insert(&[1], 1, vec![1.0; 5]);
        assert_eq!(policy.cache_size(), 2);

        // Same key overwrites
        policy.insert(&[0], 1, vec![2.0; 5]);
        assert_eq!(policy.cache_size(), 2);
    }
}
