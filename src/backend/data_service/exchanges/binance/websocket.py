"""
Binance WebSocket客户端实现
"""
import json
import asyncio
import hmac
import hashlib
import time
from typing import Dict, Optional, Callable, List
import aiohttp
from urllib.parse import urlencode
from collections import defaultdict

from ...common.models import Market, OrderStatus, OrderSide, OrderType
from ...common.decorators import retry
from .config import (
    BINANCE_WS_URL, BINANCE_API_URL,
    BINANCE_FUTURES_WS_URL, BINANCE_FUTURES_API_URL
)

class BinanceWebSocketError(Exception):
    """WebSocket错误"""
    pass

def should_retry_ws_error(e: Exception) -> bool:
    """判断是否需要重试WebSocket错误
    
    Args:
        e: 异常
        
    Returns:
        bool: 是否需要重试
    """
    # WebSocket连接错误需要重试
    if isinstance(e, (aiohttp.ClientError, aiohttp.WSServerHandshakeError)):
        return True
    
    # 其他WebSocket错误需要重试
    if isinstance(e, BinanceWebSocketError):
        return True
    
    return False

class BinanceWebsocketClient:
    """Binance WebSocket客户端"""
    
    def __init__(self, api_key: str = "", api_secret: str = "", testnet: bool = False):
        """初始化WebSocket客户端
        
        Args:
            api_key: API Key
            api_secret: API Secret
            testnet: 是否使用测试网
        """
        self.api_key = api_key
        self.api_secret = api_secret
        self.testnet = testnet
        self.ws: Optional[aiohttp.ClientWebSocketResponse] = None
        self.session: Optional[aiohttp.ClientSession] = None
        self.spot_listen_key: Optional[str] = None
        self.futures_listen_key: Optional[str] = None
        self.handlers: Dict[str, List[Callable]] = {}
        self.running = False
        self.connected = asyncio.Event()
        self.is_futures = False
        self._last_ping_time = 0
        self._last_pong_time = 0
        self._reconnect_count = 0
        self._max_reconnect_count = 10
        self._reconnect_interval = 5
        self._heartbeat_interval = 30
        self._heartbeat_task: Optional[asyncio.Task] = None
        self._keep_alive_task: Optional[asyncio.Task] = None
        self._message_task: Optional[asyncio.Task] = None
        
        # 设置基础URL
        self.spot_ws_url = BINANCE_WS_URL["testnet"] if testnet else BINANCE_WS_URL["mainnet"]
        self.futures_ws_url = BINANCE_FUTURES_WS_URL["testnet"] if testnet else BINANCE_FUTURES_WS_URL["mainnet"]
        self.spot_api_url = BINANCE_API_URL["testnet"] if testnet else BINANCE_API_URL["mainnet"]
        self.futures_api_url = BINANCE_FUTURES_API_URL["testnet"] if testnet else BINANCE_FUTURES_API_URL["mainnet"]

        print("初始化WebSocket客户端")

    async def _heartbeat(self):
        """心跳检测"""
        while self.running:
            try:
                if self.ws and not self.ws.closed:
                    # 发送ping消息
                    await self.ws.ping()
                    self._last_ping_time = time.time()
                    
                    # 等待pong响应
                    try:
                        pong_waiter = await self.ws.ping()
                        await asyncio.wait_for(pong_waiter, timeout=5)
                        self._last_pong_time = time.time()
                    except asyncio.TimeoutError:
                        print("心跳检测超时，准备重连...")
                        if self.ws:
                            await self.ws.close()
                        continue
                    
                    # 检查最后一次pong时间
                    if time.time() - self._last_pong_time > self._heartbeat_interval * 2:
                        print("心跳响应超时，准备重连...")
                        if self.ws:
                            await self.ws.close()
                        continue
                        
                await asyncio.sleep(self._heartbeat_interval)
            except Exception as e:
                print(f"心跳检测出错: {str(e)}")
                await asyncio.sleep(5)

    async def _handle_connection_lost(self):
        """处理连接断开"""
        self.connected.clear()
        self._reconnect_count += 1
        
        if self._reconnect_count > self._max_reconnect_count:
            print("重连次数超过最大限制，停止重连")
            await self.stop()
            return
        
        wait_time = min(self._reconnect_interval * (2 ** (self._reconnect_count - 1)), 300)
        print(f"连接断开，{wait_time}秒后尝试第{self._reconnect_count}次重连...")
        await asyncio.sleep(wait_time)
        
        try:
            await self._connect()
            self._reconnect_count = 0
            print("重连成功")
        except Exception as e:
            print(f"重连失败: {str(e)}")
            await self._handle_connection_lost()

    async def _message_loop(self):
        """消息处理循环"""
        while self.running:
            try:
                if not self.ws:
                    await asyncio.sleep(1)
                    continue
                    
                async for msg in self.ws:
                    if msg.type == aiohttp.WSMsgType.TEXT:
                        data = json.loads(msg.data)
                        print(f"收到WebSocket消息: {data}")
                        self._handle_message(data)
                    elif msg.type == aiohttp.WSMsgType.PONG:
                        self._last_pong_time = time.time()
                    elif msg.type in (aiohttp.WSMsgType.CLOSED, aiohttp.WSMsgType.ERROR):
                        print(f"WebSocket连接关闭或错误: {msg.type}")
                        break
                        
            except Exception as e:
                print(f"WebSocket消息处理错误: {str(e)}")
                await self._handle_connection_lost()

    async def start(self, is_futures: bool = False):
        """启动WebSocket客户端
        
        Args:
            is_futures: 是否是合约WebSocket
        """
        if self.running:
            print("WebSocket客户端已经在运行")
            return

        print("正在启动WebSocket客户端")
        self.running = True
        self.is_futures = is_futures
        self.session = aiohttp.ClientSession()
        self.connected.clear()
        
        # 启动消息处理循环
        self._message_task = asyncio.create_task(self._message_loop())
        
        # 启动心跳检测
        self._heartbeat_task = asyncio.create_task(self._heartbeat())
        
        # 启动listenKey续期
        if self.api_key and self.api_secret:
            self._keep_alive_task = asyncio.create_task(self._keep_alive_listen_key())
            
        # 建立连接
        try:
            await self._connect()
        except Exception as e:
            print(f"连接失败: {str(e)}")
            await self.stop()
            raise

    async def stop(self):
        """停止WebSocket客户端"""
        print("正在停止WebSocket客户端")
        self.running = False
        
        # 取消心跳检测任务
        if self._heartbeat_task:
            self._heartbeat_task.cancel()
            try:
                await self._heartbeat_task
            except asyncio.CancelledError:
                pass
        
        # 取消listenKey续期任务
        if self._keep_alive_task:
            self._keep_alive_task.cancel()
            try:
                await self._keep_alive_task
            except asyncio.CancelledError:
                pass
                
        # 取消消息处理任务
        if self._message_task:
            self._message_task.cancel()
            try:
                await self._message_task
            except asyncio.CancelledError:
                pass
        
        if self.ws:
            await self.ws.close()
        if self.session:
            await self.session.close()
        self.connected.clear()
        self.spot_listen_key = None
        self.futures_listen_key = None
        self._reconnect_count = 0
        self._last_ping_time = 0
        self._last_pong_time = 0
        print("WebSocket连接已关闭")

    @retry(
        max_retries=3,
        retry_delay=1.0,
        max_delay=10.0,
        exponential_base=2.0,
        exceptions=(BinanceWebSocketError, aiohttp.ClientError),
        should_retry=should_retry_ws_error
    )
    async def _get_spot_listen_key(self) -> str:
        """获取现货用户数据流的listenKey
        
        Returns:
            str: listenKey
        """
        url = f"{self.spot_api_url}/v3/userDataStream"
        headers = {"X-MBX-APIKEY": self.api_key}
        
        try:
            async with aiohttp.ClientSession() as session:
                async with session.post(url, headers=headers) as response:
                    if response.status == 200:
                        data = await response.json()
                        return data["listenKey"]
                    raise BinanceWebSocketError(f"获取listenKey失败: {await response.text()}")
        except aiohttp.ClientError as e:
            raise BinanceWebSocketError(f"网络错误: {str(e)}")
        except Exception as e:
            raise BinanceWebSocketError(f"未知错误: {str(e)}")
            
    @retry(
        max_retries=3,
        retry_delay=1.0,
        max_delay=10.0,
        exponential_base=2.0,
        exceptions=(BinanceWebSocketError, aiohttp.ClientError),
        should_retry=should_retry_ws_error
    )
    async def _get_futures_listen_key(self) -> str:
        """获取合约用户数据流的listenKey
        
        Returns:
            str: listenKey
        """
        url = f"{self.futures_api_url}/fapi/v1/listenKey"
        headers = {"X-MBX-APIKEY": self.api_key}
        
        try:
            async with aiohttp.ClientSession() as session:
                async with session.post(url, headers=headers) as response:
                    if response.status == 200:
                        data = await response.json()
                        return data["listenKey"]
                    raise BinanceWebSocketError(f"获取listenKey失败: {await response.text()}")
        except aiohttp.ClientError as e:
            raise BinanceWebSocketError(f"网络错误: {str(e)}")
        except Exception as e:
            raise BinanceWebSocketError(f"未知错误: {str(e)}")

    @retry(
        max_retries=3,
        retry_delay=1.0,
        max_delay=10.0,
        exponential_base=2.0,
        exceptions=(BinanceWebSocketError, aiohttp.ClientError),
        should_retry=should_retry_ws_error
    )
    async def _keep_alive_listen_key(self):
        """保持listenKey有效"""
        while self.running:
            try:
                if self.spot_listen_key:
                    url = f"{self.spot_api_url}/v3/userDataStream"
                    headers = {"X-MBX-APIKEY": self.api_key}
                    params = {"listenKey": self.spot_listen_key}
                    
                    async with aiohttp.ClientSession() as session:
                        async with session.put(url, headers=headers, params=params) as response:
                            if response.status != 200:
                                raise BinanceWebSocketError(f"续期spot listenKey失败: {await response.text()}")
                
                if self.futures_listen_key:
                    url = f"{self.futures_api_url}/fapi/v1/listenKey"
                    headers = {"X-MBX-APIKEY": self.api_key}
                    params = {"listenKey": self.futures_listen_key}
                    
                    async with aiohttp.ClientSession() as session:
                        async with session.put(url, headers=headers, params=params) as response:
                            if response.status != 200:
                                raise BinanceWebSocketError(f"续期futures listenKey失败: {await response.text()}")
                
                await asyncio.sleep(30 * 60)  # 每30分钟续期一次
            except Exception as e:
                print(f"续期listenKey出错: {str(e)}")
                await asyncio.sleep(60)  # 出错后等待1分钟再试
    
    async def _handle_message(self, data: dict):
        """处理WebSocket消息
        
        Args:
            data: 消息数据
        """
        print(f"处理WebSocket消息: {data}")
        
        # 处理订阅响应
        if "result" in data:
            print("收到订阅响应")
            return
            
        # 处理错误消息
        if "error" in data:
            print(f"收到错误消息: {data['error']}")
            return
            
        # 处理数据消息
        if "e" in data:
            event_type = data["e"]
            print(f"事件类型: {event_type}")
            
            if event_type == "24hrTicker":
                handlers = self.handlers.get("market_ticker", [])
                print(f"找到{len(handlers)}个行情处理器")
                for handler in handlers:
                    await handler(data)
                    
            elif event_type == "trade":
                handlers = self.handlers.get("trade", [])
                print(f"找到{len(handlers)}个交易处理器")
                for handler in handlers:
                    await handler(data)

    @retry(
        max_retries=3,
        retry_delay=1.0,
        max_delay=10.0,
        exponential_base=2.0,
        exceptions=(BinanceWebSocketError, aiohttp.ClientError),
        should_retry=should_retry_ws_error
    )
    async def _connect(self):
        """建立WebSocket连接"""
        if self.ws and not self.ws.closed:
            print("WebSocket连接已存在")
            return
            
        print("正在建立WebSocket连接...")
        
        # 选择WebSocket URL
        ws_url = self.futures_ws_url if self.is_futures else self.spot_ws_url
        print(f"使用WebSocket URL: {ws_url}")
        
        try:
            self.ws = await self.session.ws_connect(ws_url)
            print("WebSocket连接建立成功")
            self.connected.set()
        except Exception as e:
            print(f"WebSocket连接失败: {str(e)}")
            raise BinanceWebSocketError(f"WebSocket连接失败: {str(e)}")
    
    @retry(
        max_retries=3,
        retry_delay=1.0,
        max_delay=10.0,
        exponential_base=2.0,
        exceptions=(BinanceWebSocketError, aiohttp.ClientError),
        should_retry=should_retry_ws_error
    )
    async def subscribe(self, streams: List[str]):
        """订阅数据流
        
        Args:
            streams: 数据流列表
        """
        print(f"正在订阅数据流: {streams}")  # 添加日志
        # 等待连接成功
        await self.connected.wait()
        
        if not self.ws:
            raise BinanceWebSocketError("WebSocket未连接")
            
        try:
            # 在测试网环境下，我们需要使用不同的订阅方式
            if self.testnet:
                # 直接连接到指定的数据流
                if self.ws:
                    await self.ws.close()
                base_url = self.futures_ws_url if self.is_futures else self.spot_ws_url
                stream_url = f"{base_url}/stream?streams={'/'.join(streams)}"
                print(f"连接到数据流: {stream_url}")  # 添加日志
                self.ws = await self.session.ws_connect(stream_url)
                self.connected.set()
            else:
                # 使用标准的订阅方式
                msg = {
                    "method": "SUBSCRIBE",
                    "params": streams,
                    "id": int(time.time() * 1000)
                }
                print(f"发送订阅消息: {msg}")  # 添加日志
                await self.ws.send_json(msg)
        except Exception as e:
            raise BinanceWebSocketError(f"订阅失败: {str(e)}")

    @retry(
        max_retries=3,
        retry_delay=1.0,
        max_delay=10.0,
        exponential_base=2.0,
        exceptions=(BinanceWebSocketError, aiohttp.ClientError),
        should_retry=should_retry_ws_error
    )
    async def unsubscribe(self, streams: List[str]):
        """取消订阅数据流
        
        Args:
            streams: 数据流列表
        """
        # 等待连接成功
        await self.connected.wait()
        
        if not self.ws:
            raise BinanceWebSocketError("WebSocket未连接")
            
        try:
            msg = {
                "method": "UNSUBSCRIBE",
                "params": streams,
                "id": int(time.time() * 1000)
            }
            await self.ws.send_json(msg)
        except Exception as e:
            raise BinanceWebSocketError(f"取消订阅失败: {str(e)}")

    async def subscribe_market_ticker(self, symbol: str):
        """订阅市场行情数据
        
        Args:
            symbol: 交易对
        """
        stream = f"{symbol.lower()}@ticker"
        msg = {
            "method": "SUBSCRIBE",
            "params": [stream],
            "id": int(time.time() * 1000)
        }
        print(f"正在订阅数据流: {stream}")
        print(f"发送订阅消息: {msg}")
        await self.ws.send_json(msg)

    async def subscribe_trade(self, symbol: str):
        """订阅交易数据
        
        Args:
            symbol: 交易对
        """
        stream = f"{symbol.lower()}@trade"
        msg = {
            "method": "SUBSCRIBE",
            "params": [stream],
            "id": int(time.time() * 1000)
        }
        print(f"正在订阅数据流: {stream}")
        print(f"发送订阅消息: {msg}")
        await self.ws.send_json(msg)

    def add_market_ticker_handler(self, handler: Callable):
        """添加市场行情处理器"""
        self.handlers["ticker"].append(handler)

    def add_trade_handler(self, handler: Callable):
        """添加交易数据处理器"""
        self.handlers["trade"].append(handler) 