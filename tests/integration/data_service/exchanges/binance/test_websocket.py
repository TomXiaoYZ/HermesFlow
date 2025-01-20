"""
Binance WebSocket集成测试
测试WebSocket连接管理、重连和错误处理功能
"""
import os
import pytest
import asyncio
import aiohttp
from datetime import datetime

from src.backend.data_service.exchanges.binance.websocket import BinanceWebsocketClient, BinanceWebSocketError
from src.backend.data_service.common.models import Market

@pytest.fixture
async def ws_client():
    """创建WebSocket测试客户端"""
    api_key = os.getenv("BINANCE_API_KEY", "")
    api_secret = os.getenv("BINANCE_API_SECRET", "")
    client = BinanceWebsocketClient(api_key, api_secret, testnet=True)
    yield client
    await client.stop()

@pytest.mark.asyncio
async def test_connection_lifecycle(ws_client):
    """测试WebSocket连接生命周期"""
    # 测试连接建立
    await ws_client.start()
    assert ws_client.running
    assert ws_client.connected.is_set()
    
    # 测试正常断开
    await ws_client.stop()
    assert not ws_client.running
    assert not ws_client.connected.is_set()
    
    # 测试重新连接
    await ws_client.start()
    assert ws_client.running
    assert ws_client.connected.is_set()

@pytest.mark.asyncio
async def test_auto_reconnection(ws_client):
    """测试自动重连功能"""
    await ws_client.start()
    assert ws_client.connected.is_set()
    
    # 模拟连接断开
    await ws_client.ws.close()
    await asyncio.sleep(5)  # 等待自动重连
    
    assert ws_client.running
    assert ws_client.connected.is_set()
    
    # 验证重连后数据流是否正常
    received_data = []
    def on_ticker(data):
        received_data.append(data)
    
    await ws_client.subscribe_ticker(Market.SPOT, "BTCUSDT", on_ticker)
    await asyncio.sleep(5)
    
    assert len(received_data) > 0

@pytest.mark.asyncio
async def test_subscription_recovery(ws_client):
    """测试订阅恢复功能"""
    await ws_client.start()
    
    # 初始订阅
    received_data = []
    def on_ticker(data):
        received_data.append(data)
    
    symbol = "BTCUSDT"
    await ws_client.subscribe_ticker(Market.SPOT, symbol, on_ticker)
    await asyncio.sleep(2)
    
    initial_count = len(received_data)
    assert initial_count > 0
    
    # 模拟连接断开并重连
    await ws_client.ws.close()
    await asyncio.sleep(5)  # 等待自动重连
    
    # 验证订阅是否自动恢复
    await asyncio.sleep(2)
    assert len(received_data) > initial_count

@pytest.mark.asyncio
async def test_error_handling(ws_client):
    """测试错误处理"""
    await ws_client.start()
    
    # 测试无效的订阅
    with pytest.raises(BinanceWebSocketError):
        await ws_client.subscribe_ticker(Market.SPOT, "INVALID_SYMBOL", lambda x: None)
    
    # 测试无效的市场类型
    with pytest.raises(ValueError):
        await ws_client.subscribe_ticker("INVALID_MARKET", "BTCUSDT", lambda x: None)
    
    # 测试网络错误处理
    await ws_client.ws.close()
    with pytest.raises(BinanceWebSocketError):
        await ws_client.subscribe_ticker(Market.SPOT, "BTCUSDT", lambda x: None)

@pytest.mark.asyncio
async def test_multiple_subscriptions(ws_client):
    """测试多订阅管理"""
    await ws_client.start()
    
    symbols = ["BTCUSDT", "ETHUSDT", "BNBUSDT"]
    received_data = {symbol: [] for symbol in symbols}
    
    # 订阅多个交易对
    for symbol in symbols:
        def make_handler(s):
            return lambda data: received_data[s].append(data)
        await ws_client.subscribe_ticker(Market.SPOT, symbol, make_handler(symbol))
    
    await asyncio.sleep(5)
    
    # 验证所有订阅都收到数据
    for symbol in symbols:
        assert len(received_data[symbol]) > 0
    
    # 取消部分订阅
    await ws_client.unsubscribe_ticker(Market.SPOT, symbols[0])
    initial_counts = {s: len(received_data[s]) for s in symbols[1:]}
    
    await asyncio.sleep(5)
    
    # 验证取消订阅的不再收到数据，其他继续收到
    assert len(received_data[symbols[0]]) == initial_counts.get(symbols[0], 0)
    for symbol in symbols[1:]:
        assert len(received_data[symbol]) > initial_counts[symbol] 