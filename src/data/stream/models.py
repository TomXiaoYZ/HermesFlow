#!/usr/bin/env python3
"""
数据流基础模型模块 (Stream Models Module)

定义数据流处理中使用的所有数据结构和模型，包括：
- 数据流类型和状态枚举
- 标准化数据模型
- 数据质量和性能指标
- 验证和转换函数

所有数据结构都经过优化，支持高频处理和序列化
"""

import time
import uuid
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Union, Any, Tuple
from decimal import Decimal
from enum import Enum
from datetime import datetime, timezone
import json
import hashlib

class StreamDataType(Enum):
    """数据流类型枚举"""
    MARKET_DATA = "market_data"          # 市场数据
    ORDER_BOOK = "order_book"            # 订单簿数据
    TRADE_DATA = "trade_data"            # 成交数据
    TICKER_DATA = "ticker_data"          # 行情数据
    KLINE_DATA = "kline_data"            # K线数据
    USER_DATA = "user_data"              # 用户数据
    SYSTEM_DATA = "system_data"          # 系统数据

class DataStatus(Enum):
    """数据状态枚举"""
    PENDING = "pending"                  # 待处理
    PROCESSING = "processing"            # 处理中
    PROCESSED = "processed"              # 已处理
    STORED = "stored"                    # 已存储
    ERROR = "error"                      # 错误
    EXPIRED = "expired"                  # 已过期

class QualityLevel(Enum):
    """数据质量等级"""
    EXCELLENT = "excellent"              # 优秀
    GOOD = "good"                       # 良好
    FAIR = "fair"                       # 一般
    POOR = "poor"                       # 较差
    UNUSABLE = "unusable"               # 不可用

@dataclass
class StreamData:
    """数据流基础数据类"""
    # 基本信息
    id: str = field(default_factory=lambda: str(uuid.uuid4()))
    data_type: StreamDataType = StreamDataType.MARKET_DATA
    source: str = ""                                    # 数据源(交易所)
    symbol: str = ""                                    # 交易对
    
    # 时间信息
    timestamp: float = field(default_factory=time.time)     # 数据时间戳
    received_time: float = field(default_factory=time.time) # 接收时间戳
    processed_time: Optional[float] = None              # 处理时间戳
    
    # 数据内容
    data: Dict[str, Any] = field(default_factory=dict) # 原始数据
    normalized_data: Dict[str, Any] = field(default_factory=dict)  # 标准化数据
    
    # 状态和质量
    status: DataStatus = DataStatus.PENDING
    quality: Optional[QualityLevel] = None
    
    # 元数据
    sequence: Optional[int] = None                      # 序列号
    checksum: Optional[str] = None                      # 校验和
    size_bytes: int = 0                                # 数据大小
    latency_ms: Optional[float] = None                 # 延迟(毫秒)
    
    def __post_init__(self):
        """后初始化处理"""
        if not self.checksum:
            self.checksum = self._calculate_checksum()
        if not self.size_bytes:
            self.size_bytes = len(json.dumps(self.data, default=str))
        if not self.latency_ms and self.processed_time:
            self.latency_ms = (self.processed_time - self.timestamp) * 1000
    
    def _calculate_checksum(self) -> str:
        """计算数据校验和"""
        data_str = json.dumps(self.data, sort_keys=True, default=str)
        return hashlib.md5(data_str.encode()).hexdigest()[:8]
    
    def validate(self) -> bool:
        """验证数据完整性"""
        try:
            # 基本字段验证
            assert self.id, "ID不能为空"
            assert self.source, "数据源不能为空"
            assert self.symbol, "交易对不能为空"
            assert self.timestamp > 0, "时间戳必须大于0"
            
            # 数据内容验证
            assert isinstance(self.data, dict), "数据必须是字典类型"
            assert len(self.data) > 0, "数据不能为空"
            
            # 校验和验证
            current_checksum = self._calculate_checksum()
            assert current_checksum == self.checksum, "数据校验和不匹配"
            
            return True
        except AssertionError as e:
            return False
    
    def mark_processed(self):
        """标记为已处理"""
        self.processed_time = time.time()
        self.status = DataStatus.PROCESSED
        if not self.latency_ms:
            self.latency_ms = (self.processed_time - self.timestamp) * 1000
    
    def to_dict(self) -> Dict[str, Any]:
        """转换为字典"""
        return {
            'id': self.id,
            'data_type': self.data_type.value,
            'source': self.source,
            'symbol': self.symbol,
            'timestamp': self.timestamp,
            'received_time': self.received_time,
            'processed_time': self.processed_time,
            'data': self.data,
            'normalized_data': self.normalized_data,
            'status': self.status.value,
            'quality': self.quality.value if self.quality else None,
            'sequence': self.sequence,
            'checksum': self.checksum,
            'size_bytes': self.size_bytes,
            'latency_ms': self.latency_ms
        }

@dataclass  
class MarketData(StreamData):
    """市场数据类"""
    data_type: StreamDataType = field(default=StreamDataType.MARKET_DATA, init=False)
    
    # 市场数据特有字段
    price: Optional[Decimal] = None                     # 价格
    volume: Optional[Decimal] = None                    # 成交量
    price_change: Optional[Decimal] = None              # 价格变化
    price_change_percent: Optional[Decimal] = None      # 价格变化百分比
    
    # 24小时统计
    high_24h: Optional[Decimal] = None                  # 24小时最高价
    low_24h: Optional[Decimal] = None                   # 24小时最低价
    volume_24h: Optional[Decimal] = None                # 24小时成交量
    
    def __post_init__(self):
        """市场数据后初始化"""
        super().__post_init__()
        # 从原始数据中提取字段
        if self.data:
            self._extract_market_fields()
    
    def _extract_market_fields(self):
        """从原始数据中提取市场数据字段"""
        try:
            self.price = Decimal(str(self.data.get('price', 0)))
            self.volume = Decimal(str(self.data.get('volume', 0)))
            self.price_change = Decimal(str(self.data.get('priceChange', 0)))
            self.price_change_percent = Decimal(str(self.data.get('priceChangePercent', 0)))
            self.high_24h = Decimal(str(self.data.get('highPrice', 0)))
            self.low_24h = Decimal(str(self.data.get('lowPrice', 0)))
            self.volume_24h = Decimal(str(self.data.get('volume', 0)))
        except (ValueError, TypeError):
            pass  # 字段提取失败，保持None值

@dataclass
class OrderBookData(StreamData):
    """订单簿数据类"""
    data_type: StreamDataType = field(default=StreamDataType.ORDER_BOOK, init=False)
    
    # 订单簿特有字段
    bids: List[Tuple[Decimal, Decimal]] = field(default_factory=list)  # 买单 [价格, 数量]
    asks: List[Tuple[Decimal, Decimal]] = field(default_factory=list)  # 卖单 [价格, 数量]
    last_update_id: Optional[int] = None                # 最后更新ID
    
    def __post_init__(self):
        """订单簿数据后初始化"""
        super().__post_init__()
        if self.data:
            self._extract_orderbook_fields()
    
    def _extract_orderbook_fields(self):
        """从原始数据中提取订单簿字段"""
        try:
            # 处理买单
            if 'bids' in self.data:
                self.bids = [(Decimal(price), Decimal(qty)) 
                           for price, qty in self.data['bids']]
            
            # 处理卖单
            if 'asks' in self.data:
                self.asks = [(Decimal(price), Decimal(qty)) 
                           for price, qty in self.data['asks']]
            
            # 更新ID
            self.last_update_id = self.data.get('lastUpdateId')
        except (ValueError, TypeError, KeyError):
            pass
    
    def get_best_bid(self) -> Optional[Tuple[Decimal, Decimal]]:
        """获取最佳买价"""
        return self.bids[0] if self.bids else None
    
    def get_best_ask(self) -> Optional[Tuple[Decimal, Decimal]]:
        """获取最佳卖价"""
        return self.asks[0] if self.asks else None
    
    def get_spread(self) -> Optional[Decimal]:
        """获取买卖价差"""
        best_bid = self.get_best_bid()
        best_ask = self.get_best_ask()
        if best_bid and best_ask:
            return best_ask[0] - best_bid[0]
        return None

@dataclass
class TradeData(StreamData):
    """成交数据类"""
    data_type: StreamDataType = field(default=StreamDataType.TRADE_DATA, init=False)
    
    # 成交数据特有字段
    trade_id: Optional[str] = None                      # 成交ID
    price: Optional[Decimal] = None                     # 成交价格
    quantity: Optional[Decimal] = None                  # 成交数量
    is_buyer_maker: Optional[bool] = None               # 是否买方挂单
    trade_time: Optional[float] = None                  # 成交时间
    
    def __post_init__(self):
        """成交数据后初始化"""
        super().__post_init__()
        if self.data:
            self._extract_trade_fields()
    
    def _extract_trade_fields(self):
        """从原始数据中提取成交数据字段"""
        try:
            self.trade_id = str(self.data.get('id', ''))
            self.price = Decimal(str(self.data.get('price', 0)))
            self.quantity = Decimal(str(self.data.get('qty', 0)))
            self.is_buyer_maker = self.data.get('isBuyerMaker', False)
            self.trade_time = float(self.data.get('time', self.timestamp))
        except (ValueError, TypeError):
            pass

@dataclass
class DataQuality:
    """数据质量评估类"""
    # 基本指标
    total_messages: int = 0                             # 总消息数
    valid_messages: int = 0                             # 有效消息数
    invalid_messages: int = 0                           # 无效消息数
    duplicate_messages: int = 0                         # 重复消息数
    
    # 延迟指标
    avg_latency_ms: float = 0.0                        # 平均延迟
    max_latency_ms: float = 0.0                        # 最大延迟
    min_latency_ms: float = float('inf')               # 最小延迟
    
    # 质量评分
    completeness_score: float = 0.0                    # 完整性评分 (0-1)
    accuracy_score: float = 0.0                        # 准确性评分 (0-1)
    timeliness_score: float = 0.0                      # 及时性评分 (0-1)
    consistency_score: float = 0.0                     # 一致性评分 (0-1)
    
    # 时间窗口
    window_start: float = field(default_factory=time.time)
    window_end: Optional[float] = None
    
    def calculate_quality_level(self) -> QualityLevel:
        """计算整体质量等级"""
        overall_score = (
            self.completeness_score + 
            self.accuracy_score + 
            self.timeliness_score + 
            self.consistency_score
        ) / 4
        
        if overall_score >= 0.9:
            return QualityLevel.EXCELLENT
        elif overall_score >= 0.8:
            return QualityLevel.GOOD
        elif overall_score >= 0.6:
            return QualityLevel.FAIR
        elif overall_score >= 0.4:
            return QualityLevel.POOR
        else:
            return QualityLevel.UNUSABLE
    
    def get_validity_rate(self) -> float:
        """获取有效性比率"""
        if self.total_messages == 0:
            return 0.0
        return self.valid_messages / self.total_messages
    
    def update_latency(self, latency_ms: float):
        """更新延迟指标"""
        self.avg_latency_ms = (
            (self.avg_latency_ms * self.valid_messages + latency_ms) / 
            (self.valid_messages + 1)
        )
        self.max_latency_ms = max(self.max_latency_ms, latency_ms)
        self.min_latency_ms = min(self.min_latency_ms, latency_ms)

@dataclass
class PerformanceMetrics:
    """性能指标类"""
    # 吞吐量指标
    messages_per_second: float = 0.0                   # 每秒消息数
    bytes_per_second: float = 0.0                      # 每秒字节数
    peak_throughput: float = 0.0                       # 峰值吞吐量
    
    # 资源使用指标
    memory_usage_mb: float = 0.0                       # 内存使用(MB)
    cpu_usage_percent: float = 0.0                     # CPU使用率(%)
    network_io_kbps: float = 0.0                      # 网络IO(KB/s)
    
    # 连接指标
    active_connections: int = 0                         # 活跃连接数
    failed_connections: int = 0                         # 失败连接数
    reconnections: int = 0                             # 重连次数
    
    # 错误指标
    error_rate: float = 0.0                            # 错误率
    timeout_count: int = 0                             # 超时次数
    exception_count: int = 0                           # 异常次数
    
    # 时间统计
    uptime_seconds: float = 0.0                        # 运行时间(秒)
    last_update: float = field(default_factory=time.time)
    
    def calculate_availability(self) -> float:
        """计算可用性百分比"""
        total_attempts = self.active_connections + self.failed_connections
        if total_attempts == 0:
            return 100.0
        return (self.active_connections / total_attempts) * 100
    
    def is_healthy(self, thresholds: Dict[str, float] = None) -> bool:
        """检查性能是否健康"""
        if not thresholds:
            thresholds = {
                'max_error_rate': 0.05,         # 最大错误率5%
                'max_memory_mb': 2048,          # 最大内存2GB
                'max_cpu_percent': 80,          # 最大CPU使用率80%
                'min_availability': 99.0        # 最小可用性99%
            }
        
        checks = [
            self.error_rate <= thresholds['max_error_rate'],
            self.memory_usage_mb <= thresholds['max_memory_mb'],
            self.cpu_usage_percent <= thresholds['max_cpu_percent'],
            self.calculate_availability() >= thresholds['min_availability']
        ]
        
        return all(checks)

# 数据转换函数
def normalize_decimal_precision(value: Union[str, float, Decimal], precision: int = 8) -> Decimal:
    """标准化小数精度"""
    try:
        decimal_value = Decimal(str(value))
        return decimal_value.quantize(Decimal('0.' + '0' * precision))
    except (ValueError, TypeError):
        return Decimal('0')

def normalize_timestamp(timestamp: Union[int, float, str]) -> float:
    """标准化时间戳为UTC秒"""
    try:
        ts = float(timestamp)
        # 如果是毫秒时间戳，转换为秒
        if ts > 1e10:  
            ts = ts / 1000
        return ts
    except (ValueError, TypeError):
        return time.time()

def validate_symbol_format(symbol: str) -> bool:
    """验证交易对格式"""
    if not symbol or not isinstance(symbol, str):
        return False
    
    # 基本格式检查: 3-20个字符，字母数字组合
    if not (3 <= len(symbol) <= 20):
        return False
    
    # 检查是否包含常见的分隔符
    separators = ['/', '-', '_', '']
    for sep in separators:
        if sep in symbol or symbol.replace(sep, '').isalnum():
            return True
    
    return False

def create_stream_data(data_type: StreamDataType, source: str, symbol: str, 
                      raw_data: Dict[str, Any]) -> StreamData:
    """创建标准化数据流对象的工厂函数"""
    # 根据数据类型选择合适的类
    if data_type == StreamDataType.MARKET_DATA:
        return MarketData(
            source=source,
            symbol=symbol,
            data=raw_data
        )
    elif data_type == StreamDataType.ORDER_BOOK:
        return OrderBookData(
            source=source,
            symbol=symbol,
            data=raw_data
        )
    elif data_type == StreamDataType.TRADE_DATA:
        return TradeData(
            source=source,
            symbol=symbol,
            data=raw_data
        )
    else:
        # 默认返回基础StreamData
        return StreamData(
            data_type=data_type,
            source=source,
            symbol=symbol,
            data=raw_data
        )

# 常用常量
DEFAULT_DECIMAL_PRECISION = 8
MAX_SYMBOL_LENGTH = 20
MIN_SYMBOL_LENGTH = 3
MILLISECOND_TIMESTAMP_THRESHOLD = 1e10

# 数据验证规则
DATA_VALIDATION_RULES = {
    StreamDataType.MARKET_DATA: {
        'required_fields': ['price', 'volume'],
        'numeric_fields': ['price', 'volume', 'high', 'low'],
        'max_price_change_percent': 50.0  # 最大价格变化50%
    },
    StreamDataType.ORDER_BOOK: {
        'required_fields': ['bids', 'asks'],
        'max_levels': 1000,  # 最大订单簿层级
        'min_spread_ratio': 0.0001  # 最小价差比率
    },
    StreamDataType.TRADE_DATA: {
        'required_fields': ['price', 'quantity'],
        'numeric_fields': ['price', 'quantity'],
        'max_trade_size_ratio': 0.1  # 最大单笔交易占24h交易量比率
    }
} 