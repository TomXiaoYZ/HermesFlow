# Sprint 2 Test Strategy: Data Engine

**Sprint**: Sprint 2 (2025-10-28 ~ 2025-11-15)  
**Test Lead**: @sm.mdc  
**Version**: 1.0

---

## 📋 测试目标

### 主要目标

1. **验证通用架构框架的正确性**: 确保 trait 设计合理，易于扩展
2. **验证 Binance 数据采集功能**: 确保实时数据准确、低延迟、高可用
3. **验证数据质量**: 确保数据标准化、质量控制有效
4. **验证性能指标**: 确保满足高频交易需求（< 10ms 延迟，> 10k msg/s 吞吐量）
5. **验证架构可扩展性**: 确保添加新数据源便利（< 2 天）

### 质量目标

- **代码覆盖率**: ≥ 85%
- **单元测试通过率**: 100%
- **集成测试通过率**: 100%
- **性能基准达标率**: 100%
- **缺陷泄漏率**: < 5%
- **P0/P1 缺陷数**: 0

---

## 🎯 测试范围

### 功能测试范围

**In Scope**:
- ✅ DataSourceConnector trait 实现验证
- ✅ AssetType 和 StandardMarketData 模型验证
- ✅ MessageParser trait 和 ParserRegistry 验证
- ✅ BinanceConnector WebSocket 连接、订阅、重连
- ✅ BinanceParser 消息解析（trade, ticker, kline）
- ✅ 数据标准化和质量控制
- ✅ Redis 缓存写入（Hash 结构、TTL）
- ✅ ClickHouse 批量写入（unified_ticks 表）
- ✅ Prometheus 指标暴露
- ✅ 结构化日志输出

**Out of Scope**:
- ❌ 其他数据源实现（OKX, Bitget 等）- Sprint 3
- ❌ 数据质量监控和告警 - Sprint 3
- ❌ ClickHouse 查询优化 - Sprint 3
- ❌ 前端集成测试 - 后续 Sprint

### 非功能测试范围

**In Scope**:
- ✅ 性能测试（延迟、吞吐量）
- ✅ 稳定性测试（重连、异常恢复）
- ✅ 安全测试（依赖扫描）

**Out of Scope**:
- ❌ 长时间压力测试（24h+）- Sprint 3
- ❌ 负载测试（模拟生产流量）- Sprint 4
- ❌ 渗透测试 - 后续专项测试

---

## 🧪 测试类型和策略

### 1. 单元测试（Unit Tests）

**目标**: 验证每个模块的独立功能正确性

**策略**:
- 使用 `cargo test` 运行所有单元测试
- 使用 `mockall` crate mock 外部依赖（Redis, ClickHouse, WebSocket）
- 覆盖正常路径和异常路径
- 覆盖边界条件和特殊输入

**覆盖率目标**: ≥ 85%

**工具**:
- `cargo test` - 测试运行
- `cargo tarpaulin` - 覆盖率统计
- `mockall` - Mock 框架

**测试用例设计**:

#### 模块 1: `connectors::traits`

- `test_data_source_type_serialization`: 验证 DataSourceType 序列化/反序列化
- `test_data_source_type_display`: 验证 Display trait 输出格式
- `test_raw_message_creation`: 验证 RawMessage 创建
- `test_raw_message_with_metadata`: 验证元数据添加

#### 模块 2: `connectors::binance`

- `test_binance_connector_creation`: 验证 BinanceConnector 创建
- `test_subscribe_single_symbol`: 验证订阅单个交易对
- `test_subscribe_multiple_symbols`: 验证批量订阅
- `test_unsubscribe`: 验证取消订阅
- `test_reconnect_with_exponential_backoff`: 验证指数退避重连
- `test_subscription_recovery_after_reconnect`: 验证重连后订阅恢复
- `test_ping_pong_handling`: 验证心跳处理
- `test_connection_timeout`: 验证连接超时处理
- `test_raw_message_generation`: 验证 RawMessage 生成
- `test_channel_backpressure`: 验证 Channel 背压控制

#### 模块 3: `models::asset`

- `test_asset_type_spot_creation`: 验证 Spot 资产创建
- `test_asset_type_perpetual_creation`: 验证 Perpetual 资产创建
- `test_asset_type_future_creation`: 验证 Future 资产创建
- `test_asset_type_option_creation`: 验证 Option 资产创建
- `test_asset_type_stock_creation`: 验证 Stock 资产创建
- `test_asset_type_identifier`: 验证资产唯一标识符生成
- `test_asset_type_serialization`: 验证序列化/反序列化

#### 模块 4: `models::market_data`

- `test_standard_market_data_creation`: 验证 StandardMarketData 创建
- `test_with_price_builder`: 验证价格设置
- `test_with_volume_builder`: 验证成交量设置
- `test_with_extra_builder`: 验证 extra 字段设置
- `test_data_version_field`: 验证数据版本字段
- `test_quality_score_field`: 验证质量分数字段
- `test_serialization`: 验证序列化/反序列化

#### 模块 5: `processors::parser`

- `test_parser_registry_creation`: 验证 ParserRegistry 创建
- `test_register_parser`: 验证 Parser 注册
- `test_parse_with_registered_parser`: 验证解析已注册数据源
- `test_parse_with_unregistered_parser`: 验证解析未注册数据源（应返回错误）
- `test_thread_safety`: 验证多线程并发安全

#### 模块 6: `processors::binance_parser`

- `test_parse_trade_message`: 验证 trade 消息解析
- `test_parse_ticker_message`: 验证 ticker 消息解析
- `test_parse_kline_message`: 验证 kline 消息解析
- `test_parse_invalid_json`: 验证无效 JSON 处理
- `test_parse_missing_fields`: 验证缺失字段处理
- `test_parse_symbol_btcusdt`: 验证 BTCUSDT 解析
- `test_parse_symbol_ethusdt`: 验证 ETHUSDT 解析
- `test_parse_symbol_with_special_suffix`: 验证特殊后缀处理（BTCDOWNUSDT）
- `test_supported_channels`: 验证支持的频道列表

#### 模块 7: `processors::normalizer`

- `test_normalize_timestamp`: 验证时间戳标准化
- `test_normalize_symbol_format`: 验证交易对格式标准化
- `test_normalize_asset_type`: 验证 AssetType 设置
- `test_normalize_data_source`: 验证 DataSourceType 设置
- `test_decimal_precision`: 验证 Decimal 精度

#### 模块 8: `processors::quality`

- `test_quality_check_valid_price`: 验证有效价格检查
- `test_quality_check_invalid_price`: 验证无效价格检查（价格 <= 0）
- `test_quality_check_timestamp`: 验证时间戳合理性检查
- `test_quality_check_price_jump`: 验证价格跳变检测
- `test_quality_score_calculation`: 验证质量分数计算
- `test_low_quality_data_marking`: 验证低质量数据标记

#### 模块 9: `storage::redis`

- `test_redis_distributor_creation`: 验证 RedisDistributor 创建
- `test_cache_market_data`: 验证市场数据缓存
- `test_hash_key_format`: 验证 Hash 键格式
- `test_hash_fields`: 验证 Hash 字段
- `test_ttl_setting`: 验证 TTL 设置
- `test_connection_error_handling`: 验证连接错误处理

#### 模块 10: `storage::clickhouse`

- `test_clickhouse_writer_creation`: 验证 ClickHouseWriter 创建
- `test_batch_accumulation`: 验证批量累积
- `test_auto_flush_on_batch_size`: 验证批次大小触发刷新
- `test_field_mapping`: 验证字段映射正确性
- `test_source_type_mapping`: 验证 source_type 映射
- `test_asset_type_mapping`: 验证 asset_type 映射
- `test_retry_on_failure`: 验证写入失败重试

---

### 2. 集成测试（Integration Tests）

**目标**: 验证多个模块协同工作的正确性

**策略**:
- 使用真实的 Redis 和 ClickHouse（通过 Docker 容器）
- 使用 Mock WebSocket 服务器模拟 Binance
- 测试完整的数据流水线
- 验证跨模块交互

**工具**:
- `testcontainers` - Docker 容器管理
- `wiremock` - Mock HTTP/WebSocket 服务器
- `tokio::test` - 异步测试

**测试用例设计**:

#### Test 1: 端到端数据流测试

**描述**: 验证从 WebSocket 接收到存储的完整数据流

**前置条件**:
- Redis 容器启动
- ClickHouse 容器启动
- Mock WebSocket 服务器启动

**测试步骤**:
1. 初始化所有组件（Connector, Parser, Registry, Distributor, Writer）
2. 连接 Mock WebSocket 并订阅 BTC/USDT
3. 发送 10 条 trade 消息
4. 验证 RawMessage 生成
5. 验证 Parser 解析为 StandardMarketData
6. 验证 asset_type = Spot { base: "BTC", quote: "USDT" }
7. 验证 Redis 中数据正确
8. 验证 ClickHouse 中数据正确
9. 验证 source_type='CEX', asset_type='Spot'
10. 验证端到端延迟 < 10ms

**预期结果**:
- 所有数据正确写入 Redis 和 ClickHouse
- 端到端延迟 P99 < 10ms
- 数据格式和字段映射正确

---

#### Test 2: 架构扩展性验证

**描述**: 验证添加新数据源的便利性

**测试步骤**:
1. 实现 Mock OKXConnector（~200 行代码）
2. 实现 Mock OKXParser（~150 行代码）
3. 注册到 ParserRegistry
4. 发送 OKX 格式的消息
5. 验证整个数据流正常工作
6. 验证无需修改核心代码

**预期结果**:
- OKXConnector 和 OKXParser 实现顺利
- 注册到 ParserRegistry 无需修改注册表代码
- 数据流正常工作
- 添加新数据源仅需 ~350 行代码

---

#### Test 3: 多资产类型并存测试

**描述**: 验证统一存储策略支持所有资产类型

**测试步骤**:
1. 模拟接收 Spot, Perpetual, Option 三种资产的数据
2. 验证 StandardMarketData 正确设置 asset_type
3. 验证 ClickHouse 正确存储到 unified_ticks
4. 验证分区正确（按 asset_type 分区）
5. 验证 Option 的 greeks 存储在 extra 字段

**预期结果**:
- 所有资产类型正确处理
- ClickHouse 分区策略有效
- Option greeks 正确存储在 extra 字段

---

#### Test 4: 重连和订阅恢复测试

**描述**: 验证 WebSocket 自动重连和订阅恢复机制

**测试步骤**:
1. 建立 WebSocket 连接并订阅 3 个交易对
2. 强制断开 WebSocket 连接
3. 验证自动重连（指数退避策略）
4. 验证所有订阅已恢复
5. 验证数据继续正常接收

**预期结果**:
- 断线检测时间 < 1 秒
- 重连成功（指数退避：1s, 2s, 4s）
- 3 个交易对全部恢复订阅
- 数据接收正常

---

#### Test 5: 并发写入压力测试

**描述**: 验证高并发写入的性能和稳定性

**测试步骤**:
1. 启动 Redis 和 ClickHouse 测试容器
2. 同时向 Redis 和 ClickHouse 写入 10,000 条数据
3. 验证吞吐量 > 5,000 ops/s
4. 验证无数据丢失
5. 验证 P99 延迟 < 5ms

**预期结果**:
- Redis 吞吐量 > 5,000 ops/s
- ClickHouse 批量写入 > 10,000 rows/s
- 数据完整性 100%
- Redis P99 延迟 < 1ms

---

### 3. 性能基准测试（Performance Benchmarks）

**目标**: 验证系统性能满足高频交易需求

**策略**:
- 使用 `criterion` crate 进行性能基准测试
- 测试关键路径的性能（解析、标准化、写入）
- 多次运行取平均值，确保稳定性

**工具**:
- `criterion` - 性能基准测试框架

**测试用例设计**:

#### Benchmark 1: 消息解析性能

```rust
fn bench_message_parsing(c: &mut Criterion) {
    let sample_message = r#"{
        "e": "trade",
        "s": "BTCUSDT",
        "p": "43000.50",
        "q": "0.015",
        "T": 1698765432000
    }"#;
    
    let parser = BinanceParser::new();
    let raw = RawMessage::new(
        DataSourceType::CEX { exchange: "Binance".to_string() },
        sample_message.to_string(),
        chrono::Utc::now().timestamp_micros(),
    );
    
    c.bench_function("parse binance trade message", |b| {
        b.iter(|| {
            parser.parse(black_box(&raw))
        });
    });
}
```

**目标**: < 10 μs/op

---

#### Benchmark 2: AssetType 创建性能

```rust
fn bench_asset_type_creation(c: &mut Criterion) {
    c.bench_function("create AssetType::Spot", |b| {
        b.iter(|| {
            AssetType::Spot {
                base: black_box("BTC".to_string()),
                quote: black_box("USDT".to_string()),
            }
        });
    });
}
```

**目标**: < 1 μs/op

---

#### Benchmark 3: 数据标准化性能

```rust
fn bench_normalize_market_data(c: &mut Criterion) {
    let normalizer = Normalizer::new();
    let mut data = create_sample_data();
    
    c.bench_function("normalize market data", |b| {
        b.iter(|| {
            normalizer.normalize(black_box(&mut data))
        });
    });
}
```

**目标**: < 5 μs/op

---

#### Benchmark 4: Redis 写入性能

```rust
fn bench_redis_write(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let distributor = rt.block_on(async {
        RedisDistributor::new(&config).await.unwrap()
    });
    let data = create_sample_data();
    
    c.bench_function("redis hash write", |b| {
        b.to_async(&rt).iter(|| {
            distributor.cache_market_data(black_box(&data))
        });
    });
}
```

**目标**: P99 < 1 ms

---

#### Benchmark 5: ClickHouse 批量写入性能

```rust
fn bench_clickhouse_write(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut writer = rt.block_on(async {
        ClickHouseWriter::new(&config).await.unwrap()
    });
    let batch = create_sample_batch(1000);
    
    c.bench_function("clickhouse batch write", |b| {
        b.to_async(&rt).iter(|| async {
            for data in batch.iter() {
                writer.write(black_box(data.clone())).await.unwrap();
            }
            writer.flush().await.unwrap();
        });
    });
}
```

**目标**: > 10,000 rows/s

---

### 4. 手动验证测试（Manual Verification）

**目标**: 验证系统在真实环境下的行为

**策略**:
- 连接真实的 Binance WebSocket
- 观察日志输出和监控指标
- 验证 Redis 和 ClickHouse 数据

**测试用例设计**:

#### Test 1: Binance WebSocket 连接验证

**测试步骤**:
1. 启动 data-engine 服务
2. 配置 Binance WebSocket URL
3. 订阅 BTC/USDT
4. 观察日志输出
5. 验证 Prometheus 指标

**验证点**:
- WebSocket 连接成功建立
- 日志显示连接成功和订阅成功
- 持续接收实时数据
- `websocket_connections_active` 指标 = 1

---

#### Test 2: Redis 缓存验证

**测试步骤**:
1. 连接到 Redis
2. 查询 `market:Binance:BTC/USDT:latest`
3. 验证字段: bid, ask, last, volume, timestamp, quality_score
4. 验证 TTL 设置

**验证点**:
- Redis Hash 存在且数据正确
- 所有字段类型正确
- TTL ~1 小时

---

#### Test 3: ClickHouse 数据验证

**测试步骤**:
1. 连接到 ClickHouse
2. 查询 `market_data.unified_ticks` 表
3. 验证字段: source_type, source_name, asset_type, symbol, price, etc.
4. 验证分区和排序

**验证点**:
- ClickHouse 数据存在且正确
- source_type = 'CEX'
- asset_type = 'Spot'
- 分区格式正确: `YYYYMM-source_type-asset_type`

---

#### Test 4: Prometheus 指标验证

**测试步骤**:
1. 访问 `http://localhost:9090/metrics`
2. 验证指标存在性和格式
3. 验证指标数值合理性

**验证点**:
- `data_messages_received_total` 持续增长
- `data_message_latency_seconds` 延迟合理
- `websocket_connections_active` = 1
- `redis_write_latency_seconds` P99 < 1ms

---

### 5. 安全测试（Security Testing）

**目标**: 确保系统无已知安全漏洞

**策略**:
- 使用 Trivy 扫描 Docker 镜像漏洞
- 使用 `cargo audit` 检查依赖漏洞
- 代码审查敏感信息处理

**测试用例设计**:

#### Test 1: Docker 镜像安全扫描

**工具**: Trivy

**命令**:
```bash
trivy image hermesflow/data-engine:latest --severity HIGH,CRITICAL
```

**预期结果**: 0 HIGH/CRITICAL 漏洞

---

#### Test 2: 依赖安全审计

**工具**: cargo-audit

**命令**:
```bash
cargo audit
```

**预期结果**: 0 已知漏洞

---

#### Test 3: 敏感信息检查

**检查点**:
- API Key 不应硬编码在代码中
- 日志不应包含敏感信息（如 API Key, Secret）
- 配置文件中的敏感信息应通过环境变量注入

---

## 📊 测试指标和报告

### 测试完成标准

- [ ] 单元测试覆盖率 ≥ 85%
- [ ] 单元测试通过率 = 100%
- [ ] 集成测试通过率 = 100%
- [ ] 性能基准达标率 = 100%
- [ ] 手动验证测试通过率 = 100%
- [ ] 安全扫描: 0 HIGH/CRITICAL 漏洞
- [ ] P0/P1 缺陷数 = 0

### 测试报告内容

1. **测试执行摘要**:
   - 测试用例总数
   - 通过/失败/跳过数量
   - 代码覆盖率
   - 执行时间

2. **性能基准结果**:
   - 消息解析性能
   - Redis 写入性能
   - ClickHouse 批量写入性能
   - 端到端延迟

3. **缺陷列表**:
   - Bug ID, 优先级, 描述, 状态
   - 根本原因分析
   - 修复方案

4. **已知限制**:
   - 限制描述
   - 影响范围
   - 缓解措施

5. **测试风险**:
   - 风险描述
   - 影响和概率
   - 缓解计划

---

## 🚀 测试环境

### 开发环境

- **OS**: macOS 14 / Ubuntu 22.04
- **Rust**: 1.75.0
- **Docker**: 20.10+
- **Redis**: 7-alpine
- **ClickHouse**: latest

### CI 环境

- **Platform**: GitHub Actions
- **OS**: ubuntu-latest
- **Rust**: stable
- **Service Containers**: Redis, ClickHouse

---

## 📅 测试进度跟踪

### Week 1 (2025-10-28 ~ 2025-11-01)

- [x] 单元测试: `connectors::traits` (Day 2)
- [x] 单元测试: `models::asset` (Day 2)
- [x] 单元测试: `models::market_data` (Day 3)
- [x] 单元测试: `processors::parser` (Day 3)

### Week 2 (2025-11-04 ~ 2025-11-08)

- [x] 单元测试: `connectors::binance` (Day 6)
- [x] 单元测试: `processors::binance_parser` (Day 7)
- [x] 单元测试: `processors::normalizer` (Day 8)
- [x] 单元测试: `processors::quality` (Day 8)
- [x] 单元测试: `storage::redis` (Day 8)
- [x] 单元测试: `storage::clickhouse` (Day 9)
- [x] 集成测试 1-3 (Day 10)

### Week 3 (2025-11-11 ~ 2025-11-15)

- [x] 集成测试 4-5 (Day 11)
- [x] 性能基准测试 (Day 12)
- [x] 手动验证测试 (Day 13)
- [x] 安全测试 (Day 14)
- [x] 测试报告生成 (Day 15)

---

## 🎯 测试总结

本测试策略确保 Sprint 2 数据引擎的质量和性能满足需求。通过全面的单元测试、集成测试、性能基准测试和手动验证，我们将验证:

1. ✅ 通用架构框架的正确性和可扩展性
2. ✅ Binance 数据采集的准确性和可靠性
3. ✅ 数据质量和标准化的有效性
4. ✅ 性能指标满足高频交易需求
5. ✅ 系统无已知安全漏洞

---

**Last Updated**: 2025-10-21  
**Test Lead**: @sm.mdc  
**Status**: ✅ Approved







