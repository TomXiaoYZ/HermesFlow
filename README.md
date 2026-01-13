# HermesFlow 量化交易平台

[![CI](https://github.com/hermesflow/HermesFlow/actions/workflows/ci.yml/badge.svg)](https://github.com/hermesflow/HermesFlow/actions/workflows/ci.yml)

HermesFlow 是一个面向个人使用的高性能量化交易平台，采用现代化的 **Rust + Python** 双栈微服务架构。

- **Rust (核心层)**: 负责高性能行情接入 (`data-engine`) 和 API 网关 (`gateway`).
- **Python (业务层)**: 负责策略回测 (`strategy-engine`) 和风控计算 (`risk-engine`).
- **Infrastructure**: 基于 Docker Compose 和 GitHub Actions 的全自动化运维.

## 🏗️ 项目架构

```
HermesFlow/
├── services/                   # 🎯 微服务
│   ├── data-engine/            # [Rust] 行情引擎 (Repository Pattern)
│   ├── gateway/                # [Rust] API 网关 (Axum)
│   ├── strategy-engine/        # [Python] 策略引擎 (FastAPI + Pandas)
│   ├── risk-engine/            # [Python] 风控引擎 (FastAPI)
│   └── twitter-scraper/        # [Python] 舆情采集
├── infrastructure/             # 🔧 基础设施
│   ├── database/               # 数据库 DDL (Postgres/ClickHouse)
│   ├── python/                 # 共享 Python 库 (hermes-common)
│   └── terraform/              # 云资源定义 (IAC)
├── docs/                       # 📚 文档
├── .github/workflows/          # 🚀 CI/CD 流水线
├── docker-compose.yml          # 本地编排
└── Makefile                    # 统一构建工具
```

### 技术栈决策

| 服务 | 技术栈 | 职责 | 端口 |
|------|--------|------|------|
| **Gateway** | **Rust (Axum)** | 统一接入、鉴权、路由 | 3000 -> 8080 |
| **Data Engine** | **Rust (Tokio, SQLx)** | 多路行情聚合、高速入库 | 8080 |
| **Strategy Engine** | **Python 3.11** | 策略逻辑、量化分析 | 8040 |
| **Risk Engine** | **Python 3.11** | 实时风控、持仓检查 | 8030 |
| **Storage** | **TimescaleDB + Redis** | 时序数据与热缓存 | 5432 / 6379 |

## 🚀 快速开始

本项目使用统一的 `Makefile` 管理开发流程，无需手动运行脚本。

### 1. 环境准备
确保本地安装了 `Docker`, `Rust`, `Python 3.11`.

### 2. 初始化项目
一键安装依赖（创建虚拟环境、编译 Rust 依赖、安装 Python 共享库）：
```bash
make setup
```

### 3. 配置环境
复制示例配置：
```bash
cp .env.example .env
```
根据需要修改 `.env` 中的凭据（默认为本地开发配置，无需修改）。

### 4. 启动服务
```bash
make up
```
访问 http://localhost:3000/health 检查网关状态。

### 5. 开发命令
- `make test`: 运行所有服务的单元测试
- `make lint`: 运行代码检查 (Clippy/Ruff/Mypy)
- `make clean`: 清理构建产物

## 📏 开发规范 (Phase 7 Standards)

### 1. 虚拟环境
所有 Python 操作必须在 `.venv` 中进行（`make setup` 自动处理）。禁止直接使用系统 Python。

### 2. 配置管理
- **禁止**在代码或 TOML 文件中硬编码密码。
- 所有敏感配置通过 `.env` 注入，映射规则见 `docker-compose.yml` (例如 `DATA_ENGINE__POSTGRES__PASSWORD`)。

### 3. 数据库变更
数据库 Schema 变更必须在 `infrastructure/database/{type}/migrations` 中添加 SQL 文件。

### 4. Python 共享库
通用逻辑（日志、配置加载）应放入 `infrastructure/python/hermes_common`，禁止在业务服务中复制粘贴。

## 📚 文档索引
- [架构设计](docs/architecture/system-architecture.md)
- [开发指南](docs/development/dev-guide.md)
- [数据库设计](docs/database/database-design.md)
