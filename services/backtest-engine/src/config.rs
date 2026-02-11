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
