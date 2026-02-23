# strategy-engine - FOLDER_INDEX

> Real-time strategy execution service. Subscribes to market data via Redis Pub/Sub, evaluates promoted strategies using the backtest-engine VM, and publishes trade signals.

## Module Map

```
src/
  main.rs                  # Entry point: Redis sub, strategy loading, signal dispatch
  lib.rs                   # Crate re-exports
  event_bus.rs             # Redis Pub/Sub event bus (subscribe market data, publish signals)
  market_data_manager.rs   # Receives live OHLCV, maintains rolling windows, computes factors
  signal.rs                # Signal types (Buy/Sell/Hold) with metadata
  signal_buffer.rs         # Buffers signals to prevent duplicate/conflicting orders
  portfolio.rs             # Portfolio state tracking (positions, P&L)
  risk.rs                  # Real-time risk checks (max position size, daily loss limits)
  health.rs                # /health endpoint
  metrics.rs               # Prometheus metrics (signal counts, latency)
```

## Data Flow

```
Redis Pub/Sub (market data) → market_data_manager → factor computation (backtest-engine)
  → VM strategy evaluation → signal generation → signal_buffer → risk checks
  → Redis Pub/Sub (trade signals) → execution-engine
```

## Dependencies
- `common` (workspace crate)
- `backtest-engine` (factor computation + VM)
- Redis (Pub/Sub), TimescaleDB (strategy configs, positions)
