"""
OKX交易所API客户端
"""
import hmac
import base64
import json
import time
from typing import Dict, Any, List, Optional
from datetime import datetime, timezone
import aiohttp
from decimal import Decimal

from ....common.models import OrderType, OrderSide, Market
from ....common.exceptions import NetworkError, ValidationError
from .exceptions import (
    OKXAPIError, OKXRequestError, OKXRateLimitError,
    OKXAuthError
)

class OKXAPI:
    """OKX API客户端"""
    
    def __init__(
        self,
        api_key: str,
        api_secret: str,
        passphrase: str,
        testnet: bool = False
    ):
        """初始化API客户端
        
        Args:
            api_key: API Key
            api_secret: API Secret
            passphrase: API密码
            testnet: 是否使用测试网
        """
        self.api_key = api_key
        self.api_secret = api_secret.encode()
        self.passphrase = passphrase
        self.base_url = "https://www.okx.com" if not testnet else "https://www.okx.com"
        self._session = None
        
    async def __aenter__(self):
        """异步上下文管理器入口"""
        if not self._session:
            self._session = aiohttp.ClientSession()
        return self
        
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """异步上下文管理器出口"""
        if self._session:
            await self._session.close()
            self._session = None

    def _get_timestamp(self) -> str:
        """获取ISO格式的时间戳"""
        return datetime.now(timezone.utc).isoformat()[:-6] + 'Z'
        
    def _sign(self, timestamp: str, method: str, request_path: str, body: str = "") -> str:
        """生成请求签名
        
        Args:
            timestamp: 时间戳
            method: 请求方法
            request_path: 请求路径
            body: 请求体
            
        Returns:
            str: 签名字符串
        """
        message = timestamp + method + request_path + body
        mac = hmac.new(
            self.api_secret,
            message.encode(),
            digestmod='sha256'
        )
        return base64.b64encode(mac.digest()).decode()
        
    async def _request(
        self,
        method: str,
        path: str,
        params: Optional[Dict] = None,
        data: Optional[Dict] = None,
        auth: bool = False
    ) -> Any:
        """发送HTTP请求
        
        Args:
            method: 请求方法
            path: 请求路径
            params: URL参数
            data: 请求体数据
            auth: 是否需要认证
            
        Returns:
            Any: 响应数据
            
        Raises:
            NetworkError: 网络错误
            OKXAPIError: API错误
        """
        if not self._session:
            self._session = aiohttp.ClientSession()
            
        url = self.base_url + path
        headers = {}
        
        if auth:
            timestamp = self._get_timestamp()
            body = json.dumps(data) if data else ""
            signature = self._sign(timestamp, method, path, body)
            
            headers.update({
                "OK-ACCESS-KEY": self.api_key,
                "OK-ACCESS-SIGN": signature,
                "OK-ACCESS-TIMESTAMP": timestamp,
                "OK-ACCESS-PASSPHRASE": self.passphrase
            })
            
        try:
            async with self._session.request(
                method,
                url,
                params=params,
                json=data,
                headers=headers
            ) as response:
                if response.status == 429:
                    raise OKXRateLimitError("请求频率超限")
                elif response.status == 401:
                    raise OKXAuthError("认证失败")
                elif response.status != 200:
                    text = await response.text()
                    raise OKXRequestError(f"请求失败: {text}")
                    
                result = await response.json()
                if result.get("code") != "0":
                    raise OKXAPIError(f"API错误: {result}")
                    
                return result.get("data", [])
                
        except aiohttp.ClientError as e:
            raise NetworkError(f"网络错误: {str(e)}")
            
    async def get_ticker(self, symbol: str) -> Dict:
        """获取行情数据
        
        Args:
            symbol: 交易对
            
        Returns:
            Dict: 行情数据
        """
        return (await self._request(
            "GET",
            "/api/v5/market/ticker",
            params={"instId": symbol}
        ))[0]
        
    async def get_depth(self, symbol: str, limit: int = 100) -> Dict:
        """获取深度数据
        
        Args:
            symbol: 交易对
            limit: 深度
            
        Returns:
            Dict: 深度数据
        """
        return (await self._request(
            "GET",
            "/api/v5/market/books",
            params={
                "instId": symbol,
                "sz": limit
            }
        ))[0]
        
    async def get_trades(self, symbol: str, limit: int = 100) -> List[Dict]:
        """获取最近成交
        
        Args:
            symbol: 交易对
            limit: 数量
            
        Returns:
            List[Dict]: 成交记录列表
        """
        return await self._request(
            "GET",
            "/api/v5/market/trades",
            params={
                "instId": symbol,
                "limit": limit
            }
        )
        
    async def get_klines(
        self,
        symbol: str,
        interval: str = "1m",
        limit: int = 100
    ) -> List[List]:
        """获取K线数据
        
        Args:
            symbol: 交易对
            interval: K线间隔
            limit: 数量
            
        Returns:
            List[List]: K线数据列表
        """
        return await self._request(
            "GET",
            "/api/v5/market/candles",
            params={
                "instId": symbol,
                "bar": interval,
                "limit": limit
            }
        )
        
    async def get_account(self) -> Dict:
        """获取账户信息
        
        Returns:
            Dict: 账户信息
        """
        return (await self._request(
            "GET",
            "/api/v5/account/balance",
            auth=True
        ))[0]
        
    async def get_positions(self, symbol: Optional[str] = None) -> List[Dict]:
        """获取持仓信息
        
        Args:
            symbol: 交易对
            
        Returns:
            List[Dict]: 持仓信息列表
        """
        params = {"instId": symbol} if symbol else {}
        return await self._request(
            "GET",
            "/api/v5/account/positions",
            params=params,
            auth=True
        )
        
    async def create_order(
        self,
        symbol: str,
        type: OrderType,
        side: OrderSide,
        price: Optional[Decimal] = None,
        quantity: Decimal = Decimal("0"),
        client_order_id: Optional[str] = None,
        market: Market = Market.SPOT
    ) -> Dict:
        """创建订单
        
        Args:
            symbol: 交易对
            type: 订单类型
            side: 订单方向
            price: 价格
            quantity: 数量
            client_order_id: 客户端订单ID
            market: 市场类型
            
        Returns:
            Dict: 订单信息
        """
        data = {
            "instId": symbol,
            "tdMode": "cash" if market == Market.SPOT else "cross",
            "side": "buy" if side == OrderSide.BUY else "sell",
            "ordType": type.value.lower(),
            "sz": str(quantity)
        }
        
        if price:
            data["px"] = str(price)
            
        if client_order_id:
            data["clOrdId"] = client_order_id
            
        return (await self._request(
            "POST",
            "/api/v5/trade/order",
            data=data,
            auth=True
        ))[0]
        
    async def cancel_order(
        self,
        symbol: str,
        order_id: Optional[str] = None,
        client_order_id: Optional[str] = None
    ) -> Dict:
        """取消订单
        
        Args:
            symbol: 交易对
            order_id: 订单ID
            client_order_id: 客户端订单ID
            
        Returns:
            Dict: 订单信息
        """
        data = {"instId": symbol}
        
        if order_id:
            data["ordId"] = order_id
        elif client_order_id:
            data["clOrdId"] = client_order_id
        else:
            raise ValidationError("order_id和client_order_id不能同时为空")
            
        return (await self._request(
            "POST",
            "/api/v5/trade/cancel-order",
            data=data,
            auth=True
        ))[0]
        
    async def get_order(
        self,
        symbol: str,
        order_id: Optional[str] = None,
        client_order_id: Optional[str] = None
    ) -> Dict:
        """获取订单信息
        
        Args:
            symbol: 交易对
            order_id: 订单ID
            client_order_id: 客户端订单ID
            
        Returns:
            Dict: 订单信息
        """
        params = {"instId": symbol}
        
        if order_id:
            params["ordId"] = order_id
        elif client_order_id:
            params["clOrdId"] = client_order_id
        else:
            raise ValidationError("order_id和client_order_id不能同时为空")
            
        return (await self._request(
            "GET",
            "/api/v5/trade/order",
            params=params,
            auth=True
        ))[0]
        
    async def get_open_orders(
        self,
        symbol: Optional[str] = None,
        limit: int = 100
    ) -> List[Dict]:
        """获取未成交订单
        
        Args:
            symbol: 交易对
            limit: 数量
            
        Returns:
            List[Dict]: 订单列表
        """
        params = {"limit": limit}
        if symbol:
            params["instId"] = symbol
            
        return await self._request(
            "GET",
            "/api/v5/trade/orders-pending",
            params=params,
            auth=True
        ) 