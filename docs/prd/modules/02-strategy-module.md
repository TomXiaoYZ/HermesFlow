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

## 5. 策略模板库

### 5.1 预定义策略模板

1. **趋势跟踪**
   - 移动平均线交叉
   - MACD策略
   - 海龟交易法

2. **均值回归**
   - Bollinger Bands反转
   - RSI超买超卖
   - 配对交易

3. **套利策略**
   - 跨交易所套利
   - 三角套利
   - 期现套利

4. **机器学习**
   - 价格预测模型
   - 分类器信号
   - 强化学习

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

