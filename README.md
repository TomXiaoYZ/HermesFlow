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
| strategy-engine | Python 3.12 + FastAPI | 8000 | 策略开发与执行 |
| risk-engine | Java 21 + Spring Boot | 8080 | 风险控制与监控 |
| data-engine | Python 3.12 + asyncio | 8001 | 数据采集与处理 |
| user-management | Java 21 + Spring Boot | 8010 | 用户认证与管理 |
| api-gateway | Java 21 + Spring Gateway | 8000 | 统一API网关 |
| frontend | React + TypeScript | 80 | 用户界面 |

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

- [系统架构文档](docs/architecture.md)
- [开发进度跟踪](docs/progress.md)
- [模块开发指南](docs/modules/)

## 🤝 贡献指南

1. Fork项目
2. 创建特性分支: `git checkout -b feature/new-feature`
3. 提交更改: `git commit -m '[module:xxx] feat: 新功能'`
4. 推送分支: `git push origin feature/new-feature`
5. 创建Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。

---

**最后更新**: 2024年12月 | **版本**: v2.0.0

