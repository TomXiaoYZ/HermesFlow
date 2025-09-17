# -*- coding: utf-8 -*-
"""
数据连接器基础抽象类
定义所有数据源连接器的统一接口
"""

from abc import ABC, abstractmethod
from typing import Dict, List, Optional, Any, AsyncGenerator
from datetime import datetime
import asyncio
import logging
from dataclasses import dataclass
from enum import Enum

# 配置日志
logger = logging.getLogger(__name__)


class DataType(Enum):
    """数据类型枚举"""
    KLINE = "kline"              # K线数据
    TICKER = "ticker"            # 行情数据
    ORDERBOOK = "orderbook"      # 订单簿
    TRADE = "trade"              # 成交记录
    BALANCE = "balance"          # 账户余额
    SENTIMENT = "sentiment"      # 情绪数据
    NEWS = "news"                # 新闻数据
    MACRO = "macro"              # 宏观数据


class ConnectionStatus(Enum):
    """连接状态枚举"""
    DISCONNECTED = "disconnected"
    CONNECTING = "connecting"
    CONNECTED = "connected"
    RECONNECTING = "reconnecting"
    ERROR = "error"


@dataclass
class DataPoint:
    """标准化数据点"""
    symbol: str                  # 交易对符号
    timestamp: datetime          # 时间戳
    data_type: DataType         # 数据类型
    data: Dict[str, Any]        # 实际数据
    source: str                 # 数据源
    raw_data: Optional[Dict] = None  # 原始数据（用于调试）


@dataclass
class ConnectionConfig:
    """连接配置"""
    api_key: Optional[str] = None
    api_secret: Optional[str] = None
    passphrase: Optional[str] = None
    sandbox: bool = False
    testnet: bool = False
    timeout: int = 30
    retry_count: int = 3
    retry_delay: float = 1.0
    rate_limit: int = 100  # 每秒请求数限制


class BaseConnector(ABC):
    """数据连接器基础抽象类"""
    
    def __init__(self, config: ConnectionConfig, name: str):
        """
        初始化连接器
        
        Args:
            config: 连接配置
            name: 连接器名称
        """
        self.config = config
        self.name = name
        self.status = ConnectionStatus.DISCONNECTED
        self._session = None
        self._websocket = None
        self._subscriptions = set()
        self._callbacks = {}
        self._rate_limiter = None
        
        # 设置日志
        self.logger = logging.getLogger(f"{__name__}.{name}")
    
    @abstractmethod
    async def connect(self) -> bool:
        """
        建立连接
        
        Returns:
            bool: 连接是否成功
        """
        pass
    
    @abstractmethod
    async def disconnect(self) -> bool:
        """
        断开连接
        
        Returns:
            bool: 断开是否成功
        """
        pass
    
    @abstractmethod
    async def get_symbols(self) -> List[str]:
        """
        获取支持的交易对列表
        
        Returns:
            List[str]: 交易对列表
        """
        pass
    
    @abstractmethod
    async def get_klines(
        self, 
        symbol: str, 
        interval: str, 
        start_time: Optional[datetime] = None,
        end_time: Optional[datetime] = None,
        limit: int = 500
    ) -> List[DataPoint]:
        """
        获取K线数据
        
        Args:
            symbol: 交易对
            interval: 时间间隔
            start_time: 开始时间
            end_time: 结束时间
            limit: 数据条数限制
            
        Returns:
            List[DataPoint]: K线数据列表
        """
        pass
    
    @abstractmethod
    async def get_ticker(self, symbol: str) -> Optional[DataPoint]:
        """
        获取行情数据
        
        Args:
            symbol: 交易对
            
        Returns:
            Optional[DataPoint]: 行情数据
        """
        pass
    
    @abstractmethod
    async def get_orderbook(self, symbol: str, depth: int = 20) -> Optional[DataPoint]:
        """
        获取订单簿数据
        
        Args:
            symbol: 交易对
            depth: 深度
            
        Returns:
            Optional[DataPoint]: 订单簿数据
        """
        pass
    
    @abstractmethod
    async def subscribe_real_time(
        self, 
        symbols: List[str], 
        data_types: List[DataType],
        callback: callable
    ) -> bool:
        """
        订阅实时数据
        
        Args:
            symbols: 交易对列表
            data_types: 数据类型列表
            callback: 回调函数
            
        Returns:
            bool: 订阅是否成功
        """
        pass
    
    @abstractmethod
    async def unsubscribe_real_time(
        self, 
        symbols: List[str], 
        data_types: List[DataType]
    ) -> bool:
        """
        取消订阅实时数据
        
        Args:
            symbols: 交易对列表
            data_types: 数据类型列表
            
        Returns:
            bool: 取消订阅是否成功
        """
        pass
    
    async def health_check(self) -> Dict[str, Any]:
        """
        健康检查
        
        Returns:
            Dict[str, Any]: 健康状态信息
        """
        health_info = {
            "name": self.name,
            "status": self.status.value,
            "connected": self.status == ConnectionStatus.CONNECTED,
            "subscriptions": len(self._subscriptions),
            "timestamp": datetime.now().isoformat()
        }
        
        # 添加测试环境标识
        if hasattr(self.config, 'testnet') and self.config.testnet:
            health_info["testnet_mode"] = True
        if hasattr(self.config, 'sandbox') and self.config.sandbox:
            health_info["sandbox_mode"] = True
            
        return health_info
    
    def get_status(self) -> ConnectionStatus:
        """获取连接状态"""
        return self.status
    
    def is_connected(self) -> bool:
        """检查是否已连接"""
        return self.status == ConnectionStatus.CONNECTED
    
    async def _handle_rate_limit(self):
        """处理速率限制"""
        if self._rate_limiter:
            await self._rate_limiter.acquire()
    
    async def _retry_operation(self, operation, *args, **kwargs):
        """
        重试操作
        
        Args:
            operation: 要重试的操作
            *args: 位置参数
            **kwargs: 关键字参数
            
        Returns:
            操作结果
        """
        last_exception = None
        
        for attempt in range(self.config.retry_count + 1):
            try:
                return await operation(*args, **kwargs)
            except Exception as e:
                last_exception = e
                if attempt < self.config.retry_count:
                    self.logger.warning(
                        f"操作失败，第 {attempt + 1} 次重试: {str(e)}"
                    )
                    await asyncio.sleep(self.config.retry_delay * (2 ** attempt))
                else:
                    self.logger.error(f"操作最终失败: {str(e)}")
        
        raise last_exception
    
    def _normalize_symbol(self, symbol: str) -> str:
        """
        标准化交易对符号
        
        Args:
            symbol: 原始交易对符号
            
        Returns:
            str: 标准化后的交易对符号
        """
        # 默认实现，子类可以重写
        return symbol.upper().replace("/", "").replace("-", "").replace("_", "")
    
    def _create_data_point(
        self, 
        symbol: str, 
        data_type: DataType, 
        data: Dict[str, Any],
        timestamp: Optional[datetime] = None,
        raw_data: Optional[Dict] = None
    ) -> DataPoint:
        """
        创建标准化数据点
        
        Args:
            symbol: 交易对
            data_type: 数据类型
            data: 数据内容
            timestamp: 时间戳
            raw_data: 原始数据
            
        Returns:
            DataPoint: 标准化数据点
        """
        return DataPoint(
            symbol=self._normalize_symbol(symbol),
            timestamp=timestamp or datetime.now(),
            data_type=data_type,
            data=data,
            source=self.name,
            raw_data=raw_data
        )
    
    async def __aenter__(self):
        """异步上下文管理器入口"""
        await self.connect()
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """异步上下文管理器出口"""
        await self.disconnect() 