"""
Binance API客户端测试
"""
import os
import pytest
from datetime import datetime, timedelta
from decimal import Decimal

from ....common.models import Market, OrderType, OrderSide
from .client import BinanceAPI

@pytest.fixture
def client():
    """创建测试客户端"""
    api_key = os.getenv("BINANCE_API_KEY", "")
    api_secret = os.getenv("BINANCE_API_SECRET", "")
    return BinanceAPI(api_key, api_secret, testnet=True)

@pytest.mark.asyncio
async def test_get_symbols(client):
    """测试获取交易对信息"""
    symbols = await client.get_symbols(Market.SPOT)
    assert len(symbols) > 0
    
    btcusdt = next(s for s in symbols if s.base_asset == "BTC" and s.quote_asset == "USDT")
    assert btcusdt.exchange == "binance"
    assert btcusdt.market == Market.SPOT
    assert btcusdt.status == "trading"
    assert btcusdt.min_price > 0
    assert btcusdt.max_price > btcusdt.min_price
    assert btcusdt.tick_size > 0
    assert btcusdt.min_qty > 0
    assert btcusdt.max_qty > btcusdt.min_qty
    assert btcusdt.step_size > 0
    assert btcusdt.min_notional > 0

@pytest.mark.asyncio
async def test_get_ticker(client):
    """测试获取行情数据"""
    ticker = await client.get_ticker(Market.SPOT, "BTCUSDT")
    assert ticker.exchange == "binance"
    assert ticker.market == Market.SPOT
    assert ticker.symbol == "BTCUSDT"
    assert ticker.price > 0
    assert ticker.volume > 0
    assert ticker.amount > 0
    assert isinstance(ticker.timestamp, datetime)
    assert ticker.bid_price > 0
    assert ticker.bid_qty > 0
    assert ticker.ask_price > 0
    assert ticker.ask_qty > 0
    assert ticker.open_price > 0
    assert ticker.high_price > 0
    assert ticker.low_price > 0
    assert ticker.close_price > 0

@pytest.mark.asyncio
async def test_get_order_book(client):
    """测试获取订单簿数据"""
    order_book = await client.get_order_book(Market.SPOT, "BTCUSDT", limit=10)
    assert order_book.exchange == "binance"
    assert order_book.market == Market.SPOT
    assert order_book.symbol == "BTCUSDT"
    assert isinstance(order_book.timestamp, datetime)
    assert len(order_book.bids) == 10
    assert len(order_book.asks) == 10
    assert order_book.update_id > 0

    # 验证订单簿价格排序
    assert all(order_book.bids[i]["price"] >= order_book.bids[i+1]["price"] 
              for i in range(len(order_book.bids)-1))
    assert all(order_book.asks[i]["price"] <= order_book.asks[i+1]["price"] 
              for i in range(len(order_book.asks)-1))

@pytest.mark.asyncio
async def test_get_recent_trades(client):
    """测试获取最近成交"""
    trades = await client.get_recent_trades(Market.SPOT, "BTCUSDT", limit=10)
    assert len(trades) == 10
    
    for trade in trades:
        assert trade.exchange == "binance"
        assert trade.market == Market.SPOT
        assert trade.symbol == "BTCUSDT"
        assert trade.id
        assert trade.price > 0
        assert trade.quantity > 0
        assert trade.amount > 0
        assert isinstance(trade.timestamp, datetime)
        assert isinstance(trade.is_buyer_maker, bool)
        assert trade.side in [OrderSide.BUY, OrderSide.SELL]

@pytest.mark.asyncio
async def test_get_klines(client):
    """测试获取K线数据"""
    end_time = datetime.now()
    start_time = end_time - timedelta(days=1)
    
    klines = await client.get_klines(
        Market.SPOT,
        "BTCUSDT",
        interval="1h",
        start_time=start_time,
        end_time=end_time,
        limit=24
    )
    
    assert len(klines) > 0
    for kline in klines:
        assert kline.exchange == "binance"
        assert kline.market == Market.SPOT
        assert kline.symbol == "BTCUSDT"
        assert kline.interval == "1h"
        assert isinstance(kline.open_time, datetime)
        assert isinstance(kline.close_time, datetime)
        assert kline.open_price > 0
        assert kline.high_price >= kline.open_price
        assert kline.low_price <= kline.open_price
        assert kline.close_price > 0
        assert kline.volume >= 0
        assert kline.amount >= 0
        assert kline.trades_count >= 0

@pytest.mark.asyncio
async def test_get_balances(client):
    """测试获取账户余额"""
    if not client.api_key or not client.api_secret:
        pytest.skip("需要API密钥才能测试")
    
    balances = await client.get_balances()
    assert len(balances) > 0
    
    for balance in balances:
        assert balance.exchange == "binance"
        assert balance.asset
        assert balance.free >= 0
        assert balance.locked >= 0
        assert balance.total == balance.free + balance.locked
        assert isinstance(balance.timestamp, datetime)

@pytest.mark.asyncio
async def test_order_lifecycle(client):
    """测试订单生命周期"""
    if not client.api_key or not client.api_secret:
        pytest.skip("需要API密钥才能测试")
    
    # 创建限价买单
    symbol = "BTCUSDT"
    ticker = await client.get_ticker(Market.SPOT, symbol)
    price = float(ticker.price) * 0.9  # 下单价格比当前价格低10%
    quantity = 0.001
    
    order = await client.create_order(
        Market.SPOT,
        symbol,
        OrderType.LIMIT,
        OrderSide.BUY,
        price=price,
        quantity=quantity
    )
    
    assert order.exchange == "binance"
    assert order.market == Market.SPOT
    assert order.symbol == symbol
    assert order.id
    assert order.client_order_id
    assert float(order.price) == price
    assert float(order.original_quantity) == quantity
    assert order.executed_quantity == 0
    assert order.remaining_quantity == order.original_quantity
    assert order.status == "new"
    assert order.type == OrderType.LIMIT
    assert order.side == OrderSide.BUY
    assert isinstance(order.created_at, datetime)
    assert isinstance(order.updated_at, datetime)
    assert order.is_working
    
    # 获取订单信息
    order = await client.get_order(Market.SPOT, symbol, order_id=order.id)
    assert order.id
    assert order.status in ["new", "partially_filled"]
    
    # 获取未完成订单
    open_orders = await client.get_open_orders(Market.SPOT, symbol)
    assert len(open_orders) > 0
    assert any(o.id == order.id for o in open_orders)
    
    # 取消订单
    canceled_order = await client.cancel_order(Market.SPOT, symbol, order_id=order.id)
    assert canceled_order.id == order.id
    assert canceled_order.status == "canceled"
    assert not canceled_order.is_working
    
    # 获取订单成交记录
    trades = await client.get_order_trades(Market.SPOT, symbol, order.id)
    assert isinstance(trades, list) 