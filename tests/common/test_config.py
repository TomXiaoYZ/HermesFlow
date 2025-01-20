"""
测试配置
定义各个交易所的测试配置
"""
from typing import Dict, Any

# 测试环境配置
TEST_ENV = {
    "postgres": {
        "host": "localhost",
        "port": 5432,
        "database": "test_db",
        "user": "test",
        "password": "test"
    },
    "redis": {
        "host": "localhost",
        "port": 6379,
        "db": 0
    }
}

# Binance测试配置
BINANCE_CONFIG = {
    "exchange": "binance",
    "api_key_env": "BINANCE_API_KEY",
    "api_secret_env": "BINANCE_API_SECRET",
    "test_symbols": ["BTCUSDT", "ETHUSDT", "BNBUSDT"],
    "markets": ["spot", "futures"],
    "test_amount": "0.001",
    "test_leverage": 3
}

# OKX测试配置
OKX_CONFIG = {
    "exchange": "okx",
    "api_key_env": "OKX_API_KEY",
    "api_secret_env": "OKX_API_SECRET",
    "passphrase_env": "OKX_PASSPHRASE",
    "test_symbols": ["BTC-USDT", "ETH-USDT", "LTC-USDT"],
    "markets": ["spot", "futures"],
    "test_amount": "0.001",
    "test_leverage": 3
}

# Bitget测试配置
BITGET_CONFIG = {
    "exchange": "bitget",
    "api_key_env": "BITGET_API_KEY",
    "api_secret_env": "BITGET_API_SECRET",
    "passphrase_env": "BITGET_PASSPHRASE",
    "test_symbols": ["BTCUSDT", "ETHUSDT"],
    "markets": ["spot", "futures"],
    "test_amount": "0.001",
    "test_leverage": 3
}

# 性能测试配置
PERF_TEST_CONFIG = {
    "concurrent_requests": 100,  # 并发请求数
    "batch_size": 10,  # 批次大小
    "test_duration": 300,  # 测试时长(秒)
    "sample_interval": 1,  # 采样间隔(秒)
    "success_rate_threshold": 0.95,  # 成功率阈值
    "avg_latency_threshold": 1.0,  # 平均延迟阈值(秒)
    "p95_latency_threshold": 2.0,  # P95延迟阈值(秒)
    "memory_growth_threshold": 100,  # 内存增长阈值(MB)
    "max_disconnections": 3  # 最大断连次数
}

# WebSocket测试配置
WS_TEST_CONFIG = {
    "ping_interval": 20,  # 心跳间隔(秒)
    "ping_timeout": 10,  # 心跳超时(秒)
    "reconnect_delay": 5,  # 重连延迟(秒)
    "max_reconnects": 3  # 最大重连次数
}

def get_exchange_config(exchange: str) -> Dict[str, Any]:
    """获取指定交易所的配置"""
    configs = {
        "binance": BINANCE_CONFIG,
        "okx": OKX_CONFIG,
        "bitget": BITGET_CONFIG
    }
    return configs.get(exchange, {}) 