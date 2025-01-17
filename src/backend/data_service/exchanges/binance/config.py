"""
Binance配置
"""

# REST API URL
BINANCE_API_URL = {
    "mainnet": "https://api.binance.com",
    "testnet": "https://testnet.binance.vision"
}

# WebSocket URL
BINANCE_WS_URL = {
    "mainnet": "wss://stream.binance.com:9443",
    "testnet": "wss://testnet.binance.vision"
}

# 订单状态更新事件
ORDER_UPDATE_EVENT = "executionReport"

# WebSocket订阅主题
WS_TOPICS = {
    "spot": {
        "trade": "{}@trade",  # 逐笔成交
        "ticker": "{}@ticker",  # 24小时价格变动
        "kline": "{}@kline_{}",  # K线
        "depth": "{}@depth{}",  # 深度
        "bookTicker": "{}@bookTicker",  # 最优挂单
    },
    "user": {
        "account": "outboundAccountPosition",  # 账户更新
        "balance": "outboundAccountInfo",  # 余额更新
        "order": "executionReport",  # 订单更新
    }
} 