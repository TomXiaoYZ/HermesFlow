"""
Binance 数据服务测试
"""
import asyncio
from datetime import datetime, timedelta
from decimal import Decimal
from unittest.mock import AsyncMock, MagicMock, patch

import pytest
from binance import AsyncClient
from binance.exceptions import BinanceAPIException

from app.models.market_data import (
    DataType,
    Exchange,
    Interval,
    Kline,
    OrderBook,
    OrderBookLevel,
    Ticker,
    Trade,
)
from app.services.binance_service import BinanceService
from app.services.clickhouse_service import ClickHouseService
from app.services.data_validator import DataValidator
from app.services.kafka_producer import KafkaProducerService


@pytest.fixture
def mock_kafka_producer():
    """Mock Kafka生产者"""
    producer = MagicMock(spec=KafkaProducerService)
    producer.send_market_data = AsyncMock()
    return producer


@pytest.fixture
def mock_clickhouse_service():
    """Mock ClickHouse服务"""
    service = MagicMock(spec=ClickHouseService)
    service.save_ticker = MagicMock()
    service.save_kline = MagicMock()
    service.save_trade = MagicMock()
    service.save_orderbook = MagicMock()
    service.get_klines = MagicMock()
    return service


@pytest.fixture
def mock_data_validator():
    """Mock 数据验证器"""
    validator = MagicMock(spec=DataValidator)
    validator.validate_ticker = MagicMock()
    validator.validate_kline = MagicMock()
    validator.validate_trade = MagicMock()
    validator.validate_orderbook = MagicMock()
    return validator


@pytest.fixture
async def binance_service(mock_kafka_producer, mock_clickhouse_service, mock_data_validator):
    """创建Binance服务实例"""
    service = BinanceService(
        kafka_producer=mock_kafka_producer,
        clickhouse_service=mock_clickhouse_service,
        data_validator=mock_data_validator,
    )
    yield service
    await service.close()


@pytest.mark.asyncio
async def test_initialize(binance_service):
    """测试初始化"""
    # Mock exchange info response
    exchange_info = {
        "symbols": [
            {"symbol": "BTCUSDT", "status": "TRADING"},
            {"symbol": "ETHUSDT", "status": "TRADING"},
            {"symbol": "BNBUSDT", "status": "HALT"},
        ]
    }

    with patch.object(AsyncClient, "create") as mock_create:
        mock_client = AsyncMock()
        mock_client.get_exchange_info = AsyncMock(return_value=exchange_info)
        mock_create.return_value = mock_client

        await binance_service.initialize()

        assert len(binance_service.symbols) == 2
        assert "BTCUSDT" in binance_service.symbols
        assert "ETHUSDT" in binance_service.symbols
        assert "BNBUSDT" not in binance_service.symbols


@pytest.mark.asyncio
async def test_initialize_error(binance_service):
    """测试初始化错误"""
    with patch.object(AsyncClient, "create") as mock_create:
        mock_create.side_effect = BinanceAPIException("API Error")

        with pytest.raises(BinanceAPIException):
            await binance_service.initialize()


@pytest.mark.asyncio
async def test_process_ticker(binance_service, mock_kafka_producer, mock_clickhouse_service, mock_data_validator):
    """测试处理Ticker数据"""
    # 准备测试数据
    ticker_data = {
        "e": "24hrTicker",
        "E": 1609459200000,  # 时间戳
        "s": "BTCUSDT",      # 交易对
        "c": "29000.00",     # 最新价格
        "v": "1000.00",      # 成交量
        "b": "28990.00",     # 买一价
        "B": "1.5",          # 买一量
        "a": "29010.00",     # 卖一价
        "A": "2.0",          # 卖一量
        "h": "29500.00",     # 24h最高价
        "l": "28500.00",     # 24h最低价
        "p": "500.00",       # 24h价格变化
        "P": "1.75",         # 24h价格变化百分比
        "q": "29000000.00",  # 成交额
    }

    # 创建预期的Ticker对象
    expected_ticker = Ticker(
        exchange=Exchange.BINANCE,
        symbol="BTCUSDT",
        price=Decimal("29000.00"),
        volume=Decimal("1000.00"),
        timestamp=datetime.fromtimestamp(1609459200),
        bid_price=Decimal("28990.00"),
        bid_volume=Decimal("1.5"),
        ask_price=Decimal("29010.00"),
        ask_volume=Decimal("2.0"),
        high_24h=Decimal("29500.00"),
        low_24h=Decimal("28500.00"),
        volume_24h=Decimal("1000.00"),
        quote_volume_24h=Decimal("29000000.00"),
        price_change_24h=Decimal("500.00"),
        price_change_percent_24h=1.75,
    )

    # Mock数据验证器返回有效数据
    mock_data_validator.validate_ticker.return_value = expected_ticker

    # 模拟WebSocket消息处理
    mock_stream = AsyncMock()
    mock_stream.recv = AsyncMock(side_effect=[ticker_data, Exception("Stop")])

    # 执行测试
    with pytest.raises(Exception, match="Stop"):
        await binance_service._start_ticker_stream("BTCUSDT")

    # 验证调用
    mock_data_validator.validate_ticker.assert_called_once()
    mock_clickhouse_service.save_ticker.assert_called_once_with(expected_ticker)
    mock_kafka_producer.send_market_data.assert_called_once_with(
        exchange=Exchange.BINANCE,
        data_type=DataType.TICKER,
        symbol="BTCUSDT",
        data=expected_ticker.model_dump(),
    )


@pytest.mark.asyncio
async def test_process_kline(binance_service, mock_kafka_producer, mock_clickhouse_service, mock_data_validator):
    """测试处理K线数据"""
    # 准备测试数据
    kline_data = {
        "e": "kline",
        "E": 1609459200000,
        "s": "BTCUSDT",
        "k": {
            "t": 1609459200000,  # 开盘时间
            "T": 1609459500000,  # 收盘时间
            "s": "BTCUSDT",      # 交易对
            "i": "5m",           # 间隔
            "o": "29000.00",     # 开盘价
            "h": "29100.00",     # 最高价
            "l": "28900.00",     # 最低价
            "c": "29050.00",     # 收盘价
            "v": "100.00",       # 成交量
            "n": 500,            # 成交笔数
            "q": "2900000.00",   # 成交额
            "V": "60.00",        # 主动买入成交量
            "Q": "1740000.00",   # 主动买入成交额
        }
    }

    # 创建预期的Kline对象
    expected_kline = Kline(
        exchange=Exchange.BINANCE,
        symbol="BTCUSDT",
        interval="5m",
        open_time=datetime.fromtimestamp(1609459200),
        close_time=datetime.fromtimestamp(1609459500),
        open=Decimal("29000.00"),
        high=Decimal("29100.00"),
        low=Decimal("28900.00"),
        close=Decimal("29050.00"),
        volume=Decimal("100.00"),
        quote_volume=Decimal("2900000.00"),
        trades_count=500,
        taker_buy_volume=Decimal("60.00"),
        taker_buy_quote_volume=Decimal("1740000.00"),
    )

    # Mock数据验证器返回有效数据
    mock_data_validator.validate_kline.return_value = expected_kline

    # 模拟WebSocket消息处理
    mock_stream = AsyncMock()
    mock_stream.recv = AsyncMock(side_effect=[kline_data, Exception("Stop")])

    # 执行测试
    with pytest.raises(Exception, match="Stop"):
        await binance_service._start_kline_stream("BTCUSDT")

    # 验证调用
    mock_data_validator.validate_kline.assert_called_once()
    mock_clickhouse_service.save_kline.assert_called_once_with(expected_kline)
    mock_kafka_producer.send_market_data.assert_called_once_with(
        exchange=Exchange.BINANCE,
        data_type=DataType.KLINE,
        symbol="BTCUSDT",
        data=expected_kline.model_dump(),
    )


@pytest.mark.asyncio
async def test_process_trade(binance_service, mock_kafka_producer, mock_clickhouse_service, mock_data_validator):
    """测试处理成交数据"""
    # 准备测试数据
    trade_data = {
        "e": "trade",
        "E": 1609459200000,  # 事件时间
        "s": "BTCUSDT",      # 交易对
        "t": 12345,          # 成交ID
        "p": "29000.00",     # 成交价格
        "q": "1.5",          # 成交数量
        "T": 1609459200000,  # 成交时间
        "m": True,           # 是否是买方主动成交
        "M": True,           # 是否是最优撮合
    }

    # 创建预期的Trade对象
    expected_trade = Trade(
        exchange=Exchange.BINANCE,
        symbol="BTCUSDT",
        trade_id="12345",
        price=Decimal("29000.00"),
        quantity=Decimal("1.5"),
        timestamp=datetime.fromtimestamp(1609459200),
        is_buyer_maker=True,
        is_best_match=True,
    )

    # Mock数据验证器返回有效数据
    mock_data_validator.validate_trade.return_value = expected_trade

    # 模拟WebSocket消息处理
    mock_stream = AsyncMock()
    mock_stream.recv = AsyncMock(side_effect=[trade_data, Exception("Stop")])

    # 执行测试
    with pytest.raises(Exception, match="Stop"):
        await binance_service._start_trade_stream("BTCUSDT")

    # 验证调用
    mock_data_validator.validate_trade.assert_called_once()
    mock_clickhouse_service.save_trade.assert_called_once_with(expected_trade)
    mock_kafka_producer.send_market_data.assert_called_once_with(
        exchange=Exchange.BINANCE,
        data_type=DataType.TRADE,
        symbol="BTCUSDT",
        data=expected_trade.model_dump(),
    )


@pytest.mark.asyncio
async def test_process_orderbook(binance_service, mock_kafka_producer, mock_clickhouse_service, mock_data_validator):
    """测试处理订单簿数据"""
    # 准备测试数据
    orderbook_data = {
        "e": "depthUpdate",
        "E": 1609459200000,  # 事件时间
        "s": "BTCUSDT",      # 交易对
        "u": 12345,          # 最后更新ID
        "b": [               # 买盘
            ["28990.00", "1.5"],
            ["28980.00", "2.0"],
        ],
        "a": [               # 卖盘
            ["29010.00", "1.0"],
            ["29020.00", "2.5"],
        ],
    }

    # 创建预期的OrderBook对象
    expected_orderbook = OrderBook(
        exchange=Exchange.BINANCE,
        symbol="BTCUSDT",
        timestamp=datetime.fromtimestamp(1609459200),
        last_update_id=12345,
        bids=[
            OrderBookLevel(price=Decimal("28990.00"), quantity=Decimal("1.5")),
            OrderBookLevel(price=Decimal("28980.00"), quantity=Decimal("2.0")),
        ],
        asks=[
            OrderBookLevel(price=Decimal("29010.00"), quantity=Decimal("1.0")),
            OrderBookLevel(price=Decimal("29020.00"), quantity=Decimal("2.5")),
        ],
    )

    # Mock数据验证器返回有效数据
    mock_data_validator.validate_orderbook.return_value = expected_orderbook

    # 模拟WebSocket消息处理
    mock_stream = AsyncMock()
    mock_stream.recv = AsyncMock(side_effect=[orderbook_data, Exception("Stop")])

    # 执行测试
    with pytest.raises(Exception, match="Stop"):
        await binance_service._start_depth_stream("BTCUSDT")

    # 验证调用
    mock_data_validator.validate_orderbook.assert_called_once()
    mock_clickhouse_service.save_orderbook.assert_called_once_with(expected_orderbook)
    mock_kafka_producer.send_market_data.assert_called_once_with(
        exchange=Exchange.BINANCE,
        data_type=DataType.ORDERBOOK,
        symbol="BTCUSDT",
        data=expected_orderbook.model_dump(),
    )


@pytest.mark.asyncio
async def test_get_historical_klines(binance_service, mock_clickhouse_service, mock_data_validator):
    """测试获取历史K线数据"""
    # 准备测试数据
    start_time = datetime.utcnow() - timedelta(days=1)
    end_time = datetime.utcnow()
    symbol = "BTCUSDT"
    interval = "1h"
    limit = 24

    # Mock ClickHouse返回空数据，触发从Binance获取数据
    mock_clickhouse_service.get_klines.return_value = []

    # Mock Binance API返回数据
    mock_kline_data = [
        [
            1609459200000,  # 开盘时间
            "29000.00",     # 开盘价
            "29100.00",     # 最高价
            "28900.00",     # 最低价
            "29050.00",     # 收盘价
            "100.00",       # 成交量
            1609462800000,  # 收盘时间
            "2900000.00",   # 成交额
            500,            # 成交笔数
            "60.00",        # 主动买入成交量
            "1740000.00",   # 主动买入成交额
        ]
    ]

    expected_kline = Kline(
        exchange=Exchange.BINANCE,
        symbol=symbol,
        interval=interval,
        open_time=datetime.fromtimestamp(1609459200),
        close_time=datetime.fromtimestamp(1609462800),
        open=Decimal("29000.00"),
        high=Decimal("29100.00"),
        low=Decimal("28900.00"),
        close=Decimal("29050.00"),
        volume=Decimal("100.00"),
        quote_volume=Decimal("2900000.00"),
        trades_count=500,
        taker_buy_volume=Decimal("60.00"),
        taker_buy_quote_volume=Decimal("1740000.00"),
    )

    # Mock数据验证器返回有效数据
    mock_data_validator.validate_kline.return_value = expected_kline

    with patch.object(binance_service.client, "get_historical_klines") as mock_get_klines:
        mock_get_klines.return_value = mock_kline_data

        # 执行测试
        klines = await binance_service.get_historical_klines(
            symbol=symbol,
            interval=interval,
            start_time=start_time,
            end_time=end_time,
            limit=limit,
        )

        # 验证调用
        mock_clickhouse_service.get_klines.assert_called_once_with(
            exchange=Exchange.BINANCE,
            symbol=symbol,
            interval=interval,
            start_time=start_time,
            end_time=end_time,
            limit=limit,
        )
        mock_get_klines.assert_called_once()
        mock_data_validator.validate_kline.assert_called_once()
        mock_clickhouse_service.save_kline.assert_called_once_with(expected_kline)

        # 验证结果
        assert len(klines) == 1
        assert klines[0] == expected_kline 