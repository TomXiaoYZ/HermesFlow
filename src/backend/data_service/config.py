"""
数据服务配置文件
"""
import os
from typing import Dict, Any
from dotenv import load_dotenv

# 加载环境变量
load_dotenv()

# 应用配置
APP_CONFIG = {
    "name": os.getenv("APP_NAME", "HermesFlow-DataService"),
    "env": os.getenv("APP_ENV", "development"),
    "debug": os.getenv("APP_DEBUG", "true").lower() == "true",
}

# 数据库配置
DB_CONFIG = {
    "postgres": {
        "host": os.getenv("POSTGRES_HOST", "localhost"),
        "port": int(os.getenv("POSTGRES_PORT", "5432")),
        "database": os.getenv("POSTGRES_DB", "hermesflow"),
        "user": os.getenv("POSTGRES_USER", "postgres"),
        "password": os.getenv("POSTGRES_PASSWORD", "password"),
    },
    "redis": {
        "host": os.getenv("REDIS_HOST", "localhost"),
        "port": int(os.getenv("REDIS_PORT", "6379")),
        "password": os.getenv("REDIS_PASSWORD", ""),
        "db": 0,
    },
    "clickhouse": {
        "host": os.getenv("CLICKHOUSE_HOST", "localhost"),
        "port": int(os.getenv("CLICKHOUSE_PORT", "9000")),
        "database": os.getenv("CLICKHOUSE_DB", "hermesflow"),
        "user": os.getenv("CLICKHOUSE_USER", "default"),
        "password": os.getenv("CLICKHOUSE_PASSWORD", ""),
    },
}

# Kafka配置
KAFKA_CONFIG = {
    "bootstrap_servers": os.getenv("KAFKA_BROKERS", "localhost:9092").split(","),
    "group_id": os.getenv("KAFKA_GROUP_ID", "hermesflow"),
}

# 交易所API配置
EXCHANGE_CONFIG = {
    "binance": {
        "api_key": os.getenv("BINANCE_API_KEY", ""),
        "api_secret": os.getenv("BINANCE_API_SECRET", ""),
        "testnet": APP_CONFIG["env"] != "production",
    },
    "okx": {
        "api_key": os.getenv("OKX_API_KEY", ""),
        "api_secret": os.getenv("OKX_API_SECRET", ""),
        "passphrase": os.getenv("OKX_PASSPHRASE", ""),
        "testnet": APP_CONFIG["env"] != "production",
    },
    "bitget": {
        "api_key": os.getenv("BITGET_API_KEY", ""),
        "api_secret": os.getenv("BITGET_API_SECRET", ""),
        "testnet": APP_CONFIG["env"] != "production",
    },
}

# 日志配置
LOG_CONFIG = {
    "level": "DEBUG" if APP_CONFIG["debug"] else "INFO",
    "format": "%(asctime)s - %(name)s - %(levelname)s - %(message)s",
}

# 监控配置
MONITOR_CONFIG = {
    "prometheus_port": int(os.getenv("PROMETHEUS_PORT", "9090")),
}

def get_exchange_config(exchange: str) -> Dict[str, Any]:
    """获取指定交易所的配置

    Args:
        exchange: 交易所名称

    Returns:
        Dict[str, Any]: 交易所配置
    """
    return EXCHANGE_CONFIG.get(exchange, {}) 