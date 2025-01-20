"""
Binance WebSocket客户端单元测试

测试覆盖:
1. 基础连接测试 - 连接建立、断开、心跳
2. 重连机制测试 - 自动重连、最大重试
3. 订阅管理测试 - 订阅、取消订阅、订阅状态
4. 消息处理测试 - 单个处理器、多处理器、错误处理
5. 业务场景测试 - 行情数据、交易数据、错误恢复
"""
import pytest
import asyncio
import json
from unittest.mock import AsyncMock, Mock, patch
from src.backend.data_service.binance.websocket_client import BinanceWebsocketClient

@pytest.fixture
def mock_websocket():
    return AsyncMock()

@pytest.fixture
async def ws_client():
    client = BinanceWebsocketClient()
    yield client
    await client.close()

# 基础连接测试
@pytest.mark.asyncio
async def test_connect(ws_client, mock_websocket):
    """测试WebSocket连接建立"""
    with patch('websockets.connect', return_value=mock_websocket):
        await ws_client.connect()
        assert ws_client.ws == mock_websocket
        assert ws_client.connected is True

@pytest.mark.asyncio
async def test_disconnect(ws_client, mock_websocket):
    """测试WebSocket连接断开"""
    with patch('websockets.connect', return_value=mock_websocket):
        await ws_client.connect()
        await ws_client.close()
        assert ws_client.ws is None
        assert ws_client.connected is False
        mock_websocket.close.assert_called_once()

# 心跳和重连测试
@pytest.mark.asyncio
async def test_heartbeat(ws_client, mock_websocket):
    """测试心跳机制"""
    with patch('websockets.connect', return_value=mock_websocket):
        await ws_client.connect()
        # 模拟发送ping消息
        await ws_client._send_ping()
        mock_websocket.ping.assert_called_once()

@pytest.mark.asyncio
async def test_heartbeat_failure(ws_client, mock_websocket):
    """测试心跳失败场景"""
    with patch('websockets.connect', return_value=mock_websocket):
        await ws_client.connect()
        mock_websocket.ping.side_effect = ConnectionError()
        await ws_client._send_ping()
        assert ws_client.connected is False

@pytest.mark.asyncio
async def test_reconnect(ws_client, mock_websocket):
    """测试重连机制"""
    with patch('websockets.connect', side_effect=[
        mock_websocket,
        ConnectionError,
        mock_websocket
    ]):
        await ws_client.connect()
        # 模拟连接断开
        mock_websocket.close.side_effect = ConnectionError()
        await ws_client.close()
        # 测试重连
        await ws_client.connect()
        assert ws_client.connected is True
        assert ws_client.reconnect_attempts == 1

@pytest.mark.asyncio
async def test_max_reconnect_attempts(ws_client, mock_websocket):
    """测试达到最大重连次数"""
    with patch('websockets.connect', side_effect=ConnectionError):
        for _ in range(ws_client.max_reconnect_attempts + 1):
            await ws_client.connect()
        assert ws_client.reconnect_attempts == ws_client.max_reconnect_attempts
        assert not ws_client.connected

# 订阅相关测试
@pytest.mark.asyncio
async def test_subscribe(ws_client, mock_websocket):
    """测试订阅功能"""
    with patch('websockets.connect', return_value=mock_websocket):
        await ws_client.connect()
        channel = "btcusdt@trade"
        await ws_client.subscribe(channel)
        expected_message = {
            "method": "SUBSCRIBE",
            "params": [channel],
            "id": 1
        }
        mock_websocket.send.assert_called_with(json.dumps(expected_message))
        assert channel in ws_client._subscriptions

@pytest.mark.asyncio
async def test_subscribe_without_connection(ws_client):
    """测试未连接时订阅"""
    with pytest.raises(ConnectionError):
        await ws_client.subscribe("btcusdt@trade")

@pytest.mark.asyncio
async def test_subscribe_invalid_channel(ws_client, mock_websocket):
    """测试订阅无效的频道"""
    with patch('websockets.connect', return_value=mock_websocket):
        await ws_client.connect()
        with pytest.raises(ValueError):
            await ws_client.subscribe("")

@pytest.mark.asyncio
async def test_unsubscribe(ws_client, mock_websocket):
    """测试取消订阅功能"""
    with patch('websockets.connect', return_value=mock_websocket):
        await ws_client.connect()
        channel = "btcusdt@trade"
        await ws_client.subscribe(channel)
        await ws_client.unsubscribe(channel)
        expected_message = {
            "method": "UNSUBSCRIBE",
            "params": [channel],
            "id": 2
        }
        mock_websocket.send.assert_called_with(json.dumps(expected_message))
        assert channel not in ws_client._subscriptions

# 消息处理测试
@pytest.mark.asyncio
async def test_message_handler(ws_client, mock_websocket):
    """测试消息处理"""
    test_message = {
        "e": "trade",
        "s": "BTCUSDT",
        "p": "50000.00",
        "q": "0.001"
    }
    
    message_received = asyncio.Event()
    
    async def message_callback(msg):
        assert msg == test_message
        message_received.set()
    
    with patch('websockets.connect', return_value=mock_websocket):
        await ws_client.connect()
        ws_client.add_message_handler(message_callback)
        
        # 模拟接收消息
        mock_websocket.recv.return_value = json.dumps(test_message)
        await ws_client._handle_message(test_message)
        
        # 等待消息处理完成
        await asyncio.wait_for(message_received.wait(), timeout=1)

@pytest.mark.asyncio
async def test_multiple_message_handlers(ws_client, mock_websocket):
    """测试多个消息处理器"""
    test_message = {"e": "trade"}
    handlers_called = [asyncio.Event(), asyncio.Event()]
    
    async def handler1(msg):
        handlers_called[0].set()
        
    async def handler2(msg):
        handlers_called[1].set()
    
    with patch('websockets.connect', return_value=mock_websocket):
        await ws_client.connect()
        ws_client.add_message_handler(handler1)
        ws_client.add_message_handler(handler2)
        
        await ws_client._handle_message(test_message)
        
        # 验证所有处理器都被调用
        await asyncio.gather(
            asyncio.wait_for(handlers_called[0].wait(), timeout=1),
            asyncio.wait_for(handlers_called[1].wait(), timeout=1)
        )

@pytest.mark.asyncio
async def test_invalid_message_format(ws_client, mock_websocket):
    """测试无效消息格式"""
    with patch('websockets.connect', return_value=mock_websocket):
        await ws_client.connect()
        mock_websocket.recv.return_value = "invalid json"
        
        # 启动消息监听
        listen_task = asyncio.create_task(ws_client.start_listening())
        await asyncio.sleep(0.1)  # 给一点时间处理消息
        
        # 验证客户端仍然保持连接
        assert ws_client.connected
        listen_task.cancel()

# 错误处理测试
@pytest.mark.asyncio
async def test_error_handling(ws_client, mock_websocket):
    """测试错误处理"""
    with patch('websockets.connect', return_value=mock_websocket):
        await ws_client.connect()
        
        # 模拟网络错误
        mock_websocket.send.side_effect = ConnectionError()
        
        with pytest.raises(ConnectionError):
            await ws_client.subscribe("btcusdt@trade")
        
        assert ws_client.connected is False

@pytest.mark.asyncio
async def test_connection_timeout(ws_client):
    """测试连接超时"""
    with patch('websockets.connect', side_effect=asyncio.TimeoutError):
        with pytest.raises(asyncio.TimeoutError):
            await ws_client.connect()
        assert not ws_client.connected

@pytest.mark.asyncio
async def test_cleanup_on_close(ws_client, mock_websocket):
    """测试关闭时的清理操作"""
    with patch('websockets.connect', return_value=mock_websocket):
        await ws_client.connect()
        await ws_client.subscribe("btcusdt@trade")
        
        # 关闭连接
        await ws_client.close()
        
        # 验证状态重置
        assert ws_client.ws is None
        assert not ws_client.connected
        assert not ws_client._subscriptions
        assert ws_client._heartbeat_task is None

# 业务场景测试
@pytest.mark.asyncio
async def test_market_depth_subscription(ws_client, mock_websocket):
    """测试订阅市场深度数据"""
    with patch('websockets.connect', return_value=mock_websocket):
        await ws_client.connect()
        
        # 模拟深度数据
        depth_data = {
            "e": "depthUpdate",
            "E": 123456789,
            "s": "BTCUSDT",
            "U": 157,
            "u": 160,
            "b": [["0.0024", "10"]],
            "a": [["0.0026", "100"]]
        }
        
        data_received = asyncio.Event()
        
        async def depth_handler(msg):
            assert msg == depth_data
            assert msg["s"] == "BTCUSDT"
            assert len(msg["b"]) > 0  # 验证买单
            assert len(msg["a"]) > 0  # 验证卖单
            data_received.set()
        
        # 订阅深度数据
        await ws_client.subscribe("btcusdt@depth")
        ws_client.add_message_handler(depth_handler)
        
        # 模拟接收数据
        mock_websocket.recv.return_value = json.dumps(depth_data)
        await ws_client._handle_message(depth_data)
        
        # 验证数据处理
        await asyncio.wait_for(data_received.wait(), timeout=1)

@pytest.mark.asyncio
async def test_trade_data_subscription(ws_client, mock_websocket):
    """测试订阅交易数据"""
    with patch('websockets.connect', return_value=mock_websocket):
        await ws_client.connect()
        
        # 模拟交易数据
        trade_data = {
            "e": "trade",
            "E": 123456789,
            "s": "BTCUSDT",
            "t": 12345,
            "p": "50000.00",
            "q": "0.001",
            "b": 88,
            "a": 50,
            "T": 123456785,
            "m": True,
            "M": True
        }
        
        data_received = asyncio.Event()
        
        async def trade_handler(msg):
            assert msg == trade_data
            assert msg["s"] == "BTCUSDT"
            assert float(msg["p"]) > 0  # 验证价格
            assert float(msg["q"]) > 0  # 验证数量
            data_received.set()
        
        # 订阅交易数据
        await ws_client.subscribe("btcusdt@trade")
        ws_client.add_message_handler(trade_handler)
        
        # 模拟接收数据
        mock_websocket.recv.return_value = json.dumps(trade_data)
        await ws_client._handle_message(trade_data)
        
        # 验证数据处理
        await asyncio.wait_for(data_received.wait(), timeout=1)

@pytest.mark.asyncio
async def test_kline_data_subscription(ws_client, mock_websocket):
    """测试订阅K线数据"""
    with patch('websockets.connect', return_value=mock_websocket):
        await ws_client.connect()
        
        # 模拟K线数据
        kline_data = {
            "e": "kline",
            "E": 123456789,
            "s": "BTCUSDT",
            "k": {
                "t": 123400000,
                "T": 123460000,
                "s": "BTCUSDT",
                "i": "1m",
                "f": 100,
                "L": 200,
                "o": "50000.00",
                "c": "50010.00",
                "h": "50100.00",
                "l": "49900.00",
                "v": "100.00",
                "n": 100,
                "x": False,
                "q": "5000000.00",
                "V": "50.00",
                "Q": "2500000.00",
                "B": "123456"
            }
        }
        
        data_received = asyncio.Event()
        
        async def kline_handler(msg):
            assert msg == kline_data
            assert msg["s"] == "BTCUSDT"
            assert msg["k"]["i"] == "1m"  # 验证时间周期
            assert float(msg["k"]["c"]) > float(msg["k"]["o"])  # 验证收盘价大于开盘价
            data_received.set()
        
        # 订阅K线数据
        await ws_client.subscribe("btcusdt@kline_1m")
        ws_client.add_message_handler(kline_handler)
        
        # 模拟接收数据
        mock_websocket.recv.return_value = json.dumps(kline_data)
        await ws_client._handle_message(kline_data)
        
        # 验证数据处理
        await asyncio.wait_for(data_received.wait(), timeout=1)

@pytest.mark.asyncio
async def test_multiple_symbols_subscription(ws_client, mock_websocket):
    """测试订阅多个交易对"""
    with patch('websockets.connect', return_value=mock_websocket):
        await ws_client.connect()
        
        symbols = ["btcusdt", "ethusdt", "bnbusdt"]
        channels = [f"{symbol}@trade" for symbol in symbols]
        
        # 订阅多个交易对
        for channel in channels:
            await ws_client.subscribe(channel)
            assert channel in ws_client._subscriptions
        
        # 验证订阅消息
        expected_message = {
            "method": "SUBSCRIBE",
            "params": channels,
            "id": 1
        }
        
        # 验证所有交易对都在订阅列表中
        for channel in channels:
            assert channel in ws_client._subscriptions

@pytest.mark.asyncio
async def test_subscription_recovery_after_reconnect(ws_client, mock_websocket):
    """测试重连后恢复订阅"""
    with patch('websockets.connect', return_value=mock_websocket):
        await ws_client.connect()
        
        # 初始订阅
        channels = ["btcusdt@trade", "ethusdt@trade"]
        for channel in channels:
            await ws_client.subscribe(channel)
        
        # 模拟断开连接
        await ws_client.close()
        
        # 重新连接
        await ws_client.connect()
        
        # 验证所有之前的订阅都被恢复
        for channel in channels:
            assert channel in ws_client._subscriptions
            
        # 验证重新订阅的消息
        expected_message = {
            "method": "SUBSCRIBE",
            "params": channels,
            "id": mock_websocket.send.call_count
        }
        mock_websocket.send.assert_called_with(json.dumps(expected_message))

@pytest.mark.asyncio
async def test_high_frequency_message_handling(ws_client, mock_websocket):
    """测试高频消息处理"""
    with patch('websockets.connect', return_value=mock_websocket):
        await ws_client.connect()
        
        message_count = 1000
        processed_count = 0
        processing_complete = asyncio.Event()
        
        async def message_handler(msg):
            nonlocal processed_count
            processed_count += 1
            if processed_count == message_count:
                processing_complete.set()
        
        ws_client.add_message_handler(message_handler)
        
        # 模拟高频消息
        tasks = []
        for i in range(message_count):
            message = {"e": "trade", "E": i}
            tasks.append(ws_client._handle_message(message))
        
        # 等待所有消息处理完成
        await asyncio.gather(*tasks)
        await asyncio.wait_for(processing_complete.wait(), timeout=5)
        
        assert processed_count == message_count 