"""
数据库基础抽象类

定义统一的数据库接口规范
所有数据库管理器都应该继承此基类
"""

import asyncio
import logging
from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Any, Dict, List, Optional, Union
from datetime import datetime
import time

logger = logging.getLogger(__name__)


@dataclass
class DatabaseConfig:
    """数据库配置类"""
    host: str = "localhost"
    port: int = 6379
    username: Optional[str] = None
    password: Optional[str] = None
    database: str = "0"
    timeout: int = 30
    pool_size: int = 10
    testnet: bool = False
    
    # Redis特有配置
    max_connections: Optional[int] = None
    
    # ClickHouse特有配置
    cluster_name: Optional[str] = None
    
    # PostgreSQL特有配置
    ssl_mode: str = "prefer"
    pool_timeout: int = 30


class DatabaseHealthStatus:
    """数据库健康状态"""
    
    def __init__(self, database_type: str):
        self.database_type = database_type
        self.is_connected = False
        self.last_check_time = None
        self.connection_count = 0
        self.error_count = 0
        self.last_error = None
        self.latency_ms = 0
        
    def to_dict(self) -> Dict[str, Any]:
        """转换为字典格式"""
        return {
            "database_type": self.database_type,
            "is_connected": self.is_connected,
            "last_check_time": self.last_check_time.isoformat() if self.last_check_time else None,
            "connection_count": self.connection_count,
            "error_count": self.error_count,
            "last_error": str(self.last_error) if self.last_error else None,
            "latency_ms": self.latency_ms
        }


class BaseDatabaseManager(ABC):
    """数据库管理器基类"""
    
    def __init__(self, config: DatabaseConfig):
        """
        初始化数据库管理器
        
        Args:
            config: 数据库配置对象
        """
        self.config = config
        self.client = None
        self.health_status = DatabaseHealthStatus(self.get_database_type())
        self._connection_lock = asyncio.Lock()
        
    @abstractmethod
    def get_database_type(self) -> str:
        """获取数据库类型"""
        pass
        
    @abstractmethod
    async def connect(self) -> bool:
        """
        建立数据库连接
        
        Returns:
            连接是否成功
        """
        pass
        
    @abstractmethod
    async def disconnect(self) -> None:
        """断开数据库连接"""
        pass
        
    @abstractmethod
    async def check_connection(self) -> bool:
        """
        检查数据库连接状态
        
        Returns:
            连接是否正常
        """
        pass
        
    @abstractmethod
    async def ping(self) -> bool:
        """
        Ping数据库服务器
        
        Returns:
            服务器是否响应
        """
        pass
        
    def get_health_status(self) -> DatabaseHealthStatus:
        """
        获取数据库健康状态
        
        Returns:
            健康状态对象
        """
        return self.health_status
        
    async def _update_health_status(self, is_connected: bool, error: Optional[Exception] = None):
        """更新健康状态"""
        self.health_status.is_connected = is_connected
        self.health_status.last_check_time = datetime.utcnow()
        
        if is_connected:
            self.health_status.connection_count += 1
        else:
            self.health_status.error_count += 1
            self.health_status.last_error = error
            
    async def _measure_latency(self, operation_func):
        """测量操作延迟"""
        start_time = time.time()
        try:
            result = await operation_func()
            end_time = time.time()
            self.health_status.latency_ms = (end_time - start_time) * 1000
            return result
        except Exception as e:
            end_time = time.time()
            self.health_status.latency_ms = (end_time - start_time) * 1000
            raise e
            
    async def __aenter__(self):
        """异步上下文管理器入口"""
        await self.connect()
        return self
        
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """异步上下文管理器出口"""
        await self.disconnect()
        
    def __str__(self) -> str:
        """字符串表示"""
        return f"{self.get_database_type()}Manager(host={self.config.host}, port={self.config.port})"
        
    def __repr__(self) -> str:
        """详细字符串表示"""
        return (f"{self.get_database_type()}Manager("
                f"host={self.config.host}, "
                f"port={self.config.port}, "
                f"connected={self.health_status.is_connected})")


class CacheOperation:
    """缓存操作枚举"""
    GET = "GET"
    SET = "SET" 
    DELETE = "DELETE"
    EXISTS = "EXISTS"
    EXPIRE = "EXPIRE"
    
    
class TimeSeriesOperation:
    """时序数据操作枚举"""
    INSERT = "INSERT"
    SELECT = "SELECT"
    DELETE = "DELETE"
    CREATE_TABLE = "CREATE_TABLE"
    DROP_TABLE = "DROP_TABLE"
    

class DatabaseConnectionError(Exception):
    """数据库连接错误"""
    pass


class DatabaseOperationError(Exception):
    """数据库操作错误"""
    pass


class DatabaseTimeoutError(Exception):
    """数据库超时错误"""
    pass 