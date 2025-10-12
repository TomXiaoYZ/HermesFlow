# HermesFlow 快速参考指南

**版本**: v2.0.0  
**最后更新**: 2024-12-20

本文档提供HermesFlow开发和运维过程中的常用信息快速查询。

---

## 📍 服务端口映射

| 服务 | 技术栈 | 端口 | 用途 |
|------|--------|------|------|
| API Gateway | Spring Cloud Gateway | 18000 | 统一API入口 |
| 数据采集服务 | **Rust** | 18001 | 实时数据采集 |
| 数据处理服务 | **Rust** | 18002 | 历史数据处理 |
| 用户管理服务 | Java (Spring Boot) | 18010 | 用户认证授权 |
| 策略引擎服务 | Python (FastAPI) | 18020 | 策略开发执行 |
| 回测引擎 | Python | 18021 | 策略回测 |
| 交易执行服务 | Java (Spring Boot) | 18030 | 订单执行 |
| 风控服务 | Java (Spring Boot) | 18040 | 风险监控 |
| PostgreSQL | - | 15432 | 主数据库 |
| ClickHouse | - | 18123 | 时序数据库 |
| Redis | - | 16379 | 缓存 |
| Kafka | - | 19092 | 消息队列 |

---

## 🔧 Rust开发快速参考

### 安装Rust工具链

```bash
# 安装rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 更新Rust
rustup update

# 查看版本
rustc --version
cargo --version

# 安装开发工具
cargo install cargo-watch    # 文件监听自动编译
cargo install cargo-tarpaulin # 测试覆盖率
cargo install cargo-llvm-cov  # LLVM覆盖率
cargo install flamegraph      # 性能分析
```

### 常用Cargo命令

```bash
# 构建
cargo build                  # Debug构建
cargo build --release        # Release构建（优化）

# 运行
cargo run                    # 运行main
cargo run --bin data-engine  # 运行特定binary

# 测试
cargo test                   # 运行所有测试
cargo test --test test_name  # 运行特定测试
cargo test -- --nocapture    # 显示println输出

# 检查
cargo check                  # 快速类型检查
cargo clippy                 # Linter检查
cargo fmt                    # 代码格式化

# 基准测试
cargo bench                  # 运行基准测试

# 文档
cargo doc --open             # 生成并打开文档

# 依赖管理
cargo update                 # 更新依赖
cargo tree                   # 查看依赖树
```

### Rust项目结构

```
data-engine/
├── Cargo.toml           # 项目配置
├── Cargo.lock           # 依赖锁定
├── src/
│   ├── main.rs          # 入口
│   ├── lib.rs           # 库根
│   ├── api/             # API路由
│   ├── services/        # 业务逻辑
│   ├── models/          # 数据模型
│   ├── connectors/      # 外部连接器
│   └── utils/           # 工具函数
├── tests/               # 集成测试
│   └── integration_test.rs
├── benches/             # 基准测试
│   └── benchmarks.rs
├── examples/            # 示例代码
└── .cargo/
    └── config.toml      # Cargo配置
```

### 常用依赖

```toml
[dependencies]
# 异步运行时
tokio = { version = "1.35", features = ["full"] }

# Web框架
actix-web = "4.4"
# 或
axum = "0.7"

# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# HTTP客户端
reqwest = { version = "0.11", features = ["json"] }

# WebSocket
tungstenite = "0.21"

# 数据库
clickhouse-rs = "1.0"
redis = { version = "0.24", features = ["tokio-comp"] }

# Kafka
rdkafka = { version = "0.35", features = ["cmake-build"] }

# 日志
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# 错误处理
anyhow = "1.0"
thiserror = "1.0"

# 配置
config = "0.14"

# 时间
chrono = "0.4"

# 数值计算
rust_decimal = "1.33"

# 性能监控
prometheus = "0.13"
```

### Rust调试

```bash
# 使用lldb (macOS)
rust-lldb target/debug/data-engine

# 使用gdb (Linux)
rust-gdb target/debug/data-engine

# VS Code调试配置 (.vscode/launch.json)
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug data-engine",
      "cargo": {
        "args": ["build", "--bin=data-engine", "--package=data-engine"]
      },
      "args": [],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

### 环境变量

```bash
# Rust日志级别
export RUST_LOG=debug,data_engine=trace

# 性能分析
export RUSTFLAGS="-C force-frame-pointers=yes"

# 数据库连接
export DATABASE_URL=postgresql://user:pass@localhost:15432/hermesflow
export REDIS_URL=redis://localhost:16379
export CLICKHOUSE_URL=http://localhost:18123
export KAFKA_BROKERS=localhost:19092
```

---

## ☕ Java开发快速参考

### 常用Maven命令

```bash
# 编译
mvn clean compile

# 打包
mvn clean package
mvn clean package -DskipTests  # 跳过测试

# 运行
mvn spring-boot:run

# 测试
mvn test
mvn test -Dtest=TestClassName

# 安装到本地仓库
mvn install

# 查看依赖树
mvn dependency:tree
```

### Spring Boot配置

```yaml
# application.yml
spring:
  profiles:
    active: local

server:
  port: 18010

spring:
  datasource:
    url: jdbc:postgresql://localhost:15432/hermesflow
    username: hermesflow
    password: hermesflow123
  
  redis:
    host: localhost
    port: 16379
  
  kafka:
    bootstrap-servers: localhost:19092
```

### Java环境变量

```bash
export JAVA_HOME=/path/to/jdk-21
export SPRING_PROFILES_ACTIVE=local
export JVM_OPTS="-Xms512m -Xmx2g -XX:+UseZGC"
```

---

## 🐍 Python开发快速参考

### Poetry命令

```bash
# 安装Poetry
curl -sSL https://install.python-poetry.org | python3 -

# 初始化项目
poetry init

# 安装依赖
poetry install

# 添加依赖
poetry add fastapi uvicorn

# 激活虚拟环境
poetry shell

# 运行
poetry run python main.py
poetry run uvicorn main:app --reload

# 测试
poetry run pytest
```

### Python环境变量

```bash
export PYTHONPATH=/app
export REDIS_URL=redis://localhost:6379
export KAFKA_BOOTSTRAP_SERVERS=localhost:19092
```

---

## 🐳 Docker命令速查

### 常用命令

```bash
# 构建镜像
docker build -t hermesflow/data-engine:latest -f scripts/data-engine/Dockerfile .

# 运行容器
docker run -d --name data-engine -p 18001:18001 hermesflow/data-engine:latest

# 查看日志
docker logs -f data-engine

# 进入容器
docker exec -it data-engine /bin/bash

# 停止/启动容器
docker stop data-engine
docker start data-engine

# 清理
docker system prune -a  # 清理所有未使用资源
```

### docker-compose

```bash
# 启动所有服务
docker-compose up -d

# 启动特定服务
docker-compose up -d postgres redis clickhouse

# 查看日志
docker-compose logs -f data-engine

# 停止所有服务
docker-compose down

# 重启服务
docker-compose restart data-engine

# 查看状态
docker-compose ps
```

---

## 🗄️ 数据库连接字符串

### PostgreSQL

```bash
# 连接字符串
postgresql://hermesflow:hermesflow123@localhost:15432/hermesflow

# psql命令行
psql -h localhost -p 15432 -U hermesflow -d hermesflow

# 设置租户上下文
SELECT set_config('app.current_tenant', '00000000-0000-0000-0000-000000000001', false);
```

### ClickHouse

```bash
# 连接字符串
http://localhost:18123

# clickhouse-client
clickhouse-client --host localhost --port 9000

# 查询示例
SELECT * FROM market_data.ticks 
WHERE exchange = 'binance' 
  AND symbol = 'BTCUSDT' 
  AND timestamp >= now() - INTERVAL 1 HOUR 
LIMIT 100;
```

### Redis

```bash
# 连接字符串
redis://localhost:16379

# redis-cli
redis-cli -h localhost -p 16379

# 常用命令
KEYS market:*                 # 查找所有market相关key
GET market:binance:BTCUSDT:latest
HGETALL market:binance:BTCUSDT:latest
ZRANGE orderbook:binance:BTCUSDT:bids 0 10
```

---

## 🔌 API快速测试

### 健康检查

```bash
# 数据采集服务 (Rust)
curl http://localhost:18001/health

# 用户管理服务 (Java)
curl http://localhost:18010/actuator/health

# 策略引擎 (Python)
curl http://localhost:18020/health
```

### 认证

```bash
# 登录获取Token
curl -X POST http://localhost:18000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"password"}'

# 使用Token访问API
curl http://localhost:18000/api/v1/market/realtime/binance/BTCUSDT \
  -H "Authorization: Bearer <token>"
```

### 数据查询

```bash
# 获取实时行情
curl "http://localhost:18001/api/v1/market/realtime/binance/BTCUSDT"

# 获取历史数据
curl "http://localhost:18001/api/v1/market/history/binance/BTCUSDT?start_time=1703001600000000&end_time=1703088000000000&interval=1m"

# 订单簿
curl "http://localhost:18001/api/v1/market/orderbook/binance/BTCUSDT?depth=20"
```

---

## 🧪 测试命令

### 单元测试

```bash
# Rust
cargo test

# Java
mvn test

# Python
poetry run pytest
```

### 集成测试

```bash
# Rust
cargo test --test '*'

# Java
mvn verify

# Python
poetry run pytest tests/integration/
```

### 性能测试

```bash
# Rust基准测试
cargo bench

# 压力测试（使用wrk）
wrk -t12 -c400 -d30s http://localhost:18001/api/v1/market/realtime/binance/BTCUSDT

# JMeter
jmeter -n -t test-plan.jmx -l results.jtl
```

---

## 📊 监控指标查询

### Prometheus查询

```promql
# 数据采集延迟
histogram_quantile(0.99, rate(data_message_latency_seconds_bucket[5m]))

# 消息吞吐量
rate(data_messages_received_total[1m])

# Redis写入延迟
histogram_quantile(0.99, rate(redis_write_latency_seconds_bucket[5m]))

# 错误率
rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m])
```

### 日志查询

```bash
# 查看Rust服务日志
docker logs -f data-engine | grep ERROR

# 查看Java服务日志
docker logs -f user-management | grep -i exception

# 使用jq解析JSON日志
docker logs data-engine 2>&1 | jq 'select(.level=="ERROR")'
```

---

## 🔧 常见问题快速解决

### Rust编译慢

```bash
# 使用sccache缓存编译
cargo install sccache
export RUSTC_WRAPPER=sccache

# 使用mold链接器（Linux）
sudo apt install mold
export RUSTFLAGS="-C link-arg=-fuse-ld=mold"
```

### WebSocket连接失败

```bash
# 检查防火墙
sudo ufw status
sudo ufw allow 18001/tcp

# 检查端口占用
lsof -i :18001
netstat -an | grep 18001
```

### 数据库连接池耗尽

```sql
-- PostgreSQL查看连接数
SELECT count(*) FROM pg_stat_activity;

-- 终止空闲连接
SELECT pg_terminate_backend(pid) 
FROM pg_stat_activity 
WHERE state = 'idle' AND state_change < now() - interval '10 minutes';
```

### Redis内存不足

```bash
# 查看内存使用
redis-cli INFO memory

# 清理过期key
redis-cli --scan --pattern "market:*" | xargs redis-cli DEL

# 调整maxmemory策略
redis-cli CONFIG SET maxmemory-policy allkeys-lru
```

---

## 📖 参考链接

- [Rust官方文档](https://doc.rust-lang.org/)
- [Tokio文档](https://tokio.rs/)
- [Actix-web文档](https://actix.rs/)
- [Spring Boot文档](https://spring.io/projects/spring-boot)
- [FastAPI文档](https://fastapi.tiangolo.com/)

---

**维护者**: DevOps Team  
**最后更新**: 2024-12-20

