"""
OKX 数据服务测试
"""
import asyncio
from datetime import datetime, timedelta
from decimal import Decimal
from unittest.mock import AsyncMock, MagicMock, patch

import pytest
from aiohttp import ClientSession
from websockets.client import WebSocketClientProtocol

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
from app.services.okx_service import OKXService
from app.services.kafka_producer import KafkaProducerService


@pytest.fixture
def mock_kafka_producer():
    """Mock Kafka生产者"""
    producer = MagicMock(spec=KafkaProducerService)
    producer.send_market_data = AsyncMock()
    return producer


@pytest.fixture
async def okx_service(mock_kafka_producer):
    """创建OKX服务实例"""
    service = OKXService(kafka_producer=mock_kafka_producer)
    yield service
    await service.close()


@pytest.mark.asyncio
async def test_initialize(okx_service):
    """测试初始化"""
    # Mock exchange info response
    exchange_info = {
        "code": "0",
        "data": [
            {"instId": "BTC-USDT", "state": "live"},
            {"instId": "ETH-USDT", "state": "live"},
            {"instId": "BNB-USDT", "state": "suspend"},
        ]
    }

    with patch.object(ClientSession, "get") as mock_get:
        mock_response = AsyncMock()
        mock_response.status = 200
        mock_response.json = AsyncMock(return_value=exchange_info)
        mock_get.return_value.__aenter__.return_value = mock_response

        await okx_service.initialize()

        assert len(okx_service.symbols) == 2
        assert "BTC-USDT" in okx_service.symbols
        assert "ETH-USDT" in okx_service.symbols
        assert "BNB-USDT" not in okx_service.symbols


@pytest.mark.asyncio
async def test_initialize_error(okx_service):
    """测试初始化错误"""
    with patch.object(ClientSession, "get") as mock_get:
        mock_response = AsyncMock()
        mock_response.status = 500
        mock_get.return_value.__aenter__.return_value = mock_response

        with pytest.raises(Exception):
            await okx_service.initialize()


@pytest.mark.asyncio
async def test_process_ticker(okx_service, mock_kafka_producer):
    """测试处理Ticker数据"""
    # 准备测试数据
    ticker_data = {
        "arg": {
            "channel": "tickers",
            "instId": "BTC-USDT"
        },
        "data": [{
            "instId": "BTC-USDT",
            "last": "29000.00",
            "vol24h": "1000.00",
            "ts": "1609459200000",
            "bidPx": "28990.00",
            "bidSz": "1.5",
            "askPx": "29010.00",
            "askSz": "2.0",
            "high24h": "29500.00",
            "low24h": "28500.00",
            "volCcy24h": "29000000.00",
            "open24h": "28500.00",
        }]
    }

    # 创建预期的Ticker对象
    expected_ticker = Ticker(
        exchange=Exchange.OKX,
        symbol="BTC-USDT",
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
        price_change_percent_24h=1.75438596491228,
    )

    # Mock WebSocket连接
    mock_ws = AsyncMock(spec=WebSocketClientProtocol)
    mock_ws.recv = AsyncMock(side_effect=[
        '{"event":"subscribe","channel":"tickers"}',  # 订阅确认
        str(ticker_data).replace("'", '"'),  # 数据消息
        Exception("Stop"),  # 停止循环
    ])

    with patch("websockets.client.connect", return_value=AsyncMock(
        __aenter__=AsyncMock(return_value=mock_ws),
        __aexit__=AsyncMock(),
    )):
        with pytest.raises(Exception, match="Stop"):
            await okx_service._start_ticker_stream("BTC-USDT")

    # 验证调用
    mock_kafka_producer.send_market_data.assert_called_once_with(
        exchange=Exchange.OKX,
        data_type=DataType.TICKER,
        symbol="BTC-USDT",
        data=expected_ticker.model_dump(),
    )


@pytest.mark.asyncio
async def test_process_kline(okx_service, mock_kafka_producer):
    """测试处理K线数据"""
    # 准备测试数据
    kline_data = {
        "arg": {
            "channel": "candle1m",
            "instId": "BTC-USDT"
        },
        "data": [[
            "1609459200000",  # 开盘时间
            "29000.00",       # 开盘价
            "29100.00",       # 最高价
            "28900.00",       # 最低价
            "29050.00",       # 收盘价
            "100.00",         # 成交量
            "2900000.00",     # 成交额
            "500",            # 成交笔数
            "60.00",          # 主动买入成交量
            "1740000.00",     # 主动买入成交额
        ]]
    }

    # 创建预期的Kline对象
    expected_kline = Kline(
        exchange=Exchange.OKX,
        symbol="BTC-USDT",
        interval=Interval.MIN_1,
        open_time=datetime.fromtimestamp(1609459200),
        close_time=datetime.fromtimestamp(1609459200 + 60),
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

    # Mock WebSocket连接
    mock_ws = AsyncMock(spec=WebSocketClientProtocol)
    mock_ws.recv = AsyncMock(side_effect=[
        '{"event":"subscribe","channel":"candle1m"}',  # 订阅确认
        str(kline_data).replace("'", '"'),  # 数据消息
        Exception("Stop"),  # 停止循环
    ])

    with patch("websockets.client.connect", return_value=AsyncMock(
        __aenter__=AsyncMock(return_value=mock_ws),
        __aexit__=AsyncMock(),
    )):
        with pytest.raises(Exception, match="Stop"):
            await okx_service._start_kline_stream("BTC-USDT")

    # 验证调用
    mock_kafka_producer.send_market_data.assert_called_once_with(
        exchange=Exchange.OKX,
        data_type=DataType.KLINE,
        symbol="BTC-USDT",
        data=expected_kline.model_dump(),
    )


@pytest.mark.asyncio
async def test_process_depth(okx_service, mock_kafka_producer):
    """测试处理深度数据"""
    # 准备测试数据
    depth_data = {
        "arg": {
            "channel": "books",
            "instId": "BTC-USDT"
        },
        "data": [{
            "ts": "1609459200000",
            "bids": [
                ["29000.00", "1.5"],
                ["28990.00", "2.0"],
            ],
            "asks": [
                ["29010.00", "1.0"],
                ["29020.00", "2.5"],
            ]
        }]
    }

    # 创建预期的OrderBook对象
    expected_orderbook = OrderBook(
        exchange=Exchange.OKX,
        symbol="BTC-USDT",
        timestamp=datetime.fromtimestamp(1609459200),
        bids=[
            OrderBookLevel(price=Decimal("29000.00"), volume=Decimal("1.5")),
            OrderBookLevel(price=Decimal("28990.00"), volume=Decimal("2.0")),
        ],
        asks=[
            OrderBookLevel(price=Decimal("29010.00"), volume=Decimal("1.0")),
            OrderBookLevel(price=Decimal("29020.00"), volume=Decimal("2.5")),
        ],
    )

    # Mock WebSocket连接
    mock_ws = AsyncMock(spec=WebSocketClientProtocol)
    mock_ws.recv = AsyncMock(side_effect=[
        '{"event":"subscribe","channel":"books"}',  # 订阅确认
        str(depth_data).replace("'", '"'),  # 数据消息
        Exception("Stop"),  # 停止循环
    ])

    with patch("websockets.client.connect", return_value=AsyncMock(
        __aenter__=AsyncMock(return_value=mock_ws),
        __aexit__=AsyncMock(),
    )):
        with pytest.raises(Exception, match="Stop"):
            await okx_service._start_depth_stream("BTC-USDT")

    # 验证调用
    mock_kafka_producer.send_market_data.assert_called_once_with(
        exchange=Exchange.OKX,
        data_type=DataType.ORDERBOOK,
        symbol="BTC-USDT",
        data=expected_orderbook.model_dump(),
    )


@pytest.mark.asyncio
async def test_process_trade(okx_service, mock_kafka_producer):
    """测试处理成交数据"""
    # 准备测试数据
    trade_data = {
        "arg": {
            "channel": "trades",
            "instId": "BTC-USDT"
        },
        "data": [{
            "instId": "BTC-USDT",
            "tradeId": "12345",
            "px": "29000.00",
            "sz": "1.5",
            "side": "buy",
            "ts": "1609459200000",
        }]
    }

    # 创建预期的Trade对象
    expected_trade = Trade(
        exchange=Exchange.OKX,
        symbol="BTC-USDT",
        id="12345",
        price=Decimal("29000.00"),
        volume=Decimal("1.5"),
        timestamp=datetime.fromtimestamp(1609459200),
        is_buyer_maker=True,
    )

    # Mock WebSocket连接
    mock_ws = AsyncMock(spec=WebSocketClientProtocol)
    mock_ws.recv = AsyncMock(side_effect=[
        '{"event":"subscribe","channel":"trades"}',  # 订阅确认
        str(trade_data).replace("'", '"'),  # 数据消息
        Exception("Stop"),  # 停止循环
    ])

    with patch("websockets.client.connect", return_value=AsyncMock(
        __aenter__=AsyncMock(return_value=mock_ws),
        __aexit__=AsyncMock(),
    )):
        with pytest.raises(Exception, match="Stop"):
            await okx_service._start_trade_stream("BTC-USDT")

    # 验证调用
    mock_kafka_producer.send_market_data.assert_called_once_with(
        exchange=Exchange.OKX,
        data_type=DataType.TRADE,
        symbol="BTC-USDT",
        data=expected_trade.model_dump(),
    )


@pytest.mark.asyncio
async def test_get_historical_klines(okx_service):
    """测试获取历史K线数据"""
    # 准备测试数据
    klines_data = {
        "code": "0",
        "data": [
            [
                "1609459200000",  # 开盘时间
                "29000.00",       # 开盘价
                "29100.00",       # 最高价
                "28900.00",       # 最低价
                "29050.00",       # 收盘价
                "100.00",         # 成交量
                "2900000.00",     # 成交额
                "500",            # 成交笔数
                "60.00",          # 主动买入成交量
                "1740000.00",     # 主动买入成交额
            ]
        ]
    }

    # 创建预期的Kline对象
    expected_kline = Kline(
        exchange=Exchange.OKX,
        symbol="BTC-USDT",
        interval=Interval.MIN_1,
        open_time=datetime.fromtimestamp(1609459200),
        close_time=datetime.fromtimestamp(1609459200 + 60),
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

    # Mock HTTP请求
    with patch.object(ClientSession, "get") as mock_get:
        mock_response = AsyncMock()
        mock_response.status = 200
        mock_response.json = AsyncMock(return_value=klines_data)
        mock_get.return_value.__aenter__.return_value = mock_response

        # 初始化服务
        okx_service.session = ClientSession()

        # 获取历史K线数据
        klines = await okx_service.get_historical_klines(
            symbol="BTC-USDT",
            interval=Interval.MIN_1,
            start_time=datetime.fromtimestamp(1609459200),
            end_time=datetime.fromtimestamp(1609459200 + 60),
            limit=100
        )

        assert len(klines) == 1
        assert klines[0] == expected_kline 