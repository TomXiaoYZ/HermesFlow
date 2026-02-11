# HermesFlow Code Conventions

This document defines code-level conventions for the HermesFlow project.
Every new file and PR must comply with these rules.

> For architecture-level standards (repository pattern, build context, CI/CD),
> see [STANDARDS.md](./STANDARDS.md).

---

## 1. Error Handling

**Standard:** Use `thiserror` to define structured, domain-specific error enums.
`anyhow` is permitted only in `main.rs` and test code.

```rust
// services/data-engine/src/error.rs (gold standard)

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataEngineError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Error: {0}")]
    Other(String),
}
```

Structured variants with named fields are encouraged for rich context:

```rust
#[derive(Error, Debug)]
pub enum DataError {
    #[error("Connection failed for {data_source}: {reason}")]
    ConnectionFailed { data_source: String, reason: String },

    #[error("Timeout after {timeout_secs}s: {operation}")]
    TimeoutError { operation: String, timeout_secs: u64 },

    // Automatic From conversion via #[from]
    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),
}
```

**Prohibited:**

```rust
// Do NOT use anyhow in library code
use anyhow::Result;

fn connect() -> anyhow::Result<()> { // WRONG -- use thiserror enum
    Ok(())
}

// Do NOT use raw string errors
fn fetch() -> Result<(), String> { // WRONG -- use a proper error type
    Err("something broke".into())
}
```

**Gold standard:** `services/data-engine/src/error.rs`

---

## 2. Config Loading

**Standard:** Use the `config` crate with layered TOML files and environment variable overrides.
Environment variables use the prefix `{SERVICE_NAME}__` with double underscores as separators.
Never call `std::env::var()` directly for application configuration.

```rust
// services/data-engine/src/config.rs (gold standard)

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub postgres: PostgresConfig,
    pub logging: LoggingConfig,
    // ...
}

impl AppConfig {
    /// Loads configuration from multiple sources with priority:
    /// 1. Environment variables (highest priority)
    /// 2. Environment-specific file (e.g., config/prod.toml)
    /// 3. Default file (config/default.toml)
    pub fn load() -> Result<Self, ConfigError> {
        let env = std::env::var("RUST_ENV").unwrap_or_else(|_| "dev".to_string());

        let config = Config::builder()
            .add_source(File::with_name("config/default").required(true))
            .add_source(File::with_name(&format!("config/{}", env)).required(false))
            .add_source(
                Environment::with_prefix("DATA_ENGINE")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;

        config.try_deserialize()
    }
}
```

Every config struct must implement `Default`:

```rust
impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            shutdown_timeout_secs: 30,
        }
    }
}
```

**Prohibited:**

```rust
// Do NOT read config from raw env vars
let port = std::env::var("PORT").unwrap(); // WRONG

// Do NOT hardcode configuration values
let db_host = "localhost"; // WRONG -- put it in config/default.toml
```

**Gold standard:** `services/data-engine/src/config.rs`

---

## 3. Logging

**Standard:** Use `tracing` + `tracing-subscriber` with JSON output in production and pretty
output in development. The `RUST_LOG` environment variable must always be respected.

```rust
// services/data-engine/src/monitoring/logging.rs (gold standard)

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn init_logging(config: &LoggingConfig) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));

    match config.format.as_str() {
        "json" => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer().json())
                .init();
        }
        "pretty" => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer().pretty())
                .init();
        }
        _ => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer().compact())
                .init();
        }
    }
}
```

Use structured fields in log macros:

```rust
tracing::info!(host = %config.host, port = config.port, "Server started");
tracing::warn!(target = %target, err = %e, "Twitter scrape failed");
```

**Prohibited:**

```rust
// Do NOT use println! or eprintln! for application logging
println!("Server started on port {}", port); // WRONG

// Do NOT use the log crate directly
use log::info;
info!("starting up"); // WRONG -- use tracing::info!
```

**Gold standard:** `services/data-engine/src/monitoring/logging.rs`

---

## 4. Health Checks

**Standard:** Every service must expose a `/health` endpoint using `common::health::start_health_server()`.
The response format is a standard JSON object with `service`, `status`, and `timestamp` fields.

```rust
// services/common/src/health.rs (gold standard)

use axum::{routing::get, Json, Router};
use std::net::SocketAddr;
use tracing::info;

pub async fn start_health_server(service_name: &str, port: u16) {
    let name = service_name.to_owned();
    let handler = move || {
        let name = name.clone();
        async move {
            Json(serde_json::json!({
                "service": name,
                "status": "healthy",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        }
    };

    let app = Router::new().route("/health", get(handler));
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("{} health endpoint listening on {}", service_name, addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.expect("Health server failed");
}
```

The `common` crate exposes this behind a feature flag:

```toml
# services/common/Cargo.toml
[features]
health = ["dep:axum", "dep:tracing", "dep:tokio"]

# Consumer Cargo.toml
common = { workspace = true, features = ["health"] }
```

**Prohibited:**

```rust
// Do NOT implement a custom health endpoint per service
async fn my_health() -> &'static str { "OK" } // WRONG -- use common::health
```

**Gold standard:** `services/common/src/health.rs`

---

## 5. Database Access

**Standard:** All database access must go through the Repository trait pattern.
Traits are defined in `src/repository/mod.rs`; implementations live in `src/repository/postgres/`.
Handlers and business logic must never contain SQL directly.

Trait definition:

```rust
// services/data-engine/src/repository/mod.rs (gold standard)

use async_trait::async_trait;

#[async_trait]
pub trait MarketDataRepository: Send + Sync {
    async fn insert_snapshot(&self, data: &StandardMarketData) -> Result<(), DataEngineError>;
    async fn insert_candle(&self, data: &Candle) -> Result<(), DataEngineError>;
    async fn insert_candles(&self, data: &[Candle]) -> Result<(), DataEngineError>;
    async fn get_active_symbols(&self) -> Result<Vec<String>, DataEngineError>;
    async fn get_latest_candle_time(
        &self,
        exchange: &str,
        symbol: &str,
        resolution: &str,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, DataEngineError>;
}
```

Implementation:

```rust
// services/data-engine/src/repository/postgres/market_data.rs

pub struct PostgresMarketDataRepository {
    pool: PgPool,
}

impl PostgresMarketDataRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MarketDataRepository for PostgresMarketDataRepository {
    async fn insert_candle(&self, data: &Candle) -> Result<(), DataEngineError> {
        sqlx::query(r#"INSERT INTO mkt_equity_candles (...) VALUES (...)"#)
            .bind(&data.exchange)
            // ...
            .execute(&self.pool)
            .await
            .map_err(|e| DataEngineError::DatabaseError(format!("Failed to insert candle: {}", e)))?;
        Ok(())
    }
    // ...
}
```

Repositories are assembled and injected via `Arc`:

```rust
// services/data-engine/src/repository/postgres/mod.rs

pub struct PostgresRepositories {
    pub pool: PgPool,
    pub market_data: Arc<PostgresMarketDataRepository>,
    pub social: Arc<PostgresSocialRepository>,
    pub trading: Arc<PostgresTradingRepository>,
    // ...
}
```

**Prohibited:**

```rust
// Do NOT put SQL in handlers or business logic
async fn handle_request(pool: &PgPool) {
    sqlx::query("SELECT * FROM users").fetch_all(pool).await; // WRONG
}
```

**Gold standard:** `services/data-engine/src/repository/mod.rs`, `services/data-engine/src/repository/postgres/`

---

## 6. Cargo Workspace

**Standard:** All Rust crates must be workspace members. Shared dependencies are declared once
in the root `Cargo.toml` under `[workspace.dependencies]` and consumed with `{ workspace = true }`.

Root workspace definition:

```toml
# Cargo.toml (root)

[workspace]
members = [
    "services/common",
    "services/backtest-engine",
    "services/data-engine",
    "services/gateway",
    "services/strategy-engine",
    "services/strategy-generator",
]
# execution-engine excluded: Solana SDK pins tokio ~1.14 which conflicts
# with workspace tokio 1.35
exclude = ["services/execution-engine"]
resolver = "2"

[workspace.dependencies]
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
# ...
```

Service crate consuming workspace dependencies:

```toml
# services/data-engine/Cargo.toml

[dependencies]
tokio = { workspace = true }
serde = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
common = { workspace = true }
```

Service-specific dependencies that are not shared may pin their own version:

```toml
reqwest = { version = "0.11", features = ["json", "rustls-tls"] }
ibapi = "2.5"
```

**Prohibited:**

```toml
# Do NOT pin versions that are already in workspace.dependencies
[dependencies]
tokio = "1.35"      # WRONG -- use { workspace = true }
serde = "1.0"       # WRONG -- use { workspace = true }

# Do NOT add a new crate outside the workspace unless there is a documented conflict
```

**Gold standard:** `Cargo.toml` (root), `services/data-engine/Cargo.toml`

---

## 7. Database Migrations

**Standard:** All schema changes are SQL files in `infrastructure/database/{engine}/migrations/`.
Files use sequential `NNN_description.sql` naming. Every DDL statement must use `IF NOT EXISTS`
(or `IF EXISTS` for drops). Each file starts with a header comment explaining the change.

```sql
-- infrastructure/database/postgres/migrations/001_core_schema.sql (gold standard)

-- Initialize database schema for data-engine
-- Run this manually if you want to set up the database before first run
-- Otherwise, the application will create these tables automatically

CREATE TABLE IF NOT EXISTS tweets (
    id BIGINT PRIMARY KEY,
    username TEXT NOT NULL,
    text TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- ...
);

CREATE INDEX IF NOT EXISTS idx_tweets_username ON tweets(username);
```

ALTER migrations also require a header comment:

```sql
-- infrastructure/database/postgres/migrations/004_fix_numeric_overflow.sql

-- Fix numeric overflow for large crypto values (FDV, etc.)
-- Previous schema used (18,8) which caps at 10 Billion.
-- DECIMAL(40, 8) allows for 32 integer digits, plenty for any asset.

ALTER TABLE mkt_equity_candles
    ALTER COLUMN fdv TYPE DECIMAL(40, 8),
    ALTER COLUMN volume TYPE DECIMAL(40, 8);
```

Rust services reference migrations via `include_str!` pointing to the infrastructure directory:

```rust
include_str!("../../../../infrastructure/database/postgres/migrations/001_core_schema.sql")
```

**Prohibited:**

```sql
-- Do NOT omit IF NOT EXISTS on CREATE statements
CREATE TABLE tweets (...);  -- WRONG: will fail on re-run

-- Do NOT skip sequence numbers
-- 001, 002, 005  -- WRONG: gap in numbering
```

```
# Do NOT place SQL files inside service directories
services/data-engine/migrations/001.sql  -- WRONG: put in infrastructure/
```

**Gold standard:** `infrastructure/database/postgres/migrations/`

---

## 8. Project Files

### Required service directory structure

Every Rust service must contain at minimum:

```
services/{service-name}/
    Cargo.toml
    Dockerfile
    src/
        main.rs        (or lib.rs for library crates)
    config/
        default.toml   (if the service loads config)
```

### Module organization pattern

Use `mod.rs` files that declare submodules and re-export public items:

```rust
// services/data-engine/src/models/mod.rs (gold standard)

pub mod asset_type;
pub mod candle;
pub mod market_data;

pub use asset_type::*;
pub use candle::Candle;
pub use market_data::*;
```

Feature-gated modules in library crates:

```rust
// services/common/src/lib.rs (gold standard)

pub mod events;

#[cfg(feature = "health")]
pub mod health;

#[cfg(feature = "heartbeat")]
pub mod heartbeat;
```

### Prohibited commits

The following files must never be committed to the repository:

| Pattern | Reason |
|---------|--------|
| `*.log` | Runtime output |
| `*.pid` | Process IDs |
| `build_log.txt` | Build artifacts |
| `target/` | Cargo build output |
| `node_modules/` | npm dependencies |
| `.env` | Secrets |
| `*.tfstate*` | Terraform state (contains secrets) |
| `check_*.txt` | Debug scratch files |

These patterns must be present in `.gitignore`.

**Gold standard:** `services/common/src/lib.rs`, `services/data-engine/src/models/mod.rs`
