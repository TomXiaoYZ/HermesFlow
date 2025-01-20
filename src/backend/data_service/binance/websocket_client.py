import json
import asyncio
import logging
import websockets
from typing import Optional, Callable, List, Dict, Any

logger = logging.getLogger(__name__)

class BinanceWebsocketClient:
    """Binance WebSocket客户端"""
    
    def __init__(self, base_url: str = "wss://stream.binance.com:9443/ws"):
        self.base_url = base_url
        self.ws: Optional[websockets.WebSocketClientProtocol] = None
        self.connected = False
        self.message_handlers: List[Callable] = []
        self.reconnect_attempts = 0
        self.max_reconnect_attempts = 5
        self.reconnect_delay = 1  # 初始重连延迟（秒）
        self._message_id = 0
        self._subscriptions: List[str] = []
        self._heartbeat_task: Optional[asyncio.Task] = None

    async def connect(self) -> None:
        """建立WebSocket连接"""
        try:
            self.ws = await websockets.connect(self.base_url)
            self.connected = True
            self.reconnect_attempts = 0
            logger.info("WebSocket连接已建立")
            
            # 启动心跳任务
            self._heartbeat_task = asyncio.create_task(self._heartbeat_loop())
            
            # 重新订阅之前的channels
            for channel in self._subscriptions:
                await self.subscribe(channel)
                
        except Exception as e:
            logger.error(f"WebSocket连接失败: {e}")
            self.connected = False
            await self._handle_reconnect()

    async def close(self) -> None:
        """关闭WebSocket连接"""
        if self._heartbeat_task:
            self._heartbeat_task.cancel()
            
        if self.ws:
            try:
                await self.ws.close()
            except Exception as e:
                logger.error(f"关闭WebSocket连接时发生错误: {e}")
            finally:
                self.ws = None
                self.connected = False
                logger.info("WebSocket连接已关闭")

    async def _heartbeat_loop(self) -> None:
        """心跳循环"""
        while self.connected:
            try:
                await self._send_ping()
                await asyncio.sleep(30)  # 每30秒发送一次心跳
            except Exception as e:
                logger.error(f"心跳发送失败: {e}")
                await self._handle_reconnect()
                break

    async def _send_ping(self) -> None:
        """发送ping消息"""
        if self.ws and self.connected:
            await self.ws.ping()

    async def _handle_reconnect(self) -> None:
        """处理重连逻辑"""
        if self.reconnect_attempts >= self.max_reconnect_attempts:
            logger.error("达到最大重连次数")
            return

        self.reconnect_attempts += 1
        delay = min(self.reconnect_delay * (2 ** (self.reconnect_attempts - 1)), 60)
        logger.info(f"尝试重连 #{self.reconnect_attempts}, 延迟 {delay}秒")
        
        await asyncio.sleep(delay)
        await self.connect()

    def _get_next_id(self) -> int:
        """获取下一个消息ID"""
        self._message_id += 1
        return self._message_id

    async def subscribe(self, channel: str) -> None:
        """订阅频道"""
        if not self.connected:
            raise ConnectionError("WebSocket未连接")

        message = {
            "method": "SUBSCRIBE",
            "params": [channel],
            "id": self._get_next_id()
        }
        
        try:
            await self.ws.send(json.dumps(message))
            self._subscriptions.append(channel)
            logger.info(f"已订阅频道: {channel}")
        except Exception as e:
            logger.error(f"订阅失败: {e}")
            self.connected = False
            raise

    async def unsubscribe(self, channel: str) -> None:
        """取消订阅频道"""
        if not self.connected:
            raise ConnectionError("WebSocket未连接")

        message = {
            "method": "UNSUBSCRIBE",
            "params": [channel],
            "id": self._get_next_id()
        }
        
        try:
            await self.ws.send(json.dumps(message))
            self._subscriptions.remove(channel)
            logger.info(f"已取消订阅频道: {channel}")
        except Exception as e:
            logger.error(f"取消订阅失败: {e}")
            self.connected = False
            raise

    def add_message_handler(self, handler: Callable[[Dict[str, Any]], None]) -> None:
        """添加消息处理器"""
        self.message_handlers.append(handler)

    async def _handle_message(self, message: Dict[str, Any]) -> None:
        """处理接收到的消息"""
        try:
            for handler in self.message_handlers:
                await handler(message)
        except Exception as e:
            logger.error(f"消息处理失败: {e}")

    async def start_listening(self) -> None:
        """开始监听消息"""
        while self.connected:
            try:
                if self.ws:
                    message = await self.ws.recv()
                    data = json.loads(message)
                    await self._handle_message(data)
            except websockets.ConnectionClosed:
                logger.error("WebSocket连接已关闭")
                self.connected = False
                await self._handle_reconnect()
            except json.JSONDecodeError as e:
                logger.error(f"JSON解析失败: {e}")
            except Exception as e:
                logger.error(f"消息接收失败: {e}")
                await self._handle_reconnect() 