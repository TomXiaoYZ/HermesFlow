# HermesFlow Engineering Standards

This document defines **architecture-level** engineering standards and best practices for the HermesFlow project. All new code must comply with these standards.

> **Related:** For code-level conventions (error handling, config loading, logging,
> health checks, database access, Cargo workspace, migrations, and project files),
> see [CODE_CONVENTIONS.md](./CODE_CONVENTIONS.md).
>
> - **STANDARDS.md** = architecture-level standards (repository pattern, build context, CI/CD, config management)
> - **CODE_CONVENTIONS.md** = code-level conventions (how to write Rust code in this project)

## 1. Architecture Patterns

### 1.1 Repository Pattern (Rust)

In `data-engine` and `gateway`, database interactions must be decoupled via the Repository pattern.
- **Trait definition**: Define traits in `src/repository/` (e.g., `MarketDataRepository`, `TradingRepository`).
- **Implementation**: Concrete implementations live in `src/repository/postgres/`.
- **Injection**: In `main.rs`, inject via `Arc<dyn Trait>` into service/handler layers.

### 1.2 Shared Rust Library (common crate)

All Rust services must use the `common` crate for shared functionality. Do not duplicate utilities across services.
- **Health checks**: Use `common::health::start_health_server()`.
- **Event bus**: Use `common::events` for Redis Pub/Sub event types.
- **Heartbeat**: Use `common::heartbeat` for service liveness reporting.
- **Feature flags**: The `common` crate uses Cargo feature flags (`health`, `heartbeat`) to keep dependencies minimal for each consumer.

## 2. Build and Deployment

### 2.1 Monorepo Build Context

All Docker builds must execute from the **project root directory**.
- **Dockerfile**: Located in each service directory (`services/xxx/Dockerfile`).
- **Compose**:
  ```yaml
  build:
    context: .
    dockerfile: services/data-engine/Dockerfile
  ```
- **Benefit**: Allows services to access shared resources in `infrastructure/` at build time (e.g., SQL migration files via `include_str!`).

### 2.2 CI/CD

- **GitHub Actions**: CI pipeline defined in `.github/workflows/ci.yml`.
- **Build commands**: Use `Makefile` targets (`make lint`, `make test`, `make build`). Do not add shell scripts for build logic.

## 3. Configuration Management (12-Factor App)

### 3.1 Layered Configuration

Application config loading priority:
1. **Environment variables** (highest): `DATA_ENGINE__SERVER__PORT`
2. **Environment-specific file**: `config/prod.toml`
3. **Default config**: `config/default.toml`

### 3.2 Sensitive Information

- **Never** commit passwords or API keys to Git (even in `prod.toml`).
- Use `.env` files for local development; use a secret store for production.
- Variable naming: `{SERVICE_NAME}__{SECTION}__{KEY}` (double underscore separates hierarchy levels).

## 4. Database Management

### 4.1 Schema Migrations

DDL changes must be committed as SQL files:
- **Postgres**: `infrastructure/database/postgres/migrations/`
- **ClickHouse**: `infrastructure/database/clickhouse/migrations/`
- File naming: `NNN_description.sql` (sequential numbering, no gaps).
- All DDL must use `IF NOT EXISTS` / `IF EXISTS` for idempotency.

### 4.2 DDL References in Rust

Rust services reference SQL migration files via relative paths to `infrastructure/`:
```rust
include_str!("../../../../infrastructure/database/postgres/migrations/001_core_schema.sql")
```
Do not copy SQL files into service directories.

## 5. Development Workflow

### 5.1 Prohibited

1. **Do not** add new shell scripts to `scripts/` for build logic. All build automation belongs in `Makefile` or `.github/workflows/`.
2. **Do not** run `cargo build` or `npm install` directly unless you know what you are doing. Use `make setup` and `make build`.
3. **Do not** commit `node_modules/`, `target/`, `.env`, `*.log`, `*.pid`, or `check_*.txt` files.

### 5.2 New Service Checklist

When adding a new service (`services/new-service`):
1. [ ] Use root build context in `docker-compose.yml` (`context: .`).
2. [ ] Rust services must join the Cargo workspace (add to `[workspace.members]` in root `Cargo.toml`) and use `{ workspace = true }` for shared dependencies.
3. [ ] If the service has a tokio version conflict (like execution-engine), add it to `[workspace.exclude]` with a comment explaining why.
4. [ ] Add a `Dockerfile`, `config/default.toml` (if applicable), and health check endpoint.
5. [ ] Add the service to `docker-compose.yml` with appropriate health check and dependency configuration.

## 6. Documentation

- **Do not** create ad-hoc markdown files. Update existing docs first (`ARCHITECTURE.md`, `STANDARDS.md`, `CODE_CONVENTIONS.md`).
- **ADR (Architecture Decision Records)**: Append important decisions to the Appendix section of `ARCHITECTURE.md`.
