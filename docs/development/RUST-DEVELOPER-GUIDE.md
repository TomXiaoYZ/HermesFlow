# Rust 开发者完整指南

> **HermesFlow 数据引擎 - Rust 开发指南** | **适用于**: 数据引擎模块

---

## 🎯 本指南目标

帮助 Rust 开发者：
1. ✅ 快速上手 HermesFlow 数据引擎开发
2. ✅ 掌握项目特定的 Rust 最佳实践
3. ✅ 理解数据引擎架构和设计模式
4. ✅ 高效调试和优化代码

---

## 📚 必读文档

在开始之前，请先阅读：

- 📋 [数据模块 PRD](../prd/modules/01-data-module.md) - 理解业务需求
- 🏗️ [系统架构 - Rust 数据服务层](../architecture/system-architecture.md#42-rust数据服务层) - 理解架构设计
- 📜 [ADR-006: Rust 数据层决策](../architecture/decisions/ADR-006-rust-data-layer.md) - 理解技术选型
- 📝 [编码规范 - Rust 部分](../development/coding-standards.md#rust-规范) - 代码风格

---

## 🚀 快速开始

### 环境搭建（30分钟）

#### 1. 安装 Rust

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 验证安装
rustc --version  # 应为 1.75+
cargo --version

# 安装 nightly (如需使用 nightly features)
rustup toolchain install nightly
```

#### 2. 安装开发工具

```bash
# 代码格式化
rustup component add rustfmt

# Linter
rustup component add clippy

# 其他工具
cargo install cargo-watch      # 热重载
cargo install cargo-edit        # cargo add/rm/upgrade
cargo install cargo-audit       # 安全审计
cargo install cargo-tarpaulin   # 测试覆盖率
cargo install cargo-expand      # 宏展开
cargo install cargo-flamegraph  # 性能分析
```

#### 3. IDE 配置

**VS Code（推荐）**:

```json
// .vscode/settings.json
{
  // rust-analyzer 配置
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.cargo.loadOutDirsFromCheck": true,
  
  // 格式化
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  
  // 提示
  "rust-analyzer.inlayHints.enable": true,
  "rust-analyzer.inlayHints.chainingHints": true,
  "rust-analyzer.inlayHints.parameterHints": true,
  
  // 测试
  "rust-analyzer.runnables.cargoExtraArgs": ["--release"]
}
```

**推荐插件**:
- `rust-analyzer` (必装)
- `crates` (Cargo.toml 依赖管理)
- `Even Better TOML`
- `Error Lens` (错误提示)
- `CodeLLDB` (调试)

#### 4. 克隆和构建

```bash
# 克隆代码
git clone <your-repo-url>/HermesFlow.git
cd HermesFlow/modules/data-engine

# 构建
cargo build

# 运行测试
cargo test

# 运行服务
cargo run
```

---

## 📁 项目结构

```
modules/data-engine/
├── Cargo.toml           # 依赖配置
├── Cargo.lock           # 依赖锁定
├── .cargo/
│   └── config.toml      # Cargo 配置
├── src/
│   ├── main.rs          # 入口文件
│   ├── lib.rs           # 库入口
│   │
│   ├── connectors/      # 数据连接器
│   │   ├── mod.rs
│   │   ├── binance.rs   # Binance WebSocket/REST
│   │   ├── okx.rs       # OKX API
│   │   ├── polygon.rs   # Polygon.io（美股）
│   │   └── base.rs      # 连接器 Trait
│   │
│   ├── processors/      # 数据处理
│   │   ├── mod.rs
│   │   ├── aggregator.rs  # 数据聚合
│   │   ├── normalizer.rs  # 数据标准化
│   │   └── validator.rs   # 数据验证
│   │
│   ├── storage/         # 存储层
│   │   ├── mod.rs
│   │   ├── clickhouse.rs  # ClickHouse 写入
│   │   ├── redis.rs       # Redis 缓存
│   │   └── buffer.rs      # 批量写入缓冲
│   │
│   ├── api/             # API 接口
│   │   ├── mod.rs
│   │   ├── rest.rs        # REST API (Actix-web)
│   │   ├── websocket.rs   # WebSocket
│   │   └── grpc.rs        # gRPC (Tonic)
│   │
│   ├── models/          # 数据模型
│   │   ├── mod.rs
│   │   ├── market_data.rs # 市场数据
│   │   └── config.rs      # 配置
│   │
│   └── utils/           # 工具函数
│       ├── mod.rs
│       ├── logger.rs      # 日志
│       └── metrics.rs     # Prometheus 指标
│
├── tests/               # 集成测试
│   ├── integration_test.rs
│   └── performance_test.rs
│
├── benches/             # 性能基准测试
│   └── market_data_bench.rs
│
└── examples/            # 示例代码
    └── simple_connector.rs
```

---

## 🔧 核心技术栈

### 异步运行时: Tokio

HermesFlow 使用 Tokio 作为异步运行时。

#### 基本用法

```rust
// main.rs
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 异步代码
    let result = fetch_data().await?;
    Ok(())
}

// 异步函数
async fn fetch_data() -> Result<Vec<u8>, reqwest::Error> {
    let response = reqwest::get("https://api.example.com/data").await?;
    let bytes = response.bytes().await?;
    Ok(bytes.to_vec())
}
```

#### 并发任务

```rust
use tokio::task;

// 并发执行多个任务
async fn fetch_multiple_symbols() -> Result<(), Error> {
    let symbols = vec!["BTC/USDT", "ETH/USDT", "SOL/USDT"];
    
    let tasks: Vec<_> = symbols
        .into_iter()
        .map(|symbol| {
            task::spawn(async move {
                fetch_market_data(symbol).await
            })
        })
        .collect();
    
    // 等待所有任务完成
    let results = futures::future::join_all(tasks).await;
    
    Ok(())
}
```

#### CPU 密集任务

```rust
use tokio::task;

// CPU 密集任务应使用 spawn_blocking
async fn process_large_dataset(data: Vec<f64>) -> Result<Vec<f64>, Error> {
    task::spawn_blocking(move || {
        // CPU 密集计算（会阻塞线程）
        data.iter()
            .map(|&x| expensive_calculation(x))
            .collect()
    })
    .await?
}
```

---

### Web 框架: Actix-web

#### 基本 REST API

```rust
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct QueryParams {
    symbol: String,
    limit: Option<usize>,
}

#[derive(Serialize)]
struct MarketDataResponse {
    symbol: String,
    data: Vec<Candle>,
}

// GET /api/v1/market-data?symbol=BTC/USDT&limit=100
async fn get_market_data(query: web::Query<QueryParams>) -> impl Responder {
    let data = fetch_from_clickhouse(&query.symbol, query.limit.unwrap_or(100)).await;
    
    HttpResponse::Ok().json(MarketDataResponse {
        symbol: query.symbol.clone(),
        data,
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/api/v1/market-data", web::get().to(get_market_data))
    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}
```

#### 中间件

```rust
use actix_web::middleware::{Logger, Compress};
use actix_web_prom::PrometheusMetrics;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Prometheus 指标
    let prometheus = PrometheusMetrics::new("api", Some("/metrics"), None);
    
    HttpServer::new(move || {
        App::new()
            .wrap(prometheus.clone())
            .wrap(Logger::default())
            .wrap(Compress::default())
            .route("/health", web::get().to(health_check))
    })
    .bind(("0.0.0.0", 8081))?
    .run()
    .await
}
```

---

### 并行计算: Rayon

用于 CPU 密集的并行计算。

```rust
use rayon::prelude::*;

// 并行处理大量数据
fn calculate_indicators(prices: Vec<f64>) -> Vec<Indicator> {
    prices
        .par_iter()  // 并行迭代器
        .map(|&price| calculate_rsi(price, 14))
        .collect()
}

// 并行排序
fn sort_large_dataset(mut data: Vec<u64>) -> Vec<u64> {
    data.par_sort_unstable();  // 并行排序
    data
}
```

---

### 数据处理: Arrow

用于高性能列式数据处理。

```rust
use arrow::array::{Float64Array, StringArray};
use arrow::record_batch::RecordBatch;
use arrow::datatypes::{Schema, Field, DataType};
use std::sync::Arc;

// 创建 Arrow RecordBatch
fn create_market_data_batch() -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![
        Field::new("symbol", DataType::Utf8, false),
        Field::new("price", DataType::Float64, false),
        Field::new("volume", DataType::Float64, false),
    ]));
    
    let symbols = StringArray::from(vec!["BTC/USDT", "ETH/USDT"]);
    let prices = Float64Array::from(vec![50000.0, 3000.0]);
    let volumes = Float64Array::from(vec![1000.0, 5000.0]);
    
    RecordBatch::try_new(
        schema,
        vec![Arc::new(symbols), Arc::new(prices), Arc::new(volumes)],
    ).unwrap()
}
```

---

## 🎨 常用设计模式

### 1. Builder Pattern

```rust
#[derive(Debug)]
struct BinanceConnector {
    api_key: String,
    api_secret: String,
    base_url: String,
    timeout: Duration,
}

impl BinanceConnector {
    fn builder() -> BinanceConnectorBuilder {
        BinanceConnectorBuilder::default()
    }
}

#[derive(Default)]
struct BinanceConnectorBuilder {
    api_key: Option<String>,
    api_secret: Option<String>,
    base_url: Option<String>,
    timeout: Option<Duration>,
}

impl BinanceConnectorBuilder {
    fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }
    
    fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    fn build(self) -> Result<BinanceConnector, Error> {
        Ok(BinanceConnector {
            api_key: self.api_key.ok_or("api_key is required")?,
            api_secret: self.api_secret.ok_or("api_secret is required")?,
            base_url: self.base_url.unwrap_or_else(|| "https://api.binance.com".to_string()),
            timeout: self.timeout.unwrap_or(Duration::from_secs(30)),
        })
    }
}

// 使用
let connector = BinanceConnector::builder()
    .api_key("your_key")
    .api_secret("your_secret")
    .timeout(Duration::from_secs(10))
    .build()?;
```

---

### 2. Trait-based 抽象

```rust
use async_trait::async_trait;

// 定义 Trait
#[async_trait]
trait MarketDataConnector: Send + Sync {
    async fn connect(&mut self) -> Result<(), Error>;
    async fn subscribe(&mut self, symbols: Vec<String>) -> Result<(), Error>;
    async fn fetch_candles(&self, symbol: &str, interval: &str) -> Result<Vec<Candle>, Error>;
    async fn disconnect(&mut self) -> Result<(), Error>;
}

// 实现 Trait
struct BinanceConnector {
    // ...
}

#[async_trait]
impl MarketDataConnector for BinanceConnector {
    async fn connect(&mut self) -> Result<(), Error> {
        // 连接逻辑
        Ok(())
    }
    
    async fn subscribe(&mut self, symbols: Vec<String>) -> Result<(), Error> {
        // 订阅逻辑
        Ok(())
    }
    
    // ... 其他方法
}

// 使用（多态）
async fn process_market_data(connector: &mut dyn MarketDataConnector) {
    connector.connect().await.unwrap();
    connector.subscribe(vec!["BTC/USDT".to_string()]).await.unwrap();
}
```

---

### 3. 错误处理 (thiserror)

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataEngineError {
    #[error("Failed to connect to {source}: {message}")]
    ConnectionError {
        source: String,
        message: String,
    },
    
    #[error("Invalid market data: {0}")]
    InvalidData(String),
    
    #[error("ClickHouse error")]
    ClickHouseError(#[from] clickhouse::error::Error),
    
    #[error("Redis error")]
    RedisError(#[from] redis::RedisError),
    
    #[error("IO error")]
    IoError(#[from] std::io::Error),
}

// 使用
fn fetch_data(symbol: &str) -> Result<MarketData, DataEngineError> {
    if symbol.is_empty() {
        return Err(DataEngineError::InvalidData("Symbol cannot be empty".to_string()));
    }
    
    // ... 获取数据
    Ok(data)
}
```

---

### 4. 配置管理 (config + serde)

```rust
use serde::{Deserialize, Serialize};
use config::{Config, ConfigError, Environment, File};

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub server: ServerConfig,
    pub binance: BinanceConfig,
    pub clickhouse: ClickHouseConfig,
    pub redis: RedisConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = std::env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        
        let s = Config::builder()
            // 默认配置
            .add_source(File::with_name("config/default"))
            // 环境特定配置
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            // 环境变量覆盖（前缀: HERMESFLOW_）
            .add_source(Environment::with_prefix("HERMESFLOW").separator("__"))
            .build()?;
        
        s.try_deserialize()
    }
}

// 使用
let settings = Settings::new()?;
println!("Server: {}:{}", settings.server.host, settings.server.port);
```

---

## 🧪 测试

### 单元测试

```rust
// src/processors/aggregator.rs

pub fn aggregate_trades(trades: Vec<Trade>) -> Candle {
    // ... 聚合逻辑
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_aggregate_empty_trades() {
        let result = aggregate_trades(vec![]);
        assert!(result.is_empty());
    }
    
    #[test]
    fn test_aggregate_single_trade() {
        let trade = Trade {
            price: 50000.0,
            volume: 1.0,
            timestamp: 1234567890,
        };
        
        let candle = aggregate_trades(vec![trade]);
        
        assert_eq!(candle.open, 50000.0);
        assert_eq!(candle.close, 50000.0);
        assert_eq!(candle.high, 50000.0);
        assert_eq!(candle.low, 50000.0);
    }
}
```

运行测试：
```bash
# 所有测试
cargo test

# 特定测试
cargo test test_aggregate_empty_trades

# 显示输出
cargo test -- --nocapture

# 并行度
cargo test -- --test-threads=1
```

---

### 异步测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_fetch_market_data() {
        let connector = BinanceConnector::new();
        let data = connector.fetch_candles("BTC/USDT", "1m").await;
        
        assert!(data.is_ok());
        assert!(!data.unwrap().is_empty());
    }
}
```

---

### 集成测试

```rust
// tests/integration_test.rs

use hermesflow_data_engine::*;

#[tokio::test]
async fn test_end_to_end_data_flow() {
    // 1. 连接数据源
    let mut connector = BinanceConnector::builder()
        .api_key("test_key")
        .build()
        .unwrap();
    
    connector.connect().await.unwrap();
    
    // 2. 获取数据
    let data = connector.fetch_candles("BTC/USDT", "1m").await.unwrap();
    
    // 3. 处理数据
    let processed = normalize_market_data(data);
    
    // 4. 存储数据
    let storage = ClickHouseStorage::new("http://localhost:8123");
    storage.write_batch(processed).await.unwrap();
    
    // 5. 验证
    let stored = storage.read("BTC/USDT").await.unwrap();
    assert!(!stored.is_empty());
}
```

---

### 性能基准测试

```rust
// benches/market_data_bench.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hermesflow_data_engine::*;

fn benchmark_aggregation(c: &mut Criterion) {
    let trades = generate_test_trades(10000);
    
    c.bench_function("aggregate 10k trades", |b| {
        b.iter(|| {
            aggregate_trades(black_box(trades.clone()))
        });
    });
}

criterion_group!(benches, benchmark_aggregation);
criterion_main!(benches);
```

运行基准测试：
```bash
cargo bench
```

---

### 测试覆盖率

```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 生成覆盖率报告
cargo tarpaulin --out Html

# 打开报告
open tarpaulin-report.html
```

**目标**: ≥ 85% 覆盖率

---

## 🐛 调试技巧

### 1. 日志

```rust
use tracing::{info, warn, error, debug, trace};

#[tokio::main]
async fn main() {
    // 初始化 tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    
    info!("Starting data engine");
    debug!("Connecting to Binance: url={}", url);
    
    match fetch_data().await {
        Ok(data) => info!("Fetched {} records", data.len()),
        Err(e) => error!("Failed to fetch data: {:?}", e),
    }
}
```

### 2. LLDB 调试

在 VS Code 中配置 `.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug data-engine",
      "cargo": {
        "args": [
          "build",
          "--bin=data-engine",
          "--package=hermesflow-data-engine"
        ],
        "filter": {
          "name": "data-engine",
          "kind": "bin"
        }
      },
      "args": [],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

设置断点，按 F5 启动调试。

### 3. `dbg!` 宏

```rust
fn calculate_rsi(prices: &[f64]) -> f64 {
    let gains = dbg!(calculate_gains(prices));  // 打印 gains 并返回
    let losses = dbg!(calculate_losses(prices));
    
    let rsi = 100.0 - (100.0 / (1.0 + (gains / losses)));
    dbg!(rsi)  // 打印并返回 rsi
}
```

### 4. Tokio Console

实时监控异步任务。

```toml
# Cargo.toml
[dependencies]
console-subscriber = "0.1"
```

```rust
#[tokio::main]
async fn main() {
    console_subscriber::init();
    
    // ... 你的代码
}
```

运行：
```bash
# 终端 1: 运行应用
RUSTFLAGS="--cfg tokio_unstable" cargo run

# 终端 2: 启动 tokio-console
tokio-console
```

---

## ⚡ 性能优化

### 1. 避免不必要的克隆

```rust
// ❌ 不好: 不必要的 clone
fn process_data(data: Vec<u8>) -> Vec<u8> {
    data.clone()  // 浪费
}

// ✅ 好: 使用引用或移动
fn process_data(data: &[u8]) -> &[u8] {
    data
}

// 或者直接移动所有权
fn process_data(data: Vec<u8>) -> Vec<u8> {
    data  // 移动，无开销
}
```

### 2. 使用 `&str` 而非 `String`

```rust
// ❌ 不好
fn log_message(msg: String) {
    println!("{}", msg);
}

// ✅ 好
fn log_message(msg: &str) {
    println!("{}", msg);
}
```

### 3. 预分配容量

```rust
// ❌ 不好: 动态增长
let mut vec = Vec::new();
for i in 0..10000 {
    vec.push(i);
}

// ✅ 好: 预分配
let mut vec = Vec::with_capacity(10000);
for i in 0..10000 {
    vec.push(i);
}
```

### 4. 使用迭代器而非索引

```rust
// ❌ 不好: 索引访问
for i in 0..vec.len() {
    println!("{}", vec[i]);
}

// ✅ 好: 迭代器
for item in &vec {
    println!("{}", item);
}
```

### 5. 并行处理

```rust
use rayon::prelude::*;

// ❌ 不好: 串行
let result: Vec<_> = large_dataset
    .iter()
    .map(|x| expensive_operation(x))
    .collect();

// ✅ 好: 并行
let result: Vec<_> = large_dataset
    .par_iter()
    .map(|x| expensive_operation(x))
    .collect();
```

---

## 📊 性能分析

### Flamegraph

```bash
# 安装
cargo install flamegraph

# 生成火焰图
cargo flamegraph --bin data-engine

# 打开 flamegraph.svg
open flamegraph.svg
```

### Criterion

```bash
# 运行基准测试
cargo bench

# 比较（需要两次运行）
cargo bench --bench market_data_bench -- --save-baseline before
# ... 修改代码 ...
cargo bench --bench market_data_bench -- --baseline before
```

---

## 📚 推荐资源

### 官方文档
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Actix-web Documentation](https://actix.rs/docs/)

### 进阶
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Async Book](https://rust-lang.github.io/async-book/)

### HermesFlow 文档
- [编码规范](./coding-standards.md)
- [Code Review 清单](./CODE-REVIEW-CHECKLIST.md)
- [测试策略](../testing/test-strategy.md)

---

## 📞 获取帮助

- **Rust Team**: Slack `#rust-dev`
- **技术问题**: [FAQ](../FAQ.md)
- **Bug 报告**: GitHub Issues

---

**最后更新**: 2025-01-13  
**维护者**: @architect.mdc  
**版本**: v1.0

