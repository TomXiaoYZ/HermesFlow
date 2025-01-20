"""
Bitget交易所配置
"""
from typing import Dict, Any

# API URLs
API_URLS = {
    "main": {
        "rest": "https://api.bitget.com",
        "ws": "wss://ws.bitget.com/spot/v1/stream",
        "ws_private": "wss://ws.bitget.com/spot/v1/stream/private"
    },
    "test": {
        "rest": "https://api-testnet.bitget.com",
        "ws": "wss://ws-testnet.bitget.com/spot/v1/stream",
        "ws_private": "wss://ws-testnet.bitget.com/spot/v1/stream/private"
    }
}

# API端点
ENDPOINTS = {
    # 市场数据
    "market": {
        "ticker": "/api/spot/v1/market/ticker",
        "depth": "/api/spot/v1/market/depth",
        "trades": "/api/spot/v1/market/trades",
        "klines": "/api/spot/v1/market/candles",
        "symbols": "/api/spot/v1/public/products"
    },
    # 交易
    "trade": {
        "create_order": "/api/spot/v1/trade/orders",
        "cancel_order": "/api/spot/v1/trade/cancel-order",
        "get_order": "/api/spot/v1/trade/order",
        "get_open_orders": "/api/spot/v1/trade/open-orders"
    },
    # 账户
    "account": {
        "balance": "/api/spot/v1/account/assets",
        "positions": "/api/spot/v1/account/positions"
    }
}

# WebSocket订阅主题
WS_TOPICS = {
    "public": {
        "ticker": "ticker",
        "depth": "depth",
        "trades": "trade",
        "klines": "candle"
    },
    "private": {
        "orders": "orders",
        "positions": "positions",
        "account": "account"
    }
}

# 请求配置
REQUEST_CONFIG = {
    "timeout": 10,  # 请求超时时间（秒）
    "max_retries": 3,  # 最大重试次数
    "retry_delay": 1,  # 重试延迟（秒）
}

# WebSocket配置
WS_CONFIG = {
    "ping_interval": 20,  # 心跳间隔（秒）
    "ping_timeout": 10,  # 心跳超时（秒）
    "reconnect_delay": 5,  # 重连延迟（秒）
    "max_reconnects": 5,  # 最大重连次数
}

# 错误码映射
ERROR_CODES: Dict[int, str] = {
    10000: "操作成功",
    10001: "系统错误",
    10002: "系统繁忙",
    10003: "参数错误",
    10004: "无效的API Key",
    10005: "无效的签名",
    10006: "IP地址不在白名单内",
    10007: "请求频率超限",
    10008: "余额不足",
    10009: "订单不存在",
    10010: "下单数量无效",
    10011: "下单价格无效",
    10012: "交易对不存在"
} 