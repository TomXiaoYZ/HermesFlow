# Hermesflow量化交易平台 UI 实现 Prompt

> 📌 **用途**: 本文档为Lovable/V0等AI驱动的UI生成工具提供完整的实现指南

**目标平台**: Lovable.dev / Vercel V0  
**版本**: v1.0.0  
**最后更新**: 2024-12-20

---

## 项目概述

创建一个专业的量化交易平台Web应用，采用深色主题，设计风格类似Binance和TradingView，为量化交易者提供策略开发、回测、实盘交易的完整工具链。

**核心特点**：
- 深色主题，长时间使用护眼
- 金融级数据密度
- 实时数据更新
- 专业交易工具感

---

## 技术栈

### 前端框架
```json
{
  "framework": "React 18 + TypeScript",
  "styling": "TailwindCSS",
  "charting": "Recharts / Tremor",
  "icons": "Lucide React",
  "animation": "Framer Motion",
  "codeEditor": "Monaco Editor"
}
```

### 推荐依赖包

```json
{
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "typescript": "^5.0.0",
    "tailwindcss": "^3.3.0",
    "recharts": "^2.10.0",
    "@tremor/react": "^3.13.0",
    "lucide-react": "^0.300.0",
    "framer-motion": "^10.16.0",
    "@monaco-editor/react": "^4.6.0",
    "react-sparklines": "^1.7.0",
    "react-circular-progressbar": "^2.1.0"
  }
}
```

---

## 设计系统

### TailwindCSS配置

在`tailwind.config.js`中添加以下自定义配置：

```javascript
module.exports = {
  theme: {
    extend: {
      colors: {
        // 背景色
        bg: {
          primary: '#0B0E11',
          secondary: '#161A1E',
          tertiary: '#1E2329',
          elevated: '#2B3139',
        },
        // 文字色
        text: {
          primary: '#EAECEF',
          secondary: '#848E9C',
          tertiary: '#5E6673',
          disabled: '#474D57',
        },
        // 边框色
        border: {
          primary: '#2B3139',
          secondary: '#1E2329',
        },
        // 品牌色
        brand: {
          primary: '#FCD535',
          secondary: '#F0B90B',
        },
        // 功能色
        success: '#0ECB81',
        danger: '#F6465D',
        warning: '#F0B90B',
        info: '#3DCFFF',
      },
      fontFamily: {
        sans: ['Inter', 'sans-serif'],
        mono: ['JetBrains Mono', 'monospace'],
      },
      spacing: {
        1: '0.25rem',
        2: '0.5rem',
        3: '0.75rem',
        4: '1rem',
        5: '1.25rem',
        6: '1.5rem',
        8: '2rem',
        10: '2.5rem',
        12: '3rem',
        16: '4rem',
      },
      borderRadius: {
        sm: '4px',
        md: '6px',
        lg: '8px',
        xl: '12px',
      },
      boxShadow: {
        sm: '0 1px 2px rgba(0, 0, 0, 0.3)',
        md: '0 4px 6px rgba(0, 0, 0, 0.4)',
        lg: '0 10px 15px rgba(0, 0, 0, 0.5)',
        xl: '0 20px 25px rgba(0, 0, 0, 0.6)',
      },
    },
  },
  darkMode: 'class',
}
```

### 全局样式

在`globals.css`中添加：

```css
@tailwind base;
@tailwind components;
@tailwind utilities;

body {
  @apply bg-bg-primary text-text-primary font-sans;
}

/* 滚动条样式 */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  @apply bg-bg-secondary;
}

::-webkit-scrollbar-thumb {
  @apply bg-bg-elevated rounded-full;
}

::-webkit-scrollbar-thumb:hover {
  @apply bg-border-primary;
}
```

---

## 页面实现要求

### 1. Dashboard (首页)

#### 布局结构
- 顶部导航栏（固定）
- 4个统计卡片（2x2网格）
- 资金曲线图（大卡片）
- 运行中策略列表 + 实时行情（2列）

#### 组件实现

**1.1 顶部导航栏**
```tsx
<nav className="bg-bg-secondary border-b border-border-primary h-16 flex items-center justify-between px-8">
  {/* 左侧 */}
  <div className="flex items-center gap-4">
    <div className="text-2xl font-bold text-brand-primary">HermesFlow</div>
    <div className="flex gap-6 ml-8">
      <a className="text-text-primary hover:text-brand-primary">Dashboard</a>
      <a className="text-text-secondary hover:text-text-primary">策略</a>
      <a className="text-text-secondary hover:text-text-primary">交易</a>
      <a className="text-text-secondary hover:text-text-primary">风控</a>
    </div>
  </div>
  
  {/* 右侧 */}
  <div className="flex items-center gap-4">
    <Bell className="w-5 h-5 text-text-secondary cursor-pointer hover:text-text-primary" />
    <Settings className="w-5 h-5 text-text-secondary cursor-pointer hover:text-text-primary" />
    <div className="w-8 h-8 rounded-full bg-brand-primary"></div>
  </div>
</nav>
```

**1.2 账户总值卡片**
```tsx
<div className="bg-gradient-to-br from-brand-primary to-brand-secondary rounded-lg p-6 shadow-md">
  <div className="flex items-center justify-between mb-2">
    <span className="text-bg-primary text-sm opacity-80">账户总值</span>
    <Wallet className="w-6 h-6 text-bg-primary opacity-60" />
  </div>
  <div className="text-4xl font-mono font-bold text-bg-primary">
    $125,430.50
  </div>
</div>
```

**1.3 今日盈亏卡片**
```tsx
<div className="bg-bg-secondary border border-border-primary rounded-lg p-6 hover:border-success hover:shadow-md transition-all hover:-translate-y-0.5">
  <div className="flex items-center justify-between mb-2">
    <span className="text-text-secondary text-sm">今日盈亏</span>
    <TrendingUp className="w-5 h-5 text-success" />
  </div>
  <div className="text-2xl font-mono font-semibold text-success">
    +$2,340.80
    <span className="text-lg ml-2">(+1.9%)</span>
  </div>
</div>
```

**1.4 资金曲线图**
```tsx
<div className="bg-bg-secondary border border-border-primary rounded-lg p-6">
  <h3 className="text-lg font-semibold mb-4">资金曲线（7天）</h3>
  <ResponsiveContainer width="100%" height={300}>
    <AreaChart data={equityData}>
      <defs>
        <linearGradient id="colorEquity" x1="0" y1="0" x2="0" y2="1">
          <stop offset="5%" stopColor="#0ECB81" stopOpacity={0.2}/>
          <stop offset="95%" stopColor="#0ECB81" stopOpacity={0}/>
        </linearGradient>
      </defs>
      <CartesianGrid strokeDasharray="3 3" stroke="#2B3139" />
      <XAxis dataKey="date" stroke="#848E9C" />
      <YAxis stroke="#848E9C" tickFormatter={(value) => `$${value/1000}K`} />
      <Tooltip 
        contentStyle={{background: '#2B3139', border: '1px solid #3B4149'}}
        labelStyle={{color: '#EAECEF'}}
      />
      <Area 
        type="monotone" 
        dataKey="value" 
        stroke="#0ECB81" 
        strokeWidth={2}
        fill="url(#colorEquity)" 
      />
    </AreaChart>
  </ResponsiveContainer>
</div>
```

**1.5 运行中策略列表**
```tsx
<div className="bg-bg-secondary border border-border-primary rounded-lg p-6">
  <h3 className="text-lg font-semibold mb-4">运行中策略 (3)</h3>
  <div className="space-y-3">
    {strategies.map(strategy => (
      <div 
        key={strategy.id}
        className="bg-bg-tertiary rounded-md p-4 hover:bg-bg-elevated transition-all hover:translate-x-1 cursor-pointer"
      >
        <div className="flex items-center justify-between mb-2">
          <span className="font-medium">{strategy.name}</span>
          <span className="px-2 py-1 rounded-full text-xs bg-success/10 text-success">
            运行中
          </span>
        </div>
        <div className="text-success font-mono text-sm">
          {strategy.pnl}
        </div>
      </div>
    ))}
  </div>
</div>
```

**1.6 实时行情卡片**
```tsx
<div className="bg-bg-secondary border border-border-primary rounded-lg p-6">
  <h3 className="text-lg font-semibold mb-4">实时行情</h3>
  <div className="space-y-4">
    <div className="border-b border-border-primary pb-3">
      <div className="flex items-center justify-between mb-1">
        <span className="text-text-secondary">BTC</span>
        <TrendingUp className="w-4 h-4 text-success" />
      </div>
      <div className="text-xl font-mono font-semibold">$43,250.50</div>
      <div className="text-sm text-success">24h: +2.5%</div>
      {/* 迷你K线图 */}
      <div className="mt-2 h-8">
        <Sparklines data={btcSparklineData} width={100} height={32}>
          <SparklinesLine color="#0ECB81" />
        </Sparklines>
      </div>
    </div>
    {/* ETH类似 */}
  </div>
</div>
```

#### Mock数据示例

```typescript
const equityData = [
  { date: 'Mon', value: 110000 },
  { date: 'Tue', value: 115000 },
  { date: 'Wed', value: 118000 },
  { date: 'Thu', value: 122000 },
  { date: 'Fri', value: 123000 },
  { date: 'Sat', value: 124500 },
  { date: 'Sun', value: 125430 },
];

const strategies = [
  { id: 1, name: 'MA交叉策略', pnl: '+15.3%' },
  { id: 2, name: 'RSI突破策略', pnl: '+8.1%' },
  { id: 3, name: '趋势跟踪', pnl: '+12.7%' },
];

const btcSparklineData = [42500, 42800, 43000, 42700, 43100, 43250];
```

---

### 2. Strategies (策略列表页)

#### 布局结构
- 页面头部（标题 + 新建按钮）
- 筛选栏（搜索、筛选、排序）
- 策略卡片列表
- 分页

#### 组件实现

**2.1 页面头部**
```tsx
<div className="flex items-center justify-between mb-8">
  <h1 className="text-3xl font-bold">策略列表</h1>
  <button className="px-6 py-3 bg-brand-primary text-bg-primary rounded-md font-semibold hover:bg-brand-secondary transition-colors flex items-center gap-2">
    <Plus className="w-5 h-5" />
    新建策略
  </button>
</div>
```

**2.2 筛选栏**
```tsx
<div className="flex gap-4 mb-6">
  {/* 搜索框 */}
  <div className="flex-1 max-w-md relative">
    <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-text-tertiary" />
    <input 
      type="text"
      placeholder="搜索策略..."
      className="w-full bg-bg-tertiary border border-border-primary rounded-md pl-10 pr-4 py-2 text-sm focus:border-brand-primary focus:outline-none"
    />
  </div>
  
  {/* 筛选下拉 */}
  <select className="bg-bg-tertiary border border-border-primary rounded-md px-4 py-2 text-sm">
    <option>全部</option>
    <option>运行中</option>
    <option>已停止</option>
  </select>
  
  {/* 排序下拉 */}
  <select className="bg-bg-tertiary border border-border-primary rounded-md px-4 py-2 text-sm">
    <option>收益</option>
    <option>夏普比率</option>
    <option>回撤</option>
  </select>
</div>
```

**2.3 策略卡片**
```tsx
<div className="bg-bg-secondary border border-border-primary rounded-lg p-6 hover:border-brand-primary hover:shadow-md transition-all hover:-translate-y-1 cursor-pointer">
  {/* 头部 */}
  <div className="flex items-center justify-between mb-4">
    <h3 className="text-xl font-semibold">MA交叉策略</h3>
    <div className="flex items-center gap-2">
      <span className="px-3 py-1 rounded-full text-xs bg-success/10 text-success">
        运行中
      </span>
      <button className="p-2 hover:bg-bg-elevated rounded-md">
        <Edit2 className="w-4 h-4" />
      </button>
      <button className="p-2 hover:bg-bg-elevated rounded-md">
        <MoreVertical className="w-4 h-4" />
      </button>
    </div>
  </div>
  
  {/* 元数据 */}
  <div className="flex gap-6 mb-4 text-sm text-text-secondary">
    <span>类型: 趋势跟踪</span>
    <span>资产: 加密货币</span>
  </div>
  
  {/* 指标 */}
  <div className="flex gap-8 mb-4">
    <div>
      <div className="text-xs text-text-tertiary mb-1">收益</div>
      <div className="text-lg font-mono font-semibold text-success">+15.3%</div>
    </div>
    <div>
      <div className="text-xs text-text-tertiary mb-1">夏普</div>
      <div className="text-lg font-mono font-semibold">1.8</div>
    </div>
    <div>
      <div className="text-xs text-text-tertiary mb-1">回撤</div>
      <div className="text-lg font-mono font-semibold text-danger">-8.2%</div>
    </div>
  </div>
  
  {/* 迷你收益曲线 */}
  <div className="h-10">
    <Sparklines data={strategyEquityData} width={200} height={40}>
      <SparklinesLine color="#0ECB81" />
    </Sparklines>
  </div>
</div>
```

---

### 3. Strategy Editor (策略编辑器)

#### 布局结构
- 顶部操作栏（返回、保存、回测、运行）
- Tab导航（基本信息、策略代码、参数配置、回测设置）
- 主内容区（代码编辑器 + 参数面板，7:3分栏）

#### 组件实现

**3.1 顶部操作栏**
```tsx
<div className="bg-bg-secondary border-b border-border-primary px-8 py-4 flex items-center justify-between">
  <div className="flex items-center gap-4">
    <button className="p-2 hover:bg-bg-elevated rounded-md">
      <ArrowLeft className="w-5 h-5" />
    </button>
    <h2 className="text-xl font-semibold">MA交叉策略</h2>
  </div>
  <div className="flex items-center gap-3">
    <button className="px-4 py-2 bg-bg-tertiary border border-border-primary rounded-md hover:border-brand-primary transition-colors">
      保存
    </button>
    <button className="px-4 py-2 bg-info text-white rounded-md hover:bg-info/80 transition-colors">
      回测
    </button>
    <button className="px-4 py-2 bg-success text-white rounded-md hover:bg-success/80 transition-colors">
      运行
    </button>
  </div>
</div>
```

**3.2 Tab导航**
```tsx
<div className="border-b border-border-primary">
  <div className="flex gap-8 px-8">
    <button className="py-3 border-b-2 border-brand-primary text-brand-primary">
      策略代码
    </button>
    <button className="py-3 border-b-2 border-transparent text-text-secondary hover:text-text-primary">
      参数配置
    </button>
    <button className="py-3 border-b-2 border-transparent text-text-secondary hover:text-text-primary">
      回测设置
    </button>
  </div>
</div>
```

**3.3 代码编辑器（使用Monaco Editor）**
```tsx
import Editor from '@monaco-editor/react';

<div className="h-full">
  <Editor
    height="100%"
    defaultLanguage="python"
    theme="vs-dark"
    value={code}
    onChange={setCode}
    options={{
      fontSize: 14,
      fontFamily: 'JetBrains Mono',
      minimap: { enabled: true },
      wordWrap: 'on',
      lineNumbers: 'on',
      automaticLayout: true,
      scrollBeyondLastLine: false,
      tabSize: 4,
    }}
  />
</div>
```

**3.4 参数配置面板**
```tsx
<div className="bg-bg-secondary border-l border-border-primary p-6">
  <h3 className="text-lg font-semibold mb-4">参数配置</h3>
  <div className="space-y-4">
    <div>
      <label className="block text-sm text-text-secondary mb-2">快线周期</label>
      <input 
        type="number"
        value={fastPeriod}
        className="w-full bg-bg-tertiary border border-border-primary rounded-md px-4 py-2 font-mono"
      />
    </div>
    <div>
      <label className="block text-sm text-text-secondary mb-2">慢线周期</label>
      <input 
        type="number"
        value={slowPeriod}
        className="w-full bg-bg-tertiary border border-border-primary rounded-md px-4 py-2 font-mono"
      />
    </div>
    <button className="w-full px-4 py-2 bg-brand-primary text-bg-primary rounded-md font-semibold hover:bg-brand-secondary transition-colors">
      优化参数
    </button>
  </div>
</div>
```

---

### 4. Backtest Report (回测报告)

#### 组件实现

**4.1 关键指标网格**
```tsx
<div className="grid grid-cols-3 gap-6 mb-8">
  {[
    { label: '总收益', value: '+45.3%', positive: true },
    { label: '年化收益', value: '+38.2%', positive: true },
    { label: '夏普比率', value: '1.85', positive: true },
    { label: '最大回撤', value: '-12.4%', positive: false },
    { label: '胜率', value: '62.5%', positive: true },
    { label: '盈亏比', value: '2.1', positive: true },
  ].map(metric => (
    <div 
      key={metric.label}
      className="bg-bg-secondary border border-border-primary rounded-lg p-6 text-center hover:-translate-y-1 hover:shadow-md transition-all"
    >
      <div className="text-sm text-text-secondary mb-2">{metric.label}</div>
      <div className={`text-3xl font-mono font-bold ${metric.positive ? 'text-success' : 'text-danger'}`}>
        {metric.value}
      </div>
    </div>
  ))}
</div>
```

**4.2 收益曲线图（带对比和标注）**
```tsx
<div className="bg-bg-secondary border border-border-primary rounded-lg p-6 mb-8">
  <h3 className="text-lg font-semibold mb-4">收益曲线</h3>
  <ResponsiveContainer width="100%" height={400}>
    <LineChart data={backtestData}>
      <CartesianGrid strokeDasharray="3 3" stroke="#2B3139" />
      <XAxis dataKey="date" stroke="#848E9C" />
      <YAxis stroke="#848E9C" tickFormatter={(value) => `$${value}`} />
      <Tooltip 
        contentStyle={{background: '#2B3139', border: '1px solid #3B4149'}}
        labelStyle={{color: '#EAECEF'}}
      />
      <Legend />
      
      {/* 策略收益 */}
      <Line 
        type="monotone" 
        dataKey="strategy" 
        stroke="#0ECB81" 
        strokeWidth={2}
        name="策略收益"
        dot={false}
      />
      
      {/* 基准收益 */}
      <Line 
        type="monotone" 
        dataKey="benchmark" 
        stroke="#848E9C" 
        strokeWidth={1}
        strokeDasharray="5 5"
        name="基准收益"
        dot={false}
      />
      
      {/* 回撤区域可使用ReferenceArea */}
    </LineChart>
  </ResponsiveContainer>
</div>
```

**4.3 交易记录表格**
```tsx
<div className="bg-bg-secondary border border-border-primary rounded-lg p-6">
  <div className="flex items-center justify-between mb-4">
    <h3 className="text-lg font-semibold">交易记录</h3>
    <button className="px-4 py-2 bg-bg-tertiary border border-border-primary rounded-md hover:border-brand-primary transition-colors flex items-center gap-2">
      <Download className="w-4 h-4" />
      导出CSV
    </button>
  </div>
  
  <table className="w-full">
    <thead>
      <tr className="bg-bg-tertiary text-text-secondary text-sm">
        <th className="px-4 py-3 text-left">时间</th>
        <th className="px-4 py-3 text-left">方向</th>
        <th className="px-4 py-3 text-right">价格</th>
        <th className="px-4 py-3 text-right">数量</th>
        <th className="px-4 py-3 text-right">手续费</th>
        <th className="px-4 py-3 text-right">PnL</th>
      </tr>
    </thead>
    <tbody>
      {trades.map(trade => (
        <tr 
          key={trade.id}
          className={`border-b border-border-primary hover:bg-bg-tertiary transition-colors ${trade.pnl > 0 ? 'bg-success/5' : 'bg-danger/5'}`}
        >
          <td className="px-4 py-3 text-sm font-mono">{trade.time}</td>
          <td className="px-4 py-3">
            <span className={`px-2 py-1 rounded text-xs ${trade.side === 'buy' ? 'bg-success/10 text-success' : 'bg-danger/10 text-danger'}`}>
              {trade.side === 'buy' ? '买入' : '卖出'}
            </span>
          </td>
          <td className="px-4 py-3 text-right font-mono">${trade.price}</td>
          <td className="px-4 py-3 text-right font-mono">{trade.quantity}</td>
          <td className="px-4 py-3 text-right font-mono">${trade.commission}</td>
          <td className={`px-4 py-3 text-right font-mono font-semibold ${trade.pnl > 0 ? 'text-success' : 'text-danger'}`}>
            {trade.pnl > 0 ? '+' : ''}${trade.pnl}
          </td>
        </tr>
      ))}
    </tbody>
  </table>
</div>
```

---

### 5. Trading (交易监控)

#### 组件实现

**5.1 Tab切换**
```tsx
<div className="border-b border-border-primary mb-6">
  <div className="flex gap-8">
    <button className="py-3 border-b-2 border-brand-primary text-brand-primary">
      订单
    </button>
    <button className="py-3 border-b-2 border-transparent text-text-secondary hover:text-text-primary">
      持仓
    </button>
    <button className="py-3 border-b-2 border-transparent text-text-secondary hover:text-text-primary">
      交易历史
    </button>
  </div>
</div>
```

**5.2 订单表格（实时更新样式）**
```tsx
<div className="bg-bg-secondary border border-border-primary rounded-lg p-6">
  <h3 className="text-lg font-semibold mb-4">当前订单 (3)</h3>
  <table className="w-full">
    <thead>
      <tr className="bg-bg-tertiary text-text-secondary text-sm">
        <th className="px-4 py-3 text-left">交易对</th>
        <th className="px-4 py-3 text-left">方向</th>
        <th className="px-4 py-3 text-left">类型</th>
        <th className="px-4 py-3 text-right">价格</th>
        <th className="px-4 py-3 text-right">数量</th>
        <th className="px-4 py-3 text-center">状态</th>
        <th className="px-4 py-3 text-center">操作</th>
      </tr>
    </thead>
    <tbody>
      {orders.map(order => (
        <tr 
          key={order.id}
          className={`border-b border-border-primary hover:bg-bg-tertiary transition-colors ${order.isNew ? 'animate-pulse' : ''}`}
        >
          <td className="px-4 py-3 font-mono">{order.symbol}</td>
          <td className="px-4 py-3">
            <span className={`px-2 py-1 rounded text-xs ${order.side === 'buy' ? 'bg-success/10 text-success' : 'bg-danger/10 text-danger'}`}>
              {order.side === 'buy' ? '买入' : '卖出'}
            </span>
          </td>
          <td className="px-4 py-3 text-text-secondary">{order.type}</td>
          <td className="px-4 py-3 text-right font-mono">${order.price}</td>
          <td className="px-4 py-3 text-right font-mono">{order.quantity}</td>
          <td className="px-4 py-3 text-center">
            <span className={`px-2 py-1 rounded-full text-xs ${
              order.status === 'pending' ? 'bg-info/10 text-info' :
              order.status === 'partial' ? 'bg-warning/10 text-warning' :
              'bg-success/10 text-success'
            }`}>
              {order.status === 'pending' ? '等待中' :
               order.status === 'partial' ? '部分成交' : '已成交'}
            </span>
          </td>
          <td className="px-4 py-3 text-center">
            <button className="px-3 py-1 bg-danger text-white rounded-md text-sm hover:bg-danger/80">
              取消
            </button>
          </td>
        </tr>
      ))}
    </tbody>
  </table>
</div>
```

**5.3 持仓卡片（带盈亏标识）**
```tsx
<div className="bg-bg-secondary border border-border-primary rounded-lg p-6">
  <h3 className="text-lg font-semibold mb-4">当前持仓 (2)</h3>
  <div className="space-y-4">
    {positions.map(position => (
      <div 
        key={position.symbol}
        className={`border-l-4 ${position.pnl > 0 ? 'border-success' : 'border-danger'} bg-bg-tertiary rounded-md p-4`}
      >
        <div className="flex items-center justify-between mb-3">
          <div>
            <div className="font-mono font-semibold text-lg">{position.symbol}</div>
            <div className="text-sm text-text-secondary">持仓: {position.quantity}</div>
          </div>
          <div className="text-right">
            <div className={`text-2xl font-mono font-bold ${position.pnl > 0 ? 'text-success' : 'text-danger'}`}>
              {position.pnl > 0 ? '+' : ''}${position.pnl}
            </div>
            <div className={`text-sm ${position.pnlPct > 0 ? 'text-success' : 'text-danger'}`}>
              ({position.pnlPct > 0 ? '+' : ''}{position.pnlPct}%)
            </div>
          </div>
        </div>
        
        <div className="flex gap-6 text-sm mb-3">
          <div>
            <span className="text-text-tertiary">成本: </span>
            <span className="font-mono">${position.avgPrice}</span>
          </div>
          <div>
            <span className="text-text-tertiary">现价: </span>
            <span className="font-mono">${position.currentPrice}</span>
          </div>
        </div>
        
        <button className="w-full px-4 py-2 bg-danger text-white rounded-md hover:bg-danger/80 transition-colors">
          平仓
        </button>
      </div>
    ))}
  </div>
  
  {/* 持仓占比饼图 */}
  <div className="mt-6">
    <h4 className="text-sm font-semibold mb-3">持仓占比</h4>
    <div className="h-48">
      <ResponsiveContainer width="100%" height="100%">
        <PieChart>
          <Pie
            data={positionPieData}
            cx="50%"
            cy="50%"
            innerRadius={60}
            outerRadius={80}
            fill="#8884d8"
            paddingAngle={5}
            dataKey="value"
          >
            {positionPieData.map((entry, index) => (
              <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
            ))}
          </Pie>
          <Tooltip />
          <Legend />
        </PieChart>
      </ResponsiveContainer>
    </div>
  </div>
</div>
```

---

### 6. Risk Management (风控监控)

#### 组件实现

**6.1 风险等级徽章**
```tsx
<div className="flex items-center justify-between mb-8">
  <h1 className="text-3xl font-bold">风险监控</h1>
  <div className="px-4 py-2 rounded-md bg-success/10 text-success text-lg font-semibold flex items-center gap-2">
    <div className="w-3 h-3 rounded-full bg-success"></div>
    风险等级: 低
  </div>
</div>
```

**6.2 圆形进度条（使用react-circular-progressbar）**
```tsx
import { CircularProgressbar, buildStyles } from 'react-circular-progressbar';
import 'react-circular-progressbar/dist/styles.css';

<div className="bg-bg-secondary border border-border-primary rounded-lg p-6">
  <h3 className="text-lg font-semibold mb-6">账户风险指标</h3>
  
  <div className="grid grid-cols-2 gap-6">
    <div>
      <div className="w-32 h-32 mx-auto mb-3">
        <CircularProgressbar
          value={1.2}
          maxValue={5}
          text="1.2x"
          styles={buildStyles({
            pathColor: '#0ECB81',
            textColor: '#EAECEF',
            trailColor: '#2B3139',
            textSize: '20px',
          })}
        />
      </div>
      <div className="text-center text-sm text-text-secondary">杠杆率</div>
    </div>
    
    <div>
      <div className="w-32 h-32 mx-auto mb-3">
        <CircularProgressbar
          value={80}
          maxValue={100}
          text="80%"
          styles={buildStyles({
            pathColor: '#0ECB81',
            textColor: '#EAECEF',
            trailColor: '#2B3139',
            textSize: '20px',
          })}
        />
      </div>
      <div className="text-center text-sm text-text-secondary">可用保证金</div>
    </div>
  </div>
</div>
```

**6.3 风控规则列表**
```tsx
<div className="bg-bg-secondary border border-border-primary rounded-lg p-6">
  <div className="flex items-center justify-between mb-4">
    <h3 className="text-lg font-semibold">风控规则</h3>
    <button className="px-3 py-1 bg-bg-elevated border border-border-primary rounded-md text-sm hover:border-brand-primary">
      编辑规则
    </button>
  </div>
  
  <div className="space-y-3">
    {rules.map(rule => (
      <div key={rule.id} className="flex items-start gap-3 p-3 bg-bg-tertiary rounded-md">
        <div className="mt-0.5">
          {rule.enabled ? (
            <CheckCircle className="w-5 h-5 text-success" />
          ) : (
            <AlertTriangle className="w-5 h-5 text-warning" />
          )}
        </div>
        <div className="flex-1">
          <div className="text-sm">{rule.description}</div>
          {rule.triggered > 0 && (
            <div className="text-xs text-text-tertiary mt-1">
              已触发 {rule.triggered} 次
            </div>
          )}
        </div>
      </div>
    ))}
  </div>
</div>
```

---

### 7. Settings (设置)

#### 组件实现

**7.1 左侧Tab导航**
```tsx
<div className="flex">
  {/* 侧边栏 */}
  <div className="w-48 border-r border-border-primary">
    {['个人资料', 'API密钥', '通知设置', '系统配置'].map((tab, index) => (
      <button
        key={tab}
        className={`w-full px-4 py-3 text-left hover:bg-bg-tertiary transition-colors ${
          index === 1 ? 'bg-bg-tertiary border-l-2 border-brand-primary' : ''
        }`}
      >
        {tab}
      </button>
    ))}
  </div>
  
  {/* 内容区 */}
  <div className="flex-1 p-8">
    {/* API密钥管理内容 */}
  </div>
</div>
```

**7.2 API密钥卡片**
```tsx
<div>
  <h2 className="text-2xl font-bold mb-6">API密钥管理</h2>
  
  <div className="space-y-4 mb-6">
    {apiKeys.map(key => (
      <div 
        key={key.id}
        className="bg-bg-secondary border border-border-primary rounded-lg p-6 flex items-center gap-4"
      >
        {/* 交易所Logo */}
        <div className="w-12 h-12 bg-brand-primary/10 rounded-md flex items-center justify-center">
          <img src={key.logo} alt={key.exchange} className="w-8 h-8" />
        </div>
        
        {/* 信息 */}
        <div className="flex-1">
          <div className="font-semibold text-lg mb-1">{key.exchange} API</div>
          <div className="text-sm font-mono text-text-secondary">
            API Key: {key.keyMasked}
          </div>
          <div className="flex items-center gap-2 mt-2">
            <div className={`w-2 h-2 rounded-full ${key.connected ? 'bg-success' : 'bg-danger'}`}></div>
            <span className={`text-xs ${key.connected ? 'text-success' : 'text-danger'}`}>
              {key.connected ? '已连接' : '断开连接'}
            </span>
          </div>
        </div>
        
        {/* 操作 */}
        <div className="flex gap-2">
          <button className="px-3 py-2 bg-bg-elevated border border-border-primary rounded-md hover:border-brand-primary transition-colors">
            <Edit2 className="w-4 h-4" />
          </button>
          <button className="px-3 py-2 bg-danger/10 border border-danger rounded-md hover:bg-danger/20 transition-colors">
            <Trash2 className="w-4 h-4 text-danger" />
          </button>
        </div>
      </div>
    ))}
  </div>
  
  <button className="px-6 py-3 bg-brand-primary text-bg-primary rounded-md font-semibold hover:bg-brand-secondary transition-colors flex items-center gap-2">
    <Plus className="w-5 h-5" />
    添加新API密钥
  </button>
</div>
```

---

## 交互要求

### Hover效果
- **按钮**: 背景色变化 + 轻微上移（`-translate-y-0.5`）
- **卡片**: 边框高亮 + 阴影增强 + 上移（`-translate-y-1`）
- **表格行**: 背景色变为`--bg-tertiary`

### 点击反馈
- **按钮**: 轻微下压效果（`translate-y-0`）
- **卡片**: 点击波纹效果（可选）

### 数据更新动画
- **新订单**: `animate-pulse`闪烁3次
- **价格变化**: 文字颜色变化 + 短暂高亮

### 页面切换
- **路由切换**: `fade-in`淡入动画（200ms）

### Loading状态
- **Skeleton屏幕**: 渐变加载动画
- **加载指示器**: 使用`RefreshCw`图标旋转动画

---

## 响应式要求

### 移动端（< 640px）
- 导航栏折叠为汉堡菜单
- 统计卡片单列布局
- 表格横向滚动
- 隐藏次要信息

### 平板（640px - 1024px）
- 统计卡片2列布局
- 策略卡片单列
- 保留主要功能

### 桌面（>= 1024px）
- 统计卡片2x2网格
- 策略卡片列表
- 显示完整信息
- 支持悬停交互

---

## 数据Mock示例

### 完整Mock数据

```typescript
// Dashboard数据
export const dashboardMockData = {
  accountValue: 125430.50,
  dailyPnL: 2340.80,
  dailyPnLPct: 1.9,
  activeStrategies: 3,
  totalTrades: 156,
  
  equityCurve: [
    { date: 'Mon', value: 110000 },
    { date: 'Tue', value: 115000 },
    { date: 'Wed', value: 118000 },
    { date: 'Thu', value: 122000 },
    { date: 'Fri', value: 123000 },
    { date: 'Sat', value: 124500 },
    { date: 'Sun', value: 125430 },
  ],
  
  strategies: [
    { id: 1, name: 'MA交叉策略', status: 'running', pnl: '+15.3%' },
    { id: 2, name: 'RSI突破策略', status: 'running', pnl: '+8.1%' },
    { id: 3, name: '趋势跟踪', status: 'running', pnl: '+12.7%' },
  ],
  
  quotes: [
    { symbol: 'BTC', price: 43250.50, change24h: 2.5, sparkline: [42500, 42800, 43000, 42700, 43100, 43250] },
    { symbol: 'ETH', price: 2280.30, change24h: 3.2, sparkline: [2200, 2220, 2250, 2240, 2270, 2280] },
  ],
};

// 策略列表数据
export const strategiesMockData = [
  {
    id: 1,
    name: 'MA交叉策略',
    type: '趋势跟踪',
    asset: '加密货币',
    status: 'running',
    returns: 15.3,
    sharpe: 1.8,
    drawdown: -8.2,
    equityCurve: [100, 102, 105, 108, 112, 115],
  },
  {
    id: 2,
    name: 'RSI突破策略',
    type: '动量策略',
    asset: '加密货币',
    status: 'stopped',
    returns: 8.1,
    sharpe: 1.2,
    drawdown: -12.5,
    equityCurve: [100, 98, 102, 105, 107, 108],
  },
  // 更多策略...
];

// 回测报告数据
export const backtestMockData = {
  metrics: {
    totalReturn: 45.3,
    annualReturn: 38.2,
    sharpeRatio: 1.85,
    maxDrawdown: -12.4,
    winRate: 62.5,
    profitFactor: 2.1,
  },
  
  equityCurve: Array.from({ length: 365 }, (_, i) => ({
    date: new Date(2023, 0, i + 1).toLocaleDateString(),
    strategy: 10000 + Math.random() * 5000,
    benchmark: 10000 + Math.random() * 3000,
  })),
  
  trades: Array.from({ length: 50 }, (_, i) => ({
    id: i + 1,
    time: new Date(2023, Math.floor(i / 4), (i % 4) * 7 + 1).toLocaleDateString(),
    side: i % 2 === 0 ? 'buy' : 'sell',
    price: 42000 + Math.random() * 2000,
    quantity: 0.1 + Math.random() * 0.5,
    commission: 8 + Math.random() * 5,
    pnl: (Math.random() - 0.3) * 500,
  })),
};

// 交易监控数据
export const tradingMockData = {
  orders: [
    { id: 1, symbol: 'BTCUSDT', side: 'buy', type: '限价', price: 43200, quantity: 0.5, status: 'pending' },
    { id: 2, symbol: 'ETHUSDT', side: 'sell', type: '市价', price: null, quantity: 2.0, status: 'pending' },
  ],
  
  positions: [
    { symbol: 'BTCUSDT', quantity: 0.5, avgPrice: 43000, currentPrice: 43250, pnl: 125, pnlPct: 5.8 },
    { symbol: 'ETHUSDT', quantity: 2.0, avgPrice: 2200, currentPrice: 2280, pnl: 160, pnlPct: 7.3 },
  ],
};
```

---

## 开发建议

### 项目结构

```
src/
├── components/
│   ├── common/         # 通用组件（Button、Card、Badge等）
│   ├── charts/         # 图表组件
│   ├── dashboard/      # Dashboard页面组件
│   ├── strategies/     # 策略相关组件
│   └── layout/         # 布局组件（Header、Sidebar）
├── pages/
│   ├── Dashboard.tsx
│   ├── Strategies.tsx
│   ├── StrategyEditor.tsx
│   ├── BacktestReport.tsx
│   ├── Trading.tsx
│   ├── RiskManagement.tsx
│   └── Settings.tsx
├── hooks/              # 自定义Hooks
├── utils/              # 工具函数
├── types/              # TypeScript类型定义
└── App.tsx
```

### 开发顺序

1. ✅ **第一周**: 设置TailwindCSS、创建通用组件库
2. ✅ **第二周**: Dashboard页面（核心展示）
3. ✅ **第三周**: 策略列表 + 策略编辑器
4. ✅ **第四周**: 回测报告 + 交易监控
5. ✅ **第五周**: 风控监控 + 设置页面
6. ✅ **第六周**: 完善交互、优化性能、测试

### 性能优化

- **代码分割**: 使用`React.lazy()`懒加载页面组件
- **虚拟列表**: 长列表使用`react-window`或`react-virtualized`
- **图表优化**: 限制数据点数量，使用防抖/节流
- **Memo化**: 对复杂组件使用`React.memo()`

---

## 附录

### 关键库版本

```json
{
  "react": "^18.2.0",
  "typescript": "^5.0.0",
  "tailwindcss": "^3.3.0",
  "recharts": "^2.10.0",
  "@monaco-editor/react": "^4.6.0",
  "lucide-react": "^0.300.0",
  "framer-motion": "^10.16.0"
}
```

### 相关文档

- [TailwindCSS文档](https://tailwindcss.com/docs)
- [Recharts文档](https://recharts.org/)
- [Lucide Icons](https://lucide.dev/)
- [Monaco Editor文档](https://microsoft.github.io/monaco-editor/)

---

**文档维护**: HermesFlow Design Team  
**反馈联系**: design@hermesflow.com  
**最后更新**: 2024-12-20

