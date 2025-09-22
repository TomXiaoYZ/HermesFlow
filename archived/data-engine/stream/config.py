#!/usr/bin/env python3
"""
数据流配置管理模块 (Stream Configuration Module)

定义和管理数据流处理相关的所有配置项，包括：
- 流数据配置
- 订阅配置 
- 存储策略配置
- 监控告警配置

支持配置验证、默认值设置和动态配置更新
"""

import asyncio
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Set, Union, Any
from enum import Enum
import json

class StreamType(Enum):
    """数据流类型枚举"""
    MARKET_DATA = "market_data"          # 市场数据流
    ORDER_BOOK = "order_book"            # 订单簿数据流
    TRADE_DATA = "trade_data"            # 成交数据流
    TICKER_DATA = "ticker_data"          # 行情数据流
    KLINE_DATA = "kline_data"            # K线数据流
    USER_DATA = "user_data"              # 用户数据流

class CompressionType(Enum):
    """数据压缩类型"""
    NONE = "none"                        # 无压缩
    GZIP = "gzip"                        # GZIP压缩
    LZ4 = "lz4"                          # LZ4压缩
    SNAPPY = "snappy"                    # Snappy压缩

class StorageStrategy(Enum):
    """存储策略"""
    HOT_ONLY = "hot_only"                # 仅热存储
    COLD_ONLY = "cold_only"              # 仅冷存储
    HOT_COLD = "hot_cold"                # 热冷混合存储
    TIERED = "tiered"                    # 分层存储

@dataclass
class SubscriptionConfig:
    """订阅配置类"""
    # 基本配置
    symbols: List[str] = field(default_factory=list)       # 交易对列表
    exchanges: List[str] = field(default_factory=list)     # 交易所列表
    stream_types: List[StreamType] = field(default_factory=list)  # 数据流类型
    
    # 连接配置
    max_connections_per_exchange: int = 10                  # 每个交易所最大连接数
    connection_timeout: float = 30.0                       # 连接超时时间(秒)
    heartbeat_interval: float = 30.0                       # 心跳间隔(秒)
    max_reconnect_attempts: int = 5                         # 最大重连次数
    reconnect_delay: float = 5.0                           # 重连延迟(秒)
    
    # 数据配置
    max_buffer_size: int = 10000                           # 最大缓冲区大小
    batch_size: int = 100                                  # 批处理大小
    flush_interval: float = 1.0                            # 刷新间隔(秒)
    
    # 质量控制
    enable_data_validation: bool = True                     # 启用数据验证
    max_latency_ms: float = 100.0                         # 最大延迟(毫秒)
    duplicate_detection: bool = True                        # 重复检测
    
    def __post_init__(self):
        """后初始化验证"""
        if not self.symbols:
            self.symbols = ['BTCUSDT', 'ETHUSDT']  # 默认交易对
        if not self.exchanges:
            self.exchanges = ['binance', 'okx']    # 默认交易所
        if not self.stream_types:
            self.stream_types = [StreamType.MARKET_DATA, StreamType.TRADE_DATA]
        
        # 验证配置参数
        assert self.max_connections_per_exchange > 0, "最大连接数必须大于0"
        assert self.connection_timeout > 0, "连接超时时间必须大于0"
        assert self.max_buffer_size > 0, "缓冲区大小必须大于0"

@dataclass
class StorageConfig:
    """存储配置类"""
    # 存储策略
    strategy: StorageStrategy = StorageStrategy.HOT_COLD
    
    # 热存储配置 (Redis)
    hot_storage_enabled: bool = True
    hot_storage_ttl: int = 3600                            # 热存储TTL(秒)
    hot_storage_max_memory: str = "2GB"                    # 最大内存使用
    hot_compression: CompressionType = CompressionType.LZ4
    
    # 冷存储配置 (ClickHouse)
    cold_storage_enabled: bool = True
    cold_storage_batch_size: int = 1000                    # 批量写入大小
    cold_storage_flush_interval: int = 60                  # 刷新间隔(秒)
    cold_compression: CompressionType = CompressionType.LZ4
    cold_partition_by: str = "toYYYYMM(timestamp)"         # 分区策略
    
    # 数据保留策略
    retention_policy: Dict[StreamType, int] = field(default_factory=lambda: {
        StreamType.MARKET_DATA: 30,      # 市场数据保留30天
        StreamType.ORDER_BOOK: 7,        # 订单簿保留7天
        StreamType.TRADE_DATA: 90,       # 成交数据保留90天
        StreamType.TICKER_DATA: 30,      # 行情数据保留30天
        StreamType.KLINE_DATA: 365,      # K线数据保留365天
    })
    
    # 分层存储配置
    tier_hot_to_warm_hours: int = 24                       # 热到温存储时间(小时)
    tier_warm_to_cold_days: int = 7                        # 温到冷存储时间(天)
    
    def __post_init__(self):
        """后初始化验证"""
        assert self.hot_storage_ttl > 0, "热存储TTL必须大于0"
        assert self.cold_storage_batch_size > 0, "冷存储批量大小必须大于0"

@dataclass
class MonitorConfig:
    """监控配置类"""
    # 基本监控
    enabled: bool = True
    metrics_collection_interval: float = 5.0               # 指标收集间隔(秒)
    health_check_interval: float = 30.0                   # 健康检查间隔(秒)
    
    # 性能监控
    track_latency: bool = True                             # 跟踪延迟
    track_throughput: bool = True                          # 跟踪吞吐量
    track_error_rate: bool = True                          # 跟踪错误率
    track_memory_usage: bool = True                        # 跟踪内存使用
    
    # 告警配置
    alert_enabled: bool = True
    alert_channels: List[str] = field(default_factory=lambda: ['slack', 'email'])
    
    # 告警阈值
    max_latency_threshold_ms: float = 100.0               # 最大延迟阈值(毫秒)
    min_throughput_threshold: int = 1000                  # 最小吞吐量阈值(消息/秒)
    max_error_rate_threshold: float = 0.05               # 最大错误率阈值(5%)
    max_memory_usage_threshold: float = 0.85             # 最大内存使用阈值(85%)
    max_connection_failure_rate: float = 0.1             # 最大连接失败率(10%)
    
    # 日志配置
    log_level: str = "INFO"
    log_format: str = "%(asctime)s - %(name)s - %(levelname)s - %(message)s"
    log_file_enabled: bool = True
    log_file_path: str = "logs/stream.log"
    log_rotation_size: str = "100MB"
    log_backup_count: int = 5
    
    def __post_init__(self):
        """后初始化验证"""
        assert 0 < self.max_error_rate_threshold < 1, "错误率阈值必须在0-1之间"
        assert 0 < self.max_memory_usage_threshold < 1, "内存使用阈值必须在0-1之间"
        assert self.log_level in ['DEBUG', 'INFO', 'WARNING', 'ERROR'], "无效的日志级别"

@dataclass
class StreamConfig:
    """主数据流配置类"""
    # 子配置
    subscription: SubscriptionConfig = field(default_factory=SubscriptionConfig)
    storage: StorageConfig = field(default_factory=StorageConfig)  
    monitor: MonitorConfig = field(default_factory=MonitorConfig)
    
    # 全局配置
    environment: str = "development"                        # 环境: development/production
    debug_enabled: bool = False                            # 调试模式
    
    # 异步配置
    max_workers: int = 10                                  # 最大工作线程数
    event_loop_policy: Optional[str] = None               # 事件循环策略
    task_timeout: float = 300.0                           # 任务超时时间(秒)
    
    # 安全配置
    enable_ssl: bool = True                                # 启用SSL
    ssl_verify: bool = True                               # SSL验证
    api_key_rotation_enabled: bool = False                 # API密钥轮换
    
    # 扩展配置
    plugins: List[str] = field(default_factory=list)      # 插件列表
    custom_settings: Dict[str, Any] = field(default_factory=dict)  # 自定义设置
    
    @classmethod
    def from_dict(cls, config_dict: Dict[str, Any]) -> 'StreamConfig':
        """从字典创建配置"""
        subscription_config = SubscriptionConfig(**config_dict.get('subscription', {}))
        storage_config = StorageConfig(**config_dict.get('storage', {}))
        monitor_config = MonitorConfig(**config_dict.get('monitor', {}))
        
        # 移除子配置，避免重复
        main_config = {k: v for k, v in config_dict.items() 
                      if k not in ['subscription', 'storage', 'monitor']}
        
        return cls(
            subscription=subscription_config,
            storage=storage_config,
            monitor=monitor_config,
            **main_config
        )
    
    @classmethod
    def from_json_file(cls, file_path: str) -> 'StreamConfig':
        """从JSON文件加载配置"""
        with open(file_path, 'r', encoding='utf-8') as f:
            config_dict = json.load(f)
        return cls.from_dict(config_dict)
    
    def to_dict(self) -> Dict[str, Any]:
        """转换为字典"""
        def convert_dataclass(obj):
            if hasattr(obj, '__dataclass_fields__'):
                return {k: convert_dataclass(v) for k, v in obj.__dict__.items()}
            elif isinstance(obj, Enum):
                return obj.value
            elif isinstance(obj, list):
                return [convert_dataclass(item) for item in obj]
            elif isinstance(obj, dict):
                return {k: convert_dataclass(v) for k, v in obj.items()}
            else:
                return obj
        
        return convert_dataclass(self)
    
    def to_json_file(self, file_path: str):
        """保存到JSON文件"""
        with open(file_path, 'w', encoding='utf-8') as f:
            json.dump(self.to_dict(), f, indent=2, ensure_ascii=False)
    
    def validate(self) -> bool:
        """验证配置有效性"""
        try:
            # 验证订阅配置
            assert len(self.subscription.symbols) > 0, "至少需要一个交易对"
            assert len(self.subscription.exchanges) > 0, "至少需要一个交易所"
            
            # 验证存储配置
            if self.storage.strategy == StorageStrategy.HOT_ONLY:
                assert self.storage.hot_storage_enabled, "热存储策略需要启用热存储"
            elif self.storage.strategy == StorageStrategy.COLD_ONLY:
                assert self.storage.cold_storage_enabled, "冷存储策略需要启用冷存储"
            
            # 验证监控配置
            assert 0 < self.monitor.max_error_rate_threshold < 1, "错误率阈值无效"
            
            # 验证全局配置
            assert self.environment in ['development', 'production'], "无效的环境配置"
            assert self.max_workers > 0, "工作线程数必须大于0"
            
            return True
        except AssertionError as e:
            raise ValueError(f"配置验证失败: {e}")
    
    def update(self, **kwargs):
        """动态更新配置"""
        for key, value in kwargs.items():
            if hasattr(self, key):
                setattr(self, key, value)
            else:
                self.custom_settings[key] = value
        
        # 重新验证配置
        self.validate()

# 预定义配置模板
DEVELOPMENT_CONFIG = StreamConfig(
    environment="development",
    debug_enabled=True,
    subscription=SubscriptionConfig(
        symbols=['BTCUSDT', 'ETHUSDT'],
        exchanges=['binance'],
        max_connections_per_exchange=5,
        max_buffer_size=5000
    ),
    storage=StorageConfig(
        strategy=StorageStrategy.HOT_ONLY,
        hot_storage_ttl=1800,
        cold_storage_enabled=False
    ),
    monitor=MonitorConfig(
        metrics_collection_interval=10.0,
        alert_enabled=False
    )
)

PRODUCTION_CONFIG = StreamConfig(
    environment="production", 
    debug_enabled=False,
    subscription=SubscriptionConfig(
        symbols=['BTCUSDT', 'ETHUSDT', 'BNBUSDT', 'ADAUSDT', 'DOTUSDT'],
        exchanges=['binance', 'okx', 'bitget'],
        max_connections_per_exchange=20,
        max_buffer_size=50000
    ),
    storage=StorageConfig(
        strategy=StorageStrategy.TIERED,
        hot_storage_ttl=3600,
        cold_storage_enabled=True
    ),
    monitor=MonitorConfig(
        metrics_collection_interval=5.0,
        alert_enabled=True,
        alert_channels=['slack', 'email']
    )
) 