"""
OKX相关异常类
"""
from typing import Optional, Dict, Any
from ...common.exceptions import APIError

class OKXAPIError(APIError):
    """OKX API异常基类"""
    def __init__(
        self,
        message: str,
        code: str = "OKX_API_ERROR",
        http_status: int = 500,
        details: Optional[Dict[str, Any]] = None
    ):
        super().__init__(
            message=message,
            exchange="okx",
            code=code,
            http_status=http_status,
            details=details
        )

class OKXRequestError(OKXAPIError):
    """请求错误"""
    def __init__(self, message: str, status_code: int, response: Dict[str, Any]):
        super().__init__(
            message=message,
            code="OKX_REQUEST_ERROR",
            http_status=status_code,
            details={"response": response}
        )

class OKXRateLimitError(OKXAPIError):
    """频率限制错误"""
    def __init__(self, message: str, retry_after: int):
        super().__init__(
            message=message,
            code="OKX_RATE_LIMIT",
            http_status=429,
            details={"retry_after": retry_after}
        )

class OKXAuthError(OKXAPIError):
    """认证错误"""
    def __init__(self, message: str):
        super().__init__(
            message=message,
            code="OKX_AUTH_ERROR",
            http_status=401
        )

class OKXWebSocketError(OKXAPIError):
    """WebSocket错误"""
    def __init__(self, message: str, details: Optional[Dict[str, Any]] = None):
        super().__init__(
            message=message,
            code="OKX_WEBSOCKET_ERROR",
            http_status=500,
            details=details
        ) 