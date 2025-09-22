"""
Redis数据库管理器

提供Redis连接管理、缓存操作和发布/订阅功能
支持连接池管理和自动重连机制
"""

import asyncio
import json
import logging
from typing import Any, Dict, List, Optional, Union, AsyncGenerator
from datetime import datetime, timedelta

try:
    import redis.asyncio as redis
    from redis.asyncio import ConnectionPool
    REDIS_AVAILABLE = True
except ImportError:
    REDIS_AVAILABLE = False
    redis = None
    ConnectionPool = None

from .base_database import (
    BaseDatabaseManager, 
    DatabaseConfig, 
    DatabaseConnectionError,
    DatabaseOperationError,
    DatabaseTimeoutError
)

logger = logging.getLogger(__name__)


class RedisManager(BaseDatabaseManager):
    """Redis数据库管理器"""
    
    def __init__(self, config: DatabaseConfig):
        """
        初始化Redis管理器
        
        Args:
            config: 数据库配置对象
        """
        if not REDIS_AVAILABLE:
            raise ImportError("redis包未安装，请运行: pip install redis[asyncio]")
            
        super().__init__(config)
        self.connection_pool: Optional[ConnectionPool] = None
        self.pubsub_client: Optional[redis.Redis] = None
        
    def get_database_type(self) -> str:
        """获取数据库类型"""
        return "redis"
        
    async def connect(self) -> bool:
        """
        建立Redis连接
        
        Returns:
            连接是否成功
        """
        try:
            async with self._connection_lock:
                if self.client and await self.ping():
                    logger.info("Redis连接已存在且正常")
                    return True
                    
                # 创建连接池
                pool_kwargs = {
                    'host': self.config.host,
                    'port': self.config.port,
                    'db': int(self.config.database),
                    'socket_timeout': self.config.timeout,
                    'socket_connect_timeout': self.config.timeout,
                    'max_connections': self.config.max_connections or self.config.pool_size,
                    'retry_on_timeout': True,
                    'decode_responses': True
                }
                
                # 添加认证信息
                if self.config.password:
                    pool_kwargs['password'] = self.config.password
                if self.config.username:
                    pool_kwargs['username'] = self.config.username
                    
                logger.info(f"正在连接到Redis: {self.config.host}:{self.config.port}")
                
                self.connection_pool = ConnectionPool(**pool_kwargs)
                self.client = redis.Redis(connection_pool=self.connection_pool)
                
                # 测试连接
                await self.ping()
                
                await self._update_health_status(True)
                logger.info("✅ Redis连接成功建立")
                return True
                
        except Exception as e:
            error_msg = f"Redis连接失败: {e}"
            logger.error(error_msg)
            await self._update_health_status(False, e)
            raise DatabaseConnectionError(error_msg) from e
            
    async def disconnect(self) -> None:
        """断开Redis连接"""
        try:
            if self.pubsub_client:
                await self.pubsub_client.close()
                self.pubsub_client = None
                
            if self.client:
                await self.client.close()
                self.client = None
                
            if self.connection_pool:
                await self.connection_pool.disconnect()
                self.connection_pool = None
                
            await self._update_health_status(False)
            logger.info("✅ Redis连接已断开")
            
        except Exception as e:
            logger.error(f"断开Redis连接时出错: {e}")
            
    async def check_connection(self) -> bool:
        """
        检查Redis连接状态
        
        Returns:
            连接是否正常
        """
        if not self.client:
            return False
            
        try:
            return await self.ping()
        except Exception:
            return False
            
    async def ping(self) -> bool:
        """
        Ping Redis服务器
        
        Returns:
            服务器是否响应
        """
        if not self.client:
            return False
            
        try:
            result = await self._measure_latency(self.client.ping)
            return result is True
        except Exception as e:
            logger.warning(f"Redis ping失败: {e}")
            return False
            
    # ==================== 缓存操作 ====================
    
    async def get(self, key: str) -> Optional[str]:
        """
        获取缓存值
        
        Args:
            key: 缓存键
            
        Returns:
            缓存值，不存在返回None
        """
        if not await self.check_connection():
            logger.warning("Redis未连接，无法执行get操作")
            return None
            
        try:
            return await self.client.get(key)
        except Exception as e:
            logger.error(f"Redis GET操作失败 key={key}: {e}")
            return None
            
    async def set(self, key: str, value: str, expire: Optional[int] = None) -> bool:
        """
        设置缓存值
        
        Args:
            key: 缓存键
            value: 缓存值
            expire: 过期时间（秒），None表示永不过期
            
        Returns:
            操作是否成功
        """
        if not await self.check_connection():
            logger.warning("Redis未连接，无法执行set操作")
            return False
            
        try:
            if expire:
                result = await self.client.setex(key, expire, value)
            else:
                result = await self.client.set(key, value)
            return result is True
        except Exception as e:
            logger.error(f"Redis SET操作失败 key={key}: {e}")
            return False
            
    async def delete(self, key: str) -> bool:
        """
        删除缓存
        
        Args:
            key: 缓存键
            
        Returns:
            操作是否成功
        """
        if not await self.check_connection():
            logger.warning("Redis未连接，无法执行delete操作")
            return False
            
        try:
            result = await self.client.delete(key)
            return result > 0
        except Exception as e:
            logger.error(f"Redis DELETE操作失败 key={key}: {e}")
            return False
            
    async def exists(self, key: str) -> bool:
        """
        检查键是否存在
        
        Args:
            key: 缓存键
            
        Returns:
            键是否存在
        """
        if not await self.check_connection():
            logger.warning("Redis未连接，无法执行exists操作")
            return False
            
        try:
            result = await self.client.exists(key)
            return result > 0
        except Exception as e:
            logger.error(f"Redis EXISTS操作失败 key={key}: {e}")
            return False
            
    async def expire(self, key: str, seconds: int) -> bool:
        """
        设置键的过期时间
        
        Args:
            key: 缓存键
            seconds: 过期时间（秒）
            
        Returns:
            操作是否成功
        """
        if not await self.check_connection():
            logger.warning("Redis未连接，无法执行expire操作")
            return False
            
        try:
            result = await self.client.expire(key, seconds)
            return result is True
        except Exception as e:
            logger.error(f"Redis EXPIRE操作失败 key={key}: {e}")
            return False
            
    # ==================== JSON操作 ====================
    
    async def get_json(self, key: str) -> Optional[Dict[str, Any]]:
        """
        获取JSON格式的缓存值
        
        Args:
            key: 缓存键
            
        Returns:
            解析后的JSON对象，失败返回None
        """
        value = await self.get(key)
        if value is None:
            return None
            
        try:
            return json.loads(value)
        except json.JSONDecodeError as e:
            logger.error(f"JSON解析失败 key={key}: {e}")
            return None
            
    async def set_json(self, key: str, value: Dict[str, Any], expire: Optional[int] = None) -> bool:
        """
        设置JSON格式的缓存值
        
        Args:
            key: 缓存键
            value: JSON对象
            expire: 过期时间（秒）
            
        Returns:
            操作是否成功
        """
        try:
            json_str = json.dumps(value, ensure_ascii=False)
            return await self.set(key, json_str, expire)
        except Exception as e:
            logger.error(f"JSON序列化失败 key={key}: {e}")
            return False
            
    # ==================== 哈希操作 ====================
    
    async def hget(self, name: str, key: str) -> Optional[str]:
        """获取哈希表字段值"""
        if not await self.check_connection():
            return None
            
        try:
            return await self.client.hget(name, key)
        except Exception as e:
            logger.error(f"Redis HGET操作失败 name={name} key={key}: {e}")
            return None
            
    async def hset(self, name: str, key: str, value: str) -> bool:
        """设置哈希表字段值"""
        if not await self.check_connection():
            return False
            
        try:
            result = await self.client.hset(name, key, value)
            return result >= 0
        except Exception as e:
            logger.error(f"Redis HSET操作失败 name={name} key={key}: {e}")
            return False
            
    async def hgetall(self, name: str) -> Dict[str, str]:
        """获取哈希表所有字段"""
        if not await self.check_connection():
            return {}
            
        try:
            return await self.client.hgetall(name)
        except Exception as e:
            logger.error(f"Redis HGETALL操作失败 name={name}: {e}")
            return {}
            
    # ==================== 发布/订阅功能 ====================
    
    async def publish(self, channel: str, message: str) -> int:
        """
        发布消息到指定频道
        
        Args:
            channel: 频道名称
            message: 消息内容
            
        Returns:
            接收消息的订阅者数量
        """
        if not await self.check_connection():
            logger.warning("Redis未连接，无法发布消息")
            return 0
            
        try:
            result = await self.client.publish(channel, message)
            logger.debug(f"消息已发布到频道 {channel}: {message[:100]}...")
            return result
        except Exception as e:
            logger.error(f"发布消息失败 channel={channel}: {e}")
            return 0
            
    async def subscribe(self, channels: List[str]) -> AsyncGenerator[Dict[str, Any], None]:
        """
        订阅频道消息
        
        Args:
            channels: 频道列表
            
        Yields:
            消息字典: {"channel": str, "data": str, "type": str}
        """
        if not await self.check_connection():
            logger.error("Redis未连接，无法订阅频道")
            return
            
        try:
            # 创建专用的pubsub客户端
            self.pubsub_client = redis.Redis(connection_pool=self.connection_pool)
            pubsub = self.pubsub_client.pubsub()
            
            # 订阅频道
            await pubsub.subscribe(*channels)
            logger.info(f"已订阅频道: {channels}")
            
            async for message in pubsub.listen():
                if message['type'] == 'message':
                    yield {
                        'channel': message['channel'],
                        'data': message['data'],
                        'type': message['type']
                    }
                    
        except Exception as e:
            logger.error(f"订阅频道失败: {e}")
        finally:
            if self.pubsub_client:
                await pubsub.unsubscribe(*channels)
                await self.pubsub_client.close()
                
    # ==================== 列表操作 ====================
    
    async def lpush(self, key: str, *values: str) -> int:
        """向列表左侧推入元素"""
        if not await self.check_connection():
            return 0
            
        try:
            return await self.client.lpush(key, *values)
        except Exception as e:
            logger.error(f"Redis LPUSH操作失败 key={key}: {e}")
            return 0
            
    async def rpop(self, key: str) -> Optional[str]:
        """从列表右侧弹出元素"""
        if not await self.check_connection():
            return None
            
        try:
            return await self.client.rpop(key)
        except Exception as e:
            logger.error(f"Redis RPOP操作失败 key={key}: {e}")
            return None
            
    async def llen(self, key: str) -> int:
        """获取列表长度"""
        if not await self.check_connection():
            return 0
            
        try:
            return await self.client.llen(key)
        except Exception as e:
            logger.error(f"Redis LLEN操作失败 key={key}: {e}")
            return 0 