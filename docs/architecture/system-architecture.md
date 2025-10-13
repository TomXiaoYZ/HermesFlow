# HermesFlow 系统架构文档

**版本**: v2.1.0  
**最后更新**: 2024-12-20  
**文档状态**: 详细设计  
**作者**: HermesFlow Architecture Team

---

## 目录

1. [架构概述](#1-架构概述)
2. [C4架构模型](#2-c4架构模型)
3. [前端架构设计](#3-前端架构设计)
4. [后端架构设计](#4-后端架构设计)
5. [数据架构设计](#5-数据架构设计)
6. [部署架构设计](#6-部署架构设计)
7. [安全架构设计](#7-安全架构设计)
8. [性能优化策略](#8-性能优化策略)
9. [监控与运维方案](#9-监控与运维方案)
10. [技术债务与改进计划](#10-技术债务与改进计划)

---

## 1. 架构概述

### 1.1 系统愿景与目标

HermesFlow是一个**高性能、低成本、易扩展的个人量化交易平台**，致力于为个人交易者提供专业级的量化交易能力。

**核心目标**：
- **性能至上**：μs级数据延迟，支持高频交易
- **成本优化**：开源免费，云原生部署降低运维成本
- **易于扩展**：模块化设计，支持多市场、多策略
- **安全可靠**：多租户隔离，金融级安全标准

**预期成果**（MVP阶段）：
- 年化收益率：15-30%（高频套利 + 趋势跟踪）
- 数据延迟：< 1ms（WebSocket实时数据）
- 系统可用性：99.9%
- 用户体验：专业交易工具级别

### 1.2 架构设计原则

#### 1.2.1 性能至上

**原则说明**：在架构设计的每一层都优先考虑性能。

**具体实践**：
- **数据层**：采用Rust零成本抽象，实现μs级延迟
- **存储层**：多级缓存（内存 → Redis → ClickHouse）
- **网络层**：WebSocket长连接 + gRPC二进制协议
- **计算层**：Numba JIT编译加速因子计算

#### 1.2.2 技术栈互补

**原则说明**：选择最适合的语言和框架，发挥各自优势。

**技术栈分工**：
- **Rust**（数据层）：高性能、低延迟、内存安全
- **Java**（业务层）：成熟生态、企业级、虚拟线程
- **Python**（策略层）：科学计算、快速原型、丰富的量化库

#### 1.2.3 成本优化

**原则说明**：在满足性能的前提下，优化成本。

**具体实践**：
- **资源利用**：容器化部署，按需扩缩容
- **存储优化**：ClickHouse列式压缩，降低存储成本
- **计算优化**：异步IO，减少线程/进程开销
- **云原生**：使用Azure托管服务，降低运维成本

#### 1.2.4 安全隔离

**原则说明**：多层次安全隔离，保护用户数据和交易安全。

**具体实践**：
- **数据库层**：PostgreSQL RLS行级安全
- **缓存层**：Redis Key前缀隔离
- **消息层**：Kafka Topic分区隔离
- **应用层**：Tenant Context注入

### 1.3 关键架构决策汇总

本系统采用了以下关键架构决策（详见ADR文档）：

| ADR | 决策内容 | 状态 | 影响 |
|-----|---------|------|------|
| [ADR-001](./decisions/ADR-001-hybrid-tech-stack.md) | 采用混合技术栈架构 | Accepted | 高性能但需多技能团队 |
| [ADR-002](./decisions/ADR-002-tokio-runtime.md) | 选择Tokio作为Rust异步运行时 | Accepted | 生态完善但有学习曲线 |
| [ADR-003](./decisions/ADR-003-postgresql-rls.md) | PostgreSQL RLS实现多租户 | Accepted | 数据库层隔离更安全 |
| [ADR-004](./decisions/ADR-004-clickhouse-analytics.md) | ClickHouse作为分析数据库 | Accepted | 高性能查询但需学习 |
| [ADR-005](./decisions/ADR-005-kafka-event-streaming.md) | Kafka作为事件流平台 | Accepted | 解耦服务但运维复杂 |
| [ADR-006](./decisions/ADR-006-react-frontend.md) | React + TypeScript前端 | Accepted | 生态成熟但需优化 |
| [ADR-007](./decisions/ADR-007-numba-acceleration.md) | Alpha因子库使用Numba加速 | Accepted | 性能提升但有限制 |
| [ADR-008](./decisions/ADR-008-paper-trading-api.md) | 模拟交易与实盘API兼容 | Accepted | 降低切换成本 |

### 1.4 架构演进路线图

```
┌─────────────────────────────────────────────────────────────┐
│                      架构演进路线图                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  MVP (3个月)         V2 (6个月)          V3 (12个月)       │
│  ↓                   ↓                    ↓                 │
│  可盈利基础平台  →   增强盈利能力    →   平台化与生态       │
│                                                             │
│  - Rust数据层        - ML集成            - 策略市场         │
│  - Alpha因子库       - 组合管理          - 移动APP          │
│  - 策略优化          - 因子扩展200+      - 高级可视化       │
│  - 模拟交易          - 实盘交易          - 智能推荐         │
│  - 基础前端          - 多数据源          - 社区生态         │
│                                                             │
│  年化收益: 15-30%    年化收益: 20-40%    用户: 1000+       │
│  工作量: 5.5人月     工作量: 10人月      工作量: 20人月    │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. C4架构模型

### 2.1 Context图（系统上下文）

```mermaid
C4Context
    title 系统上下文图 - HermesFlow量化交易平台

    Person(user, "量化交易者", "个人交易者、策略开发者")
    
    System(hermesflow, "HermesFlow Platform", "量化交易平台，提供数据采集、策略开发、回测、实盘交易功能")
    
    System_Ext(binance, "Binance", "加密货币交易所")
    System_Ext(okx, "OKX", "加密货币交易所")
    System_Ext(ibkr, "Interactive Brokers", "美股/期权经纪商")
    System_Ext(polygon, "Polygon.io", "市场数据API")
    System_Ext(fred, "FRED API", "宏观经济数据")
    System_Ext(newsapi, "NewsAPI", "舆情数据源")
    
    Rel(user, hermesflow, "使用", "HTTPS/WebSocket")
    Rel(hermesflow, binance, "获取数据、下单", "WebSocket/REST")
    Rel(hermesflow, okx, "获取数据、下单", "WebSocket/REST")
    Rel(hermesflow, ibkr, "获取数据、下单", "TWS API")
    Rel(hermesflow, polygon, "获取美股数据", "REST")
    Rel(hermesflow, fred, "获取宏观数据", "REST")
    Rel(hermesflow, newsapi, "获取新闻数据", "REST")
```

**外部系统说明**：

1. **交易所**：
   - Binance/OKX: 加密货币现货、合约交易
   - IBKR: 美股、期权交易

2. **数据源**：
   - Polygon.io: 美股实时/历史数据
   - FRED: 宏观经济指标（利率、CPI等）
   - NewsAPI: 新闻舆情数据

3. **用户**：
   - Web浏览器：主要交互界面
   - （未来）移动端APP：监控和简单操作

### 2.2 Container图（容器架构）

```mermaid
C4Container
    title 容器架构图 - HermesFlow平台

    Person(user, "交易者", "")

    Container_Boundary(hermesflow, "HermesFlow Platform") {
        Container(frontend, "前端应用", "React 18 + TypeScript", "用户交互界面，策略管理、回测、交易监控")
        Container(gateway, "API Gateway", "Spring Cloud Gateway", "统一入口，路由、认证、限流")
        Container(data_service, "数据服务", "Rust (Tokio + Actix-web)", "实时数据采集、标准化、存储")
        Container(strategy_engine, "策略引擎", "Python (FastAPI)", "策略开发、回测、因子库、优化器")
        Container(trading_service, "交易服务", "Java (Spring Boot)", "订单管理、执行、持仓管理")
        Container(user_service, "用户服务", "Java (Spring Boot)", "认证、授权、多租户管理")
        Container(risk_service, "风控服务", "Java (Spring Boot)", "风险监控、告警、规则引擎")
    }

    ContainerDb(postgres, "PostgreSQL", "关系型数据库", "用户、策略、订单、持仓")
    ContainerDb(clickhouse, "ClickHouse", "列式数据库", "历史市场数据、时序分析")
    ContainerDb(redis, "Redis", "内存数据库", "缓存、实时数据、消息队列")
    ContainerQueue(kafka, "Kafka", "消息队列", "事件流、异步解耦")

    Rel(user, frontend, "访问", "HTTPS")
    Rel(frontend, gateway, "API调用", "HTTPS/WebSocket")
    
    Rel(gateway, data_service, "数据查询", "gRPC/REST")
    Rel(gateway, strategy_engine, "策略操作", "REST")
    Rel(gateway, trading_service, "交易操作", "REST")
    Rel(gateway, user_service, "用户操作", "REST")
    Rel(gateway, risk_service, "风控查询", "REST")
    
    Rel(data_service, redis, "写入/读取", "Redis Protocol")
    Rel(data_service, clickhouse, "批量写入", "Native Protocol")
    Rel(data_service, kafka, "发布事件", "Kafka Protocol")
    
    Rel(strategy_engine, data_service, "数据订阅", "gRPC Stream")
    Rel(strategy_engine, redis, "因子缓存", "Redis Protocol")
    Rel(strategy_engine, kafka, "发布信号", "Kafka Protocol")
    
    Rel(trading_service, postgres, "读写", "JDBC")
    Rel(trading_service, kafka, "订阅/发布", "Kafka Protocol")
    
    Rel(user_service, postgres, "读写", "JDBC")
    Rel(risk_service, postgres, "读写", "JDBC")
    Rel(risk_service, kafka, "订阅事件", "Kafka Protocol")
```

**容器说明**：

1. **前端应用**（React）：
   - 7个核心页面（Dashboard/Strategies/Editor/Report/Trading/Risk/Settings）
   - 实时数据可视化
   - Monaco代码编辑器

2. **API Gateway**（Spring Cloud Gateway）：
   - 统一入口
   - JWT认证
   - Rate Limiting
   - 路由到后端服务

3. **数据服务**（Rust）：
   - WebSocket连接管理
   - 数据标准化
   - 多级存储

4. **策略引擎**（Python）：
   - Alpha因子库（100个因子）
   - 贝叶斯优化器
   - 回测引擎
   - 模拟交易

5. **交易服务**（Java）：
   - 订单管理
   - 智能路由
   - 持仓管理

6. **用户服务**（Java）：
   - JWT认证
   - 多租户管理
   - API密钥管理

7. **风控服务**（Java）：
   - 实时监控
   - 规则引擎
   - 告警管理

### 2.3 Component图（组件架构）

#### 2.3.1 数据服务组件图

```
┌────────────────────────────────────────────────────────┐
│            数据服务 (Rust)                              │
├────────────────────────────────────────────────────────┤
│                                                        │
│  ┌──────────────────────────────────────────────┐    │
│  │       API Layer (gRPC/REST)                  │    │
│  │  - MarketDataService                         │    │
│  │  - HistoricalDataService                     │    │
│  │  - DataStreamService                         │    │
│  └──────────────────────────────────────────────┘    │
│                       ↓                                │
│  ┌──────────────────────────────────────────────┐    │
│  │       Business Logic Layer                   │    │
│  │                                              │    │
│  │  ┌────────────────┐  ┌───────────────────┐  │    │
│  │  │ConnectorManager│  │DataProcessor      │  │    │
│  │  │- Binance       │  │- Normalizer       │  │    │
│  │  │- OKX           │  │- Validator        │  │    │
│  │  │- IBKR          │  │- Aggregator       │  │    │
│  │  └────────────────┘  └───────────────────┘  │    │
│  │                                              │    │
│  │  ┌────────────────┐  ┌───────────────────┐  │    │
│  │  │StorageManager  │  │StreamPublisher    │  │    │
│  │  │- Redis Writer  │  │- Kafka Producer   │  │    │
│  │  │- ClickHouse    │  │- Event Router     │  │    │
│  │  │  Writer        │  │                   │  │    │
│  │  └────────────────┘  └───────────────────┘  │    │
│  └──────────────────────────────────────────────┘    │
│                       ↓                                │
│  ┌──────────────────────────────────────────────┐    │
│  │       Data Access Layer                      │    │
│  │  - RedisClient (redis-rs)                    │    │
│  │  - ClickHouseClient (clickhouse-rs)          │    │
│  │  - KafkaProducer (rdkafka)                   │    │
│  └──────────────────────────────────────────────┘    │
└────────────────────────────────────────────────────────┘
```

**组件职责**：

- `ConnectorManager`: 管理多个交易所的WebSocket连接
- `DataProcessor`: 数据标准化、验证、聚合
- `StorageManager`: 多级存储写入协调
- `StreamPublisher`: Kafka事件发布

#### 2.3.2 策略引擎组件图

```
┌────────────────────────────────────────────────────────┐
│            策略引擎 (Python)                            │
├────────────────────────────────────────────────────────┤
│                                                        │
│  ┌──────────────────────────────────────────────┐    │
│  │       API Layer (FastAPI)                    │    │
│  │  - StrategyAPI                               │    │
│  │  - BacktestAPI                               │    │
│  │  - OptimizerAPI                              │    │
│  │  - PaperTradingAPI                           │    │
│  └──────────────────────────────────────────────┘    │
│                       ↓                                │
│  ┌──────────────────────────────────────────────┐    │
│  │       Strategy Management                    │    │
│  │  - StrategyLoader (插件化加载)               │    │
│  │  - StrategyExecutor (执行器)                 │    │
│  │  - StrategyRegistry (注册表)                 │    │
│  └──────────────────────────────────────────────┘    │
│                       ↓                                │
│  ┌──────────────────────────────────────────────┐    │
│  │       Alpha Factor Library                   │    │
│  │                                              │    │
│  │  ┌──────────────────────────────────────┐   │    │
│  │  │FactorLibrary (100个因子)             │   │    │
│  │  │- TechnicalFactors (20个)             │   │    │
│  │  │- VolumeFactors (15个)                │   │    │
│  │  │- FundamentalFactors (30个)           │   │    │
│  │  │- OtherFactors (35个)                 │   │    │
│  │  └──────────────────────────────────────┘   │    │
│  │                                              │    │
│  │  ┌──────────────────────────────────────┐   │    │
│  │  │FactorCalculator (Numba加速)          │   │    │
│  │  └──────────────────────────────────────┘   │    │
│  │                                              │    │
│  │  ┌──────────────────────────────────────┐   │    │
│  │  │FactorCache (Redis)                   │   │    │
│  │  └──────────────────────────────────────┘   │    │
│  └──────────────────────────────────────────────┘    │
│                       ↓                                │
│  ┌──────────────────────────────────────────────┐    │
│  │       Optimization Engine                    │    │
│  │  - BayesianOptimizer (scikit-optimize)       │    │
│  │  - WalkForwardAnalyzer                       │    │
│  │  - GeneticOptimizer                          │    │
│  └──────────────────────────────────────────────┘    │
│                       ↓                                │
│  ┌──────────────────────────────────────────────┐    │
│  │       Backtest Engine                        │    │
│  │  - BacktestEngine (事件驱动)                 │    │
│  │  - PerformanceAnalyzer                       │    │
│  │  - ReportGenerator                           │    │
│  └──────────────────────────────────────────────┘    │
│                       ↓                                │
│  ┌──────────────────────────────────────────────┐    │
│  │       Paper Trading System                   │    │
│  │  - PaperTradingBroker                        │    │
│  │  - OrderSimulator                            │    │
│  │  - MarketDataFeed                            │    │
│  └──────────────────────────────────────────────┘    │
└────────────────────────────────────────────────────────┘
```

**组件职责**：

- `FactorLibrary`: 提供100个预定义因子
- `BayesianOptimizer`: 贝叶斯优化（比网格搜索快5-10倍）
- `WalkForwardAnalyzer`: 参数稳定性验证（防过拟合）
- `PaperTradingBroker`: 虚拟交易环境（与实盘API兼容）

#### 2.3.3 交易服务组件图

```
┌────────────────────────────────────────────────────────┐
│            交易服务 (Java Spring Boot)                  │
├────────────────────────────────────────────────────────┤
│                                                        │
│  ┌──────────────────────────────────────────────┐    │
│  │       REST API Layer                         │    │
│  │  - OrderController                           │    │
│  │  - PositionController                        │    │
│  │  - AccountController                         │    │
│  └──────────────────────────────────────────────┘    │
│                       ↓                                │
│  ┌──────────────────────────────────────────────┐    │
│  │       Order Management                       │    │
│  │                                              │    │
│  │  ┌────────────────┐  ┌───────────────────┐  │    │
│  │  │OrderManager    │  │SmartRouter        │  │    │
│  │  │- Create        │  │- Route Selection  │  │    │
│  │  │- Update        │  │- Load Balancing   │  │    │
│  │  │- Cancel        │  │- Failover         │  │    │
│  │  │- Query         │  │                   │  │    │
│  │  └────────────────┘  └───────────────────┘  │    │
│  │                                              │    │
│  │  ┌────────────────┐  ┌───────────────────┐  │    │
│  │  │ExecutionEngine │  │PositionManager    │  │    │
│  │  │- Market Order  │  │- Position Tracking│  │    │
│  │  │- Limit Order   │  │- P&L Calculation  │  │    │
│  │  │- Stop Order    │  │- Reconciliation   │  │    │
│  │  └────────────────┘  └───────────────────┘  │    │
│  └──────────────────────────────────────────────┘    │
│                       ↓                                │
│  ┌──────────────────────────────────────────────┐    │
│  │       Exchange Integration                   │    │
│  │  - BinanceClient                             │    │
│  │  - OKXClient                                 │    │
│  │  - IBKRClient                                │    │
│  └──────────────────────────────────────────────┘    │
│                       ↓                                │
│  ┌──────────────────────────────────────────────┐    │
│  │       Data Access Layer                      │    │
│  │  - OrderRepository (JPA)                     │    │
│  │  - PositionRepository (JPA)                  │    │
│  │  - TradeRepository (JPA)                     │    │
│  └──────────────────────────────────────────────┘    │
└────────────────────────────────────────────────────────┘
```

---

## 3. 前端架构设计

### 3.1 技术栈

基于 `docs/design/design-system.md` 和 `docs/design/lovable-v0-prompt.md`：

**核心框架**：
- React 18（并发特性、Suspense）
- TypeScript 5.x（类型安全）
- Vite（快速构建）

**UI框架**：
- TailwindCSS（自定义设计系统）
- Recharts / Tremor（图表库）
- Lucide Icons（图标库）

**状态管理**：
- Zustand（轻量级全局状态）
- React Query（服务端状态）
- Jotai（原子化状态）

**代码编辑器**：
- Monaco Editor（VS Code编辑器）

**其他库**：
- react-window（虚拟列表）
- framer-motion（动画）
- react-sparklines（迷你图表）
- react-circular-progressbar（圆形进度条）

### 3.2 应用架构

```
frontend/
├── public/                    # 静态资源
├── src/
│   ├── components/            # 组件库
│   │   ├── common/           # 通用组件
│   │   │   ├── Button.tsx    # 按钮组件
│   │   │   ├── Card.tsx      # 卡片组件
│   │   │   ├── Badge.tsx     # 徽章组件
│   │   │   ├── Input.tsx     # 输入框组件
│   │   │   ├── Table.tsx     # 表格组件
│   │   │   └── Modal.tsx     # 弹窗组件
│   │   ├── charts/           # 图表组件
│   │   │   ├── EquityCurveChart.tsx   # 资金曲线图
│   │   │   ├── LineChart.tsx          # 折线图
│   │   │   ├── PieChart.tsx           # 饼图
│   │   │   └── MiniSparkline.tsx      # 迷你图
│   │   ├── layout/           # 布局组件
│   │   │   ├── Header.tsx    # 顶部导航
│   │   │   ├── Sidebar.tsx   # 侧边栏
│   │   │   └── Layout.tsx    # 整体布局
│   │   └── domain/           # 业务组件
│   │       ├── StrategyCard.tsx       # 策略卡片
│   │       ├── OrderTable.tsx         # 订单表格
│   │       ├── PositionCard.tsx       # 持仓卡片
│   │       └── RiskIndicator.tsx      # 风险指标
│   │
│   ├── pages/                # 页面组件
│   │   ├── Dashboard/        # 总览仪表盘
│   │   │   ├── index.tsx
│   │   │   ├── AccountValueCard.tsx
│   │   │   ├── DailyPnLCard.tsx
│   │   │   └── EquityCurveSection.tsx
│   │   ├── Strategies/       # 策略列表
│   │   │   ├── index.tsx
│   │   │   ├── StrategyList.tsx
│   │   │   └── FilterBar.tsx
│   │   ├── StrategyEditor/   # 策略编辑器
│   │   │   ├── index.tsx
│   │   │   ├── CodeEditor.tsx
│   │   │   └── ParameterPanel.tsx
│   │   ├── BacktestReport/   # 回测报告
│   │   │   ├── index.tsx
│   │   │   ├── MetricsGrid.tsx
│   │   │   └── TradesTable.tsx
│   │   ├── Trading/          # 交易监控
│   │   │   ├── index.tsx
│   │   │   ├── OrdersTab.tsx
│   │   │   └── PositionsTab.tsx
│   │   ├── RiskManagement/   # 风控监控
│   │   │   ├── index.tsx
│   │   │   ├── RiskIndicators.tsx
│   │   │   └── AlertHistory.tsx
│   │   └── Settings/         # 设置
│   │       ├── index.tsx
│   │       ├── ApiKeysTab.tsx
│   │       └── NotificationTab.tsx
│   │
│   ├── hooks/                # 自定义Hooks
│   │   ├── useAuth.ts        # 认证Hook
│   │   ├── useWebSocket.ts   # WebSocket Hook
│   │   ├── useStrategies.ts  # 策略数据Hook
│   │   └── useRealTimeData.ts # 实时数据Hook
│   │
│   ├── services/             # API服务层
│   │   ├── api.ts            # Axios配置
│   │   ├── authService.ts    # 认证服务
│   │   ├── strategyService.ts # 策略服务
│   │   ├── tradingService.ts  # 交易服务
│   │   └── wsService.ts      # WebSocket服务
│   │
│   ├── stores/               # 状态管理
│   │   ├── authStore.ts      # 认证状态
│   │   ├── strategyStore.ts  # 策略状态
│   │   ├── tradingStore.ts   # 交易状态
│   │   └── uiStore.ts        # UI状态
│   │
│   ├── utils/                # 工具函数
│   │   ├── formatters.ts     # 格式化工具
│   │   ├── validators.ts     # 验证工具
│   │   └── helpers.ts        # 辅助函数
│   │
│   ├── types/                # TypeScript类型
│   │   ├── strategy.ts       # 策略类型
│   │   ├── order.ts          # 订单类型
│   │   ├── position.ts       # 持仓类型
│   │   └── api.ts            # API类型
│   │
│   ├── styles/               # 样式文件
│   │   ├── globals.css       # 全局样式
│   │   └── design-tokens.css # 设计令牌
│   │
│   ├── App.tsx               # 根组件
│   ├── main.tsx              # 入口文件
│   └── routes.tsx            # 路由配置
│
├── tailwind.config.js        # Tailwind配置
├── tsconfig.json             # TypeScript配置
├── vite.config.ts            # Vite配置
└── package.json              # 依赖配置
```

### 3.3 状态管理策略

#### 3.3.1 全局状态（Zustand）

```typescript
// stores/authStore.ts
import create from 'zustand';

interface AuthState {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  login: (email: string, password: string) => Promise<void>;
  logout: () => void;
}

export const useAuthStore = create<AuthState>((set) => ({
  user: null,
  token: null,
  isAuthenticated: false,
  login: async (email, password) => {
    const { user, token } = await authService.login(email, password);
    set({ user, token, isAuthenticated: true });
  },
  logout: () => {
    set({ user: null, token: null, isAuthenticated: false });
  },
}));
```

#### 3.3.2 服务端状态（React Query）

```typescript
// hooks/useStrategies.ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';

export function useStrategies() {
  return useQuery({
    queryKey: ['strategies'],
    queryFn: () => strategyService.getAll(),
    staleTime: 5 * 60 * 1000, // 5分钟
  });
}

export function useCreateStrategy() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (strategy: CreateStrategyDto) =>
      strategyService.create(strategy),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['strategies'] });
    },
  });
}
```

#### 3.3.3 WebSocket状态

```typescript
// hooks/useWebSocket.ts
import { useEffect, useState } from 'react';

export function useRealTimePrice(symbol: string) {
  const [price, setPrice] = useState<number | null>(null);
  
  useEffect(() => {
    const ws = new WebSocket(`ws://api.hermesflow.com/ws/market/${symbol}`);
    
    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      setPrice(data.price);
    };
    
    return () => ws.close();
  }, [symbol]);
  
  return price;
}
```

### 3.4 性能优化

#### 3.4.1 代码分割

```typescript
// routes.tsx
import { lazy, Suspense } from 'react';

const Dashboard = lazy(() => import('./pages/Dashboard'));
const Strategies = lazy(() => import('./pages/Strategies'));
const StrategyEditor = lazy(() => import('./pages/StrategyEditor'));

export const routes = [
  {
    path: '/',
    element: (
      <Suspense fallback={<Loading />}>
        <Dashboard />
      </Suspense>
    ),
  },
  // ... more routes
];
```

#### 3.4.2 虚拟列表

```typescript
// components/domain/StrategyList.tsx
import { FixedSizeList } from 'react-window';

export function StrategyList({ strategies }: Props) {
  return (
    <FixedSizeList
      height={600}
      itemCount={strategies.length}
      itemSize={120}
      width="100%"
    >
      {({ index, style }) => (
        <div style={style}>
          <StrategyCard strategy={strategies[index]} />
        </div>
      )}
    </FixedSizeList>
  );
}
```

#### 3.4.3 图表优化

```typescript
// components/charts/EquityCurveChart.tsx
import { useMemo } from 'react';
import { LineChart } from 'recharts';

export function EquityCurveChart({ data }: Props) {
  // 数据采样：当数据点超过1000时，采样到500个点
  const sampledData = useMemo(() => {
    if (data.length <= 1000) return data;
    
    const step = Math.ceil(data.length / 500);
    return data.filter((_, index) => index % step === 0);
  }, [data]);
  
  return <LineChart data={sampledData} ... />;
}
```

#### 3.4.4 Memo化

```typescript
// components/domain/StrategyCard.tsx
import { memo } from 'react';

export const StrategyCard = memo(function StrategyCard({ strategy }: Props) {
  return (
    <Card>
      <h3>{strategy.name}</h3>
      <div>收益: {strategy.returns}%</div>
    </Card>
  );
}, (prevProps, nextProps) => {
  // 只有strategy变化时才重新渲染
  return prevProps.strategy.id === nextProps.strategy.id &&
         prevProps.strategy.returns === nextProps.strategy.returns;
});
```

---

## 4. 后端架构设计

### 4.1 数据服务（Rust）

#### 4.1.1 项目结构

```
data-engine/
├── Cargo.toml
├── src/
│   ├── main.rs                # 入口文件
│   ├── config.rs              # 配置管理
│   ├── api/                   # API层
│   │   ├── mod.rs
│   │   ├── grpc/              # gRPC服务
│   │   │   ├── market_data.rs
│   │   │   └── historical_data.rs
│   │   └── rest/              # REST API
│   │       ├── health.rs
│   │       └── query.rs
│   ├── connectors/            # 交易所连接器
│   │   ├── mod.rs
│   │   ├── base.rs            # 基础Trait
│   │   ├── binance.rs
│   │   ├── okx.rs
│   │   └── ibkr.rs
│   ├── processor/             # 数据处理
│   │   ├── mod.rs
│   │   ├── normalizer.rs      # 标准化
│   │   └── validator.rs       # 验证
│   ├── storage/               # 存储层
│   │   ├── mod.rs
│   │   ├── redis_client.rs
│   │   ├── clickhouse_client.rs
│   │   └── kafka_producer.rs
│   ├── models/                # 数据模型
│   │   ├── mod.rs
│   │   ├── market_data.rs
│   │   └── errors.rs
│   └── utils/                 # 工具函数
│       ├── mod.rs
│       └── telemetry.rs       # 日志追踪
└── proto/                     # Protobuf定义
    └── market_data.proto
```

#### 4.1.2 核心组件实现

**ConnectorManager**：

```rust
// src/connectors/mod.rs
use async_trait::async_trait;
use tokio::sync::mpsc;

#[async_trait]
pub trait Connector: Send + Sync {
    async fn connect(&mut self) -> Result<()>;
    async fn subscribe(&mut self, symbols: Vec<String>) -> Result<()>;
    async fn disconnect(&mut self) -> Result<()>;
}

pub struct ConnectorManager {
    connectors: HashMap<String, Box<dyn Connector>>,
    tx: mpsc::Sender<MarketData>,
}

impl ConnectorManager {
    pub fn new(tx: mpsc::Sender<MarketData>) -> Self {
        Self {
            connectors: HashMap::new(),
            tx,
        }
    }
    
    pub async fn add_connector(&mut self, name: String, connector: Box<dyn Connector>) {
        self.connectors.insert(name, connector);
    }
    
    pub async fn start_all(&mut self) -> Result<()> {
        for (name, connector) in &mut self.connectors {
            tracing::info!("Starting connector: {}", name);
            connector.connect().await?;
        }
        Ok(())
    }
}
```

**性能指标**：

| 指标 | 目标 | 测量方法 |
|------|------|---------|
| WebSocket延迟 | < 1ms | 从接收到写入Redis的时间 |
| 数据处理吞吐 | > 100K msg/s | Prometheus `data_processed_total` |
| ClickHouse写入 | > 1M rows/s | ClickHouse `system.query_log` |
| 内存使用 | < 512MB | Prometheus `process_resident_memory_bytes` |

### 4.2 策略引擎（Python）

#### 4.2.1 项目结构

```
strategy-engine/
├── pyproject.toml             # Poetry配置
├── src/
│   ├── main.py                # FastAPI入口
│   ├── config/                # 配置
│   │   └── settings.py
│   ├── api/                   # API路由
│   │   ├── __init__.py
│   │   ├── strategies.py
│   │   ├── backtest.py
│   │   ├── optimizer.py
│   │   └── paper_trading.py
│   ├── core/                  # 核心业务逻辑
│   │   ├── strategy/          # 策略管理
│   │   │   ├── base.py        # Strategy基类
│   │   │   ├── loader.py      # 策略加载器
│   │   │   └── executor.py    # 策略执行器
│   │   ├── factors/           # Alpha因子库
│   │   │   ├── library.py     # 因子库
│   │   │   ├── technical.py   # 技术指标因子（20个）
│   │   │   ├── volume.py      # 量价因子（15个）
│   │   │   ├── fundamental.py # 基本面因子（30个）
│   │   │   ├── other.py       # 其他因子（35个）
│   │   │   ├── calculator.py  # 计算引擎（Numba）
│   │   │   └── cache.py       # Redis缓存
│   │   ├── optimizer/         # 优化器
│   │   │   ├── bayesian.py    # 贝叶斯优化
│   │   │   ├── walk_forward.py # Walk-Forward分析
│   │   │   └── genetic.py     # 遗传算法
│   │   ├── backtest/          # 回测引擎
│   │   │   ├── engine.py      # 回测引擎
│   │   │   ├── analyzer.py    # 性能分析
│   │   │   └── reporter.py    # 报告生成
│   │   └── paper_trading/     # 模拟交易
│   │       ├── broker.py      # 虚拟Broker
│   │       ├── simulator.py   # 订单模拟
│   │       └── feed.py        # 数据订阅
│   ├── models/                # 数据模型
│   │   ├── strategy.py
│   │   ├── backtest.py
│   │   └── order.py
│   ├── services/              # 外部服务
│   │   ├── data_service.py    # 数据服务客户端
│   │   ├── redis_service.py
│   │   └── kafka_service.py
│   └── utils/                 # 工具函数
│       ├── decorators.py
│       └── performance.py
└── tests/                     # 测试
    ├── unit/
    └── integration/
```

#### 4.2.2 Alpha因子库实现

**因子基类**：

```python
# src/core/factors/library.py
from abc import ABC, abstractmethod
from typing import Dict, Any
import pandas as pd
import numpy as np

class Factor(ABC):
    """因子基类"""
    
    def __init__(self, name: str, category: str, description: str):
        self.name = name
        self.category = category
        self.description = description
    
    @abstractmethod
    def calculate(self, data: pd.DataFrame, **params) -> pd.Series:
        """计算因子值"""
        pass
    
    def validate_data(self, data: pd.DataFrame, required_columns: List[str]):
        """验证数据完整性"""
        missing = set(required_columns) - set(data.columns)
        if missing:
            raise ValueError(f"缺少必需的列: {missing}")

class FactorLibrary:
    """因子库管理器"""
    
    def __init__(self):
        self._factors: Dict[str, Factor] = {}
        self._categories: Dict[str, List[str]] = {}
    
    def register(self, factor: Factor):
        """注册因子"""
        self._factors[factor.name] = factor
        
        if factor.category not in self._categories:
            self._categories[factor.category] = []
        self._categories[factor.category].append(factor.name)
    
    def get_factor(self, name: str) -> Factor:
        """获取因子"""
        if name not in self._factors:
            raise KeyError(f"因子 {name} 不存在")
        return self._factors[name]
    
    def calculate_factors(self, 
                         data: pd.DataFrame,
                         factor_names: List[str],
                         **params) -> pd.DataFrame:
        """批量计算因子"""
        result = pd.DataFrame(index=data.index)
        
        for name in factor_names:
            factor = self.get_factor(name)
            result[name] = factor.calculate(data, **params)
        
        return result
```

**Numba加速示例**：

```python
# src/core/factors/calculator.py
import numba
import numpy as np

@numba.jit(nopython=True)
def fast_rsi(prices: np.ndarray, period: int = 14) -> np.ndarray:
    """优化的RSI计算（性能提升10-100倍）"""
    deltas = np.diff(prices)
    seed = deltas[:period+1]
    up = seed[seed >= 0].sum() / period
    down = -seed[seed < 0].sum() / period
    rs = up / down
    rsi = np.zeros_like(prices)
    rsi[:period] = 100. - 100. / (1. + rs)
    
    for i in range(period, len(prices)):
        delta = deltas[i - 1]
        if delta > 0:
            upval = delta
            downval = 0.
        else:
            upval = 0.
            downval = -delta
        
        up = (up * (period - 1) + upval) / period
        down = (down * (period - 1) + downval) / period
        rs = up / down
        rsi[i] = 100. - 100. / (1. + rs)
    
    return rsi
```

#### 4.2.3 贝叶斯优化实现

```python
# src/core/optimizer/bayesian.py
from skopt import gp_minimize
from skopt.space import Real, Integer
from typing import Dict, Any, Callable

class BayesianOptimizer:
    """贝叶斯优化器"""
    
    def __init__(self, strategy_class, backtest_engine):
        self.strategy_class = strategy_class
        self.backtest_engine = backtest_engine
    
    def optimize(
        self,
        param_space: Dict[str, tuple],
        objective: str = 'sharpe_ratio',
        n_calls: int = 50,
        **backtest_kwargs
    ) -> Dict[str, Any]:
        """
        执行贝叶斯优化
        
        Args:
            param_space: 参数空间 {'fast_period': (5, 30), 'slow_period': (20, 100)}
            objective: 优化目标（sharpe_ratio/returns/calmar_ratio）
            n_calls: 迭代次数
            
        Returns:
            {'best_params': {...}, 'best_score': 1.85, 'history': [...]}
        """
        # 转换参数空间为skopt格式
        dimensions = []
        param_names = []
        for name, (low, high) in param_space.items():
            if isinstance(low, int):
                dimensions.append(Integer(low, high, name=name))
            else:
                dimensions.append(Real(low, high, name=name))
            param_names.append(name)
        
        # 定义目标函数
        def objective_func(params):
            # 构建参数字典
            param_dict = dict(zip(param_names, params))
            
            # 创建策略实例
            strategy = self.strategy_class(**param_dict)
            
            # 运行回测
            result = self.backtest_engine.run(strategy, **backtest_kwargs)
            
            # 返回负值（skopt最小化目标）
            return -result[objective]
        
        # 执行优化
        result = gp_minimize(
            objective_func,
            dimensions,
            n_calls=n_calls,
            random_state=42,
            verbose=True
        )
        
        return {
            'best_params': dict(zip(param_names, result.x)),
            'best_score': -result.fun,
            'n_iterations': n_calls,
            'history': [
                {'params': dict(zip(param_names, x)), 'score': -y}
                for x, y in zip(result.x_iters, result.func_vals)
            ]
        }
```

#### 4.2.4 Walk-Forward分析实现

```python
# src/core/optimizer/walk_forward.py
import pandas as pd
from typing import Dict, List, Any
from datetime import timedelta

class WalkForwardAnalyzer:
    """Walk-Forward分析器"""
    
    def __init__(self, strategy_class, backtest_engine, optimizer):
        self.strategy_class = strategy_class
        self.backtest_engine = backtest_engine
        self.optimizer = optimizer
    
    def analyze(
        self,
        data: pd.DataFrame,
        param_space: Dict[str, tuple],
        training_window: int = 180,  # 天
        testing_window: int = 60,
        step_size: int = 30,
        start_date: str = None,
        end_date: str = None
    ) -> Dict[str, Any]:
        """
        执行Walk-Forward分析
        
        Returns:
            {
                'windows': [
                    {
                        'training_period': ['2023-01-01', '2023-06-30'],
                        'testing_period': ['2023-07-01', '2023-08-30'],
                        'best_params': {...},
                        'training_sharpe': 2.1,
                        'testing_sharpe': 1.8
                    },
                    ...
                ],
                'average_testing_sharpe': 1.75,
                'param_stability': 0.85,
                'performance_degradation': 0.12
            }
        """
        windows = []
        
        # 生成时间窗口
        current_date = pd.to_datetime(start_date)
        end = pd.to_datetime(end_date)
        
        while current_date + timedelta(days=training_window + testing_window) <= end:
            train_start = current_date
            train_end = current_date + timedelta(days=training_window)
            test_start = train_end
            test_end = train_end + timedelta(days=testing_window)
            
            # 训练期数据
            train_data = data[(data.index >= train_start) & (data.index < train_end)]
            
            # 测试期数据
            test_data = data[(data.index >= test_start) & (data.index < test_end)]
            
            # 在训练期优化参数
            opt_result = self.optimizer.optimize(
                param_space=param_space,
                data=train_data,
                n_calls=30  # 减少迭代次数以加快速度
            )
            
            best_params = opt_result['best_params']
            training_sharpe = opt_result['best_score']
            
            # 在测试期验证参数
            strategy = self.strategy_class(**best_params)
            test_result = self.backtest_engine.run(strategy, data=test_data)
            testing_sharpe = test_result['sharpe_ratio']
            
            windows.append({
                'training_period': [
                    train_start.strftime('%Y-%m-%d'),
                    train_end.strftime('%Y-%m-%d')
                ],
                'testing_period': [
                    test_start.strftime('%Y-%m-%d'),
                    test_end.strftime('%Y-%m-%d')
                ],
                'best_params': best_params,
                'training_sharpe': training_sharpe,
                'testing_sharpe': testing_sharpe
            })
            
            # 滚动窗口
            current_date += timedelta(days=step_size)
        
        # 计算汇总指标
        testing_sharpes = [w['testing_sharpe'] for w in windows]
        average_testing_sharpe = np.mean(testing_sharpes)
        
        # 参数稳定性（参数方差）
        param_stability = self._calculate_param_stability(windows)
        
        # 性能退化（训练期vs测试期平均差异）
        training_sharpes = [w['training_sharpe'] for w in windows]
        performance_degradation = (
            np.mean(training_sharpes) - average_testing_sharpe
        ) / np.mean(training_sharpes)
        
        return {
            'windows': windows,
            'average_testing_sharpe': average_testing_sharpe,
            'param_stability': param_stability,
            'performance_degradation': performance_degradation
        }
    
    def _calculate_param_stability(self, windows: List[Dict]) -> float:
        """计算参数稳定性（0-1，越高越稳定）"""
        # 提取所有参数值
        params_by_name = {}
        for window in windows:
            for name, value in window['best_params'].items():
                if name not in params_by_name:
                    params_by_name[name] = []
                params_by_name[name].append(value)
        
        # 计算每个参数的变异系数（CV）
        cvs = []
        for name, values in params_by_name.items():
            mean = np.mean(values)
            std = np.std(values)
            cv = std / mean if mean != 0 else 0
            cvs.append(cv)
        
        # 稳定性 = 1 - 平均CV（归一化到0-1）
        average_cv = np.mean(cvs)
        stability = max(0, 1 - average_cv)
        
        return stability
```

#### 4.2.5 模拟交易系统实现

```python
# src/core/paper_trading/broker.py
from typing import Dict, Optional
import uuid
from datetime import datetime

class PaperTradingBroker:
    """虚拟Broker（与实盘API完全兼容）"""
    
    def __init__(
        self,
        initial_cash: float = 10000.0,
        commission_rate: float = 0.001,  # 0.1%
        slippage_pct: float = 0.001      # 0.1%
    ):
        self.cash = initial_cash
        self.initial_cash = initial_cash
        self.commission_rate = commission_rate
        self.slippage_pct = slippage_pct
        
        self.positions: Dict[str, Position] = {}
        self.orders: Dict[str, Order] = {}
        self.trades: List[Trade] = []
    
    def submit_order(
        self,
        symbol: str,
        side: str,  # 'buy' or 'sell'
        order_type: str,  # 'market' or 'limit'
        quantity: float,
        price: Optional[float] = None
    ) -> str:
        """
        提交订单（与实盘API完全一致）
        
        Returns:
            order_id: 订单ID
        """
        order_id = str(uuid.uuid4())
        
        # 获取当前市场价格（从实时数据订阅）
        current_price = self._get_current_price(symbol)
        
        # 计算成交价（含滑点）
        if order_type == 'market':
            filled_price = self._apply_slippage(current_price, side)
            status = 'filled'
        else:
            filled_price = price
            status = 'pending'  # 限价单等待成交
        
        # 计算手续费
        commission = filled_price * quantity * self.commission_rate
        
        # 创建订单
        order = Order(
            id=order_id,
            symbol=symbol,
            side=side,
            order_type=order_type,
            quantity=quantity,
            price=price,
            filled_price=filled_price if status == 'filled' else None,
            commission=commission if status == 'filled' else 0,
            status=status,
            created_at=datetime.now()
        )
        
        self.orders[order_id] = order
        
        # 如果是市价单，立即成交
        if order_type == 'market':
            self._execute_order(order)
        
        return order_id
    
    def _apply_slippage(self, price: float, side: str) -> float:
        """应用滑点"""
        if side == 'buy':
            return price * (1 + self.slippage_pct)
        else:
            return price * (1 - self.slippage_pct)
    
    def _execute_order(self, order: Order):
        """执行订单"""
        total_cost = order.filled_price * order.quantity + order.commission
        
        if order.side == 'buy':
            # 检查资金是否充足
            if total_cost > self.cash:
                order.status = 'rejected'
                order.reject_reason = 'Insufficient funds'
                return
            
            # 扣除资金
            self.cash -= total_cost
            
            # 更新持仓
            if order.symbol in self.positions:
                pos = self.positions[order.symbol]
                pos.quantity += order.quantity
                pos.avg_price = (
                    (pos.avg_price * (pos.quantity - order.quantity) +
                     order.filled_price * order.quantity) / pos.quantity
                )
            else:
                self.positions[order.symbol] = Position(
                    symbol=order.symbol,
                    quantity=order.quantity,
                    avg_price=order.filled_price
                )
        
        else:  # sell
            # 检查持仓是否充足
            if order.symbol not in self.positions or \
               self.positions[order.symbol].quantity < order.quantity:
                order.status = 'rejected'
                order.reject_reason = 'Insufficient position'
                return
            
            # 增加资金
            self.cash += (order.filled_price * order.quantity - order.commission)
            
            # 更新持仓
            pos = self.positions[order.symbol]
            pos.quantity -= order.quantity
            
            if pos.quantity == 0:
                del self.positions[order.symbol]
        
        # 记录交易
        self.trades.append(Trade(
            order_id=order.id,
            symbol=order.symbol,
            side=order.side,
            quantity=order.quantity,
            price=order.filled_price,
            commission=order.commission,
            timestamp=datetime.now()
        ))
        
        order.status = 'filled'
        order.filled_at = datetime.now()
    
    def get_position(self, symbol: str) -> Optional[Position]:
        """获取持仓"""
        return self.positions.get(symbol)
    
    def get_portfolio_value(self) -> float:
        """获取组合总值"""
        positions_value = sum(
            pos.quantity * self._get_current_price(pos.symbol)
            for pos in self.positions.values()
        )
        return self.cash + positions_value
    
    def get_returns(self) -> float:
        """获取收益率"""
        current_value = self.get_portfolio_value()
        return (current_value - self.initial_cash) / self.initial_cash
```

### 4.3 交易服务（Java）

#### 4.3.1 项目结构

```
trading-service/
├── pom.xml
├── src/
│   ├── main/
│   │   ├── java/
│   │   │   └── com/hermesflow/trading/
│   │   │       ├── TradingServiceApplication.java
│   │   │       ├── controller/
│   │   │       │   ├── OrderController.java
│   │   │       │   ├── PositionController.java
│   │   │       │   └── AccountController.java
│   │   │       ├── service/
│   │   │       │   ├── OrderService.java
│   │   │       │   ├── ExecutionService.java
│   │   │       │   ├── PositionService.java
│   │   │       │   └── SmartRouterService.java
│   │   │       ├── repository/
│   │   │       │   ├── OrderRepository.java
│   │   │       │   ├── PositionRepository.java
│   │   │       │   └── TradeRepository.java
│   │   │       ├── entity/
│   │   │       │   ├── Order.java
│   │   │       │   ├── Position.java
│   │   │       │   └── Trade.java
│   │   │       ├── dto/
│   │   │       │   ├── OrderRequest.java
│   │   │       │   └── OrderResponse.java
│   │   │       ├── exchange/
│   │   │       │   ├── ExchangeClient.java
│   │   │       │   ├── BinanceClient.java
│   │   │       │   ├── OKXClient.java
│   │   │       │   └── IBKRClient.java
│   │   │       ├── config/
│   │   │       │   ├── WebFluxConfig.java
│   │   │       │   └── KafkaConfig.java
│   │   │       └── util/
│   │   │           └── TenantContext.java
│   │   └── resources/
│   │       ├── application.yml
│   │       └── application-prod.yml
│   └── test/
│       └── java/
│           └── com/hermesflow/trading/
│               ├── service/
│               │   └── OrderServiceTest.java
│               └── integration/
│                   └── OrderControllerIT.java
└── Dockerfile
```

#### 4.3.2 智能路由实现

```java
// src/main/java/com/hermesflow/trading/service/SmartRouterService.java
@Service
@Slf4j
public class SmartRouterService {
    
    private final List<ExchangeClient> exchanges;
    private final CircuitBreakerRegistry circuitBreakerRegistry;
    
    public SmartRouterService(
        List<ExchangeClient> exchanges,
        CircuitBreakerRegistry circuitBreakerRegistry
    ) {
        this.exchanges = exchanges;
        this.circuitBreakerRegistry = circuitBreakerRegistry;
    }
    
    /**
     * 智能路由选择最优交易所
     *  
     * 选择标准：
     * 1. 价格优先（买入选最低价，卖出选最高价）
     * 2. 深度充足（订单量可以完全成交）
     * 3. 交易所可用（Circuit Breaker状态正常）
     * 4. 手续费考虑
     */
    public Mono<ExchangeClient> selectBestExchange(
        String symbol,
        OrderSide side,
        BigDecimal quantity
    ) {
        return Flux.fromIterable(exchanges)
            // 过滤掉Circuit Breaker打开的交易所
            .filter(exchange -> {
                String cbName = "exchange-" + exchange.getName();
                CircuitBreaker cb = circuitBreakerRegistry.circuitBreaker(cbName);
                return cb.getState() != CircuitBreaker.State.OPEN;
            })
            // 并行获取每个交易所的价格和深度
            .flatMap(exchange -> 
                exchange.getOrderBook(symbol)
                    .map(orderBook -> new ExchangeScore(
                        exchange,
                        calculateScore(orderBook, side, quantity)
                    ))
                    .onErrorResume(e -> {
                        log.warn("Failed to get orderbook from {}: {}", 
                            exchange.getName(), e.getMessage());
                        return Mono.empty();
                    })
            )
            // 选择得分最高的交易所
            .reduce((score1, score2) -> 
                score1.getScore() > score2.getScore() ? score1 : score2
            )
            .map(ExchangeScore::getExchange)
            .switchIfEmpty(Mono.error(
                new NoExchangeAvailableException("No available exchange for " + symbol)
            ));
    }
    
    private double calculateScore(OrderBook orderBook, OrderSide side, BigDecimal quantity) {
        if (side == OrderSide.BUY) {
            // 买入：选择卖盘（asks）价格最低的
            BigDecimal bestAsk = orderBook.getBestAsk();
            BigDecimal depth = orderBook.getAskDepth(quantity);
            
            // 得分 = 1 / 价格 * 深度权重
            double priceScore = 1.0 / bestAsk.doubleValue();
            double depthScore = Math.min(1.0, depth.divide(quantity).doubleValue());
            
            return priceScore * 0.7 + depthScore * 0.3;
        } else {
            // 卖出：选择买盘（bids）价格最高的
            BigDecimal bestBid = orderBook.getBestBid();
            BigDecimal depth = orderBook.getBidDepth(quantity);
            
            // 得分 = 价格 * 深度权重
            double priceScore = bestBid.doubleValue();
            double depthScore = Math.min(1.0, depth.divide(quantity).doubleValue());
            
            return priceScore * 0.7 + depthScore * 0.3;
        }
    }
    
    @Data
    @AllArgsConstructor
    private static class ExchangeScore {
        private ExchangeClient exchange;
        private double score;
    }
}
```

#### 4.3.3 订单管理实现

```java
// src/main/java/com/hermesflow/trading/service/OrderService.java
@Service
@Slf4j
public class OrderService {
    
    private final OrderRepository orderRepository;
    private final SmartRouterService smartRouter;
    private final ExecutionService executionService;
    private final KafkaTemplate<String, OrderEvent> kafkaTemplate;
    
    /**
     * 创建订单
     */
    @Transactional
    public Mono<OrderResponse> createOrder(OrderRequest request) {
        // 1. 验证订单
        return validateOrder(request)
            // 2. 创建订单实体
            .flatMap(validated -> {
                Order order = Order.builder()
                    .id(UUID.randomUUID())
                    .tenantId(TenantContext.getCurrentTenant())
                    .symbol(request.getSymbol())
                    .side(request.getSide())
                    .type(request.getType())
                    .quantity(request.getQuantity())
                    .price(request.getPrice())
                    .status(OrderStatus.PENDING)
                    .createdAt(Instant.now())
                    .build();
                
                return Mono.fromCallable(() => orderRepository.save(order));
            })
            // 3. 发布订单创建事件
            .doOnNext(order -> publishOrderEvent(order, OrderEventType.CREATED))
            // 4. 异步执行订单
            .flatMap(order -> 
                executionService.executeOrder(order)
                    .thenReturn(order)
            )
            // 5. 转换为响应
            .map(this::toOrderResponse);
    }
    
    /**
     * 取消订单
     */
    @Transactional
    public Mono<Void> cancelOrder(UUID orderId) {
        return Mono.fromCallable(() -> 
                orderRepository.findById(orderId)
                    .orElseThrow(() -> new OrderNotFoundException(orderId))
            )
            .flatMap(order -> {
                // 检查订单状态
                if (order.getStatus() == OrderStatus.FILLED) {
                    return Mono.error(new OrderAlreadyFilledException(orderId));
                }
                
                if (order.getStatus() == OrderStatus.CANCELLED) {
                    return Mono.error(new OrderAlreadyCancelledException(orderId));
                }
                
                // 如果订单已提交到交易所，需要先取消交易所订单
                if (order.getExchangeOrderId() != null) {
                    return executionService.cancelExchangeOrder(order)
                        .then(Mono.fromRunnable(() -> {
                            order.setStatus(OrderStatus.CANCELLED);
                            order.setCancelledAt(Instant.now());
                            orderRepository.save(order);
                        }));
                } else {
                    order.setStatus(OrderStatus.CANCELLED);
                    order.setCancelledAt(Instant.now());
                    orderRepository.save(order);
                    return Mono.empty();
                }
            })
            .doOnSuccess(v -> log.info("Order {} cancelled", orderId));
    }
    
    /**
     * 查询订单列表
     */
    public Flux<OrderResponse> listOrders(
        String symbol,
        OrderStatus status,
        Pageable pageable
    ) {
        UUID tenantId = TenantContext.getCurrentTenant();
        
        return Flux.defer(() -> {
            List<Order> orders;
            if (symbol != null && status != null) {
                orders = orderRepository.findByTenantIdAndSymbolAndStatus(
                    tenantId, symbol, status, pageable
                );
            } else if (symbol != null) {
                orders = orderRepository.findByTenantIdAndSymbol(
                    tenantId, symbol, pageable
                );
            } else if (status != null) {
                orders = orderRepository.findByTenantIdAndStatus(
                    tenantId, status, pageable
                );
            } else {
                orders = orderRepository.findByTenantId(tenantId, pageable);
            }
            
            return Flux.fromIterable(orders);
        }).map(this::toOrderResponse);
    }
    
    private void publishOrderEvent(Order order, OrderEventType eventType) {
        OrderEvent event = OrderEvent.builder()
            .orderId(order.getId())
            .tenantId(order.getTenantId())
            .eventType(eventType)
            .timestamp(Instant.now())
            .order(order)
            .build();
        
        kafkaTemplate.send("order.event." + order.getId(), event);
    }
    
    private OrderResponse toOrderResponse(Order order) {
        return OrderResponse.builder()
            .id(order.getId())
            .symbol(order.getSymbol())
            .side(order.getSide())
            .type(order.getType())
            .quantity(order.getQuantity())
            .price(order.getPrice())
            .filledPrice(order.getFilledPrice())
            .filledQuantity(order.getFilledQuantity())
            .status(order.getStatus())
            .commission(order.getCommission())
            .createdAt(order.getCreatedAt())
            .filledAt(order.getFilledAt())
            .build();
    }
}
```

### 4.4 用户服务（Java）

#### 4.4.1 多租户隔离实现

```java
// src/main/java/com/hermesflow/user/config/TenantInterceptor.java
@Component
@Slf4j
public class TenantInterceptor implements HandlerInterceptor {
    
    @Override
    public boolean preHandle(
        HttpServletRequest request,
        HttpServletResponse response,
        Object handler
    ) throws Exception {
        // 从JWT中提取租户ID
        String token = extractToken(request);
        if (token != null) {
            Claims claims = JwtUtils.parseToken(token);
            UUID tenantId = UUID.fromString(claims.get("tenant_id", String.class));
            
            // 设置租户上下文
            TenantContext.setCurrentTenant(tenantId);
            
            log.debug("Set tenant context: {}", tenantId);
        }
        
        return true;
    }
    
    @Override
    public void afterCompletion(
        HttpServletRequest request,
        HttpServletResponse response,
        Object handler,
        Exception ex
    ) throws Exception {
        // 清除租户上下文
        TenantContext.clear();
    }
    
    private String extractToken(HttpServletRequest request) {
        String bearerToken = request.getHeader("Authorization");
        if (StringUtils.hasText(bearerToken) && bearerToken.startsWith("Bearer ")) {
            return bearerToken.substring(7);
        }
        return null;
    }
}
```

```java
// src/main/java/com/hermesflow/user/util/TenantContext.java
public class TenantContext {
    
    private static final ThreadLocal<UUID> CURRENT_TENANT = new ThreadLocal<>();
    
    public static void setCurrentTenant(UUID tenantId) {
        CURRENT_TENANT.set(tenantId);
    }
    
    public static UUID getCurrentTenant() {
        UUID tenantId = CURRENT_TENANT.get();
        if (tenantId == null) {
            throw new IllegalStateException("No tenant context set");
        }
        return tenantId;
    }
    
    public static void clear() {
        CURRENT_TENANT.remove();
    }
}
```

**PostgreSQL RLS设置**：

```sql
-- 启用RLS
ALTER TABLE orders ENABLE ROW LEVEL SECURITY;

-- 创建租户隔离策略
CREATE POLICY tenant_isolation ON orders
USING (tenant_id = current_setting('app.tenant_id')::uuid);

-- 在每个请求中设置session变量
SET app.tenant_id = '123e4567-e89b-12d3-a456-426614174000';
```

```java
// src/main/java/com/hermesflow/user/repository/TenantAwareRepository.java
@Repository
public class TenantAwareRepository {
    
    @PersistenceContext
    private EntityManager entityManager;
    
    /**
     * 在查询前设置租户上下文
     */
    @Transactional
    public <T> List<T> findAll(Class<T> entityClass) {
        // 设置PostgreSQL session变量
        UUID tenantId = TenantContext.getCurrentTenant();
        entityManager.createNativeQuery(
            String.format("SET app.tenant_id = '%s'", tenantId)
        ).executeUpdate();
        
        // 执行查询（RLS自动过滤）
        return entityManager
            .createQuery("SELECT e FROM " + entityClass.getSimpleName() + " e", entityClass)
            .getResultList();
    }
}
```

### 4.5 风控服务（Java）

#### 4.5.1 规则引擎实现

```java
// src/main/java/com/hermesflow/risk/engine/RuleEngine.java
@Service
@Slf4j
public class RuleEngine {
    
    private final List<RiskRule> rules = new CopyOnWriteArrayList<>();
    private final AlertService alertService;
    
    public RuleEngine(AlertService alertService) {
        this.alertService = alertService;
        initializeDefaultRules();
    }
    
    private void initializeDefaultRules() {
        // 规则1: 单笔最大亏损不超过2%
        rules.add(new RiskRule(
            "MAX_SINGLE_LOSS",
            "单笔最大亏损",
            (context) -> {
                BigDecimal loss = context.getCurrentTradeLoss();
                BigDecimal accountValue = context.getAccountValue();
                BigDecimal lossPercentage = loss.divide(accountValue, 4, RoundingMode.HALF_UP);
                
                return lossPercentage.compareTo(new BigDecimal("0.02")) <= 0;
            },
            AlertLevel.HIGH
        ));
        
        // 规则2: 日总亏损不超过5%
        rules.add(new RiskRule(
            "MAX_DAILY_LOSS",
            "日总亏损",
            (context) -> {
                BigDecimal dailyLoss = context.getDailyLoss();
                BigDecimal accountValue = context.getAccountValue();
                BigDecimal lossPercentage = dailyLoss.divide(accountValue, 4, RoundingMode.HALF_UP);
                
                return lossPercentage.compareTo(new BigDecimal("0.05")) <= 0;
            },
            AlertLevel.CRITICAL
        ));
        
        // 规则3: 单策略回撤超过10%自动停止
        rules.add(new RiskRule(
            "MAX_STRATEGY_DRAWDOWN",
            "策略最大回撤",
            (context) -> {
                BigDecimal drawdown = context.getStrategyDrawdown();
                return drawdown.compareTo(new BigDecimal("0.10")) <= 0;
            },
            AlertLevel.CRITICAL,
            (context) -> {
                // 自动停止策略
                UUID strategyId = context.getStrategyId();
                log.warn("Stopping strategy {} due to drawdown > 10%", strategyId);
                // TODO: 调用策略引擎停止策略
            }
        ));
        
        // 规则4: 杠杆率不超过5倍
        rules.add(new RiskRule(
            "MAX_LEVERAGE",
            "最大杠杆率",
            (context) -> {
                BigDecimal leverage = context.getLeverage();
                return leverage.compareTo(new BigDecimal("5.0")) <= 0;
            },
            AlertLevel.HIGH
        ));
    }
    
    /**
     * 评估所有风控规则
     */
    public RiskEvaluationResult evaluate(RiskContext context) {
        List<RuleViolation> violations = new ArrayList<>();
        
        for (RiskRule rule : rules) {
            try {
                boolean passed = rule.getCondition().test(context);
                
                if (!passed) {
                    log.warn("Risk rule violated: {}", rule.getName());
                    
                    violations.add(new RuleViolation(
                        rule.getCode(),
                        rule.getName(),
                        rule.getAlertLevel()
                    ));
                    
                    // 触发告警
                    alertService.sendAlert(
                        context.getTenantId(),
                        rule.getName(),
                        rule.getAlertLevel(),
                        "风控规则触发: " + rule.getName()
                    );
                    
                    // 执行自动操作
                    if (rule.getAction() != null) {
                        rule.getAction().accept(context);
                    }
                }
            } catch (Exception e) {
                log.error("Error evaluating rule {}: {}", rule.getCode(), e.getMessage(), e);
            }
        }
        
        return new RiskEvaluationResult(
            context.getTenantId(),
            violations.isEmpty(),
            violations,
            Instant.now()
        );
    }
    
    /**
     * 添加自定义规则
     */
    public void addRule(RiskRule rule) {
        rules.add(rule);
        log.info("Added custom risk rule: {}", rule.getName());
    }
    
    /**
     * 移除规则
     */
    public void removeRule(String ruleCode) {
        rules.removeIf(rule -> rule.getCode().equals(ruleCode));
        log.info("Removed risk rule: {}", ruleCode);
    }
}
```

```java
// src/main/java/com/hermesflow/risk/model/RiskRule.java
@Data
@AllArgsConstructor
public class RiskRule {
    private String code;
    private String name;
    private Predicate<RiskContext> condition;
    private AlertLevel alertLevel;
    private Consumer<RiskContext> action;  // 可选的自动操作
    
    public RiskRule(String code, String name, Predicate<RiskContext> condition, AlertLevel alertLevel) {
        this(code, name, condition, alertLevel, null);
    }
}
```

---

## 5. 数据架构设计

### 5.1 PostgreSQL设计

#### 5.1.1 多租户隔离（Row-Level Security）

**启用RLS策略**：

```sql
-- 1. 启用RLS
ALTER TABLE orders ENABLE ROW LEVEL SECURITY;
ALTER TABLE positions ENABLE ROW LEVEL SECURITY;
ALTER TABLE strategies ENABLE ROW LEVEL SECURITY;
ALTER TABLE trades ENABLE ROW LEVEL SECURITY;

-- 2. 创建租户隔离策略
CREATE POLICY tenant_isolation_orders ON orders
USING (tenant_id = current_setting('app.tenant_id')::uuid);

CREATE POLICY tenant_isolation_positions ON positions
USING (tenant_id = current_setting('app.tenant_id')::uuid);

CREATE POLICY tenant_isolation_strategies ON strategies
USING (tenant_id = current_setting('app.tenant_id')::uuid);

CREATE POLICY tenant_isolation_trades ON trades
USING (tenant_id = current_setting('app.tenant_id')::uuid);

-- 3. 为管理员创建旁路策略
CREATE POLICY admin_bypass ON orders
TO admin_role
USING (true);
```

**在应用层设置Session变量**：

```java
// 在每个数据库事务开始时设置
@Aspect
@Component
public class TenantAspect {
    
    @PersistenceContext
    private EntityManager entityManager;
    
    @Before("@annotation(org.springframework.transaction.annotation.Transactional)")
    public void setTenantContext() {
        UUID tenantId = TenantContext.getCurrentTenant();
        
        entityManager.createNativeQuery(
            String.format("SET app.tenant_id = '%s'", tenantId)
        ).executeUpdate();
    }
}
```

#### 5.1.2 核心表设计

**用户表**：

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    name VARCHAR(100),
    role VARCHAR(50) NOT NULL DEFAULT 'user',
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    last_login_at TIMESTAMP,
    
    CONSTRAINT users_email_check CHECK (email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}$')
);

CREATE INDEX idx_users_tenant ON users(tenant_id);
CREATE INDEX idx_users_email ON users(email);
```

**策略表**：

```sql
CREATE TABLE strategies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID NOT NULL REFERENCES users(id),
    name VARCHAR(200) NOT NULL,
    description TEXT,
    code TEXT NOT NULL,
    parameters JSONB,
    status VARCHAR(20) NOT NULL DEFAULT 'inactive',
    strategy_type VARCHAR(50) NOT NULL,
    symbols TEXT[],
    
    -- 性能指标
    total_returns DECIMAL(10, 4),
    sharpe_ratio DECIMAL(10, 4),
    max_drawdown DECIMAL(10, 4),
    win_rate DECIMAL(5, 2),
    
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    activated_at TIMESTAMP,
    deactivated_at TIMESTAMP,
    
    CONSTRAINT strategies_status_check CHECK (status IN ('active', 'inactive', 'paused', 'error'))
);

CREATE INDEX idx_strategies_tenant ON strategies(tenant_id);
CREATE INDEX idx_strategies_user ON strategies(user_id);
CREATE INDEX idx_strategies_status ON strategies(status);
CREATE INDEX idx_strategies_params ON strategies USING GIN (parameters);
```

**订单表**：

```sql
CREATE TABLE orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    strategy_id UUID REFERENCES strategies(id),
    user_id UUID NOT NULL REFERENCES users(id),
    
    symbol VARCHAR(50) NOT NULL,
    exchange VARCHAR(50) NOT NULL,
    side VARCHAR(10) NOT NULL,
    order_type VARCHAR(20) NOT NULL,
    
    quantity DECIMAL(18, 8) NOT NULL,
    price DECIMAL(18, 8),
    filled_price DECIMAL(18, 8),
    filled_quantity DECIMAL(18, 8) DEFAULT 0,
    
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    commission DECIMAL(18, 8) DEFAULT 0,
    
    exchange_order_id VARCHAR(100),
    reject_reason TEXT,
    
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    filled_at TIMESTAMP,
    cancelled_at TIMESTAMP,
    
    CONSTRAINT orders_side_check CHECK (side IN ('buy', 'sell')),
    CONSTRAINT orders_type_check CHECK (order_type IN ('market', 'limit', 'stop')),
    CONSTRAINT orders_status_check CHECK (status IN ('pending', 'filled', 'partially_filled', 'cancelled', 'rejected'))
);

CREATE INDEX idx_orders_tenant ON orders(tenant_id);
CREATE INDEX idx_orders_strategy ON orders(strategy_id);
CREATE INDEX idx_orders_symbol ON orders(symbol);
CREATE INDEX idx_orders_status ON orders(status);
CREATE INDEX idx_orders_created ON orders(created_at DESC);
```

**持仓表**：

```sql
CREATE TABLE positions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    strategy_id UUID REFERENCES strategies(id),
    user_id UUID NOT NULL REFERENCES users(id),
    
    symbol VARCHAR(50) NOT NULL,
    exchange VARCHAR(50) NOT NULL,
    
    quantity DECIMAL(18, 8) NOT NULL,
    avg_price DECIMAL(18, 8) NOT NULL,
    current_price DECIMAL(18, 8),
    
    unrealized_pnl DECIMAL(18, 8),
    realized_pnl DECIMAL(18, 8) DEFAULT 0,
    
    opened_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    closed_at TIMESTAMP,
    
    UNIQUE (tenant_id, symbol, strategy_id)
);

CREATE INDEX idx_positions_tenant ON positions(tenant_id);
CREATE INDEX idx_positions_strategy ON positions(strategy_id);
CREATE INDEX idx_positions_symbol ON positions(symbol);
```

#### 5.1.3 索引优化策略

```sql
-- 复合索引（查询热路径）
CREATE INDEX idx_orders_tenant_status_created ON orders(tenant_id, status, created_at DESC);
CREATE INDEX idx_orders_strategy_symbol ON orders(strategy_id, symbol);

-- GIN索引（JSONB查询）
CREATE INDEX idx_strategies_parameters ON strategies USING GIN (parameters jsonb_path_ops);

-- 部分索引（减少索引大小）
CREATE INDEX idx_orders_active ON orders(tenant_id, symbol) 
WHERE status IN ('pending', 'partially_filled');

-- BRIN索引（时序数据）
CREATE INDEX idx_orders_created_brin ON orders USING BRIN (created_at);
```

### 5.2 ClickHouse设计

#### 5.2.1 表结构设计

**市场数据表（分布式）**：

```sql
-- 本地表
CREATE TABLE market_data_local ON CLUSTER '{cluster}' (
    timestamp DateTime64(6),
    symbol String,
    exchange LowCardinality(String),
    
    -- OHLCV数据
    open Decimal(18, 8),
    high Decimal(18, 8),
    low Decimal(18, 8),
    close Decimal(18, 8),
    volume Decimal(18, 8),
    
    -- Tick数据
    bid Decimal(18, 8),
    ask Decimal(18, 8),
    bid_size Decimal(18, 8),
    ask_size Decimal(18, 8),
    
    -- 元数据
    data_type LowCardinality(String),  -- 'tick', 'kline_1m', 'kline_5m'
    tenant_id UUID
    
) ENGINE = ReplicatedMergeTree('/clickhouse/tables/{shard}/market_data', '{replica}')
PARTITION BY toYYYYMMDD(timestamp)
ORDER BY (symbol, timestamp)
TTL timestamp + INTERVAL 365 DAY
SETTINGS index_granularity = 8192;

-- 分布式表
CREATE TABLE market_data ON CLUSTER '{cluster}' AS market_data_local
ENGINE = Distributed('{cluster}', default, market_data_local, rand());
```

**因子计算结果表**：

```sql
CREATE TABLE factor_values ON CLUSTER '{cluster}' (
    timestamp DateTime64(3),
    symbol String,
    strategy_id UUID,
    tenant_id UUID,
    
    factor_name LowCardinality(String),
    factor_value Float64,
    
    INDEX idx_factor_name factor_name TYPE bloom_filter GRANULARITY 1
    
) ENGINE = ReplicatedMergeTree('/clickhouse/tables/{shard}/factor_values', '{replica}')
PARTITION BY toYYYYMM(timestamp)
ORDER BY (tenant_id, strategy_id, symbol, timestamp)
TTL timestamp + INTERVAL 90 DAY;
```

**回测结果表**：

```sql
CREATE TABLE backtest_results ON CLUSTER '{cluster}' (
    backtest_id UUID,
    tenant_id UUID,
    strategy_id UUID,
    
    timestamp DateTime,
    equity Decimal(18, 8),
    returns Decimal(10, 6),
    drawdown Decimal(10, 6),
    
    trade_count UInt32,
    win_count UInt32,
    loss_count UInt32,
    
    parameters String  -- JSON string
    
) ENGINE = ReplicatedMergeTree('/clickhouse/tables/{shard}/backtest_results', '{replica}')
PARTITION BY toYYYYMM(timestamp)
ORDER BY (tenant_id, strategy_id, backtest_id, timestamp);
```

#### 5.2.2 物化视图（预聚合）

```sql
-- 1分钟K线聚合（从Tick数据）
CREATE MATERIALIZED VIEW kline_1m_mv ON CLUSTER '{cluster}'
TO market_data_local
AS SELECT
    toStartOfMinute(timestamp) AS timestamp,
    symbol,
    exchange,
    argMin(close, timestamp) AS open,
    max(close) AS high,
    min(close) AS low,
    argMax(close, timestamp) AS close,
    sum(volume) AS volume,
    0 AS bid,
    0 AS ask,
    0 AS bid_size,
    0 AS ask_size,
    'kline_1m' AS data_type,
    tenant_id
FROM market_data_local
WHERE data_type = 'tick'
GROUP BY timestamp, symbol, exchange, tenant_id;

-- 日收益统计视图
CREATE MATERIALIZED VIEW daily_returns_mv ON CLUSTER '{cluster}'
ENGINE = SummingMergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (tenant_id, strategy_id, date)
AS SELECT
    toDate(timestamp) AS date,
    tenant_id,
    strategy_id,
    sum(returns) AS daily_return,
    count() AS trade_count
FROM backtest_results
GROUP BY date, tenant_id, strategy_id;
```

#### 5.2.3 查询优化示例

```sql
-- 优化前（全表扫描）
SELECT * FROM market_data
WHERE symbol = 'BTCUSDT'
AND timestamp >= now() - INTERVAL 1 DAY;

-- 优化后（利用ORDER BY和分区）
SELECT * FROM market_data
WHERE symbol = 'BTCUSDT'
AND toYYYYMMDD(timestamp) >= toYYYYMMDD(now() - INTERVAL 1 DAY)
AND timestamp >= now() - INTERVAL 1 DAY
ORDER BY timestamp;

-- 使用PREWHERE优化（先过滤再读取）
SELECT * FROM market_data
PREWHERE symbol = 'BTCUSDT'
WHERE timestamp >= now() - INTERVAL 1 DAY;
```

### 5.3 Redis设计

#### 5.3.1 Key命名规范

**命名模式**：

```
{service}:{tenant_id}:{resource}:{id}:{field}
```

**示例**：

```
# 市场数据
market:global:btcusdt:latest          # 最新价格（全局）
market:global:btcusdt:orderbook:10    # 订单簿（10档）

# 用户数据
user:{tenant_id}:session:{session_id}  # 用户会话
user:{tenant_id}:api_keys              # API密钥列表

# 策略数据
strategy:{tenant_id}:{strategy_id}:state       # 策略状态
strategy:{tenant_id}:{strategy_id}:positions   # 持仓信息

# 因子缓存
factor:{tenant_id}:{symbol}:rsi:14     # RSI(14)因子值
factor:{tenant_id}:{symbol}:ma:20      # MA(20)因子值

# 实时订单
order:{tenant_id}:{order_id}:status    # 订单状态
order:{tenant_id}:pending              # 待处理订单队列
```

#### 5.3.2 数据结构使用

**String（最新价格）**：

```redis
SET market:global:btcusdt:latest "67850.23" EX 60
GET market:global:btcusdt:latest
```

**Hash（订单详情）**：

```redis
HSET order:{tenant_id}:{order_id} 
  symbol "BTCUSDT" 
  side "buy" 
  quantity "0.1" 
  price "67000" 
  status "filled"
  
HGETALL order:{tenant_id}:{order_id}
```

**ZSet（排行榜/时序数据）**：

```redis
# 策略收益排行榜
ZADD strategy:leaderboard 0.25 "strategy_1" 0.18 "strategy_2" 0.32 "strategy_3"
ZREVRANGE strategy:leaderboard 0 9 WITHSCORES  # Top 10

# 时序价格数据（最近100个Tick）
ZADD market:global:btcusdt:ticks 1703001234567 "67850.23"
ZRANGE market:global:btcusdt:ticks -100 -1 WITHSCORES
```

**List（订单队列）**：

```redis
# 待处理订单队列
LPUSH order:{tenant_id}:pending "{order_json}"
BRPOP order:{tenant_id}:pending 5  # 阻塞弹出（5秒超时）
```

**PubSub（实时推送）**：

```redis
# 发布市场数据
PUBLISH market:btcusdt '{"price": 67850.23, "timestamp": 1703001234567}'

# 订阅市场数据
SUBSCRIBE market:btcusdt
PSUBSCRIBE market:*  # 订阅所有市场数据
```

#### 5.3.3 缓存策略

**因子缓存（Cache-Aside）**：

```python
def get_factor_value(tenant_id: str, symbol: str, factor_name: str, period: int):
    cache_key = f"factor:{tenant_id}:{symbol}:{factor_name}:{period}"
    
    # 1. 尝试从缓存读取
    cached_value = redis.get(cache_key)
    if cached_value is not None:
        return float(cached_value)
    
    # 2. 缓存未命中，计算因子
    value = calculate_factor(symbol, factor_name, period)
    
    # 3. 写入缓存（TTL: 60秒）
    redis.setex(cache_key, 60, str(value))
    
    return value
```

**订单状态缓存（Write-Through）**：

```java
public void updateOrderStatus(UUID orderId, OrderStatus newStatus) {
    Order order = orderRepository.findById(orderId)
        .orElseThrow(() -> new OrderNotFoundException(orderId));
    
    order.setStatus(newStatus);
    
    // 1. 写入数据库
    orderRepository.save(order);
    
    // 2. 同步写入缓存
    String cacheKey = String.format("order:%s:%s:status", 
        order.getTenantId(), orderId);
    redisTemplate.opsForValue().set(cacheKey, newStatus.name(), 5, TimeUnit.MINUTES);
}
```

### 5.4 Kafka设计

#### 5.4.1 Topic设计

```
# 市场数据
market.data.{exchange}.{symbol}       # 例如: market.data.binance.btcusdt
market.data.aggregated               # 聚合市场数据

# 策略信号
strategy.signal.{strategy_id}        # 策略信号
strategy.performance.{strategy_id}   # 策略性能指标

# 订单事件
order.event.created                  # 订单创建
order.event.filled                   # 订单成交
order.event.cancelled                # 订单取消

# 风险告警
risk.alert.{tenant_id}               # 租户风险告警
risk.alert.critical                  # 严重风险告警（全局）

# 系统事件
system.audit                         # 审计日志
system.metrics                       # 系统指标
```

#### 5.4.2 消息格式（Avro Schema）

**MarketData消息**：

```json
{
  "type": "record",
  "name": "MarketData",
  "namespace": "com.hermesflow.data",
  "fields": [
    {"name": "timestamp", "type": "long"},
    {"name": "symbol", "type": "string"},
    {"name": "exchange", "type": "string"},
    {"name": "price", "type": "double"},
    {"name": "volume", "type": "double"},
    {"name": "bid", "type": "double"},
    {"name": "ask", "type": "double"}
  ]
}
```

**OrderEvent消息**：

```json
{
  "type": "record",
  "name": "OrderEvent",
  "namespace": "com.hermesflow.trading",
  "fields": [
    {"name": "event_id", "type": "string"},
    {"name": "timestamp", "type": "long"},
    {"name": "tenant_id", "type": "string"},
    {"name": "order_id", "type": "string"},
    {"name": "event_type", "type": {"type": "enum", "name": "EventType", "symbols": ["CREATED", "FILLED", "CANCELLED"]}},
    {"name": "order", "type": {
      "type": "record",
      "name": "Order",
      "fields": [
        {"name": "symbol", "type": "string"},
        {"name": "side", "type": "string"},
        {"name": "quantity", "type": "double"},
        {"name": "price", "type": ["null", "double"], "default": null}
      ]
    }}
  ]
}
```

#### 5.4.3 分区策略

```java
// 按symbol分区（确保同一symbol的消息有序）
ProducerRecord<String, MarketData> record = new ProducerRecord<>(
    "market.data.binance.btcusdt",
    "BTCUSDT",  // Key: symbol
    marketData
);

// 按tenant_id分区（租户隔离）
ProducerRecord<String, OrderEvent> record = new ProducerRecord<>(
    "order.event.created",
    tenantId.toString(),  // Key: tenant_id
    orderEvent
);
```

#### 5.4.4 消费者组配置

```java
@Configuration
public class KafkaConsumerConfig {
    
    @Bean
    public ConsumerFactory<String, MarketData> marketDataConsumerFactory() {
        Map<String, Object> props = new HashMap<>();
        props.put(ConsumerConfig.BOOTSTRAP_SERVERS_CONFIG, "localhost:9092");
        props.put(ConsumerConfig.GROUP_ID_CONFIG, "strategy-engine-group");
        props.put(ConsumerConfig.KEY_DESERIALIZER_CLASS_CONFIG, StringDeserializer.class);
        props.put(ConsumerConfig.VALUE_DESERIALIZER_CLASS_CONFIG, AvroDeserializer.class);
        props.put(ConsumerConfig.AUTO_OFFSET_RESET_CONFIG, "earliest");
        props.put(ConsumerConfig.ENABLE_AUTO_COMMIT_CONFIG, false);  // 手动提交
        props.put(ConsumerConfig.MAX_POLL_RECORDS_CONFIG, 500);
        
        return new DefaultKafkaConsumerFactory<>(props);
    }
}
```

---

## 6. 部署架构设计

### 6.1 本地开发环境（Docker Compose）

**完整docker-compose.yml**：

```yaml
version: '3.8'

services:
  # 前端应用
  frontend:
    build:
      context: ./modules/frontend
      dockerfile: ../../scripts/frontend/Dockerfile
    ports:
      - "3000:3000"
    environment:
      - VITE_API_URL=http://localhost:8080
      - VITE_WS_URL=ws://localhost:8080/ws
    depends_on:
      - gateway
  
  # API Gateway
  gateway:
    build:
      context: ./modules/gateway
      dockerfile: ../../scripts/gateway/Dockerfile
    ports:
      - "8080:8080"
    environment:
      - SPRING_PROFILES_ACTIVE=dev
      - SPRING_CLOUD_GATEWAY_ROUTES_0_ID=data-service
      - SPRING_CLOUD_GATEWAY_ROUTES_0_URI=http://data-service:8081
    depends_on:
      - data-service
      - strategy-engine
      - trading-service
      - user-service
  
  # 数据服务（Rust）
  data-service:
    build:
      context: ./modules/data-engine
      dockerfile: ../../scripts/data-engine/Dockerfile
    ports:
      - "8081:8081"
      - "50051:50051"  # gRPC
    environment:
      - RUST_LOG=info
      - DATABASE_URL=postgres://hermesflow:password@postgres:5432/hermesflow
      - REDIS_URL=redis://redis:6379
      - CLICKHOUSE_URL=http://clickhouse:8123
      - KAFKA_BROKERS=kafka:9092
    depends_on:
      - postgres
      - redis
      - clickhouse
      - kafka
  
  # 策略引擎（Python）
  strategy-engine:
    build:
      context: ./modules/strategy-engine
      dockerfile: ../../scripts/strategy-engine/Dockerfile
    ports:
      - "8082:8082"
    environment:
      - PYTHONUNBUFFERED=1
      - DATABASE_URL=postgresql://hermesflow:password@postgres:5432/hermesflow
      - REDIS_URL=redis://redis:6379
      - KAFKA_BROKERS=kafka:9092
      - DATA_SERVICE_GRPC_URL=data-service:50051
    depends_on:
      - postgres
      - redis
      - kafka
      - data-service
  
  # 交易服务（Java）
  trading-service:
    build:
      context: ./modules/trading-engine
      dockerfile: ../../scripts/trading-engine/Dockerfile
    ports:
      - "8083:8083"
    environment:
      - SPRING_PROFILES_ACTIVE=dev
      - SPRING_DATASOURCE_URL=jdbc:postgresql://postgres:5432/hermesflow
      - SPRING_DATASOURCE_USERNAME=hermesflow
      - SPRING_DATASOURCE_PASSWORD=password
      - SPRING_KAFKA_BOOTSTRAP_SERVERS=kafka:9092
    depends_on:
      - postgres
      - kafka
  
  # 用户服务（Java）
  user-service:
    build:
      context: ./modules/user-management
      dockerfile: ../../scripts/user-management/Dockerfile
    ports:
      - "8084:8084"
    environment:
      - SPRING_PROFILES_ACTIVE=dev
      - SPRING_DATASOURCE_URL=jdbc:postgresql://postgres:5432/hermesflow
      - SPRING_DATASOURCE_USERNAME=hermesflow
      - SPRING_DATASOURCE_PASSWORD=password
    depends_on:
      - postgres
  
  # 风控服务（Java）
  risk-service:
    build:
      context: ./modules/risk-engine
      dockerfile: ../../scripts/risk-engine/Dockerfile
    ports:
      - "8085:8085"
    environment:
      - SPRING_PROFILES_ACTIVE=dev
      - SPRING_DATASOURCE_URL=jdbc:postgresql://postgres:5432/hermesflow
      - SPRING_KAFKA_BOOTSTRAP_SERVERS=kafka:9092
    depends_on:
      - postgres
      - kafka
  
  # PostgreSQL
  postgres:
    image: postgres:16-alpine
    ports:
      - "5432:5432"
    environment:
      - POSTGRES_DB=hermesflow
      - POSTGRES_USER=hermesflow
      - POSTGRES_PASSWORD=password
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./infrastructure/init-scripts/postgres:/docker-entrypoint-initdb.d
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U hermesflow"]
      interval: 10s
      timeout: 5s
      retries: 5
  
  # ClickHouse
  clickhouse:
    image: clickhouse/clickhouse-server:latest
    ports:
      - "8123:8123"  # HTTP
      - "9000:9000"  # Native
    environment:
      - CLICKHOUSE_DB=hermesflow
      - CLICKHOUSE_USER=hermesflow
      - CLICKHOUSE_PASSWORD=password
    volumes:
      - clickhouse_data:/var/lib/clickhouse
      - ./infrastructure/init-scripts/clickhouse:/docker-entrypoint-initdb.d
    ulimits:
      nofile:
        soft: 262144
        hard: 262144
  
  # Redis
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    command: redis-server --appendonly yes --requirepass password
    volumes:
      - redis_data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "--raw", "incr", "ping"]
      interval: 10s
      timeout: 3s
      retries: 5
  
  # Kafka
  zookeeper:
    image: confluentinc/cp-zookeeper:latest
    environment:
      ZOOKEEPER_CLIENT_PORT: 2181
      ZOOKEEPER_TICK_TIME: 2000
    volumes:
      - zookeeper_data:/var/lib/zookeeper
  
  kafka:
    image: confluentinc/cp-kafka:latest
    ports:
      - "9092:9092"
    environment:
      KAFKA_BROKER_ID: 1
      KAFKA_ZOOKEEPER_CONNECT: zookeeper:2181
      KAFKA_ADVERTISED_LISTENERS: PLAINTEXT://kafka:9092
      KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR: 1
      KAFKA_AUTO_CREATE_TOPICS_ENABLE: "true"
    volumes:
      - kafka_data:/var/lib/kafka/data
    depends_on:
      - zookeeper
  
  # Prometheus
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./infrastructure/prometheus/prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
  
  # Grafana
  grafana:
    image: grafana/grafana:latest
    ports:
      - "3001:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
      - GF_INSTALL_PLUGINS=redis-datasource
    volumes:
      - ./infrastructure/grafana/dashboards:/etc/grafana/provisioning/dashboards
      - ./infrastructure/grafana/datasources:/etc/grafana/provisioning/datasources
      - grafana_data:/var/lib/grafana
    depends_on:
      - prometheus

volumes:
  postgres_data:
  clickhouse_data:
  redis_data:
  zookeeper_data:
  kafka_data:
  prometheus_data:
  grafana_data:
```

### 6.2 Kubernetes部署

#### 6.2.1 Helm Chart结构

**base-charts/microservice/values.yaml**：

```yaml
# 默认值（可被具体服务覆盖）
replicaCount: 1

image:
  repository: ""
  tag: "latest"
  pullPolicy: IfNotPresent

service:
  type: ClusterIP
  port: 8080
  targetPort: 8080

resources:
  limits:
    cpu: 1000m
    memory: 1Gi
  requests:
    cpu: 500m
    memory: 512Mi

autoscaling:
  enabled: false
  minReplicas: 2
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70
  targetMemoryUtilizationPercentage: 80

env: []

configMap:
  enabled: false
  data: {}

secret:
  enabled: false
  data: {}

probes:
  liveness:
    enabled: true
    path: /health
    initialDelaySeconds: 30
    periodSeconds: 10
  readiness:
    enabled: true
    path: /health
    initialDelaySeconds: 10
    periodSeconds: 5
```

**base-charts/microservice/templates/deployment.yaml**：

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ include "microservice.fullname" . }}
  labels:
    {{- include "microservice.labels" . | nindent 4 }}
spec:
  {{- if not .Values.autoscaling.enabled }}
  replicas: {{ .Values.replicaCount }}
  {{- end }}
  selector:
    matchLabels:
      {{- include "microservice.selectorLabels" . | nindent 6 }}
  template:
    metadata:
      labels:
        {{- include "microservice.selectorLabels" . | nindent 8 }}
    spec:
      containers:
      - name: {{ .Chart.Name }}
        image: "{{ .Values.image.repository }}:{{ .Values.image.tag }}"
        imagePullPolicy: {{ .Values.image.pullPolicy }}
        ports:
        - name: http
          containerPort: {{ .Values.service.targetPort }}
          protocol: TCP
        {{- if .Values.probes.liveness.enabled }}
        livenessProbe:
          httpGet:
            path: {{ .Values.probes.liveness.path }}
            port: http
          initialDelaySeconds: {{ .Values.probes.liveness.initialDelaySeconds }}
          periodSeconds: {{ .Values.probes.liveness.periodSeconds }}
        {{- end }}
        {{- if .Values.probes.readiness.enabled }}
        readinessProbe:
          httpGet:
            path: {{ .Values.probes.readiness.path }}
            port: http
          initialDelaySeconds: {{ .Values.probes.readiness.initialDelaySeconds }}
          periodSeconds: {{ .Values.probes.readiness.periodSeconds }}
        {{- end }}
        resources:
          {{- toYaml .Values.resources | nindent 12 }}
        env:
          {{- range .Values.env }}
          - name: {{ .name }}
            {{- if .value }}
            value: {{ .value | quote }}
            {{- else if .valueFrom }}
            valueFrom:
              {{- toYaml .valueFrom | nindent 14 }}
            {{- end }}
          {{- end }}
```

**apps/dev/data-engine/values.yaml**：

```yaml
# 继承base chart并覆盖特定值
image:
  repository: hermesflow/data-service
  tag: "v0.1.0"

replicaCount: 1

service:
  port: 8081
  targetPort: 8081

resources:
  limits:
    cpu: 2000m
    memory: 2Gi
  requests:
    cpu: 1000m
    memory: 1Gi

env:
  - name: RUST_LOG
    value: "info"
  - name: DATABASE_URL
    valueFrom:
      secretKeyRef:
        name: postgres-secret
        key: connection-string
  - name: REDIS_URL
    value: "redis://redis-service:6379"
  - name: KAFKA_BROKERS
    value: "kafka-service:9092"

autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 5
  targetCPUUtilizationPercentage: 70
```

#### 6.2.2 ArgoCD配置

**HermesFlow-GitOps/apps/dev/application.yaml**：

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: hermesflow-dev
  namespace: argocd
spec:
  project: default
  
  source:
    repoURL: https://github.com/your-org/HermesFlow-GitOps.git
    targetRevision: HEAD
    path: apps/dev
  
  destination:
    server: https://kubernetes.default.svc
    namespace: hermesflow-dev
  
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
    syncOptions:
      - CreateNamespace=true
```

### 6.3 Azure部署架构

#### 6.3.1 Azure资源拓扑

```
Azure Subscription
└── Resource Group: hermesflow-prod
    ├── AKS Cluster: hermesflow-aks-prod
    │   ├── Node Pool: system (3 nodes, Standard_D4s_v3)
    │   ├── Node Pool: app (5 nodes, Standard_D8s_v3, auto-scale)
    │   └── Node Pool: data (3 nodes, Standard_E8s_v3, memory-optimized)
    │
    ├── Azure Database for PostgreSQL Flexible Server
    │   ├── SKU: General Purpose, 4 vCores, 16 GB RAM
    │   ├── Storage: 256 GB, Auto-grow enabled
    │   └── High Availability: Zone-redundant
    │
    ├── Azure Cache for Redis
    │   ├── SKU: Premium P1 (6 GB)
    │   └── Persistence: RDB backup enabled
    │
    ├── Azure Event Hubs (Kafka-compatible)
    │   ├── Namespace: hermesflow-events
    │   ├── Throughput Units: 10 (auto-inflate enabled)
    │   └── Retention: 7 days
    │
    ├── Azure Container Registry
    │   ├── SKU: Premium
    │   └── Geo-replication: Enabled
    │
    ├── Azure Blob Storage
    │   ├── Container: logs (Hot tier)
    │   ├── Container: backups (Cool tier)
    │   └── Lifecycle Management: Archive after 90 days
    │
    ├── Azure Monitor
    │   ├── Log Analytics Workspace
    │   ├── Application Insights
    │   └── Alert Rules
    │
    └── Azure Key Vault
        ├── Secrets: DB passwords, API keys
        └── Certificates: TLS certs
```

#### 6.3.2 Terraform配置示例

**infrastructure/terraform/environments/prod/main.tf**：

```hcl
terraform {
  required_version = ">= 1.0"
  
  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 3.0"
    }
  }
  
  backend "azurerm" {
    resource_group_name  = "hermesflow-tfstate"
    storage_account_name = "hermesflowstate"
    container_name       = "tfstate"
    key                  = "prod.terraform.tfstate"
  }
}

provider "azurerm" {
  features {}
}

module "aks" {
  source = "../../modules/aks"
  
  resource_group_name = azurerm_resource_group.main.name
  location            = var.location
  cluster_name        = "hermesflow-aks-prod"
  
  system_node_pool = {
    name       = "system"
    node_count = 3
    vm_size    = "Standard_D4s_v3"
  }
  
  app_node_pool = {
    name            = "app"
    min_count       = 3
    max_count       = 10
    vm_size         = "Standard_D8s_v3"
    enable_autoscale = true
  }
}

module "postgresql" {
  source = "../../modules/postgresql"
  
  resource_group_name = azurerm_resource_group.main.name
  location            = var.location
  server_name         = "hermesflow-postgres-prod"
  
  sku_name   = "GP_Gen5_4"
  storage_mb = 262144  # 256 GB
  
  high_availability_enabled = true
}

module "redis" {
  source = "../../modules/redis"
  
  resource_group_name = azurerm_resource_group.main.name
  location            = var.location
  redis_name          = "hermesflow-redis-prod"
  
  sku_name = "Premium"
  family   = "P"
  capacity = 1
  
  enable_non_ssl_port = false
  enable_persistence  = true
}
```

---

## 7. 安全架构设计

### 7.1 认证与授权

#### 7.1.1 JWT认证流程

```mermaid
sequenceDiagram
    participant User
    participant Frontend
    participant Gateway
    participant UserService
    participant JWT
    
    User->>Frontend: 输入用户名/密码
    Frontend->>Gateway: POST /api/v1/auth/login
    Gateway->>UserService: 转发登录请求
    UserService->>UserService: 验证凭证
    UserService->>JWT: 生成Token
    JWT-->>UserService: Access Token (15min) + Refresh Token (7days)
    UserService-->>Gateway: 返回Token
    Gateway-->>Frontend: 返回Token
    Frontend->>Frontend: 存储Token (LocalStorage)
    
    Frontend->>Gateway: 请求资源 (Header: Authorization: Bearer {token})
    Gateway->>Gateway: 验证Token签名
    Gateway->>Gateway: 检查Token过期
    Gateway->>Service: 转发请求 (附带用户信息)
```

**JWT Payload结构**：

```json
{
  "sub": "user_id_uuid",
  "tenant_id": "tenant_uuid",
  "email": "user@example.com",
  "role": "user",
  "permissions": ["strategy:read", "strategy:write", "order:create"],
  "iat": 1703001234,
  "exp": 1703002134
}
```

#### 7.1.2 RBAC权限模型

**角色定义**：

```yaml
roles:
  - name: admin
    description: 系统管理员
    permissions:
      - "*:*"  # 所有权限
  
  - name: trader
    description: 交易员
    permissions:
      - "strategy:read"
      - "strategy:write"
      - "strategy:execute"
      - "order:create"
      - "order:cancel"
      - "position:read"
      - "market:read"
  
  - name: analyst
    description: 分析师
    permissions:
      - "strategy:read"
      - "backtest:create"
      - "backtest:read"
      - "market:read"
      - "report:read"
  
  - name: viewer
    description: 观察者
    permissions:
      - "strategy:read"
      - "position:read"
      - "market:read"
      - "report:read"
```

**权限检查实现**：

```java
@Service
public class PermissionService {
    
    public boolean hasPermission(String userId, String resource, String action) {
        // 1. 获取用户角色
        List<Role> roles = userRepository.findRolesByUserId(userId);
        
        // 2. 检查权限
        String requiredPermission = resource + ":" + action;
        
        for (Role role : roles) {
            for (String permission : role.getPermissions()) {
                // 通配符匹配
                if (permission.equals("*:*") ||
                    permission.equals(resource + ":*") ||
                    permission.equals(requiredPermission)) {
                    return true;
                }
            }
        }
        
        return false;
    }
}

// 使用注解进行权限检查
@RestController
@RequestMapping("/api/v1/strategies")
public class StrategyController {
    
    @GetMapping("/{id}")
    @RequiresPermission("strategy:read")
    public ResponseEntity<Strategy> getStrategy(@PathVariable UUID id) {
        // ...
    }
    
    @PostMapping
    @RequiresPermission("strategy:write")
    public ResponseEntity<Strategy> createStrategy(@RequestBody StrategyRequest request) {
        // ...
    }
}
```

### 7.2 多租户隔离

#### 7.2.1 四层隔离机制

**第1层：PostgreSQL RLS（数据库层）**：

```sql
-- 已在5.1节详述
ALTER TABLE orders ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON orders
USING (tenant_id = current_setting('app.tenant_id')::uuid);
```

**第2层：Redis Key Prefix（缓存层）**：

```java
@Component
public class TenantAwareRedisTemplate {
    
    private final RedisTemplate<String, Object> redisTemplate;
    
    private String buildKey(String key) {
        UUID tenantId = TenantContext.getCurrentTenant();
        return String.format("tenant:%s:%s", tenantId, key);
    }
    
    public void set(String key, Object value) {
        redisTemplate.opsForValue().set(buildKey(key), value);
    }
    
    public Object get(String key) {
        return redisTemplate.opsForValue().get(buildKey(key));
    }
}
```

**第3层：Kafka Topic Partition（消息层）**：

```java
// 生产者：使用tenant_id作为Key确保分区隔离
ProducerRecord<String, OrderEvent> record = new ProducerRecord<>(
    "order.event.created",
    tenantId.toString(),  // Key
    orderEvent
);

// 消费者：过滤消息
@KafkaListener(topics = "order.event.created")
public void handleOrderEvent(ConsumerRecord<String, OrderEvent> record) {
    UUID currentTenant = TenantContext.getCurrentTenant();
    UUID messageTenant = UUID.fromString(record.key());
    
    // 仅处理当前租户的消息
    if (!currentTenant.equals(messageTenant)) {
        return;
    }
    
    // 处理消息...
}
```

**第4层：应用层Tenant Context（代码层）**：

```java
@Component
@WebFilter("/*")
public class TenantFilter implements Filter {
    
    @Override
    public void doFilter(ServletRequest request, ServletResponse response, FilterChain chain)
            throws IOException, ServletException {
        
        HttpServletRequest httpRequest = (HttpServletRequest) request;
        
        // 从JWT中提取tenant_id
        String token = extractToken(httpRequest);
        if (token != null) {
            Claims claims = JwtUtils.parseToken(token);
            UUID tenantId = UUID.fromString(claims.get("tenant_id", String.class));
            
            // 设置租户上下文
            TenantContext.setCurrentTenant(tenantId);
        }
        
        try {
            chain.doFilter(request, response);
        } finally {
            // 清除租户上下文
            TenantContext.clear();
        }
    }
}
```

### 7.3 数据加密

#### 7.3.1 传输加密（TLS 1.3）

**Nginx TLS配置**：

```nginx
server {
    listen 443 ssl http2;
    server_name api.hermesflow.com;
    
    # TLS证书
    ssl_certificate /etc/nginx/certs/fullchain.pem;
    ssl_certificate_key /etc/nginx/certs/privkey.pem;
    
    # TLS协议版本
    ssl_protocols TLSv1.3;
    ssl_prefer_server_ciphers on;
    ssl_ciphers 'TLS_AES_128_GCM_SHA256:TLS_AES_256_GCM_SHA384:TLS_CHACHA20_POLY1305_SHA256';
    
    # HSTS
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    
    # OCSP Stapling
    ssl_stapling on;
    ssl_stapling_verify on;
    
    location / {
        proxy_pass http://gateway:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

#### 7.3.2 静态数据加密

**PostgreSQL透明数据加密（TDE）**：

```sql
-- 在Azure PostgreSQL中启用
ALTER DATABASE hermesflow SET azure.enable_temp_tablespaces_on_local_ssd TO on;

-- 列级加密（敏感字段）
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- 加密API密钥
INSERT INTO api_keys (user_id, key_name, encrypted_key)
VALUES (
    'user_uuid',
    'Binance API',
    pgp_sym_encrypt('actual_api_key', 'encryption_password')
);

-- 解密
SELECT pgp_sym_decrypt(encrypted_key::bytea, 'encryption_password')
FROM api_keys
WHERE user_id = 'user_uuid';
```

**应用层加密（Java）**：

```java
@Service
public class EncryptionService {
    
    private static final String ALGORITHM = "AES/GCM/NoPadding";
    private static final int GCM_TAG_LENGTH = 128;
    
    @Value("${encryption.secret-key}")
    private String secretKeyString;
    
    private SecretKey secretKey;
    
    @PostConstruct
    public void init() {
        byte[] decodedKey = Base64.getDecoder().decode(secretKeyString);
        this.secretKey = new SecretKeySpec(decodedKey, 0, decodedKey.length, "AES");
    }
    
    public String encrypt(String plaintext) throws Exception {
        Cipher cipher = Cipher.getInstance(ALGORITHM);
        byte[] iv = new byte[12];  // 96-bit IV for GCM
        SecureRandom.getInstanceStrong().nextBytes(iv);
        GCMParameterSpec parameterSpec = new GCMParameterSpec(GCM_TAG_LENGTH, iv);
        
        cipher.init(Cipher.ENCRYPT_MODE, secretKey, parameterSpec);
        byte[] ciphertext = cipher.doFinal(plaintext.getBytes(StandardCharsets.UTF_8));
        
        // 返回 IV + ciphertext (Base64编码)
        byte[] combined = new byte[iv.length + ciphertext.length];
        System.arraycopy(iv, 0, combined, 0, iv.length);
        System.arraycopy(ciphertext, 0, combined, iv.length, ciphertext.length);
        
        return Base64.getEncoder().encodeToString(combined);
    }
    
    public String decrypt(String encrypted) throws Exception {
        byte[] decoded = Base64.getDecoder().decode(encrypted);
        
        // 提取IV和ciphertext
        byte[] iv = Arrays.copyOfRange(decoded, 0, 12);
        byte[] ciphertext = Arrays.copyOfRange(decoded, 12, decoded.length);
        
        Cipher cipher = Cipher.getInstance(ALGORITHM);
        GCMParameterSpec parameterSpec = new GCMParameterSpec(GCM_TAG_LENGTH, iv);
        cipher.init(Cipher.DECRYPT_MODE, secretKey, parameterSpec);
        
        byte[] plaintext = cipher.doFinal(ciphertext);
        return new String(plaintext, StandardCharsets.UTF_8);
    }
}
```

### 7.4 API安全

#### 7.4.1 Rate Limiting（令牌桶算法）

```java
@Configuration
public class RateLimitConfig {
    
    @Bean
    public RateLimiter rateLimiter() {
        return RateLimiter.create(100.0);  // 100 requests/second
    }
}

@RestControllerAdvice
public class RateLimitInterceptor implements HandlerInterceptor {
    
    private final RateLimiter rateLimiter;
    private final RedisTemplate<String, Integer> redisTemplate;
    
    @Override
    public boolean preHandle(HttpServletRequest request, HttpServletResponse response, Object handler)
            throws Exception {
        
        String userId = getCurrentUserId(request);
        String key = "rate_limit:" + userId;
        
        // 使用Redis实现分布式限流
        Integer count = redisTemplate.opsForValue().get(key);
        if (count == null) {
            redisTemplate.opsForValue().set(key, 1, 1, TimeUnit.MINUTES);
        } else if (count >= 1000) {  // 每分钟1000次
            response.setStatus(429);  // Too Many Requests
            response.getWriter().write("{\"error\": \"Rate limit exceeded\"}");
            return false;
        } else {
            redisTemplate.opsForValue().increment(key);
        }
        
        return true;
    }
}
```

#### 7.4.2 CORS配置

```java
@Configuration
public class CorsConfig implements WebMvcConfigurer {
    
    @Override
    public void addCorsMappings(CorsRegistry registry) {
        registry.addMapping("/api/**")
            .allowedOrigins("https://app.hermesflow.com")
            .allowedMethods("GET", "POST", "PUT", "DELETE", "OPTIONS")
            .allowedHeaders("*")
            .exposedHeaders("Authorization")
            .allowCredentials(true)
            .maxAge(3600);
    }
}
```

#### 7.4.3 Content Security Policy

```java
@Configuration
@EnableWebSecurity
public class SecurityConfig extends WebSecurityConfigurerAdapter {
    
    @Override
    protected void configure(HttpSecurity http) throws Exception {
        http
            .headers()
                .contentSecurityPolicy("default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self' wss://api.hermesflow.com")
                .and()
            .xssProtection()
                .and()
            .frameOptions()
                .deny();
    }
}
```

---

## 8. 性能优化策略

### 8.1 数据层优化（Rust）

#### 8.1.1 零拷贝序列化

```rust
use bytes::{Bytes, BytesMut};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct MarketData {
    symbol: String,
    price: f64,
    volume: f64,
    timestamp: i64,
}

// 零拷贝序列化
pub fn serialize_market_data(data: &MarketData) -> Bytes {
    let json = serde_json::to_vec(data).unwrap();
    Bytes::from(json)
}

// 直接从Bytes反序列化，避免复制
pub fn deserialize_market_data(bytes: &Bytes) -> Result<MarketData, serde_json::Error> {
    serde_json::from_slice(bytes.as_ref())
}
```

#### 8.1.2 连接池复用

```rust
use r2d2::{Pool, PooledConnection};
use r2d2_redis::RedisConnectionManager;

pub struct StorageManager {
    redis_pool: Pool<RedisConnectionManager>,
}

impl StorageManager {
    pub fn new(redis_url: &str) -> Result<Self> {
        let manager = RedisConnectionManager::new(redis_url)?;
        let pool = Pool::builder()
            .max_size(50)  // 最大50个连接
            .min_idle(Some(10))  // 最少保持10个空闲连接
            .connection_timeout(Duration::from_secs(5))
            .build(manager)?;
        
        Ok(Self { redis_pool: pool })
    }
    
    pub fn get_redis_conn(&self) -> Result<PooledConnection<RedisConnectionManager>> {
        self.redis_pool.get()
            .map_err(|e| anyhow::anyhow!("Failed to get Redis connection: {}", e))
    }
}
```

#### 8.1.3 批量写入ClickHouse

```rust
use clickhouse::Client;
use tokio::sync::mpsc;

pub struct ClickHouseBatchWriter {
    client: Client,
    buffer: Vec<MarketData>,
    batch_size: usize,
}

impl ClickHouseBatchWriter {
    pub fn new(client: Client, batch_size: usize) -> Self {
        Self {
            client,
            buffer: Vec::with_capacity(batch_size),
            batch_size,
        }
    }
    
    pub async fn write(&mut self, data: MarketData) -> Result<()> {
        self.buffer.push(data);
        
        if self.buffer.len() >= self.batch_size {
            self.flush().await?;
        }
        
        Ok(())
    }
    
    pub async fn flush(&mut self) -> Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        
        let mut insert = self.client.insert("market_data")?;
        
        for data in self.buffer.drain(..) {
            insert.write(&data).await?;
        }
        
        insert.end().await?;
        
        Ok(())
    }
}
```

### 8.2 策略引擎优化（Python）

#### 8.2.1 Numba JIT编译

```python
import numba
import numpy as np

# 优化前（纯Python）
def calculate_rsi_python(prices, period=14):
    deltas = np.diff(prices)
    gains = np.where(deltas > 0, deltas, 0)
    losses = np.where(deltas < 0, -deltas, 0)
    
    avg_gain = np.mean(gains[:period])
    avg_loss = np.mean(losses[:period])
    
    # ... 计算RSI

# 优化后（Numba加速，性能提升10-100倍）
@numba.jit(nopython=True)
def calculate_rsi_numba(prices, period=14):
    deltas = np.diff(prices)
    seed = deltas[:period+1]
    up = seed[seed >= 0].sum() / period
    down = -seed[seed < 0].sum() / period
    rs = up / down
    rsi = np.zeros_like(prices)
    rsi[:period] = 100. - 100. / (1. + rs)
    
    for i in range(period, len(prices)):
        delta = deltas[i - 1]
        if delta > 0:
            upval = delta
            downval = 0.
        else:
            upval = 0.
            downval = -delta
        
        up = (up * (period - 1) + upval) / period
        down = (down * (period - 1) + downval) / period
        rs = up / down
        rsi[i] = 100. - 100. / (1. + rs)
    
    return rsi

# 性能对比
import timeit

prices = np.random.randn(10000).cumsum()

# Python版本: ~50ms
python_time = timeit.timeit(lambda: calculate_rsi_python(prices), number=100) / 100

# Numba版本: ~0.5ms（首次编译后）
numba_time = timeit.timeit(lambda: calculate_rsi_numba(prices), number=100) / 100

print(f"加速比: {python_time / numba_time:.2f}x")
```

#### 8.2.2 Pandas向量化操作

```python
import pandas as pd

# 不推荐：iterrows（慢）
def calculate_returns_bad(df):
    returns = []
    for i, row in df.iterrows():
        if i == 0:
            returns.append(0)
        else:
            ret = (row['close'] - df.iloc[i-1]['close']) / df.iloc[i-1]['close']
            returns.append(ret)
    df['returns'] = returns
    return df

# 推荐：向量化操作（快100倍）
def calculate_returns_good(df):
    df['returns'] = df['close'].pct_change()
    return df

# 推荐：apply with Numba（更快）
@numba.jit(nopython=True)
def _calculate_returns_numba(prices):
    returns = np.zeros_like(prices)
    for i in range(1, len(prices)):
        returns[i] = (prices[i] - prices[i-1]) / prices[i-1]
    return returns

def calculate_returns_best(df):
    df['returns'] = _calculate_returns_numba(df['close'].values)
    return df
```

#### 8.2.3 多进程并行回测

```python
from multiprocessing import Pool, cpu_count
from typing import List, Dict

def backtest_single_params(args):
    """单个参数组合的回测"""
    params, data = args
    strategy = Strategy(**params)
    result = backtest_engine.run(strategy, data)
    return {'params': params, 'result': result}

def parallel_backtest(
    param_grid: List[Dict],
    data: pd.DataFrame,
    n_jobs: int = -1
) -> List[Dict]:
    """并行回测多个参数组合"""
    if n_jobs == -1:
        n_jobs = cpu_count()
    
    # 准备参数
    args_list = [(params, data) for params in param_grid]
    
    # 并行执行
    with Pool(n_jobs) as pool:
        results = pool.map(backtest_single_params, args_list)
    
    return results

# 使用示例
param_grid = [
    {'fast_period': 10, 'slow_period': 20},
    {'fast_period': 10, 'slow_period': 30},
    {'fast_period': 15, 'slow_period': 30},
    # ... 100个参数组合
]

results = parallel_backtest(param_grid, market_data, n_jobs=8)

# 找到最佳参数
best_result = max(results, key=lambda x: x['result']['sharpe_ratio'])
print(f"Best params: {best_result['params']}")
print(f"Sharpe ratio: {best_result['result']['sharpe_ratio']}")
```

### 8.3 数据库优化

#### 8.3.1 PostgreSQL索引策略

```sql
-- 1. 分析查询性能
EXPLAIN ANALYZE
SELECT * FROM orders
WHERE tenant_id = '123e4567-e89b-12d3-a456-426614174000'
AND status IN ('pending', 'partially_filled')
AND created_at > NOW() - INTERVAL '1 day'
ORDER BY created_at DESC
LIMIT 100;

-- 2. 创建复合索引（覆盖查询热路径）
CREATE INDEX idx_orders_tenant_status_created
ON orders(tenant_id, status, created_at DESC)
WHERE status IN ('pending', 'partially_filled');

-- 3. 定期分析表统计信息
ANALYZE orders;

-- 4. 清理死行（提升查询速度）
VACUUM ANALYZE orders;
```

#### 8.3.2 ClickHouse查询优化

```sql
-- 1. 使用PREWHERE（先过滤再读取列）
SELECT symbol, price, volume
FROM market_data
PREWHERE symbol = 'BTCUSDT'  -- 先过滤（只读取需要的块）
WHERE timestamp > now() - INTERVAL 1 HOUR;  -- 再过滤

-- 2. 利用物化视图预聚合
CREATE MATERIALIZED VIEW hourly_ohlc
ENGINE = SummingMergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (symbol, timestamp)
AS SELECT
    toStartOfHour(timestamp) AS timestamp,
    symbol,
    argMin(price, timestamp) AS open,
    max(price) AS high,
    min(price) AS low,
    argMax(price, timestamp) AS close,
    sum(volume) AS volume
FROM market_data
GROUP BY timestamp, symbol;

-- 查询时直接使用物化视图（快1000倍）
SELECT * FROM hourly_ohlc WHERE symbol = 'BTCUSDT';
```

### 8.4 缓存策略

#### 8.4.1 多级缓存架构

```
┌─────────────────────────────────────────┐
│        Request Flow                     │
└─────────────────────────────────────────┘
            │
            ▼
┌───────────────────────┐
│   L1: Application     │  Caffeine (本地内存)
│   Memory Cache        │  TTL: 5-10秒
│   Hit Rate: 70%       │  Size: 10K entries
└───────────────────────┘
            │ Miss
            ▼
┌───────────────────────┐
│   L2: Redis Cache     │  Redis (分布式)
│   Hit Rate: 25%       │  TTL: 1-5分钟
└───────────────────────┘
            │ Miss
            ▼
┌───────────────────────┐
│   L3: Database        │  PostgreSQL/ClickHouse
│   Hit Rate: 5%        │
└───────────────────────┘
```

**实现代码**：

```java
@Service
public class MultiLevelCacheService {
    
    // L1缓存：本地内存（Caffeine）
    private final Cache<String, Object> l1Cache = Caffeine.newBuilder()
        .maximumSize(10_000)
        .expireAfterWrite(10, TimeUnit.SECONDS)
        .recordStats()
        .build();
    
    // L2缓存：Redis
    private final RedisTemplate<String, Object> redisTemplate;
    
    // L3数据源：数据库
    private final OrderRepository orderRepository;
    
    public Order getOrder(UUID orderId) {
        String key = "order:" + orderId;
        
        // 1. 尝试L1缓存
        Order order = (Order) l1Cache.getIfPresent(key);
        if (order != null) {
            return order;
        }
        
        // 2. 尝试L2缓存（Redis）
        order = (Order) redisTemplate.opsForValue().get(key);
        if (order != null) {
            l1Cache.put(key, order);  // 回填L1
            return order;
        }
        
        // 3. 查询数据库
        order = orderRepository.findById(orderId).orElse(null);
        if (order != null) {
            // 回填L2和L1
            redisTemplate.opsForValue().set(key, order, 5, TimeUnit.MINUTES);
            l1Cache.put(key, order);
        }
        
        return order;
    }
}
```

---

## 9. 监控与运维方案

### 9.1 日志系统

#### 9.1.1 结构化日志（Rust）

```rust
use tracing::{info, warn, error, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[instrument(skip(data))]
async fn process_market_data(symbol: &str, data: &MarketData) -> Result<()> {
    let start = Instant::now();
    
    // 处理数据...
    
    let latency_us = start.elapsed().as_micros();
    
    info!(
        symbol = %symbol,
        price = %data.price,
        volume = %data.volume,
        latency_us = latency_us,
        "Market data processed"
    );
    
    if latency_us > 1000 {
        warn!(
            symbol = %symbol,
            latency_us = latency_us,
            "High latency detected"
        );
    }
    
    Ok(())
}

// 初始化日志
fn init_logging() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into())
        ))
        .with(tracing_subscriber::fmt::layer().json())  // JSON格式
        .init();
}
```

**日志输出示例**：

```json
{
  "timestamp": "2024-12-20T10:30:15.123456Z",
  "level": "INFO",
  "target": "data_engine::processor",
  "fields": {
    "symbol": "BTCUSDT",
    "price": 67850.23,
    "volume": 1.5,
    "latency_us": 850,
    "message": "Market data processed"
  },
  "span": {
    "name": "process_market_data"
  }
}
```

#### 9.1.2 日志收集架构

```
Application Logs
    │
    ▼
┌─────────────────────┐
│   Filebeat          │  采集日志文件
└─────────────────────┘
    │
    ▼
┌─────────────────────┐
│   Logstash          │  解析、过滤、转换
└─────────────────────┘
    │
    ▼
┌─────────────────────┐
│   Elasticsearch     │  存储、索引
└─────────────────────┘
    │
    ▼
┌─────────────────────┐
│   Kibana            │  可视化、查询
└─────────────────────┘
```

### 9.2 监控指标（Prometheus）

#### 9.2.1 Rust服务暴露指标

```rust
use prometheus::{Counter, Histogram, Registry, TextEncoder, Encoder};
use actix_web::{web, HttpResponse};

pub struct Metrics {
    pub messages_processed: Counter,
    pub websocket_latency: Histogram,
    pub clickhouse_writes: Counter,
}

impl Metrics {
    pub fn new() -> Result<Self> {
        let messages_processed = Counter::new(
            "data_service_messages_processed_total",
            "Total number of market data messages processed"
        )?;
        
        let websocket_latency = Histogram::with_opts(
            histogram_opts!(
                "data_service_websocket_latency_milliseconds",
                "WebSocket message latency in milliseconds",
                vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0]
            )
        )?;
        
        let clickhouse_writes = Counter::new(
            "data_service_clickhouse_writes_total",
            "Total number of ClickHouse writes"
        )?;
        
        // 注册指标
        let registry = Registry::new();
        registry.register(Box::new(messages_processed.clone()))?;
        registry.register(Box::new(websocket_latency.clone()))?;
        registry.register(Box::new(clickhouse_writes.clone()))?;
        
        Ok(Self {
            messages_processed,
            websocket_latency,
            clickhouse_writes,
        })
    }
}

// 暴露Prometheus指标端点
async fn metrics_handler(metrics: web::Data<Metrics>) -> HttpResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(buffer)
}
```

#### 9.2.2 Prometheus配置

**prometheus.yml**：

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  # 数据服务
  - job_name: 'data-service'
    static_configs:
      - targets: ['data-service:8081']
  
  # 策略引擎
  - job_name: 'strategy-engine'
    static_configs:
      - targets: ['strategy-engine:8082']
  
  # 交易服务
  - job_name: 'trading-service'
    static_configs:
      - targets: ['trading-service:8083']
  
  # PostgreSQL
  - job_name: 'postgresql'
    static_configs:
      - targets: ['postgres-exporter:9187']
  
  # Redis
  - job_name: 'redis'
    static_configs:
      - targets: ['redis-exporter:9121']
  
  # Kubernetes
  - job_name: 'kubernetes-pods'
    kubernetes_sd_configs:
      - role: pod
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scrape]
        action: keep
        regex: true
```

### 9.3 告警规则

**alert-rules.yml**：

```yaml
groups:
  - name: data_service_alerts
    interval: 30s
    rules:
      # WebSocket延迟过高
      - alert: HighWebSocketLatency
        expr: histogram_quantile(0.95, data_service_websocket_latency_milliseconds) > 10
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "WebSocket延迟过高"
          description: "95分位延迟超过10ms，当前值: {{ $value }}ms"
      
      # ClickHouse写入错误
      - alert: ClickHouseWriteErrors
        expr: rate(data_service_clickhouse_write_errors_total[5m]) > 0.01
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "ClickHouse写入错误率过高"
          description: "错误率: {{ $value | humanizePercentage }}"
  
  - name: strategy_engine_alerts
    interval: 30s
    rules:
      # 回测任务堆积
      - alert: BacktestQueueBacklog
        expr: strategy_engine_backtest_queue_size > 100
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "回测任务队列堆积"
          description: "当前队列大小: {{ $value }}"
  
  - name: trading_service_alerts
    interval: 30s
    rules:
      # 订单执行失败率高
      - alert: HighOrderFailureRate
        expr: rate(trading_service_order_failures_total[5m]) / rate(trading_service_orders_total[5m]) > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "订单失败率过高"
          description: "失败率: {{ $value | humanizePercentage }}"
  
  - name: system_alerts
    interval: 30s
    rules:
      # CPU使用率过高
      - alert: HighCPUUsage
        expr: 100 - (avg by(instance) (rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100) > 80
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "CPU使用率过高"
          description: "实例 {{ $labels.instance }} CPU使用率: {{ $value }}%"
      
      # 内存使用率过高
      - alert: HighMemoryUsage
        expr: (node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes) / node_memory_MemTotal_bytes * 100 > 85
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "内存使用率过高"
          description: "实例 {{ $labels.instance }} 内存使用率: {{ $value }}%"
```

### 9.4 Grafana仪表盘

#### 9.4.1 系统概览Dashboard

**JSON配置**（部分）：

```json
{
  "dashboard": {
    "title": "HermesFlow System Overview",
    "panels": [
      {
        "title": "CPU Usage",
        "targets": [
          {
            "expr": "100 - (avg(rate(node_cpu_seconds_total{mode=\"idle\"}[5m])) * 100)"
          }
        ],
        "type": "graph"
      },
      {
        "title": "Memory Usage",
        "targets": [
          {
            "expr": "(node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes) / node_memory_MemTotal_bytes * 100"
          }
        ],
        "type": "gauge"
      },
      {
        "title": "Request Rate",
        "targets": [
          {
            "expr": "sum(rate(http_requests_total[5m]))"
          }
        ],
        "type": "graph"
      }
    ]
  }
}
```

#### 9.4.2 数据服务Dashboard

**关键指标**：
- WebSocket连接数：`data_service_websocket_connections`
- 消息处理吞吐：`rate(data_service_messages_processed_total[1m])`
- 消息延迟（P50/P95/P99）：`histogram_quantile(0.95, data_service_websocket_latency_milliseconds)`
- ClickHouse写入速率：`rate(data_service_clickhouse_writes_total[1m])`

---

## 10. 技术债务与改进计划

### 10.1 已知技术债务

#### 10.1.1 技术债务清单

| 债务项 | 影响范围 | 优先级 | 预计工作量 | 计划解决时间 |
|-------|----------|--------|-----------|-------------|
| 策略引擎缺少完整单元测试 | 策略引擎 | P1 | 2周 | MVP后1个月 |
| ClickHouse自动分区管理 | 数据服务 | P2 | 1周 | V2阶段 |
| 前端缺少SSR | 前端 | P3 | 3周 | V3阶段 |
| API文档不完整 | 全系统 | P1 | 1周 | MVP后2周 |
| 缺少端到端测试 | 全系统 | P2 | 2周 | MVP后1个月 |
| 日志缺少链路追踪 | 全系统 | P2 | 1周 | V2阶段 |
| Redis单点故障风险 | 数据服务 | P1 | 1周 | MVP后1个月 |
| 缺少灾难恢复方案 | 全系统 | P2 | 2周 | V2阶段 |

#### 10.1.2 偿还计划

**Phase 1（MVP后1个月）**：
- 补充策略引擎单元测试（覆盖率提升至85%）
- 完善API文档（OpenAPI 3.0规范）
- Redis主从复制 + Sentinel高可用

**Phase 2（V2阶段）**：
- 实现ClickHouse自动分区管理
- 添加分布式链路追踪（Jaeger）
- 编写端到端测试套件
- 制定灾难恢复计划

**Phase 3（V3阶段）**：
- 前端SSR支持（Next.js迁移）
- 完整性能测试（压测报告）
- 安全审计（第三方渗透测试）

### 10.2 V2改进计划（6个月）

#### 10.2.1 机器学习集成（10周）

**目标**：
- 建立ML Pipeline框架
- 实现特征工程工具
- 集成AutoML（自动模型选择）
- 在线预测服务

**技术栈**：
- MLflow（实验追踪）
- Scikit-learn / XGBoost（模型）
- ONNX Runtime（推理加速）

**工作分解**：
1. Week 1-2: ML Pipeline框架搭建
2. Week 3-4: 特征工程库（100+特征）
3. Week 5-6: 模型训练与评估
4. Week 7-8: 在线预测服务
5. Week 9-10: 性能优化与测试

#### 10.2.2 组合管理系统（9周）

**目标**：
- 多策略并行执行
- 动态资金分配（Markowitz优化）
- 风险预算管理（VaR/CVaR）
- 组合再平衡

**工作分解**：
1. Week 1-2: 组合管理框架
2. Week 3-4: Markowitz优化器
3. Week 5-6: VaR/CVaR风险模型
4. Week 7-8: 动态再平衡逻辑
5. Week 9: 集成测试与文档

#### 10.2.3 因子库扩展（4周）

**目标**：从100个因子扩展至200+个因子

**新增因子分类**：
- 高级技术指标（30个）：Ichimoku、Keltner、ADX等
- 时间序列因子（20个）：自相关、协整等
- 市场微观结构因子（20个）：订单簿失衡、成交量分布等
- 宏观因子（15个）：利率曲线、CPI、GDP等
- 另类数据因子（15个）：社交媒体情绪、新闻事件等

#### 10.2.4 实盘交易上线（8周）

**目标**：
- 实盘交易环境搭建
- 风控系统强化
- 监控告警完善
- 灰度发布流程

**里程碑**：
1. Week 1-2: 实盘环境准备
2. Week 3-4: 风控规则强化
3. Week 5-6: 小规模灰度（1-2个策略）
4. Week 7-8: 扩大规模 + 监控优化

### 10.3 V3改进计划（12个月）

#### 10.3.1 社区策略市场

**功能**：
- 策略分享与交易
- 策略评分与评论
- 策略订阅（付费/免费）
- 收益分成机制

**商业模式**：
- 平台抽成20%
- 策略作者80%
- 月度交易量奖励

#### 10.3.2 移动端APP（React Native）

**功能**：
- 实时监控（账户、持仓、收益）
- 风险告警推送
- 简单策略开启/停止
- 行情查看

**技术栈**：
- React Native
- Redux Toolkit
- React Navigation
- Push Notifications

#### 10.3.3 高级可视化（3D图表）

**功能**：
- 3D参数优化空间可视化
- 多维度因子分析
- 实时市场深度热力图
- VR/AR交易界面（探索性）

**技术栈**：
- Three.js / WebGL
- D3.js
- Plotly

#### 10.3.4 智能策略推荐

**功能**：
- 基于用户画像的策略推荐
- 市场环境自动识别
- 策略自动调参建议
- 风险偏好匹配

**技术**：
- 协同过滤
- 内容推荐
- 强化学习

---

## 附录

### A. 参考文献

1. **系统架构**：
   - "Designing Data-Intensive Applications" by Martin Kleppmann
   - "Building Microservices" by Sam Newman
   - "C4 Model" by Simon Brown

2. **量化交易**：
   - "Advances in Financial Machine Learning" by Marcos López de Prado
   - "Quantitative Trading" by Ernest P. Chan
   - "Algorithmic Trading" by Jeffrey Bacidore

3. **技术文档**：
   - Rust Async Book: https://rust-lang.github.io/async-book/
   - Spring Boot Reference: https://docs.spring.io/spring-boot/docs/current/reference/html/
   - Kubernetes Documentation: https://kubernetes.io/docs/

### B. 术语表

| 术语 | 英文 | 解释 |
|------|------|------|
| Alpha因子 | Alpha Factor | 预测资产收益的特征变量 |
| 贝叶斯优化 | Bayesian Optimization | 基于贝叶斯推断的超参数优化方法 |
| Walk-Forward | Walk-Forward Analysis | 滚动窗口参数优化与验证 |
| RLS | Row-Level Security | PostgreSQL行级安全策略 |
| gRPC | gRPC Remote Procedure Call | Google开发的RPC框架 |
| Tokio | Tokio | Rust异步运行时 |
| Numba | Numba | Python JIT编译器 |
| HPA | Horizontal Pod Autoscaler | Kubernetes水平自动扩缩容 |

### C. 变更历史

| 版本 | 日期 | 作者 | 变更说明 |
|------|------|------|----------|
| v2.1.0 | 2024-12-20 | Architecture Team | 初始版本，完整系统架构设计 |

---

## 11. 持续集成与持续部署（CI/CD）架构

### 11.1 CI/CD 整体流程

#### 11.1.1 完整流水线

```
开发者提交代码
    ↓
GitHub Actions 触发
    ↓
解析模块标签 [module:xxx]
    ↓
构建源代码 (Maven/Cargo/npm)
    ↓
构建 Docker 镜像
    ↓
推送到 Azure Container Registry
    ↓
触发 GitOps 仓库更新
    ↓
更新 Helm values.yaml
    ↓
ArgoCD 自动同步
    ↓
部署到 Kubernetes 集群
```

#### 11.1.2 时序图

```mermaid
sequenceDiagram
    participant Dev as 开发者
    participant GitHub as GitHub Actions
    participant ACR as Azure Container Registry
    participant GitOps as HermesFlow-GitOps
    participant ArgoCD as ArgoCD
    participant K8s as Kubernetes Cluster
    
    Dev->>GitHub: git push (commit: [module:xxx] feat: ...)
    GitHub->>GitHub: 解析模块标签
    GitHub->>GitHub: 构建源代码
    GitHub->>GitHub: 构建 Docker 镜像
    GitHub->>ACR: 推送镜像 (tag: commit-sha)
    GitHub->>GitOps: 触发 workflow dispatch
    GitOps->>GitOps: 更新 values.yaml
    GitOps->>GitOps: git commit & push
    ArgoCD->>GitOps: 监听仓库变化
    ArgoCD->>ArgoCD: 检测到新版本
    ArgoCD->>K8s: 应用 Helm Charts
    K8s->>K8s: 滚动更新 Pod
```

### 11.2 GitHub Actions 工作流详解

#### 11.2.1 工作流配置

**文件**: `.github/workflows/module-cicd.yml`

**触发条件**：

```yaml
on:
  push:
    branches: [dev, main]  # dev环境和main环境
```

#### 11.2.2 核心步骤解析

**步骤1：解析模块标签**

```yaml
- name: Parse Module from Commit
  id: parse-module
  run: |
    COMMIT_MSG="${{ github.event.head_commit.message }}"
    
    # 正则匹配 [module:xxx]
    if [[ "$COMMIT_MSG" =~ \[module:([^\]]+)\] ]]; then
      MODULE="${BASH_REMATCH[1]}"
      echo "✅ Found module: $MODULE"
      echo "module=$MODULE" >> $GITHUB_OUTPUT
      echo "has_module=true" >> $GITHUB_OUTPUT
    else
      echo "ℹ️ No module tag found, skipping build"
      echo "has_module=false" >> $GITHUB_OUTPUT
    fi
```

**支持的模块标签**：

| 模块标签 | 技术栈 | 说明 |
|---------|--------|------|
| `[module:data-engine]` | Rust | 高性能数据采集服务 |
| `[module:strategy-engine]` | Python | 策略开发与回测引擎 |
| `[module:trading-engine]` | Java | 订单管理与交易执行 |
| `[module:user-management]` | Java | 用户认证与管理 |
| `[module:risk-engine]` | Java | 风险控制与监控 |
| `[module:gateway]` | Java | API网关 |
| `[module:frontend]` | React | 用户界面 |

**步骤2：环境变量配置**

```yaml
- name: Set Environment Variables
  if: steps.parse-module.outputs.has_module == 'true'
  run: |
    ENVIRONMENT=${{ github.ref_name }}
    
    # dev分支 → dev环境
    if [[ "$ENVIRONMENT" == "dev" ]]; then
      echo "AZURE_REGISTRY=${{ secrets.DEV_AZURE_REGISTRY }}" >> $GITHUB_ENV
      echo "AZURE_CLIENT_ID=${{ secrets.DEV_AZURE_CLIENT_ID }}" >> $GITHUB_ENV
      echo "AZURE_CLIENT_SECRET=${{ secrets.DEV_AZURE_CLIENT_SECRET }}" >> $GITHUB_ENV
    
    # main分支 → main环境
    elif [[ "$ENVIRONMENT" == "main" ]]; then
      echo "AZURE_REGISTRY=${{ secrets.MAIN_AZURE_REGISTRY }}" >> $GITHUB_ENV
      echo "AZURE_CLIENT_ID=${{ secrets.MAIN_AZURE_CLIENT_ID }}" >> $GITHUB_ENV
      echo "AZURE_CLIENT_SECRET=${{ secrets.MAIN_AZURE_CLIENT_SECRET }}" >> $GITHUB_ENV
    else
      echo "❌ Unsupported environment: $ENVIRONMENT"
      exit 1
    fi
    
    echo "ENVIRONMENT=$ENVIRONMENT" >> $GITHUB_ENV
    echo "MODULE=${{ steps.parse-module.outputs.module }}" >> $GITHUB_ENV
```

**环境隔离策略**：

| 环境 | 分支 | Container Registry | 部署目标 |
|------|------|-------------------|---------|
| **dev** | `dev` | `hermesflow-dev-acr.azurecr.io` | AKS dev namespace |
| **main** | `main` | `hermesflow-prod-acr.azurecr.io` | AKS prod namespace |

**步骤3：调用统一构建脚本**

```yaml
- name: Build and Push Module
  if: steps.parse-module.outputs.has_module == 'true'
  run: |
    chmod +x ./scripts/build-module.sh
    ./scripts/build-module.sh
```

**步骤4：触发GitOps更新**

```yaml
- name: Update GitOps Repository
  if: steps.parse-module.outputs.has_module == 'true'
  run: |
    curl -X POST \
      -H "Authorization: token ${{ secrets.GITOPS_TOKEN }}" \
      -H "Content-Type: application/json" \
      -d '{
        "ref": "main",
        "inputs": {
          "module": "${{ env.MODULE }}",
          "tag": "${{ github.sha }}",
          "environment": "${{ env.ENVIRONMENT }}",
          "registry": "${{ env.AZURE_REGISTRY }}"
        }
      }' \
      https://api.github.com/repos/tomxiao/HermesFlow-GitOps/actions/workflows/update-values.yml/dispatches
```

### 11.3 统一构建脚本详解

#### 11.3.1 脚本概览

**文件**: `scripts/build-module.sh`

**核心职责**：
1. 验证环境变量
2. 根据模块类型构建源代码
3. 构建Docker镜像（多标签）
4. 推送到Azure Container Registry

#### 11.3.2 详细实现

```bash
#!/bin/bash
# HermesFlow 统一模块构建脚本

# 从环境变量获取配置
MODULE="$MODULE"
TAG="${GITHUB_SHA:-latest}"
ENVIRONMENT="$ENVIRONMENT"
REGISTRY="$AZURE_REGISTRY"

echo "🏗️ HermesFlow Module Build Configuration:"
echo "  Module: $MODULE"
echo "  Tag: $TAG"
echo "  Environment: $ENVIRONMENT"
echo "  Registry: $REGISTRY"

# 1. 验证必要的环境变量
if [[ -z "$MODULE" || -z "$REGISTRY" || -z "$AZURE_CLIENT_ID" || -z "$AZURE_CLIENT_SECRET" ]]; then
    echo "❌ Missing required environment variables"
    exit 1
fi

# 2. 根据模块类型构建源代码
cd "modules/$MODULE"

case "$MODULE" in
    strategy-engine|risk-engine|user-management|api-gateway|trading-engine)
        echo "☕ Building Java module..."
        mvn clean package -DskipTests -B
        ;;
    data-engine)
        echo "🦀 Building Rust module..."
        cargo build --release
        ;;
    frontend)
        echo "📦 Building Node.js module..."
        npm ci && npm run build
        ;;
    *)
        echo "⚠️ Unknown module type: $MODULE"
        ;;
esac

cd ../../

# 3. 构建Docker镜像（多标签）
IMAGE_TAG="$REGISTRY/$MODULE:$TAG"
LATEST_TAG="$REGISTRY/$MODULE:$ENVIRONMENT-latest"

echo "🐳 Building Docker image..."
docker build \
    -t "$IMAGE_TAG" \
    -t "$LATEST_TAG" \
    -f "scripts/$MODULE/Dockerfile" \
    "modules/$MODULE"

# 4. 登录Azure Container Registry
echo "🔐 Logging into Azure Container Registry..."
echo "$AZURE_CLIENT_SECRET" | docker login "$REGISTRY" \
    -u "$AZURE_CLIENT_ID" --password-stdin

# 5. 推送镜像
echo "📤 Pushing images to registry..."
docker push "$IMAGE_TAG"
docker push "$LATEST_TAG"

echo "✅ Successfully built and pushed $MODULE"
```

#### 11.3.3 镜像标签策略

**多标签设计**：

```
hermesflow-dev-acr.azurecr.io/data-engine:abc123def456  ← Commit SHA（精确版本）
hermesflow-dev-acr.azurecr.io/data-engine:dev-latest    ← 环境Latest（滚动更新）
```

**优点**：
- **Commit SHA标签**：精确追踪，支持回滚
- **Latest标签**：快速引用最新版本
- **环境隔离**：dev-latest vs prod-latest

### 11.4 GitOps 仓库结构

#### 11.4.1 目录结构

**仓库**: `HermesFlow-GitOps`

```
HermesFlow-GitOps/
├── base-charts/                 # 基础Helm Chart模板
│   └── microservice/
│       ├── Chart.yaml           # Chart元数据
│       ├── values.yaml          # 默认配置
│       └── templates/
│           ├── _helpers.tpl     # 模板辅助函数
│           ├── deployment.yaml  # Deployment资源
│           ├── service.yaml     # Service资源
│           ├── hpa.yaml         # 自动扩缩容
│           └── ingress.yaml     # Ingress路由
│
├── apps/                        # 环境特定配置
│   ├── dev/                     # 开发环境
│   │   ├── data-engine/
│   │   │   ├── Chart.yaml       # 继承base-charts
│   │   │   └── values.yaml      # 覆盖配置
│   │   ├── strategy-engine/
│   │   ├── trading-engine/
│   │   ├── user-management/
│   │   ├── risk-engine/
│   │   ├── gateway/
│   │   └── frontend/
│   │
│   └── main/                    # 生产环境
│       ├── data-engine/
│       ├── strategy-engine/
│       └── ...
│
└── .github/workflows/
    └── update-values.yml        # 自动更新values.yaml
```

#### 11.4.2 Base Chart配置

**base-charts/microservice/values.yaml**：

```yaml
# 默认配置（可被环境特定配置覆盖）
replicaCount: 1

image:
  repository: ""
  tag: "latest"
  pullPolicy: IfNotPresent

service:
  type: ClusterIP
  port: 8080
  targetPort: 8080

resources:
  limits:
    cpu: 1000m
    memory: 1Gi
  requests:
    cpu: 500m
    memory: 512Mi

autoscaling:
  enabled: false
  minReplicas: 2
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70
  targetMemoryUtilizationPercentage: 80

probes:
  liveness:
    enabled: true
    path: /health
    initialDelaySeconds: 30
    periodSeconds: 10
  readiness:
    enabled: true
    path: /health
    initialDelaySeconds: 10
    periodSeconds: 5

env: []

configMap:
  enabled: false
  data: {}

secret:
  enabled: false
  data: {}
```

#### 11.4.3 环境特定配置

**apps/dev/data-engine/values.yaml**：

```yaml
# 继承base-charts，覆盖特定值
image:
  repository: hermesflow-dev-acr.azurecr.io/data-engine
  tag: "abc123def456"  # 自动更新为最新commit SHA

replicaCount: 1

service:
  port: 8081
  targetPort: 8081

resources:
  limits:
    cpu: 2000m
    memory: 2Gi
  requests:
    cpu: 1000m
    memory: 1Gi

autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 5
  targetCPUUtilizationPercentage: 70

env:
  - name: RUST_LOG
    value: "info"
  - name: DATABASE_URL
    valueFrom:
      secretKeyRef:
        name: postgres-secret
        key: connection-string
  - name: REDIS_URL
    value: "redis://redis-service:6379"
  - name: KAFKA_BROKERS
    value: "kafka-service:9092"
  - name: CLICKHOUSE_URL
    value: "http://clickhouse-service:8123"
```

**apps/main/data-engine/values.yaml**（生产环境）：

```yaml
# 生产环境配置（更严格的资源限制和高可用）
image:
  repository: hermesflow-prod-acr.azurecr.io/data-engine
  tag: "xyz789prod"

replicaCount: 3  # 生产环境3副本

resources:
  limits:
    cpu: 4000m
    memory: 4Gi
  requests:
    cpu: 2000m
    memory: 2Gi

autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10
  targetCPUUtilizationPercentage: 60  # 更保守的阈值
```

### 11.5 GitOps 自动更新流程

#### 11.5.1 更新工作流

**文件**: `HermesFlow-GitOps/.github/workflows/update-values.yml`

```yaml
name: Update Helm Values

on:
  workflow_dispatch:
    inputs:
      module:
        description: 'Module name'
        required: true
      tag:
        description: 'Image tag (commit SHA)'
        required: true
      environment:
        description: 'Environment (dev/main)'
        required: true
      registry:
        description: 'Container registry'
        required: true

jobs:
  update-values:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout GitOps Repo
        uses: actions/checkout@v4
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
      
      - name: Install yq
        run: |
          sudo wget -qO /usr/local/bin/yq https://github.com/mikefarah/yq/releases/latest/download/yq_linux_amd64
          sudo chmod +x /usr/local/bin/yq
      
      - name: Update values.yaml
        run: |
          VALUES_FILE="apps/${{ github.event.inputs.environment }}/${{ github.event.inputs.module }}/values.yaml"
          
          echo "📝 Updating $VALUES_FILE"
          echo "  Image Tag: ${{ github.event.inputs.tag }}"
          echo "  Registry: ${{ github.event.inputs.registry }}"
          
          # 使用yq更新image.tag和image.repository
          yq eval ".image.tag = \"${{ github.event.inputs.tag }}\"" -i "$VALUES_FILE"
          yq eval ".image.repository = \"${{ github.event.inputs.registry }}/${{ github.event.inputs.module }}\"" -i "$VALUES_FILE"
          
          echo "✅ Updated successfully"
          cat "$VALUES_FILE"
      
      - name: Commit and Push
        run: |
          git config user.name "GitHub Actions Bot"
          git config user.email "actions@github.com"
          git add .
          git commit -m "🚀 Update ${{ github.event.inputs.module }} to ${{ github.event.inputs.tag }} in ${{ github.event.inputs.environment }}"
          git push
      
      - name: Summary
        run: |
          echo "## 🎉 GitOps Update Summary" >> $GITHUB_STEP_SUMMARY
          echo "- **Module**: ${{ github.event.inputs.module }}" >> $GITHUB_STEP_SUMMARY
          echo "- **Environment**: ${{ github.event.inputs.environment }}" >> $GITHUB_STEP_SUMMARY
          echo "- **New Tag**: \`${{ github.event.inputs.tag }}\`" >> $GITHUB_STEP_SUMMARY
          echo "- **Registry**: ${{ github.event.inputs.registry }}" >> $GITHUB_STEP_SUMMARY
```

#### 11.5.2 更新示例

**更新前**（apps/dev/data-engine/values.yaml）：

```yaml
image:
  repository: hermesflow-dev-acr.azurecr.io/data-engine
  tag: "xyz789old"
```

**更新后**（自动）：

```yaml
image:
  repository: hermesflow-dev-acr.azurecr.io/data-engine
  tag: "abc123def456"  # 新的commit SHA
```

### 11.6 ArgoCD 配置与同步

#### 11.6.1 Application定义

**文件**: `argocd/applications/hermesflow-dev.yaml`

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: hermesflow-dev
  namespace: argocd
spec:
  project: default
  
  # 源：GitOps仓库
  source:
    repoURL: https://github.com/tomxiao/HermesFlow-GitOps.git
    targetRevision: HEAD
    path: apps/dev
  
  # 目标：Kubernetes集群
  destination:
    server: https://kubernetes.default.svc
    namespace: hermesflow-dev
  
  # 自动同步策略
  syncPolicy:
    automated:
      prune: true        # 自动删除不再需要的资源
      selfHeal: true     # 自动修复配置漂移
    syncOptions:
      - CreateNamespace=true
    retry:
      limit: 5
      backoff:
        duration: 5s
        factor: 2
        maxDuration: 3m
```

#### 11.6.2 同步流程

```
1. ArgoCD轮询GitOps仓库（每3分钟，可配置）
   ↓
2. 检测到values.yaml变化
   ↓
3. 渲染Helm Chart，生成Kubernetes资源清单
   ↓
4. 计算当前集群状态与期望状态的差异
   ↓
5. 应用差异（kubectl apply）
   ↓
6. Kubernetes执行滚动更新
   ↓
7. 等待新Pod就绪
   ↓
8. 终止旧Pod
   ↓
9. 同步完成，更新ArgoCD状态
```

#### 11.6.3 滚动更新策略

**base-charts/microservice/templates/deployment.yaml**：

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ include "microservice.fullname" . }}
spec:
  replicas: {{ .Values.replicaCount }}
  
  # 滚动更新策略
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1        # 最多额外创建1个Pod
      maxUnavailable: 0  # 保持所有旧Pod可用（零停机）
  
  selector:
    matchLabels:
      {{- include "microservice.selectorLabels" . | nindent 6 }}
  
  template:
    metadata:
      labels:
        {{- include "microservice.selectorLabels" . | nindent 8 }}
    spec:
      containers:
      - name: {{ .Chart.Name }}
        image: "{{ .Values.image.repository }}:{{ .Values.image.tag }}"
        imagePullPolicy: {{ .Values.image.pullPolicy }}
        ports:
        - name: http
          containerPort: {{ .Values.service.targetPort }}
        
        # 健康检查
        {{- if .Values.probes.liveness.enabled }}
        livenessProbe:
          httpGet:
            path: {{ .Values.probes.liveness.path }}
            port: http
          initialDelaySeconds: {{ .Values.probes.liveness.initialDelaySeconds }}
          periodSeconds: {{ .Values.probes.liveness.periodSeconds }}
        {{- end }}
        
        {{- if .Values.probes.readiness.enabled }}
        readinessProbe:
          httpGet:
            path: {{ .Values.probes.readiness.path }}
            port: http
          initialDelaySeconds: {{ .Values.probes.readiness.initialDelaySeconds }}
          periodSeconds: {{ .Values.probes.readiness.periodSeconds }}
        {{- end }}
        
        resources:
          {{- toYaml .Values.resources | nindent 12 }}
        
        env:
          {{- range .Values.env }}
          - name: {{ .name }}
            {{- if .value }}
            value: {{ .value | quote }}
            {{- else if .valueFrom }}
            valueFrom:
              {{- toYaml .valueFrom | nindent 14 }}
            {{- end }}
          {{- end }}
```

### 11.7 完整部署示例

#### 11.7.1 场景描述

开发者修复了数据服务的WebSocket连接泄漏问题。

#### 11.7.2 详细步骤

**步骤1：开发者提交代码**

```bash
# 本地开发
cd modules/data-engine
vim src/websocket/connection_pool.rs  # 修复Bug

# 本地测试
cargo test

# 提交代码
git add src/websocket/connection_pool.rs
git commit -m "[module:data-engine] fix: 修复WebSocket连接泄漏问题

- 在连接池中添加超时清理机制
- 修复连接未正确关闭的Bug
- 添加连接泄漏检测日志"

git push origin dev
```

**步骤2：GitHub Actions自动执行**（约5分钟）

```
[GitHub Actions - HermesFlow Repo]

🔄 Triggered by: push to dev branch
📋 Commit: abc123def456
📝 Message: [module:data-engine] fix: 修复WebSocket连接泄漏问题

Step 1/5: Parse Module from Commit
✅ Found module: data-engine

Step 2/5: Set Environment Variables
✅ Environment: dev
✅ Registry: hermesflow-dev-acr.azurecr.io

Step 3/5: Build and Push Module
🦀 Building Rust module...
   Compiling data-engine v0.1.0
   Finished release [optimized] target(s) in 3m 45s
🐳 Building Docker image...
   => [stage-1 1/3] FROM rust:1.75-slim
   => [stage-2 1/2] COPY --from=stage-1 /app/target/release/data-engine
   => naming to hermesflow-dev-acr.azurecr.io/data-engine:abc123def456
✅ Built successfully

📤 Pushing images to registry...
   abc123def456: Pushed
   dev-latest: Pushed
✅ Push complete

Step 4/5: Update GitOps Repository
🔗 Triggering GitOps workflow...
✅ Workflow dispatched

Step 5/5: Build Summary
✅ Build completed successfully
   Duration: 4m 52s
```

**步骤3：GitOps仓库自动更新**（约1分钟）

```
[GitHub Actions - HermesFlow-GitOps Repo]

🔄 Triggered by: workflow_dispatch
📦 Module: data-engine
🏷️ Tag: abc123def456
🌍 Environment: dev

Step 1/4: Checkout GitOps Repo
✅ Checked out main branch

Step 2/4: Install yq
✅ yq installed

Step 3/4: Update values.yaml
📝 Updating apps/dev/data-engine/values.yaml
   Image Tag: abc123def456
   Registry: hermesflow-dev-acr.azurecr.io
✅ Updated successfully

Step 4/4: Commit and Push
✅ Committed: 🚀 Update data-engine to abc123def456 in dev
✅ Pushed to main branch
```

**步骤4：ArgoCD自动同步**（约3分钟）

```
[ArgoCD Dashboard]

Application: hermesflow-dev
Status: Syncing...

🔍 Detected Changes:
   apps/dev/data-engine/values.yaml
   - image.tag: xyz789old → abc123def456

🎯 Sync Operation:
   Phase 1: PreSync
   ✅ Running pre-sync hooks

   Phase 2: Sync
   📦 Applying Deployment/data-engine
      - Updating image to abc123def456
   ✅ Deployment updated

   Phase 3: SyncWait
   ⏳ Waiting for new Pod to be ready...
   🆕 Pod data-engine-abc123def-xxxxx: Creating
   🆕 Pod data-engine-abc123def-xxxxx: Running
   🆕 Pod data-engine-abc123def-xxxxx: Ready ✅
   
   🗑️ Terminating old Pod data-engine-xyz789old-yyyyy
   ✅ Old Pod terminated

   Phase 4: PostSync
   ✅ Running post-sync hooks

✅ Sync Completed
   Duration: 2m 38s
   Status: Healthy
```

**步骤5：验证部署**

```bash
# 查看Pod状态
$ kubectl get pods -n hermesflow-dev -l app=data-engine
NAME                            READY   STATUS    RESTARTS   AGE
data-engine-abc123def-xxxxx     1/1     Running   0          3m

# 查看Pod日志
$ kubectl logs -n hermesflow-dev data-engine-abc123def-xxxxx --tail=50
2024-12-20T10:30:00Z INFO  data_engine::websocket] WebSocket server started on 0.0.0.0:8081
2024-12-20T10:30:01Z INFO  data_engine::connection_pool] Connection pool initialized (max: 10000)
2024-12-20T10:30:02Z INFO  data_engine::health] Health check endpoint ready

# 查看部署历史
$ kubectl rollout history deployment/data-engine -n hermesflow-dev
REVISION  CHANGE-CAUSE
1         Initial deployment
2         Update data-engine to xyz789old
3         Update data-engine to abc123def456

# 测试服务
$ curl http://data-engine.hermesflow-dev.svc.cluster.local:8081/health
{"status":"healthy","version":"abc123def456"}
```

#### 11.7.3 总耗时

| 阶段 | 耗时 | 说明 |
|------|------|------|
| GitHub Actions构建 | ~5分钟 | Rust编译 + Docker构建 + ACR推送 |
| GitOps仓库更新 | ~1分钟 | yq更新values.yaml + git push |
| ArgoCD同步 | ~3分钟 | Helm渲染 + Kubernetes滚动更新 |
| **总计** | **~9分钟** | **全自动，零人工干预** |

### 11.8 监控与回滚

#### 11.8.1 部署监控

**ArgoCD Dashboard**：

```bash
# Web UI访问
https://argocd.hermesflow.com/applications/hermesflow-dev

# CLI查看同步状态
argocd app get hermesflow-dev

# 查看同步历史
argocd app history hermesflow-dev
```

**Kubernetes监控**：

```bash
# 查看Pod滚动更新状态
kubectl rollout status deployment/data-engine -n hermesflow-dev

# 实时监控Pod事件
kubectl get events -n hermesflow-dev --sort-by='.lastTimestamp' -w

# 查看部署详情
kubectl describe deployment data-engine -n hermesflow-dev

# 查看Pod日志（实时）
kubectl logs -n hermesflow-dev -l app=data-engine -f
```

**Prometheus监控指标**：

```promql
# Pod重启次数
kube_pod_container_status_restarts_total{namespace="hermesflow-dev", pod=~"data-engine.*"}

# CPU使用率
rate(container_cpu_usage_seconds_total{namespace="hermesflow-dev", pod=~"data-engine.*"}[5m])

# 内存使用率
container_memory_usage_bytes{namespace="hermesflow-dev", pod=~"data-engine.*"}

# 部署状态
kube_deployment_status_replicas_available{namespace="hermesflow-dev", deployment="data-engine"}
```

#### 11.8.2 快速回滚

**方法1：Kubernetes原生回滚**（最快，~30秒）

```bash
# 回滚到上一个版本
kubectl rollout undo deployment/data-engine -n hermesflow-dev

# 查看回滚状态
kubectl rollout status deployment/data-engine -n hermesflow-dev

# 回滚到特定版本
kubectl rollout undo deployment/data-engine --to-revision=2 -n hermesflow-dev
```

**方法2：ArgoCD回滚**（推荐，约2分钟）

```bash
# CLI回滚
argocd app rollback hermesflow-dev 2  # 回滚到revision 2

# Web UI回滚
# ArgoCD Dashboard → hermesflow-dev → History → Select Revision → Rollback
```

**方法3：GitOps回滚**（最安全，约3分钟）

```bash
# 进入GitOps仓库
cd HermesFlow-GitOps

# 查看commit历史
git log --oneline apps/dev/data-engine/values.yaml

# 方式A：回滚到之前的commit
git revert HEAD
git push

# 方式B：直接修改values.yaml
vim apps/dev/data-engine/values.yaml
# 修改 image.tag 为旧版本
git add apps/dev/data-engine/values.yaml
git commit -m "⏪ Rollback data-engine to xyz789old"
git push

# ArgoCD自动检测并同步
```

**回滚决策矩阵**：

| 情况 | 推荐方法 | 理由 |
|------|---------|------|
| 紧急故障 | Kubernetes原生回滚 | 最快（30秒） |
| 常规回滚 | ArgoCD回滚 | 保留GitOps审计记录 |
| 长期回滚 | GitOps回滚 | Git历史完整，可追溯 |

### 11.9 安全与权限管理

#### 11.9.1 GitHub Secrets配置

**必需的Secrets**：

```yaml
# Dev环境（hermesflow-dev-acr.azurecr.io）
DEV_AZURE_REGISTRY: hermesflow-dev-acr.azurecr.io
DEV_AZURE_CLIENT_ID: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
DEV_AZURE_CLIENT_SECRET: ********************************

# Main环境（hermesflow-prod-acr.azurecr.io）
MAIN_AZURE_REGISTRY: hermesflow-prod-acr.azurecr.io
MAIN_AZURE_CLIENT_ID: yyyyyyyy-yyyy-yyyy-yyyy-yyyyyyyyyyyy
MAIN_AZURE_CLIENT_SECRET: ********************************

# GitOps（细粒度访问令牌）
GITOPS_TOKEN: ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxx
# 权限范围：
#   - repo: HermesFlow-GitOps (Full control)
#   - workflow (Trigger workflow)
```

**配置步骤**：

```bash
# 1. 创建Azure Service Principal
az ad sp create-for-rbac --name hermesflow-dev-acr-sp \
  --role acrpush \
  --scopes /subscriptions/{sub-id}/resourceGroups/{rg}/providers/Microsoft.ContainerRegistry/registries/hermesflow-dev-acr

# 2. 将输出的appId和password配置到GitHub Secrets
# DEV_AZURE_CLIENT_ID = appId
# DEV_AZURE_CLIENT_SECRET = password

# 3. 创建GitHub Personal Access Token
# Settings → Developer settings → Personal access tokens → Fine-grained tokens
# Repository access: Only select HermesFlow-GitOps
# Permissions: Contents (Read and write), Workflows (Read and write)
```

#### 11.9.2 权限最小化原则

| 组件 | 权限范围 | 说明 |
|------|---------|------|
| **GitHub Actions** | - 推送镜像到ACR<br>- 触发GitOps workflow | 无代码写入权限 |
| **GitOps Token** | - 触发workflow<br>- 写入values.yaml | 仅限GitOps仓库 |
| **ArgoCD** | - 读取GitOps仓库<br>- 写入K8s namespace | 仅限特定namespace |
| **AKS Service Principal** | - 拉取ACR镜像 | 仅AcrPull权限 |

#### 11.9.3 审计日志

**Git审计**：

```bash
# 查看GitOps仓库所有部署变更
cd HermesFlow-GitOps
git log --oneline --all --graph

# 查看特定模块的变更历史
git log --oneline -- apps/dev/data-engine/values.yaml

# 查看谁修改了配置
git blame apps/dev/data-engine/values.yaml
```

**Kubernetes审计**：

```bash
# 查看部署事件
kubectl get events -n hermesflow-dev --field-selector involvedObject.name=data-engine

# 查看ArgoCD同步日志
kubectl logs -n argocd -l app.kubernetes.io/name=argocd-application-controller
```

### 11.10 CI/CD 最佳实践

#### 11.10.1 Commit Message规范

**规范格式**：

```
[module:<模块名>] <类型>: <简短描述>

<详细描述>

<可选的Footer>
```

**类型标签**：

| 类型 | 说明 | 示例 |
|------|------|------|
| `feat` | 新功能 | `[module:data-engine] feat: 添加Polygon数据源` |
| `fix` | Bug修复 | `[module:strategy-engine] fix: 修复RSI因子计算错误` |
| `perf` | 性能优化 | `[module:data-engine] perf: 优化WebSocket连接池` |
| `refactor` | 代码重构 | `[module:trading-engine] refactor: 重构订单管理模块` |
| `test` | 测试相关 | `[module:risk-engine] test: 添加风控规则单元测试` |
| `docs` | 文档更新 | `docs: 更新API文档` (无需模块标签) |
| `chore` | 构建/工具 | `chore: 升级依赖版本` (无需模块标签) |

**示例**：

```bash
# ✅ 好的Commit Message
git commit -m "[module:data-engine] feat: 添加Binance WebSocket自动重连机制

- 实现指数退避重连策略
- 添加连接健康检查
- 记录重连事件日志
- 测试覆盖率：95%

Closes #123"

# ❌ 不好的Commit Message
git commit -m "修复Bug"  # 缺少模块标签，不会触发部署
```

#### 11.10.2 环境分支策略

```
feature/* ──┐
            ├──→ dev ──→ dev环境（自动部署）
hotfix/*  ──┘            │
                         │
                    (Pull Request)
                         │
                         ↓
                       main ──→ main环境（自动部署）
```

**分支规则**：

| 分支 | 目的 | 部署 | 保护规则 |
|------|------|------|---------|
| `feature/*` | 功能开发 | ❌ 不部署 | 无 |
| `hotfix/*` | 紧急修复 | ❌ 不部署 | 无 |
| `dev` | 开发环境 | ✅ 自动部署到dev | - 需要PR审核 |
| `main` | 生产环境 | ✅ 自动部署到main | - 需要2人审核<br>- 必须通过CI<br>- 禁止force push |

#### 11.10.3 镜像标签策略

**推荐**：

```bash
# ✅ 使用Commit SHA（精确版本控制）
hermesflow-dev-acr.azurecr.io/data-engine:abc123def456

# ✅ 保留环境Latest（快速引用）
hermesflow-dev-acr.azurecr.io/data-engine:dev-latest
```

**不推荐**：

```bash
# ❌ 不要仅使用latest（难以追踪版本）
hermesflow-dev-acr.azurecr.io/data-engine:latest

# ❌ 不要使用v1.0.0这种语义化版本（手动维护困难）
hermesflow-dev-acr.azurecr.io/data-engine:v1.0.0
```

#### 11.10.4 金丝雀发布（可选）

**使用Argo Rollouts实现渐进式发布**：

```yaml
# apps/dev/data-engine/rollout.yaml
apiVersion: argoproj.io/v1alpha1
kind: Rollout
metadata:
  name: data-engine
spec:
  replicas: 5
  strategy:
    canary:
      steps:
      - setWeight: 20    # 20%流量到新版本
      - pause: {duration: 5m}
      - setWeight: 40
      - pause: {duration: 5m}
      - setWeight: 60
      - pause: {duration: 5m}
      - setWeight: 80
      - pause: {duration: 5m}
      - setWeight: 100   # 100%流量，完成发布
      
      # 自动回滚（如果错误率>5%）
      analysis:
        templates:
        - templateName: error-rate
        args:
        - name: service-name
          value: data-engine
      
      trafficRouting:
        istio:
          virtualService:
            name: data-engine-vsvc
```

#### 11.10.5 自动化测试集成

**在CI流程中添加测试**：

```yaml
# .github/workflows/module-cicd.yml中添加
- name: Run Unit Tests
  run: |
    cd modules/${{ env.MODULE }}
    case "${{ env.MODULE }}" in
      *-engine|user-management|gateway)
        mvn test
        ;;
      data-engine)
        cargo test
        ;;
      frontend)
        npm test
        ;;
    esac

- name: Run Integration Tests
  run: |
    docker-compose -f docker-compose.test.yml up --abort-on-container-exit
    exit_code=$?
    docker-compose -f docker-compose.test.yml down
    exit $exit_code

- name: Security Scan
  uses: aquasecurity/trivy-action@master
  with:
    image-ref: '${{ env.AZURE_REGISTRY }}/${{ env.MODULE }}:${{ github.sha }}'
    format: 'sarif'
    output: 'trivy-results.sarif'
```

### 11.11 故障排查指南

#### 11.11.1 常见问题

**问题1：GitHub Actions构建失败**

```bash
# 症状
❌ Docker build failed: Error response from daemon: pull access denied

# 原因
ACR认证失败

# 解决方案
# 1. 检查GitHub Secrets配置
# 2. 验证Service Principal权限
az role assignment list --assignee $AZURE_CLIENT_ID --scope /subscriptions/{sub-id}/resourceGroups/{rg}/providers/Microsoft.ContainerRegistry/registries/hermesflow-dev-acr

# 3. 重新生成密钥
az ad sp credential reset --id $AZURE_CLIENT_ID
```

**问题2：GitOps更新失败**

```bash
# 症状
❌ Failed to trigger GitOps workflow: 404 Not Found

# 原因
GITOPS_TOKEN权限不足或过期

# 解决方案
# 1. 重新生成GitHub Token
# 2. 确保Token有workflow权限
# 3. 更新GitHub Secret
```

**问题3：ArgoCD同步失败**

```bash
# 症状
❌ Sync operation failed: ImagePullBackOff

# 原因
K8s无法从ACR拉取镜像

# 解决方案
# 1. 检查AKS与ACR的集成
az aks check-acr --name hermesflow-aks-dev --resource-group hermesflow-rg --acr hermesflow-dev-acr.azurecr.io

# 2. 重新附加ACR到AKS
az aks update --name hermesflow-aks-dev --resource-group hermesflow-rg --attach-acr hermesflow-dev-acr
```

#### 11.11.2 调试技巧

**查看完整日志**：

```bash
# GitHub Actions日志
https://github.com/tomxiao/HermesFlow/actions

# ArgoCD日志
kubectl logs -n argocd -l app.kubernetes.io/name=argocd-application-controller --tail=100

# Pod日志
kubectl logs -n hermesflow-dev -l app=data-engine --tail=100 --previous  # 查看之前的Pod日志
```

**手动触发部署**：

```bash
# 手动触发GitOps更新
curl -X POST \
  -H "Authorization: token $GITOPS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "ref": "main",
    "inputs": {
      "module": "data-engine",
      "tag": "abc123def",
      "environment": "dev",
      "registry": "hermesflow-dev-acr.azurecr.io"
    }
  }' \
  https://api.github.com/repos/tomxiao/HermesFlow-GitOps/actions/workflows/update-values.yml/dispatches

# 手动触发ArgoCD同步
argocd app sync hermesflow-dev --force
```

---

## 附录D：CI/CD相关资源

### GitHub Repositories

- **HermesFlow**: https://github.com/tomxiao/HermesFlow (源代码)
- **HermesFlow-GitOps**: https://github.com/tomxiao/HermesFlow-GitOps (声明式配置)

### 参考文档

1. [GitHub Actions官方文档](https://docs.github.com/en/actions)
2. [Helm官方文档](https://helm.sh/docs/)
3. [ArgoCD官方文档](https://argo-cd.readthedocs.io/)
4. [GitOps最佳实践](https://opengitops.dev/)
5. [Azure Container Registry文档](https://docs.microsoft.com/en-us/azure/container-registry/)

### 工具链

| 工具 | 版本 | 用途 |
|------|------|------|
| GitHub Actions | - | CI/CD流水线 |
| Helm | 3.x | Kubernetes包管理 |
| ArgoCD | 2.x | GitOps持续部署 |
| yq | 4.x | YAML处理工具 |
| kubectl | 1.28+ | Kubernetes CLI |

---

**文档结束**

