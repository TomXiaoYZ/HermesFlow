use lazy_static::lazy_static;
use prometheus::{
    Counter, CounterVec, Encoder, Histogram, HistogramVec, IntGauge, Opts, Registry, TextEncoder,
};

lazy_static! {
    /// Registry for Prometheus metrics
    pub static ref REGISTRY: Registry = Registry::new();

    /// Total messages received from data sources
    pub static ref MESSAGES_RECEIVED: Counter = Counter::new(
        "data_engine_messages_received_total",
        "Total messages received from data sources"
    ).expect("Failed to create MESSAGES_RECEIVED counter");

    /// Total messages successfully processed
    pub static ref MESSAGES_PROCESSED: Counter = Counter::new(
        "data_engine_messages_processed_total",
        "Total messages successfully processed"
    ).expect("Failed to create MESSAGES_PROCESSED counter");

    /// Total errors encountered
    pub static ref ERRORS_TOTAL: Counter = Counter::new(
        "data_engine_errors_total",
        "Total errors encountered"
    ).expect("Failed to create ERRORS_TOTAL counter");

    /// Message parse latency histogram
    pub static ref PARSE_LATENCY: Histogram = Histogram::with_opts(
        prometheus::HistogramOpts::new(
            "data_engine_parse_latency_seconds",
            "Message parse latency in seconds"
        )
        .buckets(vec![0.00001, 0.00005, 0.0001, 0.0005, 0.001, 0.005, 0.01])
    ).expect("Failed to create PARSE_LATENCY histogram");

    /// Redis operation latency histogram
    pub static ref REDIS_LATENCY: Histogram = Histogram::with_opts(
        prometheus::HistogramOpts::new(
            "data_engine_redis_latency_seconds",
            "Redis operation latency in seconds"
        )
        .buckets(vec![0.001, 0.002, 0.005, 0.01, 0.025, 0.05, 0.1])
    ).expect("Failed to create REDIS_LATENCY histogram");

    /// ClickHouse operation latency histogram
    pub static ref CLICKHOUSE_LATENCY: Histogram = Histogram::with_opts(
        prometheus::HistogramOpts::new(
            "data_engine_clickhouse_latency_seconds",
            "ClickHouse operation latency in seconds"
        )
        .buckets(vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0])
    ).expect("Failed to create CLICKHOUSE_LATENCY histogram");

    /// ClickHouse inserts counter
    pub static ref CLICKHOUSE_INSERTS: Counter = Counter::new(
        "data_engine_clickhouse_inserts_total",
        "Total rows inserted into ClickHouse"
    ).expect("Failed to create CLICKHOUSE_INSERTS counter");

    /// Service up gauge (1 = up, 0 = down)
    pub static ref SERVICE_UP: IntGauge = IntGauge::new(
        "data_engine_service_up",
        "Service up status (1 = up, 0 = down)"
    ).expect("Failed to create SERVICE_UP gauge");

    /// Number of active symbols
    pub static ref ACTIVE_SYMBOLS_COUNT: IntGauge = IntGauge::new(
        "data_engine_active_symbols_count",
        "Number of active symbols tracked"
    ).expect("Failed to create ACTIVE_SYMBOLS_COUNT gauge");

    /// Data Quality: Stale Symbols Count
    pub static ref DQ_STALE_SYMBOLS: IntGauge = IntGauge::new(
        "data_engine_dq_stale_symbols",
        "Number of symbols with stale data"
    ).expect("Failed to create DQ_STALE_SYMBOLS gauge");

    /// Data Quality: Gap Symbols Count
    pub static ref DQ_GAP_SYMBOLS: IntGauge = IntGauge::new(
        "data_engine_dq_gap_symbols",
        "Number of symbols with missing candles"
    ).expect("Failed to create DQ_GAP_SYMBOLS gauge");

    /// Data Quality: Low Liquidity Symbols Count
    pub static ref DQ_LOW_LIQ_SYMBOLS: IntGauge = IntGauge::new(
        "data_engine_dq_low_liq_symbols",
        "Number of symbols with low liquidity"
    ).expect("Failed to create DQ_LOW_LIQ_SYMBOLS gauge");

    /// Data Quality: Price Spike Symbols Count
    pub static ref DQ_SPIKE_SYMBOLS: IntGauge = IntGauge::new(
        "data_engine_dq_spike_symbols",
        "Number of symbols with price spikes detected"
    ).expect("Failed to create DQ_SPIKE_SYMBOLS gauge");

    /// BirdEye API Request Counter
    pub static ref BIRDEYE_API_REQUESTS_TOTAL: Counter = Counter::new(
        "data_engine_birdeye_requests_total",
        "Total requests made to BirdEye API"
    ).expect("Failed to create BIRDEYE_API_REQUESTS_TOTAL counter");

    /// End-to-end ingest latency (source timestamp to DB write)
    pub static ref INGEST_LATENCY: Histogram = Histogram::with_opts(
        prometheus::HistogramOpts::new(
            "data_engine_ingest_latency_seconds",
            "End-to-end data ingest latency in seconds"
        )
        .buckets(vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0])
    ).expect("Failed to create INGEST_LATENCY histogram");

    /// Validation failures counter
    pub static ref VALIDATION_FAILURES: Counter = Counter::new(
        "data_engine_validation_failures_total",
        "Total data validation failures"
    ).expect("Failed to create VALIDATION_FAILURES counter");

    /// Dead letter counter — records permanently dropped after retry exhaustion
    pub static ref DEAD_LETTER_TOTAL: Counter = Counter::new(
        "data_engine_dead_letter_total",
        "Total records permanently dropped after retry exhaustion"
    ).expect("Failed to create DEAD_LETTER_TOTAL counter");

    /// Data Quality: Cross-source price divergence count
    pub static ref DQ_CROSS_SOURCE_DIVERGENCE: IntGauge = IntGauge::new(
        "data_engine_dq_cross_source_divergence",
        "Number of symbols with cross-source price divergence"
    ).expect("Failed to create DQ_CROSS_SOURCE_DIVERGENCE gauge");

    /// Data Quality: Volume anomaly count
    pub static ref DQ_VOLUME_ANOMALY: IntGauge = IntGauge::new(
        "data_engine_dq_volume_anomaly",
        "Number of symbols with abnormal volume"
    ).expect("Failed to create DQ_VOLUME_ANOMALY gauge");

    /// Data Quality: Timestamp drift count
    pub static ref DQ_TIMESTAMP_DRIFT: IntGauge = IntGauge::new(
        "data_engine_dq_timestamp_drift_symbols",
        "Number of symbols with excessive timestamp drift"
    ).expect("Failed to create DQ_TIMESTAMP_DRIFT gauge");

    /// Data Quality: Per-source quality score (0.0–1.0)
    pub static ref DQ_SOURCE_SCORE: prometheus::GaugeVec = prometheus::GaugeVec::new(
        prometheus::Opts::new(
            "data_engine_dq_source_score",
            "Data quality score per source (0.0-1.0)"
        ),
        &["source"]
    ).expect("Failed to create DQ_SOURCE_SCORE gauge vec");

    /// Messages received per data source (labelled by "source")
    pub static ref DATA_MESSAGES_BY_SOURCE: CounterVec = CounterVec::new(
        Opts::new(
            "data_engine_messages_by_source_total",
            "Total messages received, labelled by data source"
        ),
        &["source"]
    ).expect("Failed to create DATA_MESSAGES_BY_SOURCE counter vec");

    /// Errors per data source (labelled by "source")
    pub static ref DATA_ERRORS_BY_SOURCE: CounterVec = CounterVec::new(
        Opts::new(
            "data_engine_errors_by_source_total",
            "Total errors encountered, labelled by data source"
        ),
        &["source"]
    ).expect("Failed to create DATA_ERRORS_BY_SOURCE counter vec");

    /// Processing latency per data source (labelled by "source")
    pub static ref DATA_LATENCY_BY_SOURCE: HistogramVec = HistogramVec::new(
        prometheus::HistogramOpts::new(
            "data_engine_latency_by_source_seconds",
            "Processing latency per data source in seconds"
        )
        .buckets(vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0]),
        &["source"]
    ).expect("Failed to create DATA_LATENCY_BY_SOURCE histogram vec");

    // ── Phase 2: Timeliness & Resilience metrics ─────────────────────

    /// Circuit breaker state per source (0=closed, 1=open, 2=half_open)
    pub static ref CIRCUIT_BREAKER_STATE: prometheus::IntGaugeVec = prometheus::IntGaugeVec::new(
        prometheus::Opts::new(
            "data_engine_circuit_breaker_state",
            "Circuit breaker state per source (0=closed, 1=open, 2=half_open)"
        ),
        &["source"]
    ).expect("Failed to create CIRCUIT_BREAKER_STATE gauge vec");

    /// Circuit breaker trip counter per source
    pub static ref CIRCUIT_BREAKER_TRIPS: CounterVec = CounterVec::new(
        Opts::new(
            "data_engine_circuit_breaker_trips_total",
            "Total circuit breaker trips per source"
        ),
        &["source"]
    ).expect("Failed to create CIRCUIT_BREAKER_TRIPS counter vec");

    /// Task execution duration per task name
    pub static ref TASK_DURATION_SECONDS: HistogramVec = HistogramVec::new(
        prometheus::HistogramOpts::new(
            "data_engine_task_duration_seconds",
            "Task execution duration in seconds"
        )
        .buckets(vec![0.1, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0]),
        &["task"]
    ).expect("Failed to create TASK_DURATION_SECONDS histogram vec");

    /// Task timeout counter per task name
    pub static ref TASK_TIMEOUT_TOTAL: CounterVec = CounterVec::new(
        Opts::new(
            "data_engine_task_timeout_total",
            "Total task timeouts per task name"
        ),
        &["task"]
    ).expect("Failed to create TASK_TIMEOUT_TOTAL counter vec");

    /// Task overlap skip counter per task name
    pub static ref TASK_OVERLAP_SKIPPED: CounterVec = CounterVec::new(
        Opts::new(
            "data_engine_task_overlap_skipped_total",
            "Total task executions skipped due to overlap"
        ),
        &["task"]
    ).expect("Failed to create TASK_OVERLAP_SKIPPED counter vec");

    /// End-to-end data freshness: seconds since newest snapshot per source
    pub static ref DATA_E2E_FRESHNESS_SECONDS: prometheus::GaugeVec = prometheus::GaugeVec::new(
        prometheus::Opts::new(
            "data_engine_e2e_freshness_seconds",
            "Seconds since newest snapshot per source"
        ),
        &["source"]
    ).expect("Failed to create DATA_E2E_FRESHNESS_SECONDS gauge vec");

    /// DQ incidents recorded
    pub static ref DQ_INCIDENTS_TOTAL: CounterVec = CounterVec::new(
        Opts::new(
            "data_engine_dq_incidents_total",
            "Total data quality incidents recorded"
        ),
        &["check_type", "severity"]
    ).expect("Failed to create DQ_INCIDENTS_TOTAL counter vec");

    // ── Phase 3: Enhanced Monitoring metrics ─────────────────────────

    /// Last message timestamp per collector source (unix epoch seconds)
    pub static ref COLLECTOR_LAST_MESSAGE_TS: prometheus::GaugeVec = prometheus::GaugeVec::new(
        prometheus::Opts::new(
            "data_engine_collector_last_message_timestamp",
            "Unix epoch seconds of last message received per source"
        ),
        &["source"]
    ).expect("Failed to create COLLECTOR_LAST_MESSAGE_TS gauge vec");

    /// Redis cache hits
    pub static ref REDIS_CACHE_HITS: Counter = Counter::new(
        "data_engine_redis_cache_hits_total",
        "Total Redis cache hits"
    ).expect("Failed to create REDIS_CACHE_HITS counter");

    /// Redis cache misses
    pub static ref REDIS_CACHE_MISSES: Counter = Counter::new(
        "data_engine_redis_cache_misses_total",
        "Total Redis cache misses"
    ).expect("Failed to create REDIS_CACHE_MISSES counter");
}

/// Initializes Prometheus metrics by registering them with the registry
pub fn init_metrics() -> Result<(), prometheus::Error> {
    REGISTRY.register(Box::new(MESSAGES_RECEIVED.clone()))?;
    REGISTRY.register(Box::new(MESSAGES_PROCESSED.clone()))?;
    REGISTRY.register(Box::new(ERRORS_TOTAL.clone()))?;
    REGISTRY.register(Box::new(PARSE_LATENCY.clone()))?;
    REGISTRY.register(Box::new(REDIS_LATENCY.clone()))?;
    REGISTRY.register(Box::new(CLICKHOUSE_LATENCY.clone()))?;
    REGISTRY.register(Box::new(CLICKHOUSE_INSERTS.clone()))?;
    REGISTRY.register(Box::new(SERVICE_UP.clone()))?;
    REGISTRY.register(Box::new(ACTIVE_SYMBOLS_COUNT.clone()))?;
    REGISTRY.register(Box::new(DQ_STALE_SYMBOLS.clone()))?;
    REGISTRY.register(Box::new(DQ_GAP_SYMBOLS.clone()))?;
    REGISTRY.register(Box::new(DQ_LOW_LIQ_SYMBOLS.clone()))?;
    REGISTRY.register(Box::new(DQ_SPIKE_SYMBOLS.clone()))?;
    REGISTRY.register(Box::new(BIRDEYE_API_REQUESTS_TOTAL.clone()))?;
    REGISTRY.register(Box::new(INGEST_LATENCY.clone()))?;
    REGISTRY.register(Box::new(VALIDATION_FAILURES.clone()))?;
    REGISTRY.register(Box::new(DEAD_LETTER_TOTAL.clone()))?;
    REGISTRY.register(Box::new(DQ_CROSS_SOURCE_DIVERGENCE.clone()))?;
    REGISTRY.register(Box::new(DQ_VOLUME_ANOMALY.clone()))?;
    REGISTRY.register(Box::new(DQ_TIMESTAMP_DRIFT.clone()))?;
    REGISTRY.register(Box::new(DQ_SOURCE_SCORE.clone()))?;
    REGISTRY.register(Box::new(DATA_MESSAGES_BY_SOURCE.clone()))?;
    REGISTRY.register(Box::new(DATA_ERRORS_BY_SOURCE.clone()))?;
    REGISTRY.register(Box::new(DATA_LATENCY_BY_SOURCE.clone()))?;
    REGISTRY.register(Box::new(CIRCUIT_BREAKER_STATE.clone()))?;
    REGISTRY.register(Box::new(CIRCUIT_BREAKER_TRIPS.clone()))?;
    REGISTRY.register(Box::new(TASK_DURATION_SECONDS.clone()))?;
    REGISTRY.register(Box::new(TASK_TIMEOUT_TOTAL.clone()))?;
    REGISTRY.register(Box::new(TASK_OVERLAP_SKIPPED.clone()))?;
    REGISTRY.register(Box::new(DATA_E2E_FRESHNESS_SECONDS.clone()))?;
    REGISTRY.register(Box::new(DQ_INCIDENTS_TOTAL.clone()))?;
    REGISTRY.register(Box::new(COLLECTOR_LAST_MESSAGE_TS.clone()))?;
    REGISTRY.register(Box::new(REDIS_CACHE_HITS.clone()))?;
    REGISTRY.register(Box::new(REDIS_CACHE_MISSES.clone()))?;

    // Set service as up initially
    SERVICE_UP.set(1);

    tracing::info!("Prometheus metrics initialized");
    Ok(())
}

/// Exports metrics in Prometheus text format
pub fn export_metrics() -> Result<String, prometheus::Error> {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer)?;

    String::from_utf8(buffer)
        .map_err(|e| prometheus::Error::Msg(format!("UTF-8 conversion error: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_initialization() {
        // Metrics are initialized via lazy_static, just verify they exist
        // Note: In a test environment, metrics may already have values from other tests
        assert!(MESSAGES_RECEIVED.get() >= 0.0);
        assert!(MESSAGES_PROCESSED.get() >= 0.0);
        assert!(ERRORS_TOTAL.get() >= 0.0);
    }

    #[test]
    fn test_counter_increment() {
        // Note: In a real test environment, we'd use a separate registry
        // to avoid interference between tests
        let initial = MESSAGES_RECEIVED.get();
        MESSAGES_RECEIVED.inc();
        assert!(MESSAGES_RECEIVED.get() > initial);
    }

    #[test]
    fn test_service_up_gauge() {
        SERVICE_UP.set(1);
        assert_eq!(SERVICE_UP.get(), 1);

        SERVICE_UP.set(0);
        assert_eq!(SERVICE_UP.get(), 0);

        // Reset to up
        SERVICE_UP.set(1);
    }

    #[test]
    fn test_export_metrics() {
        // Initialize metrics first (idempotent)
        let _ = init_metrics();

        let result = export_metrics();
        assert!(result.is_ok());

        let metrics_text = result.unwrap();
        // Verify we can export metrics successfully
        assert!(!metrics_text.is_empty());
    }
}
