# HermesFlow 设计系统文档

**版本**: v1.0.0  
**最后更新**: 2024-12-20  
**作者**: HermesFlow Design Team

---

## 目录

1. [设计系统概述](#设计系统概述)
2. [颜色系统](#颜色系统)
3. [字体系统](#字体系统)
4. [间距系统](#间距系统)
5. [圆角系统](#圆角系统)
6. [阴影系统](#阴影系统)
7. [组件规范](#组件规范)
8. [图标系统](#图标系统)
9. [动效系统](#动效系统)
10. [响应式断点](#响应式断点)

---

## 设计系统概述

### 设计理念

HermesFlow是一个专业的量化交易平台，设计系统遵循以下核心理念：

- **深色主题**：适合长时间盯盘，减少眼睛疲劳
- **专业感**：金融级数据密度，类似TradingView/Binance的专业工具感
- **高性能**：实时数据更新，流畅动画，无延迟感
- **信息优先**：数据可视化清晰，层次分明，减少装饰元素

### 设计原则

1. **Data-First（数据优先）**
   - 数据为核心，减少装饰性元素
   - 信息密度适中，不过载
   - 关键数据突出显示

2. **即时反馈**
   - 所有操作立即响应
   - 加载状态清晰
   - 成功/失败反馈明确

3. **专业工具感**
   - 为交易者设计，不是普通消费者
   - 功能优先于美观
   - 快捷键和效率工具

4. **暗色护眼**
   - 深色背景为主
   - 长时间使用不疲劳
   - 高对比度文字清晰可读

---

## 颜色系统

### 主色调（深色主题）

#### 背景色

```css
/* 主背景 - 最深色，用于页面底色 */
--bg-primary: #0B0E11;

/* 卡片背景 - 次深色，用于卡片、面板 */
--bg-secondary: #161A1E;

/* 悬停背景 - 交互元素悬停状态 */
--bg-tertiary: #1E2329;

/* 弹出层背景 - 弹窗、下拉菜单 */
--bg-elevated: #2B3139;
```

**使用场景**：
- `--bg-primary`: 页面主背景、大面积区域
- `--bg-secondary`: 卡片、侧边栏、导航栏
- `--bg-tertiary`: 表格行悬停、按钮悬停、输入框
- `--bg-elevated`: Modal、Dropdown、Tooltip

#### 文字色

```css
/* 主文字 - 标题、重要内容 */
--text-primary: #EAECEF;

/* 次要文字 - 正文、描述 */
--text-secondary: #848E9C;

/* 三级文字 - 辅助信息、时间戳 */
--text-tertiary: #5E6673;

/* 禁用文字 - 不可用状态 */
--text-disabled: #474D57;
```

**使用场景**：
- `--text-primary`: 页面标题、卡片标题、按钮文字
- `--text-secondary`: 正文内容、表格内容、表单标签
- `--text-tertiary`: 时间戳、辅助说明、placeholder
- `--text-disabled`: 禁用按钮、禁用表单

#### 边框色

```css
/* 主边框 - 卡片边框、分隔线 */
--border-primary: #2B3139;

/* 次要边框 - 细分隔线 */
--border-secondary: #1E2329;
```

#### 品牌色

```css
/* 主品牌色 - 金色，象征财富和专业 */
--brand-primary: #FCD535;

/* 次要品牌色 - 深金色，悬停状态 */
--brand-secondary: #F0B90B;
```

**使用场景**：
- 主按钮背景
- 链接颜色
- Logo配色
- 重要操作提示

#### 功能色

```css
/* 成功/上涨 - 绿色 */
--success: #0ECB81;

/* 危险/下跌 - 红色 */
--danger: #F6465D;

/* 警告 - 黄色 */
--warning: #F0B90B;

/* 信息 - 蓝色 */
--info: #3DCFFF;
```

**使用场景**：
- `--success`: 上涨价格、买入按钮、成功通知
- `--danger`: 下跌价格、卖出按钮、错误通知
- `--warning`: 警告通知、风险提示
- `--info`: 信息提示、帮助文档链接

#### 图表色

```css
/* 上涨蜡烛/线条 */
--chart-up: #0ECB81;

/* 下跌蜡烛/线条 */
--chart-down: #F6465D;

/* 网格线 */
--chart-grid: #2B3139;

/* 坐标轴 */
--chart-axis: #848E9C;
```

### 渐变色

#### 品牌渐变

```css
/* 金色渐变 - 用于强调元素、高级功能标识 */
--gradient-brand: linear-gradient(135deg, #FCD535 0%, #F0B90B 100%);
```

**使用场景**：
- 账户总值卡片背景
- VIP功能标识
- 重要数据指标背景

#### 数据可视化渐变

```css
/* 成功渐变 - 盈利图表填充 */
--gradient-success: linear-gradient(135deg, #0ECB81 0%, #05A660 100%);

/* 危险渐变 - 亏损图表填充 */
--gradient-danger: linear-gradient(135deg, #F6465D 0%, #D9304E 100%);
```

**使用场景**：
- 资金曲线图填充
- 收益柱状图
- 风险区域标识

### 颜色使用指南

**对比度要求**：
- 正文文字对比度: ≥ 4.5:1
- 大号文字对比度: ≥ 3:1
- 交互元素对比度: ≥ 3:1

**可访问性**：
- 不仅使用颜色区分信息（配合图标、文字）
- 色盲友好设计（红绿色盲考虑）
- 提供高对比度模式选项

---

## 字体系统

### 字体家族

#### 主字体

```css
/* 系统字体栈 - 用于UI文本 */
--font-primary: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
```

**特点**：
- 清晰易读
- 中性专业
- 多语言支持
- 数字区分度高

#### 等宽字体

```css
/* 等宽字体栈 - 用于数字、代码 */
--font-mono: 'JetBrains Mono', 'Fira Code', 'Roboto Mono', 'Courier New', monospace;
```

**使用场景**：
- 价格显示
- 数量显示
- 订单ID
- 策略代码编辑器
- 日志输出

### 字体大小

```css
/* 辅助信息 - 12px */
--text-xs: 0.75rem;

/* 次要文字 - 14px */
--text-sm: 0.875rem;

/* 正文 - 16px（基准） */
--text-base: 1rem;

/* 小标题 - 18px */
--text-lg: 1.125rem;

/* 标题 - 20px */
--text-xl: 1.25rem;

/* 大标题 - 24px */
--text-2xl: 1.5rem;

/* 页面标题 - 30px */
--text-3xl: 1.875rem;

/* 数据展示 - 36px */
--text-4xl: 2.25rem;
```

**使用场景**：
- `--text-xs`: 时间戳、辅助信息、徽章
- `--text-sm`: 表格内容、表单标签、按钮文字
- `--text-base`: 正文段落、描述文字
- `--text-lg`: 卡片标题、Tab标题
- `--text-xl`: 页面子标题、模块标题
- `--text-2xl`: 页面主标题
- `--text-3xl`: Landing页标题
- `--text-4xl`: 账户总值、重要数据指标

### 字重

```css
--font-normal: 400;   /* 正常 - 正文 */
--font-medium: 500;   /* 中等 - 强调 */
--font-semibold: 600; /* 半粗 - 标题 */
--font-bold: 700;     /* 粗体 - 重要标题 */
```

**使用指南**：
- 正文使用`400`
- 需要强调的关键词使用`500`
- 标题使用`600`
- 特别重要的标题使用`700`（慎用）

### 行高

```css
--leading-tight: 1.25;    /* 紧凑 - 数据展示 */
--leading-normal: 1.5;    /* 正常 - 正文 */
--leading-relaxed: 1.75;  /* 宽松 - 长文本 */
```

**使用场景**：
- `--leading-tight`: 大号数字、价格、统计数据
- `--leading-normal`: 正文段落、表格内容
- `--leading-relaxed`: 长篇文档、帮助文本

### 字母间距

```css
--tracking-tight: -0.05em;  /* 紧凑 - 大号标题 */
--tracking-normal: 0;       /* 正常 - 正文 */
--tracking-wide: 0.05em;    /* 宽松 - 按钮文字 */
```

---

## 间距系统

### 间距单位

采用8px基准网格系统（8pt Grid System）：

```css
--space-1: 0.25rem;   /* 4px  - 最小间距 */
--space-2: 0.5rem;    /* 8px  - 小间距 */
--space-3: 0.75rem;   /* 12px - 紧凑间距 */
--space-4: 1rem;      /* 16px - 标准间距 */
--space-5: 1.25rem;   /* 20px - 中等间距 */
--space-6: 1.5rem;    /* 24px - 宽松间距 */
--space-8: 2rem;      /* 32px - 大间距 */
--space-10: 2.5rem;   /* 40px - 超大间距 */
--space-12: 3rem;     /* 48px - 模块间距 */
--space-16: 4rem;     /* 64px - 页面间距 */
```

### 使用指南

**内边距（Padding）**：
- 按钮: `--space-3`（垂直）, `--space-6`（水平）
- 卡片: `--space-6`
- Modal: `--space-8`
- 输入框: `--space-3`（垂直）, `--space-4`（水平）

**外边距（Margin）**：
- 段落间距: `--space-4`
- 卡片间距: `--space-6`
- 模块间距: `--space-12`
- 页面边距: `--space-8` ~ `--space-16`

**组件间距**：
- 同组元素: `--space-2` ~ `--space-3`
- 不同组元素: `--space-4` ~ `--space-6`
- 模块分隔: `--space-8` ~ `--space-12`

---

## 圆角系统

```css
/* 小圆角 - 按钮、输入框 */
--radius-sm: 4px;

/* 中圆角 - 卡片、面板 */
--radius-md: 6px;

/* 大圆角 - Modal、Drawer */
--radius-lg: 8px;

/* 超大圆角 - 特殊卡片 */
--radius-xl: 12px;

/* 圆形 - 头像、徽章 */
--radius-full: 9999px;
```

**使用场景**：
- `--radius-sm`: 按钮、输入框、Tag、Badge
- `--radius-md`: 卡片、表格、面板
- `--radius-lg`: Modal、Drawer、大型卡片
- `--radius-xl`: 特殊强调卡片
- `--radius-full`: 头像、在线状态点、圆形按钮

---

## 阴影系统

```css
/* 小阴影 - 卡片悬停 */
--shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.3);

/* 中阴影 - 卡片 */
--shadow-md: 0 4px 6px rgba(0, 0, 0, 0.4);

/* 大阴影 - Dropdown、Popover */
--shadow-lg: 0 10px 15px rgba(0, 0, 0, 0.5);

/* 超大阴影 - Modal */
--shadow-xl: 0 20px 25px rgba(0, 0, 0, 0.6);
```

**使用场景**：
- `--shadow-sm`: 卡片hover状态
- `--shadow-md`: 静态卡片
- `--shadow-lg`: Dropdown、Tooltip、Popover
- `--shadow-xl`: Modal、大型弹窗

---

## 组件规范

### 按钮组件

#### 主按钮（Primary Button）

```css
.btn-primary {
  background: var(--brand-primary);
  color: var(--bg-primary);
  padding: var(--space-3) var(--space-6);
  border-radius: var(--radius-md);
  font-weight: var(--font-semibold);
  font-size: var(--text-sm);
  transition: all 0.2s;
  border: none;
  cursor: pointer;
}

.btn-primary:hover {
  background: var(--brand-secondary);
  transform: translateY(-1px);
  box-shadow: var(--shadow-sm);
}

.btn-primary:active {
  transform: translateY(0);
}

.btn-primary:disabled {
  background: var(--bg-tertiary);
  color: var(--text-disabled);
  cursor: not-allowed;
}
```

**使用场景**：主要操作（提交、确认、创建）

#### 次要按钮（Secondary Button）

```css
.btn-secondary {
  background: var(--bg-tertiary);
  color: var(--text-primary);
  border: 1px solid var(--border-primary);
  padding: var(--space-3) var(--space-6);
  border-radius: var(--radius-md);
  font-weight: var(--font-semibold);
  font-size: var(--text-sm);
  transition: all 0.2s;
  cursor: pointer;
}

.btn-secondary:hover {
  background: var(--bg-elevated);
  border-color: var(--brand-primary);
}
```

**使用场景**：次要操作（取消、返回）

#### 危险按钮（Danger Button）

```css
.btn-danger {
  background: var(--danger);
  color: white;
  padding: var(--space-3) var(--space-6);
  border-radius: var(--radius-md);
  font-weight: var(--font-semibold);
  font-size: var(--text-sm);
  transition: all 0.2s;
  border: none;
  cursor: pointer;
}

.btn-danger:hover {
  background: #D9304E;
}
```

**使用场景**：删除、停止、清空等危险操作

#### 成功按钮（Success Button）

```css
.btn-success {
  background: var(--success);
  color: white;
  padding: var(--space-3) var(--space-6);
  border-radius: var(--radius-md);
  font-weight: var(--font-semibold);
  font-size: var(--text-sm);
  transition: all 0.2s;
  border: none;
  cursor: pointer;
}

.btn-success:hover {
  background: #05A660;
}
```

**使用场景**：买入、启动、启用等正向操作

### 卡片组件

```css
.card {
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-lg);
  padding: var(--space-6);
  box-shadow: var(--shadow-sm);
  transition: all 0.2s;
}

.card:hover {
  border-color: var(--border-primary);
  box-shadow: var(--shadow-md);
  transform: translateY(-2px);
}

.card-header {
  font-size: var(--text-lg);
  font-weight: var(--font-semibold);
  color: var(--text-primary);
  margin-bottom: var(--space-4);
}

.card-body {
  font-size: var(--text-sm);
  color: var(--text-secondary);
}
```

### 输入框组件

```css
.input {
  background: var(--bg-tertiary);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-md);
  padding: var(--space-3) var(--space-4);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  width: 100%;
  transition: all 0.2s;
}

.input:focus {
  border-color: var(--brand-primary);
  outline: none;
  box-shadow: 0 0 0 3px rgba(252, 213, 53, 0.1);
}

.input::placeholder {
  color: var(--text-tertiary);
}

.input:disabled {
  background: var(--bg-secondary);
  color: var(--text-disabled);
  cursor: not-allowed;
}
```

### 表格组件

```css
.table {
  width: 100%;
  background: var(--bg-secondary);
  border-radius: var(--radius-lg);
  overflow: hidden;
  border: 1px solid var(--border-primary);
}

.table-header {
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  font-size: var(--text-sm);
  font-weight: var(--font-semibold);
  padding: var(--space-3) var(--space-4);
  text-align: left;
}

.table-row {
  border-bottom: 1px solid var(--border-primary);
  padding: var(--space-4);
  transition: background 0.2s;
}

.table-row:last-child {
  border-bottom: none;
}

.table-row:hover {
  background: var(--bg-tertiary);
}

.table-cell {
  padding: var(--space-3) var(--space-4);
  font-size: var(--text-sm);
  color: var(--text-primary);
  font-family: var(--font-mono);
}
```

### 标签组件（Badge）

```css
.badge {
  padding: var(--space-1) var(--space-3);
  border-radius: var(--radius-full);
  font-size: var(--text-xs);
  font-weight: var(--font-semibold);
  display: inline-block;
}

.badge-success {
  background: rgba(14, 203, 129, 0.1);
  color: var(--success);
}

.badge-danger {
  background: rgba(246, 70, 93, 0.1);
  color: var(--danger);
}

.badge-warning {
  background: rgba(240, 185, 11, 0.1);
  color: var(--warning);
}

.badge-info {
  background: rgba(61, 207, 255, 0.1);
  color: var(--info);
}
```

---

## 图标系统

### 图标库

**选择**: [Lucide Icons](https://lucide.dev/) (React)

**理由**：
- 开源免费
- 现代简洁
- React组件化
- 一致的设计风格
- 持续更新

### 图标大小

```css
--icon-sm: 16px;   /* 小图标 - 按钮内、Badge旁 */
--icon-md: 20px;   /* 中图标 - 列表项、菜单项（默认） */
--icon-lg: 24px;   /* 大图标 - 页面标题、空状态 */
--icon-xl: 32px;   /* 超大图标 - 空状态、引导页 */
```

### 常用图标清单

| 功能 | 图标名称 | 使用场景 |
|------|---------|---------|
| 涨跌 | TrendingUp / TrendingDown | 价格涨跌标识 |
| 图表 | BarChart3 / LineChart | 图表展示 |
| 实时数据 | Activity | 实时行情、实时监控 |
| 账户 | Wallet | 账户管理、余额显示 |
| 风控 | ShieldCheck / ShieldAlert | 风险监控 |
| 策略执行 | Zap | 策略运行、快速操作 |
| 设置 | Settings | 设置页面、配置 |
| 通知 | Bell | 通知中心 |
| 警告 | AlertTriangle | 警告提示 |
| 成功 | CheckCircle | 成功提示 |
| 错误 | XCircle | 错误提示 |
| 信息 | Info | 信息提示 |
| 添加 | Plus | 创建、添加 |
| 删除 | Trash2 | 删除操作 |
| 编辑 | Edit2 | 编辑操作 |
| 搜索 | Search | 搜索框 |
| 筛选 | Filter | 筛选功能 |
| 排序 | ArrowUpDown | 排序功能 |
| 刷新 | RefreshCw | 刷新数据 |
| 下载 | Download | 导出、下载 |
| 上传 | Upload | 上传文件 |
| 更多 | MoreVertical | 更多操作菜单 |

### 图标使用规范

**颜色**：
- 默认使用 `currentColor` 继承文字颜色
- 特殊情况下使用功能色（如涨跌图标）

**尺寸**：
- 与文字对齐时使用相同行高
- 独立使用时居中对齐

**间距**：
- 图标与文字间距: `--space-2`
- 图标按钮内边距: `--space-2`

---

## 动效系统

### 过渡时间

```css
--transition-fast: 150ms;    /* 快速 - 按钮hover、颜色变化 */
--transition-base: 200ms;    /* 标准 - 一般过渡 */
--transition-slow: 300ms;    /* 缓慢 - 复杂动画、大元素 */
```

### 缓动函数

```css
--ease-in: cubic-bezier(0.4, 0, 1, 1);           /* 加速 */
--ease-out: cubic-bezier(0, 0, 0.2, 1);          /* 减速 */
--ease-in-out: cubic-bezier(0.4, 0, 0.2, 1);     /* 先加速后减速 */
```

**使用指南**：
- `ease-in`: 元素离开视口
- `ease-out`: 元素进入视口（推荐）
- `ease-in-out`: 元素在视口内移动

### 常用动画

#### 淡入淡出

```css
@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.fade-in {
  animation: fadeIn var(--transition-base) var(--ease-out);
}
```

#### 滑入

```css
@keyframes slideInRight {
  from {
    transform: translateX(20px);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}

.slide-in-right {
  animation: slideInRight var(--transition-base) var(--ease-out);
}
```

#### 数字跳动（用于实时数据更新）

```css
@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.7;
  }
}

.pulse {
  animation: pulse var(--transition-slow) var(--ease-in-out);
}
```

#### 旋转加载

```css
@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.spin {
  animation: spin 1s linear infinite;
}
```

### 动效使用指南

**何时使用动效**：
- 用户操作反馈（按钮点击、表单提交）
- 页面/模块切换
- 数据加载状态
- 重要信息提示
- 实时数据更新

**何时不使用动效**：
- 频繁更新的数据（过度动画造成干扰）
- 性能敏感场景
- 用户禁用动画（respect prefers-reduced-motion）

---

## 响应式断点

```css
/* 移动端 */
--breakpoint-sm: 640px;

/* 平板 */
--breakpoint-md: 768px;

/* 笔记本 */
--breakpoint-lg: 1024px;

/* 桌面 */
--breakpoint-xl: 1280px;

/* 大屏 */
--breakpoint-2xl: 1536px;
```

### 响应式策略

**移动端优先（Mobile First）**：
```css
/* 默认样式（移动端） */
.container {
  padding: var(--space-4);
}

/* 平板及以上 */
@media (min-width: 768px) {
  .container {
    padding: var(--space-8);
  }
}

/* 桌面及以上 */
@media (min-width: 1024px) {
  .container {
    padding: var(--space-12);
  }
}
```

### 布局调整

**移动端（< 640px）**：
- 单列布局
- 隐藏次要信息
- 简化导航
- 大按钮（易点击）

**平板（640px - 1024px）**：
- 2列网格
- 显示关键信息
- 折叠侧边栏

**桌面（>= 1024px）**：
- 3-4列网格
- 显示完整信息
- 固定侧边栏
- 悬停交互

---

## 使用示例

### TailwindCSS配置

```javascript
// tailwind.config.js
module.exports = {
  theme: {
    extend: {
      colors: {
        bg: {
          primary: '#0B0E11',
          secondary: '#161A1E',
          tertiary: '#1E2329',
          elevated: '#2B3139',
        },
        text: {
          primary: '#EAECEF',
          secondary: '#848E9C',
          tertiary: '#5E6673',
          disabled: '#474D57',
        },
        border: {
          primary: '#2B3139',
          secondary: '#1E2329',
        },
        brand: {
          primary: '#FCD535',
          secondary: '#F0B90B',
        },
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

### CSS变量导出

```css
/* design-tokens.css */
:root {
  /* 背景色 */
  --bg-primary: #0B0E11;
  --bg-secondary: #161A1E;
  --bg-tertiary: #1E2329;
  --bg-elevated: #2B3139;

  /* 文字色 */
  --text-primary: #EAECEF;
  --text-secondary: #848E9C;
  --text-tertiary: #5E6673;
  --text-disabled: #474D57;

  /* 边框色 */
  --border-primary: #2B3139;
  --border-secondary: #1E2329;

  /* 品牌色 */
  --brand-primary: #FCD535;
  --brand-secondary: #F0B90B;

  /* 功能色 */
  --success: #0ECB81;
  --danger: #F6465D;
  --warning: #F0B90B;
  --info: #3DCFFF;

  /* 字体 */
  --font-primary: 'Inter', sans-serif;
  --font-mono: 'JetBrains Mono', monospace;

  /* 间距 */
  --space-1: 0.25rem;
  --space-2: 0.5rem;
  --space-3: 0.75rem;
  --space-4: 1rem;
  --space-6: 1.5rem;
  --space-8: 2rem;
  --space-12: 3rem;

  /* 圆角 */
  --radius-sm: 4px;
  --radius-md: 6px;
  --radius-lg: 8px;
  --radius-xl: 12px;
  --radius-full: 9999px;

  /* 阴影 */
  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.3);
  --shadow-md: 0 4px 6px rgba(0, 0, 0, 0.4);
  --shadow-lg: 0 10px 15px rgba(0, 0, 0, 0.5);
  --shadow-xl: 0 20px 25px rgba(0, 0, 0, 0.6);

  /* 过渡 */
  --transition-fast: 150ms;
  --transition-base: 200ms;
  --transition-slow: 300ms;
}
```

---

## 附录

### 设计资源

- **Figma设计文件**: [待补充]
- **图标库**: https://lucide.dev/
- **字体下载**: 
  - Inter: https://rsms.me/inter/
  - JetBrains Mono: https://www.jetbrains.com/lp/mono/

### 变更日志

| 版本 | 日期 | 变更说明 |
|------|------|----------|
| v1.0.0 | 2024-12-20 | 初始版本，定义完整设计系统 |

---

**文档维护**: HermesFlow Design Team  
**反馈联系**: design@hermesflow.com
