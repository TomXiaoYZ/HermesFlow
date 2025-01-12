"""
KDJ交叉策略单元测试
"""
from datetime import datetime, timedelta
from decimal import Decimal

import pytest

from app.models.market_data import Kline
from app.models.strategy import SignalType
from app.strategies.kdj_strategy import KDJStrategy


@pytest.fixture
def strategy():
    """创建策略实例"""
    return KDJStrategy(
        exchange="binance",
        symbol="BTC/USDT",
        k_period=9,
        d_period=3,
        j_period=3,
        overbought=80,
        oversold=20,
        neutral_high=60,
        neutral_low=40,
        interval="1m"
    )


@pytest.fixture
def klines():
    """创建测试用K线数据"""
    base_time = datetime(2024, 1, 1)
    klines = []
    
    # 创建下跌趋势，KDJ进入超卖区域
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
        
    # 创建反弹趋势，KDJ回归中性区域
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
        
    # 创建强势上涨，KDJ进入超买区域
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
        
    # 创建盘整趋势，KDJ在中性区域交叉
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
    assert isinstance(strategy.last_k, float)
    assert isinstance(strategy.last_d, float)
    assert isinstance(strategy.last_j, float)
    assert strategy.kdj is not None


@pytest.mark.asyncio
async def test_oversold_signal(strategy, klines):
    """测试超卖信号"""
    # 模拟加载历史数据
    strategy.load_klines = lambda interval, limit: klines[:limit]
    await strategy.initialize()
    
    # 设置初始状态为空仓，KDJ在超卖阈值以上
    strategy.position = 0
    strategy.last_k = 25
    strategy.last_d = 25
    strategy.last_j = 15
    
    # 更新数据直到KDJ产生超卖信号
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
    assert "k" in signal.parameters
    assert "d" in signal.parameters
    assert "j" in signal.parameters
    assert "threshold" in signal.parameters
    assert signal.parameters["threshold"] == strategy.oversold
    
    # 验证状态更新
    assert strategy.position == 1
    assert strategy.last_k > strategy.oversold
    assert strategy.last_d > strategy.oversold
    assert strategy.last_j < strategy.oversold


@pytest.mark.asyncio
async def test_overbought_signal(strategy, klines):
    """测试超买信号"""
    # 模拟加载历史数据
    strategy.load_klines = lambda interval, limit: klines[:limit]
    await strategy.initialize()
    
    # 设置初始状态为空仓，KDJ在超买阈值以下
    strategy.position = 0
    strategy.last_k = 75
    strategy.last_d = 75
    strategy.last_j = 85
    
    # 更新数据直到KDJ产生超买信号
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
    assert "k" in signal.parameters
    assert "d" in signal.parameters
    assert "j" in signal.parameters
    assert "threshold" in signal.parameters
    assert signal.parameters["threshold"] == strategy.overbought
    
    # 验证状态更新
    assert strategy.position == -1
    assert strategy.last_k < strategy.overbought
    assert strategy.last_d < strategy.overbought
    assert strategy.last_j > strategy.overbought


@pytest.mark.asyncio
async def test_neutral_cross_signal(strategy, klines):
    """测试中性区域交叉信号"""
    # 模拟加载历史数据
    strategy.load_klines = lambda interval, limit: klines[:limit]
    await strategy.initialize()
    
    # 设置初始状态为多头，KDJ在中性区域
    strategy.position = 1
    strategy.last_k = 45
    strategy.last_d = 50
    
    # 更新数据直到KDJ在中性区域交叉
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
    assert "k" in signal.parameters
    assert "d" in signal.parameters
    assert "j" in signal.parameters
    assert "cross" in signal.parameters
    assert signal.parameters["cross"] == "death"
    
    # 验证状态更新
    assert strategy.position == 0
    assert strategy.neutral_low <= strategy.last_k <= strategy.neutral_high
    assert strategy.neutral_low <= strategy.last_d <= strategy.neutral_high


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