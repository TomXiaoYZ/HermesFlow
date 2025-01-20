"""
WebSocket客户端测试。
"""
import pytest
import asyncio
from unittest.mock import AsyncMock, patch
from async_generator import async_generator, yield_
from src.backend.data_service.exchanges.binance.websocket import BinanceWebsocketClient

@pytest.fixture
@async_generator
async def ws_client():
    """WebSocket客户端fixture"""
    client = BinanceWebsocketClient(testnet=True)
    await client.start()
    await yield_(client)
    await client.stop()

@pytest.mark.asyncio
async def test_heartbeat_detection(ws_client):
    """测试心跳检测功能"""
    client = await anext(ws_client)
    
    # 等待连接建立
    await asyncio.sleep(1)
    
    # 检查心跳任务是否在运行
    assert client._heartbeat_task is not None
    assert not client._heartbeat_task.done()
    
    # 检查连接状态
    assert client.running
    assert client.connected.is_set()
    
    # 检查ping/pong时间
    assert client._last_ping_time > 0
    assert client._last_pong_time > 0

@pytest.mark.asyncio
async def test_reconnection_with_backoff(ws_client):
    """测试断线重连功能"""
    client = await anext(ws_client)
    
    # 等待连接建立
    await asyncio.sleep(1)
    
    # 模拟连接断开
    await client.ws.close()
    
    # 等待重连
    await asyncio.sleep(6)
    
    # 检查重连状态
    assert client.running
    assert client.connected.is_set()
    assert client._reconnect_count == 0

@pytest.mark.asyncio
async def test_max_reconnection_limit(ws_client):
    """测试最大重连次数限制"""
    client = await anext(ws_client)
    
    # 等待连接建立
    await asyncio.sleep(1)
    
    # 保存原始的_connect方法
    original_connect = client._connect
    
    # 模拟连接失败
    async def mock_connect():
        raise Exception("连接失败")
    
    # 替换_connect方法
    client._connect = mock_connect
    
    # 模拟连接断开
    await client.ws.close()
    
    # 等待重连尝试
    await asyncio.sleep(30)
    
    # 检查重连次数和状态
    assert not client.running
    assert not client.connected.is_set()
    assert client._reconnect_count > client._max_reconnect_count
    
    # 恢复原始的_connect方法
    client._connect = original_connect

@pytest.mark.asyncio
async def test_resource_cleanup(ws_client):
    """测试资源清理"""
    client = await anext(ws_client)
    
    # 等待连接建立
    await asyncio.sleep(1)
    
    # 检查初始状态
    assert client.running
    assert client.connected.is_set()
    assert client._heartbeat_task is not None
    assert client._message_task is not None
    
    # 停止客户端
    await client.stop()
    
    # 检查清理状态
    assert not client.running
    assert not client.connected.is_set()
    assert client._heartbeat_task.done()
    assert client._message_task.done()
    assert client.ws is None or client.ws.closed
    assert client.session is None or client.session.closed 