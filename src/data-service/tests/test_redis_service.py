"""
Redis 服务测试
"""
import asyncio
from datetime import datetime, timedelta
from decimal import Decimal
from unittest.mock import AsyncMock, MagicMock, patch

import pytest
import redis.asyncio as redis
from redis.asyncio.client import Redis
from redis.asyncio.connection import ConnectionPool

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
from app.services.redis_service import RedisService


@pytest.fixture
async def redis_service():
    """创建Redis服务实例"""
    service = RedisService()
    yield service
    await service.close()


@pytest.mark.asyncio
async def test_initialize(redis_service):
    """测试初始化"""
    with patch.object(redis, "ConnectionPool") as mock_pool, \
         patch.object(redis, "Redis") as mock_redis:
        # Mock Redis客户端
        mock_client = AsyncMock()
        mock_client.ping = AsyncMock()
        mock_redis.return_value = mock_client

        await redis_service.initialize()

        assert redis_service.pool is not None
        assert redis_service.client is not None
        mock_client.ping.assert_called_once()


@pytest.mark.asyncio
async def test_initialize_error(redis_service):
    """测试初始化错误"""
    with patch.object(redis, "ConnectionPool") as mock_pool, \
         patch.object(redis, "Redis") as mock_redis:
        # Mock Redis客户端
        mock_client = AsyncMock()
        mock_client.ping = AsyncMock(side_effect=Exception("Connection failed"))
        mock_redis.return_value = mock_client

        with pytest.raises(Exception):
            await redis_service.initialize()


@pytest.mark.asyncio
async def test_save_and_get_ticker(redis_service):
    """测试保存和获取Ticker数据"""
    # 准备测试数据
    ticker = Ticker(
        exchange=Exchange.BINANCE,
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

    # Mock Redis客户端
    mock_client = AsyncMock()
    redis_service.client = mock_client

    # 保存数据
    await redis_service.save_ticker(ticker)

    # 验证保存调用
    mock_client.hset.assert_called_once()
    mock_client.expire.assert_called_once()

    # Mock获取数据
    mock_client.hgetall.return_value = {
        "price": "29000.00",
        "volume": "1000.00",
        "timestamp": "1609459200000",
        "bid_price": "28990.00",
        "bid_volume": "1.5",
        "ask_price": "29010.00",
        "ask_volume": "2.0",
        "high_24h": "29500.00",
        "low_24h": "28500.00",
        "volume_24h": "1000.00",
        "quote_volume_24h": "29000000.00",
        "price_change_24h": "500.00",
        "price_change_percent_24h": "1.75438596491228",
    }

    # 获取数据
    result = await redis_service.get_ticker(Exchange.BINANCE, "BTC-USDT")

    # 验证结果
    assert result == ticker


@pytest.mark.asyncio
async def test_save_and_get_klines(redis_service):
    """测试保存和获取K线数据"""
    # 准备测试数据
    kline = Kline(
        exchange=Exchange.BINANCE,
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

    # Mock Redis客户端
    mock_client = AsyncMock()
    redis_service.client = mock_client

    # 保存数据
    await redis_service.save_kline(kline)

    # 验证保存调用
    mock_client.zadd.assert_called_once()
    mock_client.zremrangebyrank.assert_called_once()
    mock_client.expire.assert_called_once()

    # Mock获取数据
    mock_client.zrangebyscore.return_value = [
        kline.model_dump_json()
    ]

    # 获取数据
    result = await redis_service.get_klines(
        Exchange.BINANCE,
        "BTC-USDT",
        Interval.MIN_1,
        start_time=datetime.fromtimestamp(1609459200),
        end_time=datetime.fromtimestamp(1609459200 + 60),
        limit=100
    )

    # 验证结果
    assert len(result) == 1
    assert result[0] == kline


@pytest.mark.asyncio
async def test_save_and_get_orderbook(redis_service):
    """测试保存和获取订单簿数据"""
    # 准备测试数据
    orderbook = OrderBook(
        exchange=Exchange.BINANCE,
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

    # Mock Redis客户端
    mock_client = AsyncMock()
    redis_service.client = mock_client

    # 保存数据
    await redis_service.save_orderbook(orderbook)

    # 验证保存调用
    mock_client.hset.assert_called_once()
    mock_client.expire.assert_called_once()

    # Mock获取数据
    mock_client.hgetall.return_value = {
        "timestamp": "1609459200000",
        "bids": '[[\"29000.00\", \"1.5\"], [\"28990.00\", \"2.0\"]]',
        "asks": '[[\"29010.00\", \"1.0\"], [\"29020.00\", \"2.5\"]]',
    }

    # 获取数据
    result = await redis_service.get_orderbook(Exchange.BINANCE, "BTC-USDT")

    # 验证结果
    assert result == orderbook


@pytest.mark.asyncio
async def test_save_and_get_trades(redis_service):
    """测试保存和获取成交记录"""
    # 准备测试数据
    trade = Trade(
        exchange=Exchange.BINANCE,
        symbol="BTC-USDT",
        id="12345",
        price=Decimal("29000.00"),
        volume=Decimal("1.5"),
        timestamp=datetime.fromtimestamp(1609459200),
        is_buyer_maker=True,
    )

    # Mock Redis客户端
    mock_client = AsyncMock()
    redis_service.client = mock_client

    # 保存数据
    await redis_service.save_trade(trade)

    # 验证保存调用
    mock_client.zadd.assert_called_once()
    mock_client.zremrangebyrank.assert_called_once()
    mock_client.expire.assert_called_once()

    # Mock获取数据
    mock_client.zrangebyscore.return_value = [
        trade.model_dump_json()
    ]

    # 获取数据
    result = await redis_service.get_trades(
        Exchange.BINANCE,
        "BTC-USDT",
        start_time=datetime.fromtimestamp(1609459200),
        end_time=datetime.fromtimestamp(1609459200 + 60),
        limit=100
    )

    # 验证结果
    assert len(result) == 1
    assert result[0] == trade 