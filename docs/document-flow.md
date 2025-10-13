# 文档使用流程图

> **按场景快速找到所需文档** | **可视化导航**

---

## 🎯 使用方法

根据您当前的**场景**，找到对应的流程图，按照箭头指引阅读相关文档。

---

## 📋 场景目录

1. [场景 1: 新开发者入职](#场景-1-新开发者入职)
2. [场景 2: 开发新功能](#场景-2-开发新功能)
3. [场景 3: Bug 修复](#场景-3-bug-修复)
4. [场景 4: Code Review](#场景-4-code-review)
5. [场景 5: 准备部署](#场景-5-准备部署)
6. [场景 6: 生产问题排查](#场景-6-生产问题排查)
7. [场景 7: Sprint Planning](#场景-7-sprint-planning)
8. [场景 8: 编写测试](#场景-8-编写测试)
9. [场景 9: 性能优化](#场景-9-性能优化)
10. [场景 10: 架构决策](#场景-10-架构决策)

---

## 场景 1: 新开发者入职

```mermaid
graph TD
    A[👋 新开发者入职] --> B[📖 阅读快速开始指南<br/>quickstart.md - 5分钟]
    B --> C{选择技术栈}
    
    C -->|Rust| D1[🦀 Rust 开发者指南<br/>rust-developer-guide.md]
    C -->|Java| D2[☕ Java 开发者指南<br/>java-developer-guide.md]
    C -->|Python| D3[🐍 Python 开发者指南<br/>python-developer-guide.md]
    
    D1 --> E[🛠️ 搭建本地环境<br/>30-40分钟]
    D2 --> E
    D3 --> E
    
    E --> F[✅ 运行所有测试]
    F --> G[📝 阅读编码规范<br/>coding-standards.md]
    G --> H[🏗️ 理解系统架构<br/>system-architecture.md]
    H --> I[🎯 开始第一个任务<br/>good first issue]
```

**文档路径**:
1. [快速开始指南](./quickstart.md)
2. 开发者指南:
   - [Rust 开发者指南](./development/rust-developer-guide.md)
   - [Java 开发者指南](./development/java-developer-guide.md)
   - [Python 开发者指南](./development/python-developer-guide.md)
3. [编码规范](./development/coding-standards.md)
4. [系统架构](./architecture/system-architecture.md)

---

## 场景 2: 开发新功能

```mermaid
graph TD
    A[🎯 开发新功能] --> B[📋 阅读 PRD<br/>prd-hermesflow.md]
    B --> C[🔍 查找模块文档<br/>module-index.md]
    C --> D[🏗️ 理解架构设计<br/>system-architecture.md]
    D --> E[📡 查看 API 设计<br/>api-design.md]
    E --> F[🗄️ 查看数据库设计<br/>database-design.md]
    F --> G[💻 开始编码]
    G --> H[🧪 编写测试<br/>test-strategy.md]
    H --> I[🔍 自查 Code Review 清单<br/>code-review-checklist.md]
    I --> J[📤 提交 Pull Request]
    J --> K[👥 Code Review]
    K --> L[✅ 合并]
```

**文档路径**:
1. [PRD 主文档](./prd/prd-hermesflow.md)
2. [模块文档索引](./modules/module-index.md)
3. [系统架构](./architecture/system-architecture.md)
4. [API 设计](./api/api-design.md)
5. [数据库设计](./database/database-design.md)
6. [测试策略](./testing/test-strategy.md)
7. [代码审查清单](./development/code-review-checklist.md)

---

## 场景 3: Bug 修复

```mermaid
graph TD
    A[🐛 发现 Bug] --> B[🔍 查看故障排查手册<br/>troubleshooting.md]
    B --> C{能否定位问题?}
    
    C -->|是| D[💻 修复代码]
    C -->|否| E[📊 查看监控日志<br/>monitoring.md]
    
    E --> F[🔍 分析日志]
    F --> D
    
    D --> G[🧪 添加回归测试]
    G --> H[✅ 验证修复]
    H --> I[📝 更新变更日志]
    I --> J[📤 提交 PR]
```

**文档路径**:
1. [故障排查手册](./operations/troubleshooting.md)
2. [监控方案](./operations/monitoring.md)
3. [测试策略](./testing/test-strategy.md)

---

## 场景 4: Code Review

```mermaid
graph TD
    A[👥 Code Review] --> B{我的角色}
    
    B -->|提交者| C1[📋 自查清单<br/>code-review-checklist.md]
    B -->|审查者| C2[📋 审查清单<br/>code-review-checklist.md]
    
    C1 --> D1[运行 Linter 和测试]
    D1 --> E1[确保 CI/CD 通过]
    E1 --> F1[提交 PR]
    
    C2 --> D2[快速浏览代码]
    D2 --> E2[深入审查代码]
    E2 --> F2[功能验证]
    F2 --> G2{是否批准?}
    
    G2 -->|是| H[✅ 批准合并]
    G2 -->|否| I[💬 提供反馈]
    
    F1 --> C2
    I --> C1
```

**文档路径**:
1. [代码审查清单](./development/code-review-checklist.md)
2. [编码规范](./development/coding-standards.md)
3. [测试策略](./testing/test-strategy.md)

---

## 场景 5: 准备部署

```mermaid
graph TD
    A[🚀 准备部署] --> B[📖 查看 CI/CD 架构<br/>system-architecture.md#ch11]
    B --> C[🐳 查看 Docker 部署指南<br/>docker-guide.md]
    C --> D[☸️ 查看 GitOps 最佳实践<br/>gitops-best-practices.md]
    D --> E[📊 配置监控<br/>monitoring.md]
    E --> F[🔧 准备应急预案<br/>troubleshooting.md]
    F --> G[✅ 验收测试<br/>acceptance-checklist.md]
    G --> H[📤 提交部署请求]
    H --> I[🔄 GitOps 自动部署]
    I --> J[✅ 验证部署成功]
```

**文档路径**:
1. [CI/CD 架构](./architecture/system-architecture.md#第11章-cicd架构)
2. [Docker 部署指南](./deployment/docker-guide.md)
3. [GitOps 最佳实践](./deployment/gitops-best-practices.md)
4. [监控方案](./operations/monitoring.md)
5. [故障排查手册](./operations/troubleshooting.md)
6. [验收测试清单](./testing/acceptance-checklist.md)

---

## 场景 6: 生产问题排查

```mermaid
graph TD
    A[🔥 生产问题] --> B[🔧 故障排查手册<br/>troubleshooting.md]
    B --> C[📊 查看监控<br/>Prometheus + Grafana]
    C --> D[📝 查看日志<br/>kubectl logs / ELK]
    D --> E{定位问题?}
    
    E -->|是| F[💻 修复或缓解]
    E -->|否| G[📖 查看 FAQ<br/>faq.md]
    
    G --> H{找到答案?}
    H -->|是| F
    H -->|否| I[👥 联系团队]
    
    F --> J[✅ 验证修复]
    J --> K[📝 记录到 troubleshooting.md]
    K --> L[🔄 根因分析]
```

**文档路径**:
1. [故障排查手册](./operations/troubleshooting.md)
2. [监控方案](./operations/monitoring.md)
3. [FAQ](./faq.md)
4. [系统架构](./architecture/system-architecture.md)

---

## 场景 7: Sprint Planning

```mermaid
graph TD
    A[📅 Sprint Planning] --> B[📋 Sprint Planning 清单<br/>sprint-planning-checklist.md]
    B --> C[📖 审查 Product Backlog<br/>progress.md]
    C --> D[🎯 制定 Sprint 目标]
    D --> E[📋 选择 Story<br/>prd-hermesflow.md]
    E --> F[🔧 任务分解]
    F --> G[⏱️ 估算<br/>Planning Poker]
    G --> H[🚨 识别风险和依赖]
    H --> I[✅ 确定 Definition of Done]
    I --> J[📤 更新任务看板]
```

**文档路径**:
1. [Sprint Planning 清单](./scrum/sprint-planning-checklist.md)
2. [Scrum Master 指南](./scrum/sm-guide.md)
3. [项目进度](./progress.md)
4. [PRD 主文档](./prd/prd-hermesflow.md)
5. [模块文档索引](./modules/module-index.md)

---

## 场景 8: 编写测试

```mermaid
graph TD
    A[🧪 编写测试] --> B[📖 查看测试策略<br/>test-strategy.md]
    B --> C{测试类型}
    
    C -->|单元测试| D1[💻 编写单元测试]
    C -->|集成测试| D2[🔗 编写集成测试]
    C -->|安全测试| D3[🔐 查看高风险访问测试<br/>high-risk-access-testing.md]
    C -->|性能测试| D4[⚡ 编写性能测试<br/>k6]
    
    D1 --> E[📊 检查覆盖率]
    D2 --> E
    D3 --> E
    D4 --> E
    
    E --> F{覆盖率达标?}
    F -->|是| G[✅ 提交测试]
    F -->|否| H[补充测试]
    H --> E
```

**文档路径**:
1. [测试策略](./testing/test-strategy.md)
2. [高风险访问测试](./testing/high-risk-access-testing.md)
3. [测试数据管理](./testing/test-data-management.md)
4. [CI/CD 测试集成](./testing/ci-cd-integration.md)
5. [验收测试清单](./testing/acceptance-checklist.md)

---

## 场景 9: 性能优化

```mermaid
graph TD
    A[⚡ 性能优化] --> B[📊 查看监控<br/>Prometheus]
    B --> C[🔍 识别瓶颈]
    C --> D{瓶颈类型}
    
    D -->|API 慢| E1[查看故障排查手册<br/>性能诊断章节]
    D -->|数据库慢| E2[查看数据库设计<br/>优化索引]
    D -->|代码性能| E3[查看开发者指南<br/>性能优化章节]
    
    E1 --> F[🛠️ 实施优化]
    E2 --> F
    E3 --> F
    
    F --> G[📊 性能测试<br/>k6]
    G --> H{性能提升?}
    H -->|是| I[✅ 提交优化]
    H -->|否| J[重新分析]
    J --> C
```

**文档路径**:
1. [故障排查手册 - 性能诊断](./operations/troubleshooting.md#性能诊断)
2. [监控方案](./operations/monitoring.md)
3. [数据库设计](./database/database-design.md)
4. 开发者指南（各语言性能优化章节）

---

## 场景 10: 架构决策

```mermaid
graph TD
    A[🏗️ 架构决策] --> B[📖 查看系统架构<br/>system-architecture.md]
    B --> C[📜 查看现有 ADR<br/>architecture/decisions/]
    C --> D[💭 评估方案]
    D --> E[📝 编写 ADR]
    E --> F[👥 团队讨论]
    F --> G{是否批准?}
    
    G -->|是| H[✅ 实施方案]
    G -->|否| I[修改方案]
    I --> D
    
    H --> J[📝 更新架构文档]
```

**文档路径**:
1. [系统架构](./architecture/system-architecture.md)
2. [ADR 文档](./architecture/decisions/)
3. [ADR 模板](./architecture/decisions/ADR-TEMPLATE.md)

---

## 🗺️ 全局文档地图

```mermaid
graph TB
    ROOT[📚 文档中心<br/>README.md]
    
    ROOT --> QUICK[⚡ 快速开始<br/>quickstart.md]
    ROOT --> FAQ[❓ FAQ<br/>faq.md]
    ROOT --> FLOW[🗺️ 文档流程<br/>document-flow.md]
    
    ROOT --> PRD[📋 PRD文档]
    ROOT --> ARCH[🏗️ 架构文档]
    ROOT --> DEV[💻 开发文档]
    ROOT --> TEST[🧪 测试文档]
    ROOT --> OPS[🔧 运维文档]
    ROOT --> SCRUM[📅 Scrum文档]
    
    PRD --> PRD1[PRD主文档]
    PRD --> PRD2[8个模块PRD]
    
    ARCH --> ARCH1[系统架构]
    ARCH --> ARCH2[8个ADR]
    
    DEV --> DEV1[开发指南]
    DEV --> DEV2[编码规范]
    DEV --> DEV3[3个语言指南]
    
    TEST --> TEST1[测试策略]
    TEST --> TEST2[高风险测试]
    TEST --> TEST3[测试数据管理]
    
    OPS --> OPS1[故障排查]
    OPS --> OPS2[监控方案]
    OPS --> OPS3[部署指南]
    
    SCRUM --> SCRUM1[SM指南]
    SCRUM --> SCRUM2[Sprint Planning]
    SCRUM --> SCRUM3[Retrospective]
```

---

## 🔍 按关键词查找

| 关键词 | 相关文档 |
|--------|---------|
| **入职、新手** | [quickstart.md](./quickstart.md), 语言开发者指南 |
| **编码、开发** | [编码规范](./development/coding-standards.md), [开发指南](./development/dev-guide.md) |
| **测试、QA** | [测试策略](./testing/test-strategy.md), [验收清单](./testing/acceptance-checklist.md) |
| **部署、运维** | [Docker指南](./deployment/docker-guide.md), [GitOps](./deployment/gitops-best-practices.md) |
| **故障、问题** | [故障排查](./operations/troubleshooting.md), [FAQ](./faq.md) |
| **架构、设计** | [系统架构](./architecture/system-architecture.md), [ADR](./architecture/decisions/) |
| **Scrum、流程** | [SM指南](./scrum/sm-guide.md), [Sprint Planning](./scrum/sprint-planning-checklist.md) |

---

## 📞 仍然找不到？

1. 使用浏览器搜索功能（Ctrl+F / Cmd+F）在 [文档导航](./README.md) 中搜索关键词
2. 查看 [FAQ](./faq.md)
3. 在 Slack `#hermesflow-dev` 提问

---

**最后更新**: 2025-01-13  
**维护者**: @pm.mdc  
**版本**: v1.0

