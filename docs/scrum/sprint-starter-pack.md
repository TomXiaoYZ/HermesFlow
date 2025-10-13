# Sprint 启动包

> **确保每个 Sprint 顺利启动的完整指南** | **版本**: v1.0

---

## 📋 目录

1. [Sprint Day 1 清单](#sprint-day-1-清单)
2. [Sprint Planning 准备](#sprint-planning-准备)
3. [团队协议模板](#团队协议模板)
4. [Sprint 目标模板](#sprint-目标模板)
5. [Story 分解模板](#story-分解模板)

---

## Sprint Day 1 清单

### 第一天上午：Sprint Planning 会议（9:00-13:00）

#### 会前检查（9:00）

- [ ] **环境准备**
  - 会议室/Zoom 链接已就绪
  - 白板或 Miro 已打开
  - 投影仪/屏幕共享正常
  - 所有参与者已加入
  
- [ ] **文档准备**
  - [ ] [Product Backlog](../prd/prd-hermesflow.md) 已打开
  - [ ] [项目进度](../progress.md) 已审查
  - [ ] [系统架构图](../architecture/system-architecture.md) 已准备
  - [ ] [上个 Sprint Retrospective](./retrospective-template.md) 行动项已确认
  
- [ ] **工具准备**
  - [ ] 任务看板（GitHub Projects）已打开
  - [ ] Planning Poker 工具已准备
  - [ ] 计时器已设置

#### Part 1: 确定 Sprint 目标（9:00-10:00）

使用 [Sprint 目标模板](#sprint-目标模板)

- [ ] **回顾上个 Sprint**
  - 完成情况：X/Y Story Points
  - 未完成的 Story 及原因
  - 速度趋势分析
  
- [ ] **制定本 Sprint 目标**
  - 一句话描述核心价值
  - 业务价值说明
  - 关键交付物（3-5个）
  - 成功标准（可衡量）
  
- [ ] **团队共识**
  - 所有人理解并同意 Sprint 目标
  - Sprint 目标与公司 OKR 对齐

#### Part 2: 选择 Story（10:00-11:30）

- [ ] **计算团队容量**
  ```
  HermesFlow 容量计算：
  - 团队人数: ___人
  - 工作天数: ___天（扣除节假日）
  - 每日可用时间: 6小时
  - 聚焦因子: 0.8
  
  Sprint 容量 = ___ × ___ × 6 × 0.8 = ___ 小时 = ___ 人日
  ```
  
- [ ] **选择 Story**
  - 从 Product Backlog 顶部开始
  - 确保支持 Sprint 目标
  - 新功能: ~80% 容量
  - 技术债务: ~20% 容量
  
- [ ] **Story 细化**
  - 使用 [Story 分解模板](#story-分解模板)
  - 每个 Story 有明确的验收标准
  - 依赖关系已标记

#### Part 3: 任务分解和估算（11:30-13:00）

- [ ] **任务分解**
  - 每个 Story 分解为 < 8小时的任务
  - 识别技术任务、测试任务、文档任务
  
- [ ] **Planning Poker 估算**
  - 每个人独立估算
  - 讨论差异
  - 达成共识
  
- [ ] **验证容量**
  - 总估算 ≤ 90% 团队容量
  - 各技术栈负载均衡

#### Part 4: 风险和依赖（12:40-13:00）

- [ ] **识别风险**
  - 技术风险（新技术、性能瓶颈）
  - 团队风险（人员休假、技能缺口）
  - 外部风险（第三方 API、依赖团队）
  
- [ ] **制定缓解措施**
  - 每个高风险项有缓解计划
  - 应急计划（Plan B）

#### Sprint Planning 输出

- [ ] **文档已创建**
  - Sprint 目标文档
  - Sprint Backlog（Story 和任务列表）
  - 风险登记表
  - Definition of Done 确认
  
- [ ] **工具已更新**
  - GitHub Projects 看板已更新
  - Sprint 燃尽图已初始化
  - `progress.md` 已更新
  
- [ ] **沟通已完成**
  - 会议纪要已发送
  - Sprint 目标已公示（Slack）

---

### 第一天下午：Sprint 启动活动（14:00-17:00）

#### 1. 团队同步会（14:00-15:00）

**目的**: 确保所有人理解 Sprint 计划

- [ ] **Scrum Master 回顾**
  - Sprint 目标
  - 关键交付物
  - 重要日期（Review、Retro）
  
- [ ] **技术架构同步**
  - 架构师讲解关键架构点
  - 模块间依赖关系
  - API 契约确认
  
- [ ] **测试策略同步**
  - QA 讲解测试重点
  - 测试环境准备
  - 验收标准确认

#### 2. 任务认领（15:00-15:30）

- [ ] **开发者认领任务**
  - 每个人在看板上认领任务
  - 优先认领高优先级和阻塞任务
  - 配对编程任务确定搭档
  
- [ ] **确认第一个任务**
  - 每个人明确今天下午开始什么任务
  - 识别立即可以开始的任务

#### 3. 环境准备（15:30-17:00）

- [ ] **开发环境检查**
  ```bash
  # Rust
  cd modules/data-engine
  cargo build --release
  cargo test
  
  # Java
  cd modules/user-management
  ./mvnw clean install
  
  # Python
  cd modules/strategy-engine
  poetry install
  poetry run pytest
  ```
  
- [ ] **依赖服务检查**
  ```bash
  # 启动本地环境
  docker-compose up -d
  
  # 验证服务
  docker-compose ps
  ```
  
- [ ] **开始第一个任务**
  - 创建功能分支
  - 开始编码！

---

## Sprint Planning 准备

### 会前 2 天（Sprint 开始前）

#### Product Owner 准备

- [ ] **Product Backlog 整理**
  - Top 20 Story 已细化
  - 优先级已排序
  - 验收标准已明确
  
- [ ] **业务价值确认**
  - Sprint 目标草案已准备
  - 关键交付物已明确
  - 客户反馈已整合

#### Scrum Master 准备

- [ ] **技术债务审查**
  - 查看 [`progress.md#技术债务`](../progress.md)
  - 评估技术债务影响
  - 准备技术债务 Story
  
- [ ] **文档准备**
  - 打印/投影系统架构图
  - 准备 ADR 文档
  - 准备模块索引
  
- [ ] **会议准备**
  - 发送会议邀请
  - 预订会议室/准备 Zoom
  - 准备白板/Miro

#### Tech Lead 准备

- [ ] **技术方案审查**
  - 审查复杂 Story 的技术方案
  - 识别技术风险
  - 准备架构讲解
  
- [ ] **依赖检查**
  - 外部依赖确认
  - 基础设施就绪确认
  - API 契约确认

---

## 团队协议模板

### Sprint 团队协议

**Sprint**: Sprint X（日期: YYYY-MM-DD 至 YYYY-MM-DD）  
**团队**: HermesFlow Dev Team

#### 工作时间

- **核心工作时间**: 10:00-17:00（所有人在线）
- **灵活工作时间**: 9:00-10:00, 17:00-18:00
- **每日站会**: 每天 10:00（15分钟）

#### 沟通协议

**同步沟通**:
- 紧急问题: Slack DM 或电话
- 一般问题: Slack `#hermesflow-dev`
- 深入讨论: 预约 Zoom 会议（15-30分钟）

**异步沟通**:
- 非紧急问题: Slack 或邮件
- 代码讨论: GitHub PR Comments
- 文档讨论: PR Review

**响应时间承诺**:
- 紧急问题: < 1小时
- Code Review: < 4小时
- Slack 消息: < 2小时（工作时间）

#### Code Review 协议

- [ ] **提交者责任**
  - PR 大小: < 500 行代码
  - 描述清晰，包含测试截图
  - CI/CD 通过后才请求 Review
  - 及时响应 Review 意见
  
- [ ] **审查者责任**
  - 4小时内提供第一次反馈
  - 提供建设性意见
  - 使用 [代码审查清单](../development/code-review-checklist.md)

#### 会议协议

- [ ] **每日站会**
  - 准时参加（10:00）
  - 提前更新任务状态
  - 站立进行，15分钟严格限时
  - 深入讨论使用 Parking Lot
  
- [ ] **技术讨论**
  - 提前预约，发送议程
  - 准时开始和结束
  - 会后发送会议纪要

#### 质量标准

**Definition of Done**:
- [ ] 代码通过 Code Review
- [ ] 单元测试覆盖率达标（Rust≥85%, Java≥80%, Python≥75%）
- [ ] 集成测试通过（如适用）
- [ ] 文档已更新
- [ ] CI/CD Pipeline 通过
- [ ] 在 Dev 环境验证

**编码规范**:
- 遵循 [编码规范](../development/coding-standards.md)
- Linter 无错误
- Security Scan 无漏洞

#### 技术债务管理

- [ ] 每个 Sprint 20% 容量用于技术债务
- [ ] 技术债务在 `progress.md` 中跟踪
- [ ] 每个 Story 完成后，创建技术债务 Story（如需要）

#### 团队签名

| 姓名 | 角色 | 签名 | 日期 |
|------|------|------|------|
|      | Scrum Master |      |      |
|      | Tech Lead |      |      |
|      | Rust Dev |      |      |
|      | Java Dev |      |      |
|      | Python Dev |      |      |
|      | QA |      |      |
|      | DevOps |      |      |

---

## Sprint 目标模板

```markdown
# Sprint X 目标

**Sprint 周期**: YYYY-MM-DD 至 YYYY-MM-DD（2周）  
**Sprint 主题**: [一个关键词，如 "Alpha因子库" "多租户安全" "性能优化"]

## 🎯 Sprint 目标（一句话）

[用一句话描述本 Sprint 的核心价值和业务目标]

**示例**:
"完成 Alpha 因子库核心功能，支持 10+ 常用技术指标，为策略开发者提供标准化因子计算能力"

## 💼 业务价值

**为什么这个 Sprint 对业务重要？**

- [业务价值点 1 - 对用户的价值]
- [业务价值点 2 - 对业务指标的影响]
- [业务价值点 3 - 竞争优势]

**示例**:
- 为策略开发者提供标准化因子库，减少重复开发
- 提升策略质量和回测效率
- 为后续机器学习特征工程打下基础

## 📦 关键交付物

1. **[交付物 1]**: [简要描述]
   - Story: [STORY-ID1, STORY-ID2]
   
2. **[交付物 2]**: [简要描述]
   - Story: [STORY-ID3, STORY-ID4]
   
3. **[交付物 3]**: [简要描述]
   - Story: [STORY-ID5]

**HermesFlow 示例**:
1. **Alpha 因子库核心引擎**: 实现因子计算框架（Rust）
   - Story: DATA-101, DATA-102
   
2. **10+ 技术指标因子**: MA, EMA, RSI, MACD, Bollinger Bands 等
   - Story: DATA-103, DATA-104, DATA-105
   
3. **Python API 和文档**: 供策略开发者调用
   - Story: DATA-106

## ✅ 成功标准

- [ ] **功能完成度**: 所有 Story 满足 Definition of Done
- [ ] **性能指标**: [具体指标]
- [ ] **质量指标**: [具体指标]
- [ ] **验收通过**: Product Owner 验收通过

**HermesFlow 示例**:
- [ ] 10+ 因子通过单元测试（覆盖率 ≥ 85%）
- [ ] 性能测试达标（吞吐量 > 10万行/秒，P95延迟 < 10ms）
- [ ] Python API 文档完成，包含使用示例
- [ ] 集成测试通过（与策略引擎集成）
- [ ] Product Owner Demo 验收通过

## 📊 Sprint Backlog

**总 Story Points**: ___ points  
**团队容量**: ___ 人日

| Story ID | 标题 | Story Points | 优先级 | 负责人 |
|----------|------|-------------|-------|--------|
| DATA-101 | 因子计算框架 | 8 | P0 | @rust-dev1 |
| DATA-102 | 数据管道优化 | 5 | P1 | @rust-dev2 |
| ...      | ...  | ... | ... | ... |

## ⚠️ 风险和依赖

**高风险项**:
1. [风险描述] - **缓解措施**: [措施]
2. [风险描述] - **缓解措施**: [措施]

**依赖项**:
- [ ] 依赖 A（负责人: ___, 截止日期: ___）
- [ ] 依赖 B（负责人: ___, 截止日期: ___）

## 📅 关键日期

- **Sprint Planning**: YYYY-MM-DD
- **Mid-Sprint Check-in**: YYYY-MM-DD
- **Sprint Review**: YYYY-MM-DD 15:00-17:00
- **Sprint Retrospective**: YYYY-MM-DD 17:00-18:30

---

**创建日期**: YYYY-MM-DD  
**创建人**: Scrum Master  
**批准人**: Product Owner
```

---

## Story 分解模板

### Story 模板

```markdown
### [STORY-ID] Story 标题

**Epic**: [Epic 名称]  
**优先级**: P0 / P1 / P2  
**Story Points**: ___ points  
**负责人**: @username

#### 用户故事

**作为** [用户角色],  
**我想要** [功能描述],  
**以便** [业务价值/目的]

**HermesFlow 示例**:
作为策略开发者，
我想要使用 RSI 因子计算接口，
以便在策略中快速获取 RSI 指标，而无需自己实现计算逻辑

#### 验收标准

- [ ] **AC1**: [可测试、可验证的标准]
- [ ] **AC2**: [可测试、可验证的标准]
- [ ] **AC3**: [可测试、可验证的标准]

**HermesFlow 示例**:
- [ ] 提供 `RSI::calculate(data: &[f64], period: usize) -> Vec<f64>` API
- [ ] 计算结果与标准 RSI 公式一致（误差 < 0.01%）
- [ ] 性能: 10万数据点计算时间 < 10ms（P95）
- [ ] 单元测试覆盖率 ≥ 85%
- [ ] Python FFI 绑定可用，API 文档完成

#### 技术任务分解

- [ ] **任务 1**: [任务描述] - 估算: ___h - 负责人: @username
- [ ] **任务 2**: [任务描述] - 估算: ___h - 负责人: @username
- [ ] **任务 3**: [任务描述] - 估算: ___h - 负责人: @username
- [ ] **任务 4**: [任务描述] - 估算: ___h - 负责人: @username

**HermesFlow 示例**:
- [ ] 设计 RSI 数据结构（Rust）- 2h - @rust-dev1
- [ ] 实现 RSI 计算逻辑（Rust）- 4h - @rust-dev1
- [ ] 编写单元测试（覆盖率 ≥ 85%）- 3h - @rust-dev1
- [ ] 创建 Python FFI 绑定 - 2h - @rust-dev2
- [ ] 编写 Python API 文档和示例 - 1h - @python-dev1
- [ ] 集成测试（与策略引擎）- 2h - @qa
- [ ] 性能测试（k6）- 2h - @qa

**总估算**: 16h

#### 依赖关系

**阻塞**:
- 阻塞 [STORY-ID]: [描述为什么阻塞]

**依赖**:
- 依赖 [STORY-ID]: [描述依赖什么]
- 依赖外部团队: [描述]

#### 测试要求

- **单元测试**: 覆盖率 ≥ ___% （Rust≥85%, Java≥80%, Python≥75%）
- **集成测试**: 需要 / 不需要
  - [ ] 集成测试场景: [描述]
- **性能测试**: 需要 / 不需要
  - [ ] 性能基线: [指标]
- **安全测试**: 需要 / 不需要
  - [ ] 安全测试场景: [描述]

#### 相关文档

- **PRD**: [链接到 PRD 相关章节]
- **架构**: [链接到架构文档]
- **API**: [链接到 API 设计文档]
- **数据库**: [链接到数据库设计]（如适用）

#### 定义完成 (Definition of Done)

- [ ] 代码通过 Code Review（至少 1 人）
- [ ] 符合编码规范，Linter 无错误
- [ ] 单元测试通过，覆盖率达标
- [ ] 集成测试通过（如适用）
- [ ] 性能测试通过（如适用）
- [ ] 安全测试通过（如适用）
- [ ] API 文档更新（如适用）
- [ ] CI/CD Pipeline 通过
- [ ] 在 Dev 环境验证
- [ ] Product Owner 验收通过

---

**创建日期**: YYYY-MM-DD  
**最后更新**: YYYY-MM-DD
```

---

## 📚 相关资源

- [Sprint Planning 清单](./sprint-planning-checklist.md)
- [Scrum Master 完整指南](./sm-guide.md)
- [项目进度跟踪](../progress.md)
- [系统架构文档](../architecture/system-architecture.md)

---

## 🎉 Sprint 启动成功！

完成以上所有清单后，您的 Sprint 就正式启动了！

**接下来**:
- 每天 10:00 参加 [每日站会](./daily-standup-guide.md)
- 持续更新任务看板
- 及时沟通障碍和风险
- 保持 Sprint 目标焦点

**祝 Sprint 顺利！** 🚀

---

**最后更新**: 2025-01-13  
**维护者**: @pm.mdc  
**版本**: v1.0

