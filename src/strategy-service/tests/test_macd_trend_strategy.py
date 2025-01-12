"""
MACD趋势策略单元测试
"""
from datetime import datetime, timedelta
from decimal import Decimal

import pytest

from app.models.market_data import Kline
from app.models.strategy import SignalType
from app.strategies.macd_trend_strategy import MACDTrendStrategy


@pytest.fixture
def strategy():
    """创建策略实例"""
    return MACDTrendStrategy(
        exchange="binance",
        symbol="BTC/USDT",
        fast_period=12,
        slow_period=26,
        signal_period=9,
        interval="1m"
    )


@pytest.fixture
def klines():
    """创建测试用K线数据"""
    base_time = datetime(2024, 1, 1)
    klines = []
    
    # 创建震荡趋势
    price = 40000
    for i in range(30):
        klines.append({
            "exchange": "binance",
            "symbol": "BTC/USDT",
            "interval": "1m",
            "timestamp": base_time + timedelta(minutes=i),
            "open": price,
            "high": price + 100,
            "low": price - 100,
            "close": price + 50,
            "volume": 10,
            "turnover": (price + 50) * 10
        })
        price += 50
        
    # 创建上涨趋势
    for i in range(30, 45):
        klines.append({
            "exchange": "binance",
            "symbol": "BTC/USDT",
            "interval": "1m",
            "timestamp": base_time + timedelta(minutes=i),
            "open": price,
            "high": price + 200,
            "low": price - 50,
            "close": price + 150,
            "volume": 20,
            "turnover": (price + 150) * 20
        })
        price += 150
        
    # 创建盘整趋势
    for i in range(45, 60):
        klines.append({
            "exchange": "binance",
            "symbol": "BTC/USDT",
            "interval": "1m",
            "timestamp": base_time + timedelta(minutes=i),
            "open": price,
            "high": price + 100,
            "low": price - 100,
            "close": price + random.randint(-50, 50),
            "volume": 15,
            "turnover": price * 15
        })
        
    # 创建下跌趋势
    for i in range(60, 75):
        klines.append({
            "exchange": "binance",
            "symbol": "BTC/USDT",
            "interval": "1m",
            "timestamp": base_time + timedelta(minutes=i),
            "open": price,
            "high": price + 50,
            "low": price - 200,
            "close": price - 150,
            "volume": 20,
            "turnover": (price - 150) * 20
        })
        price -= 150
        
    return klines


@pytest.mark.asyncio
async def test_strategy_initialization(strategy, klines):
    """测试策略初始化"""
    # 模拟加载历史数据
    strategy.load_klines = lambda interval, limit: klines[:limit]
    
    # 初始化策略
    await strategy.initialize()
    
    # 验证初始状态
    assert strategy.position == 0
    assert isinstance(strategy.last_hist, float)
    assert strategy.macd is not None


@pytest.mark.asyncio
async def test_golden_cross(strategy, klines):
    """测试金叉信号"""
    # 模拟加载历史数据
    strategy.load_klines = lambda interval, limit: klines[:limit]
    await strategy.initialize()
    
    # 设置初始状态为空仓，MACD柱状图为负
    strategy.position = 0
    strategy.last_hist = -0.1
    
    # 更新数据直到出现金叉
    signal = None
    for kline in klines[30:45]:  # 使用上涨趋势数据
        signal = await strategy.on_kline(kline)
        if signal:
            break
            
    # 验证信号
    assert signal is not None
    assert signal.signal_type == SignalType.LONG
    assert signal.exchange == "binance"
    assert signal.symbol == "BTC/USDT"
    assert isinstance(signal.price, Decimal)
    assert isinstance(signal.volume, Decimal)
    assert "macd" in signal.parameters
    assert "signal" in signal.parameters
    assert "hist" in signal.parameters
    
    # 验证状态更新
    assert strategy.position == 1
    assert strategy.last_hist > 0


@pytest.mark.asyncio
async def test_death_cross(strategy, klines):
    """测试死叉信号"""
    # 模拟加载历史数据
    strategy.load_klines = lambda interval, limit: klines[:limit]
    await strategy.initialize()
    
    # 设置初始状态为空仓，MACD柱状图为正
    strategy.position = 0
    strategy.last_hist = 0.1
    
    # 更新数据直到出现死叉
    signal = None
    for kline in klines[60:75]:  # 使用下跌趋势数据
        signal = await strategy.on_kline(kline)
        if signal:
            break
            
    # 验证信号
    assert signal is not None
    assert signal.signal_type == SignalType.SHORT
    assert signal.exchange == "binance"
    assert signal.symbol == "BTC/USDT"
    assert isinstance(signal.price, Decimal)
    assert isinstance(signal.volume, Decimal)
    assert "macd" in signal.parameters
    assert "signal" in signal.parameters
    assert "hist" in signal.parameters
    
    # 验证状态更新
    assert strategy.position == -1
    assert strategy.last_hist < 0


@pytest.mark.asyncio
async def test_histogram_reversal(strategy, klines):
    """测试柱状图反转信号"""
    # 模拟加载历史数据
    strategy.load_klines = lambda interval, limit: klines[:limit]
    await strategy.initialize()
    
    # 设置初始状态为多头，MACD柱状图为正
    strategy.position = 1
    strategy.last_hist = 0.1
    
    # 更新数据直到柱状图反转
    signal = None
    for kline in klines[45:60]:  # 使用盘整趋势数据
        signal = await strategy.on_kline(kline)
        if signal:
            break
            
    # 验证信号
    assert signal is not None
    assert signal.signal_type == SignalType.CLOSE_LONG
    assert signal.exchange == "binance"
    assert signal.symbol == "BTC/USDT"
    assert isinstance(signal.price, Decimal)
    assert isinstance(signal.volume, Decimal)
    assert "macd" in signal.parameters
    assert "signal" in signal.parameters
    assert "hist" in signal.parameters
    
    # 验证状态更新
    assert strategy.position == 0
    assert strategy.last_hist < 0


@pytest.mark.asyncio
async def test_no_signal_on_other_data(strategy):
    """测试其他数据不产生信号"""
    # 测试Ticker数据
    ticker = {
        "exchange": "binance",
        "symbol": "BTC/USDT",
        "timestamp": datetime.now(),
        "bid": 40000,
        "ask": 40001,
        "last": 40000,
        "volume": 100
    }
    signal = await strategy.on_ticker(ticker)
    assert signal is None
    
    # 测试订单簿数据
    orderbook = {
        "exchange": "binance",
        "symbol": "BTC/USDT",
        "timestamp": datetime.now(),
        "bids": [(40000, 1), (39999, 2)],
        "asks": [(40001, 1), (40002, 2)]
    }
    signal = await strategy.on_orderbook(orderbook)
    assert signal is None
    
    # 测试成交数据
    trade = {
        "exchange": "binance",
        "symbol": "BTC/USDT",
        "timestamp": datetime.now(),
        "price": 40000,
        "volume": 1,
        "side": "buy"
    }
    signal = await strategy.on_trade(trade)
    assert signal is None 