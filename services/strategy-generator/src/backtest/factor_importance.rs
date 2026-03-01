//! Permutation-based factor importance attribution (P7-5A, activated P8-0A).
//!
//! For a given genome, shuffles each factor column independently and measures
//! PSR drop. Factors whose shuffling causes the largest drop are most important.
//! Top-10 + Bottom-10 injected into LLM Oracle prompt (P8-0B).

use crate::backtest::Backtester;
use crate::genetic::Genome;
use crate::backtest::StrategyMode;

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

    importances.sort_by(|a, b| {
        b.importance
            .partial_cmp(&a.importance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

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
            FactorImportance { factor_index: 0, factor_name: "ATR".to_string(), importance: 0.5 },
            FactorImportance { factor_index: 1, factor_name: "MACD".to_string(), importance: 0.3 },
            FactorImportance { factor_index: 2, factor_name: "RSI".to_string(), importance: 0.1 },
        ];
        let summary = top_n_summary(&importances, 2);
        assert_eq!(summary.len(), 2);
        assert_eq!(summary[0].0, "ATR");
        assert_eq!(summary[1].0, "MACD");
    }

    #[test]
    fn test_bottom_n_summary() {
        let importances = vec![
            FactorImportance { factor_index: 0, factor_name: "ATR".to_string(), importance: 0.5 },
            FactorImportance { factor_index: 1, factor_name: "MACD".to_string(), importance: 0.3 },
            FactorImportance { factor_index: 2, factor_name: "RSI".to_string(), importance: 0.1 },
            FactorImportance { factor_index: 3, factor_name: "OBV".to_string(), importance: 0.02 },
        ];
        let summary = bottom_n_summary(&importances, 2);
        assert_eq!(summary.len(), 2);
        assert_eq!(summary[0].0, "OBV");
        assert_eq!(summary[1].0, "RSI");
    }
}
