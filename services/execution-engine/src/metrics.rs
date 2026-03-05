//! P10D-3: Prometheus metrics for execution engine IBKR observability.
//!
//! Tracks connection health, API timeouts, reconnection attempts, and sync
//! loop duration. Metrics are registered with the common REGISTRY and served
//! via the shared `/metrics` endpoint on port 8083.

use lazy_static::lazy_static;
use prometheus::{GaugeVec, IntCounterVec, Opts};

lazy_static! {
    /// Cumulative IBKR reconnection attempts (label: account).
    pub static ref IBKR_RECONNECT_TOTAL: IntCounterVec = IntCounterVec::new(
        Opts::new("ibkr_reconnect_total", "IBKR reconnection attempts"),
        &["account"],
    )
    .expect("metric");

    /// IBKR connection status: 1 = connected, 0 = disconnected (label: account).
    pub static ref IBKR_CONNECTED: GaugeVec = GaugeVec::new(
        Opts::new("ibkr_connected", "IBKR connection status (1=up, 0=down)"),
        &["account"],
    )
    .expect("metric");

    /// Cumulative IBKR API timeout count (labels: account, operation).
    pub static ref IBKR_API_TIMEOUT_TOTAL: IntCounterVec = IntCounterVec::new(
        Opts::new("ibkr_api_timeout_total", "IBKR API call timeouts"),
        &["account", "operation"],
    )
    .expect("metric");

    /// Duration of the last sync cycle per account in seconds.
    pub static ref IBKR_SYNC_DURATION: GaugeVec = GaugeVec::new(
        Opts::new(
            "ibkr_sync_duration_seconds",
            "Duration of IBKR portfolio sync cycle"
        ),
        &["account"],
    )
    .expect("metric");
}

/// Register all execution-engine metrics with the common Prometheus registry.
pub fn register_metrics() {
    let registry = &common::metrics::REGISTRY;
    let collectors: Vec<Box<dyn prometheus::core::Collector>> = vec![
        Box::new(IBKR_RECONNECT_TOTAL.clone()),
        Box::new(IBKR_CONNECTED.clone()),
        Box::new(IBKR_API_TIMEOUT_TOTAL.clone()),
        Box::new(IBKR_SYNC_DURATION.clone()),
    ];

    for c in collectors {
        registry.register(c).expect("metric registration failed");
    }
}
