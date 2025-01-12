"""
均线交叉策略
"""
from datetime import datetime
from decimal import Decimal
from typing import Dict, List, Optional, Set

import numpy as np
import talib

from app.models.market_data import (
    Exchange,
    Interval,
    Kline,
    OrderBook,
    Ticker,
    Trade,
)
from app.models.strategy import BaseStrategy, Signal, SignalSource, SignalType


class MACrossStrategy(BaseStrategy):
    """均线交叉策略"""

    def __init__(
        self,
        name: str,
        exchanges: Set[Exchange],
        symbols: Set[str],
        parameters: Dict
    ) -> None:
        """初始化策略"""
        super().__init__(name, exchanges, symbols, parameters)
        # 获取参数
        self.fast_period = parameters.get("fast_period", 5)
        self.slow_period = parameters.get("slow_period", 20)
        self.interval = parameters.get("interval", Interval.MIN_1)

        # 策略状态
        self._positions: Dict[str, SignalType] = {}

    async def initialize(self) -> None:
        """初始化策略，加载历史数据"""
        # 这里应该从数据服务加载历史K线数据
        pass

    async def on_ticker(self, ticker: Ticker) -> Optional[Signal]:
        """处理Ticker数据"""
        # 本策略不使用Ticker数据
        return None

    async def on_kline(self, kline: Kline) -> Optional[Signal]:
        """处理K线数据"""
        # 只处理指定周期的K线
        if kline.interval != self.interval:
            return None

        # 获取历史K线
        klines = self.get_klines(kline.exchange, kline.symbol)
        if len(klines) < self.slow_period:
            return None

        # 计算均线
        closes = np.array([float(k.close) for k in klines])
        fast_ma = talib.SMA(closes, timeperiod=self.fast_period)
        slow_ma = talib.SMA(closes, timeperiod=self.slow_period)

        # 检查是否有足够的数据
        if np.isnan(fast_ma[-1]) or np.isnan(slow_ma[-1]):
            return None

        # 获取当前持仓状态
        key = f"{kline.exchange.value}:{kline.symbol}"
        current_position = self._positions.get(key)

        # 生成信号
        signal = None
        if fast_ma[-2] <= slow_ma[-2] and fast_ma[-1] > slow_ma[-1]:
            # 金叉，做多
            if not current_position:
                signal = Signal(
                    exchange=kline.exchange,
                    symbol=kline.symbol,
                    signal_type=SignalType.LONG,
                    source=SignalSource.STRATEGY,
                    timestamp=kline.close_time,
                    price=kline.close,
                    description="MA Cross: Golden Cross"
                )
                self._positions[key] = SignalType.LONG
            elif current_position == SignalType.SHORT:
                signal = Signal(
                    exchange=kline.exchange,
                    symbol=kline.symbol,
                    signal_type=SignalType.CLOSE_SHORT,
                    source=SignalSource.STRATEGY,
                    timestamp=kline.close_time,
                    price=kline.close,
                    description="MA Cross: Close Short"
                )
                self._positions[key] = None
        elif fast_ma[-2] >= slow_ma[-2] and fast_ma[-1] < slow_ma[-1]:
            # 死叉，做空
            if not current_position:
                signal = Signal(
                    exchange=kline.exchange,
                    symbol=kline.symbol,
                    signal_type=SignalType.SHORT,
                    source=SignalSource.STRATEGY,
                    timestamp=kline.close_time,
                    price=kline.close,
                    description="MA Cross: Death Cross"
                )
                self._positions[key] = SignalType.SHORT
            elif current_position == SignalType.LONG:
                signal = Signal(
                    exchange=kline.exchange,
                    symbol=kline.symbol,
                    signal_type=SignalType.CLOSE_LONG,
                    source=SignalSource.STRATEGY,
                    timestamp=kline.close_time,
                    price=kline.close,
                    description="MA Cross: Close Long"
                )
                self._positions[key] = None

        return signal

    async def on_orderbook(self, orderbook: OrderBook) -> Optional[Signal]:
        """处理订单簿数据"""
        # 本策略不使用订单簿数据
        return None

    async def on_trade(self, trade: Trade) -> Optional[Signal]:
        """处理成交记录"""
        # 本策略不使用成交记录
        return None 