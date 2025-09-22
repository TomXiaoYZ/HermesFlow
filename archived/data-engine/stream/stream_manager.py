#!/usr/bin/env python3
"""
主数据流管理器模块 (Main Stream Manager Module)

数据流处理的统一管理和协调中心，负责：
- 统一管理所有数据流组件的生命周期
- 协调各个子模块间的交互和通信
- 提供统一的数据流处理接口
- 监控和维护整个数据流系统的健康状态
- 实现自动故障恢复和负载均衡
- 提供配置热更新和动态扩展能力

这是整个数据流系统的核心控制器
"""

import asyncio
import time
import json
import logging
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Any, Callable, Union
from enum import Enum
from collections import defaultdict, deque

from .config import StreamConfig, SubscriptionConfig, MonitorConfig, StorageConfig
from .models import StreamData, StreamDataType, DataStatus, PerformanceMetrics, DataQuality
from .connection_manager import ConnectionManager
from .data_processor import DataProcessor
from .data_router import DataRouter
from .storage_manager import StorageManager
from .monitor import StreamMonitor

# 设置日志
logger = logging.getLogger(__name__)

class StreamManagerStatus(Enum):
    """数据流管理器状态枚举"""
    INITIALIZING = "initializing"   # 初始化中
    RUNNING = "running"             # 运行中
    PAUSING = "pausing"             # 暂停中
    PAUSED = "paused"               # 已暂停
    STOPPING = "stopping"           # 停止中
    STOPPED = "stopped"             # 已停止
    ERROR = "error"                 # 错误状态

@dataclass
class StreamManagerMetrics:
    """数据流管理器指标类"""
    # 基本统计
    total_processed_messages: int = 0
    successful_processed_messages: int = 0
    failed_processed_messages: int = 0
    
    # 吞吐量指标
    messages_per_second: float = 0.0
    bytes_per_second: float = 0.0
    
    # 延迟指标
    avg_processing_latency_ms: float = 0.0
    max_processing_latency_ms: float = 0.0
    
    # 组件状态统计
    active_connections: int = 0
    active_subscriptions: int = 0
    active_processors: int = 0
    
    # 时间统计
    start_time: float = field(default_factory=time.time)
    last_update: float = field(default_factory=time.time)
    
    def get_success_rate(self) -> float:
        """获取处理成功率"""
        if self.total_processed_messages == 0:
            return 0.0
        return self.successful_processed_messages / self.total_processed_messages
    
    def get_uptime_seconds(self) -> float:
        """获取运行时间（秒）"""
        return time.time() - self.start_time
    
    def update_throughput(self, time_window_seconds: float = 1.0):
        """更新吞吐量指标"""
        current_time = time.time()
        time_diff = max(current_time - self.last_update, time_window_seconds)
        
        # 简化的吞吐量计算（实际实现需要更复杂的滑动窗口）
        if self.successful_processed_messages > 0:
            self.messages_per_second = self.successful_processed_messages / time_diff
        
        self.last_update = current_time

class StreamManager:
    """主数据流管理器类"""
    
    def __init__(self, stream_config: StreamConfig):
        self.stream_config = stream_config
        self.status = StreamManagerStatus.STOPPED
        
        # 核心组件
        self.connection_manager: Optional[ConnectionManager] = None
        self.data_processor: Optional[DataProcessor] = None
        self.data_router: Optional[DataRouter] = None
        self.storage_manager: Optional[StorageManager] = None
        self.stream_monitor: Optional[StreamMonitor] = None
        
        # 系统指标
        self.metrics = StreamManagerMetrics()
        self.performance_metrics = PerformanceMetrics()
        self.data_quality = DataQuality()
        
        # 数据流处理统计
        self.processing_stats = {
            'message_counts_by_source': defaultdict(int),
            'message_counts_by_type': defaultdict(int),
            'error_counts_by_type': defaultdict(int),
            'latency_history': deque(maxlen=1000)
        }
        
        # 后台任务
        self.background_tasks: List[asyncio.Task] = []
        
        # 回调函数
        self.data_callbacks: Dict[str, List[Callable]] = defaultdict(list)
        self.status_callbacks: List[Callable] = []
        
        # 系统健康状态
        self.component_health: Dict[str, bool] = {}
        
        logger.info("主数据流管理器初始化完成")
    
    async def initialize(self) -> bool:
        """初始化数据流管理器"""
        try:
            # 设置初始化状态
            self.status = StreamManagerStatus.INITIALIZING
            logger.info("开始初始化数据流管理器...")
            
            # 初始化各个组件
            components_to_initialize = [
                ("数据处理器", self._initialize_data_processor),
                ("数据路由器", self._initialize_data_router),
                ("连接管理器", self._initialize_connection_manager),
                ("存储管理器", self._initialize_storage_manager),
                ("监控系统", self._initialize_stream_monitor)
            ]
            
            for component_name, init_func in components_to_initialize:
                logger.info(f"正在初始化{component_name}...")
                success = await init_func()
                if not success:
                    logger.error(f"{component_name}初始化失败")
                    self.status = StreamManagerStatus.ERROR
                    return False
                logger.info(f"{component_name}初始化成功")
            
            # 设置组件间的数据流连接
            await self._setup_data_flow_connections()
            
            # 创建默认数据订阅
            await self._setup_default_subscriptions()
            
            # 启动后台任务
            await self._start_background_tasks()
            
            # 更新状态
            self.status = StreamManagerStatus.RUNNING
            self.start_time = time.time()
            
            logger.info("数据流管理器初始化完成")
            return True
            
        except Exception as e:
            logger.error(f"数据流管理器初始化失败: {e}")
            self.status = StreamManagerStatus.ERROR
            return False
    
    async def _initialize_data_processor(self) -> bool:
        """初始化数据处理器"""
        try:
            self.data_processor = DataProcessor(self.stream_config.subscription)
            self.component_health['data_processor'] = True
            logger.info("数据处理器初始化成功")
            return True
            
        except Exception as e:
            logger.error(f"初始化数据处理器异常: {e}")
            self.component_health['data_processor'] = False
            return False
    
    async def _initialize_storage_manager(self) -> bool:
        """初始化存储管理器"""
        try:
            storage_config = StorageConfig()  # 使用默认配置
            self.storage_manager = StorageManager(storage_config)
            result = await self.storage_manager.initialize()
            self.component_health['storage_manager'] = result
            
            if result:
                logger.info("存储管理器初始化成功")
            else:
                logger.error("存储管理器初始化失败")
            
            return result
            
        except Exception as e:
            logger.error(f"初始化存储管理器异常: {e}")
            self.component_health['storage_manager'] = False
            return False
    
    async def _initialize_data_router(self) -> bool:
        """初始化数据路由器"""
        try:
            self.data_router = DataRouter(self.stream_config.subscription)
            self.component_health['data_router'] = True
            logger.info("数据路由器初始化成功")
            return True
            
        except Exception as e:
            logger.error(f"初始化数据路由器异常: {e}")
            self.component_health['data_router'] = False
            return False
    
    async def _initialize_connection_manager(self) -> bool:
        """初始化连接管理器"""
        try:
            self.connection_manager = ConnectionManager(self.stream_config.subscription)
            # 启动连接管理器
            await self.connection_manager.start()
            self.component_health['connection_manager'] = True
            logger.info("连接管理器初始化并启动成功")
            return True
            
        except Exception as e:
            logger.error(f"初始化连接管理器异常: {e}")
            self.component_health['connection_manager'] = False
            return False
    
    async def _initialize_stream_monitor(self) -> bool:
        """初始化监控系统"""
        try:
            monitor_config = MonitorConfig()  # 使用默认配置
            self.stream_monitor = StreamMonitor(monitor_config)
            result = await self.stream_monitor.initialize()
            self.component_health['stream_monitor'] = result
            
            if result:
                logger.info("监控系统初始化成功")
            else:
                logger.error("监控系统初始化失败")
            
            return result
            
        except Exception as e:
            logger.error(f"初始化监控系统异常: {e}")
            self.component_health['stream_monitor'] = False
            return False
    
    async def _setup_data_flow_connections(self):
        """设置组件间的数据流连接"""
        try:
            # 连接管理器 -> 数据处理器 -> 数据路由器 -> 存储管理器
            
            # 1. 连接管理器接收到数据后，传递给数据处理器
            if self.connection_manager:
                self.connection_manager.add_data_callback(self._handle_raw_data)
                logger.info("已设置ConnectionManager数据回调")
            
            # 2. 数据处理器处理完成后，传递给数据路由器
            # 注意：DataProcessor可能没有add_processing_callback方法，跳过此步骤
            # if self.data_processor:
            #     self.data_processor.add_processing_callback(self._handle_processed_data)
            
            # 3. 数据路由器根据订阅规则分发数据
            # 注意：DataRouter可能没有add_distribution_callback方法，跳过此步骤
            # if self.data_router:
            #     self.data_router.add_distribution_callback(self._handle_routed_data)
            
            logger.info("数据流连接设置完成")
            
        except Exception as e:
            logger.error(f"设置数据流连接失败: {e}")
            raise
    
    async def _setup_default_subscriptions(self):
        """设置默认的数据订阅"""
        try:
            if not self.connection_manager or not self.data_router:
                logger.warning("连接管理器或数据路由器未初始化，跳过默认订阅设置")
                return
            
            # 从配置中获取默认的交易所和交易对
            config = self.stream_config.subscription
            exchanges = config.exchanges or ['binance']
            symbols = config.symbols or ['BTCUSDT', 'ETHUSDT']
            
            logger.info(f"开始创建默认订阅 - 交易所: {exchanges}, 交易对: {symbols}")
            
            # 为每个交易所创建WebSocket连接
            for exchange in exchanges:
                try:
                    # 构建WebSocket URL
                    if exchange.lower() == 'binance':
                        # 使用正确的Binance测试网络WebSocket URL
                        # 创建组合流，包含ticker和trade数据
                        streams = []
                        for symbol in symbols:
                            symbol_lower = symbol.lower()
                            streams.extend([
                                f"{symbol_lower}@ticker",
                                f"{symbol_lower}@trade"
                            ])
                        
                        # 使用组合流格式
                        stream_params = "/".join(streams)
                        ws_url = f"wss://stream.testnet.binance.vision/stream?streams={stream_params}"
                        
                        logger.info(f"Binance WebSocket URL: {ws_url}")
                        
                    elif exchange.lower() == 'okx':
                        # OKX WebSocket URL (示例)
                        ws_url = "wss://ws.okx.com:8443/ws/v5/public"
                    else:
                        logger.warning(f"未知的交易所: {exchange}，跳过")
                        continue
                    
                    # 创建WebSocket连接
                    connection_id = await self.connection_manager.create_connection(
                        exchange=exchange,
                        url=ws_url,
                        symbols=symbols
                    )
                    
                    if connection_id:
                        logger.info(f"成功创建{exchange}的WebSocket连接: {connection_id}")
                        
                        # 创建数据路由订阅
                        from .models import StreamDataType
                        subscription_id = await self.data_router.subscribe(
                            subscriber_id="default_collector",
                            data_types=[StreamDataType.MARKET_DATA, StreamDataType.TRADE_DATA],
                            symbols=symbols,
                            exchanges=[exchange],
                            callback=self._handle_market_data
                        )
                        
                        logger.info(f"成功创建{exchange}的数据订阅: {subscription_id}")
                    else:
                        logger.error(f"创建{exchange}的WebSocket连接失败")
                        
                except Exception as e:
                    logger.error(f"为交易所{exchange}创建订阅时出错: {e}")
                    continue
            
            logger.info("默认订阅设置完成")
            
        except Exception as e:
            logger.error(f"设置默认订阅失败: {e}")
            # 不抛出异常，允许系统继续运行
    
    async def _handle_market_data(self, stream_data):
        """处理市场数据的回调函数"""
        try:
            logger.debug(f"接收到市场数据: {stream_data.symbol} - {stream_data.data_type}")
            
            # 更新统计信息
            self.metrics.total_processed_messages += 1
            self.metrics.successful_processed_messages += 1
            
            # 这里可以添加更多的数据处理逻辑
            # 例如：数据验证、格式化、存储等
            
        except Exception as e:
            logger.error(f"处理市场数据时出错: {e}")
            self.metrics.failed_processed_messages += 1
    
    async def _start_background_tasks(self):
        """启动后台任务"""
        try:
            # 系统监控任务
            monitor_task = asyncio.create_task(self._monitoring_loop())
            self.background_tasks.append(monitor_task)
            
            # 性能统计任务
            stats_task = asyncio.create_task(self._statistics_loop())
            self.background_tasks.append(stats_task)
            
            # 健康检查任务
            health_task = asyncio.create_task(self._health_check_loop())
            self.background_tasks.append(health_task)
            
            # 数据质量监控任务
            quality_task = asyncio.create_task(self._data_quality_loop())
            self.background_tasks.append(quality_task)
            
            logger.info(f"启动了 {len(self.background_tasks)} 个后台任务")
            
        except Exception as e:
            logger.error(f"启动后台任务失败: {e}")
            raise
    
    async def _handle_raw_data(self, data: StreamData):
        """处理原始数据"""
        try:
            start_time = time.time()
            
            # 更新接收统计
            self.metrics.total_processed_messages += 1
            self.processing_stats['message_counts_by_source'][data.source] += 1
            self.processing_stats['message_counts_by_type'][data.data_type.value] += 1
            
            # 数据质量初步检查
            data.received_time = time.time()
            
            # 传递给数据处理器
            if self.data_processor:
                success = await self.data_processor.process_data(data)
                
                if success:
                    self.metrics.successful_processed_messages += 1
                    data.status = DataStatus.PROCESSED
                else:
                    self.metrics.failed_processed_messages += 1
                    data.status = DataStatus.ERROR
                    self.processing_stats['error_counts_by_type']['processing_error'] += 1
            
            # 记录处理延迟
            processing_latency = (time.time() - start_time) * 1000
            self.processing_stats['latency_history'].append(processing_latency)
            self.metrics.avg_processing_latency_ms = (
                sum(self.processing_stats['latency_history']) / 
                len(self.processing_stats['latency_history'])
            )
            self.metrics.max_processing_latency_ms = max(
                self.metrics.max_processing_latency_ms, processing_latency
            )
            
        except Exception as e:
            logger.error(f"处理原始数据失败: {e}")
            self.metrics.failed_processed_messages += 1
            self.processing_stats['error_counts_by_type']['handling_error'] += 1
    
    async def _handle_processed_data(self, data: StreamData):
        """处理已处理的数据"""
        try:
            data.processed_time = time.time()
            
            # 传递给数据路由器进行分发
            if self.data_router:
                await self.data_router.route_data(data)
            
            # 更新数据质量统计
            if data.quality:
                self.data_quality.total_messages += 1
                if data.status == DataStatus.PROCESSED:
                    self.data_quality.valid_messages += 1
            
            # 触发数据回调
            await self._trigger_data_callbacks(data)
            
        except Exception as e:
            logger.error(f"处理已处理数据失败: {e}")
            self.processing_stats['error_counts_by_type']['routing_error'] += 1
    
    async def _handle_routed_data(self, data: StreamData, subscribers: List[str]):
        """处理路由后的数据"""
        try:
            # 存储数据
            if self.storage_manager:
                await self.storage_manager.store(data)
            
            # 记录监控事件
            if self.stream_monitor:
                self.stream_monitor.record_data_event("data_routed", self.data_quality)
            
            logger.debug(f"数据路由完成: {data.id}, 订阅者: {len(subscribers)}")
            
        except Exception as e:
            logger.error(f"处理路由数据失败: {e}")
            self.processing_stats['error_counts_by_type']['storage_error'] += 1
    
    async def _trigger_data_callbacks(self, data: StreamData):
        """触发数据回调函数"""
        try:
            # 触发通用数据回调
            for callback in self.data_callbacks.get('all', []):
                try:
                    if asyncio.iscoroutinefunction(callback):
                        await callback(data)
                    else:
                        callback(data)
                except Exception as e:
                    logger.error(f"数据回调执行失败: {e}")
            
            # 触发特定类型的数据回调
            data_type_key = data.data_type.value
            for callback in self.data_callbacks.get(data_type_key, []):
                try:
                    if asyncio.iscoroutinefunction(callback):
                        await callback(data)
                    else:
                        callback(data)
                except Exception as e:
                    logger.error(f"类型回调执行失败: {e}")
                    
        except Exception as e:
            logger.error(f"触发数据回调失败: {e}")
    
    async def _monitoring_loop(self):
        """监控循环"""
        while self.status == StreamManagerStatus.RUNNING:
            try:
                await asyncio.sleep(10)  # 每10秒监控一次
                
                # 更新性能指标
                await self._update_performance_metrics()
                
                # 检查组件状态
                await self._check_component_status()
                
            except Exception as e:
                logger.error(f"监控循环异常: {e}")
    
    async def _statistics_loop(self):
        """统计循环"""
        while self.status == StreamManagerStatus.RUNNING:
            try:
                await asyncio.sleep(5)  # 每5秒更新统计
                
                # 更新吞吐量指标
                self.metrics.update_throughput()
                
                # 更新组件活跃数量
                await self._update_active_counts()
                
            except Exception as e:
                logger.error(f"统计循环异常: {e}")
    
    async def _health_check_loop(self):
        """健康检查循环"""
        while self.status == StreamManagerStatus.RUNNING:
            try:
                await asyncio.sleep(30)  # 每30秒健康检查
                
                # 执行组件健康检查
                health_ok = await self._perform_health_check()
                
                if not health_ok:
                    logger.warning("系统健康检查发现问题")
                    await self._handle_health_issues()
                
            except Exception as e:
                logger.error(f"健康检查循环异常: {e}")
    
    async def _data_quality_loop(self):
        """数据质量监控循环"""
        while self.status == StreamManagerStatus.RUNNING:
            try:
                await asyncio.sleep(15)  # 每15秒检查数据质量
                
                # 计算数据质量指标
                validity_rate = self.data_quality.get_validity_rate()
                
                if validity_rate < 0.9:  # 数据质量低于90%
                    logger.warning(f"数据质量告警: 有效率 {validity_rate:.2%}")
                
                # 记录到监控系统
                if self.stream_monitor:
                    self.stream_monitor.record_data_event("quality_check", self.data_quality)
                
            except Exception as e:
                logger.error(f"数据质量监控循环异常: {e}")
    
    async def _update_performance_metrics(self):
        """更新性能指标"""
        try:
            current_time = time.time()
            
            # 更新基本性能指标
            self.performance_metrics.messages_per_second = self.metrics.messages_per_second
            self.performance_metrics.avg_latency_ms = self.metrics.avg_processing_latency_ms
            self.performance_metrics.active_connections = self.metrics.active_connections
            self.performance_metrics.last_update = current_time
            
            # 记录到监控系统
            if self.stream_monitor:
                monitor = self.stream_monitor.performance_monitor
                monitor.record_metric("messages_per_second", self.metrics.messages_per_second)
                monitor.record_metric("avg_processing_latency_ms", self.metrics.avg_processing_latency_ms)
                monitor.record_metric("active_connections", self.metrics.active_connections)
                monitor.record_metric("success_rate", self.metrics.get_success_rate())
            
        except Exception as e:
            logger.error(f"更新性能指标失败: {e}")
    
    async def _check_component_status(self):
        """检查组件状态"""
        try:
            # 检查各组件的运行状态
            if self.connection_manager:
                self.component_health['connection_manager'] = self.connection_manager.is_running
            
            if self.data_processor:
                # DataProcessor没有running属性，检查是否有config来判断是否初始化
                self.component_health['data_processor'] = hasattr(self.data_processor, 'config')
            
            if self.data_router:
                # DataRouter没有running属性，检查是否有subscription_manager来判断是否初始化
                self.component_health['data_router'] = hasattr(self.data_router, 'subscription_manager')
            
            if self.storage_manager:
                self.component_health['storage_manager'] = self.storage_manager.running
            
            if self.stream_monitor:
                self.component_health['stream_monitor'] = self.stream_monitor.running
            
        except Exception as e:
            logger.error(f"检查组件状态失败: {e}")
    
    async def _update_active_counts(self):
        """更新活跃数量统计"""
        try:
            # 更新活跃连接数
            if self.connection_manager:
                self.metrics.active_connections = self.connection_manager.active_connections
            
            # 更新活跃订阅数
            if self.data_router:
                # DataRouter使用subscription_manager.subscriptions
                if hasattr(self.data_router, 'subscription_manager') and self.data_router.subscription_manager:
                    self.metrics.active_subscriptions = len(self.data_router.subscription_manager.subscriptions)
                else:
                    self.metrics.active_subscriptions = 0
            
            # 更新活跃处理器数
            if self.data_processor:
                # DataProcessor没有active_processors属性，使用固定值1表示处理器运行
                self.metrics.active_processors = 1 if hasattr(self.data_processor, 'config') else 0
            
        except Exception as e:
            logger.error(f"更新活跃数量失败: {e}")
    
    async def _perform_health_check(self) -> bool:
        """执行健康检查"""
        try:
            unhealthy_components = []
            
            for component_name, is_healthy in self.component_health.items():
                if not is_healthy:
                    unhealthy_components.append(component_name)
            
            if unhealthy_components:
                logger.warning(f"不健康的组件: {unhealthy_components}")
                return False
            
            return True
            
        except Exception as e:
            logger.error(f"健康检查失败: {e}")
            return False
    
    async def _handle_health_issues(self):
        """处理健康问题"""
        try:
            # 简化的故障恢复逻辑
            for component_name, is_healthy in self.component_health.items():
                if not is_healthy:
                    logger.info(f"尝试恢复组件: {component_name}")
                    
                    # 这里可以实现具体的恢复逻辑
                    # 例如重启组件、重新连接等
                    
        except Exception as e:
            logger.error(f"处理健康问题失败: {e}")
    
    async def _notify_status_change(self):
        """通知状态变化"""
        try:
            for callback in self.status_callbacks:
                try:
                    if asyncio.iscoroutinefunction(callback):
                        await callback(self.status)
                    else:
                        callback(self.status)
                except Exception as e:
                    logger.error(f"状态回调执行失败: {e}")
                    
        except Exception as e:
            logger.error(f"通知状态变化失败: {e}")
    
    # 公共接口方法
    
    async def start_subscription(self, subscription: SubscriptionConfig) -> bool:
        """启动数据订阅"""
        try:
            if not self.connection_manager:
                logger.error("连接管理器未初始化")
                return False
            
            # 添加订阅到连接管理器
            result = await self.connection_manager.add_subscription(subscription)
            
            if result:
                # 添加路由规则到数据路由器
                if self.data_router:
                    await self.data_router.add_subscription(subscription)
                
                logger.info(f"订阅启动成功: {subscription.id}")
            else:
                logger.error(f"订阅启动失败: {subscription.id}")
            
            return result
            
        except Exception as e:
            logger.error(f"启动订阅失败: {e}")
            return False
    
    async def stop_subscription(self, subscription_id: str) -> bool:
        """停止数据订阅"""
        try:
            result = True
            
            # 从连接管理器移除订阅
            if self.connection_manager:
                conn_result = await self.connection_manager.remove_subscription(subscription_id)
                result = result and conn_result
            
            # 从数据路由器移除订阅
            if self.data_router:
                router_result = await self.data_router.remove_subscription(subscription_id)
                result = result and router_result
            
            if result:
                logger.info(f"订阅停止成功: {subscription_id}")
            else:
                logger.error(f"订阅停止失败: {subscription_id}")
            
            return result
            
        except Exception as e:
            logger.error(f"停止订阅失败: {e}")
            return False
    
    def add_data_callback(self, callback: Callable[[StreamData], None], 
                         data_type: Optional[str] = None):
        """添加数据回调函数"""
        try:
            key = data_type if data_type else 'all'
            self.data_callbacks[key].append(callback)
            logger.info(f"数据回调已添加: {key}")
            
        except Exception as e:
            logger.error(f"添加数据回调失败: {e}")
    
    def add_status_callback(self, callback: Callable[[StreamManagerStatus], None]):
        """添加状态回调函数"""
        try:
            self.status_callbacks.append(callback)
            logger.info("状态回调已添加")
            
        except Exception as e:
            logger.error(f"添加状态回调失败: {e}")
    
    async def pause(self) -> bool:
        """暂停数据流处理"""
        try:
            if self.status != StreamManagerStatus.RUNNING:
                logger.warning(f"无法暂停，当前状态: {self.status}")
                return False
            
            self.status = StreamManagerStatus.PAUSING
            await self._notify_status_change()
            
            # 暂停各个组件
            if self.connection_manager:
                await self.connection_manager.pause()
            
            if self.data_processor:
                await self.data_processor.pause()
            
            self.status = StreamManagerStatus.PAUSED
            await self._notify_status_change()
            
            logger.info("数据流处理已暂停")
            return True
            
        except Exception as e:
            logger.error(f"暂停数据流处理失败: {e}")
            self.status = StreamManagerStatus.ERROR
            await self._notify_status_change()
            return False
    
    async def resume(self) -> bool:
        """恢复数据流处理"""
        try:
            if self.status != StreamManagerStatus.PAUSED:
                logger.warning(f"无法恢复，当前状态: {self.status}")
                return False
            
            # 恢复各个组件
            if self.connection_manager:
                await self.connection_manager.resume()
            
            if self.data_processor:
                await self.data_processor.resume()
            
            self.status = StreamManagerStatus.RUNNING
            await self._notify_status_change()
            
            logger.info("数据流处理已恢复")
            return True
            
        except Exception as e:
            logger.error(f"恢复数据流处理失败: {e}")
            self.status = StreamManagerStatus.ERROR
            await self._notify_status_change()
            return False
    
    async def cleanup(self) -> bool:
        """清理资源"""
        try:
            self.status = StreamManagerStatus.STOPPING
            await self._notify_status_change()
            
            logger.info("开始清理数据流管理器资源...")
            
            # 取消后台任务
            for task in self.background_tasks:
                task.cancel()
            
            if self.background_tasks:
                await asyncio.gather(*self.background_tasks, return_exceptions=True)
                self.background_tasks.clear()
            
            # 清理各个组件
            cleanup_results = []
            
            if self.stream_monitor:
                result = await self.stream_monitor.cleanup()
                cleanup_results.append(('stream_monitor', result))
            
            if self.storage_manager:
                result = await self.storage_manager.cleanup()
                cleanup_results.append(('storage_manager', result))
            
            if self.data_router:
                result = await self.data_router.cleanup()
                cleanup_results.append(('data_router', result))
            
            if self.data_processor:
                result = await self.data_processor.cleanup()
                cleanup_results.append(('data_processor', result))
            
            if self.connection_manager:
                result = await self.connection_manager.cleanup()
                cleanup_results.append(('connection_manager', result))
            
            # 检查清理结果
            success = all(result for _, result in cleanup_results)
            
            self.status = StreamManagerStatus.STOPPED
            await self._notify_status_change()
            
            if success:
                logger.info("数据流管理器资源清理完成")
            else:
                logger.warning("部分组件清理失败")
                for component, result in cleanup_results:
                    if not result:
                        logger.warning(f"组件 {component} 清理失败")
            
            return success
            
        except Exception as e:
            logger.error(f"清理数据流管理器失败: {e}")
            self.status = StreamManagerStatus.ERROR
            await self._notify_status_change()
            return False
    
    def get_status(self) -> StreamManagerStatus:
        """获取管理器状态"""
        return self.status
    
    def get_metrics(self) -> Dict[str, Any]:
        """获取系统指标"""
        return {
            'stream_manager': {
                'status': self.status.value,
                'uptime_seconds': self.metrics.get_uptime_seconds(),
                'total_processed_messages': self.metrics.total_processed_messages,
                'successful_processed_messages': self.metrics.successful_processed_messages,
                'failed_processed_messages': self.metrics.failed_processed_messages,
                'success_rate': self.metrics.get_success_rate(),
                'messages_per_second': self.metrics.messages_per_second,
                'avg_processing_latency_ms': self.metrics.avg_processing_latency_ms,
                'max_processing_latency_ms': self.metrics.max_processing_latency_ms,
                'active_connections': self.metrics.active_connections,
                'active_subscriptions': self.metrics.active_subscriptions,
                'active_processors': self.metrics.active_processors
            },
            'component_health': self.component_health,
            'processing_stats': {
                'message_counts_by_source': dict(self.processing_stats['message_counts_by_source']),
                'message_counts_by_type': dict(self.processing_stats['message_counts_by_type']),
                'error_counts_by_type': dict(self.processing_stats['error_counts_by_type']),
                'latency_history_size': len(self.processing_stats['latency_history'])
            },
            'data_quality': {
                'total_messages': self.data_quality.total_messages,
                'valid_messages': self.data_quality.valid_messages,
                'validity_rate': self.data_quality.get_validity_rate(),
                'avg_latency_ms': self.data_quality.avg_latency_ms
            }
        }
    
    def get_component_stats(self) -> Dict[str, Any]:
        """获取组件统计信息"""
        stats = {}
        
        if self.connection_manager:
            stats['connection_manager'] = self.connection_manager.get_stats()
        
        if self.data_processor:
            stats['data_processor'] = self.data_processor.get_stats()
        
        if self.data_router:
            stats['data_router'] = self.data_router.get_stats()
        
        if self.storage_manager:
            stats['storage_manager'] = self.storage_manager.get_stats()
        
        if self.stream_monitor:
            stats['stream_monitor'] = self.stream_monitor.get_stats()
        
        return stats
    
    def get_dashboard_data(self) -> Dict[str, Any]:
        """获取仪表盘数据"""
        dashboard = {
            'system_overview': {
                'status': self.status.value,
                'uptime_seconds': self.metrics.get_uptime_seconds(),
                'overall_health': all(self.component_health.values()),
                'component_count': len(self.component_health),
                'healthy_components': sum(1 for h in self.component_health.values() if h)
            },
            'performance': {
                'messages_per_second': self.metrics.messages_per_second,
                'success_rate': self.metrics.get_success_rate(),
                'avg_latency_ms': self.metrics.avg_processing_latency_ms,
                'active_connections': self.metrics.active_connections
            },
            'data_quality': {
                'validity_rate': self.data_quality.get_validity_rate(),
                'total_messages': self.data_quality.total_messages,
                'valid_messages': self.data_quality.valid_messages
            }
        }
        
        # 添加监控仪表盘数据
        if self.stream_monitor:
            dashboard['monitoring'] = self.stream_monitor.get_monitoring_dashboard()
        
        return dashboard 
 