"""
配置管理器

统一管理HermesFlow系统的所有配置：
- 环境分离（开发/测试/生产）
- 配置文件加载和验证
- 环境变量覆盖
- 配置热重载

作者: HermesFlow Team
创建时间: 2024年12月26日
"""

import os
import yaml
import logging
from typing import Dict, Any, Optional, List
from pathlib import Path
from dataclasses import dataclass, field
from enum import Enum
import threading
import time
from watchdog.observers import Observer
from watchdog.events import FileSystemEventHandler

from .secret_manager import SecretManager

logger = logging.getLogger(__name__)

class Environment(Enum):
    """环境枚举"""
    LOCAL = "local"
    PRODUCTION = "production"

@dataclass
class DatabaseConfig:
    """数据库配置"""
    host: str = "localhost"
    port: int = 5432
    database: str = "hermesflow"
    username: str = "hermesflow"
    password: str = ""
    ssl_enabled: bool = False
    connection_pool_size: int = 10
    connection_timeout: int = 30

@dataclass
class RedisConfig:
    """Redis配置"""
    host: str = "localhost"
    port: int = 6379
    database: int = 0
    password: str = ""
    connection_pool_size: int = 10
    connection_timeout: int = 5

@dataclass
class ClickHouseConfig:
    """ClickHouse配置"""
    host: str = "localhost"
    port: int = 9000
    database: str = "hermesflow"
    username: str = "default"
    password: str = ""
    secure: bool = False

@dataclass
class ConnectorConfig:
    """连接器配置"""
    name: str
    enabled: bool = True
    api_key: str = ""
    api_secret: str = ""
    base_url: str = ""
    rate_limit: int = 1000
    timeout: int = 30
    retry_attempts: int = 3
    custom_settings: Dict[str, Any] = field(default_factory=dict)

class ConfigFileHandler(FileSystemEventHandler):
    """配置文件变更监听器"""
    
    def __init__(self, config_manager):
        self.config_manager = config_manager
        super().__init__()
    
    def on_modified(self, event):
        if not event.is_directory and event.src_path.endswith(('.yml', '.yaml')):
            logger.info(f"检测到配置文件变更: {event.src_path}")
            self.config_manager.reload_config()

class ConfigManager:
    """配置管理器"""
    
    def __init__(self, environment: Optional[str] = None, config_dir: str = "config"):
        """
        初始化配置管理器
        
        Args:
            environment: 环境名称，如果未指定则从环境变量HERMESFLOW_ENV获取
            config_dir: 配置文件目录
        """
        # 确定运行环境
        self.environment = Environment(environment or os.getenv('HERMESFLOW_ENV', 'local'))
        self.config_dir = Path(config_dir)
        
        # 配置缓存
        self._config_cache: Dict[str, Any] = {}
        self._config_lock = threading.RLock()
        
        # 密钥管理器
        self.secret_manager = SecretManager()
        
        # 文件监听器
        self._observer = None
        
        # 加载配置
        self.load_config()
        
        # 启动文件监听（仅在本地环境）
        if self.environment == Environment.LOCAL:
            self._start_file_watcher()
        
        logger.info(f"配置管理器初始化完成，环境: {self.environment.value}")
    
    def load_config(self):
        """加载配置"""
        logger.info("开始加载配置...")
        
        # 加载基础配置
        base_config = self._load_yaml_file('settings.yaml')
        
        # 加载环境特定配置
        env_config = self._load_yaml_file(f'{self.environment.value}.yml')
        
        # 合并配置
        self.config = self._merge_configs(base_config, env_config)
        
        # 替换环境变量
        self._substitute_env_vars(self.config)
        
        # 应用环境变量覆盖
        self._apply_env_overrides()
        
        # 解密敏感配置
        self._decrypt_secrets()
        
        logger.info("配置加载完成")
    
    def reload_config(self):
        """重新加载配置"""
        logger.info("重新加载配置...")
        self.load_config()
    
    def get(self, key: str, default: Any = None) -> Any:
        """
        获取配置值
        
        Args:
            key: 配置键，支持点号分隔的嵌套键
            default: 默认值
            
        Returns:
            配置值
        """
        with self._config_lock:
            try:
                keys = key.split('.')
                value = self.config
                
                for k in keys:
                    value = value[k]
                
                return value
            except (KeyError, TypeError):
                return default
    
    def set(self, key: str, value: Any):
        """
        设置配置值（仅在内存中）
        
        Args:
            key: 配置键
            value: 配置值
        """
        with self._config_lock:
            keys = key.split('.')
            config = self.config
            
            # 导航到目标位置
            for k in keys[:-1]:
                if k not in config:
                    config[k] = {}
                config = config[k]
            
            config[keys[-1]] = value
            logger.debug(f"设置配置: {key} = {value}")
    
    def get_database_config(self) -> DatabaseConfig:
        """获取数据库配置"""
        db_config = self.get('database', {})
        return DatabaseConfig(
            host=db_config.get('host', 'localhost'),
            port=db_config.get('port', 5432),
            database=db_config.get('database', 'hermesflow'),
            username=db_config.get('username', 'hermesflow'),
            password=db_config.get('password', ''),
            ssl_enabled=db_config.get('ssl_enabled', False),
            connection_pool_size=db_config.get('connection_pool_size', 10),
            connection_timeout=db_config.get('connection_timeout', 30)
        )
    
    def get_redis_config(self) -> RedisConfig:
        """获取Redis配置"""
        redis_config = self.get('redis', {})
        return RedisConfig(
            host=redis_config.get('host', 'localhost'),
            port=redis_config.get('port', 6379),
            database=redis_config.get('database', 0),
            password=redis_config.get('password', ''),
            connection_pool_size=redis_config.get('connection_pool_size', 10),
            connection_timeout=redis_config.get('connection_timeout', 5)
        )
    
    def get_clickhouse_config(self) -> ClickHouseConfig:
        """获取ClickHouse配置"""
        ch_config = self.get('clickhouse', {})
        return ClickHouseConfig(
            host=ch_config.get('host', 'localhost'),
            port=ch_config.get('port', 9000),
            database=ch_config.get('database', 'hermesflow'),
            username=ch_config.get('username', 'default'),
            password=ch_config.get('password', ''),
            secure=ch_config.get('secure', False)
        )
    
    def get_connector_config(self, connector_name: str) -> Optional[ConnectorConfig]:
        """获取连接器配置"""
        connectors = self.get('connectors', {})
        
        if connector_name not in connectors:
            logger.warning(f"连接器配置不存在: {connector_name}")
            return None
        
        config = connectors[connector_name]
        return ConnectorConfig(
            name=connector_name,
            enabled=config.get('enabled', True),
            api_key=config.get('api_key', ''),
            api_secret=config.get('api_secret', ''),
            base_url=config.get('base_url', ''),
            rate_limit=config.get('rate_limit', 1000),
            timeout=config.get('timeout', 30),
            retry_attempts=config.get('retry_attempts', 3),
            custom_settings=config.get('custom_settings', {})
        )
    
    def get_all_connector_configs(self) -> Dict[str, ConnectorConfig]:
        """获取所有连接器配置"""
        connectors = self.get('connectors', {})
        return {
            name: self.get_connector_config(name)
            for name in connectors.keys()
        }
    
    def get_scheduler_config(self) -> Dict[str, Any]:
        """获取调度器配置"""
        return self.get('scheduler', {})
    
    def get_monitoring_config(self) -> Dict[str, Any]:
        """获取监控配置"""
        return self.get('monitoring', {})
    
    def get_logging_config(self) -> Dict[str, Any]:
        """获取日志配置"""
        return self.get('logging', {})
    
    def is_local(self) -> bool:
        """是否为本地环境"""
        return self.environment == Environment.LOCAL
    
    def is_production(self) -> bool:
        """是否为生产环境"""
        return self.environment == Environment.PRODUCTION
    
    def validate_config(self) -> List[str]:
        """
        验证配置完整性
        
        Returns:
            List[str]: 验证错误列表
        """
        errors = []
        
        # 验证数据库配置
        db_config = self.get_database_config()
        if not db_config.password and self.is_production():
            errors.append("生产环境必须设置数据库密码")
        
        # 验证连接器配置
        connectors = self.get_all_connector_configs()
        for name, config in connectors.items():
            if config.enabled and not config.api_key:
                errors.append(f"连接器 {name} 已启用但缺少API密钥")
        
        # 验证调度器配置
        scheduler_config = self.get_scheduler_config()
        if not scheduler_config:
            errors.append("缺少调度器配置")
        
        return errors
    
    def _load_yaml_file(self, filename: str) -> Dict[str, Any]:
        """加载YAML配置文件"""
        filepath = self.config_dir / filename
        
        if not filepath.exists():
            logger.warning(f"配置文件不存在: {filepath}")
            return {}
        
        try:
            with open(filepath, 'r', encoding='utf-8') as f:
                return yaml.safe_load(f) or {}
        except Exception as e:
            logger.error(f"加载配置文件失败 {filepath}: {e}")
            return {}
    
    def _merge_configs(self, base: Dict[str, Any], override: Dict[str, Any]) -> Dict[str, Any]:
        """合并配置字典"""
        result = base.copy()
        
        for key, value in override.items():
            if key in result and isinstance(result[key], dict) and isinstance(value, dict):
                result[key] = self._merge_configs(result[key], value)
            else:
                result[key] = value
        
        return result
    
    def _apply_env_overrides(self):
        """应用环境变量覆盖"""
        # 环境变量映射
        env_mappings = {
            'HERMESFLOW_DB_HOST': 'database.host',
            'HERMESFLOW_DB_PORT': 'database.port',
            'HERMESFLOW_DB_NAME': 'database.database',
            'HERMESFLOW_DB_USER': 'database.username',
            'HERMESFLOW_DB_PASSWORD': 'database.password',
            'HERMESFLOW_REDIS_HOST': 'redis.host',
            'HERMESFLOW_REDIS_PORT': 'redis.port',
            'HERMESFLOW_REDIS_PASSWORD': 'redis.password',
            'HERMESFLOW_LOG_LEVEL': 'logging.level'
        }
        
        for env_var, config_key in env_mappings.items():
            env_value = os.getenv(env_var)
            if env_value:
                # 类型转换
                if config_key.endswith('.port'):
                    env_value = int(env_value)
                elif config_key.endswith('.ssl_enabled'):
                    env_value = env_value.lower() in ('true', '1', 'yes')
                
                self.set(config_key, env_value)
                logger.debug(f"环境变量覆盖: {config_key} = {env_value}")
    
    def _decrypt_secrets(self):
        """解密敏感配置"""
        # 解密连接器密钥
        connectors = self.get('connectors', {})
        for name, config in connectors.items():
            if 'api_key' in config and config['api_key'].startswith('ENC:'):
                config['api_key'] = self.secret_manager.decrypt(config['api_key'][4:])
            if 'api_secret' in config and config['api_secret'].startswith('ENC:'):
                config['api_secret'] = self.secret_manager.decrypt(config['api_secret'][4:])
        
        # 解密数据库密码
        db_config = self.get('database', {})
        if 'password' in db_config and db_config['password'].startswith('ENC:'):
            db_config['password'] = self.secret_manager.decrypt(db_config['password'][4:])
    
    def _start_file_watcher(self):
        """启动配置文件监听器"""
        if not self.config_dir.exists():
            return
        
        try:
            event_handler = ConfigFileHandler(self)
            self._observer = Observer()
            self._observer.schedule(event_handler, str(self.config_dir), recursive=False)
            self._observer.start()
            logger.info("配置文件监听器已启动")
        except Exception as e:
            logger.error(f"启动配置文件监听器失败: {e}")
    
    def shutdown(self):
        """关闭配置管理器"""
        if self._observer and self._observer.is_alive():
            self._observer.stop()
            self._observer.join()
            logger.info("配置文件监听器已关闭")
    
    def __del__(self):
        """析构函数"""
        self.shutdown()

    def _substitute_env_vars(self, config: Any) -> Any:
        """
        递归替换配置中的环境变量
        
        Args:
            config: 配置对象（可以是dict、list或字符串）
            
        Returns:
            替换后的配置对象
        """
        import re
        
        if isinstance(config, dict):
            for key, value in config.items():
                config[key] = self._substitute_env_vars(value)
        elif isinstance(config, list):
            for i, item in enumerate(config):
                config[i] = self._substitute_env_vars(item)
        elif isinstance(config, str):
            # 查找${VAR}格式的环境变量
            pattern = r'\$\{([^}]+)\}'
            matches = re.findall(pattern, config)
            
            for var_name in matches:
                env_value = os.getenv(var_name, '')
                config = config.replace(f'${{{var_name}}}', env_value)
                logger.debug(f"环境变量替换: ${{{var_name}}} -> {env_value}")
        
        return config 