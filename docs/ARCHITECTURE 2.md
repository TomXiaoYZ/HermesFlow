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

- **TimescaleDB (PostgreSQL)**: Primary store for relational data and time-series ticks.
- **ClickHouse**: Analytics store for heavy OLAP queries (Optional).
- **Redis**: Real-time cache for latest prices and session state.

## 4. Deployment

- **Containerization**: All services are Dockerized.
- **Orchestration**: `docker-compose.yml` for local development.
- **CI/CD**: GitHub Actions (`.github/workflows/ci.yml`).
