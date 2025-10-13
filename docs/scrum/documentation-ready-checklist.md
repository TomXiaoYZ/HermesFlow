# 文档就绪检查清单

> **HermesFlow 量化交易平台 - Sprint 文档准备和质量检查**  
> **版本**: v1.0 | **更新日期**: 2025-01-13

---

## 📋 目录

1. [检查清单说明](#检查清单说明)
2. [Sprint 开始前检查](#sprint-开始前检查)
3. [Sprint 进行中检查](#sprint-进行中检查)
4. [Sprint 结束检查](#sprint-结束检查)
5. [文档质量标准](#文档质量标准)
6. [常见文档问题](#常见文档问题)

---

## 检查清单说明

### 目的

本检查清单确保 **Sprint 各阶段的文档准备充分**，帮助：

- ✅ Sprint Planning 前文档就绪，避免会议延误
- ✅ Sprint 执行中文档保持更新，支持团队工作
- ✅ Sprint 结束时文档完整，便于交接和回顾

### 使用方法

1. **Scrum Master** 在各阶段运行相应的检查清单
2. **逐项检查**，标记完成情况：✅ 完成 / ⚠️ 部分完成 / ❌ 未完成
3. **及时跟进** 未完成项，在会议前解决
4. **记录问题**，在 Retrospective 中讨论改进

---

## Sprint 开始前检查

> **检查时间**: Sprint Planning 前 1-2 天  
> **检查人**: Scrum Master  
> **目标**: 确保 Sprint Planning 会议所需文档齐全

---

### 1. Product Owner 准备

#### 产品需求文档

- [ ] **PRD 已更新到最新版本**
  - 检查: [`prd/prd-hermesflow.md`](../prd/prd-hermesflow.md) 版本号和更新日期
  - 标准: 更新日期在最近 1 个月内
  - 行动: 如过时，通知 PO 更新

- [ ] **Product Backlog Top 20 已细化**
  - 检查: 任务管理工具 (Jira / GitHub Projects)
  - 标准: 前 20 个 Story 有详细描述和验收标准
  - 行动: 与 PO 一起细化 Story

- [ ] **用户故事验收标准清晰明确**
  - 检查: 每个 Story 的 Acceptance Criteria
  - 标准: Given-When-Then 格式，可测试
  - 行动: 请 PO 和 QA 一起明确验收标准

- [ ] **依赖关系已标记**
  - 检查: Story 之间的依赖关系
  - 标准: 跨 Story/模块的依赖已标记
  - 行动: 与 Tech Lead 一起识别技术依赖

#### 模块需求文档

- [ ] **本 Sprint 涉及的模块 PRD 已审查**
  - 文档: `prd/modules/` 下的相关模块文档
  - 标准: 模块 PRD 与 Product Backlog 一致
  - 行动: 发现不一致时，与 PO 澄清

---

### 2. Scrum Master 准备

#### Sprint 流程文档

- [ ] **Sprint Planning 议程已准备**
  - 文档: [`scrum/sprint-planning-checklist.md`](./sprint-planning-checklist.md)
  - 标准: 会议议程、时间分配、目标明确
  - 行动: 准备会议材料，预订会议室

- [ ] **估算工具已就绪**
  - 工具: Planning Poker 卡片 / 在线工具
  - 标准: 工具可用，团队熟悉
  - 行动: 测试工具，准备备用方案

- [ ] **项目进度已更新**
  - 文档: [`progress.md`](../progress.md)
  - 标准: 上个 Sprint 的进度已记录
  - 行动: 更新完成的 Story、遗留问题、燃尽图

- [ ] **上个 Sprint 行动项已关闭/跟进**
  - 来源: 上次 Sprint Retrospective
  - 标准: 所有行动项有状态更新
  - 行动: 跟进未完成的行动项，决定是否继续

#### 团队状态文档

- [ ] **团队容量已确认**
  - 信息: 请假、培训、其他承诺
  - 标准: 每个成员的可用工作日明确
  - 行动: 收集团队成员的可用性信息

- [ ] **上 Sprint 速度已计算**
  - 数据: 上个 Sprint 完成的 Story Points
  - 标准: 速度数据准确，团队知晓
  - 行动: 计算速度，准备在 Planning 中分享

---

### 3. 技术准备

#### 架构文档

- [ ] **架构文档已审查**
  - 文档: [`architecture/system-architecture.md`](../architecture/system-architecture.md)
  - 标准: 架构文档反映当前系统状态
  - 行动: 与 Tech Lead 确认，更新过时内容

- [ ] **ADR 已更新**
  - 文档: [`architecture/decisions/`](../architecture/decisions/)
  - 标准: 上 Sprint 的架构决策已记录
  - 行动: Tech Lead 补充缺失的 ADR

- [ ] **技术债务已评估**
  - 来源: [`architecture/system-architecture.md#技术债务`](../architecture/system-architecture.md)
  - 标准: 技术债务清单更新，优先级明确
  - 行动: Tech Lead 评估本 Sprint 需要处理的技术债务

#### API 和数据库文档

- [ ] **API 文档已更新**
  - 文档: [`api/api-design.md`](../api/api-design.md)
  - 标准: 新增/变更的 API 已记录
  - 行动: 开发者更新 API 文档

- [ ] **数据库 schema 已确认**
  - 文档: [`database/database-design.md`](../database/database-design.md)
  - 标准: 数据库变更已记录，迁移脚本就绪
  - 行动: DBA/开发者审查 schema 变更

#### 部署文档

- [ ] **部署流程已确认**
  - 文档: [`deployment/gitops-best-practices.md`](../deployment/gitops-best-practices.md)
  - 标准: 部署步骤清晰，环境就绪
  - 行动: DevOps 确认部署环境可用

---

### 4. 测试准备

#### 测试策略文档

- [ ] **测试策略已确认**
  - 文档: [`testing/test-strategy.md`](../testing/test-strategy.md)
  - 标准: 测试策略与本 Sprint 目标一致
  - 行动: QA Lead 审查测试计划

- [ ] **验收标准已明确**
  - 来源: Product Backlog 中的 Story
  - 标准: 每个 Story 的验收标准可测试
  - 行动: QA 与 PO 一起确认验收标准

- [ ] **测试环境已就绪**
  - 文档: [`testing/test-data-management.md`](../testing/test-data-management.md)
  - 标准: 测试环境可用，测试数据准备完毕
  - 行动: QA/DevOps 搭建测试环境

#### 测试用例文档

- [ ] **高风险测试用例已准备**
  - 文档: [`testing/high-risk-access-testing.md`](../testing/high-risk-access-testing.md)
  - 标准: 安全、多租户隔离等高风险场景有测试用例
  - 行动: QA 准备关键测试用例

---

### 5. 文档完整性检查

#### 核心文档齐全性

- [ ] **所有必读文档可访问**
  - 检查: 点击 [`scrum/sprint-document-index.md#sprint-planning`](./sprint-document-index.md) 中的所有链接
  - 标准: 所有链接有效，文档可打开
  - 行动: 修复失效链接，补充缺失文档

- [ ] **文档格式一致**
  - 检查: 使用 [`naming-conventions.md`](../naming-conventions.md) 检查命名规范
  - 标准: 文件名遵循 kebab-case，Markdown 格式正确
  - 行动: 重命名不符合规范的文件

---

### Sprint 开始前检查摘要

#### 完成标准

| 角色 | 完成标准 | 检查方式 |
|------|---------|---------|
| **Product Owner** | 4/4 产品文档完成 | 逐项勾选 |
| **Scrum Master** | 6/6 流程文档完成 | 逐项勾选 |
| **Tech Lead** | 6/6 技术文档完成 | 逐项勾选 |
| **QA Lead** | 3/3 测试文档完成 | 逐项勾选 |
| **文档质量** | 2/2 完整性检查通过 | 逐项勾选 |

#### Go / No-Go 决策

- ✅ **Go**: 所有检查项 ≥ 90% 完成 → Sprint Planning 可以按计划进行
- ⚠️ **Conditional Go**: 80-90% 完成 → Planning 可进行，但需在 Sprint 前 2 天补充
- ❌ **No-Go**: < 80% 完成 → 推迟 Sprint Planning，集中精力完成文档

---

## Sprint 进行中检查

> **检查时间**: 每周中 (Sprint 中期)  
> **检查人**: Scrum Master  
> **目标**: 确保文档随开发进度同步更新

---

### 1. 每日文档使用情况

#### 文档访问统计

- [ ] **团队成员是否在使用文档？**
  - 方法: 在每日站会后询问，观察提问频率
  - 标准: > 80% 成员表示文档有帮助
  - 行动: 如使用率低，了解原因，改进文档

- [ ] **是否出现频繁的相同问题？**
  - 现象: 多人问同样的问题
  - 标准: 同类问题不超过 2 次
  - 行动: 将常见问题添加到 [`faq.md`](../faq.md)

---

### 2. 文档更新需求

#### 代码变更引起的文档更新

- [ ] **API 变更是否同步更新文档？**
  - 检查: [`api/api-design.md`](../api/api-design.md) 更新日期
  - 标准: API 变更后 1 天内更新文档
  - 行动: 在 Code Review 中检查文档更新

- [ ] **数据库 schema 变更是否记录？**
  - 检查: [`database/database-design.md`](../database/database-design.md) change log
  - 标准: 每次 schema 变更有记录
  - 行动: 在 PR 描述中要求说明 schema 变更

- [ ] **新功能是否更新用户故事？**
  - 检查: `prd/user-stories/` 目录
  - 标准: 新功能有对应的用户故事
  - 行动: PO 补充用户故事

---

### 3. 缺失文档识别

#### 缺失文档收集

- [ ] **记录团队提出的文档需求**
  - 方法: 在 Slack / 站会中收集
  - 记录: 维护一个"缺失文档清单"
  - 示例: "希望有 Redis 使用最佳实践文档"

- [ ] **标记过时或错误的文档**
  - 方法: 鼓励团队成员在文档中添加注释
  - 格式: `<!-- TODO: 更新为新版本的 API -->`
  - 行动: 在 Retrospective 前清理这些 TODO

---

### Sprint 进行中检查摘要

#### 每周检查点

| 检查项 | Week 1 | Week 2 | Week 3 |
|--------|--------|--------|--------|
| 文档使用率 | ☐ | ☐ | ☐ |
| API 文档更新 | ☐ | ☐ | ☐ |
| 数据库文档更新 | ☐ | ☐ | ☐ |
| 新缺失文档 | ☐ | ☐ | ☐ |

---

## Sprint 结束检查

> **检查时间**: Sprint Review 前 1 天  
> **检查人**: Scrum Master  
> **目标**: 确保 Sprint 产出文档完整，为下个 Sprint 做好准备

---

### 1. 文档更新完成度

#### 代码相关文档

- [ ] **所有 merged PR 的文档更新已完成**
  - 检查: GitHub PR 中的文档变更
  - 标准: 每个功能 PR 包含文档更新
  - 行动: 回顾 PR 列表，补充缺失文档

- [ ] **API 文档与代码一致**
  - 检查: [`api/api-design.md`](../api/api-design.md) vs. 实际代码
  - 标准: API 文档准确反映当前接口
  - 行动: 开发者验证并更新

- [ ] **数据库文档与实际 schema 一致**
  - 检查: [`database/database-design.md`](../database/database-design.md) vs. 数据库
  - 标准: DDL 与实际表结构一致
  - 行动: DBA 运行脚本验证

#### 测试文档

- [ ] **测试报告已生成**
  - 位置: CI/CD 系统 或 测试管理工具
  - 标准: 包含测试覆盖率、通过率、Bug 统计
  - 行动: QA 生成并分享测试报告

- [ ] **新增测试用例已记录**
  - 位置: `tests/` 目录 或 测试管理工具
  - 标准: 新功能有对应的测试用例
  - 行动: QA 补充测试用例文档

#### 运维文档

- [ ] **部署记录已更新**
  - 文档: DevOps 变更日志
  - 标准: 本 Sprint 的部署、配置变更已记录
  - 行动: DevOps 补充部署记录

- [ ] **监控和告警配置已更新**
  - 文档: [`operations/monitoring.md`](../operations/monitoring.md)
  - 标准: 新服务/接口的监控已配置
  - 行动: DevOps 更新监控文档

---

### 2. 新文档需求收集

#### Sprint Retrospective 准备

- [ ] **收集本 Sprint 的文档问题**
  - 来源: 团队反馈、Slack 讨论、缺失文档清单
  - 格式: 问题描述 + 影响 + 建议解决方案
  - 示例: "缺少 Redis 使用指南，导致 3 次重复提问，建议创建"

- [ ] **准备文档度量数据**
  - 文档: [`scrum/documentation-metrics.md`](./documentation-metrics.md)
  - 数据: 文档访问量、更新频率、问题数量
  - 目的: 在 Retrospective 中讨论

---

### 3. 文档问题反馈

#### 质量问题

- [ ] **记录发现的文档错误**
  - 示例: 链接失效、信息过时、格式错误
  - 行动: 创建 Issue 跟踪，指派责任人

- [ ] **记录文档缺失项**
  - 示例: 缺少故障排查步骤、缺少 API 示例
  - 行动: 加入下 Sprint 的 Backlog

---

### Sprint 结束检查摘要

#### 完成标准

| 类别 | 完成标准 | 检查方式 |
|------|---------|---------|
| **代码文档** | 3/3 完成 | 逐项勾选 |
| **测试文档** | 2/2 完成 | 逐项勾选 |
| **运维文档** | 2/2 完成 | 逐项勾选 |
| **反馈收集** | 2/2 完成 | 逐项勾选 |

#### Retrospective 议题

- 📝 本 Sprint 文档问题清单
- 📊 文档度量数据
- 💡 下 Sprint 文档改进计划

---

## 文档质量标准

### 优秀文档的特征

#### 1. 完整性 (Completeness)

✅ **好的示例**:
- 有明确的目的和适用范围
- 包含所有必要信息（如 API 的请求/响应示例）
- 有版本号和更新日期

❌ **不好的示例**:
- 只有标题没有内容
- 缺少关键信息（如配置参数说明）
- 没有版本信息，不知道是否过时

---

#### 2. 准确性 (Accuracy)

✅ **好的示例**:
- 与代码、系统实际状态一致
- 更新及时，与最新版本同步
- 经过验证（如代码示例可运行）

❌ **不好的示例**:
- API 文档与实际接口不一致
- 配置文件路径错误
- 命令示例无法运行

---

#### 3. 可用性 (Usability)

✅ **好的示例**:
- 结构清晰，有目录和章节
- 使用标题、列表、表格等格式
- 有代码高亮、注释说明
- 包含实际可用的示例

❌ **不好的示例**:
- 全是大段文字，没有格式
- 缺少示例，难以理解
- 没有目录，难以查找

---

#### 4. 可维护性 (Maintainability)

✅ **好的示例**:
- 遵循统一的命名和格式规范
- 有明确的维护者和更新频率
- 模块化，修改一处不影响其他部分

❌ **不好的示例**:
- 命名混乱，格式不统一
- 没有维护者，无人更新
- 重复内容多，修改需要多处同步

---

### 文档质量检查表

在创建或更新文档时，使用此检查表自查：

- [ ] **目的明确**: 在开头说明文档的目的和适用范围
- [ ] **结构清晰**: 有目录、章节、小标题
- [ ] **信息完整**: 包含所有必要信息，没有遗漏
- [ ] **准确性**: 与实际系统一致，示例可运行
- [ ] **有示例**: 包含代码示例、配置示例、使用场景
- [ ] **格式规范**: 遵循 [`naming-conventions.md`](../naming-conventions.md)
- [ ] **版本信息**: 有版本号和更新日期
- [ ] **维护者**: 明确维护者和审查频率
- [ ] **链接有效**: 所有内部/外部链接可访问
- [ ] **语法正确**: Markdown 语法正确，无拼写错误

---

## 常见文档问题

### 问题 1: 文档过时

**现象**:
- 文档更新日期是几个月前
- 内容与实际系统不一致
- 团队成员反馈"文档不对"

**原因**:
- 代码变更后未同步更新文档
- 缺少文档更新提醒机制
- 文档更新不在 Definition of Done 中

**解决方案**:
1. **在 DoD 中加入文档更新**
   - PR Checklist: "是否更新了相关文档？"
   - Code Review: 检查文档更新

2. **定期审查文档**
   - 每月运行 [`master-checklist-report`](../master-checklist-report-v3.md)
   - 在 Retrospective 中讨论文档问题

3. **标记过时内容**
   - 发现过时内容立即添加注释: `<!-- TODO: 更新 -->`
   - 在下个 Sprint 安排更新任务

---

### 问题 2: 文档难找

**现象**:
- 团队成员不知道某个文档是否存在
- 花很多时间搜索文档
- 频繁询问"XX 文档在哪里？"

**原因**:
- 文档散落在多个位置
- 没有统一的索引或导航
- 文件命名不规范

**解决方案**:
1. **使用文档索引**
   - [`README.md`](../README.md) - 主导航
   - [`scrum/sprint-document-index.md`](./sprint-document-index.md) - 按阶段索引
   - [`scrum/role-document-map.md`](./role-document-map.md) - 按角色索引

2. **统一命名规范**
   - 遵循 [`naming-conventions.md`](../naming-conventions.md)
   - 使用 kebab-case 命名文件

3. **建立书签**
   - 团队成员收藏常用文档
   - 在 Onboarding 时提供文档清单

---

### 问题 3: 文档质量低

**现象**:
- 文档只有标题没有内容
- 缺少示例，难以理解
- 格式混乱，难以阅读

**原因**:
- 赶时间，草草完成
- 缺少文档规范和模板
- 没有 Review 机制

**解决方案**:
1. **使用文档模板**
   - 为不同类型的文档提供模板
   - 模板包含必填章节

2. **文档 Review**
   - 重要文档需要 Review（如 Tech Lead 审查 ADR）
   - 在 PR 中包含文档变更，一起审查

3. **提供写作指南**
   - [`scrum/documentation-best-practices.md`](./documentation-best-practices.md)
   - 在团队会议中分享好文档案例

---

### 问题 4: 文档冗余

**现象**:
- 多个文档包含相同信息
- 更新时需要修改多处
- 信息不一致

**原因**:
- 缺少文档规划
- 复制粘贴内容而不是引用
- 文档合并或重构不及时

**解决方案**:
1. **单一信息源原则**
   - 每个信息只在一个地方维护
   - 其他地方通过链接引用

2. **定期文档清理**
   - 每季度检查冗余文档
   - 归档过时文档到 `archived/`

3. **使用文档架构图**
   - [`document-flow.md`](../document-flow.md) 展示文档关系
   - 避免重复创建相似文档

---

## 附录

### 快速参考

| 我需要... | 使用这个清单 |
|----------|------------|
| **准备 Sprint Planning** | [Sprint 开始前检查](#sprint-开始前检查) |
| **Sprint 中期检查** | [Sprint 进行中检查](#sprint-进行中检查) |
| **准备 Sprint Review/Retro** | [Sprint 结束检查](#sprint-结束检查) |
| **评估文档质量** | [文档质量标准](#文档质量标准) |
| **解决文档问题** | [常见文档问题](#常见文档问题) |

---

### 相关文档

- [Sprint 阶段文档索引](./sprint-document-index.md) - 各阶段所需文档
- [Scrum Master 指南](./sm-guide.md) - SM 工作手册
- [文档最佳实践](./documentation-best-practices.md) - 文档维护原则
- [文档度量](./documentation-metrics.md) - 文档使用和质量度量

---

**维护者**: Scrum Master  
**审查频率**: 每 Sprint  
**反馈**: 在 Sprint Retrospective 中收集使用反馈，持续改进检查清单

