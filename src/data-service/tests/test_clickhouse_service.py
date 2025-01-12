"""
ClickHouse 服务测试
"""
from datetime import datetime, timedelta
from decimal import Decimal
from unittest.mock import MagicMock, patch

import pytest
from clickhouse_driver import Client
from clickhouse_driver.errors import Error as ClickHouseError

from app.models.market_data import Exchange, Interval, Kline, Ticker, Trade
from app.services.clickhouse_service import ClickHouseService


@pytest.fixture
def mock_client():
    """模拟 ClickHouse 客户端"""
    with patch("clickhouse_driver.Client") as mock:
        client = mock.return_value
        client.execute = MagicMock()
        yield client


@pytest.fixture
def clickhouse_service(mock_client):
    """创建 ClickHouse 服务实例"""
    service = ClickHouseService()
    service.client = mock_client
    return service


@pytest.fixture
def sample_tickers():
    """创建样本 Ticker 数据"""
    now = datetime.now()
    return [
        Ticker(
            exchange=Exchange.BINANCE,
            symbol="BTC/USDT",
            price=Decimal("50000.00"),
            volume=Decimal("100.00"),
            timestamp=now,
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
        ),
        Ticker(
            exchange=Exchange.BINANCE,
            symbol="ETH/USDT",
            price=Decimal("3000.00"),
            volume=Decimal("1000.00"),
            timestamp=now,
            bid_price=Decimal("2999.00"),
            bid_volume=Decimal("10.00"),
            ask_price=Decimal("3001.00"),
            ask_volume=Decimal("10.00"),
            high_24h=Decimal("3100.00"),
            low_24h=Decimal("2900.00"),
            volume_24h=Decimal("10000.00"),
            quote_volume_24h=Decimal("30000000.00"),
            price_change_24h=Decimal("100.00"),
            price_change_percent_24h=3.0,
        ),
    ]


@pytest.fixture
def sample_klines():
    """创建样本 K线数据"""
    now = datetime.now()
    return [
        Kline(
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
        ),
        Kline(
            exchange=Exchange.BINANCE,
            symbol="ETH/USDT",
            interval=Interval.ONE_MINUTE,
            open_time=now - timedelta(minutes=1),
            close_time=now,
            open=Decimal("3000.00"),
            high=Decimal("3010.00"),
            low=Decimal("2990.00"),
            close=Decimal("3005.00"),
            volume=Decimal("100.00"),
            quote_volume=Decimal("300000.00"),
            trades_count=50,
            taker_buy_volume=Decimal("50.00"),
            taker_buy_quote_volume=Decimal("150000.00"),
        ),
    ]


@pytest.fixture
def sample_trades():
    """创建样本成交数据"""
    now = datetime.now()
    return [
        Trade(
            exchange=Exchange.BINANCE,
            symbol="BTC/USDT",
            trade_id="123456",
            price=Decimal("50000.00"),
            quantity=Decimal("1.00"),
            timestamp=now,
            is_buyer_maker=False,
            is_best_match=True,
        ),
        Trade(
            exchange=Exchange.BINANCE,
            symbol="ETH/USDT",
            trade_id="123457",
            price=Decimal("3000.00"),
            quantity=Decimal("10.00"),
            timestamp=now,
            is_buyer_maker=True,
            is_best_match=True,
        ),
    ]


class TestClickHouseService:
    """ClickHouse 服务测试类"""

    def test_initialize_tables(self, clickhouse_service):
        """测试初始化数据表"""
        # 验证是否调用了 execute 方法创建表
        assert clickhouse_service.client.execute.call_count >= 3

    def test_insert_tickers(self, clickhouse_service, sample_tickers):
        """测试插入 Ticker 数据"""
        clickhouse_service.insert_tickers(sample_tickers)
        
        # 验证是否调用了 execute 方法插入数据
        clickhouse_service.client.execute.assert_called_once()
        args = clickhouse_service.client.execute.call_args[0]
        
        # 验证 SQL 语句
        assert "INSERT INTO tickers" in args[0]
        
        # 验证数据
        data = args[1]
        assert len(data) == 2
        assert data[0][0] == Exchange.BINANCE.value
        assert data[0][1] == "BTC/USDT"
        assert float(data[0][2]) == 50000.00

    def test_insert_klines(self, clickhouse_service, sample_klines):
        """测试插入 K线数据"""
        clickhouse_service.insert_klines(sample_klines)
        
        # 验证是否调用了 execute 方法插入数据
        clickhouse_service.client.execute.assert_called_once()
        args = clickhouse_service.client.execute.call_args[0]
        
        # 验证 SQL 语句
        assert "INSERT INTO klines" in args[0]
        
        # 验证数据
        data = args[1]
        assert len(data) == 2
        assert data[0][0] == Exchange.BINANCE.value
        assert data[0][1] == "BTC/USDT"
        assert data[0][2] == Interval.ONE_MINUTE

    def test_insert_trades(self, clickhouse_service, sample_trades):
        """测试插入成交数据"""
        clickhouse_service.insert_trades(sample_trades)
        
        # 验证是否调用了 execute 方法插入数据
        clickhouse_service.client.execute.assert_called_once()
        args = clickhouse_service.client.execute.call_args[0]
        
        # 验证 SQL 语句
        assert "INSERT INTO trades" in args[0]
        
        # 验证数据
        data = args[1]
        assert len(data) == 2
        assert data[0][0] == Exchange.BINANCE.value
        assert data[0][1] == "BTC/USDT"
        assert data[0][2] == "123456"

    def test_get_klines(self, clickhouse_service):
        """测试查询 K线数据"""
        # 模拟查询结果
        mock_rows = [
            (
                Exchange.BINANCE.value,
                "BTC/USDT",
                Interval.ONE_MINUTE,
                datetime.now() - timedelta(minutes=1),
                datetime.now(),
                50000.00,
                50100.00,
                49900.00,
                50050.00,
                10.00,
                500000.00,
                100,
                5.00,
                250000.00,
            )
        ]
        clickhouse_service.client.execute.return_value = mock_rows

        # 执行查询
        klines = clickhouse_service.get_klines(
            exchange=Exchange.BINANCE,
            symbol="BTC/USDT",
            interval=Interval.ONE_MINUTE,
            limit=1
        )

        # 验证查询结果
        assert len(klines) == 1
        kline = klines[0]
        assert kline.exchange == Exchange.BINANCE
        assert kline.symbol == "BTC/USDT"
        assert kline.interval == Interval.ONE_MINUTE
        assert float(kline.open) == 50000.00
        assert float(kline.high) == 50100.00
        assert float(kline.low) == 49900.00
        assert float(kline.close) == 50050.00

    def test_insert_tickers_error(self, clickhouse_service, sample_tickers):
        """测试插入 Ticker 数据时发生错误"""
        clickhouse_service.client.execute.side_effect = ClickHouseError("Mock error")
        
        with pytest.raises(ClickHouseError):
            clickhouse_service.insert_tickers(sample_tickers)

    def test_insert_klines_error(self, clickhouse_service, sample_klines):
        """测试插入 K线数据时发生错误"""
        clickhouse_service.client.execute.side_effect = ClickHouseError("Mock error")
        
        with pytest.raises(ClickHouseError):
            clickhouse_service.insert_klines(sample_klines)

    def test_insert_trades_error(self, clickhouse_service, sample_trades):
        """测试插入成交数据时发生错误"""
        clickhouse_service.client.execute.side_effect = ClickHouseError("Mock error")
        
        with pytest.raises(ClickHouseError):
            clickhouse_service.insert_trades(sample_trades)

    def test_get_klines_error(self, clickhouse_service):
        """测试查询 K线数据时发生错误"""
        clickhouse_service.client.execute.side_effect = ClickHouseError("Mock error")
        
        with pytest.raises(ClickHouseError):
            clickhouse_service.get_klines(
                exchange=Exchange.BINANCE,
                symbol="BTC/USDT",
                interval=Interval.ONE_MINUTE
            ) 