"""
RSI超买超卖策略单元测试
"""
from datetime import datetime, timedelta
from decimal import Decimal

import pytest

from app.models.market_data import Kline
from app.models.strategy import SignalType
from app.strategies.rsi_strategy import RSIStrategy


@pytest.fixture
def strategy():
    """创建策略实例"""
    return RSIStrategy(
        exchange="binance",
        symbol="BTC/USDT",
        period=14,
        overbought=70,
        oversold=30,
        neutral=50,
        interval="1m"
    )


@pytest.fixture
def klines():
    """创建测试用K线数据"""
    base_time = datetime(2024, 1, 1)
    klines = []
    
    # 创建下跌趋势，RSI进入超卖区域
    price = 40000
    for i in range(30):
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
        
    # 创建上涨趋势，RSI回归中性区域
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
        
    # 创建强势上涨，RSI进入超买区域
    for i in range(45, 60):
        klines.append({
            "exchange": "binance",
            "symbol": "BTC/USDT",
            "interval": "1m",
            "timestamp": base_time + timedelta(minutes=i),
            "open": price,
            "high": price + 300,
            "low": price + 100,
            "close": price + 250,
            "volume": 30,
            "turnover": (price + 250) * 30
        })
        price += 250
        
    # 创建盘整趋势，RSI回归中性区域
    for i in range(60, 75):
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
    assert isinstance(strategy.last_rsi, float)
    assert strategy.rsi is not None


@pytest.mark.asyncio
async def test_oversold_signal(strategy, klines):
    """测试超卖信号"""
    # 模拟加载历史数据
    strategy.load_klines = lambda interval, limit: klines[:limit]
    await strategy.initialize()
    
    # 设置初始状态为空仓，RSI在超卖阈值以上
    strategy.position = 0
    strategy.last_rsi = 35
    
    # 更新数据直到RSI跌破超卖阈值
    signal = None
    for kline in klines[:30]:  # 使用下跌趋势数据
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
    assert "rsi" in signal.parameters
    assert "threshold" in signal.parameters
    assert signal.parameters["threshold"] == strategy.oversold
    
    # 验证状态更新
    assert strategy.position == 1
    assert strategy.last_rsi < strategy.oversold


@pytest.mark.asyncio
async def test_overbought_signal(strategy, klines):
    """测试超买信号"""
    # 模拟加载历史数据
    strategy.load_klines = lambda interval, limit: klines[:limit]
    await strategy.initialize()
    
    # 设置初始状态为空仓，RSI在超买阈值以下
    strategy.position = 0
    strategy.last_rsi = 65
    
    # 更新数据直到RSI突破超买阈值
    signal = None
    for kline in klines[45:60]:  # 使用强势上涨数据
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
    assert "rsi" in signal.parameters
    assert "threshold" in signal.parameters
    assert signal.parameters["threshold"] == strategy.overbought
    
    # 验证状态更新
    assert strategy.position == -1
    assert strategy.last_rsi > strategy.overbought


@pytest.mark.asyncio
async def test_neutral_signal(strategy, klines):
    """测试中性区域信号"""
    # 模拟加载历史数据
    strategy.load_klines = lambda interval, limit: klines[:limit]
    await strategy.initialize()
    
    # 设置初始状态为多头，RSI远离中性区域
    strategy.position = 1
    strategy.last_rsi = 65
    
    # 更新数据直到RSI回归中性区域
    signal = None
    for kline in klines[60:75]:  # 使用盘整趋势数据
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
    assert "rsi" in signal.parameters
    assert "threshold" in signal.parameters
    assert signal.parameters["threshold"] == strategy.neutral
    
    # 验证状态更新
    assert strategy.position == 0
    assert abs(strategy.last_rsi - strategy.neutral) < 1


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