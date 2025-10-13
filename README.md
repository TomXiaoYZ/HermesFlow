# HermesFlow 量化交易平台

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

### 环境配置

- **dev环境**: 开发测试环境，对应`dev`分支
- **main环境**: 生产环境，对应`main`分支
- **镜像仓库**: 不同环境使用独立的Azure Container Registry

### 部署流程

1. **开发者提交代码**:
   ```bash
   git commit -m "[module:strategy-engine] feat: 新增策略功能"
   git push origin dev
   ```

2. **自动构建部署**:
   - GitHub Actions解析commit中的模块标签
   - 调用统一构建脚本 `./scripts/build-module.sh`
   - 构建Docker镜像并推送到对应环境的ACR
   - 触发HermesFlow-GitOps仓库更新Helm Charts

3. **支持的模块标签**:
   - `[module:strategy-engine]` - 策略引擎
   - `[module:risk-engine]` - 风控引擎  
   - `[module:data-engine]` - 数据引擎
   - `[module:user-management]` - 用户管理
   - `[module:api-gateway]` - API网关
   - `[module:frontend]` - 前端界面

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

### GitHub Secrets 配置

在GitHub仓库Settings → Secrets中配置以下密钥：

```yaml
# Dev环境
DEV_AZURE_REGISTRY: hermesflow-dev-acr.azurecr.io
DEV_AZURE_CLIENT_ID: <dev环境客户端ID>
DEV_AZURE_CLIENT_SECRET: <dev环境客户端密钥>

# Main环境  
MAIN_AZURE_REGISTRY: hermesflow-main-acr.azurecr.io
MAIN_AZURE_CLIENT_ID: <main环境客户端ID>
MAIN_AZURE_CLIENT_SECRET: <main环境客户端密钥>

# GitOps
GITOPS_TOKEN: <HermesFlow-GitOps仓库访问令牌>
```

## 🔗 相关项目

- **HermesFlow-GitOps**: Helm Charts和ArgoCD配置仓库
- **配套监控**: Prometheus + Grafana + Jaeger

## 📚 文档

### 核心文档
- [系统架构文档](docs/architecture/system-architecture.md) - 系统整体架构设计
- [开发进度跟踪](docs/progress.md) - 开发状态和里程碑
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
- [Docker部署指南](docs/deployment/docker-guide.md) - 容器化部署
- [GitOps最佳实践](docs/deployment/gitops-best-practices.md) - CI/CD与GitOps工作流
- [测试策略](docs/testing/test-strategy.md) - 测试方法和覆盖率要求
- [监控方案](docs/operations/monitoring.md) - Prometheus+Grafana监控

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

**最后更新**: 2024年12月20日 | **版本**: v2.1.0

