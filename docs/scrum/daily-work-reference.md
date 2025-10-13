# 每日工作文档参考

> **HermesFlow 量化交易平台 - 每日工作快速参考清单**  
> **版本**: v1.0 | **更新日期**: 2025-01-13

---

## 📋 目录

1. [使用说明](#使用说明)
2. [Scrum Master 每日清单](#scrum-master-每日清单)
3. [Product Owner 每日清单](#product-owner-每日清单)
4. [开发者每日清单](#开发者每日清单)
5. [QA 工程师每日清单](#qa-工程师每日清单)
6. [DevOps 工程师每日清单](#devops-工程师每日清单)
7. [每日协作提示](#每日协作提示)

---

## 使用说明

### 目的

本文档为团队成员提供 **每日工作的文档参考清单**，帮助您：

- ✅ 快速找到每日工作所需的文档
- ✅ 不遗漏关键的检查项
- ✅ 提高工作效率和质量

### 使用方法

1. **早晨第一件事**: 打开本文档，查看您角色的每日清单
2. **逐项完成**: 按照时间顺序完成清单项
3. **标记完成**: 在您的工作笔记中标记完成情况
4. **按需查阅**: 遇到问题时，点击文档链接快速查阅

---

## Scrum Master 每日清单

### 🌅 早晨 (9:00-10:00)

#### 1. 检查项目状态 (10分钟)

- [ ] **查看项目进度**
  - 文档: [`progress.md`](../progress.md)
  - 目的: 了解昨天的进展和今天的计划
  - 关注: Sprint 燃尽图、剩余工作量

- [ ] **检查任务看板**
  - 工具: Jira / GitHub Projects
  - 目的: 确认任务状态变更
  - 关注: 阻塞任务、逾期任务

- [ ] **查看 CI/CD 状态**
  - 文档: [`testing/ci-cd-integration.md`](../testing/ci-cd-integration.md)
  - 目的: 确认构建和测试状态
  - 关注: 失败的构建、测试覆盖率下降

#### 2. 准备每日站会 (15分钟)

- [ ] **复习站会流程**
  - 文档: [`scrum/sm-guide.md#每日站会`](./sm-guide.md)
  - 工具: 计时器、记录本

- [ ] **准备讨论要点**
  - 昨天的障碍是否已移除？
  - 今天是否有新的风险？
  - 团队协作是否需要改进？

- [ ] **检查上次行动项**
  - 文档: 上次站会记录
  - 目的: 确认行动项完成情况

---

### 🕐 每日站会 (10:00-10:15)

#### 会议中 (15分钟)

- [ ] **引导团队回答三个问题**
  - 昨天完成了什么？
  - 今天计划做什么？
  - 遇到什么障碍？

- [ ] **记录关键信息**
  - 文档: 站会记录模板
  - 记录: 障碍、依赖、风险

- [ ] **时间控制**
  - 工具: 计时器
  - 目标: 严格控制在 15 分钟内

- [ ] **识别需要跟进的问题**
  - 标记: 需要会后讨论的话题
  - 行动: 安排后续会议

---

### 🌆 下午 (14:00-17:00)

#### 3. 移除障碍 (持续)

- [ ] **跟进站会中提到的障碍**
  - 文档: [`operations/troubleshooting.md`](../operations/troubleshooting.md)
  - 行动: 联系相关人员、协调资源

- [ ] **检查 Code Review 进度**
  - 文档: [`development/code-review-checklist.md`](../development/code-review-checklist.md)
  - 目的: 确保 PR 及时审查

- [ ] **更新文档（如有变更）**
  - 文档: 相关项目文档
  - 原则: 发现过时内容立即标记

#### 4. 团队支持 (按需)

- [ ] **协调跨团队沟通**
  - 场景: 团队遇到外部依赖问题
  - 行动: 联系其他团队、PO

- [ ] **提供 Scrum 指导**
  - 文档: [`scrum/sm-guide.md`](./sm-guide.md)
  - 场景: 团队成员询问流程问题

---

### 🌙 结束前 (17:00-18:00)

#### 5. 每日回顾 (15分钟)

- [ ] **更新 Sprint 燃尽图**
  - 工具: Jira / 表格
  - 数据: 今天完成的 Story Points

- [ ] **更新任务看板**
  - 检查: 任务状态是否更新
  - 提醒: 团队更新任务状态

- [ ] **记录今日要点**
  - 文档: 每日日志
  - 内容: 进展、障碍、决策

#### 6. 准备明天 (10分钟)

- [ ] **检查明天的日程**
  - 会议: 是否有特殊会议？
  - 任务: 是否有需要特别关注的任务？

- [ ] **准备明天站会的议题**
  - 思考: 今天的问题明天如何跟进？
  - 准备: 需要团队讨论的话题

---

## Product Owner 每日清单

### 🌅 早晨 (9:00-10:00)

#### 1. 检查产品状态 (15分钟)

- [ ] **查看项目进度**
  - 文档: [`progress.md`](../progress.md)
  - 关注: 功能完成度、里程碑进度

- [ ] **检查用户反馈**
  - 来源: 用户调研、客服、社区
  - 行动: 标记重要反馈

- [ ] **查看竞品动态**
  - 文档: [`analysis/market-analysis-and-gap-assessment.md`](../analysis/market-analysis-and-gap-assessment.md)
  - 目的: 了解市场变化

---

### 🕐 参与每日站会 (10:00-10:15)

- [ ] **聆听团队进展**
  - 关注: 功能实现是否符合预期
  - 关注: 是否需要澄清需求

- [ ] **准备回答需求问题**
  - 文档: [`prd/prd-hermesflow.md`](../prd/prd-hermesflow.md)

---

### 🌆 下午 (按需)

#### 2. Product Backlog 维护 (30-60分钟)

- [ ] **审查新的 Story**
  - 文档: `prd/user-stories/`
  - 行动: 编写 / 细化用户故事

- [ ] **更新优先级**
  - 文档: [`prd/prd-hermesflow.md#优先级矩阵`](../prd/prd-hermesflow.md)
  - 依据: 用户反馈、商业价值、技术依赖

- [ ] **准备下次 Sprint 的 Backlog**
  - 目标: Top 20 Story 已细化
  - 标准: 验收标准清晰、估算可行

#### 3. 需求澄清 (按需)

- [ ] **回答团队的需求问题**
  - 响应: 及时回复 Slack / 邮件
  - 行动: 必要时更新文档

---

## 开发者每日清单

### 通用清单

#### 🌅 早晨 (9:00-10:00)

##### 1. 准备工作 (15分钟)

- [ ] **拉取最新代码**
  - 命令: `git pull origin main`
  - 检查: 是否有冲突需要解决

- [ ] **查看任务看板**
  - 确认: 今天要做的任务
  - 优先级: 按 Sprint 目标排序

- [ ] **查看 CI/CD 状态**
  - 检查: 构建是否通过
  - 文档: [`testing/ci-cd-integration.md`](../testing/ci-cd-integration.md)

##### 2. 复习相关文档 (10分钟)

- [ ] **查看快速参考**
  - 文档: [`development/quick-reference.md`](../development/quick-reference.md)
  - 目的: 回顾常用命令和流程

- [ ] **查看今日任务的相关文档**
  - API: [`api/api-design.md`](../api/api-design.md)
  - 数据库: [`database/database-design.md`](../database/database-design.md)

---

#### 🕐 参与每日站会 (10:00-10:15)

- [ ] **准备三个问题的答案**
  1. 昨天完成了什么？（具体任务、PR 编号）
  2. 今天计划做什么？（任务、预计完成时间）
  3. 遇到什么障碍？（技术问题、依赖阻塞）

- [ ] **主动提出需要的帮助**
  - 技术问题、Code Review 需求、需求澄清

---

#### 🌆 开发工作 (10:15-17:00)

##### 3. 编码阶段

- [ ] **遵循编码规范**
  - 文档: [`development/coding-standards.md`](../development/coding-standards.md)
  - 自查: 代码风格、命名规范

- [ ] **编写测试用例**
  - 文档: [`testing/test-strategy.md`](../testing/test-strategy.md)
  - 目标: 达到覆盖率要求（Rust≥85%, Java≥80%, Python≥75%）

- [ ] **本地测试**
  - 运行: 单元测试、集成测试
  - 确认: 所有测试通过

##### 4. 提交代码前自查

- [ ] **Code Review 自查清单**
  - 文档: [`development/code-review-checklist.md`](../development/code-review-checklist.md)
  - 检查: 代码质量、测试覆盖、文档更新

- [ ] **提交规范**
  - 格式: `[type] scope: message`
  - 示例: `[feat] data-engine: add Binance WebSocket support`

- [ ] **创建 Pull Request**
  - 描述: 清晰说明改动内容
  - 链接: 关联 Issue / Story

##### 5. Code Review

- [ ] **审查团队成员的 PR**
  - 文档: [`development/code-review-checklist.md`](../development/code-review-checklist.md)
  - 目标: 每天至少审查 1-2 个 PR
  - 反馈: 建设性、具体

---

#### 🌙 结束前 (17:00-18:00)

##### 6. 每日总结 (10分钟)

- [ ] **更新任务状态**
  - 看板: 将任务移动到正确的列
  - 记录: 今天的进度

- [ ] **记录明天的计划**
  - 任务: 明天要做什么
  - 依赖: 是否需要等待他人

- [ ] **标记遇到的问题**
  - 记录: 技术债务、待解决问题
  - 行动: 明天站会提出

---

### Rust 开发者特定清单

#### 📖 每日必读

- [ ] **Rust 开发者指南**
  - 文档: [`development/rust-developer-guide.md`](../development/rust-developer-guide.md)

- [ ] **Rust 编码规范**
  - 文档: [`development/coding-standards.md#rust-规范`](../development/coding-standards.md)

#### 🔧 Rust 特定检查

- [ ] **运行 Clippy**
  ```bash
  cargo clippy --all-targets --all-features -- -D warnings
  ```

- [ ] **格式化代码**
  ```bash
  cargo fmt --all
  ```

- [ ] **检查测试覆盖率**
  ```bash
  cargo tarpaulin --out Html
  ```
  - 目标: ≥85%

- [ ] **性能基准测试（如有变更）**
  - 文档: [`prd/modules/01-data-module.md#性能基准`](../prd/modules/01-data-module.md)

---

### Java 开发者特定清单

#### 📖 每日必读

- [ ] **Java 开发者指南**
  - 文档: [`development/java-developer-guide.md`](../development/java-developer-guide.md)

- [ ] **Java 编码规范**
  - 文档: [`development/coding-standards.md#java-规范`](../development/coding-standards.md)

#### 🔧 Java 特定检查

- [ ] **运行 Checkstyle / SpotBugs**
  ```bash
  mvn checkstyle:check spotbugs:check
  ```

- [ ] **运行单元测试**
  ```bash
  mvn test
  ```
  - 目标: 覆盖率 ≥80%

- [ ] **构建项目**
  ```bash
  mvn clean package
  ```

---

### Python 开发者特定清单

#### 📖 每日必读

- [ ] **Python 开发者指南**
  - 文档: [`development/python-developer-guide.md`](../development/python-developer-guide.md)

- [ ] **Python 编码规范**
  - 文档: [`development/coding-standards.md#python-规范`](../development/coding-standards.md)

#### 🔧 Python 特定检查

- [ ] **运行 Flake8 / Black**
  ```bash
  flake8 .
  black --check .
  ```

- [ ] **类型检查**
  ```bash
  mypy .
  ```

- [ ] **运行测试**
  ```bash
  pytest --cov=. --cov-report=html
  ```
  - 目标: 覆盖率 ≥75%

---

## QA 工程师每日清单

### 🌅 早晨 (9:00-10:00)

#### 1. 检查测试状态 (15分钟)

- [ ] **查看 CI/CD 测试结果**
  - 文档: [`testing/ci-cd-integration.md`](../testing/ci-cd-integration.md)
  - 关注: 失败的测试、覆盖率变化

- [ ] **检查测试环境**
  - 文档: [`testing/test-data-management.md`](../testing/test-data-management.md)
  - 确认: 测试环境可用、测试数据就绪

- [ ] **查看待测试的 PR**
  - 工具: GitHub PR 列表
  - 优先级: 按 Sprint 目标排序

---

### 🕐 参与每日站会 (10:00-10:15)

- [ ] **报告测试进度**
  - 已完成的测试用例数
  - 发现的 Bug 数量和严重程度
  - 今天的测试计划

---

### 🌆 测试工作 (10:15-17:00)

#### 2. 功能测试

- [ ] **执行测试用例**
  - 文档: [`testing/qa-engineer-guide.md`](../testing/qa-engineer-guide.md)
  - 工具: 测试管理工具

- [ ] **探索性测试**
  - 目的: 发现测试用例未覆盖的问题
  - 记录: 新的测试场景

#### 3. 自动化测试

- [ ] **编写自动化测试**
  - 文档: [`testing/test-strategy.md`](../testing/test-strategy.md)
  - 语言: Python (pytest)

- [ ] **维护测试脚本**
  - 更新: 因功能变更需要更新的脚本
  - 重构: 重复的测试代码

#### 4. 安全和性能测试

- [ ] **高风险访问测试（每周）**
  - 文档: [`testing/high-risk-access-testing.md`](../testing/high-risk-access-testing.md)
  - 重点: 多租户隔离、权限验证

- [ ] **性能测试（按需）**
  - 文档: `tests/performance/load_test.js`
  - 工具: k6

#### 5. Bug 管理

- [ ] **记录发现的 Bug**
  - 工具: Jira / GitHub Issues
  - 信息: 复现步骤、截图、日志

- [ ] **跟进 Bug 修复**
  - 验证: 开发修复后的 Bug
  - 回归: 确认修复未引入新问题

---

### 🌙 结束前 (17:00-18:00)

#### 6. 每日总结

- [ ] **更新测试报告**
  - 内容: 测试覆盖率、通过率、Bug 统计
  - 分享: 团队 Slack / 看板

- [ ] **准备明天的测试计划**
  - 任务: 明天要测试的功能
  - 数据: 需要准备的测试数据

---

## DevOps 工程师每日清单

### 🌅 早晨 (9:00-10:00)

#### 1. 检查系统状态 (20分钟)

- [ ] **查看监控面板**
  - 文档: [`operations/monitoring.md`](../operations/monitoring.md)
  - 工具: Grafana Dashboard
  - 关注: 异常指标、告警

- [ ] **检查 CI/CD 状态**
  - 工具: GitHub Actions
  - 关注: 失败的构建、长时间运行的任务

- [ ] **查看日志**
  - 文档: [`operations/troubleshooting.md#日志分析`](../operations/troubleshooting.md)
  - 工具: ELK Stack
  - 关注: Error / Warning 级别日志

---

### 🕐 参与每日站会 (10:00-10:15)

- [ ] **报告系统状态**
  - 服务可用性
  - 资源使用情况
  - 昨天的部署情况

---

### 🌆 运维工作 (10:15-17:00)

#### 2. 日常维护

- [ ] **处理告警**
  - 文档: [`operations/troubleshooting.md`](../operations/troubleshooting.md)
  - 行动: 调查、修复、记录

- [ ] **优化资源使用**
  - 检查: CPU、内存、磁盘使用率
  - 行动: 扩容 / 优化配置

#### 3. CI/CD 维护

- [ ] **优化构建流程**
  - 文档: [`architecture/diagrams/cicd-flow.md`](../architecture/diagrams/cicd-flow.md)
  - 目标: 减少构建时间

- [ ] **更新 GitOps 配置**
  - 文档: [`deployment/gitops-best-practices.md`](../deployment/gitops-best-practices.md)
  - 场景: 新服务部署、配置变更

#### 4. 部署支持

- [ ] **协助团队部署**
  - 文档: [`operations/devops-guide.md`](../operations/devops-guide.md)
  - 行动: 执行部署、验证、回滚（如需）

- [ ] **环境管理**
  - 确认: Dev / Staging / Prod 环境一致性
  - 更新: 环境配置

---

### 🌙 结束前 (17:00-18:00)

#### 5. 每日总结

- [ ] **记录今日变更**
  - 文档: 变更日志
  - 内容: 部署、配置变更、事故

- [ ] **检查备份**
  - 确认: 数据库备份成功
  - 文档: 未来创建的备份恢复方案

---

## 每日协作提示

### 📞 沟通原则

1. **异步优先**: 优先使用 Slack / 文档，减少不必要的会议
2. **及时响应**: 24小时内回复非紧急消息，2小时内回复紧急消息
3. **透明分享**: 主动分享进度、问题、决策

### 🤝 协作工具

| 用途 | 工具 | 文档 |
|------|------|------|
| **任务管理** | Jira / GitHub Projects | - |
| **代码协作** | GitHub PR | [`development/code-review-checklist.md`](../development/code-review-checklist.md) |
| **即时沟通** | Slack / Teams | - |
| **文档协作** | Markdown + Git | [`naming-conventions.md`](../naming-conventions.md) |
| **监控告警** | Grafana / Prometheus | [`operations/monitoring.md`](../operations/monitoring.md) |

---

### 📚 文档更新原则

#### 每日应该更新的文档

| 角色 | 文档 | 频率 |
|------|------|------|
| **Scrum Master** | 站会记录、任务看板 | 每日 |
| **开发者** | 代码、测试、API 文档 | 每次提交 |
| **QA** | 测试报告、Bug 记录 | 每日 |
| **DevOps** | 变更日志、配置文档 | 每次变更 |

#### 发现文档问题时

```
1. 标记问题：在文档中添加注释或创建 Issue
2. 通知团队：在 Slack 中通知相关人员
3. 更新文档：自己能改的立即改，否则指派给维护者
4. 验证更新：确认更新后的文档准确性
```

---

### ⏰ 时间管理提示

#### 避免常见时间陷阱

- ❌ **无计划地开始一天**: 早晨花 10 分钟规划今天的任务
- ❌ **频繁上下文切换**: 使用番茄工作法，专注 25 分钟
- ❌ **过度会议**: 除了每日站会，其他会议要有明确议程
- ❌ **拖延 Code Review**: 每天固定时间审查 PR（如下午 3 点）

#### 推荐的时间分配（开发者）

| 时间段 | 活动 | 占比 |
|--------|------|------|
| 9:00-10:00 | 准备工作、查看文档 | 10% |
| 10:00-10:15 | 每日站会 | 3% |
| 10:15-12:00 | 深度工作（编码） | 22% |
| 14:00-16:00 | 深度工作（编码） | 25% |
| 16:00-17:00 | Code Review、沟通 | 12% |
| 17:00-18:00 | 测试、总结、计划 | 12% |
| 其他 | 休息、非计划工作 | 16% |

---

## 附录

### 快速参考链接

| 场景 | 文档 | 预计时间 |
|------|------|---------|
| **遇到问题** | [故障排查指南](../operations/troubleshooting.md) | 按需 |
| **不懂流程** | [FAQ](../faq.md) | 5分钟 |
| **Code Review** | [Code Review 检查清单](../development/code-review-checklist.md) | 5分钟 |
| **部署** | [快速参考卡片 - 部署](./quick-reference-cards.md) | 3分钟 |
| **编码规范** | [编码规范](../development/coding-standards.md) | 10分钟 |

---

### 相关文档

- [Sprint 阶段文档索引](./sprint-document-index.md) - 按 Sprint 阶段查找文档
- [角色文档地图](./role-document-map.md) - 按角色的学习路径
- [快速参考卡片](./quick-reference-cards.md) - 一页纸快速参考
- [Scrum Master 指南](./sm-guide.md) - 完整的 SM 工作手册

---

**维护者**: Scrum Master + 团队  
**审查频率**: 每 Sprint  
**反馈**: 在 Sprint Retrospective 中收集使用反馈，持续优化清单

