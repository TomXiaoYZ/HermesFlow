"""
市场数据模型
"""
from datetime import datetime
from decimal import Decimal
from enum import Enum
from typing import Dict, List, Optional

from pydantic import BaseModel, Field


class Exchange(str, Enum):
    """交易所枚举"""
    BINANCE = "binance"
    OKX = "okx"
    BITGET = "bitget"


class DataType(str, Enum):
    """数据类型枚举"""
    TICKER = "ticker"
    KLINE = "kline"
    ORDERBOOK = "orderbook"
    TRADE = "trade"


class Interval(str, Enum):
    """K线间隔枚举"""
    ONE_MINUTE = "1m"
    THREE_MINUTES = "3m"
    FIVE_MINUTES = "5m"
    FIFTEEN_MINUTES = "15m"
    THIRTY_MINUTES = "30m"
    ONE_HOUR = "1h"
    TWO_HOURS = "2h"
    FOUR_HOURS = "4h"
    SIX_HOURS = "6h"
    EIGHT_HOURS = "8h"
    TWELVE_HOURS = "12h"
    ONE_DAY = "1d"
    THREE_DAYS = "3d"
    ONE_WEEK = "1w"
    ONE_MONTH = "1M"


class Ticker(BaseModel):
    """行情数据模型"""
    exchange: Exchange
    symbol: str
    price: Decimal
    volume: Decimal
    timestamp: datetime
    bid_price: Optional[Decimal] = None
    bid_volume: Optional[Decimal] = None
    ask_price: Optional[Decimal] = None
    ask_volume: Optional[Decimal] = None
    high_24h: Optional[Decimal] = None
    low_24h: Optional[Decimal] = None
    volume_24h: Optional[Decimal] = None
    quote_volume_24h: Optional[Decimal] = None
    price_change_24h: Optional[Decimal] = None
    price_change_percent_24h: Optional[float] = None


class Kline(BaseModel):
    """K线数据模型"""
    exchange: Exchange
    symbol: str
    interval: Interval
    open_time: datetime
    close_time: datetime
    open: Decimal
    high: Decimal
    low: Decimal
    close: Decimal
    volume: Decimal
    quote_volume: Decimal
    trades_count: int
    taker_buy_volume: Optional[Decimal] = None
    taker_buy_quote_volume: Optional[Decimal] = None


class OrderBookLevel(BaseModel):
    """订单簿价格档位"""
    price: Decimal
    quantity: Decimal


class OrderBook(BaseModel):
    """订单簿数据模型"""
    exchange: Exchange
    symbol: str
    timestamp: datetime
    last_update_id: int
    bids: List[OrderBookLevel]
    asks: List[OrderBookLevel]


class Trade(BaseModel):
    """成交数据模型"""
    exchange: Exchange
    symbol: str
    id: str
    price: Decimal
    quantity: Decimal
    timestamp: datetime
    is_buyer_maker: bool
    quote_quantity: Optional[Decimal] = None
    fee: Optional[Decimal] = None
    fee_asset: Optional[str] = None


class MarketDataUpdate(BaseModel):
    """市场数据更新"""
    exchange: Exchange
    data_type: DataType
    symbol: str
    data: Dict
    timestamp: datetime = Field(default_factory=datetime.utcnow) 