"""
OKX API客户端测试
"""
import os
import pytest
from datetime import datetime, timedelta
from decimal import Decimal
import requests_mock
from unittest.mock import patch

from ....common.models import Market, OrderType, OrderSide
from ....common.exceptions import (
    APIError, NetworkError, ValidationError,
    AuthenticationError, PermissionError, RateLimitError
)
from .client import OKXAPI
from .exceptions import (
    OKXAPIError, OKXRequestError, OKXRateLimitError,
    OKXAuthError
)

@pytest.fixture
def client():
    """创建测试客户端"""
    api_key = os.getenv("OKX_API_KEY", "test_key")
    api_secret = os.getenv("OKX_API_SECRET", "test_secret")
    passphrase = os.getenv("OKX_PASSPHRASE", "test_passphrase")
    return OKXAPI(api_key, api_secret, passphrase, testnet=True)

def test_get_timestamp(client):
    """测试获取时间戳"""
    timestamp = client._get_timestamp()
    assert isinstance(timestamp, str)
    assert timestamp.endswith('Z')
    datetime.fromisoformat(timestamp[:-1])  # 验证格式是否正确

def test_sign(client):
    """测试签名生成"""
    timestamp = "2024-03-26T12:00:00.000Z"
    method = "GET"
    request_path = "/api/v5/market/ticker"
    body = ""
    
    signature = client._sign(timestamp, method, request_path, body)
    assert isinstance(signature, str)
    assert len(signature) > 0

@pytest.mark.asyncio
async def test_get_ticker(client):
    """测试获取行情数据"""
    with requests_mock.Mocker() as m:
        m.get(
            f"{client.base_url}/api/v5/market/ticker?instId=BTC-USDT",
            json={
                "code": "0",
                "data": [{
                    "instId": "BTC-USDT",
                    "last": "50000",
                    "vol24h": "1000",
                    "volCcy24h": "50000000",
                    "ts": "1616679000000",
                    "bidPx": "49999",
                    "bidSz": "1",
                    "askPx": "50001",
                    "askSz": "1",
                    "open24h": "49000",
                    "high24h": "51000",
                    "low24h": "48000"
                }]
            }
        )
        
        ticker = await client.get_ticker("BTC-USDT")
        assert ticker['instId'] == "BTC-USDT"
        assert ticker['last'] == "50000"

@pytest.mark.asyncio
async def test_get_depth(client):
    """测试获取深度数据"""
    with requests_mock.Mocker() as m:
        m.get(
            f"{client.base_url}/api/v5/market/books?instId=BTC-USDT&sz=100",
            json={
                "code": "0",
                "data": [{
                    "instId": "BTC-USDT",
                    "bids": [
                        ["50000", "1", "0", "1"],
                        ["49999", "2", "0", "1"]
                    ],
                    "asks": [
                        ["50001", "1", "0", "1"],
                        ["50002", "2", "0", "1"]
                    ],
                    "ts": "1616679000000"
                }]
            }
        )
        
        depth = await client.get_depth("BTC-USDT")
        assert depth['instId'] == "BTC-USDT"
        assert len(depth['bids']) == 2
        assert len(depth['asks']) == 2

@pytest.mark.asyncio
async def test_get_trades(client):
    """测试获取最近成交"""
    with requests_mock.Mocker() as m:
        m.get(
            f"{client.base_url}/api/v5/market/trades?instId=BTC-USDT&limit=100",
            json={
                "code": "0",
                "data": [{
                    "instId": "BTC-USDT",
                    "tradeId": "1",
                    "px": "50000",
                    "sz": "1",
                    "side": "buy",
                    "ts": "1616679000000"
                }]
            }
        )
        
        trades = await client.get_trades("BTC-USDT")
        assert len(trades) == 1
        assert trades[0]['instId'] == "BTC-USDT"
        assert trades[0]['tradeId'] == "1"

@pytest.mark.asyncio
async def test_get_klines(client):
    """测试获取K线数据"""
    with requests_mock.Mocker() as m:
        m.get(
            f"{client.base_url}/api/v5/market/candles?instId=BTC-USDT&bar=1m&limit=100",
            json={
                "code": "0",
                "data": [
                    ["1616679000000", "50000", "51000", "49000", "50500", "100", "5000000", "1000"]
                ]
            }
        )
        
        klines = await client.get_klines("BTC-USDT")
        assert len(klines) == 1
        assert len(klines[0]) == 8

@pytest.mark.asyncio
async def test_create_order(client):
    """测试创建订单"""
    with requests_mock.Mocker() as m:
        m.post(
            f"{client.base_url}/api/v5/trade/order",
            json={
                "code": "0",
                "data": [{
                    "ordId": "12345",
                    "clOrdId": "test123",
                    "instId": "BTC-USDT",
                    "px": "50000",
                    "sz": "1",
                    "side": "buy",
                    "ordType": "limit",
                    "state": "live",
                    "accFillSz": "0",
                    "cTime": "1616679000000",
                    "uTime": "1616679000000"
                }]
            }
        )
        
        order = await client.create_order(
            symbol="BTC-USDT",
            type=OrderType.LIMIT,
            side=OrderSide.BUY,
            price=50000,
            quantity=1,
            client_order_id="test123"
        )
        assert order[0]['ordId'] == "12345"
        assert order[0]['clOrdId'] == "test123"

@pytest.mark.asyncio
async def test_cancel_order(client):
    """测试取消订单"""
    with requests_mock.Mocker() as m:
        m.post(
            f"{client.base_url}/api/v5/trade/cancel-order",
            json={
                "code": "0",
                "data": [{
                    "ordId": "12345",
                    "clOrdId": "test123",
                    "instId": "BTC-USDT",
                    "state": "canceled"
                }]
            }
        )
        
        result = await client.cancel_order(
            symbol="BTC-USDT",
            order_id="12345"
        )
        assert result[0]['ordId'] == "12345"
        assert result[0]['state'] == "canceled"

@pytest.mark.asyncio
async def test_get_order(client):
    """测试获取订单信息"""
    with requests_mock.Mocker() as m:
        m.get(
            f"{client.base_url}/api/v5/trade/order?instId=BTC-USDT&ordId=12345",
            json={
                "code": "0",
                "data": [{
                    "ordId": "12345",
                    "clOrdId": "test123",
                    "instId": "BTC-USDT",
                    "px": "50000",
                    "sz": "1",
                    "side": "buy",
                    "ordType": "limit",
                    "state": "filled",
                    "accFillSz": "1",
                    "cTime": "1616679000000",
                    "uTime": "1616679000000"
                }]
            }
        )
        
        order = await client.get_order(
            symbol="BTC-USDT",
            order_id="12345"
        )
        assert order[0]['ordId'] == "12345"
        assert order[0]['state'] == "filled"

@pytest.mark.asyncio
async def test_error_handling(client):
    """测试错误处理"""
    with requests_mock.Mocker() as m:
        # 测试频率限制错误
        m.get(
            f"{client.base_url}/api/v5/market/ticker?instId=BTC-USDT",
            status_code=429,
            json={
                "code": "50111",
                "msg": "Request frequency too high"
            }
        )
        with pytest.raises(OKXRateLimitError):
            await client.get_ticker("BTC-USDT")
            
        # 测试认证错误
        m.get(
            f"{client.base_url}/api/v5/account/balance",
            status_code=401,
            json={
                "code": "50102",
                "msg": "Invalid API key"
            }
        )
        with pytest.raises(OKXAuthError):
            await client.get_account()
            
        # 测试网络错误
        m.get(
            f"{client.base_url}/api/v5/market/ticker?instId=BTC-USDT",
            exc=requests.exceptions.ConnectTimeout
        )
        with pytest.raises(NetworkError):
            await client.get_ticker("BTC-USDT") 