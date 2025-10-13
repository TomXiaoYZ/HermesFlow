# Sprint Review 清单

> **确保 Sprint Review 成功的完整清单** | **版本**: v1.0

---

## 📋 目录

1. [Review 前准备](#review-前准备)
2. [Demo 脚本模板](#demo-脚本模板)
3. [会议流程清单](#会议流程清单)
4. [反馈收集表](#反馈收集表)
5. [下个 Sprint 规划](#下个-sprint-规划)

---

## Review 前准备

### 会前 1 天准备

#### Scrum Master 准备清单

- [ ] **会议安排**
  - 发送会议邀请（提前 1 周）
  - 确认所有利益相关者参加
  - 预订会议室 / 准备 Zoom 链接
  - 预计时长: 2小时（2周 Sprint）
  
- [ ] **Demo 环境准备**
  - [ ] Dev 环境所有服务运行正常
  - [ ] 测试数据已准备（真实或类似真实数据）
  - [ ] 网络连接稳定（如远程 Demo）
  - [ ] 投影设备 / 屏幕共享正常
  - [ ] 备用 Demo 环境（如 Dev 环境出问题）
  
  ```bash
  # 环境健康检查
  kubectl get pods -n hermesflow-dev
  kubectl get svc -n hermesflow-dev
  
  # 测试关键服务
  curl https://dev.hermesflow.com/health
  ```

- [ ] **Sprint 总结 PPT / 文档**
  - [ ] Sprint 目标
  - [ ] 完成的 Story 列表
  - [ ] Story Points 完成情况
  - [ ] 速度趋势图
  - [ ] 未完成的 Story 和原因
  - [ ] 关键指标（测试覆盖率、性能、缺陷）
  
- [ ] **Demo 脚本准备**
  - [ ] 为每个功能准备 Demo 脚本
  - [ ] 分配 Demo 演讲人（通常是开发者）
  - [ ] 准备 Demo 用的测试账号和数据
  - [ ] 干跑一次 Demo（确保流畅）

#### 开发团队准备清单

- [ ] **功能验证**
  - [ ] 所有 Demo 的功能已在 Dev 环境部署
  - [ ] 功能已通过 Definition of Done
  - [ ] 没有已知的阻塞性 Bug
  
- [ ] **Demo 准备**
  - [ ] 了解自己要 Demo 的功能
  - [ ] 准备 Demo 脚本（参考 [Demo 脚本模板](#demo-脚本模板)）
  - [ ] 演练 Demo（至少一次）
  - [ ] 准备回答潜在的问题
  
- [ ] **数据准备**
  - [ ] 准备演示用的测试数据
  - [ ] 确保数据真实、有代表性
  - [ ] 准备"黄金路径"（Happy Path）演示

#### Product Owner 准备清单

- [ ] **验收准备**
  - [ ] 审查所有声称完成的 Story
  - [ ] 准备验收标准清单
  - [ ] 准备反馈意见
  
- [ ] **Product Backlog 准备**
  - [ ] 准备下个 Sprint 的候选 Story
  - [ ] 根据本 Sprint 反馈调整优先级
  - [ ] 准备新的需求（如有）

---

### Demo 干跑（Review 前 2 小时）

#### 完整演练清单

- [ ] **技术准备**
  ```bash
  # 检查服务状态
  kubectl get pods -n hermesflow-dev
  
  # 清理旧数据（如需要）
  # 加载演示数据
  
  # 测试关键流程
  ```

- [ ] **按 Demo 顺序演练**
  - [ ] Demo 1: [功能名称] - 演讲人: ___ - 时长: ___分钟
  - [ ] Demo 2: [功能名称] - 演讲人: ___ - 时长: ___分钟
  - [ ] Demo 3: [功能名称] - 演讲人: ___ - 时长: ___分钟
  
- [ ] **计时**
  - 总 Demo 时长 ≤ 60 分钟
  - 每个 Demo 预留缓冲时间
  
- [ ] **识别风险**
  - 哪些功能可能出问题？
  - 备用方案是什么？（录屏、截图等）

---

## Demo 脚本模板

### Demo 脚本结构

```markdown
## Demo X: [功能名称]

**演讲人**: [姓名]  
**Story ID**: [STORY-ID]  
**预计时长**: ___分钟

### 1. 背景和价值（1分钟）

**业务背景**:
[简要说明为什么要做这个功能]

**用户价值**:
[这个功能对用户的价值是什么]

**技术亮点**:
[如有技术亮点，简要说明]

### 2. Demo 步骤（5-10分钟）

**场景**: [描述演示场景，如"策略开发者使用 Alpha 因子库"]

**步骤 1**: [第一步做什么]
- 操作: [具体操作]
- 预期结果: [应该看到什么]
- 截图/录屏: [如有]

**步骤 2**: [第二步做什么]
- 操作: [具体操作]
- 预期结果: [应该看到什么]

**步骤 3**: [第三步做什么]
- 操作: [具体操作]
- 预期结果: [应该看到什么]

### 3. 关键指标展示（2分钟）

**性能指标**:
- 响应时间: P95 < ___ms ✅
- 吞吐量: ___ 请求/秒 ✅

**质量指标**:
- 单元测试覆盖率: ___%✅
- 集成测试: 通过 ✅

**验收标准**:
- [ ] AC1: [验收标准 1] ✅
- [ ] AC2: [验收标准 2] ✅
- [ ] AC3: [验收标准 3] ✅

### 4. 预期反馈点（1分钟）

**引导问题**:
- 这个功能是否满足您的需求？
- 界面/API 是否易用？
- 还需要什么改进？

### 5. 备用方案

**如果 Demo 失败**:
- Plan B: 使用录屏
- Plan C: 使用截图 + 讲解
```

---

### HermesFlow Demo 脚本示例

```markdown
## Demo 1: Alpha 因子库

**演讲人**: 李明（Rust Developer）  
**Story ID**: DATA-101, DATA-103  
**预计时长**: 15分钟

### 1. 背景和价值（2分钟）

**业务背景**:
量化策略开发者需要使用大量技术指标（如 RSI、MACD、Bollinger Bands）。之前每个策略都要自己实现这些指标，导致重复开发和代码质量不一致。

**用户价值**:
- 提供标准化的因子库，减少重复开发
- 高性能 Rust 实现，比 Python 快 10 倍
- 简单易用的 Python API，无缝集成到策略代码

**技术亮点**:
- Rust 实现，性能卓越（10万行/秒）
- 单元测试覆盖率 87%
- Python FFI 绑定，零拷贝数据传输

### 2. Demo 步骤（10分钟）

**场景**: 策略开发者在 Jupyter Notebook 中使用 RSI 和 MACD 因子

**步骤 1: 导入 API**
```python
from hermesflow.factors import RSI, MACD, BollingerBands
import pandas as pd
```

**步骤 2: 加载市场数据**
```python
# 加载 BTC/USDT 1分钟 K 线数据
data = pd.read_csv('btc_usdt_1m.csv')
prices = data['close'].values
```

**步骤 3: 计算 RSI**
```python
# 计算 14 周期 RSI
rsi = RSI(period=14)
rsi_values = rsi.calculate(prices)

# 可视化
import matplotlib.pyplot as plt
plt.plot(rsi_values)
plt.axhline(y=70, color='r', linestyle='--')  # 超买线
plt.axhline(y=30, color='g', linestyle='--')  # 超卖线
plt.show()
```
- 预期结果: 显示 RSI 曲线，识别超买超卖区域

**步骤 4: 计算 MACD**
```python
# 计算 MACD (12, 26, 9)
macd = MACD(fast=12, slow=26, signal=9)
macd_line, signal_line, histogram = macd.calculate(prices)

# 可视化
plt.figure(figsize=(12, 6))
plt.subplot(2, 1, 1)
plt.plot(prices, label='Price')
plt.subplot(2, 1, 2)
plt.plot(macd_line, label='MACD')
plt.plot(signal_line, label='Signal')
plt.bar(range(len(histogram)), histogram, label='Histogram')
plt.legend()
plt.show()
```
- 预期结果: 显示 MACD 指标，识别金叉死叉

**步骤 5: 性能演示**
```python
import time

# 测试 10 万数据点
large_data = np.random.randn(100000)

start = time.time()
rsi = RSI(period=14)
result = rsi.calculate(large_data)
elapsed = time.time() - start

print(f"处理 10 万数据点耗时: {elapsed*1000:.2f} ms")
print(f"吞吐量: {len(large_data)/elapsed:.0f} 行/秒")
```
- 预期结果: `处理 10 万数据点耗时: 8.5 ms，吞吐量: 11.7 万行/秒`

### 3. 关键指标展示（2分钟）

**性能指标**:
- RSI 计算: 10万行/秒 ✅ （目标: 10万行/秒）
- P95 延迟: < 10ms ✅ （目标: < 10ms）
- 内存占用: < 50MB ✅

**质量指标**:
- 单元测试覆盖率: 87% ✅ （目标: ≥85%）
- 集成测试: 10/10 通过 ✅
- 与标准库对比: 误差 < 0.01% ✅

**支持的因子**:
- ✅ RSI (Relative Strength Index)
- ✅ MACD (Moving Average Convergence Divergence)
- ✅ Bollinger Bands
- ✅ EMA (Exponential Moving Average)
- ✅ SMA (Simple Moving Average)
- ⏸️ Stochastic Oscillator (下个 Sprint)
- ⏸️ ATR (Average True Range) (下个 Sprint)

### 4. 预期反馈点（1分钟）

**引导问题**:
- Python API 是否易用？参数命名是否清晰？
- 还需要哪些因子？（我们计划增加 Stochastic、ATR 等）
- 性能是否满足您的需求？
- 文档是否清晰？

### 5. 备用方案

**如果 Demo 失败**:
- Plan B: 使用预先录制的 Jupyter Notebook 执行视频
- Plan C: 使用截图 + 代码讲解
```

---

## 会议流程清单

### Sprint Review 完整流程（2 小时）

#### 1. 欢迎和 Sprint 概述（10 分钟）

- [ ] **Scrum Master 开场**
  - 欢迎所有参与者
  - 介绍议程
  - 设定期望（这是一个协作会议，不是汇报会议）
  
- [ ] **Sprint 概述**
  ```markdown
  **Sprint X Review** - [日期]
  
  **Sprint 目标**: [重申 Sprint 目标]
  
  **参与者**: [列出参与者]
  
  **议程**:
  - Sprint 概述（10分钟）
  - Demo 完成的功能（60分钟）
  - 未完成的工作和原因（10分钟）
  - 讨论和反馈（30分钟）
  - Product Backlog 更新（10分钟）
  ```
  
- [ ] **Sprint 数据展示**
  - Sprint 目标达成情况: X% 完成
  - 完成的 Story 数量: X / Y
  - Story Points 完成: X / Y points
  - 速度: X points/sprint（最近 3 个 Sprint 平均）
  - 质量指标: 测试覆盖率、缺陷数量
  
  **可视化**:
  - Sprint 燃尽图
  - 速度趋势图
  - 累积流图

#### 2. Demo 完成的功能（60 分钟）

**Demo 原则**:
- ✅ 只 Demo 满足 Definition of Done 的功能
- ✅ 在真实环境（或接近真实）中 Demo
- ✅ 展示端到端的用户流程
- ✅ 让开发者亲自 Demo（不是 SM 或 PO）
- ✅ 聚焦用户价值，而非技术细节

**Demo 清单**:
- [ ] Demo 1: [功能名称] - 演讲人: ___ - 时长: ___分钟
- [ ] Demo 2: [功能名称] - 演讲人: ___ - 时长: ___分钟
- [ ] Demo 3: [功能名称] - 演讲人: ___ - 时长: ___分钟
- [ ] Demo 4: [功能名称] - 演讲人: ___ - 时长: ___分钟

**每个 Demo 后**:
- [ ] 简短的 Q&A（2-3 分钟）
- [ ] 记录反馈（使用 [反馈收集表](#反馈收集表)）

#### 3. 未完成的工作（10 分钟）

- [ ] **诚实透明地讨论未完成的 Story**
  ```markdown
  **未完成的 Story**:
  
  ### [STORY-ID] Story 标题
  
  **完成度**: X%
  
  **原因**:
  - [具体原因，如依赖阻塞、技术难度被低估、优先级变更等]
  
  **影响评估**:
  - 对 Sprint 目标的影响: [评估]
  - 对用户的影响: [评估]
  
  **处理计划**:
  - [ ] 回到 Product Backlog，重新排优先级
  - [ ] 下 Sprint 继续（如仍高优先级）
  - [ ] 分解为更小的 Story
  ```

- [ ] **经验教训**
  - 我们从中学到了什么？
  - 如何避免下次再发生？

#### 4. 讨论和反馈（30 分钟）

- [ ] **引导利益相关者反馈**
  
  **开放式问题**:
  - 这些功能是否满足您的期望？
  - 有哪些地方可以改进？
  - 您有什么新的需求或想法？
  - 市场/客户反馈如何？
  
- [ ] **记录所有反馈**
  - 使用 [反馈收集表](#反馈收集表)
  - Scrum Master 实时记录
  - 所有反馈可视化（白板或 Miro）
  
- [ ] **讨论优先级**
  - 哪些反馈最重要？
  - 哪些应该在下个 Sprint 处理？
  - 哪些需要进一步细化？

#### 5. Product Backlog 更新（10 分钟）

- [ ] **Product Owner 更新 Backlog**
  - 添加新的 Story（基于反馈）
  - 调整优先级
  - 细化下个 Sprint 的候选 Story
  
- [ ] **预告下个 Sprint**
  - 下个 Sprint 的可能目标
  - 关键交付物
  - 重要日期

#### 6. 总结和结束（5 分钟）

- [ ] **感谢所有参与者**
  - 感谢团队的辛勤工作
  - 感谢利益相关者的反馈
  
- [ ] **下一步行动**
  - Sprint Retrospective: [日期和时间]
  - 下个 Sprint Planning: [日期和时间]
  
- [ ] **会议纪要**
  - 会后发送会议纪要
  - 包含反馈和行动项

---

## 反馈收集表

### 反馈记录模板

```markdown
## Sprint X Review 反馈记录

**日期**: YYYY-MM-DD  
**参与者**: [列出参与者]

### 反馈汇总表

| ID | 反馈内容 | 提出人 | 类型 | 优先级 | 行动计划 | 负责人 |
|----|---------|--------|------|--------|---------|--------|
| FB-01 | RSI API 参数说明不够清晰 | 客户A | 文档 | P1 | 增强 API 文档 | @李明 |
| FB-02 | 希望增加 Bollinger Bands 因子 | PO | 新功能 | P2 | 添加到 Backlog | @PO |
| FB-03 | 性能很好，但希望支持更大数据集 | 用户B | 改进 | P2 | 性能优化 Story | @架构师 |
| FB-04 | 界面很直观，易于使用 | 用户C | 正面 | - | 继续保持 | - |
| FB-05 | 希望有更多使用示例 | 用户D | 文档 | P2 | 补充示例文档 | @李明 |
```

**类型说明**:
- **Bug**: 缺陷或问题
- **新功能**: 新需求
- **改进**: 现有功能的改进
- **文档**: 文档相关
- **正面**: 正面反馈

**优先级说明**:
- **P0**: 紧急，阻塞后续开发
- **P1**: 高优先级，下个 Sprint 处理
- **P2**: 中优先级，积压处理
- **P3**: 低优先级，长期改进

### 详细反馈记录

#### 反馈 1: RSI API 参数说明不够清晰

**提出人**: 客户A  
**具体描述**:
`period` 参数的含义不够清晰，不知道应该设置为多少。建议在文档中说明常用的 period 值（如 14、21）以及它们的适用场景。

**影响评估**:
- 用户体验: 中等影响
- 技术复杂度: 低（仅文档更新）

**行动计划**:
- [ ] 更新 API 文档，增加参数说明
- [ ] 添加常用参数推荐
- [ ] 添加使用示例
- [ ] 截止日期: 本周五
- [ ] 负责人: @李明

---

#### 反馈 2: 希望增加 Bollinger Bands 因子

**提出人**: Product Owner  
**具体描述**:
Bollinger Bands 是常用技术指标，建议在下个 Sprint 增加。

**影响评估**:
- 用户价值: 高
- 技术复杂度: 中等（1-2 天工作量）

**行动计划**:
- [ ] 创建 Story: "实现 Bollinger Bands 因子"
- [ ] 添加到 Product Backlog
- [ ] 优先级: P2（下个 Sprint 候选）
- [ ] 负责人: @PO

---

[继续记录其他反馈...]
```

---

## 下个 Sprint 规划

### Product Backlog 更新清单

基于 Sprint Review 的反馈，Product Owner 更新 Backlog：

- [ ] **新 Story 创建**
  - [ ] 基于反馈创建新 Story
  - [ ] 每个 Story 有清晰的描述和验收标准
  - [ ] 估算优先级（P0/P1/P2/P3）
  
- [ ] **现有 Story 调整**
  - [ ] 调整优先级（基于反馈）
  - [ ] 细化高优先级 Story
  - [ ] 合并重复 Story
  
- [ ] **技术债务**
  - [ ] 识别技术债务
  - [ ] 创建技术债务 Story
  - [ ] 纳入优先级排序

### 下个 Sprint 预告

```markdown
## Sprint X+1 预告

**可能的 Sprint 目标**: [基于 Review 反馈和业务优先级]

**候选 Story**（Top 10）:
1. [STORY-ID] Story 标题 - 优先级: P0 - 估算: X points
2. [STORY-ID] Story 标题 - 优先级: P0 - 估算: X points
3. [STORY-ID] Story 标题 - 优先级: P1 - 估算: X points
...

**关键交付物**:
- [交付物 1]
- [交付物 2]
- [交付物 3]

**重要日期**:
- Sprint Planning: YYYY-MM-DD
- Sprint Review: YYYY-MM-DD
- Sprint Retrospective: YYYY-MM-DD
```

---

## 📚 相关资源

- [Sprint 启动包](./sprint-starter-pack.md)
- [Scrum Master 完整指南](./sm-guide.md)
- [Sprint Planning 清单](./sprint-planning-checklist.md)
- [Retrospective 模板](./retrospective-template.md)
- [Definition of Done](./definition-of-done.md)

---

## ✅ Review 成功标准

Sprint Review 成功的标志：

- [ ] **所有利益相关者参与**
  - 至少 80% 的邀请者参加
  - 积极参与讨论和反馈
  
- [ ] **Demo 清晰有效**
  - 展示了实际工作的软件
  - 利益相关者理解功能价值
  - 技术问题少于 2 次
  
- [ ] **收集到有价值的反馈**
  - 至少 5 条反馈
  - 反馈具体可操作
  - 反馈记录完整
  
- [ ] **Product Backlog 已更新**
  - 新 Story 已添加
  - 优先级已调整
  - 下个 Sprint 方向明确
  
- [ ] **团队士气高**
  - 团队为成果自豪
  - 积极的氛围
  - 建设性的讨论

---

**最后更新**: 2025-01-13  
**维护者**: @pm.mdc  
**版本**: v1.0

