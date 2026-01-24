use lazy_static::lazy_static;
use prometheus::{Counter, Encoder, Histogram, IntGauge, Registry, TextEncoder};

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
