use super::ops::*;
use crate::config::FactorConfig;
use ndarray::{Array2, Array3};
use std::env;

#[derive(Debug, Clone)]
pub struct StackVM {
    pub feat_offset: usize,
}

impl Default for StackVM {
    fn default() -> Self {
        let config_path =
            env::var("FACTOR_CONFIG").unwrap_or_else(|_| "config/factors.yaml".to_string());

        let feat_offset = FactorConfig::from_file(&config_path)
            .map(|c| c.feat_offset())
            .unwrap_or(6); // Fallback to 6 if config missing

        Self { feat_offset }
    }
}

impl StackVM {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_config(config: &FactorConfig) -> Self {
        Self {
            feat_offset: config.feat_offset(),
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
                        // DECAY_LINEAR: linearly-weighted MA (10 periods)
                        if stack.is_empty() {
                            return None;
                        }
                        let x = stack.pop()?;
                        stack.push(op_decay_linear(&x, 10));
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
                        // TS_MEAN_10
                        if stack.is_empty() {
                            return None;
                        }
                        let x = stack.pop()?;
                        stack.push(ts_mean(&x, 10));
                    }
                    13 => {
                        // TS_STD_10
                        if stack.is_empty() {
                            return None;
                        }
                        let x = stack.pop()?;
                        stack.push(ts_std(&x, 10));
                    }
                    14 => {
                        // TS_RANK_10
                        if stack.is_empty() {
                            return None;
                        }
                        let x = stack.pop()?;
                        stack.push(ts_rank(&x, 10));
                    }
                    15 => {
                        // TS_SUM_10
                        if stack.is_empty() {
                            return None;
                        }
                        let x = stack.pop()?;
                        stack.push(ts_sum(&x, 10));
                    }
                    16 => {
                        // TS_CORR_10
                        if stack.len() < 2 {
                            return None;
                        }
                        let y = stack.pop()?;
                        let x = stack.pop()?;
                        stack.push(ts_corr(&x, &y, 10));
                    }
                    17 => {
                        // TS_MIN_10: rolling 10-period minimum
                        if stack.is_empty() {
                            return None;
                        }
                        let x = stack.pop()?;
                        stack.push(ts_min(&x, 10));
                    }
                    18 => {
                        // TS_MAX_10: rolling 10-period maximum
                        if stack.is_empty() {
                            return None;
                        }
                        let x = stack.pop()?;
                        stack.push(ts_max(&x, 10));
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
                        // TS_ARGMAX_10: position of max in 10-period window
                        if stack.is_empty() {
                            return None;
                        }
                        let x = stack.pop()?;
                        stack.push(ts_argmax(&x, 10));
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
}
