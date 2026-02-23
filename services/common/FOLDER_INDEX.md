# common - FOLDER_INDEX

> Shared Rust crate providing cross-cutting concerns: event types, health checks, heartbeat, metrics, and OpenTelemetry integration.

## Module Map

```
src/
  lib.rs           # Crate re-exports
  events.rs        # Event types for Redis Pub/Sub (MarketDataEvent, TradeSignalEvent, etc.)
  health.rs        # Standardized health check endpoint builder
  heartbeat.rs     # Service heartbeat (periodic liveness signal)
  metrics.rs       # Prometheus metric helpers (register, expose)
  telemetry.rs     # OpenTelemetry tracing setup (OTLP → Jaeger)
```

## Used By
- data-engine, gateway, strategy-engine, strategy-generator, backtest-engine
