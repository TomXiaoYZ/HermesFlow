"""
Binance交易功能集成测试
测试订单创建、撤销和查询等交易功能
"""
import os
import pytest
import asyncio
from decimal import Decimal

from src.backend.data_service.exchanges.binance.client import BinanceAPI
from src.backend.data_service.exchanges.binance.websocket import BinanceWebsocketClient
from src.backend.data_service.common.models import Market, OrderType, OrderSide

@pytest.fixture
async def api_client():
    """创建API测试客户端"""
    api_key = os.getenv("BINANCE_API_KEY", "")
    api_secret = os.getenv("BINANCE_API_SECRET", "")
    client = BinanceAPI(api_key, api_secret, testnet=True)
    yield client

@pytest.fixture
async def ws_client():
    """创建WebSocket测试客户端"""
    api_key = os.getenv("BINANCE_API_KEY", "")
    api_secret = os.getenv("BINANCE_API_SECRET", "")
    client = BinanceWebsocketClient(api_key, api_secret, testnet=True)
    await client.start()
    yield client
    await client.stop()

@pytest.mark.asyncio
async def test_order_lifecycle(api_client, ws_client):
    """测试订单生命周期"""
    if not api_client.api_key or not api_client.api_secret:
        pytest.skip("需要API密钥才能测试")

    symbol = "BTCUSDT"
    order_updates = []

    def on_order_update(data):
        order_updates.append(data)

    await ws_client.subscribe_orders(Market.SPOT, on_order_update)
    await asyncio.sleep(1)

    # 创建限价单
    ticker = await api_client.get_ticker(Market.SPOT, symbol)
    price = float(ticker.price) * 0.9  # 下限价单，价格低于市场价
    quantity = 0.001

    order = await api_client.create_order(
        Market.SPOT,
        symbol,
        OrderType.LIMIT,
        OrderSide.BUY,
        price=price,
        quantity=quantity
    )

    await asyncio.sleep(2)
    assert len(order_updates) > 0
    
    # 验证订单创建
    update = next(u for u in order_updates if u.order_id == order.id)
    assert update.symbol == symbol
    assert update.side == OrderSide.BUY
    assert float(update.price) == price
    assert float(update.original_quantity) == quantity

    # 查询订单
    order_status = await api_client.get_order(Market.SPOT, symbol, order_id=order.id)
    assert order_status.id == order.id
    assert order_status.status in ["NEW", "PARTIALLY_FILLED"]

    # 取消订单
    await api_client.cancel_order(Market.SPOT, symbol, order_id=order.id)
    await asyncio.sleep(2)

    # 验证订单取消
    update = next(u for u in order_updates if u.order_id == order.id and u.status == "CANCELED")
    assert update is not None

@pytest.mark.asyncio
async def test_market_order(api_client, ws_client):
    """测试市价单功能"""
    if not api_client.api_key or not api_client.api_secret:
        pytest.skip("需要API密钥才能测试")

    symbol = "BTCUSDT"
    order_updates = []

    def on_order_update(data):
        order_updates.append(data)

    await ws_client.subscribe_orders(Market.SPOT, on_order_update)
    await asyncio.sleep(1)

    # 创建市价单
    quantity = 0.001
    order = await api_client.create_order(
        Market.SPOT,
        symbol,
        OrderType.MARKET,
        OrderSide.BUY,
        quantity=quantity
    )

    await asyncio.sleep(2)
    assert len(order_updates) > 0

    # 验证订单执行
    update = next(u for u in order_updates if u.order_id == order.id and u.status == "FILLED")
    assert update is not None
    assert float(update.executed_quantity) == quantity

@pytest.mark.asyncio
async def test_account_updates(api_client, ws_client):
    """测试账户更新"""
    if not api_client.api_key or not api_client.api_secret:
        pytest.skip("需要API密钥才能测试")

    balance_updates = []

    def on_balance_update(data):
        balance_updates.append(data)

    await ws_client.subscribe_account(Market.SPOT, on_balance_update)
    await asyncio.sleep(1)

    # 获取账户余额
    balances = await api_client.get_balances(Market.SPOT)
    assert len(balances) > 0

    # 创建一个小额市价单来触发余额更新
    symbol = "BTCUSDT"
    quantity = 0.001
    await api_client.create_order(
        Market.SPOT,
        symbol,
        OrderType.MARKET,
        OrderSide.BUY,
        quantity=quantity
    )

    await asyncio.sleep(2)
    assert len(balance_updates) > 0 