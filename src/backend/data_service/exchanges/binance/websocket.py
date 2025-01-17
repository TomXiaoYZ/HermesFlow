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

from ...common.models import Market, OrderStatus, OrderSide, OrderType
from ...common.decorators import retry
from .config import BINANCE_WS_URL, BINANCE_API_URL

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
        self.listen_key: Optional[str] = None
        self.handlers: Dict[str, List[Callable]] = {}
        self.running = False
        
        # 设置基础URL
        self.ws_url = BINANCE_WS_URL["testnet"] if testnet else BINANCE_WS_URL["mainnet"]
        self.api_url = BINANCE_API_URL["testnet"] if testnet else BINANCE_API_URL["mainnet"]
    
    @retry(
        max_retries=3,
        retry_delay=1.0,
        max_delay=10.0,
        exponential_base=2.0,
        exceptions=(BinanceWebSocketError, aiohttp.ClientError),
        should_retry=should_retry_ws_error
    )
    async def _get_listen_key(self) -> str:
        """获取用户数据流的listenKey
        
        Returns:
            str: listenKey
        """
        url = f"{self.api_url}/v3/userDataStream"
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
                url = f"{self.api_url}/v3/userDataStream"
                headers = {"X-MBX-APIKEY": self.api_key}
                params = {"listenKey": self.listen_key}
                
                async with aiohttp.ClientSession() as session:
                    async with session.put(url, headers=headers, params=params) as response:
                        if response.status != 200:
                            raise BinanceWebSocketError(f"续期listenKey失败: {await response.text()}")
                
                await asyncio.sleep(30 * 60)  # 每30分钟续期一次
            except Exception as e:
                print(f"续期listenKey出错: {str(e)}")
                await asyncio.sleep(60)  # 出错后等待1分钟再试
    
    def _handle_message(self, msg: dict):
        """处理WebSocket消息
        
        Args:
            msg: WebSocket消息
        """
        event_type = msg.get("e")
        if not event_type:
            return
            
        handlers = self.handlers.get(event_type, [])
        for handler in handlers:
            try:
                handler(msg)
            except Exception as e:
                print(f"处理消息出错: {str(e)}")
    
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
        try:
            if self.api_key and self.api_secret:
                # 获取listenKey
                self.listen_key = await self._get_listen_key()
                # 启动续期任务
                asyncio.create_task(self._keep_alive_listen_key())
                # 连接用户数据流
                self.ws = await self.session.ws_connect(f"{self.ws_url}/ws/{self.listen_key}")
            else:
                # 连接公共数据流
                self.ws = await self.session.ws_connect(f"{self.ws_url}/ws")
        except aiohttp.ClientError as e:
            raise BinanceWebSocketError(f"WebSocket连接错误: {str(e)}")
        except Exception as e:
            raise BinanceWebSocketError(f"未知错误: {str(e)}")
    
    async def start(self):
        """启动WebSocket客户端"""
        if self.running:
            return
            
        self.running = True
        self.session = aiohttp.ClientSession()
        
        while self.running:
            try:
                await self._connect()
                
                async for msg in self.ws:
                    if msg.type == aiohttp.WSMsgType.TEXT:
                        data = json.loads(msg.data)
                        self._handle_message(data)
                    elif msg.type in (aiohttp.WSMsgType.CLOSED, aiohttp.WSMsgType.ERROR):
                        break
                        
            except Exception as e:
                print(f"WebSocket连接错误: {str(e)}")
            
            if self.running:
                print("WebSocket连接断开，5秒后重连...")
                await asyncio.sleep(5)
    
    async def stop(self):
        """停止WebSocket客户端"""
        self.running = False
        if self.ws:
            await self.ws.close()
        if self.session:
            await self.session.close()
    
    def add_handler(self, event_type: str, handler: Callable):
        """添加消息处理器
        
        Args:
            event_type: 事件类型
            handler: 处理函数
        """
        if event_type not in self.handlers:
            self.handlers[event_type] = []
        self.handlers[event_type].append(handler)
    
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
        if not self.ws:
            raise BinanceWebSocketError("WebSocket未连接")
            
        try:
            msg = {
                "method": "SUBSCRIBE",
                "params": streams,
                "id": int(time.time() * 1000)
            }
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