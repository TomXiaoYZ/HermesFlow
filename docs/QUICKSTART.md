# HermesFlow 快速开始指南

> **阅读时间**: 5 分钟 | **上手时间**: < 1 小时

欢迎加入 HermesFlow 团队！本指南将帮助您快速了解项目并开始工作。

---

## 🎯 5分钟了解项目

### 项目简介

**HermesFlow** 是一个**多租户量化交易平台**，专注于**成本优化**和**高性能交易**。

**核心特点**:
- 🦀 **Rust** 数据层：超高性能数据处理（100万行/秒）
- ☕ **Java** 业务层：交易执行、用户管理、风控
- 🐍 **Python** 策略层：量化策略开发和回测
- ☸️ **Kubernetes** 部署：Azure AKS + ArgoCD GitOps
- 🔐 **多租户隔离**：PostgreSQL RLS + Redis 前缀隔离

### 架构一览

```
┌─────────────────────────────────────────────────────────────┐
│                       前端 (React + TypeScript)              │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                    API Gateway (Spring Cloud)                │
└─────────────────────────────────────────────────────────────┘
              │                 │                 │
    ┌─────────┴────────┐  ┌────┴──────┐  ┌──────┴───────┐
    │ 数据引擎 (Rust)   │  │ 策略引擎   │  │ 交易引擎      │
    │ - 数据采集        │  │ (Python)  │  │ (Java)       │
    │ - 实时处理        │  │ - 策略开发 │  │ - 订单执行    │
    │ - 高频写入        │  │ - 回测     │  │ - 风控       │
    └──────────────────┘  └───────────┘  └──────────────┘
              │                 │                 │
    ┌─────────┴─────────────────┴─────────────────┴─────────┐
    │          PostgreSQL + ClickHouse + Redis + Kafka       │
    └───────────────────────────────────────────────────────┘
```

### 技术栈速览

| 模块 | 语言 | 框架 | 数据库 |
|------|------|------|--------|
| 数据引擎 | Rust 1.75 | Tokio + Actix-web | ClickHouse + Redis |
| 策略引擎 | Python 3.12 | FastAPI + NumPy | PostgreSQL |
| 交易引擎 | Java 21 | Spring Boot 3.2 | PostgreSQL + Redis |
| 用户管理 | Java 21 | Spring Security | PostgreSQL (RLS) |
| 风控引擎 | Java 21 | Spring Boot | PostgreSQL + Redis |
| 前端 | TypeScript 5.3 | React 18 + Vite 5 | - |

---

## 🚀 60分钟上手计划

### 第一步：克隆代码（5分钟）

```bash
# 1. 克隆主仓库
git clone <your-repo-url>/HermesFlow.git
cd HermesFlow

# 2. 克隆 GitOps 仓库（可选，DevOps 需要）
git clone <your-repo-url>/HermesFlow-GitOps.git ../HermesFlow-GitOps
```

### 第二步：选择您的开发路径（5分钟）

根据您的角色和技术栈，选择对应的指南：

#### 路径 A: Rust 开发者（数据引擎）
👉 [Rust 开发者完整指南](./development/RUST-DEVELOPER-GUIDE.md)

**您将负责**:
- 数据采集（Binance, OKX, Polygon 等）
- 实时数据处理
- 高频写入 ClickHouse/Redis

**核心技术**:
- Tokio（异步运行时）
- Actix-web（Web 框架）
- Rayon（并行计算）
- Arrow（列式数据）

---

#### 路径 B: Java 开发者（交易/用户/风控）
👉 [Java 开发者完整指南](./development/JAVA-DEVELOPER-GUIDE.md)

**您将负责**:
- 订单执行和管理
- 用户认证和授权（JWT + RBAC）
- 实时风控监控

**核心技术**:
- Spring Boot 3.2 + Spring Security 6
- Virtual Threads（JDK 21）
- Spring Data JPA
- Spring Cloud Gateway

---

#### 路径 C: Python 开发者（策略引擎）
👉 [Python 开发者完整指南](./development/PYTHON-DEVELOPER-GUIDE.md)

**您将负责**:
- 量化策略开发
- 回测引擎
- Alpha 因子库

**核心技术**:
- FastAPI
- NumPy + Pandas
- asyncio + aiohttp
- pytest

---

#### 路径 D: QA 工程师
👉 [QA 工程师完整指南](./testing/QA-ENGINEER-GUIDE.md)

**您将负责**:
- 编写和执行测试用例
- 安全测试（RLS、RBAC、SQL 注入）
- 性能测试（k6）

**核心技术**:
- pytest + HTTPX
- k6（性能测试）
- TestContainers
- GitHub Actions

---

#### 路径 E: DevOps 工程师
👉 [DevOps 工程师完整指南](./operations/DEVOPS-GUIDE.md)

**您将负责**:
- CI/CD 流水线（GitHub Actions）
- Kubernetes 部署（AKS + ArgoCD）
- 监控和告警（Prometheus + Grafana）

**核心技术**:
- Docker + Kubernetes
- Helm + ArgoCD
- Prometheus + Grafana
- Azure AKS + ACR

---

### 第三步：环境搭建（30-40分钟）

根据您选择的路径，按照对应的开发者指南搭建环境。

**通用要求**:
- Git
- Docker Desktop
- IDE（VS Code / IntelliJ IDEA / PyCharm）

**Rust 开发者额外需要**:
```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
```

**Java 开发者额外需要**:
```bash
# 安装 JDK 21
# macOS: brew install openjdk@21
# Linux: 使用包管理器安装
# Windows: 下载 Oracle JDK 21
```

**Python 开发者额外需要**:
```bash
# 安装 Python 3.12
# macOS: brew install python@3.12
# Linux: pyenv install 3.12
# Windows: 下载 Python 3.12 安装包

# 安装 Poetry
curl -sSL https://install.python-poetry.org | python3 -
```

---

### 第四步：启动本地服务（10分钟）

#### 方式 1：使用 Docker Compose（推荐新手）

```bash
# 启动所有依赖服务（PostgreSQL, Redis, ClickHouse, Kafka）
cd HermesFlow
docker-compose up -d

# 检查服务状态
docker-compose ps
```

#### 方式 2：本地开发模式

```bash
# Rust 数据引擎
cd modules/data-engine
cargo run

# Python 策略引擎
cd modules/strategy-engine
poetry install
poetry run python main.py

# Java 交易引擎
cd modules/trading-engine
./mvnw spring-boot:run
```

---

### 第五步：验证环境（5分钟）

#### 测试 1：健康检查

```bash
# API Gateway
curl http://localhost:8080/actuator/health

# 数据引擎
curl http://localhost:8081/health

# 策略引擎
curl http://localhost:8082/health
```

#### 测试 2：运行测试

```bash
# Rust 测试
cd modules/data-engine
cargo test

# Java 测试
cd modules/trading-engine
./mvnw test

# Python 测试
cd modules/strategy-engine
poetry run pytest
```

✅ **如果所有测试通过，恭喜！您的环境已就绪。**

---

## 📚 必读文档

### 新手必读（按顺序）

1. **[编码规范](./development/coding-standards.md)** (10分钟)
   - 了解代码风格和质量标准
   
2. **[开发指南](./development/dev-guide.md)** (20分钟)
   - 了解开发流程和工作流
   
3. **[系统架构文档](./architecture/system-architecture.md)** (30分钟)
   - 理解整体架构设计
   
4. **[PRD 文档](./prd/PRD-HermesFlow.md)** (60分钟)
   - 了解产品需求和功能

### 按需阅读

- **API 开发**: [API 设计文档](./api/api-design.md)
- **数据库操作**: [数据库设计文档](./database/database-design.md)
- **测试编写**: [测试策略](./testing/test-strategy.md)
- **问题排查**: [故障排查手册](./operations/troubleshooting.md)

---

## 🔧 常用命令速查

### Git 工作流

```bash
# 1. 创建功能分支
git checkout -b feature/your-feature-name

# 2. 提交代码
git add .
git commit -m "feat: your feature description"

# 3. 推送分支
git push origin feature/your-feature-name

# 4. 创建 Pull Request
# 在 GitHub 上操作
```

### Docker 命令

```bash
# 启动所有服务
docker-compose up -d

# 查看日志
docker-compose logs -f [service-name]

# 停止所有服务
docker-compose down

# 重建服务
docker-compose up -d --build
```

### 测试命令

```bash
# Rust
cargo test
cargo test --release  # 发布模式测试

# Java
./mvnw test
./mvnw test -Dtest=YourTestClass  # 运行单个测试

# Python
poetry run pytest
poetry run pytest tests/test_specific.py  # 运行单个测试文件
```

---

## 💡 开发最佳实践

### 1. 代码提交规范

使用 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

```
feat: 新功能
fix: Bug 修复
docs: 文档更新
test: 测试相关
refactor: 代码重构
perf: 性能优化
chore: 构建/工具链更新
```

**示例**:
```bash
git commit -m "feat(data-engine): 添加 Binance WebSocket 连接器"
git commit -m "fix(trading): 修复订单状态更新延迟问题"
git commit -m "docs(api): 更新 REST API 文档"
```

### 2. 分支策略

```
main          ← 生产环境，受保护
  ├── develop ← 开发环境
  │   ├── feature/xxx ← 功能开发
  │   ├── fix/xxx     ← Bug 修复
  │   └── test/xxx    ← 测试分支
```

### 3. Code Review 检查项

提交 PR 前，使用 [代码审查清单](./development/CODE-REVIEW-CHECKLIST.md) 自查：

- [ ] 代码符合编码规范
- [ ] 添加了单元测试
- [ ] 测试覆盖率达标（Rust≥85%, Java≥80%, Python≥75%）
- [ ] 更新了相关文档
- [ ] 通过了 CI/CD 检查

### 4. 测试金字塔

```
       E2E 测试 (10%)
      /            \
     集成测试 (30%)
    /                \
   单元测试 (60%)
```

**原则**: 多写单元测试，适量集成测试，少量 E2E 测试。

---

## 🆘 遇到问题？

### 常见问题快速解决

| 问题 | 解决方案 |
|------|---------|
| 🔴 **环境搭建失败** | 查看 [开发指南 - 环境搭建](./development/dev-guide.md#环境搭建) |
| 🔴 **测试失败** | 查看 [FAQ](./FAQ.md#测试相关) |
| 🔴 **Docker 启动失败** | 查看 [Docker 部署指南](./deployment/docker-guide.md#故障排查) |
| 🔴 **代码不符合规范** | 查看 [编码规范](./development/coding-standards.md) |
| 🔴 **不知道从哪开始** | 查看 [文档流程图](./DOCUMENT-FLOW.md) |

### 获取帮助

1. **查阅文档**: [FAQ](./FAQ.md) - 80% 的问题都有答案
2. **故障排查**: [故障排查手册](./operations/troubleshooting.md)
3. **联系团队**: 
   - Slack: #hermesflow-dev
   - Email: dev@hermesflow.example

---

## 🎓 学习路径

### 第 1 周：熟悉项目

- [ ] 完成环境搭建
- [ ] 阅读编码规范和开发指南
- [ ] 运行所有测试
- [ ] 修复一个 "good first issue"

### 第 2 周：深入模块

- [ ] 阅读您负责模块的 PRD
- [ ] 理解模块架构设计
- [ ] 阅读相关 API 和数据库文档
- [ ] 完成一个小功能

### 第 3 周：团队协作

- [ ] 参与 Code Review
- [ ] 编写测试用例
- [ ] 优化现有代码
- [ ] 更新文档

### 第 4 周：独立工作

- [ ] 独立完成一个中等难度任务
- [ ] 提升测试覆盖率
- [ ] 参与技术讨论
- [ ] 分享经验

---

## 📖 下一步

根据您的角色，继续阅读：

### 开发者
- 🦀 [Rust 开发者指南](./development/RUST-DEVELOPER-GUIDE.md)
- ☕ [Java 开发者指南](./development/JAVA-DEVELOPER-GUIDE.md)
- 🐍 [Python 开发者指南](./development/PYTHON-DEVELOPER-GUIDE.md)

### QA / DevOps
- 🧪 [QA 工程师指南](./testing/QA-ENGINEER-GUIDE.md)
- 🚀 [DevOps 工程师指南](./operations/DEVOPS-GUIDE.md)

### Scrum Master / PM
- 📋 [Scrum Master 指南](./scrum/SM-GUIDE.md)
- 📊 [项目进度](./progress.md)

---

## 🔗 重要链接

| 链接 | 描述 |
|------|------|
| [文档导航](./README.md) | 完整文档索引 |
| [系统架构](./architecture/system-architecture.md) | 架构设计 |
| [API 文档](./api/api-design.md) | API 规范 |
| [编码规范](./development/coding-standards.md) | 代码标准 |
| [测试策略](./testing/test-strategy.md) | 测试指南 |
| [故障排查](./operations/troubleshooting.md) | 问题解决 |

---

**欢迎加入 HermesFlow！祝您开发愉快！** 🎉

---

**最后更新**: 2025-01-13  
**维护者**: @pm.mdc  
**反馈**: 如有问题或建议，请联系团队

