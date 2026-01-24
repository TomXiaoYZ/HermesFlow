use ndarray::{Array2, s};
// use ndarray_stats::QuantileExt; // Unused here

/// Time-series delay: Shift array along time axis (axis 1) with zero padding.
/// x shape: (batch, time)
pub fn ts_delay(x: &Array2<f64>, d: usize) -> Array2<f64> {
    if d == 0 {
        return x.clone();
    }
    
    let (batch_size, time_steps) = x.dim();
    
    // If delay is larger than time steps, return all zeros
    if d >= time_steps {
        return Array2::zeros((batch_size, time_steps));
    }
    
    let mut out = Array2::zeros((batch_size, time_steps));
    
    // Slice input: x[:, :-d] -> dim (batch, time-d)
    let slice_in = x.slice(s![.., ..(time_steps - d)]);
    
    // Slice output: out[:, d:] -> dim (batch, time-d)
    // We assign the displaced input to the later part of output
    out.slice_mut(s![.., d..]).assign(&slice_in);
    
    out
}

/// Gate operator: if condition > 0 then x else y
pub fn op_gate(condition: &Array2<f64>, x: &Array2<f64>, y: &Array2<f64>) -> Array2<f64> {
    // We can use zip_mut_with or mapv
    // out = mask * x + (1 - mask) * y 
    // equivalent to: if c > 0 { x } else { y }
    
    let mut out = Array2::zeros(condition.dim());
    
    ndarray::Zip::from(&mut out)
        .and(condition)
        .and(x)
        .and(y)
        .for_each(|o, &c, &val_x, &val_y| {
            *o = if c > 0.0 { val_x } else { val_y };
        });
        
    out
}

/// Jump operator: Z-score normalization -> Relu(z - 3.0)
pub fn op_jump(x: &Array2<f64>) -> Array2<f64> {
    let mut out = x.clone();
    
    // Compute mean and std per row (batch item)
    // Note: Python code does mean(dim=1, keepdim=True), so per time series
    
    for mut row in out.rows_mut() {
        let mean = row.mean().unwrap_or(0.0);
        let std = row.std(0.0) + 1e-6; // Add epsilon
        
        row.mapv_inplace(|v| {
            let z = (v - mean) / std;
            let val = z - 3.0;
            if val > 0.0 { val } else { 0.0 } // ReLU
        });
    }
    
    out
}

/// Decay operator: x + 0.8 * delay(x, 1) + 0.6 * delay(x, 2)
pub fn op_decay(x: &Array2<f64>) -> Array2<f64> {
    let d1 = ts_delay(x, 1);
    let d2 = ts_delay(x, 2);
    
    x + &(d1 * 0.8) + &(d2 * 0.6)
}

/// Helper for element-wise ops
pub fn op_add(x: &Array2<f64>, y: &Array2<f64>) -> Array2<f64> { x + y }
pub fn op_sub(x: &Array2<f64>, y: &Array2<f64>) -> Array2<f64> { x - y }
pub fn op_mul(x: &Array2<f64>, y: &Array2<f64>) -> Array2<f64> { x * y }
pub fn op_div(x: &Array2<f64>, y: &Array2<f64>) -> Array2<f64> { x / (y + 1e-6) }
pub fn op_neg(x: &Array2<f64>) -> Array2<f64> { -x }
pub fn op_abs(x: &Array2<f64>) -> Array2<f64> { x.mapv(f64::abs) }
pub fn op_sign(x: &Array2<f64>) -> Array2<f64> { x.mapv(f64::signum) }
pub fn op_max3(x: &Array2<f64>) -> Array2<f64> {
    let d1 = ts_delay(x, 1);
    let d2 = ts_delay(x, 2);
    
    let mut out = x.clone();
    ndarray::Zip::from(&mut out)
        .and(&d1)
        .and(&d2)
        .for_each(|o, &v1, &v2| {
            *o = o.max(v1).max(v2);
        });
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::arr2;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_ts_delay() {
        let x = arr2(&[[1., 2., 3., 4.], [5., 6., 7., 8.]]);
        let d = ts_delay(&x, 1);
        
        // Expected: [[0, 1, 2, 3], [0, 5, 6, 7]]
        let expected = arr2(&[[0., 1., 2., 3.], [0., 5., 6., 7.]]);
        assert_abs_diff_eq!(d, expected, epsilon = 1e-6);
        
        let d2 = ts_delay(&x, 2);
        // Expected: [[0, 0, 1, 2], [0, 0, 5, 6]]
        let expected2 = arr2(&[[0., 0., 1., 2.], [0., 0., 5., 6.]]);
        assert_abs_diff_eq!(d2, expected2, epsilon = 1e-6);
    }
    
    #[test]
    fn test_gate() {
        let cond = arr2(&[[1., -1., 0.5], [0., 2., -2.]]);
        let x = arr2(&[[10., 10., 10.], [20., 20., 20.]]);
        let y = arr2(&[[5., 5., 5.], [15., 15., 15.]]);
        
        let res = op_gate(&cond, &x, &y);
        // cond > 0 -> x, else y
        // [T, F, T] -> [10, 5, 10]
        // [F, T, F] -> [15, 20, 15]
        let expected = arr2(&[[10., 5., 10.], [15., 20., 15.]]);
        assert_abs_diff_eq!(res, expected);
    }
}
