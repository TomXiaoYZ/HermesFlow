"""
Redis存储模块

该模块负责处理所有实时数据的存储和检索，包括：
1. 市场数据（行情、订单簿、成交记录）
2. 用户数据（订单状态、账户余额）
3. 系统状态（连接状态、心跳信息）
"""

import json
import logging
from typing import Any, Dict, List, Optional, Union
from redis import Redis, ConnectionPool
from ..common.singleton import Singleton

logger = logging.getLogger(__name__)

class RedisStorage(metaclass=Singleton):
    """Redis存储类，使用单例模式确保只有一个Redis连接池"""
    
    def __init__(self, host: str = 'localhost', port: int = 6379, 
                 db: int = 0, password: Optional[str] = None):
        """初始化Redis连接池
        
        Args:
            host: Redis服务器地址
            port: Redis服务器端口
            db: 数据库编号
            password: Redis密码
        """
        self.pool = ConnectionPool(
            host=host,
            port=port,
            db=db,
            password=password,
            decode_responses=True
        )
        self.redis = Redis(connection_pool=self.pool)
        logger.info(f"Redis连接池初始化完成: {host}:{port}")
    
    def set_market_data(self, exchange: str, symbol: str, 
                       data_type: str, data: Union[Dict, List]) -> bool:
        """存储市场数据
        
        Args:
            exchange: 交易所名称
            symbol: 交易对
            data_type: 数据类型（ticker/depth/trades/klines）
            data: 要存储的数据
            
        Returns:
            bool: 是否成功
        """
        try:
            key = f"market:{exchange}:{symbol}:{data_type}"
            self.redis.set(key, json.dumps(data))
            return True
        except Exception as e:
            logger.error(f"存储市场数据失败: {e}")
            return False
    
    def get_market_data(self, exchange: str, symbol: str, 
                       data_type: str) -> Optional[Dict]:
        """获取市场数据
        
        Args:
            exchange: 交易所名称
            symbol: 交易对
            data_type: 数据类型（ticker/depth/trades/klines）
            
        Returns:
            Optional[Dict]: 市场数据，如果不存在则返回None
        """
        try:
            key = f"market:{exchange}:{symbol}:{data_type}"
            data = self.redis.get(key)
            return json.loads(data) if data else None
        except Exception as e:
            logger.error(f"获取市场数据失败: {e}")
            return None
    
    def set_order_status(self, exchange: str, user_id: str, 
                        order_id: str, status: Dict) -> bool:
        """存储订单状态
        
        Args:
            exchange: 交易所名称
            user_id: 用户ID
            order_id: 订单ID
            status: 订单状态
            
        Returns:
            bool: 是否成功
        """
        try:
            key = f"order:{exchange}:{user_id}:{order_id}"
            self.redis.set(key, json.dumps(status))
            return True
        except Exception as e:
            logger.error(f"存储订单状态失败: {e}")
            return False
    
    def get_order_status(self, exchange: str, user_id: str, 
                        order_id: str) -> Optional[Dict]:
        """获取订单状态
        
        Args:
            exchange: 交易所名称
            user_id: 用户ID
            order_id: 订单ID
            
        Returns:
            Optional[Dict]: 订单状态，如果不存在则返回None
        """
        try:
            key = f"order:{exchange}:{user_id}:{order_id}"
            status = self.redis.get(key)
            return json.loads(status) if status else None
        except Exception as e:
            logger.error(f"获取订单状态失败: {e}")
            return None
    
    def set_account_balance(self, exchange: str, user_id: str, 
                          balance: Dict) -> bool:
        """存储账户余额
        
        Args:
            exchange: 交易所名称
            user_id: 用户ID
            balance: 账户余额
            
        Returns:
            bool: 是否成功
        """
        try:
            key = f"balance:{exchange}:{user_id}"
            self.redis.set(key, json.dumps(balance))
            return True
        except Exception as e:
            logger.error(f"存储账户余额失败: {e}")
            return False
    
    def get_account_balance(self, exchange: str, user_id: str) -> Optional[Dict]:
        """获取账户余额
        
        Args:
            exchange: 交易所名称
            user_id: 用户ID
            
        Returns:
            Optional[Dict]: 账户余额，如果不存在则返回None
        """
        try:
            key = f"balance:{exchange}:{user_id}"
            balance = self.redis.get(key)
            return json.loads(balance) if balance else None
        except Exception as e:
            logger.error(f"获取账户余额失败: {e}")
            return None
    
    def set_system_status(self, component: str, status: Dict) -> bool:
        """存储系统状态
        
        Args:
            component: 组件名称
            status: 状态信息
            
        Returns:
            bool: 是否成功
        """
        try:
            key = f"system:{component}"
            self.redis.set(key, json.dumps(status))
            return True
        except Exception as e:
            logger.error(f"存储系统状态失败: {e}")
            return False
    
    def get_system_status(self, component: str) -> Optional[Dict]:
        """获取系统状态
        
        Args:
            component: 组件名称
            
        Returns:
            Optional[Dict]: 系统状态，如果不存在则返回None
        """
        try:
            key = f"system:{component}"
            status = self.redis.get(key)
            return json.loads(status) if status else None
        except Exception as e:
            logger.error(f"获取系统状态失败: {e}")
            return None 