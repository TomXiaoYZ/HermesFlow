"""
数据库连接管理
"""
import asyncio
from typing import Optional
from contextlib import asynccontextmanager
import redis.asyncio as redis
import asyncpg
from aiokafka import AIOKafkaProducer, AIOKafkaConsumer
from sqlalchemy.ext.asyncio import create_async_engine, AsyncSession
from sqlalchemy.orm import sessionmaker

from .config import POSTGRESQL_CONFIG, REDIS_CONFIG, KAFKA_CONFIG

class DatabaseManager:
    """数据库连接管理器"""
    _instance = None
    _initialized = False

    def __new__(cls):
        if cls._instance is None:
            cls._instance = super().__new__(cls)
        return cls._instance

    def __init__(self):
        if not self._initialized:
            # PostgreSQL
            self.pg_pool = None
            self.engine = None
            self.async_session = None

            # Redis
            self.redis_pool = None

            # Kafka
            self.kafka_producer = None
            self._initialized = True

    async def init(self):
        """初始化所有数据库连接"""
        await self.init_postgresql()
        await self.init_redis()
        await self.init_kafka()

    async def init_postgresql(self):
        """初始化PostgreSQL连接"""
        if self.pg_pool is None:
            dsn = f"postgresql://{POSTGRESQL_CONFIG['user']}:{POSTGRESQL_CONFIG['password']}@" \
                  f"{POSTGRESQL_CONFIG['host']}:{POSTGRESQL_CONFIG['port']}/{POSTGRESQL_CONFIG['database']}"
            
            # 创建连接池
            self.pg_pool = await asyncpg.create_pool(
                dsn=dsn,
                min_size=POSTGRESQL_CONFIG['min_size'],
                max_size=POSTGRESQL_CONFIG['max_size']
            )

            # 创建SQLAlchemy引擎
            self.engine = create_async_engine(
                f"postgresql+asyncpg://{POSTGRESQL_CONFIG['user']}:{POSTGRESQL_CONFIG['password']}@"
                f"{POSTGRESQL_CONFIG['host']}:{POSTGRESQL_CONFIG['port']}/{POSTGRESQL_CONFIG['database']}"
            )

            # 创建会话工厂
            self.async_session = sessionmaker(
                self.engine, class_=AsyncSession, expire_on_commit=False
            )

    async def init_redis(self):
        """初始化Redis连接"""
        if self.redis_pool is None:
            self.redis_pool = redis.Redis(
                host=REDIS_CONFIG['host'],
                port=REDIS_CONFIG['port'],
                db=REDIS_CONFIG['db'],
                password=REDIS_CONFIG['password'],
                max_connections=REDIS_CONFIG['max_connections'],
                decode_responses=True
            )

    async def init_kafka(self):
        """初始化Kafka生产者"""
        if self.kafka_producer is None:
            self.kafka_producer = AIOKafkaProducer(
                bootstrap_servers=KAFKA_CONFIG['bootstrap_servers'],
                client_id=KAFKA_CONFIG['client_id']
            )
            await self.kafka_producer.start()

    @asynccontextmanager
    async def get_pg_conn(self):
        """获取PostgreSQL连接"""
        async with self.pg_pool.acquire() as conn:
            try:
                yield conn
            finally:
                pass

    @asynccontextmanager
    async def get_session(self):
        """获取SQLAlchemy会话"""
        async with self.async_session() as session:
            try:
                yield session
            finally:
                pass

    def get_redis(self):
        """获取Redis连接"""
        return self.redis_pool

    def get_kafka_producer(self):
        """获取Kafka生产者"""
        return self.kafka_producer

    async def create_kafka_consumer(self, topics):
        """创建Kafka消费者"""
        consumer = AIOKafkaConsumer(
            *topics,
            bootstrap_servers=KAFKA_CONFIG['bootstrap_servers'],
            group_id=KAFKA_CONFIG['group_id'],
            auto_offset_reset=KAFKA_CONFIG['auto_offset_reset'],
            enable_auto_commit=KAFKA_CONFIG['enable_auto_commit'],
            max_poll_interval_ms=KAFKA_CONFIG['max_poll_interval_ms'],
            max_poll_records=KAFKA_CONFIG['max_poll_records']
        )
        await consumer.start()
        return consumer

    async def close(self):
        """关闭所有连接"""
        if self.pg_pool:
            await self.pg_pool.close()
        if self.redis_pool:
            await self.redis_pool.aclose()
        if self.kafka_producer:
            await self.kafka_producer.stop()

# 全局数据库管理器实例
db_manager = DatabaseManager() 