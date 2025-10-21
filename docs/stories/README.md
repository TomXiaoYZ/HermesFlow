# HermesFlow User Stories

本目录包含HermesFlow项目所有Sprint的User Stories文档。

---

## 📂 目录结构

```
docs/stories/
├── README.md                    # 本文件 - Stories索引
├── sprint-01/                   # Sprint 1 - DevOps Foundation
│   ├── sprint-01-summary.md    # Sprint 1总结
│   ├── DEVOPS-001-github-actions-cicd.md
│   ├── DEVOPS-002-azure-terraform-iac.md
│   ├── DEVOPS-003-argocd-gitops.md
│   ├── sprint-01-dev-notes.md
│   └── sprint-01-qa-notes.md
└── sprint-XX/                   # 未来的Sprint
```

---

## 🏃 Sprint列表

### Sprint 1: DevOps Foundation ✅ COMPLETED (2025-01-10 ~ 2025-10-21)

**Sprint目标**: 建立CI/CD自动化和Azure云基础架构，部署 ArgoCD GitOps

| Story ID | Title | Priority | Estimate | Status |
|----------|-------|----------|----------|--------|
| [DEVOPS-001](./sprint-01/DEVOPS-001-github-actions-cicd.md) | GitHub Actions CI/CD Pipeline | P0 | 8 SP | ✅ Done |
| [DEVOPS-002](./sprint-01/DEVOPS-002-azure-terraform-iac.md) | Azure Infrastructure (Terraform) | P0 | 13 SP | ✅ Done |
| [DEVOPS-003](./sprint-01/DEVOPS-003-argocd-gitops.md) | ArgoCD GitOps 部署 (成本优化版) | P1 | 8 SP | ✅ Done |

**Sprint成果**:
- ✅ 29/29 Story Points 完成 (100%)
- ✅ CI/CD 自动化部署 (4-5分钟)
- ✅ 成本优化 85% ($626 → $96/月)
- ✅ 3个服务成功部署运行
- ✅ QA评分: A- (90/100)

📋 **核心文档**:
- [Sprint 1 Summary](./sprint-01/sprint-01-summary.md) - Sprint总结和回顾
- [Sprint 1 Final Report](./sprint-01/sprint-01-final-report.md) - 完整的最终报告
- [Sprint 1 Demo Guide](./sprint-01/sprint-01-demo.md) - 演示脚本和成果展示

**开发文档**:
- 📝 [Dev Notes](./sprint-01/sprint-01-dev-notes.md) - 技术决策、实施细节、问题解决
- 🧪 [QA Notes](./sprint-01/sprint-01-qa-notes.md) - 测试计划、用例、质量指标
- ✅ [PO Validation](./sprint-01/DEVOPS-003-po-validation.md) - Product Owner验证报告

**QA文档**:
- ⚠️ [Risk Profile](./sprint-01/sprint-01-risk-profile.md) - 42个风险点识别与缓解
- 🧪 [Test Strategy](./sprint-01/sprint-01-test-strategy.md) - 6大测试类型，80%自动化率
- ✅ [Test Cases](./sprint-01/sprint-01-test-cases.md) - 100个详细测试用例

**总工作量**: 29 Story Points ✅ 全部完成

---

## 📖 Story编写规范

### Story模板结构

每个Story文档应包含以下部分：

1. **元数据**: Story ID, Epic, Priority, Estimate, Sprint, Status, 创建日期
2. **User Story**: 作为...我想要...以便...
3. **验收标准**: Gherkin格式的场景描述
4. **技术任务分解**: 详细的任务列表和工作量估算
5. **测试策略**: 单元测试、集成测试、性能测试
6. **依赖关系**: 前置依赖和后续依赖
7. **相关文档**: 链接到架构、PRD、技术文档
8. **Definition of Done**: 明确的完成标准
9. **开发笔记**: 开发过程记录
10. **Story历史**: 状态变更记录

### 文件命名规范

```
{STORY-ID}-{short-description}.md

示例:
- DEVOPS-001-github-actions-cicd.md
- DATA-001-binance-websocket-connector.md
- STRATEGY-001-alpha-factor-library.md
```

### Story ID命名规范

```
{MODULE}-{NUMBER}

模块前缀:
- DEVOPS: DevOps基础设施
- DATA: 数据模块
- STRATEGY: 策略模块
- EXECUTION: 交易执行模块
- RISK: 风控模块
- ACCOUNT: 账户模块
- FRONTEND: 前端模块
- SECURITY: 安全模块

示例:
- DEVOPS-001
- DATA-001
- STRATEGY-001
```

---

## 🎯 Story状态

| 状态 | 说明 | 标记 |
|------|------|------|
| **Draft** | 初稿,待验证 | 📝 |
| **Approved** | 已验证,进入Backlog | ✅ |
| **In Progress** | 开发中 | 🚧 |
| **In Review** | 代码审查中 | 👀 |
| **Done** | 完成,通过验收 | ✔️ |
| **Blocked** | 被阻塞 | 🚫 |

---

## 📊 Story Points估算参考

| Story Points | 工作时间 | 复杂度 | 示例 |
|--------------|---------|--------|------|
| 1 SP | 2h | 极简单 | 配置文件修改 |
| 2 SP | 4h | 简单 | 单个小功能 |
| 3 SP | 6h | 简单偏中 | 单个模块功能 |
| 5 SP | 10h | 中等 | 多个相关功能 |
| 8 SP | 16h | 中偏复杂 | 完整子系统 |
| 13 SP | 26h | 复杂 | 跨模块集成 |
| 21 SP | 42h | 很复杂 | 大型特性 |

**原则**: 超过13 SP的Story应该拆分成更小的Stories

---

## 🔗 相关文档

### 项目文档
- [项目进度](../progress.md)
- [PRD主文档](../prd/prd-hermesflow.md)
- [系统架构](../architecture/system-architecture.md)

### Scrum流程
- [Scrum Master指南](../scrum/sm-guide.md)
- [Sprint Planning检查清单](../scrum/sprint-planning-checklist.md)
- [Documentation Ready检查清单](../scrum/documentation-ready-checklist.md)

### 开发指南
- [Java开发指南](../development/java-developer-guide.md)
- [Python开发指南](../development/python-developer-guide.md)
- [Rust开发指南](../development/rust-developer-guide.md)
- [编码规范](../development/coding-standards.md)

---

## 📈 Epic概览

### Epic 1: DevOps Foundation ✅ COMPLETED (Sprint 1)
**目标**: CI/CD + Azure基础设施 + ArgoCD GitOps
- [x] DEVOPS-001: GitHub Actions CI/CD ✅
- [x] DEVOPS-002: Azure Terraform IaC ✅
- [x] DEVOPS-003: ArgoCD GitOps 部署 (成本优化版) ✅
- **完成日期**: 2025-10-21
- **成果**: 29 SP完成, 85%成本节省, A-评分

### Epic 2: 数据模块 - 加密货币数据采集 (计划Q1 2025)
**目标**: 实现Binance/OKX WebSocket实时数据采集
- [ ] DATA-001: Binance WebSocket连接器
- [ ] DATA-002: OKX WebSocket连接器
- [ ] DATA-003: 数据标准化处理
- [ ] DATA-004: Redis缓存优化
- [ ] DATA-005: ClickHouse批量写入

### Epic 3: 策略模块 - Alpha因子库 (计划Q1 2025)
**目标**: 实现100+ Alpha因子计算引擎
- [ ] STRATEGY-001: 因子框架设计
- [ ] STRATEGY-002: 技术指标因子(50+)
- [ ] STRATEGY-003: 价量因子(30+)
- [ ] STRATEGY-004: 市场微观结构因子(20+)
- [ ] STRATEGY-005: 因子性能分析

---

## 🔍 Story检索

### 按优先级
- **P0 (Critical)**: [DEVOPS-001](./sprint-01/DEVOPS-001-github-actions-cicd.md), [DEVOPS-002](./sprint-01/DEVOPS-002-azure-terraform-iac.md)
- **P1 (High)**: [DEVOPS-003](./sprint-01/DEVOPS-003-argocd-gitops.md)
- **P2 (Medium)**: _待添加_

### 按状态
- **Done**: [DEVOPS-001](./sprint-01/DEVOPS-001-github-actions-cicd.md), [DEVOPS-002](./sprint-01/DEVOPS-002-azure-terraform-iac.md), [DEVOPS-003](./sprint-01/DEVOPS-003-argocd-gitops.md)
- **In Progress**: _无_
- **Approved**: _无_

### 按模块
- **DevOps**: [DEVOPS-001](./sprint-01/DEVOPS-001-github-actions-cicd.md), [DEVOPS-002](./sprint-01/DEVOPS-002-azure-terraform-iac.md), [DEVOPS-003](./sprint-01/DEVOPS-003-argocd-gitops.md)
- **Data**: _待添加_
- **Strategy**: _待添加_

---

## 👥 角色与职责

### Scrum Master (@sm.mdc)
- 起草User Stories
- 技术任务分解
- 协助团队移除障碍

### Product Owner (@po.mdc)
- 验证Stories与产品愿景对齐
- 确认验收标准完整性
- 批准Stories进入Backlog

### 开发团队
- Review Stories并提供反馈
- 估算Story Points
- 实现Stories并更新开发笔记

---

## 📝 更新日志

| 日期 | 变更 | 操作人 |
|------|------|--------|
| 2025-01-13 | 创建Stories目录和Sprint 1 Stories | @sm.mdc |
| 2025-01-13 | Sprint 1 Stories通过验证 | @po.mdc |
| 2025-10-14 | 添加 DEVOPS-003 ArgoCD GitOps Story | @sm.mdc |
| 2025-10-14 | DEVOPS-003 通过 PO 验证 (96.25/100, A 级) | @po.mdc |
| 2025-10-14 | 创建 Dev Notes 和 QA Notes | @sm.mdc |
| 2025-10-21 | Sprint 1 全部Story完成，标记为Done | @sm.mdc |
| 2025-10-21 | 创建Sprint 1 Final Report和Demo Guide | @sm.mdc |
| 2025-10-21 | 更新Sprint 1 Summary包含完整回顾 | @sm.mdc |

---

**Last Updated**: 2025-10-21  
**Maintained By**: @sm.mdc

