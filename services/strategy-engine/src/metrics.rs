use common::metrics::REGISTRY;
use lazy_static::lazy_static;
use prometheus::{Counter, CounterVec, Histogram, IntGauge, Opts};

lazy_static! {
    /// Total signals generated, labelled by strategy and direction (buy/sell)
    pub static ref SIGNALS_GENERATED_TOTAL: CounterVec = CounterVec::new(
        Opts::new(
            "strategy_signals_generated_total",
            "Total trade signals generated"
        ),
        &["strategy", "direction"]
    ).expect("Failed to create SIGNALS_GENERATED_TOTAL counter vec");

    /// Signal generation latency (VM execution + sigmoid)
    pub static ref SIGNAL_LATENCY_SECONDS: Histogram = Histogram::with_opts(
        prometheus::HistogramOpts::new(
            "strategy_signal_latency_seconds",
            "Signal generation latency in seconds"
        )
        .buckets(vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0])
    ).expect("Failed to create SIGNAL_LATENCY_SECONDS histogram");

    /// Number of active open positions
    pub static ref ACTIVE_POSITIONS: IntGauge = IntGauge::new(
        "strategy_active_positions",
        "Number of active open positions"
    ).expect("Failed to create ACTIVE_POSITIONS gauge");

    /// Risk check outcomes (approved / rejected)
    pub static ref RISK_CHECKS_TOTAL: CounterVec = CounterVec::new(
        Opts::new(
            "strategy_risk_checks_total",
            "Total risk checks performed"
        ),
        &["result"]
    ).expect("Failed to create RISK_CHECKS_TOTAL counter vec");

    /// Total market data updates consumed
    pub static ref MARKET_DATA_CONSUMED: Counter = Counter::new(
        "strategy_market_data_consumed_total",
        "Total market data updates consumed"
    ).expect("Failed to create MARKET_DATA_CONSUMED counter");

    /// Market data consumption lag (source timestamp to processing time)
    pub static ref MARKET_DATA_LAG_SECONDS: Histogram = Histogram::with_opts(
        prometheus::HistogramOpts::new(
            "strategy_market_data_lag_seconds",
            "Market data consumption lag in seconds"
        )
        .buckets(vec![0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0, 30.0])
    ).expect("Failed to create MARKET_DATA_LAG_SECONDS histogram");
}

/// Register strategy-engine specific metrics on the shared Prometheus registry.
pub fn init_strategy_metrics() -> Result<(), prometheus::Error> {
    REGISTRY.register(Box::new(SIGNALS_GENERATED_TOTAL.clone()))?;
    REGISTRY.register(Box::new(SIGNAL_LATENCY_SECONDS.clone()))?;
    REGISTRY.register(Box::new(ACTIVE_POSITIONS.clone()))?;
    REGISTRY.register(Box::new(RISK_CHECKS_TOTAL.clone()))?;
    REGISTRY.register(Box::new(MARKET_DATA_CONSUMED.clone()))?;
    REGISTRY.register(Box::new(MARKET_DATA_LAG_SECONDS.clone()))?;
    tracing::info!("Strategy-engine Prometheus metrics registered");
    Ok(())
}
