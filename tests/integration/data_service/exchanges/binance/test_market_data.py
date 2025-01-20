"""
Binance市场数据集成测试
测试各种市场数据的获取和订阅功能
"""
import os
import pytest
import asyncio
from datetime import datetime, timedelta

from src.backend.data_service.exchanges.binance.client import BinanceAPI
from src.backend.data_service.exchanges.binance.websocket import BinanceWebsocketClient
from src.backend.data_service.common.models import Market

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
async def test_ticker_flow(api_client, ws_client):
    """测试行情数据流"""
    symbol = "BTCUSDT"
    received_data = []

    def on_ticker(data):
        received_data.append(data)
    
    await ws_client.subscribe_ticker(Market.SPOT, symbol, on_ticker)
    await asyncio.sleep(5)
    
    assert len(received_data) > 0
    ticker_ws = received_data[-1]
    assert ticker_ws.symbol == symbol
    assert ticker_ws.price > 0
    
    ticker_rest = await api_client.get_ticker(Market.SPOT, symbol)
    assert abs(float(ticker_ws.price) - float(ticker_rest.price)) < 10

@pytest.mark.asyncio
async def test_depth_flow(api_client, ws_client):
    """测试深度数据流"""
    symbol = "BTCUSDT"
    received_data = []

    def on_depth(data):
        received_data.append(data)
    
    await ws_client.subscribe_depth(Market.SPOT, symbol, on_depth)
    await asyncio.sleep(5)
    
    assert len(received_data) > 0
    depth_ws = received_data[-1]
    assert depth_ws.symbol == symbol
    assert len(depth_ws.bids) > 0
    assert len(depth_ws.asks) > 0
    
    depth_rest = await api_client.get_order_book(Market.SPOT, symbol)
    assert len(depth_rest.bids) > 0
    assert len(depth_rest.asks) > 0

@pytest.mark.asyncio
async def test_kline_flow(api_client, ws_client):
    """测试K线数据流"""
    symbol = "BTCUSDT"
    interval = "1m"
    received_data = []

    def on_kline(data):
        received_data.append(data)
    
    await ws_client.subscribe_kline(Market.SPOT, symbol, interval, on_kline)
    await asyncio.sleep(5)
    
    assert len(received_data) > 0
    kline_ws = received_data[-1]
    assert kline_ws.symbol == symbol
    assert kline_ws.interval == interval
    
    end_time = datetime.now()
    start_time = end_time - timedelta(minutes=10)
    klines_rest = await api_client.get_klines(
        Market.SPOT,
        symbol,
        interval,
        start_time=start_time,
        end_time=end_time
    )
    assert len(klines_rest) > 0 