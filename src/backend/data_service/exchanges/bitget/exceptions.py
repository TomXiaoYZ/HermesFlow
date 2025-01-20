"""
Bitget交易所错误处理
"""
from typing import Optional, Dict, Any

class BitgetAPIError(Exception):
    """Bitget API错误基类"""
    
    def __init__(
        self,
        message: str,
        status_code: Optional[int] = None,
        response: Optional[Dict[str, Any]] = None,
        request_id: Optional[str] = None
    ):
        self.message = message
        self.status_code = status_code
        self.response = response
        self.request_id = request_id
        super().__init__(self.message)

class BitgetRequestError(BitgetAPIError):
    """请求错误，包含HTTP状态码和响应内容"""
    
    def __init__(
        self,
        message: str,
        status_code: int,
        response: Optional[Dict[str, Any]] = None,
        request_id: Optional[str] = None
    ):
        super().__init__(
            message=message,
            status_code=status_code,
            response=response,
            request_id=request_id
        )

class BitgetRateLimitError(BitgetAPIError):
    """频率限制错误"""
    
    def __init__(
        self,
        message: str,
        retry_after: Optional[int] = None,
        response: Optional[Dict[str, Any]] = None,
        request_id: Optional[str] = None
    ):
        self.retry_after = retry_after
        super().__init__(
            message=message,
            status_code=429,
            response=response,
            request_id=request_id
        )

class BitgetAuthError(BitgetAPIError):
    """认证错误"""
    
    def __init__(
        self,
        message: str,
        response: Optional[Dict[str, Any]] = None,
        request_id: Optional[str] = None
    ):
        super().__init__(
            message=message,
            status_code=401,
            response=response,
            request_id=request_id
        )

class BitgetWebSocketError(BitgetAPIError):
    """WebSocket错误"""
    
    def __init__(
        self,
        message: str,
        code: Optional[int] = None,
        response: Optional[Dict[str, Any]] = None
    ):
        self.code = code
        super().__init__(
            message=message,
            response=response
        )

class BitgetOrderError(BitgetAPIError):
    """订单错误"""
    
    def __init__(
        self,
        message: str,
        order_id: Optional[str] = None,
        client_order_id: Optional[str] = None,
        response: Optional[Dict[str, Any]] = None
    ):
        self.order_id = order_id
        self.client_order_id = client_order_id
        super().__init__(
            message=message,
            response=response
        ) 