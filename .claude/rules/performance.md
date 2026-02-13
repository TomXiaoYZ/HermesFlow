---
description: Performance guidelines for latency-sensitive trading platform
globs: ["**/*.rs", "**/*.ts", "**/*.tsx"]
---

# Performance Guidelines

## Trading Path (< 10ms target)
- Minimize allocations in order execution path
- Pre-allocate buffers where possible
- Use zero-copy deserialization (serde borrow)
- Avoid unnecessary clones in hot paths
- Cache frequently accessed data in Redis

## Database
- All queries use appropriate indexes
- TimescaleDB hypertable ORDER BY matches query patterns
- Batch inserts for market data (not one-at-a-time)
- Connection pool sized for workload
- Avoid N+1 query patterns

## Redis
- Pipeline multiple commands
- Use appropriate data structures (sorted sets for time-series)
- Set TTL on cache entries
- Pub/sub for inter-service events, not polling

## Async Rust
- Don't block the Tokio runtime
- Use `spawn_blocking` for CPU-intensive work
- Limit concurrent connections appropriately
- Use channels (mpsc/broadcast) for internal communication

## Frontend
- Server Components by default (less client JS)
- Dynamic imports for heavy components
- Image optimization via next/image
- Minimize bundle size

## ClickHouse
- Batch inserts (never single-row)
- Use indexed columns in WHERE first
- Leverage materialized views for aggregations
- Partition by time (month)
