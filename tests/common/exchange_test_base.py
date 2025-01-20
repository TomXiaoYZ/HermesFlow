"""
交易所测试基类
提供所有交易所测试用例的通用功能
"""
import os
import pytest
import asyncio
from typing import Any, Dict, Optional
from decimal import Decimal
from datetime import datetime

from src.backend.data_service.common.models import Market, OrderType, OrderSide, OrderStatus

class ExchangeTestConfig:
    """交易所测试配置"""
    def __init__(
        self,
        exchange_name: str,
        api_key_env: str,
        api_secret_env: str,
        passphrase_env: Optional[str] = None,
        test_symbols: Optional[list] = None
    ):
        self.exchange_name = exchange_name
        self.api_key = os.getenv(api_key_env, "")
        self.api_secret = os.getenv(api_secret_env, "")
        self.passphrase = os.getenv(passphrase_env, "") if passphrase_env else None
        self.test_symbols = test_symbols or ["BTC-USDT", "ETH-USDT"]

class BaseExchangeTest:
    """交易所测试基类"""
    
    # 子类需要实现这些属性
    api_client = None
    ws_client = None
    config = None
    
    @classmethod
    def setup_class(cls):
        """测试类初始化"""
        assert cls.config is not None, "必须设置config属性"
        assert cls.api_client is not None, "必须设置api_client属性"
        assert cls.ws_client is not None, "必须设置ws_client属性"
    
    async def verify_order_update(self, order_updates: list, expected_status: OrderStatus, timeout: int = 10):
        """验证订单状态更新"""
        start_time = datetime.now()
        while (datetime.now() - start_time).seconds < timeout:
            if any(update.status == expected_status for update in order_updates):
                return True
            await asyncio.sleep(0.1)
        return False
    
    async def wait_for_data(self, data_list: list, timeout: int = 10) -> bool:
        """等待数据到达"""
        start_time = datetime.now()
        while (datetime.now() - start_time).seconds < timeout:
            if len(data_list) > 0:
                return True
            await asyncio.sleep(0.1)
        return False
    
    def get_test_symbol(self) -> str:
        """获取测试交易对"""
        return self.config.test_symbols[0]
    
    def get_test_order_params(self, order_type: OrderType = OrderType.LIMIT) -> Dict[str, Any]:
        """获取测试订单参数"""
        params = {
            "symbol": self.get_test_symbol(),
            "side": OrderSide.BUY,
            "type": order_type,
            "quantity": Decimal("0.001")
        }
        
        if order_type == OrderType.LIMIT:
            params["price"] = Decimal("20000")
        
        return params
    
    async def create_and_verify_order(
        self,
        order_updates: list,
        order_type: OrderType = OrderType.LIMIT,
        expected_status: OrderStatus = OrderStatus.NEW
    ):
        """创建订单并验证状态"""
        params = self.get_test_order_params(order_type)
        order = await self.api_client.create_order(**params)
        
        assert order.symbol == params["symbol"]
        assert order.type == order_type
        
        success = await self.verify_order_update(order_updates, expected_status)
        assert success, f"订单未达到预期状态: {expected_status}"
        
        return order
    
    async def verify_ws_connection(self, timeout: int = 5):
        """验证WebSocket连接状态"""
        await asyncio.sleep(timeout)
        assert self.ws_client._ws is not None
        assert self.ws_client._running is True 