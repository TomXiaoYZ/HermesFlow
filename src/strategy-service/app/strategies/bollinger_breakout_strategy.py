"""
布林带突破策略

该策略使用布林带的上下轨来判断价格突破:
1. 当价格突破上轨时，产生做多信号
2. 当价格突破下轨时，产生做空信号
3. 当价格回归中轨时，平仓
"""
from datetime import datetime
from decimal import Decimal
from typing import Dict, List, Optional

import pandas as pd

from app.models.strategy import BaseStrategy, Signal, SignalType
from app.indicators.trend import BollingerBandsIndicator
from app.indicators.base import IndicatorConfig


class BollingerBreakoutStrategy(BaseStrategy):
    """布林带突破策略"""

    def __init__(
        self,
        exchange: str,
        symbol: str,
        window: int = 20,
        std_dev: float = 2.0,
        interval: str = "1m"
    ) -> None:
        """初始化策略"""
        super().__init__(exchange, symbol)
        
        self.window = window
        self.std_dev = std_dev
        self.interval = interval
        
        # 初始化布林带指标
        self.bollinger = BollingerBandsIndicator(
            IndicatorConfig(window=window)
        )
        
        # 当前持仓方向
        self.position = 0  # 1: 多头, -1: 空头, 0: 空仓
        
        # 上一次突破状态 1: 突破上轨, -1: 突破下轨, 0: 在带内
        self.last_break = 0

    async def initialize(self) -> None:
        """初始化策略"""
        # 加载历史K线数据
        klines = await self.load_klines(
            interval=self.interval,
            limit=self.window * 3
        )
        
        # 更新技术指标
        self.bollinger.update(klines)
        
        # 计算初始突破状态
        value = self.bollinger.get_value()
        if value:
            close = float(klines[-1]["close"])
            if close > value.values["upper"]:
                self.last_break = 1
            elif close < value.values["lower"]:
                self.last_break = -1
            else:
                self.last_break = 0

    async def on_kline(self, kline: Dict) -> Optional[Signal]:
        """处理K线数据"""
        # 检查K线间隔
        if kline["interval"] != self.interval:
            return None
            
        # 更新技术指标
        self.bollinger.update([kline])
        
        # 获取指标值
        value = self.bollinger.get_value()
        if not value:
            return None
            
        close = float(kline["close"])
        upper = value.values["upper"]
        middle = value.values["middle"]
        lower = value.values["lower"]
        
        # 生成信号
        signal = None
        
        # 突破上轨
        if close > upper:
            if self.last_break <= 0:  # 之前在带内或突破下轨
                if self.position <= 0:  # 空仓或空头
                    signal = Signal(
                        exchange=self.exchange,
                        symbol=self.symbol,
                        signal_type=SignalType.LONG,
                        timestamp=kline["timestamp"],
                        price=Decimal(str(close)),
                        volume=Decimal("1"),
                        parameters={
                            "upper": upper,
                            "middle": middle,
                            "lower": lower
                        }
                    )
                    self.position = 1
            self.last_break = 1
            
        # 突破下轨
        elif close < lower:
            if self.last_break >= 0:  # 之前在带内或突破上轨
                if self.position >= 0:  # 空仓或多头
                    signal = Signal(
                        exchange=self.exchange,
                        symbol=self.symbol,
                        signal_type=SignalType.SHORT,
                        timestamp=kline["timestamp"],
                        price=Decimal(str(close)),
                        volume=Decimal("1"),
                        parameters={
                            "upper": upper,
                            "middle": middle,
                            "lower": lower
                        }
                    )
                    self.position = -1
            self.last_break = -1
            
        # 回归中轨
        else:
            if self.last_break != 0:  # 之前有突破
                if self.position != 0:  # 有持仓
                    signal = Signal(
                        exchange=self.exchange,
                        symbol=self.symbol,
                        signal_type=(
                            SignalType.CLOSE_LONG 
                            if self.position > 0 
                            else SignalType.CLOSE_SHORT
                        ),
                        timestamp=kline["timestamp"],
                        price=Decimal(str(close)),
                        volume=Decimal("1"),
                        parameters={
                            "upper": upper,
                            "middle": middle,
                            "lower": lower
                        }
                    )
                    self.position = 0
            self.last_break = 0
            
        return signal

    async def on_ticker(self, ticker: Dict) -> Optional[Signal]:
        """处理Ticker数据"""
        return None

    async def on_orderbook(self, orderbook: Dict) -> Optional[Signal]:
        """处理订单簿数据"""
        return None

    async def on_trade(self, trade: Dict) -> Optional[Signal]:
        """处理成交数据"""
        return None 