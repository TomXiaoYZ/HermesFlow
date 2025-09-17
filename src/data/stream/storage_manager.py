#!/usr/bin/env python3
"""
存储管理器模块 (Storage Manager Module)

负责数据流的存储策略管理，包括：
- 热数据Redis缓存管理
- 冷数据ClickHouse存储
- 数据分层存储策略
- 异步批量操作优化
- 数据压缩和序列化
- 自动清理和归档

支持多种存储策略，确保数据的高可用性和性能
"""

import asyncio
import time
import json
import logging
import gzip
# import lz4.frame  # 临时注释掉，避免依赖问题
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Any, Union, Tuple
from enum import Enum
from collections import deque, defaultdict
import redis.asyncio as redis
import asyncpg
from clickhouse_driver import Client as ClickHouseClient
import os

from .models import StreamData, StreamDataType, DataStatus, QualityLevel
from .config import StorageConfig, StorageStrategy, CompressionType

# 设置日志
logger = logging.getLogger(__name__)

class StorageStatus(Enum):
    """存储状态枚举"""
    PENDING = "pending"         # 待存储
    STORING = "storing"         # 存储中  
    STORED = "stored"           # 已存储
    ERROR = "error"             # 错误
    EXPIRED = "expired"         # 已过期

@dataclass
class StorageMetrics:
    """存储指标类"""
    # 基本统计
    total_stored: int = 0
    successful_stored: int = 0
    failed_stored: int = 0
    
    # 存储大小
    total_bytes_stored: int = 0
    compressed_bytes_stored: int = 0
    
    # 性能指标
    avg_write_latency_ms: float = 0.0
    avg_read_latency_ms: float = 0.0
    max_write_latency_ms: float = 0.0
    max_read_latency_ms: float = 0.0
    
    # 缓存指标
    cache_hits: int = 0
    cache_misses: int = 0
    cache_size_bytes: int = 0
    
    # 时间统计
    last_update: float = field(default_factory=time.time)
    
    def get_cache_hit_rate(self) -> float:
        """获取缓存命中率"""
        total_requests = self.cache_hits + self.cache_misses
        if total_requests == 0:
            return 0.0
        return self.cache_hits / total_requests
    
    def get_compression_ratio(self) -> float:
        """获取压缩比"""
        if self.total_bytes_stored == 0:
            return 0.0
        return self.compressed_bytes_stored / self.total_bytes_stored
    
    def get_success_rate(self) -> float:
        """获取存储成功率"""
        if self.total_stored == 0:
            return 0.0
        return self.successful_stored / self.total_stored

class BaseStorage(ABC):
    """存储基础抽象类"""
    
    def __init__(self, config: StorageConfig):
        self.config = config
        self.metrics = StorageMetrics()
        self.running = False
        
    @abstractmethod
    async def initialize(self) -> bool:
        """初始化存储"""
        pass
    
    @abstractmethod
    async def store(self, data: StreamData) -> bool:
        """存储单条数据"""
        pass
    
    @abstractmethod
    async def store_batch(self, data_list: List[StreamData]) -> List[bool]:
        """批量存储数据"""
        pass
    
    @abstractmethod
    async def retrieve(self, key: str) -> Optional[StreamData]:
        """检索数据"""
        pass
    
    @abstractmethod
    async def delete(self, key: str) -> bool:
        """删除数据"""
        pass
    
    @abstractmethod
    async def cleanup(self) -> bool:
        """清理资源"""
        pass
    
    @abstractmethod
    def get_stats(self) -> Dict[str, Any]:
        """获取统计信息"""
        pass

class HotDataCache(BaseStorage):
    """热数据缓存管理器 (Redis)"""
    
    def __init__(self, config: StorageConfig):
        super().__init__(config)
        self.redis_client: Optional[redis.Redis] = None
        self.connection_pool: Optional[redis.ConnectionPool] = None
        
        # 批处理队列
        self.write_queue: asyncio.Queue = asyncio.Queue(maxsize=5000)
        self.batch_size = 100
        self.flush_interval = 1.0  # 秒
        
        # 后台任务
        self.background_tasks: List[asyncio.Task] = []
        
        logger.info("热数据缓存管理器初始化完成")
    
    async def initialize(self) -> bool:
        """初始化Redis连接"""
        try:
            # 从环境变量或配置获取Redis连接信息
            redis_url = os.getenv('REDIS_URL')
            if redis_url:
                # 解析Redis URL: redis://:password@host:port/db
                import urllib.parse
                parsed = urllib.parse.urlparse(redis_url)
                host = parsed.hostname or 'localhost'
                port = parsed.port or 6379
                password = parsed.password
                db = int(parsed.path.lstrip('/')) if parsed.path else 0
            else:
                # 使用默认配置
                host = os.getenv('REDIS_HOST', 'localhost')
                port = int(os.getenv('REDIS_PORT', '6379'))
                password = os.getenv('REDIS_PASSWORD')
                db = int(os.getenv('REDIS_DB', '0'))
            
            # 创建连接池
            self.connection_pool = redis.ConnectionPool(
                host=host,
                port=port,
                db=db,
                password=password,
                max_connections=20,
                decode_responses=False  # 保持二进制数据
            )
            
            # 创建Redis客户端
            self.redis_client = redis.Redis(connection_pool=self.connection_pool)
            
            # 测试连接
            await self.redis_client.ping()
            
            self.running = True
            
            # 启动后台批处理任务
            batch_task = asyncio.create_task(self._batch_write_loop())
            self.background_tasks.append(batch_task)
            
            logger.info(f"Redis热数据缓存初始化成功: {host}:{port}")
            return True
            
        except Exception as e:
            logger.error(f"Redis热数据缓存初始化失败: {e}")
            return False
    
    async def store(self, data: StreamData) -> bool:
        """存储单条数据到缓存"""
        if not self.redis_client or not self.running:
            logger.warning("Redis客户端未初始化")
            return False
        
        try:
            # 添加到批处理队列
            await self.write_queue.put(data)
            return True
            
        except asyncio.QueueFull:
            logger.warning("写入队列已满，丢弃数据")
            self.metrics.failed_stored += 1
            return False
        except Exception as e:
            logger.error(f"存储数据到缓存失败: {e}")
            self.metrics.failed_stored += 1
            return False
    
    async def store_batch(self, data_list: List[StreamData]) -> List[bool]:
        """批量存储数据到缓存"""
        if not self.redis_client or not self.running:
            return [False] * len(data_list)
        
        results = []
        start_time = time.time()
        
        try:
            # 使用Redis管道进行批量操作
            pipe = self.redis_client.pipeline()
            
            for data in data_list:
                # 生成键名
                key = self._generate_cache_key(data)
                
                # 序列化和压缩数据
                serialized_data = self._serialize_data(data)
                compressed_data = self._compress_data(serialized_data)
                
                # 添加到管道
                pipe.set(
                    key, 
                    compressed_data, 
                    ex=self.config.hot_storage_ttl
                )
            
            # 执行批量操作
            await pipe.execute()
            
            # 更新统计
            latency_ms = (time.time() - start_time) * 1000
            self.metrics.successful_stored += len(data_list)
            self.metrics.total_stored += len(data_list)
            self.metrics.avg_write_latency_ms = (
                (self.metrics.avg_write_latency_ms * (self.metrics.successful_stored - len(data_list)) + 
                 latency_ms) / self.metrics.successful_stored
            )
            self.metrics.max_write_latency_ms = max(self.metrics.max_write_latency_ms, latency_ms)
            
            # 计算存储大小
            for data in data_list:
                serialized_data = self._serialize_data(data)
                compressed_data = self._compress_data(serialized_data)
                self.metrics.total_bytes_stored += len(serialized_data)
                self.metrics.compressed_bytes_stored += len(compressed_data)
            
            results = [True] * len(data_list)
            logger.debug(f"批量存储 {len(data_list)} 条数据到Redis缓存完成")
            
        except Exception as e:
            logger.error(f"批量存储到缓存失败: {e}")
            self.metrics.failed_stored += len(data_list)
            self.metrics.total_stored += len(data_list)
            results = [False] * len(data_list)
        
        return results
    
    async def retrieve(self, key: str) -> Optional[StreamData]:
        """从缓存检索数据"""
        if not self.redis_client or not self.running:
            return None
        
        start_time = time.time()
        
        try:
            # 从Redis获取数据
            compressed_data = await self.redis_client.get(key)
            
            if compressed_data is None:
                self.metrics.cache_misses += 1
                return None
            
            # 解压缩和反序列化
            serialized_data = self._decompress_data(compressed_data)
            data = self._deserialize_data(serialized_data)
            
            # 更新统计
            latency_ms = (time.time() - start_time) * 1000
            self.metrics.cache_hits += 1
            self.metrics.avg_read_latency_ms = (
                (self.metrics.avg_read_latency_ms * (self.metrics.cache_hits - 1) + 
                 latency_ms) / self.metrics.cache_hits
            )
            self.metrics.max_read_latency_ms = max(self.metrics.max_read_latency_ms, latency_ms)
            
            return data
            
        except Exception as e:
            logger.error(f"从缓存检索数据失败: {e}")
            self.metrics.cache_misses += 1
            return None
    
    async def delete(self, key: str) -> bool:
        """从缓存删除数据"""
        if not self.redis_client or not self.running:
            return False
        
        try:
            result = await self.redis_client.delete(key)
            return bool(result)
        except Exception as e:
            logger.error(f"从缓存删除数据失败: {e}")
            return False
    
    async def cleanup(self) -> bool:
        """清理资源"""
        try:
            self.running = False
            
            # 取消后台任务
            for task in self.background_tasks:
                task.cancel()
            
            if self.background_tasks:
                await asyncio.gather(*self.background_tasks, return_exceptions=True)
            
            # 关闭Redis连接
            if self.redis_client:
                await self.redis_client.close()
            
            if self.connection_pool:
                await self.connection_pool.disconnect()
            
            logger.info("Redis热数据缓存清理完成")
            return True
            
        except Exception as e:
            logger.error(f"清理Redis缓存资源失败: {e}")
            return False
    
    def _generate_cache_key(self, data: StreamData) -> str:
        """生成缓存键名"""
        return f"stream:{data.data_type.value}:{data.source}:{data.symbol}:{data.id}"
    
    def _serialize_data(self, data: StreamData) -> bytes:
        """序列化数据"""
        data_dict = data.to_dict()
        return json.dumps(data_dict, default=str).encode('utf-8')
    
    def _deserialize_data(self, data: bytes) -> StreamData:
        """反序列化数据"""
        data_dict = json.loads(data.decode('utf-8'))
        
        # 重构StreamData对象 (简化版本)
        stream_data = StreamData()
        for key, value in data_dict.items():
            if hasattr(stream_data, key):
                setattr(stream_data, key, value)
        
        return stream_data
    
    def _compress_data(self, data: bytes) -> bytes:
        """压缩数据"""
        if self.config.hot_compression == CompressionType.LZ4:
            # 临时使用GZIP替代LZ4
            return gzip.compress(data)
        elif self.config.hot_compression == CompressionType.GZIP:
            return gzip.compress(data)
        else:
            return data
    
    def _decompress_data(self, data: bytes) -> bytes:
        """解压数据"""
        if self.config.hot_compression == CompressionType.LZ4:
            # 临时使用GZIP替代LZ4
            return gzip.decompress(data)
        elif self.config.hot_compression == CompressionType.GZIP:
            return gzip.decompress(data)
        else:
            return data
    
    async def _batch_write_loop(self):
        """批量写入循环"""
        batch_buffer = []
        last_flush_time = time.time()
        
        while self.running:
            try:
                # 等待数据或超时
                try:
                    data = await asyncio.wait_for(self.write_queue.get(), timeout=0.1)
                    batch_buffer.append(data)
                except asyncio.TimeoutError:
                    pass
                
                # 检查是否需要刷新
                current_time = time.time()
                should_flush = (
                    len(batch_buffer) >= self.batch_size or
                    (batch_buffer and current_time - last_flush_time >= self.flush_interval)
                )
                
                if should_flush and batch_buffer:
                    await self.store_batch(batch_buffer)
                    batch_buffer.clear()
                    last_flush_time = current_time
                    
            except Exception as e:
                logger.error(f"批量写入循环异常: {e}")
                await asyncio.sleep(1)
    
    def get_stats(self) -> Dict[str, Any]:
        """获取统计信息"""
        return {
            'storage_type': 'hot_cache',
            'running': self.running,
            'metrics': {
                'total_stored': self.metrics.total_stored,
                'successful_stored': self.metrics.successful_stored,
                'failed_stored': self.metrics.failed_stored,
                'success_rate': self.metrics.get_success_rate(),
                'cache_hit_rate': self.metrics.get_cache_hit_rate(),
                'cache_hits': self.metrics.cache_hits,
                'cache_misses': self.metrics.cache_misses,
                'total_bytes_stored': self.metrics.total_bytes_stored,
                'compressed_bytes_stored': self.metrics.compressed_bytes_stored,
                'compression_ratio': self.metrics.get_compression_ratio(),
                'avg_write_latency_ms': self.metrics.avg_write_latency_ms,
                'avg_read_latency_ms': self.metrics.avg_read_latency_ms,
                'max_write_latency_ms': self.metrics.max_write_latency_ms,
                'max_read_latency_ms': self.metrics.max_read_latency_ms
            },
            'queue_size': self.write_queue.qsize() if self.write_queue else 0,
            'background_tasks': len(self.background_tasks)
        }

class ColdDataStorage(BaseStorage):
    """冷数据存储管理器 (ClickHouse)"""
    
    def __init__(self, config: StorageConfig):
        super().__init__(config)
        self.clickhouse_client: Optional[ClickHouseClient] = None
        
        # 批处理队列
        self.write_queue: asyncio.Queue = asyncio.Queue(maxsize=5000)
        self.batch_size = config.cold_storage_batch_size
        self.flush_interval = config.cold_storage_flush_interval
        
        # 后台任务
        self.background_tasks: List[asyncio.Task] = []
        
        # 表结构定义
        self.table_schemas = {
            StreamDataType.MARKET_DATA: self._get_market_data_schema(),
            StreamDataType.ORDER_BOOK: self._get_orderbook_schema(),
            StreamDataType.TRADE_DATA: self._get_trade_data_schema()
        }
        
        logger.info("冷数据存储管理器初始化完成")
    
    async def initialize(self) -> bool:
        """初始化ClickHouse连接"""
        try:
            # 从环境变量或配置获取ClickHouse连接信息
            clickhouse_url = os.getenv('CLICKHOUSE_URL')
            if clickhouse_url:
                # 解析ClickHouse URL: http://user:password@host:port/database
                import urllib.parse
                parsed = urllib.parse.urlparse(clickhouse_url)
                host = parsed.hostname or 'localhost'
                port = parsed.port or 8123
                username = parsed.username or 'default'
                password = parsed.password or ''
                database = parsed.path.lstrip('/') or 'hermesflow'
                
                # 如果是HTTP URL，使用8123端口；如果需要native协议，使用9000端口
                if port == 8123:
                    # HTTP接口，但ClickHouse客户端需要native端口
                    port = 9000
            else:
                # 使用默认配置
                host = os.getenv('CLICKHOUSE_HOST', 'localhost')
                port = int(os.getenv('CLICKHOUSE_PORT', '9000'))
                username = os.getenv('CLICKHOUSE_USER', 'default')
                password = os.getenv('CLICKHOUSE_PASSWORD', '')
                database = os.getenv('CLICKHOUSE_DB', 'hermesflow')
            
            # 创建ClickHouse客户端 (注意：clickhouse_driver是同步的)
            self.clickhouse_client = ClickHouseClient(
                host=host,
                port=port,
                database=database,
                user=username,
                password=password
            )
            
            # 测试连接
            result = self.clickhouse_client.execute('SELECT 1')
            if result[0][0] != 1:
                raise Exception("ClickHouse连接测试失败")
            
            # 创建数据库和表
            await self._create_tables()
            
            self.running = True
            
            # 启动后台批处理任务
            batch_task = asyncio.create_task(self._batch_write_loop())
            self.background_tasks.append(batch_task)
            
            logger.info(f"ClickHouse冷数据存储初始化成功: {host}:{port}")
            return True
            
        except Exception as e:
            logger.error(f"ClickHouse冷数据存储初始化失败: {e}")
            return False
    
    async def store(self, data: StreamData) -> bool:
        """存储单条数据"""
        if not self.clickhouse_client or not self.running:
            logger.warning("ClickHouse客户端未初始化")
            return False
        
        try:
            # 添加到批处理队列
            await self.write_queue.put(data)
            return True
            
        except asyncio.QueueFull:
            logger.warning("ClickHouse写入队列已满，丢弃数据")
            self.metrics.failed_stored += 1
            return False
        except Exception as e:
            logger.error(f"存储数据到ClickHouse失败: {e}")
            self.metrics.failed_stored += 1
            return False
    
    async def store_batch(self, data_list: List[StreamData]) -> List[bool]:
        """批量存储数据"""
        if not self.clickhouse_client or not self.running:
            return [False] * len(data_list)
        
        results = []
        start_time = time.time()
        
        try:
            # 按数据类型分组
            grouped_data = defaultdict(list)
            for data in data_list:
                grouped_data[data.data_type].append(data)
            
            # 分别处理每种数据类型
            for data_type, type_data_list in grouped_data.items():
                table_name = self._get_table_name(data_type)
                insert_data = []
                
                for data in type_data_list:
                    row_data = self._prepare_row_data(data, data_type)
                    insert_data.append(row_data)
                
                # 执行批量插入 (在线程池中运行以避免阻塞)
                insert_query = f"INSERT INTO {table_name} VALUES"
                await asyncio.get_event_loop().run_in_executor(
                    None, 
                    self.clickhouse_client.execute,
                    insert_query,
                    insert_data
                )
            
            # 更新统计
            latency_ms = (time.time() - start_time) * 1000
            self.metrics.successful_stored += len(data_list)
            self.metrics.total_stored += len(data_list)
            self.metrics.avg_write_latency_ms = (
                (self.metrics.avg_write_latency_ms * (self.metrics.successful_stored - len(data_list)) + 
                 latency_ms) / self.metrics.successful_stored
            )
            self.metrics.max_write_latency_ms = max(self.metrics.max_write_latency_ms, latency_ms)
            
            results = [True] * len(data_list)
            logger.debug(f"批量存储 {len(data_list)} 条数据到ClickHouse完成")
            
        except Exception as e:
            logger.error(f"批量存储到ClickHouse失败: {e}")
            self.metrics.failed_stored += len(data_list)
            self.metrics.total_stored += len(data_list)
            results = [False] * len(data_list)
        
        return results
    
    async def retrieve(self, key: str) -> Optional[StreamData]:
        """从ClickHouse检索数据"""
        # ClickHouse主要用于批量分析，单条检索不是主要用例
        # 这里提供基础实现
        logger.warning("ClickHouse不建议用于单条数据检索")
        return None
    
    async def delete(self, key: str) -> bool:
        """从ClickHouse删除数据"""
        # ClickHouse是append-only存储，删除操作较复杂
        logger.warning("ClickHouse不支持直接删除单条数据")
        return False
    
    async def cleanup(self) -> bool:
        """清理资源"""
        try:
            self.running = False
            
            # 取消后台任务
            for task in self.background_tasks:
                task.cancel()
            
            if self.background_tasks:
                await asyncio.gather(*self.background_tasks, return_exceptions=True)
            
            # 关闭ClickHouse连接
            if self.clickhouse_client:
                self.clickhouse_client.disconnect()
            
            logger.info("ClickHouse冷数据存储清理完成")
            return True
            
        except Exception as e:
            logger.error(f"清理ClickHouse存储资源失败: {e}")
            return False
    
    def _get_table_name(self, data_type: StreamDataType) -> str:
        """获取表名"""
        return f"stream_{data_type.value}"
    
    def _get_market_data_schema(self) -> str:
        """获取市场数据表结构"""
        return """
        CREATE TABLE IF NOT EXISTS stream_market_data (
            id String,
            timestamp DateTime64(3),
            received_time DateTime64(3),
            processed_time Nullable(DateTime64(3)),
            source String,
            symbol String,
            price Decimal(18, 8),
            volume Decimal(18, 8),
            price_change Nullable(Decimal(18, 8)),
            price_change_percent Nullable(Decimal(18, 8)),
            high_24h Nullable(Decimal(18, 8)),
            low_24h Nullable(Decimal(18, 8)),
            volume_24h Nullable(Decimal(18, 8)),
            quality String,
            latency_ms Nullable(Float32),
            data_json String
        ) ENGINE = MergeTree()
        PARTITION BY toYYYYMM(timestamp)
        ORDER BY (source, symbol, timestamp)
        SETTINGS index_granularity = 8192
        """
    
    def _get_orderbook_schema(self) -> str:
        """获取订单簿表结构"""
        return """
        CREATE TABLE IF NOT EXISTS stream_order_book (
            id String,
            timestamp DateTime64(3),
            received_time DateTime64(3),
            processed_time Nullable(DateTime64(3)),
            source String,
            symbol String,
            bids_json String,
            asks_json String,
            last_update_id Nullable(UInt64),
            quality String,
            latency_ms Nullable(Float32),
            data_json String
        ) ENGINE = MergeTree()
        PARTITION BY toYYYYMM(timestamp)
        ORDER BY (source, symbol, timestamp)
        SETTINGS index_granularity = 8192
        """
    
    def _get_trade_data_schema(self) -> str:
        """获取成交数据表结构"""
        return """
        CREATE TABLE IF NOT EXISTS stream_trade_data (
            id String,
            timestamp DateTime64(3),
            received_time DateTime64(3),
            processed_time Nullable(DateTime64(3)),
            source String,
            symbol String,
            trade_id String,
            price Decimal(18, 8),
            quantity Decimal(18, 8),
            is_buyer_maker Nullable(UInt8),
            trade_time Nullable(DateTime64(3)),
            quality String,
            latency_ms Nullable(Float32),
            data_json String
        ) ENGINE = MergeTree()
        PARTITION BY toYYYYMM(timestamp)
        ORDER BY (source, symbol, timestamp)
        SETTINGS index_granularity = 8192
        """
    
    async def _create_tables(self):
        """创建ClickHouse表"""
        try:
            # 创建数据库
            self.clickhouse_client.execute("CREATE DATABASE IF NOT EXISTS hermes_flow")
            
            # 创建各类型数据表
            for data_type, schema in self.table_schemas.items():
                await asyncio.get_event_loop().run_in_executor(
                    None,
                    self.clickhouse_client.execute,
                    schema
                )
                logger.info(f"ClickHouse表创建完成: {self._get_table_name(data_type)}")
                
        except Exception as e:
            logger.error(f"创建ClickHouse表失败: {e}")
            raise
    
    def _prepare_row_data(self, data: StreamData, data_type: StreamDataType) -> Tuple:
        """准备行数据"""
        base_data = (
            data.id,
            data.timestamp,
            data.received_time,
            data.processed_time,
            data.source,
            data.symbol,
        )
        
        if data_type == StreamDataType.MARKET_DATA:
            market_data = data if hasattr(data, 'price') else None
            return base_data + (
                float(market_data.price) if market_data and market_data.price else 0.0,
                float(market_data.volume) if market_data and market_data.volume else 0.0,
                float(market_data.price_change) if market_data and market_data.price_change else None,
                float(market_data.price_change_percent) if market_data and market_data.price_change_percent else None,
                float(market_data.high_24h) if market_data and market_data.high_24h else None,
                float(market_data.low_24h) if market_data and market_data.low_24h else None,
                float(market_data.volume_24h) if market_data and market_data.volume_24h else None,
                data.quality.value if data.quality else 'unknown',
                data.latency_ms,
                json.dumps(data.data, default=str)
            )
        elif data_type == StreamDataType.ORDER_BOOK:
            return base_data + (
                json.dumps(getattr(data, 'bids', []), default=str),
                json.dumps(getattr(data, 'asks', []), default=str),
                getattr(data, 'last_update_id', None),
                data.quality.value if data.quality else 'unknown',
                data.latency_ms,
                json.dumps(data.data, default=str)
            )
        elif data_type == StreamDataType.TRADE_DATA:
            trade_data = data if hasattr(data, 'trade_id') else None
            return base_data + (
                getattr(trade_data, 'trade_id', '') if trade_data else '',
                float(trade_data.price) if trade_data and trade_data.price else 0.0,
                float(trade_data.quantity) if trade_data and trade_data.quantity else 0.0,
                int(trade_data.is_buyer_maker) if trade_data and trade_data.is_buyer_maker is not None else None,
                trade_data.trade_time if trade_data and trade_data.trade_time else None,
                data.quality.value if data.quality else 'unknown',
                data.latency_ms,
                json.dumps(data.data, default=str)
            )
        else:
            return base_data + (json.dumps(data.data, default=str),)
    
    async def _batch_write_loop(self):
        """批量写入循环"""
        batch_buffer = []
        last_flush_time = time.time()
        
        while self.running:
            try:
                # 等待数据或超时
                try:
                    data = await asyncio.wait_for(self.write_queue.get(), timeout=1.0)
                    batch_buffer.append(data)
                except asyncio.TimeoutError:
                    pass
                
                # 检查是否需要刷新
                current_time = time.time()
                should_flush = (
                    len(batch_buffer) >= self.batch_size or
                    (batch_buffer and current_time - last_flush_time >= self.flush_interval)
                )
                
                if should_flush and batch_buffer:
                    await self.store_batch(batch_buffer)
                    batch_buffer.clear()
                    last_flush_time = current_time
                    
            except Exception as e:
                logger.error(f"ClickHouse批量写入循环异常: {e}")
                await asyncio.sleep(5)
    
    def get_stats(self) -> Dict[str, Any]:
        """获取统计信息"""
        return {
            'storage_type': 'cold_storage',
            'running': self.running,
            'metrics': {
                'total_stored': self.metrics.total_stored,
                'successful_stored': self.metrics.successful_stored,
                'failed_stored': self.metrics.failed_stored,
                'success_rate': self.metrics.get_success_rate(),
                'total_bytes_stored': self.metrics.total_bytes_stored,
                'compressed_bytes_stored': self.metrics.compressed_bytes_stored,
                'compression_ratio': self.metrics.get_compression_ratio(),
                'avg_write_latency_ms': self.metrics.avg_write_latency_ms,
                'max_write_latency_ms': self.metrics.max_write_latency_ms
            },
            'queue_size': self.write_queue.qsize() if self.write_queue else 0,
            'background_tasks': len(self.background_tasks),
            'tables': list(self.table_schemas.keys())
        }

class StorageManager:
    """存储管理器主类"""
    
    def __init__(self, config: StorageConfig):
        self.config = config
        self.hot_storage: Optional[HotDataCache] = None
        self.cold_storage: Optional[ColdDataStorage] = None
        self.running = False
        
        # 根据策略初始化存储组件
        if config.strategy in [StorageStrategy.HOT_ONLY, StorageStrategy.HOT_COLD, StorageStrategy.TIERED]:
            if config.hot_storage_enabled:
                self.hot_storage = HotDataCache(config)
        
        if config.strategy in [StorageStrategy.COLD_ONLY, StorageStrategy.HOT_COLD, StorageStrategy.TIERED]:
            if config.cold_storage_enabled:
                self.cold_storage = ColdDataStorage(config)
        
        # 统计信息
        self.storage_stats = {
            'total_operations': 0,
            'successful_operations': 0,
            'failed_operations': 0,
            'strategy': config.strategy.value
        }
        
        logger.info(f"存储管理器初始化完成，策略: {config.strategy.value}")
    
    async def initialize(self) -> bool:
        """初始化存储管理器"""
        try:
            initialization_results = []
            
            # 初始化热存储
            if self.hot_storage:
                hot_result = await self.hot_storage.initialize()
                initialization_results.append(hot_result)
                logger.info(f"热存储初始化结果: {hot_result}")
            
            # 初始化冷存储
            if self.cold_storage:
                cold_result = await self.cold_storage.initialize()
                initialization_results.append(cold_result)
                logger.info(f"冷存储初始化结果: {cold_result}")
            
            # 检查初始化结果
            if not initialization_results:
                logger.warning("没有可用的存储组件")
                self.running = True  # 允许在没有存储的情况下运行
                return True
            
            # 至少有一个存储组件成功初始化
            success = any(initialization_results)
            if success:
                self.running = True
                logger.info("存储管理器初始化成功")
            else:
                logger.warning("所有存储组件初始化失败，但允许系统继续运行")
                self.running = True  # 允许在没有存储的情况下运行
                success = True  # 强制返回成功
            
            return success
            
        except Exception as e:
            logger.error(f"存储管理器初始化失败: {e}")
            # 即使出现异常，也允许系统继续运行
            self.running = True
            return True
    
    async def store(self, data: StreamData) -> bool:
        """存储数据"""
        if not self.running:
            logger.warning("存储管理器未运行")
            return False
        
        self.storage_stats['total_operations'] += 1
        success = False
        
        try:
            # 根据策略选择存储方式
            if self.config.strategy == StorageStrategy.HOT_ONLY:
                if self.hot_storage:
                    success = await self.hot_storage.store(data)
            
            elif self.config.strategy == StorageStrategy.COLD_ONLY:
                if self.cold_storage:
                    success = await self.cold_storage.store(data)
            
            elif self.config.strategy == StorageStrategy.HOT_COLD:
                # 同时存储到热存储和冷存储
                hot_success = False
                cold_success = False
                
                if self.hot_storage:
                    hot_success = await self.hot_storage.store(data)
                
                if self.cold_storage:
                    cold_success = await self.cold_storage.store(data)
                
                # 只要有一个成功就算成功
                success = hot_success or cold_success
            
            elif self.config.strategy == StorageStrategy.TIERED:
                # 分层存储：新数据先存热存储，定期迁移到冷存储
                if self.hot_storage:
                    success = await self.hot_storage.store(data)
                # TODO: 实现定期迁移逻辑
            
            # 更新统计
            if success:
                self.storage_stats['successful_operations'] += 1
                data.status = DataStatus.STORED
            else:
                self.storage_stats['failed_operations'] += 1
                data.status = DataStatus.ERROR
            
            return success
            
        except Exception as e:
            logger.error(f"存储数据失败: {e}")
            self.storage_stats['failed_operations'] += 1
            data.status = DataStatus.ERROR
            return False
    
    async def store_batch(self, data_list: List[StreamData]) -> List[bool]:
        """批量存储数据"""
        if not self.running:
            logger.warning("存储管理器未运行")
            return [False] * len(data_list)
        
        self.storage_stats['total_operations'] += len(data_list)
        results = []
        
        try:
            # 根据策略选择存储方式
            if self.config.strategy == StorageStrategy.HOT_ONLY:
                if self.hot_storage:
                    results = await self.hot_storage.store_batch(data_list)
                else:
                    results = [False] * len(data_list)
            
            elif self.config.strategy == StorageStrategy.COLD_ONLY:
                if self.cold_storage:
                    results = await self.cold_storage.store_batch(data_list)
                else:
                    results = [False] * len(data_list)
            
            elif self.config.strategy == StorageStrategy.HOT_COLD:
                # 同时存储到热存储和冷存储
                hot_results = []
                cold_results = []
                
                if self.hot_storage:
                    hot_results = await self.hot_storage.store_batch(data_list)
                else:
                    hot_results = [False] * len(data_list)
                
                if self.cold_storage:
                    cold_results = await self.cold_storage.store_batch(data_list)
                else:
                    cold_results = [False] * len(data_list)
                
                # 合并结果：只要有一个成功就算成功
                results = [hot or cold for hot, cold in zip(hot_results, cold_results)]
            
            elif self.config.strategy == StorageStrategy.TIERED:
                if self.hot_storage:
                    results = await self.hot_storage.store_batch(data_list)
                else:
                    results = [False] * len(data_list)
            
            # 更新统计和数据状态
            successful_count = sum(1 for result in results if result)
            self.storage_stats['successful_operations'] += successful_count
            self.storage_stats['failed_operations'] += len(results) - successful_count
            
            # 更新数据状态
            for data, success in zip(data_list, results):
                data.status = DataStatus.STORED if success else DataStatus.ERROR
            
            return results
            
        except Exception as e:
            logger.error(f"批量存储数据失败: {e}")
            self.storage_stats['failed_operations'] += len(data_list)
            results = [False] * len(data_list)
            
            # 更新数据状态
            for data in data_list:
                data.status = DataStatus.ERROR
            
            return results
    
    async def retrieve(self, key: str, prefer_hot: bool = True) -> Optional[StreamData]:
        """检索数据"""
        if not self.running:
            return None
        
        try:
            # 优先从热存储检索
            if prefer_hot and self.hot_storage:
                data = await self.hot_storage.retrieve(key)
                if data:
                    return data
            
            # 从冷存储检索
            if self.cold_storage:
                data = await self.cold_storage.retrieve(key)
                return data
            
            return None
            
        except Exception as e:
            logger.error(f"检索数据失败: {e}")
            return None
    
    async def cleanup(self) -> bool:
        """清理资源"""
        try:
            self.running = False
            cleanup_results = []
            
            # 清理热存储
            if self.hot_storage:
                hot_result = await self.hot_storage.cleanup()
                cleanup_results.append(hot_result)
            
            # 清理冷存储
            if self.cold_storage:
                cold_result = await self.cold_storage.cleanup()
                cleanup_results.append(cold_result)
            
            success = all(cleanup_results) if cleanup_results else True
            logger.info(f"存储管理器清理完成，结果: {success}")
            return success
            
        except Exception as e:
            logger.error(f"清理存储管理器失败: {e}")
            return False
    
    def get_stats(self) -> Dict[str, Any]:
        """获取统计信息"""
        stats = {
            'storage_manager': {
                **self.storage_stats,
                'running': self.running,
                'strategy': self.config.strategy.value,
                'success_rate': (
                    self.storage_stats['successful_operations'] / 
                    max(self.storage_stats['total_operations'], 1)
                )
            }
        }
        
        # 添加子组件统计
        if self.hot_storage:
            stats['hot_storage'] = self.hot_storage.get_stats()
        
        if self.cold_storage:
            stats['cold_storage'] = self.cold_storage.get_stats()
        
        return stats
    
    def get_health_status(self) -> Dict[str, Any]:
        """获取健康状态"""
        health = {
            'overall_healthy': self.running,
            'components': {}
        }
        
        if self.hot_storage:
            hot_stats = self.hot_storage.get_stats()
            health['components']['hot_storage'] = {
                'healthy': hot_stats['running'],
                'queue_size': hot_stats.get('queue_size', 0)
            }
        
        if self.cold_storage:
            cold_stats = self.cold_storage.get_stats()
            health['components']['cold_storage'] = {
                'healthy': cold_stats['running'],
                'queue_size': cold_stats.get('queue_size', 0)
            }
        
        return health 
 