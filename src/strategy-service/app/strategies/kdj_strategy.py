"""
KDJ交叉策略

该策略使用KDJ指标来判断市场的超买超卖状态并产生交易信号:
1. 当K线和D线同时向上穿过超卖阈值，且J线在超卖区域时，产生做多信号
2. 当K线和D线同时向下穿过超买阈值，且J线在超买区域时，产生做空信号
3. 当K线和D线在中性区域交叉时，平仓
"""
from datetime import datetime
from decimal import Decimal
from typing import Dict, List, Optional

import pandas as pd

from app.models.strategy import BaseStrategy, Signal, SignalType
from app.indicators.momentum import StochasticIndicator
from app.indicators.base import IndicatorConfig


class KDJStrategy(BaseStrategy):
    """KDJ交叉策略"""

    def __init__(
        self,
        exchange: str,
        symbol: str,
        k_period: int = 9,
        d_period: int = 3,
        j_period: int = 3,
        overbought: float = 80,
        oversold: float = 20,
        neutral_high: float = 60,
        neutral_low: float = 40,
        interval: str = "1m"
    ) -> None:
        """初始化策略
        
        Args:
            exchange: 交易所
            symbol: 交易对
            k_period: K值计算周期
            d_period: D值计算周期
            j_period: J值计算周期
            overbought: 超买阈值
            oversold: 超卖阈值
            neutral_high: 中性区域上界
            neutral_low: 中性区域下界
            interval: K线间隔
        """
        super().__init__(exchange, symbol)
        
        self.k_period = k_period
        self.d_period = d_period
        self.j_period = j_period
        self.overbought = overbought
        self.oversold = oversold
        self.neutral_high = neutral_high
        self.neutral_low = neutral_low
        self.interval = interval
        
        # 初始化KDJ指标
        self.kdj = StochasticIndicator(
            IndicatorConfig(
                window=k_period,
                k_period=k_period,
                d_period=d_period
            )
        )
        
        # 当前持仓方向
        self.position = 0  # 1: 多头, -1: 空头, 0: 空仓
        
        # 上一次KDJ值
        self.last_k = 50.0
        self.last_d = 50.0
        self.last_j = 50.0

    async def initialize(self) -> None:
        """初始化策略"""
        # 加载历史K线数据
        klines = await self.load_klines(
            interval=self.interval,
            limit=self.k_period * 3
        )
        
        # 更新技术指标
        self.kdj.update(klines)
        
        # 计算初始状态
        value = self.kdj.get_value()
        if value:
            self.last_k = value.values["k"]
            self.last_d = value.values["d"]
            self.last_j = 3 * self.last_k - 2 * self.last_d  # 计算J值

    async def on_kline(self, kline: Dict) -> Optional[Signal]:
        """处理K线数据"""
        # 检查K线间隔
        if kline["interval"] != self.interval:
            return None
            
        # 更新技术指标
        self.kdj.update([kline])
        
        # 获取指标值
        value = self.kdj.get_value()
        if not value:
            return None
            
        k = value.values["k"]
        d = value.values["d"]
        j = 3 * k - 2 * d  # 计算J值
        
        # 生成信号
        trade_signal = None
        
        # K和D同时向上穿过超卖阈值，且J在超卖区域
        if (k > self.oversold and self.last_k <= self.oversold and
            d > self.oversold and self.last_d <= self.oversold and
            j < self.oversold):
            if self.position <= 0:  # 空仓或空头
                trade_signal = Signal(
                    exchange=self.exchange,
                    symbol=self.symbol,
                    signal_type=SignalType.LONG,
                    timestamp=kline["timestamp"],
                    price=Decimal(str(kline["close"])),
                    volume=Decimal("1"),
                    parameters={
                        "k": k,
                        "d": d,
                        "j": j,
                        "threshold": self.oversold
                    }
                )
                self.position = 1
                
        # K和D同时向下穿过超买阈值，且J在超买区域
        elif (k < self.overbought and self.last_k >= self.overbought and
              d < self.overbought and self.last_d >= self.overbought and
              j > self.overbought):
            if self.position >= 0:  # 空仓或多头
                trade_signal = Signal(
                    exchange=self.exchange,
                    symbol=self.symbol,
                    signal_type=SignalType.SHORT,
                    timestamp=kline["timestamp"],
                    price=Decimal(str(kline["close"])),
                    volume=Decimal("1"),
                    parameters={
                        "k": k,
                        "d": d,
                        "j": j,
                        "threshold": self.overbought
                    }
                )
                self.position = -1
                
        # K和D在中性区域交叉
        elif (self.neutral_low <= k <= self.neutral_high and
              self.neutral_low <= d <= self.neutral_high):
            # K线上穿D线
            if k > d and self.last_k <= self.last_d:
                if self.position < 0:  # 空头
                    trade_signal = Signal(
                        exchange=self.exchange,
                        symbol=self.symbol,
                        signal_type=SignalType.CLOSE_SHORT,
                        timestamp=kline["timestamp"],
                        price=Decimal(str(kline["close"])),
                        volume=Decimal("1"),
                        parameters={
                            "k": k,
                            "d": d,
                            "j": j,
                            "cross": "golden"
                        }
                    )
                    self.position = 0
            # K线下穿D线
            elif k < d and self.last_k >= self.last_d:
                if self.position > 0:  # 多头
                    trade_signal = Signal(
                        exchange=self.exchange,
                        symbol=self.symbol,
                        signal_type=SignalType.CLOSE_LONG,
                        timestamp=kline["timestamp"],
                        price=Decimal(str(kline["close"])),
                        volume=Decimal("1"),
                        parameters={
                            "k": k,
                            "d": d,
                            "j": j,
                            "cross": "death"
                        }
                    )
                    self.position = 0
                
        # 更新状态
        self.last_k = k
        self.last_d = d
        self.last_j = j
            
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