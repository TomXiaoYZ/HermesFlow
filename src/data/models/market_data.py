"""
市场数据模型

定义标准化的市场数据结构，供所有连接器使用
"""

from dataclasses import dataclass
from datetime import datetime
from typing import Optional, List, Dict, Any
from decimal import Decimal

@dataclass
class KlineData:
    """K线数据模型"""
    symbol: str
    interval: str
    open_time: datetime
    close_time: datetime
    open_price: float
    high_price: float
    low_price: float
    close_price: float
    volume: float
    quote_volume: float
    trades_count: int
    taker_buy_base_volume: float
    taker_buy_quote_volume: float
    exchange: str
    timestamp: datetime

@dataclass
class TickerData:
    """行情数据模型"""
    symbol: str
    price: float
    bid_price: float
    bid_quantity: float
    ask_price: float
    ask_quantity: float
    high_24h: float
    low_24h: float
    volume_24h: float
    quote_volume_24h: float
    price_change_24h: float
    price_change_percent_24h: float
    weighted_avg_price: float
    prev_close_price: float
    last_quantity: float
    exchange: str
    timestamp: datetime

@dataclass
class OrderBookLevel:
    """订单簿价格层级"""
    price: float
    quantity: float

@dataclass
class OrderBookData:
    """订单簿数据模型"""
    symbol: str
    bids: List[OrderBookLevel]
    asks: List[OrderBookLevel]
    exchange: str
    timestamp: datetime
    last_update_id: Optional[int] = None

@dataclass
class TradeData:
    """交易数据模型"""
    symbol: str
    trade_id: str
    price: float
    quantity: float
    quote_quantity: float
    time: datetime
    is_buyer_maker: bool
    is_best_match: bool
    exchange: str
    timestamp: datetime 