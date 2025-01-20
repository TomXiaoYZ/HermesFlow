"""
Binance数据模型定义

该模块定义了从Binance WebSocket和REST API接收到的数据的标准化模型
"""

from dataclasses import dataclass
from datetime import datetime
from decimal import Decimal
from typing import Dict, List, Optional

@dataclass
class KlineData:
    """K线数据模型"""
    symbol: str
    interval: str
    start_time: datetime
    close_time: datetime
    open_price: Decimal
    high_price: Decimal
    low_price: Decimal
    close_price: Decimal
    volume: Decimal
    quote_volume: Decimal
    trades_count: int
    is_closed: bool

@dataclass
class TradeData:
    """逐笔交易数据模型"""
    symbol: str
    trade_id: int
    price: Decimal
    quantity: Decimal
    buyer_order_id: int
    seller_order_id: int
    trade_time: datetime
    is_buyer_maker: bool

@dataclass
class OrderBookEntry:
    """订单簿条目"""
    price: Decimal
    quantity: Decimal

@dataclass
class OrderBookData:
    """订单簿数据模型"""
    symbol: str
    update_id: int
    bids: List[OrderBookEntry]
    asks: List[OrderBookEntry]
    timestamp: datetime

@dataclass
class TickerData:
    """24小时价格变动数据"""
    symbol: str
    price_change: Decimal
    price_change_percent: Decimal
    weighted_avg_price: Decimal
    last_price: Decimal
    last_quantity: Decimal
    open_price: Decimal
    high_price: Decimal
    low_price: Decimal
    volume: Decimal
    quote_volume: Decimal
    open_time: datetime
    close_time: datetime
    first_trade_id: int
    last_trade_id: int
    trades_count: int 