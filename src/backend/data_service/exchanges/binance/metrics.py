"""
Binance 监控指标
"""
from prometheus_client import Counter, Histogram, Gauge

# API请求监控
REQUEST_LATENCY = Histogram(
    'binance_api_request_latency_seconds',
    'Binance API请求延迟',
    ['method', 'endpoint']
)

REQUEST_ERRORS = Counter(
    'binance_api_request_errors_total',
    'Binance API请求错误总数',
    ['method', 'endpoint', 'error_type']
)

RATE_LIMIT_REMAINING = Gauge(
    'binance_api_rate_limit_remaining',
    'Binance API剩余请求次数',
    ['interval']
)

# WebSocket监控
WS_CONNECTED = Gauge(
    'binance_websocket_connected',
    'Binance WebSocket连接状态',
    ['stream_type']
)

WS_MESSAGES = Counter(
    'binance_websocket_messages_total',
    'Binance WebSocket消息总数',
    ['stream_type', 'event_type']
)

WS_ERRORS = Counter(
    'binance_websocket_errors_total',
    'Binance WebSocket错误总数',
    ['stream_type', 'error_type']
)

WS_RECONNECTS = Counter(
    'binance_websocket_reconnects_total',
    'Binance WebSocket重连次数',
    ['stream_type']
)

# 订单监控
ORDER_CREATED = Counter(
    'binance_orders_created_total',
    'Binance创建订单总数',
    ['market', 'symbol', 'order_type', 'side']
)

ORDER_FILLED = Counter(
    'binance_orders_filled_total',
    'Binance成交订单总数',
    ['market', 'symbol', 'order_type', 'side']
)

ORDER_CANCELED = Counter(
    'binance_orders_canceled_total',
    'Binance取消订单总数',
    ['market', 'symbol', 'order_type', 'side']
)

ORDER_REJECTED = Counter(
    'binance_orders_rejected_total',
    'Binance拒绝订单总数',
    ['market', 'symbol', 'order_type', 'side', 'reason']
)

ORDER_LATENCY = Histogram(
    'binance_order_latency_seconds',
    'Binance订单处理延迟',
    ['market', 'symbol', 'order_type', 'side']
)

# 资金费率监控
FUNDING_RATE = Gauge(
    'binance_funding_rate',
    'Binance资金费率',
    ['symbol']
)

PREDICTED_FUNDING_RATE = Gauge(
    'binance_predicted_funding_rate',
    'Binance预测资金费率',
    ['symbol']
)

# 系统监控
SYSTEM_MEMORY = Gauge(
    'binance_client_memory_bytes',
    'Binance客户端内存使用'
)

SYSTEM_CPU = Gauge(
    'binance_client_cpu_usage',
    'Binance客户端CPU使用率'
) 