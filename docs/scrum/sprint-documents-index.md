# Sprint 文档索引

> **按开发周期阶段组织的完整文档导航** | **版本**: v1.0

---

## 📋 目录

1. [Week 1: Sprint 第一周](#week-1-sprint-第一周)
2. [Week 2: Sprint 第二周](#week-2-sprint-第二周)
3. [跨 Sprint 文档](#跨-sprint-文档)
4. [快速查找表](#快速查找表)

---

## Week 1: Sprint 第一周

### Day 1: Sprint Planning (Monday)

#### 🎯 会前准备文档

**必读** (Sprint 开始前 2 天):
- [ ] [Sprint Planning 清单](./sprint-planning-checklist.md) - 完整准备清单
- [ ] [产品需求文档 (PRD)](../prd/prd-hermesflow.md) - 了解功能需求
- [ ] [项目进度](../progress.md) - 查看当前状态和技术债务
- [ ] [系统架构文档](../architecture/system-architecture.md) - 理解技术架构

**参考文档**:
- [ ] [模块文档索引](../modules/module-index.md) - 按模块查找详细需求
- [ ] [架构决策记录 (ADR)](../architecture/decisions/) - 关键技术选型

---

#### 📦 Sprint Planning 会议文档

**会议中使用**:
- [ ] [Sprint Planning 清单](./sprint-planning-checklist.md) - 逐项检查
- [ ] [Sprint 启动包](./sprint-starter-pack.md) - 启动模板和清单
  - Sprint 目标模板
  - Story 分解模板
  - 团队协议模板

**会议输出**:
- Sprint 目标文档
- Sprint Backlog
- 风险登记表
- Definition of Done 确认

---

#### 🚀 Day 1 下午：环境准备

**开发环境**:
- [ ] [开发者快速开始](../development/developer-quickstart.md) - 环境搭建
- [ ] [开发指南](../development/dev-guide.md) - 开发流程和工具
- [ ] [快速参考手册](../development/quick-reference.md) - 常用命令

**分语言指南**:
- [ ] [Rust 开发者指南](../development/rust-developer-guide.md) - 数据引擎
- [ ] [Java 开发者指南](../development/java-developer-guide.md) - 交易/用户/风控
- [ ] [Python 开发者指南](../development/python-developer-guide.md) - 策略引擎

---

### Day 2-4: 开发冲刺 (Tuesday-Thursday)

#### 🗣️ 每日站会

**必用文档**:
- [ ] [每日站会指南](./daily-standup-guide.md) - 完整站会流程
  - 三个问题模板
  - 障碍记录表
  - Parking Lot 管理

---

#### 💻 开发阶段

**编码规范**:
- [ ] [编码规范](../development/coding-standards.md) - Rust/Java/Python 规范
- [ ] [代码审查清单](../development/code-review-checklist.md) - Code Review 标准

**API 和数据库**:
- [ ] [API 设计文档](../api/api-design.md) - REST API 和 gRPC 规范
- [ ] [数据库设计文档](../database/database-design.md) - Schema 和索引

**按模块查找**:
- [ ] [模块文档索引](../modules/module-index.md)
  - 数据模块、策略模块、交易模块等
  - 每个模块的详细需求和 API

---

#### 🧪 测试阶段

**测试策略**:
- [ ] [测试策略](../testing/test-strategy.md) - 单元/集成/性能测试
- [ ] [测试数据管理](../testing/test-data-management.md) - Fixtures 和 Mocking
- [ ] [高风险访问测试](../testing/high-risk-access-testing.md) - 安全和多租户测试

**测试执行**:
- [ ] [QA 工程师指南](../testing/qa-engineer-guide.md) - 完整测试指南
- [ ] [CI/CD 测试集成](../testing/ci-cd-integration.md) - 自动化测试

---

#### 🚧 障碍处理

**故障排查**:
- [ ] [FAQ 文档](../faq.md) - 常见问题快速解答
- [ ] [故障排查手册](../operations/troubleshooting.md) - 应急处理

---

### Day 5: Mid-Sprint Check-in (Friday)

#### 📊 进度检查

**必用文档**:
- [ ] [开发周期总览](./development-cycle-overview.md) - Mid-Sprint 检查点
- [ ] [项目进度](../progress.md) - 更新当前进度

**检查清单**:
- Sprint 目标达成情况（50% 检查点）
- Sprint 燃尽图分析
- 阻塞和风险识别
- Sprint Backlog 调整（如需要）

---

## Week 2: Sprint 第二周

### Day 6-8: 功能完成冲刺 (Monday-Wednesday)

#### 💻 持续开发

**继续使用 Week 1 的开发文档**:
- [ ] [每日站会指南](./daily-standup-guide.md)
- [ ] [编码规范](../development/coding-standards.md)
- [ ] [代码审查清单](../development/code-review-checklist.md)

**新增关注**:
- [ ] [Definition of Done](./definition-of-done.md) - 确保所有 DoD 标准满足

---

#### ✅ Definition of Done 检查

**必查文档**:
- [ ] [Definition of Done](./definition-of-done.md) - 完整 DoD 清单
  - 代码层面（Code Review, Linter, Security Scan）
  - 测试层面（Unit/Integration/Performance/Security Tests）
  - 文档层面（Code Doc, API Doc, README, CHANGELOG）
  - 部署层面（CI/CD, Docker, Helm, Dev Env）
  - 验收层面（验收标准、PO 验收）

---

### Day 9: Demo 准备 (Thursday)

#### 🎬 Demo 准备

**必用文档**:
- [ ] [Sprint Review 清单](./sprint-review-checklist.md) - Review 完整清单
  - Demo 脚本模板
  - Demo 干跑清单
  - 反馈收集表

**Demo 脚本准备**:
- 每个功能准备 Demo 脚本
- 分配演讲人
- 准备测试数据
- 干跑一次 Demo

---

### Day 10: Sprint Review & Retrospective (Friday)

#### 🎭 Sprint Review (15:00-17:00)

**必用文档**:
- [ ] [Sprint Review 清单](./sprint-review-checklist.md)
  - 会议流程
  - Demo 原则
  - 反馈收集表
  - Product Backlog 更新

**会议议程**:
1. Sprint 概述（10分钟）
2. Demo 完成的功能（60分钟）
3. 未完成的工作和原因（10分钟）
4. 讨论和反馈（30分钟）
5. Product Backlog 更新（10分钟）

---

#### 🔄 Sprint Retrospective (17:00-18:30)

**必用文档**:
- [ ] [Retrospective 模板](./retrospective-template.md) - 完整 Retro 流程
  - Start/Stop/Continue 方法
  - 5 Whys 分析法
  - 改进行动模板

**会议议程**:
1. 设定氛围（5分钟）
2. 收集数据（20分钟）
3. 产生洞察（30分钟）
4. 决定行动（30分钟）
5. 回顾上次行动（10分钟）
6. 总结和关闭（5分钟）

---

## 跨 Sprint 文档

### 持续参考文档

这些文档在整个 Sprint 中持续参考：

#### 📘 产品和需求
- [ ] [产品需求文档 (PRD)](../prd/prd-hermesflow.md)
- [ ] [模块文档索引](../modules/module-index.md)
- [ ] [用户故事](../prd/user-stories/)

#### 🏗️ 架构和设计
- [ ] [系统架构文档](../architecture/system-architecture.md)
- [ ] [架构决策记录 (ADR)](../architecture/decisions/)
- [ ] [CI/CD 架构](../architecture/system-architecture.md#第11章-cicd架构)
- [ ] [CI/CD 流程图](../architecture/diagrams/cicd-flow.md)

#### 🎨 设计系统
- [ ] [设计系统](../design/design-system.md)
- [ ] [页面设计规范](../design/page-designs.md)

#### 🔧 开发工具
- [ ] [编码规范](../development/coding-standards.md)
- [ ] [开发指南](../development/dev-guide.md)
- [ ] [代码审查清单](../development/code-review-checklist.md)

#### 🧪 测试文档
- [ ] [测试策略](../testing/test-strategy.md)
- [ ] [QA 工程师指南](../testing/qa-engineer-guide.md)
- [ ] [验收测试清单](../testing/acceptance-checklist.md)

#### 🚀 部署和运维
- [ ] [Docker 部署指南](../deployment/docker-guide.md)
- [ ] [GitOps 最佳实践](../deployment/gitops-best-practices.md)
- [ ] [DevOps 工程师指南](../operations/devops-guide.md)
- [ ] [监控方案](../operations/monitoring.md)
- [ ] [故障排查手册](../operations/troubleshooting.md)

#### ❓ 帮助文档
- [ ] [FAQ 文档](../faq.md)
- [ ] [快速开始指南](../quickstart.md)
- [ ] [文档导航中心](../README.md)

---

## 快速查找表

### 按场景查找文档

| 场景 | 文档 |
|------|------|
| **我是新成员** | [开发者快速开始](../development/developer-quickstart.md), [团队入职包](../development/team-onboarding-pack.md) |
| **准备 Sprint Planning** | [Sprint Planning 清单](./sprint-planning-checklist.md), [Sprint 启动包](./sprint-starter-pack.md) |
| **开始编码** | [编码规范](../development/coding-standards.md), [语言指南](../development/) |
| **不知道如何实现** | [系统架构](../architecture/system-architecture.md), [ADR](../architecture/decisions/) |
| **需要 Code Review** | [代码审查清单](../development/code-review-checklist.md) |
| **编写测试** | [测试策略](../testing/test-strategy.md), [测试数据管理](../testing/test-data-management.md) |
| **遇到问题** | [FAQ](../faq.md), [故障排查手册](../operations/troubleshooting.md) |
| **部署失败** | [Docker 指南](../deployment/docker-guide.md), [GitOps 最佳实践](../deployment/gitops-best-practices.md) |
| **准备 Demo** | [Sprint Review 清单](./sprint-review-checklist.md) |
| **Sprint 回顾** | [Retrospective 模板](./retrospective-template.md) |

---

### 按角色查找文档

| 角色 | 核心文档 |
|------|---------|
| **Scrum Master** | [SM 指南](./sm-guide.md), [Sprint Planning 清单](./sprint-planning-checklist.md), [每日站会指南](./daily-standup-guide.md), [开发周期总览](./development-cycle-overview.md) |
| **Product Owner** | [PRD](../prd/prd-hermesflow.md), [项目进度](../progress.md), [模块索引](../modules/module-index.md) |
| **Rust 开发者** | [Rust 开发者指南](../development/rust-developer-guide.md), [数据模块需求](../prd/modules/01-data-module.md) |
| **Java 开发者** | [Java 开发者指南](../development/java-developer-guide.md), [交易/用户/风控模块](../modules/module-index.md) |
| **Python 开发者** | [Python 开发者指南](../development/python-developer-guide.md), [策略模块需求](../prd/modules/02-strategy-module.md) |
| **QA 工程师** | [QA 工程师指南](../testing/qa-engineer-guide.md), [测试策略](../testing/test-strategy.md), [验收清单](../testing/acceptance-checklist.md) |
| **DevOps 工程师** | [DevOps 指南](../operations/devops-guide.md), [GitOps 最佳实践](../deployment/gitops-best-practices.md), [监控方案](../operations/monitoring.md) |
| **UX 设计师** | [设计系统](../design/design-system.md), [页面设计](../design/page-designs.md) |

---

### 按文档类型查找

| 类型 | 文档 |
|------|------|
| **清单类** | [Sprint Planning](./sprint-planning-checklist.md), [Sprint Review](./sprint-review-checklist.md), [Code Review](../development/code-review-checklist.md), [验收测试](../testing/acceptance-checklist.md) |
| **指南类** | [SM 指南](./sm-guide.md), [每日站会](./daily-standup-guide.md), [开发指南](../development/dev-guide.md), [QA 指南](../testing/qa-engineer-guide.md) |
| **模板类** | [Sprint 启动包](./sprint-starter-pack.md), [Retrospective 模板](./retrospective-template.md) |
| **参考类** | [编码规范](../development/coding-standards.md), [API 设计](../api/api-design.md), [数据库设计](../database/database-design.md) |
| **故障排查** | [FAQ](../faq.md), [故障排查手册](../operations/troubleshooting.md) |

---

## 📊 文档使用频率

### 高频文档（每天使用）

1. [每日站会指南](./daily-standup-guide.md) - 每天 10:00
2. [编码规范](../development/coding-standards.md) - 编码时
3. [代码审查清单](../development/code-review-checklist.md) - Code Review 时
4. [FAQ 文档](../faq.md) - 遇到问题时

### 中频文档（每周使用）

1. [系统架构文档](../architecture/system-architecture.md) - 设计时
2. [测试策略](../testing/test-strategy.md) - 测试时
3. [故障排查手册](../operations/troubleshooting.md) - 排查时
4. [项目进度](../progress.md) - 进度同步时

### 低频文档（每 Sprint 使用）

1. [Sprint Planning 清单](./sprint-planning-checklist.md) - Sprint 开始时
2. [Sprint Review 清单](./sprint-review-checklist.md) - Sprint 结束时
3. [Retrospective 模板](./retrospective-template.md) - Sprint 回顾时
4. [开发周期总览](./development-cycle-overview.md) - Sprint 规划时

---

## 🔍 文档更新流程

### 如何更新文档

1. **识别需要更新的文档**
   - Sprint 中遇到的问题
   - Retrospective 中的改进项
   - 新的最佳实践

2. **创建更新 PR**
   ```bash
   git checkout -b docs/update-sprint-guide
   # 编辑文档
   git add docs/scrum/sprint-documents-index.md
   git commit -m "docs(scrum): update sprint documents index"
   git push origin docs/update-sprint-guide
   ```

3. **请求 Review**
   - 文档维护者: @pm.mdc
   - 相关角色审查

4. **合并和发布**
   - 合并 PR
   - 在 Slack 通知团队

---

## 📚 相关资源

### 主要导航
- [文档导航中心](../README.md) - 所有文档的入口
- [文档流程图](../document-flow.md) - 可视化文档使用流程
- [角色文档快速索引](../role-based-quick-index.md) - 按角色快速查找

### Scrum 文档
- [Scrum Master 完整指南](./sm-guide.md)
- [Sprint 启动包](./sprint-starter-pack.md)
- [每日站会指南](./daily-standup-guide.md)
- [Sprint Review 清单](./sprint-review-checklist.md)
- [Retrospective 模板](./retrospective-template.md)
- [Definition of Done](./definition-of-done.md)
- [开发周期总览](./development-cycle-overview.md)

---

## 💡 使用建议

### 新 Sprint 开始时

1. **Day 1 上午**（Sprint Planning 前）:
   - 打开 [Sprint Planning 清单](./sprint-planning-checklist.md)
   - 逐项检查准备工作
   - 准备相关文档（PRD, 架构, 进度）

2. **Day 1 下午**（Sprint 启动后）:
   - 查看 [Sprint 启动包](./sprint-starter-pack.md)
   - 跟随清单完成启动活动
   - 设置开发环境

### 每天开发时

1. **上午 10:00**:
   - 打开 [每日站会指南](./daily-standup-guide.md)
   - 准备三个问题的答案
   - 参加站会

2. **开发时**:
   - 参考 [编码规范](../development/coding-standards.md)
   - 使用 [代码审查清单](../development/code-review-checklist.md)
   - 遇到问题查看 [FAQ](../faq.md)

### Sprint 结束时

1. **Day 9 下午**:
   - 打开 [Sprint Review 清单](./sprint-review-checklist.md)
   - 准备 Demo 脚本
   - 干跑 Demo

2. **Day 10**:
   - 使用 [Sprint Review 清单](./sprint-review-checklist.md) 进行 Review
   - 使用 [Retrospective 模板](./retrospective-template.md) 进行回顾

---

**最后更新**: 2025-01-13  
**维护者**: @pm.mdc  
**版本**: v1.0

