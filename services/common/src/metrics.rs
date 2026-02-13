use axum::response::IntoResponse;
use lazy_static::lazy_static;
use prometheus::{Encoder, IntGauge, Registry, TextEncoder};

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    pub static ref SERVICE_UP: IntGauge =
        IntGauge::new("service_up", "Service up status (1 = up, 0 = down)")
            .expect("Failed to create SERVICE_UP gauge");
}

/// Initialize the common metrics registry with base service metrics.
/// Call this once at service startup before registering service-specific metrics.
pub fn init_metrics(service_name: &str) -> Result<(), prometheus::Error> {
    REGISTRY.register(Box::new(SERVICE_UP.clone()))?;
    SERVICE_UP.set(1);
    tracing::info!("{} Prometheus metrics initialized", service_name);
    Ok(())
}

/// Export all registered metrics in Prometheus text format.
pub fn export_metrics() -> Result<String, prometheus::Error> {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer)?;
    String::from_utf8(buffer)
        .map_err(|e| prometheus::Error::Msg(format!("UTF-8 conversion error: {}", e)))
}

/// Axum handler that serves Prometheus metrics at `/metrics`.
pub async fn metrics_handler() -> impl IntoResponse {
    match export_metrics() {
        Ok(metrics) => (
            axum::http::StatusCode::OK,
            [(
                axum::http::header::CONTENT_TYPE,
                "text/plain; version=0.0.4; charset=utf-8",
            )],
            metrics,
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to export metrics: {}", e),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_up_gauge() {
        SERVICE_UP.set(1);
        assert_eq!(SERVICE_UP.get(), 1);
        SERVICE_UP.set(0);
        assert_eq!(SERVICE_UP.get(), 0);
        SERVICE_UP.set(1);
    }

    #[test]
    fn test_export_metrics() {
        let _ = init_metrics("test-service");
        let result = export_metrics();
        assert!(result.is_ok());
        let text = result.unwrap();
        assert!(!text.is_empty());
    }
}
