# 数据模块详细需求文档

**模块名称**: 数据模块 (Data Module)  
**技术栈**: Rust + Tokio + Actix-web/Axum  
**版本**: v2.0.0  
**最后更新**: 2024-12-20

---

## 目录

1. [模块概述](#1-模块概述)
2. [Rust技术选型说明](#2-rust技术选型说明)
3. [架构设计](#3-架构设计)
4. [Epic详述](#4-epic详述)
5. [数据模型](#5-数据模型)
6. [API规范](#6-api规范)
7. [性能基线与测试](#7-性能基线与测试)
8. [部署与运维](#8-部署与运维)

---

## 1. 模块概述

### 1.1 模块职责

数据模块是HermesFlow平台的**数据基础设施层**，负责：

1. **多源数据采集**: 从CEX、DEX、美股、期权、舆情、宏观等多种数据源采集数据
2. **实时数据流处理**: 高并发WebSocket连接管理，实时数据流处理
3. **数据标准化**: 统一不同数据源的格式、时间戳、命名规范
4. **质量控制**: 异常检测、缺失值处理、数据去重
5. **高性能分发**: 通过Redis、Kafka、gRPC高效分发数据
6. **历史数据管理**: 高性能ClickHouse写入和查询

### 1.2 核心价值

- **超低延迟**: μs级数据采集延迟，适合高频交易
- **高吞吐量**: 支持100k+ msg/s的数据处理
- **多市场支持**: 统一接口访问加密货币和传统金融市场
- **数据可靠性**: 99.9%+的数据完整性和准确性
- **易扩展**: 模块化设计，易于添加新数据源

### 1.3 性能目标

| 指标 | 目标值 | 测量方法 |
|------|--------|---------|
| WebSocket消息延迟 | P99 < 1ms | Prometheus直方图 |
| 消息吞吐量 | > 100,000 msg/s | Kafka监控 |
| WebSocket并发连接 | > 500 | 连接池监控 |
| ClickHouse写入性能 | > 100,000 rows/s | 批量写入基准测试 |
| Redis操作延迟 | P99 < 1ms | Redis slowlog |
| 数据准确率 | > 99.99% | 与数据源对账 |
| 服务可用性 | > 99.9% | Uptime监控 |

---

## 2. Rust技术选型说明

### 2.1 为什么选择Rust？

#### 性能优势

1. **零成本抽象**: 抽象不带来运行时开销
2. **无GC**: 没有垃圾回收导致的STW（Stop-The-World）暂停
3. **高效内存管理**: 编译时确定内存分配/释放，无运行时开销
4. **原生性能**: 接近C/C++的性能

**性能对比** (单线程消息处理，1M条消息)

| 语言 | 平均延迟 | P99延迟 | 内存占用 |
|------|---------|---------|---------|
| Rust | 0.5μs | 1.2μs | 50MB |
| Java | 5μs | 15μs | 200MB |
| Python | 50μs | 100μs | 300MB |

#### 安全优势

1. **内存安全**: 编译时保证无悬垂指针、无缓冲区溢出
2. **线程安全**: 编译时保证无数据竞争
3. **类型安全**: 强类型系统，编译时检查

#### 并发优势

1. **Tokio异步运行时**: 高效的M:N调度，支持百万级任务
2. **零成本异步**: async/await语法编译为状态机，无运行时开销
3. **无共享并发**: 基于消息传递的并发模型（Channel）

### 2.2 核心依赖选择

| 依赖 | 版本 | 用途 | 选型理由 |
|------|------|------|---------|
| tokio | 1.35+ | 异步运行时 | 生态最成熟，性能最好 |
| actix-web | 4.4+ | Web框架 | 高性能，actor模型 |
| tungstenite | 0.21+ | WebSocket客户端 | 零依赖，高性能 |
| rdkafka | 0.35+ | Kafka客户端 | librdkafka绑定，稳定可靠 |
| clickhouse-rs | 1.0+ | ClickHouse驱动 | 异步支持，高性能 |
| redis-rs | 0.24+ | Redis客户端 | 官方推荐，功能完善 |
| serde | 1.0+ | 序列化/反序列化 | 事实标准 |
| tracing | 0.1+ | 结构化日志 | 高性能，异步友好 |
| anyhow | 1.0+ | 错误处理 | 简化错误传播 |
| thiserror | 1.0+ | 自定义错误 | 派生宏，易用 |

### 2.3 替代方案对比

**为什么不用Go？**

- Go的GC会导致P99延迟抖动
- Goroutine调度器无法保证延迟一致性
- 没有零成本抽象

**为什么不用C++？**

- 内存安全需要人工保证，易出错
- 无现代化的包管理（Cargo vs Conan/vcpkg）
- 编译速度慢

---

## 3. 架构设计

### 3.1 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                      数据采集服务 (Port 18001)                 │
│                    Rust + Tokio + Actix-web                  │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  Connectors  │  │  Processors  │  │  Distributors│      │
│  │              │  │              │  │              │      │
│  │ • Binance    │──│ • Normalizer │──│ • Redis      │      │
│  │ • OKX        │  │ • Validator  │  │ • Kafka      │      │
│  │ • IBKR       │  │ • Aggregator │  │ • gRPC       │      │
│  │ • Twitter    │  │              │  │              │      │
│  │ • ...        │  │              │  │              │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│                                                               │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    数据处理服务 (Port 18002)                   │
│                    Rust + Rayon + Arrow                      │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  Storage     │  │  Analytics   │  │  Query       │      │
│  │              │  │              │  │              │      │
│  │ • ClickHouse │  │ • Indicators │  │ • API Server │      │
│  │ • Timeseries │  │ • Statistics │  │ • Cache      │      │
│  │ • Archiver   │  │ • Alerts     │  │              │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 并发模型

#### Actor模型（数据采集服务）

```rust
use actix::prelude::*;

// 定义Actor
struct ExchangeConnector {
    exchange: String,
    websocket: WebSocketClient,
    distributor: Addr<DataDistributor>,
}

impl Actor for ExchangeConnector {
    type Context = Context<Self>;
    
    fn started(&mut self, ctx: &mut Self::Context) {
        self.connect_websocket(ctx);
    }
}

// 定义消息
#[derive(Message)]
#[rtype(result = "()")]
struct MarketData {
    symbol: String,
    price: Decimal,
    volume: Decimal,
    timestamp: i64,
}

// 处理消息
impl Handler<MarketData> for ExchangeConnector {
    type Result = ();
    
    fn handle(&mut self, msg: MarketData, ctx: &mut Context<Self>) {
        // 标准化数据
        let normalized = self.normalize(msg);
        
        // 分发数据
        self.distributor.do_send(normalized);
    }
}
```

#### 管道模型（数据处理服务）

```rust
use tokio::sync::mpsc;

async fn data_pipeline() {
    let (tx1, rx1) = mpsc::channel(1000);
    let (tx2, rx2) = mpsc::channel(1000);
    let (tx3, rx3) = mpsc::channel(1000);
    
    // Stage 1: 数据接收
    tokio::spawn(async move {
        while let Some(raw_data) = receive_from_kafka().await {
            tx1.send(raw_data).await.unwrap();
        }
    });
    
    // Stage 2: 数据处理
    tokio::spawn(async move {
        while let Some(raw_data) = rx1.recv().await {
            let processed = process_data(raw_data);
            tx2.send(processed).await.unwrap();
        }
    });
    
    // Stage 3: 数据写入
    tokio::spawn(async move {
        let mut batch = Vec::new();
        while let Some(data) = rx2.recv().await {
            batch.push(data);
            if batch.len() >= 1000 {
                write_to_clickhouse(&batch).await;
                batch.clear();
            }
        }
    });
}
```

### 3.3 错误处理策略

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("WebSocket连接失败: {0}")]
    WebSocketError(#[from] tungstenite::Error),
    
    #[error("数据解析失败: {0}")]
    ParseError(String),
    
    #[error("数据验证失败: {field}字段{reason}")]
    ValidationError { field: String, reason: String },
    
    #[error("外部API错误: {source}返回{status}")]
    ApiError { source: String, status: u16 },
    
    #[error("数据库错误: {0}")]
    DatabaseError(#[from] clickhouse_rs::errors::Error),
}

// 使用Result统一错误处理
pub type Result<T> = std::result::Result<T, DataError>;

// 自动重试包装器
async fn retry_with_backoff<F, T, E>(
    mut f: F,
    max_retries: u32,
) -> Result<T>
where
    F: FnMut() -> Pin<Box<dyn Future<Output = std::result::Result<T, E>>>>,
    E: Into<DataError>,
{
    let mut delay = Duration::from_millis(100);
    
    for attempt in 0..max_retries {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt < max_retries - 1 => {
                tracing::warn!("重试 {}/{}: {:?}", attempt + 1, max_retries, e);
                tokio::time::sleep(delay).await;
                delay *= 2; // 指数退避
            }
            Err(e) => return Err(e.into()),
        }
    }
    
    unreachable!()
}
```

### 3.4 配置管理

```rust
use serde::Deserialize;
use config::{Config, File, Environment};

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub exchanges: ExchangesConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub kafka: KafkaConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Deserialize)]
pub struct ExchangesConfig {
    pub binance: ExchangeConfig,
    pub okx: ExchangeConfig,
    pub ibkr: ExchangeConfig,
}

#[derive(Debug, Deserialize)]
pub struct ExchangeConfig {
    pub enabled: bool,
    pub api_key: String,
    pub api_secret: String,
    pub ws_url: String,
    pub rest_url: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        let config = Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name(&format!(
                "config/{}",
                std::env::var("RUST_ENV").unwrap_or_else(|_| "development".into())
            )).required(false))
            .add_source(Environment::with_prefix("HERMESFLOW"))
            .build()?;
        
        config.try_deserialize().map_err(Into::into)
    }
}
```

---

## 4. Epic详述

### Epic 1: 加密货币数据采集

#### Story 1.1: Binance WebSocket连接管理 [P0]

**用户故事**
```gherkin
Feature: Binance实时行情数据采集
  作为一个量化交易者
  我想要实时接收Binance的行情数据
  以便基于最新价格进行交易决策

Scenario: 建立WebSocket连接并订阅BTC/USDT
  Given 数据采集服务正在运行
  And Binance API配置正确
  When 我订阅 "BTCUSDT" 的实时行情
  Then 系统应建立wss://stream.binance.com:9443/ws连接
  And 系统应发送订阅消息: {"method":"SUBSCRIBE","params":["btcusdt@trade"],"id":1}
  And 系统应持续接收trade事件
  And 数据延迟应小于1ms (P99)

Scenario: WebSocket连接断开自动重连
  Given WebSocket连接已建立
  When 连接因网络问题断开
  Then 系统应在1秒内自动重连
  And 系统应重新订阅之前的所有交易对
  And 系统应记录重连事件到日志
```

**技术实现**

```rust
use tungstenite::{connect, Message};
use url::Url;

pub struct BinanceConnector {
    ws_url: String,
    subscriptions: Vec<String>,
    tx: mpsc::Sender<MarketData>,
}

impl BinanceConnector {
    pub async fn connect(&mut self) -> Result<()> {
        let url = Url::parse(&self.ws_url)?;
        let (mut socket, response) = connect(url)?;
        
        tracing::info!("Binance WebSocket连接成功: {:?}", response);
        
        // 订阅交易对
        for symbol in &self.subscriptions {
            let subscribe_msg = json!({
                "method": "SUBSCRIBE",
                "params": [format!("{}@trade", symbol.to_lowercase())],
                "id": 1
            });
            socket.write_message(Message::Text(subscribe_msg.to_string()))?;
        }
        
        // 接收消息循环
        loop {
            match socket.read_message() {
                Ok(Message::Text(text)) => {
                    if let Ok(data) = self.parse_message(&text) {
                        self.tx.send(data).await?;
                    }
                }
                Ok(Message::Ping(ping)) => {
                    socket.write_message(Message::Pong(ping))?;
                }
                Ok(Message::Close(_)) => {
                    tracing::warn!("WebSocket连接关闭，准备重连");
                    return Err(DataError::WebSocketError(
                        tungstenite::Error::ConnectionClosed
                    ));
                }
                Err(e) => {
                    tracing::error!("WebSocket错误: {:?}", e);
                    return Err(e.into());
                }
                _ => {}
            }
        }
    }
    
    fn parse_message(&self, text: &str) -> Result<MarketData> {
        let value: serde_json::Value = serde_json::from_str(text)?;
        
        Ok(MarketData {
            exchange: "binance".to_string(),
            symbol: value["s"].as_str().unwrap().to_string(),
            price: value["p"].as_str().unwrap().parse()?,
            volume: value["q"].as_str().unwrap().parse()?,
            timestamp: value["T"].as_i64().unwrap(),
        })
    }
    
    pub async fn run_with_reconnect(&mut self) {
        loop {
            match self.connect().await {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("连接失败: {:?}, 1秒后重试", e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }
}
```

**验收标准**

- [ ] 能够成功建立WebSocket连接
- [ ] 能够订阅至少100个交易对
- [ ] 消息接收延迟 P99 < 1ms
- [ ] 连接断开后1秒内自动重连
- [ ] 重连后自动恢复所有订阅
- [ ] 单元测试覆盖率 > 85%
- [ ] 集成测试通过

**测试用例**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_binance_connection() {
        let (tx, mut rx) = mpsc::channel(100);
        let mut connector = BinanceConnector::new(
            "wss://stream.binance.com:9443/ws".to_string(),
            vec!["BTCUSDT".to_string()],
            tx,
        );
        
        tokio::spawn(async move {
            connector.connect().await.unwrap();
        });
        
        // 等待接收第一条消息
        tokio::time::timeout(
            Duration::from_secs(10),
            rx.recv()
        ).await.expect("应该接收到消息");
    }
    
    #[tokio::test]
    async fn test_message_parsing() {
        let json = r#"{
            "e": "trade",
            "E": 1672515782136,
            "s": "BTCUSDT",
            "t": 12345,
            "p": "46000.00",
            "q": "0.001",
            "T": 1672515782136
        }"#;
        
        let connector = BinanceConnector::default();
        let data = connector.parse_message(json).unwrap();
        
        assert_eq!(data.symbol, "BTCUSDT");
        assert_eq!(data.price, Decimal::from_str("46000.00").unwrap());
    }
}
```

#### Story 1.2: OKX WebSocket连接 [P0]

**用户故事**
```gherkin
Feature: OKX实时行情数据采集
  作为一个量化交易者
  我想要实时接收OKX的行情数据
  以便进行跨交易所套利

Scenario: 订阅OKX现货和合约数据
  Given 数据采集服务正在运行
  When 我订阅 OKX SPOT BTC-USDT 和 SWAP BTC-USDT-SWAP
  Then 系统应同时接收现货和合约的实时数据
  And 数据应包含best bid/ask
  And 数据延迟应小于2ms (P99)
```

**技术实现要点**

1. OKX需要API Key认证（即使是公开数据）
2. 订阅消息格式与Binance不同
3. 需要处理多种数据类型（trade, ticker, orderbook）
4. 支持多个instType（SPOT, SWAP, FUTURES, OPTION）

```rust
pub struct OKXConnector {
    api_key: String,
    api_secret: String,
    passphrase: String,
    // ...
}

impl OKXConnector {
    fn sign_request(&self, timestamp: i64, method: &str, path: &str) -> String {
        let message = format!("{}{}{}", timestamp, method, path);
        let key = base64::decode(&self.api_secret).unwrap();
        let mut mac = Hmac::<Sha256>::new_from_slice(&key).unwrap();
        mac.update(message.as_bytes());
        base64::encode(mac.finalize().into_bytes())
    }
    
    async fn subscribe(&mut self, inst_type: &str, inst_id: &str, channel: &str) -> Result<()> {
        let subscribe_msg = json!({
            "op": "subscribe",
            "args": [{
                "channel": channel,
                "instType": inst_type,
                "instId": inst_id
            }]
        });
        
        self.socket.write_message(Message::Text(subscribe_msg.to_string()))?;
        Ok(())
    }
}
```

**验收标准**

- [ ] 支持SPOT、SWAP、FUTURES、OPTION四种产品类型
- [ ] 支持trade、ticker、orderbook三种数据频道
- [ ] 消息延迟 P99 < 2ms
- [ ] API认证成功率 > 99.9%

---

#### Story 1.3: DEX数据采集（GMGN集成）[P1]

**用户故事**
```gherkin
Feature: GMGN链上数据采集
  作为一个加密货币交易者
  我想要获取GMGN的土狗项目数据
  以便发现早期投资机会

Scenario: 获取Solana链新币列表
  Given 数据采集服务已配置GMGN API
  When 我查询过去1小时内Solana链上的新币
  Then 系统应返回新币列表
  And 数据应包含代币地址、流动性、持币地址数、社交链接
  And 数据应按热度排序

Scenario: 监控鲸鱼地址交易
  Given 我订阅了某个鲸鱼地址的交易监控
  When 该地址买入或卖出代币
  Then 系统应实时推送通知
  And 通知应包含代币信息、交易金额、交易哈希
```

**技术实现**

```rust
pub struct GMGNConnector {
    api_url: String,
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
pub struct GMGNToken {
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub chain: String, // "sol" or "eth"
    pub liquidity_usd: Decimal,
    pub holder_count: u32,
    pub market_cap: Option<Decimal>,
    pub social_links: SocialLinks,
    pub hot_level: u32, // 0-100
}

impl GMGNConnector {
    pub async fn get_new_tokens(&self, chain: &str, hours: u32) -> Result<Vec<GMGNToken>> {
        let url = format!("{}/tokens/new?chain={}&hours={}", self.api_url, chain, hours);
        
        let response = self.client
            .get(&url)
            .timeout(Duration::from_secs(10))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(DataError::ApiError {
                source: "GMGN".to_string(),
                status: response.status().as_u16(),
            });
        }
        
        let tokens: Vec<GMGNToken> = response.json().await?;
        Ok(tokens)
    }
    
    pub async fn watch_whale_address(&self, address: &str) -> Result<()> {
        // 轮询或WebSocket监听鲸鱼地址交易
        // 实现细节...
        Ok(())
    }
}
```

**验收标准**

- [ ] 支持Solana和Ethereum两条链
- [ ] 新币数据延迟 < 5分钟
- [ ] 鲸鱼地址交易通知延迟 < 30秒
- [ ] API限流自动管理
- [ ] 数据准确率 > 95%

---

### Epic 2: 传统金融数据采集

#### Story 2.1: IBKR美股实时数据 [P0]

**用户故事**
```gherkin
Feature: 交互券商(IBKR)美股实时数据
  作为一个美股交易者
  我想要获取纳斯达克和纽交所的实时行情
  以便进行美股量化交易

Scenario: 订阅AAPL实时行情
  Given IBKR TWS Gateway正在运行
  And 账户已订阅实时数据
  When 我订阅 AAPL 的实时行情
  Then 系统应接收Bid/Ask/Last价格
  And 数据更新频率应 > 1次/秒
  And 数据应包含Volume和MarketCap
```

**技术实现**

```rust
use ibapi::{Client, Contract, TickType};

pub struct IBKRConnector {
    client: Client,
    subscriptions: HashMap<i32, String>, // req_id -> symbol
}

impl IBKRConnector {
    pub async fn connect(&mut self, host: &str, port: u16) -> Result<()> {
        self.client.connect(host, port, 0).await?;
        tracing::info!("已连接到IBKR TWS Gateway");
        Ok(())
    }
    
    pub async fn subscribe_market_data(&mut self, symbol: &str) -> Result<i32> {
        let contract = Contract::stock(symbol);
        let req_id = self.next_req_id();
        
        self.client.req_mkt_data(
            req_id,
            &contract,
            "", // generic_tick_list
            false, // snapshot
            false, // regulatory_snapshot
            vec![]
        ).await?;
        
        self.subscriptions.insert(req_id, symbol.to_string());
        Ok(req_id)
    }
    
    pub async fn handle_tick(&self, req_id: i32, tick_type: TickType, value: f64) {
        if let Some(symbol) = self.subscriptions.get(&req_id) {
            let data = match tick_type {
                TickType::BidPrice => MarketData {
                    symbol: symbol.clone(),
                    data_type: DataType::Bid,
                    price: Decimal::from_f64(value).unwrap(),
                    timestamp: Utc::now().timestamp_micros(),
                },
                TickType::AskPrice => MarketData {
                    symbol: symbol.clone(),
                    data_type: DataType::Ask,
                    price: Decimal::from_f64(value).unwrap(),
                    timestamp: Utc::now().timestamp_micros(),
                },
                TickType::LastPrice => MarketData {
                    symbol: symbol.clone(),
                    data_type: DataType::Last,
                    price: Decimal::from_f64(value).unwrap(),
                    timestamp: Utc::now().timestamp_micros(),
                },
                _ => return,
            };
            
            // 分发数据
            self.distributor.send(data).await;
        }
    }
}
```

**验收标准**

- [ ] 支持至少1000个美股标的
- [ ] 实时数据延迟 < 100ms
- [ ] 数据更新频率 > 1次/秒
- [ ] 支持盘前/盘后数据
- [ ] 自动处理IBKR限流和错误码

---

#### Story 2.2: 期权链数据采集 [P1]

**用户故事**
```gherkin
Feature: 期权链数据采集
  作为一个期权交易者
  我想要获取完整的期权链数据
  以便分析市场结构和波动率

Scenario: 查询TSLA期权链
  Given 我已连接到IBKR
  When 我查询 TSLA 2024-12-20 到期的期权链
  Then 系统应返回所有行权价的Call和Put
  And 数据应包含Bid/Ask/Last/Volume/OpenInterest
  And 数据应包含希腊值(Delta/Gamma/Theta/Vega/Rho)
  And 数据应包含隐含波动率(IV)
  And 响应时间应 < 5秒
```

**数据模型**

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct OptionChain {
    pub underlying: String,
    pub expiry: NaiveDate,
    pub calls: Vec<OptionContract>,
    pub puts: Vec<OptionContract>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptionContract {
    pub strike: Decimal,
    pub bid: Decimal,
    pub ask: Decimal,
    pub last: Decimal,
    pub volume: i32,
    pub open_interest: i32,
    pub greeks: Greeks,
    pub implied_volatility: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Greeks {
    pub delta: Decimal,
    pub gamma: Decimal,
    pub theta: Decimal,
    pub vega: Decimal,
    pub rho: Decimal,
}
```

**验收标准**

- [ ] 支持查询未来6个月内的所有到期日
- [ ] 单次查询返回完整期权链 < 5秒
- [ ] 希腊值计算准确率 > 99%
- [ ] 支持美式和欧式期权

---

### Epic 3: 舆情数据采集

#### Story 3.1: Twitter情绪监控 [P1]

**用户故事**
```gherkin
Feature: Twitter情绪分析
  作为一个加密货币交易者
  我想要监控Twitter上关于BTC的讨论
  以便捕捉市场情绪变化

Scenario: 监控BTC相关推文
  Given 数据采集服务已配置Twitter API v2
  When 我订阅 "#BTC" 和 "#Bitcoin" 的实时推文
  Then 系统应实时接收相关推文
  And 系统应计算情绪分数 (-1到+1)
  And 系统应检测情绪突变(变化>0.3)
  And 情绪数据应写入时序数据库
```

**技术实现**

```rust
use reqwest::Client;
use serde_json::Value;

pub struct TwitterConnector {
    bearer_token: String,
    client: Client,
}

#[derive(Debug)]
pub struct Tweet {
    pub id: String,
    pub text: String,
    pub author_id: String,
    pub created_at: DateTime<Utc>,
    pub metrics: TweetMetrics,
}

#[derive(Debug)]
pub struct TweetMetrics {
    pub retweet_count: i32,
    pub reply_count: i32,
    pub like_count: i32,
    pub quote_count: i32,
}

#[derive(Debug)]
pub struct SentimentScore {
    pub keyword: String,
    pub score: f64, // -1 to +1
    pub volume: i32,
    pub timestamp: DateTime<Utc>,
}

impl TwitterConnector {
    pub async fn stream_tweets(&self, keywords: Vec<String>) -> Result<()> {
        let query = keywords.join(" OR ");
        let url = format!(
            "https://api.twitter.com/2/tweets/search/stream?query={}",
            urlencoding::encode(&query)
        );
        
        let mut response = self.client
            .get(&url)
            .bearer_auth(&self.bearer_token)
            .send()
            .await?
            .bytes_stream();
        
        while let Some(chunk) = response.next().await {
            let chunk = chunk?;
            if let Ok(tweet) = serde_json::from_slice::<Tweet>(&chunk) {
                let sentiment = self.analyze_sentiment(&tweet.text);
                // 处理情绪数据...
            }
        }
        
        Ok(())
    }
    
    fn analyze_sentiment(&self, text: &str) -> f64 {
        // 简单的情绪分析（实际应该使用NLP模型）
        let positive_words = ["bullish", "moon", "pump", "good", "great"];
        let negative_words = ["bearish", "dump", "crash", "bad", "terrible"];
        
        let text_lower = text.to_lowercase();
        let mut score = 0.0;
        
        for word in positive_words {
            if text_lower.contains(word) {
                score += 0.2;
            }
        }
        
        for word in negative_words {
            if text_lower.contains(word) {
                score -= 0.2;
            }
        }
        
        score.clamp(-1.0, 1.0)
    }
}
```

**验收标准**

- [ ] 支持至少10个关键词同时监控
- [ ] 推文接收延迟 < 5秒
- [ ] 情绪分数更新频率 > 1次/分钟
- [ ] 情绪突变检测准确率 > 80%
- [ ] API限流自动管理（每月500k请求）

---

### Epic 4: 宏观经济数据采集

#### Story 4.1: FRED经济指标 [P1]

**用户故事**
```gherkin
Feature: FRED宏观经济数据
  作为一个宏观交易者
  我想要获取美国CPI、GDP等经济指标
  以便分析宏观环境对市场的影响

Scenario: 查询美国CPI数据
  Given 数据采集服务已配置FRED API Key
  When 我查询过去12个月的CPI数据 (series_id: CPIAUCSL)
  Then 系统应返回月度CPI序列
  And 数据应包含发布时间和修订信息
  And 数据应缓存24小时
```

**技术实现**

```rust
pub struct FREDConnector {
    api_key: String,
    client: Client,
}

#[derive(Debug, Deserialize)]
pub struct FREDSeries {
    pub series_id: String,
    pub title: String,
    pub observations: Vec<FREDObservation>,
}

#[derive(Debug, Deserialize)]
pub struct FREDObservation {
    pub date: NaiveDate,
    pub value: String, // FRED返回字符串
    pub realtime_start: NaiveDate,
    pub realtime_end: NaiveDate,
}

impl FREDConnector {
    pub async fn get_series(&self, series_id: &str, start_date: NaiveDate) -> Result<FREDSeries> {
        let url = format!(
            "https://api.stlouisfed.org/fred/series/observations?series_id={}&api_key={}&file_type=json&observation_start={}",
            series_id,
            self.api_key,
            start_date.format("%Y-%m-%d")
        );
        
        let response = self.client.get(&url).send().await?;
        let data: Value = response.json().await?;
        
        // 解析并返回
        Ok(FREDSeries {
            series_id: series_id.to_string(),
            title: data["seriess"][0]["title"].as_str().unwrap().to_string(),
            observations: serde_json::from_value(data["observations"].clone())?,
        })
    }
}
```

**常用指标列表**

| 指标 | Series ID | 更新频率 | 重要性 |
|------|-----------|---------|--------|
| CPI | CPIAUCSL | 月度 | 高 |
| GDP | GDP | 季度 | 高 |
| 失业率 | UNRATE | 月度 | 高 |
| 联邦基金利率 | FEDFUNDS | 月度 | 极高 |
| 10年期国债收益率 | GS10 | 日度 | 高 |
| M2货币供应 | M2SL | 月度 | 中 |

**验收标准**

- [ ] 支持至少50个经济指标
- [ ] 数据发布后1小时内更新
- [ ] 数据完整性 > 99%
- [ ] 缓存策略有效减少API调用

---

### Epic 5: 数据标准化与质量控制

#### Story 5.1: 统一数据格式 [P0]

**标准数据模型**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardMarketData {
    // 元数据
    pub data_source: DataSource,
    pub data_type: DataType,
    pub quality_score: u8, // 0-100
    
    // 标的信息
    pub symbol: StandardSymbol,
    pub exchange: String,
    
    // 时间戳（UTC, 微秒精度）
    pub exchange_time: i64,
    pub received_time: i64,
    pub processed_time: i64,
    
    // 价格数据
    pub bid: Option<Decimal>,
    pub ask: Option<Decimal>,
    pub last: Option<Decimal>,
    pub open: Option<Decimal>,
    pub high: Option<Decimal>,
    pub low: Option<Decimal>,
    pub close: Option<Decimal>,
    
    // 成交量
    pub volume: Option<Decimal>,
    pub quote_volume: Option<Decimal>,
    
    // 额外字段
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataSource {
    Binance,
    OKX,
    Bitget,
    IBKR,
    Polygon,
    GMGN,
    Twitter,
    FRED,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    Tick,
    Trade,
    OrderBook,
    Kline(TimeFrame),
    Sentiment,
    Macro,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardSymbol {
    pub base: String,
    pub quote: String,
    pub contract_type: Option<ContractType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContractType {
    Spot,
    Future,
    Perpetual,
    Option { strike: Decimal, expiry: NaiveDate, option_type: OptionType },
}
```

**标准化流程**

```rust
pub trait Normalizer: Send + Sync {
    fn normalize(&self, raw_data: RawData) -> Result<StandardMarketData>;
}

pub struct BinanceNormalizer;

impl Normalizer for BinanceNormalizer {
    fn normalize(&self, raw_data: RawData) -> Result<StandardMarketData> {
        let json: Value = serde_json::from_str(&raw_data.content)?;
        
        Ok(StandardMarketData {
            data_source: DataSource::Binance,
            data_type: DataType::Trade,
            quality_score: 100,
            
            symbol: StandardSymbol {
                base: extract_base(&json["s"].as_str().unwrap()),
                quote: extract_quote(&json["s"].as_str().unwrap()),
                contract_type: Some(ContractType::Spot),
            },
            exchange: "binance".to_string(),
            
            exchange_time: json["T"].as_i64().unwrap(),
            received_time: raw_data.received_at,
            processed_time: Utc::now().timestamp_micros(),
            
            last: Some(Decimal::from_str(json["p"].as_str().unwrap())?),
            volume: Some(Decimal::from_str(json["q"].as_str().unwrap())?),
            
            ..Default::default()
        })
    }
}

fn extract_base(symbol: &str) -> String {
    // BTCUSDT -> BTC
    symbol.trim_end_matches("USDT")
          .trim_end_matches("USDC")
          .trim_end_matches("BUSD")
          .to_string()
}
```

**验收标准**

- [ ] 支持所有数据源的标准化
- [ ] 时间戳统一为UTC微秒精度
- [ ] 交易对命名统一为 BASE/QUOTE 格式
- [ ] 标准化准确率 > 99.99%
- [ ] 标准化延迟 < 100μs

---

#### Story 5.2: 数据质量控制 [P0]

**异常检测**

```rust
pub struct QualityChecker {
    history_window: VecDeque<StandardMarketData>,
    config: QualityConfig,
}

#[derive(Debug)]
pub struct QualityConfig {
    pub max_price_change_pct: Decimal,
    pub max_volume_spike: Decimal,
    pub min_timestamp_gap_ms: i64,
    pub max_timestamp_gap_ms: i64,
}

impl QualityChecker {
    pub fn check(&mut self, data: &mut StandardMarketData) -> Vec<QualityIssue> {
        let mut issues = Vec::new();
        
        // 1. 价格跳变检测
        if let Some(last_data) = self.history_window.back() {
            if let (Some(current_price), Some(last_price)) = (data.last, last_data.last) {
                let change_pct = (current_price - last_price).abs() / last_price;
                if change_pct > self.config.max_price_change_pct {
                    issues.push(QualityIssue::PriceSpike {
                        current: current_price,
                        previous: last_price,
                        change_pct,
                    });
                    data.quality_score -= 20;
                }
            }
        }
        
        // 2. 成交量异常检测
        if let Some(volume) = data.volume {
            let avg_volume = self.calculate_avg_volume();
            if volume > avg_volume * self.config.max_volume_spike {
                issues.push(QualityIssue::VolumeSpike {
                    current: volume,
                    average: avg_volume,
                });
                data.quality_score -= 10;
            }
        }
        
        // 3. 时间戳检查
        if let Some(last_data) = self.history_window.back() {
            let gap = data.exchange_time - last_data.exchange_time;
            if gap < self.config.min_timestamp_gap_ms {
                issues.push(QualityIssue::TimestampTooClose { gap });
                data.quality_score -= 30;
            } else if gap > self.config.max_timestamp_gap_ms {
                issues.push(QualityIssue::TimestampGapTooLarge { gap });
                data.quality_score -= 20;
            }
        }
        
        // 4. 缺失值检查
        if data.bid.is_none() || data.ask.is_none() {
            issues.push(QualityIssue::MissingBidAsk);
            data.quality_score -= 10;
        }
        
        // 更新历史窗口
        self.history_window.push_back(data.clone());
        if self.history_window.len() > 100 {
            self.history_window.pop_front();
        }
        
        issues
    }
}

#[derive(Debug)]
pub enum QualityIssue {
    PriceSpike { current: Decimal, previous: Decimal, change_pct: Decimal },
    VolumeSpike { current: Decimal, average: Decimal },
    TimestampTooClose { gap: i64 },
    TimestampGapTooLarge { gap: i64 },
    MissingBidAsk,
    DuplicateData,
}
```

**验收标准**

- [ ] 异常价格检出率 > 95%
- [ ] 假阳性率 < 5%
- [ ] 质量评分计算准确
- [ ] 支持自定义阈值配置

---

### Epic 6: 高性能数据分发

#### Story 6.1: Redis缓存优化 [P0]

**Redis数据结构设计**

```rust
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, RedisResult};

pub struct RedisDistributor {
    conn: ConnectionManager,
    config: RedisConfig,
}

impl RedisDistributor {
    // 实时行情缓存（Hash）
    pub async fn cache_market_data(&mut self, data: &StandardMarketData) -> RedisResult<()> {
        let key = format!("market:{}:{}:latest", data.exchange, data.symbol);
        
        self.conn.hset_multiple(&key, &[
            ("bid", data.bid.map(|d| d.to_string()).unwrap_or_default()),
            ("ask", data.ask.map(|d| d.to_string()).unwrap_or_default()),
            ("last", data.last.map(|d| d.to_string()).unwrap_or_default()),
            ("volume", data.volume.map(|d| d.to_string()).unwrap_or_default()),
            ("timestamp", data.exchange_time.to_string()),
        ]).await?;
        
        // 设置过期时间（1小时）
        self.conn.expire(&key, 3600).await?;
        
        Ok(())
    }
    
    // 订单簿缓存（ZSet）
    pub async fn cache_orderbook(&mut self, orderbook: &OrderBook) -> RedisResult<()> {
        let bids_key = format!("orderbook:{}:{}:bids", orderbook.exchange, orderbook.symbol);
        let asks_key = format!("orderbook:{}:{}:asks", orderbook.exchange, orderbook.symbol);
        
        // 清空旧数据
        self.conn.del(&[&bids_key, &asks_key]).await?;
        
        // 插入买单（价格降序）
        for bid in &orderbook.bids {
            self.conn.zadd(&bids_key, bid.quantity.to_string(), -bid.price.to_f64().unwrap()).await?;
        }
        
        // 插入卖单（价格升序）
        for ask in &orderbook.asks {
            self.conn.zadd(&asks_key, ask.quantity.to_string(), ask.price.to_f64().unwrap()).await?;
        }
        
        // 设置过期时间
        self.conn.expire(&bids_key, 300).await?;
        self.conn.expire(&asks_key, 300).await?;
        
        Ok(())
    }
    
    // K线缓存（List + TTL）
    pub async fn cache_kline(&mut self, kline: &Kline) -> RedisResult<()> {
        let key = format!("kline:{}:{}:{}", kline.exchange, kline.symbol, kline.timeframe);
        
        let value = serde_json::to_string(kline).unwrap();
        self.conn.lpush(&key, value).await?;
        self.conn.ltrim(&key, 0, 999).await?; // 保留最近1000根K线
        self.conn.expire(&key, 86400 * 7).await?; // 7天过期
        
        Ok(())
    }
}
```

**性能优化**

1. **连接池**: 使用`ConnectionManager`自动管理连接
2. **Pipeline**: 批量操作使用pipeline
3. **序列化优化**: 使用MessagePack代替JSON
4. **TTL策略**: 根据数据热度设置不同TTL

**验收标准**

- [ ] Redis写入延迟 P99 < 1ms
- [ ] 并发操作 > 50,000 ops/s
- [ ] 内存使用 < 4GB
- [ ] 缓存命中率 > 90%

---

#### Story 6.2: Kafka高吞吐发布 [P0]

**Kafka生产者优化**

```rust
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};

pub struct KafkaDistributor {
    producer: FutureProducer,
    config: KafkaConfig,
}

impl KafkaDistributor {
    pub fn new(brokers: &str) -> Result<Self> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .set("compression.type", "lz4") // 压缩
            .set("batch.size", "1000000") // 1MB批量
            .set("linger.ms", "10") // 10ms等待批量
            .set("acks", "1") // 仅等待leader确认
            .set("retries", "3")
            .create()?;
        
        Ok(Self {
            producer,
            config: Default::default(),
        })
    }
    
    pub async fn publish_market_data(&self, data: &StandardMarketData) -> Result<()> {
        let topic = "market_data";
        let key = format!("{}:{}", data.exchange, data.symbol);
        let value = serde_json::to_vec(data)?;
        
        let record = FutureRecord::to(topic)
            .key(&key)
            .payload(&value)
            .timestamp(data.exchange_time);
        
        self.producer.send(record, Duration::from_secs(0)).await
            .map_err(|(err, _)| DataError::from(err))?;
        
        Ok(())
    }
    
    pub async fn publish_batch(&self, batch: Vec<StandardMarketData>) -> Result<()> {
        let mut futures = Vec::new();
        
        for data in batch {
            let fut = self.publish_market_data(&data);
            futures.push(fut);
        }
        
        // 并发发送
        futures::future::try_join_all(futures).await?;
        
        Ok(())
    }
}
```

**Topic设计**

| Topic | 分区数 | 副本数 | 用途 |
|-------|--------|--------|------|
| market_data | 16 | 2 | 实时行情 |
| orderbook_updates | 8 | 2 | 订单簿更新 |
| sentiment_data | 4 | 2 | 舆情数据 |
| macro_data | 1 | 2 | 宏观数据 |

**验收标准**

- [ ] 发布吞吐量 > 100,000 msg/s
- [ ] 发布延迟 P99 < 5ms
- [ ] 消息不丢失（acks=1）
- [ ] 支持自动重试

---

#### Story 6.3: gRPC流式推送 [P0]

**Protobuf定义**

```protobuf
syntax = "proto3";

package hermesflow.data;

service MarketDataService {
  // 订阅实时行情流
  rpc StreamMarketData(StreamRequest) returns (stream MarketDataEvent);
  
  // 获取最新价格
  rpc GetLatestPrice(PriceRequest) returns (PriceResponse);
  
  // 获取订单簿
  rpc GetOrderBook(OrderBookRequest) returns (OrderBookResponse);
}

message StreamRequest {
  repeated string exchanges = 1;
  repeated string symbols = 2;
  repeated DataType data_types = 3;
}

message MarketDataEvent {
  string exchange = 1;
  string symbol = 2;
  double last_price = 3;
  double volume = 4;
  int64 timestamp = 5;
  // ...
}

enum DataType {
  TICK = 0;
  TRADE = 1;
  ORDERBOOK = 2;
  KLINE_1M = 3;
  KLINE_5M = 4;
}
```

**Rust服务实现**

```rust
use tonic::{transport::Server, Request, Response, Status};
use tokio_stream::wrappers::ReceiverStream;

pub struct MarketDataServer {
    data_stream: Arc<Mutex<mpsc::Receiver<MarketDataEvent>>>,
}

#[tonic::async_trait]
impl MarketDataService for MarketDataServer {
    type StreamMarketDataStream = ReceiverStream<Result<MarketDataEvent, Status>>;
    
    async fn stream_market_data(
        &self,
        request: Request<StreamRequest>,
    ) -> Result<Response<Self::StreamMarketDataStream>, Status> {
        let req = request.into_inner();
        
        tracing::info!("新客户端订阅: exchanges={:?}, symbols={:?}", 
            req.exchanges, req.symbols);
        
        let (tx, rx) = mpsc::channel(1000);
        
        // 启动数据推送任务
        let exchanges = req.exchanges.clone();
        let symbols = req.symbols.clone();
        tokio::spawn(async move {
            // 从Redis/Kafka订阅数据
            let mut subscriber = subscribe_data(&exchanges, &symbols).await;
            
            while let Some(data) = subscriber.next().await {
                if tx.send(Ok(data)).await.is_err() {
                    break; // 客户端断开
                }
            }
        });
        
        Ok(Response::new(ReceiverStream::new(rx)))
    }
    
    async fn get_latest_price(
        &self,
        request: Request<PriceRequest>,
    ) -> Result<Response<PriceResponse>, Status> {
        let req = request.into_inner();
        
        // 从Redis查询最新价格
        let price = query_latest_price(&req.exchange, &req.symbol).await
            .map_err(|e| Status::internal(e.to_string()))?;
        
        Ok(Response::new(PriceResponse {
            exchange: req.exchange,
            symbol: req.symbol,
            price,
            timestamp: Utc::now().timestamp_micros(),
        }))
    }
}
```

**验收标准**

- [ ] 支持 > 500并发客户端
- [ ] 流式推送延迟 P99 < 2ms
- [ ] 支持背压控制
- [ ] 自动处理客户端断线

---

### Epic 7: 历史数据存储与查询

#### Story 7.1: ClickHouse批量写入优化 [P0]

**批量写入策略**

```rust
use clickhouse_rs::{Pool, Block};

pub struct ClickHouseWriter {
    pool: Pool,
    buffer: Arc<Mutex<Vec<StandardMarketData>>>,
    config: ClickHouseConfig,
}

#[derive(Debug)]
pub struct ClickHouseConfig {
    pub batch_size: usize,
    pub flush_interval_ms: u64,
    pub max_retries: u32,
}

impl ClickHouseWriter {
    pub fn new(connection_string: &str, config: ClickHouseConfig) -> Result<Self> {
        let pool = Pool::new(connection_string);
        
        Ok(Self {
            pool,
            buffer: Arc::new(Mutex::new(Vec::with_capacity(config.batch_size))),
            config,
        })
    }
    
    pub async fn write(&self, data: StandardMarketData) -> Result<()> {
        let mut buffer = self.buffer.lock().await;
        buffer.push(data);
        
        if buffer.len() >= self.config.batch_size {
            let batch = buffer.drain(..).collect();
            drop(buffer); // 释放锁
            
            self.flush_batch(batch).await?;
        }
        
        Ok(())
    }
    
    async fn flush_batch(&self, batch: Vec<StandardMarketData>) -> Result<()> {
        let mut client = self.pool.get_handle().await?;
        
        let block = Block::new()
            .column("timestamp", batch.iter().map(|d| d.exchange_time).collect::<Vec<_>>())
            .column("exchange", batch.iter().map(|d| d.exchange.as_str()).collect::<Vec<_>>())
            .column("symbol", batch.iter().map(|d| format!("{}/{}", d.symbol.base, d.symbol.quote)).collect::<Vec<_>>())
            .column("price", batch.iter().filter_map(|d| d.last).collect::<Vec<_>>())
            .column("volume", batch.iter().filter_map(|d| d.volume).collect::<Vec<_>>());
        
        client.insert("market_data.ticks", block).await?;
        
        tracing::info!("批量写入{}条数据到ClickHouse", batch.len());
        
        Ok(())
    }
    
    pub async fn start_auto_flush(&self) {
        let buffer = self.buffer.clone();
        let interval = self.config.flush_interval_ms;
        let writer = self.clone();
        
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_millis(interval));
            
            loop {
                ticker.tick().await;
                
                let batch = {
                    let mut buffer = buffer.lock().await;
                    if buffer.is_empty() {
                        continue;
                    }
                    buffer.drain(..).collect()
                };
                
                if let Err(e) = writer.flush_batch(batch).await {
                    tracing::error!("自动刷新失败: {:?}", e);
                }
            }
        });
    }
}
```

**ClickHouse表结构**

```sql
CREATE TABLE market_data.ticks (
    timestamp DateTime64(6) CODEC(Delta, ZSTD),
    exchange LowCardinality(String),
    symbol LowCardinality(String),
    price Decimal64(8) CODEC(Delta, ZSTD),
    volume Decimal64(8) CODEC(Delta, ZSTD),
    bid Nullable(Decimal64(8)) CODEC(Delta, ZSTD),
    ask Nullable(Decimal64(8)) CODEC(Delta, ZSTD),
    quality_score UInt8 CODEC(ZSTD)
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (exchange, symbol, timestamp)
SETTINGS index_granularity = 8192;

-- 创建物化视图：1分钟K线
CREATE MATERIALIZED VIEW market_data.kline_1m
ENGINE = AggregatingMergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (exchange, symbol, timestamp)
AS SELECT
    exchange,
    symbol,
    toStartOfMinute(timestamp) as timestamp,
    argMin(price, timestamp) as open,
    max(price) as high,
    min(price) as low,
    argMax(price, timestamp) as close,
    sum(volume) as volume
FROM market_data.ticks
GROUP BY exchange, symbol, timestamp;
```

**验收标准**

- [ ] 批量写入性能 > 100,000 rows/s
- [ ] 数据压缩率 > 10:1
- [ ] 写入延迟 P99 < 100ms
- [ ] 支持自动分区管理

---

#### Story 7.2: 高效历史数据查询 [P0]

**查询接口**

```rust
#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub exchange: String,
    pub symbol: String,
    pub start_time: i64,
    pub end_time: i64,
    pub aggregation: Option<Aggregation>,
}

#[derive(Debug, Deserialize)]
pub enum Aggregation {
    Raw,          // 原始Tick数据
    Kline1m,      // 1分钟K线
    Kline5m,      // 5分钟K线
    Kline1h,      // 1小时K线
}

impl ClickHouseReader {
    pub async fn query_history(&self, req: QueryRequest) -> Result<Vec<MarketData>> {
        let sql = match req.aggregation.unwrap_or(Aggregation::Raw) {
            Aggregation::Raw => format!(
                "SELECT * FROM market_data.ticks WHERE exchange = '{}' AND symbol = '{}' AND timestamp BETWEEN {} AND {} ORDER BY timestamp",
                req.exchange, req.symbol, req.start_time, req.end_time
            ),
            Aggregation::Kline1m => format!(
                "SELECT * FROM market_data.kline_1m WHERE exchange = '{}' AND symbol = '{}' AND timestamp BETWEEN {} AND {} ORDER BY timestamp",
                req.exchange, req.symbol, req.start_time, req.end_time
            ),
            // ...其他聚合级别
        };
        
        let mut client = self.pool.get_handle().await?;
        let block = client.query(&sql).fetch_all().await?;
        
        // 转换为MarketData
        let result = self.block_to_market_data(block)?;
        
        Ok(result)
    }
}
```

**查询优化**

1. **分区剪裁**: 利用`toYYYYMM`分区减少扫描范围
2. **索引利用**: `ORDER BY`字段与查询条件匹配
3. **结果缓存**: Redis缓存常用查询结果
4. **并行查询**: 大范围查询拆分为多个子查询并行执行

**验收标准**

- [ ] 1天数据查询 < 1秒
- [ ] 1个月数据聚合查询 < 5秒
- [ ] 支持并发查询 > 100 QPS
- [ ] 缓存命中率 > 80%

---

## 5. 数据模型

### 5.1 核心数据结构

参见第4章Epic详述中的数据模型定义。

### 5.2 数据库Schema

参见`docs/database/schema/`目录下的完整DDL定义。

---

## 6. API规范

### 6.1 REST API

详见 `docs/api/rest-api-spec.yaml`

### 6.2 gRPC API

详见 `docs/api/grpc-proto/market_data.proto`

---

## 7. 性能基线与测试

### 7.1 基准测试

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_data_normalization(c: &mut Criterion) {
    let normalizer = BinanceNormalizer::new();
    let raw_data = r#"{"e":"trade","E":1672515782136,"s":"BTCUSDT","p":"46000.00","q":"0.001","T":1672515782136}"#;
    
    c.bench_function("normalize binance data", |b| {
        b.iter(|| {
            normalizer.normalize(black_box(raw_data)).unwrap()
        });
    });
}

fn bench_redis_write(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut distributor = rt.block_on(async {
        RedisDistributor::new("redis://localhost:6379").await.unwrap()
    });
    
    let data = StandardMarketData::default();
    
    c.bench_function("redis write market data", |b| {
        b.iter(|| {
            rt.block_on(distributor.cache_market_data(black_box(&data))).unwrap()
        });
    });
}

criterion_group!(benches, bench_data_normalization, bench_redis_write);
criterion_main!(benches);
```

### 7.2 集成测试

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use testcontainers::*;
    
    #[tokio::test]
    async fn test_end_to_end_data_flow() {
        // 启动测试容器
        let docker = clients::Cli::default();
        let redis_container = docker.run(images::redis::Redis::default());
        let clickhouse_container = docker.run(images::generic::GenericImage::new("clickhouse/clickhouse-server", "latest"));
        
        // 创建数据采集服务
        let collector = DataCollector::new(/* ... */);
        
        // 模拟接收数据
        let raw_data = mock_binance_trade();
        collector.handle_data(raw_data).await.unwrap();
        
        // 验证数据已写入Redis
        tokio::time::sleep(Duration::from_millis(100)).await;
        let redis_data = query_redis(&redis_container).await;
        assert!(redis_data.is_some());
        
        // 验证数据已写入ClickHouse
        tokio::time::sleep(Duration::from_secs(2)).await;
        let ch_data = query_clickhouse(&clickhouse_container).await;
        assert!(!ch_data.is_empty());
    }
}
```

### 7.3 压力测试

```bash
# 使用wrk进行压力测试
wrk -t12 -c400 -d30s http://localhost:18001/api/v1/market/realtime/binance/BTCUSDT

# 预期结果
# Requests/sec: > 50,000
# Latency P99: < 10ms
```

---

## 8. 部署与运维

### 8.1 Docker部署

```dockerfile
# 多阶段构建
FROM rust:1.75 as builder
WORKDIR /app

# 缓存依赖
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

# 构建应用
COPY src ./src
RUN cargo build --release

# 运行镜像
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/data-collector /usr/local/bin/

ENV RUST_LOG=info
EXPOSE 18001

CMD ["data-collector"]
```

### 8.2 监控指标

```rust
use prometheus::{IntCounter, Histogram, register_int_counter, register_histogram};

lazy_static! {
    static ref MESSAGES_RECEIVED: IntCounter = register_int_counter!(
        "data_messages_received_total",
        "Total number of messages received"
    ).unwrap();
    
    static ref MESSAGE_LATENCY: Histogram = register_histogram!(
        "data_message_latency_seconds",
        "Message processing latency in seconds"
    ).unwrap();
    
    static ref REDIS_WRITE_LATENCY: Histogram = register_histogram!(
        "redis_write_latency_seconds",
        "Redis write latency in seconds"
    ).unwrap();
}

// 在代码中使用
MESSAGES_RECEIVED.inc();
let timer = MESSAGE_LATENCY.start_timer();
// ... 处理消息
timer.observe_duration();
```

### 8.3 日志配置

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_tracing() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}
```

---

## 附录

### A. 性能调优建议

1. **编译优化**
   ```toml
   [profile.release]
   lto = true              # 启用链接时优化
   codegen-units = 1       # 减少代码生成单元
   opt-level = 3           # 最高优化级别
   ```

2. **异步优化**
   - 使用`spawn_blocking`处理CPU密集型任务
   - 避免在异步上下文中长时间阻塞
   - 合理设置tokio运行时worker数量

3. **内存优化**
   - 使用`Arc`共享数据而非克隆
   - 使用对象池复用大对象
   - 及时释放不再使用的资源

### B. 常见问题

**Q: WebSocket频繁断线？**
A: 检查心跳机制、网络稳定性、交易所限流政策。

**Q: ClickHouse写入性能不足？**
A: 增大批量大小、优化表结构、使用SSD磁盘。

**Q: Redis内存占用过高？**
A: 调整TTL策略、使用压缩、清理无用键。

---

**文档维护者**: Data Team  
**最后更新**: 2024-12-20  
**下次审阅**: 2025-01-20

