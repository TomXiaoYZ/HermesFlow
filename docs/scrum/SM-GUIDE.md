# Scrum Master 完整指南

> **HermesFlow 量化交易平台 - Scrum Master 工作手册**  
> **版本**: v1.0 | **更新日期**: 2025-01-13

---

## 📋 目录

1. [角色职责](#角色职责)
2. [Sprint Planning](#sprint-planning)
3. [每日站会](#每日站会)
4. [Sprint Review](#sprint-review)
5. [Sprint Retrospective](#sprint-retrospective)
6. [文档更新检查清单](#文档更新检查清单)
7. [团队协作工具](#团队协作工具)
8. [度量指标](#度量指标)
9. [常见挑战与解决方案](#常见挑战与解决方案)

---

## 角色职责

### 核心职责

作为 HermesFlow 项目的 Scrum Master，您的主要职责包括：

#### 1. 促进 Scrum 流程

- ✅ 组织和引导所有 Scrum 会议
- ✅ 确保团队遵循 Scrum 原则和实践
- ✅ 移除团队开发障碍
- ✅ 保护团队免受外部干扰

#### 2. 服务团队

- ✅ 帮助团队自组织和跨职能协作
- ✅ 指导团队如何在 Scrum 框架内工作
- ✅ 提升团队效能和质量
- ✅ 促进团队成长和持续改进

#### 3. 服务产品负责人

- ✅ 协助管理 Product Backlog
- ✅ 确保 Product Backlog 清晰可见
- ✅ 促进需求澄清和优先级排序

#### 4. 服务组织

- ✅ 推广 Scrum 在组织中的实施
- ✅ 与其他 Scrum Master 协作
- ✅ 提升组织敏捷成熟度

### HermesFlow 项目特定职责

由于 HermesFlow 是混合技术栈项目（Rust + Java + Python），您还需要：

- 🦀 **协调多技术栈团队**: 确保 Rust、Java、Python 开发者有效协作
- 🔄 **管理跨模块依赖**: 数据引擎、策略引擎、交易引擎之间的集成
- 📊 **监控技术债务**: 定期审查技术债务并优先级排序
- 🔐 **关注安全和合规**: 确保多租户隔离和安全测试完成
- 📈 **性能跟踪**: 监控性能指标（吞吐量、延迟、覆盖率）

---

## Sprint Planning

### 会议时间

- **频率**: 每个 Sprint 开始时（2周 Sprint）
- **时长**: 4小时（最大）
- **参与者**: 开发团队、Product Owner、Scrum Master

### 会前准备（Sprint 开始前 2 天）

#### 1. 审查 Product Backlog

```bash
# 检查清单
- [ ] Product Backlog 已按优先级排序
- [ ] Top 20 Story 已细化并准备就绪
- [ ] 所有 Story 都有明确的验收标准
- [ ] 依赖关系已识别并标记
```

**使用文档**:
- [PRD 主文档](../prd/PRD-HermesFlow.md) - 了解功能需求
- [项目进度](../progress.md) - 查看当前状态

#### 2. 确认优先级

与 Product Owner 确认：
- ✅ Q1 2025 路线图目标
- ✅ 本 Sprint 的业务价值重点
- ✅ 客户反馈和市场变化

#### 3. 检查技术债务

```bash
# 技术债务审查
- [ ] 查看 progress.md#技术债务 部分
- [ ] 评估技术债务影响
- [ ] 决定本 Sprint 是否处理技术债务（建议 20% 容量）
```

#### 4. 准备架构图

```bash
# 架构文档准备
- [ ] 打印或投影系统架构图
- [ ] 准备相关模块的详细设计
- [ ] 准备 ADR 文档（如有新的架构决策）
```

**使用文档**:
- [系统架构文档](../architecture/system-architecture.md)
- [ADR 文档](../architecture/decisions/)

---

### Sprint Planning 会议流程

#### Part 1: 确定 Sprint 目标（1小时）

**目标**: 回答 "我们为什么要做这个 Sprint？"

```markdown
Sprint 目标模板：

**Sprint X 目标**: [一句话描述本 Sprint 的核心价值]

**业务价值**:
- [为什么这个 Sprint 对业务重要？]

**关键交付物**:
1. [交付物 1]
2. [交付物 2]
3. [交付物 3]

**成功标准**:
- [ ] [可衡量的标准 1]
- [ ] [可衡量的标准 2]
```

**示例**:
```markdown
**Sprint 12 目标**: 完成 Alpha 因子库核心功能，支持 10+ 常用因子

**业务价值**:
- 为策略开发者提供标准化因子库
- 减少重复开发，提升策略质量

**关键交付物**:
1. 实现 10+ 技术指标因子（MA, EMA, RSI, MACD 等）
2. 实现因子计算引擎（Rust 实现，性能 > 10万行/秒）
3. 提供 Python API 供策略调用

**成功标准**:
- [ ] 10+ 因子通过单元测试（覆盖率 ≥ 85%）
- [ ] 性能测试达标（10万行/秒）
- [ ] Python API 文档完成
- [ ] 集成测试通过
```

#### Part 2: 选择 Story（1.5小时）

**目标**: 回答 "我们在这个 Sprint 做什么？"

##### 2.1 团队容量估算

```bash
# 容量计算公式
Sprint 容量 = 团队人数 × 工作天数 × 每日可用时间 × 聚焦因子

示例：
- 团队人数: 6人（2 Rust, 2 Java, 2 Python）
- 工作天数: 10天（2周 Sprint，除去节假日）
- 每日可用时间: 6小时（除去会议、邮件等）
- 聚焦因子: 0.8（考虑中断和意外）

Sprint 容量 = 6 × 10 × 6 × 0.8 = 288 小时 = 36 人日
```

##### 2.2 Story 选择原则

1. **优先级驱动**: 从 Product Backlog 顶部开始
2. **Sprint 目标对齐**: 所有 Story 必须支持 Sprint 目标
3. **容量匹配**: 总估算不超过团队容量
4. **技术债务平衡**: 20% 容量用于技术债务
5. **风险管理**: 识别高风险 Story，留有缓冲

##### 2.3 Story 细化

对于每个选中的 Story，确保：

```markdown
**Story 模板**:

### [STORY-ID] Story 标题

**描述**: 
作为 [角色]，我想要 [功能]，以便 [业务价值]

**验收标准**:
- [ ] 标准 1
- [ ] 标准 2
- [ ] 标准 3

**技术任务**:
- [ ] 任务 1（估算: Xh）
- [ ] 任务 2（估算: Xh）
- [ ] 任务 3（估算: Xh）

**依赖关系**:
- 依赖 STORY-XXX（已完成/进行中/阻塞）

**测试要求**:
- 单元测试覆盖率: ≥ X%
- 集成测试: 需要/不需要
- 性能测试: 需要/不需要

**相关文档**:
- PRD: [链接]
- 架构: [链接]
- API: [链接]
```

#### Part 3: 任务分解和估算（1.5小时）

**目标**: 回答 "我们如何完成这些 Story？"

##### 3.1 任务分解

对于每个 Story，团队协作分解为小任务（< 8小时）：

```markdown
示例 Story: 实现 RSI 因子计算

**技术任务分解**:
1. [ ] 设计 RSI 数据结构（Rust）- 2h
2. [ ] 实现 RSI 计算逻辑（Rust）- 4h
3. [ ] 编写单元测试（覆盖率 ≥ 85%）- 3h
4. [ ] 创建 Python FFI 绑定 - 2h
5. [ ] 编写 Python API 文档 - 1h
6. [ ] 集成测试 - 2h
7. [ ] 性能测试（k6）- 2h
**总计**: 16h
```

##### 3.2 估算技术

使用 **Planning Poker** 进行估算：

1. **每个人独立估算**（以小时或故事点）
2. **同时亮牌**
3. **讨论差异**（最高和最低估算者说明理由）
4. **重新估算**（直到达成共识）

**HermesFlow 估算参考**:

| 任务类型 | 典型估算（小时） |
|---------|----------------|
| 简单 CRUD API（Java） | 2-4h |
| 数据采集连接器（Rust） | 4-8h |
| 量化因子实现（Rust） | 4-6h |
| 策略回测模块（Python） | 6-12h |
| 集成测试 | 2-4h |
| 性能优化 | 4-8h |
| 文档编写 | 1-2h |

#### Part 4: 识别风险和依赖（30分钟）

```markdown
**风险识别清单**:

**技术风险**:
- [ ] 新技术/库的学习曲线
- [ ] 性能瓶颈（需要 POC）
- [ ] 第三方 API 限制（Binance 限流等）
- [ ] 跨语言集成复杂度（Rust FFI）

**团队风险**:
- [ ] 关键人员休假
- [ ] 技能缺口（需要培训）
- [ ] 跨团队依赖（等待其他团队）

**依赖关系**:
- [ ] Story A 依赖 Story B
- [ ] 等待外部团队交付
- [ ] 基础设施未就绪（Kubernetes 集群等）

**缓解措施**:
- [针对每个风险的缓解计划]
```

#### Part 5: 确定 Definition of Done（30分钟）

确保团队对 "Done" 有共识：

```markdown
**HermesFlow Definition of Done**:

**代码层面**:
- [ ] 代码通过 Code Review（至少 1 人审查）
- [ ] 符合编码规范（Rust/Java/Python）
- [ ] 无 Linter 错误
- [ ] 无 Security Scan 漏洞（Trivy）

**测试层面**:
- [ ] 单元测试通过（覆盖率达标：Rust≥85%, Java≥80%, Python≥75%）
- [ ] 集成测试通过（如适用）
- [ ] 性能测试通过（如适用）
- [ ] 安全测试通过（多租户隔离等）

**文档层面**:
- [ ] API 文档更新（如有新 API）
- [ ] README 更新（如有配置变更）
- [ ] 变更日志更新

**部署层面**:
- [ ] CI/CD Pipeline 通过（GitHub Actions）
- [ ] Docker 镜像构建成功
- [ ] Helm Chart 更新（如有变更）
- [ ] 在 Dev 环境验证

**验收层面**:
- [ ] 所有验收标准满足
- [ ] Product Owner 验收通过
```

---

### Sprint Planning 输出

会议结束时，您应该有：

1. ✅ **Sprint 目标**（清晰、可衡量）
2. ✅ **Sprint Backlog**（选中的 Story 列表）
3. ✅ **任务分解**（所有任务 < 8小时）
4. ✅ **估算**（总估算 ≤ 团队容量）
5. ✅ **风险登记表**（识别的风险和缓解措施）
6. ✅ **Definition of Done**（团队共识）

**使用清单**:
👉 [Sprint Planning 清单](./SPRINT-PLANNING-CHECKLIST.md)

---

## 每日站会

### 会议设置

- **时间**: 每天上午 10:00（固定时间）
- **时长**: 15 分钟（严格）
- **地点**: 团队工作区 / Zoom 会议室
- **参与者**: 开发团队（必须）、Scrum Master（必须）、Product Owner（可选）

### 会议目标

**回答三个问题**:
1. 我昨天做了什么来帮助团队达成 Sprint 目标？
2. 我今天计划做什么来帮助团队达成 Sprint 目标？
3. 我遇到什么障碍？

### 会议流程

#### 1. 准备（会前 5 分钟）

```bash
# Scrum Master 准备清单
- [ ] 打开任务看板（Jira/Trello/GitHub Projects）
- [ ] 查看昨天的进展
- [ ] 检查 Sprint 燃尽图
- [ ] 准备计时器（15分钟）
```

#### 2. 轮流发言（10-12 分钟）

**每个人发言模板**（每人 1-2 分钟）:

```markdown
**[姓名] - [角色]**

✅ **昨天完成**:
- 完成了 TASK-123（RSI 因子实现）
- Code Review 了 PR#45

⏭️ **今天计划**:
- 编写 RSI 单元测试
- 开始 MACD 因子实现

🚫 **障碍**:
- 等待 Data Service API 文档（阻塞 TASK-125）
```

#### 3. 可视化进展（2 分钟）

Scrum Master 快速更新：
- 📊 **Sprint 燃尽图**: 当前进度 vs 理想进度
- 🎯 **Sprint 目标风险**: 绿色/黄色/红色
- 🔴 **阻塞的任务数量**: X 个

#### 4. 快速同步（1 分钟）

- 📅 **今日关键活动**: Code Review 在 14:00，部署窗口在 16:00
- 📢 **重要通知**: 明天客户演示，请准备 Demo 环境

### 站会后活动

**Parking Lot 讨论**（15 分钟，可选）:

如果站会中有需要深入讨论的话题，记录到 "Parking Lot"，站会后单独讨论。

```markdown
**Parking Lot 话题**:
1. RSI 因子性能优化方案（参与者：Rust 团队）
2. 多租户测试策略讨论（参与者：QA + Java 团队）
```

### Scrum Master 注意事项

❌ **避免**:
- 不要让站会变成状态汇报会（向 SM 汇报）
- 不要深入讨论技术细节（使用 Parking Lot）
- 不要超过 15 分钟
- 不要让人坐下（保持站立，提升效率）

✅ **促进**:
- 鼓励团队成员之间对话
- 识别和移除障碍
- 保持节奏和焦点
- 记录跟进事项

### 障碍移除

对于站会中提出的障碍，Scrum Master 应立即行动：

| 障碍类型 | 处理方式 | 时间目标 |
|---------|---------|---------|
| 技术问题 | 组织技术讨论，邀请相关专家 | 当天 |
| 依赖其他团队 | 联系相关团队，推动进展 | 24小时 |
| 工具/环境问题 | 联系 DevOps，解决环境问题 | 4小时 |
| 需求不清晰 | 组织 Product Owner 和团队澄清会 | 当天 |
| 个人技能缺口 | 安排配对编程或培训 | 本 Sprint |

---

## Sprint Review

### 会议设置

- **时间**: Sprint 最后一天下午
- **时长**: 2 小时（2周 Sprint）
- **参与者**: 开发团队、Product Owner、Scrum Master、利益相关者

### 会议目标

- 展示本 Sprint 完成的工作（Demo）
- 获取反馈
- 更新 Product Backlog

### 会前准备（Review 前 1 天）

```bash
# Scrum Master 准备清单
- [ ] 确认 Demo 环境就绪
- [ ] 准备 Sprint 总结 PPT
- [ ] 邀请利益相关者
- [ ] 准备 Demo 脚本
- [ ] 测试 Demo 流程（干跑）
```

**Demo 环境检查**:
```bash
# 确保 Demo 环境稳定
- [ ] Dev 环境所有服务运行正常
- [ ] 测试数据已准备
- [ ] 网络连接稳定（如远程 Demo）
- [ ] 投影/屏幕共享正常
```

### 会议流程

#### 1. 欢迎和 Sprint 概述（10 分钟）

```markdown
**Sprint X Review - [日期]**

**Sprint 目标**: [重申 Sprint 目标]

**参与者**: [列出参与者]

**议程**:
1. Sprint 概述（10分钟）
2. Demo 完成的功能（60分钟）
3. 未完成的工作和原因（10分钟）
4. 讨论和反馈（30分钟）
5. Product Backlog 更新（10分钟）
```

**数据展示**:
- Sprint 目标达成情况: X% 完成
- 完成的 Story 数量: X / Y
- Story Points 完成: X / Y
- 速度: X points/sprint（最近 3 个 Sprint 平均）

#### 2. Demo 完成的功能（60 分钟）

**Demo 原则**:
- ✅ 只 Demo 满足 Definition of Done 的功能
- ✅ 在真实环境（或接近真实）中 Demo
- ✅ 展示端到端的用户流程
- ✅ 让开发者亲自 Demo（不是 SM 或 PO）

**HermesFlow Demo 脚本示例**:

```markdown
### Demo 1: Alpha 因子库（15分钟）

**主讲人**: [Rust 开发者]

**场景**: 策略开发者使用 Alpha 因子库计算技术指标

**步骤**:
1. 展示 Python API 调用（Jupyter Notebook）
   ```python
   from hermesflow.factors import RSI, MACD
   
   # 计算 RSI
   rsi = RSI(period=14)
   result = rsi.calculate(price_data)
   ```

2. 展示计算结果和可视化

3. 展示性能指标
   - 处理速度: 10万行/秒
   - 延迟: P95 < 10ms

4. 展示单元测试覆盖率: 87%

**预期反馈点**:
- 因子计算准确性
- API 易用性
- 性能是否满足需求
```

#### 3. 未完成的工作（10 分钟）

诚实透明地讨论：

```markdown
**未完成的 Story**:
- [STORY-ID] Story 标题

**原因**:
- [具体原因，如依赖阻塞、技术难度被低估等]

**处理计划**:
- 回到 Product Backlog，重新排优先级
- 下 Sprint 继续（如仍高优先级）
```

#### 4. 讨论和反馈（30 分钟）

引导利益相关者反馈：

**引导问题**:
- 这些功能是否满足您的期望？
- 有哪些地方可以改进？
- 您有什么新的需求或想法？
- 市场/客户反馈如何？

**记录反馈**（Scrum Master 职责）:
```markdown
**反馈记录** - Sprint X Review

| ID | 反馈内容 | 提出人 | 优先级 | 行动计划 |
|----|---------|--------|--------|---------|
| FB-01 | RSI API 参数说明不够清晰 | 客户A | P1 | 增强 API 文档 |
| FB-02 | 希望增加 Bollinger Bands 因子 | PO | P2 | 添加到 Backlog |
```

#### 5. Product Backlog 更新（10 分钟）

Product Owner 基于反馈更新 Backlog：
- 添加新的 Story
- 调整优先级
- 细化下个 Sprint 的候选 Story

---

## Sprint Retrospective

### 会议设置

- **时间**: Sprint Review 之后（或第二天上午）
- **时长**: 1.5 小时（2周 Sprint）
- **参与者**: 开发团队、Scrum Master（Product Owner 可选）

### 会议目标

- 反思 Sprint 的工作方式
- 识别改进机会
- 制定行动计划

### 会议原则

- 🤝 **安全环境**: 诚实、尊重、不指责
- 📊 **基于事实**: 使用数据而非情绪
- 💡 **聚焦改进**: 关注未来而非过去
- 🎯 **可执行**: 制定具体可行的行动计划

### 会议流程

#### 1. 设定氛围（5 分钟）

**开场活动**（选一个）:
- 一词总结: 每人用一个词描述本 Sprint 的感受
- 天气图: 画一个天气符号（晴天/阴天/雨天）表示心情
- 快乐/悲伤: 分享一件开心的事和一件难过的事

#### 2. 收集数据（20 分钟）

**使用 "Start, Stop, Continue" 方法**:

```markdown
### Start（开始做）
- [我们应该开始做什么？]

### Stop（停止做）
- [我们应该停止做什么？]

### Continue（继续做）
- [我们应该继续做什么？]
```

**工具**: 白板、便利贴、或在线工具（Miro, Mural）

**HermesFlow 团队数据收集示例**:

```markdown
### Start
- 每周一次的架构同步会（Rust/Java/Python 团队）
- 配对编程（跨语言学习）
- 性能测试自动化

### Stop
- 会议太多（每天 3+ 小时）
- Code Review 延迟（24小时+）
- 技术债务积累

### Continue
- 每日站会（高效）
- 自动化测试（覆盖率提升）
- 文档更新（及时）
```

**使用数据**:
- Sprint 燃尽图
- 速度趋势（最近 5 个 Sprint）
- 缺陷趋势
- 测试覆盖率
- 部署频率

#### 3. 产生洞察（30 分钟）

**分析收集的数据**，识别模式和根因：

**5 Whys 分析法**:
```markdown
**问题**: Code Review 延迟（平均 30 小时）

1. Why? Reviewer 没有及时响应
2. Why? Reviewer 任务太多，没时间
3. Why? 每个人都在忙自己的任务
4. Why? Code Review 不在 Sprint 计划中
5. Why? 我们没有将 Code Review 视为任务

**根因**: Code Review 时间没有被计入 Sprint 容量
```

**分组讨论**:
- 将相关的反馈分组
- 投票选出最重要的话题（每人 3 票）
- 深入讨论 Top 3 话题

#### 4. 决定行动（30 分钟）

**为每个改进点制定行动计划**:

```markdown
**改进行动模板**:

### 改进 1: 减少 Code Review 延迟

**目标**: Code Review 在 4 小时内完成

**行动计划**:
1. **行动**: 每个 Sprint 预留 10% 容量用于 Code Review
   - **负责人**: Scrum Master
   - **截止日期**: 下个 Sprint Planning
   
2. **行动**: 设置 GitHub 通知和提醒
   - **负责人**: DevOps Lead
   - **截止日期**: 本周五
   
3. **行动**: 引入 "Code Review" 时间块（每天 14:00-15:00）
   - **负责人**: Scrum Master
   - **截止日期**: 下周一开始

**成功标准**:
- Code Review 平均时间 < 4小时
- 无 PR 积压 > 24小时

**验证方式**:
- GitHub Insights 统计
- 下次 Retrospective 回顾
```

**HermesFlow 改进行动示例**:

| 改进点 | 行动 | 负责人 | 截止日期 | 成功标准 |
|--------|------|--------|---------|---------|
| 减少会议时间 | 合并 Rust/Java/Python 同步会为一个 | SM | 下周 | 会议时间 < 2h/天 |
| 提升测试覆盖率 | 引入测试覆盖率门禁（Rust≥85%） | Tech Lead | 本周 | 新 PR 不低于目标 |
| 加速部署 | 优化 Docker 构建（多阶段构建） | DevOps | 下 Sprint | 构建时间 < 5min |

#### 5. 回顾上次行动（10 分钟）

**检查上次 Retrospective 的行动是否完成**:

```markdown
**上次 Retrospective 行动回顾**:

1. ✅ **行动**: 引入 Rust 编码规范
   - **状态**: 已完成
   - **效果**: Linter 错误减少 50%
   
2. ⏱️ **行动**: 搭建 Prometheus 监控
   - **状态**: 进行中（70% 完成）
   - **阻塞**: 等待 Azure AKS 集群
   - **下一步**: 下周完成
   
3. ❌ **行动**: 每周技术分享
   - **状态**: 未开始
   - **原因**: 团队时间不足
   - **决策**: 改为每两周一次
```

#### 6. 总结和关闭（5 分钟）

**Retrospective 总结**:
```markdown
**Sprint X Retrospective 总结**

**参与者**: [列表]

**收集到的反馈**: X 条

**Top 3 改进点**:
1. 减少 Code Review 延迟
2. 提升测试覆盖率
3. 优化 CI/CD 流程

**行动计划**: [共 X 个行动]

**下次检查**: Sprint X+1 Retrospective
```

**反馈 Retrospective 本身**（Plus/Delta）:
- **Plus**: 这次 Retrospective 什么做得好？
- **Delta**: 下次 Retrospective 如何改进？

---

### Retrospective 技巧

#### 变换形式（避免疲劳）

不同的 Sprint 使用不同的 Retrospective 格式：

| Sprint | 格式 | 适用场景 |
|--------|------|---------|
| Sprint 1-2 | Start/Stop/Continue | 新团队，建立基础 |
| Sprint 3-4 | Mad/Sad/Glad | 关注情绪和氛围 |
| Sprint 5-6 | 4L (Liked/Learned/Lacked/Longed for) | 深入反思 |
| Sprint 7-8 | Sailboat（帆船） | 可视化目标和障碍 |
| Sprint 9-10 | Timeline（时间线） | 回顾关键事件 |

#### 鼓励参与

- 🤐 **沉默的声音**: 使用匿名便利贴，让内向的人也能表达
- 🎲 **轮流发言**: 使用随机顺序，避免总是同样的人先说
- ⏰ **时间盒**: 每个环节严格限时，保持节奏

#### 数据驱动

展示数据，而不是凭感觉：

```bash
# 准备数据
- Sprint 燃尽图
- 速度趋势（velocity chart）
- 累积流图（cumulative flow diagram）
- 缺陷趋势
- 测试覆盖率趋势
- 部署频率
- 平均前置时间（lead time）
```

---

## 文档更新检查清单

作为 Scrum Master，您需要确保文档及时更新：

### 每 Sprint 开始

```bash
- [ ] 更新 progress.md（当前 Sprint 目标、待办事项）
- [ ] 更新项目看板（Jira/GitHub Projects）
- [ ] 审查 PRD 是否有变更
- [ ] 检查 ADR 是否需要新增
```

### 每 Sprint 结束

```bash
- [ ] 更新 progress.md（完成的里程碑、当前阶段）
- [ ] 更新技术债务列表
- [ ] 更新关键指标（速度、质量、部署频率）
- [ ] 归档 Sprint 总结（Wiki/Confluence）
```

### 每月

```bash
- [ ] 运行文档主检查清单（Master Checklist）
- [ ] 更新 FAQ（基于团队常见问题）
- [ ] 审查和更新 README
- [ ] 检查文档链接有效性
```

**使用文档**:
- [项目进度](../progress.md)
- [文档主检查清单](../MASTER-CHECKLIST-REPORT-V2.md)

---

## 团队协作工具

### 推荐工具栈

| 用途 | 工具 | HermesFlow 使用 |
|------|------|----------------|
| 任务管理 | Jira / GitHub Projects | GitHub Projects |
| 文档协作 | Confluence / Notion | Markdown in Git |
| 视频会议 | Zoom / Teams | [根据团队选择] |
| 即时通讯 | Slack / Teams | Slack (#hermesflow-dev) |
| 白板 | Miro / Mural | Miro |
| 代码仓库 | GitHub / GitLab | GitHub |
| CI/CD | GitHub Actions / Jenkins | GitHub Actions |
| 监控 | Prometheus + Grafana | Prometheus + Grafana |

### 任务看板配置

**GitHub Projects 推荐列**:

```
Backlog → Ready → In Progress → In Review → Testing → Done
```

**列定义**:
- **Backlog**: Product Backlog（所有未来的 Story）
- **Ready**: Sprint Backlog（本 Sprint 选中的 Story）
- **In Progress**: 正在开发
- **In Review**: Code Review 中
- **Testing**: 测试中（QA 验证）
- **Done**: 满足 Definition of Done

**泳道**（Swimlanes）:
- Rust 团队（数据引擎）
- Java 团队（交易/用户/风控）
- Python 团队（策略引擎）
- DevOps 团队
- 阻塞/紧急

---

## 度量指标

### 关键指标

#### 1. 速度（Velocity）

**定义**: 每个 Sprint 完成的 Story Points

**计算**:
```
速度 = 本 Sprint 完成的 Story Points 总和
平均速度 = 最近 3 个 Sprint 的平均值
```

**HermesFlow 目标**: 速度稳定在 ±15% 范围内

**可视化**: 速度趋势图

#### 2. Sprint 燃尽图（Sprint Burndown）

**定义**: 每天剩余工作量

**HermesFlow 使用**: 
- X 轴：Sprint 天数（1-10 天）
- Y 轴：剩余 Story Points
- 理想线 vs 实际线

**解读**:
- 实际线高于理想线 → 进度落后，需要调整
- 实际线平坦 → 任务被阻塞，需要移除障碍
- 实际线低于理想线 → 进度良好

#### 3. 累积流图（Cumulative Flow Diagram）

**显示**: 每个阶段的任务数量随时间变化

**解读**:
- 流畅 → 健康
- 瓶颈（某列堆积）→ 需要优化

#### 4. 前置时间（Lead Time）

**定义**: 从 Story 创建到完成的时间

**HermesFlow 目标**: 平均前置时间 < 5 天

#### 5. 缺陷密度

**定义**: 每个 Story 的平均缺陷数

**HermesFlow 目标**: < 0.5 缺陷/Story

#### 6. 测试覆盖率

**目标**:
- Rust: ≥ 85%
- Java: ≥ 80%
- Python: ≥ 75%

**跟踪**: 每个 Sprint 在 CI/CD 中自动报告

#### 7. 部署频率

**HermesFlow 目标**: 每周至少 1 次部署到 Dev 环境

#### 8. 幸福指数（Team Happiness）

**测量**: 每个 Sprint 结束时，团队成员匿名打分（1-5）

**HermesFlow 目标**: 平均分 ≥ 4.0

---

### 度量仪表盘

创建一个度量仪表盘（Grafana 或 Google Sheets）:

```markdown
**HermesFlow Scrum Dashboard**

### Sprint 进度
- Sprint 目标: [显示]
- 完成进度: X / Y Story Points（Z%）
- 剩余天数: X 天

### 速度
- 本 Sprint 速度: X points
- 平均速度（3 Sprint）: Y points
- 趋势: 📈 / 📉

### 质量
- 测试覆盖率: Rust X%, Java Y%, Python Z%
- 缺陷数量: X 个（P0: A, P1: B, P2: C）
- Code Review 平均时间: X 小时

### 团队
- 幸福指数: X / 5
- 障碍数量: X 个
```

---

## 常见挑战与解决方案

### 挑战 1: Sprint 目标无法完成

**症状**:
- 多个 Sprint 无法完成承诺的 Story
- 速度不稳定

**根因分析**:
- 估算过于乐观
- 外部中断太多
- 技术债务影响

**解决方案**:
1. **降低承诺**: 下个 Sprint 减少 20% Story Points
2. **保护团队**: 拒绝 Sprint 中增加新 Story
3. **技术债务**: 每个 Sprint 分配 20% 容量处理技术债务
4. **回顾估算**: 使用历史数据校准估算

---

### 挑战 2: 跨技术栈协作困难

**症状**:
- Rust、Java、Python 团队各自为战
- 集成延迟和冲突

**根因分析**:
- 缺乏跨团队沟通
- API 契约不清晰
- 依赖关系管理不善

**解决方案**:
1. **架构同步会**: 每周一次，所有 Tech Lead 参加
2. **Contract-First 开发**: API 设计先行，使用 OpenAPI/gRPC
3. **集成测试**: 每个 Sprint 包含跨模块集成测试
4. **配对编程**: 跨语言配对，促进知识传递

---

### 挑战 3: 技术债务积累

**症状**:
- 代码质量下降
- 新功能开发变慢
- 缺陷增加

**根因分析**:
- 只关注新功能，忽视技术债务
- "稍后再修"的心态

**解决方案**:
1. **可视化技术债务**: 在 `progress.md` 中跟踪
2. **强制分配容量**: 每个 Sprint 20% 容量用于技术债务
3. **Definition of Done**: 包含代码质量标准
4. **重构 Story**: 将大的重构拆分为小 Story

---

### 挑战 4: Code Review 延迟

**症状**:
- PR 平均审查时间 > 24 小时
- 开发者阻塞

**解决方案**: [见 Retrospective 示例](#4-决定行动30-分钟)

---

### 挑战 5: 测试覆盖率不达标

**症状**:
- Rust < 85%, Java < 80%, Python < 75%
- 缺陷逃逸到生产环境

**解决方案**:
1. **质量门禁**: CI/CD 中强制覆盖率检查
2. **测试 Story**: 将"提升覆盖率"作为独立 Story
3. **TDD 培训**: 团队培训测试驱动开发
4. **配对测试**: QA 和开发配对编写测试

---

## 📚 参考资源

### Scrum 指南
- [官方 Scrum Guide](https://scrumguides.org/)
- [Scrum Master 学习路径](https://www.scrum.org/pathway/scrum-master)

### HermesFlow 项目文档
- [项目进度](../progress.md)
- [Sprint Planning 清单](./SPRINT-PLANNING-CHECKLIST.md)
- [Retrospective 模板](./RETROSPECTIVE-TEMPLATE.md)
- [系统架构](../architecture/system-architecture.md)
- [PRD 主文档](../prd/PRD-HermesFlow.md)

### 工具和模板
- [Retrospective 技巧库](https://retromat.org/)
- [Sprint Planning Poker](https://planningpokeronline.com/)
- [Miro Scrum 模板](https://miro.com/templates/scrum/)

---

## 📞 获取帮助

如有 Scrum 流程问题，请联系：
- **Scrum Community**: [内部 Scrum Master 社区]
- **Agile Coach**: [组织 Agile Coach]

---

**祝您 Scrum Master 工作顺利！** 🚀

---

**最后更新**: 2025-01-13  
**维护者**: @pm.mdc  
**版本**: v1.0

