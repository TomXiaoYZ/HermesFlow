import pytest
import asyncio
from typing import Dict, Any
from unittest.mock import AsyncMock, patch

from src.backend.data_service.exchanges.okx.websocket import OKXWebSocket
from src.backend.data_service.exchanges.okx.exceptions import OKXWebSocketError

@pytest.fixture
async def ws_client():
    """创建WebSocket测试客户端"""
    client = OKXWebSocket(
        api_key="test_key",
        api_secret="test_secret",
        passphrase="test_pass",
        testnet=True
    )
    yield client
    await client.close()

@pytest.mark.asyncio
async def test_connect(ws_client):
    """测试WebSocket连接"""
    await ws_client.connect()
    assert ws_client.is_connected()
    await ws_client.close()
    assert not ws_client.is_connected()

@pytest.mark.asyncio
async def test_subscribe_ticker(ws_client):
    """测试订阅Ticker数据"""
    received_data = []
    
    async def on_ticker(data: Dict[str, Any]):
        received_data.append(data)
    
    await ws_client.connect()
    await ws_client.subscribe(
        topic="tickers",
        symbol="BTC-USDT",
        callback=on_ticker
    )
    
    # 等待接收数据
    await asyncio.sleep(5)
    assert len(received_data) > 0
    
    # 验证数据格式
    ticker = received_data[0]
    assert "last" in ticker
    assert "vol24h" in ticker
    assert "volCcy24h" in ticker

@pytest.mark.asyncio
async def test_subscribe_depth(ws_client):
    """测试订阅深度数据"""
    received_data = []
    
    async def on_depth(data: Dict[str, Any]):
        received_data.append(data)
    
    await ws_client.connect()
    await ws_client.subscribe(
        topic="books",
        symbol="BTC-USDT",
        callback=on_depth
    )
    
    await asyncio.sleep(5)
    assert len(received_data) > 0
    
    depth = received_data[0]
    assert "bids" in depth
    assert "asks" in depth

@pytest.mark.asyncio
async def test_heartbeat(ws_client):
    """测试心跳机制"""
    await ws_client.connect()
    await asyncio.sleep(30)  # 等待几个心跳周期
    assert ws_client.is_connected()

@pytest.mark.asyncio
async def test_reconnection(ws_client):
    """测试自动重连"""
    await ws_client.connect()
    
    # 模拟连接断开
    await ws_client._ws.close()
    await asyncio.sleep(5)  # 等待重连
    
    assert ws_client.is_connected()

@pytest.mark.asyncio
async def test_error_handling(ws_client):
    """测试错误处理"""
    with pytest.raises(OKXWebSocketError):
        await ws_client.subscribe(
            topic="invalid_topic",
            symbol="BTC-USDT",
            callback=AsyncMock()
        )

@pytest.mark.asyncio
async def test_multiple_subscriptions(ws_client):
    """测试多个订阅"""
    received_ticker = []
    received_depth = []
    
    async def on_ticker(data):
        received_ticker.append(data)
    
    async def on_depth(data):
        received_depth.append(data)
    
    await ws_client.connect()
    
    # 同时订阅多个主题
    await asyncio.gather(
        ws_client.subscribe("tickers", "BTC-USDT", on_ticker),
        ws_client.subscribe("books", "BTC-USDT", on_depth)
    )
    
    await asyncio.sleep(5)
    assert len(received_ticker) > 0
    assert len(received_depth) > 0

@pytest.mark.asyncio
async def test_unsubscribe(ws_client):
    """测试取消订阅"""
    received_data = []
    
    async def on_ticker(data):
        received_data.append(data)
    
    await ws_client.connect()
    await ws_client.subscribe("tickers", "BTC-USDT", on_ticker)
    await asyncio.sleep(2)
    
    # 记录当前接收到的数据量
    data_count = len(received_data)
    
    # 取消订阅
    await ws_client.unsubscribe("tickers", "BTC-USDT")
    await asyncio.sleep(2)
    
    # 验证不再接收新数据
    assert len(received_data) == data_count 