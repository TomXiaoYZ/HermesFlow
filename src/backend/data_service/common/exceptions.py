"""
异常模块
"""
from typing import Optional, Any, Dict

class DataServiceError(Exception):
    """数据服务基础异常类"""
    def __init__(
        self,
        message: str,
        code: str = "UNKNOWN_ERROR",
        http_status: int = 500,
        details: Optional[Dict[str, Any]] = None
    ):
        self.message = message
        self.code = code
        self.http_status = http_status
        self.details = details or {}
        super().__init__(message)

class APIError(DataServiceError):
    """API调用异常"""
    def __init__(
        self,
        message: str,
        exchange: str,
        code: str = "API_ERROR",
        http_status: int = 500,
        details: Optional[Dict[str, Any]] = None
    ):
        super().__init__(
            message=message,
            code=code,
            http_status=http_status,
            details={"exchange": exchange, **(details or {})}
        )

class NetworkError(DataServiceError):
    """网络异常"""
    def __init__(
        self,
        message: str,
        code: str = "NETWORK_ERROR",
        http_status: int = 503,
        details: Optional[Dict[str, Any]] = None
    ):
        super().__init__(
            message=message,
            code=code,
            http_status=http_status,
            details=details
        )

class ValidationError(DataServiceError):
    """数据验证异常"""
    def __init__(
        self,
        message: str,
        code: str = "VALIDATION_ERROR",
        http_status: int = 400,
        details: Optional[Dict[str, Any]] = None
    ):
        super().__init__(
            message=message,
            code=code,
            http_status=http_status,
            details=details
        )

class AuthenticationError(DataServiceError):
    """认证异常"""
    def __init__(
        self,
        message: str,
        code: str = "AUTHENTICATION_ERROR",
        http_status: int = 401,
        details: Optional[Dict[str, Any]] = None
    ):
        super().__init__(
            message=message,
            code=code,
            http_status=http_status,
            details=details
        )

class PermissionError(DataServiceError):
    """权限异常"""
    def __init__(
        self,
        message: str,
        code: str = "PERMISSION_ERROR",
        http_status: int = 403,
        details: Optional[Dict[str, Any]] = None
    ):
        super().__init__(
            message=message,
            code=code,
            http_status=http_status,
            details=details
        )

class RateLimitError(DataServiceError):
    """频率限制异常"""
    def __init__(
        self,
        message: str,
        code: str = "RATE_LIMIT_ERROR",
        http_status: int = 429,
        details: Optional[Dict[str, Any]] = None
    ):
        super().__init__(
            message=message,
            code=code,
            http_status=http_status,
            details=details
        )

class OrderError(DataServiceError):
    """订单相关异常"""
    def __init__(
        self,
        message: str,
        code: str = "ORDER_ERROR",
        http_status: int = 400,
        details: Optional[Dict[str, Any]] = None
    ):
        super().__init__(
            message=message,
            code=code,
            http_status=http_status,
            details=details
        )

class PositionError(DataServiceError):
    """持仓相关异常"""
    def __init__(
        self,
        message: str,
        code: str = "POSITION_ERROR",
        http_status: int = 400,
        details: Optional[Dict[str, Any]] = None
    ):
        super().__init__(
            message=message,
            code=code,
            http_status=http_status,
            details=details
        )

class WebSocketError(DataServiceError):
    """WebSocket相关异常"""
    def __init__(
        self,
        message: str,
        code: str = "WEBSOCKET_ERROR",
        http_status: int = 500,
        details: Optional[Dict[str, Any]] = None
    ):
        super().__init__(
            message=message,
            code=code,
            http_status=http_status,
            details=details
        ) 