"""
Binance 相关异常类
"""
from typing import Optional, Dict, Any
from ...common.exceptions import APIError

class BinanceAPIError(APIError):
    """Binance API 异常"""
    def __init__(
        self,
        message: str,
        code: str = "BINANCE_API_ERROR",
        http_status: int = 500,
        details: Optional[Dict[str, Any]] = None
    ):
        super().__init__(
            message=message,
            exchange="binance",
            code=code,
            http_status=http_status,
            details=details
        )

class BinanceRequestError(BinanceAPIError):
    """请求错误"""
    def __init__(self, message: str, status_code: int, response: Dict[str, Any]):
        super().__init__(
            message=message,
            code="BINANCE_REQUEST_ERROR",
            http_status=status_code,
            details={"response": response}
        )

class BinanceRateLimitError(BinanceAPIError):
    """频率限制错误"""
    def __init__(self, message: str, retry_after: int):
        super().__init__(
            message=message,
            code="BINANCE_RATE_LIMIT",
            http_status=429,
            details={"retry_after": retry_after}
        )

class BinanceAuthError(BinanceAPIError):
    """认证错误"""
    def __init__(self, message: str):
        super().__init__(
            message=message,
            code="BINANCE_AUTH_ERROR",
            http_status=401
        )

class BinanceWebSocketError(BinanceAPIError):
    """WebSocket 错误"""
    def __init__(self, message: str, details: Optional[Dict[str, Any]] = None):
        super().__init__(
            message=message,
            code="BINANCE_WEBSOCKET_ERROR",
            http_status=500,
            details=details
        ) 