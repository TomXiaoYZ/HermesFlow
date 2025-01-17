"""
交易所接口的抽象基类
"""
from abc import ABC, abstractmethod
from typing import Dict, List, Optional
from datetime import datetime

from .models import (
    Symbol, Ticker, OrderBook, Trade, Kline, Balance,
    Order, Market, OrderType, OrderSide
)

class ExchangeAPI(ABC):
    """交易所API抽象基类"""

    def __init__(self, api_key: str = "", api_secret: str = "", testnet: bool = False):
        """初始化交易所API

        Args:
            api_key: API Key
            api_secret: API Secret
            testnet: 是否使用测试网络
        """
        self.api_key = api_key
        self.api_secret = api_secret
        self.testnet = testnet

    @abstractmethod
    async def get_symbols(self, market: Market) -> List[Symbol]:
        """获取所有交易对信息

        Args:
            market: 市场类型

        Returns:
            List[Symbol]: 交易对列表
        """
        pass

    @abstractmethod
    async def get_ticker(self, market: Market, symbol: str) -> Ticker:
        """获取行情数据

        Args:
            market: 市场类型
            symbol: 交易对

        Returns:
            Ticker: 行情数据
        """
        pass

    @abstractmethod
    async def get_order_book(self, market: Market, symbol: str, limit: int = 100) -> OrderBook:
        """获取订单簿数据

        Args:
            market: 市场类型
            symbol: 交易对
            limit: 深度

        Returns:
            OrderBook: 订单簿数据
        """
        pass

    @abstractmethod
    async def get_recent_trades(self, market: Market, symbol: str, limit: int = 100) -> List[Trade]:
        """获取最近成交

        Args:
            market: 市场类型
            symbol: 交易对
            limit: 数量

        Returns:
            List[Trade]: 成交列表
        """
        pass

    @abstractmethod
    async def get_klines(
        self,
        market: Market,
        symbol: str,
        interval: str,
        start_time: Optional[datetime] = None,
        end_time: Optional[datetime] = None,
        limit: int = 500
    ) -> List[Kline]:
        """获取K线数据

        Args:
            market: 市场类型
            symbol: 交易对
            interval: 时间间隔
            start_time: 开始时间
            end_time: 结束时间
            limit: 数量

        Returns:
            List[Kline]: K线数据列表
        """
        pass

    @abstractmethod
    async def get_balances(self) -> List[Balance]:
        """获取账户余额

        Returns:
            List[Balance]: 余额列表
        """
        pass

    @abstractmethod
    async def create_order(
        self,
        market: Market,
        symbol: str,
        order_type: OrderType,
        side: OrderSide,
        price: Optional[float] = None,
        quantity: Optional[float] = None,
        client_order_id: Optional[str] = None,
    ) -> Order:
        """创建订单

        Args:
            market: 市场类型
            symbol: 交易对
            order_type: 订单类型
            side: 订单方向
            price: 价格
            quantity: 数量
            client_order_id: 客户端订单ID

        Returns:
            Order: 订单信息
        """
        pass

    @abstractmethod
    async def cancel_order(
        self,
        market: Market,
        symbol: str,
        order_id: Optional[str] = None,
        client_order_id: Optional[str] = None,
    ) -> Order:
        """取消订单

        Args:
            market: 市场类型
            symbol: 交易对
            order_id: 订单ID
            client_order_id: 客户端订单ID

        Returns:
            Order: 订单信息
        """
        pass

    @abstractmethod
    async def get_order(
        self,
        market: Market,
        symbol: str,
        order_id: Optional[str] = None,
        client_order_id: Optional[str] = None,
    ) -> Order:
        """获取订单信息

        Args:
            market: 市场类型
            symbol: 交易对
            order_id: 订单ID
            client_order_id: 客户端订单ID

        Returns:
            Order: 订单信息
        """
        pass

    @abstractmethod
    async def get_open_orders(self, market: Market, symbol: Optional[str] = None) -> List[Order]:
        """获取未完成订单

        Args:
            market: 市场类型
            symbol: 交易对

        Returns:
            List[Order]: 订单列表
        """
        pass

    @abstractmethod
    async def get_order_trades(self, market: Market, symbol: str, order_id: str) -> List[Trade]:
        """获取订单成交记录

        Args:
            market: 市场类型
            symbol: 交易对
            order_id: 订单ID

        Returns:
            List[Trade]: 成交记录列表
        """
        pass 