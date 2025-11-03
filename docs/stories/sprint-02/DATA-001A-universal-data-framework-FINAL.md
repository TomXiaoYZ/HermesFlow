# DATA-001A: Universal Data Framework & HTTP API

**Status**: ✅ **APPROVED** - Ready for Sprint 2  
**Version**: 2.0 (Final)  
**Date**: 2025-10-22  
**Epic**: Epic 1 - Cryptocurrency Data Collection  
**Story Points**: 7 SP  
**Priority**: P0 🔴 Critical  
**Sprint**: Sprint 2 (Oct 28 - Nov 8, 2 weeks)

---

## 📋 Story Overview

### User Story

**As a** HermesFlow system developer  
**I want** a universal data framework with standardized interfaces and HTTP API  
**So that** we can rapidly integrate multiple data sources (CEX, DEX, stocks, sentiment) with consistent behavior, type safety, and observability

### Business Value

- **Strategic ROI**: 29× efficiency gain for future integrations
  - First source: 14 days (framework + implementation)
  - Subsequent sources: 0.5-1 day each (framework reuse)
  - 20 planned sources: Save 280 person-days
- **Quality**: Type-safe architecture prevents runtime errors
- **Scalability**: Supports incremental performance optimization (10k → 100k msg/s)
- **Maintainability**: Single framework for all data sources
- **Observability**: Built-in health monitoring and metrics

### Scope

✅ **In Scope** (Sprint 2):
1. Core traits and interfaces (`DataSourceConnector`, `MessageParser`)
2. Data models (`AssetType`, `StandardMarketData`)
3. ClickHouse unified schema design
4. HTTP server (Axum) with health, metrics, and query endpoints
5. Parser registry and routing system
6. Architecture documentation
7. Unit tests (85%+ coverage)
8. Performance benchmarking framework

❌ **Out of Scope** (Deferred to DATA-001B):
- Binance WebSocket implementation
- Live data collection
- Production deployment
- 24-hour stability testing
- Load testing (>10k msg/s)

---

## 🎯 Acceptance Criteria

### AC-1: DataSourceConnector Trait Design ✅

**Given** a new data source needs to be integrated  
**When** the developer implements the `DataSourceConnector` trait  
**Then** the following interface must be supported:

```rust
use async_trait::async_trait;
use tokio::sync::mpsc;

#[async_trait]
pub trait DataSourceConnector: Send + Sync {
    /// Returns the data source type
    fn source_type(&self) -> DataSourceType;
    
    /// Returns supported asset types for this source
    fn supported_assets(&self) -> Vec<AssetType>;
    
    /// Connects to the data source and starts streaming
    /// Returns a receiver channel for standardized market data
    async fn connect(&mut self) -> Result<mpsc::Receiver<StandardMarketData>>;
    
    /// Gracefully disconnects from the data source
    async fn disconnect(&mut self) -> Result<()>;
    
    /// Health check - returns true if connection is healthy
    async fn is_healthy(&self) -> bool;
    
    /// Returns connection statistics
    fn stats(&self) -> ConnectorStats;
}

pub struct ConnectorStats {
    pub messages_received: u64,
    pub messages_processed: u64,
    pub errors: u64,
    pub uptime_secs: u64,
    pub last_message_at: Option<SystemTime>,
}
```

**Validation**:
- ✅ Trait compiles without errors
- ✅ All methods have clear documentation
- ✅ Mock implementation passes type checks
- ✅ `async_trait` macro applied correctly

---

### AC-2: StandardMarketData Model ✅

**Given** different data sources emit market data  
**When** data is normalized  
**Then** it must conform to this unified structure:

```rust
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardMarketData {
    // Source identification
    pub source: DataSourceType,      // e.g., BinanceSpot
    pub exchange: String,             // e.g., "Binance"
    pub symbol: String,               // e.g., "BTCUSDT"
    pub asset_type: AssetType,        // e.g., Spot
    
    // Market data
    pub data_type: MarketDataType,    // Trade, Ticker, Kline, OrderBook
    pub price: Decimal,               // Last price
    pub quantity: Decimal,            // Volume
    pub timestamp: i64,               // Exchange timestamp (ms)
    pub received_at: i64,             // System received timestamp (ms)
    
    // Optional fields
    pub bid: Option<Decimal>,
    pub ask: Option<Decimal>,
    pub high_24h: Option<Decimal>,
    pub low_24h: Option<Decimal>,
    pub volume_24h: Option<Decimal>,
    pub open_interest: Option<Decimal>,  // For futures
    pub funding_rate: Option<Decimal>,   // For perpetuals
    
    // Metadata
    pub sequence_id: Option<u64>,     // For ordering
    pub raw_data: String,             // Original message (for debugging)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DataSourceType {
    BinanceSpot,
    BinanceFutures,
    BinancePerp,
    OkxSpot,
    OkxFutures,
    GmgnDex,
    IbkrStock,
    PolygonStock,
    // Extensible for future sources
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AssetType {
    Spot,
    Perpetual,
    Future,
    Option,
    Stock,
    Index,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum MarketDataType {
    Trade,
    Ticker,
    Kline,
    OrderBook,
    FundingRate,
}
```

**Validation**:
- ✅ All fields have clear semantic meaning
- ✅ Serialization/deserialization works (serde tests)
- ✅ `Decimal` type used for all financial values (no f64)
- ✅ Timestamps in milliseconds (consistent with most APIs)
- ✅ Optional fields support partial data
- ✅ `raw_data` preserved for debugging

---

### AC-3: MessageParser Trait & Registry ✅

**Given** raw messages arrive from different data sources  
**When** parsing is needed  
**Then** the parser system must support dynamic routing:

```rust
#[async_trait]
pub trait MessageParser: Send + Sync {
    /// Returns the data source this parser handles
    fn source_type(&self) -> DataSourceType;
    
    /// Parses raw message into standardized format
    /// Returns Ok(None) if message should be ignored (e.g., heartbeat)
    async fn parse(&self, raw: &str) -> Result<Option<StandardMarketData>>;
    
    /// Validates message format before parsing
    fn validate(&self, raw: &str) -> bool;
}

pub struct ParserRegistry {
    parsers: HashMap<DataSourceType, Arc<dyn MessageParser>>,
}

impl ParserRegistry {
    pub fn new() -> Self { /* ... */ }
    
    pub fn register(&mut self, parser: Arc<dyn MessageParser>) {
        self.parsers.insert(parser.source_type(), parser);
    }
    
    pub async fn parse(
        &self, 
        source: DataSourceType, 
        raw: &str
    ) -> Result<Option<StandardMarketData>> {
        self.parsers
            .get(&source)
            .ok_or(DataError::ParserNotFound)?
            .parse(raw)
            .await
    }
}
```

**Validation**:
- ✅ Registry supports dynamic parser registration
- ✅ Parser lookup is O(1) with HashMap
- ✅ Thread-safe with `Arc` and `Send + Sync`
- ✅ Returns `Ok(None)` for ignorable messages (heartbeats, pings)
- ✅ Errors handled gracefully

---

### AC-4: ClickHouse Unified Schema ✅

**Given** standardized market data needs persistent storage  
**When** data is ingested  
**Then** the following ClickHouse schema must be used:

```sql
-- Unified ticks table for all market data
CREATE TABLE IF NOT EXISTS unified_ticks (
    -- Identifiers
    source LowCardinality(String),       -- 'BinanceSpot', 'OkxFutures'
    exchange LowCardinality(String),     -- 'Binance', 'OKX'
    symbol String,                        -- 'BTCUSDT', 'ETHUSDT'
    asset_type LowCardinality(String),   -- 'Spot', 'Perpetual', 'Future'
    data_type LowCardinality(String),    -- 'Trade', 'Ticker', 'Kline'
    
    -- Market data
    price Decimal(32, 8),                 -- High precision
    quantity Decimal(32, 8),
    timestamp DateTime64(3),              -- Exchange timestamp (ms precision)
    received_at DateTime64(3),            -- System timestamp
    
    -- Optional fields (nullable)
    bid Nullable(Decimal(32, 8)),
    ask Nullable(Decimal(32, 8)),
    high_24h Nullable(Decimal(32, 8)),
    low_24h Nullable(Decimal(32, 8)),
    volume_24h Nullable(Decimal(32, 8)),
    open_interest Nullable(Decimal(32, 8)),
    funding_rate Nullable(Decimal(32, 8)),
    
    -- Metadata
    sequence_id Nullable(UInt64),
    raw_data String,                      -- Original message
    
    -- Ingestion metadata
    ingested_at DateTime64(3) DEFAULT now64(3)
    
) ENGINE = MergeTree()
PARTITION BY toYYYYMMDD(timestamp)
ORDER BY (source, symbol, timestamp)
SETTINGS index_granularity = 8192;

-- Index for fast latest price lookup
CREATE INDEX idx_symbol_timestamp ON unified_ticks (symbol, timestamp) TYPE minmax GRANULARITY 4;

-- Materialized view for 1-minute aggregates (optional, future optimization)
CREATE MATERIALIZED VIEW IF NOT EXISTS unified_ticks_1m
ENGINE = AggregatingMergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (source, symbol, timestamp)
AS SELECT
    source,
    exchange,
    symbol,
    asset_type,
    toStartOfMinute(timestamp) AS timestamp,
    argMax(price, timestamp) AS close,
    max(price) AS high,
    min(price) AS low,
    argMin(price, timestamp) AS open,
    sum(quantity) AS volume,
    count() AS ticks
FROM unified_ticks
GROUP BY source, exchange, symbol, asset_type, timestamp;
```

**Schema Design Principles**:
- ✅ **Single table** for all data sources (simplicity)
- ✅ **LowCardinality** for enum-like columns (compression)
- ✅ **Decimal(32, 8)** for financial precision
- ✅ **DateTime64(3)** for millisecond timestamps
- ✅ **Partitioned by date** for efficient queries and retention
- ✅ **Ordered by (source, symbol, timestamp)** for time-series queries
- ✅ **Nullable fields** for optional data
- ✅ **Raw data preserved** for debugging

**Validation**:
- ✅ Schema creation script runs without errors
- ✅ Test INSERT with mock data succeeds
- ✅ Query latency < 100ms for single symbol (local)
- ✅ Partitioning works (verify with `SHOW PARTITIONS`)
- ✅ Compression ratio > 5:1 (typical for time-series)

---

### AC-5: HTTP Server with Axum ✅

**Given** the service needs to expose health, metrics, and query APIs  
**When** the HTTP server starts  
**Then** the following endpoints must be available:

```rust
use axum::{
    Router,
    routing::{get, post},
    extract::{Path, Query, State},
    response::Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

// Health check endpoint
// GET /health
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,           // "healthy" | "degraded" | "unhealthy"
    pub version: String,          // e.g., "0.1.0"
    pub uptime_secs: u64,
    pub dependencies: HealthDeps,
}

#[derive(Serialize)]
pub struct HealthDeps {
    pub redis: DependencyStatus,
    pub clickhouse: DependencyStatus,
    pub websocket: DependencyStatus,
}

#[derive(Serialize)]
pub struct DependencyStatus {
    pub status: String,       // "up" | "down"
    pub latency_ms: Option<f64>,
    pub last_check: String,   // ISO 8601 timestamp
}

// Metrics endpoint (Prometheus format)
// GET /metrics
// Returns:
//   data_engine_messages_received_total{source="BinanceSpot"} 12345
//   data_engine_messages_processed_total{source="BinanceSpot"} 12340
//   data_engine_errors_total{source="BinanceSpot",type="parse_error"} 5
//   data_engine_latency_seconds{quantile="0.5"} 0.003
//   data_engine_latency_seconds{quantile="0.99"} 0.018

// Latest market data query
// GET /api/v1/market/{symbol}/latest
#[derive(Serialize)]
pub struct LatestPriceResponse {
    pub symbol: String,
    pub price: String,        // Decimal as string
    pub timestamp: i64,
    pub source: String,
    pub bid: Option<String>,
    pub ask: Option<String>,
}

// Historical data query
// GET /api/v1/market/{symbol}/history?start=<timestamp>&end=<timestamp>&limit=1000
#[derive(Deserialize)]
pub struct HistoryQuery {
    pub start: Option<i64>,   // Start timestamp (ms)
    pub end: Option<i64>,     // End timestamp (ms)
    pub limit: Option<usize>, // Max records (default: 1000, max: 10000)
}

#[derive(Serialize)]
pub struct HistoryResponse {
    pub symbol: String,
    pub data: Vec<MarketDataPoint>,
    pub count: usize,
}

#[derive(Serialize)]
pub struct MarketDataPoint {
    pub timestamp: i64,
    pub price: String,
    pub quantity: String,
    pub source: String,
}

// Server setup
pub fn create_router(app_state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics))
        .route("/api/v1/market/:symbol/latest", get(get_latest_price))
        .route("/api/v1/market/:symbol/history", get(get_history))
        .with_state(app_state)
}
```

**Endpoint Requirements**:

1. **Health Check** (`/health`):
   - ✅ Returns 200 if all dependencies healthy
   - ✅ Returns 503 if any critical dependency down
   - ✅ Returns 200 with "degraded" if non-critical dependency down
   - ✅ Checks Redis, ClickHouse, WebSocket connection status
   - ✅ Response time < 100ms

2. **Metrics** (`/metrics`):
   - ✅ Prometheus-compatible format
   - ✅ Exposes message counters, error counters, latency histograms
   - ✅ Labels by source type
   - ✅ Response time < 50ms

3. **Latest Price** (`/api/v1/market/{symbol}/latest`):
   - ✅ Fetches latest price from Redis cache
   - ✅ Returns 404 if symbol not found
   - ✅ Returns 200 with data if found
   - ✅ Response time < 10ms (Redis lookup)

4. **History** (`/api/v1/market/{symbol}/history`):
   - ✅ Queries ClickHouse for historical data
   - ✅ Supports time range filtering (start/end timestamps)
   - ✅ Limits results (default 1000, max 10000)
   - ✅ Returns 400 if invalid parameters
   - ✅ Returns 200 with data
   - ✅ Response time < 200ms for 1000 records

**Validation**:
- ✅ Server starts on port 8080 (configurable)
- ✅ All endpoints return valid JSON
- ✅ CORS headers configured (if needed)
- ✅ Error responses follow RFC 7807 (Problem Details)
- ✅ Graceful shutdown on SIGTERM

---

### AC-6: Configuration Management ✅

**Given** the service needs flexible configuration  
**When** deployed in different environments  
**Then** configuration must support layered sources:

```rust
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub redis: RedisConfig,
    pub clickhouse: ClickHouseConfig,
    pub data_sources: Vec<DataSourceConfig>,
    pub performance: PerformanceConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,              // Default: "0.0.0.0"
    pub port: u16,                 // Default: 8080
    pub shutdown_timeout_secs: u64, // Default: 30
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    pub url: String,               // e.g., "redis://localhost:6379"
    pub pool_size: usize,          // Default: 10
    pub ttl_secs: u64,             // Default: 86400 (24h)
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClickHouseConfig {
    pub url: String,               // e.g., "tcp://localhost:9000"
    pub database: String,          // e.g., "hermesflow"
    pub username: String,
    pub password: String,
    pub batch_size: usize,         // Default: 1000
    pub flush_interval_ms: u64,    // Default: 5000
}

#[derive(Debug, Deserialize, Clone)]
pub struct DataSourceConfig {
    pub name: String,              // e.g., "binance_spot"
    pub source_type: String,       // e.g., "BinanceSpot"
    pub enabled: bool,
    pub symbols: Vec<String>,      // e.g., ["BTCUSDT", "ETHUSDT"]
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PerformanceConfig {
    pub channel_buffer_size: usize,     // Default: 10000
    pub max_reconnect_attempts: u32,    // Default: 5
    pub reconnect_delay_secs: u64,      // Default: 5
    pub health_check_interval_secs: u64, // Default: 10
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,             // Default: "info"
    pub format: String,            // "json" | "pretty"
    pub output: String,            // "stdout" | "file"
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        Config::builder()
            // Start with defaults
            .add_source(File::with_name("config/default.toml").required(false))
            // Layer environment-specific config
            .add_source(File::with_name(&format!("config/{}.toml", 
                std::env::var("RUST_ENV").unwrap_or_else(|_| "dev".to_string())
            )).required(false))
            // Layer environment variables (DATA_ENGINE_*)
            .add_source(Environment::with_prefix("DATA_ENGINE").separator("__"))
            .build()?
            .try_deserialize()
    }
}
```

**Configuration Layers** (priority order):
1. **Environment variables** (highest): `DATA_ENGINE__SERVER__PORT=8081`
2. **Environment-specific file**: `config/prod.toml`
3. **Default file** (lowest): `config/default.toml`

**Example `config/default.toml`**:
```toml
[server]
host = "0.0.0.0"
port = 8080
shutdown_timeout_secs = 30

[redis]
url = "redis://localhost:6379"
pool_size = 10
ttl_secs = 86400

[clickhouse]
url = "tcp://localhost:9000"
database = "hermesflow"
username = "default"
password = ""
batch_size = 1000
flush_interval_ms = 5000

[[data_sources]]
name = "binance_spot"
source_type = "BinanceSpot"
enabled = true
symbols = ["BTCUSDT", "ETHUSDT"]

[performance]
channel_buffer_size = 10000
max_reconnect_attempts = 5
reconnect_delay_secs = 5
health_check_interval_secs = 10

[logging]
level = "info"
format = "json"
output = "stdout"
```

**Validation**:
- ✅ Config loads from TOML files
- ✅ Environment variables override file values
- ✅ Missing optional files don't cause errors
- ✅ Invalid config returns clear error messages
- ✅ Secrets not committed to Git (use `.env` for local dev)

---

### AC-7: Error Handling ✅

**Given** various failure scenarios  
**When** errors occur  
**Then** custom error types with context must be used:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Connection failed for {source}: {reason}")]
    ConnectionFailed {
        source: String,
        reason: String,
    },
    
    #[error("Parse error for {source}: {message}")]
    ParseError {
        source: String,
        message: String,
        raw_data: String,
    },
    
    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),
    
    #[error("ClickHouse error: {0}")]
    ClickHouseError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(#[from] config::ConfigError),
    
    #[error("WebSocket error: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),
    
    #[error("Parser not found for source: {0}")]
    ParserNotFound(String),
    
    #[error("Invalid data: {0}")]
    ValidationError(String),
    
    #[error("Timeout after {timeout_secs}s: {operation}")]
    TimeoutError {
        operation: String,
        timeout_secs: u64,
    },
}

pub type Result<T> = std::result::Result<T, DataError>;

// Retry logic with exponential backoff
pub async fn retry_with_backoff<F, T>(
    operation: F,
    max_attempts: u32,
    initial_delay_ms: u64,
) -> Result<T>
where
    F: Fn() -> std::pin::Pin<Box<dyn Future<Output = Result<T>> + Send>>,
{
    let mut delay = initial_delay_ms;
    for attempt in 1..=max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt == max_attempts => return Err(e),
            Err(e) => {
                tracing::warn!("Attempt {}/{} failed: {}. Retrying in {}ms", 
                    attempt, max_attempts, e, delay);
                tokio::time::sleep(Duration::from_millis(delay)).await;
                delay = std::cmp::min(delay * 2, 60000); // Cap at 60s
            }
        }
    }
    unreachable!()
}
```

**Error Handling Requirements**:
- ✅ All errors implement `std::error::Error`
- ✅ Errors include context (source, operation, etc.)
- ✅ Retry logic for transient failures (connection, network)
- ✅ Exponential backoff with max delay cap
- ✅ Errors logged with `tracing` (structured logging)
- ✅ Critical errors trigger health check failure

---

### AC-8: Architecture Documentation ✅

**Given** future developers need to understand the framework  
**When** onboarding or extending the system  
**Then** comprehensive architecture documentation must exist:

**Required Documentation**:

1. **`docs/architecture/data-engine-architecture.md`**:
   - ✅ System overview and design principles
   - ✅ Component diagrams (ASCII art)
   - ✅ Data flow diagrams
   - ✅ Technology stack rationale
   - ✅ Performance considerations
   - ✅ Scalability roadmap

2. **`modules/data-engine/README.md`**:
   - ✅ Quick start guide
   - ✅ Configuration examples
   - ✅ API endpoint documentation
   - ✅ Troubleshooting guide
   - ✅ Development setup instructions

3. **Inline code documentation**:
   - ✅ All public traits documented with `///`
   - ✅ Example usage for complex traits
   - ✅ Design rationale for non-obvious decisions
   - ✅ Performance notes where relevant

4. **Integration guide** (`docs/guides/adding-new-data-source.md`):
   - ✅ Step-by-step guide for implementing new sources
   - ✅ Mock OKX example implementation
   - ✅ Checklist for implementation completion
   - ✅ Testing requirements

**Documentation Quality Standards**:
- ✅ Clear and concise language
- ✅ Code examples compile and run
- ✅ Diagrams accurate and up-to-date
- ✅ Links work (no 404s)
- ✅ Reviewed by at least one other developer

---

### AC-9: Service Health & Monitoring ✅

**Given** the service runs in production  
**When** monitoring its health  
**Then** comprehensive observability must be available:

**AC-9.1: Structured Logging**
```rust
use tracing::{info, warn, error, debug, instrument};

#[instrument(skip(self))]
async fn process_message(&self, raw: &str) -> Result<()> {
    debug!("Processing message: len={}", raw.len());
    
    match self.parser.parse(raw).await {
        Ok(Some(data)) => {
            info!(
                source = %data.source,
                symbol = %data.symbol,
                price = %data.price,
                "Market data processed"
            );
            Ok(())
        }
        Ok(None) => {
            debug!("Message ignored (heartbeat or non-market)");
            Ok(())
        }
        Err(e) => {
            error!(
                error = %e,
                raw_len = raw.len(),
                "Parse error"
            );
            Err(e)
        }
    }
}
```

**Requirements**:
- ✅ Structured logs (JSON in production)
- ✅ Log levels: DEBUG, INFO, WARN, ERROR
- ✅ Correlation IDs for distributed tracing
- ✅ Performance: <1% overhead

**AC-9.2: Prometheus Metrics**
```rust
use prometheus::{Counter, Histogram, register_counter, register_histogram};

lazy_static! {
    static ref MESSAGES_RECEIVED: Counter = register_counter!(
        "data_engine_messages_received_total",
        "Total messages received"
    ).unwrap();
    
    static ref PARSE_LATENCY: Histogram = register_histogram!(
        "data_engine_parse_latency_seconds",
        "Message parse latency"
    ).unwrap();
}
```

**Required Metrics**:
- ✅ `messages_received_total` (counter, by source)
- ✅ `messages_processed_total` (counter, by source)
- ✅ `errors_total` (counter, by type)
- ✅ `parse_latency_seconds` (histogram, p50/p95/p99)
- ✅ `redis_latency_seconds` (histogram)
- ✅ `clickhouse_latency_seconds` (histogram)
- ✅ `service_up` (gauge, 1=up, 0=down)

**AC-9.3: Health Monitoring**
```rust
pub struct HealthMonitor {
    last_message: Arc<RwLock<Option<Instant>>>,
    redis_status: Arc<RwLock<DependencyStatus>>,
    clickhouse_status: Arc<RwLock<DependencyStatus>>,
}

impl HealthMonitor {
    pub async fn check_health(&self) -> HealthStatus {
        let redis_ok = self.check_redis().await;
        let clickhouse_ok = self.check_clickhouse().await;
        let recent_data = self.last_message.read().await
            .map(|t| t.elapsed() < Duration::from_secs(60))
            .unwrap_or(false);
        
        match (redis_ok, clickhouse_ok, recent_data) {
            (true, true, true) => HealthStatus::Healthy,
            (true, true, false) => HealthStatus::Degraded("No recent data"),
            _ => HealthStatus::Unhealthy,
        }
    }
}
```

**AC-9.4: Service Availability** ⭐ NEW
**Requirement**: Service must maintain **≥ 99.9% uptime**

**Monitoring**:
- ✅ Health check endpoint polled every 10 seconds
- ✅ Alert if health check fails 3 consecutive times
- ✅ Track downtime in monitoring dashboard
- ✅ Monthly uptime report

**Uptime Calculation**:
```
Uptime % = (Total Time - Downtime) / Total Time × 100%
99.9% uptime = max 86 seconds downtime per day
              = max 43 minutes downtime per month
```

**Failure Scenarios**:
- ✅ Redis down → Degraded (cache miss, query ClickHouse fallback)
- ✅ ClickHouse down → Unhealthy (cannot persist data)
- ✅ WebSocket disconnected → Healthy (auto-reconnect within 30s)
- ✅ All dependencies down → Unhealthy (critical failure)

**Auto-Recovery**:
- ✅ Automatic reconnection for transient failures
- ✅ Circuit breaker for repeated failures
- ✅ Graceful degradation (serve stale data from Redis)

**Alerting**:
- ✅ Slack/PagerDuty integration for critical alerts
- ✅ Email for degraded status
- ✅ Dashboard for real-time visibility

**Validation**:
- ✅ Health check responds in < 100ms
- ✅ Prometheus metrics exported at `/metrics`
- ✅ Structured logs parseable by log aggregator
- ✅ All dependency statuses tracked
- ✅ Uptime monitoring active

---

### AC-10: Unit Tests (85%+ Coverage) ✅

**Given** the framework code is complete  
**When** running tests  
**Then** comprehensive unit tests must pass:

**Test Coverage Requirements**:

1. **Trait Tests**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    struct MockConnector;
    
    #[async_trait]
    impl DataSourceConnector for MockConnector {
        fn source_type(&self) -> DataSourceType {
            DataSourceType::BinanceSpot
        }
        
        fn supported_assets(&self) -> Vec<AssetType> {
            vec![AssetType::Spot, AssetType::Perpetual]
        }
        
        async fn connect(&mut self) -> Result<mpsc::Receiver<StandardMarketData>> {
            let (tx, rx) = mpsc::channel(100);
            Ok(rx)
        }
        
        async fn disconnect(&mut self) -> Result<()> { Ok(()) }
        async fn is_healthy(&self) -> bool { true }
        fn stats(&self) -> ConnectorStats { /* ... */ }
    }
    
    #[tokio::test]
    async fn test_mock_connector() {
        let mut connector = MockConnector;
        assert_eq!(connector.source_type(), DataSourceType::BinanceSpot);
        assert!(connector.supported_assets().contains(&AssetType::Spot));
        
        let rx = connector.connect().await.unwrap();
        assert!(connector.is_healthy().await);
    }
}
```

2. **Parser Tests**:
```rust
#[tokio::test]
async fn test_parser_registry() {
    let mut registry = ParserRegistry::new();
    let parser = Arc::new(MockParser);
    registry.register(parser);
    
    let result = registry.parse(
        DataSourceType::BinanceSpot,
        r#"{"e":"trade","s":"BTCUSDT","p":"50000.00"}"#
    ).await;
    
    assert!(result.is_ok());
}
```

3. **Data Model Tests**:
```rust
#[test]
fn test_standard_market_data_serialization() {
    let data = StandardMarketData {
        source: DataSourceType::BinanceSpot,
        symbol: "BTCUSDT".to_string(),
        price: Decimal::from_str("50000.12345678").unwrap(),
        // ... other fields
    };
    
    let json = serde_json::to_string(&data).unwrap();
    let deserialized: StandardMarketData = serde_json::from_str(&json).unwrap();
    assert_eq!(data.price, deserialized.price);
}
```

4. **Error Handling Tests**:
```rust
#[tokio::test]
async fn test_retry_with_backoff() {
    let mut attempts = 0;
    let result = retry_with_backoff(
        || {
            attempts += 1;
            Box::pin(async move {
                if attempts < 3 {
                    Err(DataError::ConnectionFailed {
                        source: "test".to_string(),
                        reason: "mock failure".to_string(),
                    })
                } else {
                    Ok(())
                }
            })
        },
        5,
        100,
    ).await;
    
    assert!(result.is_ok());
    assert_eq!(attempts, 3);
}
```

5. **Configuration Tests**:
```rust
#[test]
fn test_config_loading() {
    std::env::set_var("DATA_ENGINE__SERVER__PORT", "9090");
    let config = AppConfig::load().unwrap();
    assert_eq!(config.server.port, 9090);
}
```

**Coverage Targets**:
- ✅ Overall: ≥ 85%
- ✅ Core traits: 100%
- ✅ Data models: 100%
- ✅ Error types: 90%
- ✅ Utilities: 80%

**Test Execution**:
```bash
# Run all tests
cargo test

# Run with coverage
cargo tarpaulin --out Html --output-dir coverage

# Verify coverage ≥ 85%
cargo tarpaulin --fail-under 85
```

**Validation**:
- ✅ All tests pass (`cargo test`)
- ✅ Coverage report generated
- ✅ Coverage ≥ 85% overall
- ✅ No flaky tests (run 3× to verify)
- ✅ Tests run in < 30 seconds

---

### AC-11: Performance Benchmarking ✅

**Given** performance is a key requirement  
**When** benchmarking the framework  
**Then** the following benchmarks must be implemented:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn benchmark_parser(c: &mut Criterion) {
    let parser = MockParser;
    let raw_message = r#"{"e":"trade","s":"BTCUSDT","p":"50000.00","q":"0.001"}"#;
    
    c.bench_function("parse_message", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                parser.parse(black_box(raw_message)).await.unwrap()
            })
        })
    });
}

fn benchmark_serialization(c: &mut Criterion) {
    let data = StandardMarketData { /* ... */ };
    
    c.bench_function("serialize_to_json", |b| {
        b.iter(|| serde_json::to_string(black_box(&data)).unwrap())
    });
}

criterion_group!(benches, benchmark_parser, benchmark_serialization);
criterion_main!(benches);
```

**Benchmark Targets**:
- ✅ Message parsing: < 50 μs/msg (P95)
- ✅ JSON serialization: < 10 μs/msg
- ✅ Parser registry lookup: < 1 μs
- ✅ Error creation: < 5 μs

**Validation**:
- ✅ Benchmarks run without errors
- ✅ Results documented in `BENCHMARKS.md`
- ✅ Baseline established for future comparisons
- ✅ No performance regressions vs baseline

---

### AC-12: Extensibility Validation ✅

**Given** the framework is designed for multiple data sources  
**When** validating extensibility  
**Then** a mock OKX implementation must be created:

**Task**: Implement a mock `OkxConnector` to prove the framework is extensible

```rust
// Mock OKX implementation (for testing extensibility)
pub struct MockOkxConnector {
    symbols: Vec<String>,
    tx: Option<mpsc::Sender<StandardMarketData>>,
}

#[async_trait]
impl DataSourceConnector for MockOkxConnector {
    fn source_type(&self) -> DataSourceType {
        DataSourceType::OkxSpot
    }
    
    fn supported_assets(&self) -> Vec<AssetType> {
        vec![AssetType::Spot, AssetType::Perpetual, AssetType::Future]
    }
    
    async fn connect(&mut self) -> Result<mpsc::Receiver<StandardMarketData>> {
        let (tx, rx) = mpsc::channel(1000);
        self.tx = Some(tx);
        
        // Simulate periodic mock data
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(100)).await;
                // Send mock data
            }
        });
        
        Ok(rx)
    }
    
    async fn disconnect(&mut self) -> Result<()> {
        self.tx = None;
        Ok(())
    }
    
    async fn is_healthy(&self) -> bool {
        self.tx.is_some()
    }
    
    fn stats(&self) -> ConnectorStats {
        ConnectorStats::default()
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_okx_connector_extensibility() {
        let mut connector = MockOkxConnector {
            symbols: vec!["BTC-USDT".to_string()],
            tx: None,
        };
        
        let mut rx = connector.connect().await.unwrap();
        assert!(connector.is_healthy().await);
        
        // Verify we can receive mock data
        let data = tokio::time::timeout(
            Duration::from_secs(1),
            rx.recv()
        ).await;
        
        assert!(data.is_ok());
    }
}
```

**Validation Criteria**:
- ✅ Mock OKX implementation compiles without errors
- ✅ Implements all `DataSourceConnector` trait methods
- ✅ Can be registered in `ParserRegistry`
- ✅ Mock data flows through standardized pipeline
- ✅ No framework changes needed for new source
- ✅ Implementation time: < 2 hours (validates efficiency)

**Documentation**:
- ✅ Mock OKX code included in `docs/guides/adding-new-data-source.md`
- ✅ Step-by-step guide explains extensibility
- ✅ Checklist for new data source integration

---

## 📊 Performance Targets

### Sprint 2 - MVP Baseline (Framework Only)

| Metric | Target | Rationale |
|--------|--------|-----------|
| **Parser Latency** | P95 < 50 μs | JSON parsing + validation |
| **Serialization** | < 10 μs/msg | serde JSON serialization |
| **Registry Lookup** | < 1 μs | HashMap O(1) lookup |
| **HTTP /health** | < 100ms | Dependency checks |
| **HTTP /latest** | < 10ms | Redis lookup |
| **HTTP /history** | < 200ms | ClickHouse query (1000 rows) |
| **Unit Test Suite** | < 30 seconds | Fast feedback loop |
| **Build Time** | < 2 minutes | Developer productivity |

**Note**: Live data throughput targets (10k msg/s) will be validated in Sprint 3 with actual Binance WebSocket.

### Future Performance Tiers (Reference)

#### Sprint 4 - Optimization Phase
- **Target**: 50k msg/s (5× improvement)
- **Techniques**: Connection pooling, multi-threading, larger batches
- **Risk**: 🟡 Medium (requires profiling)

#### Sprint 6 - Production Scale
- **Target**: 100k+ msg/s (10× improvement)
- **Techniques**: Horizontal scaling, Kafka distribution, sharded storage
- **Risk**: 🟢 Low (architecture supports this)

**Scaling Roadmap**: See `docs/architecture/performance-scaling-roadmap.md`

---

## 🏗️ Technical Implementation Notes

### Technology Stack

| Component | Technology | Version | Rationale |
|-----------|-----------|---------|-----------|
| **Async Runtime** | Tokio | 1.35+ | Industry standard, excellent docs |
| **HTTP Server** | Axum | 0.7+ | Type-safe, fast, ergonomic |
| **WebSocket** | tungstenite | 0.21+ | Lightweight, async-aware |
| **Database** | clickhouse-rs | 1.0+ | Native ClickHouse driver |
| **Cache** | redis-rs | 0.24+ | High-performance Redis client |
| **Messaging** | rdkafka | 0.36+ | (Future) Librdkafka bindings |
| **Serialization** | serde | 1.0+ | Zero-copy, type-safe |
| **Decimals** | rust_decimal | 1.33+ | No floating point errors |
| **Logging** | tracing | 0.1+ | Structured, async-aware |
| **Metrics** | prometheus | 0.13+ | Industry standard |
| **Errors** | thiserror | 1.0+ | Ergonomic error definitions |
| **Config** | config | 0.14+ | Layered configuration |
| **Testing** | testcontainers | 0.15+ | (Future) Integration tests |

### Project Structure

```
modules/data-engine/
├── Cargo.toml                  # Dependencies
├── README.md                   # Quick start guide
├── src/
│   ├── main.rs                 # Entry point
│   ├── config.rs               # Configuration management
│   ├── error.rs                # Error types
│   ├── models/
│   │   ├── mod.rs
│   │   ├── asset_type.rs       # AssetType enum
│   │   ├── data_source_type.rs # DataSourceType enum
│   │   └── market_data.rs      # StandardMarketData struct
│   ├── traits/
│   │   ├── mod.rs
│   │   ├── connector.rs        # DataSourceConnector trait
│   │   └── parser.rs           # MessageParser trait
│   ├── registry/
│   │   └── parser_registry.rs  # ParserRegistry
│   ├── server/
│   │   ├── mod.rs
│   │   ├── routes.rs           # HTTP routes
│   │   └── handlers.rs         # Request handlers
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── clickhouse.rs       # ClickHouse client
│   │   └── redis.rs            # Redis client
│   ├── monitoring/
│   │   ├── mod.rs
│   │   ├── health.rs           # Health checks
│   │   ├── metrics.rs          # Prometheus metrics
│   │   └── logging.rs          # Tracing setup
│   └── utils/
│       ├── mod.rs
│       └── retry.rs            # Retry logic
├── tests/
│   ├── integration_tests.rs    # Integration tests
│   └── extensibility_test.rs   # Mock OKX test
├── benches/
│   └── benchmarks.rs           # Criterion benchmarks
└── config/
    ├── default.toml            # Default configuration
    ├── dev.toml                # Development overrides
    └── prod.toml               # Production overrides
```

### Development Phases

**Phase 1: Core Types (Days 1-3)**
- Define traits, enums, structs
- Write comprehensive documentation
- Unit tests for all types
- Mock implementations

**Phase 2: Storage & Caching (Days 4-5)**
- ClickHouse schema creation
- Redis integration
- Connection management
- Error handling

**Phase 3: HTTP Server (Days 6-7)**
- Axum server setup
- Health endpoint with dependency checks
- Metrics endpoint (Prometheus)
- Query endpoints (latest, history)
- Integration tests

**Phase 4: Testing & Documentation (Days 8-10)**
- Complete unit test coverage (85%+)
- Integration tests
- Performance benchmarks
- Mock OKX extensibility test
- Architecture documentation
- Code review and refinement

### Code Quality Standards

- ✅ All public APIs documented with `///` comments
- ✅ Clippy warnings resolved (`cargo clippy -- -D warnings`)
- ✅ Formatted with `rustfmt` (`cargo fmt --check`)
- ✅ No `unwrap()` in production code (use `?` operator)
- ✅ Async functions use `#[instrument]` for tracing
- ✅ Error messages include context
- ✅ Test coverage ≥ 85%

---

## 📦 Deliverables

### Code Artifacts

| Artifact | Description | Status |
|----------|-------------|--------|
| `modules/data-engine/src/` | Framework source code | ⏳ In Sprint |
| `modules/data-engine/tests/` | Unit + integration tests | ⏳ In Sprint |
| `modules/data-engine/benches/` | Performance benchmarks | ⏳ In Sprint |
| `modules/data-engine/config/` | Configuration files | ⏳ In Sprint |
| `modules/data-engine/Cargo.toml` | Dependencies | ⏳ In Sprint |

### Documentation Artifacts

| Artifact | Description | Status |
|----------|-------------|--------|
| `docs/architecture/data-engine-architecture.md` | Architecture overview | ⏳ In Sprint |
| `docs/architecture/performance-scaling-roadmap.md` | Performance tiers | ⏳ In Sprint |
| `modules/data-engine/README.md` | Quick start guide | ⏳ In Sprint |
| `docs/guides/adding-new-data-source.md` | Integration guide | ⏳ In Sprint |
| `BENCHMARKS.md` | Performance results | ⏳ In Sprint |

### Database Artifacts

| Artifact | Description | Status |
|----------|-------------|--------|
| `modules/data-engine/migrations/001_create_unified_ticks.sql` | ClickHouse schema | ⏳ In Sprint |
| `modules/data-engine/migrations/002_create_materialized_view.sql` | 1m aggregates | ⏳ In Sprint |

### Test Artifacts

| Artifact | Description | Status |
|----------|-------------|--------|
| Unit test coverage report | HTML coverage report | ⏳ In Sprint |
| Benchmark results | Criterion HTML report | ⏳ In Sprint |
| Mock OKX implementation | Extensibility proof | ⏳ In Sprint |

---

## 🎯 Definition of Done (DoD)

### Code Quality ✅
- [ ] All code compiles without warnings (`cargo build --release`)
- [ ] Clippy lints pass (`cargo clippy -- -D warnings`)
- [ ] Code formatted (`cargo fmt --check`)
- [ ] No `TODO` or `FIXME` comments in production code
- [ ] All public APIs documented
- [ ] Code reviewed by at least one other developer

### Testing ✅
- [ ] Unit tests pass (`cargo test`)
- [ ] Test coverage ≥ 85% (`cargo tarpaulin --fail-under 85`)
- [ ] Integration tests pass (mock OKX implementation)
- [ ] Performance benchmarks run successfully
- [ ] No flaky tests (run 3× to verify stability)

### Documentation ✅
- [ ] Architecture documentation complete
- [ ] README with quick start guide
- [ ] Integration guide for adding new data sources
- [ ] API documentation generated (`cargo doc --no-deps --open`)
- [ ] Performance scaling roadmap documented

### Performance ✅
- [ ] Parser latency P95 < 50 μs
- [ ] HTTP /health responds in < 100ms
- [ ] HTTP /latest responds in < 10ms
- [ ] Unit tests complete in < 30 seconds
- [ ] Build time < 2 minutes

### Database ✅
- [ ] ClickHouse schema created and tested
- [ ] Schema creation scripts in `migrations/`
- [ ] Test data inserted successfully
- [ ] Query latency validated (< 100ms local)

### HTTP Server ✅
- [ ] Server starts and responds to requests
- [ ] All endpoints return valid JSON
- [ ] Health endpoint checks all dependencies
- [ ] Metrics endpoint exports Prometheus format
- [ ] Query endpoints (latest, history) functional
- [ ] Graceful shutdown on SIGTERM

### Observability ✅
- [ ] Structured logging implemented
- [ ] Prometheus metrics exported
- [ ] Health monitoring active
- [ ] Uptime tracking operational
- [ ] All logs parseable by log aggregator

### Extensibility ✅
- [ ] Mock OKX implementation compiles and runs
- [ ] Extensibility test passes
- [ ] No framework changes needed for mock OKX
- [ ] Integration guide validated with mock example

### Acceptance ✅
- [ ] All 12 acceptance criteria met
- [ ] PO acceptance demo completed
- [ ] QA sign-off received
- [ ] Architecture review approved by Tech Lead
- [ ] Sprint review presentation prepared

### Deployment Readiness ✅
- [ ] Configuration management works (dev/prod)
- [ ] Environment variables documented
- [ ] Secrets management approach defined
- [ ] Docker build succeeds (multi-stage)
- [ ] Health check endpoint ready for k8s

---

## 📋 Dependencies & Risks

### Dependencies

| Dependency | Status | Owner | Notes |
|------------|--------|-------|-------|
| **Sprint 1 Complete** | ✅ Done | @sm.mdc | CI/CD, IaC, GitOps ready |
| **Rust Workshop** | 📅 Scheduled | @sm.mdc | Oct 25-27 (3 days) |
| **Redis Environment** | 🟡 Needed | @dev.mdc | Dev: localhost, Prod: Azure Cache |
| **ClickHouse Environment** | 🟡 Needed | @dev.mdc | Dev: Docker, Prod: Azure VM/Cloud |
| **Tech Lead Review** | ⏳ Pending | Tech Lead | Architecture sign-off |

### Risks (After Mitigation)

| Risk | Severity | Probability | Impact | Mitigation | Status |
|------|----------|-------------|--------|------------|--------|
| **Rust Learning Curve** | 🟡 Medium | 35% | High | 3-day workshop, pair programming | ✅ Mitigated |
| **Scope Too Large** | 🟢 Low | 30% | High | Story split (7 SP) | ✅ Resolved |
| **Performance Validation** | 🟢 Low | 30% | Medium | Documented tiers, defer to Sprint 3 | ✅ Mitigated |
| **ClickHouse Schema Evolution** | 🟡 Medium | 40% | Medium | Version migrations, backward compat | 🔄 Monitoring |
| **Testing Time Insufficient** | 🟢 Low | 25% | Medium | Start tests early (Day 5), dedicated phase | ✅ Mitigated |
| **External Dependencies** | 🟢 Low | 20% | Low | Redis/ClickHouse Docker for dev | ✅ Mitigated |

**Overall Risk Level**: 🟢 **LOW** (down from 🔴 HIGH before split)

---

## 📅 Timeline & Milestones

### Pre-Sprint (Oct 25-27): Preparation

| Date | Activity | Duration | Owner |
|------|----------|----------|-------|
| Oct 25 | Rust Workshop Day 1: Async Rust | 4h | @dev.mdc |
| Oct 26 | Rust Workshop Day 2: Tokio + WebSocket | 4h | @dev.mdc |
| Oct 27 | Rust Workshop Day 3: POC Connector | 4h | @dev.mdc |

### Sprint 2 (Oct 28 - Nov 8): Development

| Week | Days | Phase | Deliverables |
|------|------|-------|--------------|
| **Week 1** | Oct 28-Nov 1 | Core Types + Storage | Traits, models, ClickHouse, Redis |
| **Week 2** | Nov 4-8 | HTTP + Testing | Server, endpoints, tests, docs |

**Detailed Schedule**:

| Day | Date | Phase | Tasks |
|-----|------|-------|-------|
| 1 | Oct 28 | Core Types | Define traits, enums, data models |
| 2 | Oct 29 | Core Types | Implement parser registry, error types |
| 3 | Oct 30 | Core Types | Unit tests for core types, mock implementations |
| 4 | Oct 31 | Storage | ClickHouse schema, Redis integration |
| 5 | Nov 1 | Storage | Storage layer tests, connection management |
| 6 | Nov 4 | HTTP Server | Axum setup, health + metrics endpoints |
| 7 | Nov 5 | HTTP Server | Query endpoints, integration tests |
| 8 | Nov 6 | Testing | Complete unit tests (85%+ coverage) |
| 9 | Nov 7 | Testing | Benchmarks, mock OKX extensibility test |
| 10 | Nov 8 | Documentation | Finalize docs, code review, demo prep |

**Milestones**:

- ✅ **Milestone 1 (Nov 1)**: Core types + storage complete
- ✅ **Milestone 2 (Nov 5)**: HTTP server operational
- ✅ **Milestone 3 (Nov 8)**: All tests pass, docs complete, demo ready

### Post-Sprint (Nov 11): Sprint Review & Planning

| Date | Activity | Duration |
|------|----------|----------|
| Nov 11 | Sprint 2 Review & Retrospective | 2h |
| Nov 11 | Sprint 3 Planning (DATA-001B) | 2h |

---

## 🔗 Related Documents

### User Stories
- **DATA-001B**: Binance WebSocket Implementation (Sprint 3, 5 SP)
- **DATA-002**: OKX WebSocket Connector (Sprint 3/4, 3 SP)

### Documentation
- **PRD**: `docs/prd/modules/01-data-module.md` - Original requirements
- **Architecture**: `docs/architecture/data-engine-architecture.md` - System design
- **Roadmap**: `docs/architecture/performance-scaling-roadmap.md` - Performance tiers
- **Guide**: `docs/guides/adding-new-data-source.md` - Integration guide

### Review Documents
- **QA Review**: `docs/stories/sprint-02/DATA-001-qa-review.md`
- **PO Validation**: `docs/stories/sprint-02/DATA-001-po-validation.md`
- **Risk Profile**: `docs/stories/sprint-02/sprint-02-risk-profile.md`
- **Test Strategy**: `docs/stories/sprint-02/sprint-02-test-strategy.md`
- **Team Meeting Notes**: `docs/stories/sprint-02/DATA-001-team-meeting-notes.md`

### Sprint Documents
- **Sprint 2 Summary**: `docs/stories/sprint-02/sprint-02-summary.md`
- **Dev Notes**: `docs/stories/sprint-02/sprint-02-dev-notes.md`
- **QA Notes**: `docs/stories/sprint-02/sprint-02-qa-notes.md`

---

## 🎯 Success Metrics

### Quantitative Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Test Coverage** | ≥ 85% | `cargo tarpaulin` |
| **Parser Latency** | P95 < 50 μs | Criterion benchmarks |
| **HTTP Latency** | /health < 100ms | Integration tests |
| **Build Time** | < 2 min | CI/CD pipeline |
| **Code Quality** | 0 Clippy warnings | `cargo clippy` |
| **Documentation** | 100% public APIs | `cargo doc` |
| **Uptime** | ≥ 99.9% | Health monitoring |

### Qualitative Metrics

| Metric | Success Criteria |
|--------|-----------------|
| **Extensibility** | Mock OKX implementation takes < 2 hours |
| **Code Clarity** | Reviewed positively by 2+ developers |
| **Architecture** | Approved by Tech Lead |
| **Team Confidence** | Team feels prepared for Sprint 3 |
| **PO Satisfaction** | Unconditional approval from @po.mdc |

---

## 👥 Team

| Role | Member | Responsibilities |
|------|--------|------------------|
| **Product Owner** | @po.mdc | Requirements, acceptance, prioritization |
| **Scrum Master** | @sm.mdc | Facilitation, blockers, process |
| **Dev Lead** | @dev.mdc | Architecture, code review, implementation |
| **QA Lead** | @qa.mdc | Test strategy, quality assurance |
| **Tech Lead** | TBD | Architecture review, technical guidance |

---

## 📝 Notes

### Design Decisions

1. **Why Single ClickHouse Table?**
   - Simplicity: One schema for all sources
   - Flexibility: Easy to add new sources
   - Query convenience: No JOINs needed
   - Trade-off: Nullable fields, some storage overhead
   - Future: Can partition by source if needed

2. **Why Axum Over Actix-web?**
   - Type-safe extractors (better DX)
   - Seamless Tokio integration
   - Growing ecosystem
   - Simpler error handling

3. **Why redis-rs Over fred?**
   - Simpler API for our use case
   - Better documentation
   - Sufficient performance for MVP
   - Can migrate to fred later if needed

4. **Why Split Story?**
   - Risk reduction: 90% → 30% failure probability
   - Quality focus: More testing time
   - Team health: Achievable goals
   - Clear milestone: Framework completion

### Future Considerations

- **Kafka Integration**: Deferred to Sprint 4 (not needed for MVP)
- **Multi-instance Deployment**: Sprint 6 (horizontal scaling)
- **Advanced Analytics**: Sprint 6+ (real-time indicators)
- **WebSocket Server**: Sprint 5+ (push notifications to clients)

---

## ✅ Approval

| Role | Name | Status | Date | Signature |
|------|------|--------|------|-----------|
| **Product Owner** | @po.mdc | ✅ Approved | 2025-10-22 | Conditional → Unconditional |
| **QA Lead** | @qa.mdc | ✅ Approved | 2025-10-22 | Risks mitigated |
| **Dev Lead** | @dev.mdc | ✅ Committed | 2025-10-22 | Ready to deliver |
| **Scrum Master** | @sm.mdc | ✅ Approved | 2025-10-22 | Story finalized |

**Approval Status**: ✅ **UNCONDITIONALLY APPROVED**  
**Sprint 2 Start Date**: October 28, 2025 (Monday)  
**Sprint 2 End Date**: November 8, 2025 (Friday)

---

**Version History**:
- v1.0 (2025-10-21): Initial draft (13 SP, unsplit)
- v1.5 (2025-10-22): QA review, conditional approval
- v2.0 (2025-10-22): Final version (7 SP, split, approved) ✅

**Story Status**: ✅ **READY FOR SPRINT 2**

---

**Let's build something amazing! 🚀**






