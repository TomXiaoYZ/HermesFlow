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
  data-engine/         # [Rust] Market data aggregation (Binance, Polygon, IBKR, Jupiter, Polymarket, etc.)
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
  factors.yaml         # Factor definitions for backtest/strategy engines
```

## Architecture Patterns

### Rust Services
- **Repository Pattern**: Traits in `src/repository/mod.rs`, implementations in `src/repository/postgres/`. Inject via `Arc<dyn Trait>` in `main.rs`.
- **Error Handling**: `thiserror` derive macros. Two-tier: `DataError` (low-level) + `DataEngineError` (service-level). Use `retry_with_backoff()` for resilience.
- **Config**: 12-Factor. Priority: env vars > `config/prod.toml` > `config/default.toml`. Naming: `{SERVICE_NAME}__{SECTION}__{KEY}`.
- **execution-engine** is excluded from the Cargo workspace because Solana SDK pins `tokio ~1.14`, conflicting with workspace `tokio 1.35+`. Build and test it separately.

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
