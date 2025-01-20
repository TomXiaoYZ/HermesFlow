"""
Binance WebSocket消息处理器
"""
from typing import Dict, Any, Optional
from datetime import datetime
from decimal import Decimal

from ...common.models import (
    Market, Exchange, Trade, OrderBook, Kline, Ticker,
    OrderType, OrderSide, OrderStatus, FundingRate
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
    def handle_trade(self, data: Dict[str, Any]):
        """处理成交数据
        
        Args:
            data: 成交数据
        """
        trade = Trade(
            id=str(data["t"]),
            exchange=Exchange.BINANCE,
            market=Market.FUTURES if "X-" in data["s"] else Market.SPOT,
            side=OrderSide.BUY if data["m"] else OrderSide.SELL,
            symbol=data["s"],
            price=Decimal(str(data["p"])),
            quantity=Decimal(str(data["q"])),
            timestamp=datetime.fromtimestamp(data["T"] / 1000)
        )
        return trade
    
    @publish_event("ticker")
    def handle_ticker(self, data: Dict[str, Any]):
        """处理24小时价格变动
        
        Args:
            data: 价格数据
        """
        ticker = Ticker(
            exchange=Exchange.BINANCE,
            market=Market.FUTURES if "X-" in data["s"] else Market.SPOT,
            symbol=data["s"],
            price=Decimal(str(data["c"])),
            price_change=Decimal(str(data["p"])),
            price_change_percent=Decimal(str(data["P"])),
            weighted_avg_price=Decimal(str(data["w"])),
            open_price=Decimal(str(data["o"])),
            high_price=Decimal(str(data["h"])),
            low_price=Decimal(str(data["l"])),
            volume=Decimal(str(data["v"])),
            quote_volume=Decimal(str(data["q"])),
            open_time=datetime.fromtimestamp(data["O"] / 1000),
            close_time=datetime.fromtimestamp(data["C"] / 1000),
            first_trade_id=str(data["F"]),
            last_trade_id=str(data["L"]),
            trades_count=data["n"]
        )
        return ticker
    
    @publish_event("depth")
    def handle_depth(self, data: Dict[str, Any]):
        """处理深度数据
        
        Args:
            data: 深度数据
        """
        order_book = OrderBook(
            exchange=Exchange.BINANCE,
            market=Market.FUTURES if "X-" in data["s"] else Market.SPOT,
            symbol=data["s"],
            bids=[(Decimal(str(p)), Decimal(str(q))) for p, q in data["b"]],
            asks=[(Decimal(str(p)), Decimal(str(q))) for p, q in data["a"]],
            timestamp=datetime.fromtimestamp(data["T"] / 1000) if "T" in data else datetime.now()
        )
        return order_book
    
    @publish_event("kline")
    def handle_kline(self, data: Dict[str, Any]):
        """处理K线数据
        
        Args:
            data: K线数据
        """
        k = data["k"]
        kline = Kline(
            exchange=Exchange.BINANCE,
            market=Market.FUTURES if "X-" in data["s"] else Market.SPOT,
            symbol=k["s"],
            interval=k["i"],
            open_time=datetime.fromtimestamp(k["t"] / 1000),
            close_time=datetime.fromtimestamp(k["T"] / 1000),
            open_price=Decimal(str(k["o"])),
            high_price=Decimal(str(k["h"])),
            low_price=Decimal(str(k["l"])),
            close_price=Decimal(str(k["c"])),
            volume=Decimal(str(k["v"])),
            quote_volume=Decimal(str(k["q"])),
            trades_count=k["n"],
            is_closed=k["x"]
        )
        return kline
    
    @publish_event("mark_price")
    def handle_mark_price(self, data: Dict[str, Any]):
        """处理标记价格数据
        
        Args:
            data: 标记价格数据
        """
        return {
            "symbol": data["s"],
            "mark_price": Decimal(str(data["p"])),
            "index_price": Decimal(str(data["i"])),
            "estimated_settle_price": Decimal(str(data["P"])),
            "timestamp": datetime.fromtimestamp(data["T"] / 1000)
        }
    
    @publish_event("funding_rate")
    def handle_funding_rate(self, data: Dict[str, Any]):
        """处理资金费率数据
        
        Args:
            data: 资金费率数据
        """
        funding = FundingRate(
            symbol=data["s"],
            funding_rate=Decimal(str(data["r"])),
            estimated_rate=Decimal(str(data["p"])),
            next_funding_time=datetime.fromtimestamp(data["T"] / 1000),
            timestamp=datetime.fromtimestamp(data["E"] / 1000)
        )
        return funding 