# Data Engine - Universal Data Framework

High-performance Rust-based data collection and distribution engine for HermesFlow with support for market data, social media, and prediction markets.

## Overview

The Data Engine provides a unified framework for collecting, processing, and distributing real-time data from multiple sources including cryptocurrency exchanges, traditional finance APIs, social media (Twitter/X), and prediction markets (Polymarket).

### Key Features

- **Universal Framework**: Type-safe traits for easy integration of new data sources
- **High Performance**: μs-level latency, 10k+ messages/second throughput
- **Multi-Source Support**: Market data, social media scraping, prediction markets
- **Standardized Data Models**: Unified structures across all data types
- **Hybrid Storage**: Postgres (relational/social), ClickHouse (time-series), Redis (cache)
- **Native Scrapers**: Built-in Twitter scraper using headless Chrome
- **HTTP API**: Health checks, metrics, and query endpoints
- **Observability**: Prometheus metrics, structured logging, health monitoring

## Supported Data Sources

### Market Data
- Cryptocurrency exchanges (Binance, OKX, Bitget)
- Traditional finance (IBKR, Polygon, Alpaca)
- DEX protocols (Uniswap, etc.)

### Social Media
- **Twitter/X**: Native Rust scraper using headless Chrome for timeline and search scraping

### Prediction Markets
- **Polymarket**: Real-time market data via Gamma API

## Quick Start

### Prerequisites

- Rust 1.75+
- PostgreSQL (for social/prediction data)
- ClickHouse (for high-frequency market data)
- Redis (for caching)
- Chrome/Chromium (for Twitter scraping)

### Installation

```bash
cargo build --release
```

### Configuration

Create a `config/dev.toml` or `config/prod.toml` file:

```toml
[server]
host = "0.0.0.0"
port = 8080

[redis]
url = "redis://localhost:6379"

[postgres]
host = "your-postgres-host"
port = 5432
database = "main"
username = "postgres"
password = "your-password"

[clickhouse]
url = "tcp://localhost:9000"
database = "hermesflow"

# Optional: Twitter Configuration
[twitter]
username = "your_username"
email = "your_email@example.com"
password = "your_password"
max_tweets_per_session = 1000
headless = true

# Optional: Polymarket Configuration
[polymarket]
api_base_url = "https://gamma-api.polymarket.com"
poll_interval_secs = 60
tracked_markets = []

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
├── collectors/      # Data collectors (Twitter, Polymarket)
├── models/          # Data models (MarketData, SocialData, PredictionMarket)
├── traits/          # Core traits (DataSourceConnector, MessageParser)
├── registry/        # Parser registry for routing
├── storage/         # Postgres, ClickHouse, and Redis integrations
├── server/          # HTTP API (Axum)
├── monitoring/      # Health, metrics, logging
├── config/          # Configuration management
└── error/           # Error types and retry logic
```

### Data Flow

```
┌─────────────────┐
│  Data Sources   │
├─────────────────┤
│ • Exchanges     │
│ • Twitter/X     │
│ • Polymarket    │
└────────┬────────┘
         │
    ┌────▼────────────────┐
    │   Collectors        │
    │ - TwitterCollector  │
    │ - PolymarketCollector│
    │ - ExchangeConnector │
    └────────┬────────────┘
             │
    ┌────────▼────────────┐
    │ Data Processing     │
    │ - Parsing           │
    │ - Validation        │
    │ - Transformation    │
    └────────┬────────────┘
             │
    ┌────────▼──────────────────┐
    │  Storage Layer            │
    ├───────────────────────────┤
    │ Postgres (Social/Markets) │
    │ ClickHouse (Time-series)  │
    │ Redis (Cache)             │
    └───────────────────────────┘
```

## Database Schemas

### Postgres Tables

#### tweets
Stores Twitter/X social media data:
- `id`: Tweet ID (bigint, PK)
- `username`: Twitter username
- `text`: Tweet content
- `created_at`: Tweet timestamp
- `engagement metrics`: retweet_count, favorite_count, etc.
- `hashtags`, `media_urls`: Arrays
- `raw_data`: JSONB for full tweet data

#### prediction_markets
Stores prediction market metadata:
- `id`: Market ID (text, PK)
- `source`: Data source (e.g., "Polymarket")
- `title`: Market question
- `description`, `category`: Market details
- `end_date`: Market close time
- `metadata`: JSONB for additional data

#### market_outcomes
Stores market outcome prices over time:
- `market_id`: FK to prediction_markets
- `outcome`: Outcome name
- `price`: Current probability (0-1)
- `volume_24h`: 24-hour trading volume
- `timestamp`: Data point timestamp

### ClickHouse Tables

Used for high-frequency market tick data (existing schema).

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
