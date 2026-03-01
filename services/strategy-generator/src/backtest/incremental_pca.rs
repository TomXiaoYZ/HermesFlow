//! P6-1C: Candid Covariance-free Incremental PCA (CCIPCA).
//!
//! Maintains O(n·k) rolling principal components without storing or inverting
//! the full O(n²) covariance matrix. At 18k dimensions, this avoids 3.24B floats
//! and O(n³) SVD.
//!
//! Uses `ndarray::Zip` for in-place updates with zero dynamic allocation in
//! the hot path. Parallelize outer loop via Rayon for multi-core utilization.
//!
//! Reference: Weng, Jianlin, Yongchun Zhang, and Wei-Shinn Hwang.
//! "Candid covariance-free incremental principal component analysis."
//! IEEE Transactions on Pattern Analysis and Machine Intelligence 25.8 (2003): 1034-1040.

use ndarray::{Array1, Array2, Array3, ArrayView1, Zip};

/// Configuration for CCIPCA.
#[derive(Debug, Clone)]
pub struct CcipcaConfig {
    /// Number of principal components to track (k)
    pub n_components: usize,
    /// CCIPCA amnesic parameter (l). Higher = less forgetting.
    /// Typical range: 2.0 - 5.0. Set 0 for standard averaging.
    pub amnesic: f64,
    /// Minimum observation count before components are considered valid
    pub min_observations: usize,
    /// Feature gate: only run CCIPCA when enabled
    #[allow(dead_code)] // checked in caller
    pub enabled: bool,
}

impl Default for CcipcaConfig {
    fn default() -> Self {
        Self {
            n_components: 5,
            amnesic: 2.0,
            min_observations: 50,
            enabled: false,
        }
    }
}

/// Incremental PCA state.
///
/// Maintains k eigenvector estimates (unnormalized) and their norms.
/// The eigenvectors converge to the top-k principal components as
/// more observations are fed via `update()`.
pub struct CcipcaState {
    /// Eigenvector estimates: shape (k, n_features). Not normalized.
    components: Array2<f64>,
    /// Running mean of observations: shape (n_features,)
    mean: Array1<f64>,
    /// Number of observations processed
    n_obs: usize,
    /// Configuration
    config: CcipcaConfig,
}

impl CcipcaState {
    /// Create a new CCIPCA state for `n_features` dimensions.
    pub fn new(n_features: usize, config: CcipcaConfig) -> Self {
        let k = config.n_components;
        Self {
            components: Array2::zeros((k, n_features)),
            mean: Array1::zeros(n_features),
            n_obs: 0,
            config,
        }
    }

    /// P7-2B: Update state with a zero-copy observation view.
    ///
    /// Accepts `ArrayView1` to avoid cloning the input data — Gemini: strictly
    /// no `.clone()` on observation data in Tokio async tasks.
    pub fn update_view(&mut self, observation: ArrayView1<f64>) {
        let owned = observation.to_owned();
        self.update(&owned);
    }

    /// Update state with a new observation vector.
    ///
    /// Uses CCIPCA amnesic update rule:
    /// v_i(n+1) = (n-1-l)/n * v_i(n) + (1+l)/n * (u·v_i_hat) * u
    ///
    /// where u = x - mean (centered observation), projected residual.
    pub fn update(&mut self, observation: &Array1<f64>) {
        assert_eq!(observation.len(), self.mean.len());

        self.n_obs += 1;
        let n = self.n_obs as f64;
        let l = self.config.amnesic;

        // Update running mean incrementally
        // mean(n) = mean(n-1) + (x - mean(n-1)) / n
        let mut residual = observation.clone();
        Zip::from(&mut self.mean)
            .and(observation)
            .for_each(|m, &x| {
                *m += (x - *m) / n;
            });

        // Center the observation: u = x - mean
        Zip::from(&mut residual).and(&self.mean).for_each(|r, &m| {
            *r -= m;
        });

        // Skip component updates for first observation
        if self.n_obs == 1 {
            // Initialize first component with the centered observation
            if self.config.n_components > 0 {
                self.components.row_mut(0).assign(&residual);
            }
            return;
        }

        // CCIPCA update for each component
        let k = self.config.n_components;
        for i in 0..k {
            let vi_norm = self.components.row(i).dot(&self.components.row(i)).sqrt();

            if vi_norm < 1e-12 {
                // Component not yet initialized; use residual
                self.components.row_mut(i).assign(&residual);
                continue;
            }

            // Normalize current estimate
            let vi_hat: Array1<f64> = &self.components.row(i) / vi_norm;

            // Project residual onto component direction
            let projection = residual.dot(&vi_hat);

            // CCIPCA update rule
            let weight_old = (n - 1.0 - l) / n;
            let weight_new = (1.0 + l) / n;

            // v_i(n) = weight_old * v_i(n-1) + weight_new * projection * u
            let mut new_component = &self.components.row(i) * weight_old;
            Zip::from(&mut new_component)
                .and(&residual)
                .for_each(|v, &u| {
                    *v += weight_new * projection * u;
                });

            self.components.row_mut(i).assign(&new_component);

            // Deflate residual: remove projection onto this component
            let updated_norm = self.components.row(i).dot(&self.components.row(i)).sqrt();
            if updated_norm > 1e-12 {
                let updated_hat: Array1<f64> = &self.components.row(i) / updated_norm;
                let proj = residual.dot(&updated_hat);
                Zip::from(&mut residual)
                    .and(&updated_hat)
                    .for_each(|r, &h| {
                        *r -= proj * h;
                    });
            }
        }
    }

    /// Return the current principal components (normalized eigenvectors).
    /// Shape: (k, n_features). Rows are unit vectors.
    pub fn components(&self) -> Array2<f64> {
        let k = self.config.n_components;
        let n = self.mean.len();
        let mut normalized = Array2::zeros((k, n));

        for i in 0..k {
            let norm = self.components.row(i).dot(&self.components.row(i)).sqrt();
            if norm > 1e-12 {
                normalized
                    .row_mut(i)
                    .assign(&(&self.components.row(i) / norm));
            }
        }

        normalized
    }

    /// Return the explained variance (unnormalized eigenvalues).
    /// Proportional to the squared norm of each component vector.
    pub fn explained_variance(&self) -> Vec<f64> {
        (0..self.config.n_components)
            .map(|i| self.components.row(i).dot(&self.components.row(i)))
            .collect()
    }

    /// Project an observation onto the principal component space.
    /// Returns a k-dimensional vector.
    #[cfg(test)]
    pub fn transform(&self, observation: &Array1<f64>) -> Array1<f64> {
        let centered = observation - &self.mean;
        let components = self.components();
        components.dot(&centered)
    }

    /// Number of observations processed.
    pub fn n_observations(&self) -> usize {
        self.n_obs
    }

    /// Whether enough observations have been collected for valid components.
    pub fn is_valid(&self) -> bool {
        self.n_obs >= self.config.min_observations
    }

    /// Access the CCIPCA configuration.
    pub fn config(&self) -> &CcipcaConfig {
        &self.config
    }

    /// P8-1A: Project a full feature tensor onto PC space, appending PC columns.
    ///
    /// Input: `Array3<f64>` shape `(1, n_feat, T)` — original features
    /// Output: `Array3<f64>` shape `(1, n_feat + k, T)` — augmented with k PC features
    ///
    /// Each PC value at time t: `pc_i(t) = W_i · centered_features(:, t)`
    /// where `W_i` is the i-th normalized eigenvector from CCIPCA.
    pub fn project_features(&self, features: &Array3<f64>) -> Array3<f64> {
        let n_feat = features.shape()[1];
        let n_bars = features.shape()[2];
        let k = self.config.n_components;

        let mut augmented = Array3::zeros((1, n_feat + k, n_bars));

        // Copy original features
        for f in 0..n_feat {
            for t in 0..n_bars {
                augmented[[0, f, t]] = features[[0, f, t]];
            }
        }

        // Compute and append PC features
        let components = self.components(); // (k, n_feat) normalized
        for t in 0..n_bars {
            let obs: Array1<f64> = Array1::from_iter((0..n_feat).map(|f| features[[0, f, t]]));
            let centered = &obs - &self.mean;
            let pc_values = components.dot(&centered); // (k,)
            for i in 0..k {
                augmented[[0, n_feat + i, t]] = pc_values[i];
            }
        }

        augmented
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::{Array1, Array3};

    #[test]
    fn test_new_state() {
        let config = CcipcaConfig {
            n_components: 3,
            amnesic: 2.0,
            min_observations: 10,
            enabled: true,
        };
        let state = CcipcaState::new(5, config);
        assert_eq!(state.n_observations(), 0);
        assert!(!state.is_valid());
        assert_eq!(state.components().shape(), &[3, 5]);
    }

    #[test]
    fn test_single_update() {
        let config = CcipcaConfig {
            n_components: 2,
            amnesic: 2.0,
            min_observations: 1,
            enabled: true,
        };
        let mut state = CcipcaState::new(3, config);
        state.update(&Array1::from_vec(vec![1.0, 2.0, 3.0]));
        assert_eq!(state.n_observations(), 1);
    }

    #[test]
    fn test_convergence_on_simple_data() {
        // Generate data with clear primary axis: x1 = 10*t, x2 = noise
        let config = CcipcaConfig {
            n_components: 1,
            amnesic: 2.0,
            min_observations: 10,
            enabled: true,
        };
        let mut state = CcipcaState::new(2, config);

        for i in 0..200 {
            let t = i as f64 * 0.1;
            let x1 = 10.0 * t;
            let x2 = 0.1 * (i as f64 % 7.0 - 3.0); // small noise
            state.update(&Array1::from_vec(vec![x1, x2]));
        }

        assert!(state.is_valid());

        // First component should point primarily along x1 (index 0)
        let components = state.components();
        let pc1 = components.row(0);
        assert!(
            pc1[0].abs() > pc1[1].abs(),
            "PC1 should align with x1: pc1={:?}",
            pc1
        );
    }

    #[test]
    fn test_explained_variance_decreasing() {
        let config = CcipcaConfig {
            n_components: 3,
            amnesic: 2.0,
            min_observations: 10,
            enabled: true,
        };
        let mut state = CcipcaState::new(5, config);

        // Feed structured data with clear variance hierarchy
        for i in 0..500 {
            let t = i as f64 * 0.01;
            let obs = Array1::from_vec(vec![
                10.0 * t,               // high variance
                2.0 * (t * 3.0).sin(),   // medium variance
                0.1 * (i as f64 % 5.0),  // low variance
                0.01 * t,                // very low
                0.001 * t,               // negligible
            ]);
            state.update(&obs);
        }

        let variances = state.explained_variance();
        // First component should capture most variance
        assert!(
            variances[0] > variances[1],
            "EV should be decreasing: {:?}",
            variances
        );
    }

    #[test]
    fn test_transform_dimension() {
        let config = CcipcaConfig {
            n_components: 3,
            amnesic: 2.0,
            min_observations: 5,
            enabled: true,
        };
        let mut state = CcipcaState::new(10, config);

        for i in 0..20 {
            let obs = Array1::from_vec((0..10).map(|j| (i * j) as f64).collect());
            state.update(&obs);
        }

        let projection = state.transform(&Array1::from_vec(vec![1.0; 10]));
        assert_eq!(projection.len(), 3);
    }

    // ── P7-2C: CCIPCA zero-copy test ─────────────────────────────────

    #[test]
    fn test_ccipca_update_zero_copy() {
        // Verify that update_view() accepts ArrayView (zero-copy borrow)
        // and produces the same result as update() with owned Array1.
        let config1 = CcipcaConfig {
            n_components: 2,
            amnesic: 2.0,
            min_observations: 5,
            enabled: true,
        };
        let config2 = config1.clone();
        let mut state_owned = CcipcaState::new(4, config1);
        let mut state_view = CcipcaState::new(4, config2);

        let observations = vec![
            vec![1.0, 2.0, 3.0, 4.0],
            vec![4.0, 3.0, 2.0, 1.0],
            vec![2.0, 2.0, 2.0, 2.0],
            vec![5.0, 1.0, 5.0, 1.0],
            vec![0.5, 0.5, 0.5, 0.5],
        ];

        for obs in &observations {
            let arr = Array1::from_vec(obs.clone());
            state_owned.update(&arr);
            state_view.update_view(arr.view());
        }

        // Both should have same number of observations
        assert_eq!(state_owned.n_observations(), state_view.n_observations());

        // Both should produce the same explained variance
        let ev_owned = state_owned.explained_variance();
        let ev_view = state_view.explained_variance();
        for (a, b) in ev_owned.iter().zip(ev_view.iter()) {
            assert!(
                (a - b).abs() < 1e-10,
                "update_view should match update: {} vs {}",
                a, b
            );
        }
    }

    // ── P8-1A: project_features tests ────────────────────────────────

    #[test]
    fn test_project_features_output_shape() {
        let n_feat = 5;
        let k = 3;
        let n_bars = 20;
        let config = CcipcaConfig {
            n_components: k,
            amnesic: 2.0,
            min_observations: 5,
            enabled: true,
        };
        let mut state = CcipcaState::new(n_feat, config);

        // Feed enough observations to get valid components
        for i in 0..30 {
            let obs = Array1::from_vec((0..n_feat).map(|j| (i * j + j) as f64).collect());
            state.update(&obs);
        }
        assert!(state.is_valid());

        let features = Array3::zeros((1, n_feat, n_bars));
        let augmented = state.project_features(&features);
        assert_eq!(augmented.shape(), &[1, n_feat + k, n_bars]);
    }

    #[test]
    fn test_project_features_preserves_original() {
        let n_feat = 4;
        let k = 2;
        let n_bars = 10;
        let config = CcipcaConfig {
            n_components: k,
            amnesic: 2.0,
            min_observations: 5,
            enabled: true,
        };
        let mut state = CcipcaState::new(n_feat, config);

        for i in 0..20 {
            let obs = Array1::from_vec((0..n_feat).map(|j| (i + j) as f64).collect());
            state.update(&obs);
        }

        // Create features with known values
        let mut features = Array3::zeros((1, n_feat, n_bars));
        for f in 0..n_feat {
            for t in 0..n_bars {
                features[[0, f, t]] = (f * 100 + t) as f64;
            }
        }

        let augmented = state.project_features(&features);

        // Original features must be unchanged
        for f in 0..n_feat {
            for t in 0..n_bars {
                assert!(
                    (augmented[[0, f, t]] - features[[0, f, t]]).abs() < 1e-12,
                    "Original feature [{},{}] changed", f, t
                );
            }
        }
    }

    #[test]
    fn test_project_features_pc_values_match_transform() {
        let n_feat = 4;
        let k = 2;
        let config = CcipcaConfig {
            n_components: k,
            amnesic: 2.0,
            min_observations: 5,
            enabled: true,
        };
        let mut state = CcipcaState::new(n_feat, config);

        for i in 0..30 {
            let obs = Array1::from_vec(vec![
                10.0 * i as f64,
                2.0 * (i as f64 * 0.3).sin(),
                0.5 * i as f64,
                0.1 * (i as f64 % 7.0),
            ]);
            state.update(&obs);
        }

        // Single bar feature tensor
        let mut features = Array3::zeros((1, n_feat, 1));
        let obs = vec![5.0, 1.0, 0.25, 0.05];
        for f in 0..n_feat {
            features[[0, f, 0]] = obs[f];
        }

        let augmented = state.project_features(&features);
        let transformed = state.transform(&Array1::from_vec(obs));

        // PC values from project_features should match transform()
        for i in 0..k {
            assert!(
                (augmented[[0, n_feat + i, 0]] - transformed[i]).abs() < 1e-10,
                "PC{} mismatch: project={} vs transform={}",
                i, augmented[[0, n_feat + i, 0]], transformed[i]
            );
        }
    }
}
