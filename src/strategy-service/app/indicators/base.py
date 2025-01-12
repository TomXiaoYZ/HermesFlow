"""
技术指标基类
"""
from abc import ABC, abstractmethod
from datetime import datetime
from decimal import Decimal
from typing import Dict, List, Optional, Union

import numpy as np
import pandas as pd
from pydantic import BaseModel

from app.models.market_data import Kline


class IndicatorConfig(BaseModel):
    """指标配置"""
    window: int  # 计算窗口
    source: str = "close"  # 数据来源字段
    alpha: float = 0.0  # 平滑系数
    adjust: bool = True  # 是否调整异常值


class IndicatorValue(BaseModel):
    """指标值"""
    timestamp: datetime  # 时间戳
    values: Dict[str, float]  # 指标值字典


class BaseIndicator(ABC):
    """技术指标基类"""

    def __init__(self, config: IndicatorConfig) -> None:
        """初始化指标"""
        self.config = config
        self.values: List[IndicatorValue] = []
        self._df: Optional[pd.DataFrame] = None

    def update(self, klines: List[Kline]) -> None:
        """更新指标值"""
        if not klines:
            return

        # 转换为DataFrame
        df = pd.DataFrame([
            {
                "timestamp": k.timestamp,
                "open": float(k.open),
                "high": float(k.high),
                "low": float(k.low),
                "close": float(k.close),
                "volume": float(k.volume),
                "turnover": float(k.turnover)
            }
            for k in klines
        ])
        df.set_index("timestamp", inplace=True)

        # 合并历史数据
        if self._df is not None:
            df = pd.concat([self._df, df])
            df = df[~df.index.duplicated(keep="last")]
            df.sort_index(inplace=True)

        # 保留足够的历史数据用于计算
        if len(df) > self.config.window * 3:
            df = df.tail(self.config.window * 3)

        self._df = df

        # 计算指标值
        result = self._calculate(df)
        if result is not None:
            for timestamp, values in result.items():
                self.values.append(
                    IndicatorValue(
                        timestamp=timestamp,
                        values=values
                    )
                )

            # 只保留最新的N个值
            if len(self.values) > self.config.window * 2:
                self.values = self.values[-self.config.window * 2:]

    @abstractmethod
    def _calculate(
        self,
        df: pd.DataFrame
    ) -> Optional[Dict[datetime, Dict[str, float]]]:
        """计算指标值"""
        pass

    def get_value(
        self,
        timestamp: Optional[datetime] = None
    ) -> Optional[IndicatorValue]:
        """获取指标值"""
        if not self.values:
            return None

        if timestamp is None:
            return self.values[-1]

        for value in reversed(self.values):
            if value.timestamp <= timestamp:
                return value

        return None

    def get_values(
        self,
        start_time: Optional[datetime] = None,
        end_time: Optional[datetime] = None
    ) -> List[IndicatorValue]:
        """获取指标值列表"""
        if not self.values:
            return []

        if start_time is None and end_time is None:
            return self.values

        result = []
        for value in self.values:
            if start_time and value.timestamp < start_time:
                continue
            if end_time and value.timestamp > end_time:
                continue
            result.append(value)

        return result

    def _get_source_data(self, df: pd.DataFrame) -> pd.Series:
        """获取数据源"""
        if self.config.source == "open":
            return df["open"]
        elif self.config.source == "high":
            return df["high"]
        elif self.config.source == "low":
            return df["low"]
        elif self.config.source == "close":
            return df["close"]
        elif self.config.source == "volume":
            return df["volume"]
        elif self.config.source == "turnover":
            return df["turnover"]
        elif self.config.source == "hl2":
            return (df["high"] + df["low"]) / 2
        elif self.config.source == "hlc3":
            return (df["high"] + df["low"] + df["close"]) / 3
        elif self.config.source == "ohlc4":
            return (df["open"] + df["high"] + df["low"] + df["close"]) / 4
        else:
            raise ValueError(f"Invalid source: {self.config.source}")

    def _adjust_value(self, value: float) -> float:
        """调整异常值"""
        if not self.config.adjust:
            return value

        if np.isnan(value) or np.isinf(value):
            return 0.0

        return value 