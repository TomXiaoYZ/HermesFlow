# 测试策略

**版本**: v3.0.0  
**最后更新**: 2024-12-20

---

## 目录

1. [测试金字塔](#1-测试金字塔)
2. [测试覆盖率要求](#2-测试覆盖率要求)
3. [各层测试策略](#3-各层测试策略)
4. [测试环境](#4-测试环境)
5. [安全测试策略](#5-安全测试策略)
6. [高风险访问点测试](#6-高风险访问点测试)
7. [测试数据管理](#7-测试数据管理)
8. [CI/CD集成](#8-cicd集成)
9. [质量门禁](#9-质量门禁)

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
| 单元测试 | 70% | 快（ms级） | 单个函数/类 | 验证逻辑正确性 |
| 集成测试 | 20% | 中（秒级） | 多个组件 | 验证组件协作 |
| 安全测试 | ✅ 100%高风险点 | 中（秒级） | 多租户隔离、认证授权 | 验证系统安全性 |
| E2E测试 | 10% | 慢（分钟级） | 完整系统 | 验证用户场景 |

> 📌 **注意**: 安全测试是独立维度，所有高风险访问点必须有对应测试用例

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

## 5. 安全测试策略

### 5.1 认证与授权测试

**JWT Token验证**:
- ✅ 过期Token被拒绝
- ✅ 篡改Token被拒绝  
- ✅ 无Token请求被拒绝
- ✅ 有效Token正常访问

**RBAC权限验证**:
- ✅ 管理员拥有完全访问权限
- ✅ 交易员无法管理用户
- ✅ 分析师只能读取策略
- ✅ 查看者只能查看

详细测试用例参见: [`tests/security/test_authentication.py`](../../tests/security/test_authentication.py), [`tests/security/test_rbac.py`](../../tests/security/test_rbac.py)

### 5.2 多租户隔离测试

**PostgreSQL RLS测试**:
- ✅ 租户A无法访问租户B数据
- ✅ RLS策略生效验证
- ✅ 尝试绕过RLS被阻止

**Redis Key隔离测试**:
- ✅ Key前缀隔离验证
- ✅ 跨租户访问被阻止

**Kafka Topic分区测试**:
- ✅ 消息不跨租户传递
- ✅ 分区隔离验证

详细测试用例参见: [`tests/security/test_tenant_isolation.py`](../../tests/security/test_tenant_isolation.py)

### 5.3 输入验证与注入防护

**SQL注入防护**:
- ✅ 查询参数SQL注入防护
- ✅ 搜索查询SQL注入防护
- ✅ ORDER BY SQL注入防护
- ✅ 盲注SQL注入防护

**XSS防护**:
- ✅ 存储型XSS防护
- ✅ 反射型XSS防护
- ✅ DOM型XSS防护
- ✅ JSON响应XSS防护

详细测试用例参见: [`tests/security/test_sql_injection.py`](../../tests/security/test_sql_injection.py), [`tests/security/test_xss.py`](../../tests/security/test_xss.py)

### 5.4 Rate Limiting测试

**速率限制验证**:
- ✅ 基本速率限制生效
- ✅ 速率限制响应头正确
- ✅ 不同用户独立计数
- ✅ 速率限制重置功能
- ✅ 429响应格式正确
- ✅ 突发流量保护

详细测试用例参见: [`tests/security/test_rate_limiting.py`](../../tests/security/test_rate_limiting.py)

---

## 6. 高风险访问点测试

### 6.1 数据库访问高风险点

| 风险点 | 测试用例数 | 覆盖率 | 文档 |
|--------|-----------|--------|------|
| PostgreSQL RLS | 15+ | 100% | [high-risk-access-testing.md](./high-risk-access-testing.md#21-postgresql-rls) |
| ClickHouse隔离 | 8+ | 100% | [high-risk-access-testing.md](./high-risk-access-testing.md#22-clickhouse) |
| Redis Key隔离 | 6+ | 100% | [high-risk-access-testing.md](./high-risk-access-testing.md#23-redis) |
| Kafka分区隔离 | 5+ | 100% | [high-risk-access-testing.md](./high-risk-access-testing.md#24-kafka) |

### 6.2 API安全高风险点

| 风险点 | 测试用例数 | 覆盖率 | 文档 |
|--------|-----------|--------|------|
| JWT Token验证 | 10+ | 100% | [high-risk-access-testing.md](./high-risk-access-testing.md#31-jwt) |
| RBAC权限 | 12+ | 100% | [high-risk-access-testing.md](./high-risk-access-testing.md#32-rbac) |
| SQL注入防护 | 8+ | 100% | [high-risk-access-testing.md](./high-risk-access-testing.md#33-sql) |
| Rate Limiting | 10+ | 100% | [high-risk-access-testing.md](./high-risk-access-testing.md#34-rate) |

### 6.3 外部服务集成高风险点

| 风险点 | 测试用例数 | 覆盖率 | 文档 |
|--------|-----------|--------|------|
| 交易所API失败处理 | 6+ | 100% | [high-risk-access-testing.md](./high-risk-access-testing.md#41-exchange) |
| 服务降级测试 | 5+ | 100% | [high-risk-access-testing.md](./high-risk-access-testing.md#42-fallback) |

> 📖 **详细文档**: [高风险访问点测试计划](./high-risk-access-testing.md)

---

## 7. 测试数据管理

### 7.1 测试数据策略

**单元测试数据**:
- ✅ 最小化、硬编码
- ✅ 每个测试独立（无共享状态）
- ✅ 使用Fixtures或Mock

**集成测试数据**:
- ✅ 生产级真实数据（脱敏）
- ✅ 使用TestContainers初始化
- ✅ 测试后自动清理

### 7.2 Fixtures库

已实现的Fixtures:
- ✅ 租户Fixtures (`tests/fixtures/tenants.py`)
- ✅ 用户Fixtures (`tests/fixtures/users.py`)
- ✅ 市场数据Fixtures (`tests/conftest.py`)
- ✅ Token Fixtures (`tests/conftest.py`)

### 7.3 数据库初始化

测试数据自动从以下脚本加载:
- ✅ PostgreSQL: `tests/fixtures/init.sql`
- ✅ ClickHouse: `tests/fixtures/clickhouse_init.sql`

### 7.4 清理策略

**自动清理**:
- ✅ 事务回滚（单元测试）
- ✅ Redis FLUSHDB（function级别）
- ✅ Docker容器销毁（集成测试）

> 📖 **详细文档**: [测试数据管理指南](./test-data-management.md)

---

## 8. CI/CD集成

### 8.1 GitHub Actions测试流程

```
代码提交/PR创建
   ↓
并行执行单元测试（5个模块）
├── data-engine (Rust)
├── strategy-engine (Python)
├── trading-engine (Java)
├── user-management (Java)
└── risk-engine (Java)
   ↓
并行执行：
├── 安全测试（SQL注入、XSS、认证、RBAC、多租户隔离）
└── 集成测试（Docker Compose环境）
   ↓
性能测试（仅main分支，k6负载测试）
   ↓
代码质量检查（SonarQube + Trivy）
   ↓
生成测试报告
```

### 8.2 测试自动化

**触发条件**:
- ✅ 推送到 dev/main 分支
- ✅ 创建/更新 Pull Request
- ✅ 每次commit自动运行

**预计执行时间**:
- 单元测试: 5-10分钟
- 安全测试: 3-5分钟
- 集成测试: 5-8分钟
- 性能测试: 15-20分钟（仅main分支）
- **总计**: 20-35分钟

### 8.3 测试环境配置

**本地测试环境**:
```bash
# 启动完整测试环境
docker-compose -f docker-compose.test.yml up -d

# 运行测试
pytest tests/ -v
```

**CI测试环境**:
- ✅ PostgreSQL 15 (带RLS)
- ✅ ClickHouse 23.8
- ✅ Redis 7
- ✅ Kafka 7.5.0 + Zookeeper

> 📖 **详细文档**: [CI/CD集成指南](./ci-cd-integration.md)

---

## 9. 质量门禁

### 9.1 代码合并门禁

**必须满足**:
- ✅ 单元测试100%通过
- ✅ 安全测试100%通过
- ✅ 集成测试100%通过
- ✅ 覆盖率达标（Rust≥85%, Java≥80%, Python≥75%）
- ✅ 代码审查通过（至少1人）
- ✅ 无高危安全漏洞（Trivy扫描）

### 9.2 发布门禁

**必须满足**:
- ✅ 全部测试通过（包括性能测试）
- ✅ 性能测试符合基线（P95<500ms, 错误率<1%）
- ✅ 安全扫描无高危漏洞
- ✅ 压力测试系统稳定（无内存泄漏）
- ✅ 回归测试通过
- ✅ 生产环境预演成功

### 9.3 成功指标

- ✅ 测试覆盖率: Rust≥85%, Java≥80%, Python≥75%
- ✅ 所有高风险访问点有对应测试用例
- ✅ CI/CD每次commit自动运行测试
- ✅ 测试失败率 <5%（非功能性问题）
- ✅ P95响应时间 <500ms（API）
- ✅ P99响应时间 <1000ms（API）
- ✅ 错误率 <1%（负载测试）
- ✅ 安全测试100%通过

---

## 10. 测试文档索引

| 文档 | 描述 | 状态 |
|------|------|------|
| [早期测试策略](./early-test-strategy.md) | 完整测试策略框架（2500+行） | ✅ |
| [高风险访问点测试](./high-risk-access-testing.md) | 数据库、API、外部服务测试（1200+行） | ✅ |
| [测试数据管理](./test-data-management.md) | Fixtures、数据生成、清理（1000+行） | ✅ |
| [CI/CD集成指南](./ci-cd-integration.md) | GitHub Actions配置、故障排查 | ✅ |

---

**文档维护者**: QA Team  
**最后更新**: 2024-12-20  
**版本**: v3.0.0 (整合早期测试策略)

