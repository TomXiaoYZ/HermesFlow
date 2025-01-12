"""
配置模块，用于管理服务的所有配置项
"""
from typing import List

from pydantic import Field
from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    """服务配置类"""
    
    # 服务配置
    SERVICE_NAME: str = "data-service"
    API_V1_STR: str = "/api/v1"
    DEBUG: bool = False
    
    # 服务器配置
    HOST: str = "0.0.0.0"
    PORT: int = 8000
    
    # Redis 配置
    REDIS_HOST: str = "localhost"
    REDIS_PORT: int = 6379
    REDIS_PASSWORD: str = ""
    REDIS_DB: int = 0
    
    # ClickHouse 配置
    CLICKHOUSE_HOST: str = "localhost"
    CLICKHOUSE_PORT: int = 8123
    CLICKHOUSE_USER: str = "default"
    CLICKHOUSE_PASSWORD: str = ""
    CLICKHOUSE_DATABASE: str = "hermesflow"
    
    # Kafka 配置
    KAFKA_BROKERS: List[str] = Field(default_factory=lambda: ["localhost:9092"])
    KAFKA_GROUP_ID: str = "hermesflow-data-service"
    
    # Binance 配置
    BINANCE_API_KEY: str = ""
    BINANCE_API_SECRET: str = ""
    BINANCE_TESTNET: bool = False
    
    # 日志配置
    LOG_LEVEL: str = "INFO"
    
    # 监控配置
    ENABLE_METRICS: bool = True
    METRICS_PORT: int = 8001
    
    # 缓存配置
    CACHE_TTL: int = 60  # 秒
    
    model_config = SettingsConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        case_sensitive=True,
    )


settings = Settings() 