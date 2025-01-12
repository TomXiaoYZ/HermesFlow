"""
策略服务测试
"""
from datetime import datetime
from decimal import Decimal
from unittest.mock import AsyncMock, MagicMock, patch

import pytest

from app.models.market_data import (
    Exchange,
    Interval,
    Kline,
    OrderBook,
    OrderBookLevel,
    Ticker,
    Trade,
)
from app.models.strategy import Signal, SignalSource, SignalType
from app.services.strategy_service import StrategyService
from app.strategies.ma_cross_strategy import MACrossStrategy


@pytest.fixture
async def strategy_service():
    """创建策略服务实例"""
    service = StrategyService()
    yield service
    await service.close()


@pytest.mark.asyncio
async def test_initialize(strategy_service):
    """测试初始化"""
    with patch("app.services.redis_service.RedisService") as mock_redis, \
         patch("app.services.postgresql_service.PostgresqlService") as mock_pg:
        # Mock Redis服务
        mock_redis_instance = AsyncMock()
        mock_redis.return_value = mock_redis_instance

        # Mock PostgreSQL服务
        mock_pg_instance = AsyncMock()
        mock_pg.return_value = mock_pg_instance

        await strategy_service.initialize()

        assert strategy_service.redis_service is not None
        assert strategy_service.postgresql_service is not None
        mock_redis_instance.initialize.assert_called_once()
        mock_pg_instance.initialize.assert_called_once()


@pytest.mark.asyncio
async def test_register_strategy(strategy_service):
    """测试注册策略"""
    with patch("app.services.redis_service.RedisService") as mock_redis, \
         patch("app.services.postgresql_service.PostgresqlService") as mock_pg:
        # Mock服务
        mock_redis_instance = AsyncMock()
        mock_redis.return_value = mock_redis_instance
        mock_pg_instance = AsyncMock()
        mock_pg.return_value = mock_pg_instance

        # 初始化服务
        await strategy_service.initialize()

        # 注册策略
        await strategy_service.register_strategy(
            strategy_class=MACrossStrategy,
            name="test_strategy",
            exchanges={Exchange.BINANCE},
            symbols={"BTC-USDT"},
            parameters={
                "fast_period": 5,
                "slow_period": 20,
                "interval": Interval.MIN_1
            }
        )

        # 验证结果
        assert "test_strategy" in strategy_service.strategies
        strategy = strategy_service.strategies["test_strategy"]
        assert isinstance(strategy, MACrossStrategy)
        assert strategy.name == "test_strategy"
        assert strategy.exchanges == {Exchange.BINANCE}
        assert strategy.symbols == {"BTC-USDT"}
        assert strategy.fast_period == 5
        assert strategy.slow_period == 20
        assert strategy.interval == Interval.MIN_1


@pytest.mark.asyncio
async def test_unregister_strategy(strategy_service):
    """测试注销策略"""
    with patch("app.services.redis_service.RedisService") as mock_redis, \
         patch("app.services.postgresql_service.PostgresqlService") as mock_pg:
        # Mock服务
        mock_redis_instance = AsyncMock()
        mock_redis.return_value = mock_redis_instance
        mock_pg_instance = AsyncMock()
        mock_pg.return_value = mock_pg_instance

        # 初始化服务
        await strategy_service.initialize()

        # 注册策略
        await strategy_service.register_strategy(
            strategy_class=MACrossStrategy,
            name="test_strategy",
            exchanges={Exchange.BINANCE},
            symbols={"BTC-USDT"},
            parameters={
                "fast_period": 5,
                "slow_period": 20,
                "interval": Interval.MIN_1
            }
        )

        # 注销策略
        await strategy_service.unregister_strategy("test_strategy")

        # 验证结果
        assert "test_strategy" not in strategy_service.strategies


@pytest.mark.asyncio
async def test_on_kline(strategy_service):
    """测试处理K线数据"""
    with patch("app.services.redis_service.RedisService") as mock_redis, \
         patch("app.services.postgresql_service.PostgresqlService") as mock_pg:
        # Mock服务
        mock_redis_instance = AsyncMock()
        mock_redis.return_value = mock_redis_instance
        mock_pg_instance = AsyncMock()
        mock_pg.return_value = mock_pg_instance

        # 初始化服务
        await strategy_service.initialize()

        # 注册策略
        await strategy_service.register_strategy(
            strategy_class=MACrossStrategy,
            name="test_strategy",
            exchanges={Exchange.BINANCE},
            symbols={"BTC-USDT"},
            parameters={
                "fast_period": 5,
                "slow_period": 20,
                "interval": Interval.MIN_1
            }
        )

        # 准备测试数据
        klines = []
        base_timestamp = 1609459200
        base_price = 29000.0
        for i in range(30):
            klines.append(Kline(
                exchange=Exchange.BINANCE,
                symbol="BTC-USDT",
                interval=Interval.MIN_1,
                open_time=datetime.fromtimestamp(base_timestamp + i * 60),
                close_time=datetime.fromtimestamp(base_timestamp + (i + 1) * 60),
                open=Decimal(str(base_price + i)),
                high=Decimal(str(base_price + i + 10)),
                low=Decimal(str(base_price + i - 10)),
                close=Decimal(str(base_price + i + 5)),
                volume=Decimal("100.0"),
                quote_volume=Decimal(str((base_price + i) * 100)),
                trades_count=100,
                taker_buy_volume=Decimal("60.0"),
                taker_buy_quote_volume=Decimal(str((base_price + i) * 60))
            ))

        # 处理K线数据
        strategy = strategy_service.strategies["test_strategy"]
        for kline in klines[:-1]:
            strategy.update_kline(kline)

        # 处理最后一根K线
        signals = await strategy_service.on_kline(klines[-1])

        # 验证结果
        assert len(signals) == 1
        signal = signals[0]
        assert signal.exchange == Exchange.BINANCE
        assert signal.symbol == "BTC-USDT"
        assert signal.signal_type == SignalType.LONG
        assert signal.source == SignalSource.STRATEGY
        assert signal.timestamp == klines[-1].close_time
        assert signal.price == klines[-1].close


@pytest.mark.asyncio
async def test_on_ticker(strategy_service):
    """测试处理Ticker数据"""
    with patch("app.services.redis_service.RedisService") as mock_redis, \
         patch("app.services.postgresql_service.PostgresqlService") as mock_pg:
        # Mock服务
        mock_redis_instance = AsyncMock()
        mock_redis.return_value = mock_redis_instance
        mock_pg_instance = AsyncMock()
        mock_pg.return_value = mock_pg_instance

        # 初始化服务
        await strategy_service.initialize()

        # 注册策略
        await strategy_service.register_strategy(
            strategy_class=MACrossStrategy,
            name="test_strategy",
            exchanges={Exchange.BINANCE},
            symbols={"BTC-USDT"},
            parameters={
                "fast_period": 5,
                "slow_period": 20,
                "interval": Interval.MIN_1
            }
        )

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
            price_change_percent_24h=1.75438596491228
        )

        # 处理Ticker数据
        signals = await strategy_service.on_ticker(ticker)

        # 验证结果
        assert len(signals) == 0


@pytest.mark.asyncio
async def test_on_orderbook(strategy_service):
    """测试处理订单簿数据"""
    with patch("app.services.redis_service.RedisService") as mock_redis, \
         patch("app.services.postgresql_service.PostgresqlService") as mock_pg:
        # Mock服务
        mock_redis_instance = AsyncMock()
        mock_redis.return_value = mock_redis_instance
        mock_pg_instance = AsyncMock()
        mock_pg.return_value = mock_pg_instance

        # 初始化服务
        await strategy_service.initialize()

        # 注册策略
        await strategy_service.register_strategy(
            strategy_class=MACrossStrategy,
            name="test_strategy",
            exchanges={Exchange.BINANCE},
            symbols={"BTC-USDT"},
            parameters={
                "fast_period": 5,
                "slow_period": 20,
                "interval": Interval.MIN_1
            }
        )

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
            ]
        )

        # 处理订单簿数据
        signals = await strategy_service.on_orderbook(orderbook)

        # 验证结果
        assert len(signals) == 0


@pytest.mark.asyncio
async def test_on_trade(strategy_service):
    """测试处理成交记录"""
    with patch("app.services.redis_service.RedisService") as mock_redis, \
         patch("app.services.postgresql_service.PostgresqlService") as mock_pg:
        # Mock服务
        mock_redis_instance = AsyncMock()
        mock_redis.return_value = mock_redis_instance
        mock_pg_instance = AsyncMock()
        mock_pg.return_value = mock_pg_instance

        # 初始化服务
        await strategy_service.initialize()

        # 注册策略
        await strategy_service.register_strategy(
            strategy_class=MACrossStrategy,
            name="test_strategy",
            exchanges={Exchange.BINANCE},
            symbols={"BTC-USDT"},
            parameters={
                "fast_period": 5,
                "slow_period": 20,
                "interval": Interval.MIN_1
            }
        )

        # 准备测试数据
        trade = Trade(
            exchange=Exchange.BINANCE,
            symbol="BTC-USDT",
            id="12345",
            price=Decimal("29000.00"),
            volume=Decimal("1.5"),
            timestamp=datetime.fromtimestamp(1609459200),
            is_buyer_maker=True
        )

        # 处理成交记录
        signals = await strategy_service.on_trade(trade)

        # 验证结果
        assert len(signals) == 0 