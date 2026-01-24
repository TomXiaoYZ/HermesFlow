use anyhow::{anyhow, Result};
use backtest_engine::vm::StackVM;
use ndarray::Array3;
use tracing::info;

pub struct SignalGenerator {
    vm: StackVM,
}

impl Default for SignalGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalGenerator {
    pub fn new() -> Self {
        Self { vm: StackVM::new() }
    }

    /// Run the formula on the pre-computed feature tensor.
    /// features shape: (batch_size, num_features, time_steps)
    /// formula: Vec<usize> tokens
    /// Returns: Vec<f64> scores for the last timestep for each batch item
    pub fn generate_signals(&self, formula: &[usize], features: &Array3<f64>) -> Result<Vec<f64>> {
        info!("Running inference on batch size: {}", features.dim().0);

        let raw_output = self
            .vm
            .execute(formula, features)
            .ok_or_else(|| anyhow!("VM Execution returned None"))?;

        // raw_output shape: (batch, time)
        // We only care about the last timestep for live trading signal

        let (batch_size, time_steps) = raw_output.dim();
        if time_steps == 0 {
            return Err(anyhow!("VM Output has 0 timesteps"));
        }

        let mut scores = Vec::with_capacity(batch_size);

        for i in 0..batch_size {
            let last_val = raw_output[[i, time_steps - 1]];
            // Sigmoid: 1 / (1 + exp(-x))
            let score = 1.0 / (1.0 + (-last_val).exp());
            scores.push(score);
        }

        Ok(scores)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::Array3;

    #[test]
    fn test_generate_signals() {
        let generator = SignalGenerator::new();

        // Create a dummy formula: JUST a feature index 0
        // If feature 0 is high, score should be high
        let formula = vec![0];

        // Features: (batch=2, feats=3, time=4)
        let mut features = Array3::zeros((2, 3, 4));

        // Set last timestep values for feature 0
        features[[0, 0, 3]] = 5.0; // High value -> close to 1
        features[[1, 0, 3]] = -5.0; // Low value -> close to 0

        let scores = generator.generate_signals(&formula, &features).unwrap();

        assert_eq!(scores.len(), 2);
        assert!(scores[0] > 0.99); // Sigmoid(5) ~= 0.993
        assert!(scores[1] < 0.01); // Sigmoid(-5) ~= 0.006
    }
}
