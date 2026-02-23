# services - FOLDER_INDEX

> All microservices and shared libraries.

## Service Map

| Directory | Language | Type | Description |
|-----------|----------|------|-------------|
| [data-engine/](data-engine/FOLDER_INDEX.md) | Rust | Service | Market data aggregation (12+ sources) |
| [gateway/](gateway/FOLDER_INDEX.md) | Rust | Service | API gateway + WebSocket router |
| [strategy-engine/](strategy-engine/FOLDER_INDEX.md) | Rust | Service | Real-time strategy execution |
| [strategy-generator/](strategy-generator/FOLDER_INDEX.md) | Rust | Service | GA strategy evolution + backtest |
| [execution-engine/](execution-engine/FOLDER_INDEX.md) | Rust | Service | Trade execution (IBKR/Solana/Futu) |
| [backtest-engine/](backtest-engine/FOLDER_INDEX.md) | Rust | Library | Factor computation + VM execution |
| [common/](common/FOLDER_INDEX.md) | Rust | Library | Shared types, events, metrics |
| [user-management/](user-management/FOLDER_INDEX.md) | Java | Service | Auth + RBAC (Spring Boot) |
| [futu-bridge/](futu-bridge/FOLDER_INDEX.md) | Python | Service | Futu OpenD HTTP bridge |
| [web/](web/FOLDER_INDEX.md) | TypeScript | Frontend | Dashboard (Next.js 16) |

## Workspace Structure

**Cargo workspace members**: common, backtest-engine, data-engine, gateway, strategy-engine, strategy-generator

**Excluded**: execution-engine (Solana SDK tokio conflict — build separately)

## Dependency Graph

```
data-engine ──→ common
gateway ──→ common
strategy-engine ──→ common + backtest-engine
strategy-generator ──→ common + backtest-engine
backtest-engine ──→ (no workspace crate deps)
execution-engine ──→ (excluded from workspace, Solana SDK tokio conflict)
```
