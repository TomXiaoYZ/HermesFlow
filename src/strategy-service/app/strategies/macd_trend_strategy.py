"""
MACD趋势策略

该策略使用MACD指标来判断趋势和产生交易信号:
1. 当MACD线上穿信号线时，产生做多信号
2. 当MACD线下穿信号线时，产生做空信号
3. 当MACD柱状图反转时，平仓
"""
from datetime import datetime
from decimal import Decimal
from typing import Dict, List, Optional

import pandas as pd

from app.models.strategy import BaseStrategy, Signal, SignalType
from app.indicators.trend import MACDIndicator
from app.indicators.base import IndicatorConfig


class MACDTrendStrategy(BaseStrategy):
    """MACD趋势策略"""

    def __init__(
        self,
        exchange: str,
        symbol: str,
        fast_period: int = 12,
        slow_period: int = 26,
        signal_period: int = 9,
        interval: str = "1m"
    ) -> None:
        """初始化策略"""
        super().__init__(exchange, symbol)
        
        self.fast_period = fast_period
        self.slow_period = slow_period
        self.signal_period = signal_period
        self.interval = interval
        
        # 初始化MACD指标
        self.macd = MACDIndicator(
            IndicatorConfig(window=slow_period)
        )
        
        # 当前持仓方向
        self.position = 0  # 1: 多头, -1: 空头, 0: 空仓
        
        # 上一次MACD柱状图
        self.last_hist = 0.0

    async def initialize(self) -> None:
        """初始化策略"""
        # 加载历史K线数据
        klines = await self.load_klines(
            interval=self.interval,
            limit=self.slow_period * 3
        )
        
        # 更新技术指标
        self.macd.update(klines)
        
        # 计算初始状态
        value = self.macd.get_value()
        if value:
            self.last_hist = value.values["hist"]

    async def on_kline(self, kline: Dict) -> Optional[Signal]:
        """处理K线数据"""
        # 检查K线间隔
        if kline["interval"] != self.interval:
            return None
            
        # 更新技术指标
        self.macd.update([kline])
        
        # 获取指标值
        value = self.macd.get_value()
        if not value:
            return None
            
        macd = value.values["macd"]
        signal = value.values["signal"]
        hist = value.values["hist"]
        
        # 生成信号
        trade_signal = None
        
        # MACD线上穿信号线
        if macd > signal and self.last_hist <= 0 and hist > 0:
            if self.position <= 0:  # 空仓或空头
                trade_signal = Signal(
                    exchange=self.exchange,
                    symbol=self.symbol,
                    signal_type=SignalType.LONG,
                    timestamp=kline["timestamp"],
                    price=Decimal(str(kline["close"])),
                    volume=Decimal("1"),
                    parameters={
                        "macd": macd,
                        "signal": signal,
                        "hist": hist
                    }
                )
                self.position = 1
                
        # MACD线下穿信号线
        elif macd < signal and self.last_hist >= 0 and hist < 0:
            if self.position >= 0:  # 空仓或多头
                trade_signal = Signal(
                    exchange=self.exchange,
                    symbol=self.symbol,
                    signal_type=SignalType.SHORT,
                    timestamp=kline["timestamp"],
                    price=Decimal(str(kline["close"])),
                    volume=Decimal("1"),
                    parameters={
                        "macd": macd,
                        "signal": signal,
                        "hist": hist
                    }
                )
                self.position = -1
                
        # 柱状图反转
        elif (self.last_hist > 0 and hist < 0) or (self.last_hist < 0 and hist > 0):
            if self.position != 0:  # 有持仓
                trade_signal = Signal(
                    exchange=self.exchange,
                    symbol=self.symbol,
                    signal_type=(
                        SignalType.CLOSE_LONG 
                        if self.position > 0 
                        else SignalType.CLOSE_SHORT
                    ),
                    timestamp=kline["timestamp"],
                    price=Decimal(str(kline["close"])),
                    volume=Decimal("1"),
                    parameters={
                        "macd": macd,
                        "signal": signal,
                        "hist": hist
                    }
                )
                self.position = 0
                
        # 更新状态
        self.last_hist = hist
            
        return trade_signal

    async def on_ticker(self, ticker: Dict) -> Optional[Signal]:
        """处理Ticker数据"""
        return None

    async def on_orderbook(self, orderbook: Dict) -> Optional[Signal]:
        """处理订单簿数据"""
        return None

    async def on_trade(self, trade: Dict) -> Optional[Signal]:
        """处理成交数据"""
        return None 