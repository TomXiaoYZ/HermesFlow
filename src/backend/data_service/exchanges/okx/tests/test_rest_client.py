import pytest
import asyncio
from datetime import datetime, timezone
from decimal import Decimal
from typing import Dict, Any

from src.backend.data_service.exchanges.okx.client import OKXRestClient
from src.backend.data_service.exchanges.okx.exceptions import OKXAPIError, OKXRequestError

@pytest.fixture
async def client():
    """创建测试客户端"""
    client = OKXRestClient(
        api_key="test_key",
        api_secret="test_secret",
        passphrase="test_pass",
        testnet=True
    )
    yield client
    await client.close()

@pytest.mark.asyncio
async def test_get_ticker(client):
    """测试获取Ticker数据"""
    ticker = await client.get_ticker(symbol="BTC-USDT")
    assert isinstance(ticker, Dict)
    assert "last" in ticker
    assert "vol24h" in ticker
    assert "volCcy24h" in ticker
    assert float(ticker["last"]) > 0

@pytest.mark.asyncio
async def test_get_depth(client):
    """测试获取深度数据"""
    depth = await client.get_depth(symbol="BTC-USDT", limit=20)
    assert isinstance(depth, Dict)
    assert "bids" in depth
    assert "asks" in depth
    assert len(depth["bids"]) <= 20
    assert len(depth["asks"]) <= 20

@pytest.mark.asyncio
async def test_get_trades(client):
    """测试获取最近成交"""
    trades = await client.get_trades(symbol="BTC-USDT", limit=10)
    assert isinstance(trades, list)
    assert len(trades) <= 10
    for trade in trades:
        assert "price" in trade
        assert "size" in trade
        assert "side" in trade
        assert "ts" in trade

@pytest.mark.asyncio
async def test_get_klines(client):
    """测试获取K线数据"""
    klines = await client.get_klines(
        symbol="BTC-USDT",
        interval="1m",
        limit=100
    )
    assert isinstance(klines, list)
    assert len(klines) <= 100
    for kline in klines:
        assert len(kline) == 6  # [timestamp, open, high, low, close, volume]
        assert all(isinstance(x, (int, float, str)) for x in kline)

@pytest.mark.asyncio
async def test_create_order(client):
    """测试创建订单"""
    order = await client.create_order(
        symbol="BTC-USDT",
        type="limit",
        side="buy",
        price=Decimal("50000"),
        size=Decimal("0.001")
    )
    assert isinstance(order, Dict)
    assert "ordId" in order
    assert "clOrdId" in order
    assert order["symbol"] == "BTC-USDT"

@pytest.mark.asyncio
async def test_cancel_order(client):
    """测试取消订单"""
    result = await client.cancel_order(
        symbol="BTC-USDT",
        order_id="test_order_id"
    )
    assert isinstance(result, Dict)
    assert "ordId" in result

@pytest.mark.asyncio
async def test_get_order(client):
    """测试获取订单信息"""
    order = await client.get_order(
        symbol="BTC-USDT",
        order_id="test_order_id"
    )
    assert isinstance(order, Dict)
    assert "ordId" in order
    assert "symbol" in order
    assert "state" in order

@pytest.mark.asyncio
async def test_get_open_orders(client):
    """测试获取未完成订单"""
    orders = await client.get_open_orders(symbol="BTC-USDT")
    assert isinstance(orders, list)
    for order in orders:
        assert "ordId" in order
        assert "symbol" in order
        assert order["state"] in ["live", "partially_filled"]

@pytest.mark.asyncio
async def test_error_handling(client):
    """测试错误处理"""
    with pytest.raises(OKXRequestError):
        await client.get_order(
            symbol="BTC-USDT",
            order_id="invalid_order_id"
        )

@pytest.mark.asyncio
async def test_rate_limit(client):
    """测试频率限制处理"""
    tasks = []
    for _ in range(100):
        tasks.append(client.get_ticker(symbol="BTC-USDT"))
    
    results = await asyncio.gather(*tasks, return_exceptions=True)
    assert any(isinstance(x, OKXAPIError) for x in results) 