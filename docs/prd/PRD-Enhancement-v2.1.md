# HermesFlow PRD改进补充文档 (v2.1)

**基于**: v2.0.0 PRD  
**分析依据**: 市场分析与差距评估报告  
**版本**: v2.1.0  
**日期**: 2024-12-20  
**状态**: ⚠️ 待审核

---

## 📋 变更说明

本文档基于市场调研和竞品分析，补充v2.0 PRD中缺失的关键功能，以确保平台具备足够的盈利能力。

### 关键补充内容

1. **Alpha因子库** - 策略开发的核心基础设施
2. **策略优化引擎升级** - 从简单网格搜索到多种高级算法
3. **模拟交易系统** - 实盘前的必要验证环节
4. **机器学习集成路线图** - V2版本的核心竞争力
5. **组合管理系统** - 多策略协同的基础
6. **优先级调整** - 重新规划MVP和后续版本

---

## 目录

1. [Alpha因子库（P0-MVP必须）](#1-alpha因子库)
2. [策略优化引擎增强（P0-MVP必须）](#2-策略优化引擎增强)
3. [模拟交易系统（P0-MVP必须）](#3-模拟交易系统)
4. [机器学习集成（P1-V2）](#4-机器学习集成)
5. [组合管理系统（P1-V2）](#5-组合管理系统)
6. [调整后的开发路线图](#6-调整后的开发路线图)

---

## 1. Alpha因子库

### 1.1 需求背景

**问题陈述**：
- ❌ 当前PRD中没有提供预定义的Alpha因子
- ❌ 用户需要从零实现所有因子，开发效率极低
- ❌ 竞品（优矿300+，聚宽200+）都有丰富的因子库

**市场对标**：
| 平台 | 因子数量 | 分类 |
|------|---------|------|
| 优矿 | 300+ | 价值、成长、质量、技术、量价等 |
| 聚宽 | 200+ | 基本面、技术面、情绪等 |
| Qlib | 300+ | ML特征工程导向 |
| **HermesFlow v2.0** | **0** | ❌ 无 |
| **HermesFlow v2.1** | **100+** | ✅ 目标 |

### 1.2 功能需求

#### 1.2.1 因子分类与数量

**MVP版本（100个因子）**：

| 分类 | 数量 | 优先级 | 说明 |
|------|------|--------|------|
| **技术指标因子** | 20 | P0 | 趋势、动量、振荡器 |
| **量价因子** | 15 | P0 | 成交量、换手率、资金流 |
| **价值因子** | 10 | P1 | PE、PB、PS、PCF |
| **成长因子** | 10 | P1 | 营收增长、利润增长 |
| **质量因子** | 10 | P1 | ROE、ROA、资产负债率 |
| **情绪因子** | 10 | P1 | 涨跌停、新高新低 |
| **波动率因子** | 10 | P0 | 历史波动率、ATR |
| **另类因子** | 15 | P2 | 舆情、资金流向 |
| **总计** | **100** | - | MVP版本 |

**V2版本扩展（200+因子）**：
- 分析师因子（20个）
- 行业因子（30个）
- 宏观因子（20个）
- 高级技术因子（30个）

#### 1.2.2 技术指标因子详细列表（P0）

```python
# 1. 趋势类（8个）
class TrendFactors:
    """趋势类因子"""
    
    @staticmethod
    def MA(data, period):
        """简单移动平均"""
        return data.rolling(period).mean()
    
    @staticmethod
    def EMA(data, period):
        """指数移动平均"""
        return data.ewm(span=period, adjust=False).mean()
    
    @staticmethod
    def MACD(data, fast=12, slow=26, signal=9):
        """MACD指标"""
        ema_fast = data.ewm(span=fast).mean()
        ema_slow = data.ewm(span=slow).mean()
        dif = ema_fast - ema_slow
        dea = dif.ewm(span=signal).mean()
        macd = (dif - dea) * 2
        return {'DIF': dif, 'DEA': dea, 'MACD': macd}
    
    @staticmethod
    def ADX(high, low, close, period=14):
        """平均趋向指标"""
        # 实现ADX计算
        pass
    
    # 其他趋势因子：
    # - Parabolic SAR
    # - Aroon
    # - Ichimoku
    # - SuperTrend

# 2. 动量类（6个）
class MomentumFactors:
    """动量类因子"""
    
    @staticmethod
    def RSI(data, period=14):
        """相对强弱指标"""
        delta = data.diff()
        gain = delta.where(delta > 0, 0).rolling(period).mean()
        loss = -delta.where(delta < 0, 0).rolling(period).mean()
        rs = gain / loss
        return 100 - (100 / (1 + rs))
    
    @staticmethod
    def Momentum(data, period=10):
        """动量指标"""
        return data.diff(period)
    
    @staticmethod
    def ROC(data, period=12):
        """变动率指标"""
        return ((data - data.shift(period)) / data.shift(period)) * 100
    
    # 其他动量因子：
    # - Stochastic Oscillator
    # - Williams %R
    # - CCI (Commodity Channel Index)

# 3. 波动率类（6个）
class VolatilityFactors:
    """波动率类因子"""
    
    @staticmethod
    def BollingerBands(data, period=20, std_dev=2):
        """布林带"""
        sma = data.rolling(period).mean()
        std = data.rolling(period).std()
        return {
            'upper': sma + (std * std_dev),
            'middle': sma,
            'lower': sma - (std * std_dev)
        }
    
    @staticmethod
    def ATR(high, low, close, period=14):
        """真实波幅"""
        tr1 = high - low
        tr2 = abs(high - close.shift())
        tr3 = abs(low - close.shift())
        tr = pd.concat([tr1, tr2, tr3], axis=1).max(axis=1)
        return tr.rolling(period).mean()
    
    @staticmethod
    def HistoricalVolatility(returns, period=20):
        """历史波动率"""
        return returns.rolling(period).std() * np.sqrt(252)
    
    # 其他波动率因子：
    # - Keltner Channels
    # - Donchian Channels
    # - Standard Deviation
```

#### 1.2.3 量价因子详细列表（P0）

```python
# 4. 量价类（15个）
class VolumeFactors:
    """量价类因子"""
    
    @staticmethod
    def OBV(close, volume):
        """能量潮指标"""
        obv = pd.Series(index=close.index, dtype=float)
        obv.iloc[0] = volume.iloc[0]
        for i in range(1, len(close)):
            if close.iloc[i] > close.iloc[i-1]:
                obv.iloc[i] = obv.iloc[i-1] + volume.iloc[i]
            elif close.iloc[i] < close.iloc[i-1]:
                obv.iloc[i] = obv.iloc[i-1] - volume.iloc[i]
            else:
                obv.iloc[i] = obv.iloc[i-1]
        return obv
    
    @staticmethod
    def VWAP(high, low, close, volume):
        """成交量加权平均价"""
        typical_price = (high + low + close) / 3
        return (typical_price * volume).cumsum() / volume.cumsum()
    
    @staticmethod
    def VolumeRatio(volume, period=5):
        """量比"""
        return volume / volume.rolling(period).mean()
    
    @staticmethod
    def Turnover(volume, circulating_shares):
        """换手率"""
        return (volume / circulating_shares) * 100
    
    @staticmethod
    def MoneyFlow(close, volume, period=14):
        """资金流向"""
        typical_price = close
        money_flow = typical_price * volume
        positive_flow = money_flow.where(close.diff() > 0, 0)
        negative_flow = money_flow.where(close.diff() < 0, 0)
        
        mfi = 100 - (100 / (1 + 
            positive_flow.rolling(period).sum() / 
            negative_flow.rolling(period).sum()))
        return mfi
    
    # 其他量价因子：
    # - Accumulation/Distribution Line
    # - Chaikin Money Flow
    # - Force Index
    # - Ease of Movement
    # - Volume Price Trend
    # - Money Flow Index
    # - Volume Weighted Moving Average
    # - Price Volume Trend
    # - Negative Volume Index
    # - Positive Volume Index
```

#### 1.2.4 基本面因子列表（P1）

```python
# 5. 价值因子（10个）
class ValueFactors:
    """价值类因子"""
    
    @staticmethod
    def PE_Ratio(price, eps):
        """市盈率"""
        return price / eps
    
    @staticmethod
    def PB_Ratio(price, book_value_per_share):
        """市净率"""
        return price / book_value_per_share
    
    @staticmethod
    def PS_Ratio(market_cap, revenue):
        """市销率"""
        return market_cap / revenue
    
    @staticmethod
    def PCF_Ratio(market_cap, cash_flow):
        """市现率"""
        return market_cap / cash_flow
    
    @staticmethod
    def DividendYield(dividend_per_share, price):
        """股息率"""
        return (dividend_per_share / price) * 100
    
    @staticmethod
    def EV_EBITDA(enterprise_value, ebitda):
        """企业价值倍数"""
        return enterprise_value / ebitda
    
    # 其他价值因子：
    # - PEG Ratio
    # - Price to Sales Growth
    # - EV to Sales
    # - Graham Number

# 6. 成长因子（10个）
class GrowthFactors:
    """成长类因子"""
    
    @staticmethod
    def RevenueGrowth(revenue_current, revenue_previous):
        """营收增长率"""
        return ((revenue_current - revenue_previous) / 
                revenue_previous) * 100
    
    @staticmethod
    def EPSGrowth(eps_current, eps_previous):
        """每股收益增长率"""
        return ((eps_current - eps_previous) / 
                eps_previous) * 100
    
    @staticmethod
    def NetIncomeGrowth(net_income_current, net_income_previous):
        """净利润增长率"""
        return ((net_income_current - net_income_previous) / 
                net_income_previous) * 100
    
    # 其他成长因子：
    # - Operating Income Growth
    # - Free Cash Flow Growth
    # - Book Value Growth
    # - Asset Growth
    # - Operating Margin Expansion
    # - Return on Equity Growth
    # - ROIC Growth

# 7. 质量因子（10个）
class QualityFactors:
    """质量类因子"""
    
    @staticmethod
    def ROE(net_income, shareholders_equity):
        """净资产收益率"""
        return (net_income / shareholders_equity) * 100
    
    @staticmethod
    def ROA(net_income, total_assets):
        """总资产收益率"""
        return (net_income / total_assets) * 100
    
    @staticmethod
    def DebtToEquity(total_debt, shareholders_equity):
        """资产负债率"""
        return total_debt / shareholders_equity
    
    @staticmethod
    def CurrentRatio(current_assets, current_liabilities):
        """流动比率"""
        return current_assets / current_liabilities
    
    @staticmethod
    def GrossMargin(revenue, cogs):
        """毛利率"""
        return ((revenue - cogs) / revenue) * 100
    
    # 其他质量因子：
    # - Operating Margin
    # - Net Margin
    # - Asset Turnover
    # - Inventory Turnover
    # - Receivables Turnover
```

### 1.3 技术实现

#### 1.3.1 因子计算引擎架构

```python
# hermesflow/factors/engine.py

from abc import ABC, abstractmethod
from typing import Dict, Any, List
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
    
    def list_factors(self, category: str = None) -> List[str]:
        """列出因子"""
        if category:
            return self._categories.get(category, [])
        return list(self._factors.keys())
    
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

# 使用示例
library = FactorLibrary()

# 注册所有因子
from hermesflow.factors.technical import TechnicalFactors
from hermesflow.factors.volume import VolumeFactors
from hermesflow.factors.fundamental import FundamentalFactors

TechnicalFactors.register_all(library)
VolumeFactors.register_all(library)
FundamentalFactors.register_all(library)

# 计算因子
data = pd.DataFrame(...)  # 历史数据
factors = library.calculate_factors(
    data,
    factor_names=['RSI', 'MACD', 'ATR', 'OBV'],
    period=14
)
```

#### 1.3.2 因子性能优化

```python
# 使用NumPy加速
import numba

@numba.jit(nopython=True)
def fast_rsi(prices: np.ndarray, period: int = 14) -> np.ndarray:
    """优化的RSI计算"""
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

### 1.4 用户故事

```gherkin
Feature: 使用Alpha因子开发多因子策略
  作为一个策略开发者
  我想要使用预定义的Alpha因子
  以便快速开发多因子选股策略

Scenario: 查看可用因子列表
  Given 我登录到策略开发平台
  When 我打开因子库浏览器
  Then 我应该看到100+个预定义因子
  And 因子应该按类别分组（技术/量价/基本面等）
  And 每个因子应该有详细说明和使用示例

Scenario: 使用因子开发策略
  Given 我创建一个新的多因子策略
  When 我选择因子['RSI', 'MACD', 'ROE', 'PE_Ratio']
  And 我设置因子权重[0.3, 0.2, 0.3, 0.2]
  And 我设置选股规则："综合得分 > 0.7"
  Then 系统应该自动计算所有因子值
  And 系统应该根据规则筛选股票
  And 我可以查看选股结果和因子分布

Scenario: 因子性能回测
  Given 我有一个基于RSI和MACD的策略
  When 我运行回测（2023-01-01 to 2024-12-01）
  Then 系统应该展示因子IC值（信息系数）
  And 系统应该展示因子覆盖率
  And 系统应该展示因子相关性矩阵
  And 我可以根据结果优化因子组合
```

### 1.5 验收标准

- [ ] 实现100个预定义因子（技术20+量价15+基本面30+其他35）
- [ ] 因子计算性能：>1000 bars/s
- [ ] 因子API文档完整，包含公式和示例
- [ ] 因子单元测试覆盖率 >90%
- [ ] 支持因子缓存（Redis）
- [ ] 支持因子可视化（IC、覆盖率、相关性）

### 1.6 工作量估算

| 任务 | 工作量 | 负责人 |
|------|--------|--------|
| 技术指标因子（20个） | 1周 | Python Dev |
| 量价因子（15个） | 0.5周 | Python Dev |
| 基本面因子（30个） | 1.5周 | Python Dev |
| 其他因子（35个） | 1.5周 | Python Dev |
| 因子引擎框架 | 1周 | Python Dev |
| 性能优化（Numba） | 0.5周 | Python Dev |
| 测试与文档 | 1周 | QA + Python Dev |
| **总计** | **7周（1.75人月）** | - |

---

## 2. 策略优化引擎增强

### 2.1 需求背景

**当前状态**：
- ✅ 基础网格搜索（已实现）
- ⚠️ 遗传算法（计划中）
- ❌ 贝叶斯优化（无）
- ❌ Walk-Forward分析（无）
- ❌ 粒子群算法（无）

**竞品对标**：
| 平台 | 优化算法数量 | 高级功能 |
|------|------------|---------|
| 聚宽 | 5+ | ✅ Walk-Forward, ✅ 蒙特卡洛 |
| 掘金 | 6+ | ✅ 多目标优化 |
| QuantConnect | 8+ | ✅ 云端分布式优化 |
| **HermesFlow v2.0** | 2 | ⚠️ 基础 |
| **HermesFlow v2.1** | 6+ | ✅ 完整 |

### 2.2 功能需求

#### 2.2.1 优化算法清单

**P0 - MVP必须**：

1. ✅ **网格搜索**（已实现）
2. ✅ **随机搜索**（简单）
3. **贝叶斯优化** ⭐⭐⭐
4. **Walk-Forward分析** ⭐⭐⭐

**P1 - V2应该有**：

5. **遗传算法** ⭐⭐
6. **粒子群算法** ⭐
7. **模拟退火** ⭐

**P2 - V3可选**：

8. 多目标优化（Pareto最优）
9. 强化学习优化

#### 2.2.2 贝叶斯优化实现

```python
# hermesflow/optimization/bayesian.py

from skopt import gp_minimize
from skopt.space import Real, Integer, Categorical
from skopt.utils import use_named_args
import numpy as np

class BayesianOptimizer:
    """贝叶斯优化器"""
    
    def __init__(self, strategy_class, backtest_engine):
        self.strategy_class = strategy_class
        self.backtest_engine = backtest_engine
    
    def optimize(self, 
                 param_space: dict,
                 objective: str = 'sharpe_ratio',
                 n_calls: int = 50,
                 random_state: int = 42):
        """
        使用贝叶斯优化寻找最优参数
        
        Args:
            param_space: 参数空间定义
                例如: {
                    'fast_period': (5, 50),
                    'slow_period': (20, 200),
                    'stop_loss': (0.01, 0.10)
                }
            objective: 优化目标（'sharpe_ratio', 'total_return', 'max_drawdown'等）
            n_calls: 优化迭代次数
            random_state: 随机种子
        
        Returns:
            OptimizationResult对象
        """
        
        # 1. 构建搜索空间
        dimensions = []
        param_names = []
        
        for name, bounds in param_space.items():
            if isinstance(bounds[0], int):
                dimensions.append(Integer(bounds[0], bounds[1], name=name))
            else:
                dimensions.append(Real(bounds[0], bounds[1], name=name))
            param_names.append(name)
        
        # 2. 定义目标函数
        @use_named_args(dimensions)
        def objective_function(**params):
            # 运行回测
            result = self.backtest_engine.run(
                strategy_class=self.strategy_class,
                params=params
            )
            
            # 根据优化目标返回值（注意：贝叶斯优化是最小化，所以取负值）
            if objective == 'sharpe_ratio':
                return -result.sharpe_ratio
            elif objective == 'total_return':
                return -result.total_return
            elif objective == 'max_drawdown':
                return result.max_drawdown  # 最小化回撤
            else:
                raise ValueError(f"未知的优化目标: {objective}")
        
        # 3. 执行优化
        result = gp_minimize(
            func=objective_function,
            dimensions=dimensions,
            n_calls=n_calls,
            random_state=random_state,
            verbose=True
        )
        
        # 4. 返回结果
        best_params = dict(zip(param_names, result.x))
        
        return OptimizationResult(
            best_params=best_params,
            best_score=-result.fun if objective != 'max_drawdown' else result.fun,
            optimization_history=result.func_vals,
            convergence_plot=result
        )

# 使用示例
optimizer = BayesianOptimizer(MovingAverageCrossover, backtest_engine)

result = optimizer.optimize(
    param_space={
        'fast_period': (5, 30),
        'slow_period': (20, 100),
        'stop_loss': (0.01, 0.05)
    },
    objective='sharpe_ratio',
    n_calls=50
)

print(f"最优参数: {result.best_params}")
print(f"最优夏普比率: {result.best_score}")
```

#### 2.2.3 Walk-Forward分析实现

```python
# hermesflow/optimization/walk_forward.py

import pandas as pd
from typing import List, Tuple
from datetime import datetime, timedelta

class WalkForwardAnalyzer:
    """Walk-Forward分析器"""
    
    def __init__(self, 
                 strategy_class,
                 backtest_engine,
                 optimizer):
        self.strategy_class = strategy_class
        self.backtest_engine = backtest_engine
        self.optimizer = optimizer
    
    def analyze(self,
                start_date: str,
                end_date: str,
                train_period_days: int = 252,  # 1年训练期
                test_period_days: int = 63,    # 3个月测试期
                param_space: dict,
                objective: str = 'sharpe_ratio'):
        """
        Walk-Forward分析
        
        步骤：
        1. 将时间序列分成多个训练期和测试期
        2. 在训练期优化参数
        3. 在测试期验证参数
        4. 滚动窗口，重复步骤2-3
        5. 汇总所有测试期结果
        
        Returns:
            WalkForwardResult对象
        """
        
        # 1. 分割时间窗口
        windows = self._create_windows(
            start_date, 
            end_date,
            train_period_days,
            test_period_days
        )
        
        # 2. 对每个窗口进行优化和测试
        results = []
        
        for i, (train_start, train_end, test_start, test_end) in enumerate(windows):
            print(f"\n=== Window {i+1}/{len(windows)} ===")
            print(f"Train: {train_start} to {train_end}")
            print(f"Test: {test_start} to {test_end}")
            
            # 2.1 在训练期优化参数
            opt_result = self.optimizer.optimize(
                start_date=train_start,
                end_date=train_end,
                param_space=param_space,
                objective=objective
            )
            
            best_params = opt_result.best_params
            print(f"最优参数: {best_params}")
            
            # 2.2 在测试期验证
            test_result = self.backtest_engine.run(
                strategy_class=self.strategy_class,
                params=best_params,
                start_date=test_start,
                end_date=test_end
            )
            
            results.append({
                'window': i + 1,
                'train_start': train_start,
                'train_end': train_end,
                'test_start': test_start,
                'test_end': test_end,
                'best_params': best_params,
                'train_sharpe': opt_result.best_score,
                'test_sharpe': test_result.sharpe_ratio,
                'test_return': test_result.total_return,
                'test_max_drawdown': test_result.max_drawdown,
                'degradation': opt_result.best_score - test_result.sharpe_ratio
            })
        
        # 3. 汇总结果
        return self._aggregate_results(results)
    
    def _create_windows(self, 
                       start_date: str,
                       end_date: str,
                       train_period_days: int,
                       test_period_days: int) -> List[Tuple]:
        """创建训练/测试窗口"""
        windows = []
        
        current_date = pd.Timestamp(start_date)
        end_timestamp = pd.Timestamp(end_date)
        
        while True:
            train_start = current_date
            train_end = current_date + timedelta(days=train_period_days)
            test_start = train_end + timedelta(days=1)
            test_end = test_start + timedelta(days=test_period_days)
            
            if test_end > end_timestamp:
                break
            
            windows.append((
                train_start.strftime('%Y-%m-%d'),
                train_end.strftime('%Y-%m-%d'),
                test_start.strftime('%Y-%m-%d'),
                test_end.strftime('%Y-%m-%d')
            ))
            
            # 滚动窗口
            current_date = test_end + timedelta(days=1)
        
        return windows
    
    def _aggregate_results(self, results: List[dict]):
        """汇总Walk-Forward结果"""
        df = pd.DataFrame(results)
        
        return WalkForwardResult(
            windows=df,
            avg_test_sharpe=df['test_sharpe'].mean(),
            avg_test_return=df['test_return'].mean(),
            avg_max_drawdown=df['test_max_drawdown'].mean(),
            avg_degradation=df['degradation'].mean(),
            consistency=(df['test_sharpe'] > 0).sum() / len(df),  # 胜率
            parameter_stability=self._calculate_param_stability(df['best_params'])
        )

# 使用示例
wf_analyzer = WalkForwardAnalyzer(
    strategy_class=MovingAverageCrossover,
    backtest_engine=backtest_engine,
    optimizer=BayesianOptimizer(...)
)

wf_result = wf_analyzer.analyze(
    start_date='2020-01-01',
    end_date='2024-12-01',
    train_period_days=252,  # 1年训练
    test_period_days=63,    # 3个月测试
    param_space={
        'fast_period': (5, 30),
        'slow_period': (20, 100)
    },
    objective='sharpe_ratio'
)

print(f"平均测试夏普比率: {wf_result.avg_test_sharpe}")
print(f"参数稳定性: {wf_result.parameter_stability}")
print(f"策略一致性: {wf_result.consistency * 100}%")
```

### 2.3 用户故事

```gherkin
Feature: 使用贝叶斯优化寻找最优参数
  作为一个策略开发者
  我想要使用贝叶斯优化算法
  以便更高效地找到最优参数

Scenario: 运行贝叶斯优化
  Given 我有一个MA交叉策略
  And 我设置参数空间：
    | 参数 | 最小值 | 最大值 |
    | fast_period | 5 | 30 |
    | slow_period | 20 | 100 |
    | stop_loss | 0.01 | 0.05 |
  When 我选择"贝叶斯优化"
  And 我设置优化目标为"夏普比率"
  And 我设置迭代次数为50次
  Then 系统应该在10分钟内完成优化
  And 系统应该返回最优参数组合
  And 优化效率应该比网格搜索快5-10倍
  And 我可以查看优化历史和收敛曲线

Scenario: 运行Walk-Forward分析
  Given 我有一个优化后的策略
  When 我运行Walk-Forward分析
  And 我设置训练期为1年
  And 我设置测试期为3个月
  Then 系统应该输出多个时间窗口的结果
  And 我应该看到参数稳定性评分
  And 我应该看到策略一致性评分
  And 我应该看到样本内外性能对比
  And 我可以判断策略是否过拟合
```

### 2.4 验收标准

- [ ] 实现贝叶斯优化（使用scikit-optimize）
- [ ] 实现Walk-Forward分析
- [ ] 优化速度：贝叶斯优化比网格搜索快5-10倍
- [ ] 支持多目标优化
- [ ] 优化历史可视化
- [ ] 参数稳定性分析
- [ ] API文档完整

### 2.5 工作量估算

| 任务 | 工作量 |
|------|--------|
| 贝叶斯优化实现 | 1周 |
| Walk-Forward分析 | 1周 |
| 遗传算法 | 0.5周 |
| 粒子群算法 | 0.5周 |
| 可视化和报告 | 0.5周 |
| 测试与文档 | 0.5周 |
| **总计** | **4周（1人月）** |

---

## 3. 模拟交易系统

### 3.1 需求背景

**问题**：
- ❌ 当前无Paper Trading功能
- ❌ 策略无法在实盘前验证
- ❌ 用户风险较大

**竞品对标**：
- ✅ 聚宽、掘金、QuantConnect都提供模拟交易
- ✅ 100%市场覆盖率

### 3.2 功能需求

#### 3.2.1 核心功能

1. **虚拟账户管理**
   - 初始资金设置
   - 资金变动追踪
   - 持仓管理

2. **实时数据订阅**
   - 使用真实市场数据
   - WebSocket实时推送
   - 与实盘数据源一致

3. **模拟订单撮合**
   - 市价单立即成交
   - 限价单按规则撮合
   - 模拟滑点和手续费

4. **完全兼容实盘API**
   - 同一套策略代码
   - 切换环境变量即可
   - 降低实盘迁移成本

#### 3.2.2 技术实现

```python
# hermesflow/paper_trading/broker.py

from typing import Dict, Optional
from decimal import Decimal
from dataclasses import dataclass
from enum import Enum

class OrderStatus(Enum):
    PENDING = "pending"
    FILLED = "filled"
    PARTIALLY_FILLED = "partially_filled"
    CANCELLED = "cancelled"
    REJECTED = "rejected"

@dataclass
class PaperOrder:
    """模拟订单"""
    order_id: str
    symbol: str
    side: str  # 'buy' or 'sell'
    type: str  # 'market' or 'limit'
    quantity: Decimal
    price: Optional[Decimal]
    filled_quantity: Decimal = Decimal('0')
    status: OrderStatus = OrderStatus.PENDING
    created_at: int = None
    filled_at: Optional[int] = None

class PaperTradingBroker:
    """模拟交易Broker"""
    
    def __init__(self, 
                 initial_cash: Decimal,
                 commission_rate: Decimal = Decimal('0.001'),
                 slippage_pct: Decimal = Decimal('0.001')):
        """
        初始化模拟Broker
        
        Args:
            initial_cash: 初始资金
            commission_rate: 手续费率（默认0.1%）
            slippage_pct: 滑点百分比（默认0.1%）
        """
        self.cash = initial_cash
        self.initial_cash = initial_cash
        self.commission_rate = commission_rate
        self.slippage_pct = slippage_pct
        
        self.positions: Dict[str, Decimal] = {}  # symbol -> quantity
        self.orders: Dict[str, PaperOrder] = {}  # order_id -> order
        self.trades: List[Dict] = []
        
        self.market_data_feed = None
    
    def submit_order(self, 
                    symbol: str,
                    side: str,
                    order_type: str,
                    quantity: Decimal,
                    price: Optional[Decimal] = None) -> str:
        """
        提交订单
        
        Returns:
            order_id
        """
        order_id = f"paper_{int(time.time() * 1000)}"
        
        order = PaperOrder(
            order_id=order_id,
            symbol=symbol,
            side=side,
            type=order_type,
            quantity=quantity,
            price=price,
            created_at=int(time.time() * 1000)
        )
        
        self.orders[order_id] = order
        
        # 如果是市价单，立即尝试成交
        if order_type == 'market':
            self._fill_market_order(order)
        
        return order_id
    
    def _fill_market_order(self, order: PaperOrder):
        """撮合市价单"""
        # 获取当前市场价格
        current_price = self._get_current_price(order.symbol)
        
        if current_price is None:
            order.status = OrderStatus.REJECTED
            return
        
        # 应用滑点
        if order.side == 'buy':
            fill_price = current_price * (1 + self.slippage_pct)
        else:
            fill_price = current_price * (1 - self.slippage_pct)
        
        # 计算手续费
        commission = fill_price * order.quantity * self.commission_rate
        
        # 检查资金是否足够（买入时）
        if order.side == 'buy':
            total_cost = fill_price * order.quantity + commission
            if self.cash < total_cost:
                order.status = OrderStatus.REJECTED
                return
        
        # 检查持仓是否足够（卖出时）
        if order.side == 'sell':
            current_position = self.positions.get(order.symbol, Decimal('0'))
            if current_position < order.quantity:
                order.status = OrderStatus.REJECTED
                return
        
        # 成交
        order.filled_quantity = order.quantity
        order.status = OrderStatus.FILLED
        order.filled_at = int(time.time() * 1000)
        
        # 更新持仓和资金
        if order.side == 'buy':
            self.cash -= (fill_price * order.quantity + commission)
            self.positions[order.symbol] = (
                self.positions.get(order.symbol, Decimal('0')) + order.quantity
            )
        else:
            self.cash += (fill_price * order.quantity - commission)
            self.positions[order.symbol] -= order.quantity
        
        # 记录成交
        self.trades.append({
            'order_id': order.order_id,
            'symbol': order.symbol,
            'side': order.side,
            'quantity': order.quantity,
            'price': fill_price,
            'commission': commission,
            'timestamp': order.filled_at
        })
    
    def _get_current_price(self, symbol: str) -> Optional[Decimal]:
        """获取当前市场价格"""
        if self.market_data_feed is None:
            return None
        
        # 从实时数据源获取最新价格
        tick = self.market_data_feed.get_latest_tick(symbol)
        if tick is None:
            return None
        
        return Decimal(str(tick.price))
    
    def get_position(self, symbol: str) -> Decimal:
        """获取持仓"""
        return self.positions.get(symbol, Decimal('0'))
    
    def get_portfolio_value(self) -> Decimal:
        """获取组合总值"""
        total = self.cash
        
        for symbol, quantity in self.positions.items():
            current_price = self._get_current_price(symbol)
            if current_price:
                total += current_price * quantity
        
        return total
    
    def get_returns(self) -> Decimal:
        """获取收益率"""
        current_value = self.get_portfolio_value()
        return (current_value - self.initial_cash) / self.initial_cash

# 使用示例
paper_broker = PaperTradingBroker(
    initial_cash=Decimal('10000'),
    commission_rate=Decimal('0.001'),  # 0.1%
    slippage_pct=Decimal('0.001')  # 0.1%
)

# 连接实时数据源
paper_broker.market_data_feed = real_time_data_feed

# 提交订单（与实盘API完全一致）
order_id = paper_broker.submit_order(
    symbol='BTCUSDT',
    side='buy',
    order_type='market',
    quantity=Decimal('0.1')
)

# 查看持仓
position = paper_broker.get_position('BTCUSDT')
print(f"持仓: {position}")

# 查看收益
returns = paper_broker.get_returns()
print(f"收益率: {returns * 100}%")
```

### 3.3 用户故事

```gherkin
Feature: 模拟交易验证策略
  作为一个策略开发者
  我想要在模拟环境中运行策略
  以便在实盘前验证策略表现

Scenario: 启动模拟交易
  Given 我有一个经过回测的策略
  When 我点击"启动模拟交易"
  And 我设置初始资金为10000 USDT
  And 我设置手续费率为0.1%
  And 我设置滑点为0.1%
  Then 系统应该创建虚拟账户
  And 系统应该使用实时市场数据
  And 策略应该开始运行
  And 我可以在仪表盘看到实时更新

Scenario: 切换到实盘交易
  Given 我的策略已在模拟环境运行1个月
  And 模拟交易表现符合预期
  When 我切换环境变量为"production"
  And 我重启策略
  Then 策略应该连接到实盘Broker
  And 策略代码无需任何修改
  And 订单应该发送到真实交易所
```

### 3.4 验收标准

- [ ] 虚拟账户管理（资金、持仓）
- [ ] 使用实时市场数据
- [ ] 模拟订单撮合（市价单、限价单）
- [ ] 模拟滑点和手续费
- [ ] API与实盘完全一致
- [ ] 性能报表（收益、夏普、回撤）
- [ ] 一键切换到实盘

### 3.5 工作量估算

| 任务 | 工作量 |
|------|--------|
| 虚拟Broker实现 | 1周 |
| 订单撮合逻辑 | 0.5周 |
| 实时数据集成 | 0.5周 |
| 报表和仪表盘 | 1周 |
| 测试 | 0.5周 |
| **总计** | **3.5周（0.9人月）** |

---

## 4. 机器学习集成

### 4.1 需求背景（P1 - V2版本）

**问题**：
- ❌ 当前无ML/DL集成
- ❌ 无法使用前沿AI策略
- ❌ 竞争力下降

**竞品对标**：
- ✅ Qlib：完整ML框架
- ✅ QuantConnect：支持TensorFlow/PyTorch
- ⚠️ 聚宽：简单ML支持

### 4.2 功能需求（V2）

#### 4.2.1 ML Pipeline

1. **特征工程**
   - 因子提取
   - 特征标准化
   - 特征选择

2. **模型训练**
   - 随机森林
   - XGBoost/LightGBM
   - LSTM/GRU
   - Transformer

3. **模型评估**
   - IC（信息系数）
   - Precision/Recall
   - Backtesting

4. **在线预测**
   - 实时特征生成
   - 模型推理
   - 信号生成

### 4.3 技术实现（简要）

```python
# hermesflow/ml/pipeline.py

from sklearn.ensemble import RandomForestClassifier
from sklearn.model_selection import TimeSeriesSplit
import pandas as pd

class MLStrategyPipeline:
    """ML策略Pipeline"""
    
    def __init__(self, factor_library):
        self.factor_library = factor_library
        self.model = None
    
    def prepare_features(self, data: pd.DataFrame) -> pd.DataFrame:
        """准备特征"""
        # 计算所有因子
        factors = self.factor_library.calculate_factors(
            data,
            factor_names=['RSI', 'MACD', 'ATR', 'OBV', 'ROE', 'PE']
        )
        return factors
    
    def train(self, data: pd.DataFrame, target: pd.Series):
        """训练模型"""
        X = self.prepare_features(data)
        
        # 使用时间序列交叉验证
        tscv = TimeSeriesSplit(n_splits=5)
        
        self.model = RandomForestClassifier(n_estimators=100)
        self.model.fit(X, target)
    
    def predict(self, data: pd.DataFrame) -> pd.Series:
        """预测"""
        X = self.prepare_features(data)
        return self.model.predict(X)

# 使用示例
ml_strategy = MLStrategyPipeline(factor_library)

# 训练
ml_strategy.train(historical_data, target_returns > 0)

# 预测
signals = ml_strategy.predict(current_data)
```

### 4.4 工作量估算（V2阶段）

| 任务 | 工作量 |
|------|--------|
| ML Pipeline框架 | 2周 |
| 特征工程 | 2周 |
| 模型库集成 | 2周 |
| 在线预测 | 1周 |
| AutoML | 2周 |
| 测试与文档 | 1周 |
| **总计** | **10周（2.5人月）** |

---

## 5. 组合管理系统

### 5.1 需求背景（P1 - V2版本）

**问题**：
- ❌ 只支持单策略运行
- ❌ 无法分散风险
- ❌ 无法优化资金配置

### 5.2 功能需求（V2）

1. **多策略并行执行**
2. **动态资金分配**
3. **组合风险分析**
4. **相关性分析**
5. **Markowitz组合优化**

### 5.3 工作量估算（V2阶段）

| 任务 | 工作量 |
|------|--------|
| 多策略框架 | 2周 |
| 资金分配 | 1周 |
| 组合优化 | 2周 |
| 风险分析 | 1周 |
| 测试与文档 | 1周 |
| **总计** | **7周（1.75人月）** |

---

## 6. 调整后的开发路线图

### 6.1 MVP阶段（3个月）

**目标**：可盈利的基础平台

**必须完成（P0）**：

1. ✅ Rust数据层（已完成）
2. ✅ 基础策略框架（已完成）
3. ✅ 基础回测引擎（已完成）
4. ✅ 交易执行模块（已完成）
5. ✅ 基础风控（已完成）
6. **Alpha因子库**（100个因子）- 新增 ⭐
7. **策略优化引擎**（贝叶斯+Walk-Forward）- 增强 ⭐
8. **模拟交易系统** - 新增 ⭐
9. **高频套利策略包**（10个模板）- 新增 ⭐

**总工作量**：~5.5人月
**预期成果**：
- 可以进行高频套利交易
- 可以进行趋势跟踪交易
- 具备完整的策略开发-回测-模拟-实盘流程
- 预期年化收益：15-30%

### 6.2 V2阶段（6个月）

**目标**：增强盈利能力

**应该完成（P1）**：

1. **机器学习集成** ⭐
2. **组合管理系统** ⭐
3. **因子库扩展**（200+因子）
4. **高级回测功能**
5. **多因子策略模板**（20个）
6. **期权策略支持**（初步）

**总工作量**：~10人月
**预期成果**：
- 支持多因子选股策略
- 支持AI驱动策略
- 支持多策略组合运行
- 预期年化收益：20-40%

### 6.3 V3阶段（12个月）

**目标**：完整平台

**可以完成（P2）**：

1. 社区策略市场
2. 高级可视化
3. 完整期权策略支持
4. 跨链DEX支持扩展
5. 移动端APP

---

## 7. PRD文档修改清单

### 7.1 需要删除的内容

❌ **低优先级功能（移至V3或删除）**：

1. 社区策略市场（P2）
2. 高级可视化（P2）
3. 移动端支持（P2）
4. 策略分享功能（P2）
5. 完整的链上清算保护（P2，简化版保留）

### 7.2 需要补充的章节

✅ **在主PRD中新增章节**：

1. **第3.2.4章**："Alpha因子库"
2. **第3.2.5章**："策略优化引擎"
3. **第3.2.6章**："模拟交易系统"
4. **第3.2.7章**："机器学习集成路线图"（V2）
5. **第3.5章**："组合管理系统"（V2）

### 7.3 需要修改的章节

📝 **更新现有章节**：

1. **第5.1章"MVP功能范围"**：
   - 补充因子库、优化器、模拟交易
   
2. **第5.3章"开发路线图"**：
   - 重新规划MVP/V2/V3里程碑
   - 明确工作量和交付时间
   
3. **第5.2章"优先级矩阵"**：
   - 调整功能优先级
   - P0: MVP必须，P1: V2应该，P2: V3可选

---

## 8. 总结

### 8.1 关键改进

1. **补充Alpha因子库**（100个 -> 200+个因子）
2. **增强策略优化**（网格 -> 6种算法）
3. **新增模拟交易**（0 -> 完整Paper Trading）
4. **规划ML集成**（V2路线图）
5. **调整优先级**（聚焦MVP盈利能力）

### 8.2 预期效果

**MVP阶段（3个月后）**：
- ✅ 可以开始实盘交易
- ✅ 预期年化收益15-30%
- ✅ 高频套利 + 趋势跟踪

**V2阶段（9个月后）**：
- ✅ 支持多因子和AI策略
- ✅ 预期年化收益20-40%
- ✅ 多策略组合运行

### 8.3 下一步行动

1. ✅ 审核本补充文档
2. 📝 更新主PRD文档
3. 📝 更新策略模块详细需求
4. 🚀 启动MVP开发
5. 📊 制定详细开发计划

---

**文档维护者**: PM Team  
**审核状态**: ⚠️ 待审核  
**下一步**: 更新主PRD文档

