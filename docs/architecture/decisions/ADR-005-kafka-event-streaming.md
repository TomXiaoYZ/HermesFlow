# ADR-005: Kafka作为事件流平台

**状态**: Accepted  
**日期**: 2024-12-20  
**决策者**: Architecture Team  
**相关人员**: 全体开发团队

---

## 上下文

HermesFlow采用微服务架构，服务间需要异步通信和事件驱动：

**通信需求**：
- 数据服务 → 策略引擎：实时市场数据推送
- 策略引擎 → 交易服务：策略信号传递
- 交易服务 → 风控服务：订单事件通知
- 各服务 → 日志中心：审计日志

**技术要求**：
- 高吞吐：峰值10K+ msg/s
- 低延迟：<10ms
- 持久化：消息不能丢失
- 回溯：支持历史消息重放

### 候选方案

| 消息系统 | 吞吐量 | 延迟 | 持久化 | 回溯 | 运维复杂度 | 社区 |
|---------|-------|------|--------|------|-----------|------|
| RabbitMQ | ★★★☆☆ | ★★★★☆ | ★★★★☆ | ★★☆☆☆ | ★★★☆☆ | ★★★★☆ |
| Kafka | ★★★★★ | ★★★☆☆ | ★★★★★ | ★★★★★ | ★★★☆☆ | ★★★★★ |
| Redis Pub/Sub | ★★★★☆ | ★★★★★ | ★☆☆☆☆ | ☆☆☆☆☆ | ★★★★★ | ★★★★☆ |
| NATS | ★★★★☆ | ★★★★★ | ★★★☆☆ | ★★★☆☆ | ★★★★☆ | ★★★☆☆ |

## 决策

选择**Apache Kafka**作为事件流平台。

### 主要理由

#### 1. 超高吞吐量

**基准测试**（3节点集群）：

```
单分区吞吐：
- 生产者：100K msg/s（1KB消息）
- 消费者：150K msg/s
- 磁盘写入：500MB/s

多分区吞吐（10分区）：
- 生产者：1M msg/s
- 消费者：1.5M msg/s

结论：Kafka轻松满足10K msg/s需求，且有10倍扩展空间
```

#### 2. 持久化与可靠性

**所有消息持久化到磁盘**：

```properties
# Kafka生产者配置
acks=all                      # 等待所有副本确认
retries=3                     # 失败重试3次
max.in.flight.requests=1      # 保证顺序
```

**副本机制保证高可用**：

```
Topic配置：
- replication.factor=3        # 3个副本
- min.insync.replicas=2       # 至少2个副本写入成功
- unclean.leader.election.enable=false  # 不允许非同步副本成为leader

结论：单节点故障不丢消息
```

#### 3. 消息回溯能力

**Kafka保留所有消息（可配置时长）**：

```properties
# Topic配置
retention.ms=604800000        # 保留7天
retention.bytes=1073741824    # 或者保留1GB

# 消费者可以从任意位置开始消费
consumer.seekToBeginning()    # 从最早消息开始
consumer.seek(offset)         # 从指定offset开始
```

**回溯场景**：
- 策略引擎崩溃后重启，从故障点继续消费
- 回测需要重放历史市场数据
- 审计需要追溯历史订单事件

#### 4. 事件驱动架构支持

**发布-订阅模式**：

```
Topic: market.data.btcusdt
Producer: 数据服务（1个）
Consumers:
  - 策略引擎 Group 1（多实例负载均衡）
  - 风控服务 Group 2（独立消费）
  - 日志服务 Group 3（独立消费）
  
每个Consumer Group独立消费，互不影响
```

### 技术实现

#### Topic设计

```
市场数据Topic：
market.data.{exchange}.{symbol}
例如：market.data.binance.btcusdt
分区：按symbol哈希分区（保证同一symbol消息有序）

策略信号Topic：
strategy.signal.{strategy_id}
例如：strategy.signal.ma-cross-123
分区：按strategy_id分区

订单事件Topic：
order.event.{type}
例如：order.event.created, order.event.filled
分区：按tenant_id分区（租户隔离）

风险告警Topic：
risk.alert.{tenant_id}
分区：按tenant_id分区
```

#### Rust生产者

```rust
use rdkafka::producer::{FutureProducer, FutureRecord};

pub struct MarketDataPublisher {
    producer: FutureProducer,
}

impl MarketDataPublisher {
    pub fn new(brokers: &str) -> Result<Self> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("acks", "all")
            .set("compression.type", "snappy")
            .set("linger.ms", "5")  // 批量发送，5ms延迟
            .create()?;
        
        Ok(Self { producer })
    }
    
    pub async fn publish(&self, symbol: &str, data: &MarketData) -> Result<()> {
        let topic = format!("market.data.binance.{}", symbol.to_lowercase());
        let key = symbol.to_string();
        let value = serde_json::to_vec(data)?;
        
        let record = FutureRecord::to(&topic)
            .key(&key)
            .payload(&value);
        
        self.producer.send(record, Duration::from_secs(0)).await
            .map_err(|(e, _)| anyhow::anyhow!("Failed to send: {}", e))?;
        
        Ok(())
    }
}
```

#### Python消费者

```python
from kafka import KafkaConsumer
import json

consumer = KafkaConsumer(
    'market.data.binance.btcusdt',
    bootstrap_servers=['localhost:9092'],
    group_id='strategy-engine-group',
    auto_offset_reset='earliest',  # 从最早消息开始
    enable_auto_commit=False,       # 手动提交offset
    value_deserializer=lambda m: json.loads(m.decode('utf-8'))
)

for message in consumer:
    try:
        market_data = message.value
        process_market_data(market_data)
        
        # 处理成功后提交offset
        consumer.commit()
    except Exception as e:
        logger.error(f"Failed to process message: {e}")
        # 失败时不提交，重启后重新消费
```

#### Java消费者

```java
@Service
public class OrderEventConsumer {
    
    @KafkaListener(
        topics = "order.event.filled",
        groupId = "risk-service-group",
        containerFactory = "kafkaListenerContainerFactory"
    )
    public void handleOrderFilled(ConsumerRecord<String, OrderEvent> record) {
        String tenantId = record.key();
        OrderEvent event = record.value();
        
        // 处理订单成交事件
        riskService.checkPosition(tenantId, event);
    }
}
```

## 后果

### 优点

1. **高吞吐低延迟**：
   - 峰值处理能力>1M msg/s
   - 端到端延迟<10ms（P95）
   - 适合实时流处理

2. **持久化与可靠性**：
   - 所有消息持久化
   - 副本机制保证高可用
   - 消息不丢失

3. **解耦服务**：
   - 生产者和消费者独立部署
   - 消费者可以按需扩缩容
   - 新增消费者无需修改生产者

4. **回溯能力**：
   - 支持从任意offset消费
   - 便于调试和回测
   - 审计日志可追溯

5. **生态成熟**：
   - 社区活跃
   - 第三方工具丰富
   - 企业级采用广泛

### 缺点

1. **运维复杂**：
   - 需要部署ZooKeeper（或KRaft）
   - 配置参数众多
   - 监控和调优需要经验

2. **延迟不是最低**：
   - 相比Redis Pub/Sub延迟较高
   - 批量发送增加延迟
   - 不适合<1ms延迟场景

3. **资源消耗**：
   - 磁盘IO较高
   - 内存占用较大（页缓存）
   - 网络带宽需求高

4. **学习曲线**：
   - 概念多（Topic/Partition/Consumer Group）
   - 调优需要深入理解
   - 错误排查困难

### 缓解措施

1. **容器化部署**：
   ```yaml
   # docker-compose.yml
   zookeeper:
     image: confluentinc/cp-zookeeper:latest
     environment:
       ZOOKEEPER_CLIENT_PORT: 2181
   
   kafka:
     image: confluentinc/cp-kafka:latest
     depends_on:
       - zookeeper
     environment:
       KAFKA_BROKER_ID: 1
       KAFKA_ZOOKEEPER_CONNECT: zookeeper:2181
       KAFKA_ADVERTISED_LISTENERS: PLAINTEXT://kafka:9092
   ```

2. **监控告警**：
   ```
   关键指标：
   - kafka.server:type=BrokerTopicMetrics,name=MessagesInPerSec
   - kafka.server:type=BrokerTopicMetrics,name=BytesInPerSec
   - kafka.server:type=ReplicaManager,name=UnderReplicatedPartitions
   - kafka.consumer:type=consumer-fetch-manager-metrics,client-id=*
   
   告警规则：
   - UnderReplicatedPartitions > 0（副本不同步）
   - Consumer Lag > 10000（消费积压）
   ```

3. **性能优化**：
   ```properties
   # 生产者优化
   batch.size=16384              # 批量大小
   linger.ms=5                   # 批量等待时间
   compression.type=snappy       # 压缩
   
   # 消费者优化
   fetch.min.bytes=1024          # 批量拉取
   max.poll.records=500          # 单次拉取记录数
   ```

4. **最佳实践文档**：
   - Topic命名规范
   - 分区数量规划
   - 消费者错误处理
   - offset管理策略

## 实施经验

### 3个月后回顾

**成功点**：
- ✅ 稳定处理5K msg/s，峰值10K msg/s
- ✅ 无消息丢失事件
- ✅ 服务解耦，独立部署升级
- ✅ 回溯功能用于调试和回测

**挑战点**：
- ⚠️ 初期ZooKeeper配置错误导致集群不稳定
- ⚠️ Consumer Lag监控不及时
- ⚠️ Topic分区数量规划不合理

**改进建议**：
1. 使用KRaft模式替代ZooKeeper（简化运维）
2. 建立Consumer Lag实时监控
3. 定期审计Topic分区数量
4. 建立Kafka运维知识库

## 备选方案

### 为什么不选择RabbitMQ？

虽然RabbitMQ易用性更好，但：
- 吞吐量不如Kafka
- 不支持消息回溯
- 不适合流处理场景

**结论**：对于高吞吐+持久化+回溯需求，Kafka更合适。

### 为什么不选择Redis Pub/Sub？

虽然Redis延迟最低，但：
- 消息不持久化（重启丢失）
- 无消息回溯
- 吞吐量有限

**结论**：Redis适合实时通知，不适合关键业务消息。

## 相关决策

- [ADR-001: 采用混合技术栈架构](./ADR-001-hybrid-tech-stack.md)
- [ADR-002: 选择Tokio作为Rust异步运行时](./ADR-002-tokio-runtime.md)

## 参考资料

1. [Kafka官方文档](https://kafka.apache.org/documentation/)
2. [Kafka: The Definitive Guide](https://www.confluent.io/resources/kafka-the-definitive-guide/)
3. [Kafka性能调优](https://www.confluent.io/blog/configure-kafka-to-minimize-latency/)
4. [Kafka监控最佳实践](https://www.datadoghq.com/blog/monitoring-kafka-performance-metrics/)

