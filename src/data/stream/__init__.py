"""
HermesFlow 数据流处理模块

这个模块提供了实时数据流处理的核心功能，包括：
- 数据连接管理
- 数据流处理和路由
- 数据存储管理
- 性能监控和质量控制
"""

from .connection_manager import ConnectionManager, ConnectionState
from .stream_manager import StreamManager
from .data_router import DataRouter
from .data_processor import DataProcessor, DataValidator, DataNormalizer
from .storage_manager import StorageManager, BaseStorage, HotDataCache, ColdDataStorage
from .monitor import StreamMonitor, PerformanceMonitor, MonitorStatus, MonitorMetric
from .models import StreamData, StreamDataType
from .config import StreamConfig, MonitorConfig, SubscriptionConfig

__all__ = [
    # 连接管理
    'ConnectionManager',
    'ConnectionState',
    
    # 流管理
    'StreamManager',
    
    # 数据路由
    'DataRouter',
    
    # 数据处理
    'DataProcessor',
    'DataValidator', 
    'DataNormalizer',
    
    # 存储管理
    'StorageManager',
    'BaseStorage',
    'HotDataCache',
    'ColdDataStorage',
    
    # 监控
    'StreamMonitor',
    'PerformanceMonitor',
    'MonitorStatus',
    'MonitorMetric',
    
    # 数据模型
    'StreamData',
    'StreamDataType',
    'SubscriptionConfig',
    
    # 配置
    'StreamConfig',
    'MonitorConfig'
]

__version__ = '1.0.0' 