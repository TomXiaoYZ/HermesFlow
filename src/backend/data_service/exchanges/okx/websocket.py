"""
OKX WebSocket客户端
"""
import json
import asyncio
import logging
from typing import Dict, Any, List, Optional, Callable, Set
from datetime import datetime, timedelta
import aiohttp
from aiohttp import ClientWebSocketResponse
import hmac
import base64

from ....common.models import Market
from ....common.exceptions import NetworkError
from .exceptions import OKXWebSocketError

logger = logging.getLogger(__name__)

class OKXWebSocket:
    """OKX WebSocket客户端"""
    
    def __init__(
        self,
        api_key: Optional[str] = None,
        api_secret: Optional[str] = None,
        passphrase: Optional[str] = None,
        testnet: bool = False,
        ping_interval: int = 20,
        ping_timeout: int = 10,
        reconnect_delay: int = 5
    ):
        """初始化WebSocket客户端
        
        Args:
            api_key: API Key
            api_secret: API Secret
            passphrase: API密码
            testnet: 是否使用测试网
            ping_interval: 心跳间隔(秒)
            ping_timeout: 心跳超时(秒)
            reconnect_delay: 重连延迟(秒)
        """
        self.api_key = api_key
        self.api_secret = api_secret.encode() if api_secret else None
        self.passphrase = passphrase
        self.ws_url = "wss://ws.okx.com:8443/ws/v5/public" if not testnet else "wss://ws.okx.com:8443/ws/v5/public"
        self.private_ws_url = "wss://ws.okx.com:8443/ws/v5/private" if not testnet else "wss://ws.okx.com:8443/ws/v5/private"
        
        self.ping_interval = ping_interval
        self.ping_timeout = ping_timeout
        self.reconnect_delay = reconnect_delay
        
        self._ws: Optional[ClientWebSocketResponse] = None
        self._private_ws: Optional[ClientWebSocketResponse] = None
        self._session: Optional[aiohttp.ClientSession] = None
        self._subscriptions: Set[str] = set()
        self._private_subscriptions: Set[str] = set()
        self._handlers: Dict[str, Callable] = {}
        self._last_ping: Optional[datetime] = None
        self._ping_task: Optional[asyncio.Task] = None
        self._receive_task: Optional[asyncio.Task] = None
        self._running = False
        
    def _get_timestamp(self) -> str:
        """获取ISO格式的时间戳"""
        return datetime.utcnow().isoformat()[:-3] + 'Z'
        
    def _sign(self, timestamp: str, method: str, request_path: str, body: str = '') -> str:
        """生成签名
        
        Args:
            timestamp: 时间戳
            method: 请求方法
            request_path: 请求路径
            body: 请求体
            
        Returns:
            str: 签名字符串
        """
        if not self.api_secret:
            raise OKXWebSocketError("API Secret未配置")
            
        message = timestamp + method + request_path + body
        mac = hmac.new(
            self.api_secret,
            message.encode(),
            digestmod='sha256'
        )
        return base64.b64encode(mac.digest()).decode()
        
    async def _connect(self) -> None:
        """连接WebSocket"""
        if not self._session:
            self._session = aiohttp.ClientSession()
            
        # 公共频道
        self._ws = await self._session.ws_connect(
            self.ws_url,
            heartbeat=self.ping_interval,
            receive_timeout=self.ping_timeout
        )
        
        # 私有频道(如果配置了API Key)
        if self.api_key and self.api_secret and self.passphrase:
            timestamp = self._get_timestamp()
            sign = self._sign(timestamp, 'GET', '/users/self/verify')
            
            self._private_ws = await self._session.ws_connect(
                self.private_ws_url,
                heartbeat=self.ping_interval,
                receive_timeout=self.ping_timeout,
                headers={
                    'OK-ACCESS-KEY': self.api_key,
                    'OK-ACCESS-SIGN': sign,
                    'OK-ACCESS-TIMESTAMP': timestamp,
                    'OK-ACCESS-PASSPHRASE': self.passphrase
                }
            )
            
        # 启动心跳和接收任务
        self._running = True
        self._ping_task = asyncio.create_task(self._ping_loop())
        self._receive_task = asyncio.create_task(self._receive_loop())
        
    async def _ping_loop(self) -> None:
        """心跳循环"""
        while self._running:
            try:
                if self._ws:
                    await self._ws.ping()
                if self._private_ws:
                    await self._private_ws.ping()
                self._last_ping = datetime.utcnow()
                await asyncio.sleep(self.ping_interval)
            except Exception as e:
                logger.error(f"心跳发送失败: {str(e)}")
                await self._reconnect()
                
    async def _receive_loop(self) -> None:
        """消息接收循环"""
        while self._running:
            try:
                # 接收公共频道消息
                if self._ws:
                    msg = await self._ws.receive_json()
                    await self._handle_message(msg)
                    
                # 接收私有频道消息
                if self._private_ws:
                    msg = await self._private_ws.receive_json()
                    await self._handle_message(msg, is_private=True)
                    
            except Exception as e:
                logger.error(f"消息接收失败: {str(e)}")
                await self._reconnect()
                
    async def _reconnect(self) -> None:
        """重新连接"""
        self._running = False
        
        # 关闭现有连接
        if self._ws:
            await self._ws.close()
            self._ws = None
        if self._private_ws:
            await self._private_ws.close()
            self._private_ws = None
            
        # 取消任务
        if self._ping_task:
            self._ping_task.cancel()
            self._ping_task = None
        if self._receive_task:
            self._receive_task.cancel()
            self._receive_task = None
            
        # 等待重连延迟
        await asyncio.sleep(self.reconnect_delay)
        
        try:
            # 重新连接
            await self._connect()
            
            # 重新订阅
            for topic in self._subscriptions:
                await self.subscribe(topic)
            for topic in self._private_subscriptions:
                await self.subscribe(topic, private=True)
                
        except Exception as e:
            logger.error(f"重连失败: {str(e)}")
            await self._reconnect()
            
    async def _handle_message(self, message: Dict[str, Any], is_private: bool = False) -> None:
        """处理WebSocket消息
        
        Args:
            message: 消息数据
            is_private: 是否是私有频道消息
        """
        if 'event' in message:
            # 处理系统事件
            event = message['event']
            if event == 'error':
                logger.error(f"WebSocket错误: {message}")
                raise OKXWebSocketError(message.get('msg', '未知错误'))
            elif event == 'subscribe':
                logger.info(f"订阅成功: {message}")
            elif event == 'unsubscribe':
                logger.info(f"取消订阅成功: {message}")
        else:
            # 处理数据更新
            channel = message.get('arg', {}).get('channel')
            if channel and channel in self._handlers:
                await self._handlers[channel](message)
                
    async def subscribe(
        self,
        topic: str,
        symbol: str,
        callback: Callable[[Dict[str, Any]], None],
        private: bool = False
    ) -> None:
        """订阅主题
        
        Args:
            topic: 主题
            symbol: 交易对
            callback: 回调函数
            private: 是否是私有频道
        """
        channel = f"{topic}:{symbol}"
        
        # 添加处理器
        self._handlers[channel] = callback
        
        # 发送订阅请求
        subscribe_message = {
            "op": "subscribe",
            "args": [{
                "channel": topic,
                "instId": symbol
            }]
        }
        
        if private:
            if not self._private_ws:
                raise OKXWebSocketError("未连接私有频道")
            await self._private_ws.send_json(subscribe_message)
            self._private_subscriptions.add(channel)
        else:
            if not self._ws:
                raise OKXWebSocketError("未连接公共频道")
            await self._ws.send_json(subscribe_message)
            self._subscriptions.add(channel)
            
    async def unsubscribe(self, topic: str, symbol: str, private: bool = False) -> None:
        """取消订阅
        
        Args:
            topic: 主题
            symbol: 交易对
            private: 是否是私有频道
        """
        channel = f"{topic}:{symbol}"
        
        # 发送取消订阅请求
        unsubscribe_message = {
            "op": "unsubscribe",
            "args": [{
                "channel": topic,
                "instId": symbol
            }]
        }
        
        if private:
            if not self._private_ws:
                raise OKXWebSocketError("未连接私有频道")
            await self._private_ws.send_json(unsubscribe_message)
            self._private_subscriptions.remove(channel)
        else:
            if not self._ws:
                raise OKXWebSocketError("未连接公共频道")
            await self._ws.send_json(unsubscribe_message)
            self._subscriptions.remove(channel)
            
        # 移除处理器
        if channel in self._handlers:
            del self._handlers[channel]
            
    async def start(self) -> None:
        """启动WebSocket客户端"""
        await self._connect()
        
    async def stop(self) -> None:
        """停止WebSocket客户端"""
        self._running = False
        
        # 取消所有订阅
        for topic in list(self._subscriptions):
            channel, symbol = topic.split(':')
            await self.unsubscribe(channel, symbol)
        for topic in list(self._private_subscriptions):
            channel, symbol = topic.split(':')
            await self.unsubscribe(channel, symbol, private=True)
            
        # 关闭连接
        if self._ws:
            await self._ws.close()
            self._ws = None
        if self._private_ws:
            await self._private_ws.close()
            self._private_ws = None
            
        # 取消任务
        if self._ping_task:
            self._ping_task.cancel()
            self._ping_task = None
        if self._receive_task:
            self._receive_task.cancel()
            self._receive_task = None
            
        # 关闭会话
        if self._session:
            await self._session.close()
            self._session = None 