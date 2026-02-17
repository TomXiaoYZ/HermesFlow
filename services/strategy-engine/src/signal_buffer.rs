use std::collections::{HashMap, VecDeque};

const DEFAULT_WINDOW: usize = 200;
const MIN_SAMPLES: usize = 50;

/// Rolling buffer of sigmoid values per (symbol, mode) pair.
/// Computes adaptive thresholds matching the backtester's percentile logic.
pub struct SignalBuffer {
    buffers: HashMap<(String, String), VecDeque<f64>>,
    window_size: usize,
}

impl Default for SignalBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalBuffer {
    pub fn new() -> Self {
        Self {
            buffers: HashMap::new(),
            window_size: DEFAULT_WINDOW,
        }
    }

    /// Push a sigmoid value for (symbol, mode).
    pub fn push(&mut self, symbol: &str, mode: &str, sigmoid_val: f64) {
        let key = (symbol.to_string(), mode.to_string());
        let buf = self.buffers.entry(key).or_default();
        buf.push_back(sigmoid_val);
        if buf.len() > self.window_size {
            buf.pop_front();
        }
    }

    /// Compute adaptive upper threshold (70th percentile, clamped [0.52, 0.80]).
    /// Returns `None` when fewer than `MIN_SAMPLES` collected (warmup period).
    pub fn upper_threshold(&self, symbol: &str, mode: &str) -> Option<f64> {
        let key = (symbol.to_string(), mode.to_string());
        let buf = self.buffers.get(&key)?;
        if buf.len() < MIN_SAMPLES {
            return None;
        }
        let mut vals: Vec<f64> = buf.iter().copied().collect();
        vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let idx = ((vals.len() as f64) * 0.70) as usize;
        Some(vals[idx.min(vals.len() - 1)].clamp(0.52, 0.80))
    }

    /// Compute adaptive lower threshold (30th percentile, clamped [0.20, 0.48]).
    /// Returns `None` when fewer than `MIN_SAMPLES` collected (warmup period).
    pub fn lower_threshold(&self, symbol: &str, mode: &str) -> Option<f64> {
        let key = (symbol.to_string(), mode.to_string());
        let buf = self.buffers.get(&key)?;
        if buf.len() < MIN_SAMPLES {
            return None;
        }
        let mut vals: Vec<f64> = buf.iter().copied().collect();
        vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let idx = ((vals.len() as f64) * 0.30) as usize;
        Some(vals[idx.min(vals.len() - 1)].clamp(0.20, 0.48))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warmup_returns_none() {
        let mut buf = SignalBuffer::new();
        for i in 0..49 {
            buf.push("AAPL", "long_only", 0.5 + (i as f64) * 0.001);
        }
        assert!(buf.upper_threshold("AAPL", "long_only").is_none());
        assert!(buf.lower_threshold("AAPL", "long_only").is_none());
    }

    #[test]
    fn test_threshold_after_warmup() {
        let mut buf = SignalBuffer::new();
        // Push 100 evenly spaced values [0.01, 0.02, ..., 1.00]
        for i in 1..=100 {
            buf.push("AAPL", "long_only", i as f64 / 100.0);
        }
        let upper = buf.upper_threshold("AAPL", "long_only").unwrap();
        assert!(upper >= 0.52 && upper <= 0.80, "upper={}", upper);

        let lower = buf.lower_threshold("AAPL", "long_only").unwrap();
        assert!(lower >= 0.20 && lower <= 0.48, "lower={}", lower);
    }

    #[test]
    fn test_clamping_high() {
        let mut buf = SignalBuffer::new();
        // All values near 1.0 — 70th percentile would be ~1.0, clamped to 0.80
        for _ in 0..100 {
            buf.push("AAPL", "long_only", 0.99);
        }
        let upper = buf.upper_threshold("AAPL", "long_only").unwrap();
        assert!((upper - 0.80).abs() < 0.001, "upper={}", upper);
    }

    #[test]
    fn test_clamping_low() {
        let mut buf = SignalBuffer::new();
        // All values near 0.0 — 30th percentile would be ~0.01, clamped to 0.20
        for _ in 0..100 {
            buf.push("AAPL", "long_short", 0.01);
        }
        let lower = buf.lower_threshold("AAPL", "long_short").unwrap();
        assert!((lower - 0.20).abs() < 0.001, "lower={}", lower);
    }

    #[test]
    fn test_rolling_window_evicts_old() {
        let mut buf = SignalBuffer::new();
        // Fill 200 values of 0.5
        for _ in 0..200 {
            buf.push("AAPL", "long_only", 0.5);
        }
        // Now push 200 values of 0.9 — old 0.5 values should be evicted
        for _ in 0..200 {
            buf.push("AAPL", "long_only", 0.9);
        }
        let upper = buf.upper_threshold("AAPL", "long_only").unwrap();
        // 70th percentile of all 0.9 should clamp to 0.80
        assert!((upper - 0.80).abs() < 0.001, "upper={}", upper);
    }

    #[test]
    fn test_independent_symbol_mode_pairs() {
        let mut buf = SignalBuffer::new();
        for _ in 0..100 {
            buf.push("AAPL", "long_only", 0.7);
            buf.push("AAPL", "long_short", 0.3);
        }
        let upper_lo = buf.upper_threshold("AAPL", "long_only").unwrap();
        let lower_ls = buf.lower_threshold("AAPL", "long_short").unwrap();
        // long_only all 0.7 -> 70th pct = 0.7, in [0.52, 0.80]
        assert!((upper_lo - 0.70).abs() < 0.001, "upper_lo={}", upper_lo);
        // long_short all 0.3 -> 30th pct = 0.3, in [0.20, 0.48]
        assert!((lower_ls - 0.30).abs() < 0.001, "lower_ls={}", lower_ls);
    }
}
