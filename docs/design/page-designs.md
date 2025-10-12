# HermesFlow 页面设计规范

**版本**: v1.0.0  
**最后更新**: 2024-12-20  
**作者**: HermesFlow Design Team

---

## 目录

1. [总览仪表盘 (Dashboard)](#总览仪表盘-dashboard)
2. [策略管理页面 (Strategies)](#策略管理页面-strategies)
3. [策略编辑器页面 (Strategy Editor)](#策略编辑器页面-strategy-editor)
4. [回测报告页面 (Backtest Report)](#回测报告页面-backtest-report)
5. [交易监控页面 (Trading)](#交易监控页面-trading)
6. [风控监控页面 (Risk Management)](#风控监控页面-risk-management)
7. [设置页面 (Settings)](#设置页面-settings)

---

## 总览仪表盘 (Dashboard)

### 页面概述

仪表盘是用户登录后的默认页面，提供账户总览、策略状态、实时行情等核心信息的快速查看。

### 布局结构

```
+----------------------------------------------------------+
| [Logo] HermesFlow                   [User] [🔔] [⚙️]    |
+----------------------------------------------------------+
| [Dashboard] [策略] [交易] [风控] [因子库] [回测]...      |
+----------------------------------------------------------+
|                                                          |
| +------------------------+  +--------------------------+ |
| | 账户总值卡片            |  | 今日盈亏卡片              | |
| | $125,430.50           |  | +$2,340.80 (+1.9%)      | |
| | 💰 [Wallet Icon]      |  | 📈 [TrendingUp Icon]    | |
| +------------------------+  +--------------------------+ |
|                                                          |
| +------------------------+  +--------------------------+ |
| | 运行中策略              |  | 总交易次数                | |
| | 3 个                   |  | 156 笔                   | |
| | ⚡ [Zap Icon]          |  | 🔄 [Activity Icon]       | |
| +------------------------+  +--------------------------+ |
|                                                          |
| +--------------------------------------------------------+
| | 资金曲线图 (7天)                                        |
| | [Recharts折线图 - 渐变填充]                            |
| |                                                        |
| |  $130K ┤                                    ╱          |
| |  $125K ┤                            ╱──────            |
| |  $120K ┤                    ╱──────                    |
| |  $115K ┤            ╱──────                            |
| |  $110K ┤    ╱──────                                    |
| |        └────┴────┴────┴────┴────┴────┴────            |
| |         Mon  Tue  Wed  Thu  Fri  Sat  Sun             |
| +--------------------------------------------------------+
|                                                          |
| +------------------------+  +--------------------------+ |
| | 运行中策略 (3)          |  | 实时行情                  | |
| |                        |  |                          | |
| | MA交叉策略             |  | BTC: $43,250.50 ↗        | |
| | [运行中] +15.3%        |  | 24h: +2.5%               | |
| |                        |  | [迷你K线图]               | |
| | RSI突破策略            |  |                          | |
| | [运行中] +8.1%         |  | ETH: $2,280.30 ↗         | |
| |                        |  | 24h: +3.2%               | |
| | 趋势跟踪               |  | [迷你K线图]               | |
| | [运行中] +12.7%        |  |                          | |
| +------------------------+  +--------------------------+ |
|                                                          |
+----------------------------------------------------------+
```

### 组件详细设计

#### 1. 顶部导航栏

**组件**: `TopNavBar`

**样式**：
- 背景: `--bg-secondary`
- 高度: `64px`
- 边框底部: `1px solid var(--border-primary)`

**内容**：
- 左侧: Logo + 产品名称
- 中部: 主导航菜单
- 右侧: 通知图标 + 设置图标 + 用户头像

#### 2. 账户总值卡片

**组件**: `AccountValueCard`

**样式**：
```css
.account-value-card {
  background: linear-gradient(135deg, #FCD535 0%, #F0B90B 100%);
  border-radius: var(--radius-lg);
  padding: var(--space-6);
  color: var(--bg-primary);
  box-shadow: var(--shadow-md);
}

.account-value-number {
  font-size: var(--text-4xl);
  font-family: var(--font-mono);
  font-weight: var(--font-bold);
  margin-bottom: var(--space-2);
}

.account-value-label {
  font-size: var(--text-sm);
  opacity: 0.8;
}
```

**内容**：
- 大数字显示总资产（36px，等宽字体）
- Wallet图标（24px）
- 标签"账户总值"

#### 3. 今日盈亏卡片

**组件**: `DailyPnLCard`

**样式**：
```css
.daily-pnl-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-lg);
  padding: var(--space-6);
  transition: all 0.2s;
}

.daily-pnl-card:hover {
  border-color: var(--success);
  box-shadow: var(--shadow-md);
  transform: translateY(-2px);
}

.pnl-positive {
  color: var(--success);
}

.pnl-negative {
  color: var(--danger);
}

.pnl-number {
  font-size: var(--text-2xl);
  font-family: var(--font-mono);
  font-weight: var(--font-semibold);
}

.pnl-percentage {
  font-size: var(--text-lg);
  margin-left: var(--space-2);
}
```

**内容**：
- 盈亏金额（绿色为正，红色为负）
- 百分比（同色）
- TrendingUp/TrendingDown图标

#### 4. 资金曲线图

**组件**: `EquityCurveChart`

**使用库**: Recharts / Tremor

**配置**：
```typescript
{
  type: 'area',  // 区域图
  data: last7DaysEquity,
  xAxis: { dataKey: 'date', format: 'ddd' },
  yAxis: { format: '$,.0f' },
  fill: {
    type: 'gradient',
    colors: ['rgba(14, 203, 129, 0.2)', 'rgba(14, 203, 129, 0)']
  },
  stroke: {
    color: '#0ECB81',
    width: 2
  },
  grid: {
    stroke: '#2B3139',
    strokeDasharray: '3 3'
  },
  tooltip: {
    background: '#2B3139',
    border: '1px solid #3B4149',
    textColor: '#EAECEF'
  }
}
```

**样式**：
- 高度: `300px`
- 背景: `--bg-secondary`
- 圆角: `--radius-lg`
- 内边距: `--space-6`

#### 5. 运行中策略列表

**组件**: `ActiveStrategiesList`

**样式**：
```css
.strategy-item {
  background: var(--bg-tertiary);
  border-radius: var(--radius-md);
  padding: var(--space-4);
  margin-bottom: var(--space-3);
  cursor: pointer;
  transition: all 0.2s;
}

.strategy-item:hover {
  background: var(--bg-elevated);
  transform: translateX(4px);
}

.strategy-name {
  font-size: var(--text-sm);
  font-weight: var(--font-semibold);
  color: var(--text-primary);
  margin-bottom: var(--space-2);
}

.strategy-status {
  display: inline-block;
  padding: var(--space-1) var(--space-2);
  border-radius: var(--radius-full);
  font-size: var(--text-xs);
  background: rgba(14, 203, 129, 0.1);
  color: var(--success);
}

.strategy-pnl {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--success);
  float: right;
}
```

#### 6. 实时行情卡片

**组件**: `LiveQuotesCard`

**样式**：
- 每个币种一行
- 价格大字显示（20px，等宽字体）
- 涨跌幅颜色标识
- 迷你K线图（使用react-sparklines）

---

## 策略管理页面 (Strategies)

### 页面概述

策略管理页面显示所有用户创建的策略，支持搜索、筛选、排序，并可快速查看每个策略的关键指标。

### 布局结构

```
+----------------------------------------------------------+
| 策略列表                                    [+ 新建策略]  |
+----------------------------------------------------------+
| [🔍 搜索框] [筛选: 全部 ▼] [排序: 收益 ▼]               |
+----------------------------------------------------------+
|                                                          |
| +--------------------------------------------------------+
| | MA交叉策略                        [运行中] [编辑] [...] |
| | 类型: 趋势跟踪  |  资产: 加密货币                       |
| | 收益: +15.3%  |  夏普: 1.8  |  回撤: -8.2%            |
| | [迷你收益曲线 ──────╱───╱──]                           |
| +--------------------------------------------------------+
|                                                          |
| +--------------------------------------------------------+
| | RSI突破策略                       [已停止] [编辑] [...] |
| | 类型: 动量策略  |  资产: 加密货币                       |
| | 收益: +8.1%   |  夏普: 1.2  |  回撤: -12.5%           |
| | [迷你收益曲线 ──╱───╲──────]                           |
| +--------------------------------------------------------+
|                                                          |
| +--------------------------------------------------------+
| | 趋势跟踪策略                      [回测中] [编辑] [...] |
| | 类型: 趋势跟踪  |  资产: 美股                           |
| | 收益: +12.7%  |  夏普: 2.1  |  回撤: -5.8%            |
| | [迷你收益曲线 ───╱──────╱──]                           |
| +--------------------------------------------------------+
|                                                          |
+----------------------------------------------------------+
| [显示 1-10 / 共 23 条]                      [上一页][下一页] |
+----------------------------------------------------------+
```

### 组件详细设计

#### 1. 页面头部

**组件**: `StrategyListHeader`

**内容**：
- 左侧: 页面标题"策略列表"
- 右侧: "新建策略"按钮（主按钮，带Plus图标）

#### 2. 筛选栏

**组件**: `StrategyFilterBar`

**内容**：
- 搜索框（占30%宽度）
- 筛选下拉框：全部/运行中/已停止/错误
- 排序下拉框：收益/夏普比率/回撤/创建时间

#### 3. 策略卡片

**组件**: `StrategyCard`

**样式**：
```css
.strategy-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-lg);
  padding: var(--space-6);
  margin-bottom: var(--space-4);
  transition: all 0.2s;
}

.strategy-card:hover {
  border-color: var(--brand-primary);
  box-shadow: var(--shadow-md);
  transform: translateY(-2px);
}

.strategy-card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--space-4);
}

.strategy-name {
  font-size: var(--text-xl);
  font-weight: var(--font-semibold);
  color: var(--text-primary);
}

.strategy-actions {
  display: flex;
  gap: var(--space-2);
}

.strategy-meta {
  display: flex;
  gap: var(--space-6);
  margin-bottom: var(--space-4);
  font-size: var(--text-sm);
  color: var(--text-secondary);
}

.strategy-metrics {
  display: flex;
  gap: var(--space-8);
  margin-bottom: var(--space-4);
}

.metric {
  display: flex;
  flex-direction: column;
}

.metric-label {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  margin-bottom: var(--space-1);
}

.metric-value {
  font-size: var(--text-lg);
  font-family: var(--font-mono);
  font-weight: var(--font-semibold);
}

.metric-positive {
  color: var(--success);
}

.metric-negative {
  color: var(--danger);
}
```

**状态徽章**：
- 运行中: 绿色（`--success`）
- 已停止: 灰色（`--text-tertiary`）
- 回测中: 蓝色（`--info`）
- 错误: 红色（`--danger`）

#### 4. 迷你收益曲线

**组件**: `MiniEquityCurve`

**使用库**: react-sparklines

**配置**：
- 高度: `40px`
- 线条颜色: 根据收益正负（绿色/红色）
- 无坐标轴，无网格

---

## 策略编辑器页面 (Strategy Editor)

### 页面概述

策略编辑器提供完整的策略开发环境，包括代码编辑、参数配置、回测运行等功能。

### 布局结构

```
+----------------------------------------------------------+
| [← 返回] MA交叉策略                   [保存] [回测] [运行]  |
+----------------------------------------------------------+
| [基本信息] [策略代码] [参数配置] [回测设置]                |
+----------------------------------------------------------+
|                                                          |
| +------------------------+  +--------------------------+ |
| |                        |  |                          | |
| | # 策略代码编辑器        |  | 参数配置                  | |
| | from hermesflow import |  |                          | |
| | import Strategy        |  | 快线周期: [12]           | |
| |                        |  | 慢线周期: [26]           | |
| | class MA Cross         |  | 止损: [2%]               | |
| | Strategy(Strategy):    |  |                          | |
| |     def __init__(self, |  | [优化参数]               | |
| |         fast=12,       |  |                          | |
| |         slow=26):      |  |                          | |
| |         ...            |  |                          | |
| |                        |  |                          | |
| | [语法高亮、自动补全]    |  |                          | |
| +------------------------+  +--------------------------+ |
|                                                          |
+----------------------------------------------------------+
```

### 组件详细设计

#### 1. 页面头部

**组件**: `StrategyEditorHeader`

**内容**：
- 左侧: 返回按钮 + 策略名称
- 右侧: 保存、回测、运行按钮

#### 2. Tab导航

**组件**: `StrategyEditorTabs`

**Tab项**：
- 基本信息: 策略名称、描述、类型、资产类别
- 策略代码: 代码编辑器
- 参数配置: 参数表单、参数优化入口
- 回测设置: 时间范围、初始资金、手续费等

#### 3. 代码编辑器

**组件**: `CodeEditor`

**使用库**: Monaco Editor

**配置**：
```typescript
{
  theme: 'vs-dark',  // 深色主题
  language: 'python',
  fontSize: 14,
  fontFamily: 'JetBrains Mono',
  minimap: { enabled: true },
  wordWrap: 'on',
  lineNumbers: 'on',
  automaticLayout: true,
  scrollBeyondLastLine: false,
  tabSize: 4,
  insertSpaces: true,
  autoIndent: 'full',
  suggest: {
    showMethods: true,
    showFunctions: true,
    showVariables: true
  }
}
```

**样式**：
- 宽度: 70%
- 高度: 填满剩余空间
- 背景: `--bg-secondary`

#### 4. 参数配置面板

**组件**: `ParameterConfigPanel`

**样式**：
- 宽度: 30%
- 表单输入框
- 优化参数按钮（跳转到优化器）

---

## 回测报告页面 (Backtest Report)

### 页面概述

回测报告展示策略的历史回测结果，包括关键指标、收益曲线、交易记录等详细信息。

### 布局结构

```
+----------------------------------------------------------+
| MA交叉策略 - 回测报告                                     |
| 回测周期: 2023-01-01 至 2024-12-01                       |
+----------------------------------------------------------+
|                                                          |
| +----------------------------------------------------------+
| | 关键指标                                               |
| |                                                        |
| | +-------------+ +-------------+ +-------------+        |
| | | 总收益      | | 年化收益     | | 夏普比率    |        |
| | | +45.3%     | | +38.2%      | | 1.85       |        |
| | +-------------+ +-------------+ +-------------+        |
| |                                                        |
| | +-------------+ +-------------+ +-------------+        |
| | | 最大回撤    | | 胜率        | | 盈亏比      |        |
| | | -12.4%     | | 62.5%       | | 2.1        |        |
| | +-------------+ +-------------+ +-------------+        |
| +----------------------------------------------------------+
|                                                          |
| +----------------------------------------------------------+
| | 收益曲线                                               |
| |                                                        |
| | $15K ┤               策略收益 ──────                   |
| | $12K ┤                          ╱────                  |
| | $9K  ┤                  ╱──────                        |
| | $6K  ┤          ╱──────                                |
| | $3K  ┤  ╱──────                                        |
| | $0   └──┴───┴───┴───┴───┴───┴───                     |
| |      Jan Mar May Jul Sep Nov                          |
| |                                                        |
| |      基准收益（Buy&Hold） - - - -                      |
| |      回撤区域 ░░░░                                     |
| |      交易点标记 ● 买入 ● 卖出                          |
| +----------------------------------------------------------+
|                                                          |
| +----------------------------------------------------------+
| | 交易记录                                     [导出CSV]  |
| |                                                        |
| | | 时间         | 方向 | 价格      | 数量 | 手续费 | PnL  |
| | |-------------|-----|----------|-----|--------|-------|
| | | 2023-01-05  | 买入 | $42,150  | 0.2 | $8.43  | -     |
| | | 2023-01-12  | 卖出 | $43,800  | 0.2 | $8.76  | +$321 |
| | | ...         | ... | ...      | ... | ...    | ...   |
| |                                                        |
| +----------------------------------------------------------+
|                                                          |
+----------------------------------------------------------+
```

### 组件详细设计

#### 1. 关键指标网格

**组件**: `MetricsGrid`

**布局**: 2x3网格

**指标卡片样式**：
```css
.metric-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-lg);
  padding: var(--space-6);
  text-align: center;
  transition: all 0.2s;
}

.metric-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
}

.metric-card-label {
  font-size: var(--text-sm);
  color: var(--text-secondary);
  margin-bottom: var(--space-2);
}

.metric-card-value {
  font-size: var(--text-3xl);
  font-family: var(--font-mono);
  font-weight: var(--font-bold);
  color: var(--text-primary);
}

.metric-card-value.positive {
  color: var(--success);
}

.metric-card-value.negative {
  color: var(--danger);
}

.metric-card-tooltip {
  margin-top: var(--space-2);
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}
```

#### 2. 收益曲线图

**组件**: `EquityCurveChart`

**配置**：
```typescript
{
  type: 'line',
  series: [
    {
      name: '策略收益',
      data: strategyEquity,
      color: '#0ECB81',
      width: 2
    },
    {
      name: '基准收益',
      data: benchmarkEquity,
      color: '#848E9C',
      width: 1,
      dashArray: '5 5'
    }
  ],
  annotations: [
    {
      type: 'area',  // 回撤区域
      y: drawdownPeriods,
      fillColor: 'rgba(246, 70, 93, 0.1)'
    },
    {
      type: 'points',  // 交易点
      data: trades,
      marker: {
        size: 6,
        colors: ['#0ECB81', '#F6465D']  // 买入绿、卖出红
      }
    }
  ]
}
```

#### 3. 交易记录表格

**组件**: `TradesTable`

**功能**：
- 分页（每页50条）
- 排序（按时间、PnL）
- 筛选（多头/空头）
- 导出CSV

**样式**：
- 使用表格组件规范
- 盈利行高亮绿色背景（rgba(14, 203, 129, 0.05)）
- 亏损行高亮红色背景（rgba(246, 70, 93, 0.05)）

---

## 交易监控页面 (Trading)

### 页面概述

交易监控页面实时显示当前订单、持仓和交易历史，支持快速操作。

### 布局结构

```
+----------------------------------------------------------+
| [订单] [持仓] [交易历史]                                  |
+----------------------------------------------------------+
|                                                          |
| 当前订单 (3)                                              |
| +----------------------------------------------------------+
| | BTCUSDT | 买入 | 限价 | $43,200 | 0.5 | [等待中] [取消] |
| | ETHUSDT | 卖出 | 市价 | -       | 2.0 | [等待中] [取消] |
| | BNBUSDT | 买入 | 限价 | $305    | 10  | [部分成交] [取消] |
| +----------------------------------------------------------+
|                                                          |
| 当前持仓 (2)                                              |
| +----------------------------------------------------------+
| | BTCUSDT | 0.5 | $43,000 | $43,250 | +$125 (+5.8%) | [平仓] |
| | ETHUSDT | 2.0 | $2,200  | $2,280  | +$160 (+7.3%) | [平仓] |
| +----------------------------------------------------------+
|                                                          |
| 持仓占比                                                  |
| [饼图: BTC 60%, ETH 40%]                                 |
|                                                          |
+----------------------------------------------------------+
```

### 组件详细设计

#### 1. Tab切换

**组件**: `TradingTabs`

**Tab项**：
- 订单: 当前活跃订单
- 持仓: 当前持仓
- 交易历史: 已完成交易

#### 2. 订单表格

**组件**: `OrdersTable`

**实时更新**：
- 使用WebSocket订阅订单状态
- 状态变化时闪烁动画（pulse）

**状态徽章**：
- 等待中: 蓝色
- 部分成交: 黄色
- 已成交: 绿色
- 已取消: 灰色

#### 3. 持仓卡片

**组件**: `PositionsCard`

**样式**：
- 盈利持仓左侧绿色边框
- 亏损持仓左侧红色边框
- 盈亏百分比大字显示

#### 4. 持仓占比饼图

**组件**: `PositionPieChart`

**使用库**: Recharts

**配置**：
- 颜色自动分配
- Hover显示详细信息

---

## 风控监控页面 (Risk Management)

### 页面概述

风控监控页面实时显示账户和策略的风险指标，以及风控规则和告警历史。

### 布局结构

```
+----------------------------------------------------------+
| 风险监控                                 [风险等级: 低 🟢] |
+----------------------------------------------------------+
|                                                          |
| +------------------------+  +--------------------------+ |
| | 账户风险指标            |  | 策略风险指标              | |
| |                        |  |                          | |
| | 杠杆率                 |  | 单策略最大回撤:           | |
| | [圆形进度条 1.2x/5x]   |  | MA策略: -8.2%            | |
| |                        |  | RSI策略: -12.5%          | |
| | 可用保证金             |  | 趋势策略: -5.8%          | |
| | [圆形进度条 80%]       |  |                          | |
| +------------------------+  +--------------------------+ |
|                                                          |
| +----------------------------------------------------------+
| | 风控规则                                     [编辑规则]  |
| |                                                        |
| | ✓ 单笔最大亏损不超过2%                                  |
| | ✓ 日总亏损不超过5%                                      |
| | ⚠ 单策略回撤超过10%自动停止 (已触发1次)                 |
| | ✓ 杠杆率不超过5倍                                       |
| +----------------------------------------------------------+
|                                                          |
| +----------------------------------------------------------+
| | 告警历史                                               |
| |                                                        |
| | | 时间       | 类型     | 描述                  | 状态  |
| | |-----------|---------|----------------------|-------|
| | | 12-20 10:05 | 回撤警告 | MA策略回撤超10%      | 已处理 |
| | | 12-19 15:30 | 杠杆警告 | 杠杆率接近上限       | 已处理 |
| | | ...       | ...     | ...                  | ...   |
| +----------------------------------------------------------+
|                                                          |
+----------------------------------------------------------+
```

### 组件详细设计

#### 1. 风险等级徽章

**组件**: `RiskLevelBadge`

**样式**：
```css
.risk-level-badge {
  padding: var(--space-2) var(--space-4);
  border-radius: var(--radius-md);
  font-size: var(--text-lg);
  font-weight: var(--font-semibold);
}

.risk-level-low {
  background: rgba(14, 203, 129, 0.1);
  color: var(--success);
}

.risk-level-medium {
  background: rgba(240, 185, 11, 0.1);
  color: var(--warning);
}

.risk-level-high {
  background: rgba(246, 70, 93, 0.1);
  color: var(--danger);
}
```

#### 2. 圆形进度条

**组件**: `CircularProgress`

**使用库**: react-circular-progressbar

**配置**：
```typescript
{
  value: 1.2,
  maxValue: 5,
  text: '1.2x',
  styles: {
    path: {
      stroke: value < 3 ? '#0ECB81' : value < 4 ? '#F0B90B' : '#F6465D',
      strokeWidth: 8
    },
    trail: {
      stroke: '#2B3139'
    },
    text: {
      fill: '#EAECEF',
      fontSize: '20px',
      fontFamily: 'JetBrains Mono'
    }
  }
}
```

#### 3. 风控规则列表

**组件**: `RiskRulesList`

**样式**：
- 每条规则一行
- 左侧复选框图标（✓ 或 ⚠）
- 触发次数显示在右侧（灰色小字）

#### 4. 告警历史表格

**组件**: `AlertHistoryTable`

**样式**：
- 使用表格组件规范
- 状态列使用徽章
- 按时间倒序排列

---

## 设置页面 (Settings)

### 页面概述

设置页面提供个人资料、API密钥、通知设置和系统配置。

### 布局结构

```
+----------------------------------------------------------+
| [个人资料] [API密钥] [通知设置] [系统配置]                |
+----------------------------------------------------------+
|                                                          |
| API密钥管理                                               |
|                                                          |
| +----------------------------------------------------------+
| | [Binance Logo]                                         |
| | Binance API                                            |
| | API Key: *********************                         |
| | [已连接 🟢]                                             |
| | [编辑] [删除]                                           |
| +----------------------------------------------------------+
|                                                          |
| +----------------------------------------------------------+
| | [OKX Logo]                                             |
| | OKX API                                                |
| | API Key: *********************                         |
| | [断开连接 🔴]                                           |
| | [编辑] [删除]                                           |
| +----------------------------------------------------------+
|                                                          |
| [+ 添加新API密钥]                                         |
|                                                          |
+----------------------------------------------------------+
```

### 组件详细设计

#### 1. Tab导航

**组件**: `SettingsTabs`

**样式**：
- 左侧垂直Tab（类似侧边栏）
- 宽度: 200px
- 右侧内容区域

#### 2. API密钥卡片

**组件**: `ApiKeyCard`

**样式**：
```css
.api-key-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-lg);
  padding: var(--space-6);
  margin-bottom: var(--space-4);
  display: flex;
  align-items: center;
  gap: var(--space-4);
}

.api-key-logo {
  width: 48px;
  height: 48px;
  border-radius: var(--radius-md);
}

.api-key-info {
  flex: 1;
}

.api-key-name {
  font-size: var(--text-lg);
  font-weight: var(--font-semibold);
  margin-bottom: var(--space-2);
}

.api-key-value {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--text-secondary);
}

.api-key-status {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  margin-top: var(--space-2);
}

.status-connected {
  color: var(--success);
}

.status-disconnected {
  color: var(--danger);
}
```

**安全操作确认**：
- 删除操作需要二次确认Modal
- 编辑操作需要输入密码验证

---

## 通用设计规范

### Loading状态

**Skeleton屏幕**：
```css
.skeleton {
  background: linear-gradient(
    90deg,
    var(--bg-tertiary) 25%,
    var(--bg-elevated) 50%,
    var(--bg-tertiary) 75%
  );
  background-size: 200% 100%;
  animation: loading 1.5s ease-in-out infinite;
}

@keyframes loading {
  0% {
    background-position: 200% 0;
  }
  100% {
    background-position: -200% 0;
  }
}
```

### 空状态

**组件**: `EmptyState`

**内容**：
- 大图标（64px）
- 标题文字
- 描述文字
- 操作按钮（如"创建第一个策略"）

### 错误状态

**组件**: `ErrorState`

**内容**：
- 错误图标（AlertTriangle）
- 错误信息
- 错误代码（可选）
- "重试"按钮

### Toast通知

**位置**: 右上角

**类型**：
- Success: 绿色，CheckCircle图标
- Error: 红色，XCircle图标
- Warning: 黄色，AlertTriangle图标
- Info: 蓝色，Info图标

**动画**: 从右侧滑入，3秒后自动消失

---

## 附录

### 设计资源

- **Figma原型**: [待补充]
- **交互演示**: [待补充]

### 参考案例

- TradingView: 专业图表工具
- Binance: 交易界面设计
- QuantConnect: 策略开发界面

---

**文档维护**: HermesFlow Design Team  
**反馈联系**: design@hermesflow.com

