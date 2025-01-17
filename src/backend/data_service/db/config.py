"""
数据库配置
"""
import os
from typing import Dict, Any

# PostgreSQL配置
POSTGRESQL_CONFIG = {
    "host": os.getenv("POSTGRES_HOST", "localhost"),
    "port": int(os.getenv("POSTGRES_PORT", "5432")),
    "database": os.getenv("POSTGRES_DB", "hermesflow"),
    "user": os.getenv("POSTGRES_USER", "postgres"),
    "password": os.getenv("POSTGRES_PASSWORD", "postgres"),
    "min_size": int(os.getenv("POSTGRES_POOL_MIN_SIZE", "5")),
    "max_size": int(os.getenv("POSTGRES_POOL_MAX_SIZE", "20")),
}

# Redis配置
REDIS_CONFIG = {
    "host": os.getenv("REDIS_HOST", "localhost"),
    "port": int(os.getenv("REDIS_PORT", "6379")),
    "db": int(os.getenv("REDIS_DB", "0")),
    "password": os.getenv("REDIS_PASSWORD", None),
    "max_connections": int(os.getenv("REDIS_MAX_CONNECTIONS", "10")),
}

# Kafka配置
KAFKA_CONFIG = {
    "bootstrap_servers": os.getenv("KAFKA_BOOTSTRAP_SERVERS", "localhost:9092"),
    "client_id": "hermesflow",
    "group_id": "hermesflow_order_processor",
    "auto_offset_reset": "earliest",
    "enable_auto_commit": False,
    "max_poll_interval_ms": 300000,
    "max_poll_records": 500,
}

# 主题配置
KAFKA_TOPICS = {
    "order_events": "hermesflow.orders.events",
    "trade_events": "hermesflow.trades.events",
} 