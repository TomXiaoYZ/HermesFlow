use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::runtime::Tokio;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize tracing with OpenTelemetry OTLP export if `OTEL_EXPORTER_OTLP_ENDPOINT` is set.
///
/// When OTel is configured, this sets up a combined subscriber with:
///   - fmt layer (structured logs)
///   - OpenTelemetry layer (OTLP traces to Jaeger/Tempo)
///   - env-filter (respects RUST_LOG)
///
/// Returns `true` if the subscriber was initialized (caller should NOT init tracing again),
/// or `false` if OTEL is not configured (caller should set up its own subscriber).
pub fn try_init_telemetry(service_name: &str) -> bool {
    let otlp_endpoint = match std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
        Ok(ep) if !ep.is_empty() => ep,
        _ => return false,
    };

    match init_otel_subscriber(service_name, &otlp_endpoint) {
        Ok(()) => {
            tracing::info!(
                service = service_name,
                endpoint = %otlp_endpoint,
                "OpenTelemetry tracing initialized"
            );
            true
        }
        Err(e) => {
            eprintln!(
                "Failed to initialize OpenTelemetry for {}: {}. Falling back to basic tracing.",
                service_name, e
            );
            false
        }
    }
}

fn init_otel_subscriber(
    service_name: &str,
    otlp_endpoint: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(otlp_endpoint);

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(opentelemetry_sdk::trace::Config::default().with_resource(
            opentelemetry_sdk::Resource::new(vec![opentelemetry::KeyValue::new(
                "service.name",
                service_name.to_string(),
            )]),
        ))
        .install_batch(Tokio)?;

    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    let fmt_layer = tracing_subscriber::fmt::layer();
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .with(otel_layer)
        .init();

    Ok(())
}

/// Gracefully shut down the OpenTelemetry pipeline, flushing any pending spans.
pub fn shutdown_telemetry() {
    opentelemetry::global::shutdown_tracer_provider();
}
