# DATA-001B: Binance WebSocket Implementation

**Status**: ✅ **READY** - Ready for Sprint 3  
**Version**: 2.0 (Updated for Sprint 3)  
**Date**: 2025-11-02  
**Epic**: Epic 1 - Cryptocurrency Data Collection  
**Story Points**: 5 SP  
**Priority**: P0 🔴 Critical  
**Sprint**: Sprint 3 (Nov 11 - Nov 22, 1.5 weeks)

---

## 📋 Story Overview

### User Story

**As a** cryptocurrency trader using HermesFlow  
**I want** real-time Binance market data (spot, futures, perpetual)  
**So that** I can make informed trading decisions based on live prices, volumes, and order book data

### Business Value

- **MVP Completion**: First live data source operational
- **Market Coverage**: Binance (largest CEX by volume, 60%+ market share)
- **Data Types**: Trade, ticker, kline (candlestick), order book
- **Asset Types**: Spot, futures, perpetual
- **Latency**: < 100ms from exchange to storage (E2E)
- **Throughput**: 10k+ messages/second validated
- **Stability**: 99.9% uptime, 24-hour continuous operation

### Dependencies

| Dependency | Status | Notes |
|------------|--------|-------|
| **DATA-001A Complete** | ✅ **完成** | Framework fully operational (2025-11-02) |
| **Framework Validated** | ✅ **完成** | Mock OKX test passed (3/3 integration tests) |
| **Redis Production** | 🟡 Needed | Azure Redis Cache or self-hosted |
| **ClickHouse Production** | 🟡 Needed | Azure VM or managed service |
| **Binance API Access** | ✅ Ready | Public WebSocket API (no auth for market data) |

### Scope

✅ **In Scope** (Sprint 3):
1. `BinanceConnector` implementation (WebSocket streams)
2. `BinanceParser` implementation (trade, ticker, kline, depth)
3. Data normalization and quality control
4. Redis caching integration (latest prices)
5. ClickHouse persistence integration
6. Reconnection logic with exponential backoff
7. Integration tests (end-to-end data flow)
8. Load testing (10k msg/s validation)
9. 24-hour stability test
10. Production deployment

❌ **Out of Scope** (Deferred):
- OKX connector (DATA-002, same sprint or Sprint 4)
- Options data (Epic 2, Sprint 5+)
- Advanced analytics (indicators, signals)
- WebSocket server for clients (Epic 7, Sprint 5+)
- Performance optimization >10k msg/s (Sprint 4)

---

## 🎯 Acceptance Criteria

### AC-1: BinanceConnector Implementation ✅

**Given** the universal data framework is ready  
**When** implementing Binance WebSocket connector  
**Then** the following structure must be used:

```rust
use async_trait::async_trait;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};

pub struct BinanceConnector {
    config: BinanceConfig,
    ws_stream: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    parser: Arc<BinanceParser>,
    stats: Arc<RwLock<ConnectorStats>>,
}

#[derive(Debug, Clone)]
pub struct BinanceConfig {
    pub ws_url: String,              // "wss://stream.binance.com:9443/ws"
    pub symbols: Vec<String>,        // ["btcusdt", "ethusdt"]
    pub streams: Vec<StreamType>,    // [Trade, Ticker, Kline1m, Depth5]
    pub asset_type: AssetType,       // Spot, Futures, Perpetual
}

#[derive(Debug, Clone)]
pub enum StreamType {
    Trade,          // @trade
    Ticker,         // @ticker
    Kline1m,        // @kline_1m
    Kline5m,        // @kline_5m
    Depth5,         // @depth5
    Depth20,        // @depth20
}

impl BinanceConnector {
    pub fn new(config: BinanceConfig, parser: Arc<BinanceParser>) -> Self {
        Self {
            config,
            ws_stream: None,
            parser,
            stats: Arc::new(RwLock::new(ConnectorStats::default())),
        }
    }
    
    async fn build_subscription_message(&self) -> String {
        let streams: Vec<String> = self.config.symbols.iter()
            .flat_map(|symbol| {
                self.config.streams.iter().map(move |stream| {
                    format!("{}@{}", symbol.to_lowercase(), stream.to_stream_name())
                })
            })
            .collect();
        
        json!({
            "method": "SUBSCRIBE",
            "params": streams,
            "id": 1
        }).to_string()
    }
}

#[async_trait]
impl DataSourceConnector for BinanceConnector {
    fn source_type(&self) -> DataSourceType {
        match self.config.asset_type {
            AssetType::Spot => DataSourceType::BinanceSpot,
            AssetType::Perpetual => DataSourceType::BinancePerp,
            AssetType::Future => DataSourceType::BinanceFutures,
            _ => unreachable!(),
        }
    }
    
    fn supported_assets(&self) -> Vec<AssetType> {
        vec![self.config.asset_type]
    }
    
    async fn connect(&mut self) -> Result<mpsc::Receiver<StandardMarketData>> {
        let (tx, rx) = mpsc::channel(10000);
        
        // Connect to Binance WebSocket
        let (ws_stream, response) = connect_async(&self.config.ws_url).await?;
        info!("Connected to Binance WebSocket: {:?}", response.status());
        
        self.ws_stream = Some(ws_stream);
        
        // Subscribe to streams
        let subscription = self.build_subscription_message().await;
        self.ws_stream.as_mut().unwrap()
            .send(Message::Text(subscription))
            .await?;
        
        // Spawn message processing task
        let parser = self.parser.clone();
        let stats = self.stats.clone();
        let mut stream = self.ws_stream.take().unwrap();
        
        tokio::spawn(async move {
            loop {
                match stream.next().await {
                    Some(Ok(Message::Text(text))) => {
                        stats.write().await.messages_received += 1;
                        
                        match parser.parse(&text).await {
                            Ok(Some(data)) => {
                                if tx.send(data).await.is_err() {
                                    error!("Receiver dropped, shutting down");
                                    break;
                                }
                                stats.write().await.messages_processed += 1;
                            }
                            Ok(None) => {} // Heartbeat or non-market message
                            Err(e) => {
                                error!("Parse error: {}", e);
                                stats.write().await.errors += 1;
                            }
                        }
                    }
                    Some(Ok(Message::Ping(payload))) => {
                        stream.send(Message::Pong(payload)).await.ok();
                    }
                    Some(Ok(Message::Close(_))) => {
                        warn!("WebSocket closed by server");
                        break;
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error: {}", e);
                        stats.write().await.errors += 1;
                        break;
                    }
                    None => {
                        warn!("WebSocket stream ended");
                        break;
                    }
                    _ => {}
                }
            }
        });
        
        Ok(rx)
    }
    
    async fn disconnect(&mut self) -> Result<()> {
        if let Some(mut stream) = self.ws_stream.take() {
            stream.close(None).await?;
        }
        Ok(())
    }
    
    async fn is_healthy(&self) -> bool {
        self.ws_stream.is_some()
    }
    
    fn stats(&self) -> ConnectorStats {
        // Return copy of stats
        self.stats.blocking_read().clone()
    }
}
```

**Validation**:
- ✅ Compiles without errors or warnings
- ✅ Connects to Binance WebSocket successfully
- ✅ Subscribes to specified streams
- ✅ Handles ping/pong for keep-alive
- ✅ Graceful disconnect on SIGTERM
- ✅ Stats tracking operational

---

### AC-2: BinanceParser Implementation ✅

**Given** raw Binance WebSocket messages  
**When** parsing is needed  
**Then** support for trade, ticker, kline, and depth messages:

```rust
pub struct BinanceParser {
    source_type: DataSourceType,
}

#[async_trait]
impl MessageParser for BinanceParser {
    fn source_type(&self) -> DataSourceType {
        self.source_type
    }
    
    async fn parse(&self, raw: &str) -> Result<Option<StandardMarketData>> {
        let value: Value = serde_json::from_str(raw)?;
        
        // Subscription confirmation or heartbeat
        if value.get("result").is_some() || value.get("id").is_some() {
            return Ok(None);
        }
        
        let event_type = value["e"].as_str().ok_or_else(|| {
            DataError::ParseError {
                source: "Binance".to_string(),
                message: "Missing event type".to_string(),
                raw_data: raw.to_string(),
            }
        })?;
        
        match event_type {
            "trade" => self.parse_trade(&value, raw),
            "24hrTicker" => self.parse_ticker(&value, raw),
            "kline" => self.parse_kline(&value, raw),
            "depthUpdate" => self.parse_depth(&value, raw),
            _ => Ok(None), // Unknown event type
        }
    }
    
    fn validate(&self, raw: &str) -> bool {
        serde_json::from_str::<Value>(raw).is_ok()
    }
}

impl BinanceParser {
    fn parse_trade(&self, value: &Value, raw: &str) -> Result<Option<StandardMarketData>> {
        Ok(Some(StandardMarketData {
            source: self.source_type,
            exchange: "Binance".to_string(),
            symbol: value["s"].as_str().unwrap().to_string(),
            asset_type: AssetType::Spot,
            data_type: MarketDataType::Trade,
            price: Decimal::from_str(value["p"].as_str().unwrap())?,
            quantity: Decimal::from_str(value["q"].as_str().unwrap())?,
            timestamp: value["T"].as_i64().unwrap(),
            received_at: chrono::Utc::now().timestamp_millis(),
            sequence_id: Some(value["t"].as_u64().unwrap()),
            raw_data: raw.to_string(),
            ..Default::default()
        }))
    }
    
    fn parse_ticker(&self, value: &Value, raw: &str) -> Result<Option<StandardMarketData>> {
        Ok(Some(StandardMarketData {
            source: self.source_type,
            exchange: "Binance".to_string(),
            symbol: value["s"].as_str().unwrap().to_string(),
            asset_type: AssetType::Spot,
            data_type: MarketDataType::Ticker,
            price: Decimal::from_str(value["c"].as_str().unwrap())?, // Close price
            quantity: Decimal::from_str(value["v"].as_str().unwrap())?, // Volume
            timestamp: value["E"].as_i64().unwrap(),
            received_at: chrono::Utc::now().timestamp_millis(),
            bid: Some(Decimal::from_str(value["b"].as_str().unwrap())?),
            ask: Some(Decimal::from_str(value["a"].as_str().unwrap())?),
            high_24h: Some(Decimal::from_str(value["h"].as_str().unwrap())?),
            low_24h: Some(Decimal::from_str(value["l"].as_str().unwrap())?),
            volume_24h: Some(Decimal::from_str(value["v"].as_str().unwrap())?),
            raw_data: raw.to_string(),
            ..Default::default()
        }))
    }
    
    fn parse_kline(&self, value: &Value, raw: &str) -> Result<Option<StandardMarketData>> {
        let kline = &value["k"];
        if kline["x"].as_bool().unwrap() == false {
            // Kline not closed yet, ignore
            return Ok(None);
        }
        
        Ok(Some(StandardMarketData {
            source: self.source_type,
            exchange: "Binance".to_string(),
            symbol: value["s"].as_str().unwrap().to_string(),
            asset_type: AssetType::Spot,
            data_type: MarketDataType::Kline,
            price: Decimal::from_str(kline["c"].as_str().unwrap())?, // Close
            quantity: Decimal::from_str(kline["v"].as_str().unwrap())?, // Volume
            timestamp: kline["T"].as_i64().unwrap(), // Close time
            received_at: chrono::Utc::now().timestamp_millis(),
            raw_data: raw.to_string(),
            ..Default::default()
        }))
    }
    
    fn parse_depth(&self, value: &Value, raw: &str) -> Result<Option<StandardMarketData>> {
        // For simplicity, store best bid/ask as a ticker-like record
        let bids = value["b"].as_array().unwrap();
        let asks = value["a"].as_array().unwrap();
        
        if bids.is_empty() || asks.is_empty() {
            return Ok(None);
        }
        
        let best_bid = Decimal::from_str(bids[0][0].as_str().unwrap())?;
        let best_ask = Decimal::from_str(asks[0][0].as_str().unwrap())?;
        let mid_price = (best_bid + best_ask) / Decimal::from(2);
        
        Ok(Some(StandardMarketData {
            source: self.source_type,
            exchange: "Binance".to_string(),
            symbol: value["s"].as_str().unwrap().to_string(),
            asset_type: AssetType::Spot,
            data_type: MarketDataType::OrderBook,
            price: mid_price,
            quantity: Decimal::ZERO,
            timestamp: value["E"].as_i64().unwrap(),
            received_at: chrono::Utc::now().timestamp_millis(),
            bid: Some(best_bid),
            ask: Some(best_ask),
            raw_data: raw.to_string(),
            ..Default::default()
        }))
    }
}
```

**Validation**:
- ✅ Parses trade messages correctly
- ✅ Parses ticker (24hr) messages correctly
- ✅ Parses kline messages (only closed candles)
- ✅ Parses depth updates (best bid/ask)
- ✅ Handles subscription confirmations (returns None)
- ✅ Handles invalid JSON gracefully
- ✅ All financial values use `Decimal` (not f64)
- ✅ Timestamps preserved accurately

---

### AC-3: Redis Integration ✅

**Given** parsed market data  
**When** storing latest prices  
**Then** Redis must cache for fast lookup:

```rust
use redis::{Client, Commands, Connection};

pub struct RedisCache {
    client: Client,
    ttl_secs: u64,
}

impl RedisCache {
    pub fn new(url: &str, ttl_secs: u64) -> Result<Self> {
        let client = Client::open(url)?;
        Ok(Self { client, ttl_secs })
    }
    
    pub async fn store_latest(&self, data: &StandardMarketData) -> Result<()> {
        let mut conn = self.client.get_connection()?;
        
        // Key: "market:{source}:{symbol}:latest"
        let key = format!("market:{}:{}:latest", data.source, data.symbol);
        
        // Serialize to JSON
        let json = serde_json::to_string(data)?;
        
        // Store with TTL
        conn.set_ex(&key, json, self.ttl_secs)?;
        
        Ok(())
    }
    
    pub async fn get_latest(&self, source: &str, symbol: &str) -> Result<Option<StandardMarketData>> {
        let mut conn = self.client.get_connection()?;
        
        let key = format!("market:{}:{}:latest", source, symbol);
        let result: Option<String> = conn.get(&key)?;
        
        match result {
            Some(json) => Ok(Some(serde_json::from_str(&json)?)),
            None => Ok(None),
        }
    }
}
```

**Requirements**:
- ✅ Latest price cached with key `market:{source}:{symbol}:latest`
- ✅ TTL configured (default 24 hours)
- ✅ JSON serialization for easy querying
- ✅ Connection pooling (use `r2d2` or `deadpool-redis`)
- ✅ Error handling for Redis failures
- ✅ Fallback to ClickHouse if Redis unavailable

**Validation**:
- ✅ Data stored and retrieved correctly
- ✅ TTL works (keys expire after configured time)
- ✅ Performance: < 5ms per operation (P95)
- ✅ Connection pool prevents exhaustion

---

### AC-4: ClickHouse Integration ✅

**Given** parsed market data  
**When** persisting to ClickHouse  
**Then** batch insertion with the unified schema:

```rust
use clickhouse::Client;

pub struct ClickHouseWriter {
    client: Client,
    batch: Vec<StandardMarketData>,
    batch_size: usize,
    flush_interval: Duration,
}

impl ClickHouseWriter {
    pub fn new(url: &str, database: &str, batch_size: usize, flush_interval_ms: u64) -> Result<Self> {
        let client = Client::default()
            .with_url(url)
            .with_database(database);
        
        Ok(Self {
            client,
            batch: Vec::with_capacity(batch_size),
            batch_size,
            flush_interval: Duration::from_millis(flush_interval_ms),
        })
    }
    
    pub async fn write(&mut self, data: StandardMarketData) -> Result<()> {
        self.batch.push(data);
        
        if self.batch.len() >= self.batch_size {
            self.flush().await?;
        }
        
        Ok(())
    }
    
    pub async fn flush(&mut self) -> Result<()> {
        if self.batch.is_empty() {
            return Ok(());
        }
        
        let start = Instant::now();
        let count = self.batch.len();
        
        // Insert batch
        let mut insert = self.client.insert("unified_ticks")?;
        for data in self.batch.drain(..) {
            insert.write(&data).await?;
        }
        insert.end().await?;
        
        let elapsed = start.elapsed();
        info!("Flushed {} rows to ClickHouse in {:?}", count, elapsed);
        
        // Track metrics
        CLICKHOUSE_INSERTS.inc_by(count as u64);
        CLICKHOUSE_LATENCY.observe(elapsed.as_secs_f64());
        
        Ok(())
    }
    
    pub async fn start_auto_flush(&mut self) {
        let mut interval = tokio::time::interval(self.flush_interval);
        loop {
            interval.tick().await;
            if let Err(e) = self.flush().await {
                error!("Auto-flush error: {}", e);
            }
        }
    }
}
```

**Requirements**:
- ✅ Batch size configurable (default 1000 rows)
- ✅ Auto-flush interval configurable (default 5 seconds)
- ✅ Manual flush on graceful shutdown
- ✅ Error handling with retry (3 attempts)
- ✅ Metrics tracking (rows inserted, latency)
- ✅ Backpressure handling if ClickHouse slow

**Validation**:
- ✅ Data inserted correctly to `unified_ticks` table
- ✅ Batch insertion works (1000 rows at once)
- ✅ Auto-flush triggers every 5 seconds
- ✅ Performance: > 10k rows/s insertion rate
- ✅ No data loss on graceful shutdown

---

### AC-5: Reconnection Logic ✅

**Given** WebSocket connections can drop  
**When** disconnection occurs  
**Then** automatic reconnection with exponential backoff:

```rust
impl BinanceConnector {
    pub async fn run_with_reconnect(&mut self, tx: mpsc::Sender<StandardMarketData>) -> Result<()> {
        let max_attempts = 5;
        let mut attempt = 0;
        
        loop {
            match self.connect_and_stream(tx.clone()).await {
                Ok(_) => {
                    info!("Stream ended gracefully");
                    break;
                }
                Err(e) => {
                    attempt += 1;
                    error!("Connection error (attempt {}/{}): {}", attempt, max_attempts, e);
                    
                    if attempt >= max_attempts {
                        error!("Max reconnection attempts reached, giving up");
                        return Err(e);
                    }
                    
                    let delay = Duration::from_secs(2u64.pow(attempt.min(6))); // Cap at 64s
                    warn!("Reconnecting in {:?}...", delay);
                    tokio::time::sleep(delay).await;
                }
            }
        }
        
        Ok(())
    }
    
    async fn connect_and_stream(&mut self, tx: mpsc::Sender<StandardMarketData>) -> Result<()> {
        // Connect, subscribe, and process messages
        // Returns when connection closes or error occurs
        // ...
    }
}
```

**Requirements**:
- ✅ Exponential backoff: 2s, 4s, 8s, 16s, 32s, 64s (max)
- ✅ Max reconnection attempts: 5
- ✅ Health status reflects connection state
- ✅ Metrics track reconnection attempts
- ✅ Alerts if reconnections exceed threshold

**Validation**:
- ✅ Reconnects automatically on disconnect
- ✅ Backoff timing correct
- ✅ Gives up after 5 attempts
- ✅ Health endpoint shows connection status

---

### AC-6: Integration Tests ✅

**Given** the complete data pipeline  
**When** running integration tests  
**Then** end-to-end data flow must be validated:

```rust
#[tokio::test]
async fn test_binance_to_clickhouse_e2e() {
    // Setup test environment
    let redis = setup_test_redis().await;
    let clickhouse = setup_test_clickhouse().await;
    
    // Create connector and parser
    let config = BinanceConfig {
        ws_url: "wss://stream.binance.com:9443/ws".to_string(),
        symbols: vec!["btcusdt".to_string()],
        streams: vec![StreamType::Trade],
        asset_type: AssetType::Spot,
    };
    let parser = Arc::new(BinanceParser::new(DataSourceType::BinanceSpot));
    let mut connector = BinanceConnector::new(config, parser);
    
    // Connect and receive data
    let mut rx = connector.connect().await.unwrap();
    
    // Wait for first message (max 10 seconds)
    let data = tokio::time::timeout(
        Duration::from_secs(10),
        rx.recv()
    ).await.unwrap().unwrap();
    
    // Validate data structure
    assert_eq!(data.source, DataSourceType::BinanceSpot);
    assert_eq!(data.exchange, "Binance");
    assert_eq!(data.symbol, "BTCUSDT");
    assert!(data.price > Decimal::ZERO);
    
    // Store in Redis
    redis.store_latest(&data).await.unwrap();
    
    // Verify Redis storage
    let cached = redis.get_latest("BinanceSpot", "BTCUSDT").await.unwrap();
    assert!(cached.is_some());
    
    // Store in ClickHouse
    clickhouse.write(data.clone()).await.unwrap();
    clickhouse.flush().await.unwrap();
    
    // Verify ClickHouse storage
    let query = "SELECT * FROM unified_ticks WHERE symbol = 'BTCUSDT' ORDER BY timestamp DESC LIMIT 1";
    let result: Option<StandardMarketData> = clickhouse.query(query).await.unwrap();
    assert!(result.is_some());
    
    // Cleanup
    connector.disconnect().await.unwrap();
}
```

**Test Coverage**:
- ✅ Binance connection and subscription
- ✅ Message parsing (trade, ticker, kline, depth)
- ✅ Redis caching
- ✅ ClickHouse persistence
- ✅ HTTP endpoint queries
- ✅ Health check functionality
- ✅ Reconnection logic
- ✅ Graceful shutdown

---

### AC-7: Load Testing (10k msg/s) ✅

**Given** the system must handle 10k messages/second  
**When** load testing  
**Then** performance targets must be met:

**Load Test Setup**:
```bash
# Use real Binance data with 100 trading pairs
# Expected: 100 pairs × 100 msg/s = 10k msg/s total

# Monitor with:
# - Prometheus metrics
# - Grafana dashboard
# - System resource monitoring (htop, iostat)
```

**Performance Targets**:
- ✅ Throughput: > 10k msg/s sustained for 1 hour
- ✅ Parse latency: P95 < 50 μs
- ✅ Redis latency: P95 < 5ms
- ✅ ClickHouse latency: P95 < 100ms (batch)
- ✅ E2E latency: P99 < 20ms (local), P50 < 5ms
- ✅ Memory: < 500MB steady state
- ✅ CPU: < 60% single core (or distributed across cores)
- ✅ No data loss
- ✅ No memory leaks

**Validation**:
- ✅ Load test runs successfully for 1 hour
- ✅ All performance targets met
- ✅ No errors or warnings in logs
- ✅ Metrics dashboard shows healthy system
- ✅ Resource usage within limits

---

### AC-8: 24-Hour Stability Test ✅

**Given** the system must be production-ready  
**When** running for 24 hours  
**Then** stability and reliability must be proven:

**Stability Test**:
```bash
# Start service
cargo run --release

# Monitor for 24 hours:
# - Uptime
# - Error rate
# - Memory usage (check for leaks)
# - Connection stability
# - Data accuracy (spot checks)

# Automated checks every hour:
# - Health endpoint returns 200
# - Metrics show increasing message counts
# - Redis cache operational
# - ClickHouse accepting writes
# - No critical errors in logs
```

**Success Criteria**:
- ✅ Uptime: ≥ 99.9% (max 86 seconds downtime)
- ✅ Error rate: < 0.1% (< 1 in 1000 messages)
- ✅ Memory: No leaks (stable over time)
- ✅ Reconnections: < 5 per 24 hours
- ✅ Data accuracy: 100% (no corrupt data)
- ✅ Health checks: All pass throughout
- ✅ No manual intervention required

**Monitoring**:
- ✅ Prometheus alerts configured
- ✅ Grafana dashboard for visualization
- ✅ Log aggregation for analysis
- ✅ On-call rotation for critical issues

---

### AC-9: Production Deployment ✅

**Given** all tests pass  
**When** deploying to production  
**Then** the service must be containerized and deployed to AKS:

**Docker Multi-Stage Build**:
```dockerfile
# Build stage
FROM rust:1.75 as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/data-engine /app/
COPY config /app/config
EXPOSE 8080
CMD ["./data-engine"]
```

**Kubernetes Deployment**:
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: data-engine
  namespace: hermesflow
spec:
  replicas: 1  # Single instance for Sprint 3
  selector:
    matchLabels:
      app: data-engine
  template:
    metadata:
      labels:
        app: data-engine
    spec:
      containers:
      - name: data-engine
        image: hermesflow/data-engine:v0.1.0
        ports:
        - containerPort: 8080
        env:
        - name: RUST_ENV
          value: "prod"
        - name: DATA_ENGINE__REDIS__URL
          valueFrom:
            secretKeyRef:
              name: redis-credentials
              key: url
        - name: DATA_ENGINE__CLICKHOUSE__URL
          valueFrom:
            secretKeyRef:
              name: clickhouse-credentials
              key: url
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
        resources:
          requests:
            memory: "256Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "2000m"
```

**Deployment Requirements**:
- ✅ Docker image builds successfully
- ✅ Image size < 100MB (optimized)
- ✅ Kubernetes manifests in GitOps repo
- ✅ ArgoCD syncs and deploys automatically
- ✅ Health checks pass after deployment
- ✅ Service accessible via internal DNS
- ✅ Secrets managed via Azure Key Vault
- ✅ Monitoring dashboards operational

**Validation**:
- ✅ Service running in production AKS
- ✅ Health endpoint returns 200
- ✅ Metrics visible in Prometheus
- ✅ Logs in Azure Log Analytics
- ✅ Alerts configured in Azure Monitor
- ✅ No errors in first hour of production

---

## 📊 Performance Targets (Sprint 3)

| Metric | Target | Notes |
|--------|--------|-------|
| **Throughput** | > 10k msg/s | 100 pairs × 100 msg/s |
| **Parse Latency** | P95 < 50 μs | JSON parsing |
| **Redis Latency** | P95 < 5ms | Cache writes |
| **ClickHouse Latency** | P95 < 100ms | Batch writes |
| **E2E Latency** | P99 < 20ms | Exchange → storage |
| **Memory Usage** | < 500MB | Steady state |
| **CPU Usage** | < 60% | Single core or distributed |
| **Uptime** | ≥ 99.9% | 24-hour test |
| **Error Rate** | < 0.1% | < 1 in 1000 messages |
| **Reconnections** | < 5 per day | Auto-recovery |

---

## 📦 Deliverables

1. ✅ `BinanceConnector` implementation
2. ✅ `BinanceParser` implementation
3. ✅ Redis integration code
4. ✅ ClickHouse integration code
5. ✅ Integration tests
6. ✅ Load test results
7. ✅ 24-hour stability test report
8. ✅ Docker image
9. ✅ Kubernetes manifests
10. ✅ Production deployment (ArgoCD)
11. ✅ Monitoring dashboards
12. ✅ Runbook documentation

---

## 🎯 Definition of Done

- [ ] All code compiles and passes tests
- [ ] Integration tests pass (E2E data flow)
- [ ] Load test validates 10k msg/s sustained
- [ ] 24-hour stability test passes (≥ 99.9% uptime)
- [ ] Docker image built and pushed
- [ ] Deployed to production AKS
- [ ] Health checks passing
- [ ] Monitoring dashboards operational
- [ ] Runbook documentation complete
- [ ] PO acceptance demo completed
- [ ] QA sign-off received

---

## 📅 Timeline

**Sprint 3**: Nov 11 - Nov 22 (1.5 weeks, 5 SP)

| Day | Date | Tasks |
|-----|------|-------|
| 1 | Nov 11 | BinanceConnector implementation |
| 2 | Nov 12 | BinanceParser implementation |
| 3 | Nov 13 | Redis + ClickHouse integration |
| 4 | Nov 14 | Reconnection logic, integration tests |
| 5 | Nov 15 | Load testing (10k msg/s) |
| 6-7 | Nov 18-19 | 24-hour stability test |
| 8 | Nov 20 | Docker + Kubernetes deployment |
| 9 | Nov 21 | Production validation, monitoring |
| 10 | Nov 22 | Sprint review, retrospective |

---

## 🔗 Related Documents

- **DATA-001A**: Universal Data Framework (Sprint 2)
- **DATA-002**: OKX WebSocket Connector (Sprint 3/4)
- **PRD**: `docs/prd/modules/01-data-module.md`

---

## 🔧 实施说明（基于 DATA-001A 完成情况）

### 框架就绪状态

**DATA-001A 已完成（2025-11-02）**，以下组件已就绪，可直接使用：

#### 1. DataSourceConnector Trait
```rust
// 位置: modules/data-engine/src/traits/connector.rs
// 状态: ✅ 完成，已通过 6 个单元测试
use data_engine::traits::DataSourceConnector;
use data_engine::models::{DataSourceType, AssetType, StandardMarketData};
use tokio::sync::mpsc;

#[async_trait]
impl DataSourceConnector for BinanceConnector {
    fn source_type(&self) -> DataSourceType {
        DataSourceType::BinanceSpot  // 或 BinancePerp, BinanceFutures
    }
    
    fn supported_assets(&self) -> Vec<AssetType> {
        vec![AssetType::Spot]  // 根据实际支持的类型
    }
    
    async fn connect(&mut self) -> Result<mpsc::Receiver<StandardMarketData>> {
        // 实现 WebSocket 连接和订阅逻辑
        // 返回 Receiver channel
    }
    
    async fn disconnect(&mut self) -> Result<()> {
        // 实现断开连接逻辑
    }
    
    async fn is_healthy(&self) -> bool {
        // 检查连接状态
    }
    
    fn stats(&self) -> ConnectorStats {
        // 返回连接统计信息
    }
}
```

#### 2. MessageParser Trait + ParserRegistry
```rust
// 位置: modules/data-engine/src/traits/parser.rs
// 位置: modules/data-engine/src/registry/parser_registry.rs
// 状态: ✅ 完成，已通过 11 个单元测试

use data_engine::traits::MessageParser;
use data_engine::registry::ParserRegistry;
use std::sync::Arc;

// 实现 BinanceParser
#[async_trait]
impl MessageParser for BinanceParser {
    fn source_type(&self) -> DataSourceType {
        DataSourceType::BinanceSpot
    }
    
    async fn parse(&self, raw: &str) -> Result<Option<StandardMarketData>> {
        // 解析 Binance WebSocket 消息
        // 返回 Ok(None) 如果消息应被忽略（心跳等）
    }
    
    fn validate(&self, raw: &str) -> bool {
        // 验证消息格式
    }
}

// 注册到 ParserRegistry
let mut registry = ParserRegistry::new();
let parser = Arc::new(BinanceParser::new(DataSourceType::BinanceSpot));
registry.register(parser);
```

#### 3. Redis 缓存集成
```rust
// 位置: modules/data-engine/src/storage/redis.rs
// 状态: ✅ 完成，已通过 2 个单元测试

use data_engine::storage::RedisCache;

let mut redis = RedisCache::new(&config.redis.url, config.redis.ttl_secs).await?;

// 存储最新价格
redis.store_latest(&market_data).await?;

// 获取最新价格
let latest = redis.get_latest("BinanceSpot", "BTCUSDT").await?;
```

#### 4. ClickHouse 存储集成
```rust
// 位置: modules/data-engine/src/storage/clickhouse.rs
// 状态: ✅ 完成框架，批量写入逻辑待完善（Sprint 3）

use data_engine::storage::ClickHouseWriter;

let mut clickhouse = ClickHouseWriter::new(
    &config.clickhouse.url,
    &config.clickhouse.database,
    config.clickhouse.batch_size,
    config.clickhouse.flush_interval_ms,
)?;

// 写入数据（批量）
clickhouse.write(market_data).await?;

// 手动刷新（或等待自动刷新）
clickhouse.flush().await?;
```

### 实施注意事项

1. **ClickHouse 插入逻辑**：
   - 当前 `ClickHouseWriter::flush()` 为占位符实现（仅日志输出）
   - Sprint 3 需要实现实际的 `Row` trait 序列化
   - 参考：`modules/data-engine/src/storage/clickhouse.rs:85-95`

2. **标准数据模型**：
   - 使用 `StandardMarketData` 结构体
   - 确保所有字段正确映射（symbol, price, timestamp 等）
   - 使用 `Decimal` 类型处理价格（不是 `f64`）

3. **错误处理**：
   - 使用 `DataError` 枚举
   - 利用 `retry_with_backoff()` 函数进行重试
   - 参考：`modules/data-engine/src/error.rs`

4. **监控和日志**：
   - 使用 `tracing` 宏进行结构化日志
   - Prometheus 指标已集成，直接使用即可
   - 参考：`modules/data-engine/src/monitoring/`

### 快速开始检查清单

- [ ] 创建 `src/connectors/binance.rs` 模块
- [ ] 实现 `BinanceConnector`（实现 `DataSourceConnector` trait）
- [ ] 创建 `src/parsers/binance_parser.rs` 模块
- [ ] 实现 `BinanceParser`（实现 `MessageParser` trait）
- [ ] 在 `main.rs` 中注册 parser 到 `ParserRegistry`
- [ ] 实现 WebSocket 连接和订阅逻辑
- [ ] 实现消息解析（trade, ticker, kline, depth）
- [ ] 集成 Redis 缓存（存储最新价格）
- [ ] 完善 ClickHouse 批量写入逻辑
- [ ] 实现自动重连机制（指数退避）
- [ ] 编写单元测试和集成测试
- [ ] 执行性能测试（10k msg/s）
- [ ] 24 小时稳定性测试

### 参考文档

- **框架架构**: `docs/architecture/data-engine-architecture.md`
- **集成指南**: `docs/guides/adding-new-data-source.md`
- **Mock OKX 示例**: `modules/data-engine/tests/extensibility_test.rs`

---

**Version**: 2.0 (Updated for Sprint 3)  
**Status**: ✅ **READY** - DATA-001A 已完成，可立即开始开发






