# CLAUDE.md - HermesFlow

## Project Overview

HermesFlow is a quantitative trading platform built with a hybrid Rust + Python microservices architecture. It handles multi-exchange market data aggregation (crypto + traditional stocks), strategy backtesting, real-time risk management, and trade execution (Solana/Raydium/Jupiter focus).

## Tech Stack

- **Rust** (Tokio/Axum/SQLx): Data engine, API gateway, execution engine, strategy engine
- **Python 3.11** (FastAPI): Futu bridge (HK stocks)
- **Next.js 16** (React 19 + TypeScript + Tailwind CSS 4): Frontend dashboard
- **Java** (Spring Boot 3.2): User management / RBAC
- **Databases**: TimescaleDB (time-series), Redis (cache/pub-sub), ClickHouse (OLAP)

## Common Commands

```bash
make setup        # Verify Rust + install frontend deps
make lint         # Clippy (Rust), Next.js build check
make test         # cargo test (Rust)
make build        # Docker compose build all images
make up           # Start all services locally
make down         # Stop all services
make web-dev      # Start Next.js dev server (port 3000)
make web-build    # Build frontend for production
make clean        # Full cleanup
```

## Project Structure

```
services/
  data-engine/         # [Rust] Market data aggregation
    collectors/        # 12+ data source connectors (Binance, OKX, Bybit, Polygon, Jupiter, Birdeye, etc.)
    monitoring/        # Data quality (7-stage), health checks, Prometheus metrics
    tasks/             # Candle aggregation (1m/5m/15m/1h/4h/1d/1w), scheduling
    repository/        # TimescaleDB persistence (snapshots, candles, predictions)
    storage/           # Redis cache + pub/sub, ClickHouse client
    server/            # Axum HTTP + WebSocket handlers
  gateway/             # [Rust] API gateway + WebSocket router (port 8080)
  execution-engine/    # [Rust] Trade execution (Raydium, IBKR, Futu) - excluded from workspace (Solana SDK tokio conflict)
  strategy-engine/     # [Rust] Real-time strategy execution, event-driven signals via Redis Pub/Sub
  strategy-generator/  # [Rust] Genetic algorithm strategy evolution + backtest fitness
  backtest-engine/     # [Rust] Factor computation (ATR, MACD, Bollinger, etc.) + VM-based strategy execution
  common/              # [Rust] Shared types, event bus, health endpoints
  user-management/     # [Java] RBAC, multi-tenancy (port 8086)
  futu-bridge/         # [Python] HTTP bridge to Futu OpenD (HK stocks, port 8088)
  web/                 # [Next.js] Dashboard UI (port 3000)
infrastructure/
  database/postgres/migrations/   # TimescaleDB DDL migrations
  database/clickhouse/migrations/ # ClickHouse DDL migrations
  terraform/                      # Azure IaC (AKS, ACR, DB, networking)
  vector/                         # Log aggregation pipeline -> ClickHouse
docs/
  ARCHITECTURE.md      # System design & ADRs
  STANDARDS.md         # Engineering standards (authoritative)
  CODE_CONVENTIONS.md  # Error handling, config patterns
config/
  factors.yaml         # Factor definitions for crypto backtest/strategy engines
  factors-stock.yaml   # Factor definitions for stock (Polygon) evolution
  generator.yaml       # Strategy generator config (population, resolution, lookback, symbols)
```

## Architecture Patterns

### Rust Services
- **Repository Pattern**: Traits in `src/repository/mod.rs`, implementations in `src/repository/postgres/`. Inject via `Arc<dyn Trait>` in `main.rs`.
- **Error Handling**: `thiserror` derive macros. Two-tier: `DataError` (low-level) + `DataEngineError` (service-level). Use `retry_with_backoff()` for resilience.
- **Config**: 12-Factor. Priority: env vars > `config/prod.toml` > `config/default.toml`. Naming: `{SERVICE_NAME}__{SECTION}__{KEY}`.
- **execution-engine** is excluded from the Cargo workspace because Solana SDK pins `tokio ~1.14`, conflicting with workspace `tokio 1.35+`. Build and test it separately.
- **Data Quality**: 7-stage monitoring pipeline (active count, freshness, gap detection, liquidity guard, price spike, cross-source divergence, volume anomaly, timestamp drift). Tiered scheduling: critical (30s), warning (5min), full audit (1h). See `services/data-engine/src/monitoring/quality.rs`.
- **StandardMarketData**: Unified data type for all 12+ data sources, using `rust_decimal::Decimal` for financial precision. All collectors normalize into this struct before persistence/broadcast. See `services/data-engine/src/models/market_data.rs`.

### Strategy Generator
- **ALPS** (Age-Layered Population Structure): 5 Fibonacci-aged layers (5/13/34/89/500), 100 genomes each. Replaces flat GA stagnation-restart.
- **PSR fitness**: Probabilistic Sharpe Ratio (Bailey & Lopez de Prado, 2012) replaces raw PnL. Both IS and OOS evaluation use PSR z-scores.
- **Operator pruning**: 14 of 23 VM opcodes used for new genomes; VM retains all 23 for backward compatibility.
- **Embargo**: Resolution-aware gaps at K-fold boundaries (20/10/8 bars for 1d/1h/15m).
- **Dual-mode**: Long Only + Long Short evolution per (exchange, symbol).

### IBKR Dual-Gateway
- Two IB Gateway containers: `ib-gateway` (long_only account) and `ib-gateway-ls` (long_short account).
- Execution engine creates two `IBKRTrader` instances, one per gateway.
- Per-account financial data via `get_account_summaries()` cached to DB every 30s.

### Docker
- All builds use **root directory** as build context (`context: .` in compose).
- Dockerfiles live in `services/xxx/Dockerfile`.

## Strict Rules

1. **No shell scripts.** All build/automation logic goes in `Makefile` or `.github/workflows/`.
2. **No hardcoded secrets.** Use `.env` locally, Secret Store in prod.
3. **No `node_modules` or `.venv` in Git.**
4. **SQL migrations** go in `infrastructure/database/{postgres,clickhouse}/migrations/` as numbered `.sql` files.
5. **Prefer updating existing docs** (especially `ARCHITECTURE.md`) over creating new markdown files. ADRs go in `ARCHITECTURE.md` appendix.
6. **Commit after completing work.** Every time a task or feature is completed, organize changes and commit with a clear message. Do not leave completed work uncommitted.
7. **Local deploy verification per module.** After modifying any service, must: build Docker image → start service → verify health endpoint → smoke test changed functionality → commit → push. Never skip to the next module without verifying the current one runs correctly in Docker.
