"""
基础数据模型
"""
from enum import Enum
from typing import List, Tuple, Optional
from datetime import datetime
from decimal import Decimal
from dataclasses import dataclass

class Exchange(str, Enum):
    """交易所"""
    BINANCE = "binance"
    OKX = "okx"
    BITGET = "bitget"

class Market(str, Enum):
    """市场类型"""
    SPOT = "spot"
    MARGIN = "margin"
    FUTURES = "futures"
    OPTIONS = "options"

class OrderType(str, Enum):
    """订单类型"""
    LIMIT = "limit"
    MARKET = "market"
    STOP = "stop"
    TRAILING_STOP = "trailing_stop"

class OrderSide(str, Enum):
    """订单方向"""
    BUY = "buy"
    SELL = "sell"

class OrderStatus(str, Enum):
    """订单状态"""
    NEW = "new"
    PARTIALLY_FILLED = "partially_filled"
    FILLED = "filled"
    CANCELED = "canceled"
    REJECTED = "rejected"
    EXPIRED = "expired"

@dataclass
class Symbol:
    """交易对信息"""
    exchange: Exchange
    market: Market
    base_asset: str
    quote_asset: str
    min_price: Decimal
    max_price: Decimal
    tick_size: Decimal
    min_qty: Decimal
    max_qty: Decimal
    step_size: Decimal
    min_notional: Decimal
    status: str
    created_at: datetime

@dataclass
class Trade:
    """交易信息"""
    exchange: Exchange
    market: Market
    symbol: str
    id: str
    price: Decimal
    quantity: Decimal
    amount: Decimal
    timestamp: datetime
    is_buyer_maker: bool
    side: OrderSide

@dataclass
class OrderBook:
    """订单簿信息"""
    exchange: Exchange
    market: Market
    symbol: str
    bids: List[Tuple[Decimal, Decimal]]
    asks: List[Tuple[Decimal, Decimal]]
    timestamp: datetime

@dataclass
class Kline:
    """K线信息"""
    exchange: Exchange
    market: Market
    symbol: str
    interval: str
    open_time: datetime
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
class Ticker:
    """Ticker信息"""
    exchange: Exchange
    market: Market
    symbol: str
    price: Decimal
    price_change: Decimal
    price_change_percent: Decimal
    weighted_avg_price: Decimal
    open_price: Decimal
    high_price: Decimal
    low_price: Decimal
    volume: Decimal
    quote_volume: Decimal
    open_time: datetime
    close_time: datetime
    first_trade_id: str
    last_trade_id: str
    trades_count: int

@dataclass
class Balance:
    """账户余额"""
    exchange: Exchange
    market: Market
    asset: str
    free: Decimal
    locked: Decimal
    total: Decimal
    timestamp: datetime

@dataclass
class Order:
    """订单信息"""
    exchange: Exchange
    market: Market
    symbol: str
    id: str
    client_order_id: str
    price: Decimal
    original_quantity: Decimal
    executed_quantity: Decimal
    remaining_quantity: Decimal
    status: OrderStatus
    type: OrderType
    side: OrderSide
    created_at: datetime
    updated_at: datetime
    is_working: bool 