"""
ClickHouse数据库管理器

提供ClickHouse连接管理、时序数据存储和查询功能
支持分区策略、批量插入和查询优化
"""

import asyncio
import logging
from typing import Any, Dict, List, Optional, Union, Tuple
from datetime import datetime, timedelta
import json

try:
    import asynch
    from asynch import connect
    CLICKHOUSE_AVAILABLE = True
except ImportError:
    CLICKHOUSE_AVAILABLE = False
    asynch = None
    connect = None

from .base_database import (
    BaseDatabaseManager, 
    DatabaseConfig, 
    DatabaseConnectionError,
    DatabaseOperationError,
    DatabaseTimeoutError
)

logger = logging.getLogger(__name__)


class ClickHouseManager(BaseDatabaseManager):
    """ClickHouse数据库管理器"""
    
    def __init__(self, config: DatabaseConfig):
        """
        初始化ClickHouse管理器
        
        Args:
            config: 数据库配置对象
        """
        if not CLICKHOUSE_AVAILABLE:
            raise ImportError("asynch包未安装，请运行: pip install asynch")
            
        super().__init__(config)
        self.connection = None
        
    def get_database_type(self) -> str:
        """获取数据库类型"""
        return "clickhouse"
        
    async def connect(self) -> bool:
        """
        建立ClickHouse连接
        
        Returns:
            连接是否成功
        """
        try:
            async with self._connection_lock:
                if self.connection and await self.ping():
                    logger.info("ClickHouse连接已存在且正常")
                    return True
                    
                # 构建连接字符串
                dsn_kwargs = {
                    'host': self.config.host,
                    'port': self.config.port,
                    'database': self.config.database or 'default',
                    'timeout': self.config.timeout,
                    'pool_size': self.config.pool_size,
                    'autocommit': True
                }
                
                # 添加认证信息
                if self.config.username:
                    dsn_kwargs['user'] = self.config.username
                if self.config.password:
                    dsn_kwargs['password'] = self.config.password
                    
                logger.info(f"正在连接到ClickHouse: {self.config.host}:{self.config.port}")
                
                self.connection = await connect(**dsn_kwargs)
                self.client = self.connection  # 为了兼容基类接口
                
                # 测试连接
                await self.ping()
                
                await self._update_health_status(True)
                logger.info("✅ ClickHouse连接成功建立")
                return True
                
        except Exception as e:
            error_msg = f"ClickHouse连接失败: {e}"
            logger.error(error_msg)
            await self._update_health_status(False, e)
            raise DatabaseConnectionError(error_msg) from e
            
    async def disconnect(self) -> None:
        """断开ClickHouse连接"""
        try:
            if self.connection:
                await self.connection.close()
                self.connection = None
                self.client = None
                
            await self._update_health_status(False)
            logger.info("✅ ClickHouse连接已断开")
            
        except Exception as e:
            logger.error(f"断开ClickHouse连接时出错: {e}")
            
    async def check_connection(self) -> bool:
        """
        检查ClickHouse连接状态
        
        Returns:
            连接是否正常
        """
        if not self.connection:
            return False
            
        try:
            return await self.ping()
        except Exception:
            return False
            
    async def ping(self) -> bool:
        """
        Ping ClickHouse服务器
        
        Returns:
            服务器是否响应
        """
        if not self.connection:
            return False
            
        try:
            async def ping_operation():
                cursor = await self.connection.cursor()
                await cursor.execute("SELECT 1")
                result = await cursor.fetchone()
                return result == (1,)
                
            result = await self._measure_latency(ping_operation)
            return result
        except Exception as e:
            logger.warning(f"ClickHouse ping失败: {e}")
            return False
            
    # ==================== 表管理操作 ====================
    
    async def create_kline_table(self, table_name: str = "kline_data") -> bool:
        """
        创建K线数据表
        
        Args:
            table_name: 表名
            
        Returns:
            创建是否成功
        """
        if not await self.check_connection():
            logger.warning("ClickHouse未连接，无法创建表")
            return False
            
        try:
            create_sql = f"""
            CREATE TABLE IF NOT EXISTS {table_name} (
                id UInt64,
                exchange String,
                symbol String,
                interval String,
                open_time DateTime64(3),
                close_time DateTime64(3),
                open_price Decimal(20, 8),
                high_price Decimal(20, 8),
                low_price Decimal(20, 8),
                close_price Decimal(20, 8),
                volume Decimal(20, 8),
                quote_volume Decimal(20, 8),
                trades_count UInt32,
                created_at DateTime64(3) DEFAULT now64()
            ) ENGINE = MergeTree()
            PARTITION BY toYYYYMM(open_time)
            ORDER BY (exchange, symbol, interval, open_time)
            TTL open_time + INTERVAL 2 YEAR
            SETTINGS index_granularity = 8192
            """
            
            cursor = await self.connection.cursor()
            await cursor.execute(create_sql)
            logger.info(f"✅ K线数据表 {table_name} 创建成功")
            return True
            
        except Exception as e:
            logger.error(f"创建K线数据表失败: {e}")
            return False
            
    async def create_ticker_table(self, table_name: str = "ticker_data") -> bool:
        """
        创建行情数据表
        
        Args:
            table_name: 表名
            
        Returns:
            创建是否成功
        """
        if not await self.check_connection():
            logger.warning("ClickHouse未连接，无法创建表")
            return False
            
        try:
            create_sql = f"""
            CREATE TABLE IF NOT EXISTS {table_name} (
                id UInt64,
                exchange String,
                symbol String,
                price Decimal(20, 8),
                bid_price Decimal(20, 8),
                ask_price Decimal(20, 8),
                volume Decimal(20, 8),
                quote_volume Decimal(20, 8),
                high_24h Decimal(20, 8),
                low_24h Decimal(20, 8),
                change_24h Decimal(10, 4),
                timestamp DateTime64(3),
                created_at DateTime64(3) DEFAULT now64()
            ) ENGINE = MergeTree()
            PARTITION BY toYYYYMM(timestamp)
            ORDER BY (exchange, symbol, timestamp)
            TTL timestamp + INTERVAL 6 MONTH
            SETTINGS index_granularity = 8192
            """
            
            cursor = await self.connection.cursor()
            await cursor.execute(create_sql)
            logger.info(f"✅ 行情数据表 {table_name} 创建成功")
            return True
            
        except Exception as e:
            logger.error(f"创建行情数据表失败: {e}")
            return False
            
    async def create_trade_table(self, table_name: str = "trade_data") -> bool:
        """
        创建交易数据表
        
        Args:
            table_name: 表名
            
        Returns:
            创建是否成功
        """
        if not await self.check_connection():
            logger.warning("ClickHouse未连接，无法创建表")
            return False
            
        try:
            create_sql = f"""
            CREATE TABLE IF NOT EXISTS {table_name} (
                id UInt64,
                exchange String,
                symbol String,
                trade_id String,
                price Decimal(20, 8),
                quantity Decimal(20, 8),
                is_buyer_maker UInt8,
                timestamp DateTime64(3),
                created_at DateTime64(3) DEFAULT now64()
            ) ENGINE = MergeTree()
            PARTITION BY toYYYYMM(timestamp)
            ORDER BY (exchange, symbol, timestamp)
            TTL timestamp + INTERVAL 3 MONTH
            SETTINGS index_granularity = 8192
            """
            
            cursor = await self.connection.cursor()
            await cursor.execute(create_sql)
            logger.info(f"✅ 交易数据表 {table_name} 创建成功")
            return True
            
        except Exception as e:
            logger.error(f"创建交易数据表失败: {e}")
            return False
            
    # ==================== 数据插入操作 ====================
    
    async def insert_klines(self, klines: List[Dict[str, Any]], table_name: str = "kline_data") -> bool:
        """
        批量插入K线数据
        
        Args:
            klines: K线数据列表
            table_name: 表名
            
        Returns:
            插入是否成功
        """
        if not klines or not await self.check_connection():
            return False
            
        try:
            cursor = await self.connection.cursor()
            
            insert_sql = f"""
            INSERT INTO {table_name} 
            (id, exchange, symbol, interval, open_time, close_time, 
             open_price, high_price, low_price, close_price, 
             volume, quote_volume, trades_count) 
            VALUES
            """
            
            # 准备数据
            values = []
            for kline in klines:
                values.append((
                    kline.get('id', 0),
                    kline['exchange'],
                    kline['symbol'],
                    kline['interval'],
                    kline['open_time'],
                    kline['close_time'],
                    float(kline['open_price']),
                    float(kline['high_price']),
                    float(kline['low_price']),
                    float(kline['close_price']),
                    float(kline['volume']),
                    float(kline['quote_volume']),
                    kline.get('trades_count', 0)
                ))
                
            await cursor.executemany(insert_sql, values)
            logger.debug(f"成功插入 {len(klines)} 条K线数据到 {table_name}")
            return True
            
        except Exception as e:
            logger.error(f"插入K线数据失败: {e}")
            return False
            
    async def insert_tickers(self, tickers: List[Dict[str, Any]], table_name: str = "ticker_data") -> bool:
        """
        批量插入行情数据
        
        Args:
            tickers: 行情数据列表
            table_name: 表名
            
        Returns:
            插入是否成功
        """
        if not tickers or not await self.check_connection():
            return False
            
        try:
            cursor = await self.connection.cursor()
            
            insert_sql = f"""
            INSERT INTO {table_name} 
            (id, exchange, symbol, price, bid_price, ask_price, 
             volume, quote_volume, high_24h, low_24h, change_24h, timestamp) 
            VALUES
            """
            
            # 准备数据
            values = []
            for ticker in tickers:
                values.append((
                    ticker.get('id', 0),
                    ticker['exchange'],
                    ticker['symbol'],
                    float(ticker['price']),
                    float(ticker.get('bid_price', 0)),
                    float(ticker.get('ask_price', 0)),
                    float(ticker['volume']),
                    float(ticker['quote_volume']),
                    float(ticker['high_24h']),
                    float(ticker['low_24h']),
                    float(ticker['change_24h']),
                    ticker['timestamp']
                ))
                
            await cursor.executemany(insert_sql, values)
            logger.debug(f"成功插入 {len(tickers)} 条行情数据到 {table_name}")
            return True
            
        except Exception as e:
            logger.error(f"插入行情数据失败: {e}")
            return False
            
    # ==================== 数据查询操作 ====================
    
    async def query_klines(self, exchange: str, symbol: str, interval: str, 
                          start_time: datetime, end_time: datetime,
                          table_name: str = "kline_data") -> List[Dict[str, Any]]:
        """
        查询K线数据
        
        Args:
            exchange: 交易所名称
            symbol: 交易对
            interval: 时间间隔
            start_time: 开始时间
            end_time: 结束时间
            table_name: 表名
            
        Returns:
            K线数据列表
        """
        if not await self.check_connection():
            return []
            
        try:
            cursor = await self.connection.cursor()
            
            query_sql = f"""
            SELECT * FROM {table_name}
            WHERE exchange = %s AND symbol = %s AND interval = %s
            AND open_time >= %s AND open_time <= %s
            ORDER BY open_time ASC
            """
            
            await cursor.execute(query_sql, (exchange, symbol, interval, start_time, end_time))
            rows = await cursor.fetchall()
            
            # 转换为字典列表
            columns = [desc[0] for desc in cursor.description]
            result = [dict(zip(columns, row)) for row in rows]
            
            logger.debug(f"查询到 {len(result)} 条K线数据")
            return result
            
        except Exception as e:
            logger.error(f"查询K线数据失败: {e}")
            return []
            
    async def query_latest_ticker(self, exchange: str, symbol: str,
                                 table_name: str = "ticker_data") -> Optional[Dict[str, Any]]:
        """
        查询最新行情数据
        
        Args:
            exchange: 交易所名称
            symbol: 交易对
            table_name: 表名
            
        Returns:
            最新行情数据
        """
        if not await self.check_connection():
            return None
            
        try:
            cursor = await self.connection.cursor()
            
            query_sql = f"""
            SELECT * FROM {table_name}
            WHERE exchange = %s AND symbol = %s
            ORDER BY timestamp DESC
            LIMIT 1
            """
            
            await cursor.execute(query_sql, (exchange, symbol))
            row = await cursor.fetchone()
            
            if row:
                columns = [desc[0] for desc in cursor.description]
                result = dict(zip(columns, row))
                logger.debug(f"查询到最新行情数据: {symbol}")
                return result
            else:
                return None
                
        except Exception as e:
            logger.error(f"查询最新行情数据失败: {e}")
            return None
            
    async def execute_raw_query(self, sql: str, params: Optional[Tuple] = None) -> List[Dict[str, Any]]:
        """
        执行原始SQL查询
        
        Args:
            sql: SQL语句
            params: 查询参数
            
        Returns:
            查询结果
        """
        if not await self.check_connection():
            return []
            
        try:
            cursor = await self.connection.cursor()
            
            if params:
                await cursor.execute(sql, params)
            else:
                await cursor.execute(sql)
                
            rows = await cursor.fetchall()
            
            # 转换为字典列表
            columns = [desc[0] for desc in cursor.description]
            result = [dict(zip(columns, row)) for row in rows]
            
            logger.debug(f"原始查询返回 {len(result)} 条记录")
            return result
            
        except Exception as e:
            logger.error(f"执行原始查询失败: {e}")
            return []
            
    # ==================== 统计查询操作 ====================
    
    async def get_table_count(self, table_name: str) -> int:
        """
        获取表记录数量
        
        Args:
            table_name: 表名
            
        Returns:
            记录数量
        """
        if not await self.check_connection():
            return 0
            
        try:
            cursor = await self.connection.cursor()
            await cursor.execute(f"SELECT COUNT(*) FROM {table_name}")
            result = await cursor.fetchone()
            return result[0] if result else 0
            
        except Exception as e:
            logger.error(f"获取表记录数量失败: {e}")
            return 0
            
    async def get_symbols_list(self, exchange: str, table_name: str = "kline_data") -> List[str]:
        """
        获取交易所的交易对列表
        
        Args:
            exchange: 交易所名称
            table_name: 表名
            
        Returns:
            交易对列表
        """
        if not await self.check_connection():
            return []
            
        try:
            cursor = await self.connection.cursor()
            
            query_sql = f"""
            SELECT DISTINCT symbol FROM {table_name}
            WHERE exchange = %s
            ORDER BY symbol
            """
            
            await cursor.execute(query_sql, (exchange,))
            rows = await cursor.fetchall()
            
            return [row[0] for row in rows]
            
        except Exception as e:
            logger.error(f"获取交易对列表失败: {e}")
            return [] 