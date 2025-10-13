# 开发周期总览

> **HermesFlow 完整 2 周 Sprint 时间线** | **版本**: v1.0

---

## 📋 目录

1. [Sprint 时间线](#sprint-时间线)
2. [Week 1: 开发冲刺第一周](#week-1-开发冲刺第一周)
3. [Week 2: 开发冲刺第二周](#week-2-开发冲刺第二周)
4. [关键里程碑](#关键里程碑)
5. [角色责任矩阵](#角色责任矩阵)

---

## Sprint 时间线

### 2 周 Sprint 完整视图

```
Week 1                                          Week 2
┌─────────────────────────────────────────────┬─────────────────────────────────────────────┐
│  Day 1   Day 2   Day 3   Day 4   Day 5      │  Day 6   Day 7   Day 8   Day 9   Day 10    │
├─────────────────────────────────────────────┼─────────────────────────────────────────────┤
│  🎯 Sprint Planning (4h)                     │                                             │
│     ↓                                        │                                             │
│  💻 Development Start                        │  💻 Development (冲刺)                       │
│     ↓                                        │     ↓                                       │
│  🗣️ Daily Standup (15min/day)              │  🗣️ Daily Standup (15min/day)              │
│     ↓                                        │     ↓                                       │
│  ⚙️ Mid-Sprint Check-in (Day 5, 1h)         │  🎬 Demo 准备 (Day 9)                       │
│     ↓                                        │     ↓                                       │
│  💻 Development (持续)                       │  🎭 Sprint Review (Day 10, 2h)              │
│                                             │     ↓                                       │
│                                             │  🔄 Sprint Retrospective (Day 10, 1.5h)     │
│                                             │     ↓                                       │
│                                             │  🎉 Sprint 结束                             │
└─────────────────────────────────────────────┴─────────────────────────────────────────────┘
```

---

## Week 1: 开发冲刺第一周

### Day 1: Sprint Planning & Kickoff (Monday)

#### 上午：Sprint Planning（9:00-13:00）

**目标**: 确定 Sprint 目标和 Sprint Backlog

**议程**:
- ✅ 回顾上个 Sprint（15分钟）
- ✅ 确定 Sprint 目标（45分钟）
- ✅ 选择 Story（1小时）
- ✅ 任务分解和估算（1.5小时）
- ✅ 识别风险和依赖（30分钟）
- ✅ 确定 Definition of Done（30分钟）

**输出**:
- Sprint 目标文档
- Sprint Backlog（Story 和任务列表）
- 风险登记表
- Definition of Done

**参考**: [Sprint 启动包](./sprint-starter-pack.md)

---

#### 下午：团队同步 & 环境准备（14:00-17:00）

**14:00-15:00 团队同步会**
- Scrum Master 回顾 Sprint 计划
- 架构师讲解关键架构点
- QA 讲解测试策略

**15:00-15:30 任务认领**
- 开发者在看板上认领任务
- 确认第一个任务

**15:30-17:00 环境准备**
- 验证开发环境
- 启动本地服务
- 开始第一个任务

---

### Day 2-3: 开发冲刺启动（Tuesday-Wednesday）

#### 每日节奏

**10:00-10:15 Daily Standup**
- 三个问题：昨天完成、今天计划、遇到障碍
- 更新 Sprint 燃尽图
- 识别障碍

**10:15-12:00 开发时间**
- 编写代码
- 编写测试
- Code Review

**14:00-15:00 Code Review 时间块**
- 集中处理 Code Review
- 目标：4小时内完成 Review

**15:00-17:00 开发时间**
- 继续开发
- 集成测试
- 文档更新

#### 关键活动

**Day 2**:
- 🎯 高优先级 Story 开始开发
- 📝 API 设计确认（如有新 API）
- 🔧 基础设施确认（数据库、Kafka 等）

**Day 3**:
- 🚀 第一批功能代码完成
- ✅ 单元测试编写
- 🔍 Code Review 进行中

---

### Day 4: 持续开发（Thursday）

#### 每日节奏（同 Day 2-3）

#### 关键活动

- 💻 核心功能开发（60-70% 完成）
- 🧪 集成测试开始
- 📊 关注 Sprint 燃尽图
  - 是否在轨？
  - 是否有风险？
  
#### 风险检查

如果 Sprint 燃尽图落后：
- 🚨 识别阻塞任务
- 🚨 调整任务优先级
- 🚨 考虑移除低优先级 Story

---

### Day 5: Mid-Sprint Check-in（Friday）

#### 10:00-10:15 Daily Standup（照常）

#### 15:00-16:00 Mid-Sprint Check-in

**目标**: 评估 Sprint 进度，识别风险

**议程**:
- ✅ Sprint 目标达成情况（50% 检查点）
- ✅ Sprint 燃尽图分析
- ✅ 阻塞和风险识别
- ✅ 需要调整的地方

**关键问题**:
1. 我们能完成 Sprint 目标吗？
   - 🟢 是 → 继续执行
   - 🟡 可能 → 识别风险，制定缓解措施
   - 🔴 不能 → 调整 Sprint Backlog

2. 有哪些阻塞？
   - 技术问题
   - 依赖问题
   - 人员问题

3. 需要调整什么？
   - 移除低优先级 Story
   - 增加配对编程
   - 请求外部帮助

**输出**:
- 调整后的 Sprint Backlog（如需要）
- 更新的风险登记表
- 行动计划

---

## Week 2: 开发冲刺第二周

### Day 6: 开发加速（Monday）

#### 每日节奏（同 Week 1）

#### 关键活动

- 💻 冲刺完成剩余功能（目标：80% 完成）
- ✅ 单元测试完成
- 🧪 集成测试进行中
- 📝 文档更新

#### 本周重点

**开发重点**:
- 完成所有核心功能
- 确保 Definition of Done 满足
- 开始准备 Demo

**测试重点**:
- 单元测试覆盖率达标
- 集成测试通过
- 性能测试（如适用）

---

### Day 7-8: 功能完成冲刺（Tuesday-Wednesday）

#### 每日节奏（同 Week 1）

#### 关键活动

**Day 7**:
- 🎯 目标：90% 功能完成
- ✅ 所有代码 Code Review 完成
- 🔧 Bug 修复
- 📊 CI/CD Pipeline 全部通过

**Day 8**:
- 🎯 目标：95% 功能完成
- ✅ Definition of Done 检查
- 🚀 Dev 环境部署和验证
- 📝 Demo 脚本准备

#### Code Freeze 讨论

在 Day 8 下午，团队讨论是否需要 Code Freeze：
- 如果 Sprint 进度良好 → Day 9 上午 Code Freeze
- 如果有风险 → 继续开发到 Day 9 下午

---

### Day 9: Demo 准备（Thursday）

#### 10:00-10:15 Daily Standup（照常）

#### 10:15-12:00 最后冲刺

- 🐛 修复关键 Bug
- ✅ 最后的测试
- 🚀 确保 Dev 环境稳定

#### 14:00-15:00 Demo 干跑

**目标**: 完整演练所有 Demo

- [ ] 按 Demo 顺序演练
- [ ] 计时（总时长 ≤ 60分钟）
- [ ] 识别潜在问题
- [ ] 准备备用方案（截图、录屏）

**参考**: [Sprint Review 清单](./sprint-review-checklist.md)

#### 15:00-17:00 Demo 准备完善

- 📊 准备 Sprint 总结 PPT
- 🗂️ 准备反馈收集表
- 📝 最后的文档更新

---

### Day 10: Sprint Review & Retrospective（Friday）

#### 10:00-10:15 Daily Standup（最后一次）

**特殊关注**:
- 确认所有 Story 状态
- 确认 Demo 准备就绪
- 感谢团队的辛勤工作

---

#### 15:00-17:00 Sprint Review

**目标**: 展示完成的工作，收集反馈

**议程**:
- Sprint 概述（10分钟）
- Demo 完成的功能（60分钟）
- 未完成的工作和原因（10分钟）
- 讨论和反馈（30分钟）
- Product Backlog 更新（10分钟）

**参考**: [Sprint Review 清单](./sprint-review-checklist.md)

---

#### 17:00-18:30 Sprint Retrospective

**目标**: 反思 Sprint 工作方式，识别改进

**议程**:
- 设定氛围（5分钟）
- 收集数据（20分钟）- Start/Stop/Continue
- 产生洞察（30分钟）- 分析根因
- 决定行动（30分钟）- 制定改进计划
- 回顾上次行动（10分钟）
- 总结和关闭（5分钟）

**参考**: [Retrospective 模板](./retrospective-template.md)

---

#### 18:30 Sprint 结束 🎉

**庆祝**:
- 感谢团队
- 庆祝完成的工作
- 为下个 Sprint 做好准备

---

## 关键里程碑

| 里程碑 | 日期 | 完成度目标 | 关键活动 |
|-------|------|----------|---------|
| **Sprint Kickoff** | Day 1 | 0% | Sprint Planning, 任务认领 |
| **开发启动** | Day 2-3 | 30% | 核心功能开发开始 |
| **Mid-Sprint Check** | Day 5 | 50% | 进度检查，风险评估 |
| **功能完成冲刺** | Day 7 | 90% | 所有功能基本完成 |
| **Demo 准备** | Day 9 | 100% | Demo 干跑，环境准备 |
| **Sprint Review** | Day 10 | - | 展示工作，收集反馈 |
| **Sprint Retro** | Day 10 | - | 反思改进 |

---

## 角色责任矩阵

### Scrum Master

| 阶段 | 责任 |
|------|------|
| **Sprint Planning** | 组织和引导会议，确保输出完整 |
| **Daily** | 促进每日站会，移除障碍 |
| **Mid-Sprint** | 组织 Check-in，评估风险 |
| **Sprint Review** | 组织会议，记录反馈 |
| **Retrospective** | 引导回顾，跟踪行动项 |

### Product Owner

| 阶段 | 责任 |
|------|------|
| **Sprint Planning** | 澄清需求，确定优先级 |
| **Daily** | 可选参加，回答需求问题 |
| **Mid-Sprint** | 参加 Check-in，调整优先级 |
| **Sprint Review** | 验收功能，提供反馈，更新 Backlog |
| **Retrospective** | 可选参加 |

### Tech Lead / 架构师

| 阶段 | 责任 |
|------|------|
| **Sprint Planning** | 提供技术方案，识别技术风险 |
| **Daily** | 参加站会，提供技术支持 |
| **Mid-Sprint** | 评估技术风险，提供指导 |
| **Sprint Review** | Demo 技术亮点 |
| **Retrospective** | 参与讨论，制定技术改进计划 |

### 开发者

| 阶段 | 责任 |
|------|------|
| **Sprint Planning** | 估算任务，识别风险 |
| **Daily** | 参加站会，更新进度 |
| **开发** | 编码、测试、Code Review |
| **Sprint Review** | Demo 自己的功能 |
| **Retrospective** | 参与讨论，提供改进建议 |

### QA 工程师

| 阶段 | 责任 |
|------|------|
| **Sprint Planning** | 确认测试策略，估算测试工作 |
| **Daily** | 参加站会，报告测试进度 |
| **开发** | 编写测试用例，执行测试 |
| **Sprint Review** | Demo 测试结果，报告质量 |
| **Retrospective** | 参与讨论，提出质量改进建议 |

### DevOps 工程师

| 阶段 | 责任 |
|------|------|
| **Sprint Planning** | 确认基础设施准备就绪 |
| **Daily** | 参加站会，支持环境问题 |
| **开发** | CI/CD 支持，环境维护 |
| **Sprint Review** | 展示部署流程（如适用） |
| **Retrospective** | 参与讨论，提出部署改进建议 |

---

## 📊 每日典型时间分配

### 开发者典型一天（Day 2-8）

```
09:00-10:00  🌅 到达，查看邮件，准备站会
10:00-10:15  🗣️ Daily Standup
10:15-12:00  💻 开发时间（编码、测试）
12:00-13:00  🍱 午休
13:00-14:00  💻 开发时间（继续编码）
14:00-15:00  🔍 Code Review 时间块
15:00-17:00  💻 开发时间（集成测试、文档）
17:00-18:00  📝 整理，准备明天
```

**会议时间**: 约 1-1.5 小时/天（15分钟站会 + 不定期技术讨论）  
**开发时间**: 约 6 小时/天

---

## 📈 Sprint 健康指标

### 每日检查

Scrum Master 每天检查以下指标：

| 指标 | 健康标准 | 行动 |
|------|---------|------|
| **出勤率** | 100% | 如有缺席，及时沟通 |
| **燃尽图** | 实际 ≤ 理想 + 10% | 如落后，分析原因 |
| **阻塞任务** | ≤ 2 个 | 立即移除障碍 |
| **Code Review 延迟** | < 4 小时 | 提醒 Reviewer |
| **CI/CD 失败率** | < 10% | 修复环境问题 |

### Week 1 检查（Day 5）

| 指标 | 健康标准 | 行动 |
|------|---------|------|
| **Story Points 完成** | ≥ 50% | 如 < 40%，调整 Backlog |
| **Story 完成数量** | ≥ 40% | 分析原因 |
| **测试覆盖率** | 按计划增长 | 提醒开发者 |
| **未开始 Story** | ≤ 20% | 分析为什么没开始 |

### Week 2 检查（Day 8）

| 指标 | 健康标准 | 行动 |
|------|---------|------|
| **Story Points 完成** | ≥ 90% | 如 < 80%，风险管理 |
| **DoD 满足** | ≥ 80% Story | 加速 Review 和测试 |
| **Demo 准备** | 就绪 | 组织 Demo 干跑 |
| **缺陷数量** | ≤ 5 个 | 优先修复 |

---

## 💡 最佳实践

### 1. 保持节奏

- ✅ 固定的时间（每天 10:00 站会）
- ✅ 固定的模式（相同的 Sprint 结构）
- ✅ 可预测的交付（每 2 周一次 Review）

### 2. 聚焦 Sprint 目标

- ✅ 每天在站会上重申 Sprint 目标
- ✅ 所有决策对齐 Sprint 目标
- ✅ 拒绝 Sprint 中的新需求

### 3. 持续集成

- ✅ 每天至少一次提交
- ✅ 小步快跑，频繁集成
- ✅ 保持 main 分支可部署

### 4. 透明沟通

- ✅ 问题及时暴露
- ✅ 风险及早识别
- ✅ 进度公开透明

### 5. 团队协作

- ✅ 配对编程（复杂任务）
- ✅ 及时 Code Review
- ✅ 知识分享

---

## 📚 相关资源

- [Sprint 启动包](./sprint-starter-pack.md)
- [每日站会指南](./daily-standup-guide.md)
- [Sprint Review 清单](./sprint-review-checklist.md)
- [Retrospective 模板](./retrospective-template.md)
- [Definition of Done](./definition-of-done.md)
- [Scrum Master 完整指南](./sm-guide.md)

---

**最后更新**: 2025-01-13  
**维护者**: @pm.mdc  
**版本**: v1.0

