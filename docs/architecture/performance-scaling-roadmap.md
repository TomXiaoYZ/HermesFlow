# Performance Scaling Roadmap

**Document Version**: 1.0  
**Date**: 2025-10-24  
**Status**: Planning Document

---

## Table of Contents

1. [Overview](#overview)
2. [Sprint 2: MVP Baseline (10k msg/s)](#sprint-2-mvp-baseline-10k-msgs)
3. [Sprint 4: Optimization Phase (50k msg/s)](#sprint-4-optimization-phase-50k-msgs)
4. [Sprint 6: Production Scale (100k+ msg/s)](#sprint-6-production-scale-100k-msgs)
5. [Performance Testing Strategy](#performance-testing-strategy)
6. [Monitoring & Alerting](#monitoring--alerting)

---

## Overview

This document outlines the incremental performance optimization strategy for the Data Engine, from MVP baseline (10k msg/s) to production scale (100k+ msg/s).

### Philosophy

**"Premature optimization is the root of all evil" - Donald Knuth**

We follow a **measure-optimize-measure** approach:
1. Build with best practices (Sprint 2)
2. Measure actual performance under load
3. Profile to find bottlenecks
4. Optimize based on data
5. Repeat

### Target Use Cases

| Sprint | msg/s | Trading Pairs | msg/pair/s | Use Case |
|--------|-------|---------------|------------|----------|
| Sprint 2 | 10k | 100 | 100 | MVP users, single exchange |
| Sprint 4 | 50k | 500 | 100 | Multiple exchanges, active traders |
| Sprint 6 | 100k+ | 1000+ | 100+ | Production scale, institutional |

---

## Sprint 2: MVP Baseline (10k msg/s)

### Architecture

```
Single Data Engine Instance
    ↓
Redis (Single Instance)
    ↓
ClickHouse (Single Instance)
```

### Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| **Throughput** | > 10k msg/s | Sustained for 1 hour |
| **Parser Latency** | P95 < 50 μs | JSON parsing + validation |
| **Serialization** | < 10 μs/msg | serde JSON |
| **Redis Write** | P95 < 5ms | Including network latency |
| **ClickHouse Batch** | > 10k rows/s | Batch insert throughput |
| **E2E Latency** | P99 < 20ms | Exchange → storage |
| **Memory** | < 500MB | Steady state |
| **CPU** | < 60% | Single core or distributed |

### Characteristics

**What's Included**:
- Single-threaded message processing
- Basic batching (1000 rows/batch)
- No connection pooling
- Direct Redis/ClickHouse connections
- In-memory channel buffering (10k messages)

**What's NOT Included**:
- No multi-threading
- No connection pooling
- No horizontal scaling
- No advanced caching strategies

### Capacity Analysis

```
Assumptions:
- 100 trading pairs
- 100 messages per pair per second
- Total: 100 × 100 = 10,000 msg/s

Resource Usage (estimated):
- Network: ~5 Mbps (500 bytes/msg × 10k msg/s)
- CPU: ~50% single core
- Memory: ~300MB steady state
- Redis: ~1k ops/s (latest price updates)
- ClickHouse: 10k inserts/s (batched)
```

### Bottleneck Analysis

**Expected Bottlenecks** (in order of impact):
1. **Network I/O** (3-5ms) - Dominates E2E latency
2. **Redis Write** (1-3ms) - Network + I/O
3. **JSON Parsing** (20-50μs) - CPU-bound
4. **Data Transformation** (5-10μs) - CPU-bound

**NOT a Bottleneck**:
- CPU processing (plenty of headroom)
- Memory allocation (Rust is efficient)

### Validation Method

```bash
# Load test with 100 trading pairs
./load-test.sh \
  --pairs 100 \
  --msg-rate 100 \
  --duration 3600

# Monitor metrics
curl http://localhost:8080/metrics | grep data_engine_messages_processed_total

# Check resource usage
docker stats data-engine
```

---

## Sprint 4: Optimization Phase (50k msg/s)

**Target**: 5× throughput improvement

### Architecture Changes

```
Single Data Engine Instance (optimized)
    ↓
Redis Connection Pool (10 connections)
    ↓
ClickHouse Connection Pool (5 connections)
```

### Optimization Techniques

#### 1. Connection Pooling

**Before**:
```rust
// Direct connection per operation
let mut conn = client.get_connection()?;
conn.set(key, value)?;
```

**After**:
```rust
// Connection pool
let pool = ConnectionPool::new(10);
let mut conn = pool.get().await?;
conn.set(key, value)?;
```

**Impact**: Reduce connection overhead by ~30-40%

#### 2. Multi-threaded Processing

**Before**:
```rust
// Single-threaded parsing
for msg in messages {
    let parsed = parser.parse(msg).await?;
    channel.send(parsed).await?;
}
```

**After**:
```rust
// Parallel parsing with Rayon
use rayon::prelude::*;

messages.par_iter()
    .filter_map(|msg| parser.parse(msg).ok())
    .for_each(|parsed| {
        channel.try_send(parsed).ok();
    });
```

**Impact**: Reduce CPU bottleneck by 3-4×

#### 3. Larger Batch Sizes

**Before**: 1000 rows/batch, 5s flush interval

**After**: 5000 rows/batch, 3s flush interval

**Impact**: Reduce ClickHouse insertion overhead by ~40%

#### 4. Pipeline Optimizations

```rust
// Before: Sequential
parse → validate → transform → write

// After: Pipelined
parse → (validate + transform) → write
```

**Impact**: Reduce overall latency by ~20%

### Performance Targets

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Throughput | 10k msg/s | 50k msg/s | **5×** |
| Parser Latency | 50 μs | 30 μs | 1.7× |
| Redis Write | 3ms | 2ms | 1.5× |
| Memory | 500MB | 800MB | +300MB |
| CPU | 60% | 75% | +15% |

### Implementation Plan

**Week 1** (Sprint 4.1):
- [ ] Add connection pooling (Redis + ClickHouse)
- [ ] Benchmark and measure impact
- [ ] Load test at 20k msg/s

**Week 2** (Sprint 4.2):
- [ ] Implement multi-threaded parsing (Rayon)
- [ ] Increase batch sizes
- [ ] Load test at 40k msg/s

**Week 3** (Sprint 4.3):
- [ ] Pipeline optimizations
- [ ] Final load test at 50k msg/s
- [ ] Stability test (24 hours)

### Risk Assessment

| Risk | Severity | Probability | Mitigation |
|------|----------|-------------|------------|
| CPU becomes bottleneck | 🟡 Medium | 40% | Profile first, optimize hot paths |
| Memory leaks | 🟢 Low | 20% | Use `valgrind`, monitor metrics |
| Race conditions | 🟡 Medium | 30% | Thorough testing, use thread-safe types |
| Regression in latency | 🟢 Low | 25% | Benchmark before/after, have rollback plan |

---

## Sprint 6: Production Scale (100k+ msg/s)

**Target**: 10× throughput from baseline (2× from Sprint 4)

### Architecture Changes

```
Load Balancer (Nginx)
    ↓
┌──────────┬──────────┬──────────┐
│ Engine 1 │ Engine 2 │ Engine 3 │ (Horizontal scaling)
└──────────┴──────────┴──────────┘
    ↓           ↓           ↓
    └───────────┴───────────┘
            Kafka
             ↓
    Redis Cluster (3 nodes)
             ↓
    ClickHouse Cluster (3 shards)
```

### Major Architectural Shifts

#### 1. Horizontal Scaling

**Multiple Data Engine Instances**:
- 3-5 instances behind load balancer
- Each handles 20-30k msg/s
- Stateless design (no shared state)
- Independent failure domains

#### 2. Kafka Integration

**Purpose**: Message distribution and backpressure handling

```rust
// Producer (Data Engine)
kafka_producer.send(StandardMarketData)?;

// Consumer (Storage Workers)
for msg in kafka_consumer.poll() {
    redis.store(msg)?;
    clickhouse.write(msg)?;
}
```

**Benefits**:
- Decouple data collection from storage
- Buffer during storage slowdowns
- Replay capability for recovery
- Exactly-once semantics

#### 3. Redis Cluster

**Configuration**:
- 3 master nodes
- 3 replica nodes
- Hash slot distribution
- Automatic failover

**Capacity**:
- ~30k ops/s per node
- Total: 90k ops/s

#### 4. ClickHouse Cluster

**Configuration**:
- 3 shards (horizontal partitioning)
- 2 replicas per shard (high availability)
- Distributed tables with sharding key
- Automatic data distribution

**Schema**:
```sql
CREATE TABLE unified_ticks_distributed ON CLUSTER 'hermesflow'
AS unified_ticks
ENGINE = Distributed('hermesflow', 'default', 'unified_ticks', rand());
```

**Capacity**:
- ~50k inserts/s per shard
- Total: 150k inserts/s

### Performance Targets

| Metric | Sprint 4 | Sprint 6 | Improvement |
|--------|----------|----------|-------------|
| Throughput | 50k msg/s | 120k msg/s | **2.4×** |
| Horizontal Scalability | N/A | Yes (3-5 instances) | ∞ |
| High Availability | Single point of failure | Multi-node | ✅ |
| Recovery Time | Manual | Automatic (< 30s) | ✅ |

### Deployment Topology

```
Azure Kubernetes Service (AKS)

┌─────────────────────────────────────────────┐
│  Namespace: hermesflow                       │
│                                              │
│  ┌────────────────────────────────────────┐ │
│  │  LoadBalancer Service                   │ │
│  │  (External IP)                          │ │
│  └──────────┬──────────────────────────────┘ │
│             ↓                                │
│  ┌──────────┴──────────────────────────────┐ │
│  │  Data Engine Deployment (3 replicas)    │ │
│  │  - Pod 1: data-engine-xxxx              │ │
│  │  - Pod 2: data-engine-yyyy              │ │
│  │  - Pod 3: data-engine-zzzz              │ │
│  └──────────┬──────────────────────────────┘ │
│             ↓                                │
│  ┌──────────┴──────────────────────────────┐ │
│  │  Kafka StatefulSet (3 brokers)          │ │
│  └──────────┬──────────────────────────────┘ │
│             ↓                                │
│  ┌──────────┴──────────────────────────────┐ │
│  │  Redis Cluster (6 pods)                  │ │
│  └──────────┬──────────────────────────────┘ │
│             ↓                                │
│  ┌──────────┴──────────────────────────────┐ │
│  │  ClickHouse Cluster (6 pods)             │ │
│  └──────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
```

### Implementation Plan

**Months 1-2** (Pre-Sprint 6):
- [ ] Set up Kafka cluster
- [ ] Implement Kafka producer in Data Engine
- [ ] Create storage consumer workers
- [ ] Test message distribution

**Sprint 6.1** (Week 1-2):
- [ ] Deploy multiple Data Engine instances
- [ ] Implement load balancing
- [ ] Test horizontal scaling (3 instances)

**Sprint 6.2** (Week 3-4):
- [ ] Deploy Redis Cluster
- [ ] Migrate to cluster-aware client
- [ ] Test failover scenarios

**Sprint 6.3** (Week 5-6):
- [ ] Deploy ClickHouse Cluster
- [ ] Migrate to distributed tables
- [ ] Load test at 100k msg/s
- [ ] 48-hour stability test

### Cost Analysis

| Component | Sprint 2 | Sprint 6 | Monthly Cost |
|-----------|----------|----------|--------------|
| Data Engine | 1 VM (2 vCPU) | 3 pods (2 vCPU each) | +$200 |
| Redis | 1 instance | Cluster (6 nodes) | +$800 |
| ClickHouse | 1 VM | Cluster (6 nodes) | +$1500 |
| Kafka | N/A | 3 brokers | +$600 |
| **Total** | ~$300/mo | ~$3400/mo | **+$3100/mo** |

**ROI**: Supports 10× more users, enabling revenue growth

---

## Performance Testing Strategy

### Load Testing Tools

**Primary**: Custom Rust-based load generator
**Alternative**: `wrk`, `hey`, `artillery`

### Test Scenarios

#### Scenario 1: Baseline (10k msg/s)
```bash
./load-test \
  --pairs 100 \
  --rate 100 \
  --duration 3600 \
  --output baseline.json
```

#### Scenario 2: Burst (30k msg/s)
```bash
./load-test \
  --pairs 300 \
  --rate 100 \
  --duration 300 \
  --output burst.json
```

#### Scenario 3: Stress (Until Failure)
```bash
./load-test \
  --pairs 1000 \
  --rate 200 \
  --duration 600 \
  --output stress.json
```

### Metrics Collection

```bash
# Export Prometheus metrics during test
curl http://localhost:8080/metrics > metrics_$(date +%s).txt

# Monitor resources
docker stats --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}" > resources.log

# ClickHouse insert rate
clickhouse-client --query "
  SELECT
    toStartOfMinute(ingested_at) AS minute,
    count() AS inserts
  FROM unified_ticks
  WHERE ingested_at > now() - INTERVAL 1 HOUR
  GROUP BY minute
  ORDER BY minute
"
```

---

## Monitoring & Alerting

### Key Metrics

| Metric | Threshold | Action |
|--------|-----------|--------|
| `data_engine_errors_total` | > 100/min | Alert: High error rate |
| `data_engine_service_up` | < 1 | Alert: Service down |
| CPU Usage | > 80% | Warning: Consider scaling |
| Memory Usage | > 1GB | Warning: Check for leaks |
| Redis Latency | P95 > 10ms | Warning: Redis slow |
| ClickHouse Latency | P95 > 200ms | Warning: ClickHouse slow |

### Alerting Rules (Prometheus)

```yaml
groups:
  - name: data_engine
    rules:
      - alert: HighErrorRate
        expr: rate(data_engine_errors_total[5m]) > 100
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }} errors/sec"

      - alert: ServiceDown
        expr: data_engine_service_up == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Data Engine is down"

      - alert: HighMemoryUsage
        expr: process_resident_memory_bytes > 1073741824  # 1GB
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Memory usage exceeds 1GB"
```

### Dashboards

**Grafana Dashboard** (key panels):
1. **Throughput**: Messages processed per second
2. **Latency**: P50, P95, P99 latency histograms
3. **Error Rate**: Errors per minute by type
4. **Resource Usage**: CPU, Memory, Network
5. **Storage**: Redis ops/s, ClickHouse inserts/s
6. **Health**: Service up, dependency status

---

## Conclusion

This roadmap provides a clear path from MVP (10k msg/s) to production scale (100k+ msg/s) through:

1. **Sprint 2**: Build with best practices, establish baseline
2. **Sprint 4**: Optimize based on measurements (5× improvement)
3. **Sprint 6**: Scale horizontally (10× improvement total)

**Key Takeaway**: The architecture is designed to support this growth path without fundamental rewrites. Each phase builds incrementally on the previous one.

---

**Document Status**: Living Document  
**Next Review**: After Sprint 2 load testing  
**Owner**: Data Engine Team






