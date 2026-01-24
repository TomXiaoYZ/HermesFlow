# HermesFlow System Architecture

## 1. High Level Overview

HermesFlow is a high-performance quantitative trading platform using a hybrid **Rust + Python** architecture.

```mermaid
graph TD
    User[Client / UI] --> Gateway[API Gateway (Rust)]
    Gateway --> DataEngine[Data Engine (Rust)]
    Gateway --> Strat[Strategy Engine (Python)]
    Gateway --> Risk[Risk Engine (Python)]
    
    DataEngine --> Redis[(Redis Cache)]
    DataEngine --> TSDB[(TimescaleDB)]
    DataEngine --> CH[(ClickHouse)]
    
    Strat --> Risk
    Strat --> DataEngine
```

## 2. Core Components

### 2.1 API Gateway (Rust)
- **Tech**: Axum, Tokio
- **Role**: Single entry point, Authentication, Rate Limiting, Request Routing.
- **Port**: 8080 (Internal), 3000 (Exposed via Docker)

### 2.2 Data Engine (Rust)
- **Tech**: Rust, SQLx, Tokio
- **Role**: 
  - Connects to Exchanges (IBKR, Binance, AkShare).
  - Normalizes market data.
  - Persists ticks/candles to database.
- **Pattern**: Repository Pattern for database abstraction.

### 2.3 Strategy Engine (Python)
- **Tech**: Python 3.11, Pandas, FastAPI
- **Role**: 
  - Runs quantitative strategies.
  - Backtesting.
  - Generates signals.
- **Shared Lib**: Uses `infrastructure/python/hermes_common`.

### 2.4 Risk Engine (Python)
- **Tech**: Python 3.11, FastAPI
- **Role**:
  - Pre-trade checks.
  - Position monitoring.
  - P&L calculation.

## 3. Data Infrastructure

## 3. Data Infrastructure

### 3.1 TimescaleDB (Time-Series Store)
- **Role**: Primary store for all market data (Ticks, Candles, Snapshots).
- **Rationale**: Selected for efficient partitioning (Hypertables) and compression. See [ADR-001](ADR/001_timescaledb_selection.md).
- **Schema**:
  - `mkt_equity_snapshots` (Hypertable)
  - `mkt_equity_candles` (Hypertable)

### 3.2 Redis (Real-time Cache)
- **Role**: Pub/Sub channel for live market stream (`market.stream.*`) and latest price cache.

### 3.3 ClickHouse (Optional)
- **Role**: Reserved for heavy OLAP analytics if TimescaleDB query performance degrades for large-scale backtesting.

## 4. Supported Data Sources (Phase 3)

| Source | Asset | Type | Protocol |
| :--- | :--- | :--- | :--- |
| **Binance** | Crypto | Trade/Candle | WS/REST |
| **OKX** | Crypto | Trade | WS (V5) |
| **Bybit** | Crypto | Trade | WS (V5) |
| **IBKR** | US Stock | Candle | TCP Gateway |
| **AkShare** | A-Share | Snapshot | HTTP Polling |
| **Massive** | US Stock | Candle | HTTP REST |

## 5. Deployment

- **Containerization**: All services are Dockerized.
- **Orchestration**: `docker-compose.yml` for local development.

## Appendix A: Architecture Decision Records (ADR)

### ADR-001: Adoption of TimescaleDB (2026-01-17)

**Context**: Need high-performance storage for billions of market data rows.
**Decision**: Selected **TimescaleDB** (Self-Hosted on ECS for Dev/Staging, Managed Cloud for Prod).
**Rationale**:
1.  **Write Performance**: Hypertables maintain constant ingest rates vs Vanilla Postgres bloat.
2.  **Compression**: Columnar compression saves ~90% storage costs.
3.  **Operations**: Self-hosted on ECS requires attached EBS persistence and automated snapshots (DLM).
