"""
动量指标
"""
from datetime import datetime
from typing import Dict, Optional

import numpy as np
import pandas as pd
import talib

from app.indicators.base import BaseIndicator, IndicatorConfig


class RSIIndicator(BaseIndicator):
    """相对强弱指标"""

    def _calculate(
        self,
        df: pd.DataFrame
    ) -> Optional[Dict[datetime, Dict[str, float]]]:
        """计算指标值"""
        if len(df) < self.config.window:
            return None

        # 获取数据源
        source = self._get_source_data(df)

        # 计算RSI
        rsi = talib.RSI(source, timeperiod=self.config.window)

        # 转换为字典
        result = {}
        for i in range(len(df)):
            if i < self.config.window:
                continue
            timestamp = df.index[i]
            value = float(rsi[i])
            result[timestamp] = {
                "rsi": self._adjust_value(value)
            }

        return result


class StochasticIndicator(BaseIndicator):
    """随机指标(KDJ)"""

    def _calculate(
        self,
        df: pd.DataFrame
    ) -> Optional[Dict[datetime, Dict[str, float]]]:
        """计算指标值"""
        if len(df) < self.config.window:
            return None

        # 计算KD
        k, d = talib.STOCH(
            df["high"],
            df["low"],
            df["close"],
            fastk_period=self.config.window,
            slowk_period=3,
            slowk_matype=0,
            slowd_period=3,
            slowd_matype=0
        )

        # 计算J
        j = 3 * k - 2 * d

        # 转换为字典
        result = {}
        for i in range(len(df)):
            if i < self.config.window + 3:
                continue
            timestamp = df.index[i]
            result[timestamp] = {
                "k": self._adjust_value(float(k[i])),
                "d": self._adjust_value(float(d[i])),
                "j": self._adjust_value(float(j[i]))
            }

        return result


class CCIIndicator(BaseIndicator):
    """顺势指标"""

    def _calculate(
        self,
        df: pd.DataFrame
    ) -> Optional[Dict[datetime, Dict[str, float]]]:
        """计算指标值"""
        if len(df) < self.config.window:
            return None

        # 计算CCI
        cci = talib.CCI(
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
            value = float(cci[i])
            result[timestamp] = {
                "cci": self._adjust_value(value)
            }

        return result


class WilliamsRIndicator(BaseIndicator):
    """威廉指标"""

    def _calculate(
        self,
        df: pd.DataFrame
    ) -> Optional[Dict[datetime, Dict[str, float]]]:
        """计算指标值"""
        if len(df) < self.config.window:
            return None

        # 计算威廉指标
        wr = talib.WILLR(
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
            value = float(wr[i])
            result[timestamp] = {
                "wr": self._adjust_value(value)
            }

        return result


class ROCIndicator(BaseIndicator):
    """变动率指标"""

    def _calculate(
        self,
        df: pd.DataFrame
    ) -> Optional[Dict[datetime, Dict[str, float]]]:
        """计算指标值"""
        if len(df) < self.config.window:
            return None

        # 获取数据源
        source = self._get_source_data(df)

        # 计算ROC
        roc = talib.ROC(source, timeperiod=self.config.window)
        rocma = talib.SMA(roc, timeperiod=self.config.window)

        # 转换为字典
        result = {}
        for i in range(len(df)):
            if i < self.config.window * 2:
                continue
            timestamp = df.index[i]
            result[timestamp] = {
                "roc": self._adjust_value(float(roc[i])),
                "rocma": self._adjust_value(float(rocma[i]))
            }

        return result


class MFIIndicator(BaseIndicator):
    """资金流量指标"""

    def _calculate(
        self,
        df: pd.DataFrame
    ) -> Optional[Dict[datetime, Dict[str, float]]]:
        """计算指标值"""
        if len(df) < self.config.window:
            return None

        # 计算MFI
        mfi = talib.MFI(
            df["high"],
            df["low"],
            df["close"],
            df["volume"],
            timeperiod=self.config.window
        )

        # 转换为字典
        result = {}
        for i in range(len(df)):
            if i < self.config.window:
                continue
            timestamp = df.index[i]
            value = float(mfi[i])
            result[timestamp] = {
                "mfi": self._adjust_value(value)
            }

        return result 