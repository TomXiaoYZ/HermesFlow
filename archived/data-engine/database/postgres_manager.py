"""
PostgreSQL数据库管理器

提供PostgreSQL连接管理、配置数据存储和ORM操作
支持事务管理、连接池和数据迁移
"""

import asyncio
import logging
from typing import Any, Dict, List, Optional, Union, Type
from datetime import datetime
import json

try:
    import asyncpg
    from sqlalchemy import Column, Integer, String, DateTime, JSON, Boolean, Text, create_engine
    from sqlalchemy.ext.asyncio import create_async_engine, AsyncSession, async_sessionmaker
    from sqlalchemy.ext.declarative import declarative_base
    from sqlalchemy.orm import sessionmaker
    from sqlalchemy.pool import NullPool
    POSTGRES_AVAILABLE = True
except ImportError:
    POSTGRES_AVAILABLE = False
    asyncpg = None
    AsyncSession = None
    declarative_base = None

from .base_database import (
    BaseDatabaseManager, 
    DatabaseConfig, 
    DatabaseConnectionError,
    DatabaseOperationError,
    DatabaseTimeoutError
)

logger = logging.getLogger(__name__)

# SQLAlchemy基类
Base = declarative_base() if POSTGRES_AVAILABLE else None


class ExchangeConfig(Base):
    """交易所配置表"""
    __tablename__ = 'exchange_config'
    
    id = Column(Integer, primary_key=True)
    exchange_name = Column(String(50), unique=True, nullable=False)
    api_key = Column(Text)  # 加密存储
    secret_key = Column(Text)  # 加密存储
    passphrase = Column(String(100))
    testnet = Column(Boolean, default=False)
    is_active = Column(Boolean, default=True)
    created_at = Column(DateTime, default=datetime.utcnow)
    updated_at = Column(DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)


class StrategyConfig(Base):
    """策略配置表"""
    __tablename__ = 'strategy_config'
    
    id = Column(Integer, primary_key=True)
    strategy_name = Column(String(100), unique=True, nullable=False)
    strategy_type = Column(String(50), nullable=False)
    parameters = Column(JSON)
    risk_limits = Column(JSON)
    is_active = Column(Boolean, default=True)
    created_at = Column(DateTime, default=datetime.utcnow)
    updated_at = Column(DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)


class SystemConfig(Base):
    """系统配置表"""
    __tablename__ = 'system_config'
    
    id = Column(Integer, primary_key=True)
    config_key = Column(String(100), unique=True, nullable=False)
    config_value = Column(JSON)
    description = Column(Text)
    created_at = Column(DateTime, default=datetime.utcnow)
    updated_at = Column(DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)


class PostgresManager(BaseDatabaseManager):
    """PostgreSQL数据库管理器"""
    
    def __init__(self, config: DatabaseConfig):
        """
        初始化PostgreSQL管理器
        
        Args:
            config: 数据库配置对象
        """
        if not POSTGRES_AVAILABLE:
            raise ImportError("asyncpg和sqlalchemy包未安装，请运行: pip install asyncpg sqlalchemy")
            
        super().__init__(config)
        self.engine = None
        self.session_factory = None
        
    def get_database_type(self) -> str:
        """获取数据库类型"""
        return "postgres"
        
    def _get_connection_url(self) -> str:
        """构建数据库连接URL"""
        if self.config.username and self.config.password:
            auth = f"{self.config.username}:{self.config.password}@"
        else:
            auth = ""
            
        return (f"postgresql+asyncpg://{auth}{self.config.host}:{self.config.port}/"
                f"{self.config.database}")
        
    async def connect(self) -> bool:
        """
        建立PostgreSQL连接
        
        Returns:
            连接是否成功
        """
        try:
            async with self._connection_lock:
                if self.engine and await self.ping():
                    logger.info("PostgreSQL连接已存在且正常")
                    return True
                    
                connection_url = self._get_connection_url()
                
                # 创建异步引擎
                self.engine = create_async_engine(
                    connection_url,
                    poolclass=NullPool,  # 使用NullPool避免连接池问题
                    echo=False,  # 设置为True可以看到SQL语句
                    future=True
                )
                
                # 创建会话工厂
                self.session_factory = async_sessionmaker(
                    self.engine,
                    class_=AsyncSession,
                    expire_on_commit=False
                )
                
                self.client = self.engine  # 为了兼容基类接口
                
                logger.info(f"正在连接到PostgreSQL: {self.config.host}:{self.config.port}")
                
                # 测试连接
                await self.ping()
                
                await self._update_health_status(True)
                logger.info("✅ PostgreSQL连接成功建立")
                return True
                
        except Exception as e:
            error_msg = f"PostgreSQL连接失败: {e}"
            logger.error(error_msg)
            await self._update_health_status(False, e)
            raise DatabaseConnectionError(error_msg) from e
            
    async def disconnect(self) -> None:
        """断开PostgreSQL连接"""
        try:
            if self.engine:
                await self.engine.dispose()
                self.engine = None
                self.client = None
                
            self.session_factory = None
            
            await self._update_health_status(False)
            logger.info("✅ PostgreSQL连接已断开")
            
        except Exception as e:
            logger.error(f"断开PostgreSQL连接时出错: {e}")
            
    async def check_connection(self) -> bool:
        """
        检查PostgreSQL连接状态
        
        Returns:
            连接是否正常
        """
        if not self.engine:
            return False
            
        try:
            return await self.ping()
        except Exception:
            return False
            
    async def ping(self) -> bool:
        """
        Ping PostgreSQL服务器
        
        Returns:
            服务器是否响应
        """
        if not self.engine:
            return False
            
        try:
            async def ping_operation():
                async with self.engine.connect() as conn:
                    result = await conn.execute("SELECT 1")
                    return True
                    
            result = await self._measure_latency(ping_operation)
            return result
        except Exception as e:
            logger.warning(f"PostgreSQL ping失败: {e}")
            return False
            
    # ==================== 表管理操作 ====================
    
    async def create_tables(self) -> bool:
        """
        创建所有配置表
        
        Returns:
            创建是否成功
        """
        if not await self.check_connection():
            logger.warning("PostgreSQL未连接，无法创建表")
            return False
            
        try:
            async with self.engine.begin() as conn:
                await conn.run_sync(Base.metadata.create_all)
            
            logger.info("✅ PostgreSQL配置表创建成功")
            return True
            
        except Exception as e:
            logger.error(f"创建PostgreSQL表失败: {e}")
            return False
            
    async def drop_tables(self) -> bool:
        """
        删除所有配置表
        
        Returns:
            删除是否成功
        """
        if not await self.check_connection():
            logger.warning("PostgreSQL未连接，无法删除表")
            return False
            
        try:
            async with self.engine.begin() as conn:
                await conn.run_sync(Base.metadata.drop_all)
            
            logger.info("✅ PostgreSQL配置表删除成功")
            return True
            
        except Exception as e:
            logger.error(f"删除PostgreSQL表失败: {e}")
            return False
            
    # ==================== 交易所配置操作 ====================
    
    async def save_exchange_config(self, exchange_name: str, api_key: str, 
                                  secret_key: str, passphrase: Optional[str] = None,
                                  testnet: bool = False) -> bool:
        """
        保存交易所配置
        
        Args:
            exchange_name: 交易所名称
            api_key: API密钥
            secret_key: 密钥
            passphrase: 口令（OKX需要）
            testnet: 是否为测试网
            
        Returns:
            保存是否成功
        """
        if not await self.check_connection():
            return False
            
        try:
            async with self.session_factory() as session:
                # 查找现有配置
                existing = await session.get(ExchangeConfig, exchange_name)
                
                if existing:
                    # 更新现有配置
                    existing.api_key = api_key  # 实际应用中需要加密
                    existing.secret_key = secret_key  # 实际应用中需要加密
                    existing.passphrase = passphrase
                    existing.testnet = testnet
                    existing.updated_at = datetime.utcnow()
                else:
                    # 创建新配置
                    new_config = ExchangeConfig(
                        exchange_name=exchange_name,
                        api_key=api_key,  # 实际应用中需要加密
                        secret_key=secret_key,  # 实际应用中需要加密
                        passphrase=passphrase,
                        testnet=testnet
                    )
                    session.add(new_config)
                
                await session.commit()
                logger.info(f"✅ 交易所配置保存成功: {exchange_name}")
                return True
                
        except Exception as e:
            logger.error(f"保存交易所配置失败: {e}")
            return False
            
    async def get_exchange_config(self, exchange_name: str) -> Optional[Dict[str, Any]]:
        """
        获取交易所配置
        
        Args:
            exchange_name: 交易所名称
            
        Returns:
            交易所配置字典
        """
        if not await self.check_connection():
            return None
            
        try:
            async with self.session_factory() as session:
                config = await session.get(ExchangeConfig, exchange_name)
                
                if config:
                    return {
                        'exchange_name': config.exchange_name,
                        'api_key': config.api_key,  # 实际应用中需要解密
                        'secret_key': config.secret_key,  # 实际应用中需要解密
                        'passphrase': config.passphrase,
                        'testnet': config.testnet,
                        'is_active': config.is_active,
                        'created_at': config.created_at,
                        'updated_at': config.updated_at
                    }
                else:
                    return None
                    
        except Exception as e:
            logger.error(f"获取交易所配置失败: {e}")
            return None
            
    async def list_exchange_configs(self) -> List[Dict[str, Any]]:
        """
        获取所有交易所配置
        
        Returns:
            交易所配置列表
        """
        if not await self.check_connection():
            return []
            
        try:
            async with self.session_factory() as session:
                result = await session.execute("SELECT * FROM exchange_config WHERE is_active = true")
                configs = result.fetchall()
                
                return [
                    {
                        'exchange_name': config.exchange_name,
                        'testnet': config.testnet,
                        'is_active': config.is_active,
                        'created_at': config.created_at,
                        'updated_at': config.updated_at
                    }
                    for config in configs
                ]
                
        except Exception as e:
            logger.error(f"获取交易所配置列表失败: {e}")
            return []
            
    # ==================== 策略配置操作 ====================
    
    async def save_strategy_config(self, strategy_name: str, strategy_type: str,
                                  parameters: Dict[str, Any], risk_limits: Dict[str, Any]) -> bool:
        """
        保存策略配置
        
        Args:
            strategy_name: 策略名称
            strategy_type: 策略类型
            parameters: 策略参数
            risk_limits: 风险限制
            
        Returns:
            保存是否成功
        """
        if not await self.check_connection():
            return False
            
        try:
            async with self.session_factory() as session:
                # 查找现有配置
                result = await session.execute(
                    "SELECT * FROM strategy_config WHERE strategy_name = :name",
                    {'name': strategy_name}
                )
                existing = result.fetchone()
                
                if existing:
                    # 更新现有配置
                    await session.execute(
                        """UPDATE strategy_config 
                           SET strategy_type = :type, parameters = :params, 
                               risk_limits = :limits, updated_at = :updated 
                           WHERE strategy_name = :name""",
                        {
                            'name': strategy_name,
                            'type': strategy_type,
                            'params': json.dumps(parameters),
                            'limits': json.dumps(risk_limits),
                            'updated': datetime.utcnow()
                        }
                    )
                else:
                    # 创建新配置
                    await session.execute(
                        """INSERT INTO strategy_config 
                           (strategy_name, strategy_type, parameters, risk_limits) 
                           VALUES (:name, :type, :params, :limits)""",
                        {
                            'name': strategy_name,
                            'type': strategy_type,
                            'params': json.dumps(parameters),
                            'limits': json.dumps(risk_limits)
                        }
                    )
                
                await session.commit()
                logger.info(f"✅ 策略配置保存成功: {strategy_name}")
                return True
                
        except Exception as e:
            logger.error(f"保存策略配置失败: {e}")
            return False
            
    async def get_strategy_config(self, strategy_name: str) -> Optional[Dict[str, Any]]:
        """
        获取策略配置
        
        Args:
            strategy_name: 策略名称
            
        Returns:
            策略配置字典
        """
        if not await self.check_connection():
            return None
            
        try:
            async with self.session_factory() as session:
                result = await session.execute(
                    "SELECT * FROM strategy_config WHERE strategy_name = :name",
                    {'name': strategy_name}
                )
                config = result.fetchone()
                
                if config:
                    return {
                        'strategy_name': config.strategy_name,
                        'strategy_type': config.strategy_type,
                        'parameters': json.loads(config.parameters) if config.parameters else {},
                        'risk_limits': json.loads(config.risk_limits) if config.risk_limits else {},
                        'is_active': config.is_active,
                        'created_at': config.created_at,
                        'updated_at': config.updated_at
                    }
                else:
                    return None
                    
        except Exception as e:
            logger.error(f"获取策略配置失败: {e}")
            return None
            
    # ==================== 系统配置操作 ====================
    
    async def set_system_config(self, config_key: str, config_value: Any, 
                               description: Optional[str] = None) -> bool:
        """
        设置系统配置
        
        Args:
            config_key: 配置键
            config_value: 配置值
            description: 配置描述
            
        Returns:
            设置是否成功
        """
        if not await self.check_connection():
            return False
            
        try:
            async with self.session_factory() as session:
                # 查找现有配置
                result = await session.execute(
                    "SELECT * FROM system_config WHERE config_key = :key",
                    {'key': config_key}
                )
                existing = result.fetchone()
                
                if existing:
                    # 更新现有配置
                    await session.execute(
                        """UPDATE system_config 
                           SET config_value = :value, description = :desc, updated_at = :updated 
                           WHERE config_key = :key""",
                        {
                            'key': config_key,
                            'value': json.dumps(config_value),
                            'desc': description,
                            'updated': datetime.utcnow()
                        }
                    )
                else:
                    # 创建新配置
                    await session.execute(
                        """INSERT INTO system_config 
                           (config_key, config_value, description) 
                           VALUES (:key, :value, :desc)""",
                        {
                            'key': config_key,
                            'value': json.dumps(config_value),
                            'desc': description
                        }
                    )
                
                await session.commit()
                logger.info(f"✅ 系统配置设置成功: {config_key}")
                return True
                
        except Exception as e:
            logger.error(f"设置系统配置失败: {e}")
            return False
            
    async def get_system_config(self, config_key: str) -> Optional[Any]:
        """
        获取系统配置
        
        Args:
            config_key: 配置键
            
        Returns:
            配置值
        """
        if not await self.check_connection():
            return None
            
        try:
            async with self.session_factory() as session:
                result = await session.execute(
                    "SELECT config_value FROM system_config WHERE config_key = :key",
                    {'key': config_key}
                )
                config = result.fetchone()
                
                if config and config.config_value:
                    return json.loads(config.config_value)
                else:
                    return None
                    
        except Exception as e:
            logger.error(f"获取系统配置失败: {e}")
            return None 