# Adding a New Data Source

**Target Time**: < 2 hours  
**Difficulty**: Intermediate  
**Prerequisites**: Basic Rust knowledge, understanding of async programming

---

## Overview

This guide walks you through adding a new data source to the Data Engine. We'll use **OKX** as an example, demonstrating how the universal framework enables rapid integration.

## Step 1: Define Data Source Type (5 minutes)

Add your data source to the `DataSourceType` enum:

```rust
// File: src/models/data_source_type.rs

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DataSourceType {
    // ... existing types
    
    // Add your new source
    OkxSpot,
    OkxFutures,
    OkxPerp,
}
```

Update the helper methods:

```rust
impl DataSourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            // ...
            DataSourceType::OkxSpot => "OkxSpot",
            DataSourceType::OkxFutures => "OkxFutures",
            DataSourceType::OkxPerp => "OkxPerp",
        }
    }

    pub fn exchange(&self) -> &'static str {
        match self {
            // ...
            DataSourceType::OkxSpot 
            | DataSourceType::OkxFutures 
            | DataSourceType::OkxPerp => "OKX",
        }
    }
}
```

## Step 2: Implement Connector (30 minutes)

Create a new connector file:

```rust
// File: src/connectors/okx.rs

use async_trait::async_trait;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};

use crate::{
    error::Result,
    models::{AssetType, DataSourceType, StandardMarketData},
    traits::{ConnectorStats, DataSourceConnector},
};

pub struct OkxConnector {
    config: OkxConfig,
    ws_stream: Option<WebSocketStream>,
    parser: Arc<OkxParser>,
    stats: Arc<RwLock<ConnectorStats>>,
}

#[derive(Debug, Clone)]
pub struct OkxConfig {
    pub ws_url: String,              // "wss://ws.okx.com:8443/ws/v5/public"
    pub symbols: Vec<String>,        // ["BTC-USDT", "ETH-USDT"]
    pub channels: Vec<String>,       // ["trades", "tickers"]
    pub asset_type: AssetType,
}

impl OkxConnector {
    pub fn new(config: OkxConfig, parser: Arc<OkxParser>) -> Self {
        Self {
            config,
            ws_stream: None,
            parser,
            stats: Arc::new(RwLock::new(ConnectorStats::default())),
        }
    }

    async fn build_subscription_message(&self) -> String {
        // OKX subscription format
        let args: Vec<_> = self.config.symbols.iter()
            .flat_map(|symbol| {
                self.config.channels.iter().map(move |channel| {
                    json!({
                        "channel": channel,
                        "instId": symbol
                    })
                })
            })
            .collect();

        json!({
            "op": "subscribe",
            "args": args
        }).to_string()
    }
}

#[async_trait]
impl DataSourceConnector for OkxConnector {
    fn source_type(&self) -> DataSourceType {
        match self.config.asset_type {
            AssetType::Spot => DataSourceType::OkxSpot,
            AssetType::Perpetual => DataSourceType::OkxPerp,
            AssetType::Future => DataSourceType::OkxFutures,
            _ => DataSourceType::OkxSpot,
        }
    }

    fn supported_assets(&self) -> Vec<AssetType> {
        vec![self.config.asset_type]
    }

    async fn connect(&mut self) -> Result<mpsc::Receiver<StandardMarketData>> {
        let (tx, rx) = mpsc::channel(10000);

        // Connect to OKX WebSocket
        let (ws_stream, _) = connect_async(&self.config.ws_url).await?;
        tracing::info!("Connected to OKX WebSocket");

        // Subscribe to channels
        let subscription = self.build_subscription_message().await;
        ws_stream.send(Message::Text(subscription)).await?;

        // Spawn message processing task
        let parser = self.parser.clone();
        let stats = self.stats.clone();
        
        tokio::spawn(async move {
            loop {
                match ws_stream.next().await {
                    Some(Ok(Message::Text(text))) => {
                        stats.write().await.messages_received += 1;

                        match parser.parse(&text).await {
                            Ok(Some(data)) => {
                                if tx.send(data).await.is_err() {
                                    break;
                                }
                                stats.write().await.messages_processed += 1;
                            }
                            Ok(None) => {} // Heartbeat
                            Err(e) => {
                                tracing::error!("Parse error: {}", e);
                                stats.write().await.errors += 1;
                            }
                        }
                    }
                    Some(Ok(Message::Ping(payload))) => {
                        ws_stream.send(Message::Pong(payload)).await.ok();
                    }
                    Some(Err(e)) => {
                        tracing::error!("WebSocket error: {}", e);
                        break;
                    }
                    None => break,
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
        self.stats.blocking_read().clone()
    }
}
```

## Step 3: Implement Parser (30 minutes)

Create the message parser:

```rust
// File: src/parsers/okx.rs

use async_trait::async_trait;
use serde_json::Value;

use crate::{
    error::{DataError, Result},
    models::{AssetType, DataSourceType, MarketDataType, StandardMarketData},
    traits::MessageParser,
};

pub struct OkxParser {
    source_type: DataSourceType,
}

impl OkxParser {
    pub fn new(source_type: DataSourceType) -> Self {
        Self { source_type }
    }

    fn parse_trade(&self, data: &Value) -> Result<StandardMarketData> {
        // OKX trade message format:
        // {"arg":{"channel":"trades","instId":"BTC-USDT"},"data":[{"instId":"BTC-USDT","tradeId":"123","px":"50000","sz":"0.1","side":"buy","ts":"1234567890000"}]}
        
        let inst_id = data["instId"].as_str()
            .ok_or_else(|| DataError::ParseError {
                source: "OkxParser".to_string(),
                message: "Missing instId".to_string(),
                raw_data: data.to_string(),
            })?;

        let price = data["px"].as_str()
            .and_then(|s| Decimal::from_str(s).ok())
            .ok_or_else(|| DataError::ParseError {
                source: "OkxParser".to_string(),
                message: "Invalid price".to_string(),
                raw_data: data.to_string(),
            })?;

        let quantity = data["sz"].as_str()
            .and_then(|s| Decimal::from_str(s).ok())
            .ok_or_else(|| DataError::ParseError {
                source: "OkxParser".to_string(),
                message: "Invalid size".to_string(),
                raw_data: data.to_string(),
            })?;

        let timestamp = data["ts"].as_str()
            .and_then(|s| s.parse::<i64>().ok())
            .ok_or_else(|| DataError::ParseError {
                source: "OkxParser".to_string(),
                message: "Invalid timestamp".to_string(),
                raw_data: data.to_string(),
            })?;

        Ok(StandardMarketData::new(
            self.source_type,
            inst_id.to_string(),
            AssetType::Spot,
            MarketDataType::Trade,
            price,
            quantity,
            timestamp,
        ))
    }

    fn parse_ticker(&self, data: &Value) -> Result<StandardMarketData> {
        // Similar implementation for ticker messages
        // ...
    }
}

#[async_trait]
impl MessageParser for OkxParser {
    fn source_type(&self) -> DataSourceType {
        self.source_type
    }

    async fn parse(&self, raw: &str) -> Result<Option<StandardMarketData>> {
        let value: Value = serde_json::from_str(raw)?;

        // Check if it's a subscription confirmation
        if value.get("event").is_some() {
            return Ok(None);
        }

        // Get channel type
        let channel = value["arg"]["channel"].as_str()
            .ok_or_else(|| DataError::ParseError {
                source: "OkxParser".to_string(),
                message: "Missing channel".to_string(),
                raw_data: raw.to_string(),
            })?;

        // Get data array
        let data_array = value["data"].as_array()
            .ok_or_else(|| DataError::ParseError {
                source: "OkxParser".to_string(),
                message: "Missing data array".to_string(),
                raw_data: raw.to_string(),
            })?;

        if data_array.is_empty() {
            return Ok(None);
        }

        // Parse first data item (could batch process in production)
        let data = &data_array[0];

        match channel {
            "trades" => Ok(Some(self.parse_trade(data)?)),
            "tickers" => Ok(Some(self.parse_ticker(data)?)),
            _ => Ok(None),
        }
    }

    fn validate(&self, raw: &str) -> bool {
        serde_json::from_str::<Value>(raw).is_ok()
    }
}
```

## Step 4: Register with ParserRegistry (5 minutes)

In your application setup:

```rust
// File: src/main.rs or initialization code

let mut registry = ParserRegistry::new();

// Register OKX parser
let okx_parser = Arc::new(OkxParser::new(DataSourceType::OkxSpot));
registry.register(okx_parser);

// Register connector
let okx_config = OkxConfig {
    ws_url: "wss://ws.okx.com:8443/ws/v5/public".to_string(),
    symbols: vec!["BTC-USDT".to_string(), "ETH-USDT".to_string()],
    channels: vec!["trades".to_string(), "tickers".to_string()],
    asset_type: AssetType::Spot,
};

let mut okx_connector = OkxConnector::new(okx_config, okx_parser.clone());
let mut rx = okx_connector.connect().await?;

// Process messages
tokio::spawn(async move {
    while let Some(data) = rx.recv().await {
        // Store in Redis
        redis.store_latest(&data).await.ok();
        
        // Store in ClickHouse
        clickhouse.write(data).await.ok();
    }
});
```

## Step 5: Write Tests (30 minutes)

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_okx_parser_trade() {
        let parser = OkxParser::new(DataSourceType::OkxSpot);
        
        let raw = r#"{
            "arg": {"channel": "trades", "instId": "BTC-USDT"},
            "data": [{
                "instId": "BTC-USDT",
                "tradeId": "123",
                "px": "50000",
                "sz": "0.1",
                "side": "buy",
                "ts": "1234567890000"
            }]
        }"#;

        let result = parser.parse(raw).await;
        assert!(result.is_ok());
        
        let data = result.unwrap().unwrap();
        assert_eq!(data.symbol, "BTC-USDT");
        assert_eq!(data.price, dec!(50000));
    }

    #[tokio::test]
    async fn test_okx_connector() {
        // Integration test with actual WebSocket (optional)
        // Or use a mock WebSocket server
    }
}
```

## Step 6: Documentation (20 minutes)

Add to the README or create a dedicated doc:

```markdown
### OKX Integration

**Status**: ✅ Implemented  
**Support**: Spot, Futures, Perpetuals  
**Channels**: Trades, Tickers  

**Configuration**:
```toml
[[data_sources]]
name = "okx_spot"
source_type = "OkxSpot"
enabled = true
symbols = ["BTC-USDT", "ETH-USDT"]
```

**WebSocket URL**: `wss://ws.okx.com:8443/ws/v5/public`

**Rate Limits**: 100 subscriptions per connection

**Error Handling**:
- Automatic reconnection on disconnect
- Exponential backoff (2s, 4s, 8s, ...)
- Max 5 retry attempts
```

## Checklist

Before considering your integration complete, verify:

- [ ] Data source type added to `DataSourceType` enum
- [ ] Connector implements `DataSourceConnector` trait
- [ ] Parser implements `MessageParser` trait
- [ ] Parser registered in `ParserRegistry`
- [ ] Connector handles reconnection automatically
- [ ] Unit tests written and passing
- [ ] Integration test with live data (optional)
- [ ] Documentation updated
- [ ] Configuration example added
- [ ] Error handling verified
- [ ] Logging statements added for debugging
- [ ] Metrics tracked (messages received, errors, etc.)

## Common Pitfalls

### 1. Forgetting to Handle Heartbeats

```rust
// ✅ Good: Return Ok(None) for heartbeats
if message.event == "heartbeat" {
    return Ok(None);
}

// ❌ Bad: Trying to parse heartbeats
let data = parse_as_market_data(message)?; // Will fail!
```

### 2. Using f64 for Prices

```rust
// ✅ Good: Use Decimal
let price = Decimal::from_str("50000.12345678")?;

// ❌ Bad: Use f64 (precision loss!)
let price = 50000.12345678_f64;
```

### 3. Blocking in Async Context

```rust
// ✅ Good: Use async methods
let result = client.query_async().await?;

// ❌ Bad: Blocking call in async function
let result = client.query_sync()?; // Blocks entire thread!
```

### 4. Not Handling WebSocket Pings

```rust
// ✅ Good: Respond to pings
Some(Ok(Message::Ping(payload))) => {
    stream.send(Message::Pong(payload)).await.ok();
}

// ❌ Bad: Ignore pings (connection will timeout!)
```

## Performance Tips

1. **Batch Redis Writes**: If high throughput, batch multiple symbols
2. **Connection Pooling**: Reuse connections where possible
3. **Parse Once**: Don't parse the same JSON multiple times
4. **Avoid Allocations**: Use `&str` instead of `String` where possible
5. **Channel Buffer Size**: Tune based on message rate (default: 10000)

## Example: Complete Integration Time

Based on the OKX example above:

| Task | Time | Cumulative |
|------|------|------------|
| Define data source type | 5 min | 5 min |
| Implement connector | 30 min | 35 min |
| Implement parser | 30 min | 65 min |
| Register components | 5 min | 70 min |
| Write tests | 30 min | 100 min |
| Documentation | 20 min | **120 min (2 hours)** |

✅ **Target achieved: < 2 hours**

## Conclusion

The universal data framework enables rapid integration of new data sources. The key is following the established patterns:

1. Implement the traits
2. Register with the system
3. Test thoroughly
4. Document clearly

For any questions or issues, refer to existing implementations (Binance, OKX) as reference examples.

---

**Happy Coding! 🚀**






