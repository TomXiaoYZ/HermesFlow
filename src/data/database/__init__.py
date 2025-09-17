"""
数据库模块

提供统一的数据库访问接口
支持Redis缓存、ClickHouse时序数据存储、PostgreSQL配置存储
"""

from .base_database import BaseDatabaseManager, DatabaseConfig
from .redis_manager import RedisManager
from .clickhouse_manager import ClickHouseManager
from .postgres_manager import PostgresManager
from .database_factory import DatabaseFactory

__all__ = [
    'BaseDatabaseManager',
    'DatabaseConfig',
    'RedisManager', 
    'ClickHouseManager',
    'PostgresManager',
    'DatabaseFactory'
]

# 数据库类型常量
DATABASE_TYPES = {
    'redis': 'redis',
    'clickhouse': 'clickhouse', 
    'postgres': 'postgres'
} 