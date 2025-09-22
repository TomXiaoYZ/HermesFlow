#!/usr/bin/env python3
"""
WebSocket连接管理器模块 (Connection Manager Module)

负责管理所有交易所的WebSocket连接，包括：
- 连接池管理和维护
- 自动断线重连机制
- 心跳检测和健康监控
- 负载均衡和故障转移
- 连接状态统计和监控

支持多交易所并发连接，确保高可用性和性能
"""

import asyncio
import time
import json
import logging
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Set, Callable, Any, Tuple
from enum import Enum
import aiohttp
import websockets
from websockets.exceptions import WebSocketException, ConnectionClosed
from collections import defaultdict, deque

from .config import SubscriptionConfig
from .models import PerformanceMetrics, StreamData, StreamDataType, DataStatus

# 设置日志
logger = logging.getLogger(__name__)

class ConnectionState(Enum):
    """连接状态枚举"""
    DISCONNECTED = "disconnected"       # 已断开
    CONNECTING = "connecting"           # 连接中
    CONNECTED = "connected"             # 已连接
    RECONNECTING = "reconnecting"       # 重连中
    ERROR = "error"                     # 错误状态
    CLOSED = "closed"                   # 已关闭

class ConnectionPriority(Enum):
    """连接优先级"""
    LOW = 1         # 低优先级
    NORMAL = 2      # 正常优先级
    HIGH = 3        # 高优先级
    CRITICAL = 4    # 关键优先级

@dataclass
class ConnectionInfo:
    """连接信息类"""
    # 基本信息
    connection_id: str
    exchange: str
    url: str
    symbols: List[str] = field(default_factory=list)
    
    # 连接状态
    state: ConnectionState = ConnectionState.DISCONNECTED
    priority: ConnectionPriority = ConnectionPriority.NORMAL
    
    # 时间信息
    created_time: float = field(default_factory=time.time)
    connected_time: Optional[float] = None
    last_ping_time: Optional[float] = None
    last_pong_time: Optional[float] = None
    last_message_time: Optional[float] = None
    
    # 统计信息
    messages_received: int = 0
    messages_sent: int = 0
    reconnect_count: int = 0
    error_count: int = 0
    
    # 网络统计
    bytes_received: int = 0
    bytes_sent: int = 0
    avg_latency_ms: float = 0.0
    
    # WebSocket对象
    websocket: Optional[websockets.WebSocketServerProtocol] = None
    
    def calculate_uptime(self) -> float:
        """计算连接运行时间(秒)"""
        if self.connected_time:
            return time.time() - self.connected_time
        return 0.0
    
    def is_healthy(self, max_idle_seconds: float = 60.0) -> bool:
        """检查连接是否健康"""
        if self.state != ConnectionState.CONNECTED:
            return False
        
        # 检查是否长时间无消息
        if self.last_message_time:
            idle_time = time.time() - self.last_message_time
            if idle_time > max_idle_seconds:
                return False
        
        return True
    
    def update_latency(self, latency_ms: float):
        """更新延迟统计"""
        if self.messages_received == 0:
            self.avg_latency_ms = latency_ms
        else:
            # 使用指数移动平均
            alpha = 0.1
            self.avg_latency_ms = alpha * latency_ms + (1 - alpha) * self.avg_latency_ms

class ConnectionPool:
    """连接池管理器"""
    
    def __init__(self, max_connections: int = 50):
        self.max_connections = max_connections
        self.connections: Dict[str, ConnectionInfo] = {}
        self.exchange_connections: Dict[str, Set[str]] = defaultdict(set)
        self.symbol_connections: Dict[str, Set[str]] = defaultdict(set)
        self._lock = asyncio.Lock()
    
    async def add_connection(self, connection: ConnectionInfo) -> bool:
        """添加连接到池中"""
        async with self._lock:
            if len(self.connections) >= self.max_connections:
                logger.warning(f"连接池已满，无法添加新连接: {connection.connection_id}")
                return False
            
            self.connections[connection.connection_id] = connection
            self.exchange_connections[connection.exchange].add(connection.connection_id)
            
            # 更新符号映射
            for symbol in connection.symbols:
                self.symbol_connections[symbol].add(connection.connection_id)
            
            logger.info(f"连接已添加到池中: {connection.connection_id}")
            return True
    
    async def remove_connection(self, connection_id: str) -> bool:
        """从池中移除连接"""
        async with self._lock:
            if connection_id not in self.connections:
                return False
            
            connection = self.connections[connection_id]
            
            # 清理映射
            self.exchange_connections[connection.exchange].discard(connection_id)
            for symbol in connection.symbols:
                self.symbol_connections[symbol].discard(connection_id)
            
            # 关闭WebSocket连接
            if connection.websocket:
                try:
                    await connection.websocket.close()
                except Exception as e:
                    logger.warning(f"关闭WebSocket连接时出错: {e}")
            
            del self.connections[connection_id]
            logger.info(f"连接已从池中移除: {connection_id}")
            return True
    
    async def get_connections_by_exchange(self, exchange: str) -> List[ConnectionInfo]:
        """获取指定交易所的所有连接"""
        async with self._lock:
            connection_ids = self.exchange_connections.get(exchange, set())
            return [self.connections[cid] for cid in connection_ids 
                   if cid in self.connections]
    
    async def get_connections_by_symbol(self, symbol: str) -> List[ConnectionInfo]:
        """获取订阅指定交易对的所有连接"""
        async with self._lock:
            connection_ids = self.symbol_connections.get(symbol, set())
            return [self.connections[cid] for cid in connection_ids 
                   if cid in self.connections]
    
    async def get_healthy_connections(self) -> List[ConnectionInfo]:
        """获取所有健康的连接"""
        async with self._lock:
            return [conn for conn in self.connections.values() if conn.is_healthy()]
    
    async def cleanup_dead_connections(self):
        """清理死连接"""
        dead_connections = []
        async with self._lock:
            for connection in self.connections.values():
                if not connection.is_healthy() and connection.state == ConnectionState.ERROR:
                    dead_connections.append(connection.connection_id)
        
        for connection_id in dead_connections:
            await self.remove_connection(connection_id)
            logger.info(f"清理死连接: {connection_id}")

class ConnectionManager:
    """WebSocket连接管理器主类"""
    
    def __init__(self, config: SubscriptionConfig):
        self.config = config
        self.pool = ConnectionPool(max_connections=config.max_connections_per_exchange * 10)
        self.running = False
        self.is_running = False
        self.active_connections = 0
        self.message_handlers: Dict[str, Callable] = {}
        self.error_handlers: Dict[str, Callable] = {}
        
        # 数据回调列表
        self.data_callbacks: List[Callable] = []
        
        # 统计信息
        self.metrics = PerformanceMetrics()
        self.start_time = time.time()
        
        # 任务管理
        self.background_tasks: Set[asyncio.Task] = set()
        self.reconnect_queue = asyncio.Queue()
        
        # 心跳和健康检查
        self.last_health_check = time.time()
        self.ping_interval = config.heartbeat_interval
        
        logger.info(f"连接管理器初始化完成，配置: {config}")
    
    async def start(self):
        """启动连接管理器"""
        if self.running:
            logger.warning("连接管理器已在运行")
            return
        
        self.running = True
        self.is_running = True
        self.start_time = time.time()
        
        # 启动后台任务
        tasks = [
            self._health_check_loop(),
            self._reconnect_loop(),
            self._metrics_collection_loop(),
            self._cleanup_loop()
        ]
        
        for task_func in tasks:
            task = asyncio.create_task(task_func)
            self.background_tasks.add(task)
            task.add_done_callback(self.background_tasks.discard)
        
        logger.info("连接管理器已启动")
    
    async def stop(self):
        """停止连接管理器"""
        if not self.running:
            return
        
        self.running = False
        self.is_running = False
        
        # 取消所有后台任务
        for task in self.background_tasks:
            task.cancel()
        
        # 等待任务完成
        if self.background_tasks:
            await asyncio.gather(*self.background_tasks, return_exceptions=True)
        
        # 关闭所有连接
        connection_ids = list(self.pool.connections.keys())
        for connection_id in connection_ids:
            await self.pool.remove_connection(connection_id)
        
        # 重置连接计数
        self.active_connections = 0
        
        logger.info("连接管理器已停止")
    
    async def create_connection(self, exchange: str, url: str, symbols: List[str],
                              priority: ConnectionPriority = ConnectionPriority.NORMAL) -> Optional[str]:
        """创建新的WebSocket连接"""
        connection_id = f"{exchange}_{int(time.time() * 1000)}"
        
        try:
            # 创建连接信息
            connection_info = ConnectionInfo(
                connection_id=connection_id,
                exchange=exchange,
                url=url,
                symbols=symbols,
                priority=priority
            )
            
            # 建立WebSocket连接
            websocket = await self._establish_websocket(url, connection_info)
            if not websocket:
                return None
            
            connection_info.websocket = websocket
            connection_info.state = ConnectionState.CONNECTED
            connection_info.connected_time = time.time()
            
            # 添加到连接池
            success = await self.pool.add_connection(connection_info)
            if not success:
                await websocket.close()
                return None
            
            # 注册消息处理器
            await self._register_exchange_handlers(connection_info)
            
            # 启动消息处理循环
            task = asyncio.create_task(
                self._message_handler_loop(connection_info)
            )
            self.background_tasks.add(task)
            task.add_done_callback(self.background_tasks.discard)
            
            # 发送订阅消息（如果需要）
            await self._send_subscription_messages(connection_info)
            
            logger.info(f"WebSocket连接创建成功: {connection_id}")
            return connection_id
            
        except Exception as e:
            logger.error(f"WebSocket连接异常: {e}")
            return None
    
    async def _register_exchange_handlers(self, connection_info: ConnectionInfo):
        """为特定交易所注册消息处理器"""
        exchange = connection_info.exchange.lower()
        
        if exchange == 'binance':
            # 注册Binance消息处理器
            handler_key = f"binance_{connection_info.symbols[0] if connection_info.symbols else 'default'}"
            self.register_message_handler(handler_key, self._handle_binance_message)
            logger.info(f"已注册Binance消息处理器: {handler_key}")
        elif exchange == 'okx':
            # 注册OKX消息处理器
            handler_key = f"okx_{connection_info.symbols[0] if connection_info.symbols else 'default'}"
            self.register_message_handler(handler_key, self._handle_okx_message)
            logger.info(f"已注册OKX消息处理器: {handler_key}")
        elif exchange == 'bitget':
            # 注册Bitget消息处理器
            handler_key = f"bitget_{connection_info.symbols[0] if connection_info.symbols else 'default'}"
            self.register_message_handler(handler_key, self._handle_bitget_message)
            logger.info(f"已注册Bitget消息处理器: {handler_key}")
    
    async def _send_subscription_messages(self, connection_info: ConnectionInfo):
        """发送订阅消息"""
        exchange = connection_info.exchange.lower()
        
        if exchange == 'binance':
            # Binance使用组合流，不需要额外的订阅消息
            # 因为订阅信息已经在URL中指定
            logger.info(f"Binance连接使用组合流，无需发送订阅消息")
        elif exchange == 'okx':
            # OKX需要发送订阅消息
            await self._send_okx_subscription(connection_info)
        elif exchange == 'bitget':
            # Bitget需要发送订阅消息
            await self._send_bitget_subscription(connection_info)
    
    async def _handle_binance_message(self, data: dict, connection_info: ConnectionInfo):
        """处理Binance WebSocket消息"""
        try:
            # 检查是否为组合流消息格式: {"stream":"btcusdt@ticker","data":{...}}
            if 'stream' in data and 'data' in data:
                stream_name = data['stream']
                stream_data = data['data']
                
                # 解析流名称
                parts = stream_name.split('@')
                if len(parts) >= 2:
                    symbol = parts[0].upper()
                    stream_type = parts[1]
                    
                    logger.debug(f"收到Binance组合流消息: {symbol} - {stream_type}")
                    
                    # 根据流类型确定数据类型
                    if stream_type == 'ticker':
                        data_type = StreamDataType.MARKET_DATA
                    elif stream_type == 'trade':
                        data_type = StreamDataType.TRADE_DATA
                        # 添加调试日志显示 trade 数据结构
                        logger.debug(f"Trade 数据结构: {stream_data}")
                    elif stream_type.startswith('depth'):
                        data_type = StreamDataType.ORDER_BOOK
                    else:
                        data_type = StreamDataType.MARKET_DATA
                    
                    # 创建StreamData对象并传递给数据处理器
                    stream_obj = StreamData(
                        id=f"{connection_info.connection_id}_{int(time.time() * 1000)}",
                        symbol=symbol,
                        source=connection_info.exchange,
                        data_type=data_type,
                        data=stream_data,
                        status=DataStatus.PENDING
                    )
                    
                    # 触发数据回调
                    await self._trigger_data_callbacks(stream_obj)
                    
                    # 这里可以添加数据处理逻辑
                    if stream_type == 'ticker':
                        logger.info(f"处理Binance组合流数据: {symbol} - {stream_type} - 价格: {stream_data.get('c', 'N/A')}")
                    elif stream_type == 'trade':
                        logger.info(f"处理Binance组合流数据: {symbol} - {stream_type} - 价格: {stream_data.get('p', 'N/A')}")
                    else:
                        logger.info(f"处理Binance组合流数据: {symbol} - {stream_type} - 价格: N/A")
                    
            # 检查是否为单个流消息格式: {"e":"24hrTicker","s":"BTCUSDT",...}
            elif 'e' in data and 's' in data:
                event_type = data['e']
                symbol = data['s']
                
                logger.debug(f"收到Binance单个流消息: {symbol} - {event_type}")
                
                # 根据事件类型确定数据类型
                if event_type == '24hrTicker':
                    data_type = StreamDataType.MARKET_DATA
                elif event_type == 'trade':
                    data_type = StreamDataType.TRADE_DATA
                elif event_type == 'depthUpdate':
                    data_type = StreamDataType.ORDER_BOOK
                else:
                    data_type = StreamDataType.MARKET_DATA
                
                # 创建StreamData对象
                stream_obj = StreamData(
                    id=f"{connection_info.connection_id}_{int(time.time() * 1000)}",
                    symbol=symbol,
                    source=connection_info.exchange,
                    data_type=data_type,
                    data=data,
                    status=DataStatus.PENDING
                )
                
                # 触发数据回调
                await self._trigger_data_callbacks(stream_obj)
                
                # 这里可以添加数据处理逻辑
                logger.info(f"处理Binance单个流数据: {symbol} - {event_type} - 价格: {data.get('c', 'N/A')}")
                
            else:
                logger.warning(f"未知的Binance消息格式: {data}")
                
        except Exception as e:
            logger.error(f"处理Binance消息时出错: {e}")
    
    async def _handle_okx_message(self, data: dict, connection_info: ConnectionInfo):
        """处理OKX WebSocket消息"""
        try:
            # OKX消息格式: {"arg":{"channel":"tickers","instId":"BTC-USDT"},"data":[{...}]}
            if 'arg' in data and 'data' in data:
                arg = data['arg']
                channel = arg.get('channel', '')
                inst_id = arg.get('instId', '')
                data_list = data.get('data', [])
                
                if not data_list:
                    logger.warning(f"OKX消息数据为空: {data}")
                    return
                
                # 取第一条数据
                stream_data = data_list[0]
                symbol = inst_id.replace('-', '').upper()  # 转换为标准格式
                
                logger.debug(f"收到OKX消息: {symbol} - {channel}")
                
                # 根据频道类型确定数据类型
                if channel == 'tickers':
                    data_type = StreamDataType.MARKET_DATA
                elif channel == 'trades':
                    data_type = StreamDataType.TRADE_DATA
                    # 添加调试日志显示 trade 数据结构
                    logger.debug(f"OKX Trade 数据结构: {stream_data}")
                elif channel == 'books' or channel.startswith('books'):
                    data_type = StreamDataType.ORDER_BOOK
                elif channel.startswith('candle'):
                    data_type = StreamDataType.MARKET_DATA  # K线数据归类为市场数据
                else:
                    data_type = StreamDataType.MARKET_DATA
                    logger.warning(f"未知的OKX频道类型: {channel}")
                
                # 创建StreamData对象
                stream_obj = StreamData(
                    id=f"{connection_info.connection_id}_{int(time.time() * 1000)}",
                    symbol=symbol,
                    source=connection_info.exchange,
                    data_type=data_type,
                    data=stream_data,
                    status=DataStatus.PENDING
                )
                
                # 触发数据回调
                await self._trigger_data_callbacks(stream_obj)
                
                # 记录处理日志
                if channel == 'tickers':
                    logger.info(f"处理OKX数据: {symbol} - {channel} - 价格: {stream_data.get('last', 'N/A')}")
                elif channel == 'trades':
                    logger.info(f"处理OKX数据: {symbol} - {channel} - 价格: {stream_data.get('px', 'N/A')}")
                else:
                    logger.info(f"处理OKX数据: {symbol} - {channel}")
                    
            else:
                logger.warning(f"未知的OKX消息格式: {data}")
                
        except Exception as e:
            logger.error(f"处理OKX消息时出错: {e}")
    
    async def _handle_bitget_message(self, data: dict, connection_info: ConnectionInfo):
        """处理Bitget WebSocket消息"""
        try:
            # Bitget消息格式通常为: {"action":"snapshot","arg":{"instType":"sp","channel":"ticker","instId":"BTCUSDT"},"data":[{...}]}
            if 'action' in data and 'arg' in data and 'data' in data:
                action = data.get('action', '')
                arg = data.get('arg', {})
                channel = arg.get('channel', '')
                inst_id = arg.get('instId', '')
                data_list = data.get('data', [])
                
                if not data_list:
                    logger.warning(f"Bitget消息数据为空: {data}")
                    return
                
                # 取第一条数据
                stream_data = data_list[0]
                symbol = inst_id.upper()  # Bitget通常使用BTCUSDT格式
                
                logger.debug(f"收到Bitget消息: {symbol} - {channel} - {action}")
                
                # 根据频道类型确定数据类型
                if channel == 'ticker':
                    data_type = StreamDataType.MARKET_DATA
                elif channel == 'trade':
                    data_type = StreamDataType.TRADE_DATA
                    # 添加调试日志显示 trade 数据结构
                    logger.debug(f"Bitget Trade 数据结构: {stream_data}")
                elif channel == 'books' or channel.startswith('books'):
                    data_type = StreamDataType.ORDER_BOOK
                elif channel.startswith('candle'):
                    data_type = StreamDataType.MARKET_DATA  # K线数据归类为市场数据
                else:
                    data_type = StreamDataType.MARKET_DATA
                    logger.warning(f"未知的Bitget频道类型: {channel}")
                
                # 创建StreamData对象
                stream_obj = StreamData(
                    id=f"{connection_info.connection_id}_{int(time.time() * 1000)}",
                    symbol=symbol,
                    source=connection_info.exchange,
                    data_type=data_type,
                    data=stream_data,
                    status=DataStatus.PENDING
                )
                
                # 触发数据回调
                await self._trigger_data_callbacks(stream_obj)
                
                # 记录处理日志
                if channel == 'ticker':
                    logger.info(f"处理Bitget数据: {symbol} - {channel} - 价格: {stream_data.get('close', 'N/A')}")
                elif channel == 'trade':
                    logger.info(f"处理Bitget数据: {symbol} - {channel} - 价格: {stream_data.get('price', 'N/A')}")
                else:
                    logger.info(f"处理Bitget数据: {symbol} - {channel}")
                    
            else:
                logger.warning(f"未知的Bitget消息格式: {data}")
                
        except Exception as e:
            logger.error(f"处理Bitget消息时出错: {e}")

    async def _send_okx_subscription(self, connection_info: ConnectionInfo):
        """发送OKX订阅消息"""
        try:
            # OKX订阅消息格式
            subscriptions = []
            
            for symbol in connection_info.symbols:
                # 转换为OKX格式 (例如: BTCUSDT -> BTC-USDT)
                okx_symbol = f"{symbol[:-4]}-{symbol[-4:]}" if symbol.endswith('USDT') else symbol
                
                # 订阅ticker数据
                subscriptions.append({
                    "op": "subscribe",
                    "args": [{
                        "channel": "tickers",
                        "instId": okx_symbol
                    }]
                })
                
                # 订阅trade数据
                subscriptions.append({
                    "op": "subscribe", 
                    "args": [{
                        "channel": "trades",
                        "instId": okx_symbol
                    }]
                })
            
            # 发送订阅消息
            for subscription in subscriptions:
                if connection_info.websocket:
                    await connection_info.websocket.send(json.dumps(subscription))
                    logger.info(f"发送OKX订阅: {subscription}")
                    
        except Exception as e:
            logger.error(f"发送OKX订阅消息时出错: {e}")

    async def _send_bitget_subscription(self, connection_info: ConnectionInfo):
        """发送Bitget订阅消息"""
        try:
            # Bitget订阅消息格式
            subscriptions = []
            
            for symbol in connection_info.symbols:
                # Bitget通常使用BTCUSDT格式，无需转换
                
                # 订阅ticker数据
                subscriptions.append({
                    "op": "subscribe",
                    "args": [{
                        "instType": "sp",  # spot
                        "channel": "ticker",
                        "instId": symbol.upper()
                    }]
                })
                
                # 订阅trade数据
                subscriptions.append({
                    "op": "subscribe", 
                    "args": [{
                        "instType": "sp",  # spot
                        "channel": "trade",
                        "instId": symbol.upper()
                    }]
                })
            
            # 发送订阅消息
            for subscription in subscriptions:
                if connection_info.websocket:
                    await connection_info.websocket.send(json.dumps(subscription))
                    logger.info(f"发送Bitget订阅: {subscription}")
                    
        except Exception as e:
            logger.error(f"发送Bitget订阅消息时出错: {e}")
    
    async def _establish_websocket(self, url: str, connection_info: ConnectionInfo) -> Optional[websockets.WebSocketServerProtocol]:
        """建立WebSocket连接"""
        try:
            # 根据URL协议自动判断是否使用SSL
            if url.startswith('wss://'):
                ssl_context = True  # 使用默认SSL上下文
            else:
                ssl_context = None
            
            # 建立连接，使用asyncio.wait_for来实现超时控制
            websocket = await asyncio.wait_for(
                websockets.connect(
                    url,
                    ssl=ssl_context,
                    ping_interval=self.ping_interval,
                    ping_timeout=self.ping_interval / 2
                ),
                timeout=self.config.connection_timeout
            )
            
            logger.info(f"WebSocket连接已建立: {connection_info.connection_id} -> {url}")
            return websocket
            
        except asyncio.TimeoutError:
            logger.error(f"WebSocket连接超时: {url}")
        except WebSocketException as e:
            logger.error(f"WebSocket连接异常: {e}")
        except Exception as e:
            logger.error(f"建立WebSocket连接时出现未知错误: {e}")
        
        return None
    
    async def _message_handler_loop(self, connection_info: ConnectionInfo):
        """消息处理循环"""
        connection_id = connection_info.connection_id
        websocket = connection_info.websocket
        
        try:
            async for message in websocket:
                try:
                    # 更新统计信息
                    connection_info.messages_received += 1
                    connection_info.bytes_received += len(message)
                    connection_info.last_message_time = time.time()
                    
                    # 解析消息
                    if isinstance(message, str):
                        data = json.loads(message)
                    else:
                        data = message
                    
                    # 调用消息处理器
                    handler_key = f"{connection_info.exchange}_{connection_info.symbols[0] if connection_info.symbols else 'default'}"
                    if handler_key in self.message_handlers:
                        await self.message_handlers[handler_key](data, connection_info)
                    
                    # 更新指标
                    self.metrics.messages_per_second += 1
                    
                except json.JSONDecodeError as e:
                    logger.warning(f"JSON解析错误: {e}")
                    connection_info.error_count += 1
                except Exception as e:
                    logger.error(f"处理消息时出错: {e}")
                    connection_info.error_count += 1
                    
        except ConnectionClosed:
            logger.warning(f"WebSocket连接已关闭: {connection_id}")
            connection_info.state = ConnectionState.DISCONNECTED
        except Exception as e:
            logger.error(f"消息处理循环异常: {e}")
            connection_info.state = ConnectionState.ERROR
            connection_info.error_count += 1
        
        # 连接断开，加入重连队列
        if connection_info.state != ConnectionState.CLOSED:
            await self.reconnect_queue.put(connection_info)
    
    async def _health_check_loop(self):
        """健康检查循环"""
        while self.running:
            try:
                await asyncio.sleep(30)  # 每30秒检查一次
                
                current_time = time.time()
                self.last_health_check = current_time
                
                # 检查所有连接的健康状态
                unhealthy_connections = []
                async with self.pool._lock:
                    for connection in self.pool.connections.values():
                        if not connection.is_healthy():
                            unhealthy_connections.append(connection)
                
                # 处理不健康的连接
                for connection in unhealthy_connections:
                    logger.warning(f"检测到不健康连接: {connection.connection_id}")
                    connection.state = ConnectionState.ERROR
                    await self.reconnect_queue.put(connection)
                
                # 清理死连接
                await self.pool.cleanup_dead_connections()
                
                logger.debug(f"健康检查完成，活跃连接数: {len(self.pool.connections)}")
                
            except Exception as e:
                logger.error(f"健康检查循环异常: {e}")
    
    async def _reconnect_loop(self):
        """重连处理循环"""
        while self.running:
            try:
                # 等待需要重连的连接
                connection_info = await asyncio.wait_for(
                    self.reconnect_queue.get(), timeout=1.0
                )
                
                # 检查是否需要重连
                if (connection_info.reconnect_count >= self.config.max_reconnect_attempts or
                    connection_info.state == ConnectionState.CLOSED):
                    logger.warning(f"连接重连次数过多或已关闭，不再重连: {connection_info.connection_id}")
                    await self.pool.remove_connection(connection_info.connection_id)
                    continue
                
                # 执行重连
                await self._perform_reconnect(connection_info)
                
            except asyncio.TimeoutError:
                continue
            except Exception as e:
                logger.error(f"重连循环异常: {e}")
    
    async def _perform_reconnect(self, connection_info: ConnectionInfo):
        """执行重连"""
        connection_id = connection_info.connection_id
        logger.info(f"开始重连: {connection_id}")
        
        try:
            # 等待重连延迟
            await asyncio.sleep(self.config.reconnect_delay * (connection_info.reconnect_count + 1))
            
            # 关闭旧连接
            if connection_info.websocket:
                try:
                    await connection_info.websocket.close()
                except:
                    pass
            
            # 建立新连接
            connection_info.state = ConnectionState.RECONNECTING
            websocket = await self._establish_websocket(connection_info.url, connection_info)
            
            if websocket:
                connection_info.websocket = websocket
                connection_info.state = ConnectionState.CONNECTED
                connection_info.connected_time = time.time()
                connection_info.reconnect_count += 1
                
                # 重新启动消息处理
                task = asyncio.create_task(
                    self._message_handler_loop(connection_info)
                )
                self.background_tasks.add(task)
                task.add_done_callback(self.background_tasks.discard)
                
                # 更新统计
                self.metrics.reconnections += 1
                
                logger.info(f"重连成功: {connection_id}")
            else:
                connection_info.state = ConnectionState.ERROR
                connection_info.reconnect_count += 1
                logger.error(f"重连失败: {connection_id}")
                
                # 重新加入队列
                await self.reconnect_queue.put(connection_info)
                
        except Exception as e:
            logger.error(f"重连过程中出错: {e}")
            connection_info.state = ConnectionState.ERROR
            connection_info.reconnect_count += 1
    
    async def _metrics_collection_loop(self):
        """指标收集循环"""
        last_messages = 0
        last_time = time.time()
        
        while self.running:
            try:
                await asyncio.sleep(5)  # 每5秒收集一次
                
                current_time = time.time()
                
                # 计算吞吐量
                time_diff = current_time - last_time
                if time_diff > 0:
                    total_messages = sum(conn.messages_received for conn in self.pool.connections.values())
                    messages_diff = total_messages - last_messages
                    self.metrics.messages_per_second = messages_diff / time_diff
                    
                    last_messages = total_messages
                    last_time = current_time
                
                # 更新连接统计
                self.metrics.active_connections = len([
                    conn for conn in self.pool.connections.values() 
                    if conn.state == ConnectionState.CONNECTED
                ])
                
                # 计算运行时间
                self.metrics.uptime_seconds = current_time - self.start_time
                
                # 更新最后更新时间
                self.metrics.last_update = current_time
                
            except Exception as e:
                logger.error(f"指标收集异常: {e}")
    
    async def _cleanup_loop(self):
        """清理循环"""
        while self.running:
            try:
                await asyncio.sleep(300)  # 每5分钟清理一次
                
                # 清理死连接
                await self.pool.cleanup_dead_connections()
                
                # 清理完成的任务
                completed_tasks = [task for task in self.background_tasks if task.done()]
                for task in completed_tasks:
                    self.background_tasks.discard(task)
                
                logger.debug("清理循环完成")
                
            except Exception as e:
                logger.error(f"清理循环异常: {e}")
    
    def register_message_handler(self, key: str, handler: Callable):
        """注册消息处理器"""
        self.message_handlers[key] = handler
        logger.info(f"消息处理器已注册: {key}")
    
    def register_error_handler(self, key: str, handler: Callable):
        """注册错误处理器"""
        self.error_handlers[key] = handler
        logger.info(f"错误处理器已注册: {key}")
    
    async def send_message(self, connection_id: str, message: Dict[str, Any]) -> bool:
        """发送消息到指定连接"""
        try:
            if connection_id not in self.pool.connections:
                logger.warning(f"连接不存在: {connection_id}")
                return False
            
            connection = self.pool.connections[connection_id]
            if connection.state != ConnectionState.CONNECTED or not connection.websocket:
                logger.warning(f"连接状态异常: {connection_id}")
                return False
            
            message_str = json.dumps(message)
            await connection.websocket.send(message_str)
            
            # 更新统计
            connection.messages_sent += 1
            connection.bytes_sent += len(message_str)
            
            return True
            
        except Exception as e:
            logger.error(f"发送消息失败: {e}")
            return False
    
    async def get_statistics(self) -> Dict[str, Any]:
        """获取连接统计信息"""
        stats = {
            'total_connections': len(self.pool.connections),
            'active_connections': len([
                conn for conn in self.pool.connections.values()
                if conn.state == ConnectionState.CONNECTED
            ]),
            'connections_by_exchange': {},
            'connections_by_state': defaultdict(int),
            'total_messages_received': sum(
                conn.messages_received for conn in self.pool.connections.values()
            ),
            'total_messages_sent': sum(
                conn.messages_sent for conn in self.pool.connections.values()
            ),
            'total_reconnections': self.metrics.reconnections,
            'uptime_seconds': self.metrics.uptime_seconds,
            'messages_per_second': self.metrics.messages_per_second
        }
        
        # 按交易所统计
        for connection in self.pool.connections.values():
            exchange = connection.exchange
            if exchange not in stats['connections_by_exchange']:
                stats['connections_by_exchange'][exchange] = 0
            stats['connections_by_exchange'][exchange] += 1
            
            # 按状态统计
            stats['connections_by_state'][connection.state.value] += 1
        
        return stats

    def add_data_callback(self, callback: Callable):
        """添加数据回调函数"""
        if callback not in self.data_callbacks:
            self.data_callbacks.append(callback)
            logger.info(f"添加数据回调函数: {callback.__name__}")

    def remove_data_callback(self, callback: Callable):
        """移除数据回调函数"""
        if callback in self.data_callbacks:
            self.data_callbacks.remove(callback)
            logger.info(f"移除数据回调函数: {callback.__name__}")

    async def _trigger_data_callbacks(self, stream_data):
        """触发所有数据回调函数"""
        for callback in self.data_callbacks:
            try:
                if asyncio.iscoroutinefunction(callback):
                    await callback(stream_data)
                else:
                    callback(stream_data)
            except Exception as e:
                logger.error(f"数据回调函数执行失败: {e}") 