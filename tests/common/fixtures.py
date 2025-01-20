"""
测试固件
提供通用的测试准备和清理功能
"""
import os
import pytest
import asyncio
from typing import Dict, Any

from src.backend.data_service.exchanges.binance.client import BinanceAPI
from src.backend.data_service.exchanges.binance.websocket import BinanceWebsocketClient
from src.backend.data_service.exchanges.okx.client import OKXAPI
from src.backend.data_service.exchanges.okx.websocket import OKXWebSocket
from src.backend.data_service.exchanges.bitget.client import BitgetAPI
from src.backend.data_service.exchanges.bitget.websocket import BitgetWebSocket

from .test_config import get_exchange_config

@pytest.fixture
def event_loop():
    """创建事件循环"""
    loop = asyncio.get_event_loop_policy().new_event_loop()
    yield loop
    loop.close()

@pytest.fixture
async def binance_api():
    """创建Binance API客户端"""
    config = get_exchange_config("binance")
    client = BinanceAPI(
        api_key=os.getenv(config["api_key_env"], ""),
        api_secret=os.getenv(config["api_secret_env"], ""),
        testnet=True
    )
    yield client

@pytest.fixture
async def binance_ws():
    """创建Binance WebSocket客户端"""
    config = get_exchange_config("binance")
    client = BinanceWebsocketClient(
        api_key=os.getenv(config["api_key_env"], ""),
        api_secret=os.getenv(config["api_secret_env"], ""),
        testnet=True
    )
    await client.start()
    yield client
    await client.stop()

@pytest.fixture
async def okx_api():
    """创建OKX API客户端"""
    config = get_exchange_config("okx")
    client = OKXAPI(
        api_key=os.getenv(config["api_key_env"], ""),
        api_secret=os.getenv(config["api_secret_env"], ""),
        passphrase=os.getenv(config["passphrase_env"], ""),
        testnet=True
    )
    yield client

@pytest.fixture
async def okx_ws():
    """创建OKX WebSocket客户端"""
    config = get_exchange_config("okx")
    client = OKXWebSocket(
        api_key=os.getenv(config["api_key_env"], ""),
        api_secret=os.getenv(config["api_secret_env"], ""),
        passphrase=os.getenv(config["passphrase_env"], ""),
        testnet=True
    )
    await client.start()
    yield client
    await client.stop()

@pytest.fixture
async def bitget_api():
    """创建Bitget API客户端"""
    config = get_exchange_config("bitget")
    client = BitgetAPI(
        api_key=os.getenv(config["api_key_env"], ""),
        api_secret=os.getenv(config["api_secret_env"], ""),
        passphrase=os.getenv(config["passphrase_env"], ""),
        testnet=True
    )
    yield client

@pytest.fixture
async def bitget_ws():
    """创建Bitget WebSocket客户端"""
    config = get_exchange_config("bitget")
    client = BitgetWebSocket(
        api_key=os.getenv(config["api_key_env"], ""),
        api_secret=os.getenv(config["api_secret_env"], ""),
        passphrase=os.getenv(config["passphrase_env"], ""),
        testnet=True
    )
    await client.start()
    yield client
    await client.stop()

@pytest.fixture
def performance_metrics():
    """创建性能指标收集器"""
    from .test_utils import PerformanceMetrics
    return PerformanceMetrics()

@pytest.fixture
def resource_monitor():
    """创建资源监控器"""
    from .test_utils import ResourceMonitor
    return ResourceMonitor()

@pytest.fixture
async def database():
    """创建测试数据库连接"""
    from src.backend.data_service.db.connection import DatabaseManager
    db = DatabaseManager()
    await db.init()
    yield db
    await db.close()

@pytest.fixture
async def redis():
    """创建Redis连接"""
    import aioredis
    from .test_config import TEST_ENV
    
    redis = await aioredis.create_redis_pool(
        f'redis://{TEST_ENV["redis"]["host"]}:{TEST_ENV["redis"]["port"]}',
        db=TEST_ENV["redis"]["db"]
    )
    yield redis
    redis.close()
    await redis.wait_closed() 