# ADR-001: 采用混合技术栈架构

**状态**: Accepted  
**日期**: 2024-12-20  
**决策者**: Architecture Team  
**相关人员**: 全体开发团队

---

## 上下文

HermesFlow量化交易平台需要同时满足以下需求：

1. **极低延迟**：市场数据采集和处理需要μs级响应时间
2. **高可靠性**：交易执行、订单管理需要成熟稳定的企业级框架
3. **快速开发**：策略开发需要快速原型和科学计算能力
4. **成本优化**：在满足性能的前提下控制开发和运维成本

传统的单一技术栈方案存在明显短板：
- **纯Java方案**：虽然生态成熟，但数据处理性能不足，延迟较高
- **纯Python方案**：虽然科学计算能力强，但并发性能差，不适合高频交易
- **纯Rust方案**：虽然性能优异，但生态不够成熟，开发效率较低

## 决策

采用**混合技术栈架构**，根据各层的特点选择最合适的技术：

### 技术栈分工

| 层级 | 技术 | 理由 | 职责 |
|------|------|------|------|
| **数据层** | Rust | 超低延迟、内存安全、高并发 | 实时数据采集、标准化、存储 |
| **业务层** | Java | 成熟生态、虚拟线程、企业级 | 用户管理、交易执行、风控 |
| **策略层** | Python | 科学计算、快速原型、丰富库 | 策略开发、回测、因子计算 |

### 技术栈详细说明

#### Rust数据层

**选择理由**：
- 零成本抽象，性能接近C/C++
- 编译时保证内存安全，无数据竞争
- Tokio异步运行时，高效处理大量并发连接
- 无垃圾回收（GC），延迟稳定可预测

**关键依赖**：
```toml
[dependencies]
tokio = { version = "1.35", features = ["full"] }
actix-web = "4.4"
serde = { version = "1.0", features = ["derive"] }
rdkafka = "0.36"
clickhouse-rs = "1.0"
redis-rs = "0.24"
```

**性能基线**：
- WebSocket消息延迟: < 1ms（P99）
- 数据处理吞吐: > 100K msg/s
- ClickHouse批量写入: > 1M rows/s

#### Java业务层

**选择理由**：
- Spring Boot生态完善，开发效率高
- JDK 21虚拟线程（Project Loom），高并发低开销
- 企业级安全和事务支持
- 丰富的第三方库和工具链

**技术栈**：
```xml
<dependencies>
    <dependency>
        <groupId>org.springframework.boot</groupId>
        <artifactId>spring-boot-starter-web</artifactId>
        <version>3.2.0</version>
    </dependency>
    <dependency>
        <groupId>org.springframework.boot</groupId>
        <artifactId>spring-boot-starter-data-jpa</artifactId>
    </dependency>
    <dependency>
        <groupId>org.springframework.kafka</groupId>
        <artifactId>spring-kafka</artifactId>
    </dependency>
</dependencies>
```

**适用场景**：
- 用户认证与授权
- 订单管理与执行
- 风险控制与监控
- API Gateway

#### Python策略层

**选择理由**：
- NumPy/Pandas科学计算生态成熟
- 策略开发周期短，便于快速迭代
- 丰富的量化和机器学习库
- 社区活跃，资源丰富

**技术栈**：
```python
# pyproject.toml
[tool.poetry.dependencies]
python = "^3.12"
fastapi = "^0.108"
pandas = "^2.1"
numpy = "^1.26"
numba = "^0.58"
scikit-learn = "^1.3"
```

**适用场景**：
- 策略开发与回测
- Alpha因子计算
- 参数优化
- 机器学习模型训练

### 服务间通信

**同步通信**：gRPC（高性能二进制协议）
```protobuf
service MarketDataService {
  rpc GetLatestPrice(PriceRequest) returns (PriceResponse);
  rpc SubscribeData(stream DataRequest) returns (stream MarketData);
}
```

**异步通信**：Kafka（事件驱动）
```
Topic: market.data.btcusdt
Producer: Rust数据服务
Consumer: Python策略引擎
```

## 后果

### 优点

1. **性能优化**：
   - 数据层延迟降低90%（与纯Java方案对比）
   - 策略开发效率提升50%（与纯Rust方案对比）
   - 交易执行可靠性提升（企业级Java框架）

2. **技术优势互补**：
   - 每层都使用最合适的技术
   - 降低单一技术的短板影响
   - 提升整体系统质量

3. **团队灵活性**：
   - 不同背景的开发者可以贡献各自擅长的领域
   - 降低招聘难度

### 缺点

1. **团队技能要求高**：
   - 需要维护3套技术栈
   - 团队需要多技能成员
   - 学习曲线陡峭

2. **运维复杂度增加**：
   - 需要管理多种运行时环境
   - 日志和监控需要统一标准
   - 部署流程相对复杂

3. **调试难度**：
   - 跨语言问题定位困难
   - 需要完善的分布式追踪

### 缓解措施

1. **统一标准**：
   - 统一日志格式（JSON结构化）
   - 统一监控指标（Prometheus）
   - 统一API规范（OpenAPI/gRPC）

2. **容器化**：
   - Docker统一运行环境
   - Kubernetes统一编排
   - 简化部署流程

3. **文档与培训**：
   - 编写详细的开发指南
   - 定期技术分享会
   - 建立最佳实践库

4. **工具链**：
   - 统一IDE配置
   - CI/CD自动化
   - 代码质量检查

## 经验教训

### 实施3个月后回顾

**成功点**：
- Rust数据层性能符合预期，WebSocket延迟<1ms
- Python策略开发效率高，2周内实现5个策略
- Java交易服务稳定可靠，无严重Bug

**改进点**：
- 团队成员Rust学习曲线较陡，需要更多培训
- gRPC跨语言调试工具不足，需要自研工具
- 监控指标命名不一致，需要统一规范

**建议**：
- 新加入团队成员先从Python层开始
- 建立混合技术栈最佳实践库
- 投入资源开发内部开发工具

## 相关决策

- [ADR-002: 选择Tokio作为Rust异步运行时](./ADR-002-tokio-runtime.md)
- [ADR-007: Alpha因子库使用Numba加速](./ADR-007-numba-acceleration.md)

## 参考资料

1. [Rust官方文档](https://doc.rust-lang.org/)
2. [Spring Boot官方文档](https://spring.io/projects/spring-boot)
3. [Python科学计算生态](https://www.scipy.org/)
4. "Designing Data-Intensive Applications" by Martin Kleppmann

