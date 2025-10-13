# Sprint 阶段文档索引

> **HermesFlow 量化交易平台 - Sprint 全周期文档导航**  
> **版本**: v1.0 | **更新日期**: 2025-01-13

---

## 📋 目录

1. [文档索引说明](#文档索引说明)
2. [Sprint 准备阶段 (Pre-Sprint)](#sprint-准备阶段-pre-sprint)
3. [Sprint Planning (2-4小时)](#sprint-planning-2-4小时)
4. [Sprint 执行阶段 (每日)](#sprint-执行阶段-每日)
5. [Sprint Review (2小时)](#sprint-review-2小时)
6. [Sprint Retrospective (1.5小时)](#sprint-retrospective-15小时)
7. [文档快速查找表](#文档快速查找表)

---

## 文档索引说明

### 使用方法

本文档为 Scrum Master 和团队成员提供 **按 Sprint 阶段组织的文档清单**。每个阶段列出：

- ✅ **必读文档**: 该阶段必须查阅的关键文档
- 📖 **推荐文档**: 建议阅读以提高效率的文档
- 🔍 **按需文档**: 遇到特定问题时查阅的文档

### 文档标记说明

| 标记 | 含义 | 示例 |
|------|------|------|
| 🔴 | 高优先级，必读 | Sprint Planning 前必读 PRD |
| 🟡 | 中优先级，推荐 | Code Review 前推荐阅读规范 |
| 🟢 | 低优先级，按需 | 遇到部署问题时查阅 |
| ⏱️ | 预计阅读时间 | 5分钟 / 30分钟 / 2小时 |

---

## Sprint 准备阶段 (Pre-Sprint)

> **时间**: Sprint 开始前 3-5 天  
> **参与者**: Product Owner, Scrum Master, Tech Lead  
> **目标**: 确保 Product Backlog 就绪，为 Sprint Planning 做好准备

### Product Owner 文档清单

#### 🔴 必读文档

| 文档 | 路径 | 目的 | 预计时间 |
|------|------|------|---------|
| **PRD 主文档** | [`prd/prd-hermesflow.md`](../prd/prd-hermesflow.md) | 审查产品需求，确认功能优先级 | ⏱️ 1小时 |
| **项目进度** | [`progress.md`](../progress.md) | 了解当前开发状态和已完成功能 | ⏱️ 15分钟 |
| **模块索引** | [`modules/module-index.md`](../modules/module-index.md) | 查看各模块文档和依赖关系 | ⏱️ 10分钟 |

#### 📖 推荐文档

| 文档 | 路径 | 目的 | 预计时间 |
|------|------|------|---------|
| **市场分析** | [`analysis/market-analysis-and-gap-assessment.md`](../analysis/market-analysis-and-gap-assessment.md) | 理解产品定位和竞争优势 | ⏱️ 30分钟 |
| **UX 设计** | [`design/design-system.md`](../design/design-system.md) | 确认 UI/UX 设计规范 | ⏱️ 20分钟 |

#### 🟢 按需文档

- **用户故事模板**: `prd/user-stories/` (需要细化用户故事时)
- **模块 PRD**: `prd/modules/` (深入了解特定模块需求)

---

### Scrum Master 文档清单

#### 🔴 必读文档

| 文档 | 路径 | 目的 | 预计时间 |
|------|------|------|---------|
| **项目进度** | [`progress.md`](../progress.md) | 更新 Sprint 燃尽图基线 | ⏱️ 15分钟 |
| **上次回顾行动项** | 上次 Sprint Retrospective 记录 | 确认行动项完成情况 | ⏱️ 10分钟 |
| **Sprint Planning 检查清单** | [`scrum/sprint-planning-checklist.md`](./sprint-planning-checklist.md) | 准备 Planning 会议材料 | ⏱️ 10分钟 |

#### 📖 推荐文档

| 文档 | 路径 | 目的 | 预计时间 |
|------|------|------|---------|
| **Scrum Master 指南** | [`scrum/sm-guide.md`](./sm-guide.md) | 复习 Scrum 最佳实践 | ⏱️ 20分钟 |
| **文档就绪检查清单** | [`scrum/documentation-ready-checklist.md`](./documentation-ready-checklist.md) | 确认文档完备性 | ⏱️ 15分钟 |

---

### Tech Lead 文档清单

#### 🔴 必读文档

| 文档 | 路径 | 目的 | 预计时间 |
|------|------|------|---------|
| **系统架构** | [`architecture/system-architecture.md`](../architecture/system-architecture.md) | 审查架构设计和技术债务 | ⏱️ 30分钟 |
| **ADR 决策记录** | [`architecture/decisions/`](../architecture/decisions/) | 了解最新架构决策 | ⏱️ 20分钟 |
| **技术债务** | [`architecture/system-architecture.md#技术债务`](../architecture/system-architecture.md) | 评估技术债务优先级 | ⏱️ 15分钟 |

#### 📖 推荐文档

| 文档 | 路径 | 目的 | 预计时间 |
|------|------|------|---------|
| **API 设计** | [`api/api-design.md`](../api/api-design.md) | 确认 API 接口设计 | ⏱️ 20分钟 |
| **数据库设计** | [`database/database-design.md`](../database/database-design.md) | 审查数据库 schema 变更 | ⏱️ 15分钟 |

---

## Sprint Planning (2-4小时)

> **时间**: Sprint 第1天上午  
> **参与者**: 全体团队 + PO + SM  
> **目标**: 确定 Sprint 目标，选择 Story，分解任务，估算工作量

### 准备阶段 (会前30分钟)

#### 🔴 Scrum Master 准备

| 文档 | 路径 | 目的 | 预计时间 |
|------|------|------|---------|
| **Sprint Planning 检查清单** | [`scrum/sprint-planning-checklist.md`](./sprint-planning-checklist.md) | 确认会议准备完成 | ⏱️ 10分钟 |
| **快速参考卡片** | [`scrum/quick-reference-cards.md#sprint-planning-快速参考`](./quick-reference-cards.md) | 快速查看会议流程 | ⏱️ 5分钟 |

#### 🔴 全体成员准备

| 文档 | 路径 | 目的 | 预计时间 |
|------|------|------|---------|
| **PRD 主文档** | [`prd/prd-hermesflow.md`](../prd/prd-hermesflow.md) | 了解 Product Backlog Top 项 | ⏱️ 30分钟 |
| **项目进度** | [`progress.md`](../progress.md) | 了解当前状态 | ⏱️ 10分钟 |

---

### Part 1: 确定 Sprint 目标 (1小时)

#### 🔴 必读文档

| 文档 | 路径 | 目的 |
|------|------|------|
| **PRD - 开发路线图** | [`prd/prd-hermesflow.md#开发路线图`](../prd/prd-hermesflow.md) | 对齐产品方向 |
| **项目进度** | [`progress.md`](../progress.md) | 确认当前阶段 |

#### 📖 推荐文档

| 文档 | 路径 | 目的 |
|------|------|------|
| **模块 PRD** | `prd/modules/` | 深入了解模块需求 |

---

### Part 2: 选择 Story (1.5小时)

#### 🔴 必读文档

| 文档 | 路径 | 目的 |
|------|------|------|
| **用户故事** | `prd/user-stories/` | 理解验收标准 |
| **模块索引** | [`modules/module-index.md`](../modules/module-index.md) | 查看模块依赖 |

#### 📖 推荐文档

| 文档 | 路径 | 目的 |
|------|------|------|
| **API 设计** | [`api/api-design.md`](../api/api-design.md) | 确认接口定义 |
| **数据库设计** | [`database/database-design.md`](../database/database-design.md) | 确认数据模型 |

---

### Part 3: 任务分解和估算 (1.5小时)

#### 🔴 必读文档 (按开发语言)

**Rust 开发者**:
| 文档 | 路径 | 目的 |
|------|------|------|
| **Rust 开发者指南** | [`development/rust-developer-guide.md`](../development/rust-developer-guide.md) | 了解 Rust 开发规范 |
| **编码规范** | [`development/coding-standards.md#rust-规范`](../development/coding-standards.md) | 确认代码标准 |

**Java 开发者**:
| 文档 | 路径 | 目的 |
|------|------|------|
| **Java 开发者指南** | [`development/java-developer-guide.md`](../development/java-developer-guide.md) | 了解 Java 开发规范 |
| **编码规范** | [`development/coding-standards.md#java-规范`](../development/coding-standards.md) | 确认代码标准 |

**Python 开发者**:
| 文档 | 路径 | 目的 |
|------|------|------|
| **Python 开发者指南** | [`development/python-developer-guide.md`](../development/python-developer-guide.md) | 了解 Python 开发规范 |
| **编码规范** | [`development/coding-standards.md#python-规范`](../development/coding-standards.md) | 确认代码标准 |

#### 📖 推荐文档

| 文档 | 路径 | 目的 |
|------|------|------|
| **开发指南** | [`development/dev-guide.md`](../development/dev-guide.md) | 查看开发流程 |
| **测试策略** | [`testing/test-strategy.md`](../testing/test-strategy.md) | 了解测试要求 |

---

### Part 4: 风险识别和 DoD (1小时)

#### 🔴 必读文档

| 文档 | 路径 | 目的 |
|------|------|------|
| **验收检查清单** | [`testing/acceptance-checklist.md`](../testing/acceptance-checklist.md) | 确认 Definition of Done |
| **CI/CD 集成** | [`testing/ci-cd-integration.md`](../testing/ci-cd-integration.md) | 了解自动化测试要求 |

#### 📖 推荐文档

| 文档 | 路径 | 目的 |
|------|------|------|
| **故障排查指南** | [`operations/troubleshooting.md`](../operations/troubleshooting.md) | 识别潜在风险 |
| **高风险访问测试** | [`testing/high-risk-access-testing.md`](../testing/high-risk-access-testing.md) | 安全相关风险 |

---

## Sprint 执行阶段 (每日)

> **时间**: Sprint 第2天 - 最后1天  
> **参与者**: 全体团队  
> **目标**: 完成 Sprint Backlog，交付可工作的增量

### 每日站会 (10:00-10:15)

#### 🔴 Scrum Master 必读

| 文档 | 路径 | 目的 | 预计时间 |
|------|------|------|---------|
| **每日工作参考** | [`scrum/daily-work-reference.md#scrum-master-每日清单`](./daily-work-reference.md) | 查看每日清单 | ⏱️ 5分钟 |
| **SM 指南 - 每日站会** | [`scrum/sm-guide.md#每日站会`](./sm-guide.md) | 复习站会流程 | ⏱️ 5分钟 |

#### 🔴 全体成员必读

| 文档 | 路径 | 目的 | 预计时间 |
|------|------|------|---------|
| **快速参考卡片 - 每日站会** | [`scrum/quick-reference-cards.md#每日站会快速参考`](./quick-reference-cards.md) | 准备三个问题 | ⏱️ 3分钟 |

---

### 开发工作 (全天)

#### 🔴 开发者每日必读

| 文档 | 路径 | 目的 | 预计时间 |
|------|------|------|---------|
| **每日工作参考 - 开发者** | [`scrum/daily-work-reference.md#开发者每日清单`](./daily-work-reference.md) | 查看每日清单 | ⏱️ 5分钟 |
| **快速参考** | [`development/quick-reference.md`](../development/quick-reference.md) | 常用命令和流程 | ⏱️ 按需 |

#### 📖 推荐文档 (按需查阅)

**编码阶段**:
| 文档 | 路径 | 使用场景 |
|------|------|---------|
| **编码规范** | [`development/coding-standards.md`](../development/coding-standards.md) | 不确定代码风格时 |
| **API 设计** | [`api/api-design.md`](../api/api-design.md) | 调用接口时 |
| **数据库设计** | [`database/database-design.md`](../database/database-design.md) | 查询数据表结构时 |

**测试阶段**:
| 文档 | 路径 | 使用场景 |
|------|------|---------|
| **测试策略** | [`testing/test-strategy.md`](../testing/test-strategy.md) | 编写测试用例时 |
| **测试数据管理** | [`testing/test-data-management.md`](../testing/test-data-management.md) | 准备测试数据时 |

**问题排查**:
| 文档 | 路径 | 使用场景 |
|------|------|---------|
| **故障排查指南** | [`operations/troubleshooting.md`](../operations/troubleshooting.md) | 遇到问题时 |
| **FAQ** | [`faq.md`](../faq.md) | 常见问题 |

---

### Code Review

#### 🔴 必读文档

| 文档 | 路径 | 目的 | 预计时间 |
|------|------|------|---------|
| **Code Review 检查清单** | [`development/code-review-checklist.md`](../development/code-review-checklist.md) | 审查代码前必读 | ⏱️ 5分钟 |
| **快速参考卡片 - Code Review** | [`scrum/quick-reference-cards.md#code-review快速参考`](./quick-reference-cards.md) | 快速参考标准 | ⏱️ 2分钟 |

#### 📖 推荐文档

| 文档 | 路径 | 目的 |
|------|------|------|
| **编码规范** | [`development/coding-standards.md`](../development/coding-standards.md) | 确认代码规范 |
| **测试策略** | [`testing/test-strategy.md`](../testing/test-strategy.md) | 验证测试覆盖率 |

---

### 部署

#### 🔴 必读文档

| 文档 | 路径 | 目的 | 预计时间 |
|------|------|------|---------|
| **快速参考卡片 - 部署** | [`scrum/quick-reference-cards.md#部署快速参考`](./quick-reference-cards.md) | 部署步骤速查 | ⏱️ 3分钟 |
| **Docker 指南** | [`deployment/docker-guide.md`](../deployment/docker-guide.md) | 容器化部署 | ⏱️ 15分钟 |

#### 📖 推荐文档

| 文档 | 路径 | 目的 |
|------|------|------|
| **GitOps 最佳实践** | [`deployment/gitops-best-practices.md`](../deployment/gitops-best-practices.md) | 声明式部署 |
| **DevOps 指南** | [`operations/devops-guide.md`](../operations/devops-guide.md) | 运维流程 |

---

### QA 测试

#### 🔴 QA 每日必读

| 文档 | 路径 | 目的 | 预计时间 |
|------|------|------|---------|
| **每日工作参考 - QA** | [`scrum/daily-work-reference.md#qa-每日清单`](./daily-work-reference.md) | 查看每日清单 | ⏱️ 5分钟 |
| **QA 工程师指南** | [`testing/qa-engineer-guide.md`](../testing/qa-engineer-guide.md) | 测试流程 | ⏱️ 20分钟 |

#### 📖 推荐文档

| 文档 | 路径 | 目的 |
|------|------|------|
| **测试策略** | [`testing/test-strategy.md`](../testing/test-strategy.md) | 测试计划 |
| **高风险访问测试** | [`testing/high-risk-access-testing.md`](../testing/high-risk-access-testing.md) | 安全测试 |
| **验收检查清单** | [`testing/acceptance-checklist.md`](../testing/acceptance-checklist.md) | 验收标准 |

---

## Sprint Review (2小时)

> **时间**: Sprint 最后1天下午  
> **参与者**: 团队 + PO + Stakeholders  
> **目标**: 展示 Sprint 成果，收集反馈

### Sprint Review 准备 (会前1天)

#### 🔴 Scrum Master 准备

| 文档 | 路径 | 目的 | 预计时间 |
|------|------|------|---------|
| **SM 指南 - Sprint Review** | [`scrum/sm-guide.md#sprint-review`](./sm-guide.md) | 准备 Review 会议 | ⏱️ 15分钟 |
| **项目进度** | [`progress.md`](../progress.md) | 更新完成情况 | ⏱️ 20分钟 |

#### 🔴 开发者准备

| 文档 | 路径 | 目的 | 预计时间 |
|------|------|------|---------|
| **用户故事** | `prd/user-stories/` | 准备 Demo 脚本 | ⏱️ 30分钟 |
| **验收检查清单** | [`testing/acceptance-checklist.md`](../testing/acceptance-checklist.md) | 确认验收标准 | ⏱️ 10分钟 |

#### 📖 推荐文档

| 文档 | 路径 | 目的 |
|------|------|------|
| **UX 设计** | [`design/page-designs.md`](../design/page-designs.md) | 对比设计稿 |
| **PRD** | [`prd/prd-hermesflow.md`](../prd/prd-hermesflow.md) | 确认需求实现 |

---

### Sprint Review 会议中

#### 🔴 必读文档

| 文档 | 路径 | 目的 |
|------|------|------|
| **快速参考卡片 - Sprint Review** | 未来创建 | 会议流程速查 |

#### 📖 备用文档 (按需查阅)

| 文档 | 路径 | 使用场景 |
|------|------|---------|
| **系统架构** | [`architecture/system-architecture.md`](../architecture/system-architecture.md) | 解释技术实现 |
| **ADR** | [`architecture/decisions/`](../architecture/decisions/) | 说明架构决策 |

---

## Sprint Retrospective (1.5小时)

> **时间**: Sprint 最后1天下午 (Review 之后)  
> **参与者**: 团队 + SM (PO 可选)  
> **目标**: 反思 Sprint 过程，制定改进行动计划

### Retrospective 准备 (会前1天)

#### 🔴 Scrum Master 准备

| 文档 | 路径 | 目的 | 预计时间 |
|------|------|------|---------|
| **回顾模板** | [`scrum/retrospective-template.md`](./retrospective-template.md) | 准备回顾会议 | ⏱️ 15分钟 |
| **SM 指南 - Retrospective** | [`scrum/sm-guide.md#sprint-retrospective`](./sm-guide.md) | 复习回顾流程 | ⏱️ 10分钟 |
| **文档度量** | [`scrum/documentation-metrics.md`](./documentation-metrics.md) | 收集数据指标 | ⏱️ 20分钟 |

#### 📖 全体成员准备

| 文档 | 路径 | 目的 | 预计时间 |
|------|------|------|---------|
| **上次行动项** | 上次 Retrospective 记录 | 检查完成情况 | ⏱️ 5分钟 |

---

### Retrospective 会议中

#### 🔴 必读文档

| 文档 | 路径 | 目的 |
|------|------|------|
| **回顾模板** | [`scrum/retrospective-template.md`](./retrospective-template.md) | 引导讨论 |
| **快速参考卡片 - Retrospective** | 未来创建 | 会议流程速查 |

---

### Retrospective 会后

#### 🔴 Scrum Master 任务

| 文档 | 路径 | 任务 | 预计时间 |
|------|------|------|---------|
| **项目进度** | [`progress.md`](../progress.md) | 更新 Sprint 总结 | ⏱️ 30分钟 |
| **行动项跟踪** | 团队协作工具 | 创建行动项任务 | ⏱️ 15分钟 |

#### 📖 按需更新

| 文档 | 路径 | 更新内容 |
|------|------|---------|
| **文档最佳实践** | [`scrum/documentation-best-practices.md`](./documentation-best-practices.md) | 新的文档使用经验 |
| **故障排查指南** | [`operations/troubleshooting.md`](../operations/troubleshooting.md) | 新的问题解决方案 |
| **FAQ** | [`faq.md`](../faq.md) | 新的常见问题 |

---

## 文档快速查找表

### 按角色查找

| 角色 | 详细文档地图 |
|------|-------------|
| **Scrum Master** | [`scrum/role-document-map.md#scrum-master-文档地图`](./role-document-map.md) |
| **Product Owner** | [`scrum/role-document-map.md#product-owner-文档地图`](./role-document-map.md) |
| **Tech Lead** | [`scrum/role-document-map.md#tech-lead-文档地图`](./role-document-map.md) |
| **Rust 开发者** | [`scrum/role-document-map.md#rust-开发者文档地图`](./role-document-map.md) |
| **Java 开发者** | [`scrum/role-document-map.md#java-开发者文档地图`](./role-document-map.md) |
| **Python 开发者** | [`scrum/role-document-map.md#python-开发者文档地图`](./role-document-map.md) |
| **QA 工程师** | [`scrum/role-document-map.md#qa-工程师文档地图`](./role-document-map.md) |
| **DevOps** | [`scrum/role-document-map.md#devops-文档地图`](./role-document-map.md) |

---

### 按场景查找

| 场景 | 推荐文档 |
|------|---------|
| **新成员入职** | [新手开发者文档地图](./role-document-map.md#新手开发者文档地图) |
| **遇到问题** | [故障排查指南](../operations/troubleshooting.md) → [FAQ](../faq.md) |
| **编写代码** | [编码规范](../development/coding-standards.md) → [快速参考](../development/quick-reference.md) |
| **Code Review** | [Code Review 检查清单](../development/code-review-checklist.md) |
| **部署发布** | [部署快速参考](./quick-reference-cards.md#部署快速参考) |
| **测试** | [测试策略](../testing/test-strategy.md) → [QA 指南](../testing/qa-engineer-guide.md) |

---

### 按频率查找

#### 每天使用

- [每日工作参考](./daily-work-reference.md)
- [快速参考卡片](./quick-reference-cards.md)
- [快速参考](../development/quick-reference.md)

#### 每周使用

- [Code Review 检查清单](../development/code-review-checklist.md)
- [项目进度](../progress.md)

#### 每 Sprint 使用

- [Sprint Planning 检查清单](./sprint-planning-checklist.md)
- [回顾模板](./retrospective-template.md)
- [验收检查清单](../testing/acceptance-checklist.md)

#### 按需使用

- [故障排查指南](../operations/troubleshooting.md)
- [系统架构](../architecture/system-architecture.md)
- [PRD](../prd/prd-hermesflow.md)

---

## 附录

### 文档更新责任

| Sprint 阶段 | 需要更新的文档 | 责任人 |
|------------|---------------|--------|
| **Pre-Sprint** | Product Backlog, PRD | Product Owner |
| **Planning** | Sprint Backlog, 任务分解 | Scrum Master + 团队 |
| **执行** | 代码、测试、文档 | 开发者、QA |
| **Review** | 项目进度、Demo 记录 | Scrum Master |
| **Retrospective** | 回顾记录、行动项 | Scrum Master |

---

### 相关文档

- [每日工作参考](./daily-work-reference.md) - 按角色的每日清单
- [角色文档地图](./role-document-map.md) - 按角色的学习路径
- [文档就绪检查清单](./documentation-ready-checklist.md) - Sprint 前的文档检查
- [快速参考卡片](./quick-reference-cards.md) - 一页纸快速参考
- [Scrum Master 指南](./sm-guide.md) - 完整的 SM 工作手册

---

**维护者**: Scrum Master  
**审查频率**: 每月  
**反馈**: 在 Sprint Retrospective 中收集文档使用反馈

