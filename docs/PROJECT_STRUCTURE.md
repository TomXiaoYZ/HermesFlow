# HermesFlow Project Structure

## Root Directory

```
HermesFlow/
├── Cargo.toml                 # Rust workspace definition (6 members, 1 excluded)
├── Cargo.lock                 # Pinned dependency versions
├── docker-compose.yml         # Local development orchestration (all services + infra)
├── docker-compose.prod.yml    # Production overrides
├── Makefile                   # Unified build/dev commands
├── rustfmt.toml               # Rust formatting configuration
├── LICENSE
├── README.md
│
├── config/                    # Shared runtime configuration
│   └── factors.yaml           # Factor definitions for backtest/strategy engines
│
├── services/                  # All microservices
├── infrastructure/            # Database schemas, migrations, IaC, logging
├── docs/                      # Project documentation
└── scripts/                   # Utility scripts (legacy, not for build logic)
```

## Services

| Service | Language | Port | Responsibility | Key Dependencies |
|---------|----------|------|---------------|-----------------|
| **common** | Rust (lib) | -- | Shared types, event bus, health server, heartbeat | tokio, axum, redis, serde |
| **backtest-engine** | Rust (lib) | -- | Factor computation (ATR, MACD, Bollinger, etc.), VM-based strategy execution | ndarray, common |
| **data-engine** | Rust | 8081 ext / 8080 int | Market data ingestion from Birdeye, Jupiter, Helius, Polygon; candle aggregation; historical sync | actix-web, sqlx, redis, clickhouse, common |
| **gateway** | Rust | 8080 | API gateway, JWT auth, rate limiting, WebSocket proxy, health aggregation | actix-web, sqlx, redis, clickhouse |
| **strategy-engine** | Rust | 8082 (health only) | Real-time strategy execution, event-driven signal generation, risk management | tokio, redis, common, backtest-engine |
| **strategy-generator** | Rust | 8082 ext / 8084 health | Genetic algorithm evolution of trading strategies, fitness evaluation | actix-web, sqlx, redis, common, backtest-engine |
| **execution-engine** | Rust | 8083 (health only) | Trade execution across Raydium (Solana DEX), IBKR (US equities), Futu (HK stocks) | solana-sdk, tokio, redis, reqwest |
| **user-management** | Java | 8086 | User authentication, tenant management, Spring Security | Spring Boot, JPA, PostgreSQL |
| **futu-bridge** | Python | 8088 | HTTP bridge to Futu OpenD for HK stock trading | Flask, futu-api |
| **web** | TypeScript | 3000 | Dashboard, strategy lab, data discovery, market overview | Next.js, React, WebSocket |

## Cargo Workspace

The root `Cargo.toml` defines a Rust workspace with 6 members and 1 excluded crate:

**Members** (share workspace dependencies):
- `services/common`
- `services/backtest-engine`
- `services/data-engine`
- `services/gateway`
- `services/strategy-engine`
- `services/strategy-generator`

**Excluded**:
- `services/execution-engine` -- The Solana SDK pins `tokio ~1.14`, which conflicts with the workspace `tokio 1.35+`. This crate maintains its own `Cargo.toml` and is built independently.

Shared dependencies are declared once in `[workspace.dependencies]` and consumed in each member crate with `{ workspace = true }`. Service-specific dependencies (e.g., `ibapi`, `solana-sdk`) are pinned locally in the service's own `Cargo.toml`.

## Infrastructure

```
infrastructure/
├── database/
│   ├── postgres/
│   │   ├── migrations/        # Sequential SQL migrations (001_ through 999_)
│   │   │   ├── 001_core_schema.sql
│   │   │   ├── 002_market_data_unified.sql
│   │   │   ├── 003_trading_system_unified.sql
│   │   │   ├── ...
│   │   │   ├── 015_performance_optimization.sql
│   │   │   └── 999_verification.sql
│   │   └── schema.sql         # Full schema reference
│   └── clickhouse/
│       └── migrations/
│           ├── 002_clickhouse_ticks.sql
│           ├── 003_materialized_view.sql
│           └── 004_system_logs.sql
├── vector/                    # Vector log pipeline config (Docker -> ClickHouse)
├── terraform/                 # Infrastructure-as-Code (Azure modules)
├── python/                    # Shared Python library (hermes_common, legacy)
└── aws/                       # AWS-specific infrastructure scripts
```

### Database Migrations

Postgres migrations use sequential `NNN_description.sql` naming. All DDL uses `IF NOT EXISTS` / `IF EXISTS` for idempotency. Rust services reference migration files via `include_str!()` paths pointing to the infrastructure directory:

```rust
include_str!("../../../../infrastructure/database/postgres/migrations/001_core_schema.sql")
```

## Configuration

Configuration files are located at multiple levels:

- **Root**: `config/factors.yaml` -- shared factor definitions used by backtest and strategy engines
- **Service-level**: Each Rust service with config loading has a `config/` directory containing `default.toml` (and optionally `prod.toml`, `dev.toml`)
- **Docker Compose**: `docker-compose.yml` defines environment variables that override service config via the `{SERVICE_NAME}__SECTION__KEY` pattern
- **Infrastructure**: `infrastructure/vector/vector.toml` for log pipeline, Terraform modules for cloud resources

## Documentation

```
docs/
├── ARCHITECTURE.md            # System design, component descriptions, data sources
├── STANDARDS.md               # Architecture-level engineering standards
├── CODE_CONVENTIONS.md        # Rust code patterns and conventions
├── PROJECT_STRUCTURE.md       # This file
├── system_architecture.md     # Legacy architecture notes
└── prd/                       # Product requirements documents
    ├── prd-hermesflow.md
    ├── modules/
    └── user-stories/
```
