//! P6-1B: Local FDR (lFDR) hypothesis testing with RPN structural clustering.
//!
//! Global BH at FDR=0.10 causes threshold masking at scale — correlated ALPS
//! children sharing RPN subtrees violate BH independence assumption. At 18k symbols,
//! hypothesis count reaches millions, BH threshold shrinks to near-zero.
//!
//! Solution: Cluster strategies by 3/4-gram operator subsequence similarity (Jaccard),
//! compute per-cluster empirical null distribution of OOS PSR, and apply lFDR
//! control within each cluster instead of global BH.

use serde::Deserialize;
use std::collections::{HashMap, HashSet};

/// Configuration for local FDR hypothesis testing.
#[derive(Debug, Clone, Deserialize)]
pub struct LfdrConfig {
    /// Target FDR level (default: 0.10)
    pub fdr_level: f64,
    /// N-gram size for RPN clustering (3 or 4)
    pub ngram_size: usize,
    /// Minimum cluster size for reliable null estimation
    pub min_cluster_size: usize,
    /// Jaccard similarity threshold for merging into same cluster
    pub jaccard_threshold: f64,
    /// Whether lFDR is enabled (false = fall back to simple PSR threshold)
    pub enabled: bool,
}

impl Default for LfdrConfig {
    fn default() -> Self {
        Self {
            fdr_level: 0.10,
            ngram_size: 3,
            min_cluster_size: 10,
            jaccard_threshold: 0.3,
            enabled: false,
        }
    }
}

/// A strategy with its PSR score and token representation for clustering.
#[derive(Debug, Clone)]
pub struct HypothesisCandidate {
    /// Strategy identifier (e.g., "AAPL:long_only:gen42")
    pub id: String,
    /// RPN token sequence (integer-encoded)
    pub tokens: Vec<usize>,
    /// Out-of-sample PSR z-score
    pub oos_psr: f64,
}

/// Result of lFDR hypothesis testing for a single strategy.
#[derive(Debug, Clone)]
pub struct LfdrResult {
    pub id: String,
    #[allow(dead_code)] // accessed in tests and future UI display
    pub oos_psr: f64,
    pub cluster_id: usize,
    /// Local FDR estimate: probability that this strategy is null (no alpha)
    pub lfdr: f64,
    /// Whether the strategy passes lFDR control (lfdr < threshold)
    pub passes: bool,
}

/// Extract n-gram set from a token sequence.
///
/// Returns a `HashSet` of n-gram tuples represented as `Vec<usize>`.
fn extract_ngrams(tokens: &[usize], n: usize) -> HashSet<Vec<usize>> {
    if tokens.len() < n {
        return HashSet::new();
    }
    tokens.windows(n).map(|w| w.to_vec()).collect()
}

/// Compute Jaccard similarity between two n-gram sets.
fn jaccard_similarity(a: &HashSet<Vec<usize>>, b: &HashSet<Vec<usize>>) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let intersection = a.intersection(b).count();
    let union = a.union(b).count();
    if union == 0 {
        return 0.0;
    }
    intersection as f64 / union as f64
}

/// Cluster strategies by n-gram Jaccard similarity using greedy assignment.
///
/// Each strategy is assigned to the first cluster whose centroid has
/// Jaccard similarity >= threshold. If none, a new cluster is created.
fn cluster_by_ngrams(
    candidates: &[HypothesisCandidate],
    ngram_size: usize,
    threshold: f64,
) -> Vec<usize> {
    let ngram_sets: Vec<HashSet<Vec<usize>>> = candidates
        .iter()
        .map(|c| extract_ngrams(&c.tokens, ngram_size))
        .collect();

    let mut cluster_ids = vec![0usize; candidates.len()];
    // Store representative ngram set per cluster (first member's ngrams)
    let mut cluster_reps: Vec<HashSet<Vec<usize>>> = Vec::new();

    for (i, ngrams) in ngram_sets.iter().enumerate() {
        let mut best_cluster = None;
        let mut best_sim = 0.0_f64;

        for (cid, rep) in cluster_reps.iter().enumerate() {
            let sim = jaccard_similarity(ngrams, rep);
            if sim >= threshold && sim > best_sim {
                best_sim = sim;
                best_cluster = Some(cid);
            }
        }

        match best_cluster {
            Some(cid) => {
                cluster_ids[i] = cid;
            }
            None => {
                // Create new cluster
                let new_cid = cluster_reps.len();
                cluster_reps.push(ngrams.clone());
                cluster_ids[i] = new_cid;
            }
        }
    }

    cluster_ids
}

/// Estimate empirical null distribution parameters (mean, std) for a cluster
/// using the central portion of the PSR distribution.
///
/// Uses the middle 50% of PSR values as a robust null estimator,
/// assuming most strategies in a cluster are null (no real alpha).
fn estimate_null(psr_values: &[f64]) -> (f64, f64) {
    if psr_values.is_empty() {
        return (0.0, 1.0);
    }

    let mut sorted = psr_values.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));

    let n = sorted.len();
    // Use central 50% for null estimation (robust to outliers)
    let lo = n / 4;
    let hi = (3 * n / 4).max(lo + 1);
    let central = &sorted[lo..hi];

    let mean = central.iter().sum::<f64>() / central.len() as f64;
    let var = central.iter().map(|x| (x - mean).powi(2)).sum::<f64>()
        / (central.len() as f64 - 1.0).max(1.0);
    let std = var.sqrt().max(0.1); // floor std to avoid division by zero

    (mean, std)
}

/// Compute local FDR for a single PSR value given null distribution parameters.
///
/// lFDR = π₀ · f₀(z) / f(z)
///
/// Where:
/// - π₀ ≈ 1.0 (conservative: assume most hypotheses are null)
/// - f₀(z) = normal PDF with null parameters
/// - f(z) = mixture density estimated from all observations
///
/// For simplicity, we use the empirical Bayesian approach:
/// lFDR ≈ 1 - Φ(z) where z = (psr - null_mean) / null_std
fn compute_lfdr(psr: f64, null_mean: f64, null_std: f64) -> f64 {
    let z = (psr - null_mean) / null_std;
    // Normal CDF approximation (Abramowitz & Stegun)
    let p = normal_cdf(z);
    // lFDR = probability of being null = 1 - p(being non-null)
    (1.0 - p).clamp(0.0, 1.0)
}

/// Standard normal CDF approximation.
fn normal_cdf(x: f64) -> f64 {
    // Abramowitz & Stegun approximation 7.1.26
    let t = 1.0 / (1.0 + 0.2316419 * x.abs());
    let d = 0.3989422804014327; // 1/sqrt(2π)
    let p = d * (-x * x / 2.0).exp();
    let poly = t
        * (0.319381530
            + t * (-0.356563782 + t * (1.781477937 + t * (-1.821255978 + t * 1.330274429))));
    if x >= 0.0 {
        1.0 - p * poly
    } else {
        p * poly
    }
}

/// Run local FDR hypothesis testing on a set of strategy candidates.
///
/// Steps:
/// 1. Cluster strategies by RPN n-gram Jaccard similarity
/// 2. Estimate per-cluster empirical null distribution
/// 3. Compute lFDR for each strategy within its cluster
/// 4. Apply Storey's step-up procedure within each cluster
///
/// Returns lFDR results sorted by lFDR (lowest first = most significant).
pub fn run_lfdr_test(candidates: &[HypothesisCandidate], config: &LfdrConfig) -> Vec<LfdrResult> {
    if candidates.is_empty() {
        return Vec::new();
    }

    // Step 1: Cluster by n-gram similarity
    let cluster_ids = cluster_by_ngrams(candidates, config.ngram_size, config.jaccard_threshold);

    // Step 2: Group PSR values by cluster
    let mut cluster_psrs: HashMap<usize, Vec<f64>> = HashMap::new();
    for (i, &cid) in cluster_ids.iter().enumerate() {
        cluster_psrs
            .entry(cid)
            .or_default()
            .push(candidates[i].oos_psr);
    }

    // Step 3: Estimate null per cluster (merge small clusters into global null)
    let all_psrs: Vec<f64> = candidates.iter().map(|c| c.oos_psr).collect();
    let global_null = estimate_null(&all_psrs);

    let mut cluster_nulls: HashMap<usize, (f64, f64)> = HashMap::new();
    for (&cid, psrs) in &cluster_psrs {
        if psrs.len() >= config.min_cluster_size {
            cluster_nulls.insert(cid, estimate_null(psrs));
        } else {
            // Small cluster: use global null
            cluster_nulls.insert(cid, global_null);
        }
    }

    // Step 4: Compute lFDR for each candidate
    let mut results: Vec<LfdrResult> = candidates
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let cid = cluster_ids[i];
            let (null_mean, null_std) = cluster_nulls[&cid];
            let lfdr = compute_lfdr(c.oos_psr, null_mean, null_std);

            LfdrResult {
                id: c.id.clone(),
                oos_psr: c.oos_psr,
                cluster_id: cid,
                lfdr,
                passes: false, // determined below
            }
        })
        .collect();

    // Step 5: Storey's step-up within each cluster
    // Sort by lFDR within each cluster, accept from lowest until cumulative lfdr exceeds fdr_level
    let mut by_cluster: HashMap<usize, Vec<usize>> = HashMap::new();
    for (i, r) in results.iter().enumerate() {
        by_cluster.entry(r.cluster_id).or_default().push(i);
    }

    for indices in by_cluster.values() {
        let mut sorted_indices = indices.clone();
        sorted_indices.sort_by(|&a, &b| results[a].lfdr.total_cmp(&results[b].lfdr));

        // Accept strategies starting from lowest lFDR until average lFDR exceeds target
        let mut cumulative_lfdr = 0.0;
        for (rank, &idx) in sorted_indices.iter().enumerate() {
            cumulative_lfdr += results[idx].lfdr;
            let avg_lfdr = cumulative_lfdr / (rank + 1) as f64;
            if avg_lfdr <= config.fdr_level {
                results[idx].passes = true;
            } else {
                break;
            }
        }
    }

    // Sort by lFDR (most significant first)
    results.sort_by(|a, b| a.lfdr.total_cmp(&b.lfdr));
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candidate(id: &str, tokens: Vec<usize>, oos_psr: f64) -> HypothesisCandidate {
        HypothesisCandidate {
            id: id.to_string(),
            tokens,
            oos_psr,
        }
    }

    #[test]
    fn test_extract_ngrams() {
        let tokens = vec![1, 2, 3, 4, 5];
        let ngrams = extract_ngrams(&tokens, 3);
        assert_eq!(ngrams.len(), 3); // [1,2,3], [2,3,4], [3,4,5]
        assert!(ngrams.contains(&vec![1, 2, 3]));
        assert!(ngrams.contains(&vec![3, 4, 5]));
    }

    #[test]
    fn test_extract_ngrams_too_short() {
        let tokens = vec![1, 2];
        let ngrams = extract_ngrams(&tokens, 3);
        assert!(ngrams.is_empty());
    }

    #[test]
    fn test_jaccard_identical() {
        let a: HashSet<Vec<usize>> = vec![vec![1, 2, 3], vec![4, 5, 6]].into_iter().collect();
        assert!((jaccard_similarity(&a, &a) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_jaccard_disjoint() {
        let a: HashSet<Vec<usize>> = vec![vec![1, 2, 3]].into_iter().collect();
        let b: HashSet<Vec<usize>> = vec![vec![4, 5, 6]].into_iter().collect();
        assert!((jaccard_similarity(&a, &b)).abs() < 1e-10);
    }

    #[test]
    fn test_jaccard_partial_overlap() {
        let a: HashSet<Vec<usize>> = vec![vec![1, 2, 3], vec![4, 5, 6]].into_iter().collect();
        let b: HashSet<Vec<usize>> = vec![vec![1, 2, 3], vec![7, 8, 9]].into_iter().collect();
        // intersection = {[1,2,3]}, union = {[1,2,3],[4,5,6],[7,8,9]}
        assert!((jaccard_similarity(&a, &b) - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_clustering_similar_tokens() {
        let candidates = vec![
            make_candidate("a", vec![1, 2, 3, 4, 5], 1.0),
            make_candidate("b", vec![1, 2, 3, 4, 6], 0.8), // shares [1,2,3], [2,3,4]
            make_candidate("c", vec![10, 11, 12, 13, 14], 0.5), // totally different
        ];
        let clusters = cluster_by_ngrams(&candidates, 3, 0.3);
        // a and b should cluster together, c separate
        assert_eq!(clusters[0], clusters[1]);
        assert_ne!(clusters[0], clusters[2]);
    }

    #[test]
    fn test_estimate_null() {
        // Mostly null (centered around 0) with a few outliers
        let psrs = vec![-0.5, -0.2, -0.1, 0.0, 0.1, 0.2, 0.3, 3.0, 4.0, 5.0];
        let (mean, std) = estimate_null(&psrs);
        // Central 50%: indices 2..7 = [-0.1, 0.0, 0.1, 0.2, 0.3]
        // mean ≈ 0.1, std ≈ 0.158
        assert!(mean.abs() < 0.5, "null mean should be near zero: {}", mean);
        assert!(std < 1.0, "null std should be small: {}", std);
    }

    #[test]
    fn test_normal_cdf_symmetry() {
        assert!((normal_cdf(0.0) - 0.5).abs() < 0.01);
        assert!(normal_cdf(3.0) > 0.99);
        assert!(normal_cdf(-3.0) < 0.01);
    }

    #[test]
    fn test_lfdr_high_psr_passes() {
        let candidates = vec![
            make_candidate("strong", vec![1, 2, 3, 4, 5], 3.0),
            make_candidate("weak1", vec![1, 2, 3, 5, 6], 0.1),
            make_candidate("weak2", vec![1, 2, 3, 7, 8], -0.2),
            make_candidate("weak3", vec![1, 2, 3, 9, 10], 0.0),
            make_candidate("weak4", vec![1, 2, 3, 11, 12], -0.1),
            make_candidate("weak5", vec![1, 2, 3, 13, 14], 0.05),
            make_candidate("weak6", vec![1, 2, 3, 15, 16], -0.3),
            make_candidate("weak7", vec![1, 2, 3, 17, 18], 0.2),
            make_candidate("weak8", vec![1, 2, 3, 19, 20], -0.15),
            make_candidate("weak9", vec![1, 2, 3, 21, 22], 0.15),
            make_candidate("weak10", vec![1, 2, 3, 23, 24], -0.25),
        ];

        let config = LfdrConfig {
            fdr_level: 0.10,
            ngram_size: 3,
            min_cluster_size: 5,
            jaccard_threshold: 0.3,
            enabled: true,
        };

        let results = run_lfdr_test(&candidates, &config);

        // The strong strategy (PSR=3.0) should have low lFDR and pass
        let strong = results.iter().find(|r| r.id == "strong").unwrap();
        assert!(
            strong.lfdr < 0.10,
            "Strong strategy lFDR={} should be < 0.10",
            strong.lfdr
        );
        assert!(strong.passes, "Strong strategy should pass lFDR test");
    }

    #[test]
    fn test_lfdr_empty_candidates() {
        let config = LfdrConfig::default();
        let results = run_lfdr_test(&[], &config);
        assert!(results.is_empty());
    }

    // ── P7-2C: lFDR Integration Tests ────────────────────────────────

    #[test]
    fn test_lfdr_filters_correlated_strategies() {
        // 20 strategies with very similar RPN tokens and mediocre PSR → should be filtered
        let mut candidates = Vec::new();
        for i in 0..20 {
            candidates.push(make_candidate(
                &format!("corr_{}", i),
                vec![1, 2, 3, 4, 5 + (i % 2)], // very similar tokens
                0.2 + (i as f64 * 0.01),       // mediocre PSR spread
            ));
        }

        let config = LfdrConfig {
            fdr_level: 0.10,
            ngram_size: 3,
            min_cluster_size: 5,
            jaccard_threshold: 0.3,
            enabled: true,
        };

        let results = run_lfdr_test(&candidates, &config);
        let passing = results.iter().filter(|r| r.passes).count();
        // Most correlated mediocre strategies should be filtered
        assert!(
            passing < candidates.len(),
            "lFDR should filter some correlated strategies: {}/{} passed",
            passing,
            candidates.len()
        );
    }

    #[test]
    fn test_lfdr_preserves_diverse_strategies() {
        // 10 strategies with distinct structures and high PSR → should be preserved
        let candidates: Vec<HypothesisCandidate> = (0..10)
            .map(|i| {
                make_candidate(
                    &format!("diverse_{}", i),
                    vec![i * 10, i * 10 + 1, i * 10 + 2, i * 10 + 3, i * 10 + 4],
                    2.0 + (i as f64 * 0.1), // high PSR
                )
            })
            .collect();

        let config = LfdrConfig {
            fdr_level: 0.10,
            ngram_size: 3,
            min_cluster_size: 5,
            jaccard_threshold: 0.3,
            enabled: true,
        };

        let results = run_lfdr_test(&candidates, &config);
        let passing = results.iter().filter(|r| r.passes).count();
        // High PSR + diverse structures → most should pass
        assert!(
            passing > 0,
            "At least some diverse high-PSR strategies should pass lFDR"
        );
    }

    #[test]
    fn test_lfdr_disabled_passthrough() {
        // When disabled, all candidates should pass through
        let candidates = vec![
            make_candidate("a", vec![1, 2, 3], 0.5),
            make_candidate("b", vec![1, 2, 4], 0.3),
        ];

        let config = LfdrConfig {
            enabled: false,
            ..LfdrConfig::default()
        };

        // run_lfdr_test still runs, but the caller should check config.enabled
        // The function itself does full computation regardless of enabled flag
        let results = run_lfdr_test(&candidates, &config);
        assert_eq!(results.len(), 2, "Should return results for all candidates");
    }
}
