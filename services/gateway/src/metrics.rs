use common::metrics::REGISTRY;
use lazy_static::lazy_static;
use prometheus::{CounterVec, HistogramVec, IntGauge, Opts};

lazy_static! {
    /// Total HTTP requests handled by the gateway
    pub static ref HTTP_REQUESTS_TOTAL: CounterVec = CounterVec::new(
        Opts::new(
            "gateway_http_requests_total",
            "Total HTTP requests handled by the gateway"
        ),
        &["method", "path", "status"]
    ).expect("Failed to create HTTP_REQUESTS_TOTAL counter vec");

    /// Number of active WebSocket connections
    pub static ref WEBSOCKET_CONNECTIONS: IntGauge = IntGauge::new(
        "gateway_websocket_connections",
        "Number of active WebSocket connections"
    ).expect("Failed to create WEBSOCKET_CONNECTIONS gauge");

    /// Total proxy errors by target service
    pub static ref PROXY_ERRORS_TOTAL: CounterVec = CounterVec::new(
        Opts::new(
            "gateway_proxy_errors_total",
            "Total proxy errors by target service"
        ),
        &["target"]
    ).expect("Failed to create PROXY_ERRORS_TOTAL counter vec");

    /// HTTP request duration per method, path, and status
    pub static ref HTTP_REQUEST_DURATION_SECONDS: HistogramVec = HistogramVec::new(
        prometheus::HistogramOpts::new(
            "gateway_http_request_duration_seconds",
            "HTTP request duration in seconds"
        )
        .buckets(vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
        &["method", "path", "status"]
    ).expect("Failed to create HTTP_REQUEST_DURATION_SECONDS histogram vec");

    /// Upstream service health (1=up, 0=down)
    pub static ref UPSTREAM_HEALTH: prometheus::IntGaugeVec = prometheus::IntGaugeVec::new(
        prometheus::Opts::new(
            "gateway_upstream_health",
            "Upstream service health (1=up, 0=down)"
        ),
        &["target"]
    ).expect("Failed to create UPSTREAM_HEALTH gauge vec");

    /// Upstream proxy latency per target service
    pub static ref UPSTREAM_LATENCY_SECONDS: HistogramVec = HistogramVec::new(
        prometheus::HistogramOpts::new(
            "gateway_upstream_latency_seconds",
            "Upstream proxy latency in seconds"
        )
        .buckets(vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]),
        &["target"]
    ).expect("Failed to create UPSTREAM_LATENCY_SECONDS histogram vec");
}

pub fn init_gateway_metrics() -> Result<(), prometheus::Error> {
    common::metrics::init_metrics("gateway")?;
    REGISTRY.register(Box::new(HTTP_REQUESTS_TOTAL.clone()))?;
    REGISTRY.register(Box::new(WEBSOCKET_CONNECTIONS.clone()))?;
    REGISTRY.register(Box::new(PROXY_ERRORS_TOTAL.clone()))?;
    REGISTRY.register(Box::new(HTTP_REQUEST_DURATION_SECONDS.clone()))?;
    REGISTRY.register(Box::new(UPSTREAM_HEALTH.clone()))?;
    REGISTRY.register(Box::new(UPSTREAM_LATENCY_SECONDS.clone()))?;
    tracing::info!("Gateway-specific Prometheus metrics registered");
    Ok(())
}
