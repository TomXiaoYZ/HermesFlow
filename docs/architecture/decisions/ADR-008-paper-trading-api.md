# ADR-008: 模拟交易与实盘API兼容设计

**状态**: Accepted  
**日期**: 2024-12-20  
**决策者**: Architecture Team  
**相关人员**: 策略引擎团队、交易服务团队

---

## 上下文

量化交易策略开发流程：

```
策略开发 → 回测 → 模拟交易 → 实盘交易
          ↓                   ↑
          └─────调优迭代───────┘
```

**问题**：
- 模拟交易和实盘交易代码不一致
- 切换环境需要修改代码
- 模拟环境无法真实验证策略

**需求**：
- 模拟交易与实盘API完全兼容
- 切换环境仅需配置改变
- 模拟环境精确模拟市场行为

### 设计目标

```python
# 目标：同一套策略代码，无缝切换模拟/实盘

class Strategy:
    def __init__(self, broker: Broker):
        self.broker = broker  # 可以是PaperBroker或LiveBroker
    
    def on_bar(self, bar):
        if self.should_buy(bar):
            # 模拟和实盘使用相同的API
            self.broker.submit_order(
                symbol='BTCUSDT',
                side='buy',
                order_type='market',
                quantity=0.1
            )

# 模拟环境
strategy = Strategy(PaperTradingBroker(initial_cash=10000))

# 实盘环境（仅更换Broker，策略代码不变）
strategy = Strategy(LiveTradingBroker(api_key='...'))
```

## 决策

设计**统一Broker接口**，PaperTradingBroker与LiveBroker实现相同接口。

### 主要理由

#### 1. 降低切换成本

**统一接口设计**：

```python
from abc import ABC, abstractmethod
from typing import Optional
from enum import Enum

class OrderSide(Enum):
    BUY = "buy"
    SELL = "sell"

class OrderType(Enum):
    MARKET = "market"
    LIMIT = "limit"
    STOP = "stop"

class OrderStatus(Enum):
    PENDING = "pending"
    FILLED = "filled"
    CANCELLED = "cancelled"
    REJECTED = "rejected"

class Broker(ABC):
    """统一Broker接口"""
    
    @abstractmethod
    def submit_order(
        self,
        symbol: str,
        side: OrderSide,
        order_type: OrderType,
        quantity: float,
        price: Optional[float] = None
    ) -> str:
        """
        提交订单
        
        Returns:
            order_id: 订单ID
        """
        pass
    
    @abstractmethod
    def cancel_order(self, order_id: str) -> bool:
        """取消订单"""
        pass
    
    @abstractmethod
    def get_position(self, symbol: str) -> Optional['Position']:
        """获取持仓"""
        pass
    
    @abstractmethod
    def get_portfolio_value(self) -> float:
        """获取组合总值"""
        pass
    
    @abstractmethod
    def get_cash_balance(self) -> float:
        """获取现金余额"""
        pass
```

#### 2. 精确模拟市场行为

**PaperTradingBroker实现**：

```python
class PaperTradingBroker(Broker):
    """模拟交易Broker"""
    
    def __init__(
        self,
        initial_cash: float = 10000.0,
        commission_rate: float = 0.001,  # 0.1%手续费
        slippage_pct: float = 0.001      # 0.1%滑点
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
        side: OrderSide,
        order_type: OrderType,
        quantity: float,
        price: Optional[float] = None
    ) -> str:
        order_id = str(uuid.uuid4())
        
        # 获取当前市场价格（从实时数据订阅）
        current_price = self._get_current_price(symbol)
        
        # 计算成交价（含滑点）
        if order_type == OrderType.MARKET:
            if side == OrderSide.BUY:
                filled_price = current_price * (1 + self.slippage_pct)
            else:
                filled_price = current_price * (1 - self.slippage_pct)
            status = OrderStatus.FILLED
        else:
            filled_price = price
            status = OrderStatus.PENDING
        
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
            filled_price=filled_price if status == OrderStatus.FILLED else None,
            commission=commission if status == OrderStatus.FILLED else 0,
            status=status,
            created_at=datetime.now()
        )
        
        self.orders[order_id] = order
        
        # 如果是市价单，立即执行
        if order_type == OrderType.MARKET:
            self._execute_order(order)
        
        return order_id
    
    def _execute_order(self, order: Order):
        """执行订单（模拟撮合）"""
        total_cost = order.filled_price * order.quantity + order.commission
        
        if order.side == OrderSide.BUY:
            # 检查资金是否充足
            if total_cost > self.cash:
                order.status = OrderStatus.REJECTED
                order.reject_reason = 'Insufficient funds'
                return
            
            # 扣除资金
            self.cash -= total_cost
            
            # 更新持仓
            if order.symbol in self.positions:
                pos = self.positions[order.symbol]
                old_quantity = pos.quantity
                new_quantity = old_quantity + order.quantity
                pos.avg_price = (
                    (pos.avg_price * old_quantity + 
                     order.filled_price * order.quantity) / new_quantity
                )
                pos.quantity = new_quantity
            else:
                self.positions[order.symbol] = Position(
                    symbol=order.symbol,
                    quantity=order.quantity,
                    avg_price=order.filled_price
                )
        
        else:  # SELL
            # 检查持仓是否充足
            if order.symbol not in self.positions:
                order.status = OrderStatus.REJECTED
                order.reject_reason = 'No position to sell'
                return
            
            pos = self.positions[order.symbol]
            if pos.quantity < order.quantity:
                order.status = OrderStatus.REJECTED
                order.reject_reason = 'Insufficient position'
                return
            
            # 增加资金
            self.cash += (order.filled_price * order.quantity - order.commission)
            
            # 更新持仓
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
        
        order.status = OrderStatus.FILLED
        order.filled_at = datetime.now()
```

#### 3. 一致的错误处理

**模拟和实盘错误完全一致**：

```python
class InsufficientFundsError(Exception):
    """资金不足"""
    pass

class InsufficientPositionError(Exception):
    """持仓不足"""
    pass

class InvalidOrderError(Exception):
    """无效订单"""
    pass

# PaperBroker和LiveBroker抛出相同的异常
try:
    broker.submit_order('BTCUSDT', OrderSide.BUY, OrderType.MARKET, 100)
except InsufficientFundsError:
    logger.error("资金不足，无法下单")
```

#### 4. 完整的监控指标

**模拟环境也提供完整的交易指标**：

```python
class PerformanceAnalyzer:
    """性能分析（模拟和实盘通用）"""
    
    def analyze(self, broker: Broker) -> Dict[str, float]:
        """分析交易性能"""
        trades = broker.get_trades()
        
        # 计算关键指标
        total_return = broker.get_portfolio_value() / broker.initial_cash - 1
        sharpe_ratio = self._calculate_sharpe(trades)
        max_drawdown = self._calculate_max_drawdown(broker.get_equity_curve())
        win_rate = len([t for t in trades if t.pnl > 0]) / len(trades)
        
        return {
            'total_return': total_return,
            'sharpe_ratio': sharpe_ratio,
            'max_drawdown': max_drawdown,
            'win_rate': win_rate,
            'total_trades': len(trades),
        }

# 模拟和实盘使用相同的分析代码
paper_metrics = PerformanceAnalyzer().analyze(paper_broker)
live_metrics = PerformanceAnalyzer().analyze(live_broker)
```

### 切换环境

**通过配置切换**：

```python
# config.py
class Config:
    TRADING_MODE = os.getenv('TRADING_MODE', 'paper')  # 'paper' or 'live'
    
    PAPER_TRADING = {
        'initial_cash': 10000,
        'commission_rate': 0.001,
        'slippage_pct': 0.001,
    }
    
    LIVE_TRADING = {
        'api_key': os.getenv('API_KEY'),
        'api_secret': os.getenv('API_SECRET'),
        'exchange': 'binance',
    }

# main.py
def create_broker() -> Broker:
    if Config.TRADING_MODE == 'paper':
        return PaperTradingBroker(**Config.PAPER_TRADING)
    else:
        return LiveTradingBroker(**Config.LIVE_TRADING)

# 策略初始化
broker = create_broker()
strategy = MyStrategy(broker)

# 运行策略（模拟和实盘代码完全相同）
strategy.run()
```

## 后果

### 优点

1. **无缝切换**：
   - 策略代码无需修改
   - 仅更改配置即可切换
   - 降低上线风险

2. **精确验证**：
   - 模拟环境真实模拟市场
   - 手续费、滑点、延迟都模拟
   - 提前发现问题

3. **快速迭代**：
   - 模拟环境无风险
   - 可以快速测试策略
   - 节省实盘调试成本

4. **统一监控**：
   - 模拟和实盘使用相同监控
   - 指标完全一致
   - 便于对比分析

### 缺点

1. **模拟精度有限**：
   - 无法完全模拟极端行情
   - 订单簿深度模拟简化
   - 市场影响成本难以模拟

2. **开发成本**：
   - 需要实现两套Broker
   - 接口需要精心设计
   - 测试工作量增加

3. **心理差异**：
   - 模拟交易无心理压力
   - 实盘可能做出不同决策
   - 需要额外的心理准备

### 缓解措施

1. **提升模拟精度**：
   ```python
   class EnhancedPaperBroker(PaperTradingBroker):
       """增强型模拟Broker"""
       
       def __init__(self, *args, **kwargs):
           super().__init__(*args, **kwargs)
           self.orderbook_simulator = OrderBookSimulator()
           self.latency_simulator = LatencySimulator(mean_latency_ms=50)
       
       def submit_order(self, *args, **kwargs):
           # 模拟延迟
           time.sleep(self.latency_simulator.sample() / 1000)
           
           # 基于订单簿深度模拟成交
           filled_price = self.orderbook_simulator.simulate_fill(*args, **kwargs)
           
           return super().submit_order(*args, **kwargs)
   ```

2. **压力测试**：
   ```python
   def stress_test_strategy():
       """压力测试模拟环境"""
       # 极端行情场景
       scenarios = [
           'flash_crash',      # 闪崩
           'high_volatility',  # 高波动
           'low_liquidity',    # 低流动性
       ]
       
       for scenario in scenarios:
           broker = PaperTradingBroker()
           broker.load_scenario(scenario)
           strategy = MyStrategy(broker)
           strategy.run()
           
           print(f"Scenario {scenario}: {broker.get_metrics()}")
   ```

3. **逐步上线**：
   ```
   阶段1: 模拟交易（2周）
   - 验证策略逻辑
   - 调整参数
   - 积累信心
   
   阶段2: 小资金实盘（1周）
   - 100美元试跑
   - 验证API对接
   - 观察心理状态
   
   阶段3: 正式上线
   - 逐步增加资金
   - 持续监控
   - 及时调整
   ```

## 实施经验

### 3个月后回顾

**成功点**：
- ✅ 10个策略从模拟切换到实盘，代码零修改
- ✅ 模拟环境发现3个重大Bug
- ✅ 实盘与模拟收益差异<5%
- ✅ 切换环境仅需改配置

**挑战点**：
- ⚠️ 模拟环境滑点模型需要持续优化
- ⚠️ 极端行情模拟不够精确
- ⚠️ 部分策略实盘表现不如模拟

**改进建议**：
1. 基于历史数据优化滑点模型
2. 引入订单簿回放功能
3. 建立模拟vs实盘对比报告
4. 定期审计模拟精度

## 备选方案

### 为什么不使用两套代码？

虽然分开开发更灵活，但：
- 维护成本高（修改需要改两处）
- 容易出现不一致
- 切换需要重写代码

**结论**：统一接口降低风险。

## 相关决策

- [ADR-001: 采用混合技术栈架构](./ADR-001-hybrid-tech-stack.md)

## 参考资料

1. [Backtrader框架](https://www.backtrader.com/)（参考其Broker接口设计）
2. [Zipline框架](https://github.com/quantopian/zipline)
3. "Algorithmic Trading" by Ernest P. Chan
4. "Quantitative Trading" by Ernest P. Chan

