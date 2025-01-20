"""
WebSocket功能测试
"""
import uuid
import pytest
import asyncio
from datetime import datetime
from typing import List

from ..exchanges.binance.websocket import BinanceWebsocketClient
from .service import OrderService
from .models import OrderRecord, OrderUpdate, TradeRecord
from .enums import OrderType, OrderSide, OrderStatus, TimeInForce, OrderUpdateType

@pytest.fixture
async def ws_client():
    """创建WebSocket客户端"""
    client = BinanceWebsocketClient(testnet=True)
    await client.start()
    yield client
    await client.stop()

@pytest.fixture
async def order_service():
    """创建订单服务实例"""
    service = OrderService(
        redis_url="redis://localhost:6379/0",
        postgres_dsn="postgresql://user:pass@localhost:5432/test",
        clickhouse_settings={
            "host": "localhost",
            "port": 9000,
            "database": "test",
            "user": "default",
            "password": ""
        }
    )
    await service.init()
    yield service
    await service.close()

@pytest.mark.asyncio
async def test_order_update_subscription(ws_client, order_service):
    """测试订单更新订阅"""
    # 创建测试订单
    order = OrderRecord(
        id=str(uuid.uuid4()),
        exchange="binance",
        exchange_order_id="123456",
        client_order_id="test123",
        symbol="BTCUSDT",
        type=OrderType.LIMIT,
        side=OrderSide.BUY,
        price=50000.0,
        quantity=1.0,
        executed_qty=0.0,
        avg_price=0.0,
        status=OrderStatus.NEW,
        time_in_force=TimeInForce.GTC,
        created_time=int(datetime.now().timestamp() * 1000),
        updated_time=int(datetime.now().timestamp() * 1000),
        is_contract=False,
        position_side=None,
        margin_type=None,
        leverage=None,
        stop_price=None,
        working_type=None,
        reduce_only=False
    )
    await order_service.create_order(order)
    
    # 订阅订单更新
    updates_received = []
    
    def handle_order_update(msg):
        updates_received.append(msg)
    
    ws_client.add_handler("executionReport", handle_order_update)
    await ws_client.subscribe([f"{order.symbol.lower()}@trade"])
    
    # 等待订单更新
    await asyncio.sleep(5)
    
    # 更新订单状态
    update = OrderUpdate(
        id=str(uuid.uuid4()),
        order_id=order.id,
        update_type=OrderUpdateType.STATUS,
        prev_status=OrderStatus.NEW,
        curr_status=OrderStatus.FILLED,
        executed_qty=1.0,
        remaining_qty=0.0,
        avg_price=50000.0,
        created_time=int(datetime.now().timestamp() * 1000),
        reason="Order filled"
    )
    await order_service.update_order(update)
    
    # 等待WebSocket消息
    await asyncio.sleep(5)
    
    # 验证是否收到订单更新
    assert len(updates_received) > 0
    assert updates_received[0]["s"] == order.symbol
    assert updates_received[0]["X"] == "FILLED"

@pytest.mark.asyncio
async def test_market_data_subscription(ws_client):
    """测试市场数据订阅"""
    # 订阅市场数据
    trades_received = []
    klines_received = []
    ticker_received = []
    
    def handle_trade(msg):
        trades_received.append(msg)
    
    def handle_kline(msg):
        klines_received.append(msg)
    
    def handle_ticker(msg):
        ticker_received.append(msg)
    
    # 添加处理器
    ws_client.add_handler("trade", handle_trade)
    ws_client.add_handler("kline", handle_kline)
    ws_client.add_handler("24hrTicker", handle_ticker)
    
    # 订阅数据流
    symbol = "btcusdt"
    streams = [
        f"{symbol}@trade",
        f"{symbol}@kline_1m",
        f"{symbol}@ticker"
    ]
    await ws_client.subscribe(streams)
    
    # 等待数据接收
    await asyncio.sleep(10)
    
    # 验证数据接收
    assert len(trades_received) > 0
    assert len(klines_received) > 0
    assert len(ticker_received) > 0
    
    # 验证数据格式
    trade = trades_received[0]
    assert "p" in trade  # 价格
    assert "q" in trade  # 数量
    assert "t" in trade  # 时间戳
    
    kline = klines_received[0]
    assert "k" in kline
    assert "o" in kline["k"]  # 开盘价
    assert "c" in kline["k"]  # 收盘价
    assert "v" in kline["k"]  # 成交量
    
    ticker = ticker_received[0]
    assert "c" in ticker  # 最新价
    assert "v" in ticker  # 成交量
    assert "p" in ticker  # 价格变动

@pytest.mark.asyncio
async def test_reconnection(ws_client):
    """测试自动重连"""
    # 订阅数据流
    symbol = "btcusdt"
    await ws_client.subscribe([f"{symbol}@trade"])
    
    # 等待连接建立
    await asyncio.sleep(2)
    
    # 模拟连接断开
    await ws_client.ws.close()
    
    # 等待重连
    await asyncio.sleep(10)
    
    # 验证重连成功
    assert ws_client.ws is not None
    assert ws_client.connected.is_set()

@pytest.mark.asyncio
async def test_error_handling(ws_client):
    """测试错误处理"""
    # 测试无效的订阅
    with pytest.raises(Exception):
        await ws_client.subscribe(["invalid_stream"])
    
    # 测试无效的取消订阅
    with pytest.raises(Exception):
        await ws_client.unsubscribe(["invalid_stream"])
    
    # 测试连接断开时的错误处理
    await ws_client.ws.close()
    with pytest.raises(Exception):
        await ws_client.subscribe(["btcusdt@trade"])
    
    # 等待重连
    await asyncio.sleep(5)
    
    # 验证重连后可以正常订阅
    await ws_client.subscribe(["btcusdt@trade"])
    assert ws_client.ws is not None 