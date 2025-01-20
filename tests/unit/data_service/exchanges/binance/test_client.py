"""
Binance API客户端单元测试
"""
import pytest
from decimal import Decimal
from src.backend.data_service.common.models import OrderType, OrderSide, OrderStatus
from tests.common.exchange_test_base import BaseExchangeTest

class TestBinanceClient(BaseExchangeTest):
    """Binance API客户端测试类"""
    
    @pytest.fixture(autouse=True)
    async def setup(self, binance_api):
        """测试初始化"""
        self.api_client = binance_api
    
    async def test_get_exchange_info(self):
        """测试获取交易所信息"""
        info = await self.api_client.get_exchange_info()
        assert info is not None
        assert len(info.symbols) > 0
    
    async def test_get_ticker(self):
        """测试获取行情数据"""
        symbol = self.get_test_symbol()
        ticker = await self.api_client.get_ticker(symbol)
        assert ticker is not None
        assert ticker.symbol == symbol
        assert ticker.last_price > 0
    
    async def test_get_order_book(self):
        """测试获取订单簿"""
        symbol = self.get_test_symbol()
        depth = await self.api_client.get_order_book(symbol)
        assert depth is not None
        assert len(depth.bids) > 0
        assert len(depth.asks) > 0
        
    async def test_get_recent_trades(self):
        """测试获取最近成交"""
        symbol = self.get_test_symbol()
        trades = await self.api_client.get_recent_trades(symbol)
        assert trades is not None
        assert len(trades) > 0
        
    async def test_get_klines(self):
        """测试获取K线数据"""
        symbol = self.get_test_symbol()
        klines = await self.api_client.get_klines(symbol, "1m")
        assert klines is not None
        assert len(klines) > 0
        
    async def test_get_24h_ticker(self):
        """测试获取24小时行情"""
        symbol = self.get_test_symbol()
        ticker = await self.api_client.get_24h_ticker(symbol)
        assert ticker is not None
        assert ticker.volume > 0
        assert ticker.amount > 0
    
    async def test_create_order(self):
        """测试创建订单"""
        params = self.get_test_order_params()
        order = await self.api_client.create_order(**params)
        assert order is not None
        assert order.symbol == params["symbol"]
        assert order.type == params["type"]
        assert order.side == params["side"]
    
    async def test_cancel_order(self):
        """测试取消订单"""
        # 先创建订单
        params = self.get_test_order_params()
        order = await self.api_client.create_order(**params)
        
        # 取消订单
        result = await self.api_client.cancel_order(
            symbol=order.symbol,
            order_id=order.order_id
        )
        assert result is not None
        assert result.status == OrderStatus.CANCELED
    
    async def test_get_order(self):
        """测试查询订单"""
        # 先创建订单
        params = self.get_test_order_params()
        order = await self.api_client.create_order(**params)
        
        # 查询订单
        result = await self.api_client.get_order(
            symbol=order.symbol,
            order_id=order.order_id
        )
        assert result is not None
        assert result.order_id == order.order_id
    
    async def test_get_open_orders(self):
        """测试查询未完成订单"""
        orders = await self.api_client.get_open_orders()
        assert isinstance(orders, list)
    
    async def test_get_account(self):
        """测试获取账户信息"""
        account = await self.api_client.get_account()
        assert account is not None
        assert len(account.balances) > 0 