# Data Engine Architecture

**Version**: 1.0  
**Date**: 2025-10-24  
**Status**: Sprint 2 - Universal Framework MVP

---

## Table of Contents

1. [System Overview](#system-overview)
2. [Design Principles](#design-principles)
3. [Component Architecture](#component-architecture)
4. [Data Flow](#data-flow)
5. [Technology Stack](#technology-stack)
6. [Performance Considerations](#performance-considerations)
7. [Scalability Roadmap](#scalability-roadmap)
8. [Security](#security)

---

## System Overview

The Data Engine is a high-performance Rust-based service responsible for collecting, processing, and distributing real-time market data from multiple sources across cryptocurrency exchanges, traditional finance APIs, and sentiment data providers.

### Key Characteristics

- **Language**: Rust (for performance and safety)
- **Async Runtime**: Tokio
- **HTTP Framework**: Axum
- **Storage**: Redis (cache) + ClickHouse (time-series)
- **Target Throughput**: 10k-100k msg/s (incremental scaling)
- **Target Latency**: μs-level parsing, ms-level E2E

### Architecture Goals

1. **Extensibility**: Easy to add new data sources (< 2 hour implementation time)
2. **Performance**: High throughput with low latency
3. **Reliability**: 99.9% uptime, automatic reconnection
4. **Type Safety**: Compile-time guarantees using Rust's type system
5. **Observability**: Comprehensive metrics and logging

---

## Design Principles

### 1. Open-Closed Principle

The framework is open for extension but closed for modification. New data sources can be added without changing core framework code.

```
┌─────────────────────────────┐
│    DataSourceConnector     │  ← Trait (interface)
│       (Trait)               │
└─────────────────────────────┘
         ▲         ▲
         │         │
┌────────┴────┐  ┌─┴──────────┐
│   Binance   │  │    OKX     │  ← Implementations
│  Connector  │  │ Connector  │
└─────────────┘  └────────────┘
```

### 2. Dependency Inversion Principle

High-level modules depend on abstractions (traits), not concrete implementations.

### 3. Single Responsibility Principle

Each component has one clear purpose:
- **Connectors**: Connect to data sources
- **Parsers**: Parse messages
- **Storage**: Persist data
- **Server**: Serve HTTP requests

### 4. Data Versioning

All data includes:
- Exchange timestamp
- System received timestamp
- Sequence ID (where available)
- Raw data (for debugging)

---

## Component Architecture

### High-Level Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                      Data Engine                              │
│                                                                │
│  ┌────────────┐      ┌──────────────┐     ┌───────────────┐  │
│  │ HTTP API   │      │  Connectors  │     │ Monitoring    │  │
│  │ (Axum)     │      │              │     │ (Prometheus)  │  │
│  └────────────┘      └──────────────┘     └───────────────┘  │
│         │                    │                     │          │
│         │            ┌───────┴─────────┐           │          │
│         ├────────────┤  ParserRegistry │───────────┤          │
│         │            └───────┬─────────┘           │          │
│         │                    │                     │          │
│  ┌──────┴───────┐    ┌──────┴───────┐     ┌───────┴───────┐  │
│  │ RedisCache   │    │  ClickHouse  │     │ HealthMonitor │  │
│  │ (Latest)     │    │  (History)   │     │               │  │
│  └──────────────┘    └──────────────┘     └───────────────┘  │
│                                                                │
└──────────────────────────────────────────────────────────────┘
         ▲                    ▲                    ▲
         │                    │                    │
    ┌────┴────┐          ┌────┴────┐         ┌────┴────┐
    │ Binance │          │   OKX   │         │ Polygon │
    │  (CEX)  │          │  (CEX)  │         │ (Stock) │
    └─────────┘          └─────────┘         └─────────┘
```

### Core Components

#### 1. Models (`models/`)

**Purpose**: Define data structures

```rust
// Core data types
pub enum AssetType { Spot, Perpetual, Future, Option, Stock, Index }
pub enum DataSourceType { BinanceSpot, OkxSpot, ... }
pub enum MarketDataType { Trade, Ticker, Kline, OrderBook, FundingRate }

// Unified data structure
pub struct StandardMarketData {
    source: DataSourceType,
    symbol: String,
    price: Decimal,      // High precision, no f64!
    timestamp: i64,      // Milliseconds
    // ... more fields
}
```

**Key Design Decision**: Use `rust_decimal::Decimal` for all financial values to avoid floating-point precision issues.

#### 2. Traits (`traits/`)

**Purpose**: Define interfaces for extensibility

```rust
#[async_trait]
pub trait DataSourceConnector: Send + Sync {
    fn source_type(&self) -> DataSourceType;
    fn supported_assets(&self) -> Vec<AssetType>;
    async fn connect(&mut self) -> Result<mpsc::Receiver<StandardMarketData>>;
    async fn disconnect(&mut self) -> Result<()>;
    async fn is_healthy(&self) -> bool;
    fn stats(&self) -> ConnectorStats;
}

#[async_trait]
pub trait MessageParser: Send + Sync {
    fn source_type(&self) -> DataSourceType;
    async fn parse(&self, raw: &str) -> Result<Option<StandardMarketData>>;
    fn validate(&self, raw: &str) -> bool;
}
```

**Key Design Decision**: Use `async_trait` for async methods in traits (required until Rust stabilizes async fn in traits).

#### 3. Parser Registry (`registry/`)

**Purpose**: Route messages to appropriate parsers

```rust
pub struct ParserRegistry {
    parsers: HashMap<DataSourceType, Arc<dyn MessageParser>>,
}
```

**Performance**: O(1) lookup using HashMap.

#### 4. Storage Layer (`storage/`)

**Purpose**: Persist and cache data

**Redis**:
- Cache latest prices
- Key pattern: `market:{source}:{symbol}:latest`
- TTL: 24 hours (configurable)
- Use case: Fast latest price queries

**ClickHouse**:
- Store all historical data
- Batch inserts (1000 rows/flush, 5s intervals)
- Schema: `unified_ticks` table
- Partitioning: Daily partitions by timestamp
- Use case: Historical queries, analytics

#### 5. HTTP Server (`server/`)

**Purpose**: Expose APIs

**Routes**:
- `GET /health` - Health check with dependency status
- `GET /metrics` - Prometheus metrics
- `GET /api/v1/market/:symbol/latest` - Latest price
- `GET /api/v1/market/:symbol/history` - Historical data

**Framework**: Axum (type-safe, performant)

#### 6. Monitoring (`monitoring/`)

**Purpose**: Observability

**Metrics** (Prometheus):
- `data_engine_messages_received_total`
- `data_engine_messages_processed_total`
- `data_engine_errors_total`
- `data_engine_parse_latency_seconds`
- `data_engine_redis_latency_seconds`
- `data_engine_clickhouse_latency_seconds`
- `data_engine_service_up`

**Logging** (tracing):
- Structured JSON logs in production
- Pretty logs in development
- Instrumentation with `#[instrument]`

**Health**:
- Check Redis, ClickHouse, data freshness
- Return: Healthy / Degraded / Unhealthy

---

## Data Flow

### Message Processing Pipeline

```
1. Data Source (Exchange WebSocket)
        ↓
2. DataSourceConnector receives raw message
        ↓
3. Message sent to channel (mpsc)
        ↓
4. ParserRegistry routes to correct parser
        ↓
5. MessageParser converts to StandardMarketData
        ↓
6. Parallel distribution:
   ├─→ RedisCache.store_latest()    (< 5ms)
   └─→ ClickHouseWriter.write()     (batched)
        ↓
7. HTTP API serves queries
```

### Concurrency Model

```
┌──────────────────────────────────────────┐
│  Main Thread                              │
│  - HTTP Server (Axum)                     │
│  - Configuration loading                  │
│  - Graceful shutdown handling             │
└──────────────────────────────────────────┘
           │
           ├──────────────────────────────┐
           │                              │
┌──────────▼─────────┐       ┌────────────▼──────────┐
│  Connector Task    │       │  Auto-Flush Task       │
│  (per source)      │       │  (ClickHouse)          │
│  - WebSocket recv  │       │  - Periodic flush      │
│  - Message parsing │       │  - Error handling      │
│  - Channel send    │       └────────────────────────┘
└────────────────────┘
```

**Key Design Decision**: Each connector runs in its own task. This provides isolation - if one connector fails, others continue operating.

---

## Technology Stack

### Core Technologies

| Component | Technology | Version | Rationale |
|-----------|-----------|---------|-----------|
| Language | Rust | 1.75+ | Performance, safety, concurrency |
| Async Runtime | Tokio | 1.35+ | Industry standard, excellent ecosystem |
| HTTP Framework | Axum | 0.7+ | Type-safe, performant, ergonomic |
| WebSocket | tungstenite | 0.21+ | Lightweight, async-aware |
| Serialization | serde | 1.0+ | Zero-copy, type-safe |
| Decimals | rust_decimal | 1.33+ | Financial precision |
| Logging | tracing | 0.1+ | Structured, async-aware |
| Metrics | prometheus | 0.13+ | Industry standard |
| Error Handling | thiserror | 1.0+ | Ergonomic error definitions |
| Config | config | 0.14+ | Layered configuration |

### Storage Technologies

| Component | Technology | Use Case |
|-----------|-----------|----------|
| Cache | Redis | Latest prices, sub-ms latency |
| Time-Series DB | ClickHouse | Historical data, analytics |
| Message Queue | Kafka (future) | Load distribution, backpressure |

---

## Performance Considerations

### Sprint 2 Targets (MVP Baseline)

```
┌─────────────────────┬─────────────┬──────────────────┐
│ Metric              │ Target      │ Measurement      │
├─────────────────────┼─────────────┼──────────────────┤
│ Parser Latency      │ P95 < 50 μs │ Criterion bench  │
│ JSON Serialization  │ < 10 μs/msg │ Criterion bench  │
│ Redis Write         │ P95 < 5ms   │ Histogram metric │
│ ClickHouse Batch    │ >10k rows/s │ Throughput log   │
│ HTTP /health        │ < 100ms     │ Integration test │
│ HTTP /latest        │ < 10ms      │ Integration test │
│ Memory Usage        │ < 500MB     │ System monitor   │
│ CPU Usage           │ < 60%       │ System monitor   │
└─────────────────────┴─────────────┴──────────────────┘
```

### Optimization Techniques

#### 1. Zero-Copy Deserialization
Use `serde` with `&str` where possible to avoid allocations.

#### 2. Batch Processing
Buffer ClickHouse inserts (1000 rows) to amortize overhead.

#### 3. Connection Pooling
Reuse Redis connections via `ConnectionManager`.

#### 4. Async All the Way
No blocking operations in async tasks.

#### 5. Decimal Instead of Float
`Decimal` for financial values prevents precision loss.

### Bottleneck Analysis

```
Typical Message Processing Time Breakdown:
┌──────────────────────┬─────────────┐
│ WebSocket Receive    │ ~1 ms       │  (Network)
│ JSON Parse           │ ~20 μs      │  (CPU-bound)
│ Data Normalization   │ ~5 μs       │  (CPU-bound)
│ Redis Write          │ ~2 ms       │  (Network + I/O)
│ ClickHouse Buffer    │ ~1 μs       │  (Memory only)
├──────────────────────┼─────────────┤
│ Total E2E            │ ~3 ms       │  (Dominated by I/O)
└──────────────────────┴─────────────┘
```

**Insight**: Network I/O dominates. CPU is not the bottleneck.

---

## Scalability Roadmap

### Sprint 2 (Current): 10k msg/s

**Architecture**:
- Single instance
- Direct Redis/ClickHouse connections
- In-memory buffering

**Capacity**:
- 100 trading pairs × 100 msg/s = 10k msg/s
- Sufficient for MVP

### Sprint 4: 50k msg/s (5× scale)

**Optimizations**:
- Connection pooling (Redis + ClickHouse)
- Multi-threaded processing (Rayon)
- Larger batch sizes (5000 rows)
- Pipeline optimizations

**Risk**: 🟡 Medium (requires profiling)

### Sprint 6: 100k+ msg/s (10× scale)

**Architecture Changes**:
- Horizontal scaling (multiple instances)
- Kafka for message distribution
- Load balancer (Nginx/HAProxy)
- Sharded ClickHouse cluster
- Redis Cluster

**Deployment**:
```
Load Balancer
     ↓
┌────┴────┬────────┬────────┐
│ Engine1 │ Engine2 │ Engine3│
└────┬────┴────┬───┴────┬───┘
     └─────────┼────────┘
           Kafka
            ↓
      ClickHouse Cluster
```

**Risk**: 🟢 Low (architecture supports this)

---

## Security

### Authentication

**Sprint 2**: No authentication (internal service)  
**Future**: JWT tokens for external access

### Data Security

- **In Transit**: TLS for WebSocket connections
- **At Rest**: ClickHouse encryption (if configured)
- **Secrets**: Environment variables, Azure Key Vault

### Input Validation

- All user inputs validated
- Symbol names sanitized
- Query limits enforced (max 10k records)

### Rate Limiting

**Future Enhancement**: Rate limit API endpoints to prevent abuse

---

## Diagram: Complete System

```
External Sources                Data Engine                 Storage Layer
┌──────────────┐               ┌────────────────┐          ┌──────────────┐
│              │  WebSocket    │                │   Redis  │              │
│   Binance    ├──────────────→│  Connector     ├─────────→│  RedisCache  │
│              │               │                │          │ (Latest)     │
└──────────────┘               └────────┬───────┘          └──────────────┘
                                        │
┌──────────────┐               ┌────────▼───────┐          ┌──────────────┐
│              │  WebSocket    │                │ ClickHouse│             │
│     OKX      ├──────────────→│  Parser        ├─────────→│ ClickHouse   │
│              │               │  Registry      │          │ (Historical) │
└──────────────┘               └────────┬───────┘          └──────────────┘
                                        │
┌──────────────┐               ┌────────▼───────┐          ┌──────────────┐
│              │    HTTP       │                │          │              │
│   Polygon    ├──────────────→│  HTTP Server   │          │  Prometheus  │
│   (Stocks)   │               │  (Axum)        ├─────────→│  (Metrics)   │
└──────────────┘               └────────────────┘          └──────────────┘
```

---

## Best Practices

### 1. Error Handling

```rust
// ✅ Good: Propagate errors with context
async fn process_data(&self, raw: &str) -> Result<()> {
    let data = self.parser.parse(raw).await?;
    self.storage.write(data).await?;
    Ok(())
}

// ❌ Bad: Swallow errors
async fn process_data(&self, raw: &str) {
    let data = self.parser.parse(raw).await.unwrap();  // Panics!
}
```

### 2. Async Functions

```rust
// ✅ Good: Instrument for tracing
#[instrument(skip(self))]
async fn connect(&mut self) -> Result<()> {
    tracing::info!("Connecting to {}", self.url);
    // ...
}
```

### 3. Configuration

```rust
// ✅ Good: Environment variable overrides
DATA_ENGINE__REDIS__URL=redis://prod:6379 cargo run

// ✅ Good: Layered config (defaults → env file → env vars)
let config = AppConfig::load()?;
```

---

## Future Enhancements

1. **WebSocket Server**: Push real-time data to clients
2. **GraphQL API**: Flexible query interface
3. **Data Replay**: Historical data replay for backtesting
4. **Advanced Analytics**: Real-time indicators (RSI, MACD, etc.)
5. **Multi-Region Deployment**: Edge deployments for lower latency
6. **Machine Learning Integration**: Anomaly detection, pattern recognition

---

## References

- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [Tokio Documentation](https://tokio.rs/)
- [ClickHouse Documentation](https://clickhouse.com/docs)
- [Redis Documentation](https://redis.io/documentation)
- [Prometheus Best Practices](https://prometheus.io/docs/practices/)

---

**Document Version**: 1.0  
**Last Updated**: 2025-10-24  
**Next Review**: Sprint 3 Retrospective
