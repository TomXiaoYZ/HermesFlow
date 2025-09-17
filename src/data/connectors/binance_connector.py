# -*- coding: utf-8 -*-
"""
Binance 交易所数据连接器
提供Binance现货和期货市场的实时和历史数据接入
"""

import asyncio
import json
import hmac
import hashlib
import time
from typing import Dict, List, Optional, Any
from datetime import datetime, timedelta
import aiohttp
import websockets
from urllib.parse import urlencode

from .base_connector import BaseConnector, DataPoint, DataType, ConnectionStatus, ConnectionConfig


class BinanceConnector(BaseConnector):
    """Binance交易所数据连接器"""
    
    # 生产环境API端点
    SPOT_BASE_URL = "https://api.binance.com"
    FUTURES_BASE_URL = "https://fapi.binance.com"
    SPOT_WS_URL = "wss://stream.binance.com:9443/ws/"
    FUTURES_WS_URL = "wss://fstream.binance.com/ws/"
    
    # 测试环境API端点
    TESTNET_SPOT_BASE_URL = "https://testnet.binance.vision"
    TESTNET_FUTURES_BASE_URL = "https://testnet.binancefuture.com"
    TESTNET_SPOT_WS_URL = "wss://testnet.binance.vision/ws/"
    TESTNET_FUTURES_WS_URL = "wss://stream.binancefuture.com/ws/"
    
    # 时间间隔映射
    INTERVAL_MAP = {
        "1m": "1m", "3m": "3m", "5m": "5m", "15m": "15m", "30m": "30m",
        "1h": "1h", "2h": "2h", "4h": "4h", "6h": "6h", "8h": "8h", "12h": "12h",
        "1d": "1d", "3d": "3d", "1w": "1w", "1M": "1M"
    }
    
    def __init__(self, config: ConnectionConfig, market_type: str = "spot"):
        """
        初始化Binance连接器
        
        Args:
            config: 连接配置
            market_type: 市场类型 ("spot" 或 "futures")
        """
        super().__init__(config, f"binance_{market_type}")
        self.market_type = market_type
        
        # 根据配置选择API端点
        if config.testnet:
            self.base_url = self.TESTNET_SPOT_BASE_URL if market_type == "spot" else self.TESTNET_FUTURES_BASE_URL
            self.ws_url = self.TESTNET_SPOT_WS_URL if market_type == "spot" else self.TESTNET_FUTURES_WS_URL
            self.logger.info(f"使用Binance测试网环境: {self.base_url}")
        else:
            self.base_url = self.SPOT_BASE_URL if market_type == "spot" else self.FUTURES_BASE_URL
            self.ws_url = self.SPOT_WS_URL if market_type == "spot" else self.FUTURES_WS_URL
            self.logger.info(f"使用Binance生产环境: {self.base_url}")
            
        self._listen_key = None
        self._ws_tasks = []
    
    async def connect(self) -> bool:
        """建立连接"""
        try:
            self.status = ConnectionStatus.CONNECTING
            self.logger.info(f"正在连接到 Binance {self.market_type} 市场...")
            
            # 预检查网络连通性
            await self._check_network_connectivity()
            
            # 创建HTTP会话
            connector = aiohttp.TCPConnector(
                ssl=True,
                limit=100,
                limit_per_host=30,
                ttl_dns_cache=300,
                use_dns_cache=True,
            )
            
            self._session = aiohttp.ClientSession(
                connector=connector,
                timeout=aiohttp.ClientTimeout(
                    total=self.config.timeout,
                    connect=10,
                    sock_connect=10,
                    sock_read=10
                ),
                headers={
                    'User-Agent': 'HermesFlow/1.0.0',
                    'Accept': 'application/json',
                    'Content-Type': 'application/json'
                }
            )
            
            # 测试API连接
            await self._test_connection()
            
            self.status = ConnectionStatus.CONNECTED
            self.logger.info(f"成功连接到 Binance {self.market_type} 市场")
            return True
            
        except aiohttp.ClientError as e:
            self.status = ConnectionStatus.ERROR
            error_msg = f"网络连接错误: {str(e)}"
            self.logger.error(error_msg)
            await self._cleanup_session()
            return False
        except asyncio.TimeoutError as e:
            self.status = ConnectionStatus.ERROR
            error_msg = f"连接超时: {str(e)}"
            self.logger.error(error_msg)
            await self._cleanup_session()
            return False
        except Exception as e:
            self.status = ConnectionStatus.ERROR
            error_msg = f"连接失败: {type(e).__name__}: {str(e)}"
            self.logger.error(error_msg)
            await self._cleanup_session()
            return False
    
    async def disconnect(self) -> bool:
        """断开连接"""
        try:
            self.logger.info("正在断开Binance连接...")
            self.status = ConnectionStatus.DISCONNECTED
            
            # 关闭WebSocket连接
            await self._cleanup_websockets()
            
            # 关闭HTTP会话
            await self._cleanup_session()
            
            self.logger.info("已断开Binance连接")
            return True
            
        except Exception as e:
            self.logger.error(f"断开连接失败: {type(e).__name__}: {str(e)}")
            # 即使出错也要尝试清理
            try:
                await self._cleanup_websockets()
                await self._cleanup_session()
            except:
                pass
            return False
    
    async def get_symbols(self) -> List[str]:
        """获取支持的交易对列表"""
        try:
            endpoint = "/api/v3/exchangeInfo" if self.market_type == "spot" else "/fapi/v1/exchangeInfo"
            response = await self._make_request("GET", endpoint)
            
            symbols = []
            for symbol_info in response.get("symbols", []):
                if symbol_info.get("status") == "TRADING":
                    symbols.append(symbol_info["symbol"])
            
            return symbols
            
        except Exception as e:
            self.logger.error(f"获取交易对列表失败: {str(e)}")
            return []
    
    async def get_klines(
        self, 
        symbol: str, 
        interval: str, 
        start_time: Optional[datetime] = None,
        end_time: Optional[datetime] = None,
        limit: int = 500
    ) -> List[DataPoint]:
        """获取K线数据"""
        try:
            # 验证时间间隔
            if interval not in self.INTERVAL_MAP:
                raise ValueError(f"不支持的时间间隔: {interval}")
            
            # 构建参数
            params = {
                "symbol": symbol.upper(),
                "interval": self.INTERVAL_MAP[interval],
                "limit": min(limit, 1000)  # Binance限制最大1000条
            }
            
            if start_time:
                params["startTime"] = int(start_time.timestamp() * 1000)
            if end_time:
                params["endTime"] = int(end_time.timestamp() * 1000)
            
            # 发送请求
            endpoint = "/api/v3/klines" if self.market_type == "spot" else "/fapi/v1/klines"
            response = await self._make_request("GET", endpoint, params)
            
            # 解析数据
            data_points = []
            for kline in response:
                data_point = self._create_data_point(
                    symbol=symbol,
                    data_type=DataType.KLINE,
                    data={
                        "open_time": datetime.fromtimestamp(kline[0] / 1000),
                        "close_time": datetime.fromtimestamp(kline[6] / 1000),
                        "open": float(kline[1]),
                        "high": float(kline[2]),
                        "low": float(kline[3]),
                        "close": float(kline[4]),
                        "volume": float(kline[5]),
                        "quote_volume": float(kline[7]),
                        "trades": int(kline[8]),
                        "taker_buy_base_volume": float(kline[9]),
                        "taker_buy_quote_volume": float(kline[10]),
                        "interval": interval
                    },
                    timestamp=datetime.fromtimestamp(kline[0] / 1000),
                    raw_data=kline
                )
                data_points.append(data_point)
            
            return data_points
            
        except Exception as e:
            self.logger.error(f"获取K线数据失败: {str(e)}")
            return []
    
    async def get_ticker(self, symbol: str) -> Optional[DataPoint]:
        """获取行情数据"""
        try:
            params = {"symbol": symbol.upper()}
            endpoint = "/api/v3/ticker/24hr" if self.market_type == "spot" else "/fapi/v1/ticker/24hr"
            response = await self._make_request("GET", endpoint, params)
            
            return self._create_data_point(
                symbol=symbol,
                data_type=DataType.TICKER,
                data={
                    "price_change": float(response["priceChange"]),
                    "price_change_percent": float(response["priceChangePercent"]),
                    "weighted_avg_price": float(response["weightedAvgPrice"]),
                    "prev_close_price": float(response["prevClosePrice"]),
                    "last_price": float(response["lastPrice"]),
                    "last_qty": float(response["lastQty"]),
                    "bid_price": float(response["bidPrice"]),
                    "bid_qty": float(response["bidQty"]),
                    "ask_price": float(response["askPrice"]),
                    "ask_qty": float(response["askQty"]),
                    "open_price": float(response["openPrice"]),
                    "high_price": float(response["highPrice"]),
                    "low_price": float(response["lowPrice"]),
                    "volume": float(response["volume"]),
                    "quote_volume": float(response["quoteVolume"]),
                    "open_time": datetime.fromtimestamp(response["openTime"] / 1000),
                    "close_time": datetime.fromtimestamp(response["closeTime"] / 1000),
                    "count": int(response["count"])
                },
                timestamp=datetime.fromtimestamp(response["closeTime"] / 1000),
                raw_data=response
            )
            
        except Exception as e:
            self.logger.error(f"获取行情数据失败: {str(e)}")
            return None
    
    async def get_orderbook(self, symbol: str, depth: int = 20) -> Optional[DataPoint]:
        """获取订单簿数据"""
        try:
            params = {
                "symbol": symbol.upper(),
                "limit": min(depth, 5000)  # Binance限制
            }
            
            endpoint = "/api/v3/depth" if self.market_type == "spot" else "/fapi/v1/depth"
            response = await self._make_request("GET", endpoint, params)
            
            return self._create_data_point(
                symbol=symbol,
                data_type=DataType.ORDERBOOK,
                data={
                    "last_update_id": response["lastUpdateId"],
                    "bids": [[float(price), float(qty)] for price, qty in response["bids"]],
                    "asks": [[float(price), float(qty)] for price, qty in response["asks"]],
                    "depth": depth
                },
                raw_data=response
            )
            
        except Exception as e:
            self.logger.error(f"获取订单簿数据失败: {str(e)}")
            return None
    
    async def subscribe_real_time(
        self, 
        symbols: List[str], 
        data_types: List[DataType],
        callback: callable
    ) -> bool:
        """订阅实时数据"""
        try:
            # 构建订阅流
            streams = []
            for symbol in symbols:
                symbol_lower = symbol.lower()
                for data_type in data_types:
                    if data_type == DataType.TICKER:
                        streams.append(f"{symbol_lower}@ticker")
                    elif data_type == DataType.KLINE:
                        streams.append(f"{symbol_lower}@kline_1m")  # 默认1分钟
                    elif data_type == DataType.ORDERBOOK:
                        streams.append(f"{symbol_lower}@depth20@100ms")
                    elif data_type == DataType.TRADE:
                        streams.append(f"{symbol_lower}@trade")
            
            if not streams:
                return False
            
            # 创建WebSocket连接
            ws_url = f"{self.ws_url}{'/'.join(streams)}"
            task = asyncio.create_task(self._websocket_handler(ws_url, callback))
            self._ws_tasks.append(task)
            
            # 记录订阅
            for symbol in symbols:
                for data_type in data_types:
                    self._subscriptions.add(f"{symbol}:{data_type.value}")
            
            self.logger.info(f"成功订阅实时数据: {len(streams)} 个流")
            return True
            
        except Exception as e:
            self.logger.error(f"订阅实时数据失败: {str(e)}")
            return False
    
    async def unsubscribe_real_time(
        self, 
        symbols: List[str], 
        data_types: List[DataType]
    ) -> bool:
        """取消订阅实时数据"""
        try:
            # 移除订阅记录
            for symbol in symbols:
                for data_type in data_types:
                    subscription = f"{symbol}:{data_type.value}"
                    self._subscriptions.discard(subscription)
            
            # 如果没有订阅了，关闭WebSocket
            if not self._subscriptions:
                for task in self._ws_tasks:
                    if not task.done():
                        task.cancel()
                self._ws_tasks.clear()
            
            return True
            
        except Exception as e:
            self.logger.error(f"取消订阅失败: {str(e)}")
            return False
    
    async def _test_connection(self):
        """测试连接"""
        try:
            endpoint = "/api/v3/ping" if self.market_type == "spot" else "/fapi/v1/ping"
            self.logger.info(f"测试API连接: {self.base_url}{endpoint}")
            
            start_time = time.time()
            response = await self._make_request("GET", endpoint)
            latency = (time.time() - start_time) * 1000
            
            self.logger.info(f"API连接测试成功，延迟: {latency:.2f}ms")
            
            # 额外测试：获取服务器时间
            time_endpoint = "/api/v3/time" if self.market_type == "spot" else "/fapi/v1/time"
            time_response = await self._make_request("GET", time_endpoint)
            server_time = time_response.get("serverTime", 0)
            local_time = int(time.time() * 1000)
            time_diff = abs(server_time - local_time)
            
            self.logger.info(f"服务器时间同步检查，时差: {time_diff}ms")
            
            if time_diff > 5000:  # 5秒时差警告
                self.logger.warning(f"服务器时间差异较大: {time_diff}ms")
            
        except Exception as e:
            error_msg = f"API连接测试失败: {type(e).__name__}: {str(e)}"
            self.logger.error(error_msg)
            raise Exception(error_msg)
    
    async def _make_request(
        self, 
        method: str, 
        endpoint: str, 
        params: Optional[Dict] = None,
        signed: bool = False
    ) -> Dict[str, Any]:
        """发送HTTP请求"""
        await self._handle_rate_limit()
        
        url = f"{self.base_url}{endpoint}"
        headers = {"X-MBX-APIKEY": self.config.api_key} if self.config.api_key else {}
        
        if params is None:
            params = {}
        
        # 签名请求
        if signed and self.config.api_secret:
            params["timestamp"] = int(time.time() * 1000)
            query_string = urlencode(params)
            signature = hmac.new(
                self.config.api_secret.encode(),
                query_string.encode(),
                hashlib.sha256
            ).hexdigest()
            params["signature"] = signature
        
        async with self._session.request(method, url, params=params, headers=headers) as response:
            if response.status == 200:
                return await response.json()
            else:
                error_text = await response.text()
                raise Exception(f"API请求失败: {response.status} - {error_text}")
    
    async def _websocket_handler(self, ws_url: str, callback: callable):
        """WebSocket处理器"""
        try:
            async with websockets.connect(ws_url) as websocket:
                self.logger.info(f"WebSocket连接已建立: {ws_url}")
                
                async for message in websocket:
                    try:
                        data = json.loads(message)
                        data_point = self._parse_websocket_data(data)
                        if data_point:
                            await callback(data_point)
                    except Exception as e:
                        self.logger.error(f"处理WebSocket消息失败: {str(e)}")
                        
        except Exception as e:
            self.logger.error(f"WebSocket连接失败: {str(e)}")
    
    def _parse_websocket_data(self, data: Dict) -> Optional[DataPoint]:
        """解析WebSocket数据"""
        try:
            stream = data.get("stream", "")
            event_data = data.get("data", {})
            
            if "@ticker" in stream:
                return self._parse_ticker_stream(event_data)
            elif "@kline" in stream:
                return self._parse_kline_stream(event_data)
            elif "@depth" in stream:
                return self._parse_depth_stream(event_data)
            elif "@trade" in stream:
                return self._parse_trade_stream(event_data)
            
            return None
            
        except Exception as e:
            self.logger.error(f"解析WebSocket数据失败: {str(e)}")
            return None
    
    def _parse_ticker_stream(self, data: Dict) -> DataPoint:
        """解析行情流数据"""
        return self._create_data_point(
            symbol=data["s"],
            data_type=DataType.TICKER,
            data={
                "price_change": float(data["p"]),
                "price_change_percent": float(data["P"]),
                "weighted_avg_price": float(data["w"]),
                "last_price": float(data["c"]),
                "last_qty": float(data["Q"]),
                "bid_price": float(data["b"]),
                "bid_qty": float(data["B"]),
                "ask_price": float(data["a"]),
                "ask_qty": float(data["A"]),
                "open_price": float(data["o"]),
                "high_price": float(data["h"]),
                "low_price": float(data["l"]),
                "volume": float(data["v"]),
                "quote_volume": float(data["q"]),
                "count": int(data["n"])
            },
            timestamp=datetime.fromtimestamp(data["E"] / 1000),
            raw_data=data
        )
    
    def _parse_kline_stream(self, data: Dict) -> DataPoint:
        """解析K线流数据"""
        kline = data["k"]
        return self._create_data_point(
            symbol=kline["s"],
            data_type=DataType.KLINE,
            data={
                "open_time": datetime.fromtimestamp(kline["t"] / 1000),
                "close_time": datetime.fromtimestamp(kline["T"] / 1000),
                "open": float(kline["o"]),
                "high": float(kline["h"]),
                "low": float(kline["l"]),
                "close": float(kline["c"]),
                "volume": float(kline["v"]),
                "quote_volume": float(kline["q"]),
                "trades": int(kline["n"]),
                "taker_buy_base_volume": float(kline["V"]),
                "taker_buy_quote_volume": float(kline["Q"]),
                "interval": kline["i"],
                "is_closed": kline["x"]
            },
            timestamp=datetime.fromtimestamp(kline["t"] / 1000),
            raw_data=data
        )
    
    def _parse_depth_stream(self, data: Dict) -> DataPoint:
        """解析深度流数据"""
        return self._create_data_point(
            symbol=data["s"],
            data_type=DataType.ORDERBOOK,
            data={
                "last_update_id": data["u"],
                "bids": [[float(price), float(qty)] for price, qty in data["b"]],
                "asks": [[float(price), float(qty)] for price, qty in data["a"]],
                "event_time": datetime.fromtimestamp(data["E"] / 1000)
            },
            timestamp=datetime.fromtimestamp(data["E"] / 1000),
            raw_data=data
        )
    
    def _parse_trade_stream(self, data: Dict) -> DataPoint:
        """解析交易流数据"""
        return self._create_data_point(
            symbol=data["s"],
            data_type=DataType.TRADE,
            data={
                "trade_id": data["t"],
                "price": float(data["p"]),
                "quantity": float(data["q"]),
                "buyer_order_id": data["b"],
                "seller_order_id": data["a"],
                "trade_time": datetime.fromtimestamp(data["T"] / 1000),
                "is_buyer_maker": data["m"]
            },
            timestamp=datetime.fromtimestamp(data["T"] / 1000),
            raw_data=data
        )
    
    async def _check_network_connectivity(self):
        """检查网络连通性"""
        try:
            import socket
            
            # 检查DNS解析
            host = "api.binance.com"
            self.logger.info(f"检查DNS解析: {host}")
            socket.gethostbyname(host)
            
            # 检查端口连通性
            sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            sock.settimeout(5)
            result = sock.connect_ex((host, 443))
            sock.close()
            
            if result != 0:
                raise Exception(f"无法连接到 {host}:443")
            
            self.logger.info("网络连通性检查通过")
            
        except Exception as e:
            error_msg = f"网络连通性检查失败: {str(e)}"
            self.logger.error(error_msg)
            raise Exception(error_msg)
    
    async def _cleanup_websockets(self):
        """清理WebSocket连接"""
        try:
            if self._ws_tasks:
                self.logger.info(f"正在关闭 {len(self._ws_tasks)} 个WebSocket连接...")
                for task in self._ws_tasks:
                    if not task.done():
                        task.cancel()
                        try:
                            await task
                        except asyncio.CancelledError:
                            pass
                        except Exception as e:
                            self.logger.warning(f"关闭WebSocket任务时出错: {str(e)}")
                
                self._ws_tasks.clear()
                self.logger.info("WebSocket连接已清理")
        except Exception as e:
            self.logger.error(f"清理WebSocket连接失败: {str(e)}")
    
    async def _cleanup_session(self):
        """清理HTTP会话"""
        try:
            if self._session and not self._session.closed:
                self.logger.info("正在关闭HTTP会话...")
                await self._session.close()
                # 等待底层连接关闭
                await asyncio.sleep(0.1)
                self.logger.info("HTTP会话已关闭")
            self._session = None
        except Exception as e:
            self.logger.error(f"清理HTTP会话失败: {str(e)}")
            self._session = None


def create_binance_connector(config: ConnectionConfig, market_type: str = "spot") -> BinanceConnector:
    """
    创建Binance连接器实例的工厂函数
    
    Args:
        config: 连接配置
        market_type: 市场类型 ("spot" 或 "futures")
        
    Returns:
        BinanceConnector: Binance连接器实例
    """
    return BinanceConnector(config, market_type) 