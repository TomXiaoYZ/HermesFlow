# 策略模块详细需求文档

**模块名称**: 策略模块 (Strategy Module)  
**技术栈**: Python 3.12 + FastAPI  
**版本**: v2.0.0  
**最后更新**: 2024-12-20

---

## 目录

1. [模块概述](#1-模块概述)
2. [Python技术选型说明](#2-python技术选型说明)
3. [架构设计](#3-架构设计)
4. [Epic详述](#4-epic详述)
5. [策略模板库](#5-策略模板库)
6. [API规范](#6-api规范)
7. [性能基线与测试](#7-性能基线与测试)

---

## 1. 模块概述

### 1.1 模块职责

策略模块是HermesFlow平台的**策略开发与执行核心**，负责：

1. **策略开发框架**: 提供灵活的Python策略开发API
2. **策略执行引擎**: 事件驱动的策略实时执行
3. **回测引擎**: 基于历史数据的策略性能验证
4. **策略优化**: 参数优化和策略性能分析
5. **策略管理**: 策略版本控制和生命周期管理

### 1.2 核心价值

- **易用性**: Python语法简洁，开发效率高
- **灵活性**: 支持自定义策略逻辑和指标
- **完整性**: 从开发、回测到实盘的完整流程
- **可扩展**: 支持多策略并发执行
- **数据分析**: 集成Pandas、NumPy等数据分析库

### 1.3 性能目标

| 指标 | 目标值 | 测量方法 |
|------|--------|---------|
| 策略执行延迟 | P99 < 10ms | Prometheus监控 |
| 回测速度 | > 1000 bars/s | 基准测试 |
| 并发策略数 | > 50 | 负载测试 |
| 策略代码编译 | < 2s | 启动时间测量 |

---

## 2. Python技术选型说明

### 2.1 为什么选择Python？

#### 易用性优势

1. **语法简洁**: 降低策略开发门槛
2. **开发效率**: 快速原型验证
3. **社区生态**: 丰富的量化交易库

#### 数据分析优势

1. **Pandas**: 强大的时间序列处理
2. **NumPy**: 高效的数值计算
3. **TA-Lib**: 技术指标库
4. **Scikit-learn**: 机器学习支持

#### 不足之处及缓解

- **性能**: 通过NumPy/Cython优化热点代码
- **并发**: 使用asyncio异步编程
- **类型安全**: 使用Type Hints + mypy

### 2.2 核心依赖

```python
# pyproject.toml
[tool.poetry.dependencies]
python = "^3.12"
fastapi = "^0.104"
uvicorn = {extras = ["standard"], version = "^0.24"}
pandas = "^2.1"
numpy = "^1.26"
ta-lib = "^0.4"  # 技术指标
backtrader = "^1.9"  # 回测框架
scikit-learn = "^1.3"  # 机器学习
pydantic = "^2.5"  # 数据验证
asyncpg = "^0.29"  # PostgreSQL异步驱动
redis = {extras = ["hiredis"], version = "^5.0"}
aiokafka = "^0.10"  # Kafka异步客户端
grpcio = "^1.60"  # gRPC客户端
prometheus-client = "^0.19"  # 指标导出
```

---

## 3. 架构设计

### 3.1 整体架构

```
┌─────────────────────────────────────────────────────────┐
│              策略引擎服务 (Port 18020)                     │
│                 Python + FastAPI                         │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │  Strategy    │  │   Execution  │  │  Performance │  │
│  │  Framework   │  │   Engine     │  │  Analyzer    │  │
│  │              │  │              │  │              │  │
│  │ • BaseStrat  │──│ • EventLoop  │──│ • Metrics    │  │
│  │ • Indicators │  │ • Signals    │  │ • Reports    │  │
│  │ • Templates  │  │ • Orders     │  │ • Visualize  │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
│                                                           │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│              回测引擎服务 (Port 18021)                     │
│                    Python + Pandas                        │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │  Backtest    │  │  Simulation  │  │   Report     │  │
│  │  Engine      │  │  Broker      │  │  Generator   │  │
│  │              │  │              │  │              │  │
│  │ • DataFeed   │──│ • Orders     │──│ • Stats      │  │
│  │ • Strategy   │  │ • Positions  │  │ • Charts     │  │
│  │ • Runner     │  │ • Slippage   │  │ • Export     │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
│                                                           │
└─────────────────────────────────────────────────────────┘
```

### 3.2 策略基类设计

```python
from abc import ABC, abstractmethod
from typing import Dict, Any, Optional
from dataclasses import dataclass
import pandas as pd

@dataclass
class Bar:
    """K线数据"""
    timestamp: int
    symbol: str
    open: float
    high: float
    low: float
    close: float
    volume: float

class BaseStrategy(ABC):
    """策略基类"""
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.positions = {}
        self.orders = []
        self.data = {}
        
    @abstractmethod
    def on_init(self):
        """策略初始化，设置指标等"""
        pass
    
    @abstractmethod
    def on_bar(self, bar: Bar):
        """K线更新回调"""
        pass
    
    def on_tick(self, tick: Dict):
        """Tick数据回调（可选）"""
        pass
    
    def on_order(self, order: Dict):
        """订单状态更新回调"""
        pass
    
    def on_trade(self, trade: Dict):
        """成交回调"""
        pass
    
    # 交易API
    def buy(self, symbol: str, size: float, price: Optional[float] = None):
        """买入"""
        pass
    
    def sell(self, symbol: str, size: float, price: Optional[float] = None):
        """卖出"""
        pass
    
    def get_position(self, symbol: str) -> Dict:
        """获取持仓"""
        pass
```

---

## 4. Epic详述

### Epic 1: 策略开发框架 [P0]

#### 功能描述

提供完整的策略开发API，支持自定义策略逻辑、技术指标、信号生成。

#### 子功能

1. **策略基类** [P0]
   - BaseStrategy抽象类
   - 生命周期回调（on_init/on_bar/on_tick等）
   - 交易API（buy/sell/cancel）
   - 仓位管理API

2. **技术指标库** [P0]
   - 常用指标（MA/EMA/RSI/MACD/Bollinger等）
   - 自定义指标支持
   - 指标缓存优化

3. **数据访问API** [P0]
   - 历史数据查询
   - 实时数据订阅
   - 多周期数据支持

4. **策略参数配置** [P1]
   - 参数定义和验证
   - 动态参数调整
   - 参数持久化

#### 用户故事

```gherkin
Feature: 开发移动平均线交叉策略
  作为一个策略开发者
  我想要使用Python开发MA交叉策略
  以便验证策略逻辑

Scenario: 编写策略代码
  Given 我创建一个新策略文件
  When 我继承BaseStrategy基类
  And 我实现on_init和on_bar方法
  And 我使用MA指标生成交易信号
  Then 策略应该能够成功编译
  And 策略应该能够接收实时数据

Scenario: 策略参数配置
  Given 我的策略需要fast_period和slow_period参数
  When 我在配置中定义参数默认值
  And 我在Web界面修改参数值
  Then 策略应该使用新的参数值
  And 参数修改应该保存到数据库
```

#### 技术实现

```python
# 策略示例
from hermesflow.strategy import BaseStrategy, Indicator

class MovingAverageCrossover(BaseStrategy):
    """移动平均线交叉策略"""
    
    # 参数定义
    params = {
        'fast_period': 10,
        'slow_period': 30,
        'position_size': 1.0
    }
    
    def on_init(self):
        """初始化指标"""
        self.fast_ma = Indicator.SMA(self.params['fast_period'])
        self.slow_ma = Indicator.SMA(self.params['slow_period'])
        
        # 订阅数据
        self.subscribe(['BTCUSDT'], timeframe='1m')
    
    def on_bar(self, bar: Bar):
        """K线更新"""
        # 更新指标
        self.fast_ma.update(bar.close)
        self.slow_ma.update(bar.close)
        
        # 检查指标是否就绪
        if not self.fast_ma.ready or not self.slow_ma.ready:
            return
        
        # 获取当前持仓
        position = self.get_position(bar.symbol)
        
        # 生成交易信号
        if self.fast_ma.crossover(self.slow_ma):
            # 金叉：买入
            if position.size == 0:
                self.buy(bar.symbol, self.params['position_size'])
                self.log(f"金叉买入 {bar.symbol} @ {bar.close}")
                
        elif self.fast_ma.crossunder(self.slow_ma):
            # 死叉：卖出
            if position.size > 0:
                self.sell(bar.symbol, position.size)
                self.log(f"死叉卖出 {bar.symbol} @ {bar.close}")
```

#### 验收标准

- [ ] 支持至少20个常用技术指标
- [ ] 策略代码热重载 < 2秒
- [ ] 支持多交易对同时交易
- [ ] 参数验证准确率 100%
- [ ] API文档完整覆盖

---

### Epic 2: 回测引擎 [P0]

#### 功能描述

基于历史数据验证策略性能，计算收益、风险等关键指标。

#### 子功能

1. **回测框架** [P0]
   - 历史数据回放
   - 事件驱动模拟
   - 多周期回测支持

2. **模拟交易** [P0]
   - 订单撮合模拟
   - 滑点模拟
   - 手续费计算
   - 仓位管理

3. **性能分析** [P0]
   - 收益率计算（总收益、年化、月度等）
   - 风险指标（夏普比率、最大回撤、波动率）
   - 交易统计（胜率、盈亏比、交易次数）
   - 资金曲线生成

4. **回测报告** [P1]
   - HTML报告生成
   - 图表可视化
   - PDF导出

#### 用户故事

```gherkin
Feature: 回测MA交叉策略
  作为一个策略开发者
  我想要回测我的策略
  以便评估策略历史表现

Scenario: 执行回测
  Given 我有一个MA交叉策略
  And 我设置回测参数：
    | 开始日期 | 2024-01-01 |
    | 结束日期 | 2024-12-01 |
    | 初始资金 | 10000 USDT |
    | 交易对   | BTCUSDT    |
  When 我点击"开始回测"按钮
  Then 系统应该在30秒内完成回测
  And 系统应该返回完整的回测报告
  And 报告应该包含收益率、夏普比率、最大回撤等指标

Scenario: 查看回测报告
  Given 回测已完成
  When 我打开回测报告页面
  Then 我应该看到资金曲线图
  And 我应该看到收益率：+25.3%
  And 我应该看到夏普比率：1.85
  And 我应该看到最大回撤：-8.5%
  And 我应该看到交易明细表
```

#### 技术实现

```python
class BacktestEngine:
    """回测引擎"""
    
    def __init__(self, strategy_class, config: BacktestConfig):
        self.strategy_class = strategy_class
        self.config = config
        self.broker = SimulatedBroker(
            cash=config.initial_cash,
            commission=config.commission,
            slippage=config.slippage
        )
        
    async def run(self) -> BacktestResult:
        """运行回测"""
        # 1. 加载历史数据
        data = await self.load_data(
            symbols=self.config.symbols,
            start=self.config.start_date,
            end=self.config.end_date,
            timeframe=self.config.timeframe
        )
        
        # 2. 初始化策略
        strategy = self.strategy_class(self.config.strategy_params)
        strategy.on_init()
        
        # 3. 逐条回放数据
        for timestamp, bars in data.iterrows():
            # 更新市场价格
            self.broker.update_prices(bars)
            
            # 调用策略
            for symbol in self.config.symbols:
                bar = Bar(
                    timestamp=timestamp,
                    symbol=symbol,
                    open=bars[f'{symbol}_open'],
                    high=bars[f'{symbol}_high'],
                    low=bars[f'{symbol}_low'],
                    close=bars[f'{symbol}_close'],
                    volume=bars[f'{symbol}_volume']
                )
                strategy.on_bar(bar)
            
            # 处理订单
            self.broker.process_orders()
            
            # 更新持仓
            self.broker.update_positions()
        
        # 4. 计算性能指标
        result = self.calculate_metrics(
            trades=self.broker.trades,
            equity_curve=self.broker.equity_curve
        )
        
        return result
    
    def calculate_metrics(self, trades, equity_curve) -> Dict:
        """计算性能指标"""
        return {
            'total_return': self._total_return(equity_curve),
            'annual_return': self._annual_return(equity_curve),
            'sharpe_ratio': self._sharpe_ratio(equity_curve),
            'max_drawdown': self._max_drawdown(equity_curve),
            'win_rate': self._win_rate(trades),
            'profit_factor': self._profit_factor(trades),
            'total_trades': len(trades),
            'avg_trade': np.mean([t.pnl for t in trades])
        }
```

#### 验收标准

- [ ] 回测速度 > 1000 bars/s
- [ ] 支持1分钟到日线多周期
- [ ] 性能指标计算准确（vs手工计算误差<0.1%）
- [ ] 支持最长1年历史数据回测
- [ ] 内存占用 < 2GB（1年分钟数据）

---

### Epic 3: 策略执行引擎 [P0]

#### 功能描述

实盘环境下实时执行策略，处理实时数据流，生成交易信号。

#### 子功能

1. **实时数据订阅** [P0]
   - 通过gRPC订阅Rust数据服务
   - 多交易对数据管理
   - 断线重连

2. **策略执行循环** [P0]
   - 事件驱动架构
   - 异步非阻塞
   - 错误隔离

3. **订单管理** [P0]
   - 订单提交到执行服务
   - 订单状态跟踪
   - 成交回调

4. **状态管理** [P1]
   - 持仓状态同步
   - 策略状态持久化
   - 断点恢复

#### 用户故事

```gherkin
Feature: 实盘运行策略
  作为一个交易者
  我想要在实盘运行我的策略
  以便实现自动化交易

Scenario: 启动策略
  Given 我有一个回测表现良好的策略
  And 我已配置API密钥
  When 我点击"启动实盘"按钮
  Then 系统应该连接到数据服务
  And 系统应该订阅相关交易对数据
  And 策略状态应该变为"运行中"
  And 我应该在仪表盘看到实时更新

Scenario: 处理交易信号
  Given 策略正在运行
  When 策略生成买入信号
  Then 系统应该提交订单到交易执行服务
  And 订单应该在100ms内提交
  And 我应该收到订单确认通知
  And 订单应该记录到数据库
```

#### 技术实现

```python
class StrategyExecutor:
    """策略执行器"""
    
    def __init__(self, strategy: BaseStrategy):
        self.strategy = strategy
        self.running = False
        self.data_client = None
        self.order_client = None
        
    async def start(self):
        """启动策略"""
        self.running = True
        
        # 1. 连接数据服务（gRPC）
        self.data_client = await self.connect_data_service()
        
        # 2. 订阅数据流
        symbols = self.strategy.get_subscribed_symbols()
        stream = self.data_client.stream_market_data(
            exchanges=['binance'],
            symbols=symbols,
            data_types=['trade', 'kline_1m']
        )
        
        # 3. 初始化策略
        self.strategy.on_init()
        
        # 4. 事件循环
        async for event in stream:
            if not self.running:
                break
                
            try:
                # 处理数据事件
                if event.data_type == 'kline_1m':
                    bar = self._convert_to_bar(event)
                    self.strategy.on_bar(bar)
                    
                # 检查策略是否生成订单
                await self._process_pending_orders()
                
            except Exception as e:
                logger.error(f"策略执行错误: {e}", exc_info=True)
                # 错误隔离，不影响其他策略
    
    async def _process_pending_orders(self):
        """处理待提交订单"""
        orders = self.strategy.get_pending_orders()
        
        for order in orders:
            try:
                # 提交到交易执行服务
                result = await self.order_client.create_order(
                    exchange=order.exchange,
                    symbol=order.symbol,
                    side=order.side,
                    type=order.type,
                    quantity=order.quantity,
                    price=order.price
                )
                
                # 更新策略订单状态
                self.strategy.on_order_submitted(order.id, result.order_id)
                
            except Exception as e:
                logger.error(f"订单提交失败: {e}")
                self.strategy.on_order_error(order.id, str(e))
```

#### 验收标准

- [ ] 数据接收延迟 < 100ms
- [ ] 订单提交延迟 < 100ms
- [ ] 支持 > 50并发策略
- [ ] 策略异常不影响其他策略
- [ ] 支持热重启（恢复状态）

---

### Epic 4: 策略优化 [P1]

#### 功能描述

自动优化策略参数，提升策略性能。

#### 子功能

1. **参数网格搜索** [P1]
   - 定义参数范围
   - 笛卡尔积遍历
   - 并行回测

2. **遗传算法优化** [P1]
   - 种群初始化
   - 适应度函数
   - 交叉变异

3. **Walk-Forward分析** [P2]
   - 滚动窗口优化
   - 样本外验证
   - 过拟合检测

#### 用户故事

```gherkin
Feature: 优化策略参数
  作为一个策略开发者
  我想要自动优化策略参数
  以便找到最优参数组合

Scenario: 网格搜索
  Given 我有一个MA交叉策略
  And 我设置参数范围：
    | 参数        | 最小值 | 最大值 | 步长 |
    | fast_period | 5      | 20     | 1    |
    | slow_period | 20     | 50     | 2    |
  When 我启动网格搜索
  Then 系统应该回测所有参数组合
  And 系统应该返回最优参数组合
  And 最优参数应该是 fast_period=10, slow_period=30
  And 最优收益率应该 > 基准参数收益率
```

#### 验收标准

- [ ] 支持最多5个参数同时优化
- [ ] 网格搜索支持并行回测（8并发）
- [ ] 遗传算法收敛速度 < 100代
- [ ] 优化结果可视化

---

### Epic 5: 策略管理 [P1]

#### 功能描述

策略版本控制、权限管理、分享等。

#### 子功能

1. **策略CRUD** [P0]
2. **版本控制** [P1]
3. **策略分享** [P2]
4. **策略市场** [P2]

#### 验收标准

- [ ] 支持策略创建/修改/删除
- [ ] 支持版本回滚
- [ ] 支持策略导入/导出

---

## 5. 策略模板库 ⭐ **重大扩展**

> **市场分析发现**：当前仅10个策略模板远不能满足快速盈利需求。竞品QuantConnect提供1000+策略，聚宽提供500+策略。我们需要至少50个成熟策略模板才能帮助用户快速启动交易。

### 5.1 策略模板总览

| 类别 | 策略数量 | 难度 | 预期年化收益率 | 适用市场 |
|------|---------|------|---------------|---------|
| 趋势跟踪 | 12个 | 中等 | 20-60% | 牛市/单边市 |
| 均值回归 | 10个 | 中等 | 15-40% | 震荡市 |
| 套利策略 | 10个 | 高 | 10-100%+ | 所有市场 |
| 网格/DCA | 8个 | 简单 | 10-30% | 震荡市/熊市 |
| 做市策略 | 5个 | 高 | 15-50% | 高流动性市场 |
| 机器学习 | 8个 | 高 | 30-100%+ | 所有市场 |
| 高频策略 | 5个 | 极高 | 50-200%+ | 高流动性市场 |
| **总计** | **58个** | - | - | - |

---

### 5.2 趋势跟踪策略（12个）⭐

#### T1. 双移动平均线交叉（经典）
```python
# 策略参数
fast_period = 10  # 快线周期
slow_period = 30  # 慢线周期

# 信号逻辑
if fast_ma.crossover(slow_ma):  # 金叉
    buy()
elif fast_ma.crossunder(slow_ma):  # 死叉
    sell()
```
**适用场景**：趋势明显的市场  
**预期收益**：年化30-50%  
**风险等级**：中等  
**优势**：简单易懂，参数稳定

#### T2. MACD金叉死叉策略
```python
# MACD指标
macd_line, signal_line, histogram = MACD(12, 26, 9)

# 信号
if macd_line > signal_line and histogram > 0:
    buy()
elif macd_line < signal_line and histogram < 0:
    sell()
```
**适用场景**：中期趋势  
**预期收益**：年化25-45%

#### T3. 海龟交易法（Turtle Trading）
```python
# 突破系统
if price > highest_high(20):  # 20日新高
    buy()
if price < lowest_low(10):    # 10日新低
    sell()

# 止损：2倍ATR
stop_loss = entry_price - 2 * ATR(14)
```
**适用场景**：大趋势市场  
**预期收益**：年化40-80%  
**优势**：经典策略，参数稳定

#### T4. ATR动态止损趋势策略
- 使用ATR（真实波幅）动态调整止损距离
- 波动大时止损宽松，波动小时止损紧密

#### T5. Donchian Channel突破
- 突破N日高点买入，突破N日低点卖出

#### T6. Parabolic SAR跟踪止损
- 使用抛物线指标跟踪趋势

#### T7. Supertrend策略
- 基于ATR和价格的趋势指标

#### T8. Ichimoku Cloud（一目均衡表）
- 日本经典趋势系统

#### T9. 动量策略（Momentum）
- 买入过去N日涨幅最大的标的

#### T10. 多周期共振策略
- 日线+小时线+分钟线三周期确认

#### T11. 趋势强度过滤策略
- 使用ADX（平均趋向指数）过滤弱趋势

#### T12. 突破回踩确认策略
- 突破后回踩支撑位再买入

---

### 5.3 均值回归策略（10个）⭐

#### M1. Bollinger Bands反转
```python
# 布林带参数
bb_upper, bb_middle, bb_lower = BollingerBands(20, 2.0)

# 超卖买入，超买卖出
if price < bb_lower:
    buy()
elif price > bb_upper:
    sell()
```
**适用场景**：震荡市  
**预期收益**：年化15-30%

#### M2. RSI超买超卖
```python
rsi = RSI(14)

if rsi < 30:  # 超卖
    buy()
elif rsi > 70:  # 超买
    sell()
```
**适用场景**：震荡市  
**预期收益**：年化20-35%

#### M3. 配对交易（Pairs Trading）
```python
# 寻找协整资产对
spread = price_A - hedge_ratio * price_B

if spread < mean - 2 * std:  # 价差过低
    buy(A), sell(B)
elif spread > mean + 2 * std:  # 价差过高
    sell(A), buy(B)
```
**适用场景**：相关资产  
**预期收益**：年化15-25%  
**优势**：市场中性，风险低

#### M4. Z-Score均值回归
- 使用标准差倍数判断偏离程度

#### M5. Keltner Channel反转
- 基于ATR的通道指标

#### M6. Williams %R
- 动量型超买超卖指标

#### M7. CCI均值回归
- 顺势指标的逆向应用

#### M8. Stochastic Oscillator
- 随机震荡指标

#### M9. Mean Reversion Grid
- 均值回归+网格交易结合

#### M10. Cointegration策略
- 统计套利的高级形式

---

### 5.4 套利策略（10个）⭐⭐ **高盈利潜力**

#### A1. 跨交易所价差套利
```python
# 实时监控价差
binance_price = get_price('binance', 'BTCUSDT')
okx_price = get_price('okx', 'BTCUSDT')

spread = (okx_price - binance_price) / binance_price

# 考虑手续费和滑点
if spread > (fee + slippage + profit_threshold):
    buy('binance', 'BTCUSDT')
    sell('okx', 'BTCUSDT')
```
**适用场景**：任何市场  
**预期收益**：单次0.3-1.5%，年化30-100%+  
**优势**：市场中性，低风险

#### A2. 三角套利（Triangular Arbitrage）
```python
# 例如：USDT -> BTC -> ETH -> USDT
rate_1 = get_price('BTC/USDT')
rate_2 = get_price('ETH/BTC')
rate_3 = get_price('USDT/ETH')

implied_rate = rate_1 * rate_2 * rate_3

if implied_rate > 1 + threshold:
    execute_arbitrage()
```
**预期收益**：单次0.1-0.5%，年化20-60%

#### A3. 期现套利
- 合约价格与现货价格偏离时套利

#### A4. 资金费率套利
```python
funding_rate = get_funding_rate('BTCUSDT')

if funding_rate > 0.05%:  # 多头支付空头
    # 做空合约+现货做多
    sell_perpetual('BTCUSDT')
    buy_spot('BTCUSDT')
```
**预期收益**：年化15-50%

#### A5. DEX-CEX套利
- Uniswap vs Binance价差套利

#### A6. 闪电贷套利（Flash Loan Arbitrage）
- 使用无抵押贷款进行套利

#### A7. 流动性挖矿优化
- 自动切换高收益LP池

#### A8. LP无常损失对冲
- 使用期权对冲无常损失

#### A9. 跨链桥套利
- 不同链上同一资产的价差

#### A10. 稳定币套利
- USDT/USDC/DAI之间的微小价差

---

### 5.5 网格/DCA策略（8个）⭐⭐⭐ **最易上手**

#### G1. 等差网格交易
```python
class ArithmeticGridBot:
    """等差网格交易机器人"""
    
    def __init__(self, symbol, price_range, grid_count, investment):
        self.symbol = symbol
        self.price_upper = price_range[1]
        self.price_lower = price_range[0]
        self.grid_count = grid_count
        self.investment = investment
    
    def calculate_grid_levels(self):
        """计算网格价格"""
        step = (self.price_upper - self.price_lower) / (self.grid_count - 1)
        return [self.price_lower + i * step for i in range(self.grid_count)]
    
    def place_grid_orders(self):
        """在每个价格水平下买单和卖单"""
        for price in self.grid_levels:
            place_limit_buy(price, quantity)
            place_limit_sell(price + step, quantity)
```
**适用场景**：震荡市  
**预期收益**：月收益3-8%，年化40-100%+  
**优势**：完全自动化，无需盯盘

#### G2. 等比网格交易
- 适合波动大的币种（如山寨币）

#### G3. 动态网格（根据波动率调整）
```python
def adjust_grid_by_volatility():
    volatility = calculate_atr() / price
    if volatility > 0.05:  # 高波动
        grid_count = 20  # 增加网格密度
    else:
        grid_count = 10
```

#### G4. Martingale Grid（马丁格尔网格）
- 下跌时加倍加仓（高风险）

#### G5. 时间定投（DCA）
```python
# 每天固定时间买入固定金额
schedule.every().day.at("10:00").do(buy, amount=100)
```
**适用场景**：长期看好的资产  
**预期收益**：跟随市场  
**优势**：平滑成本，降低择时风险

#### G6. RSI触发DCA
- RSI < 30时加仓

#### G7. 波动率触发DCA
- 高波动时降低投入，低波动时增加

#### G8. 智能DCA（ML预测）
- 使用LSTM预测价格，低点加仓

---

### 5.6 做市策略（5个）⭐⭐ **高级策略**

#### MM1. 简单做市策略
```python
# 在买一和卖一之间挂单
bid_price = best_bid + spread / 2
ask_price = best_ask - spread / 2

place_limit_buy(bid_price, size)
place_limit_sell(ask_price, size)
```
**适用场景**：高流动性市场  
**预期收益**：年化20-50%

#### MM2. Inventory Management（库存管理）
- 根据持仓动态调整报价

#### MM3. Order Book Imbalance
- 根据订单簿失衡调整报价

#### MM4. Avellaneda-Stoikov做市模型
- 学术界经典做市模型

#### MM5. 多层做市策略
- 在多个价格水平提供流动性

---

### 5.7 机器学习策略（8个）⭐⭐ **AI驱动**

#### ML1. LSTM价格预测
```python
from tensorflow.keras.models import Sequential
from tensorflow.keras.layers import LSTM, Dense

# 构建LSTM模型
model = Sequential([
    LSTM(50, return_sequences=True, input_shape=(60, 1)),
    LSTM(50),
    Dense(1)
])

# 预测未来价格
predicted_price = model.predict(recent_prices)

if predicted_price > current_price * 1.02:
    buy()
```
**预期收益**：年化30-80%（取决于模型质量）

#### ML2. 随机森林分类器
- 预测涨跌方向

#### ML3. 强化学习（DQN）
- 让AI自主学习交易策略

#### ML4. 因子模型（Factor Model）
- 多因子选股/选币

#### ML5. 异常检测（Anomaly Detection）
- 检测异常交易机会

#### ML6. NLP情绪分析
- 分析Twitter/Reddit情绪

#### ML7. 集成学习（Ensemble）
- 结合多个模型提升准确率

#### ML8. 时间序列预测（Prophet/ARIMA）
- 统计模型预测

---

### 5.8 高频策略（5个）⭐⭐⭐ **极高难度**

#### HFT1. Market Making（做市）
- 利用买卖价差获利

#### HFT2. Statistical Arbitrage
- 统计套利的高频版本

#### HFT3. Latency Arbitrage
- 利用延迟差套利

#### HFT4. Order Flow Imbalance
- 订单流失衡策略

#### HFT5. Tick数据微观结构
- 基于Tick级别数据的策略

---

### 5.9 策略实施优先级

**第一阶段（MVP，2周）**：
1. ✅ 双均线交叉（T1）
2. ✅ RSI超买超卖（M2）
3. ✅ 等差网格（G1）
4. ✅ 时间定投（G5）
5. ✅ 跨交易所套利（A1）

**第二阶段（1个月）**：
6-15. 趋势跟踪类（T2-T7）
16-20. 均值回归类（M3-M7）
21-25. 套利策略（A2-A6）

**第三阶段（2个月）**：
26-40. 网格/DCA/做市策略
41-50. 机器学习策略

**第四阶段（3个月）**：
51-58. 高频策略

---

## 6. API规范

### 6.1 REST API

```python
# 策略管理
POST   /api/v1/strategies              # 创建策略
GET    /api/v1/strategies              # 策略列表
GET    /api/v1/strategies/:id          # 策略详情
PUT    /api/v1/strategies/:id          # 更新策略
DELETE /api/v1/strategies/:id          # 删除策略

# 策略执行
POST   /api/v1/strategies/:id/start    # 启动策略
POST   /api/v1/strategies/:id/stop     # 停止策略
GET    /api/v1/strategies/:id/status   # 策略状态

# 回测
POST   /api/v1/backtest                # 创建回测
GET    /api/v1/backtest/:id            # 回测结果
GET    /api/v1/backtest/:id/report     # 回测报告

# 优化
POST   /api/v1/optimize                # 参数优化
GET    /api/v1/optimize/:id            # 优化结果
```

---

## 7. 性能基线与测试

### 7.1 性能基线

| 指标 | 目标 | 实际 |
|------|------|------|
| 策略执行延迟 | < 10ms | TBD |
| 回测速度 | > 1000 bars/s | TBD |
| 并发策略数 | > 50 | TBD |
| 内存占用 | < 1GB/策略 | TBD |

### 7.2 测试用例

```python
# tests/test_strategy.py
import pytest
from hermesflow.strategy import BaseStrategy

class TestMAStrategy:
    
    @pytest.fixture
    def strategy(self):
        return MovingAverageCrossover({'fast_period': 10, 'slow_period': 30})
    
    def test_strategy_init(self, strategy):
        """测试策略初始化"""
        strategy.on_init()
        assert strategy.fast_ma is not None
        assert strategy.slow_ma is not None
    
    @pytest.mark.asyncio
    async def test_backtest(self, strategy):
        """测试回测"""
        engine = BacktestEngine(MovingAverageCrossover, BacktestConfig(
            start_date='2024-01-01',
            end_date='2024-12-01',
            initial_cash=10000,
            symbols=['BTCUSDT']
        ))
        
        result = await engine.run()
        
        assert result.total_return > 0
        assert result.sharpe_ratio > 0
        assert len(result.trades) > 0
```

---

**文档维护者**: Strategy Team  
**最后更新**: 2024-12-20

