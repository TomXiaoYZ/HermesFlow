# HermesFlow 文档导航中心

> **版本**: v2.1.0 | **更新日期**: 2025-01-13

欢迎使用 HermesFlow 量化交易平台文档中心！本导航旨在帮助团队成员快速找到所需文档，提升开发效率和协作质量。

---

## 🚀 快速开始

- **新手上路**: [快速开始指南](./quickstart.md) - 5分钟了解项目基础
- **常见问题**: [FAQ文档](./faq.md) - 解决80%的常见疑问
- **故障排查**: [故障排查手册](./operations/troubleshooting.md) - 应急处理指南

---

## 🎯 按角色浏览

### 产品经理 (Product Manager)
- 📋 [产品需求文档 (PRD)](./prd/prd-hermesflow.md)
- 📊 [项目进度跟踪](./progress.md)
- 🔍 [市场分析报告](./analysis/market-analysis-and-gap-assessment.md)
- 📅 [开发路线图](./prd/prd-hermesflow.md#53-开发路线图)

### Scrum Master
- 📖 [Scrum Master 完整指南](./scrum/sm-guide.md)
- ✅ [Sprint Planning 清单](./scrum/sprint-planning-checklist.md)
- 🔄 [Retrospective 模板](./scrum/retrospective-template.md)
- 📈 [项目进度仪表盘](./progress.md)

### 开发者 (Developer)

#### 通用资源
- 🏁 [开发者快速开始](./developmen./quickstart.md)
- 📝 [编码规范](./development/coding-standards.md)
- 🔍 [代码审查清单](./development/code-review-checklist.md)
- 📚 [开发指南](./development/dev-guide.md)
- 🔗 [快速参考手册](./development/quick-reference.md)

#### 分语言指南
- 🦀 [Rust 开发者指南](./development/rust-developer-guide.md) - 数据引擎
- ☕ [Java 开发者指南](./development/java-developer-guide.md) - 交易/用户/风控服务
- 🐍 [Python 开发者指南](./development/python-developer-guide.md) - 策略引擎

### QA 工程师
- 🧪 [QA 工程师完整指南](./testing/qa-engineer-guide.md)
- 📋 [测试策略](./testing/test-strategy.md)
- ✅ [验收测试清单](./testing/acceptance-checklist.md)
- 🔐 [高风险访问测试](./testing/high-risk-access-testing.md)
- 🗄️ [测试数据管理](./testing/test-data-management.md)
- 🔄 [CI/CD 测试集成](./testing/ci-cd-integration.md)

### DevOps 工程师
- 🚀 [DevOps 工程师指南](./operations/devops-guide.md)
- 🐳 [Docker 部署指南](./deployment/docker-guide.md)
- ☸️ [GitOps 最佳实践](./deployment/gitops-best-practices.md)
- 📊 [监控方案](./operations/monitoring.md)
- 🔧 [故障排查手册](./operations/troubleshooting.md)
- 🔄 [CI/CD 流程图](./architecture/diagrams/cicd-flow.md)

### UX 设计师
- 🎨 [设计系统](./design/design-system.md)
- 📱 [页面设计规范](./design/page-designs.md)
- 💻 [UI 实现指南 (Lovable/V0)](./design/lovable-v0-prompt.md)

---

## 📅 按开发周期浏览

### Sprint Planning（Sprint 计划阶段）

**必读文档**:
1. [产品需求文档 (PRD)](./prd/prd-hermesflow.md) - 了解功能需求
2. [模块详细需求](./modules/module-index.md) - 查看具体模块任务
3. [系统架构文档](./architecture/system-architecture.md) - 理解技术架构
4. [项目进度](./progress.md) - 当前状态和待办事项
5. [架构决策记录 (ADR)](./architecture/decisions/) - 关键技术选型

**工作流程**:
```
Sprint Planning清单 → 审查PRD → 选择Story → 估算 → 分配任务
```

📋 [使用 Sprint Planning 清单](./scrum/sprint-planning-checklist.md)

---

### Development（开发阶段）

**必读文档**:
1. [开发者快速开始](./developmen./quickstart.md) - 环境搭建
2. [开发指南](./development/dev-guide.md) - 开发流程
3. [编码规范](./development/coding-standards.md) - 代码质量标准
4. [API 设计文档](./api/api-design.md) - API 规范
5. [数据库设计文档](./database/database-design.md) - 数据库 Schema

**分模块导航**:
- 🔍 [按模块查找文档](./modules/module-index.md)

**工作流程**:
```
创建分支 → 编写代码 → 自测 → Code Review → 合并
```

🔍 [使用代码审查清单](./development/code-review-checklist.md)

---

### Testing（测试阶段）

**必读文档**:
1. [测试策略](./testing/test-strategy.md) - 整体测试方法
2. [早期测试策略](./testing/early-test-strategy.md) - Alpha 阶段测试
3. [高风险访问测试](./testing/high-risk-access-testing.md) - 安全测试
4. [测试数据管理](./testing/test-data-management.md) - 测试数据准备
5. [CI/CD 测试集成](./testing/ci-cd-integration.md) - 自动化测试

**工作流程**:
```
单元测试 → 集成测试 → 安全测试 → 性能测试 → 验收测试
```

✅ [使用验收测试清单](./testing/acceptance-checklist.md)

---

### Deployment（部署阶段）

**必读文档**:
1. [Docker 部署指南](./deployment/docker-guide.md) - 容器化部署
2. [GitOps 最佳实践](./deployment/gitops-best-practices.md) - 声明式部署
3. [CI/CD 架构](./architecture/system-architecture.md#第11章-cicd架构) - 完整 CI/CD 流程
4. [CI/CD 流程图](./architecture/diagrams/cicd-flow.md) - 可视化流程

**工作流程**:
```
GitHub Actions构建 → 推送ACR → ArgoCD同步 → Kubernetes部署
```

🚀 [查看完整部署流程](./architecture/diagrams/cicd-flow.md)

---

### Operations（运维阶段）

**必读文档**:
1. [故障排查手册](./operations/troubleshooting.md) - 应急响应
2. [监控方案](./operations/monitoring.md) - Prometheus + Grafana
3. [系统架构](./architecture/system-architecture.md) - 理解系统结构

**工作流程**:
```
监控告警 → 日志分析 → 故障定位 → 应急处理 → 根因分析 → 改进
```

🔧 [快速故障排查](./operations/troubleshooting.md#快速诊断流程)

---

### Retrospective（回顾阶段）

**必读文档**:
1. [Retrospective 模板](./scrum/retrospective-template.md)
2. [项目进度](./progress.md) - 对比计划与实际
3. [技术债务](./progress.md#技术债务) - 识别改进点

**工作流程**:
```
收集反馈 → 分析问题 → 识别改进点 → 制定行动计划 → 更新最佳实践
```

🔄 [使用 Retrospective 模板](./scrum/retrospective-template.md)

---

## 📚 按文档类型浏览

### 产品需求文档 (PRD)

| 文档 | 描述 | 优先级 |
|------|------|--------|
| [PRD 主文档](./prd/prd-hermesflow.md) | 产品整体需求和路线图（80页） | 🔴 P0 |
| [数据模块 (Rust)](./prd/modules/01-data-module.md) | 数据采集、处理、存储 | 🔴 P0 |
| [策略模块 (Python)](./prd/modules/02-strategy-module.md) | 策略编写、回测、优化 | 🔴 P0 |
| [执行模块 (Java)](./prd/modules/03-execution-module.md) | 订单执行、风控 | 🔴 P0 |
| [风控模块 (Java)](./prd/modules/04-risk-module.md) | 实时风险监控 | 🟡 P1 |
| [账户模块 (Java)](./prd/modules/05-account-module.md) | 用户管理、多租户 | 🟡 P1 |
| [安全模块 (Java)](./prd/modules/06-security-module.md) | 身份认证、授权 | 🔴 P0 |
| [报表模块](./prd/modules/07-report-module.md) | 数据可视化、报表 | 🟢 P2 |
| [UX 模块](./prd/modules/08-ux-module.md) | 用户体验、交互设计 | 🟡 P1 |

---

### 架构设计文档

| 文档 | 描述 | 字数 |
|------|------|------|
| [系统架构文档](./architecture/system-architecture.md) | 完整系统架构（11章，8000+行） | ~40,000 |
| [CI/CD 流程图](./architecture/diagrams/cicd-flow.md) | 可视化 CI/CD 流程 | ~2,000 |

**架构决策记录 (ADR)**:
- [ADR-001: 混合技术栈](./architecture/decisions/ADR-001-hybrid-tech-stack.md)
- [ADR-002: 多租户架构](./architecture/decisions/ADR-002-multi-tenancy-architecture.md)
- [ADR-003: 消息通信](./architecture/decisions/ADR-003-message-communication.md)
- [ADR-004: 数据存储](./architecture/decisions/ADR-004-data-storage-strategy.md)
- [ADR-005: CI/CD 流程](./architecture/decisions/ADR-005-cicd-gitops.md)
- [ADR-006: Rust 数据层](./architecture/decisions/ADR-006-rust-data-layer.md)
- [ADR-007: Alpha 因子库](./architecture/decisions/ADR-007-alpha-factor-library.md)
- [ADR-008: 模拟交易 API](./architecture/decisions/ADR-008-paper-trading-api.md)

---

### 技术规范文档

| 类别 | 文档 | 描述 |
|------|------|------|
| **API** | [API 设计文档](./api/api-design.md) | OpenAPI + gRPC 规范 |
| **数据库** | [数据库设计文档](./database/database-design.md) | PostgreSQL + ClickHouse + Redis |
| **开发** | [编码规范](./development/coding-standards.md) | Rust/Java/Python 代码标准 |
| **开发** | [开发指南](./development/dev-guide.md) | 完整开发流程 |
| **快速参考** | [快速参考手册](./development/quick-reference.md) | 常用命令和API |

---

### 测试文档

| 文档 | 描述 | 覆盖率目标 |
|------|------|-----------|
| [测试策略](./testing/test-strategy.md) | 整体测试金字塔和策略 | 总览 |
| [早期测试策略](./testing/early-test-strategy.md) | Alpha 阶段测试计划 | - |
| [高风险访问测试](./testing/high-risk-access-testing.md) | 安全关键路径测试 | 100% |
| [测试数据管理](./testing/test-data-management.md) | Fixtures + Mock + Generators | - |
| [CI/CD 测试集成](./testing/ci-cd-integration.md) | GitHub Actions 自动化 | - |

**覆盖率要求**:
- Rust: ≥ 85%
- Java: ≥ 80%
- Python: ≥ 75%

---

### 运维文档

| 文档 | 描述 | 行数 |
|------|------|------|
| [故障排查手册](./operations/troubleshooting.md) | 完整故障处理指南 | ~1,400 |
| [监控方案](./operations/monitoring.md) | Prometheus + Grafana | ~800 |

---

### 部署文档

| 文档 | 描述 |
|------|------|
| [Docker 部署指南](./deployment/docker-guide.md) | 容器化和多阶段构建 |
| [GitOps 最佳实践](./deployment/gitops-best-practices.md) | ArgoCD + Helm 声明式部署 |

---

### 设计文档

| 文档 | 描述 |
|------|------|
| [设计系统](./design/design-system.md) | 颜色、字体、组件规范 |
| [页面设计](./design/page-designs.md) | 核心页面设计 |
| [UI 实现指南](./design/lovable-v0-prompt.md) | Lovable/V0 Prompt |

---

### 分析文档

| 文档 | 描述 | 页数 |
|------|------|------|
| [市场分析报告](./analysis/market-analysis-and-gap-assessment.md) | 竞品分析和差距评估 | ~40页 |

---

## 🔍 快速查找

### 按使用场景查找

#### 场景1: 我是新加入的开发者
```
1. 阅读 [快速开始指南](./quickstart.md)
2. 根据技术栈选择:
   - Rust → [Rust 开发者指南](./development/rust-developer-guide.md)
   - Java → [Java 开发者指南](./development/java-developer-guide.md)
   - Python → [Python 开发者指南](./development/python-developer-guide.md)
3. 搭建环境 → [开发指南](./development/dev-guide.md)
4. 查看 [编码规范](./development/coding-standards.md)
5. 开始第一个任务
```

#### 场景2: 我要开发新功能
```
1. 查看 [PRD 文档](./prd/prd-hermesflow.md) 了解需求
2. 查看 [模块索引](./modules/module-index.md) 找到相关模块
3. 查看 [系统架构](./architecture/system-architecture.md) 理解设计
4. 查看 [API 设计](./api/api-design.md) 和 [数据库设计](./database/database-design.md)
5. 参考 [编码规范](./development/coding-standards.md) 开始编码
6. 使用 [代码审查清单](./development/code-review-checklist.md) 自查
7. 编写测试，参考 [测试策略](./testing/test-strategy.md)
```

#### 场景3: 我遇到了Bug
```
1. 查看 [故障排查手册](./operations/troubleshooting.md)
2. 检查 [监控方案](./operations/monitoring.md) 中的日志和指标
3. 查阅 [FAQ](./faq.md) 是否有类似问题
4. 查看相关模块的 [架构文档](./architecture/system-architecture.md)
```

#### 场景4: 我要准备部署
```
1. 查看 [CI/CD 流程图](./architecture/diagrams/cicd-flow.md)
2. 阅读 [Docker 部署指南](./deployment/docker-guide.md)
3. 了解 [GitOps 最佳实践](./deployment/gitops-best-practices.md)
4. 配置监控，参考 [监控方案](./operations/monitoring.md)
5. 准备应急预案，参考 [故障排查手册](./operations/troubleshooting.md)
```

#### 场景5: 我要编写测试
```
1. 了解 [测试策略](./testing/test-strategy.md)
2. 查看 [验收测试清单](./testing/acceptance-checklist.md)
3. 准备测试数据，参考 [测试数据管理](./testing/test-data-management.md)
4. 对于安全关键功能，参考 [高风险访问测试](./testing/high-risk-access-testing.md)
5. 集成到 CI/CD，参考 [CI/CD 测试集成](./testing/ci-cd-integration.md)
```

---

### 常见问题快速入口

- ❓ **如何搭建开发环境?** → [开发指南](./development/dev-guide.md#环境搭建)
- ❓ **编码规范是什么?** → [编码规范](./development/coding-standards.md)
- ❓ **如何提交代码?** → [开发指南](./development/dev-guide.md#开发工作流)
- ❓ **API 文档在哪?** → [API 设计文档](./api/api-design.md)
- ❓ **数据库 Schema?** → [数据库设计文档](./database/database-design.md)
- ❓ **测试覆盖率要求?** → [测试策略](./testing/test-strategy.md#覆盖率要求)
- ❓ **如何部署?** → [Docker 部署指南](./deployment/docker-guide.md)
- ❓ **系统出问题了?** → [故障排查手册](./operations/troubleshooting.md)
- ❓ **更多问题?** → [FAQ 文档](./faq.md)

---

## 📊 文档统计

| 类型 | 数量 | 总行数 | 平均行数 |
|------|------|--------|---------|
| PRD 文档 | 9 | ~15,000 | ~1,667 |
| 架构文档 | 10 | ~10,000 | ~1,000 |
| 技术规范 | 5 | ~4,000 | ~800 |
| 测试文档 | 6 | ~5,500 | ~917 |
| 运维文档 | 2 | ~2,200 | ~1,100 |
| 部署文档 | 2 | ~2,000 | ~1,000 |
| 设计文档 | 3 | ~2,000 | ~667 |
| 其他文档 | 8 | ~1,550 | ~194 |
| **总计** | **45** | **~42,250** | **~939** |

---

## 🔄 文档更新流程

### 文档维护责任

| 文档类型 | 主要维护者 | 审查者 | 更新频率 |
|---------|-----------|--------|---------|
| PRD 文档 | Product Manager | Product Team | 每月/按需 |
| 架构文档 | Architect | Architecture Team | 每Sprint/按需 |
| 开发指南 | Tech Lead | Dev Team | 每Sprint |
| 测试文档 | QA Lead | QA Team | 每Sprint |
| 运维文档 | DevOps Lead | DevOps Team | 每月 |
| Scrum 文档 | Scrum Master | Scrum Master | 每Sprint |

### 文档审查流程

#### 每 Sprint 开始
- [ ] SM 审查 Sprint Planning 文档
- [ ] 团队审查相关模块文档
- [ ] 更新 `progress.md`

#### 每 Sprint 结束
- [ ] 更新完成的文档
- [ ] 标记过时内容
- [ ] 补充新的最佳实践

#### 每月
- [ ] 运行文档主检查清单
- [ ] 修复不一致问题
- [ ] 更新 FAQ

---

## 🔗 相关资源

### 项目资源
- **主 README**: [/README.md](../README.md)
- **项目进度**: [progress.md](./progress.md)
- **架构文档**: [architecture.md](./architecture/system-architecture.md)

### 代码仓库
- **主仓库**: HermesFlow (本地)
- **GitOps 仓库**: HermesFlow-GitOps (本地)

### 外部工具
- **监控**: Prometheus + Grafana
- **日志**: ELK Stack
- **CI/CD**: GitHub Actions + ArgoCD
- **容器**: Docker + Kubernetes (AKS)

---

## 📝 文档反馈

如果您发现文档有误或需要改进，请：

1. **技术文档问题**: 联系对应的文档维护者
2. **流程问题**: 联系 Scrum Master
3. **紧急问题**: 直接更新并通知团队

---

## 📌 文档使用技巧

### 提示 1: 使用 Ctrl+F / Cmd+F 搜索
在本页使用浏览器搜索功能，快速定位关键词。

### 提示 2: 收藏常用文档
根据您的角色，收藏以下文档：
- **开发者**: 开发指南 + 编码规范 + API 文档
- **QA**: 测试策略 + 验收清单
- **DevOps**: 部署指南 + 故障排查手册

### 提示 3: 使用文档流程图
不确定从哪开始？查看 [文档流程图](./document-flow.md)，它会根据您的场景推荐阅读顺序。

### 提示 4: 定期查看 progress.md
[项目进度](./progress.md) 每周更新，了解最新进展和待办事项。

---

**最后更新**: 2025-01-13  
**维护者**: @pm.mdc  
**版本**: v2.1.0
