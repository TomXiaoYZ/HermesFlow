use super::ops::*;
use crate::config::FactorConfig;
use ndarray::{Array2, Array3};
use std::env;

/// Statistics from VM execution for quality monitoring.
/// Tracks how often safety guards (NaN/Inf sanitization) fired during formula evaluation.
#[derive(Debug, Clone, Default)]
pub struct ExecutionStats {
    /// Total number of operator evaluations performed.
    pub total_ops: u32,
    /// Number of times the post-op NaN/Inf sanitization changed at least one element.
    pub protection_triggers: u32,
}

impl ExecutionStats {
    /// Ratio of ops that triggered protection guards. Range [0.0, 1.0].
    pub fn protection_ratio(&self) -> f64 {
        if self.total_ops == 0 {
            0.0
        } else {
            self.protection_triggers as f64 / self.total_ops as f64
        }
    }
}

#[derive(Debug, Clone)]
pub struct StackVM {
    pub feat_offset: usize,
    /// Rolling window size for time-series operators (TS_MEAN, TS_STD, etc.).
    /// Default: 10 for 1h data, 20 for 1d data, 8 for 15m data.
    pub ts_window: usize,
}

impl Default for StackVM {
    fn default() -> Self {
        let config_path =
            env::var("FACTOR_CONFIG").unwrap_or_else(|_| "config/factors.yaml".to_string());

        let feat_offset = FactorConfig::from_file(&config_path)
            .map(|c| c.feat_offset())
            .unwrap_or(6); // Fallback to 6 if config missing

        Self {
            feat_offset,
            ts_window: 10,
        }
    }
}

impl StackVM {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_config(config: &FactorConfig) -> Self {
        Self {
            feat_offset: config.feat_offset(),
            ts_window: 10,
        }
    }

    /// Create a StackVM with a specific time-series window size.
    pub fn with_window(config: &FactorConfig, ts_window: usize) -> Self {
        Self {
            feat_offset: config.feat_offset(),
            ts_window,
        }
    }

    /// Select appropriate TS window based on data resolution.
    pub fn ts_window_for_resolution(resolution: &str) -> usize {
        match resolution {
            "1d" => 20, // ~1 trading month
            "1h" => 10, // 10 hours
            "15m" => 8, // 2 hours
            _ => 10,
        }
    }

    /// Execute a formula on the given feature tensor.
    /// features shape: (batch, features, time)
    pub fn execute(&self, formula_tokens: &[usize], features: &Array3<f64>) -> Option<Array2<f64>> {
        let mut stack: Vec<Array2<f64>> = Vec::new();

        for &token in formula_tokens {
            if token < self.feat_offset {
                // Token is a feature index
                // Slice: (:, token, :) -> (batch, time)
                let feature_slice = features.index_axis(ndarray::Axis(1), token).to_owned();
                stack.push(feature_slice);
            } else {
                // Token is an operator
                // op_map in Python: {i + offset: cfg[1]}
                // We map token to op manually here or via a lookup
                let op_idx = token - self.feat_offset;

                // See OPS_CONFIG in Python for indices
                // 0: ADD, 1: SUB, 2: MUL, 3: DIV, 4: NEG, 5: ABS, 6: SIGN
                // 7: GATE, 8: JUMP, 9: DECAY, 10: DELAY1, 11: MAX3

                match op_idx {
                    0 => {
                        // ADD (arity 2)
                        if stack.len() < 2 {
                            return None;
                        }
                        let y = stack.pop()?;
                        let x = stack.pop()?;
                        stack.push(op_add(&x, &y));
                    }
                    1 => {
                        // SUB
                        if stack.len() < 2 {
                            return None;
                        }
                        let y = stack.pop()?;
                        let x = stack.pop()?;
                        stack.push(op_sub(&x, &y));
                    }
                    2 => {
                        // MUL
                        if stack.len() < 2 {
                            return None;
                        }
                        let y = stack.pop()?;
                        let x = stack.pop()?;
                        stack.push(op_mul(&x, &y));
                    }
                    3 => {
                        // DIV
                        if stack.len() < 2 {
                            return None;
                        }
                        let y = stack.pop()?;
                        let x = stack.pop()?;
                        stack.push(op_div(&x, &y));
                    }
                    4 => {
                        // NEG
                        if stack.is_empty() {
                            return None;
                        }
                        let x = stack.pop()?;
                        stack.push(op_neg(&x));
                    }
                    5 => {
                        // ABS
                        if stack.is_empty() {
                            return None;
                        }
                        let x = stack.pop()?;
                        stack.push(op_abs(&x));
                    }
                    6 => {
                        // SIGN
                        if stack.is_empty() {
                            return None;
                        }
                        let x = stack.pop()?;
                        stack.push(op_sign(&x));
                    }
                    7 => {
                        // GATE (arity 3)
                        if stack.len() < 3 {
                            return None;
                        }
                        let y = stack.pop()?;
                        let x = stack.pop()?;
                        let cond = stack.pop()?;
                        stack.push(op_gate(&cond, &x, &y));
                    }
                    8 => {
                        // SIGNED_POWER: sign(x) * |x|^0.5
                        if stack.is_empty() {
                            return None;
                        }
                        let x = stack.pop()?;
                        stack.push(op_signed_power(&x));
                    }
                    9 => {
                        // DECAY_LINEAR: linearly-weighted MA (legacy, no longer generated)
                        if stack.is_empty() {
                            return None;
                        }
                        let x = stack.pop()?;
                        stack.push(op_decay_linear(&x, self.ts_window));
                    }
                    10 => {
                        // DELAY1
                        if stack.is_empty() {
                            return None;
                        }
                        let x = stack.pop()?;
                        stack.push(ts_delay(&x, 1));
                    }
                    11 => {
                        // DELAY5
                        if stack.is_empty() {
                            return None;
                        }
                        let x = stack.pop()?;
                        stack.push(ts_delay(&x, 5));
                    }
                    12 => {
                        // TS_MEAN
                        if stack.is_empty() {
                            return None;
                        }
                        let x = stack.pop()?;
                        stack.push(ts_mean(&x, self.ts_window));
                    }
                    13 => {
                        // TS_STD
                        if stack.is_empty() {
                            return None;
                        }
                        let x = stack.pop()?;
                        stack.push(ts_std(&x, self.ts_window));
                    }
                    14 => {
                        // TS_RANK
                        if stack.is_empty() {
                            return None;
                        }
                        let x = stack.pop()?;
                        stack.push(ts_rank(&x, self.ts_window));
                    }
                    15 => {
                        // TS_SUM (legacy, no longer generated)
                        if stack.is_empty() {
                            return None;
                        }
                        let x = stack.pop()?;
                        stack.push(ts_sum(&x, self.ts_window));
                    }
                    16 => {
                        // TS_CORR
                        if stack.len() < 2 {
                            return None;
                        }
                        let y = stack.pop()?;
                        let x = stack.pop()?;
                        stack.push(ts_corr(&x, &y, self.ts_window));
                    }
                    17 => {
                        // TS_MIN: rolling minimum
                        if stack.is_empty() {
                            return None;
                        }
                        let x = stack.pop()?;
                        stack.push(ts_min(&x, self.ts_window));
                    }
                    18 => {
                        // TS_MAX: rolling maximum
                        if stack.is_empty() {
                            return None;
                        }
                        let x = stack.pop()?;
                        stack.push(ts_max(&x, self.ts_window));
                    }
                    19 => {
                        // LOG
                        if stack.is_empty() {
                            return None;
                        }
                        let x = stack.pop()?;
                        stack.push(op_log(&x));
                    }
                    20 => {
                        // SQRT
                        if stack.is_empty() {
                            return None;
                        }
                        let x = stack.pop()?;
                        stack.push(op_sqrt(&x));
                    }
                    21 => {
                        // TS_ARGMAX (legacy, no longer generated)
                        if stack.is_empty() {
                            return None;
                        }
                        let x = stack.pop()?;
                        stack.push(ts_argmax(&x, self.ts_window));
                    }
                    22 => {
                        // TS_DELTA
                        if stack.is_empty() {
                            return None;
                        }
                        let x = stack.pop()?;
                        stack.push(ts_delta(&x, 1)); // Default delta=1? Or specific. Let's assume 1 for basic op.
                                                     // Actually gplearn delta usually takes a parameter.
                                                     // In our RPN, parameters are hardcoded ops (TS_DELTA_1, TS_DELTA_5) or we pop parameter?
                                                     // For simplicity, let's make 22 = ts_delta(1).
                                                     // We can add TS_DELTA_5 as 23 later if needed.
                    }
                    _ => return None, // Unknown operator
                }
            }

            // Safety check for NaN/Inf (from Python: if torch.isnan(res).any()...)
            if let Some(top) = stack.last_mut() {
                top.mapv_inplace(|v| {
                    if v.is_nan() {
                        0.0
                    } else if v.is_infinite() {
                        if v.is_sign_positive() {
                            1.0
                        } else {
                            -1.0
                        }
                    } else {
                        v
                    }
                });
            }
        }

        if stack.len() == 1 {
            stack.pop()
        } else {
            None
        }
    }

    /// Execute a formula and return both the result and execution statistics.
    /// Tracks how often NaN/Inf sanitization fires per operator, indicating
    /// the genome relies on protected operations rather than clean arithmetic.
    pub fn execute_with_stats(
        &self,
        formula_tokens: &[usize],
        features: &Array3<f64>,
    ) -> (Option<Array2<f64>>, ExecutionStats) {
        let mut stack: Vec<Array2<f64>> = Vec::new();
        let mut stats = ExecutionStats::default();

        for &token in formula_tokens {
            if token < self.feat_offset {
                let feature_slice = features.index_axis(ndarray::Axis(1), token).to_owned();
                stack.push(feature_slice);
            } else {
                let op_idx = token - self.feat_offset;
                stats.total_ops += 1;

                match op_idx {
                    0 => {
                        if stack.len() < 2 { return (None, stats); }
                        let y = stack.pop().unwrap();
                        let x = stack.pop().unwrap();
                        stack.push(op_add(&x, &y));
                    }
                    1 => {
                        if stack.len() < 2 { return (None, stats); }
                        let y = stack.pop().unwrap();
                        let x = stack.pop().unwrap();
                        stack.push(op_sub(&x, &y));
                    }
                    2 => {
                        if stack.len() < 2 { return (None, stats); }
                        let y = stack.pop().unwrap();
                        let x = stack.pop().unwrap();
                        stack.push(op_mul(&x, &y));
                    }
                    3 => {
                        if stack.len() < 2 { return (None, stats); }
                        let y = stack.pop().unwrap();
                        let x = stack.pop().unwrap();
                        stack.push(op_div(&x, &y));
                    }
                    4 => {
                        if stack.is_empty() { return (None, stats); }
                        let x = stack.pop().unwrap();
                        stack.push(op_neg(&x));
                    }
                    5 => {
                        if stack.is_empty() { return (None, stats); }
                        let x = stack.pop().unwrap();
                        stack.push(op_abs(&x));
                    }
                    6 => {
                        if stack.is_empty() { return (None, stats); }
                        let x = stack.pop().unwrap();
                        stack.push(op_sign(&x));
                    }
                    7 => {
                        if stack.len() < 3 { return (None, stats); }
                        let y = stack.pop().unwrap();
                        let x = stack.pop().unwrap();
                        let cond = stack.pop().unwrap();
                        stack.push(op_gate(&cond, &x, &y));
                    }
                    8 => {
                        if stack.is_empty() { return (None, stats); }
                        let x = stack.pop().unwrap();
                        stack.push(op_signed_power(&x));
                    }
                    9 => {
                        if stack.is_empty() { return (None, stats); }
                        let x = stack.pop().unwrap();
                        stack.push(op_decay_linear(&x, self.ts_window));
                    }
                    10 => {
                        if stack.is_empty() { return (None, stats); }
                        let x = stack.pop().unwrap();
                        stack.push(ts_delay(&x, 1));
                    }
                    11 => {
                        if stack.is_empty() { return (None, stats); }
                        let x = stack.pop().unwrap();
                        stack.push(ts_delay(&x, 5));
                    }
                    12 => {
                        if stack.is_empty() { return (None, stats); }
                        let x = stack.pop().unwrap();
                        stack.push(ts_mean(&x, self.ts_window));
                    }
                    13 => {
                        if stack.is_empty() { return (None, stats); }
                        let x = stack.pop().unwrap();
                        stack.push(ts_std(&x, self.ts_window));
                    }
                    14 => {
                        if stack.is_empty() { return (None, stats); }
                        let x = stack.pop().unwrap();
                        stack.push(ts_rank(&x, self.ts_window));
                    }
                    15 => {
                        if stack.is_empty() { return (None, stats); }
                        let x = stack.pop().unwrap();
                        stack.push(ts_sum(&x, self.ts_window));
                    }
                    16 => {
                        if stack.len() < 2 { return (None, stats); }
                        let y = stack.pop().unwrap();
                        let x = stack.pop().unwrap();
                        stack.push(ts_corr(&x, &y, self.ts_window));
                    }
                    17 => {
                        if stack.is_empty() { return (None, stats); }
                        let x = stack.pop().unwrap();
                        stack.push(ts_min(&x, self.ts_window));
                    }
                    18 => {
                        if stack.is_empty() { return (None, stats); }
                        let x = stack.pop().unwrap();
                        stack.push(ts_max(&x, self.ts_window));
                    }
                    19 => {
                        if stack.is_empty() { return (None, stats); }
                        let x = stack.pop().unwrap();
                        stack.push(op_log(&x));
                    }
                    20 => {
                        if stack.is_empty() { return (None, stats); }
                        let x = stack.pop().unwrap();
                        stack.push(op_sqrt(&x));
                    }
                    21 => {
                        if stack.is_empty() { return (None, stats); }
                        let x = stack.pop().unwrap();
                        stack.push(ts_argmax(&x, self.ts_window));
                    }
                    22 => {
                        if stack.is_empty() { return (None, stats); }
                        let x = stack.pop().unwrap();
                        stack.push(ts_delta(&x, 1));
                    }
                    _ => return (None, stats),
                }

                // Count NaN/Inf sanitization triggers
                if let Some(top) = stack.last_mut() {
                    let mut triggered = false;
                    top.mapv_inplace(|v| {
                        if v.is_nan() {
                            triggered = true;
                            0.0
                        } else if v.is_infinite() {
                            triggered = true;
                            if v.is_sign_positive() { 1.0 } else { -1.0 }
                        } else {
                            v
                        }
                    });
                    if triggered {
                        stats.protection_triggers += 1;
                    }
                }
            }
        }

        let result = if stack.len() == 1 { stack.pop() } else { None };
        (result, stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::Array3;

    fn make_vm(feat_offset: usize) -> StackVM {
        StackVM {
            feat_offset,
            ts_window: 10,
        }
    }

    /// Build a (1, n_features, time) feature tensor from a flat slice per feature.
    fn features_1d(data: &[&[f64]], n_features: usize) -> Array3<f64> {
        let time = data[0].len();
        let mut arr = Array3::zeros((1, n_features, time));
        for (i, row) in data.iter().enumerate() {
            for (t, &v) in row.iter().enumerate() {
                arr[[0, i, t]] = v;
            }
        }
        arr
    }

    #[test]
    fn execution_stats_default_zero() {
        let stats = ExecutionStats::default();
        assert_eq!(stats.total_ops, 0);
        assert_eq!(stats.protection_triggers, 0);
        assert!((stats.protection_ratio() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn execution_stats_ratio_computation() {
        let stats = ExecutionStats {
            total_ops: 10,
            protection_triggers: 3,
        };
        assert!((stats.protection_ratio() - 0.3).abs() < 1e-10);
    }

    #[test]
    fn execute_with_stats_clean_genome_no_triggers() {
        // Formula: feature0 + feature1 (clean addition, no NaN/Inf)
        let vm = make_vm(2);
        let data: Vec<&[f64]> = vec![&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]];
        let features = features_1d(&data, 2);
        // Tokens: [0, 1, 2] = feat0, feat1, ADD (op_idx=0)
        let tokens = vec![0, 1, 2];
        let (result, stats) = vm.execute_with_stats(&tokens, &features);
        assert!(result.is_some());
        assert_eq!(stats.total_ops, 1);
        assert_eq!(stats.protection_triggers, 0);
    }

    #[test]
    fn execute_with_stats_nan_producing_formula_triggers() {
        // Build a formula that produces NaN through chained operations.
        // SUB(feat0, feat0) = 0, then DIV(zero, zero) produces 0/(0+1e-6)=0 (no trigger).
        // Instead, use LOG of values that include 0.0 in the feature data —
        // op_log protects near-zero but the result is 0.0 (no NaN). We need actual NaN/Inf output.
        //
        // The most reliable way: use MUL to create Inf (1e308 * 1e308 = Inf).
        let vm = make_vm(2);
        let data: Vec<&[f64]> = vec![&[1e308, 1e308, 1e308], &[1e308, 1e308, 1e308]];
        let features = features_1d(&data, 2);
        // Tokens: [0, 1, 4] = feat0, feat1, MUL (op_idx=2, token=2+2=4)
        let tokens = vec![0, 1, 4];
        let (result, stats) = vm.execute_with_stats(&tokens, &features);
        assert!(result.is_some());
        assert_eq!(stats.total_ops, 1);
        // 1e308 * 1e308 = Inf → sanitization triggers
        assert!(stats.protection_triggers > 0, "expected Inf trigger from overflow multiplication");
    }

    #[test]
    fn execute_with_stats_clean_div_no_triggers() {
        // Normal division: no NaN/Inf produced
        let vm = make_vm(2);
        let data: Vec<&[f64]> = vec![&[10.0, 20.0, 30.0], &[2.0, 4.0, 5.0]];
        let features = features_1d(&data, 2);
        // Tokens: [0, 1, 5] = feat0, feat1, DIV (op_idx=3, token=2+3=5)
        let tokens = vec![0, 1, 5];
        let (result, stats) = vm.execute_with_stats(&tokens, &features);
        assert!(result.is_some());
        assert_eq!(stats.total_ops, 1);
        assert_eq!(stats.protection_triggers, 0);
    }

    #[test]
    fn execute_with_stats_returns_same_result_as_execute() {
        let vm = make_vm(2);
        let data: Vec<&[f64]> = vec![&[1.0, 2.0, 3.0, 4.0], &[5.0, 6.0, 7.0, 8.0]];
        let features = features_1d(&data, 2);
        // feat0 * feat1
        let tokens = vec![0, 1, 4]; // op_idx 2 = MUL (token = 2 + 2 = 4)
        let result_basic = vm.execute(&tokens, &features);
        let (result_stats, _) = vm.execute_with_stats(&tokens, &features);
        assert!(result_basic.is_some());
        assert!(result_stats.is_some());
        let a = result_basic.unwrap();
        let b = result_stats.unwrap();
        assert_eq!(a.shape(), b.shape());
        for (va, vb) in a.iter().zip(b.iter()) {
            assert!((va - vb).abs() < 1e-10, "results differ: {} vs {}", va, vb);
        }
    }

    #[test]
    fn execute_with_stats_invalid_formula_returns_none() {
        let vm = make_vm(2);
        let data: Vec<&[f64]> = vec![&[1.0], &[2.0]];
        let features = features_1d(&data, 2);
        // ADD with only one feature on stack → should fail
        let tokens = vec![0, 2]; // feat0, ADD needs 2 args
        let (result, stats) = vm.execute_with_stats(&tokens, &features);
        assert!(result.is_none());
        // total_ops is incremented before the arity check, so it counts the failed attempt
        assert_eq!(stats.total_ops, 1);
    }
}
