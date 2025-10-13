# Python 开发者完整指南

> **HermesFlow 策略引擎 - Python 开发指南** | **适用于**: 策略引擎模块

---

## 🎯 本指南目标

帮助 Python 开发者：
1. ✅ 快速上手 HermesFlow 策略引擎开发
2. ✅ 掌握 FastAPI + NumPy/Pandas 最佳实践
3. ✅ 理解量化策略开发流程
4. ✅ 高效调试和优化代码

---

## 📚 必读文档

- 📋 [策略模块 PRD](../prd/modules/02-strategy-module.md) - 策略引擎需求
- 🏗️ [系统架构 - Python 策略引擎](../architecture/system-architecture.md#43-python-策略引擎)
- 📜 [ADR-007: Alpha 因子库](../architecture/decisions/ADR-007-alpha-factor-library.md)
- 📝 [编码规范 - Python 部分](../development/coding-standards.md#python-规范)

---

## 🚀 快速开始

### 环境搭建（20分钟）

#### 1. 安装 Python 3.12

```bash
# macOS
brew install python@3.12

# Linux (使用 pyenv)
pyenv install 3.12.0
pyenv global 3.12.0

# 验证
python3 --version  # 应为 3.12+
```

#### 2. 安装 Poetry

```bash
# 安装 Poetry
curl -sSL https://install.python-poetry.org | python3 -

# 验证
poetry --version
```

#### 3. IDE 配置

**VS Code（推荐）**:

```json
// .vscode/settings.json
{
  // Python 配置
  "python.defaultInterpreterPath": "${workspaceFolder}/.venv/bin/python",
  
  // Linting
  "python.linting.enabled": true,
  "python.linting.pylintEnabled": true,
  "python.linting.pylintArgs": [
    "--max-line-length=120",
    "--disable=C0111"  // Missing docstring
  ],
  
  // Formatting
  "python.formatting.provider": "black",
  "editor.formatOnSave": true,
  "[python]": {
    "editor.defaultFormatter": "ms-python.black-formatter",
    "editor.codeActionsOnSave": {
      "source.organizeImports": true
    }
  },
  
  // Testing
  "python.testing.pytestEnabled": true,
  "python.testing.pytestArgs": [
    "tests",
    "-v"
  ],
  
  // Type Checking
  "python.analysis.typeCheckingMode": "basic"
}
```

**推荐插件**:
- **Python** (Microsoft, 必装)
- **Pylance** (类型检查)
- **Black Formatter**
- **isort** (导入排序)

#### 4. 克隆和构建

```bash
# 克隆代码
git clone <your-repo-url>/HermesFlow.git
cd HermesFlow/modules/strategy-engine

# 安装依赖
poetry install

# 激活虚拟环境
poetry shell

# 运行测试
pytest

# 启动服务
python main.py
```

---

## 📁 项目结构

```
modules/strategy-engine/
├── pyproject.toml           # Poetry 配置
├── poetry.lock              # 依赖锁定
├── main.py                  # FastAPI 入口
├── src/
│   ├── __init__.py
│   │
│   ├── api/                 # FastAPI 路由
│   │   ├── __init__.py
│   │   ├── strategies.py    # 策略 CRUD
│   │   ├── backtest.py      # 回测 API
│   │   └── factors.py       # 因子 API
│   │
│   ├── strategies/          # 策略实现
│   │   ├── __init__.py
│   │   ├── base.py          # 基础策略类
│   │   ├── ma_cross.py      # 均线交叉策略
│   │   └── rsi_mean_reversion.py
│   │
│   ├── backtest/            # 回测引擎
│   │   ├── __init__.py
│   │   ├── engine.py        # 回测引擎
│   │   ├── broker.py        # 模拟券商
│   │   └── metrics.py       # 性能指标
│   │
│   ├── factors/             # Alpha 因子库
│   │   ├── __init__.py
│   │   ├── technical.py     # 技术指标（MA, RSI, MACD）
│   │   ├── fundamental.py   # 基本面因子
│   │   └── sentiment.py     # 情绪因子
│   │
│   ├── optimizers/          # 策略优化
│   │   ├── __init__.py
│   │   ├── grid_search.py   # 网格搜索
│   │   └── bayesian.py      # 贝叶斯优化
│   │
│   ├── models/              # 数据模型
│   │   ├── __init__.py
│   │   ├── strategy.py      # Strategy Pydantic Model
│   │   └── backtest.py      # Backtest Result Model
│   │
│   ├── database/            # 数据库
│   │   ├── __init__.py
│   │   └── postgres.py      # PostgreSQL 连接
│   │
│   └── utils/               # 工具函数
│       ├── __init__.py
│       ├── logger.py        # 日志
│       └── validator.py     # 数据验证
│
├── tests/                   # 测试
│   ├── __init__.py
│   ├── conftest.py          # pytest fixtures
│   ├── test_strategies.py
│   ├── test_backtest.py
│   └── test_factors.py
│
└── notebooks/               # Jupyter Notebooks
    └── strategy_research.ipynb
```

---

## 🔧 核心技术栈

### FastAPI

#### 基本应用结构

```python
# main.py
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from contextlib import asynccontextmanager

from src.api import strategies, backtest, factors
from src.database import init_db, close_db

@asynccontextmanager
async def lifespan(app: FastAPI):
    # Startup
    await init_db()
    yield
    # Shutdown
    await close_db()

app = FastAPI(
    title="HermesFlow Strategy Engine",
    version="1.0.0",
    lifespan=lifespan
)

# CORS
app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:3000"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# 路由
app.include_router(strategies.router, prefix="/api/v1/strategies", tags=["strategies"])
app.include_router(backtest.router, prefix="/api/v1/backtest", tags=["backtest"])
app.include_router(factors.router, prefix="/api/v1/factors", tags=["factors"])

@app.get("/health")
async def health_check():
    return {"status": "healthy"}

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8082)
```

#### API 路由

```python
# src/api/strategies.py
from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.ext.asyncio import AsyncSession
from typing import List

from src.models.strategy import StrategyCreate, StrategyResponse
from src.database import get_db
from src.services.strategy_service import StrategyService

router = APIRouter()

@router.post("/", response_model=StrategyResponse, status_code=201)
async def create_strategy(
    strategy: StrategyCreate,
    db: AsyncSession = Depends(get_db)
):
    """创建策略"""
    service = StrategyService(db)
    return await service.create(strategy)

@router.get("/", response_model=List[StrategyResponse])
async def list_strategies(
    skip: int = 0,
    limit: int = 100,
    db: AsyncSession = Depends(get_db)
):
    """获取策略列表"""
    service = StrategyService(db)
    return await service.list(skip=skip, limit=limit)

@router.get("/{strategy_id}", response_model=StrategyResponse)
async def get_strategy(
    strategy_id: int,
    db: AsyncSession = Depends(get_db)
):
    """获取单个策略"""
    service = StrategyService(db)
    strategy = await service.get(strategy_id)
    if not strategy:
        raise HTTPException(status_code=404, detail="Strategy not found")
    return strategy
```

#### Pydantic Models

```python
# src/models/strategy.py
from pydantic import BaseModel, Field, validator
from typing import Optional, Dict, Any
from datetime import datetime

class StrategyBase(BaseModel):
    name: str = Field(..., min_length=1, max_length=100)
    description: Optional[str] = None
    code: str = Field(..., min_length=1)
    parameters: Dict[str, Any] = Field(default_factory=dict)

class StrategyCreate(StrategyBase):
    @validator('code')
    def validate_code(cls, v):
        # 基本的代码验证
        if 'import os' in v or 'import sys' in v:
            raise ValueError("Forbidden imports detected")
        return v

class StrategyResponse(StrategyBase):
    id: int
    user_id: int
    tenant_id: str
    created_at: datetime
    updated_at: datetime
    
    class Config:
        from_attributes = True
```

---

### NumPy/Pandas

#### 向量化计算

```python
import numpy as np
import pandas as pd

def calculate_sma(prices: np.ndarray, period: int) -> np.ndarray:
    """计算简单移动平均（向量化）"""
    return np.convolve(prices, np.ones(period) / period, mode='valid')

def calculate_rsi(prices: pd.Series, period: int = 14) -> pd.Series:
    """计算 RSI（向量化）"""
    delta = prices.diff()
    
    gain = delta.where(delta > 0, 0)
    loss = -delta.where(delta < 0, 0)
    
    avg_gain = gain.rolling(window=period).mean()
    avg_loss = loss.rolling(window=period).mean()
    
    rs = avg_gain / avg_loss
    rsi = 100 - (100 / (1 + rs))
    
    return rsi

def calculate_macd(prices: pd.Series, 
                   fast: int = 12, 
                   slow: int = 26, 
                   signal: int = 9) -> pd.DataFrame:
    """计算 MACD（向量化）"""
    ema_fast = prices.ewm(span=fast).mean()
    ema_slow = prices.ewm(span=slow).mean()
    
    macd = ema_fast - ema_slow
    signal_line = macd.ewm(span=signal).mean()
    histogram = macd - signal_line
    
    return pd.DataFrame({
        'macd': macd,
        'signal': signal_line,
        'histogram': histogram
    })
```

---

## 🎨 策略开发

### 基础策略类

```python
# src/strategies/base.py
from abc import ABC, abstractmethod
from typing import Dict, Any
import pandas as pd

class BaseStrategy(ABC):
    """策略基类"""
    
    def __init__(self, parameters: Dict[str, Any]):
        self.parameters = parameters
        self.positions = {}
    
    @abstractmethod
    def on_bar(self, bar: pd.Series) -> Dict[str, Any]:
        """
        每根K线触发
        
        Args:
            bar: K线数据（包含 open, high, low, close, volume）
        
        Returns:
            信号字典: {'action': 'buy'/'sell'/'hold', 'quantity': float}
        """
        pass
    
    @abstractmethod
    def on_trade(self, trade: Dict[str, Any]):
        """成交回调"""
        pass
    
    def reset(self):
        """重置策略状态"""
        self.positions = {}
```

### 示例策略

```python
# src/strategies/ma_cross.py
from src.strategies.base import BaseStrategy
from src.factors.technical import calculate_sma
import pandas as pd

class MACrossStrategy(BaseStrategy):
    """均线交叉策略"""
    
    def __init__(self, parameters: Dict[str, Any]):
        super().__init__(parameters)
        self.fast_period = parameters.get('fast_period', 20)
        self.slow_period = parameters.get('slow_period', 50)
        self.price_history = []
    
    def on_bar(self, bar: pd.Series) -> Dict[str, Any]:
        # 更新价格历史
        self.price_history.append(bar['close'])
        
        # 需要足够的数据计算慢均线
        if len(self.price_history) < self.slow_period:
            return {'action': 'hold', 'quantity': 0}
        
        # 计算均线
        prices = np.array(self.price_history)
        fast_ma = calculate_sma(prices, self.fast_period)[-1]
        slow_ma = calculate_sma(prices, self.slow_period)[-1]
        
        # 生成信号
        if fast_ma > slow_ma and not self.has_position():
            return {'action': 'buy', 'quantity': 1.0}
        elif fast_ma < slow_ma and self.has_position():
            return {'action': 'sell', 'quantity': 1.0}
        else:
            return {'action': 'hold', 'quantity': 0}
    
    def on_trade(self, trade: Dict[str, Any]):
        if trade['action'] == 'buy':
            self.positions[trade['symbol']] = trade['quantity']
        elif trade['action'] == 'sell':
            self.positions.pop(trade['symbol'], None)
    
    def has_position(self) -> bool:
        return len(self.positions) > 0
```

---

## 🧪 回测引擎

```python
# src/backtest/engine.py
from typing import List, Dict, Any
import pandas as pd
import numpy as np

from src.strategies.base import BaseStrategy
from src.backtest.broker import SimulatedBroker
from src.backtest.metrics import calculate_metrics

class BacktestEngine:
    """回测引擎"""
    
    def __init__(self, strategy: BaseStrategy, initial_capital: float = 100000.0):
        self.strategy = strategy
        self.broker = SimulatedBroker(initial_capital)
        self.trades = []
    
    def run(self, data: pd.DataFrame) -> Dict[str, Any]:
        """
        运行回测
        
        Args:
            data: 历史数据（包含 open, high, low, close, volume, timestamp）
        
        Returns:
            回测结果
        """
        # 重置策略和券商
        self.strategy.reset()
        self.broker.reset()
        
        # 逐根K线回测
        for idx, bar in data.iterrows():
            # 策略信号
            signal = self.strategy.on_bar(bar)
            
            # 执行订单
            if signal['action'] == 'buy':
                trade = self.broker.buy(
                    symbol=bar.get('symbol', 'BTC/USDT'),
                    quantity=signal['quantity'],
                    price=bar['close']
                )
                if trade:
                    self.trades.append(trade)
                    self.strategy.on_trade(trade)
            
            elif signal['action'] == 'sell':
                trade = self.broker.sell(
                    symbol=bar.get('symbol', 'BTC/USDT'),
                    quantity=signal['quantity'],
                    price=bar['close']
                )
                if trade:
                    self.trades.append(trade)
                    self.strategy.on_trade(trade)
            
            # 更新账户价值
            self.broker.update_portfolio_value(bar['close'])
        
        # 计算指标
        equity_curve = self.broker.get_equity_curve()
        metrics = calculate_metrics(equity_curve, self.trades)
        
        return {
            'trades': self.trades,
            'equity_curve': equity_curve,
            'metrics': metrics
        }

# src/backtest/metrics.py
def calculate_metrics(equity_curve: List[float], trades: List[Dict]) -> Dict[str, float]:
    """计算性能指标"""
    returns = np.diff(equity_curve) / equity_curve[:-1]
    
    # 基本指标
    total_return = (equity_curve[-1] - equity_curve[0]) / equity_curve[0]
    sharpe_ratio = np.mean(returns) / np.std(returns) * np.sqrt(252) if np.std(returns) > 0 else 0
    max_drawdown = calculate_max_drawdown(equity_curve)
    
    # 交易统计
    win_trades = [t for t in trades if t.get('pnl', 0) > 0]
    win_rate = len(win_trades) / len(trades) if trades else 0
    
    return {
        'total_return': total_return,
        'sharpe_ratio': sharpe_ratio,
        'max_drawdown': max_drawdown,
        'win_rate': win_rate,
        'total_trades': len(trades)
    }

def calculate_max_drawdown(equity_curve: List[float]) -> float:
    """计算最大回撤"""
    equity = np.array(equity_curve)
    cummax = np.maximum.accumulate(equity)
    drawdown = (equity - cummax) / cummax
    return np.min(drawdown)
```

---

## 🧪 测试

### 单元测试

```python
# tests/test_strategies.py
import pytest
import pandas as pd
from src.strategies.ma_cross import MACrossStrategy

def test_ma_cross_buy_signal():
    """测试买入信号"""
    strategy = MACrossStrategy({'fast_period': 5, 'slow_period': 10})
    
    # 模拟价格上涨趋势
    for price in range(100, 150):
        bar = pd.Series({'close': price})
        signal = strategy.on_bar(bar)
    
    # 应该生成买入信号
    assert signal['action'] == 'buy'
    assert signal['quantity'] > 0

def test_ma_cross_sell_signal():
    """测试卖出信号"""
    strategy = MACrossStrategy({'fast_period': 5, 'slow_period': 10})
    
    # 先建立仓位
    for price in range(100, 150):
        bar = pd.Series({'close': price})
        strategy.on_bar(bar)
    
    strategy.positions['BTC/USDT'] = 1.0
    
    # 模拟价格下跌趋势
    for price in range(150, 100, -1):
        bar = pd.Series({'close': price})
        signal = strategy.on_bar(bar)
    
    # 应该生成卖出信号
    assert signal['action'] == 'sell'
```

### 异步测试

```python
# tests/test_api.py
import pytest
from httpx import AsyncClient
from main import app

@pytest.mark.asyncio
async def test_create_strategy():
    """测试创建策略"""
    async with AsyncClient(app=app, base_url="http://test") as client:
        response = await client.post(
            "/api/v1/strategies",
            json={
                "name": "Test Strategy",
                "code": "def on_bar(bar): return {'action': 'hold', 'quantity': 0}",
                "parameters": {}
            }
        )
        
        assert response.status_code == 201
        data = response.json()
        assert data['name'] == "Test Strategy"
```

**覆盖率目标**: ≥ 75%

---

## ⚡ 性能优化

### 1. 使用 NumPy 向量化

```python
# ❌ 不好: 使用循环
def calculate_returns_slow(prices):
    returns = []
    for i in range(1, len(prices)):
        ret = (prices[i] - prices[i-1]) / prices[i-1]
        returns.append(ret)
    return returns

# ✅ 好: 使用 NumPy
def calculate_returns_fast(prices):
    return np.diff(prices) / prices[:-1]
```

### 2. 避免 DataFrame 逐行迭代

```python
# ❌ 不好
for index, row in df.iterrows():
    result = calculate(row['value'])

# ✅ 好: 使用 apply
result = df['value'].apply(calculate)

# ✅ 更好: 向量化
result = calculate_vectorized(df['value'].values)
```

### 3. 使用 Numba JIT

```python
from numba import jit

@jit(nopython=True)
def calculate_indicators_fast(prices, period):
    """使用 Numba 加速计算"""
    n = len(prices)
    result = np.zeros(n)
    
    for i in range(period, n):
        result[i] = np.mean(prices[i-period:i])
    
    return result
```

---

## 📚 推荐资源

- [FastAPI 文档](https://fastapi.tiangolo.com/)
- [Pandas 文档](https://pandas.pydata.org/docs/)
- [NumPy 文档](https://numpy.org/doc/)
- [量化交易教程](https://www.quantstart.com/)

---

## 📞 获取帮助

- **Python Team**: Slack `#python-dev`
- **技术问题**: [FAQ](../FAQ.md)

---

**最后更新**: 2025-01-13  
**维护者**: @architect.mdc  
**版本**: v1.0

