# Data Engine - Universal Data Framework

High-performance Rust-based data collection and distribution engine for HermesFlow.

## Overview

The Data Engine provides a unified framework for collecting, processing, and distributing real-time market data from multiple sources (cryptocurrency exchanges, traditional finance APIs, sentiment data, etc.).

### Key Features

- **Universal Framework**: Type-safe traits for easy integration of new data sources
- **High Performance**: μs-level latency, 10k+ messages/second throughput
- **Standardized Data Model**: Unified `StandardMarketData` structure across all sources
- **Storage Layer**: Redis for caching, ClickHouse for time-series storage
- **HTTP API**: Health checks, metrics, and query endpoints
- **Observability**: Prometheus metrics, structured logging, health monitoring

## Quick Start

### Prerequisites

- Rust 1.75+
- Redis (for caching)
- ClickHouse (for storage)

### Installation

```bash
cargo build --release
```

### Configuration

Create a `config/dev.toml` file:

```toml
[server]
host = "0.0.0.0"
port = 8080

[redis]
url = "redis://localhost:6379"

[clickhouse]
url = "tcp://localhost:9000"
database = "hermesflow"

[logging]
level = "info"
format = "json"
```

### Running

```bash
# Set environment
export RUST_ENV=dev

# Run the service
cargo run --release
```

The service will start on `http://localhost:8080`.

## API Endpoints

### Health Check
```bash
GET /health

Response:
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_secs": 3600,
  "dependencies": {
    "redis": { "status": "up", "latency_ms": 2.5 },
    "clickhouse": { "status": "up", "latency_ms": 5.0 }
  }
}
```

### Metrics (Prometheus)
```bash
GET /metrics

Response: (Prometheus text format)
data_engine_messages_received_total 12345
data_engine_messages_processed_total 12340
...
```

### Latest Price
```bash
GET /api/v1/market/BTCUSDT/latest

Response:
{
  "symbol": "BTCUSDT",
  "price": "50000.12345678",
  "timestamp": 1234567890000,
  "source": "BinanceSpot",
  "bid": "49999.00",
  "ask": "50001.00"
}
```

### Historical Data
```bash
GET /api/v1/market/BTCUSDT/history?start=1234567890000&limit=1000

Response:
{
  "symbol": "BTCUSDT",
  "data": [
    {
      "timestamp": 1234567890000,
      "price": "50000.00",
      "quantity": "0.1",
      "source": "BinanceSpot"
    }
  ],
  "count": 1000
}
```

## Architecture

### Core Components

```
data-engine/
├── models/          # Data models (AssetType, StandardMarketData, etc.)
├── traits/          # Core traits (DataSourceConnector, MessageParser)
├── registry/        # Parser registry for routing
├── storage/         # Redis and ClickHouse integrations
├── server/          # HTTP API (Axum)
├── monitoring/      # Health, metrics, logging
├── config/          # Configuration management
└── error/           # Error types and retry logic
```

### Data Flow

```
Data Source (Exchange)
    ↓ (WebSocket/HTTP)
DataSourceConnector
    ↓ (Raw Messages)
MessageParser
    ↓ (Standardized Data)
ParserRegistry
    ↓
├─→ RedisCache (latest prices)
└─→ ClickHouseWriter (historical storage)
```

## Adding a New Data Source

See `docs/guides/adding-new-data-source.md` for a detailed guide.

### Quick Example

```rust
use async_trait::async_trait;
use data_engine::*;

struct MyConnector;

#[async_trait]
impl DataSourceConnector for MyConnector {
    fn source_type(&self) -> DataSourceType {
        DataSourceType::OkxSpot
    }

    fn supported_assets(&self) -> Vec<AssetType> {
        vec![AssetType::Spot]
    }

    async fn connect(&mut self) -> Result<mpsc::Receiver<StandardMarketData>> {
        // Implementation here
    }

    // ... other methods
}
```

## Performance

### Targets (Sprint 2 - MVP)

- Parser latency: P95 < 50 μs
- JSON serialization: < 10 μs/msg
- HTTP /health: < 100ms
- HTTP /latest: < 10ms (Redis)
- Redis latency: P95 < 5ms
- ClickHouse batch insert: > 10k rows/s

### Running Benchmarks

```bash
cargo bench
```

## Testing

### Unit Tests
```bash
cargo test
```

### Integration Tests
```bash
cargo test --test extensibility_test
```

### Test Coverage
```bash
cargo tarpaulin --out Html --output-dir coverage
```

Target: ≥ 85% coverage

## Development

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Build
cargo build --release
```

### Environment Variables

- `RUST_ENV`: Environment (dev/prod)
- `DATA_ENGINE__SERVER__PORT`: Override server port
- `DATA_ENGINE__REDIS__URL`: Override Redis URL
- `DATA_ENGINE__CLICKHOUSE__URL`: Override ClickHouse URL

## Troubleshooting

### Redis Connection Failed
```bash
# Check Redis is running
redis-cli ping

# Check connection string
echo $DATA_ENGINE__REDIS__URL
```

### ClickHouse Connection Failed
```bash
# Check ClickHouse is running
clickhouse-client --query "SELECT 1"

# Check database exists
clickhouse-client --query "SHOW DATABASES"
```

### High Memory Usage
- Check batch size configuration
- Monitor metrics at `/metrics`
- Review ClickHouse flush interval

## Documentation

- [Architecture Guide](../../docs/architecture/data-engine-architecture.md)
- [Adding New Data Sources](../../docs/guides/adding-new-data-source.md)
- [Performance Scaling Roadmap](../../docs/architecture/performance-scaling-roadmap.md)

## License

Internal use only - HermesFlow Project

## Version

Current version: 0.1.0 (Sprint 2 - Universal Framework MVP)
