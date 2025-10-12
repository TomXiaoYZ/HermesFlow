# Docker部署指南

**版本**: v2.0.0  
**最后更新**: 2024-12-20

---

## 目录

1. [Dockerfile最佳实践](#1-dockerfile最佳实践)
2. [多阶段构建](#2-多阶段构建)
3. [Docker Compose](#3-docker-compose)
4. [镜像管理](#4-镜像管理)

---

## 1. Dockerfile最佳实践

### 1.1 通用原则

- 使用官方基础镜像
- 最小化层数
- 使用 `.dockerignore` 排除不必要文件
- 不在镜像中存储敏感信息
- 使用多阶段构建减小镜像大小

### 1.2 `.dockerignore`示例

```
# Git
.git
.gitignore

# IDE
.vscode
.idea
*.swp

# 构建产物
target/
dist/
build/
node_modules/

# 测试和文档
tests/
docs/
*.md

# Rust
target/
Cargo.lock  # 如果是库项目
```

---

## 2. 多阶段构建

### 2.1 Rust服务 Dockerfile ⭐

**数据采集服务示例**

```dockerfile
# ====================
# 构建阶段
# ====================
FROM rust:1.75-slim as builder

# 安装依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 复制依赖文件（利用缓存层）
COPY Cargo.toml Cargo.lock ./

# 创建虚拟main.rs预编译依赖（加速构建）
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# 复制实际源代码
COPY src ./src
COPY proto ./proto

# 构建应用
RUN cargo build --release && \
    strip target/release/data-engine

# ====================
# 运行阶段
# ====================
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# 创建非root用户
RUN useradd -m -u 1000 appuser

WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/data-engine /usr/local/bin/

# 切换到非root用户
USER appuser

# 暴露端口
EXPOSE 18001

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:18001/health || exit 1

# 启动应用
CMD ["data-engine"]
```

**构建和运行**

```bash
# 构建镜像
docker build -t hermesflow/data-engine:latest -f modules/data-engine/Dockerfile .

# 运行容器
docker run -d \
    --name data-engine \
    -p 18001:18001 \
    -e RUST_LOG=info \
    -e DATABASE_URL=postgres://user:pass@postgres:5432/hermesflow \
    hermesflow/data-engine:latest

# 查看日志
docker logs -f data-engine
```

### 2.2 Java服务 Dockerfile

**交易执行服务示例**

```dockerfile
# ====================
# 构建阶段
# ====================
FROM maven:3.9-eclipse-temurin-21 as builder

WORKDIR /app

# 复制pom.xml（利用缓存）
COPY pom.xml ./
RUN mvn dependency:go-offline -B

# 复制源代码并构建
COPY src ./src
RUN mvn clean package -DskipTests && \
    java -Djarmode=layertools -jar target/*.jar extract

# ====================
# 运行阶段
# ====================
FROM eclipse-temurin:21-jre-jammy

RUN useradd -m -u 1000 appuser

WORKDIR /app

# 复制分层JAR（优化缓存）
COPY --from=builder /app/dependencies/ ./
COPY --from=builder /app/spring-boot-loader/ ./
COPY --from=builder /app/snapshot-dependencies/ ./
COPY --from=builder /app/application/ ./

USER appuser

EXPOSE 18030

HEALTHCHECK --interval=30s --timeout=3s \
    CMD curl -f http://localhost:18030/actuator/health || exit 1

# 使用Virtual Threads
ENV JAVA_OPTS="-XX:+UseVirtualThreads -Xmx512m -Xms256m"

ENTRYPOINT ["sh", "-c", "java $JAVA_OPTS org.springframework.boot.loader.JarLauncher"]
```

### 2.3 Python服务 Dockerfile

**策略引擎服务示例**

```dockerfile
# ====================
# 构建阶段
# ====================
FROM python:3.12-slim as builder

WORKDIR /app

# 安装Poetry
RUN pip install --no-cache-dir poetry==1.7.1

# 复制依赖文件
COPY pyproject.toml poetry.lock ./

# 导出requirements.txt
RUN poetry export -f requirements.txt --output requirements.txt --without-hashes

# ====================
# 运行阶段
# ====================
FROM python:3.12-slim

RUN useradd -m -u 1000 appuser

WORKDIR /app

# 安装依赖
COPY --from=builder /app/requirements.txt ./
RUN pip install --no-cache-dir -r requirements.txt

# 复制应用代码
COPY --chown=appuser:appuser . .

USER appuser

EXPOSE 18020

HEALTHCHECK --interval=30s --timeout=3s \
    CMD curl -f http://localhost:18020/health || exit 1

CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "18020"]
```

---

## 3. Docker Compose

### 3.1 本地开发环境

**docker-compose.local.yml**

```yaml
version: '3.9'

services:
  # PostgreSQL数据库
  postgres:
    image: postgres:16-alpine
    container_name: hermesflow-postgres
    environment:
      POSTGRES_DB: hermesflow
      POSTGRES_USER: admin
      POSTGRES_PASSWORD: admin123
    ports:
      - "5432:5432"
    volumes:
      - postgres-data:/var/lib/postgresql/data
      - ./infrastructure/postgres/init.sql:/docker-entrypoint-initdb.d/init.sql
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U admin"]
      interval: 10s
      timeout: 5s
      retries: 5

  # Redis缓存
  redis:
    image: redis:7-alpine
    container_name: hermesflow-redis
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data
    command: redis-server --appendonly yes
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 3s
      retries: 5

  # ClickHouse分析数据库
  clickhouse:
    image: clickhouse/clickhouse-server:23.12
    container_name: hermesflow-clickhouse
    ports:
      - "8123:8123"  # HTTP
      - "9000:9000"  # Native
    volumes:
      - clickhouse-data:/var/lib/clickhouse
      - ./infrastructure/clickhouse/init.sql:/docker-entrypoint-initdb.d/init.sql
    environment:
      CLICKHOUSE_DB: hermesflow
      CLICKHOUSE_USER: admin
      CLICKHOUSE_PASSWORD: admin123
    healthcheck:
      test: ["CMD", "clickhouse-client", "--query", "SELECT 1"]
      interval: 30s
      timeout: 10s
      retries: 3

  # Kafka消息队列
  kafka:
    image: bitnami/kafka:3.6
    container_name: hermesflow-kafka
    ports:
      - "9092:9092"
    environment:
      KAFKA_CFG_NODE_ID: 1
      KAFKA_CFG_PROCESS_ROLES: controller,broker
      KAFKA_CFG_CONTROLLER_QUORUM_VOTERS: 1@kafka:9093
      KAFKA_CFG_LISTENERS: PLAINTEXT://:9092,CONTROLLER://:9093
      KAFKA_CFG_ADVERTISED_LISTENERS: PLAINTEXT://localhost:9092
      KAFKA_CFG_CONTROLLER_LISTENER_NAMES: CONTROLLER
    volumes:
      - kafka-data:/bitnami/kafka
    healthcheck:
      test: ["CMD-SHELL", "kafka-topics.sh --bootstrap-server localhost:9092 --list"]
      interval: 30s
      timeout: 10s
      retries: 5

  # Prometheus监控
  prometheus:
    image: prom/prometheus:latest
    container_name: hermesflow-prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./infrastructure/prometheus/prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'

  # Grafana可视化
  grafana:
    image: grafana/grafana:latest
    container_name: hermesflow-grafana
    ports:
      - "3001:3000"
    environment:
      GF_SECURITY_ADMIN_PASSWORD: admin123
    volumes:
      - grafana-data:/var/lib/grafana
      - ./infrastructure/grafana/dashboards:/etc/grafana/provisioning/dashboards
    depends_on:
      - prometheus

volumes:
  postgres-data:
  redis-data:
  clickhouse-data:
  kafka-data:
  prometheus-data:
  grafana-data:

networks:
  default:
    name: hermesflow-network
```

**启动和管理**

```bash
# 启动所有服务
docker-compose -f docker-compose.local.yml up -d

# 启动特定服务
docker-compose -f docker-compose.local.yml up -d postgres redis

# 查看状态
docker-compose -f docker-compose.local.yml ps

# 查看日志
docker-compose -f docker-compose.local.yml logs -f

# 停止服务
docker-compose -f docker-compose.local.yml down

# 停止并删除数据卷
docker-compose -f docker-compose.local.yml down -v
```

---

## 4. 镜像管理

### 4.1 构建镜像

```bash
# Rust服务
docker build -t hermesflow/data-engine:v2.0.0 \
    -f modules/data-engine/Dockerfile .

# Java服务
docker build -t hermesflow/trading-engine:v2.0.0 \
    -f modules/trading-engine/Dockerfile .

# Python服务
docker build -t hermesflow/strategy-engine:v2.0.0 \
    -f modules/strategy-engine/Dockerfile .
```

### 4.2 推送到Registry

```bash
# 登录Azure Container Registry
az acr login --name hermesflowacr

# 打标签
docker tag hermesflow/data-engine:v2.0.0 \
    hermesflowacr.azurecr.io/data-engine:v2.0.0

# 推送
docker push hermesflowacr.azurecr.io/data-engine:v2.0.0
```

### 4.3 镜像优化技巧

**减小Rust镜像大小** ⭐

```bash
# 使用strip去除调试符号
RUN strip target/release/data-engine

# 使用musl构建静态链接二进制（可选）
FROM rust:1.75-alpine as builder
RUN apk add --no-cache musl-dev
RUN cargo build --release --target x86_64-unknown-linux-musl

# 使用scratch或distroless镜像
FROM gcr.io/distroless/static-debian12
COPY --from=builder /app/target/release/data-engine /
CMD ["/data-engine"]
```

**镜像大小对比**

| 基础镜像 | 最终大小 | 适用场景 |
|---------|---------|---------|
| debian:bookworm | ~80MB | 需要调试工具 |
| alpine | ~20MB | 生产环境 |
| distroless | ~10MB | 极简生产环境 |
| scratch | ~8MB | 静态链接二进制 |

---

## 5. 常见问题

### 5.1 Rust服务编译慢

```dockerfile
# 使用sccache加速编译
FROM rust:1.75 as builder
RUN cargo install sccache
ENV RUSTC_WRAPPER=/usr/local/cargo/bin/sccache
```

### 5.2 Java服务启动慢

```dockerfile
# 使用GraalVM Native Image（可选）
FROM ghcr.io/graalvm/native-image:ol8-java21 as builder
RUN native-image --no-fallback -jar app.jar
```

---

**文档维护者**: DevOps Team  
**最后更新**: 2024-12-20

