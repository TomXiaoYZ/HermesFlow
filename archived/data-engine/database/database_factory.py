"""
数据库工厂类

统一管理数据库实例的创建、配置和生命周期
支持单例模式和配置管理
"""

import asyncio
import logging
from typing import Dict, Optional, Any, Type
from enum import Enum

from .base_database import BaseDatabaseManager, DatabaseConfig
from .redis_manager import RedisManager
from .clickhouse_manager import ClickHouseManager  
from .postgres_manager import PostgresManager

logger = logging.getLogger(__name__)


class DatabaseType(Enum):
    """数据库类型枚举"""
    REDIS = "redis"
    CLICKHOUSE = "clickhouse"
    POSTGRES = "postgres"


class DatabaseFactory:
    """数据库工厂类"""
    
    _instances: Dict[str, BaseDatabaseManager] = {}
    _configs: Dict[str, DatabaseConfig] = {}
    _lock = asyncio.Lock()
    
    @classmethod
    def register_config(cls, db_type: DatabaseType, config: DatabaseConfig) -> None:
        """
        注册数据库配置
        
        Args:
            db_type: 数据库类型
            config: 数据库配置
        """
        cls._configs[db_type.value] = config
        logger.info(f"✅ 数据库配置已注册: {db_type.value}")
        
    @classmethod
    def register_configs(cls, configs: Dict[str, DatabaseConfig]) -> None:
        """
        批量注册数据库配置
        
        Args:
            configs: 数据库配置字典
        """
        for db_type, config in configs.items():
            if isinstance(db_type, str):
                cls._configs[db_type] = config
            else:
                cls._configs[db_type.value] = config
        logger.info(f"✅ 批量注册数据库配置完成，共 {len(configs)} 个")
        
    @classmethod
    async def get_database(cls, db_type: DatabaseType, 
                          config: Optional[DatabaseConfig] = None) -> BaseDatabaseManager:
        """
        获取数据库实例（单例模式）
        
        Args:
            db_type: 数据库类型
            config: 数据库配置（可选，优先使用已注册的配置）
            
        Returns:
            数据库管理器实例
            
        Raises:
            ValueError: 未找到配置或不支持的数据库类型
        """
        async with cls._lock:
            instance_key = db_type.value
            
            # 如果实例已存在且连接正常，直接返回
            if instance_key in cls._instances:
                instance = cls._instances[instance_key]
                if await instance.check_connection():
                    logger.debug(f"返回现有数据库实例: {db_type.value}")
                    return instance
                else:
                    # 连接异常，移除旧实例
                    logger.warning(f"数据库连接异常，移除旧实例: {db_type.value}")
                    del cls._instances[instance_key]
                    
            # 获取配置
            if config is None:
                if db_type.value not in cls._configs:
                    raise ValueError(f"未找到数据库配置: {db_type.value}")
                config = cls._configs[db_type.value]
                
            # 创建新实例
            manager_class = cls._get_manager_class(db_type)
            instance = manager_class(config)
            
            # 建立连接
            try:
                await instance.connect()
                cls._instances[instance_key] = instance
                logger.info(f"✅ 数据库实例创建并连接成功: {db_type.value}")
                return instance
            except Exception as e:
                logger.error(f"数据库连接失败: {db_type.value}, 错误: {e}")
                raise
                
    @classmethod
    def _get_manager_class(cls, db_type: DatabaseType) -> Type[BaseDatabaseManager]:
        """
        根据数据库类型获取管理器类
        
        Args:
            db_type: 数据库类型
            
        Returns:
            数据库管理器类
            
        Raises:
            ValueError: 不支持的数据库类型
        """
        manager_map = {
            DatabaseType.REDIS: RedisManager,
            DatabaseType.CLICKHOUSE: ClickHouseManager,
            DatabaseType.POSTGRES: PostgresManager
        }
        
        if db_type not in manager_map:
            raise ValueError(f"不支持的数据库类型: {db_type.value}")
            
        return manager_map[db_type]
        
    @classmethod
    async def get_redis(cls, config: Optional[DatabaseConfig] = None) -> RedisManager:
        """
        获取Redis实例的便捷方法
        
        Args:
            config: Redis配置
            
        Returns:
            Redis管理器实例
        """
        return await cls.get_database(DatabaseType.REDIS, config)
        
    @classmethod
    async def get_clickhouse(cls, config: Optional[DatabaseConfig] = None) -> ClickHouseManager:
        """
        获取ClickHouse实例的便捷方法
        
        Args:
            config: ClickHouse配置
            
        Returns:
            ClickHouse管理器实例
        """
        return await cls.get_database(DatabaseType.CLICKHOUSE, config)
        
    @classmethod
    async def get_postgres(cls, config: Optional[DatabaseConfig] = None) -> PostgresManager:
        """
        获取PostgreSQL实例的便捷方法
        
        Args:
            config: PostgreSQL配置
            
        Returns:
            PostgreSQL管理器实例
        """
        return await cls.get_database(DatabaseType.POSTGRES, config)
        
    @classmethod
    async def close_all(cls) -> None:
        """关闭所有数据库连接"""
        async with cls._lock:
            for db_type, instance in cls._instances.items():
                try:
                    await instance.disconnect()
                    logger.info(f"✅ 数据库连接已关闭: {db_type}")
                except Exception as e:
                    logger.error(f"关闭数据库连接失败: {db_type}, 错误: {e}")
                    
            cls._instances.clear()
            logger.info("✅ 所有数据库连接已关闭")
            
    @classmethod
    async def close_database(cls, db_type: DatabaseType) -> None:
        """
        关闭指定数据库连接
        
        Args:
            db_type: 数据库类型
        """
        async with cls._lock:
            instance_key = db_type.value
            if instance_key in cls._instances:
                try:
                    await cls._instances[instance_key].disconnect()
                    del cls._instances[instance_key]
                    logger.info(f"✅ 数据库连接已关闭: {db_type.value}")
                except Exception as e:
                    logger.error(f"关闭数据库连接失败: {db_type.value}, 错误: {e}")
            else:
                logger.warning(f"数据库实例不存在: {db_type.value}")
                
    @classmethod
    async def health_check(cls) -> Dict[str, Dict[str, Any]]:
        """
        检查所有数据库的健康状态
        
        Returns:
            健康状态字典
        """
        health_status = {}
        
        for db_type, instance in cls._instances.items():
            try:
                is_connected = await instance.check_connection()
                health_info = instance.get_health_status().to_dict()
                health_info['is_connected'] = is_connected
                health_status[db_type] = health_info
            except Exception as e:
                health_status[db_type] = {
                    'database_type': db_type,
                    'is_connected': False,
                    'error': str(e)
                }
                
        return health_status
        
    @classmethod
    def list_registered_configs(cls) -> Dict[str, Dict[str, Any]]:
        """
        列出所有已注册的配置
        
        Returns:
            配置信息字典（不包含敏感信息）
        """
        configs_info = {}
        
        for db_type, config in cls._configs.items():
            configs_info[db_type] = {
                'host': config.host,
                'port': config.port,
                'database': config.database,
                'timeout': config.timeout,
                'pool_size': config.pool_size,
                'testnet': config.testnet,
                'has_username': bool(config.username),
                'has_password': bool(config.password)
            }
            
        return configs_info
        
    @classmethod
    def get_instance_count(cls) -> int:
        """
        获取当前实例数量
        
        Returns:
            实例数量
        """
        return len(cls._instances)
        
    @classmethod
    def clear_configs(cls) -> None:
        """清空所有配置"""
        cls._configs.clear()
        logger.info("✅ 数据库配置已清空")
        
    @classmethod
    async def create_default_configs(cls, testnet: bool = True) -> Dict[str, DatabaseConfig]:
        """
        创建默认数据库配置
        
        Args:
            testnet: 是否为测试环境
            
        Returns:
            默认配置字典
        """
        configs = {
            DatabaseType.REDIS.value: DatabaseConfig(
                host="localhost",
                port=6379,
                database="0",
                timeout=30,
                pool_size=10,
                testnet=testnet,
                max_connections=20
            ),
            DatabaseType.CLICKHOUSE.value: DatabaseConfig(
                host="localhost", 
                port=8123,
                database="hermes_flow",
                username="default",
                timeout=60,
                pool_size=5,
                testnet=testnet
            ),
            DatabaseType.POSTGRES.value: DatabaseConfig(
                host="localhost",
                port=5432,
                database="hermes_flow",
                username="postgres",
                password="password",
                timeout=30,
                pool_size=5,
                testnet=testnet,
                ssl_mode="prefer"
            )
        }
        
        # 注册配置
        cls.register_configs(configs)
        
        logger.info(f"✅ 默认数据库配置创建完成（testnet={testnet}）")
        return configs


# 便捷的全局函数
async def get_redis(config: Optional[DatabaseConfig] = None) -> RedisManager:
    """获取Redis实例的全局函数"""
    return await DatabaseFactory.get_redis(config)


async def get_clickhouse(config: Optional[DatabaseConfig] = None) -> ClickHouseManager:
    """获取ClickHouse实例的全局函数"""
    return await DatabaseFactory.get_clickhouse(config)


async def get_postgres(config: Optional[DatabaseConfig] = None) -> PostgresManager:
    """获取PostgreSQL实例的全局函数"""
    return await DatabaseFactory.get_postgres(config)


async def close_all_databases() -> None:
    """关闭所有数据库连接的全局函数"""
    await DatabaseFactory.close_all()


async def database_health_check() -> Dict[str, Dict[str, Any]]:
    """数据库健康检查的全局函数"""
    return await DatabaseFactory.health_check() 