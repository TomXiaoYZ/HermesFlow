# 测试策略

**版本**: v2.0.0  
**最后更新**: 2024-12-20

---

## 目录

1. [测试金字塔](#1-测试金字塔)
2. [测试覆盖率要求](#2-测试覆盖率要求)
3. [各层测试策略](#3-各层测试策略)
4. [测试环境](#4-测试环境)

---

## 1. 测试金字塔

```
           /\
          /  \         E2E测试 (5%)
         /    \        - UI测试
        /------\       - 完整流程测试
       /        \
      /          \     集成测试 (20%)
     /            \    - API测试
    /--------------\   - 服务间集成
   /                \
  /                  \ 单元测试 (75%)
 /____________________\ - 函数/类测试
```

### 测试分层

| 层级 | 占比 | 执行速度 | 范围 | 目的 |
|------|------|---------|------|------|
| 单元测试 | 75% | 快（ms级） | 单个函数/类 | 验证逻辑正确性 |
| 集成测试 | 20% | 中（秒级） | 多个组件 | 验证组件协作 |
| E2E测试 | 5% | 慢（分钟级） | 完整系统 | 验证用户场景 |

---

## 2. 测试覆盖率要求

### 2.1 代码覆盖率目标

| 服务类型 | 语言 | 单元测试覆盖率 | 集成测试覆盖率 |
|---------|------|---------------|---------------|
| 数据采集服务 | **Rust** | **>85%** ⭐ | >70% |
| 策略引擎 | Python | >75% | >60% |
| 交易执行 | Java | >80% | >65% |
| 风控服务 | Java | >90% | >75% |
| 用户管理 | Java | >80% | >70% |

### 2.2 关键路径100%覆盖

以下代码必须达到100%测试覆盖：

- **风控规则执行逻辑**
- **订单状态机转换**
- **资金计算逻辑**
- **权限验证逻辑**
- **数据解析和验证** ⭐

---

## 3. 各层测试策略

### 3.1 单元测试策略

#### Rust单元测试 ⭐

**测试组织**

```rust
// src/parser.rs
pub fn parse_market_data(raw: &str) -> Result<MarketData> {
    // 实现...
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_data() {
        let raw = r#"{"symbol":"BTCUSDT","price":46000.0}"#;
        let result = parse_market_data(raw);
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.symbol, "BTCUSDT");
        assert_eq!(data.price, 46000.0);
    }

    #[test]
    fn test_parse_invalid_json() {
        let raw = "invalid json";
        let result = parse_market_data(raw);
        assert!(result.is_err());
    }

    #[test]
    #[should_panic(expected = "missing field")]
    fn test_parse_missing_field() {
        let raw = r#"{"symbol":"BTCUSDT"}"#;
        parse_market_data(raw).unwrap();
    }
}
```

**异步测试**

```rust
#[tokio::test]
async fn test_fetch_market_data() {
    let result = fetch_market_data("binance", "BTCUSDT").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_websocket_connection() {
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    let handle = tokio::spawn(async move {
        connect_websocket("wss://example.com", tx).await
    });
    
    // 等待连接建立
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // 验证接收到数据
    let msg = rx.recv().await;
    assert!(msg.is_some());
}
```

**Mock测试**

```rust
use mockall::*;

#[automock]
trait DataSource {
    fn fetch_data(&self, symbol: &str) -> Result<MarketData>;
}

#[test]
fn test_with_mock() {
    let mut mock = MockDataSource::new();
    mock.expect_fetch_data()
        .with(eq("BTCUSDT"))
        .returning(|_| Ok(MarketData::default()));
    
    let result = process_with_source(&mock, "BTCUSDT");
    assert!(result.is_ok());
}
```

#### Java单元测试

```java
@SpringBootTest
class OrderServiceTest {
    
    @Autowired
    private OrderService orderService;
    
    @MockBean
    private OrderRepository orderRepository;
    
    @Test
    void testCreateOrder() {
        // Given
        CreateOrderRequest request = new CreateOrderRequest();
        request.setSymbol("BTCUSDT");
        request.setQuantity(BigDecimal.valueOf(0.1));
        
        // When
        Order order = orderService.createOrder(request).block();
        
        // Then
        assertNotNull(order);
        assertEquals("BTCUSDT", order.getSymbol());
    }
}
```

#### Python单元测试

```python
import pytest
from strategy_engine import Strategy

def test_strategy_signal():
    strategy = Strategy(params={'fast_period': 10, 'slow_period': 30})
    strategy.on_init()
    
    # 模拟K线数据
    for i in range(50):
        bar = Bar(close=100 + i * 0.1)
        strategy.on_bar(bar)
    
    # 验证信号
    assert len(strategy.signals) > 0
```

### 3.2 集成测试策略

#### Rust集成测试 ⭐

**项目结构**

```
data-engine/
├── src/
│   └── lib.rs
├── tests/          # 集成测试目录
│   ├── websocket_test.rs
│   ├── kafka_test.rs
│   └── common/
│       └── mod.rs  # 测试辅助函数
```

**集成测试示例**

```rust
// tests/websocket_test.rs
use data_engine::*;
use tokio;

#[tokio::test]
async fn test_binance_websocket_integration() {
    // 启动测试WebSocket服务器
    let mock_server = start_mock_ws_server().await;
    
    // 连接到模拟服务器
    let client = WebSocketClient::new(mock_server.url());
    client.connect().await.unwrap();
    
    // 订阅数据
    client.subscribe("BTCUSDT").await.unwrap();
    
    // 验证接收到数据
    let data = client.recv().await.unwrap();
    assert_eq!(data.symbol, "BTCUSDT");
    
    // 清理
    client.disconnect().await.unwrap();
    mock_server.stop().await;
}
```

**使用TestContainers**

```rust
use testcontainers::*;

#[tokio::test]
async fn test_redis_integration() {
    // 启动Redis容器
    let docker = clients::Cli::default();
    let redis = docker.run(images::redis::Redis::default());
    let port = redis.get_host_port_ipv4(6379);
    
    // 连接Redis
    let client = redis::Client::open(format!("redis://127.0.0.1:{}", port)).unwrap();
    let mut con = client.get_async_connection().await.unwrap();
    
    // 测试Redis操作
    redis::cmd("SET")
        .arg("test_key")
        .arg("test_value")
        .query_async(&mut con)
        .await
        .unwrap();
    
    let value: String = redis::cmd("GET")
        .arg("test_key")
        .query_async(&mut con)
        .await
        .unwrap();
    
    assert_eq!(value, "test_value");
}
```

### 3.3 性能测试策略

#### Rust基准测试 ⭐

```rust
// benches/parser_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use data_engine::parse_market_data;

fn bench_parse_market_data(c: &mut Criterion) {
    let raw = r#"{"symbol":"BTCUSDT","price":46000.0,"volume":123.45}"#;
    
    c.bench_function("parse_market_data", |b| {
        b.iter(|| {
            parse_market_data(black_box(raw))
        })
    });
}

criterion_group!(benches, bench_parse_market_data);
criterion_main!(benches);
```

**运行基准测试**

```bash
cargo bench

# 生成火焰图
cargo flamegraph --bench parser_bench
```

#### Java性能测试

使用JMH (Java Microbenchmark Harness)

```java
@BenchmarkMode(Mode.Throughput)
@OutputTimeUnit(TimeUnit.SECONDS)
public class OrderProcessingBenchmark {
    
    @Benchmark
    public void benchmarkCreateOrder(Blackhole blackhole) {
        Order order = orderService.createOrder(request);
        blackhole.consume(order);
    }
}
```

---

## 4. 测试环境

### 4.1 本地测试环境

```bash
# 启动测试依赖
docker-compose -f docker-compose.test.yml up -d

# Rust测试
cd modules/data-engine
cargo test

# Java测试
cd modules/trading-engine
./mvnw test

# Python测试
cd modules/strategy-engine
pytest
```

### 4.2 CI测试环境

GitHub Actions配置：

```yaml
name: Tests

on: [push, pull_request]

jobs:
  rust-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test --all-features
      - name: Run clippy
        run: cargo clippy -- -D warnings
      - name: Check coverage
        run: cargo tarpaulin --out Xml
      - name: Upload coverage
        uses: codecov/codecov-action@v3

  java-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-java@v4
        with:
          java-version: '21'
      - name: Run tests
        run: ./mvnw test
      - name: Generate coverage report
        run: ./mvnw jacoco:report
```

---

## 5. 测试数据管理

### 5.1 测试数据策略

- 使用工厂模式生成测试数据
- 使用 fixtures 管理共享测试数据
- 敏感数据脱敏

**Rust测试数据工厂** ⭐

```rust
// tests/common/factories.rs
pub struct MarketDataFactory;

impl MarketDataFactory {
    pub fn create_btc_data() -> MarketData {
        MarketData {
            symbol: "BTCUSDT".to_string(),
            price: 46000.0,
            volume: 123.45,
            timestamp: Utc::now().timestamp(),
        }
    }
    
    pub fn create_multiple(count: usize) -> Vec<MarketData> {
        (0..count).map(|i| MarketData {
            symbol: "BTCUSDT".to_string(),
            price: 46000.0 + i as f64,
            volume: 100.0 + i as f64,
            timestamp: Utc::now().timestamp() + i as i64,
        }).collect()
    }
}
```

---

## 6. 测试报告

### 6.1 覆盖率报告

```bash
# Rust
cargo tarpaulin --out Html
open tarpaulin-report.html

# Java
./mvnw jacoco:report
open target/site/jacoco/index.html

# Python
pytest --cov=. --cov-report=html
open htmlcov/index.html
```

### 6.2 性能报告

```bash
# Rust基准测试
cargo bench
# 结果在 target/criterion/report/index.html

# 火焰图
cargo flamegraph
open flamegraph.svg
```

---

**文档维护者**: QA Team  
**最后更新**: 2024-12-20

