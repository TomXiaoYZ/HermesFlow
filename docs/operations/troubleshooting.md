# HermesFlow 故障排查手册

**版本**: v1.0.0  
**最后更新**: 2024-12-20  
**维护者**: Architecture Team

---

## 目录

1. [常见问题FAQ](#1-常见问题faq)
2. [日志分析方法](#2-日志分析方法)
3. [应急响应流程](#3-应急响应流程)
4. [性能诊断工具](#4-性能诊断工具)
5. [监控告警处理](#5-监控告警处理)
6. [故障案例库](#6-故障案例库)

---

## 1. 常见问题FAQ

### 1.1 服务启动失败

#### Q1.1.1: Rust数据采集服务启动失败

**症状**:
```
Error: Failed to bind to address 0.0.0.0:18001
```

**原因**: 端口被占用

**解决方案**:
```bash
# 查看端口占用
lsof -i :18001

# 或使用netstat
netstat -an | grep 18001

# 杀死占用进程
kill -9 <PID>

# 或修改配置文件中的端口
export DATA_SERVICE_PORT=18101
```

**预防措施**:
- 使用环境变量配置端口
- 启动前检查端口可用性
- 使用容器隔离避免端口冲突

#### Q1.1.2: Java服务启动失败 - JVM版本不兼容

**症状**:
```
Error: A JNI error has occurred, please check your installation
Unsupported class file major version 65
```

**原因**: JDK版本低于21

**解决方案**:
```bash
# 检查Java版本
java -version

# 应该显示: openjdk version "21.0.x"

# 如果版本不对，安装JDK 21
# macOS
brew install openjdk@21

# Ubuntu
sudo apt install openjdk-21-jdk

# 设置JAVA_HOME
export JAVA_HOME=/path/to/jdk-21
export PATH=$JAVA_HOME/bin:$PATH
```

#### Q1.1.3: Python策略引擎导入错误

**症状**:
```
ModuleNotFoundError: No module named 'fastapi'
```

**原因**: 依赖未安装或虚拟环境未激活

**解决方案**:
```bash
# 激活虚拟环境
cd modules/strategy-engine
python -m venv venv
source venv/bin/activate  # macOS/Linux
# 或 venv\Scripts\activate  # Windows

# 安装依赖
pip install -r requirements.txt

# 验证安装
python -c "import fastapi; print(fastapi.__version__)"
```

### 1.2 数据库连接问题

#### Q1.2.1: PostgreSQL连接超时

**症状**:
```
psycopg2.OperationalError: could not connect to server: Connection timed out
```

**原因分析**:
1. PostgreSQL服务未启动
2. 防火墙阻止连接
3. 连接字符串错误
4. max_connections达到上限

**解决方案**:
```bash
# 1. 检查PostgreSQL状态
sudo systemctl status postgresql
# 或
docker ps | grep postgres

# 2. 检查防火墙
sudo ufw status
sudo ufw allow 5432/tcp

# 3. 验证连接
psql -h localhost -p 5432 -U hermesflow -d hermesflow

# 4. 检查连接数
SELECT count(*) FROM pg_stat_activity;

# 查看max_connections
SHOW max_connections;

# 如果达到上限，调整配置
# 编辑 postgresql.conf
max_connections = 200
```

#### Q1.2.2: ClickHouse "Too many connections"

**症状**:
```
Code: 210. DB::NetException: Too many connections
```

**原因**: 连接池配置不当或连接泄漏

**解决方案**:
```bash
# 1. 检查当前连接数
SELECT count() FROM system.processes;

# 2. 查看连接详情
SELECT 
    user,
    query_id,
    elapsed,
    query 
FROM system.processes 
ORDER BY elapsed DESC;

# 3. 杀死长时间运行的查询
KILL QUERY WHERE query_id = 'xxx';

# 4. 调整max_connections配置
# 编辑 /etc/clickhouse-server/config.xml
<max_connections>1000</max_connections>
```

**代码修复（Rust）**:
```rust
// 正确使用连接池
use clickhouse::Client;

let client = Client::default()
    .with_url("http://localhost:8123")
    .with_pool_size(10, 50);  // min=10, max=50

// 确保连接及时释放
async fn query_data(client: &Client) -> Result<()> {
    let result = client
        .query("SELECT * FROM market_data")
        .fetch_all::<Row>()
        .await?;
    
    // 连接自动释放
    Ok(())
}
```

### 1.3 Redis连接超时

#### Q1.3.1: Redis "READONLY You can't write against a read only replica"

**症状**:
```
redis.exceptions.ReadOnlyError: You can't write against a read only replica
```

**原因**: 连接到了Redis从节点

**解决方案**:
```bash
# 1. 检查Redis角色
redis-cli -h localhost -p 6379 INFO replication

# 2. 确认主节点地址
# 应该显示: role:master

# 3. 如果是从节点，连接到主节点
# 或临时将从节点提升为主节点
redis-cli -h localhost -p 6379 REPLICAOF NO ONE
```

#### Q1.3.2: Redis连接数耗尽

**症状**:
```
Error: max number of clients reached
```

**解决方案**:
```bash
# 1. 检查当前连接数
redis-cli INFO clients

# 2. 查看maxclients配置
redis-cli CONFIG GET maxclients

# 3. 增加maxclients
redis-cli CONFIG SET maxclients 20000

# 4. 永久修改（编辑redis.conf）
maxclients 20000

# 5. 检查是否有连接泄漏
redis-cli CLIENT LIST | wc -l
```

### 1.4 Kafka消息丢失

#### Q1.4.1: 消息未被消费

**症状**: Producer发送成功，但Consumer未收到消息

**排查步骤**:
```bash
# 1. 检查Topic是否存在
kafka-topics --bootstrap-server localhost:9092 \
    --list | grep market_data

# 2. 查看Topic详情
kafka-topics --bootstrap-server localhost:9092 \
    --describe --topic market_data

# 3. 查看消费者组状态
kafka-consumer-groups --bootstrap-server localhost:9092 \
    --group hermesflow-group \
    --describe

# 4. 手动消费测试
kafka-console-consumer --bootstrap-server localhost:9092 \
    --topic market_data \
    --from-beginning
```

**常见原因**:
1. ✅ Consumer Group ID不匹配
2. ✅ 分区分配问题
3. ✅ Offset管理错误
4. ✅ 消息过期（retention.ms）

**解决方案**:
```rust
// Rust Consumer正确配置
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::ClientConfig;

let consumer: StreamConsumer = ClientConfig::new()
    .set("bootstrap.servers", "localhost:9092")
    .set("group.id", "hermesflow-group")
    .set("auto.offset.reset", "earliest")  // 从最早的消息开始
    .set("enable.auto.commit", "false")     // 手动提交offset
    .create()?;

consumer.subscribe(&["market_data"])?;
```

### 1.5 WebSocket断连

#### Q1.5.1: WebSocket频繁断开重连

**症状**: 日志显示WebSocket连接频繁断开并重连

**原因分析**:
1. 网络不稳定
2. 心跳超时
3. 交易所限流
4. 服务器资源不足

**解决方案**:
```rust
// Rust WebSocket客户端增强版
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use tokio::time::{interval, Duration};

async fn websocket_with_heartbeat(url: &str) -> Result<()> {
    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();
    
    // 心跳定时器（每30秒）
    let mut heartbeat = interval(Duration::from_secs(30));
    
    loop {
        tokio::select! {
            Some(msg) = read.next() => {
                match msg? {
                    Message::Text(text) => {
                        // 处理消息
                        process_message(&text).await?;
                    }
                    Message::Ping(_) => {
                        // 自动响应Pong
                    }
                    Message::Close(_) => {
                        tracing::warn!("WebSocket closed by server");
                        break;
                    }
                    _ => {}
                }
            }
            _ = heartbeat.tick() => {
                // 发送心跳
                write.send(Message::Ping(vec![])).await?;
                tracing::debug!("Heartbeat sent");
            }
        }
    }
    
    // 自动重连
    tokio::time::sleep(Duration::from_secs(5)).await;
    websocket_with_heartbeat(url).await
}
```

#### Q1.5.2: WebSocket "Too many open files"

**症状**:
```
Error: Too many open files (os error 24)
```

**原因**: 系统文件描述符限制

**解决方案**:
```bash
# 1. 检查当前限制
ulimit -n

# 2. 临时增加限制
ulimit -n 65536

# 3. 永久修改（编辑 /etc/security/limits.conf）
*  soft  nofile  65536
*  hard  nofile  65536

# 4. 重启服务使配置生效

# 5. 验证
ulimit -n
# 应该显示: 65536
```

### 1.6 性能问题

#### Q1.6.1: Rust服务CPU使用率过高

**症状**: Rust数据服务CPU持续>80%

**排查步骤**:
```bash
# 1. 查看进程CPU使用
top -p <PID>

# 2. 使用perf分析（Linux）
perf record -F 99 -p <PID> -g -- sleep 30
perf report

# 3. 生成火焰图
cargo flamegraph --pid <PID>

# 4. 查看线程状态
ps -eLf | grep data-service
```

**常见原因**:
1. ✅ 无限循环
2. ✅ 繁忙等待（busy-wait）
3. ✅ 大量字符串拷贝
4. ✅ 序列化/反序列化性能瓶颈

**优化方案**:
```rust
// ❌ 错误：繁忙等待
loop {
    if let Some(data) = try_receive() {
        process(data);
    }
    // 空循环占用CPU
}

// ✅ 正确：使用异步等待
while let Some(data) = receiver.recv().await {
    process(data).await;
}

// ❌ 错误：频繁克隆
for item in large_vec.iter() {
    let cloned = item.clone();  // 避免不必要的克隆
    process(cloned);
}

// ✅ 正确：使用引用
for item in large_vec.iter() {
    process(item);  // 直接使用引用
}
```

#### Q1.6.2: Java服务内存占用过高

**症状**: Java服务内存持续增长，最终OOM

**排查步骤**:
```bash
# 1. 查看JVM内存使用
jstat -gcutil <PID> 1000

# 2. 生成堆转储
jmap -dump:format=b,file=heap.bin <PID>

# 3. 分析堆转储（使用MAT或VisualVM）
mat heap.bin

# 4. 查看GC日志
# 启动参数添加:
-Xlog:gc*:file=gc.log
```

**常见原因**:
1. ✅ 内存泄漏
2. ✅ 大对象频繁创建
3. ✅ 缓存未设置上限
4. ✅ 连接未关闭

**优化方案**:
```java
// ✅ 使用try-with-resources自动关闭资源
try (Connection conn = dataSource.getConnection();
     PreparedStatement stmt = conn.prepareStatement(sql)) {
    ResultSet rs = stmt.executeQuery();
    // 处理结果
}  // 自动关闭

// ✅ 设置缓存大小限制
@Configuration
public class CacheConfig {
    @Bean
    public CacheManager cacheManager() {
        return CacheManagerBuilder.newCacheManagerBuilder()
            .withCache("marketData",
                CacheConfigurationBuilder.newCacheConfigurationBuilder(
                    String.class, MarketData.class,
                    ResourcePoolsBuilder.heap(10000))  // 限制10000条
                .withExpiry(ExpiryPolicyBuilder.timeToLiveExpiration(Duration.ofMinutes(5)))
            ).build(true);
    }
}
```

### 1.7 数据不一致

#### Q1.7.1: 多租户数据泄漏

**症状**: 租户A能看到租户B的数据

**紧急处理**:
```bash
# 1. 立即停止受影响的服务
kubectl scale deployment user-management --replicas=0

# 2. 检查RLS策略
psql -U hermesflow -d hermesflow << EOF
SELECT schemaname, tablename, policyname, permissive, roles, cmd, qual 
FROM pg_policies 
WHERE tablename IN ('strategies', 'orders');
EOF

# 3. 验证RLS是否启用
SELECT tablename, row_security 
FROM pg_tables 
WHERE tablename IN ('strategies', 'orders');
```

**根因分析**:
```sql
-- 检查是否有绕过RLS的查询
SELECT query 
FROM pg_stat_statements 
WHERE query LIKE '%BYPASS ROW LEVEL SECURITY%' 
   OR query LIKE '%SET LOCAL row_security%';

-- 验证tenant_id设置
SHOW app.current_tenant;
```

**修复方案**:
```sql
-- 1. 确保RLS策略正确
DROP POLICY IF EXISTS tenant_isolation_strategies ON strategies;

CREATE POLICY tenant_isolation_strategies ON strategies
    FOR ALL
    USING (tenant_id = current_setting('app.current_tenant', true));

-- 2. 确保RLS已启用
ALTER TABLE strategies ENABLE ROW LEVEL SECURITY;
ALTER TABLE strategies FORCE ROW LEVEL SECURITY;

-- 3. 撤销可能绕过RLS的权限
REVOKE BYPASSRLS ON ALL TABLES IN SCHEMA public FROM hermesflow_app;
```

---

## 2. 日志分析方法

### 2.1 Rust服务日志（tracing）

#### 日志级别配置

**环境变量**:
```bash
# 全局DEBUG级别
export RUST_LOG=debug

# 按模块设置
export RUST_LOG=data_service=debug,actix_web=info,tokio=warn

# 生产环境推荐
export RUST_LOG=data_service=info,actix_web=warn
```

#### 日志格式

**标准格式**:
```
2024-12-20T10:30:45.123Z INFO  data_service::connectors::binance: WebSocket connected symbol="BTCUSDT" connection_id="conn_123"
```

**字段说明**:
- `2024-12-20T10:30:45.123Z`: 时间戳（UTC）
- `INFO`: 日志级别
- `data_service::connectors::binance`: 模块路径
- `WebSocket connected`: 消息
- `symbol="BTCUSDT"`: 结构化字段

#### 常见日志模式

**正常运行**:
```
INFO  WebSocket connected symbol="BTCUSDT"
INFO  Market data received symbol="BTCUSDT" price=43250.50
DEBUG Data stored to Redis key="market:BTCUSDT:latest"
```

**错误模式**:
```
ERROR WebSocket connection failed error="Connection refused (os error 61)"
WARN  Reconnecting in 5 seconds attempt=1
ERROR Failed to parse market data error="missing field `symbol`" raw_data="..."
```

#### 日志查询示例

```bash
# 查看错误日志
grep ERROR data-service.log | tail -100

# 统计错误频率
grep ERROR data-service.log | cut -d' ' -f5- | sort | uniq -c | sort -rn

# 查看特定symbol的日志
grep "symbol=\"BTCUSDT\"" data-service.log

# 查看最近10分钟的日志
awk -v date="$(date -u -d '10 minutes ago' '+%Y-%m-%dT%H:%M')" '$1 > date' data-service.log
```

### 2.2 Java服务日志（logback）

#### 日志配置（logback-spring.xml）

```xml
<configuration>
    <appender name="FILE" class="ch.qos.logback.core.rolling.RollingFileAppender">
        <file>logs/user-management.log</file>
        <rollingPolicy class="ch.qos.logback.core.rolling.TimeBasedRollingPolicy">
            <fileNamePattern>logs/user-management.%d{yyyy-MM-dd}.log</fileNamePattern>
            <maxHistory>30</maxHistory>
        </rollingPolicy>
        <encoder>
            <pattern>%d{yyyy-MM-dd HH:mm:ss.SSS} [%thread] %-5level %logger{36} - %msg%n</pattern>
        </encoder>
    </appender>
    
    <logger name="com.hermesflow" level="DEBUG"/>
    <logger name="org.springframework" level="INFO"/>
    
    <root level="INFO">
        <appender-ref ref="FILE"/>
    </root>
</configuration>
```

#### 日志示例

```
2024-12-20 10:30:45.123 [http-nio-18010-exec-1] INFO  c.h.user.controller.AuthController - User login attempt username="user@example.com"
2024-12-20 10:30:45.456 [http-nio-18010-exec-1] DEBUG c.h.user.service.AuthService - JWT token generated user_id="usr_123" expires_at="2024-12-20T11:30:45Z"
```

#### 关键日志查询

```bash
# 查看所有ERROR级别日志
grep ERROR user-management.log

# 查看特定用户的操作日志
grep "user_id=\"usr_123\"" user-management.log

# 统计API调用次数
grep "API Request" user-management.log | awk '{print $8}' | sort | uniq -c

# 查看慢查询（>1000ms）
grep "Query executed" user-management.log | awk -F'duration=' '{if($2+0>1000) print}'
```

### 2.3 Python服务日志（logging）

#### 日志配置

```python
import logging
from logging.handlers import RotatingFileHandler

# 配置日志
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s [%(process)d] [%(levelname)s] %(name)s: %(message)s',
    handlers=[
        RotatingFileHandler(
            'logs/strategy-engine.log',
            maxBytes=100*1024*1024,  # 100MB
            backupCount=10
        ),
        logging.StreamHandler()
    ]
)

logger = logging.getLogger(__name__)
```

#### 结构化日志

```python
import structlog

structlog.configure(
    processors=[
        structlog.processors.TimeStamper(fmt="iso"),
        structlog.processors.JSONRenderer()
    ]
)

log = structlog.get_logger()
log.info("strategy_executed", 
         strategy_id="stg_123", 
         symbol="BTCUSDT", 
         signal="BUY",
         price=43250.50)
```

### 2.4 集中式日志（ELK）

#### Elasticsearch查询

```json
// 查询最近1小时的错误日志
GET /logs-*/_search
{
  "query": {
    "bool": {
      "must": [
        {"match": {"level": "ERROR"}},
        {"range": {"@timestamp": {"gte": "now-1h"}}}
      ]
    }
  },
  "sort": [{"@timestamp": "desc"}],
  "size": 100
}

// 聚合分析错误类型
GET /logs-*/_search
{
  "size": 0,
  "query": {
    "match": {"level": "ERROR"}
  },
  "aggs": {
    "error_types": {
      "terms": {"field": "error.keyword", "size": 20}
    }
  }
}
```

---

## 3. 应急响应流程

### 3.1 服务降级SOP

**触发条件**:
- 系统负载>80%持续5分钟
- API错误率>5%
- 数据库连接池耗尽

**响应步骤**:

**Step 1: 评估影响范围** (2分钟)
```bash
# 检查各服务状态
kubectl get pods -n hermesflow

# 查看资源使用
kubectl top pods -n hermesflow

# 检查错误率
curl http://localhost:9090/api/v1/query?query=rate(http_requests_total{code=~"5.."}[5m])
```

**Step 2: 启动降级措施** (5分钟)
```bash
# 1. 降级非核心功能
# 关闭回测服务
kubectl scale deployment backtest-engine --replicas=0

# 2. 限制并发
# 调整Nginx限流
kubectl edit configmap nginx-config
# rate=100r/s → rate=50r/s

# 3. 启用缓存
# 增加Redis缓存过期时间
redis-cli CONFIG SET maxmemory-policy allkeys-lru
```

**Step 3: 通知相关方** (立即)
```bash
# 发送告警
curl -X POST https://hooks.slack.com/services/XXX \
  -d '{"text":"⚠️ 系统进入降级模式"}}'
```

**Step 4: 监控恢复** (持续)
```bash
# 持续监控指标
watch -n 5 'kubectl top pods'
```

**Step 5: 恢复服务** (负载<60%时)
```bash
# 逐步恢复服务
kubectl scale deployment backtest-engine --replicas=2

# 恢复限流配置
kubectl rollout restart deployment nginx
```

### 3.2 数据回滚SOP

**触发条件**:
- 检测到数据错误
- 批量操作失败
- 数据库迁移失败

**前提条件**:
- ✅ 已有备份
- ✅ 已验证备份可用性
- ✅ 已获得审批

**回滚步骤**:

**Step 1: 停止写入** (立即)
```bash
# 设置数据库为只读
psql -U postgres -d hermesflow << EOF
ALTER DATABASE hermesflow SET default_transaction_read_only = on;
EOF

# 停止写入服务
kubectl scale deployment trading-engine --replicas=0
```

**Step 2: 导出当前数据** (5分钟)
```bash
# 导出当前状态（以防万一）
pg_dump -U hermesflow -d hermesflow -F c -f /backup/before_rollback_$(date +%Y%m%d_%H%M%S).dump
```

**Step 3: 执行回滚** (30分钟)
```bash
# 从备份恢复
pg_restore -U hermesflow -d hermesflow -c /backup/hermesflow_20241220_080000.dump

# 验证数据
psql -U hermesflow -d hermesflow << EOF
SELECT COUNT(*) FROM strategies;
SELECT COUNT(*) FROM orders;
EOF
```

**Step 4: 恢复服务** (10分钟)
```bash
# 恢复写入
psql -U postgres -d hermesflow << EOF
ALTER DATABASE hermesflow SET default_transaction_read_only = off;
EOF

# 重启服务
kubectl scale deployment trading-engine --replicas=3
```

**Step 5: 验证恢复** (15分钟)
```bash
# 执行冒烟测试
./scripts/smoke-test.sh

# 检查数据一致性
./scripts/data-consistency-check.sh
```

### 3.3 紧急扩容SOP

**触发条件**:
- CPU使用率>80%持续10分钟
- 内存使用率>85%
- 请求队列积压>1000

**扩容步骤**:

**Step 1: 快速扩容** (2分钟)
```bash
# 水平扩展Pod
kubectl scale deployment data-service --replicas=5

# 垂直扩展（增加资源）
kubectl set resources deployment data-service \
  --limits=cpu=4,memory=8Gi \
  --requests=cpu=2,memory=4Gi
```

**Step 2: 扩展数据库** (10分钟)
```bash
# PostgreSQL添加只读副本
kubectl apply -f k8s/postgres-replica.yaml

# Redis添加分片
redis-cli --cluster add-node new-node:6379 existing-node:6379
```

**Step 3: 负载均衡调整** (5分钟)
```bash
# 更新Nginx配置
kubectl edit configmap nginx-config

# upstream backend {
#     server data-service-1:18001;
#     server data-service-2:18001;
#     server data-service-3:18001;
#     server data-service-4:18001;
#     server data-service-5:18001;
# }
```

---

## 4. 性能诊断工具

### 4.1 Rust工具

#### cargo flamegraph
```bash
# 安装
cargo install flamegraph

# 生成火焰图
cargo flamegraph --bin data-service

# 分析特定函数
cargo flamegraph --bin data-service --features profiling
```

#### perf (Linux)
```bash
# 记录性能数据
perf record -F 99 -p <PID> -g -- sleep 30

# 生成报告
perf report

# 生成火焰图
perf script | stackcollapse-perf.pl | flamegraph.pl > flamegraph.svg
```

#### valgrind (内存检查)
```bash
# 检查内存泄漏
valgrind --leak-check=full --show-leak-kinds=all ./target/release/data-service
```

### 4.2 Java工具

#### JProfiler
```bash
# 启动参数
java -agentpath:/path/to/libjprofilerti.so=port=8849 -jar user-management.jar

# 连接JProfiler GUI
# File → Attach → localhost:8849
```

#### Java Flight Recorder (JFR)
```bash
# 启动时启用
java -XX:StartFlightRecording=duration=60s,filename=recording.jfr -jar user-management.jar

# 运行时启用
jcmd <PID> JFR.start duration=60s filename=recording.jfr

# 分析
jfr print recording.jfr
```

#### VisualVM
```bash
# 启动VisualVM
jvisualvm

# 连接到进程
# 文件 → 添加JMX连接 → localhost:<jmx-port>
```

### 4.3 Python工具

#### py-spy
```bash
# 安装
pip install py-spy

# 实时采样
py-spy top --pid <PID>

# 生成火焰图
py-spy record -o profile.svg --pid <PID>
```

#### cProfile
```python
import cProfile
import pstats

# 性能分析
profiler = cProfile.Profile()
profiler.enable()

# 执行代码
run_strategy()

profiler.disable()
stats = pstats.Stats(profiler)
stats.sort_stats('cumulative')
stats.print_stats(20)
```

#### memory_profiler
```python
from memory_profiler import profile

@profile
def process_large_data():
    # 分析内存使用
    large_list = [i for i in range(1000000)]
    return sum(large_list)
```

---

## 5. 监控告警处理

### 5.1 Prometheus告警规则

**配置示例** (`prometheus-rules.yaml`):
```yaml
groups:
  - name: hermesflow_alerts
    interval: 30s
    rules:
      # 高错误率告警
      - alert: HighErrorRate
        expr: rate(http_requests_total{code=~"5.."}[5m]) > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value | humanizePercentage }}"
      
      # 高CPU使用率
      - alert: HighCPUUsage
        expr: rate(process_cpu_seconds_total[5m]) > 0.8
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High CPU usage"
          description: "CPU usage is {{ $value | humanizePercentage }}"
      
      # 数据库连接池告警
      - alert: DatabaseConnectionPoolExhausted
        expr: pg_stat_database_numbackends / pg_settings_max_connections > 0.9
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Database connection pool nearly exhausted"
```

### 5.2 告警处理流程

#### P0告警（立即响应）
- 服务完全不可用
- 数据泄漏
- 安全漏洞

**响应时间**: 5分钟内  
**处理流程**:
1. ✅ 立即确认告警
2. ✅ 评估影响范围
3. ✅ 启动应急预案
4. ✅ 通知相关人员
5. ✅ 实时更新状态

#### P1告警（紧急）
- 性能严重下降
- 部分功能不可用
- 数据延迟>30秒

**响应时间**: 15分钟内  
**处理流程**:
1. ✅ 确认告警
2. ✅ 分析根因
3. ✅ 实施临时修复
4. ✅ 计划永久修复

#### P2告警（重要）
- 资源使用率高
- 非核心功能异常

**响应时间**: 1小时内  
**处理流程**:
1. ✅ 记录告警
2. ✅ 排期修复
3. ✅ 监控趋势

---

## 6. 故障案例库

### 案例1: Redis OOM导致服务中断

**时间**: 2024-12-15 14:30  
**持续时间**: 45分钟  
**影响**: 所有服务无法访问缓存

**根因**: Redis内存配置不当，未设置淘汰策略

**解决方案**:
```bash
# 设置最大内存
redis-cli CONFIG SET maxmemory 4gb

# 设置淘汰策略
redis-cli CONFIG SET maxmemory-policy allkeys-lru

# 永久配置
echo "maxmemory 4gb" >> /etc/redis/redis.conf
echo "maxmemory-policy allkeys-lru" >> /etc/redis/redis.conf
```

**预防措施**:
- ✅ 设置内存告警（>80%）
- ✅ 定期清理过期键
- ✅ 监控键空间大小

### 案例2: Kafka分区不平衡导致消费延迟

**时间**: 2024-12-10 09:00  
**持续时间**: 2小时  
**影响**: 部分symbol数据延迟>5分钟

**根因**: Kafka分区分配不均，单个Consumer负载过高

**解决方案**:
```bash
# 重新平衡分区
kafka-reassign-partitions --bootstrap-server localhost:9092 \
  --reassignment-json-file reassignment.json \
  --execute

# 增加Consumer实例
kubectl scale deployment data-consumer --replicas=5
```

**预防措施**:
- ✅ 监控Consumer Lag
- ✅ 定期检查分区分布
- ✅ 合理设置分区数（2倍Consumer数）

---

**维护团队**: Operations Team  
**紧急联系**: ops@hermesflow.com  
**最后更新**: 2024-12-20

