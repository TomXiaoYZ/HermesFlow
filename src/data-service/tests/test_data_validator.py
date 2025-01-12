"""
数据验证服务测试
"""
from datetime import datetime, timedelta
from decimal import Decimal

import pytest

from app.models.market_data import Exchange, Interval, Kline, Ticker, Trade
from app.services.data_validator import DataValidator


@pytest.fixture
def valid_ticker():
    """创建有效的 Ticker 数据"""
    return Ticker(
        exchange=Exchange.BINANCE,
        symbol="BTC/USDT",
        price=Decimal("50000.00"),
        volume=Decimal("100.00"),
        timestamp=datetime.now(),
        bid_price=Decimal("49999.00"),
        bid_volume=Decimal("1.00"),
        ask_price=Decimal("50001.00"),
        ask_volume=Decimal("1.00"),
        high_24h=Decimal("51000.00"),
        low_24h=Decimal("49000.00"),
        volume_24h=Decimal("1000.00"),
        quote_volume_24h=Decimal("50000000.00"),
        price_change_24h=Decimal("1000.00"),
        price_change_percent_24h=2.0,
    )


@pytest.fixture
def valid_kline():
    """创建有效的 K线数据"""
    now = datetime.now()
    return Kline(
        exchange=Exchange.BINANCE,
        symbol="BTC/USDT",
        interval=Interval.ONE_MINUTE,
        open_time=now - timedelta(minutes=1),
        close_time=now,
        open=Decimal("50000.00"),
        high=Decimal("50100.00"),
        low=Decimal("49900.00"),
        close=Decimal("50050.00"),
        volume=Decimal("10.00"),
        quote_volume=Decimal("500000.00"),
        trades_count=100,
        taker_buy_volume=Decimal("5.00"),
        taker_buy_quote_volume=Decimal("250000.00"),
    )


@pytest.fixture
def valid_trade():
    """创建有效的成交数据"""
    return Trade(
        exchange=Exchange.BINANCE,
        symbol="BTC/USDT",
        trade_id="123456",
        price=Decimal("50000.00"),
        quantity=Decimal("1.00"),
        timestamp=datetime.now(),
        is_buyer_maker=False,
        is_best_match=True,
    )


class TestDataValidator:
    """数据验证服务测试类"""

    def test_validate_ticker_valid(self, valid_ticker):
        """测试验证有效的 Ticker 数据"""
        assert DataValidator.validate_ticker(valid_ticker) is True

    def test_validate_ticker_invalid_price(self, valid_ticker):
        """测试验证无效价格的 Ticker 数据"""
        valid_ticker.price = Decimal("-1.00")
        assert DataValidator.validate_ticker(valid_ticker) is False

    def test_validate_ticker_invalid_volume(self, valid_ticker):
        """测试验证无效成交量的 Ticker 数据"""
        valid_ticker.volume = Decimal("-1.00")
        assert DataValidator.validate_ticker(valid_ticker) is False

    def test_validate_ticker_invalid_timestamp(self, valid_ticker):
        """测试验证无效时间戳的 Ticker 数据"""
        valid_ticker.timestamp = datetime.now() + timedelta(days=2)
        assert DataValidator.validate_ticker(valid_ticker) is False

    def test_validate_ticker_invalid_bid_ask(self, valid_ticker):
        """测试验证无效买卖价格的 Ticker 数据"""
        valid_ticker.bid_price = Decimal("50002.00")
        valid_ticker.ask_price = Decimal("50001.00")
        assert DataValidator.validate_ticker(valid_ticker) is False

    def test_validate_kline_valid(self, valid_kline):
        """测试验证有效的 K线数据"""
        assert DataValidator.validate_kline(valid_kline) is True

    def test_validate_kline_invalid_price(self, valid_kline):
        """测试验证无效价格的 K线数据"""
        valid_kline.high = Decimal("-1.00")
        assert DataValidator.validate_kline(valid_kline) is False

    def test_validate_kline_invalid_time(self, valid_kline):
        """测试验证无效时间的 K线数据"""
        valid_kline.close_time = valid_kline.open_time - timedelta(minutes=1)
        assert DataValidator.validate_kline(valid_kline) is False

    def test_validate_kline_invalid_high_low(self, valid_kline):
        """测试验证无效最高最低价的 K线数据"""
        valid_kline.high = Decimal("49000.00")
        valid_kline.low = Decimal("50000.00")
        assert DataValidator.validate_kline(valid_kline) is False

    def test_validate_trade_valid(self, valid_trade):
        """测试验证有效的成交数据"""
        assert DataValidator.validate_trade(valid_trade) is True

    def test_validate_trade_invalid_price(self, valid_trade):
        """测试验证无效价格的成交数据"""
        valid_trade.price = Decimal("-1.00")
        assert DataValidator.validate_trade(valid_trade) is False

    def test_validate_trade_invalid_quantity(self, valid_trade):
        """测试验证无效数量的成交数据"""
        valid_trade.quantity = Decimal("0")
        assert DataValidator.validate_trade(valid_trade) is False

    def test_validate_trade_invalid_timestamp(self, valid_trade):
        """测试验证无效时间戳的成交数据"""
        valid_trade.timestamp = datetime.now() - timedelta(days=2)
        assert DataValidator.validate_trade(valid_trade) is False

    def test_clean_ticker(self, valid_ticker):
        """测试清洗 Ticker 数据"""
        valid_ticker.volume = Decimal("-1.00")
        cleaned = DataValidator.clean_ticker(valid_ticker)
        assert cleaned is not None
        assert cleaned.volume == Decimal("0")

    def test_clean_kline(self, valid_kline):
        """测试清洗 K线数据"""
        valid_kline.volume = Decimal("-1.00")
        valid_kline.high = Decimal("49000.00")  # 低于 open 和 close
        cleaned = DataValidator.clean_kline(valid_kline)
        assert cleaned is not None
        assert cleaned.volume == Decimal("0")
        assert cleaned.high == Decimal("50050.00")  # 应该等于最高价

    def test_clean_trade(self, valid_trade):
        """测试清洗成交数据"""
        valid_trade.quantity = Decimal("-1.00")
        cleaned = DataValidator.clean_trade(valid_trade)
        assert cleaned is None  # 无效的成交量应该返回 None 