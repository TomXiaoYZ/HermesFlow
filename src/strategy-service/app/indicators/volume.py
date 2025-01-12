"""
成交量指标
"""
from datetime import datetime
from typing import Dict, Optional

import numpy as np
import pandas as pd
import talib

from app.indicators.base import BaseIndicator, IndicatorConfig


class OBVIndicator(BaseIndicator):
    """能量潮"""

    def _calculate(
        self,
        df: pd.DataFrame
    ) -> Optional[Dict[datetime, Dict[str, float]]]:
        """计算指标值"""
        if len(df) < 2:
            return None

        # 计算OBV
        obv = talib.OBV(df["close"], df["volume"])

        # 转换为字典
        result = {}
        for i in range(len(df)):
            if i < 1:
                continue
            timestamp = df.index[i]
            value = float(obv[i])
            result[timestamp] = {
                "obv": self._adjust_value(value)
            }

        return result


class VWAPIndicator(BaseIndicator):
    """成交量加权平均价格"""

    def _calculate(
        self,
        df: pd.DataFrame
    ) -> Optional[Dict[datetime, Dict[str, float]]]:
        """计算指标值"""
        if len(df) < self.config.window:
            return None

        # 计算VWAP
        typical_price = (df["high"] + df["low"] + df["close"]) / 3
        vwap = (
            (typical_price * df["volume"]).rolling(window=self.config.window).sum()
            / df["volume"].rolling(window=self.config.window).sum()
        )

        # 转换为字典
        result = {}
        for i in range(len(df)):
            if i < self.config.window:
                continue
            timestamp = df.index[i]
            value = float(vwap[i])
            result[timestamp] = {
                "vwap": self._adjust_value(value)
            }

        return result


class ADIndicator(BaseIndicator):
    """累积/派发线"""

    def _calculate(
        self,
        df: pd.DataFrame
    ) -> Optional[Dict[datetime, Dict[str, float]]]:
        """计算指标值"""
        if len(df) < 2:
            return None

        # 计算AD
        ad = talib.AD(
            df["high"],
            df["low"],
            df["close"],
            df["volume"]
        )

        # 转换为字典
        result = {}
        for i in range(len(df)):
            if i < 1:
                continue
            timestamp = df.index[i]
            value = float(ad[i])
            result[timestamp] = {
                "ad": self._adjust_value(value)
            }

        return result


class ADOSCIndicator(BaseIndicator):
    """震荡指标"""

    def _calculate(
        self,
        df: pd.DataFrame
    ) -> Optional[Dict[datetime, Dict[str, float]]]:
        """计算指标值"""
        if len(df) < 10:  # 3 + 10 - 2
            return None

        # 计算ADOSC
        adosc = talib.ADOSC(
            df["high"],
            df["low"],
            df["close"],
            df["volume"],
            fastperiod=3,
            slowperiod=10
        )

        # 转换为字典
        result = {}
        for i in range(len(df)):
            if i < 10:
                continue
            timestamp = df.index[i]
            value = float(adosc[i])
            result[timestamp] = {
                "adosc": self._adjust_value(value)
            }

        return result


class CMFIndicator(BaseIndicator):
    """钱德资金流量"""

    def _calculate(
        self,
        df: pd.DataFrame
    ) -> Optional[Dict[datetime, Dict[str, float]]]:
        """计算指标值"""
        if len(df) < self.config.window:
            return None

        # 计算资金流量乘数
        high_low = df["high"] - df["low"]
        close_low = df["close"] - df["low"]
        high_close = df["high"] - df["close"]
        multiplier = np.where(
            high_low > 0,
            ((2 * close_low) - high_low) / high_low,
            0.0
        )

        # 计算资金流量量
        money_flow_volume = multiplier * df["volume"]

        # 计算CMF
        cmf = (
            money_flow_volume.rolling(window=self.config.window).sum()
            / df["volume"].rolling(window=self.config.window).sum()
        )

        # 转换为字典
        result = {}
        for i in range(len(df)):
            if i < self.config.window:
                continue
            timestamp = df.index[i]
            value = float(cmf[i])
            result[timestamp] = {
                "cmf": self._adjust_value(value)
            }

        return result


class EMVIndicator(BaseIndicator):
    """简易波动指标"""

    def _calculate(
        self,
        df: pd.DataFrame
    ) -> Optional[Dict[datetime, Dict[str, float]]]:
        """计算指标值"""
        if len(df) < self.config.window:
            return None

        # 计算中间价
        mid_price = (df["high"] + df["low"]) / 2
        mid_price_diff = mid_price - mid_price.shift(1)

        # 计算成交量除以价格范围
        volume_range = df["volume"] / (df["high"] - df["low"])

        # 计算EMV
        emv = mid_price_diff * volume_range * 100000000
        emv_ma = emv.rolling(window=self.config.window).mean()

        # 转换为字典
        result = {}
        for i in range(len(df)):
            if i < self.config.window:
                continue
            timestamp = df.index[i]
            result[timestamp] = {
                "emv": self._adjust_value(float(emv[i])),
                "emv_ma": self._adjust_value(float(emv_ma[i]))
            }

        return result 