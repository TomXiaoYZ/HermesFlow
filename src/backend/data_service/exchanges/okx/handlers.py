"""
OKX WebSocket消息处理器
"""
import logging
from typing import Dict, Any, Callable, Awaitable
from decimal import Decimal

from ....common.models import (
    Market, Exchange, Trade, OrderBook, Kline, Ticker,
    OrderType, OrderSide, OrderStatus, FundingRate
)
from .models import (
    parse_ticker, parse_order_book, parse_trade,
    parse_kline, parse_order
)

logger = logging.getLogger(__name__)

class OKXMessageHandler:
    """OKX消息处理器"""
    
    def __init__(self):
        """初始化处理器"""
        self._callbacks: Dict[str, Callable[[Any], Awaitable[None]]] = {}
        
    async def handle_ticker(self, message: Dict[str, Any]) -> None:
        """处理行情消息
        
        Args:
            message: 消息数据
        """
        try:
            data = message['data'][0]
            ticker = parse_ticker(data)
            
            channel = f"tickers:{ticker.symbol}"
            if channel in self._callbacks:
                await self._callbacks[channel](ticker)
                
        except Exception as e:
            logger.error(f"处理行情消息失败: {str(e)}")
            
    async def handle_depth(self, message: Dict[str, Any]) -> None:
        """处理深度消息
        
        Args:
            message: 消息数据
        """
        try:
            data = message['data'][0]
            order_book = parse_order_book(data)
            
            channel = f"books:{order_book.symbol}"
            if channel in self._callbacks:
                await self._callbacks[channel](order_book)
                
        except Exception as e:
            logger.error(f"处理深度消息失败: {str(e)}")
            
    async def handle_trade(self, message: Dict[str, Any]) -> None:
        """处理成交消息
        
        Args:
            message: 消息数据
        """
        try:
            data = message['data'][0]
            trade = parse_trade(data)
            
            channel = f"trades:{trade.symbol}"
            if channel in self._callbacks:
                await self._callbacks[channel](trade)
                
        except Exception as e:
            logger.error(f"处理成交消息失败: {str(e)}")
            
    async def handle_kline(self, message: Dict[str, Any]) -> None:
        """处理K线消息
        
        Args:
            message: 消息数据
        """
        try:
            data = message['data'][0]
            symbol = message['arg']['instId']
            interval = message['arg']['channel'].split('_')[1]
            kline = parse_kline(data, symbol, interval)
            
            channel = f"candle{interval}s:{kline.symbol}"
            if channel in self._callbacks:
                await self._callbacks[channel](kline)
                
        except Exception as e:
            logger.error(f"处理K线消息失败: {str(e)}")
            
    async def handle_order(self, message: Dict[str, Any]) -> None:
        """处理订单消息
        
        Args:
            message: 消息数据
        """
        try:
            data = message['data'][0]
            order = parse_order(data)
            
            channel = f"orders:{order.symbol}"
            if channel in self._callbacks:
                await self._callbacks[channel](order)
                
        except Exception as e:
            logger.error(f"处理订单消息失败: {str(e)}")
            
    def register_callback(
        self,
        channel: str,
        callback: Callable[[Any], Awaitable[None]]
    ) -> None:
        """注册回调函数
        
        Args:
            channel: 频道
            callback: 回调函数
        """
        self._callbacks[channel] = callback
        
    def unregister_callback(self, channel: str) -> None:
        """注销回调函数
        
        Args:
            channel: 频道
        """
        if channel in self._callbacks:
            del self._callbacks[channel] 