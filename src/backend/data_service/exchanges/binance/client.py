"""
Binance API客户端实现
"""
import hmac
import hashlib
import time
from typing import Dict, List, Optional, Any
from datetime import datetime
from decimal import Decimal
import aiohttp
import ujson
from urllib.parse import urlencode
import asyncio

from ...common.models import (
    Exchange, Market, Symbol, Ticker, OrderBook, Trade,
    Kline, Balance, Order, OrderType, OrderSide, OrderStatus
)
from ...common.exchange import ExchangeAPI
from ...common.decorators import retry
from .websocket import BinanceWebsocketClient
from .handlers import OrderUpdateHandler
from .config import BINANCE_API_URL, BINANCE_WS_URL

class BinanceAPIError(Exception):
    """Binance API错误"""
    pass

def should_retry_error(e: Exception) -> bool:
    """判断是否需要重试
    
    Args:
        e: 异常
        
    Returns:
        bool: 是否需要重试
    """
    if isinstance(e, BinanceAPIError):
        # 需要重试的错误码
        retry_codes = [
            -1000,  # 未知错误
            -1001,  # 断开连接
            -1002,  # 未授权
            -1003,  # 请求过多
            -1006,  # 非常规响应
            -1007,  # 超时
            -1015,  # 请求权重过大
            -1016,  # 服务器维护
            -1020,  # 不支持的操作
            -1021,  # 时间同步问题
            -1022,  # 签名无效
        ]
        
        # 从错误消息中提取错误码
        msg = str(e)
        try:
            code = int(msg.split(":")[0])
            return code in retry_codes
        except:
            return False
    
    # 网络相关错误需要重试
    if isinstance(e, aiohttp.ClientError):
        return True
    
    return False

class BinanceAPI(ExchangeAPI):
    """Binance API实现"""

    def __init__(self, api_key: str = "", api_secret: str = "", testnet: bool = False):
        """初始化Binance API

        Args:
            api_key: API Key
            api_secret: API Secret
            testnet: 是否使用测试网络
        """
        super().__init__(api_key, api_secret, testnet)
        
        # API接口地址
        self.base_url = BINANCE_API_URL["testnet"] if testnet else BINANCE_API_URL["mainnet"]
        self.ws_url = BINANCE_WS_URL["testnet"] if testnet else BINANCE_WS_URL["mainnet"]
        
        # 请求头
        self.headers = {
            "Content-Type": "application/json",
            "X-MBX-APIKEY": api_key
        }

        # 创建WebSocket客户端
        self.ws_client = BinanceWebsocketClient(api_key, api_secret, testnet)
        
        # 创建订单更新处理器
        self.order_handler = OrderUpdateHandler(Market.SPOT)
        
        # 注册订单更新处理器
        self.ws_client.add_handler("executionReport", self.order_handler)

    def _get_timestamp(self) -> int:
        """获取当前时间戳"""
        return int(time.time() * 1000)

    def _generate_signature(self, params: Dict[str, Any]) -> str:
        """生成签名

        Args:
            params: 请求参数

        Returns:
            str: 签名
        """
        # 将所有参数转换为字符串
        str_params = {k: str(v) for k, v in params.items()}
        # 使用urlencode对参数进行编码并按字母顺序排序
        query_string = urlencode(sorted(str_params.items()))
        
        # 使用HMAC SHA256生成签名
        return hmac.new(
            self.api_secret.encode("utf-8"),
            query_string.encode("utf-8"),
            hashlib.sha256
        ).hexdigest()

    @retry(
        max_retries=3,
        retry_delay=1.0,
        max_delay=10.0,
        exponential_base=2.0,
        exceptions=(BinanceAPIError, aiohttp.ClientError),
        should_retry=should_retry_error
    )
    async def _request(
        self,
        method: str,
        endpoint: str,
        params: Optional[Dict[str, Any]] = None,
        signed: bool = False
    ) -> Any:
        """发送HTTP请求

        Args:
            method: 请求方法
            endpoint: 接口地址
            params: 请求参数
            signed: 是否需要签名

        Returns:
            Any: 响应数据

        Raises:
            BinanceAPIError: API调用错误
        """
        url = f"{self.base_url}{endpoint}"
        params = params or {}

        # 如果需要签名，添加时间戳并生成签名
        if signed:
            # 添加时间戳
            params["timestamp"] = str(self._get_timestamp())
            # 将所有参数转换为字符串
            str_params = {k: str(v) for k, v in params.items()}
            # 使用urlencode对参数进行编码并按字母顺序排序
            query_string = urlencode(sorted(str_params.items()))
            # 生成签名
            signature = hmac.new(
                self.api_secret.encode("utf-8"),
                query_string.encode("utf-8"),
                hashlib.sha256
            ).hexdigest()
            # 将签名添加到查询字符串
            query_string = f"{query_string}&signature={signature}"

        try:
            async with aiohttp.ClientSession() as session:
                if method == "GET":
                    if signed:
                        url = f"{url}?{query_string}"
                        async with session.get(url, headers=self.headers) as response:
                            data = await response.json(loads=ujson.loads)
                    else:
                        async with session.get(url, params=params, headers=self.headers) as response:
                            data = await response.json(loads=ujson.loads)
                elif method == "POST":
                    if signed:
                        url = f"{url}?{query_string}"
                        async with session.post(url, headers=self.headers) as response:
                            data = await response.json(loads=ujson.loads)
                    else:
                        async with session.post(url, params=params, headers=self.headers) as response:
                            data = await response.json(loads=ujson.loads)
                elif method == "DELETE":
                    if signed:
                        url = f"{url}?{query_string}"
                        async with session.delete(url, headers=self.headers) as response:
                            data = await response.json(loads=ujson.loads)
                    else:
                        async with session.delete(url, params=params, headers=self.headers) as response:
                            data = await response.json(loads=ujson.loads)

                if response.status >= 400:
                    raise BinanceAPIError(f"API错误: {data.get('msg', str(data))}")

                return data
        except aiohttp.ClientError as e:
            raise BinanceAPIError(f"网络错误: {str(e)}")
        except Exception as e:
            raise BinanceAPIError(f"未知错误: {str(e)}")

    async def get_symbols(self, market: Market) -> List[Symbol]:
        """获取所有交易对信息

        Args:
            market: 市场类型

        Returns:
            List[Symbol]: 交易对列表
        """
        endpoint = "/v3/exchangeInfo"
        response = await self._request("GET", endpoint)

        symbols = []
        for item in response["symbols"]:
            if item["status"] != "TRADING":
                continue

            # 获取价格过滤器
            price_filter = next(f for f in item["filters"] if f["filterType"] == "PRICE_FILTER")
            lot_size = next(f for f in item["filters"] if f["filterType"] == "LOT_SIZE")
            min_notional = next(f for f in item["filters"] if f["filterType"] == "MIN_NOTIONAL")

            symbols.append(Symbol(
                exchange=Exchange.BINANCE,
                market=market,
                base_asset=item["baseAsset"],
                quote_asset=item["quoteAsset"],
                min_price=Decimal(price_filter["minPrice"]),
                max_price=Decimal(price_filter["maxPrice"]),
                tick_size=Decimal(price_filter["tickSize"]),
                min_qty=Decimal(lot_size["minQty"]),
                max_qty=Decimal(lot_size["maxQty"]),
                step_size=Decimal(lot_size["stepSize"]),
                min_notional=Decimal(min_notional["minNotional"]),
                status=item["status"].lower(),
                created_at=datetime.now()
            ))

        return symbols

    async def get_ticker(self, market: Market, symbol: str) -> Ticker:
        """获取行情数据

        Args:
            market: 市场类型
            symbol: 交易对

        Returns:
            Ticker: 行情数据
        """
        endpoint = "/v3/ticker/24hr"
        params = {"symbol": symbol}
        response = await self._request("GET", endpoint, params)

        return Ticker(
            exchange=Exchange.BINANCE,
            market=market,
            symbol=symbol,
            price=Decimal(response["lastPrice"]),
            volume=Decimal(response["volume"]),
            amount=Decimal(response["quoteVolume"]),
            timestamp=datetime.fromtimestamp(response["closeTime"] / 1000),
            bid_price=Decimal(response["bidPrice"]),
            bid_qty=Decimal(response["bidQty"]),
            ask_price=Decimal(response["askPrice"]),
            ask_qty=Decimal(response["askQty"]),
            open_price=Decimal(response["openPrice"]),
            high_price=Decimal(response["highPrice"]),
            low_price=Decimal(response["lowPrice"]),
            close_price=Decimal(response["lastPrice"])
        )

    async def get_order_book(self, market: Market, symbol: str, limit: int = 100) -> OrderBook:
        """获取订单簿数据

        Args:
            market: 市场类型
            symbol: 交易对
            limit: 深度

        Returns:
            OrderBook: 订单簿数据
        """
        endpoint = "/v3/depth"
        params = {
            "symbol": symbol,
            "limit": limit
        }
        response = await self._request("GET", endpoint, params)

        return OrderBook(
            exchange=Exchange.BINANCE,
            market=market,
            symbol=symbol,
            timestamp=datetime.fromtimestamp(response["lastUpdateId"] / 1000),
            bids=[{"price": Decimal(p), "quantity": Decimal(q)} for p, q in response["bids"]],
            asks=[{"price": Decimal(p), "quantity": Decimal(q)} for p, q in response["asks"]],
            update_id=response["lastUpdateId"]
        )

    async def get_recent_trades(self, market: Market, symbol: str, limit: int = 100) -> List[Trade]:
        """获取最近成交

        Args:
            market: 市场类型
            symbol: 交易对
            limit: 数量

        Returns:
            List[Trade]: 成交列表
        """
        endpoint = "/v3/trades"
        params = {
            "symbol": symbol,
            "limit": limit
        }
        response = await self._request("GET", endpoint, params)

        trades = []
        for item in response:
            trades.append(Trade(
                exchange=Exchange.BINANCE,
                market=market,
                symbol=symbol,
                id=str(item["id"]),
                price=Decimal(item["price"]),
                quantity=Decimal(item["qty"]),
                amount=Decimal(item["price"]) * Decimal(item["qty"]),
                timestamp=datetime.fromtimestamp(item["time"] / 1000),
                is_buyer_maker=item["isBuyerMaker"],
                side=OrderSide.SELL if item["isBuyerMaker"] else OrderSide.BUY
            ))

        return trades

    async def get_klines(
        self,
        market: Market,
        symbol: str,
        interval: str,
        start_time: Optional[datetime] = None,
        end_time: Optional[datetime] = None,
        limit: int = 500
    ) -> List[Kline]:
        """获取K线数据

        Args:
            market: 市场类型
            symbol: 交易对
            interval: 时间间隔
            start_time: 开始时间
            end_time: 结束时间
            limit: 数量

        Returns:
            List[Kline]: K线数据列表
        """
        endpoint = "/v3/klines"
        params = {
            "symbol": symbol,
            "interval": interval,
            "limit": limit
        }

        if start_time:
            params["startTime"] = int(start_time.timestamp() * 1000)
        if end_time:
            params["endTime"] = int(end_time.timestamp() * 1000)

        response = await self._request("GET", endpoint, params)

        klines = []
        for item in response:
            klines.append(Kline(
                exchange=Exchange.BINANCE,
                market=market,
                symbol=symbol,
                interval=interval,
                open_time=datetime.fromtimestamp(item[0] / 1000),
                close_time=datetime.fromtimestamp(item[6] / 1000),
                open_price=Decimal(item[1]),
                high_price=Decimal(item[2]),
                low_price=Decimal(item[3]),
                close_price=Decimal(item[4]),
                volume=Decimal(item[5]),
                amount=Decimal(item[7]),
                trades_count=item[8]
            ))

        return klines

    async def get_balances(self) -> List[Balance]:
        """获取账户余额

        Returns:
            List[Balance]: 余额列表
        """
        endpoint = "/v3/account"
        response = await self._request("GET", endpoint, signed=True)

        balances = []
        for item in response["balances"]:
            free = Decimal(item["free"])
            locked = Decimal(item["locked"])
            total = free + locked

            if total == 0:
                continue

            balances.append(Balance(
                exchange=Exchange.BINANCE,
                asset=item["asset"],
                free=free,
                locked=locked,
                total=total,
                timestamp=datetime.fromtimestamp(response["updateTime"] / 1000)
            ))

        return balances

    async def create_order(
        self,
        market: Market,
        symbol: str,
        order_type: OrderType,
        side: OrderSide,
        price: Optional[float] = None,
        quantity: Optional[float] = None,
        client_order_id: Optional[str] = None,
    ) -> Order:
        """创建订单

        Args:
            market: 市场类型
            symbol: 交易对
            order_type: 订单类型
            side: 订单方向
            price: 价格
            quantity: 数量
            client_order_id: 客户端订单ID

        Returns:
            Order: 订单信息
        """
        params = {
            "symbol": symbol,
            "side": side.value.upper(),
            "type": order_type.value.upper(),
            "quantity": quantity
        }

        if order_type == OrderType.LIMIT:
            params["timeInForce"] = "GTC"  # Good Till Cancel
            params["price"] = price
        elif order_type == OrderType.MARKET:
            if price:
                del params["price"]

        if client_order_id:
            params["newClientOrderId"] = client_order_id

        response = await self._request("POST", "/v3/order", params, signed=True)

        return Order(
            exchange=Exchange.BINANCE,
            market=market,
            symbol=symbol,
            id=str(response["orderId"]),
            client_order_id=response["clientOrderId"],
            price=Decimal(response["price"]),
            original_quantity=Decimal(response["origQty"]),
            executed_quantity=Decimal(response["executedQty"]),
            remaining_quantity=Decimal(response["origQty"]) - Decimal(response["executedQty"]),
            status=OrderStatus(response["status"].lower()),
            type=order_type,
            side=side,
            created_at=datetime.fromtimestamp(response["transactTime"] / 1000),
            updated_at=datetime.fromtimestamp(response["transactTime"] / 1000),
            is_working=response["status"] not in ["FILLED", "CANCELED", "REJECTED", "EXPIRED"]
        )

    async def cancel_order(
        self,
        market: Market,
        symbol: str,
        order_id: Optional[str] = None,
        client_order_id: Optional[str] = None
    ) -> Order:
        """取消订单

        Args:
            market: 市场类型
            symbol: 交易对
            order_id: 订单ID
            client_order_id: 客户端订单ID

        Returns:
            Order: 订单信息
        """
        params = {
            "symbol": symbol
        }

        if order_id:
            params["orderId"] = order_id
        elif client_order_id:
            params["origClientOrderId"] = client_order_id
        else:
            raise ValueError("order_id和client_order_id必须指定一个")

        response = await self._request("DELETE", "/v3/order", params, signed=True)

        return Order(
            exchange=Exchange.BINANCE,
            market=market,
            symbol=symbol,
            id=str(response["orderId"]),
            client_order_id=response["clientOrderId"],
            price=Decimal(response["price"]),
            original_quantity=Decimal(response["origQty"]),
            executed_quantity=Decimal(response["executedQty"]),
            remaining_quantity=Decimal(response["origQty"]) - Decimal(response["executedQty"]),
            status=OrderStatus(response["status"].lower()),
            type=OrderType(response["type"].lower()),
            side=OrderSide(response["side"].lower()),
            created_at=datetime.fromtimestamp(response["time"] / 1000),
            updated_at=datetime.fromtimestamp(response["updateTime"] / 1000),
            is_working=response["status"] not in ["FILLED", "CANCELED", "REJECTED", "EXPIRED"]
        )

    async def get_order(
        self,
        market: Market,
        symbol: str,
        order_id: Optional[str] = None,
        client_order_id: Optional[str] = None
    ) -> Order:
        """获取订单信息

        Args:
            market: 市场类型
            symbol: 交易对
            order_id: 订单ID
            client_order_id: 客户端订单ID

        Returns:
            Order: 订单信息
        """
        params = {
            "symbol": symbol
        }

        if order_id:
            params["orderId"] = order_id
        elif client_order_id:
            params["origClientOrderId"] = client_order_id
        else:
            raise ValueError("order_id和client_order_id必须指定一个")

        response = await self._request("GET", "/v3/order", params, signed=True)

        return Order(
            exchange=Exchange.BINANCE,
            market=market,
            symbol=symbol,
            id=str(response["orderId"]),
            client_order_id=response["clientOrderId"],
            price=Decimal(response["price"]),
            original_quantity=Decimal(response["origQty"]),
            executed_quantity=Decimal(response["executedQty"]),
            remaining_quantity=Decimal(response["origQty"]) - Decimal(response["executedQty"]),
            status=OrderStatus(response["status"].lower()),
            type=OrderType(response["type"].lower()),
            side=OrderSide(response["side"].lower()),
            created_at=datetime.fromtimestamp(response["time"] / 1000),
            updated_at=datetime.fromtimestamp(response["updateTime"] / 1000),
            is_working=response["status"] not in ["FILLED", "CANCELED", "REJECTED", "EXPIRED"]
        )

    async def get_open_orders(self, market: Market, symbol: Optional[str] = None) -> List[Order]:
        """获取未完成订单

        Args:
            market: 市场类型
            symbol: 交易对

        Returns:
            List[Order]: 订单列表
        """
        endpoint = "/v3/openOrders"
        params = {}
        if symbol:
            params["symbol"] = symbol

        response = await self._request("GET", endpoint, params, signed=True)

        orders = []
        for item in response:
            orders.append(Order(
                exchange=Exchange.BINANCE,
                market=market,
                symbol=item["symbol"],
                id=str(item["orderId"]),
                client_order_id=item["clientOrderId"],
                price=Decimal(item["price"]),
                original_quantity=Decimal(item["origQty"]),
                executed_quantity=Decimal(item["executedQty"]),
                remaining_quantity=Decimal(item["origQty"]) - Decimal(item["executedQty"]),
                status=OrderStatus(item["status"].lower()),
                type=OrderType(item["type"].lower()),
                side=OrderSide(item["side"].lower()),
                created_at=datetime.fromtimestamp(item["time"] / 1000),
                updated_at=datetime.fromtimestamp(item["updateTime"] / 1000),
                is_working=True
            ))

        return orders

    async def get_order_trades(self, market: Market, symbol: str, order_id: str) -> List[Trade]:
        """获取订单成交记录

        Args:
            market: 市场类型
            symbol: 交易对
            order_id: 订单ID

        Returns:
            List[Trade]: 成交记录列表
        """
        endpoint = "/v3/myTrades"
        params = {
            "symbol": symbol,
            "orderId": order_id
        }

        response = await self._request("GET", endpoint, params, signed=True)

        trades = []
        for item in response:
            trades.append(Trade(
                exchange=Exchange.BINANCE,
                market=market,
                symbol=symbol,
                id=str(item["id"]),
                price=Decimal(item["price"]),
                quantity=Decimal(item["qty"]),
                amount=Decimal(item["quoteQty"]),
                timestamp=datetime.fromtimestamp(item["time"] / 1000),
                is_buyer_maker=item["isBuyer"],
                side=OrderSide.BUY if item["isBuyer"] else OrderSide.SELL
            ))

        return trades

    async def start(self):
        """启动API客户端"""
        # 启动WebSocket客户端
        await self.ws_client.start()

    async def stop(self):
        """停止API客户端"""
        # 停止WebSocket客户端
        await self.ws_client.stop() 