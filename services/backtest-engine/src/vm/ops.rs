use ndarray::{s, Array2, ArrayBase, Data, Ix2};

/// Time-series delay: Shift array along time axis (axis 1) with zero padding.
/// x shape: (batch, time)
pub fn ts_delay<S: Data<Elem = f64>>(x: &ArrayBase<S, Ix2>, d: usize) -> Array2<f64> {
    if d == 0 {
        return x.mapv(|v| v);
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
pub fn op_gate<S1, S2, S3>(
    condition: &ArrayBase<S1, Ix2>,
    x: &ArrayBase<S2, Ix2>,
    y: &ArrayBase<S3, Ix2>,
) -> Array2<f64>
where
    S1: Data<Elem = f64>,
    S2: Data<Elem = f64>,
    S3: Data<Elem = f64>,
{
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
pub fn op_jump<S: Data<Elem = f64>>(x: &ArrayBase<S, Ix2>) -> Array2<f64> {
    let mut out = x.mapv(|v| v);

    // Compute mean and std per row (batch item)
    // Note: Python code does mean(dim=1, keepdim=True), so per time series

    for mut row in out.rows_mut() {
        let mean = row.mean().unwrap_or(0.0);
        let std = row.std(0.0) + 1e-6; // Add epsilon

        row.mapv_inplace(|v| {
            let z = (v - mean) / std;
            let val = z - 3.0;
            if val > 0.0 {
                val
            } else {
                0.0
            } // ReLU
        });
    }

    out
}

/// Decay operator: x + 0.8 * delay(x, 1) + 0.6 * delay(x, 2)
pub fn op_decay<S: Data<Elem = f64>>(x: &ArrayBase<S, Ix2>) -> Array2<f64> {
    let d1 = ts_delay(x, 1);
    let d2 = ts_delay(x, 2);

    x + &(d1 * 0.8) + &(d2 * 0.6)
}

/// Helper for element-wise ops
pub fn op_add<S1: Data<Elem = f64>, S2: Data<Elem = f64>>(
    x: &ArrayBase<S1, Ix2>,
    y: &ArrayBase<S2, Ix2>,
) -> Array2<f64> {
    x + y
}
pub fn op_sub<S1: Data<Elem = f64>, S2: Data<Elem = f64>>(
    x: &ArrayBase<S1, Ix2>,
    y: &ArrayBase<S2, Ix2>,
) -> Array2<f64> {
    x - y
}
pub fn op_mul<S1: Data<Elem = f64>, S2: Data<Elem = f64>>(
    x: &ArrayBase<S1, Ix2>,
    y: &ArrayBase<S2, Ix2>,
) -> Array2<f64> {
    x * y
}
pub fn op_div<S1: Data<Elem = f64>, S2: Data<Elem = f64>>(
    x: &ArrayBase<S1, Ix2>,
    y: &ArrayBase<S2, Ix2>,
) -> Array2<f64> {
    let y_safe = y + 1e-6;
    x / &y_safe
}
pub fn op_neg<S: Data<Elem = f64>>(x: &ArrayBase<S, Ix2>) -> Array2<f64> {
    -x
}
pub fn op_abs<S: Data<Elem = f64>>(x: &ArrayBase<S, Ix2>) -> Array2<f64> {
    x.mapv(f64::abs)
}
pub fn op_sign<S: Data<Elem = f64>>(x: &ArrayBase<S, Ix2>) -> Array2<f64> {
    x.mapv(f64::signum)
}
pub fn op_max3<S: Data<Elem = f64>>(x: &ArrayBase<S, Ix2>) -> Array2<f64> {
    let d1 = ts_delay(x, 1);
    let d2 = ts_delay(x, 2);

    let mut out = x.mapv(|v| v);
    ndarray::Zip::from(&mut out)
        .and(&d1)
        .and(&d2)
        .for_each(|o, &v1, &v2| {
            *o = o.max(v1).max(v2);
        });
    out
}

/// Natural Logarithm: ln(|x|). Protected: if x approx 0 -> 0.
pub fn op_log<S: Data<Elem = f64>>(x: &ArrayBase<S, Ix2>) -> Array2<f64> {
    x.mapv(|v| {
        let abs_v = v.abs();
        if abs_v < 1e-9 {
            0.0
        } else {
            abs_v.ln()
        }
    })
}

/// Square Root: sqrt(|x|).
pub fn op_sqrt<S: Data<Elem = f64>>(x: &ArrayBase<S, Ix2>) -> Array2<f64> {
    x.mapv(|v| v.abs().sqrt())
}

/// Inverse: 1/x. Protected: if x approx 0 -> 0.
pub fn op_inv<S: Data<Elem = f64>>(x: &ArrayBase<S, Ix2>) -> Array2<f64> {
    x.mapv(|v| if v.abs() < 1e-9 { 0.0 } else { 1.0 / v })
}

/// Time-series Delta: x[t] - x[t-d]
pub fn ts_delta<S: Data<Elem = f64>>(x: &ArrayBase<S, Ix2>, d: usize) -> Array2<f64> {
    let delayed = ts_delay(x, d);
    x - &delayed
}

/// Signed Power: sign(x) * |x|^0.5
/// Preserves sign while compressing magnitude — standard WorldQuant operator.
pub fn op_signed_power<S: Data<Elem = f64>>(x: &ArrayBase<S, Ix2>) -> Array2<f64> {
    x.mapv(|v| v.signum() * v.abs().sqrt())
}

/// Decay Linear: linearly-weighted moving average over d periods.
/// Weight d for most recent, d-1 for lag 1, ..., 1 for lag d-1.
/// Equivalent to WorldQuant's decay_linear.
pub fn op_decay_linear<S: Data<Elem = f64>>(x: &ArrayBase<S, Ix2>, d: usize) -> Array2<f64> {
    let total_weight: f64 = (1..=d).sum::<usize>() as f64;
    let mut sum = Array2::zeros(x.dim());
    for i in 0..d {
        let weight = (d - i) as f64;
        sum = sum + ts_delay(x, i) * weight;
    }
    sum / total_weight
}

/// Time-series Argmax: position of maximum value in last d periods.
/// Normalized to [0, 1] where 0 = max at oldest, 1 = max at most recent.
pub fn ts_argmax<S: Data<Elem = f64>>(x: &ArrayBase<S, Ix2>, d: usize) -> Array2<f64> {
    let (batch, time) = x.dim();
    let mut out = Array2::zeros((batch, time));
    for b in 0..batch {
        for t in 0..time {
            let mut best_val = f64::NEG_INFINITY;
            let mut best_pos: usize = 0;
            for i in 0..d {
                let idx = t as isize - i as isize;
                let val = if idx >= 0 { x[[b, idx as usize]] } else { 0.0 };
                if val > best_val {
                    best_val = val;
                    best_pos = i;
                }
            }
            // best_pos=0 means max is at current (most recent), best_pos=d-1 means oldest
            // Normalize: 0 = oldest, 1 = most recent
            out[[b, t]] = if d > 1 {
                1.0 - (best_pos as f64 / (d - 1) as f64)
            } else {
                0.5
            };
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use ndarray::arr2;

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

    #[test]
    fn test_ts_mean() {
        let x = arr2(&[[1., 2., 3., 4.], [10., 10., 10., 10.]]);
        let m = ts_mean(&x, 2);
        // t=0: mean(1) = 1/2? No, sum(1, 0)/2 = 0.5.
        // Logic: sum = x + delay(x,1).
        // t=0: 1+0=1. mean=0.5.
        // t=1: 2+1=3. mean=1.5.
        // t=2: 3+2=5. mean=2.5.
        // t=3: 4+3=7. mean=3.5.
        // Row 2: 10+0=10 (5), 10+10=20 (10)...
        let expected = arr2(&[[0.5, 1.5, 2.5, 3.5], [5.0, 10.0, 10.0, 10.0]]);
        assert_abs_diff_eq!(m, expected, epsilon = 1e-6);
    }
}

/// Time-series Mean — P8-3B: row-wise running sum avoids d allocations.
///
/// Computes a rolling mean over the last `d` periods along axis 1 (time).
/// Uses a single-pass running sum per row instead of creating d delayed arrays.
pub fn ts_mean<S: Data<Elem = f64>>(x: &ArrayBase<S, Ix2>, d: usize) -> Array2<f64> {
    let (batch, time) = x.dim();
    if d == 0 {
        return Array2::zeros((batch, time));
    }
    let mut out = Array2::zeros((batch, time));
    let inv_d = 1.0 / d as f64;

    for b in 0..batch {
        let mut running_sum = 0.0;
        for t in 0..time {
            running_sum += x[[b, t]];
            if t >= d {
                running_sum -= x[[b, t - d]];
            }
            // Before the full window is available, we still divide by d
            // (zero-padded past values, matching the original delay-based behavior)
            out[[b, t]] = running_sum * inv_d;
        }
    }
    out
}

/// Time-series Sum — P8-3B: row-wise running sum avoids d allocations.
pub fn ts_sum<S: Data<Elem = f64>>(x: &ArrayBase<S, Ix2>, d: usize) -> Array2<f64> {
    let (batch, time) = x.dim();
    if d == 0 {
        return Array2::zeros((batch, time));
    }
    let mut out = Array2::zeros((batch, time));

    for b in 0..batch {
        let mut running_sum = 0.0;
        for t in 0..time {
            running_sum += x[[b, t]];
            if t >= d {
                running_sum -= x[[b, t - d]];
            }
            out[[b, t]] = running_sum;
        }
    }
    out
}

/// Time-series Standard Deviation
pub fn ts_std<S: Data<Elem = f64>>(x: &ArrayBase<S, Ix2>, d: usize) -> Array2<f64> {
    let mean = ts_mean(x, d);

    // std = sqrt( E[x^2] - E[x]^2 )
    // E[x^2]
    let x2 = x.mapv(|v| v * v);
    let mean_x2 = ts_mean(&x2, d);

    let var = &mean_x2 - &(&mean * &mean);
    // relu var to avoid negative due to float precision
    var.mapv(|v| if v > 0.0 { v.sqrt() } else { 0.0 })
}

/// Time-series Correlation (x, y, d)
pub fn ts_corr<S1: Data<Elem = f64>, S2: Data<Elem = f64>>(
    x: &ArrayBase<S1, Ix2>,
    y: &ArrayBase<S2, Ix2>,
    d: usize,
) -> Array2<f64> {
    // Corr(X, Y) = Cov(X, Y) / (Std(X) * Std(Y))
    // Cov(X, Y) = E[XY] - E[X]E[Y]

    let xy = x * y;
    let mean_xy = ts_mean(&xy, d);
    let mean_x = ts_mean(x, d);
    let mean_y = ts_mean(y, d);

    let cov = &mean_xy - &(&mean_x * &mean_y);

    let std_x = ts_std(x, d);
    let std_y = ts_std(y, d);

    let denom = &std_x * &std_y + 1e-9;
    cov / denom
}

/// Time-series Product
pub fn ts_product<S: Data<Elem = f64>>(x: &ArrayBase<S, Ix2>, d: usize) -> Array2<f64> {
    let mut prod = Array2::ones(x.dim());
    for i in 0..d {
        prod = prod * ts_delay(x, i);
    }
    prod
}

/// Time-series Min
pub fn ts_min<S: Data<Elem = f64>>(x: &ArrayBase<S, Ix2>, d: usize) -> Array2<f64> {
    let mut val = x.mapv(|v| v);
    for i in 1..d {
        // Start from 1 since 0 is x itself
        let delayed = ts_delay(x, i);
        // Element-wise min
        ndarray::Zip::from(&mut val)
            .and(&delayed)
            .for_each(|v, &d| *v = v.min(d));
    }
    val
}

/// Time-series Max
pub fn ts_max<S: Data<Elem = f64>>(x: &ArrayBase<S, Ix2>, d: usize) -> Array2<f64> {
    let mut val = x.mapv(|v| v);
    for i in 1..d {
        let delayed = ts_delay(x, i);
        ndarray::Zip::from(&mut val)
            .and(&delayed)
            .for_each(|v, &d| *v = v.max(d));
    }
    val
}

/// Time-series Rank (of current value in last d days)
pub fn ts_rank<S: Data<Elem = f64>>(x: &ArrayBase<S, Ix2>, d: usize) -> Array2<f64> {
    // For each element, count how many in the past d window are < current
    // Output normalized to [0, 1]

    let mut count = Array2::zeros(x.dim());

    // Naively:
    for i in 0..d {
        let delayed = ts_delay(x, i);

        let is_smaller =
            ndarray::Zip::from(x).and(&delayed).map_collect(
                |&cx, &dx| {
                    if dx < cx {
                        1.0
                    } else {
                        0.0
                    }
                },
            );
        count = count + is_smaller;
    }

    count / (d as f64)
}

/// Cross-Sectional Rank
/// Rank inputs along the batch dimension (Axis 0) for each timestep
/// Normalized to [0, 1]
pub fn cs_rank<S: Data<Elem = f64>>(x: &ArrayBase<S, Ix2>) -> Array2<f64> {
    let (batch, time) = x.dim();
    let mut out = Array2::zeros(x.dim());

    // Iterate over columns (time steps)
    for t in 0..time {
        // Extract column t
        let col = x.index_axis(ndarray::Axis(1), t);
        let mut v: Vec<(usize, f64)> = col.iter().enumerate().map(|(i, &v)| (i, v)).collect();

        // Sort by value
        v.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        // Assign ranks
        for (rank, (original_idx, _)) in v.iter().enumerate() {
            // Normalize rank to [0, 1]
            // rank 0 is smallest -> 0.0
            // rank N-1 is largest -> 1.0
            let norm_rank = if batch > 1 {
                rank as f64 / (batch - 1) as f64
            } else {
                0.5
            };
            out[[*original_idx, t]] = norm_rank;
        }
    }

    out
}

/// Cross-Sectional Mean
pub fn cs_mean<S: Data<Elem = f64>>(x: &ArrayBase<S, Ix2>) -> Array2<f64> {
    let mean_1d = x.mean_axis(ndarray::Axis(0)).unwrap(); // Shape (time,)

    let mut out = Array2::zeros(x.dim());
    // Copy row to all rows
    for mut row in out.rows_mut() {
        row.assign(&mean_1d);
    }
    out
}
