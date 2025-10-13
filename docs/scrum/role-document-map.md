# 角色文档地图

> **HermesFlow 量化交易平台 - 按角色的文档学习路径**  
> **版本**: v1.0 | **更新日期**: 2025-01-13

---

## 📋 目录

1. [文档地图说明](#文档地图说明)
2. [Scrum Master 文档地图](#scrum-master-文档地图)
3. [Product Owner 文档地图](#product-owner-文档地图)
4. [Tech Lead 文档地图](#tech-lead-文档地图)
5. [新手开发者文档地图](#新手开发者文档地图)
6. [Rust 开发者文档地图](#rust-开发者文档地图)
7. [Java 开发者文档地图](#java-开发者文档地图)
8. [Python 开发者文档地图](#python-开发者文档地图)
9. [QA 工程师文档地图](#qa-工程师文档地图)
10. [DevOps 工程师文档地图](#devops-工程师文档地图)

---

## 文档地图说明

### 什么是文档地图？

文档地图为每个角色提供 **个性化的文档学习路径**，帮助您：

- 🎯 **快速上手**: 新成员入职时知道先读什么、后读什么
- 📚 **系统学习**: 按照逻辑顺序学习，而不是盲目浏览
- 🔍 **快速查找**: 工作中需要文档时，知道去哪里找

### 如何使用？

1. **找到您的角色**: 在目录中点击您的角色
2. **按阶段学习**: 从"入职第1天"开始，逐步深入
3. **建立书签**: 将常用文档添加到浏览器书签
4. **按需查阅**: 遇到具体场景时，查看"场景化文档查找"

---

## Scrum Master 文档地图

### 🎯 入职第 1 天（预计 2 小时）

#### 必读文档

| 顺序 | 文档 | 路径 | 目的 | 预计时间 |
|------|------|------|------|---------|
| 1 | **快速入门** | [`quickstart.md`](../quickstart.md) | 了解项目概况 | ⏱️ 10分钟 |
| 2 | **Scrum Master 完整指南** | [`scrum/sm-guide.md`](./sm-guide.md) | 深入了解 SM 职责 | ⏱️ 1小时 |
| 3 | **项目进度** | [`progress.md`](../progress.md) | 了解当前开发状态 | ⏱️ 20分钟 |
| 4 | **每日工作参考** | [`scrum/daily-work-reference.md`](./daily-work-reference.md) | 熟悉每日工作流程 | ⏱️ 15分钟 |

#### 推荐浏览

| 文档 | 路径 | 目的 |
|------|------|------|
| **文档流程图** | [`document-flow.md`](../document-flow.md) | 理解文档关系 |
| **FAQ** | [`faq.md`](../faq.md) | 了解常见问题 |

---

### 📅 入职第 1 周（深入学习）

#### Sprint 流程文档

| 顺序 | 文档 | 路径 | 学习重点 | 预计时间 |
|------|------|------|---------|---------|
| 1 | **Sprint Planning 检查清单** | [`scrum/sprint-planning-checklist.md`](./sprint-planning-checklist.md) | Planning 准备和执行 | ⏱️ 30分钟 |
| 2 | **Sprint 阶段文档索引** | [`scrum/sprint-document-index.md`](./sprint-document-index.md) | 了解各阶段所需文档 | ⏱️ 45分钟 |
| 3 | **回顾模板** | [`scrum/retrospective-template.md`](./retrospective-template.md) | 学习回顾会议组织 | ⏱️ 30分钟 |
| 4 | **快速参考卡片** | [`scrum/quick-reference-cards.md`](./quick-reference-cards.md) | 记住关键流程 | ⏱️ 20分钟 |

#### 技术理解文档

| 文档 | 路径 | 目的 |
|------|------|------|
| **系统架构** | [`architecture/system-architecture.md`](../architecture/system-architecture.md) | 理解技术架构（不需深入） |
| **PRD** | [`prd/prd-hermesflow.md`](../prd/prd-hermesflow.md) | 了解产品需求 |
| **模块索引** | [`modules/module-index.md`](../modules/module-index.md) | 了解模块结构 |

---

### 🎯 每 Sprint Planning 前（1 小时）

#### 准备清单

| 顺序 | 文档 | 目的 |
|------|------|------|
| 1 | [`progress.md`](../progress.md) | 了解当前状态 |
| 2 | [`prd/prd-hermesflow.md`](../prd/prd-hermesflow.md) | 审查需求优先级 |
| 3 | [`modules/module-index.md`](../modules/module-index.md) | 查看模块依赖 |
| 4 | [`scrum/sprint-planning-checklist.md`](./sprint-planning-checklist.md) | 确认准备完成 |
| 5 | [`scrum/documentation-ready-checklist.md`](./documentation-ready-checklist.md) | 检查文档就绪 |

---

### 📆 每日工作（15 分钟）

#### 每日必读

| 时间 | 文档 | 目的 |
|------|------|------|
| **早晨 9:00** | [`scrum/daily-work-reference.md#scrum-master-每日清单`](./daily-work-reference.md) | 查看每日清单 |
| **站会前** | [`scrum/sm-guide.md#每日站会`](./sm-guide.md) | 复习站会流程 |
| **遇到问题** | [`operations/troubleshooting.md`](../operations/troubleshooting.md) | 排查问题 |

---

### 🎯 每 Sprint 结束前（1 小时）

#### Review 和 Retrospective

| 顺序 | 文档 | 目的 |
|------|------|------|
| 1 | [`scrum/sm-guide.md#sprint-review`](./sm-guide.md) | 准备 Review |
| 2 | [`scrum/retrospective-template.md`](./retrospective-template.md) | 准备 Retrospective |
| 3 | [`scrum/documentation-metrics.md`](./documentation-metrics.md) | 收集文档度量 |
| 4 | [`progress.md`](../progress.md) | 更新项目进度 |

---

### 🔍 场景化文档查找

| 场景 | 推荐文档 |
|------|---------|
| **团队遇到技术障碍** | [`operations/troubleshooting.md`](../operations/troubleshooting.md) → [`faq.md`](../faq.md) |
| **需要澄清需求** | [`prd/prd-hermesflow.md`](../prd/prd-hermesflow.md) → `prd/modules/` |
| **Code Review 慢** | [`development/code-review-checklist.md`](../development/code-review-checklist.md) |
| **测试失败** | [`testing/ci-cd-integration.md`](../testing/ci-cd-integration.md) |
| **文档过时** | [`scrum/documentation-best-practices.md`](./documentation-best-practices.md) |

---

## Product Owner 文档地图

### 🎯 入职第 1 天（预计 3 小时）

#### 必读文档

| 顺序 | 文档 | 路径 | 目的 | 预计时间 |
|------|------|------|------|---------|
| 1 | **快速入门** | [`quickstart.md`](../quickstart.md) | 了解项目概况 | ⏱️ 10分钟 |
| 2 | **PRD 主文档** | [`prd/prd-hermesflow.md`](../prd/prd-hermesflow.md) | 深入理解产品需求 | ⏱️ 2小时 |
| 3 | **市场分析** | [`analysis/market-analysis-and-gap-assessment.md`](../analysis/market-analysis-and-gap-assessment.md) | 了解市场定位 | ⏱️ 40分钟 |
| 4 | **项目进度** | [`progress.md`](../progress.md) | 了解开发状态 | ⏱️ 15分钟 |

---

### 📅 入职第 1 周（深入学习）

#### 产品文档

| 顺序 | 文档 | 目的 |
|------|------|------|
| 1 | **模块 PRD** (`prd/modules/`) | 深入了解各模块需求 |
| 2 | **UX 设计系统** ([`design/design-system.md`](../design/design-system.md)) | 理解 UI/UX 规范 |
| 3 | **页面设计** ([`design/page-designs.md`](../design/page-designs.md)) | 了解页面布局 |
| 4 | **用户故事** (`prd/user-stories/`) | 学习 Story 写法 |

#### 技术理解

| 文档 | 目的 |
|------|------|
| **系统架构概览** ([`architecture/system-architecture.md#系统概览`](../architecture/system-architecture.md)) | 理解技术限制（不需深入） |
| **API 设计** ([`api/api-design.md`](../api/api-design.md)) | 了解接口能力 |

---

### 📆 每日 / 每周工作

#### 每日参与

| 时间 | 文档 | 目的 |
|------|------|------|
| **早晨** | [`progress.md`](../progress.md) | 查看开发进度 |
| **每日站会** | [`scrum/daily-work-reference.md#product-owner-每日清单`](./daily-work-reference.md) | 聆听团队进展 |

#### 每周维护

| 文档 | 任务 | 频率 |
|------|------|------|
| **PRD** ([`prd/prd-hermesflow.md`](../prd/prd-hermesflow.md)) | 更新需求优先级 | 每周 |
| **Product Backlog** | 细化 Top 20 Story | 每周 |
| **市场分析** ([`analysis/market-analysis-and-gap-assessment.md`](../analysis/market-analysis-and-gap-assessment.md)) | 关注竞品动态 | 每周 |

---

## Tech Lead 文档地图

### 🎯 入职第 1 天（预计 3-4 小时）

#### 必读文档

| 顺序 | 文档 | 路径 | 目的 | 预计时间 |
|------|------|------|------|---------|
| 1 | **快速入门** | [`quickstart.md`](../quickstart.md) | 了解项目 | ⏱️ 10分钟 |
| 2 | **系统架构** | [`architecture/system-architecture.md`](../architecture/system-architecture.md) | 深入理解架构 | ⏱️ 2小时 |
| 3 | **ADR 决策记录** | [`architecture/decisions/`](../architecture/decisions/) | 了解架构决策 | ⏱️ 1小时 |
| 4 | **PRD** | [`prd/prd-hermesflow.md`](../prd/prd-hermesflow.md) | 理解需求 | ⏱️ 1小时 |

---

### 📅 入职第 1 周（深入学习）

#### 技术文档

| 优先级 | 文档 | 目的 |
|--------|------|------|
| 🔴 高 | **API 设计** ([`api/api-design.md`](../api/api-design.md)) | 理解接口设计 |
| 🔴 高 | **数据库设计** ([`database/database-design.md`](../database/database-design.md)) | 理解数据模型 |
| 🔴 高 | **CI/CD 架构** ([`architecture/system-architecture.md#ci-cd架构`](../architecture/system-architecture.md)) | 了解部署流程 |
| 🟡 中 | **开发指南** ([`development/dev-guide.md`](../development/dev-guide.md)) | 了解开发流程 |
| 🟡 中 | **编码规范** ([`development/coding-standards.md`](../development/coding-standards.md)) | 理解代码标准 |
| 🟢 低 | **测试策略** ([`testing/test-strategy.md`](../testing/test-strategy.md)) | 了解测试要求 |

---

### 📆 每 Sprint Planning 前

#### 技术准备

| 文档 | 任务 |
|------|------|
| **系统架构 - 技术债务** | 评估技术债务优先级 |
| **ADR** | 确认是否需要新的架构决策 |
| **API/数据库设计** | 审查本 Sprint 的设计变更 |

---

### 🔍 场景化文档查找

| 场景 | 推荐文档 |
|------|---------|
| **做架构决策** | [`architecture/decisions/`](../architecture/decisions/) (查看 ADR 模板) |
| **设计新 API** | [`api/api-design.md`](../api/api-design.md) |
| **数据库变更** | [`database/database-design.md`](../database/database-design.md) |
| **性能问题** | [`architecture/system-architecture.md#性能优化`](../architecture/system-architecture.md) |
| **Code Review** | [`development/code-review-checklist.md`](../development/code-review-checklist.md) |

---

## 新手开发者文档地图

> 适用于刚加入团队、编程经验 < 2 年的开发者

### 🎯 第 1 天（预计 2-3 小时）

#### 快速入门

| 顺序 | 文档 | 路径 | 目的 | 预计时间 |
|------|------|------|------|---------|
| 1 | **快速入门** | [`quickstart.md`](../quickstart.md) | 5分钟了解项目 | ⏱️ 10分钟 |
| 2 | **开发者快速入门** | [`development/developer-quickstart.md`](../development/developer-quickstart.md) | 环境搭建 | ⏱️ 1-2小时 |
| 3 | **项目进度** | [`progress.md`](../progress.md) | 了解当前状态 | ⏱️ 15分钟 |
| 4 | **FAQ** | [`faq.md`](../faq.md) | 了解常见问题 | ⏱️ 15分钟 |

---

### 📅 第 1 周（系统学习）

#### Day 1-2: 开发环境和基础

| 文档 | 目的 |
|------|------|
| **语言开发者指南** | 根据您的技术栈选择: [Rust](../development/rust-developer-guide.md) / [Java](../development/java-developer-guide.md) / [Python](../development/python-developer-guide.md) |
| **开发指南** ([`development/dev-guide.md`](../development/dev-guide.md)) | 了解项目结构和开发流程 |
| **快速参考** ([`development/quick-reference.md`](../development/quick-reference.md)) | 常用命令速查 |

#### Day 3-4: 代码规范和质量

| 文档 | 目的 |
|------|------|
| **编码规范** ([`development/coding-standards.md`](../development/coding-standards.md)) | 学习代码规范 |
| **Code Review 检查清单** ([`development/code-review-checklist.md`](../development/code-review-checklist.md)) | 了解 Code Review 标准 |
| **测试策略** ([`testing/test-strategy.md`](../testing/test-strategy.md)) | 学习如何写测试 |

#### Day 5: 熟悉业务

| 文档 | 目的 |
|------|------|
| **PRD** ([`prd/prd-hermesflow.md`](../prd/prd-hermesflow.md)) | 理解产品需求 |
| **模块索引** ([`modules/module-index.md`](../modules/module-index.md)) | 了解模块职责 |

---

### 📆 第 2 周（开始编码）

#### 编码前必读

| 文档 | 使用场景 |
|------|---------|
| **API 设计** ([`api/api-design.md`](../api/api-design.md)) | 需要调用 API 时 |
| **数据库设计** ([`database/database-design.md`](../database/database-design.md)) | 需要查询数据库时 |
| **每日工作参考** ([`scrum/daily-work-reference.md`](./daily-work-reference.md)) | 每天早晨查看 |

---

### 🔍 学习路径建议

#### 前 2 周重点

1. ✅ **环境搭建**: 能够成功运行项目
2. ✅ **理解流程**: 知道如何领取任务、提交代码、Code Review
3. ✅ **代码规范**: 能够写出符合规范的代码
4. ✅ **测试基础**: 能够编写基本的单元测试

#### 1 个月目标

1. ✅ 独立完成简单的 Story
2. ✅ 能够进行 Code Review
3. ✅ 理解项目架构和业务逻辑
4. ✅ 遇到问题知道查阅哪些文档

---

## Rust 开发者文档地图

### 🎯 入职必读（3-4 小时）

| 顺序 | 文档 | 路径 | 预计时间 |
|------|------|------|---------|
| 1 | **快速入门** | [`quickstart.md`](../quickstart.md) | ⏱️ 10分钟 |
| 2 | **Rust 开发者指南** | [`development/rust-developer-guide.md`](../development/rust-developer-guide.md) | ⏱️ 1.5小时 |
| 3 | **数据模块 PRD** | [`prd/modules/01-data-module.md`](../prd/modules/01-data-module.md) | ⏱️ 1小时 |
| 4 | **编码规范 - Rust** | [`development/coding-standards.md#rust-规范`](../development/coding-standards.md) | ⏱️ 30分钟 |
| 5 | **开发者快速入门** | [`development/developer-quickstart.md`](../development/developer-quickstart.md) | ⏱️ 1小时 |

---

### 📆 每日开发

#### 每日必读

| 时间 | 文档 | 目的 |
|------|------|------|
| **早晨** | [`scrum/daily-work-reference.md#rust-开发者特定清单`](./daily-work-reference.md) | 查看每日清单 |
| **编码时** | [`development/quick-reference.md`](../development/quick-reference.md) | 常用命令 |
| **Code Review 前** | [`development/code-review-checklist.md`](../development/code-review-checklist.md) | 自查清单 |

---

### 🔍 场景化文档查找

| 场景 | 推荐文档 |
|------|---------|
| **数据采集** | [`prd/modules/01-data-module.md#数据采集`](../prd/modules/01-data-module.md) |
| **性能优化** | [`prd/modules/01-data-module.md#性能基准`](../prd/modules/01-data-module.md) |
| **测试覆盖率** | [`testing/test-strategy.md#rust-测试`](../testing/test-strategy.md) (目标 ≥85%) |
| **部署** | [`deployment/docker-guide.md#rust-多阶段构建`](../deployment/docker-guide.md) |

---

## Java 开发者文档地图

### 🎯 入职必读（2-3 小时）

| 顺序 | 文档 | 路径 | 预计时间 |
|------|------|------|---------|
| 1 | **快速入门** | [`quickstart.md`](../quickstart.md) | ⏱️ 10分钟 |
| 2 | **Java 开发者指南** | [`development/java-developer-guide.md`](../development/java-developer-guide.md) | ⏱️ 1小时 |
| 3 | **相关模块 PRD** | `prd/modules/03-execution-module.md` (Trading Engine)<br>`prd/modules/04-risk-module.md` (Risk Engine)<br>`prd/modules/05-account-module.md` (User Management) | ⏱️ 1-2小时 |
| 4 | **编码规范 - Java** | [`development/coding-standards.md#java-规范`](../development/coding-standards.md) | ⏱️ 30分钟 |

---

### 📆 每日开发

#### 每日必读

| 时间 | 文档 | 目的 |
|------|------|------|
| **早晨** | [`scrum/daily-work-reference.md#java-开发者特定清单`](./daily-work-reference.md) | 查看每日清单 |
| **编码时** | [`development/quick-reference.md`](../development/quick-reference.md) | 常用命令 |

---

### 🔍 场景化文档查找

| 场景 | 推荐文档 |
|------|---------|
| **Spring Boot 配置** | [`development/java-developer-guide.md#spring-boot`](../development/java-developer-guide.md) |
| **数据库访问** | [`database/database-design.md`](../database/database-design.md) |
| **测试覆盖率** | [`testing/test-strategy.md#java-测试`](../testing/test-strategy.md) (目标 ≥80%) |
| **部署** | [`deployment/docker-guide.md#java-dockerfile`](../deployment/docker-guide.md) |

---

## Python 开发者文档地图

### 🎯 入职必读（2-3 小时）

| 顺序 | 文档 | 路径 | 预计时间 |
|------|------|------|---------|
| 1 | **快速入门** | [`quickstart.md`](../quickstart.md) | ⏱️ 10分钟 |
| 2 | **Python 开发者指南** | [`development/python-developer-guide.md`](../development/python-developer-guide.md) | ⏱️ 1小时 |
| 3 | **策略模块 PRD** | [`prd/modules/02-strategy-module.md`](../prd/modules/02-strategy-module.md) | ⏱️ 1小时 |
| 4 | **编码规范 - Python** | [`development/coding-standards.md#python-规范`](../development/coding-standards.md) | ⏱️ 30分钟 |

---

### 📆 每日开发

#### 每日必读

| 时间 | 文档 | 目的 |
|------|------|------|
| **早晨** | [`scrum/daily-work-reference.md#python-开发者特定清单`](./daily-work-reference.md) | 查看每日清单 |
| **编码时** | [`development/quick-reference.md`](../development/quick-reference.md) | 常用命令 |

---

### 🔍 场景化文档查找

| 场景 | 推荐文档 |
|------|---------|
| **策略开发** | [`prd/modules/02-strategy-module.md`](../prd/modules/02-strategy-module.md) |
| **回测引擎** | `archived/strategy-engine/` (v1.0 参考) |
| **测试覆盖率** | [`testing/test-strategy.md#python-测试`](../testing/test-strategy.md) (目标 ≥75%) |
| **部署** | [`deployment/docker-guide.md#python-dockerfile`](../deployment/docker-guide.md) |

---

## QA 工程师文档地图

### 🎯 入职第 1 天（2 小时）

| 顺序 | 文档 | 路径 | 预计时间 |
|------|------|------|---------|
| 1 | **快速入门** | [`quickstart.md`](../quickstart.md) | ⏱️ 10分钟 |
| 2 | **QA 工程师指南** | [`testing/qa-engineer-guide.md`](../testing/qa-engineer-guide.md) | ⏱️ 1小时 |
| 3 | **测试策略** | [`testing/test-strategy.md`](../testing/test-strategy.md) | ⏱️ 45分钟 |
| 4 | **PRD** | [`prd/prd-hermesflow.md`](../prd/prd-hermesflow.md) | ⏱️ 1小时 |

---

### 📅 入职第 1 周

#### 测试文档

| 优先级 | 文档 | 目的 |
|--------|------|------|
| 🔴 高 | **验收检查清单** ([`testing/acceptance-checklist.md`](../testing/acceptance-checklist.md)) | 了解验收标准 |
| 🔴 高 | **高风险访问测试** ([`testing/high-risk-access-testing.md`](../testing/high-risk-access-testing.md)) | 学习安全测试 |
| 🔴 高 | **测试数据管理** ([`testing/test-data-management.md`](../testing/test-data-management.md)) | 准备测试数据 |
| 🟡 中 | **CI/CD 集成** ([`testing/ci-cd-integration.md`](../testing/ci-cd-integration.md)) | 了解自动化测试 |
| 🟡 中 | **早期测试策略** ([`testing/early-test-strategy.md`](../testing/early-test-strategy.md)) | 理解测试计划 |

---

### 📆 每日测试

#### 每日必读

| 时间 | 文档 | 目的 |
|------|------|------|
| **早晨** | [`scrum/daily-work-reference.md#qa-工程师每日清单`](./daily-work-reference.md) | 查看每日清单 |
| **测试前** | [`testing/qa-engineer-guide.md`](../testing/qa-engineer-guide.md) | 复习测试流程 |
| **遇到问题** | [`operations/troubleshooting.md`](../operations/troubleshooting.md) | 问题排查 |

---

### 🔍 场景化文档查找

| 场景 | 推荐文档 |
|------|---------|
| **功能测试** | [`testing/qa-engineer-guide.md#功能测试`](../testing/qa-engineer-guide.md) |
| **安全测试** | [`testing/high-risk-access-testing.md`](../testing/high-risk-access-testing.md) |
| **性能测试** | `tests/performance/load_test.js` |
| **自动化测试** | [`testing/test-strategy.md#自动化测试`](../testing/test-strategy.md) |
| **Bug 管理** | [`testing/qa-engineer-guide.md#缺陷管理`](../testing/qa-engineer-guide.md) |

---

## DevOps 工程师文档地图

### 🎯 入职第 1 天（3-4 小时）

| 顺序 | 文档 | 路径 | 预计时间 |
|------|------|------|---------|
| 1 | **快速入门** | [`quickstart.md`](../quickstart.md) | ⏱️ 10分钟 |
| 2 | **DevOps 指南** | [`operations/devops-guide.md`](../operations/devops-guide.md) | ⏱️ 1.5小时 |
| 3 | **CI/CD 架构** | [`architecture/system-architecture.md#ci-cd架构`](../architecture/system-architecture.md) | ⏱️ 1小时 |
| 4 | **监控指南** | [`operations/monitoring.md`](../operations/monitoring.md) | ⏱️ 1小时 |

---

### 📅 入职第 1 周

#### 部署和运维文档

| 优先级 | 文档 | 目的 |
|--------|------|------|
| 🔴 高 | **Docker 指南** ([`deployment/docker-guide.md`](../deployment/docker-guide.md)) | 学习容器化 |
| 🔴 高 | **GitOps 最佳实践** ([`deployment/gitops-best-practices.md`](../deployment/gitops-best-practices.md)) | 学习声明式部署 |
| 🔴 高 | **CI/CD 流程图** ([`architecture/diagrams/cicd-flow.md`](../architecture/diagrams/cicd-flow.md)) | 理解部署流程 |
| 🟡 中 | **故障排查指南** ([`operations/troubleshooting.md`](../operations/troubleshooting.md)) | 学习问题排查 |

---

### 📆 每日运维

#### 每日必读

| 时间 | 文档 | 目的 |
|------|------|------|
| **早晨** | [`scrum/daily-work-reference.md#devops-工程师每日清单`](./daily-work-reference.md) | 查看每日清单 |
| **部署时** | [`scrum/quick-reference-cards.md#部署快速参考`](./quick-reference-cards.md) | 部署步骤 |
| **遇到问题** | [`operations/troubleshooting.md`](../operations/troubleshooting.md) | 问题排查 |

---

### 🔍 场景化文档查找

| 场景 | 推荐文档 |
|------|---------|
| **部署新服务** | [`deployment/gitops-best-practices.md#新服务部署`](../deployment/gitops-best-practices.md) |
| **配置变更** | [`deployment/gitops-best-practices.md#配置管理`](../deployment/gitops-best-practices.md) |
| **监控告警** | [`operations/monitoring.md`](../operations/monitoring.md) |
| **性能优化** | [`architecture/system-architecture.md#性能优化`](../architecture/system-architecture.md) |
| **故障处理** | [`operations/troubleshooting.md`](../operations/troubleshooting.md) |

---

## 附录

### 快速导航表

| 我是... | 从这里开始 | 我的每日文档 |
|---------|-----------|-------------|
| **Scrum Master** | [SM 文档地图](#scrum-master-文档地图) | [每日工作参考](./daily-work-reference.md#scrum-master-每日清单) |
| **Product Owner** | [PO 文档地图](#product-owner-文档地图) | [PRD](../prd/prd-hermesflow.md) + [进度](../progress.md) |
| **Tech Lead** | [Tech Lead 文档地图](#tech-lead-文档地图) | [系统架构](../architecture/system-architecture.md) |
| **新手开发者** | [新手开发者文档地图](#新手开发者文档地图) | [每日工作参考](./daily-work-reference.md#开发者每日清单) |
| **Rust 开发者** | [Rust 文档地图](#rust-开发者文档地图) | [Rust 每日清单](./daily-work-reference.md#rust-开发者特定清单) |
| **Java 开发者** | [Java 文档地图](#java-开发者文档地图) | [Java 每日清单](./daily-work-reference.md#java-开发者特定清单) |
| **Python 开发者** | [Python 文档地图](#python-开发者文档地图) | [Python 每日清单](./daily-work-reference.md#python-开发者特定清单) |
| **QA 工程师** | [QA 文档地图](#qa-工程师文档地图) | [QA 每日清单](./daily-work-reference.md#qa-工程师每日清单) |
| **DevOps** | [DevOps 文档地图](#devops-工程师文档地图) | [DevOps 每日清单](./daily-work-reference.md#devops-工程师每日清单) |

---

### 相关文档

- [Sprint 阶段文档索引](./sprint-document-index.md) - 按 Sprint 阶段查找文档
- [每日工作参考](./daily-work-reference.md) - 每日工作清单
- [快速参考卡片](./quick-reference-cards.md) - 一页纸快速参考
- [文档流程图](../document-flow.md) - 文档关系可视化

---

**维护者**: Scrum Master + HR  
**审查频率**: 每季度  
**反馈**: 新成员入职时收集使用反馈

