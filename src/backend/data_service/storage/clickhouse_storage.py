"""
ClickHouse存储模块

该模块负责处理所有历史数据的存储和检索，包括：
1. 历史行情数据
2. 历史交易记录
3. 历史订单数据
4. 系统日志数据
"""

import logging
from typing import Dict, List, Optional, Union
from datetime import datetime, timedelta
from clickhouse_driver import Client
from ..common.singleton import Singleton

logger = logging.getLogger(__name__)

class ClickHouseStorage(metaclass=Singleton):
    """ClickHouse存储类，使用单例模式确保只有一个连接实例"""
    
    def __init__(self, host: str = 'localhost', port: int = 9000,
                 database: str = 'hermesflow', user: str = 'default',
                 password: Optional[str] = None):
        """初始化ClickHouse连接
        
        Args:
            host: ClickHouse服务器地址
            port: ClickHouse服务器端口
            database: 数据库名称
            user: 用户名
            password: 密码
        """
        self.client = Client(
            host=host,
            port=port,
            database=database,
            user=user,
            password=password
        )
        self._init_tables()
        logger.info(f"ClickHouse连接初始化完成: {host}:{port}")
    
    def _init_tables(self):
        """初始化数据表"""
        # 创建行情数据表
        self.client.execute('''
            CREATE TABLE IF NOT EXISTS market_data (
                exchange String,
                symbol String,
                timestamp DateTime,
                price Float64,
                volume Float64,
                amount Float64,
                trades UInt32,
                bid Float64,
                ask Float64,
                version UInt32
            ) ENGINE = MergeTree()
            PARTITION BY toYYYYMM(timestamp)
            ORDER BY (exchange, symbol, timestamp)
        ''')
        
        # 创建交易记录表
        self.client.execute('''
            CREATE TABLE IF NOT EXISTS trade_history (
                exchange String,
                symbol String,
                trade_id String,
                timestamp DateTime,
                price Float64,
                amount Float64,
                direction String,
                maker_order_id String,
                taker_order_id String,
                version UInt32
            ) ENGINE = MergeTree()
            PARTITION BY toYYYYMM(timestamp)
            ORDER BY (exchange, symbol, timestamp)
        ''')
        
        # 创建订单历史表
        self.client.execute('''
            CREATE TABLE IF NOT EXISTS order_history (
                exchange String,
                user_id String,
                order_id String,
                symbol String,
                timestamp DateTime,
                type String,
                side String,
                price Float64,
                amount Float64,
                filled Float64,
                status String,
                version UInt32
            ) ENGINE = MergeTree()
            PARTITION BY toYYYYMM(timestamp)
            ORDER BY (exchange, user_id, timestamp)
        ''')
        
        # 创建系统日志表
        self.client.execute('''
            CREATE TABLE IF NOT EXISTS system_logs (
                timestamp DateTime,
                component String,
                level String,
                message String,
                details String,
                version UInt32
            ) ENGINE = MergeTree()
            PARTITION BY toYYYYMM(timestamp)
            ORDER BY (timestamp, component)
        ''')
    
    def insert_market_data(self, data: Dict) -> bool:
        """插入市场数据
        
        Args:
            data: 市场数据，包含exchange, symbol, timestamp等字段
            
        Returns:
            bool: 是否成功
        """
        try:
            self.client.execute(
                'INSERT INTO market_data VALUES',
                [{
                    'exchange': data['exchange'],
                    'symbol': data['symbol'],
                    'timestamp': data['timestamp'],
                    'price': data['price'],
                    'volume': data['volume'],
                    'amount': data['amount'],
                    'trades': data['trades'],
                    'bid': data['bid'],
                    'ask': data['ask'],
                    'version': data.get('version', 1)
                }]
            )
            return True
        except Exception as e:
            logger.error(f"插入市场数据失败: {e}")
            return False
    
    def query_market_data(self, exchange: str, symbol: str,
                         start_time: datetime,
                         end_time: datetime) -> List[Dict]:
        """查询市场数据
        
        Args:
            exchange: 交易所名称
            symbol: 交易对
            start_time: 开始时间
            end_time: 结束时间
            
        Returns:
            List[Dict]: 市场数据列表
        """
        try:
            query = '''
                SELECT *
                FROM market_data
                WHERE exchange = %(exchange)s
                AND symbol = %(symbol)s
                AND timestamp BETWEEN %(start_time)s AND %(end_time)s
                ORDER BY timestamp
            '''
            result = self.client.execute(
                query,
                {
                    'exchange': exchange,
                    'symbol': symbol,
                    'start_time': start_time,
                    'end_time': end_time
                }
            )
            return [dict(zip(
                ['exchange', 'symbol', 'timestamp', 'price', 'volume',
                 'amount', 'trades', 'bid', 'ask', 'version'],
                row
            )) for row in result]
        except Exception as e:
            logger.error(f"查询市场数据失败: {e}")
            return []
    
    def insert_trade(self, trade: Dict) -> bool:
        """插入交易记录
        
        Args:
            trade: 交易记录，包含exchange, symbol, trade_id等字段
            
        Returns:
            bool: 是否成功
        """
        try:
            self.client.execute(
                'INSERT INTO trade_history VALUES',
                [{
                    'exchange': trade['exchange'],
                    'symbol': trade['symbol'],
                    'trade_id': trade['trade_id'],
                    'timestamp': trade['timestamp'],
                    'price': trade['price'],
                    'amount': trade['amount'],
                    'direction': trade['direction'],
                    'maker_order_id': trade['maker_order_id'],
                    'taker_order_id': trade['taker_order_id'],
                    'version': trade.get('version', 1)
                }]
            )
            return True
        except Exception as e:
            logger.error(f"插入交易记录失败: {e}")
            return False
    
    def query_trades(self, exchange: str, symbol: str,
                    start_time: datetime,
                    end_time: datetime) -> List[Dict]:
        """查询交易记录
        
        Args:
            exchange: 交易所名称
            symbol: 交易对
            start_time: 开始时间
            end_time: 结束时间
            
        Returns:
            List[Dict]: 交易记录列表
        """
        try:
            query = '''
                SELECT *
                FROM trade_history
                WHERE exchange = %(exchange)s
                AND symbol = %(symbol)s
                AND timestamp BETWEEN %(start_time)s AND %(end_time)s
                ORDER BY timestamp
            '''
            result = self.client.execute(
                query,
                {
                    'exchange': exchange,
                    'symbol': symbol,
                    'start_time': start_time,
                    'end_time': end_time
                }
            )
            return [dict(zip(
                ['exchange', 'symbol', 'trade_id', 'timestamp', 'price',
                 'amount', 'direction', 'maker_order_id', 'taker_order_id',
                 'version'],
                row
            )) for row in result]
        except Exception as e:
            logger.error(f"查询交易记录失败: {e}")
            return []
    
    def insert_order(self, order: Dict) -> bool:
        """插入订单记录
        
        Args:
            order: 订单记录，包含exchange, user_id, order_id等字段
            
        Returns:
            bool: 是否成功
        """
        try:
            self.client.execute(
                'INSERT INTO order_history VALUES',
                [{
                    'exchange': order['exchange'],
                    'user_id': order['user_id'],
                    'order_id': order['order_id'],
                    'symbol': order['symbol'],
                    'timestamp': order['timestamp'],
                    'type': order['type'],
                    'side': order['side'],
                    'price': order['price'],
                    'amount': order['amount'],
                    'filled': order['filled'],
                    'status': order['status'],
                    'version': order.get('version', 1)
                }]
            )
            return True
        except Exception as e:
            logger.error(f"插入订单记录失败: {e}")
            return False
    
    def query_orders(self, exchange: str, user_id: str,
                    start_time: datetime,
                    end_time: datetime) -> List[Dict]:
        """查询订单记录
        
        Args:
            exchange: 交易所名称
            user_id: 用户ID
            start_time: 开始时间
            end_time: 结束时间
            
        Returns:
            List[Dict]: 订单记录列表
        """
        try:
            query = '''
                SELECT *
                FROM order_history
                WHERE exchange = %(exchange)s
                AND user_id = %(user_id)s
                AND timestamp BETWEEN %(start_time)s AND %(end_time)s
                ORDER BY timestamp
            '''
            result = self.client.execute(
                query,
                {
                    'exchange': exchange,
                    'user_id': user_id,
                    'start_time': start_time,
                    'end_time': end_time
                }
            )
            return [dict(zip(
                ['exchange', 'user_id', 'order_id', 'symbol', 'timestamp',
                 'type', 'side', 'price', 'amount', 'filled', 'status',
                 'version'],
                row
            )) for row in result]
        except Exception as e:
            logger.error(f"查询订单记录失败: {e}")
            return []
    
    def insert_system_log(self, log: Dict) -> bool:
        """插入系统日志
        
        Args:
            log: 日志记录，包含timestamp, component, level等字段
            
        Returns:
            bool: 是否成功
        """
        try:
            self.client.execute(
                'INSERT INTO system_logs VALUES',
                [{
                    'timestamp': log['timestamp'],
                    'component': log['component'],
                    'level': log['level'],
                    'message': log['message'],
                    'details': log.get('details', ''),
                    'version': log.get('version', 1)
                }]
            )
            return True
        except Exception as e:
            logger.error(f"插入系统日志失败: {e}")
            return False
    
    def query_system_logs(self, component: Optional[str],
                         start_time: datetime,
                         end_time: datetime,
                         level: Optional[str] = None) -> List[Dict]:
        """查询系统日志
        
        Args:
            component: 组件名称，可选
            start_time: 开始时间
            end_time: 结束时间
            level: 日志级别，可选
            
        Returns:
            List[Dict]: 日志记录列表
        """
        try:
            conditions = [
                'timestamp BETWEEN %(start_time)s AND %(end_time)s'
            ]
            params = {
                'start_time': start_time,
                'end_time': end_time
            }
            
            if component:
                conditions.append('component = %(component)s')
                params['component'] = component
            
            if level:
                conditions.append('level = %(level)s')
                params['level'] = level
            
            query = f'''
                SELECT *
                FROM system_logs
                WHERE {' AND '.join(conditions)}
                ORDER BY timestamp
            '''
            
            result = self.client.execute(query, params)
            return [dict(zip(
                ['timestamp', 'component', 'level', 'message', 'details',
                 'version'],
                row
            )) for row in result]
        except Exception as e:
            logger.error(f"查询系统日志失败: {e}")
            return [] 