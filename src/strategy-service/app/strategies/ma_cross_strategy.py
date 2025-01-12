"""
双均线交叉策略

该策略使用快速和慢速移动平均线的交叉来产生交易信号:
1. 当快线上穿慢线时，产生做多信号
2. 当快线下穿慢线时，产生做空信号
"""
from datetime import datetime
from decimal import Decimal
from typing import Dict, List, Optional

import pandas as pd

from app.models.strategy import BaseStrategy, Signal, SignalType
from app.indicators.trend import MAIndicator
from app.indicators.base import IndicatorConfig


class MACrossStrategy(BaseStrategy):
    """双均线交叉策略"""

    def __init__(
        self,
        exchange: str,
        symbol: str,
        fast_period: int = 5,
        slow_period: int = 20,
        interval: str = "1m"
    ) -> None:
        """初始化策略"""
        super().__init__(exchange, symbol)
        
        self.fast_period = fast_period
        self.slow_period = slow_period
        self.interval = interval
        
        # 初始化技术指标
        self.fast_ma = MAIndicator(
            IndicatorConfig(window=fast_period)
        )
        self.slow_ma = MAIndicator(
            IndicatorConfig(window=slow_period)
        )
        
        # 当前持仓方向
        self.position = 0  # 1: 多头, -1: 空头, 0: 空仓
        
        # 上一次交叉状态 1: 快线在上方, -1: 快线在下方
        self.last_cross = 0

    async def initialize(self) -> None:
        """初始化策略"""
        # 加载历史K线数据
        klines = await self.load_klines(
            interval=self.interval,
            limit=self.slow_period * 3
        )
        
        # 更新技术指标
        self.fast_ma.update(klines)
        self.slow_ma.update(klines)
        
        # 计算初始交叉状态
        fast_value = self.fast_ma.get_value()
        slow_value = self.slow_ma.get_value()
        if fast_value and slow_value:
            if fast_value.values["ma"] > slow_value.values["ma"]:
                self.last_cross = 1
            else:
                self.last_cross = -1

    async def on_kline(self, kline: Dict) -> Optional[Signal]:
        """处理K线数据"""
        # 检查K线间隔
        if kline["interval"] != self.interval:
            return None
            
        # 更新技术指标
        self.fast_ma.update([kline])
        self.slow_ma.update([kline])
        
        # 获取指标值
        fast_value = self.fast_ma.get_value()
        slow_value = self.slow_ma.get_value()
        if not fast_value or not slow_value:
            return None
            
        fast_ma = fast_value.values["ma"]
        slow_ma = slow_value.values["ma"]
        
        # 判断交叉
        signal = None
        if fast_ma > slow_ma:  # 快线在上方
            if self.last_cross == -1:  # 发生金叉
                if self.position <= 0:  # 空仓或空头
                    signal = Signal(
                        exchange=self.exchange,
                        symbol=self.symbol,
                        signal_type=SignalType.LONG,
                        timestamp=kline["timestamp"],
                        price=Decimal(str(kline["close"])),
                        volume=Decimal("1"),
                        parameters={
                            "fast_ma": fast_ma,
                            "slow_ma": slow_ma
                        }
                    )
                    self.position = 1
            self.last_cross = 1
        else:  # 快线在下方
            if self.last_cross == 1:  # 发生死叉
                if self.position >= 0:  # 空仓或多头
                    signal = Signal(
                        exchange=self.exchange,
                        symbol=self.symbol,
                        signal_type=SignalType.SHORT,
                        timestamp=kline["timestamp"],
                        price=Decimal(str(kline["close"])),
                        volume=Decimal("1"),
                        parameters={
                            "fast_ma": fast_ma,
                            "slow_ma": slow_ma
                        }
                    )
                    self.position = -1
            self.last_cross = -1
            
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