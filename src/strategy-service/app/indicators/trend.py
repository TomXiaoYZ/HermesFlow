"""
趋势指标
"""
from datetime import datetime
from typing import Dict, Optional

import numpy as np
import pandas as pd
import talib

from app.indicators.base import BaseIndicator, IndicatorConfig


class MAIndicator(BaseIndicator):
    """移动平均线"""

    def _calculate(
        self,
        df: pd.DataFrame
    ) -> Optional[Dict[datetime, Dict[str, float]]]:
        """计算指标值"""
        if len(df) < self.config.window:
            return None

        # 获取数据源
        source = self._get_source_data(df)

        # 计算移动平均线
        ma = talib.SMA(source, timeperiod=self.config.window)

        # 转换为字典
        result = {}
        for i in range(len(df)):
            if i < self.config.window - 1:
                continue
            timestamp = df.index[i]
            value = float(ma[i])
            result[timestamp] = {
                "ma": self._adjust_value(value)
            }

        return result


class EMAIndicator(BaseIndicator):
    """指数移动平均线"""

    def _calculate(
        self,
        df: pd.DataFrame
    ) -> Optional[Dict[datetime, Dict[str, float]]]:
        """计算指标值"""
        if len(df) < self.config.window:
            return None

        # 获取数据源
        source = self._get_source_data(df)

        # 计算指数移动平均线
        ema = talib.EMA(source, timeperiod=self.config.window)

        # 转换为字典
        result = {}
        for i in range(len(df)):
            if i < self.config.window - 1:
                continue
            timestamp = df.index[i]
            value = float(ema[i])
            result[timestamp] = {
                "ema": self._adjust_value(value)
            }

        return result


class MACDIndicator(BaseIndicator):
    """MACD指标"""

    def _calculate(
        self,
        df: pd.DataFrame
    ) -> Optional[Dict[datetime, Dict[str, float]]]:
        """计算指标值"""
        if len(df) < self.config.window:
            return None

        # 获取数据源
        source = self._get_source_data(df)

        # 计算MACD
        macd, signal, hist = talib.MACD(
            source,
            fastperiod=12,
            slowperiod=26,
            signalperiod=9
        )

        # 转换为字典
        result = {}
        for i in range(len(df)):
            if i < 33:  # 26 + 9 - 2
                continue
            timestamp = df.index[i]
            result[timestamp] = {
                "macd": self._adjust_value(float(macd[i])),
                "signal": self._adjust_value(float(signal[i])),
                "hist": self._adjust_value(float(hist[i]))
            }

        return result


class BollingerBandsIndicator(BaseIndicator):
    """布林带"""

    def _calculate(
        self,
        df: pd.DataFrame
    ) -> Optional[Dict[datetime, Dict[str, float]]]:
        """计算指标值"""
        if len(df) < self.config.window:
            return None

        # 获取数据源
        source = self._get_source_data(df)

        # 计算布林带
        upper, middle, lower = talib.BBANDS(
            source,
            timeperiod=self.config.window,
            nbdevup=2,
            nbdevdn=2,
            matype=0
        )

        # 转换为字典
        result = {}
        for i in range(len(df)):
            if i < self.config.window - 1:
                continue
            timestamp = df.index[i]
            result[timestamp] = {
                "upper": self._adjust_value(float(upper[i])),
                "middle": self._adjust_value(float(middle[i])),
                "lower": self._adjust_value(float(lower[i]))
            }

        return result


class ADXIndicator(BaseIndicator):
    """平均趋向指数"""

    def _calculate(
        self,
        df: pd.DataFrame
    ) -> Optional[Dict[datetime, Dict[str, float]]]:
        """计算指标值"""
        if len(df) < self.config.window:
            return None

        # 计算ADX
        adx = talib.ADX(
            df["high"],
            df["low"],
            df["close"],
            timeperiod=self.config.window
        )
        pdi = talib.PLUS_DI(
            df["high"],
            df["low"],
            df["close"],
            timeperiod=self.config.window
        )
        mdi = talib.MINUS_DI(
            df["high"],
            df["low"],
            df["close"],
            timeperiod=self.config.window
        )

        # 转换为字典
        result = {}
        for i in range(len(df)):
            if i < self.config.window - 1:
                continue
            timestamp = df.index[i]
            result[timestamp] = {
                "adx": self._adjust_value(float(adx[i])),
                "pdi": self._adjust_value(float(pdi[i])),
                "mdi": self._adjust_value(float(mdi[i]))
            }

        return result


class IchimokuIndicator(BaseIndicator):
    """一目均衡图"""

    def _calculate(
        self,
        df: pd.DataFrame
    ) -> Optional[Dict[datetime, Dict[str, float]]]:
        """计算指标值"""
        if len(df) < self.config.window:
            return None

        # 计算转换线和基准线
        high_9 = df["high"].rolling(window=9).max()
        low_9 = df["low"].rolling(window=9).min()
        high_26 = df["high"].rolling(window=26).max()
        low_26 = df["low"].rolling(window=26).min()
        conversion = (high_9 + low_9) / 2
        base = (high_26 + low_26) / 2

        # 计算先行带
        leading_span_a = ((conversion + base) / 2).shift(26)
        high_52 = df["high"].rolling(window=52).max()
        low_52 = df["low"].rolling(window=52).min()
        leading_span_b = ((high_52 + low_52) / 2).shift(26)

        # 计算延迟线
        lagging_span = df["close"].shift(-26)

        # 转换为字典
        result = {}
        for i in range(len(df)):
            if i < 52:
                continue
            timestamp = df.index[i]
            result[timestamp] = {
                "conversion": self._adjust_value(float(conversion[i])),
                "base": self._adjust_value(float(base[i])),
                "leading_span_a": self._adjust_value(float(leading_span_a[i])),
                "leading_span_b": self._adjust_value(float(leading_span_b[i])),
                "lagging_span": self._adjust_value(float(lagging_span[i]))
                if i + 26 < len(df) else 0.0
            }

        return result 