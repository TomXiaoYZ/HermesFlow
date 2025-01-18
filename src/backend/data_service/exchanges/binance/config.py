"""
Binance配置
"""

# API URL配置
BINANCE_API_URL = {
    "mainnet": "https://api.binance.com/api",
    "testnet": "https://testnet.binance.vision/api"
}

# WebSocket URL配置
BINANCE_WS_URL = {
    "mainnet": "wss://stream.binance.com:9443",
    "testnet": "wss://testnet.binance.vision"
}

# WebSocket订阅主题
BINANCE_WS_TOPICS = {
    # 现货市场
    "spot": {
        # 市场数据
        "trade": "{symbol}@trade",           # 逐笔交易
        "ticker": "{symbol}@ticker",         # 24小时价格变动
        "miniTicker": "{symbol}@miniTicker", # 简洁版24小时价格变动
        "depth": "{symbol}@depth",           # 增量深度信息
        "depth5": "{symbol}@depth5",         # 5档深度信息
        "depth10": "{symbol}@depth10",       # 10档深度信息
        "depth20": "{symbol}@depth20",       # 20档深度信息
        "kline": "{symbol}@kline_{interval}",# K线数据
        
        # 用户数据
        "account": "outboundAccountPosition",  # 账户更新
        "balance": "outboundAccountInfo",      # 余额更新
        "order": "executionReport",           # 订单更新
    },
    
    # 合约市场
    "futures": {
        # 市场数据
        "trade": "{symbol}@trade",
        "ticker": "{symbol}@ticker",
        "miniTicker": "{symbol}@miniTicker",
        "depth": "{symbol}@depth",
        "depth5": "{symbol}@depth5",
        "depth10": "{symbol}@depth10",
        "depth20": "{symbol}@depth20",
        "kline": "{symbol}@kline_{interval}",
        "markPrice": "{symbol}@markPrice",
        "indexPrice": "{symbol}@indexPrice",
        
        # 用户数据
        "account": "outboundAccountPosition",
        "balance": "outboundAccountInfo",
        "order": "executionReport",
        "position": "ACCOUNT_UPDATE",
    }
}

# K线间隔
BINANCE_KLINE_INTERVALS = [
    "1m", "3m", "5m", "15m", "30m",  # 分钟
    "1h", "2h", "4h", "6h", "8h", "12h",  # 小时
    "1d", "3d",  # 天
    "1w",  # 周
    "1M"   # 月
] 