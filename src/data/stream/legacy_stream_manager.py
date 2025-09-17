"""
WebSocket数据流管理器

负责管理多个交易所的WebSocket连接，提供统一的数据流接口。
支持连接池管理、自动重连、数据分发等功能。
"""

import asyncio
import json
import time
import logging
from typing import Dict, List, Optional, Callable, Any, Set
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from enum import Enum
import websockets
from websockets.exceptions import ConnectionClosed, WebSocketException

logger = logging.getLogger(__name__)


class StreamType(Enum):
    """数据流类型"""
    TICKER = "ticker"           # 行情数据
    KLINE = "kline"            # K线数据
    ORDERBOOK = "orderbook"    # 订单簿数据
    TRADES = "trades"          # 成交记录
    ACCOUNT = "account"        # 账户数据
    ORDERS = "orders"          # 订单状态


class ConnectionStatus(Enum):
    """连接状态"""
    DISCONNECTED = "disconnected"
    CONNECTING = "connecting"
    CONNECTED = "connected"
    RECONNECTING = "reconnecting"
    ERROR = "error"


@dataclass
class StreamConfig:
    """数据流配置"""
    # 基础配置
    name: str                                    # 流名称
    exchange: str                               # 交易所名称
    stream_type: StreamType                     # 流类型
    symbols: List[str] = field(default_factory=list)  # 订阅符号列表
    
    # WebSocket配置
    ws_url: str = ""                           # WebSocket URL
    ping_interval: int = 30                    # 心跳间隔(秒)
    ping_timeout: int = 10                     # 心跳超时(秒)
    max_size: int = 1024 * 1024               # 最大消息大小
    
    # 重连配置
    auto_reconnect: bool = True                # 自动重连
    max_reconnect_attempts: int = 10           # 最大重连次数
    reconnect_delay: float = 5.0              # 重连延迟(秒)
    backoff_factor: float = 1.5               # 退避因子
    
    # 数据处理配置
    buffer_size: int = 1000                   # 缓冲区大小
    batch_size: int = 100                     # 批处理大小
    flush_interval: float = 1.0               # 刷新间隔(秒)
    
    # 回调函数
    on_message: Optional[Callable] = None      # 消息回调
    on_connect: Optional[Callable] = None      # 连接回调
    on_disconnect: Optional[Callable] = None   # 断开回调
    on_error: Optional[Callable] = None        # 错误回调


class WebSocketStream:
    """单个WebSocket数据流"""
    
    def __init__(self, config: StreamConfig):
        """
        初始化WebSocket数据流
        
        Args:
            config: 流配置
        """
        self.config = config
        self.websocket: Optional[websockets.WebSocketServerProtocol] = None
        self.status = ConnectionStatus.DISCONNECTED
        
        # 重连控制
        self.reconnect_count = 0
        self.last_reconnect_time = 0
        self.is_running = False
        
        # 数据缓冲
        self.message_buffer: List[Dict] = []
        self.last_flush_time = time.time()
        
        # 统计信息
        self.stats = {
            'messages_received': 0,
            'messages_processed': 0,
            'bytes_received': 0,
            'connection_time': None,
            'last_message_time': None,
            'errors': 0
        }
        
        logger.info(f"WebSocket流初始化: {config.name}")
    
    async def connect(self) -> bool:
        """
        建立WebSocket连接
        
        Returns:
            bool: 连接是否成功
        """
        if self.status == ConnectionStatus.CONNECTED:
            logger.warning(f"流 {self.config.name} 已连接")
            return True
        
        try:
            self.status = ConnectionStatus.CONNECTING
            logger.info(f"连接WebSocket流: {self.config.name} -> {self.config.ws_url}")
            
            # 建立WebSocket连接
            self.websocket = await websockets.connect(
                self.config.ws_url,
                ping_interval=self.config.ping_interval,
                ping_timeout=self.config.ping_timeout,
                max_size=self.config.max_size
            )
            
            self.status = ConnectionStatus.CONNECTED
            self.stats['connection_time'] = datetime.now()
            self.reconnect_count = 0
            
            # 调用连接回调
            if self.config.on_connect:
                await self.config.on_connect(self)
            
            logger.info(f"WebSocket流连接成功: {self.config.name}")
            return True
            
        except Exception as e:
            self.status = ConnectionStatus.ERROR
            self.stats['errors'] += 1
            
            # 调用错误回调
            if self.config.on_error:
                await self.config.on_error(self, e)
            
            logger.error(f"WebSocket流连接失败: {self.config.name} - {e}")
            return False
    
    async def disconnect(self):
        """断开WebSocket连接"""
        try:
            self.is_running = False
            
            if self.websocket and not self.websocket.closed:
                await self.websocket.close()
            
            self.status = ConnectionStatus.DISCONNECTED
            
            # 调用断开回调
            if self.config.on_disconnect:
                await self.config.on_disconnect(self)
            
            logger.info(f"WebSocket流已断开: {self.config.name}")
            
        except Exception as e:
            logger.error(f"断开WebSocket流异常: {self.config.name} - {e}")
    
    async def send_message(self, message: Dict):
        """
        发送消息到WebSocket
        
        Args:
            message: 要发送的消息
        """
        if not self.websocket or self.status != ConnectionStatus.CONNECTED:
            logger.warning(f"WebSocket未连接，无法发送消息: {self.config.name}")
            return
        
        try:
            message_str = json.dumps(message)
            await self.websocket.send(message_str)
            logger.debug(f"发送WebSocket消息: {self.config.name} - {message_str[:100]}")
            
        except Exception as e:
            logger.error(f"发送WebSocket消息失败: {self.config.name} - {e}")
            self.stats['errors'] += 1
    
    async def _handle_message(self, message: str):
        """
        处理接收到的消息
        
        Args:
            message: 原始消息字符串
        """
        try:
            # 解析JSON消息
            data = json.loads(message)
            
            # 更新统计信息
            self.stats['messages_received'] += 1
            self.stats['bytes_received'] += len(message)
            self.stats['last_message_time'] = datetime.now()
            
            # 添加元数据
            data['_stream_name'] = self.config.name
            data['_exchange'] = self.config.exchange
            data['_stream_type'] = self.config.stream_type.value
            data['_timestamp'] = time.time()
            
            # 添加到缓冲区
            self.message_buffer.append(data)
            
            # 检查是否需要刷新缓冲区
            current_time = time.time()
            if (len(self.message_buffer) >= self.config.batch_size or 
                current_time - self.last_flush_time >= self.config.flush_interval):
                await self._flush_buffer()
            
        except json.JSONDecodeError as e:
            logger.error(f"JSON解析失败: {self.config.name} - {e}")
            self.stats['errors'] += 1
        except Exception as e:
            logger.error(f"消息处理异常: {self.config.name} - {e}")
            self.stats['errors'] += 1
    
    async def _flush_buffer(self):
        """刷新消息缓冲区"""
        if not self.message_buffer:
            return
        
        try:
            # 复制缓冲区数据
            messages = self.message_buffer.copy()
            self.message_buffer.clear()
            self.last_flush_time = time.time()
            
            # 调用消息回调
            if self.config.on_message:
                await self.config.on_message(messages)
            
            self.stats['messages_processed'] += len(messages)
            
        except Exception as e:
            logger.error(f"刷新缓冲区异常: {self.config.name} - {e}")
            self.stats['errors'] += 1
    
    async def _reconnect(self):
        """自动重连逻辑"""
        if not self.config.auto_reconnect:
            return
        
        if self.reconnect_count >= self.config.max_reconnect_attempts:
            logger.error(f"达到最大重连次数: {self.config.name}")
            return
        
        # 计算重连延迟
        delay = self.config.reconnect_delay * (self.config.backoff_factor ** self.reconnect_count)
        current_time = time.time()
        
        if current_time - self.last_reconnect_time < delay:
            return
        
        self.reconnect_count += 1
        self.last_reconnect_time = current_time
        self.status = ConnectionStatus.RECONNECTING
        
        logger.info(f"尝试重连 ({self.reconnect_count}/{self.config.max_reconnect_attempts}): {self.config.name}")
        
        # 尝试重连
        if await self.connect():
            logger.info(f"重连成功: {self.config.name}")
        else:
            logger.warning(f"重连失败: {self.config.name}")
    
    async def run(self):
        """运行WebSocket数据流"""
        self.is_running = True
        
        while self.is_running:
            try:
                # 确保连接
                if self.status != ConnectionStatus.CONNECTED:
                    if not await self.connect():
                        await asyncio.sleep(1)
                        continue
                
                # 监听消息
                async for message in self.websocket:
                    if not self.is_running:
                        break
                    
                    await self._handle_message(message)
                
            except ConnectionClosed:
                logger.warning(f"WebSocket连接关闭: {self.config.name}")
                self.status = ConnectionStatus.DISCONNECTED
                await self._reconnect()
                
            except WebSocketException as e:
                logger.error(f"WebSocket异常: {self.config.name} - {e}")
                self.status = ConnectionStatus.ERROR
                await self._reconnect()
                
            except Exception as e:
                logger.error(f"数据流运行异常: {self.config.name} - {e}")
                self.stats['errors'] += 1
                await asyncio.sleep(1)
        
        # 清理资源
        await self.disconnect()
    
    def get_stats(self) -> Dict[str, Any]:
        """获取流统计信息"""
        return {
            'name': self.config.name,
            'exchange': self.config.exchange,
            'stream_type': self.config.stream_type.value,
            'status': self.status.value,
            'reconnect_count': self.reconnect_count,
            'buffer_size': len(self.message_buffer),
            **self.stats
        }


class StreamManager:
    """WebSocket数据流管理器"""
    
    def __init__(self):
        """初始化流管理器"""
        self.streams: Dict[str, WebSocketStream] = {}
        self.tasks: Dict[str, asyncio.Task] = {}
        self.is_running = False
        
        logger.info("数据流管理器初始化完成")
    
    def add_stream(self, config: StreamConfig) -> bool:
        """
        添加数据流
        
        Args:
            config: 流配置
            
        Returns:
            bool: 添加是否成功
        """
        if config.name in self.streams:
            logger.warning(f"数据流已存在: {config.name}")
            return False
        
        try:
            stream = WebSocketStream(config)
            self.streams[config.name] = stream
            
            logger.info(f"添加数据流: {config.name}")
            return True
            
        except Exception as e:
            logger.error(f"添加数据流失败: {config.name} - {e}")
            return False
    
    def remove_stream(self, name: str) -> bool:
        """
        移除数据流
        
        Args:
            name: 流名称
            
        Returns:
            bool: 移除是否成功
        """
        if name not in self.streams:
            logger.warning(f"数据流不存在: {name}")
            return False
        
        try:
            # 停止任务
            if name in self.tasks:
                self.tasks[name].cancel()
                del self.tasks[name]
            
            # 移除流
            del self.streams[name]
            
            logger.info(f"移除数据流: {name}")
            return True
            
        except Exception as e:
            logger.error(f"移除数据流失败: {name} - {e}")
            return False
    
    async def start_stream(self, name: str) -> bool:
        """
        启动指定数据流
        
        Args:
            name: 流名称
            
        Returns:
            bool: 启动是否成功
        """
        if name not in self.streams:
            logger.error(f"数据流不存在: {name}")
            return False
        
        if name in self.tasks and not self.tasks[name].done():
            logger.warning(f"数据流已在运行: {name}")
            return True
        
        try:
            stream = self.streams[name]
            task = asyncio.create_task(stream.run())
            self.tasks[name] = task
            
            logger.info(f"启动数据流: {name}")
            return True
            
        except Exception as e:
            logger.error(f"启动数据流失败: {name} - {e}")
            return False
    
    async def stop_stream(self, name: str) -> bool:
        """
        停止指定数据流
        
        Args:
            name: 流名称
            
        Returns:
            bool: 停止是否成功
        """
        if name not in self.streams:
            logger.error(f"数据流不存在: {name}")
            return False
        
        try:
            # 停止流
            stream = self.streams[name]
            await stream.disconnect()
            
            # 取消任务
            if name in self.tasks:
                self.tasks[name].cancel()
                try:
                    await self.tasks[name]
                except asyncio.CancelledError:
                    pass
                del self.tasks[name]
            
            logger.info(f"停止数据流: {name}")
            return True
            
        except Exception as e:
            logger.error(f"停止数据流失败: {name} - {e}")
            return False
    
    async def start_all(self):
        """启动所有数据流"""
        self.is_running = True
        
        for name in self.streams:
            await self.start_stream(name)
        
        logger.info("所有数据流已启动")
    
    async def stop_all(self):
        """停止所有数据流"""
        self.is_running = False
        
        for name in list(self.streams.keys()):
            await self.stop_stream(name)
        
        logger.info("所有数据流已停止")
    
    def get_stream_stats(self, name: Optional[str] = None) -> Dict[str, Any]:
        """
        获取流统计信息
        
        Args:
            name: 流名称，None表示获取所有流
            
        Returns:
            Dict: 统计信息
        """
        if name:
            if name in self.streams:
                return self.streams[name].get_stats()
            else:
                return {}
        
        return {
            stream_name: stream.get_stats()
            for stream_name, stream in self.streams.items()
        }
    
    def get_status(self) -> Dict[str, Any]:
        """获取管理器状态"""
        return {
            'is_running': self.is_running,
            'total_streams': len(self.streams),
            'active_streams': len([
                name for name, task in self.tasks.items()
                if not task.done()
            ]),
            'stream_names': list(self.streams.keys())
        } 