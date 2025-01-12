"""
波动率指标
"""
from datetime import datetime
from typing import Dict, Optional

import numpy as np
import pandas as pd
import talib

from app.indicators.base import BaseIndicator, IndicatorConfig


class ATRIndicator(BaseIndicator):
    """真实波幅"""

    def _calculate(
        self,
        df: pd.DataFrame
    ) -> Optional[Dict[datetime, Dict[str, float]]]:
        """计算指标值"""
        if len(df) < self.config.window:
            return None

        # 计算ATR
        atr = talib.ATR(
            df["high"],
            df["low"],
            df["close"],
            timeperiod=self.config.window
        )

        # 转换为字典
        result = {}
        for i in range(len(df)):
            if i < self.config.window:
                continue
            timestamp = df.index[i]
            value = float(atr[i])
            result[timestamp] = {
                "atr": self._adjust_value(value)
            }

        return result


class NATRIndicator(BaseIndicator):
    """归一化真实波幅"""

    def _calculate(
        self,
        df: pd.DataFrame
    ) -> Optional[Dict[datetime, Dict[str, float]]]:
        """计算指标值"""
        if len(df) < self.config.window:
            return None

        # 计算NATR
        natr = talib.NATR(
            df["high"],
            df["low"],
            df["close"],
            timeperiod=self.config.window
        )

        # 转换为字典
        result = {}
        for i in range(len(df)):
            if i < self.config.window:
                continue
            timestamp = df.index[i]
            value = float(natr[i])
            result[timestamp] = {
                "natr": self._adjust_value(value)
            }

        return result


class TRUERANGEIndicator(BaseIndicator):
    """真实波动幅度"""

    def _calculate(
        self,
        df: pd.DataFrame
    ) -> Optional[Dict[datetime, Dict[str, float]]]:
        """计算指标值"""
        if len(df) < 2:
            return None

        # 计算TRANGE
        trange = talib.TRANGE(
            df["high"],
            df["low"],
            df["close"]
        )

        # 转换为字典
        result = {}
        for i in range(len(df)):
            if i < 1:
                continue
            timestamp = df.index[i]
            value = float(trange[i])
            result[timestamp] = {
                "trange": self._adjust_value(value)
            }

        return result


class STDDEVIndicator(BaseIndicator):
    """标准差"""

    def _calculate(
        self,
        df: pd.DataFrame
    ) -> Optional[Dict[datetime, Dict[str, float]]]:
        """计算指标值"""
        if len(df) < self.config.window:
            return None

        # 获取数据源
        source = self._get_source_data(df)

        # 计算标准差
        stddev = talib.STDDEV(
            source,
            timeperiod=self.config.window,
            nbdev=1
        )

        # 转换为字典
        result = {}
        for i in range(len(df)):
            if i < self.config.window:
                continue
            timestamp = df.index[i]
            value = float(stddev[i])
            result[timestamp] = {
                "stddev": self._adjust_value(value)
            }

        return result


class VARIndicator(BaseIndicator):
    """方差"""

    def _calculate(
        self,
        df: pd.DataFrame
    ) -> Optional[Dict[datetime, Dict[str, float]]]:
        """计算指标值"""
        if len(df) < self.config.window:
            return None

        # 获取数据源
        source = self._get_source_data(df)

        # 计算方差
        var = talib.VAR(
            source,
            timeperiod=self.config.window,
            nbdev=1
        )

        # 转换为字典
        result = {}
        for i in range(len(df)):
            if i < self.config.window:
                continue
            timestamp = df.index[i]
            value = float(var[i])
            result[timestamp] = {
                "var": self._adjust_value(value)
            }

        return result


class ParkinsonsVolatilityIndicator(BaseIndicator):
    """帕金森波动率"""

    def _calculate(
        self,
        df: pd.DataFrame
    ) -> Optional[Dict[datetime, Dict[str, float]]]:
        """计算指标值"""
        if len(df) < self.config.window:
            return None

        # 计算帕金森波动率
        hl = np.log(df["high"] / df["low"])
        pv = np.sqrt(
            hl.rolling(window=self.config.window).apply(
                lambda x: sum(x * x) / (4 * len(x) * np.log(2))
            )
        ) * np.sqrt(252)

        # 转换为字典
        result = {}
        for i in range(len(df)):
            if i < self.config.window:
                continue
            timestamp = df.index[i]
            value = float(pv[i])
            result[timestamp] = {
                "pv": self._adjust_value(value)
            }

        return result 