use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorDefinition {
    pub id: usize,
    pub name: String,
    pub description: String,
    pub normalization: NormalizationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NormalizationType {
    None,
    Robust,
    ZScore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorConfig {
    pub active_factors: Vec<FactorDefinition>,
}

impl FactorConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn feat_count(&self) -> usize {
        self.active_factors.len()
    }

    pub fn feat_offset(&self) -> usize {
        self.feat_count()
    }
}

/// Multi-timeframe factor configuration for P3.
///
/// Wraps a single `FactorConfig` (e.g. 25 factors) and replicates it across
/// multiple resolutions (e.g. 1h, 4h, 1d) to produce a combined feature space.
/// Token layout: [0..n-1] = res[0], [n..2n-1] = res[1], [2n..3n-1] = res[2].
#[derive(Debug, Clone)]
pub struct MultiTimeframeFactorConfig {
    pub base_config: FactorConfig,
    pub resolutions: Vec<String>,
}

impl MultiTimeframeFactorConfig {
    pub fn new(base_config: FactorConfig, resolutions: Vec<String>) -> Self {
        Self {
            base_config,
            resolutions,
        }
    }

    /// Total features across all resolutions.
    pub fn feat_count(&self) -> usize {
        self.base_config.feat_count() * self.resolutions.len()
    }

    /// Token boundary between features and operators.
    pub fn feat_offset(&self) -> usize {
        self.feat_count()
    }

    /// Number of factors per single resolution.
    pub fn base_feat_count(&self) -> usize {
        self.base_config.feat_count()
    }

    /// Factor names with timeframe suffixes.
    /// e.g. ["return_1h", "momentum_1h", ..., "return_4h", ..., "return_1d", ...]
    pub fn factor_names(&self) -> Vec<String> {
        let mut names = Vec::with_capacity(self.feat_count());
        for res in &self.resolutions {
            for factor in &self.base_config.active_factors {
                names.push(format!("{}_{}", factor.name, res));
            }
        }
        names
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> FactorConfig {
        FactorConfig {
            active_factors: vec![
                FactorDefinition {
                    id: 0,
                    name: "return".to_string(),
                    description: "Log return".to_string(),
                    normalization: NormalizationType::Robust,
                },
                FactorDefinition {
                    id: 1,
                    name: "momentum".to_string(),
                    description: "20-period momentum".to_string(),
                    normalization: NormalizationType::Robust,
                },
            ],
        }
    }

    #[test]
    fn test_mtf_feat_count() {
        let mtf = MultiTimeframeFactorConfig::new(
            sample_config(),
            vec!["1h".into(), "4h".into(), "1d".into()],
        );
        assert_eq!(mtf.base_feat_count(), 2);
        assert_eq!(mtf.feat_count(), 6);
        assert_eq!(mtf.feat_offset(), 6);
    }

    #[test]
    fn test_mtf_factor_names() {
        let mtf = MultiTimeframeFactorConfig::new(
            sample_config(),
            vec!["1h".into(), "4h".into(), "1d".into()],
        );
        let names = mtf.factor_names();
        assert_eq!(names.len(), 6);
        assert_eq!(names[0], "return_1h");
        assert_eq!(names[1], "momentum_1h");
        assert_eq!(names[2], "return_4h");
        assert_eq!(names[3], "momentum_4h");
        assert_eq!(names[4], "return_1d");
        assert_eq!(names[5], "momentum_1d");
    }

    #[test]
    fn test_mtf_single_resolution_matches_base() {
        let base = sample_config();
        let mtf = MultiTimeframeFactorConfig::new(base.clone(), vec!["1h".into()]);
        assert_eq!(mtf.feat_count(), base.feat_count());
        assert_eq!(mtf.feat_offset(), base.feat_offset());
    }
}
