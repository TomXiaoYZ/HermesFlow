"""
Binance交易所集成测试
"""
import pytest
import asyncio
from datetime import datetime, timedelta
from decimal import Decimal

from src.backend.data_service.common.models import Market, OrderType, OrderSide, OrderStatus
from tests.common.exchange_test_base import BaseExchangeTest

class TestBinanceIntegration(BaseExchangeTest):
    """Binance交易所集成测试类"""
    
    @pytest.fixture(autouse=True)
    async def setup(self, binance_api, binance_ws):
        """测试初始化"""
        self.api_client = binance_api
        self.ws_client = binance_ws
    
    async def test_order_lifecycle(self):
        """测试订单生命周期"""
        # 订阅订单更新
        order_updates = []
        await self.ws_client.subscribe_user_data(
            on_order=lambda x: order_updates.append(x)
        )
        
        # 创建限价单
        params = self.get_test_order_params(OrderType.LIMIT)
        order = await self.api_client.create_order(**params)
        assert order.status == OrderStatus.NEW
        
        # 等待订单更新
        success = await self.verify_order_update(
            order_updates,
            OrderStatus.NEW
        )
        assert success
        
        # 查询订单
        order = await self.api_client.get_order(
            symbol=params["symbol"],
            order_id=order.order_id
        )
        assert order.status in [OrderStatus.NEW, OrderStatus.PARTIALLY_FILLED]
        
        # 取消订单
        result = await self.api_client.cancel_order(
            symbol=params["symbol"],
            order_id=order.order_id
        )
        assert result.status == OrderStatus.CANCELED
        
        # 等待取消更新
        success = await self.verify_order_update(
            order_updates,
            OrderStatus.CANCELED
        )
        assert success
    
    async def test_market_data_flow(self):
        """测试市场数据流"""
        symbol = self.get_test_symbol()
        
        # 订阅行情数据
        ticker_updates = []
        await self.ws_client.subscribe_ticker(
            symbol,
            lambda x: ticker_updates.append(x)
        )
        
        # 订阅深度数据
        depth_updates = []
        await self.ws_client.subscribe_depth(
            symbol,
            lambda x: depth_updates.append(x)
        )
        
        # 订阅K线数据
        kline_updates = []
        await self.ws_client.subscribe_kline(
            symbol,
            "1m",
            lambda x: kline_updates.append(x)
        )
        
        # 等待数据更新
        await asyncio.sleep(10)
        
        # 验证行情数据
        assert len(ticker_updates) > 0
        ticker = ticker_updates[-1]
        assert ticker.symbol == symbol
        assert ticker.last_price > 0
        
        # 验证深度数据
        assert len(depth_updates) > 0
        depth = depth_updates[-1]
        assert depth.symbol == symbol
        assert len(depth.bids) > 0
        assert len(depth.asks) > 0
        
        # 验证K线数据
        assert len(kline_updates) > 0
        kline = kline_updates[-1]
        assert kline.symbol == symbol
        assert kline.interval == "1m"
    
    async def test_error_handling(self):
        """测试错误处理"""
        # 测试无效的交易对
        with pytest.raises(ValueError):
            await self.api_client.get_ticker("INVALID-SYMBOL")
        
        # 测试无效的订单参数
        with pytest.raises(ValueError):
            await self.api_client.create_order(
                symbol=self.get_test_symbol(),
                type=OrderType.LIMIT,
                side=OrderSide.BUY,
                quantity=Decimal("0.0001"),  # 小于最小数量
                price=Decimal("1.0")  # 价格太低
            )
        
        # 测试无效的订单ID
        with pytest.raises(ValueError):
            await self.api_client.get_order(
                symbol=self.get_test_symbol(),
                order_id="invalid-id"
            )
    
    async def test_websocket_reconnection(self):
        """测试WebSocket重连"""
        symbol = self.get_test_symbol()
        updates = []
        
        # 订阅数据
        await self.ws_client.subscribe_ticker(
            symbol,
            lambda x: updates.append(x)
        )
        
        # 等待初始数据
        success = await self.wait_for_data(updates)
        assert success
        
        # 模拟断连
        await self.ws_client._ws.close()
        
        # 等待重连
        await asyncio.sleep(5)
        
        # 清空更新列表
        updates.clear()
        
        # 验证重连后能继续收到数据
        success = await self.wait_for_data(updates)
        assert success 