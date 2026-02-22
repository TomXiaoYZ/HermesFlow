//! Hierarchical Risk Parity (HRP) allocation algorithm.
//!
//! Implements López de Prado (2016): correlation → distance → single-linkage
//! clustering → quasi-diagonalization (seriation) → recursive bisection.
//!
//! Pure math module — no I/O, no DB. Only depends on ndarray.

use ndarray::Array2;

/// Result of an HRP allocation.
#[derive(Debug, Clone)]
pub struct HrpResult {
    /// Portfolio weights, one per strategy. Sum = 1.0.
    pub weights: Vec<f64>,
    /// N x N Pearson correlation matrix.
    pub correlation_matrix: Array2<f64>,
    /// N x N distance matrix: sqrt(0.5 * (1 - corr)).
    #[allow(dead_code)]
    pub distance_matrix: Array2<f64>,
    /// Leaf ordering from quasi-diagonalization (seriation).
    pub leaf_order: Vec<usize>,
    /// Dendrogram linkage: Vec<(left, right, distance, merged_size)>.
    pub linkage: Vec<(usize, usize, f64, usize)>,
}

// ── Correlation ────────────────────────────────────────────────────────

/// Compute Pearson correlation matrix from a T x N return matrix.
///
/// Each column is a strategy's return series. Handles constant columns
/// (std = 0) by setting their correlation to 0 with others, 1 with self.
pub fn correlation_matrix(returns: &Array2<f64>) -> Array2<f64> {
    let n = returns.ncols();
    let t = returns.nrows();
    let mut corr = Array2::<f64>::eye(n);

    if t < 2 || n == 0 {
        return corr;
    }

    // Pre-compute means and stds
    let means: Vec<f64> = (0..n).map(|j| returns.column(j).sum() / t as f64).collect();
    let stds: Vec<f64> = (0..n)
        .map(|j| {
            let var = returns
                .column(j)
                .iter()
                .map(|&x| (x - means[j]).powi(2))
                .sum::<f64>()
                / (t as f64 - 1.0);
            var.sqrt()
        })
        .collect();

    for i in 0..n {
        if stds[i] < 1e-12 {
            // Constant column: corr = 0 with others (diagonal already 1)
            continue;
        }
        for j in (i + 1)..n {
            if stds[j] < 1e-12 {
                continue;
            }
            let mut cov = 0.0_f64;
            for t_idx in 0..t {
                cov += (returns[[t_idx, i]] - means[i]) * (returns[[t_idx, j]] - means[j]);
            }
            cov /= t as f64 - 1.0;
            let r = (cov / (stds[i] * stds[j])).clamp(-1.0, 1.0);
            corr[[i, j]] = r;
            corr[[j, i]] = r;
        }
    }

    corr
}

/// Compute covariance matrix from a T x N return matrix.
fn covariance_matrix(returns: &Array2<f64>) -> Array2<f64> {
    let n = returns.ncols();
    let t = returns.nrows();
    let mut cov = Array2::<f64>::zeros((n, n));

    if t < 2 || n == 0 {
        return cov;
    }

    let means: Vec<f64> = (0..n).map(|j| returns.column(j).sum() / t as f64).collect();

    for i in 0..n {
        for j in i..n {
            let mut s = 0.0_f64;
            for t_idx in 0..t {
                s += (returns[[t_idx, i]] - means[i]) * (returns[[t_idx, j]] - means[j]);
            }
            let c = s / (t as f64 - 1.0);
            cov[[i, j]] = c;
            cov[[j, i]] = c;
        }
    }

    cov
}

// ── Distance ───────────────────────────────────────────────────────────

/// Convert correlation matrix to distance: D_ij = sqrt(0.5 * (1 - corr_ij)).
///
/// Distances are floored at 1e-10 to prevent zero-distance merges in clustering.
pub fn distance_matrix(corr: &Array2<f64>) -> Array2<f64> {
    let n = corr.nrows();
    let mut dist = Array2::<f64>::zeros((n, n));

    for i in 0..n {
        for j in 0..n {
            if i == j {
                dist[[i, j]] = 0.0;
            } else {
                let d = (0.5 * (1.0 - corr[[i, j]])).max(0.0).sqrt();
                dist[[i, j]] = d.max(1e-10);
            }
        }
    }

    dist
}

// ── Clustering ─────────────────────────────────────────────────────────

/// Single-linkage agglomerative clustering.
///
/// Returns a linkage list: Vec<(left_cluster, right_cluster, merge_distance, merged_size)>.
/// Cluster indices 0..N are original items; indices N.. are merged clusters.
pub fn single_linkage_clustering(dist: &Array2<f64>) -> Vec<(usize, usize, f64, usize)> {
    let n = dist.nrows();
    if n <= 1 {
        return vec![];
    }

    // Working distance matrix (mutable copy of upper triangle)
    let mut d = dist.clone();
    // Track which items belong to each active cluster
    let mut cluster_size = vec![1_usize; n];
    // Active cluster indices
    let mut active: Vec<usize> = (0..n).collect();
    let mut linkage = Vec::with_capacity(n - 1);
    let mut _next_cluster_id = n;

    for _ in 0..(n - 1) {
        // Find minimum distance pair among active clusters
        let mut min_dist = f64::INFINITY;
        let mut min_i = 0;
        let mut min_j = 0;

        for (ai, &ci) in active.iter().enumerate() {
            for (aj, &cj) in active.iter().enumerate() {
                if ai < aj && d[[ci, cj]] < min_dist {
                    min_dist = d[[ci, cj]];
                    min_i = ai;
                    min_j = aj;
                }
            }
        }

        let ci = active[min_i];
        let cj = active[min_j];
        let merged_size = cluster_size[ci] + cluster_size[cj];

        linkage.push((ci, cj, min_dist, merged_size));

        // Update distances: new cluster takes minimum distance (single-linkage)
        // Extend matrices if needed by reusing ci slot
        cluster_size[ci] = merged_size;

        // Update distances from merged cluster (ci) to all remaining
        for &ck in &active {
            if ck != ci && ck != cj {
                let new_dist = d[[ci, ck]].min(d[[cj, ck]]);
                d[[ci, ck]] = new_dist;
                d[[ck, ci]] = new_dist;
            }
        }

        // Remove cj from active set
        active.remove(min_j);

        // Remap ci to new cluster id for linkage tracking
        // (we use the original indices in the linkage output)
        _next_cluster_id += 1;
    }

    // Re-index linkage to use proper cluster IDs (standard scipy convention)
    let mut id_map: std::collections::HashMap<usize, usize> = (0..n).map(|i| (i, i)).collect();
    let mut reindexed = Vec::with_capacity(linkage.len());
    let mut next_id = n;

    for (left, right, dist_val, size) in linkage {
        let l = *id_map.get(&left).unwrap_or(&left);
        let r = *id_map.get(&right).unwrap_or(&right);
        reindexed.push((l, r, dist_val, size));
        id_map.insert(left, next_id);
        next_id += 1;
    }

    reindexed
}

// ── Quasi-Diagonalization (Seriation) ──────────────────────────────────

/// Extract leaf ordering from linkage dendrogram via recursive traversal.
///
/// This orders the original items so that correlated strategies are adjacent,
/// enabling the recursive bisection to properly split the portfolio.
pub fn quasi_diagonalize(linkage: &[(usize, usize, f64, usize)], n: usize) -> Vec<usize> {
    if n == 0 {
        return vec![];
    }
    if n == 1 {
        return vec![0];
    }
    if linkage.is_empty() {
        return (0..n).collect();
    }

    // Build a tree from linkage
    // Nodes 0..n are leaves; nodes n.. are internal (merged clusters)
    let mut left_child = vec![0_usize; linkage.len()];
    let mut right_child = vec![0_usize; linkage.len()];

    for (i, &(l, r, _, _)) in linkage.iter().enumerate() {
        left_child[i] = l;
        right_child[i] = r;
    }

    // Root is the last merge = node (n + linkage.len() - 1)
    let root = n + linkage.len() - 1;

    // Iterative DFS to collect leaf order
    let mut order = Vec::with_capacity(n);
    let mut stack = vec![root];

    while let Some(node) = stack.pop() {
        if node < n {
            // Leaf node
            order.push(node);
        } else {
            // Internal node
            let idx = node - n;
            if idx < linkage.len() {
                // Push right first so left is processed first (DFS)
                stack.push(right_child[idx]);
                stack.push(left_child[idx]);
            }
        }
    }

    order
}

// ── Recursive Bisection ────────────────────────────────────────────────

/// Compute inverse-variance weight for a cluster of items.
///
/// The cluster variance is the variance of the equal-weighted portfolio
/// of the items in the cluster. Weight is proportional to 1/variance.
fn cluster_variance(cov: &Array2<f64>, items: &[usize]) -> f64 {
    if items.is_empty() {
        return 1e-10;
    }
    if items.len() == 1 {
        return cov[[items[0], items[0]]].max(1e-10);
    }

    // Equal-weighted portfolio variance within the cluster
    let k = items.len() as f64;
    let mut var = 0.0_f64;
    for &i in items {
        for &j in items {
            var += cov[[i, j]];
        }
    }
    var /= k * k;
    var.max(1e-10)
}

/// Recursive bisection: allocate weights by splitting the quasi-diagonalized
/// ordering at the midpoint and distributing by inverse cluster variance.
///
/// Returns weights summing to 1.0.
pub fn recursive_bisection(cov: &Array2<f64>, leaf_order: &[usize]) -> Vec<f64> {
    let n = cov.nrows();
    if n == 0 {
        return vec![];
    }

    let mut weights = vec![1.0_f64; n];

    // BFS-style bisection
    let mut clusters: Vec<Vec<usize>> = vec![leaf_order.to_vec()];

    while let Some(cluster) = clusters.pop() {
        if cluster.len() <= 1 {
            continue;
        }

        let mid = cluster.len() / 2;
        let left = &cluster[..mid];
        let right = &cluster[mid..];

        let var_left = cluster_variance(cov, left);
        let var_right = cluster_variance(cov, right);

        // Allocate proportional to inverse variance
        let alpha = 1.0 - var_left / (var_left + var_right);

        for &i in left {
            weights[i] *= alpha;
        }
        for &i in right {
            weights[i] *= 1.0 - alpha;
        }

        if left.len() > 1 {
            clusters.push(left.to_vec());
        }
        if right.len() > 1 {
            clusters.push(right.to_vec());
        }
    }

    // Normalize to sum = 1.0
    let total: f64 = weights.iter().sum();
    if total > 1e-12 {
        for w in &mut weights {
            *w /= total;
        }
    }

    weights
}

// ── Public Entry Point ─────────────────────────────────────────────────

/// Full HRP pipeline: correlation → distance → clustering → seriation → bisection.
///
/// Input: T x N matrix where each column is a strategy's return series.
/// Output: HrpResult with weights, correlation matrix, diagnostics.
///
/// Returns None if input is empty or has fewer than 2 time steps.
pub fn allocate_hrp(returns: &Array2<f64>) -> Option<HrpResult> {
    let n = returns.ncols();
    let t = returns.nrows();

    if n == 0 || t < 2 {
        return None;
    }

    // Single strategy → 100% weight
    if n == 1 {
        let corr = Array2::<f64>::eye(1);
        let dist = Array2::<f64>::zeros((1, 1));
        return Some(HrpResult {
            weights: vec![1.0],
            correlation_matrix: corr,
            distance_matrix: dist,
            leaf_order: vec![0],
            linkage: vec![],
        });
    }

    let corr = correlation_matrix(returns);
    let dist = distance_matrix(&corr);
    let linkage = single_linkage_clustering(&dist);
    let leaf_order = quasi_diagonalize(&linkage, n);
    let cov = covariance_matrix(returns);
    let weights = recursive_bisection(&cov, &leaf_order);

    Some(HrpResult {
        weights,
        correlation_matrix: corr,
        distance_matrix: dist,
        leaf_order,
        linkage,
    })
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::Array2;

    fn approx_eq(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() < tol
    }

    // ── Correlation tests ──────────────────────────────────────────

    #[test]
    fn correlation_identity_matrix() {
        // Perfectly uncorrelated: identity correlation
        let returns = Array2::from_shape_vec((100, 2), {
            let mut v = Vec::with_capacity(200);
            for i in 0..100 {
                v.push((i as f64 * 0.1).sin());
                v.push((i as f64 * 0.3).cos());
            }
            v
        })
        .unwrap();
        let corr = correlation_matrix(&returns);
        assert!(approx_eq(corr[[0, 0]], 1.0, 1e-10));
        assert!(approx_eq(corr[[1, 1]], 1.0, 1e-10));
    }

    #[test]
    fn correlation_perfect_positive() {
        // Same series → corr = 1
        let mut returns = Array2::<f64>::zeros((100, 2));
        for i in 0..100 {
            let v = (i as f64 * 0.1).sin();
            returns[[i, 0]] = v;
            returns[[i, 1]] = v;
        }
        let corr = correlation_matrix(&returns);
        assert!(approx_eq(corr[[0, 1]], 1.0, 1e-10));
    }

    #[test]
    fn correlation_perfect_negative() {
        let mut returns = Array2::<f64>::zeros((100, 2));
        for i in 0..100 {
            let v = (i as f64 * 0.1).sin();
            returns[[i, 0]] = v;
            returns[[i, 1]] = -v;
        }
        let corr = correlation_matrix(&returns);
        assert!(approx_eq(corr[[0, 1]], -1.0, 1e-10));
    }

    #[test]
    fn correlation_constant_column() {
        let mut returns = Array2::<f64>::zeros((100, 2));
        for i in 0..100 {
            returns[[i, 0]] = 5.0; // constant
            returns[[i, 1]] = (i as f64 * 0.1).sin();
        }
        let corr = correlation_matrix(&returns);
        assert!(approx_eq(corr[[0, 0]], 1.0, 1e-10));
        assert!(approx_eq(corr[[0, 1]], 0.0, 1e-10));
    }

    #[test]
    fn correlation_symmetric() {
        let mut returns = Array2::<f64>::zeros((50, 3));
        for i in 0..50 {
            returns[[i, 0]] = (i as f64 * 0.1).sin();
            returns[[i, 1]] = (i as f64 * 0.2).cos();
            returns[[i, 2]] = (i as f64 * 0.3).sin() + 0.5;
        }
        let corr = correlation_matrix(&returns);
        for i in 0..3 {
            for j in 0..3 {
                assert!(approx_eq(corr[[i, j]], corr[[j, i]], 1e-12));
            }
        }
    }

    #[test]
    fn correlation_diagonal_is_one() {
        let mut returns = Array2::<f64>::zeros((50, 4));
        for i in 0..50 {
            for j in 0..4 {
                returns[[i, j]] = (i as f64 * (j as f64 + 1.0) * 0.1).sin();
            }
        }
        let corr = correlation_matrix(&returns);
        for i in 0..4 {
            assert!(approx_eq(corr[[i, i]], 1.0, 1e-10));
        }
    }

    // ── Distance tests ─────────────────────────────────────────────

    #[test]
    fn distance_corr_one_is_zero() {
        let mut corr = Array2::<f64>::eye(2);
        corr[[0, 1]] = 1.0;
        corr[[1, 0]] = 1.0;
        let dist = distance_matrix(&corr);
        // sqrt(0.5*(1-1)) = 0, but floored at 1e-10
        assert!(dist[[0, 1]] <= 1e-9);
    }

    #[test]
    fn distance_corr_neg_one_is_one() {
        let mut corr = Array2::<f64>::eye(2);
        corr[[0, 1]] = -1.0;
        corr[[1, 0]] = -1.0;
        let dist = distance_matrix(&corr);
        assert!(approx_eq(dist[[0, 1]], 1.0, 1e-10));
    }

    #[test]
    fn distance_corr_zero_is_sqrt_half() {
        let mut corr = Array2::<f64>::eye(2);
        corr[[0, 1]] = 0.0;
        corr[[1, 0]] = 0.0;
        let dist = distance_matrix(&corr);
        assert!(approx_eq(dist[[0, 1]], (0.5_f64).sqrt(), 1e-10));
    }

    #[test]
    fn distance_non_negative() {
        let mut corr = Array2::<f64>::eye(3);
        corr[[0, 1]] = 0.5;
        corr[[1, 0]] = 0.5;
        corr[[0, 2]] = -0.3;
        corr[[2, 0]] = -0.3;
        corr[[1, 2]] = 0.8;
        corr[[2, 1]] = 0.8;
        let dist = distance_matrix(&corr);
        for i in 0..3 {
            for j in 0..3 {
                assert!(dist[[i, j]] >= 0.0);
            }
        }
    }

    // ── Clustering tests ───────────────────────────────────────────

    #[test]
    fn clustering_two_assets() {
        let mut dist = Array2::<f64>::zeros((2, 2));
        dist[[0, 1]] = 0.5;
        dist[[1, 0]] = 0.5;
        let linkage = single_linkage_clustering(&dist);
        assert_eq!(linkage.len(), 1);
        assert_eq!(linkage[0].3, 2); // merged size = 2
    }

    #[test]
    fn clustering_three_assets() {
        // Assets 0,1 are close; 2 is far
        let mut dist = Array2::<f64>::zeros((3, 3));
        dist[[0, 1]] = 0.1;
        dist[[1, 0]] = 0.1;
        dist[[0, 2]] = 0.9;
        dist[[2, 0]] = 0.9;
        dist[[1, 2]] = 0.8;
        dist[[2, 1]] = 0.8;
        let linkage = single_linkage_clustering(&dist);
        assert_eq!(linkage.len(), 2);
        // First merge should be 0,1 (distance 0.1)
        assert!(approx_eq(linkage[0].2, 0.1, 1e-10));
    }

    #[test]
    fn clustering_identical_assets() {
        let mut dist = Array2::<f64>::zeros((3, 3));
        for i in 0..3 {
            for j in 0..3 {
                if i != j {
                    dist[[i, j]] = 1e-10;
                }
            }
        }
        let linkage = single_linkage_clustering(&dist);
        assert_eq!(linkage.len(), 2);
    }

    // ── Quasi-diagonalize tests ────────────────────────────────────

    #[test]
    fn quasi_diag_preserves_all_indices() {
        let linkage = vec![
            (0, 1, 0.1, 2), // merge 0,1 → node 3
            (3, 2, 0.5, 3), // merge {0,1},2 → node 4
        ];
        let order = quasi_diagonalize(&linkage, 3);
        assert_eq!(order.len(), 3);
        let mut sorted = order.clone();
        sorted.sort();
        assert_eq!(sorted, vec![0, 1, 2]);
    }

    #[test]
    fn quasi_diag_two_assets() {
        let linkage = vec![(0, 1, 0.3, 2)];
        let order = quasi_diagonalize(&linkage, 2);
        assert_eq!(order.len(), 2);
        assert!(order.contains(&0));
        assert!(order.contains(&1));
    }

    #[test]
    fn quasi_diag_single_asset() {
        let order = quasi_diagonalize(&[], 1);
        assert_eq!(order, vec![0]);
    }

    // ── Bisection tests ────────────────────────────────────────────

    #[test]
    fn bisection_equal_variance_equal_weights() {
        // Two assets with identical variance and zero covariance
        let mut cov = Array2::<f64>::zeros((2, 2));
        cov[[0, 0]] = 1.0;
        cov[[1, 1]] = 1.0;
        let weights = recursive_bisection(&cov, &[0, 1]);
        assert!(approx_eq(weights[0], 0.5, 0.05));
        assert!(approx_eq(weights[1], 0.5, 0.05));
    }

    #[test]
    fn bisection_high_var_gets_lower_weight() {
        let mut cov = Array2::<f64>::zeros((2, 2));
        cov[[0, 0]] = 1.0; // low variance
        cov[[1, 1]] = 10.0; // high variance
        let weights = recursive_bisection(&cov, &[0, 1]);
        assert!(weights[0] > weights[1]); // lower variance → higher weight
    }

    #[test]
    fn bisection_weights_sum_to_one() {
        let mut cov = Array2::<f64>::zeros((4, 4));
        for i in 0..4 {
            cov[[i, i]] = (i as f64 + 1.0) * 0.5;
        }
        let weights = recursive_bisection(&cov, &[0, 1, 2, 3]);
        let sum: f64 = weights.iter().sum();
        assert!(approx_eq(sum, 1.0, 1e-10));
    }

    // ── Integration tests ──────────────────────────────────────────

    #[test]
    fn allocate_hrp_five_strategies() {
        // 5 synthetic strategy returns
        let t = 200;
        let n = 5;
        let mut returns = Array2::<f64>::zeros((t, n));
        for i in 0..t {
            for j in 0..n {
                returns[[i, j]] = ((i as f64 + j as f64) * 0.1 * (j as f64 + 1.0)).sin()
                    * 0.01
                    * (j as f64 + 1.0);
            }
        }

        let result = allocate_hrp(&returns).unwrap();
        assert_eq!(result.weights.len(), n);

        let sum: f64 = result.weights.iter().sum();
        assert!(approx_eq(sum, 1.0, 1e-10));

        // All weights positive
        for &w in &result.weights {
            assert!(w > 0.0);
        }

        // Correlation matrix is N x N
        assert_eq!(result.correlation_matrix.nrows(), n);
        assert_eq!(result.correlation_matrix.ncols(), n);

        // Leaf order contains all indices
        let mut sorted_order = result.leaf_order.clone();
        sorted_order.sort();
        assert_eq!(sorted_order, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn allocate_hrp_single_strategy() {
        let returns = Array2::from_shape_vec((100, 1), vec![0.01; 100]).unwrap();
        let result = allocate_hrp(&returns).unwrap();
        assert_eq!(result.weights, vec![1.0]);
    }

    #[test]
    fn allocate_hrp_empty_returns_none() {
        let returns = Array2::<f64>::zeros((0, 0));
        assert!(allocate_hrp(&returns).is_none());
    }

    #[test]
    fn allocate_hrp_insufficient_time_returns_none() {
        let returns = Array2::<f64>::zeros((1, 3));
        assert!(allocate_hrp(&returns).is_none());
    }

    #[test]
    fn allocate_hrp_two_identical_strategies() {
        let t = 100;
        let mut returns = Array2::<f64>::zeros((t, 2));
        for i in 0..t {
            let v = (i as f64 * 0.05).sin() * 0.01;
            returns[[i, 0]] = v;
            returns[[i, 1]] = v;
        }
        let result = allocate_hrp(&returns).unwrap();
        // Both should get roughly equal weight
        assert!(approx_eq(result.weights[0], 0.5, 0.1));
        assert!(approx_eq(result.weights[1], 0.5, 0.1));
    }
}
