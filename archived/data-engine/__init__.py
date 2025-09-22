# -*- coding: utf-8 -*-
"""
HermesFlow 数据模块
提供多交易所数据接入、实时行情、历史数据管理等功能
"""

# 只导入已经实现的连接器
from .connectors.binance_connector import BinanceConnector
from .connectors.base_connector import BaseConnector, DataPoint, DataType, ConnectionStatus, ConnectionConfig

__version__ = "1.0.0"
__author__ = "HermesFlow Team"

__all__ = [
    # 基础类
    "BaseConnector",
    "DataPoint", 
    "DataType",
    "ConnectionStatus",
    "ConnectionConfig",
    
    # 已实现的连接器
    "BinanceConnector",
] 