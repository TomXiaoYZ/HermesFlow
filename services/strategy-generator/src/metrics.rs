//! P10C: Prometheus metrics for strategy-generator evolution observability.
//!
//! Exposes evolution-quality metrics via a `/metrics` HTTP endpoint (port 8084).
//! Labels: exchange, symbol, mode.

use lazy_static::lazy_static;
use prometheus::{
    Gauge, GaugeVec, IntCounter, IntCounterVec, IntGauge, Opts, Registry, TextEncoder,
};

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();

    /// OOS valid rate (fraction of genomes passing OOS evaluation).
    pub static ref OOS_VALID_RATE: GaugeVec = GaugeVec::new(
        Opts::new("strategy_oos_valid_rate", "OOS valid rate per symbol"),
        &["exchange", "symbol", "mode"],
    )
    .expect("metric");

    /// IS-OOS PSR gap (mean IS PSR minus mean OOS PSR).
    pub static ref IS_OOS_GAP: GaugeVec = GaugeVec::new(
        Opts::new("strategy_is_oos_gap", "IS minus OOS PSR gap"),
        &["exchange", "symbol", "mode"],
    )
    .expect("metric");

    /// Too-few-trades rate (fraction of genomes hitting TFT sentinel).
    pub static ref TFT_RATE: GaugeVec = GaugeVec::new(
        Opts::new("strategy_tft_rate", "Too-few-trades rate per symbol"),
        &["exchange", "symbol", "mode"],
    )
    .expect("metric");

    /// Current evolution generation counter.
    pub static ref GENERATION: GaugeVec = GaugeVec::new(
        Opts::new("strategy_generation", "Current evolution generation"),
        &["exchange", "symbol", "mode"],
    )
    .expect("metric");

    /// Cumulative diversity trigger count.
    pub static ref DIVERSITY_TRIGGERS: IntCounterVec = IntCounterVec::new(
        Opts::new(
            "strategy_diversity_triggers_total",
            "Diversity trigger activations"
        ),
        &["exchange", "symbol", "mode"],
    )
    .expect("metric");

    /// Cumulative LLM oracle invocation count.
    pub static ref ORACLE_INVOCATIONS: IntCounterVec = IntCounterVec::new(
        Opts::new(
            "strategy_oracle_invocations_total",
            "LLM oracle invocations"
        ),
        &["exchange", "symbol", "mode"],
    )
    .expect("metric");

    /// Best OOS PSR in current population.
    pub static ref BEST_OOS_PSR: GaugeVec = GaugeVec::new(
        Opts::new("strategy_best_oos_psr", "Best OOS PSR in population"),
        &["exchange", "symbol", "mode"],
    )
    .expect("metric");

    /// SubformulaArchive total entry count.
    pub static ref ARCHIVE_SIZE: IntGauge = IntGauge::new(
        "strategy_archive_size",
        "SubformulaArchive total entries",
    )
    .expect("metric");

    /// MCTS seeds injected per round.
    pub static ref MCTS_SEEDS_INJECTED: IntCounterVec = IntCounterVec::new(
        Opts::new(
            "strategy_mcts_seeds_injected_total",
            "MCTS seeds injected into ALPS"
        ),
        &["exchange", "symbol", "mode"],
    )
    .expect("metric");

    /// Evolution loop duration in seconds.
    pub static ref GENERATION_DURATION: GaugeVec = GaugeVec::new(
        Opts::new(
            "strategy_generation_duration_seconds",
            "Time per generation in seconds"
        ),
        &["exchange", "symbol", "mode"],
    )
    .expect("metric");

    /// Process uptime gauge (set once at startup).
    pub static ref UPTIME: Gauge = Gauge::new(
        "strategy_uptime_seconds",
        "Process uptime in seconds",
    )
    .expect("metric");

    /// Total active evolution loops.
    pub static ref ACTIVE_LOOPS: IntGauge = IntGauge::new(
        "strategy_active_loops",
        "Number of active evolution loops",
    )
    .expect("metric");

    /// Counter for completed generations (total across all loops).
    pub static ref TOTAL_GENERATIONS: IntCounter = IntCounter::new(
        "strategy_total_generations",
        "Total generations completed across all loops",
    )
    .expect("metric");
}

/// Register all metrics with the custom registry.
pub fn register_metrics() {
    let collectors: Vec<Box<dyn prometheus::core::Collector>> = vec![
        Box::new(OOS_VALID_RATE.clone()),
        Box::new(IS_OOS_GAP.clone()),
        Box::new(TFT_RATE.clone()),
        Box::new(GENERATION.clone()),
        Box::new(DIVERSITY_TRIGGERS.clone()),
        Box::new(ORACLE_INVOCATIONS.clone()),
        Box::new(BEST_OOS_PSR.clone()),
        Box::new(ARCHIVE_SIZE.clone()),
        Box::new(MCTS_SEEDS_INJECTED.clone()),
        Box::new(GENERATION_DURATION.clone()),
        Box::new(UPTIME.clone()),
        Box::new(ACTIVE_LOOPS.clone()),
        Box::new(TOTAL_GENERATIONS.clone()),
    ];

    for c in collectors {
        REGISTRY.register(c).expect("metric registration failed");
    }
}

/// Encode all registered metrics as Prometheus text format.
pub fn gather_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    encoder
        .encode_to_string(&metric_families)
        .unwrap_or_default()
}
