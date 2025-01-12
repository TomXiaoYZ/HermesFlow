"""
RSI超买超卖策略

该策略使用RSI指标来判断市场的超买超卖状态并产生交易信号:
1. 当RSI低于超卖阈值时，产生做多信号
2. 当RSI高于超买阈值时，产生做空信号
3. 当RSI回归到中性区域时，平仓
"""
from datetime import datetime
from decimal import Decimal
from typing import Dict, List, Optional

import pandas as pd

from app.models.strategy import BaseStrategy, Signal, SignalType
from app.indicators.momentum import RSIIndicator
from app.indicators.base import IndicatorConfig


class RSIStrategy(BaseStrategy):
    """RSI超买超卖策略"""

    def __init__(
        self,
        exchange: str,
        symbol: str,
        period: int = 14,
        overbought: float = 70,
        oversold: float = 30,
        neutral: float = 50,
        interval: str = "1m"
    ) -> None:
        """初始化策略
        
        Args:
            exchange: 交易所
            symbol: 交易对
            period: RSI计算周期
            overbought: 超买阈值
            oversold: 超卖阈值
            neutral: 中性区域阈值
            interval: K线间隔
        """
        super().__init__(exchange, symbol)
        
        self.period = period
        self.overbought = overbought
        self.oversold = oversold
        self.neutral = neutral
        self.interval = interval
        
        # 初始化RSI指标
        self.rsi = RSIIndicator(
            IndicatorConfig(window=period)
        )
        
        # 当前持仓方向
        self.position = 0  # 1: 多头, -1: 空头, 0: 空仓
        
        # 上一次RSI值
        self.last_rsi = 50.0

    async def initialize(self) -> None:
        """初始化策略"""
        # 加载历史K线数据
        klines = await self.load_klines(
            interval=self.interval,
            limit=self.period * 3
        )
        
        # 更新技术指标
        self.rsi.update(klines)
        
        # 计算初始状态
        value = self.rsi.get_value()
        if value:
            self.last_rsi = value.values["rsi"]

    async def on_kline(self, kline: Dict) -> Optional[Signal]:
        """处理K线数据"""
        # 检查K线间隔
        if kline["interval"] != self.interval:
            return None
            
        # 更新技术指标
        self.rsi.update([kline])
        
        # 获取指标值
        value = self.rsi.get_value()
        if not value:
            return None
            
        rsi = value.values["rsi"]
        
        # 生成信号
        trade_signal = None
        
        # RSI进入超卖区域
        if rsi < self.oversold and self.last_rsi >= self.oversold:
            if self.position <= 0:  # 空仓或空头
                trade_signal = Signal(
                    exchange=self.exchange,
                    symbol=self.symbol,
                    signal_type=SignalType.LONG,
                    timestamp=kline["timestamp"],
                    price=Decimal(str(kline["close"])),
                    volume=Decimal("1"),
                    parameters={
                        "rsi": rsi,
                        "threshold": self.oversold
                    }
                )
                self.position = 1
                
        # RSI进入超买区域
        elif rsi > self.overbought and self.last_rsi <= self.overbought:
            if self.position >= 0:  # 空仓或多头
                trade_signal = Signal(
                    exchange=self.exchange,
                    symbol=self.symbol,
                    signal_type=SignalType.SHORT,
                    timestamp=kline["timestamp"],
                    price=Decimal(str(kline["close"])),
                    volume=Decimal("1"),
                    parameters={
                        "rsi": rsi,
                        "threshold": self.overbought
                    }
                )
                self.position = -1
                
        # RSI回归中性区域
        elif abs(rsi - self.neutral) < 1 and abs(self.last_rsi - self.neutral) >= 1:
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
                        "rsi": rsi,
                        "threshold": self.neutral
                    }
                )
                self.position = 0
                
        # 更新状态
        self.last_rsi = rsi
            
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