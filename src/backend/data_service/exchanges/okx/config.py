"""
OKX配置
"""

# API URL配置
OKX_API_URL = {
    "mainnet": "https://www.okx.com",  # 主网
    "testnet": "https://www.okx.com/api-test",  # 测试网
}

# WebSocket URL配置
OKX_WS_URL = {
    "mainnet": "wss://ws.okx.com:8443/ws/v5/public",  # 主网公共频道
    "testnet": "wss://wspap.okx.com:8443/ws/v5/public?brokerId=9999",  # 测试网公共频道
}

OKX_PRIVATE_WS_URL = {
    "mainnet": "wss://ws.okx.com:8443/ws/v5/private",  # 主网私有频道
    "testnet": "wss://wspap.okx.com:8443/ws/v5/private?brokerId=9999",  # 测试网私有频道
}

# WebSocket订阅主题
OKX_WS_TOPICS = {
    "spot": {
        "tickers": "spot/ticker:{}",  # 行情频道
        "trades": "spot/trade:{}",  # 交易频道
        "depth": "spot/depth_{}:{}",  # 深度频道
        "kline": "spot/candle{}s:{}",  # K线频道
    },
    "futures": {
        "tickers": "futures/ticker:{}",  # 行情频道
        "trades": "futures/trade:{}",  # 交易频道
        "depth": "futures/depth_{}:{}",  # 深度频道
        "kline": "futures/candle{}s:{}",  # K线频道
        "funding_rate": "futures/funding_rate:{}",  # 资金费率频道
        "price_limit": "futures/price_limit:{}",  # 价格限制频道
        "mark_price": "futures/mark_price:{}",  # 标记价格频道
    },
    "account": {
        "account": "account/account",  # 账户频道
        "positions": "account/positions",  # 持仓频道
        "orders": "account/orders",  # 订单频道
        "algo_orders": "account/algo_orders",  # 策略委托频道
    }
}

# API接口
OKX_API_ENDPOINTS = {
    "spot": {
        "instruments": "/api/v5/public/instruments",  # 获取交易产品基础信息
        "ticker": "/api/v5/market/ticker",  # 获取单个产品行情信息
        "tickers": "/api/v5/market/tickers",  # 获取所有产品行情信息
        "depth": "/api/v5/market/books",  # 获取产品深度
        "trades": "/api/v5/market/trades",  # 获取交易产品历史成交记录
        "kline": "/api/v5/market/candles",  # 获取交易产品K线数据
    },
    "account": {
        "balance": "/api/v5/account/balance",  # 获取账户余额
        "positions": "/api/v5/account/positions",  # 获取持仓信息
        "leverage": "/api/v5/account/leverage-info",  # 获取杠杆倍数
        "max_size": "/api/v5/account/max-size",  # 获取最大可交易数量
        "max_avail_size": "/api/v5/account/max-avail-size",  # 获取最大可用数量
        "margin_balance": "/api/v5/account/margin-balance",  # 获取保证金余额
        "position_mode": "/api/v5/account/position-mode",  # 获取持仓方式
    },
    "trade": {
        "order": "/api/v5/trade/order",  # 下单
        "batch_orders": "/api/v5/trade/batch-orders",  # 批量下单
        "cancel_order": "/api/v5/trade/cancel-order",  # 撤单
        "cancel_batch_orders": "/api/v5/trade/cancel-batch-orders",  # 批量撤单
        "amend_order": "/api/v5/trade/amend-order",  # 修改订单
        "amend_batch_orders": "/api/v5/trade/amend-batch-orders",  # 批量修改订单
        "close_position": "/api/v5/trade/close-position",  # 市价全平
        "order_info": "/api/v5/trade/order",  # 获取订单信息
        "orders_pending": "/api/v5/trade/orders-pending",  # 获取未成交订单列表
        "orders_history": "/api/v5/trade/orders-history",  # 获取历史订单记录
        "fills": "/api/v5/trade/fills",  # 获取成交明细
    }
} 