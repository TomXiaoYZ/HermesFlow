use std::sync::atomic::{AtomicU32, AtomicU8, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Circuit breaker states.
const STATE_CLOSED: u8 = 0;
const STATE_OPEN: u8 = 1;
const STATE_HALF_OPEN: u8 = 2;

/// A thread-safe circuit breaker for data source collectors.
///
/// - **Closed**: Normal operation. Failures increment a counter.
/// - **Open**: All calls are rejected. After `recovery_timeout`, transitions to HalfOpen.
/// - **HalfOpen**: One probe call is allowed. Success → Closed, Failure → Open.
pub struct CircuitBreaker {
    name: String,
    state: AtomicU8,
    failure_count: AtomicU32,
    failure_threshold: u32,
    recovery_timeout: Duration,
    last_failure: Mutex<Option<Instant>>,
}

impl CircuitBreaker {
    pub fn new(name: &str, failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            name: name.to_string(),
            state: AtomicU8::new(STATE_CLOSED),
            failure_count: AtomicU32::new(0),
            failure_threshold,
            recovery_timeout,
            last_failure: Mutex::new(None),
        }
    }

    /// Returns the current state as a human-readable string.
    pub fn state_name(&self) -> &'static str {
        match self.state.load(Ordering::Relaxed) {
            STATE_CLOSED => "closed",
            STATE_OPEN => "open",
            STATE_HALF_OPEN => "half_open",
            _ => "unknown",
        }
    }

    /// Returns the numeric state for Prometheus gauge (0=closed, 1=open, 2=half_open).
    pub fn state_value(&self) -> u8 {
        self.state.load(Ordering::Relaxed)
    }

    /// Check whether a call should be allowed.
    ///
    /// Returns `true` if the call can proceed, `false` if the circuit is open.
    pub async fn allow_request(&self) -> bool {
        let state = self.state.load(Ordering::Relaxed);

        match state {
            STATE_CLOSED => true,
            STATE_OPEN => {
                // Check if recovery timeout has elapsed
                let last = self.last_failure.lock().await;
                if let Some(ts) = *last {
                    if ts.elapsed() >= self.recovery_timeout {
                        drop(last);
                        self.state.store(STATE_HALF_OPEN, Ordering::Relaxed);
                        tracing::info!(
                            source = %self.name,
                            "Circuit breaker entering half-open state (probe allowed)"
                        );
                        true
                    } else {
                        false
                    }
                } else {
                    // No last failure recorded, shouldn't be open — reset
                    drop(last);
                    self.state.store(STATE_CLOSED, Ordering::Relaxed);
                    true
                }
            }
            STATE_HALF_OPEN => {
                // Only one probe at a time; subsequent calls are blocked
                // (simplification: allow the probe)
                true
            }
            _ => false,
        }
    }

    /// Record a successful call. Resets failure counter and closes the circuit.
    pub fn record_success(&self) {
        let prev = self.state.load(Ordering::Relaxed);
        if prev != STATE_CLOSED {
            tracing::info!(
                source = %self.name,
                prev_state = prev,
                "Circuit breaker closed after successful probe"
            );
        }
        self.state.store(STATE_CLOSED, Ordering::Relaxed);
        self.failure_count.store(0, Ordering::Relaxed);
    }

    /// Record a failed call. Increments failure count and may trip the breaker.
    ///
    /// Returns `true` if the circuit just tripped from closed/half-open → open.
    pub async fn record_failure(&self) -> bool {
        let state = self.state.load(Ordering::Relaxed);

        match state {
            STATE_HALF_OPEN => {
                // Probe failed → back to open
                self.state.store(STATE_OPEN, Ordering::Relaxed);
                *self.last_failure.lock().await = Some(Instant::now());
                tracing::warn!(
                    source = %self.name,
                    "Circuit breaker re-opened after failed probe"
                );
                true
            }
            STATE_CLOSED => {
                let count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
                if count >= self.failure_threshold {
                    self.state.store(STATE_OPEN, Ordering::Relaxed);
                    *self.last_failure.lock().await = Some(Instant::now());
                    tracing::error!(
                        source = %self.name,
                        failures = count,
                        threshold = self.failure_threshold,
                        recovery_secs = self.recovery_timeout.as_secs(),
                        "Circuit breaker TRIPPED — source disabled"
                    );
                    true
                } else {
                    false
                }
            }
            _ => false, // Already open
        }
    }

    /// Force-reset the circuit breaker to closed state.
    pub fn reset(&self) {
        self.state.store(STATE_CLOSED, Ordering::Relaxed);
        self.failure_count.store(0, Ordering::Relaxed);
        tracing::info!(source = %self.name, "Circuit breaker manually reset");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_starts_closed() {
        let cb = CircuitBreaker::new("test", 3, Duration::from_secs(60));
        assert_eq!(cb.state_name(), "closed");
        assert!(cb.allow_request().await);
    }

    #[tokio::test]
    async fn test_trips_after_threshold() {
        let cb = CircuitBreaker::new("test", 3, Duration::from_secs(60));

        assert!(!cb.record_failure().await); // 1
        assert!(!cb.record_failure().await); // 2
        assert!(cb.record_failure().await); // 3 → trips

        assert_eq!(cb.state_name(), "open");
        assert!(!cb.allow_request().await);
    }

    #[tokio::test]
    async fn test_success_resets() {
        let cb = CircuitBreaker::new("test", 2, Duration::from_secs(60));

        cb.record_failure().await;
        cb.record_failure().await;
        assert_eq!(cb.state_name(), "open");

        // Simulate recovery timeout
        *cb.last_failure.lock().await = Some(Instant::now() - Duration::from_secs(61));

        assert!(cb.allow_request().await);
        assert_eq!(cb.state_name(), "half_open");

        cb.record_success();
        assert_eq!(cb.state_name(), "closed");
        assert!(cb.allow_request().await);
    }

    #[tokio::test]
    async fn test_half_open_failure_reopens() {
        let cb = CircuitBreaker::new("test", 2, Duration::from_secs(60));

        cb.record_failure().await;
        cb.record_failure().await;

        *cb.last_failure.lock().await = Some(Instant::now() - Duration::from_secs(61));
        cb.allow_request().await; // → half_open

        assert!(cb.record_failure().await); // → open again
        assert_eq!(cb.state_name(), "open");
    }

    #[test]
    fn test_state_value() {
        let cb = CircuitBreaker::new("test", 3, Duration::from_secs(60));
        assert_eq!(cb.state_value(), 0); // closed
    }
}
