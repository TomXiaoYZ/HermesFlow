"""
技术指标单元测试
"""
from datetime import datetime, timedelta
from decimal import Decimal
from typing import Dict, List, Optional, Union

import pytest
from pydantic import BaseModel

from app.indicators.base import BaseIndicator, IndicatorConfig, IndicatorValue
from app.indicators.momentum import (
    CCIIndicator,
    MFIIndicator,
    ROCIndicator,
    RSIIndicator,
    StochasticIndicator,
    WilliamsRIndicator,
)
from app.indicators.trend import (
    ADXIndicator,
    BollingerBandsIndicator,
    EMAIndicator,
    IchimokuIndicator,
    MACDIndicator,
    MAIndicator,
)
from app.indicators.volatility import (
    ATRIndicator,
    NATRIndicator,
    ParkinsonsVolatilityIndicator,
    STDDEVIndicator,
    TRUERANGEIndicator,
    VARIndicator,
)
from app.indicators.volume import (
    ADIndicator,
    ADOSCIndicator,
    CMFIndicator,
    EMVIndicator,
    OBVIndicator,
    VWAPIndicator,
)
from app.models.market_data import Kline


@pytest.fixture
def klines():
    """创建测试用K线数据"""
    return [
        Kline(
            exchange="binance",
            symbol="BTC/USDT",
            interval="1m",
            timestamp=datetime(2024, 1, 1) + timedelta(minutes=i),
            open=Decimal("40000"),
            high=Decimal("40100"),
            low=Decimal("39900"),
            close=Decimal("40050"),
            volume=Decimal("10"),
            turnover=Decimal("400500")
        )
        for i in range(100)
    ]


def test_indicator_config():
    """测试指标配置"""
    config = IndicatorConfig(window=14)
    assert config.window == 14
    assert config.source == "close"
    assert config.alpha == 0.0
    assert config.adjust is True


def test_indicator_value():
    """测试指标值"""
    value = IndicatorValue(
        timestamp=datetime(2024, 1, 1),
        values={"ma": 40000.0}
    )
    assert value.timestamp == datetime(2024, 1, 1)
    assert value.values == {"ma": 40000.0}


@pytest.mark.asyncio
async def test_ma_indicator(klines):
    """测试移动平均线"""
    indicator = MAIndicator(IndicatorConfig(window=5))
    indicator.update(klines)

    value = indicator.get_value()
    assert value is not None
    assert "ma" in value.values
    assert abs(value.values["ma"] - 40050.0) < 0.01

    values = indicator.get_values(
        start_time=datetime(2024, 1, 1),
        end_time=datetime(2024, 1, 1, 1)
    )
    assert len(values) > 0


@pytest.mark.asyncio
async def test_macd_indicator(klines):
    """测试MACD指标"""
    indicator = MACDIndicator(IndicatorConfig(window=26))
    indicator.update(klines)

    value = indicator.get_value()
    assert value is not None
    assert "macd" in value.values
    assert "signal" in value.values
    assert "hist" in value.values


@pytest.mark.asyncio
async def test_rsi_indicator(klines):
    """测试RSI指标"""
    indicator = RSIIndicator(IndicatorConfig(window=14))
    indicator.update(klines)

    value = indicator.get_value()
    assert value is not None
    assert "rsi" in value.values
    assert 0 <= value.values["rsi"] <= 100


@pytest.mark.asyncio
async def test_atr_indicator(klines):
    """测试ATR指标"""
    indicator = ATRIndicator(IndicatorConfig(window=14))
    indicator.update(klines)

    value = indicator.get_value()
    assert value is not None
    assert "atr" in value.values
    assert value.values["atr"] > 0


@pytest.mark.asyncio
async def test_obv_indicator(klines):
    """测试OBV指标"""
    indicator = OBVIndicator(IndicatorConfig(window=1))
    indicator.update(klines)

    value = indicator.get_value()
    assert value is not None
    assert "obv" in value.values


@pytest.mark.asyncio
async def test_bollinger_bands_indicator(klines):
    """测试布林带指标"""
    indicator = BollingerBandsIndicator(IndicatorConfig(window=20))
    indicator.update(klines)

    value = indicator.get_value()
    assert value is not None
    assert "upper" in value.values
    assert "middle" in value.values
    assert "lower" in value.values
    assert value.values["upper"] > value.values["middle"]
    assert value.values["middle"] > value.values["lower"]


@pytest.mark.asyncio
async def test_stochastic_indicator(klines):
    """测试KDJ指标"""
    indicator = StochasticIndicator(IndicatorConfig(window=9))
    indicator.update(klines)

    value = indicator.get_value()
    assert value is not None
    assert "k" in value.values
    assert "d" in value.values
    assert "j" in value.values
    assert 0 <= value.values["k"] <= 100
    assert 0 <= value.values["d"] <= 100


@pytest.mark.asyncio
async def test_vwap_indicator(klines):
    """测试VWAP指标"""
    indicator = VWAPIndicator(IndicatorConfig(window=14))
    indicator.update(klines)

    value = indicator.get_value()
    assert value is not None
    assert "vwap" in value.values
    assert value.values["vwap"] > 0


@pytest.mark.asyncio
async def test_cmf_indicator(klines):
    """测试CMF指标"""
    indicator = CMFIndicator(IndicatorConfig(window=20))
    indicator.update(klines)

    value = indicator.get_value()
    assert value is not None
    assert "cmf" in value.values
    assert -1 <= value.values["cmf"] <= 1 