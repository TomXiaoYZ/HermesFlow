"""
OKX监控指标
"""
from prometheus_client import Counter, Histogram, Gauge

# API请求监控
REQUEST_LATENCY = Histogram(
    'okx_api_request_latency_seconds',
    'OKX API请求延迟',
    ['method', 'endpoint']
)

REQUEST_ERRORS = Counter(
    'okx_api_request_errors_total',
    'OKX API请求错误总数',
    ['method', 'endpoint', 'error_type']
)

RATE_LIMIT_REMAINING = Gauge(
    'okx_api_rate_limit_remaining',
    'OKX API剩余请求次数',
    ['interval']
)

# WebSocket监控
WS_CONNECTED = Gauge(
    'okx_websocket_connected',
    'OKX WebSocket连接状态',
    ['stream_type']
)

WS_MESSAGES = Counter(
    'okx_websocket_messages_total',
    'OKX WebSocket消息总数',
    ['stream_type', 'event_type']
)

WS_ERRORS = Counter(
    'okx_websocket_errors_total',
    'OKX WebSocket错误总数',
    ['stream_type', 'error_type']
)

WS_RECONNECTS = Counter(
    'okx_websocket_reconnects_total',
    'OKX WebSocket重连次数',
    ['stream_type']
)

# 订单监控
ORDER_CREATED = Counter(
    'okx_orders_created_total',
    'OKX创建订单总数',
    ['market', 'symbol', 'order_type', 'side']
)

ORDER_FILLED = Counter(
    'okx_orders_filled_total',
    'OKX成交订单总数',
    ['market', 'symbol', 'order_type', 'side']
)

ORDER_CANCELED = Counter(
    'okx_orders_canceled_total',
    'OKX取消订单总数',
    ['market', 'symbol', 'order_type', 'side']
)

ORDER_REJECTED = Counter(
    'okx_orders_rejected_total',
    'OKX拒绝订单总数',
    ['market', 'symbol', 'order_type', 'side', 'reason']
)

ORDER_LATENCY = Histogram(
    'okx_order_latency_seconds',
    'OKX订单处理延迟',
    ['market', 'symbol', 'order_type', 'side']
)

# 资金费率监控
FUNDING_RATE = Gauge(
    'okx_funding_rate',
    'OKX资金费率',
    ['symbol']
)

PREDICTED_FUNDING_RATE = Gauge(
    'okx_predicted_funding_rate',
    'OKX预测资金费率',
    ['symbol']
)

# 系统监控
SYSTEM_MEMORY = Gauge(
    'okx_client_memory_bytes',
    'OKX客户端内存使用'
)

SYSTEM_CPU = Gauge(
    'okx_client_cpu_usage',
    'OKX客户端CPU使用率'
) 