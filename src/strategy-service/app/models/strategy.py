"""
策略模型
"""
from abc import ABC, abstractmethod
from datetime import datetime
from decimal import Decimal
from enum import Enum
from typing import Dict, List, Optional, Set, Union

from pydantic import BaseModel, Field

from app.models.market_data import (
    Exchange,
    Interval,
    Kline,
    OrderBook,
    Ticker,
    Trade,
)


class SignalType(str, Enum):
    """信号类型"""
    LONG = "LONG"  # 做多
    SHORT = "SHORT"  # 做空
    CLOSE_LONG = "CLOSE_LONG"  # 平多
    CLOSE_SHORT = "CLOSE_SHORT"  # 平空
    CLOSE_ALL = "CLOSE_ALL"  # 平所有仓位


class SignalSource(str, Enum):
    """信号来源"""
    STRATEGY = "STRATEGY"  # 策略生成
    RISK_CONTROL = "RISK_CONTROL"  # 风控触发
    MANUAL = "MANUAL"  # 手动干预


class Signal(BaseModel):
    """交易信号"""
    exchange: Exchange = Field(..., description="交易所")
    symbol: str = Field(..., description="交易对")
    signal_type: SignalType = Field(..., description="信号类型")
    source: SignalSource = Field(..., description="信号来源")
    timestamp: datetime = Field(..., description="信号时间")
    price: Optional[Decimal] = Field(None, description="信号价格")
    volume: Optional[Decimal] = Field(None, description="信号数量")
    parameters: Dict = Field(default_factory=dict, description="信号参数")
    description: Optional[str] = Field(None, description="信号描述")


class StrategyState(str, Enum):
    """策略状态"""
    INITIALIZED = "INITIALIZED"  # 已初始化
    RUNNING = "RUNNING"  # 运行中
    STOPPED = "STOPPED"  # 已停止
    ERROR = "ERROR"  # 错误


class BaseStrategy(ABC):
    """策略基类"""

    def __init__(
        self,
        name: str,
        exchanges: Set[Exchange],
        symbols: Set[str],
        parameters: Dict
    ) -> None:
        """初始化策略"""
        self.name = name
        self.exchanges = exchanges
        self.symbols = symbols
        self.parameters = parameters
        self.state = StrategyState.INITIALIZED

        # 数据缓存
        self._tickers: Dict[str, Ticker] = {}
        self._klines: Dict[str, List[Kline]] = {}
        self._orderbooks: Dict[str, OrderBook] = {}
        self._trades: Dict[str, List[Trade]] = {}

    @abstractmethod
    async def initialize(self) -> None:
        """初始化策略，加载历史数据"""
        pass

    @abstractmethod
    async def on_ticker(self, ticker: Ticker) -> Optional[Signal]:
        """处理Ticker数据"""
        pass

    @abstractmethod
    async def on_kline(self, kline: Kline) -> Optional[Signal]:
        """处理K线数据"""
        pass

    @abstractmethod
    async def on_orderbook(self, orderbook: OrderBook) -> Optional[Signal]:
        """处理订单簿数据"""
        pass

    @abstractmethod
    async def on_trade(self, trade: Trade) -> Optional[Signal]:
        """处理成交记录"""
        pass

    def _get_key(self, exchange: Exchange, symbol: str) -> str:
        """获取缓存键"""
        return f"{exchange.value}:{symbol}"

    def update_ticker(self, ticker: Ticker) -> None:
        """更新Ticker数据"""
        key = self._get_key(ticker.exchange, ticker.symbol)
        self._tickers[key] = ticker

    def get_ticker(self, exchange: Exchange, symbol: str) -> Optional[Ticker]:
        """获取Ticker数据"""
        key = self._get_key(exchange, symbol)
        return self._tickers.get(key)

    def update_kline(self, kline: Kline) -> None:
        """更新K线数据"""
        key = self._get_key(kline.exchange, kline.symbol)
        if key not in self._klines:
            self._klines[key] = []
        self._klines[key].append(kline)
        # 只保留最近1000根K线
        if len(self._klines[key]) > 1000:
            self._klines[key] = self._klines[key][-1000:]

    def get_klines(
        self,
        exchange: Exchange,
        symbol: str,
        limit: int = 100
    ) -> List[Kline]:
        """获取K线数据"""
        key = self._get_key(exchange, symbol)
        klines = self._klines.get(key, [])
        return klines[-limit:]

    def update_orderbook(self, orderbook: OrderBook) -> None:
        """更新订单簿数据"""
        key = self._get_key(orderbook.exchange, orderbook.symbol)
        self._orderbooks[key] = orderbook

    def get_orderbook(
        self,
        exchange: Exchange,
        symbol: str
    ) -> Optional[OrderBook]:
        """获取订单簿数据"""
        key = self._get_key(exchange, symbol)
        return self._orderbooks.get(key)

    def update_trade(self, trade: Trade) -> None:
        """更新成交记录"""
        key = self._get_key(trade.exchange, trade.symbol)
        if key not in self._trades:
            self._trades[key] = []
        self._trades[key].append(trade)
        # 只保留最近1000条成交记录
        if len(self._trades[key]) > 1000:
            self._trades[key] = self._trades[key][-1000:]

    def get_trades(
        self,
        exchange: Exchange,
        symbol: str,
        limit: int = 100
    ) -> List[Trade]:
        """获取成交记录"""
        key = self._get_key(exchange, symbol)
        trades = self._trades.get(key, [])
        return trades[-limit:] 