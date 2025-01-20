"""
OKX日志配置
"""
import structlog
from ...common.logger import setup_logging

# 创建logger
logger = structlog.get_logger("okx")

def setup_okx_logging():
    """设置OKX日志配置"""
    setup_logging(
        log_level="INFO",
        log_dir="logs",
        app_name="okx"
    )

    # 添加OKX特定的处理器
    structlog.configure(
        processors=[
            structlog.stdlib.filter_by_level,
            structlog.processors.TimeStamper(fmt="iso"),
            structlog.stdlib.add_logger_name,
            structlog.stdlib.add_log_level,
            structlog.stdlib.PositionalArgumentsFormatter(),
            structlog.processors.StackInfoRenderer(),
            structlog.processors.format_exc_info,
            structlog.processors.UnicodeDecoder(),
            # 添加OKX特定的处理器
            _add_okx_context,
            structlog.processors.JSONRenderer()
        ],
        context_class=dict,
        logger_factory=structlog.stdlib.LoggerFactory(),
        wrapper_class=structlog.stdlib.BoundLogger,
        cache_logger_on_first_use=True,
    )

def _add_okx_context(logger, method_name, event_dict):
    """添加OKX上下文信息"""
    # 添加交易所标识
    event_dict["exchange"] = "okx"
    
    # 添加环境标识
    event_dict["env"] = "testnet" if event_dict.get("testnet", False) else "mainnet"
    
    # 添加API类型
    if "ws" in event_dict.get("type", ""):
        event_dict["api_type"] = "websocket"
    else:
        event_dict["api_type"] = "rest"
    
    return event_dict

def log_request(method: str, endpoint: str, params: dict = None):
    """记录API请求日志"""
    logger.info(
        "okx_api_request",
        type="request",
        method=method,
        endpoint=endpoint,
        params=params
    )

def log_response(method: str, endpoint: str, status_code: int, response: dict):
    """记录API响应日志"""
    logger.info(
        "okx_api_response",
        type="response",
        method=method,
        endpoint=endpoint,
        status_code=status_code,
        response=response
    )

def log_error(method: str, endpoint: str, error: Exception):
    """记录API错误日志"""
    logger.error(
        "okx_api_error",
        type="error",
        method=method,
        endpoint=endpoint,
        error=str(error),
        error_type=error.__class__.__name__
    )

def log_ws_message(stream_type: str, message: dict):
    """记录WebSocket消息日志"""
    logger.debug(
        "okx_ws_message",
        type="ws_message",
        stream_type=stream_type,
        message=message
    )

def log_ws_error(stream_type: str, error: Exception):
    """记录WebSocket错误日志"""
    logger.error(
        "okx_ws_error",
        type="ws_error",
        stream_type=stream_type,
        error=str(error),
        error_type=error.__class__.__name__
    )

def log_order_event(event_type: str, order: dict):
    """记录订单事件日志"""
    logger.info(
        "okx_order_event",
        type="order_event",
        event_type=event_type,
        order=order
    ) 