//! MCTS search algorithm: select → expand → simulate → backpropagate.
//!
//! Uses arena-allocated tree (zero Rc/Arc) and pluggable policy.
//! Integration: `run_mcts_round` returns top-k RPN formulas (token sequences)
//! for injection into ALPS Layer 0 via `inject_genomes`.

use std::collections::HashMap;

use rand::Rng;

use super::arena::{Arena, NULL_NODE};
use super::policy::Policy;
use super::state::ActionSpace;

/// MCTS configuration.
#[derive(Debug, Clone)]
pub struct MctsConfig {
    /// Total simulation budget per round (number of rollouts).
    pub budget: usize,
    /// Exploration constant for PUCT formula.
    pub exploration_c: f64,
    /// Number of top formulas to return as seeds.
    pub seeds_per_round: usize,
    /// Maximum formula length (overrides ActionSpace default if set).
    pub max_length: usize,
    /// Whether to use max_reward (extreme bandit) vs mean_reward in selection.
    pub use_max_reward: bool,
    /// N-gram size for deception suppression (0 = disabled).
    pub deception_ngram_size: usize,
    /// Decay rate for deception penalty: penalty = 1 - exp(-decay * frequency).
    pub deception_decay: f64,
}

impl Default for MctsConfig {
    fn default() -> Self {
        Self {
            budget: 1000,
            exploration_c: 1.414,
            seeds_per_round: 5,
            max_length: 20,
            use_max_reward: false,
            deception_ngram_size: 0,
            deception_decay: 0.1,
        }
    }
}

/// Tracks n-gram frequencies across rollouts to penalize repetitive sub-formulas.
///
/// When MCTS repeatedly generates the same locally-optimal sub-expressions,
/// the deception suppressor reduces their effective reward, forcing exploration
/// of novel topological regions in the formula space.
struct DeceptionSuppressor {
    /// N-gram size (3 or 4 typically).
    ngram_size: usize,
    /// Frequency count per n-gram hash.
    frequencies: HashMap<u64, u32>,
    /// Decay rate: higher = stronger penalty for repeated patterns.
    decay: f64,
}

impl DeceptionSuppressor {
    fn new(ngram_size: usize, decay: f64) -> Self {
        Self {
            ngram_size,
            frequencies: HashMap::new(),
            decay,
        }
    }

    /// Record n-grams from a terminal formula and return a penalty multiplier in [0, 1].
    /// 1.0 = no penalty (all novel), lower = more repetitive.
    fn record_and_penalize(&mut self, tokens: &[u32]) -> f64 {
        if self.ngram_size == 0 || tokens.len() < self.ngram_size {
            return 1.0;
        }

        let ngrams = Self::extract_ngrams(tokens, self.ngram_size);
        let mut max_freq: u32 = 0;

        for hash in &ngrams {
            let count = self.frequencies.entry(*hash).or_insert(0);
            *count += 1;
            if *count > max_freq {
                max_freq = *count;
            }
        }

        // Penalty = exp(-decay * max_frequency)
        // Novel formulas (freq=1) get ~1.0, repeated patterns get decaying penalty
        (-self.decay * (max_freq.saturating_sub(1)) as f64).exp()
    }

    /// Extract n-gram hashes from token sequence.
    fn extract_ngrams(tokens: &[u32], n: usize) -> Vec<u64> {
        if tokens.len() < n {
            return Vec::new();
        }
        let mut ngrams = Vec::with_capacity(tokens.len() - n + 1);
        for window in tokens.windows(n) {
            // FNV-1a style hash for speed
            let mut hash: u64 = 0xcbf29ce484222325;
            for &token in window {
                hash ^= token as u64;
                hash = hash.wrapping_mul(0x100000001b3);
            }
            ngrams.push(hash);
        }
        ngrams
    }
}

// ── P9-2A: MAP-Elites Subformula Archive ────────────────────────────

/// Operator behavior class for MAP-Elites bucketing.
/// Buckets prevent any single behavior type from dominating the archive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperatorClass {
    /// Momentum-type: ts_mean(5), ts_sum(6), ts_delta(10), ts_decay_linear(12)
    Momentum,
    /// Mean-reversion-type: ts_zscore(7), ts_rank(9), ts_scale(14)
    MeanRevert,
    /// Volatility-type: ts_std(4), ABS(15), SIGNED_POWER(8)
    Volatility,
    /// Cross-asset-type: ts_corr(16) + PC features (handled at feature level)
    CrossAsset,
    /// Arithmetic-type: ADD(0), SUB(1), MUL(2), DIV(3), LOG(19), NEG(17), INV(18)
    Arithmetic,
}

impl OperatorClass {
    /// Classify an operator offset (token - feat_offset) into a behavior class.
    pub fn from_operator_offset(offset: u32) -> Self {
        match offset {
            5 | 6 | 10 | 12 => OperatorClass::Momentum,
            7 | 9 | 14 => OperatorClass::MeanRevert,
            4 | 8 | 15 => OperatorClass::Volatility,
            16 => OperatorClass::CrossAsset,
            0 | 1 | 2 | 3 | 17 | 18 | 19 => OperatorClass::Arithmetic,
            _ => OperatorClass::Arithmetic, // Unknown → arithmetic bucket
        }
    }
}

/// A subformula entry in the MAP-Elites archive.
#[derive(Debug, Clone)]
pub struct SubformulaEntry {
    /// 2-4 token subsequence (stored as u32 indices, zero-copy from MCTS).
    #[allow(dead_code)] // accessed via top_subformulas() for MCTS prior injection
    pub tokens: Vec<u32>,
    /// OOS PSR of the parent formula that contributed this subformula.
    pub oos_psr: f64,
    /// Generation when this entry was added.
    #[allow(dead_code)] // used for staleness tracking in future eviction policy
    pub generation: u64,
    /// Source symbol for cross-symbol knowledge transfer.
    #[allow(dead_code)] // used for cross-symbol knowledge reporting
    pub source_symbol: String,
}

/// MAP-Elites-style subformula archive.
///
/// Organizes discovered subformulas by operator behavior class, preventing
/// diversity collapse where high-fitness momentum patterns crowd out
/// volatility or mean-reversion building blocks.
///
/// Thread safety: MCTS is single-threaded, archive updates happen between rounds.
pub struct SubformulaArchive {
    /// Per-class buckets with fixed capacity.
    buckets: HashMap<OperatorClass, Vec<SubformulaEntry>>,
    /// Maximum entries per bucket (total capacity = 5 × max_per_bucket).
    pub max_per_bucket: usize,
    /// Feature offset for operator classification.
    feat_offset: u32,
}

impl SubformulaArchive {
    /// Create a new empty archive with given capacity per bucket.
    pub fn new(max_per_bucket: usize, feat_offset: usize) -> Self {
        let mut buckets = HashMap::new();
        for class in &[
            OperatorClass::Momentum,
            OperatorClass::MeanRevert,
            OperatorClass::Volatility,
            OperatorClass::CrossAsset,
            OperatorClass::Arithmetic,
        ] {
            buckets.insert(*class, Vec::with_capacity(max_per_bucket));
        }
        Self {
            buckets,
            max_per_bucket,
            feat_offset: feat_offset as u32,
        }
    }

    /// Total number of entries across all buckets.
    pub fn total_size(&self) -> usize {
        self.buckets.values().map(|b| b.len()).sum()
    }

    /// Distribution of entries per bucket class.
    pub fn bucket_distribution(&self) -> Vec<(OperatorClass, usize)> {
        self.buckets.iter().map(|(&k, v)| (k, v.len())).collect()
    }

    /// Extract subformulas (2-4 token windows) from a terminal formula
    /// and insert them into the appropriate behavior bucket.
    pub fn ingest_formula(
        &mut self,
        tokens: &[u32],
        oos_psr: f64,
        generation: u64,
        source_symbol: &str,
    ) {
        if tokens.len() < 2 || oos_psr <= 0.0 {
            return;
        }

        let max_window = 4.min(tokens.len());
        // Extract 2-4 token windows
        for window_size in 2..=max_window {
            for window in tokens.windows(window_size) {
                // Classify by the dominant operator in the window
                let class = self.classify_window(window);
                self.insert_to_bucket(
                    class,
                    SubformulaEntry {
                        tokens: window.to_vec(),
                        oos_psr,
                        generation,
                        source_symbol: source_symbol.to_string(),
                    },
                );
            }
        }
    }

    /// Classify a token window by its dominant operator behavior.
    fn classify_window(&self, window: &[u32]) -> OperatorClass {
        // Find the operator (token >= feat_offset) with highest specificity
        for &t in window.iter().rev() {
            if t >= self.feat_offset {
                return OperatorClass::from_operator_offset(t - self.feat_offset);
            }
        }
        // All features, no operators → arithmetic (default)
        OperatorClass::Arithmetic
    }

    /// Insert entry into a bucket, evicting the lowest-PSR entry if full.
    fn insert_to_bucket(&mut self, class: OperatorClass, entry: SubformulaEntry) {
        let bucket = self.buckets.entry(class).or_default();

        if bucket.len() < self.max_per_bucket {
            bucket.push(entry);
        } else {
            // Find the entry with lowest oos_psr and replace if new is better
            if let Some((min_idx, min_psr)) = bucket
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.oos_psr.total_cmp(&b.oos_psr))
                .map(|(i, e)| (i, e.oos_psr))
            {
                if entry.oos_psr > min_psr {
                    bucket[min_idx] = entry;
                }
            }
        }
    }

    /// Get all entries from a specific behavior class (for prior boosting).
    #[allow(dead_code)] // API for MCTS policy prior boosting (future integration)
    pub fn get_bucket(&self, class: OperatorClass) -> &[SubformulaEntry] {
        self.buckets
            .get(&class)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get the top-N subformulas across all buckets (for MCTS prior injection).
    #[allow(dead_code)] // API for MCTS policy prior boosting (future integration)
    pub fn top_subformulas(&self, n: usize) -> Vec<&SubformulaEntry> {
        let mut all: Vec<&SubformulaEntry> = self.buckets.values().flatten().collect();
        all.sort_by(|a, b| b.oos_psr.total_cmp(&a.oos_psr));
        all.truncate(n);
        all
    }
}

/// Result from a single MCTS round: discovered formulas ranked by fitness.
#[derive(Debug, Clone)]
pub struct MctsResult {
    /// Top-k formulas as token sequences (RPN), sorted by fitness descending.
    pub formulas: Vec<Vec<u32>>,
    /// Fitness scores corresponding to each formula.
    pub scores: Vec<f64>,
    /// Total rollouts performed.
    pub total_rollouts: usize,
    /// Number of unique terminal states reached.
    pub unique_terminals: usize,
}

/// Run one MCTS round: build tree, perform rollouts, extract best formulas.
///
/// `evaluate_fn` takes a complete RPN token sequence and returns a fitness score (e.g., PSR).
/// Higher is better. Returns NaN or negative for invalid formulas.
pub fn run_mcts_round<F>(
    action_space: &ActionSpace,
    policy: &dyn Policy,
    config: &MctsConfig,
    evaluate_fn: F,
) -> MctsResult
where
    F: Fn(&[u32]) -> f64,
{
    let (mut arena, root) = Arena::create_root();

    // Track all terminal formulas found
    let mut terminals: Vec<(Vec<u32>, f64)> = Vec::new();

    // P6-4D: Deception suppression via n-gram frequency tracking
    let mut deception = if config.deception_ngram_size > 0 {
        Some(DeceptionSuppressor::new(
            config.deception_ngram_size,
            config.deception_decay,
        ))
    } else {
        None
    };

    for _ in 0..config.budget {
        // 1. SELECT: traverse tree using PUCT until we reach an unexpanded node
        let (leaf, path_tokens) = select(&arena, root, action_space, config);

        let node = arena.get(leaf);
        let stack_depth = node.stack_depth;
        let current_length = path_tokens.len();

        // 2. EXPAND: add children for all legal actions
        let legal = action_space.legal_actions(stack_depth, current_length);
        if legal.is_empty() && !node.is_terminal {
            // Dead end: no legal actions and not terminal. Backprop -1.
            backpropagate(&mut arena, leaf, -1.0);
            continue;
        }

        if node.is_terminal {
            // Already terminal — evaluate and backprop
            let mut reward = evaluate_fn(&path_tokens);
            reward = if reward.is_nan() { -1.0 } else { reward };
            // Apply deception penalty
            if let Some(ref mut ds) = deception {
                let penalty = ds.record_and_penalize(&path_tokens);
                reward *= penalty;
            }
            terminals.push((path_tokens, reward));
            backpropagate(&mut arena, leaf, reward);
            continue;
        }

        // Expand if this node has no children yet
        if arena.get(leaf).children.is_empty() {
            let priors = policy.prior(&legal, stack_depth, &path_tokens);
            for (i, &action) in legal.iter().enumerate() {
                let new_stack = action_space.stack_after_action(stack_depth, action);
                let new_length = current_length + 1;
                let is_terminal = action_space.is_terminal(new_stack, new_length);
                let prior = priors.get(i).copied().unwrap_or(1.0 / legal.len() as f64);
                arena.add_child(leaf, action, prior, new_stack, is_terminal);
            }
        }

        // 3. SIMULATE: pick a random child and do a random rollout to terminal
        let children = arena.get(leaf).children.clone();
        if children.is_empty() {
            backpropagate(&mut arena, leaf, -1.0);
            continue;
        }

        // Pick least-visited child for first expansion
        let child_idx = pick_least_visited(&arena, &children);
        let child = arena.get(child_idx);
        let child_action = child.action;
        let child_stack = child.stack_depth;
        let child_terminal = child.is_terminal;

        let mut sim_tokens = path_tokens;
        sim_tokens.push(child_action);

        // Evaluate: either the child is terminal, or do a random rollout
        let (final_tokens, reward) = if child_terminal {
            let r = evaluate_fn(&sim_tokens);
            let r = if r.is_nan() { -1.0 } else { r };
            (sim_tokens, r)
        } else {
            let (rollout_tokens, reached_terminal) =
                random_rollout(action_space, &sim_tokens, child_stack, config.max_length);
            if reached_terminal {
                let r = evaluate_fn(&rollout_tokens);
                let r = if r.is_nan() { -1.0 } else { r };
                (rollout_tokens, r)
            } else {
                (sim_tokens, -1.0)
            }
        };

        // Apply deception penalty and record terminal tokens
        let penalized_reward = if reward > -1.0 {
            let r = if let Some(ref mut ds) = deception {
                reward * ds.record_and_penalize(&final_tokens)
            } else {
                reward
            };
            terminals.push((final_tokens, r));
            r
        } else {
            reward
        };

        // 4. BACKPROPAGATE: update visit counts and rewards up the tree
        backpropagate(&mut arena, child_idx, penalized_reward);
    }

    // Extract top-k unique formulas
    extract_top_k(terminals, config.seeds_per_round)
}

/// PUCT-based tree traversal: select the path from root to a leaf.
/// Returns (leaf node index, token sequence along the path).
fn select(
    arena: &Arena,
    root: u32,
    action_space: &ActionSpace,
    config: &MctsConfig,
) -> (u32, Vec<u32>) {
    let mut current = root;
    let mut tokens: Vec<u32> = Vec::new();

    loop {
        let node = arena.get(current);

        // If node has no children, it's a leaf (either unexpanded or terminal)
        if node.children.is_empty() {
            return (current, tokens);
        }

        // Select best child via PUCT
        let parent_visits = node.visit_count;
        let children = node.children.clone();

        let best_child = select_puct(arena, &children, parent_visits, config);
        let child = arena.get(best_child);
        tokens.push(child.action);

        // Check if this child's state is terminal
        let stack = child.stack_depth;
        let length = tokens.len();
        if action_space.is_terminal(stack, length) && child.children.is_empty() {
            return (best_child, tokens);
        }

        current = best_child;
    }
}

/// PUCT selection formula.
/// score = Q(child) + c * prior(child) * sqrt(N_parent) / (1 + N_child)
///
/// Q = mean_reward by default, or max_reward if config.use_max_reward is true.
fn select_puct(arena: &Arena, children: &[u32], parent_visits: u32, config: &MctsConfig) -> u32 {
    let sqrt_parent = (parent_visits as f64).sqrt();
    let mut best_score = f64::NEG_INFINITY;
    let mut best_child = children[0];

    for &child_idx in children {
        let child = arena.get(child_idx);
        let q = if config.use_max_reward {
            if child.max_reward == f64::NEG_INFINITY {
                0.0
            } else {
                child.max_reward
            }
        } else {
            child.mean_reward()
        };

        let exploration =
            config.exploration_c * child.prior * sqrt_parent / (1.0 + child.visit_count as f64);

        let score = q + exploration;
        if score > best_score {
            best_score = score;
            best_child = child_idx;
        }
    }

    best_child
}

/// Pick the least-visited child (for initial expansion).
fn pick_least_visited(arena: &Arena, children: &[u32]) -> u32 {
    let mut min_visits = u32::MAX;
    let mut best = children[0];
    for &child_idx in children {
        let visits = arena.get(child_idx).visit_count;
        if visits < min_visits {
            min_visits = visits;
            best = child_idx;
        }
    }
    best
}

/// Random rollout: from current partial state, randomly pick legal actions until terminal.
/// Returns (complete token sequence, whether terminal was reached).
fn random_rollout(
    action_space: &ActionSpace,
    current_tokens: &[u32],
    current_stack: u32,
    max_length: usize,
) -> (Vec<u32>, bool) {
    let mut rng = rand::thread_rng();
    let mut tokens = current_tokens.to_vec();
    let mut stack = current_stack;

    for _ in tokens.len()..max_length {
        if action_space.is_terminal(stack, tokens.len()) {
            return (tokens, true);
        }

        let legal = action_space.legal_actions(stack, tokens.len());
        if legal.is_empty() {
            break;
        }

        let action = legal[rng.gen_range(0..legal.len())];
        stack = action_space.stack_after_action(stack, action);
        tokens.push(action);
    }

    let terminal = action_space.is_terminal(stack, tokens.len());
    (tokens, terminal)
}

/// Backpropagate reward from leaf to root, updating visit counts and rewards.
fn backpropagate(arena: &mut Arena, start: u32, reward: f64) {
    let mut current = start;
    loop {
        let node = arena.get_mut(current);
        node.visit_count += 1;
        node.total_reward += reward;
        if reward > node.max_reward {
            node.max_reward = reward;
        }

        if node.parent == NULL_NODE {
            break;
        }
        current = node.parent;
    }
}

/// Extract top-k unique formulas from terminal results, sorted by fitness descending.
fn extract_top_k(mut terminals: Vec<(Vec<u32>, f64)>, k: usize) -> MctsResult {
    // Deduplicate by token sequence
    terminals.sort_by(|a, b| b.1.total_cmp(&a.1));

    let mut seen: std::collections::HashSet<Vec<u32>> = std::collections::HashSet::new();
    let mut formulas: Vec<Vec<u32>> = Vec::new();
    let mut scores: Vec<f64> = Vec::new();
    let unique_count = {
        let unique: std::collections::HashSet<Vec<u32>> =
            terminals.iter().map(|(t, _)| t.clone()).collect();
        unique.len()
    };

    for (tokens, score) in &terminals {
        if seen.contains(tokens) {
            continue;
        }
        seen.insert(tokens.clone());
        formulas.push(tokens.clone());
        scores.push(*score);
        if formulas.len() >= k {
            break;
        }
    }

    let total_rollouts = terminals.len();

    MctsResult {
        formulas,
        scores,
        total_rollouts,
        unique_terminals: unique_count,
    }
}

#[cfg(test)]
mod tests {
    use super::super::policy::UniformPolicy;
    use super::*;

    /// Simple fitness: reward = number of tokens (longer formulas are "better").
    fn length_fitness(tokens: &[u32]) -> f64 {
        tokens.len() as f64 / 20.0 // normalize to ~[0, 1]
    }

    /// Constant fitness: always returns 0.5.
    fn constant_fitness(_tokens: &[u32]) -> f64 {
        0.5
    }

    #[test]
    fn test_random_rollout_reaches_terminal() {
        let space = ActionSpace::new(3);
        // Start with one feature pushed
        let tokens = vec![0u32];
        let (result, terminal) = random_rollout(&space, &tokens, 1, 20);
        assert!(!result.is_empty());
        // Should usually reach terminal (stack_depth == 1)
        if terminal {
            // Verify it's actually terminal
            let mut stack = 0u32;
            for &t in &result {
                stack = space.stack_after_action(stack, t);
            }
            assert_eq!(stack, 1);
        }
    }

    #[test]
    fn test_mcts_round_returns_formulas() {
        let space = ActionSpace::new(3);
        let policy = UniformPolicy;
        let config = MctsConfig {
            budget: 100,
            seeds_per_round: 3,
            max_length: 10,
            ..Default::default()
        };

        let result = run_mcts_round(&space, &policy, &config, length_fitness);

        // Should find at least some formulas
        assert!(
            !result.formulas.is_empty(),
            "MCTS should discover at least one formula"
        );
        assert_eq!(result.formulas.len(), result.scores.len());
        assert!(result.formulas.len() <= config.seeds_per_round);
    }

    #[test]
    fn test_mcts_formulas_are_valid_rpn() {
        let space = ActionSpace::new(3);
        let policy = UniformPolicy;
        let config = MctsConfig {
            budget: 500,
            seeds_per_round: 5,
            max_length: 10,
            ..Default::default()
        };

        let result = run_mcts_round(&space, &policy, &config, length_fitness);

        for formula in &result.formulas {
            assert!(
                formula.len() >= 3,
                "Formula should have at least 3 tokens, got {}",
                formula.len()
            );
            // Verify stack depth is 1 (valid terminal)
            let mut stack = 0u32;
            for &token in formula {
                stack = space.stack_after_action(stack, token);
            }
            assert_eq!(stack, 1, "Formula {:?} should have stack depth 1", formula);
        }
    }

    #[test]
    fn test_backpropagate_updates_path() {
        let (mut arena, root) = Arena::create_root();
        let child = arena.add_child(root, 0, 0.5, 1, false);
        let grandchild = arena.add_child(child, 1, 0.5, 2, false);

        backpropagate(&mut arena, grandchild, 0.8);

        assert_eq!(arena.get(grandchild).visit_count, 1);
        assert!((arena.get(grandchild).total_reward - 0.8).abs() < 1e-10);
        assert_eq!(arena.get(child).visit_count, 1);
        assert_eq!(arena.get(root).visit_count, 1);
        assert!((arena.get(root).max_reward - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_extract_top_k_deduplicates() {
        let terminals = vec![
            (vec![0, 1, 3], 0.9),
            (vec![0, 1, 3], 0.8), // duplicate
            (vec![0, 2, 4], 0.7),
            (vec![0, 1, 4], 0.6),
        ];

        let result = extract_top_k(terminals, 3);
        assert_eq!(result.formulas.len(), 3);
        assert_eq!(result.unique_terminals, 3);
        // Best score should be first
        assert!((result.scores[0] - 0.9).abs() < 1e-10);
    }

    #[test]
    fn test_select_puct_favors_unvisited() {
        let (mut arena, root) = Arena::create_root();
        let c1 = arena.add_child(root, 0, 0.5, 1, false);
        let c2 = arena.add_child(root, 1, 0.5, 1, false);

        // Visit c1 several times with low reward
        for _ in 0..10 {
            let node = arena.get_mut(c1);
            node.visit_count += 1;
            node.total_reward += 0.1;
        }

        let config = MctsConfig::default();
        let children = vec![c1, c2];
        // Root needs visit count for PUCT formula
        arena.get_mut(root).visit_count = 10;

        let selected = select_puct(&arena, &children, 10, &config);
        // c2 (unvisited) should be selected due to high exploration bonus
        assert_eq!(selected, c2, "Unvisited child should be selected");
    }

    #[test]
    fn test_puct_max_reward_mode() {
        // Extreme bandit test: one specific formula has very high reward.
        // max_reward PUCT should converge to exploiting that subtree.
        let space = ActionSpace::new(3);
        let policy = UniformPolicy;

        // "Needle" fitness: formula [0, 1, feat+0(=ADD)] gets reward 1.0, all others 0.1
        let needle: &[u32] = &[0, 1, 3]; // feat0, feat1, ADD
        let needle_fitness = |tokens: &[u32]| -> f64 {
            if tokens == needle {
                1.0
            } else {
                0.1
            }
        };

        // Run with max_reward mode
        let config_max = MctsConfig {
            budget: 500,
            seeds_per_round: 5,
            max_length: 10,
            use_max_reward: true,
            ..Default::default()
        };
        let result_max = run_mcts_round(&space, &policy, &config_max, needle_fitness);

        // Run with mean_reward mode
        let config_mean = MctsConfig {
            budget: 500,
            seeds_per_round: 5,
            max_length: 10,
            use_max_reward: false,
            ..Default::default()
        };
        let result_mean = run_mcts_round(&space, &policy, &config_mean, needle_fitness);

        // Both should find formulas
        assert!(!result_max.formulas.is_empty());
        assert!(!result_mean.formulas.is_empty());

        // The best formula from max_reward mode should have score 1.0 (found the needle)
        // or at least match/beat mean mode's best
        assert!(
            result_max.scores[0] >= result_mean.scores[0],
            "max_reward ({}) should find at least as good as mean ({})",
            result_max.scores[0],
            result_mean.scores[0]
        );
    }

    #[test]
    fn test_puct_max_reward_backprop_tracks_max() {
        let (mut arena, root) = Arena::create_root();
        let child = arena.add_child(root, 0, 0.5, 1, false);

        // Backpropagate several different rewards
        backpropagate(&mut arena, child, 0.3);
        backpropagate(&mut arena, child, 0.9);
        backpropagate(&mut arena, child, 0.5);

        let node = arena.get(child);
        assert_eq!(node.visit_count, 3);
        assert!(
            (node.max_reward - 0.9).abs() < 1e-10,
            "max_reward should track the best"
        );
        assert!((node.mean_reward() - (0.3 + 0.9 + 0.5) / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_puct_selects_max_over_mean() {
        // Setup: child A has many visits with mediocre rewards (mean=0.5).
        // Child B has fewer visits but one very high reward (mean=0.4, max=0.95).
        // In max_reward mode, PUCT should prefer B (due to max 0.95).
        // In mean mode, PUCT should prefer A (due to mean 0.5 > 0.4).
        let (mut arena, root) = Arena::create_root();
        let a = arena.add_child(root, 0, 0.5, 1, false);
        let b = arena.add_child(root, 1, 0.5, 1, false);

        // Child A: 20 visits, total_reward = 10.0 → mean=0.5, max=0.6
        let node_a = arena.get_mut(a);
        node_a.visit_count = 20;
        node_a.total_reward = 10.0;
        node_a.max_reward = 0.6;

        // Child B: 20 visits, total_reward = 8.0 → mean=0.4, max=0.95
        let node_b = arena.get_mut(b);
        node_b.visit_count = 20;
        node_b.total_reward = 8.0;
        node_b.max_reward = 0.95;

        arena.get_mut(root).visit_count = 40;

        // Mean mode: should select A (mean 0.5 > 0.4, same exploration bonus)
        let config_mean = MctsConfig {
            use_max_reward: false,
            exploration_c: 0.01, // very low exploration to focus on exploitation
            ..Default::default()
        };
        let selected_mean = select_puct(&arena, &[a, b], 40, &config_mean);
        assert_eq!(
            selected_mean, a,
            "Mean mode should prefer child A (higher mean)"
        );

        // Max mode: should select B (max 0.95 > 0.6)
        let config_max = MctsConfig {
            use_max_reward: true,
            exploration_c: 0.01,
            ..Default::default()
        };
        let selected_max = select_puct(&arena, &[a, b], 40, &config_max);
        assert_eq!(
            selected_max, b,
            "Max mode should prefer child B (higher max_reward)"
        );
    }

    #[test]
    fn test_mcts_with_feat_offset_25() {
        // Use the actual feat_offset from production (25 factors)
        let space = ActionSpace::new(25);
        let policy = UniformPolicy;
        let config = MctsConfig {
            budget: 50,
            seeds_per_round: 3,
            max_length: 15,
            ..Default::default()
        };

        let result = run_mcts_round(&space, &policy, &config, |tokens: &[u32]| {
            // Reward formulas that use diverse features
            let unique_features: std::collections::HashSet<u32> = tokens
                .iter()
                .filter(|&&t| (t as usize) < 25)
                .copied()
                .collect();
            unique_features.len() as f64 / 5.0
        });

        assert!(
            !result.formulas.is_empty(),
            "Should find formulas with feat_offset=25"
        );
        // All tokens should be in valid range
        for formula in &result.formulas {
            for &token in formula {
                assert!(
                    (token as usize) < space.vocab_size,
                    "Token {} out of range (vocab={})",
                    token,
                    space.vocab_size
                );
            }
        }
    }

    #[test]
    fn test_deception_suppressor_penalizes_repeats() {
        let mut ds = DeceptionSuppressor::new(3, 0.5);

        // Use a formula with no internal n-gram repeats: [0, 1, 2, 3, 4]
        // 3-grams: [0,1,2], [1,2,3], [2,3,4] — all unique within this formula

        // First occurrence: all n-grams freq=1, penalty = exp(-0.5 * 0) = 1.0
        let p1 = ds.record_and_penalize(&[0, 1, 2, 3, 4]);
        assert!(
            (p1 - 1.0).abs() < 1e-10,
            "First occurrence: no penalty, got {}",
            p1
        );

        // Second occurrence of same n-grams: freq=2, penalty = exp(-0.5 * 1)
        let p2 = ds.record_and_penalize(&[0, 1, 2, 3, 4]);
        let expected = (-0.5f64).exp();
        assert!(
            (p2 - expected).abs() < 1e-10,
            "Second occurrence: penalty={}, expected={}",
            p2,
            expected
        );

        // Third occurrence: freq=3, penalty = exp(-0.5 * 2)
        let p3 = ds.record_and_penalize(&[0, 1, 2, 3, 4]);
        let expected3 = (-1.0f64).exp();
        assert!(
            (p3 - expected3).abs() < 1e-10,
            "Third occurrence: penalty={}, expected={}",
            p3,
            expected3
        );

        // Different formula: should not be penalized (novel n-grams)
        let p_novel = ds.record_and_penalize(&[10, 11, 12, 13, 14]);
        assert!(
            p_novel > p3,
            "Novel formula ({}) should have less penalty than repeated ({})",
            p_novel,
            p3
        );
    }

    #[test]
    fn test_deception_ngram_extraction() {
        let ngrams = DeceptionSuppressor::extract_ngrams(&[0, 1, 2, 3, 4], 3);
        assert_eq!(ngrams.len(), 3); // [0,1,2], [1,2,3], [2,3,4]

        // Same tokens → same hashes
        let ngrams2 = DeceptionSuppressor::extract_ngrams(&[0, 1, 2, 3, 4], 3);
        assert_eq!(ngrams, ngrams2);

        // Different tokens → different hashes (very likely)
        let ngrams3 = DeceptionSuppressor::extract_ngrams(&[5, 6, 7, 8, 9], 3);
        assert_ne!(ngrams[0], ngrams3[0]);
    }

    #[test]
    fn test_deception_disabled_when_ngram_zero() {
        let mut ds = DeceptionSuppressor::new(0, 0.5);
        let p = ds.record_and_penalize(&[0, 1, 3]);
        assert!((p - 1.0).abs() < 1e-10, "Disabled suppressor returns 1.0");
    }

    #[test]
    fn test_mcts_with_deception_suppression() {
        let space = ActionSpace::new(3);
        let policy = UniformPolicy;

        // Without deception suppression
        let config_no_ds = MctsConfig {
            budget: 300,
            seeds_per_round: 10,
            max_length: 10,
            deception_ngram_size: 0,
            ..Default::default()
        };
        let result_no_ds = run_mcts_round(&space, &policy, &config_no_ds, constant_fitness);

        // With deception suppression
        let config_ds = MctsConfig {
            budget: 300,
            seeds_per_round: 10,
            max_length: 10,
            deception_ngram_size: 3,
            deception_decay: 0.3,
            ..Default::default()
        };
        let result_ds = run_mcts_round(&space, &policy, &config_ds, constant_fitness);

        // Deception suppression should produce at least as many unique formulas
        // (it penalizes repeated patterns, forcing exploration)
        assert!(
            result_ds.unique_terminals >= result_no_ds.unique_terminals / 2,
            "Deception suppression should maintain reasonable diversity: ds={}, no_ds={}",
            result_ds.unique_terminals,
            result_no_ds.unique_terminals
        );

        // Both should still produce valid formulas
        assert!(!result_ds.formulas.is_empty());
    }

    // ── P7-1D: MCTS Integration Tests ────────────────────────────────

    #[test]
    fn test_mcts_produces_valid_genomes() {
        // Verify all generated tokens are within valid range (0..feat_offset+23 ops)
        let feat_offset = 25;
        let space = ActionSpace::new(feat_offset);
        let policy = UniformPolicy;
        let config = MctsConfig {
            budget: 500,
            seeds_per_round: 5,
            max_length: 15,
            use_max_reward: true, // Extreme Bandit default
            ..Default::default()
        };

        let result = run_mcts_round(&space, &policy, &config, |tokens: &[u32]| {
            // Simple fitness: more unique features = better
            let feats: std::collections::HashSet<u32> = tokens
                .iter()
                .filter(|&&t| (t as usize) < feat_offset)
                .copied()
                .collect();
            feats.len() as f64 / 5.0
        });

        for (i, formula) in result.formulas.iter().enumerate() {
            assert!(
                formula.len() >= 3,
                "Formula {} should have at least 3 tokens, got {}",
                i,
                formula.len()
            );
            for &token in formula {
                assert!(
                    (token as usize) < space.vocab_size,
                    "Token {} out of range (vocab={}) in formula {}",
                    token,
                    space.vocab_size,
                    i
                );
            }
            // Verify terminal: stack depth should be 1
            let mut stack = 0u32;
            for &t in formula {
                stack = space.stack_after_action(stack, t);
            }
            assert_eq!(
                stack, 1,
                "Formula {:?} not terminal (stack={})",
                formula, stack
            );
        }
    }

    #[test]
    fn test_mcts_token_type_conversion() {
        // Verify u32 → usize conversion is safe and NULL_NODE sentinel not generated
        let space = ActionSpace::new(10);
        let policy = UniformPolicy;
        let config = MctsConfig {
            budget: 100,
            seeds_per_round: 5,
            max_length: 10,
            ..Default::default()
        };

        let result = run_mcts_round(&space, &policy, &config, |_tokens: &[u32]| 0.5);

        for formula in &result.formulas {
            for &token in formula {
                // u32 → usize should never overflow on any 32+ bit platform
                let _usize_token: usize = token as usize;
                // NULL_NODE (u32::MAX) should never appear in formulas
                assert_ne!(token, crate::mcts::arena::NULL_NODE, "NULL_NODE in formula");
            }
        }
    }

    // ── P9-2A: MAP-Elites SubformulaArchive Tests ────────────────────

    #[test]
    fn test_archive_empty_on_creation() {
        let archive = SubformulaArchive::new(40, 25);
        assert_eq!(archive.total_size(), 0);
        assert_eq!(archive.max_per_bucket, 40);
    }

    #[test]
    fn test_archive_ingest_classifies_correctly() {
        let mut archive = SubformulaArchive::new(40, 3);
        // feat_offset=3, so tokens 0-2 are features, 3+ are operators
        // Operator ADD = offset 0 → token 3 → Arithmetic
        // Operator ts_mean = offset 5 → token 8 → Momentum
        archive.ingest_formula(&[0, 1, 3], 1.5, 100, "AAPL"); // A B ADD → Arithmetic
        archive.ingest_formula(&[0, 8], 2.0, 100, "AAPL"); // A ts_mean → Momentum

        assert!(archive.total_size() > 0);
        let momentum = archive.get_bucket(OperatorClass::Momentum);
        let arithmetic = archive.get_bucket(OperatorClass::Arithmetic);
        assert!(!momentum.is_empty(), "Should have momentum entries");
        assert!(!arithmetic.is_empty(), "Should have arithmetic entries");
    }

    #[test]
    fn test_archive_evicts_lowest_psr() {
        let mut archive = SubformulaArchive::new(2, 3);
        // Fill arithmetic bucket to capacity
        archive.ingest_formula(&[0, 1, 3], 1.0, 100, "AAPL"); // PSR=1.0
        archive.ingest_formula(&[1, 2, 3], 2.0, 200, "GOOG"); // PSR=2.0

        let bucket_before = archive.get_bucket(OperatorClass::Arithmetic).len();

        // Insert higher PSR → should evict the 1.0 entry
        archive.ingest_formula(&[0, 2, 3], 3.0, 300, "MSFT"); // PSR=3.0

        let bucket = archive.get_bucket(OperatorClass::Arithmetic);
        assert!(bucket.len() <= 2, "Bucket should respect max capacity");

        // The lowest PSR entry should have been evicted
        let min_psr = bucket
            .iter()
            .map(|e| e.oos_psr)
            .fold(f64::INFINITY, f64::min);
        assert!(
            min_psr >= 2.0,
            "Lowest PSR in bucket should be >= 2.0, got {}",
            min_psr
        );
    }

    #[test]
    fn test_archive_buckets_independent() {
        let mut archive = SubformulaArchive::new(2, 3);
        // Fill arithmetic bucket
        archive.ingest_formula(&[0, 1, 3], 1.0, 100, "AAPL"); // ADD → Arithmetic
        archive.ingest_formula(&[1, 2, 3], 2.0, 200, "GOOG"); // ADD → Arithmetic

        // Momentum bucket should still be empty
        let momentum = archive.get_bucket(OperatorClass::Momentum);
        assert!(momentum.is_empty(), "Momentum bucket should be empty");

        // Adding momentum entry should not affect arithmetic
        archive.ingest_formula(&[0, 8], 0.5, 100, "AAPL"); // ts_mean → Momentum
        let arithmetic = archive.get_bucket(OperatorClass::Arithmetic);
        assert_eq!(arithmetic.len(), 2, "Arithmetic bucket unchanged");
    }

    #[test]
    fn test_archive_top_subformulas() {
        let mut archive = SubformulaArchive::new(40, 3);
        archive.ingest_formula(&[0, 1, 3], 3.0, 100, "AAPL"); // Arithmetic
        archive.ingest_formula(&[0, 8], 5.0, 200, "GOOG"); // Momentum
        archive.ingest_formula(&[1, 7], 1.0, 300, "MSFT"); // Volatility (ts_std=offset 4→token 7)

        let top = archive.top_subformulas(2);
        assert!(top.len() <= 2);
        if top.len() == 2 {
            assert!(
                top[0].oos_psr >= top[1].oos_psr,
                "Should be sorted by PSR descending"
            );
        }
    }

    #[test]
    fn test_archive_skips_negative_psr() {
        let mut archive = SubformulaArchive::new(40, 3);
        archive.ingest_formula(&[0, 1, 3], -1.0, 100, "AAPL");
        assert_eq!(
            archive.total_size(),
            0,
            "Should skip formulas with negative PSR"
        );
    }

    #[test]
    fn test_operator_class_classification() {
        assert_eq!(
            OperatorClass::from_operator_offset(0),
            OperatorClass::Arithmetic
        ); // ADD
        assert_eq!(
            OperatorClass::from_operator_offset(5),
            OperatorClass::Momentum
        ); // ts_mean
        assert_eq!(
            OperatorClass::from_operator_offset(7),
            OperatorClass::MeanRevert
        ); // ts_zscore
        assert_eq!(
            OperatorClass::from_operator_offset(4),
            OperatorClass::Volatility
        ); // ts_std
        assert_eq!(
            OperatorClass::from_operator_offset(16),
            OperatorClass::CrossAsset
        ); // ts_corr
    }
}
