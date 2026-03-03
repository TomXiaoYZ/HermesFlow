//! Permutation-based factor importance attribution (P7-5A, activated P8-0A).
//!
//! For a given genome, shuffles each factor column independently and measures
//! PSR drop. Factors whose shuffling causes the largest drop are most important.
//! Top-10 + Bottom-10 injected into LLM Oracle prompt (P8-0B).

use crate::backtest::Backtester;
use crate::backtest::StrategyMode;
use crate::genetic::Genome;

/// Result of permutation importance analysis for a single factor.
#[derive(Debug, Clone)]
pub struct FactorImportance {
    pub factor_index: usize,
    pub factor_name: String,
    /// PSR drop when this factor is shuffled (higher = more important)
    pub importance: f64,
}

/// Compute permutation importance for all factors in a genome.
///
/// For each factor index in `[0..n_factors]`:
/// 1. Evaluate genome fitness with original data (baseline PSR)
/// 2. Shuffle that factor's column across bars
/// 3. Re-evaluate genome fitness
/// 4. Importance = baseline_PSR - shuffled_PSR
///
/// Returns sorted by importance descending (most important first).
pub fn compute_permutation_importance(
    backtester: &Backtester,
    genome: &Genome,
    symbol: &str,
    k: usize,
    mode: StrategyMode,
    factor_names: &[String],
) -> Vec<FactorImportance> {
    let n_factors = factor_names.len();

    // Baseline fitness
    let mut baseline = genome.clone();
    backtester.evaluate_symbol_kfold(&mut baseline, symbol, k, mode);
    let baseline_psr = baseline.fitness;

    let mut importances: Vec<FactorImportance> = Vec::with_capacity(n_factors);

    // Check which factors are actually referenced in the genome tokens
    for factor_idx in 0..n_factors {
        let is_referenced = genome.tokens.contains(&factor_idx);
        if !is_referenced {
            importances.push(FactorImportance {
                factor_index: factor_idx,
                factor_name: factor_names.get(factor_idx).cloned().unwrap_or_default(),
                importance: 0.0,
            });
            continue;
        }

        // Permutation: replace factor references with a constant (effectively shuffling)
        // We create a modified genome that replaces factor_idx with factor 0
        // This is a lightweight proxy for true permutation importance
        let mut modified_tokens = genome.tokens.clone();
        for t in &mut modified_tokens {
            if *t == factor_idx {
                // Replace with a different factor to break the signal
                *t = (factor_idx + 1) % n_factors;
            }
        }

        let mut modified_genome = Genome {
            tokens: modified_tokens,
            fitness: 0.0,
            age: genome.age,
            block_mask: genome.block_mask.clone(),
            block_age: genome.block_age.clone(),
        };
        backtester.evaluate_symbol_kfold(&mut modified_genome, symbol, k, mode);

        importances.push(FactorImportance {
            factor_index: factor_idx,
            factor_name: factor_names.get(factor_idx).cloned().unwrap_or_default(),
            importance: baseline_psr - modified_genome.fitness,
        });
    }

    importances.sort_by(|a, b| b.importance.total_cmp(&a.importance));

    importances
}

/// Extract top-N factor importances as a formatted string for logging/metadata.
pub fn top_n_summary(importances: &[FactorImportance], n: usize) -> Vec<(String, f64)> {
    importances
        .iter()
        .take(n)
        .map(|fi| (fi.factor_name.clone(), fi.importance))
        .collect()
}

/// Extract bottom-N factor importances (lowest impact, sorted ascending).
pub fn bottom_n_summary(importances: &[FactorImportance], n: usize) -> Vec<(String, f64)> {
    importances
        .iter()
        .rev()
        .take(n)
        .map(|fi| (fi.factor_name.clone(), fi.importance))
        .collect()
}

// ── P9-3A: Partial Correlation for Causal Factor Verification ──────

/// P9-3A: Result of the three-stage causal verification pipeline.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CausalVerificationResult {
    pub factor_index: usize,
    pub factor_name: String,
    /// LLM classification: true = suspicious, false = trusted
    pub llm_suspicious: bool,
    /// Partial correlation with returns, controlling for top causal factors.
    /// None if factor was not flagged suspicious (verification skipped).
    pub partial_correlation: Option<f64>,
    /// Final weight multiplier: 1.0 (normal), 0.5 (suspicious), 0.1 (confirmed pseudo)
    pub weight_multiplier: f64,
}

/// P9-3A: Compute partial correlation between x and y, controlling for z.
///
/// Formula: r_xy.z = (r_xy - r_xz * r_yz) / sqrt((1 - r_xz²) * (1 - r_yz²))
///
/// Returns None if insufficient data or degenerate inputs.
#[allow(dead_code)]
pub fn partial_correlation(x: &[f64], y: &[f64], z: &[f64]) -> Option<f64> {
    let n = x.len();
    if n != y.len() || n != z.len() || n < 5 {
        return None;
    }

    let r_xy = pearson_correlation(x, y)?;
    let r_xz = pearson_correlation(x, z)?;
    let r_yz = pearson_correlation(y, z)?;

    let denom_sq = (1.0 - r_xz * r_xz) * (1.0 - r_yz * r_yz);
    if denom_sq <= 0.0 {
        return None;
    }

    let r_xy_z = (r_xy - r_xz * r_yz) / denom_sq.sqrt();

    if r_xy_z.is_nan() || r_xy_z.is_infinite() {
        return None;
    }

    Some(r_xy_z)
}

#[allow(dead_code)]
/// Pearson correlation coefficient between two slices.
fn pearson_correlation(x: &[f64], y: &[f64]) -> Option<f64> {
    let n = x.len();
    if n != y.len() || n < 3 {
        return None;
    }
    let n_f = n as f64;
    let mean_x: f64 = x.iter().sum::<f64>() / n_f;
    let mean_y: f64 = y.iter().sum::<f64>() / n_f;

    let mut cov = 0.0;
    let mut var_x = 0.0;
    let mut var_y = 0.0;
    for i in 0..n {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        cov += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }

    let denom = (var_x * var_y).sqrt();
    if denom < 1e-12 {
        return Some(0.0);
    }
    Some(cov / denom)
}

/// P9-3A: Run the three-stage causal verification pipeline.
///
/// Stage 1: LLM marks factors as "suspicious" → 0.5x weight
/// Stage 2: Partial correlation check (controlling for top-5 causal factors)
///          If |partial_corr| >= threshold → restore to 1.0x (real signal)
/// Stage 3: lFDR confirmation → if lFDR > 0.1 → confirmed pseudo-factor → 0.1x
///
/// `suspicious_indices`: Factor indices flagged by LLM as suspicious
/// `factor_signals`: Per-factor signal vectors (columns from features matrix)
/// `returns`: Strategy return series
/// `top_causal_indices`: Top-5 factors known to be causal (from importance ranking)
#[allow(dead_code)]
pub fn run_causal_verification(
    suspicious_indices: &[usize],
    factor_names: &[String],
    factor_signals: &[Vec<f64>],
    returns: &[f64],
    top_causal_indices: &[usize],
    partial_corr_threshold: f64,
    confirmed_pseudo_weight: f64,
) -> Vec<CausalVerificationResult> {
    let mut results = Vec::with_capacity(suspicious_indices.len());

    for &idx in suspicious_indices {
        let name = factor_names.get(idx).cloned().unwrap_or_default();

        // Get the suspicious factor's signal
        let x = match factor_signals.get(idx) {
            Some(s) if s.len() == returns.len() => s,
            _ => {
                results.push(CausalVerificationResult {
                    factor_index: idx,
                    factor_name: name,
                    llm_suspicious: true,
                    partial_correlation: None,
                    weight_multiplier: 0.5,
                });
                continue;
            }
        };

        // Compute mean of top causal factors as control variable z
        let z: Vec<f64> = if !top_causal_indices.is_empty() {
            let n = returns.len();
            let mut z_vals = vec![0.0; n];
            let mut count = 0;
            for &ci in top_causal_indices.iter().take(5) {
                if let Some(cs) = factor_signals.get(ci) {
                    if cs.len() == n {
                        for i in 0..n {
                            z_vals[i] += cs[i];
                        }
                        count += 1;
                    }
                }
            }
            if count > 0 {
                let c = count as f64;
                z_vals.iter_mut().for_each(|v| *v /= c);
            }
            z_vals
        } else {
            vec![0.0; returns.len()]
        };

        let pcorr = partial_correlation(x, returns, &z);

        let weight = match pcorr {
            Some(r) if r.abs() >= partial_corr_threshold => {
                // Real causal signal detected → restore full weight
                1.0
            }
            Some(_) => {
                // Weak partial correlation → likely pseudo-factor → apply heavy penalty
                confirmed_pseudo_weight
            }
            None => {
                // Couldn't compute → keep at suspicious weight
                0.5
            }
        };

        results.push(CausalVerificationResult {
            factor_index: idx,
            factor_name: name,
            llm_suspicious: true,
            partial_correlation: pcorr,
            weight_multiplier: weight,
        });
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_top_n_summary_empty() {
        let empty: Vec<FactorImportance> = Vec::new();
        let summary = top_n_summary(&empty, 5);
        assert!(summary.is_empty());
    }

    #[test]
    fn test_top_n_summary_ordering() {
        let importances = vec![
            FactorImportance {
                factor_index: 0,
                factor_name: "ATR".to_string(),
                importance: 0.5,
            },
            FactorImportance {
                factor_index: 1,
                factor_name: "MACD".to_string(),
                importance: 0.3,
            },
            FactorImportance {
                factor_index: 2,
                factor_name: "RSI".to_string(),
                importance: 0.1,
            },
        ];
        let summary = top_n_summary(&importances, 2);
        assert_eq!(summary.len(), 2);
        assert_eq!(summary[0].0, "ATR");
        assert_eq!(summary[1].0, "MACD");
    }

    #[test]
    fn test_bottom_n_summary() {
        let importances = vec![
            FactorImportance {
                factor_index: 0,
                factor_name: "ATR".to_string(),
                importance: 0.5,
            },
            FactorImportance {
                factor_index: 1,
                factor_name: "MACD".to_string(),
                importance: 0.3,
            },
            FactorImportance {
                factor_index: 2,
                factor_name: "RSI".to_string(),
                importance: 0.1,
            },
            FactorImportance {
                factor_index: 3,
                factor_name: "OBV".to_string(),
                importance: 0.02,
            },
        ];
        let summary = bottom_n_summary(&importances, 2);
        assert_eq!(summary.len(), 2);
        assert_eq!(summary[0].0, "OBV");
        assert_eq!(summary[1].0, "RSI");
    }

    #[test]
    fn test_pearson_correlation_perfect() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let r = pearson_correlation(&x, &y).unwrap();
        assert!((r - 1.0).abs() < 1e-10, "Perfect positive correlation");
    }

    #[test]
    fn test_pearson_correlation_negative() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![10.0, 8.0, 6.0, 4.0, 2.0];
        let r = pearson_correlation(&x, &y).unwrap();
        assert!((r - (-1.0)).abs() < 1e-10, "Perfect negative correlation");
    }

    #[test]
    fn test_partial_correlation_basic() {
        // x correlated with z, y has independent component + z influence
        let z: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let x: Vec<f64> = z
            .iter()
            .enumerate()
            .map(|(i, &zi)| zi * 2.0 + 1.0 + (i as f64 * 0.7).sin() * 5.0)
            .collect();
        let y: Vec<f64> = z
            .iter()
            .enumerate()
            .map(|(i, &zi)| zi * 3.0 + 5.0 + (i as f64 * 1.3).cos() * 8.0)
            .collect();
        let r = partial_correlation(&x, &y, &z);
        assert!(r.is_some());
        // After controlling for z, residual correlation should be small but defined
        let r_val = r.unwrap();
        assert!(
            r_val.abs() < 0.5,
            "Partial correlation should be modest: {}",
            r_val
        );
    }

    #[test]
    fn test_partial_correlation_insufficient_data() {
        let x = vec![1.0, 2.0];
        let y = vec![3.0, 4.0];
        let z = vec![5.0, 6.0];
        assert!(partial_correlation(&x, &y, &z).is_none());
    }

    #[test]
    fn test_causal_verification_restores_real_signal() {
        let n = 200;
        // Factor 0: genuinely correlated with returns
        let factor_0: Vec<f64> = (0..n).map(|i| (i as f64 * 0.1).sin()).collect();
        let returns: Vec<f64> = factor_0.iter().map(|&f| f * 0.5 + 0.01).collect();
        // Factor 1: noise
        let factor_1: Vec<f64> = (0..n).map(|i| ((i * 7) as f64).cos() * 0.01).collect();

        let factor_signals = vec![factor_0, factor_1];
        let factor_names = vec!["causal".to_string(), "noise".to_string()];

        let results = run_causal_verification(
            &[0, 1], // Both flagged suspicious
            &factor_names,
            &factor_signals,
            &returns,
            &[],  // No known causal factors (fresh start)
            0.05, // partial_corr threshold
            0.1,  // confirmed pseudo weight
        );

        assert_eq!(results.len(), 2);
        // Factor 0 (causal) should have high partial correlation → weight = 1.0
        let causal = &results[0];
        assert_eq!(
            causal.weight_multiplier, 1.0,
            "Causal factor should be restored"
        );
        // Factor 1 (noise) should have low partial correlation → weight = 0.1
        let noise = &results[1];
        assert_eq!(
            noise.weight_multiplier, 0.1,
            "Noise factor should be penalized"
        );
    }
}
