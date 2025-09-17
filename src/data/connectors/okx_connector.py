# -*- coding: utf-8 -*-
"""
OKX 交易所数据连接器
提供OKX现货和期货市场的实时和历史数据接入
"""

import asyncio
import json
import hmac
import hashlib
import time
import base64
from typing import Dict, List, Optional, Any
from datetime import datetime, timedelta
import aiohttp
import websockets
from urllib.parse import urlencode

from .base_connector import BaseConnector, DataPoint, DataType, ConnectionStatus, ConnectionConfig


class OKXConnector(BaseConnector):
    """OKX交易所数据连接器"""
    
    # 生产环境API端点
    BASE_URL = "https://www.okx.com"
    WS_PUBLIC_URL = "wss://ws.okx.com:8443/ws/v5/public"
    WS_PRIVATE_URL = "wss://ws.okx.com:8443/ws/v5/private"
    
    # 沙盒环境API端点 (OKX使用相同域名，通过API密钥区分)
    SANDBOX_BASE_URL = "https://www.okx.com"
    SANDBOX_WS_PUBLIC_URL = "wss://wspap.okx.com:8443/ws/v5/public?brokerId=9999"
    SANDBOX_WS_PRIVATE_URL = "wss://wspap.okx.com:8443/ws/v5/private?brokerId=9999"
    
    # 时间间隔映射 (OKX格式)
    INTERVAL_MAP = {
        "1m": "1m", "3m": "3m", "5m": "5m", "15m": "15m", "30m": "30m",
        "1h": "1H", "2h": "2H", "4h": "4H", "6h": "6H", "8h": "8H", "12h": "12H",
        "1d": "1D", "3d": "3D", "1w": "1W", "1M": "1M"
    }
    
    # 产品类型映射
    PRODUCT_TYPE_MAP = {
        "spot": "SPOT",
        "futures": "FUTURES",
        "swap": "SWAP",
        "option": "OPTION"
    }
    
    def __init__(self, config: ConnectionConfig, market_type: str = "spot"):
        """
        初始化OKX连接器
        
        Args:
            config: 连接配置
            market_type: 市场类型 ("spot", "futures", "swap", "option")
        """
        super().__init__(config, f"okx_{market_type}")
        self.market_type = market_type
        self.product_type = self.PRODUCT_TYPE_MAP.get(market_type, "SPOT")
        
        # 根据配置选择API端点
        if config.sandbox or config.testnet:
            self.base_url = self.SANDBOX_BASE_URL
            self.ws_public_url = self.SANDBOX_WS_PUBLIC_URL
            self.ws_private_url = self.SANDBOX_WS_PRIVATE_URL
            self.logger.info(f"使用OKX沙盒环境: {self.base_url}")
        else:
            self.base_url = self.BASE_URL
            self.ws_public_url = self.WS_PUBLIC_URL
            self.ws_private_url = self.WS_PRIVATE_URL
            self.logger.info(f"使用OKX生产环境: {self.base_url}")
            
        self._ws_tasks = []
    
    async def connect(self) -> bool:
        """建立连接"""
        try:
            self.status = ConnectionStatus.CONNECTING
            self.logger.info(f"正在连接到 OKX {self.market_type} 市场...")
            
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
            self.logger.info(f"成功连接到 OKX {self.market_type} 市场")
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
            self.logger.info("正在断开OKX连接...")
            self.status = ConnectionStatus.DISCONNECTED
            
            # 关闭WebSocket连接
            await self._cleanup_websockets()
            
            # 关闭HTTP会话
            await self._cleanup_session()
            
            self.logger.info("已断开OKX连接")
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
            endpoint = "/api/v5/public/instruments"
            params = {"instType": self.product_type}
            response = await self._make_request("GET", endpoint, params)
            
            if response.get("code") != "0":
                raise Exception(f"API错误: {response.get('msg', 'Unknown error')}")
            
            symbols = []
            for instrument in response.get("data", []):
                if instrument.get("state") == "live":
                    symbols.append(instrument["instId"])
            
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
        limit: int = 100
    ) -> List[DataPoint]:
        """获取K线数据"""
        try:
            # 验证时间间隔
            if interval not in self.INTERVAL_MAP:
                raise ValueError(f"不支持的时间间隔: {interval}")
            
            # 构建参数
            params = {
                "instId": symbol.upper(),
                "bar": self.INTERVAL_MAP[interval],
                "limit": str(min(limit, 300))  # OKX限制最大300条
            }
            
            # OKX使用毫秒时间戳
            if end_time:
                params["before"] = str(int(end_time.timestamp() * 1000))
            if start_time:
                params["after"] = str(int(start_time.timestamp() * 1000))
            
            # 发送请求
            endpoint = "/api/v5/market/history-candles"
            response = await self._make_request("GET", endpoint, params)
            
            if response.get("code") != "0":
                raise Exception(f"API错误: {response.get('msg', 'Unknown error')}")
            
            # 解析数据
            data_points = []
            for candle in response.get("data", []):
                # OKX K线数据格式: [timestamp, open, high, low, close, volume, volCcy, volCcyQuote, confirm]
                data_point = self._create_data_point(
                    symbol=symbol,
                    data_type=DataType.KLINE,
                    data={
                        "open_time": datetime.fromtimestamp(int(candle[0]) / 1000),
                        "open": float(candle[1]),
                        "high": float(candle[2]),
                        "low": float(candle[3]),
                        "close": float(candle[4]),
                        "volume": float(candle[5]),
                        "volume_ccy": float(candle[6]),
                        "volume_ccy_quote": float(candle[7]),
                        "confirm": candle[8] == "1",
                        "interval": interval
                    },
                    timestamp=datetime.fromtimestamp(int(candle[0]) / 1000),
                    raw_data=candle
                )
                data_points.append(data_point)
            
            # OKX返回的数据是按时间倒序，需要反转
            data_points.reverse()
            return data_points
            
        except Exception as e:
            self.logger.error(f"获取K线数据失败: {str(e)}")
            return []
    
    async def get_ticker(self, symbol: str) -> Optional[DataPoint]:
        """获取行情数据"""
        try:
            params = {"instId": symbol.upper()}
            endpoint = "/api/v5/market/ticker"
            response = await self._make_request("GET", endpoint, params)
            
            if response.get("code") != "0":
                raise Exception(f"API错误: {response.get('msg', 'Unknown error')}")
            
            data = response.get("data")
            if not data:
                return None
            
            ticker_data = data[0]
            
            return self._create_data_point(
                symbol=symbol,
                data_type=DataType.TICKER,
                data={
                    "inst_type": ticker_data.get("instType"),
                    "inst_id": ticker_data.get("instId"),
                    "last": float(ticker_data.get("last", 0)),
                    "last_sz": float(ticker_data.get("lastSz", 0)),
                    "ask_px": float(ticker_data.get("askPx", 0)),
                    "ask_sz": float(ticker_data.get("askSz", 0)),
                    "bid_px": float(ticker_data.get("bidPx", 0)),
                    "bid_sz": float(ticker_data.get("bidSz", 0)),
                    "open_24h": float(ticker_data.get("open24h", 0)),
                    "high_24h": float(ticker_data.get("high24h", 0)),
                    "low_24h": float(ticker_data.get("low24h", 0)),
                    "vol_24h": float(ticker_data.get("vol24h", 0)),
                    "vol_ccy_24h": float(ticker_data.get("volCcy24h", 0)),
                    "ts": datetime.fromtimestamp(int(ticker_data.get("ts", 0)) / 1000)
                },
                timestamp=datetime.fromtimestamp(int(ticker_data.get("ts", 0)) / 1000),
                raw_data=ticker_data
            )
            
        except Exception as e:
            self.logger.error(f"获取行情数据失败: {str(e)}")
            return None
    
    async def get_orderbook(self, symbol: str, depth: int = 20) -> Optional[DataPoint]:
        """获取订单簿数据"""
        try:
            params = {
                "instId": symbol.upper(),
                "sz": str(min(depth, 400))  # OKX限制最大400档
            }
            
            endpoint = "/api/v5/market/books"
            response = await self._make_request("GET", endpoint, params)
            
            if response.get("code") != "0":
                raise Exception(f"API错误: {response.get('msg', 'Unknown error')}")
            
            data = response.get("data")
            if not data:
                return None
            
            book_data = data[0]
            
            return self._create_data_point(
                symbol=symbol,
                data_type=DataType.ORDERBOOK,
                data={
                    "asks": [[float(ask[0]), float(ask[1]), int(ask[2]), int(ask[3])] 
                             for ask in book_data.get("asks", [])],
                    "bids": [[float(bid[0]), float(bid[1]), int(bid[2]), int(bid[3])] 
                             for bid in book_data.get("bids", [])],
                    "ts": datetime.fromtimestamp(int(book_data.get("ts", 0)) / 1000),
                    "seq_id": book_data.get("seqId"),
                    "prev_seq_id": book_data.get("prevSeqId")
                },
                timestamp=datetime.fromtimestamp(int(book_data.get("ts", 0)) / 1000),
                raw_data=book_data
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
            # 构建订阅参数
            subscribe_args = []
            
            for symbol in symbols:
                for data_type in data_types:
                    if data_type == DataType.TICKER:
                        subscribe_args.append({"channel": "tickers", "instId": symbol.upper()})
                    elif data_type == DataType.KLINE:
                        subscribe_args.append({"channel": "candle1m", "instId": symbol.upper()})
                    elif data_type == DataType.ORDERBOOK:
                        subscribe_args.append({"channel": "books", "instId": symbol.upper()})
                    elif data_type == DataType.TRADE:
                        subscribe_args.append({"channel": "trades", "instId": symbol.upper()})
            
            if not subscribe_args:
                return False
            
            # 创建WebSocket连接
            task = asyncio.create_task(
                self._websocket_handler(self.WS_PUBLIC_URL, subscribe_args, callback)
            )
            self._ws_tasks.append(task)
            
            # 记录订阅
            for symbol in symbols:
                for data_type in data_types:
                    self._subscriptions.add(f"{symbol}:{data_type.value}")
            
            self.logger.info(f"成功订阅实时数据: {len(subscribe_args)} 个频道")
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
            endpoint = "/api/v5/public/time"
            self.logger.info(f"测试API连接: {self.BASE_URL}{endpoint}")
            
            start_time = time.time()
            response = await self._make_request("GET", endpoint)
            latency = (time.time() - start_time) * 1000
            
            if response.get("code") != "0":
                raise Exception(f"API测试失败: {response.get('msg', 'Unknown error')}")
            
            self.logger.info(f"API连接测试成功，延迟: {latency:.2f}ms")
            
            # 检查服务器时间
            server_time = int(response.get("data", [{}])[0].get("ts", 0))
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
        headers = {}
        
        # 如果需要签名且配置了API密钥
        if signed and self.config.api_key:
            timestamp = str(int(time.time() * 1000))
            headers.update(self._get_auth_headers(method, endpoint, params or {}, timestamp))
        
        if params is None:
            params = {}
        
        async with self._session.request(method, url, params=params, headers=headers) as response:
            if response.status == 200:
                return await response.json()
            else:
                error_text = await response.text()
                raise Exception(f"API请求失败: {response.status} - {error_text}")
    
    def _get_auth_headers(self, method: str, endpoint: str, params: Dict, timestamp: str) -> Dict[str, str]:
        """生成认证头"""
        if not self.config.api_secret or not self.config.passphrase:
            return {}
        
        # 构建签名字符串
        if method == "GET" and params:
            query_string = urlencode(params)
            sign_str = f"{timestamp}{method}{endpoint}?{query_string}"
        else:
            sign_str = f"{timestamp}{method}{endpoint}"
        
        # 生成签名
        signature = base64.b64encode(
            hmac.new(
                self.config.api_secret.encode(),
                sign_str.encode(),
                hashlib.sha256
            ).digest()
        ).decode()
        
        return {
            "OK-ACCESS-KEY": self.config.api_key,
            "OK-ACCESS-SIGN": signature,
            "OK-ACCESS-TIMESTAMP": timestamp,
            "OK-ACCESS-PASSPHRASE": self.config.passphrase
        }
    
    async def _websocket_handler(self, ws_url: str, subscribe_args: List[Dict], callback: callable):
        """WebSocket处理器"""
        try:
            async with websockets.connect(ws_url) as websocket:
                self.logger.info(f"WebSocket连接已建立: {ws_url}")
                
                # 发送订阅请求
                subscribe_msg = {
                    "op": "subscribe",
                    "args": subscribe_args
                }
                await websocket.send(json.dumps(subscribe_msg))
                
                async for message in websocket:
                    try:
                        data = json.loads(message)
                        
                        # 处理订阅确认
                        if data.get("event") == "subscribe":
                            self.logger.info(f"订阅确认: {data.get('arg', {}).get('channel')}")
                            continue
                        
                        # 处理数据推送
                        if "data" in data:
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
            arg = data.get("arg", {})
            channel = arg.get("channel", "")
            inst_id = arg.get("instId", "")
            data_list = data.get("data", [])
            
            if not data_list:
                return None
            
            ws_data = data_list[0]  # 取第一条数据
            
            if channel == "tickers":
                return self._parse_ticker_stream(ws_data, inst_id)
            elif channel.startswith("candle"):
                return self._parse_kline_stream(ws_data, inst_id, channel)
            elif channel == "books":
                return self._parse_depth_stream(ws_data, inst_id)
            elif channel == "trades":
                return self._parse_trade_stream(ws_data, inst_id)
            
            return None
            
        except Exception as e:
            self.logger.error(f"解析WebSocket数据失败: {str(e)}")
            return None
    
    def _parse_ticker_stream(self, data: Dict, symbol: str) -> DataPoint:
        """解析行情流数据"""
        return self._create_data_point(
            symbol=symbol,
            data_type=DataType.TICKER,
            data={
                "last": float(data.get("last", 0)),
                "last_sz": float(data.get("lastSz", 0)),
                "ask_px": float(data.get("askPx", 0)),
                "ask_sz": float(data.get("askSz", 0)),
                "bid_px": float(data.get("bidPx", 0)),
                "bid_sz": float(data.get("bidSz", 0)),
                "open_24h": float(data.get("open24h", 0)),
                "high_24h": float(data.get("high24h", 0)),
                "low_24h": float(data.get("low24h", 0)),
                "vol_24h": float(data.get("vol24h", 0)),
                "vol_ccy_24h": float(data.get("volCcy24h", 0))
            },
            timestamp=datetime.fromtimestamp(int(data.get("ts", 0)) / 1000),
            raw_data=data
        )
    
    def _parse_kline_stream(self, data: List, symbol: str, channel: str) -> DataPoint:
        """解析K线流数据"""
        # OKX K线数据格式: [timestamp, open, high, low, close, volume, volCcy, volCcyQuote, confirm]
        return self._create_data_point(
            symbol=symbol,
            data_type=DataType.KLINE,
            data={
                "open_time": datetime.fromtimestamp(int(data[0]) / 1000),
                "open": float(data[1]),
                "high": float(data[2]),
                "low": float(data[3]),
                "close": float(data[4]),
                "volume": float(data[5]),
                "volume_ccy": float(data[6]),
                "volume_ccy_quote": float(data[7]),
                "confirm": data[8] == "1",
                "interval": channel.replace("candle", "")
            },
            timestamp=datetime.fromtimestamp(int(data[0]) / 1000),
            raw_data=data
        )
    
    def _parse_depth_stream(self, data: Dict, symbol: str) -> DataPoint:
        """解析深度流数据"""
        return self._create_data_point(
            symbol=symbol,
            data_type=DataType.ORDERBOOK,
            data={
                "asks": [[float(ask[0]), float(ask[1]), int(ask[2]), int(ask[3])] 
                         for ask in data.get("asks", [])],
                "bids": [[float(bid[0]), float(bid[1]), int(bid[2]), int(bid[3])] 
                         for bid in data.get("bids", [])],
                "ts": datetime.fromtimestamp(int(data.get("ts", 0)) / 1000),
                "seq_id": data.get("seqId"),
                "prev_seq_id": data.get("prevSeqId")
            },
            timestamp=datetime.fromtimestamp(int(data.get("ts", 0)) / 1000),
            raw_data=data
        )
    
    def _parse_trade_stream(self, data: Dict, symbol: str) -> DataPoint:
        """解析交易流数据"""
        return self._create_data_point(
            symbol=symbol,
            data_type=DataType.TRADE,
            data={
                "inst_id": data.get("instId"),
                "trade_id": data.get("tradeId"),
                "px": float(data.get("px", 0)),
                "sz": float(data.get("sz", 0)),
                "side": data.get("side"),
                "ts": datetime.fromtimestamp(int(data.get("ts", 0)) / 1000)
            },
            timestamp=datetime.fromtimestamp(int(data.get("ts", 0)) / 1000),
            raw_data=data
        )
    
    async def _check_network_connectivity(self):
        """检查网络连通性"""
        try:
            import socket
            
            # 检查DNS解析
            host = "www.okx.com"
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


def create_okx_connector(config: ConnectionConfig, market_type: str = "spot") -> OKXConnector:
    """
    创建OKX连接器实例的工厂函数
    
    Args:
        config: 连接配置
        market_type: 市场类型 ("spot" 或 "futures")
        
    Returns:
        OKXConnector: OKX连接器实例
    """
    return OKXConnector(config, market_type) 