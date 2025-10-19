# HermesFlow 量化交易平台

[![CI - Rust](https://github.com/hermesflow/HermesFlow/workflows/CI%20-%20Rust%20Services/badge.svg)](https://github.com/hermesflow/HermesFlow/actions/workflows/ci-rust.yml)
[![CI - Java](https://github.com/hermesflow/HermesFlow/workflows/CI%20-%20Java%20Services/badge.svg)](https://github.com/hermesflow/HermesFlow/actions/workflows/ci-java.yml)
[![CI - Python](https://github.com/hermesflow/HermesFlow/workflows/CI%20-%20Python%20Services/badge.svg)](https://github.com/hermesflow/HermesFlow/actions/workflows/ci-python.yml)
[![CI - Frontend](https://github.com/hermesflow/HermesFlow/workflows/CI%20-%20Frontend/badge.svg)](https://github.com/hermesflow/HermesFlow/actions/workflows/ci-frontend.yml)
[![Terraform](https://github.com/hermesflow/HermesFlow/workflows/Terraform%20-%20Azure%20Infrastructure/badge.svg)](https://github.com/hermesflow/HermesFlow/actions/workflows/terraform.yml)
[![Security Scan](https://github.com/hermesflow/HermesFlow/workflows/Security%20Scan/badge.svg)](https://github.com/hermesflow/HermesFlow/actions/workflows/security-scan.yml)

## 📋 项目简介

HermesFlow 是一个面向个人使用的多租户量化交易平台，采用微服务架构，支持多交易所数据采集、策略开发、风险控制和自动化执行。

## 🏗️ 项目架构

### 模块化设计

```
HermesFlow/
├── modules/                     # 🎯 业务模块
│   ├── strategy-engine/         # 策略引擎 (Python)
│   ├── risk-engine/            # 风控引擎 (Java)
│   ├── data-engine/            # 数据引擎 (Python)
│   ├── user-management/        # 用户管理 (Java)
│   ├── api-gateway/           # API网关 (Java)
│   └── frontend/              # 前端界面 (React)
├── scripts/                   # 🐳 Docker构建脚本
│   ├── strategy-engine/Dockerfile
│   ├── risk-engine/Dockerfile
│   ├── data-engine/Dockerfile
│   ├── user-management/Dockerfile
│   ├── api-gateway/Dockerfile
│   ├── frontend/Dockerfile
│   └── build-module.sh        # 统一构建脚本
├── .github/workflows/         # 🚀 CI/CD流水线
│   └── module-cicd.yml        # 模块化构建部署
└── infrastructure/            # 🔧 基础设施
    ├── dev/                   # 开发环境
    └── main/                  # 生产环境
```

### 技术栈

| 模块 | 技术栈 | 端口 | 说明 |
|------|--------|------|------|
| **data-engine** ⭐ | **Rust 1.75 + Tokio 1.35 + Actix-web 4.4** | **18001-18002** | **高性能数据采集与处理** |
| strategy-engine | Python 3.12 + FastAPI 0.104 | 18020-18021 | 策略开发、回测与执行 |
| trading-engine | Java 21 + Spring Boot 3.2 + WebFlux | 18030 | 订单管理与交易执行 |
| risk-engine | Java 21 + Spring Boot 3.2 | 18040 | 风险控制与监控 |
| user-management | Java 21 + Spring Boot 3.2 | 18010 | 用户认证与管理 |
| gateway | Java 21 + Spring Cloud Gateway 4.1 | 18000 | 统一API网关 |
| frontend | React 18.2 + TypeScript 5.3 + Vite 5.0 | 3000 | 用户界面 |

#### 核心优势

- **⚡ 超低延迟**: Rust数据引擎实现<1ms P99延迟
- **🚀 高吞吐量**: 支持>100k消息/秒的数据处理能力
- **🔒 内存安全**: Rust零成本抽象保证并发安全
- **📊 多数据源**: 支持加密货币、美股、期权、舆情等多种数据源

## 🚀 CI/CD 流程

### 自动化流水线

HermesFlow 使用 GitHub Actions 实现完全自动化的 CI/CD 流水线：

- **多语言 CI**: Rust, Java, Python, React 独立并行构建
- **智能检测**: 自动检测代码变更，只构建受影响的模块
- **安全扫描**: Trivy 镜像扫描, 依赖审计, Secrets 检测
- **基础设施**: Terraform 自动化 Azure 资源管理
- **GitOps**: 自动更新 Helm Charts，ArgoCD 持续部署

### 环境配置

- **dev环境**: 开发测试环境 (Azure East US)
- **main环境**: 生产环境 (待配置)
- **镜像仓库**: Azure Container Registry
- **K8s集群**: Azure Kubernetes Service (AKS)

### 部署流程

1. **开发者推送代码** → GitHub Actions 触发
2. **路径检测** → 识别变更的模块
3. **并行构建** → Rust/Java/Python/React 独立构建
4. **质量检查** → 
   - 代码格式检查 (fmt, checkstyle, pylint)
   - 单元测试 (覆盖率: Rust ≥85%, Java ≥80%, Python ≥75%)
   - 安全扫描 (Trivy)
5. **镜像构建** → Docker 多阶段构建
6. **推送 ACR** → 标签: `${sha}`, `latest`
7. **GitOps 更新** → 自动更新 values.yaml
8. **ArgoCD 同步** → 自动部署到 K8s

### 支持的工作流

| Workflow | 触发条件 | 说明 |
|----------|---------|------|
| `ci-rust.yml` | Rust 代码变更 | 构建 data-engine, gateway |
| `ci-java.yml` | Java 代码变更 | 构建 user-management, api-gateway, trading-engine |
| `ci-python.yml` | Python 代码变更 | 构建 strategy-engine, backtest-engine, risk-engine |
| `ci-frontend.yml` | React 代码变更 | 构建 frontend |
| `terraform.yml` | Terraform 变更 | 基础设施部署 |
| `security-scan.yml` | 定时/手动 | 每日安全扫描 |
| `update-gitops.yml` | CI 成功后 | 自动更新 GitOps |

## 🛠️ 开发指南

### 本地开发

1. **克隆代码**:
   ```bash
   git clone https://github.com/your-org/HermesFlow.git
   cd HermesFlow
   ```

2. **开发特定模块**:
   ```bash
   cd modules/strategy-engine
   # 按照各模块的README进行开发
   ```

3. **本地构建测试**:
   ```bash
   # 设置环境变量
   export MODULE=strategy-engine
   export AZURE_REGISTRY=your-registry.azurecr.io
   export AZURE_CLIENT_ID=your-client-id
   export AZURE_CLIENT_SECRET=your-client-secret
   
   # 执行构建
   ./scripts/build-module.sh
   ```

### 代码提交规范

- **功能开发**: `[module:xxx] feat: 功能描述`
- **问题修复**: `[module:xxx] fix: 问题描述`
- **性能优化**: `[module:xxx] perf: 优化描述`
- **文档更新**: `docs: 文档更新` (无需模块标签)

## 📦 部署配置

### 基础设施即代码 (IaC)

使用 Terraform 管理 Azure 基础设施：

```bash
cd infrastructure/terraform/environments/dev
terraform init
terraform plan
terraform apply
```

**创建的资源**:
- Azure Kubernetes Service (AKS) - 3 nodes
- Azure Container Registry (ACR)
- PostgreSQL Flexible Server
- Azure Key Vault
- Log Analytics Workspace
- Virtual Network + Subnets

详见 [Terraform README](infrastructure/terraform/README.md) 和 [Setup Guide](infrastructure/terraform/environments/dev/SETUP.md)

### GitHub Secrets 配置

完整的 Secrets 配置指南: **[GitHub Secrets Setup](docs/deployment/github-secrets-setup.md)**

必需的 Secrets:
```
AZURE_CLIENT_ID           # Service Principal ID
AZURE_CLIENT_SECRET       # Service Principal Secret
AZURE_SUBSCRIPTION_ID     # Azure 订阅 ID
AZURE_TENANT_ID          # Azure Tenant ID
ACR_LOGIN_SERVER         # ACR 地址
ACR_USERNAME             # ACR 用户名
ACR_PASSWORD             # ACR 密码
GITOPS_PAT               # GitOps 仓库访问令牌
POSTGRES_ADMIN_PASSWORD  # 数据库密码
SLACK_WEBHOOK_URL        # Slack 通知
ALERT_EMAIL              # 告警邮箱
```

快速配置:
```bash
# 使用 GitHub CLI
gh secret set AZURE_CLIENT_ID --body "your-value"

# 或使用自动化脚本
./scripts/setup-github-secrets.sh
```

## 🔗 相关项目

- **HermesFlow-GitOps**: Helm Charts和ArgoCD配置仓库
- **配套监控**: Prometheus + Grafana + Jaeger

## 📚 文档

### 核心文档
- [系统架构文档](docs/architecture/system-architecture.md) - 系统整体架构设计
- [开发进度跟踪](docs/progress.md) - 开发状态和里程碑
- **[Sprint Stories](docs/stories/README.md)** ⭐ - 用户故事和Sprint计划
- [快速参考指南](docs/QUICK-REFERENCE.md) - 常用命令和配置

### 架构设计文档 ⭐
- **[系统架构设计](docs/architecture/system-architecture.md)** - 完整系统架构文档（4400+行）
  - C4架构模型（Context/Container/Component）
  - 前端架构设计（React 18 + TypeScript + TailwindCSS）
  - 后端架构设计（Rust数据层 + Java业务层 + Python策略层）
  - 数据架构设计（PostgreSQL RLS + ClickHouse + Redis + Kafka）
  - 部署架构设计（Docker Compose + Kubernetes + Azure）
  - 安全架构设计（JWT + RBAC + 多租户隔离 + 数据加密）
  - 性能优化策略（零拷贝 + Numba JIT + 多级缓存）
  - 监控与运维方案（Prometheus + Grafana + ELK）

#### 架构决策记录 (ADR)
- [ADR-001: 采用混合技术栈架构](docs/architecture/decisions/ADR-001-hybrid-tech-stack.md) - Rust + Java + Python
- [ADR-002: 选择Tokio作为Rust异步运行时](docs/architecture/decisions/ADR-002-tokio-runtime.md)
- [ADR-003: PostgreSQL RLS实现多租户隔离](docs/architecture/decisions/ADR-003-postgresql-rls.md)
- [ADR-004: ClickHouse作为分析数据库](docs/architecture/decisions/ADR-004-clickhouse-analytics.md)
- [ADR-005: Kafka作为事件流平台](docs/architecture/decisions/ADR-005-kafka-event-streaming.md)
- [ADR-006: React + TypeScript前端技术栈](docs/architecture/decisions/ADR-006-react-frontend.md)
- [ADR-007: Alpha因子库使用Numba加速](docs/architecture/decisions/ADR-007-numba-acceleration.md)
- [ADR-008: 模拟交易与实盘API兼容设计](docs/architecture/decisions/ADR-008-paper-trading-api.md)

### PRD与需求文档
- [产品需求文档 (PRD)](docs/prd/PRD-HermesFlow.md) - 完整产品需求规格说明
- [数据模块需求 (Rust)](docs/prd/modules/01-data-module.md) ⭐
- [策略模块需求 (Python)](docs/prd/modules/02-strategy-module.md)
- [执行模块需求 (Java)](docs/prd/modules/03-execution-module.md)
- [风控模块需求](docs/prd/modules/04-risk-module.md)
- [其他模块需求](docs/prd/modules/) - 账户、安全、报表、UX

### 技术文档
- [API设计文档](docs/api/api-design.md) - RESTful和gRPC API规范
- [数据库设计文档](docs/database/database-design.md) - PostgreSQL/ClickHouse/Redis设计
- [开发指南](docs/development/dev-guide.md) - 开发环境搭建与工作流
- [编码规范](docs/development/coding-standards.md) - Rust/Java/Python编码标准
- [测试策略](docs/testing/test-strategy.md) - 测试方法和覆盖率要求
- [监控方案](docs/operations/monitoring.md) - Prometheus+Grafana监控

### DevOps 与部署
- **[Terraform Infrastructure](infrastructure/terraform/README.md)** ⭐ - 基础设施即代码
- **[GitHub Secrets Setup](docs/deployment/github-secrets-setup.md)** 🔐 - CI/CD 密钥配置
- [Docker部署指南](docs/deployment/docker-guide.md) - 容器化部署
- [GitOps最佳实践](docs/deployment/gitops-best-practices.md) - CI/CD与GitOps工作流

### CI/CD与部署
- [CI/CD架构](docs/architecture/system-architecture.md#11-持续集成与持续部署cicd架构) - 完整的CI/CD流程设计
- [CI/CD流程图](docs/architecture/diagrams/cicd-flow.md) - 可视化流程与时序图
- [GitOps最佳实践](docs/deployment/gitops-best-practices.md) - 故障排查与运维指南

完整文档导航请查看 [docs/README.md](docs/README.md)

## 🤝 贡献指南

1. Fork项目
2. 创建特性分支: `git checkout -b feature/new-feature`
3. 提交更改: `git commit -m '[module:xxx] feat: 新功能'`
4. 推送分支: `git push origin feature/new-feature`
5. 创建Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。

---

**最后更新**: 2025年1月13日 | **版本**: v2.1.0 | **当前Sprint**: [Sprint 1 - DevOps Foundation](docs/stories/sprint-01/sprint-01-summary.md)

