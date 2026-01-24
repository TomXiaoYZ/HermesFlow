use ndarray::{Array2, Array3};
use super::ops::*;

#[derive(Debug, Clone)]
pub struct StackVM {
    pub feat_offset: usize,
}

impl Default for StackVM {
    fn default() -> Self {
        Self {
            feat_offset: 6, // Matches AlphaGPT default
        }
    }
}

impl StackVM {
    pub fn new() -> Self {
        Self::default()
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
                    0 => { // ADD (arity 2)
                        if stack.len() < 2 { return None; }
                        let y = stack.pop()?;
                        let x = stack.pop()?;
                        stack.push(op_add(&x, &y));
                    },
                    1 => { // SUB
                        if stack.len() < 2 { return None; }
                        let y = stack.pop()?;
                        let x = stack.pop()?;
                        stack.push(op_sub(&x, &y));
                    },
                    2 => { // MUL
                        if stack.len() < 2 { return None; }
                        let y = stack.pop()?;
                        let x = stack.pop()?;
                        stack.push(op_mul(&x, &y));
                    },
                    3 => { // DIV
                        if stack.len() < 2 { return None; }
                        let y = stack.pop()?;
                        let x = stack.pop()?;
                        stack.push(op_div(&x, &y));
                    },
                    4 => { // NEG
                        if stack.len() < 1 { return None; }
                        let x = stack.pop()?;
                        stack.push(op_neg(&x));
                    },
                    5 => { // ABS
                        if stack.len() < 1 { return None; }
                        let x = stack.pop()?;
                        stack.push(op_abs(&x));
                    },
                    6 => { // SIGN
                        if stack.len() < 1 { return None; }
                        let x = stack.pop()?;
                        stack.push(op_sign(&x));
                    },
                    7 => { // GATE (arity 3)
                        if stack.len() < 3 { return None; }
                        let y = stack.pop()?;
                        let x = stack.pop()?;
                        let cond = stack.pop()?;
                        stack.push(op_gate(&cond, &x, &y));
                    },
                    8 => { // JUMP
                        if stack.len() < 1 { return None; }
                        let x = stack.pop()?;
                        stack.push(op_jump(&x));
                    },
                    9 => { // DECAY
                        if stack.len() < 1 { return None; }
                        let x = stack.pop()?;
                        stack.push(op_decay(&x));
                    },
                    10 => { // DELAY1
                        if stack.len() < 1 { return None; }
                        let x = stack.pop()?;
                        stack.push(ts_delay(&x, 1));
                    },
                    11 => { // MAX3
                        if stack.len() < 1 { return None; }
                        let x = stack.pop()?;
                        stack.push(op_max3(&x));
                    },
                    _ => return None, // Unknown operator
                }
            }
            
            // Safety check for NaN/Inf (from Python: if torch.isnan(res).any()...)
            if let Some(top) = stack.last_mut() {
                 top.mapv_inplace(|v| {
                     if v.is_nan() { 0.0 }
                     else if v.is_infinite() { 
                         if v.is_sign_positive() { 1.0 } else { -1.0 }
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
