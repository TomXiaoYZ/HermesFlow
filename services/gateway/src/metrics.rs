use common::metrics::REGISTRY;
use lazy_static::lazy_static;
use prometheus::{CounterVec, IntGauge, Opts};

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
}

pub fn init_gateway_metrics() -> Result<(), prometheus::Error> {
    common::metrics::init_metrics("gateway")?;
    REGISTRY.register(Box::new(HTTP_REQUESTS_TOTAL.clone()))?;
    REGISTRY.register(Box::new(WEBSOCKET_CONNECTIONS.clone()))?;
    REGISTRY.register(Box::new(PROXY_ERRORS_TOTAL.clone()))?;
    tracing::info!("Gateway-specific Prometheus metrics registered");
    Ok(())
}
