use super::traits::{Factor, FactorContext};
use super::registry::FactorRegistry;
use super::engineer::FeatureEngineer;
use ndarray::{Array2, Array3, Axis};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FactorConfig {
    pub slug: String,
    pub normalization: Option<String>,
   pub parameters: Option<serde_json::Value>,
}

/// Dynamic feature engineer that loads factors from database
pub struct DynamicFeatureEngineer {
    registry: FactorRegistry,
    active_factors: Vec<FactorConfig>,
}



impl DynamicFeatureEngineer {
    /// Create from factor configurations
    /// Configurations typically loaded from database in strategy-generator
    pub fn from_configs(configs: Vec<FactorConfig>) -> Self {
        Self {
            registry: FactorRegistry::new(),
            active_factors: configs,
        }
    }
    
    /// Get number of active factors
    pub fn num_factors(&self) -> usize {
        self.active_factors.len()
    }
    
    /// Compute all active factors
    pub fn compute_features(
        &self,
        ohlcv: &super::traits::OhlcvData<'_>,
    ) -> Array3<f64> {
        let num_factors = self.active_factors.len();
        let (batch, time) = ohlcv.close.dim();
        let mut features = Array3::zeros((batch, num_factors, time));

        let mut ctx = FactorContext {
            close: ohlcv.close.clone(),
            open: ohlcv.open.clone(),
            high: ohlcv.high.clone(),
            low: ohlcv.low.clone(),
            volume: ohlcv.volume.clone(),
            liquidity: ohlcv.liquidity.clone(),
            fdv: ohlcv.fdv.clone(),
            cache: HashMap::new(),
        };
        
        for (idx, config) in self.active_factors.iter().enumerate() {
            // Create factor instance
            let params = config.parameters.as_ref()
                .unwrap_or(&serde_json::Value::Null);
            
            let factor = self.registry.create(&config.slug, params)
                .unwrap_or_else(|| panic!("Unknown factor: {}", config.slug));
            
            // Compute
            let mut result = factor.compute(&mut ctx);
            
            // Apply normalization
            if let Some(norm) = &config.normalization {
                result = match norm.as_str() {
                    "robust" => FeatureEngineer::robust_norm(&result),
                    "minmax" => Self::minmax_norm(&result),
                    "zscore" => Self::zscore_norm(&result),
                    "custom" | "none" | _ => result,
                };
            }
            
            // Store in features array
            features.index_axis_mut(Axis(1), idx).assign(&result);
            
            // Cache for dependencies
            ctx.cache.insert(config.slug.clone(), result);
        }
        
        features
    }
    
    /// Min-max normalization
    fn minmax_norm(x: &Array2<f64>) -> Array2<f64> {
        let mut out = x.clone();
        for mut row in out.rows_mut() {
            let min = row.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = row.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let range = max - min + 1e-9;
            row.mapv_inplace(|v| (v - min) / range);
        }
        out
    }
    
    /// Z-score normalization
    fn zscore_norm(x: &Array2<f64>) -> Array2<f64> {
        let mut out = x.clone();
        for mut row in out.rows_mut() {
            let mean = row.mean().unwrap_or(0.0);
            let std = row.std(0.0) + 1e-9;
            row.mapv_inplace(|v| (v - mean) / std);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::arr2;
    
    #[test]
    fn test_dynamic_engineer_basic() {
        let configs = vec![
            FactorConfig {
                slug: "log_returns".to_string(),
                normalization: Some("robust".to_string()),
                parameters: None,
            },
            FactorConfig {
                slug: "ema_12".to_string(),
                normalization: Some("robust".to_string()),
                parameters: None,
            },
        ];
        
        let engineer = DynamicFeatureEngineer::from_configs(configs);
        assert_eq!(engineer.num_factors(), 2);
        
        let close = arr2(&[[10.0, 11.0, 12.0, 13.0]]);
        let open = arr2(&[[9.5, 10.5, 11.5, 12.5]]);
        let high = arr2(&[[10.5, 11.5, 12.5, 13.5]]);
        let low = arr2(&[[9.0, 10.0, 11.0, 12.0]]);
        let volume = arr2(&[[1000.0, 1100.0, 1200.0, 1300.0]]);
        let liq = arr2(&[[5000.0, 5000.0, 5000.0, 5000.0]]);
        let fdv = arr2(&[[100000.0, 110000.0, 120000.0, 130000.0]]);
        
        let ohlcv = super::traits::OhlcvData {
            close: &close,
            open: &open,
            high: &high,
            low: &low,
            volume: &volume,
            liquidity: &liq,
            fdv: &fdv,
            ref_close: None,
        };
        let features = engineer.compute_features(&ohlcv);
        
        assert_eq!(features.dim(), (1, 2, 4)); // batch=1, factors=2, time=4
    }
}
