"""
PostgreSQL 服务测试
"""
import json
from datetime import datetime
from decimal import Decimal
from unittest.mock import AsyncMock, MagicMock, patch

import pytest
from asyncpg import Connection, Pool

from app.models.config_data import (
    ApiKey,
    ExchangeConfig,
    StrategyConfig,
    SystemConfig,
    TradingPairConfig,
)
from app.models.market_data import Exchange
from app.services.postgresql_service import PostgresqlService


@pytest.fixture
async def postgresql_service():
    """创建PostgreSQL服务实例"""
    service = PostgresqlService()
    yield service
    await service.close()


@pytest.mark.asyncio
async def test_initialize(postgresql_service):
    """测试初始化"""
    with patch("asyncpg.create_pool") as mock_create_pool:
        # Mock连接池
        mock_pool = AsyncMock(spec=Pool)
        mock_create_pool.return_value = mock_pool

        # Mock连接
        mock_conn = AsyncMock(spec=Connection)
        mock_pool.acquire.return_value.__aenter__.return_value = mock_conn

        await postgresql_service.initialize()

        assert postgresql_service.pool is not None
        mock_create_pool.assert_called_once()
        mock_conn.execute.assert_called()


@pytest.mark.asyncio
async def test_initialize_error(postgresql_service):
    """测试初始化错误"""
    with patch("asyncpg.create_pool") as mock_create_pool:
        mock_create_pool.side_effect = Exception("Connection failed")

        with pytest.raises(Exception):
            await postgresql_service.initialize()


@pytest.mark.asyncio
async def test_save_and_get_api_key(postgresql_service):
    """测试保存和获取API密钥"""
    # 准备测试数据
    api_key = ApiKey(
        exchange=Exchange.BINANCE,
        name="test_key",
        api_key="test_api_key",
        api_secret="test_api_secret",
        passphrase="test_passphrase",
        is_test=True
    )

    # Mock连接池
    mock_pool = AsyncMock(spec=Pool)
    postgresql_service.pool = mock_pool

    # Mock连接
    mock_conn = AsyncMock(spec=Connection)
    mock_pool.acquire.return_value.__aenter__.return_value = mock_conn

    # Mock查询结果
    mock_conn.fetchrow.return_value = {
        "exchange": "BINANCE",
        "name": "test_key",
        "api_key": "test_api_key",
        "api_secret": "test_api_secret",
        "passphrase": "test_passphrase",
        "is_test": True
    }

    # 保存数据
    await postgresql_service.save_api_key(api_key)

    # 验证保存调用
    mock_conn.execute.assert_called_once()

    # 获取数据
    result = await postgresql_service.get_api_key(
        Exchange.BINANCE.value,
        "test_key"
    )

    # 验证结果
    assert result == api_key


@pytest.mark.asyncio
async def test_list_api_keys(postgresql_service):
    """测试获取API密钥列表"""
    # Mock连接池
    mock_pool = AsyncMock(spec=Pool)
    postgresql_service.pool = mock_pool

    # Mock连接
    mock_conn = AsyncMock(spec=Connection)
    mock_pool.acquire.return_value.__aenter__.return_value = mock_conn

    # Mock查询结果
    mock_conn.fetch.return_value = [
        {
            "exchange": "BINANCE",
            "name": "test_key_1",
            "api_key": "test_api_key_1",
            "api_secret": "test_api_secret_1",
            "passphrase": "test_passphrase_1",
            "is_test": True
        },
        {
            "exchange": "BINANCE",
            "name": "test_key_2",
            "api_key": "test_api_key_2",
            "api_secret": "test_api_secret_2",
            "passphrase": "test_passphrase_2",
            "is_test": False
        }
    ]

    # 获取数据
    result = await postgresql_service.list_api_keys(Exchange.BINANCE.value)

    # 验证结果
    assert len(result) == 2
    assert result[0].name == "test_key_1"
    assert result[1].name == "test_key_2"


@pytest.mark.asyncio
async def test_delete_api_key(postgresql_service):
    """测试删除API密钥"""
    # Mock连接池
    mock_pool = AsyncMock(spec=Pool)
    postgresql_service.pool = mock_pool

    # Mock连接
    mock_conn = AsyncMock(spec=Connection)
    mock_pool.acquire.return_value.__aenter__.return_value = mock_conn

    # Mock删除结果
    mock_conn.execute.return_value = "DELETE 1"

    # 删除数据
    result = await postgresql_service.delete_api_key(
        Exchange.BINANCE.value,
        "test_key"
    )

    # 验证结果
    assert result is True


@pytest.mark.asyncio
async def test_save_and_get_exchange_config(postgresql_service):
    """测试保存和获取交易所配置"""
    # 准备测试数据
    config = ExchangeConfig(
        exchange=Exchange.BINANCE,
        ws_url="wss://test.binance.com/ws",
        rest_url="https://test.binance.com/api",
        rate_limit_per_second=20
    )

    # Mock连接池
    mock_pool = AsyncMock(spec=Pool)
    postgresql_service.pool = mock_pool

    # Mock连接
    mock_conn = AsyncMock(spec=Connection)
    mock_pool.acquire.return_value.__aenter__.return_value = mock_conn

    # Mock查询结果
    mock_conn.fetchrow.return_value = {
        "exchange": "BINANCE",
        "ws_url": "wss://test.binance.com/ws",
        "rest_url": "https://test.binance.com/api",
        "rate_limit_per_second": 20
    }

    # 保存数据
    await postgresql_service.save_exchange_config(config)

    # 验证保存调用
    mock_conn.execute.assert_called_once()

    # 获取数据
    result = await postgresql_service.get_exchange_config(Exchange.BINANCE.value)

    # 验证结果
    assert result == config


@pytest.mark.asyncio
async def test_save_and_get_trading_pair_config(postgresql_service):
    """测试保存和获取交易对配置"""
    # 准备测试数据
    config = TradingPairConfig(
        exchange=Exchange.BINANCE,
        symbol="BTC-USDT",
        base_asset="BTC",
        quote_asset="USDT",
        price_precision=2,
        volume_precision=6,
        min_price=Decimal("0.01"),
        max_price=Decimal("100000"),
        min_volume=Decimal("0.00001"),
        max_volume=Decimal("1000"),
        min_notional=Decimal("10")
    )

    # Mock连接池
    mock_pool = AsyncMock(spec=Pool)
    postgresql_service.pool = mock_pool

    # Mock连接
    mock_conn = AsyncMock(spec=Connection)
    mock_pool.acquire.return_value.__aenter__.return_value = mock_conn

    # Mock查询结果
    mock_conn.fetchrow.return_value = {
        "exchange": "BINANCE",
        "symbol": "BTC-USDT",
        "base_asset": "BTC",
        "quote_asset": "USDT",
        "price_precision": 2,
        "volume_precision": 6,
        "min_price": Decimal("0.01"),
        "max_price": Decimal("100000"),
        "min_volume": Decimal("0.00001"),
        "max_volume": Decimal("1000"),
        "min_notional": Decimal("10")
    }

    # 保存数据
    await postgresql_service.save_trading_pair_config(config)

    # 验证保存调用
    mock_conn.execute.assert_called_once()

    # 获取数据
    result = await postgresql_service.get_trading_pair_config(
        Exchange.BINANCE.value,
        "BTC-USDT"
    )

    # 验证结果
    assert result == config


@pytest.mark.asyncio
async def test_list_trading_pair_configs(postgresql_service):
    """测试获取交易对配置列表"""
    # Mock连接池
    mock_pool = AsyncMock(spec=Pool)
    postgresql_service.pool = mock_pool

    # Mock连接
    mock_conn = AsyncMock(spec=Connection)
    mock_pool.acquire.return_value.__aenter__.return_value = mock_conn

    # Mock查询结果
    mock_conn.fetch.return_value = [
        {
            "exchange": "BINANCE",
            "symbol": "BTC-USDT",
            "base_asset": "BTC",
            "quote_asset": "USDT",
            "price_precision": 2,
            "volume_precision": 6,
            "min_price": Decimal("0.01"),
            "max_price": Decimal("100000"),
            "min_volume": Decimal("0.00001"),
            "max_volume": Decimal("1000"),
            "min_notional": Decimal("10")
        },
        {
            "exchange": "BINANCE",
            "symbol": "ETH-USDT",
            "base_asset": "ETH",
            "quote_asset": "USDT",
            "price_precision": 2,
            "volume_precision": 6,
            "min_price": Decimal("0.01"),
            "max_price": Decimal("10000"),
            "min_volume": Decimal("0.0001"),
            "max_volume": Decimal("1000"),
            "min_notional": Decimal("10")
        }
    ]

    # 获取数据
    result = await postgresql_service.list_trading_pair_configs(
        Exchange.BINANCE.value
    )

    # 验证结果
    assert len(result) == 2
    assert result[0].symbol == "BTC-USDT"
    assert result[1].symbol == "ETH-USDT"


@pytest.mark.asyncio
async def test_save_and_get_strategy_config(postgresql_service):
    """测试保存和获取策略配置"""
    # 准备测试数据
    config = StrategyConfig(
        name="test_strategy",
        description="Test strategy",
        parameters={
            "param1": "value1",
            "param2": 123,
            "param3": True
        },
        is_active=True
    )

    # Mock连接池
    mock_pool = AsyncMock(spec=Pool)
    postgresql_service.pool = mock_pool

    # Mock连接
    mock_conn = AsyncMock(spec=Connection)
    mock_pool.acquire.return_value.__aenter__.return_value = mock_conn

    # Mock查询结果
    mock_conn.fetchrow.return_value = {
        "name": "test_strategy",
        "description": "Test strategy",
        "parameters": json.dumps({
            "param1": "value1",
            "param2": 123,
            "param3": True
        }),
        "is_active": True
    }

    # 保存数据
    await postgresql_service.save_strategy_config(config)

    # 验证保存调用
    mock_conn.execute.assert_called_once()

    # 获取数据
    result = await postgresql_service.get_strategy_config("test_strategy")

    # 验证结果
    assert result == config


@pytest.mark.asyncio
async def test_list_strategy_configs(postgresql_service):
    """测试获取策略配置列表"""
    # Mock连接池
    mock_pool = AsyncMock(spec=Pool)
    postgresql_service.pool = mock_pool

    # Mock连接
    mock_conn = AsyncMock(spec=Connection)
    mock_pool.acquire.return_value.__aenter__.return_value = mock_conn

    # Mock查询结果
    mock_conn.fetch.return_value = [
        {
            "name": "test_strategy_1",
            "description": "Test strategy 1",
            "parameters": json.dumps({
                "param1": "value1",
                "param2": 123
            }),
            "is_active": True
        },
        {
            "name": "test_strategy_2",
            "description": "Test strategy 2",
            "parameters": json.dumps({
                "param1": "value2",
                "param2": 456
            }),
            "is_active": False
        }
    ]

    # 获取数据
    result = await postgresql_service.list_strategy_configs()

    # 验证结果
    assert len(result) == 2
    assert result[0].name == "test_strategy_1"
    assert result[1].name == "test_strategy_2"


@pytest.mark.asyncio
async def test_save_and_get_system_config(postgresql_service):
    """测试保存和获取系统配置"""
    # Mock连接池
    mock_pool = AsyncMock(spec=Pool)
    postgresql_service.pool = mock_pool

    # Mock连接
    mock_conn = AsyncMock(spec=Connection)
    mock_pool.acquire.return_value.__aenter__.return_value = mock_conn

    # Mock查询结果
    mock_conn.fetchrow.return_value = {
        "key": "test_key",
        "value": json.dumps("test_value"),
        "description": "Test config"
    }

    # 保存数据
    await postgresql_service.save_system_config(
        "test_key",
        "test_value",
        "Test config"
    )

    # 验证保存调用
    mock_conn.execute.assert_called_once()

    # 获取数据
    result = await postgresql_service.get_system_config("test_key")

    # 验证结果
    assert result.key == "test_key"
    assert result.value == "test_value"
    assert result.description == "Test config"


@pytest.mark.asyncio
async def test_list_system_configs(postgresql_service):
    """测试获取系统配置列表"""
    # Mock连接池
    mock_pool = AsyncMock(spec=Pool)
    postgresql_service.pool = mock_pool

    # Mock连接
    mock_conn = AsyncMock(spec=Connection)
    mock_pool.acquire.return_value.__aenter__.return_value = mock_conn

    # Mock查询结果
    mock_conn.fetch.return_value = [
        {
            "key": "test_key_1",
            "value": json.dumps("test_value_1"),
            "description": "Test config 1"
        },
        {
            "key": "test_key_2",
            "value": json.dumps(123),
            "description": "Test config 2"
        }
    ]

    # 获取数据
    result = await postgresql_service.list_system_configs()

    # 验证结果
    assert len(result) == 2
    assert result[0].key == "test_key_1"
    assert result[1].key == "test_key_2" 