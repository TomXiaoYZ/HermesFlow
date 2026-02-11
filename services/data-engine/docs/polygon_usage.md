# Polygon.io US Stock Data - Usage Guide

## Setup

### 1. Get API Key
Sign up at [polygon.io](https://polygon.io) and get your API key

### 2. Configure Environment
```bash
# Copy example
cp .env.polygon.example .env

# Edit .env and add your API key
POLYGON_API_KEY=your_actual_api_key_here
POLYGON_ENABLED=true
POLYGON_WS_ENABLED=false  # WebSocket not yet implemented
POLYGON_RATE_LIMIT=5
```

### 3. Add to docker-compose.yml
```yaml
services:
  data-engine:
    environment:
      - POLYGON_API_KEY=${POLYGON_API_KEY}
      - POLYGON_ENABLED=${POLYGON_ENABLED:-false}
      - POLYGON_RATE_LIMIT=${POLYGON_RATE_LIMIT:-5}
```

---

## Basic Usage

### Fetch Historical Data (Rust)

```rust
use data_engine::collectors::polygon::{PolygonConfig, PolygonConnector};

#[tokio::main]
async fn main() {
    // Load config
    let config = PolygonConfig::from_env().unwrap();
    let connector = PolygonConnector::new(config);

    // Fetch 1 year of daily data
    let candles = connector
        .fetch_history_candles(
            "AAPL",      // ticker
            "1d",        // resolution
            "2023-01-01", // from
            "2023-12-31"  // to
        )
        .await
        .unwrap();

    println!("Fetched {} candles", candles.len());
}
```

### Sync to Database

```rust
use data_engine::collectors::polygon::{
    PolygonConfig, PolygonConnector, sync_polygon_history
};
use sqlx::PgPool;

#[tokio::main]
async fn main() {
    let pool = PgPool::connect("postgresql://localhost/hermesflow").await.unwrap();
    let config = PolygonConfig::from_env().unwrap();
    let connector = PolygonConnector::new(config);

    let tickers = vec!["AAPL".to_string(), "MSFT".to_string(), "GOOGL".to_string()];
    let resolutions = vec!["1h".to_string(), "1d".to_string()];

    sync_polygon_history(
        &pool,
        &connector,
        tickers,
        resolutions,
        "2024-01-01",
        "2024-12-31"
    )
    .await
    .unwrap();
}
```

---

## Testing

### Unit Tests
```bash
cd services/data-engine
cargo test polygon
```

### Integration Tests (Requires API Key)
```bash
# Set API key
export POLYGON_API_KEY=your_key

# Run ignored tests
cargo test --test polygon_integration -- --ignored
```

### Manual Testing via curl
```bash
# After syncing data, query via API
curl 'http://localhost:3000/api/v1/data/market/AAPL/history?resolution=1h&exchange=Polygon&limit=100' | jq
```

---

## Supported Resolutions

| Resolution | Polygon Format | Chunk Size | Bars/Day | Max Days/Request |
|------------|----------------|------------|----------|------------------|
| 1m         | 1/minute       | 7 days     | ~390     | 7                |
| 5m         | 5/minute       | 30 days    | ~78      | 30               |
| 15m        | 15/minute      | 90 days    | ~26      | 90               |
| 30m        | 30/minute      | 180 days   | ~13      | 180              |
| 1h         | 1/hour         | 365 days   | ~6.5     | 365              |
| 4h         | 4/hour         | 1000 days  | ~1.6     | 1000             |
| 1d         | 1/day          | 10000 days | ~1       | 10000            |

---

## Rate Limiting

Default: **5 requests/second** (configurable)

The client automatically:
- Enforces rate limits via token bucket
- Retries on 429 (rate limit exceeded)
- Adds delays between chunks (200ms)

### Adjust Rate Limit
```bash
# In .env
POLYGON_RATE_LIMIT=10  # For higher tier plans
```

---

## Database Schema

Data is stored in `mkt_equity_candles` table:

```sql
SELECT 
    time, symbol, resolution, 
    open, high, low, close, volume
FROM mkt_equity_candles
WHERE exchange = 'Polygon' AND symbol = 'AAPL'
ORDER BY time DESC
LIMIT 10;
```

---

## Common Issues

### "POLYGON_API_KEY not set"
- Ensure `.env` file has the key
- Reload environment: `source .env`

### "Rate limit exceeded"
- Reduce `POLYGON_RATE_LIMIT` in config
- Wait and retry (automatic retry built in)

### "No data returned"
- Check ticker symbol is valid (must be uppercase)
- Verify date range (markets closed on weekends/holidays)
- Check API plan includes historical data

---

## Next Steps

### Phase 3: Real-time WebSocket (Coming Soon)
- Live price updates during market hours
- Subscription management for multiple tickers
- Automatic reconnection

### Phase 4: Advanced Features
- Incremental sync (only fetch new data)
- Watchlist management UI
- Data quality monitoring
