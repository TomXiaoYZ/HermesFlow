"""
指标监控模块，配置 Prometheus 指标
"""
from prometheus_client import Counter, Gauge, Histogram, Info

# 服务信息
SERVICE_INFO = Info("data_service", "Data service information")
SERVICE_INFO.info({"version": "0.1.0"})

# API 请求计数器
API_REQUEST_COUNT = Counter(
    "api_request_total",
    "Total count of API requests",
    ["method", "endpoint", "status"]
)

# API 请求延迟直方图
API_REQUEST_LATENCY = Histogram(
    "api_request_latency_seconds",
    "API request latency in seconds",
    ["method", "endpoint"]
)

# 数据处理计数器
DATA_PROCESSING_COUNT = Counter(
    "data_processing_total",
    "Total count of data processing operations",
    ["exchange", "data_type", "status"]
)

# 数据处理延迟直方图
DATA_PROCESSING_LATENCY = Histogram(
    "data_processing_latency_seconds",
    "Data processing latency in seconds",
    ["exchange", "data_type"]
)

# WebSocket 连接计数器
WEBSOCKET_CONNECTION_COUNT = Counter(
    "websocket_connection_total",
    "Total count of WebSocket connections",
    ["exchange", "status"]
)

# 当前活跃的 WebSocket 连接数
ACTIVE_WEBSOCKET_CONNECTIONS = Gauge(
    "active_websocket_connections",
    "Number of currently active WebSocket connections",
    ["exchange"]
)

# 数据存储计数器
DATA_STORAGE_COUNT = Counter(
    "data_storage_total",
    "Total count of data storage operations",
    ["storage_type", "operation", "status"]
)

# 数据存储延迟直方图
DATA_STORAGE_LATENCY = Histogram(
    "data_storage_latency_seconds",
    "Data storage latency in seconds",
    ["storage_type", "operation"]
)

# Kafka 消息计数器
KAFKA_MESSAGE_COUNT = Counter(
    "kafka_message_total",
    "Total count of Kafka messages",
    ["topic", "operation", "status"]
)

# 内存使用量
MEMORY_USAGE = Gauge(
    "memory_usage_bytes",
    "Memory usage in bytes",
    ["type"]
)

# CPU 使用率
CPU_USAGE = Gauge(
    "cpu_usage_percent",
    "CPU usage percentage",
    ["type"]
) 