"""
Binance WebSocket消息处理器
"""
from typing import Dict, Any
from datetime import datetime
from decimal import Decimal

from ...common.models import Exchange, Market, OrderStatus, OrderSide, OrderType
from ...db.decorators import publish_event

class OrderUpdateHandler:
    """订单更新处理器"""
    
    def __init__(self, market: Market = Market.SPOT):
        """初始化处理器
        
        Args:
            market: 市场类型
        """
        self.market = market
    
    @publish_event("order.updated")
    def __call__(self, msg: Dict[str, Any]) -> Dict[str, Any]:
        """处理订单更新消息
        
        Args:
            msg: WebSocket消息
            
        Returns:
            Dict[str, Any]: 处理后的订单数据
        """
        # 解析订单状态
        status_map = {
            "NEW": OrderStatus.NEW,
            "PARTIALLY_FILLED": OrderStatus.PARTIALLY_FILLED,
            "FILLED": OrderStatus.FILLED,
            "CANCELED": OrderStatus.CANCELED,
            "REJECTED": OrderStatus.REJECTED,
            "EXPIRED": OrderStatus.EXPIRED
        }
        
        # 解析订单方向
        side_map = {
            "BUY": OrderSide.BUY,
            "SELL": OrderSide.SELL
        }
        
        # 解析订单类型
        type_map = {
            "LIMIT": OrderType.LIMIT,
            "MARKET": OrderType.MARKET,
            "STOP": OrderType.STOP,
            "STOP_MARKET": OrderType.STOP,
            "TAKE_PROFIT": OrderType.STOP,
            "TAKE_PROFIT_MARKET": OrderType.STOP,
            "TRAILING_STOP_MARKET": OrderType.TRAILING_STOP
        }
        
        # 构造标准化的订单数据
        order_data = {
            "exchange": Exchange.BINANCE.value,
            "market": self.market.value,
            "exchange_order_id": str(msg["i"]),
            "client_order_id": msg["c"],
            "symbol": msg["s"],
            "type": type_map[msg["o"]].value,
            "side": side_map[msg["S"]].value,
            "price": Decimal(str(msg["p"])),
            "quantity": Decimal(str(msg["q"])),
            "executed_quantity": Decimal(str(msg["z"])),
            "status": status_map[msg["X"]].value,
            "created_at": datetime.fromtimestamp(msg["O"] / 1000).isoformat(),
            "updated_at": datetime.fromtimestamp(msg["E"] / 1000).isoformat()
        }
        
        # 添加条件单相关字段
        if msg["o"] in ["STOP", "STOP_MARKET", "TAKE_PROFIT", "TAKE_PROFIT_MARKET"]:
            order_data["stop_price"] = Decimal(str(msg["P"]))
        
        return order_data 