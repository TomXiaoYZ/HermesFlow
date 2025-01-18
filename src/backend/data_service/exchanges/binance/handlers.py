"""
Binance WebSocket消息处理器
"""
from typing import Dict, Any, Optional
from datetime import datetime
from decimal import Decimal

from ...common.models import (
    Market, Exchange, Trade, OrderBook, Kline, Ticker,
    OrderType, OrderSide, OrderStatus
)
from ...common.decorators import publish_event

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

class MarketDataHandler:
    """市场数据处理器"""
    
    def __init__(self, market: Market = Market.SPOT):
        """初始化处理器
        
        Args:
            market: 市场类型
        """
        self.market = market
    
    @publish_event("trade")
    def handle_trade(self, msg: Dict[str, Any]) -> Optional[Trade]:
        """处理交易消息
        
        Args:
            msg: 消息数据
            
        Returns:
            Optional[Trade]: 交易信息
        """
        try:
            return Trade(
                exchange=Exchange.BINANCE,
                market=self.market,
                symbol=msg["s"],
                id=str(msg["t"]),
                price=Decimal(msg["p"]),
                quantity=Decimal(msg["q"]),
                amount=Decimal(msg["p"]) * Decimal(msg["q"]),
                timestamp=datetime.fromtimestamp(msg["T"] / 1000),
                is_buyer_maker=msg["m"],
                side=OrderSide.SELL if msg["m"] else OrderSide.BUY
            )
        except Exception as e:
            print(f"处理trade消息出错: {str(e)}")
            return None
    
    @publish_event("orderbook")
    def handle_depth(self, msg: Dict[str, Any]) -> Optional[OrderBook]:
        """处理深度消息
        
        Args:
            msg: 消息数据
            
        Returns:
            Optional[OrderBook]: 深度信息
        """
        try:
            return OrderBook(
                exchange=Exchange.BINANCE,
                market=self.market,
                symbol=msg["s"],
                timestamp=datetime.fromtimestamp(msg["T"] / 1000),
                bids=[(Decimal(p), Decimal(q)) for p, q in msg["b"]],
                asks=[(Decimal(p), Decimal(q)) for p, q in msg["a"]]
            )
        except Exception as e:
            print(f"处理depth消息出错: {str(e)}")
            return None
    
    @publish_event("kline")
    def handle_kline(self, msg: Dict[str, Any]) -> Optional[Kline]:
        """处理K线消息
        
        Args:
            msg: 消息数据
            
        Returns:
            Optional[Kline]: K线信息
        """
        try:
            k = msg["k"]
            return Kline(
                exchange=Exchange.BINANCE,
                market=self.market,
                symbol=k["s"],
                interval=k["i"],
                open_time=datetime.fromtimestamp(k["t"] / 1000),
                close_time=datetime.fromtimestamp(k["T"] / 1000),
                open_price=Decimal(k["o"]),
                high_price=Decimal(k["h"]),
                low_price=Decimal(k["l"]),
                close_price=Decimal(k["c"]),
                volume=Decimal(k["v"]),
                quote_volume=Decimal(k["q"]),
                trades_count=k["n"],
                is_closed=k["x"]
            )
        except Exception as e:
            print(f"处理kline消息出错: {str(e)}")
            return None
    
    @publish_event("ticker")
    def handle_ticker(self, msg: Dict[str, Any]) -> Optional[Ticker]:
        """处理Ticker消息
        
        Args:
            msg: 消息数据
            
        Returns:
            Optional[Ticker]: Ticker信息
        """
        try:
            return Ticker(
                exchange=Exchange.BINANCE,
                market=self.market,
                symbol=msg["s"],
                price=Decimal(msg["c"]),
                price_change=Decimal(msg["p"]),
                price_change_percent=Decimal(msg["P"]),
                weighted_avg_price=Decimal(msg["w"]),
                open_price=Decimal(msg["o"]),
                high_price=Decimal(msg["h"]),
                low_price=Decimal(msg["l"]),
                volume=Decimal(msg["v"]),
                quote_volume=Decimal(msg["q"]),
                open_time=datetime.fromtimestamp(msg["O"] / 1000),
                close_time=datetime.fromtimestamp(msg["C"] / 1000),
                first_trade_id=str(msg["F"]),
                last_trade_id=str(msg["L"]),
                trades_count=msg["n"]
            )
        except Exception as e:
            print(f"处理ticker消息出错: {str(e)}")
            return None 