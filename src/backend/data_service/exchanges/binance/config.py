"""
Binance配置
"""

# API URL配置
BINANCE_API_URL = {
    "mainnet": "https://api.binance.com/api",  # 主网
    "testnet": "https://testnet.binance.vision/api",  # 测试网
}

# 合约API URL配置
BINANCE_FUTURES_API_URL = {
    "mainnet": "https://fapi.binance.com",  # 主网
    "testnet": "https://testnet.binancefuture.com",  # 测试网
}

# WebSocket URL配置
BINANCE_WS_URL = {
    "mainnet": "wss://stream.binance.com:9443/ws",  # 主网
    "testnet": "wss://testnet.binance.vision/ws",  # 测试网
}

# 合约WebSocket URL配置
BINANCE_FUTURES_WS_URL = {
    "mainnet": "wss://fstream.binance.com/ws",  # 主网
    "testnet": "wss://stream.binancefuture.com/ws",  # 测试网
}

# WebSocket订阅主题
BINANCE_WS_TOPICS = {
    "spot": {
        "trade": "{}@trade",  # 逐笔成交
        "ticker": "{}@ticker",  # 24小时价格变动
        "kline": "{}@kline_{}",  # K线
        "depth": "{}@depth{}",  # 深度
        "bookTicker": "{}@bookTicker",  # 最优挂单
    },
    "futures": {
        "trade": "{}@trade",  # 逐笔成交
        "ticker": "{}@ticker",  # 24小时价格变动
        "kline": "{}@kline_{}",  # K线
        "depth": "{}@depth{}",  # 深度
        "bookTicker": "{}@bookTicker",  # 最优挂单
        "markPrice": "{}@markPrice",  # 标记价格
        "fundingRate": "{}@fundingRate",  # 资金费率
    },
    "user": {
        "account": "{}@account",  # 账户更新
        "balance": "{}@balance",  # 余额更新
        "order": "{}@order",  # 订单更新
        "position": "{}@position",  # 持仓更新
    }
}

# K线间隔
BINANCE_KLINE_INTERVALS = [
    "1m", "3m", "5m", "15m", "30m",  # 分钟线
    "1h", "2h", "4h", "6h", "8h", "12h",  # 小时线
    "1d", "3d",  # 日线
    "1w",  # 周线
    "1M"  # 月线
] 