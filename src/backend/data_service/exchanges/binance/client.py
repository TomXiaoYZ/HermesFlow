"""
Binance API客户端
"""
import json
import logging
import time
from typing import Optional, List, Dict, Any
import requests
from ...common.exceptions import (
    APIError,
    NetworkError,
    ValidationError,
    AuthenticationError,
    PermissionError,
    RateLimitError,
    OrderError,
    PositionError
)
from ...common.models import (
    ContractInfo,
    FundingRate,
    Ticker,
    Kline,
    OrderBook,
    Trade,
    ContractOrder,
    PositionInfo,
    OrderSide,
    OrderType,
    PositionSide,
    TimeInForce,
    MarginType
)

logger = logging.getLogger(__name__)

class BinanceAPI:
    """Binance API客户端"""
    
    def __init__(
        self,
        api_key: Optional[str] = None,
        api_secret: Optional[str] = None,
        testnet: bool = False
    ):
        """初始化Binance API客户端
        
        Args:
            api_key: API Key
            api_secret: API Secret
            testnet: 是否使用测试网
        """
        self.api_key = api_key
        self.api_secret = api_secret
        self.testnet = testnet
        
        # API基础URL
        self.base_url = "https://testnet.binancefuture.com" if testnet else "https://fapi.binance.com"
        
    def _request(
        self,
        method: str,
        endpoint: str,
        params: Optional[Dict[str, Any]] = None,
        data: Optional[Dict[str, Any]] = None,
        headers: Optional[Dict[str, str]] = None,
        auth_required: bool = False,
        timeout: int = 10
    ) -> Dict[str, Any]:
        """发送HTTP请求
        
        Args:
            method: 请求方法
            endpoint: API端点
            params: URL参数
            data: 请求数据
            headers: 请求头
            auth_required: 是否需要认证
            timeout: 超时时间(秒)
            
        Returns:
            响应数据
            
        Raises:
            NetworkError: 网络错误
            APIError: API错误
            AuthenticationError: 认证错误
            PermissionError: 权限错误
            RateLimitError: 频率限制错误
        """
        url = f"{self.base_url}{endpoint}"
        
        # 添加认证头
        if auth_required:
            if not self.api_key or not self.api_secret:
                raise AuthenticationError("API Key和Secret未配置")
            headers = headers or {}
            headers["X-MBX-APIKEY"] = self.api_key
            
        try:
            logger.debug(f"发送请求: {method} {url}")
            logger.debug(f"参数: {params}")
            logger.debug(f"数据: {data}")
            
            response = requests.request(
                method=method,
                url=url,
                params=params,
                json=data,
                headers=headers,
                timeout=timeout
            )
            
            logger.debug(f"响应状态码: {response.status_code}")
            logger.debug(f"响应内容: {response.text}")
            
            if response.status_code == 200:
                return response.json()
                
            error_data = response.json()
            error_code = error_data.get("code", 0)
            error_msg = error_data.get("msg", "Unknown error")
            
            if response.status_code == 401:
                raise AuthenticationError(f"认证失败: {error_msg}")
            elif response.status_code == 403:
                raise PermissionError(f"权限不足: {error_msg}")
            elif response.status_code == 429:
                raise RateLimitError(f"请求频率超限: {error_msg}")
            else:
                raise APIError(
                    message=f"API错误: {error_msg}",
                    exchange="binance",
                    code=str(error_code),
                    http_status=response.status_code,
                    details=error_data
                )
                
        except requests.exceptions.Timeout:
            raise NetworkError("请求超时")
        except requests.exceptions.ConnectionError:
            raise NetworkError("网络连接错误")
        except json.JSONDecodeError:
            raise APIError(
                message="响应解析失败",
                exchange="binance",
                code="INVALID_RESPONSE"
            )
        except Exception as e:
            raise APIError(
                message=f"未知错误: {str(e)}",
                exchange="binance",
                code="UNKNOWN_ERROR"
            )
            
    def get_contract_info(self, symbol: Optional[str] = None) -> List[ContractInfo]:
        """获取合约信息
        
        Args:
            symbol: 交易对,如果不指定则返回所有交易对
            
        Returns:
            合约信息列表
        """
        try:
            response = self._request(
                method="GET",
                endpoint="/fapi/v1/exchangeInfo",
                params={"symbol": symbol} if symbol else None
            )
            
            symbols = response["symbols"]
            return [
                ContractInfo(
                    exchange="binance",
                    symbol=s["symbol"],
                    base_asset=s["baseAsset"],
                    quote_asset=s["quoteAsset"],
                    price_precision=s["pricePrecision"],
                    quantity_precision=s["quantityPrecision"],
                    min_price=float(s["filters"][0]["minPrice"]),
                    max_price=float(s["filters"][0]["maxPrice"]),
                    tick_size=float(s["filters"][0]["tickSize"]),
                    min_qty=float(s["filters"][1]["minQty"]),
                    max_qty=float(s["filters"][1]["maxQty"]),
                    step_size=float(s["filters"][1]["stepSize"]),
                    min_notional=float(s["filters"][5]["notional"]),
                    status=s["status"],
                    created_time=None,
                    updated_time=None
                )
                for s in symbols
                if symbol is None or s["symbol"] == symbol
            ]
        except (KeyError, ValueError) as e:
            raise APIError(
                message=f"解析合约信息失败: {str(e)}",
                exchange="binance",
                code="PARSE_ERROR"
            )
            
    def get_funding_rate(self, symbol: str) -> FundingRate:
        """获取资金费率
        
        Args:
            symbol: 交易对
            
        Returns:
            资金费率信息
        """
        try:
            response = self._request(
                method="GET",
                endpoint="/fapi/v1/premiumIndex",
                params={"symbol": symbol}
            )
            
            return FundingRate(
                exchange="binance",
                symbol=response["symbol"],
                funding_rate=float(response["lastFundingRate"]),
                estimated_rate=float(response["lastFundingRate"]),
                next_timestamp=response["nextFundingTime"],
                timestamp=response["time"]
            )
        except (KeyError, ValueError) as e:
            raise APIError(
                message=f"解析资金费率失败: {str(e)}",
                exchange="binance",
                code="PARSE_ERROR"
            )
            
    def get_contract_ticker(self, symbol: str) -> Ticker:
        """获取24小时价格变动
        
        Args:
            symbol: 交易对
            
        Returns:
            24小时价格变动信息
        """
        try:
            response = self._request(
                method="GET",
                endpoint="/fapi/v1/ticker/24hr",
                params={"symbol": symbol}
            )
            
            print("响应数据:", json.dumps(response, indent=2))
            
            return Ticker(
                exchange="binance",
                symbol=response["symbol"],
                last_price=float(response["lastPrice"]),
                last_qty=float(response["lastQty"]),
                open_price=float(response["openPrice"]),
                high_price=float(response["highPrice"]),
                low_price=float(response["lowPrice"]),
                volume=float(response["volume"]),
                quote_volume=float(response["quoteVolume"]),
                open_time=response["openTime"],
                close_time=response["closeTime"],
                first_trade_id=response["firstId"],
                last_trade_id=response["lastId"],
                trade_count=response["count"]
            )
        except (KeyError, ValueError) as e:
            raise APIError(
                message=f"解析行情数据失败: {str(e)}",
                exchange="binance",
                code="PARSE_ERROR"
            )
            
    def get_contract_klines(
        self,
        symbol: str,
        interval: str,
        start_time: Optional[int] = None,
        end_time: Optional[int] = None,
        limit: int = 500
    ) -> List[Kline]:
        """获取K线数据
        
        Args:
            symbol: 交易对
            interval: K线间隔
            start_time: 开始时间(毫秒时间戳)
            end_time: 结束时间(毫秒时间戳)
            limit: 返回记录数量
            
        Returns:
            K线数据列表
        """
        try:
            params = {
                "symbol": symbol,
                "interval": interval,
                "limit": limit
            }
            if start_time:
                params["startTime"] = start_time
            if end_time:
                params["endTime"] = end_time
                
            response = self._request(
                method="GET",
                endpoint="/fapi/v1/klines",
                params=params
            )
            
            return [
                Kline(
                    exchange="binance",
                    symbol=symbol,
                    interval=interval,
                    open_time=k[0],
                    open_price=float(k[1]),
                    high_price=float(k[2]),
                    low_price=float(k[3]),
                    close_price=float(k[4]),
                    volume=float(k[5]),
                    close_time=k[6],
                    quote_volume=float(k[7]),
                    trade_count=k[8],
                    taker_buy_volume=float(k[9]),
                    taker_buy_quote_volume=float(k[10]),
                    ignore=k[11]
                )
                for k in response
            ]
        except (KeyError, ValueError, IndexError) as e:
            raise APIError(
                message=f"解析K线数据失败: {str(e)}",
                exchange="binance",
                code="PARSE_ERROR"
            )
            
    def get_contract_depth(
        self,
        symbol: str,
        limit: int = 100
    ) -> OrderBook:
        """获取深度数据
        
        Args:
            symbol: 交易对
            limit: 返回记录数量
            
        Returns:
            深度数据
        """
        try:
            response = self._request(
                method="GET",
                endpoint="/fapi/v1/depth",
                params={
                    "symbol": symbol,
                    "limit": limit
                }
            )
            
            return OrderBook(
                exchange="binance",
                symbol=symbol,
                bids=[
                    [float(price), float(qty)]
                    for price, qty in response["bids"]
                ],
                asks=[
                    [float(price), float(qty)]
                    for price, qty in response["asks"]
                ],
                timestamp=response["T"] if "T" in response else int(time.time() * 1000)
            )
        except (KeyError, ValueError) as e:
            raise APIError(
                message=f"解析深度数据失败: {str(e)}",
                exchange="binance",
                code="PARSE_ERROR"
            )
            
    def get_recent_trades(
        self,
        symbol: str,
        limit: int = 500
    ) -> List[Trade]:
        """获取最近成交
        
        Args:
            symbol: 交易对
            limit: 返回记录数量
            
        Returns:
            成交记录列表
        """
        try:
            response = self._request(
                method="GET",
                endpoint="/fapi/v1/trades",
                params={
                    "symbol": symbol,
                    "limit": limit
                }
            )
            
            return [
                Trade(
                    exchange="binance",
                    symbol=symbol,
                    id=t["id"],
                    price=float(t["price"]),
                    qty=float(t["qty"]),
                    quote_qty=float(t["price"]) * float(t["qty"]),
                    time=t["time"],
                    is_buyer_maker=t["isBuyerMaker"]
                )
                for t in response
            ]
        except (KeyError, ValueError) as e:
            raise APIError(
                message=f"解析成交数据失败: {str(e)}",
                exchange="binance",
                code="PARSE_ERROR"
            )
            
    def change_leverage(
        self,
        symbol: str,
        leverage: int
    ) -> Dict[str, Any]:
        """调整杠杆倍数
        
        Args:
            symbol: 交易对
            leverage: 杠杆倍数
            
        Returns:
            响应数据
        """
        if not isinstance(leverage, int) or leverage < 1 or leverage > 125:
            raise ValidationError("杠杆倍数必须是1-125之间的整数")
            
        return self._request(
            method="POST",
            endpoint="/fapi/v1/leverage",
            data={
                "symbol": symbol,
                "leverage": leverage
            },
            auth_required=True
        )
        
    def change_margin_type(
        self,
        symbol: str,
        margin_type: MarginType
    ) -> Dict[str, Any]:
        """调整保证金类型
        
        Args:
            symbol: 交易对
            margin_type: 保证金类型
            
        Returns:
            响应数据
        """
        return self._request(
            method="POST",
            endpoint="/fapi/v1/marginType",
            data={
                "symbol": symbol,
                "marginType": margin_type.value
            },
            auth_required=True
        )
        
    def get_position_info(
        self,
        symbol: Optional[str] = None
    ) -> List[PositionInfo]:
        """获取持仓信息
        
        Args:
            symbol: 交易对,如果不指定则返回所有持仓
            
        Returns:
            持仓信息列表
        """
        try:
            response = self._request(
                method="GET",
                endpoint="/fapi/v2/positionRisk",
                params={"symbol": symbol} if symbol else None,
                auth_required=True
            )
            
            return [
                PositionInfo(
                    exchange="binance",
                    symbol=p["symbol"],
                    position_side=PositionSide(p["positionSide"]),
                    margin_type=MarginType(p["marginType"]),
                    isolated_margin=float(p["isolatedMargin"]),
                    leverage=int(p["leverage"]),
                    position_amt=float(p["positionAmt"]),
                    entry_price=float(p["entryPrice"]),
                    mark_price=float(p["markPrice"]),
                    unreal_profit=float(p["unRealizedProfit"]),
                    liquidation_price=float(p["liquidationPrice"]),
                    created_time=None,
                    updated_time=p["updateTime"]
                )
                for p in response
                if float(p["positionAmt"]) != 0
            ]
        except (KeyError, ValueError) as e:
            raise APIError(
                message=f"解析持仓信息失败: {str(e)}",
                exchange="binance",
                code="PARSE_ERROR"
            )
            
    def create_contract_order(
        self,
        symbol: str,
        side: OrderSide,
        position_side: PositionSide,
        order_type: OrderType,
        quantity: float,
        price: Optional[float] = None,
        stop_price: Optional[float] = None,
        time_in_force: TimeInForce = TimeInForce.GTC,
        reduce_only: bool = False,
        working_type: str = "CONTRACT_PRICE",
        client_order_id: Optional[str] = None
    ) -> ContractOrder:
        """创建合约订单
        
        Args:
            symbol: 交易对
            side: 订单方向
            position_side: 持仓方向
            order_type: 订单类型
            quantity: 数量
            price: 价格(限价单必填)
            stop_price: 触发价格(止损/止盈单必填)
            time_in_force: 有效方式
            reduce_only: 是否只减仓
            working_type: 触发价格类型
            client_order_id: 客户端订单ID
            
        Returns:
            订单信息
        """
        try:
            # 验证参数
            if order_type in [OrderType.LIMIT, OrderType.STOP, OrderType.TAKE_PROFIT] and price is None:
                raise ValidationError("限价单必须指定价格")
                
            if order_type in [OrderType.STOP, OrderType.TAKE_PROFIT] and stop_price is None:
                raise ValidationError("止损/止盈单必须指定触发价格")
                
            data = {
                "symbol": symbol,
                "side": side.value,
                "positionSide": position_side.value,
                "type": order_type.value,
                "quantity": quantity,
                "timeInForce": time_in_force.value,
                "reduceOnly": reduce_only,
                "workingType": working_type
            }
            
            if price is not None:
                data["price"] = price
                
            if stop_price is not None:
                data["stopPrice"] = stop_price
                
            if client_order_id is not None:
                data["newClientOrderId"] = client_order_id
                
            response = self._request(
                method="POST",
                endpoint="/fapi/v1/order",
                data=data,
                auth_required=True
            )
            
            return ContractOrder(
                exchange="binance",
                symbol=response["symbol"],
                order_id=str(response["orderId"]),
                client_order_id=response.get("clientOrderId"),
                price=float(response["price"]),
                avg_price=0.0,
                stop_price=float(response.get("stopPrice", 0)),
                quantity=float(response["origQty"]),
                executed_qty=float(response["executedQty"]),
                status=response["status"],
                time_in_force=TimeInForce(response["timeInForce"]),
                type=OrderType(response["type"]),
                side=OrderSide(response["side"]),
                position_side=PositionSide(response["positionSide"]),
                reduce_only=response["reduceOnly"],
                working_type=response["workingType"],
                created_time=response["time"],
                updated_time=response["updateTime"]
            )
        except (KeyError, ValueError) as e:
            raise APIError(
                message=f"解析订单数据失败: {str(e)}",
                exchange="binance",
                code="PARSE_ERROR"
            )
            
    def cancel_contract_order(
        self,
        symbol: str,
        order_id: Optional[str] = None,
        client_order_id: Optional[str] = None
    ) -> ContractOrder:
        """撤销合约订单
        
        Args:
            symbol: 交易对
            order_id: 订单ID
            client_order_id: 客户端订单ID
            
        Returns:
            订单信息
        """
        try:
            if order_id is None and client_order_id is None:
                raise ValidationError("订单ID和客户端订单ID不能同时为空")
                
            data = {"symbol": symbol}
            if order_id is not None:
                data["orderId"] = order_id
            if client_order_id is not None:
                data["origClientOrderId"] = client_order_id
                
            response = self._request(
                method="DELETE",
                endpoint="/fapi/v1/order",
                data=data,
                auth_required=True
            )
            
            return ContractOrder(
                exchange="binance",
                symbol=response["symbol"],
                order_id=str(response["orderId"]),
                client_order_id=response.get("clientOrderId"),
                price=float(response["price"]),
                avg_price=0.0,
                stop_price=float(response.get("stopPrice", 0)),
                quantity=float(response["origQty"]),
                executed_qty=float(response["executedQty"]),
                status=response["status"],
                time_in_force=TimeInForce(response["timeInForce"]),
                type=OrderType(response["type"]),
                side=OrderSide(response["side"]),
                position_side=PositionSide(response["positionSide"]),
                reduce_only=response["reduceOnly"],
                working_type=response["workingType"],
                created_time=response["time"],
                updated_time=response["updateTime"]
            )
        except (KeyError, ValueError) as e:
            raise APIError(
                message=f"解析订单数据失败: {str(e)}",
                exchange="binance",
                code="PARSE_ERROR"
            )
            
    def get_contract_order(
        self,
        symbol: str,
        order_id: Optional[str] = None,
        client_order_id: Optional[str] = None
    ) -> ContractOrder:
        """查询合约订单
        
        Args:
            symbol: 交易对
            order_id: 订单ID
            client_order_id: 客户端订单ID
            
        Returns:
            订单信息
        """
        try:
            if order_id is None and client_order_id is None:
                raise ValidationError("订单ID和客户端订单ID不能同时为空")
                
            params = {"symbol": symbol}
            if order_id is not None:
                params["orderId"] = order_id
            if client_order_id is not None:
                params["origClientOrderId"] = client_order_id
                
            response = self._request(
                method="GET",
                endpoint="/fapi/v1/order",
                params=params,
                auth_required=True
            )
            
            return ContractOrder(
                exchange="binance",
                symbol=response["symbol"],
                order_id=str(response["orderId"]),
                client_order_id=response.get("clientOrderId"),
                price=float(response["price"]),
                avg_price=float(response["avgPrice"]),
                stop_price=float(response.get("stopPrice", 0)),
                quantity=float(response["origQty"]),
                executed_qty=float(response["executedQty"]),
                status=response["status"],
                time_in_force=TimeInForce(response["timeInForce"]),
                type=OrderType(response["type"]),
                side=OrderSide(response["side"]),
                position_side=PositionSide(response["positionSide"]),
                reduce_only=response["reduceOnly"],
                working_type=response["workingType"],
                created_time=response["time"],
                updated_time=response["updateTime"]
            )
        except (KeyError, ValueError) as e:
            raise APIError(
                message=f"解析订单数据失败: {str(e)}",
                exchange="binance",
                code="PARSE_ERROR"
            )
            
    def get_open_contract_orders(
        self,
        symbol: Optional[str] = None
    ) -> List[ContractOrder]:
        """查询当前挂单
        
        Args:
            symbol: 交易对,如果不指定则返回所有挂单
            
        Returns:
            订单列表
        """
        try:
            response = self._request(
                method="GET",
                endpoint="/fapi/v1/openOrders",
                params={"symbol": symbol} if symbol else None,
                auth_required=True
            )
            
            return [
                ContractOrder(
                    exchange="binance",
                    symbol=o["symbol"],
                    order_id=str(o["orderId"]),
                    client_order_id=o.get("clientOrderId"),
                    price=float(o["price"]),
                    avg_price=float(o["avgPrice"]),
                    stop_price=float(o.get("stopPrice", 0)),
                    quantity=float(o["origQty"]),
                    executed_qty=float(o["executedQty"]),
                    status=o["status"],
                    time_in_force=TimeInForce(o["timeInForce"]),
                    type=OrderType(o["type"]),
                    side=OrderSide(o["side"]),
                    position_side=PositionSide(o["positionSide"]),
                    reduce_only=o["reduceOnly"],
                    working_type=o["workingType"],
                    created_time=o["time"],
                    updated_time=o["updateTime"]
                )
                for o in response
            ]
        except (KeyError, ValueError) as e:
            raise APIError(
                message=f"解析订单数据失败: {str(e)}",
                exchange="binance",
                code="PARSE_ERROR"
            ) 