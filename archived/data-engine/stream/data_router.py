#!/usr/bin/env python3
"""
数据路由器模块 (Data Router Module)

负责实时数据的路由和分发，包括：
- 基于订阅的数据路由管理
- 多目标数据分发机制
- 负载均衡和性能优化
- 实时订阅状态监控
- 动态路由规则配置

支持高并发数据分发，确保实时性和可靠性
"""

import asyncio
import time
import logging
from abc import ABC, abstractmethod
from typing import Dict, List, Optional, Set, Callable, Any, Tuple
from dataclasses import dataclass, field
from enum import Enum
from collections import defaultdict, deque
import weakref
import json

from .models import StreamData, StreamDataType, PerformanceMetrics
from .config import SubscriptionConfig

# 设置日志
logger = logging.getLogger(__name__)

class DistributionStrategy(Enum):
    """分发策略枚举"""
    BROADCAST = "broadcast"         # 广播到所有订阅者
    ROUND_ROBIN = "round_robin"     # 轮询分发
    LOAD_BALANCE = "load_balance"   # 负载均衡
    PRIORITY = "priority"           # 按优先级分发
    FILTER = "filter"               # 基于过滤条件分发

class SubscriptionStatus(Enum):
    """订阅状态枚举"""
    ACTIVE = "active"               # 活跃
    PAUSED = "paused"               # 暂停
    CANCELLED = "cancelled"         # 已取消
    ERROR = "error"                 # 错误状态

@dataclass
class Subscription:
    """订阅信息类"""
    # 基本信息
    subscription_id: str
    subscriber_id: str
    data_types: Set[StreamDataType] = field(default_factory=set)
    symbols: Set[str] = field(default_factory=set)
    exchanges: Set[str] = field(default_factory=set)
    
    # 订阅配置
    status: SubscriptionStatus = SubscriptionStatus.ACTIVE
    priority: int = 1                              # 优先级(1-10，10最高)
    rate_limit: Optional[int] = None               # 速率限制(消息/秒)
    buffer_size: int = 1000                        # 缓冲区大小
    
    # 过滤条件
    filters: Dict[str, Any] = field(default_factory=dict)
    
    # 回调函数
    callback: Optional[Callable] = None
    error_callback: Optional[Callable] = None
    
    # 统计信息
    created_time: float = field(default_factory=time.time)
    last_activity_time: float = field(default_factory=time.time)
    messages_sent: int = 0
    messages_dropped: int = 0
    total_bytes_sent: int = 0
    
    # 缓冲区
    message_buffer: deque = field(default_factory=lambda: deque(maxlen=1000))
    
    def matches(self, stream_data: StreamData) -> bool:
        """检查数据是否匹配订阅条件"""
        # 检查数据类型
        if self.data_types and stream_data.data_type not in self.data_types:
            return False
        
        # 检查交易对
        if self.symbols and stream_data.symbol not in self.symbols:
            return False
        
        # 检查交易所
        if self.exchanges and stream_data.source not in self.exchanges:
            return False
        
        # 检查自定义过滤条件
        if self.filters:
            if not self._apply_filters(stream_data):
                return False
        
        return True
    
    def _apply_filters(self, stream_data: StreamData) -> bool:
        """应用自定义过滤条件"""
        try:
            for filter_key, filter_value in self.filters.items():
                if filter_key == 'min_price':
                    if hasattr(stream_data, 'price') and stream_data.price:
                        if stream_data.price < filter_value:
                            return False
                elif filter_key == 'max_price':
                    if hasattr(stream_data, 'price') and stream_data.price:
                        if stream_data.price > filter_value:
                            return False
                elif filter_key == 'min_volume':
                    if hasattr(stream_data, 'volume') and stream_data.volume:
                        if stream_data.volume < filter_value:
                            return False
                elif filter_key == 'quality_level':
                    if stream_data.quality and stream_data.quality.value not in filter_value:
                        return False
            
            return True
        except Exception as e:
            logger.warning(f"应用过滤条件时出错: {e}")
            return True  # 过滤器错误时默认通过
    
    def add_to_buffer(self, stream_data: StreamData) -> bool:
        """添加数据到缓冲区"""
        if len(self.message_buffer) >= self.buffer_size:
            # 缓冲区满，丢弃最老的消息
            self.message_buffer.popleft()
            self.messages_dropped += 1
        
        self.message_buffer.append(stream_data)
        self.last_activity_time = time.time()
        return True
    
    def get_buffer_size(self) -> int:
        """获取缓冲区大小"""
        return len(self.message_buffer)
    
    def clear_buffer(self):
        """清空缓冲区"""
        self.message_buffer.clear()

class SubscriptionManager:
    """订阅管理器"""
    
    def __init__(self):
        self.subscriptions: Dict[str, Subscription] = {}
        self.subscriber_subscriptions: Dict[str, Set[str]] = defaultdict(set)
        self.type_subscriptions: Dict[StreamDataType, Set[str]] = defaultdict(set)
        self.symbol_subscriptions: Dict[str, Set[str]] = defaultdict(set)
        self.exchange_subscriptions: Dict[str, Set[str]] = defaultdict(set)
        
        # 统计信息
        self.stats = {
            'total_subscriptions': 0,
            'active_subscriptions': 0,
            'total_matches': 0,
            'total_distributions': 0
        }
        
        # 锁保护
        self._lock = asyncio.Lock()
        
        logger.info("订阅管理器初始化完成")
    
    async def add_subscription(self, subscription: Subscription) -> bool:
        """添加订阅"""
        async with self._lock:
            try:
                subscription_id = subscription.subscription_id
                
                # 检查订阅是否已存在
                if subscription_id in self.subscriptions:
                    logger.warning(f"订阅已存在: {subscription_id}")
                    return False
                
                # 添加订阅
                self.subscriptions[subscription_id] = subscription
                
                # 更新索引
                self.subscriber_subscriptions[subscription.subscriber_id].add(subscription_id)
                
                for data_type in subscription.data_types:
                    self.type_subscriptions[data_type].add(subscription_id)
                
                for symbol in subscription.symbols:
                    self.symbol_subscriptions[symbol].add(subscription_id)
                
                for exchange in subscription.exchanges:
                    self.exchange_subscriptions[exchange].add(subscription_id)
                
                # 更新统计
                self.stats['total_subscriptions'] += 1
                if subscription.status == SubscriptionStatus.ACTIVE:
                    self.stats['active_subscriptions'] += 1
                
                logger.info(f"订阅添加成功: {subscription_id}")
                return True
                
            except Exception as e:
                logger.error(f"添加订阅失败: {e}")
                return False
    
    async def remove_subscription(self, subscription_id: str) -> bool:
        """移除订阅"""
        async with self._lock:
            try:
                if subscription_id not in self.subscriptions:
                    logger.warning(f"订阅不存在: {subscription_id}")
                    return False
                
                subscription = self.subscriptions[subscription_id]
                
                # 更新索引
                self.subscriber_subscriptions[subscription.subscriber_id].discard(subscription_id)
                
                for data_type in subscription.data_types:
                    self.type_subscriptions[data_type].discard(subscription_id)
                
                for symbol in subscription.symbols:
                    self.symbol_subscriptions[symbol].discard(subscription_id)
                
                for exchange in subscription.exchanges:
                    self.exchange_subscriptions[exchange].discard(subscription_id)
                
                # 删除订阅
                del self.subscriptions[subscription_id]
                
                # 更新统计
                self.stats['total_subscriptions'] -= 1
                if subscription.status == SubscriptionStatus.ACTIVE:
                    self.stats['active_subscriptions'] -= 1
                
                logger.info(f"订阅移除成功: {subscription_id}")
                return True
                
            except Exception as e:
                logger.error(f"移除订阅失败: {e}")
                return False
    
    async def get_matching_subscriptions(self, stream_data: StreamData) -> List[Subscription]:
        """获取匹配的订阅"""
        matching_subscriptions = []
        
        async with self._lock:
            # 快速索引查找候选订阅
            candidate_ids = set()
            
            # 按数据类型查找
            candidate_ids.update(self.type_subscriptions.get(stream_data.data_type, set()))
            
            # 按交易对查找
            candidate_ids.update(self.symbol_subscriptions.get(stream_data.symbol, set()))
            
            # 按交易所查找
            candidate_ids.update(self.exchange_subscriptions.get(stream_data.source, set()))
            
            # 检查每个候选订阅
            for subscription_id in candidate_ids:
                if subscription_id in self.subscriptions:
                    subscription = self.subscriptions[subscription_id]
                    
                    # 检查订阅状态
                    if subscription.status != SubscriptionStatus.ACTIVE:
                        continue
                    
                    # 检查匹配条件
                    if subscription.matches(stream_data):
                        matching_subscriptions.append(subscription)
            
            # 更新统计
            if matching_subscriptions:
                self.stats['total_matches'] += 1
        
        return matching_subscriptions
    
    async def update_subscription_status(self, subscription_id: str, 
                                       status: SubscriptionStatus) -> bool:
        """更新订阅状态"""
        async with self._lock:
            if subscription_id not in self.subscriptions:
                return False
            
            old_status = self.subscriptions[subscription_id].status
            self.subscriptions[subscription_id].status = status
            
            # 更新统计
            if old_status == SubscriptionStatus.ACTIVE and status != SubscriptionStatus.ACTIVE:
                self.stats['active_subscriptions'] -= 1
            elif old_status != SubscriptionStatus.ACTIVE and status == SubscriptionStatus.ACTIVE:
                self.stats['active_subscriptions'] += 1
            
            return True
    
    async def get_subscriber_subscriptions(self, subscriber_id: str) -> List[Subscription]:
        """获取指定订阅者的所有订阅"""
        async with self._lock:
            subscription_ids = self.subscriber_subscriptions.get(subscriber_id, set())
            return [self.subscriptions[sid] for sid in subscription_ids 
                   if sid in self.subscriptions]
    
    def get_stats(self) -> Dict[str, Any]:
        """获取统计信息"""
        return {
            **self.stats,
            'subscriptions_by_type': {
                data_type.value: len(subscription_ids)
                for data_type, subscription_ids in self.type_subscriptions.items()
            },
            'subscriptions_by_exchange': {
                exchange: len(subscription_ids)
                for exchange, subscription_ids in self.exchange_subscriptions.items()
            }
        }

class DataDistributor:
    """数据分发器"""
    
    def __init__(self, strategy: DistributionStrategy = DistributionStrategy.BROADCAST):
        self.strategy = strategy
        self.round_robin_counters: Dict[str, int] = defaultdict(int)
        
        # 分发统计
        self.distribution_stats = {
            'total_distributions': 0,
            'successful_distributions': 0,
            'failed_distributions': 0,
            'total_bytes_distributed': 0
        }
        
        logger.info(f"数据分发器初始化完成，策略: {strategy.value}")
    
    async def distribute(self, stream_data: StreamData, 
                        subscriptions: List[Subscription]) -> Dict[str, bool]:
        """分发数据到订阅者"""
        if not subscriptions:
            return {}
        
        distribution_results = {}
        
        try:
            # 根据策略选择分发方法
            if self.strategy == DistributionStrategy.BROADCAST:
                distribution_results = await self._broadcast_distribute(stream_data, subscriptions)
            elif self.strategy == DistributionStrategy.ROUND_ROBIN:
                distribution_results = await self._round_robin_distribute(stream_data, subscriptions)
            elif self.strategy == DistributionStrategy.LOAD_BALANCE:
                distribution_results = await self._load_balance_distribute(stream_data, subscriptions)
            elif self.strategy == DistributionStrategy.PRIORITY:
                distribution_results = await self._priority_distribute(stream_data, subscriptions)
            else:
                distribution_results = await self._broadcast_distribute(stream_data, subscriptions)
            
            # 更新统计
            self.distribution_stats['total_distributions'] += len(distribution_results)
            successful = sum(1 for success in distribution_results.values() if success)
            self.distribution_stats['successful_distributions'] += successful
            self.distribution_stats['failed_distributions'] += len(distribution_results) - successful
            
            # 估算分发的数据大小
            data_size = len(json.dumps(stream_data.to_dict(), default=str))
            self.distribution_stats['total_bytes_distributed'] += data_size * len(distribution_results)
            
        except Exception as e:
            logger.error(f"数据分发异常: {e}")
            distribution_results = {sub.subscription_id: False for sub in subscriptions}
        
        return distribution_results
    
    async def _broadcast_distribute(self, stream_data: StreamData, 
                                  subscriptions: List[Subscription]) -> Dict[str, bool]:
        """广播分发"""
        results = {}
        
        # 并发分发到所有订阅者
        tasks = []
        for subscription in subscriptions:
            task = self._send_to_subscription(stream_data, subscription)
            tasks.append((subscription.subscription_id, task))
        
        # 等待所有任务完成
        for subscription_id, task in tasks:
            try:
                success = await task
                results[subscription_id] = success
            except Exception as e:
                logger.error(f"分发到订阅 {subscription_id} 失败: {e}")
                results[subscription_id] = False
        
        return results
    
    async def _round_robin_distribute(self, stream_data: StreamData, 
                                    subscriptions: List[Subscription]) -> Dict[str, bool]:
        """轮询分发"""
        if not subscriptions:
            return {}
        
        # 基于数据类型的轮询计数器
        counter_key = f"{stream_data.data_type.value}_{stream_data.source}_{stream_data.symbol}"
        
        # 选择下一个订阅者
        index = self.round_robin_counters[counter_key] % len(subscriptions)
        selected_subscription = subscriptions[index]
        
        # 更新计数器
        self.round_robin_counters[counter_key] += 1
        
        # 发送数据
        success = await self._send_to_subscription(stream_data, selected_subscription)
        return {selected_subscription.subscription_id: success}
    
    async def _load_balance_distribute(self, stream_data: StreamData, 
                                     subscriptions: List[Subscription]) -> Dict[str, bool]:
        """负载均衡分发"""
        if not subscriptions:
            return {}
        
        # 选择缓冲区最小的订阅者
        selected_subscription = min(subscriptions, key=lambda s: s.get_buffer_size())
        
        # 发送数据
        success = await self._send_to_subscription(stream_data, selected_subscription)
        return {selected_subscription.subscription_id: success}
    
    async def _priority_distribute(self, stream_data: StreamData, 
                                 subscriptions: List[Subscription]) -> Dict[str, bool]:
        """按优先级分发"""
        if not subscriptions:
            return {}
        
        # 按优先级排序(优先级高的在前)
        sorted_subscriptions = sorted(subscriptions, key=lambda s: s.priority, reverse=True)
        
        results = {}
        
        # 分发到最高优先级的订阅者
        highest_priority = sorted_subscriptions[0].priority
        high_priority_subscriptions = [
            sub for sub in sorted_subscriptions 
            if sub.priority == highest_priority
        ]
        
        # 并发分发到同等最高优先级的订阅者
        tasks = []
        for subscription in high_priority_subscriptions:
            task = self._send_to_subscription(stream_data, subscription)
            tasks.append((subscription.subscription_id, task))
        
        # 等待所有任务完成
        for subscription_id, task in tasks:
            try:
                success = await task
                results[subscription_id] = success
            except Exception as e:
                logger.error(f"分发到高优先级订阅 {subscription_id} 失败: {e}")
                results[subscription_id] = False
        
        return results
    
    async def _send_to_subscription(self, stream_data: StreamData, 
                                  subscription: Subscription) -> bool:
        """发送数据到单个订阅"""
        try:
            # 检查速率限制
            if subscription.rate_limit:
                # 简单的速率限制检查(可以改进为令牌桶算法)
                current_time = time.time()
                time_window = 1.0  # 1秒窗口
                
                if hasattr(subscription, '_last_send_time'):
                    time_diff = current_time - subscription._last_send_time
                    if time_diff < (1.0 / subscription.rate_limit):
                        # 超过速率限制，丢弃消息
                        subscription.messages_dropped += 1
                        return False
                
                subscription._last_send_time = current_time
            
            # 调用回调函数
            if subscription.callback:
                try:
                    if asyncio.iscoroutinefunction(subscription.callback):
                        await subscription.callback(stream_data)
                    else:
                        subscription.callback(stream_data)
                except Exception as e:
                    logger.error(f"订阅回调函数执行失败: {e}")
                    
                    # 调用错误回调
                    if subscription.error_callback:
                        try:
                            if asyncio.iscoroutinefunction(subscription.error_callback):
                                await subscription.error_callback(stream_data, e)
                            else:
                                subscription.error_callback(stream_data, e)
                        except Exception as callback_error:
                            logger.error(f"错误回调函数执行失败: {callback_error}")
                    
                    return False
            else:
                # 没有回调函数，添加到缓冲区
                subscription.add_to_buffer(stream_data)
            
            # 更新统计
            subscription.messages_sent += 1
            subscription.total_bytes_sent += len(json.dumps(stream_data.to_dict(), default=str))
            subscription.last_activity_time = time.time()
            
            return True
            
        except Exception as e:
            logger.error(f"发送数据到订阅失败: {e}")
            subscription.messages_dropped += 1
            return False
    
    def get_stats(self) -> Dict[str, Any]:
        """获取分发统计"""
        return {
            **self.distribution_stats,
            'strategy': self.strategy.value,
            'round_robin_counters': dict(self.round_robin_counters)
        }

class DataRouter:
    """数据路由器主类"""
    
    def __init__(self, config: SubscriptionConfig, 
                 strategy: DistributionStrategy = DistributionStrategy.BROADCAST):
        self.config = config
        self.subscription_manager = SubscriptionManager()
        self.distributor = DataDistributor(strategy)
        
        # 路由统计
        self.routing_stats = {
            'total_routed': 0,
            'successful_routed': 0,
            'failed_routed': 0,
            'routing_time_ms': 0.0
        }
        
        # 性能监控
        self.performance_metrics = PerformanceMetrics()
        self.last_metrics_update = time.time()
        
        logger.info(f"数据路由器初始化完成，分发策略: {strategy.value}")
    
    async def route_data(self, stream_data: StreamData) -> Dict[str, Any]:
        """路由数据到匹配的订阅者"""
        start_time = time.time()
        
        try:
            # 获取匹配的订阅
            matching_subscriptions = await self.subscription_manager.get_matching_subscriptions(stream_data)
            
            if not matching_subscriptions:
                # 没有匹配的订阅
                return {
                    'success': True,
                    'subscriptions_count': 0,
                    'distribution_results': {},
                    'processing_time_ms': (time.time() - start_time) * 1000
                }
            
            # 分发数据
            distribution_results = await self.distributor.distribute(stream_data, matching_subscriptions)
            
            # 更新统计
            self.routing_stats['total_routed'] += 1
            successful_distributions = sum(1 for success in distribution_results.values() if success)
            
            if successful_distributions > 0:
                self.routing_stats['successful_routed'] += 1
            else:
                self.routing_stats['failed_routed'] += 1
            
            processing_time = (time.time() - start_time) * 1000
            self.routing_stats['routing_time_ms'] += processing_time
            
            # 更新性能指标
            await self._update_performance_metrics()
            
            return {
                'success': True,
                'subscriptions_count': len(matching_subscriptions),
                'distribution_results': distribution_results,
                'processing_time_ms': processing_time
            }
            
        except Exception as e:
            self.routing_stats['failed_routed'] += 1
            error_msg = f"路由数据失败: {e}"
            logger.error(error_msg)
            
            return {
                'success': False,
                'error': error_msg,
                'subscriptions_count': 0,
                'distribution_results': {},
                'processing_time_ms': (time.time() - start_time) * 1000
            }
    
    async def subscribe(self, subscriber_id: str, data_types: List[StreamDataType],
                       symbols: List[str] = None, exchanges: List[str] = None,
                       callback: Callable = None, priority: int = 1,
                       filters: Dict[str, Any] = None) -> str:
        """创建订阅"""
        subscription_id = f"{subscriber_id}_{int(time.time() * 1000)}"
        
        subscription = Subscription(
            subscription_id=subscription_id,
            subscriber_id=subscriber_id,
            data_types=set(data_types),
            symbols=set(symbols or []),
            exchanges=set(exchanges or []),
            callback=callback,
            priority=priority,
            filters=filters or {}
        )
        
        success = await self.subscription_manager.add_subscription(subscription)
        if success:
            logger.info(f"订阅创建成功: {subscription_id}")
            return subscription_id
        else:
            raise RuntimeError(f"创建订阅失败: {subscription_id}")
    
    async def unsubscribe(self, subscription_id: str) -> bool:
        """取消订阅"""
        success = await self.subscription_manager.remove_subscription(subscription_id)
        if success:
            logger.info(f"订阅取消成功: {subscription_id}")
        return success
    
    async def pause_subscription(self, subscription_id: str) -> bool:
        """暂停订阅"""
        return await self.subscription_manager.update_subscription_status(
            subscription_id, SubscriptionStatus.PAUSED
        )
    
    async def resume_subscription(self, subscription_id: str) -> bool:
        """恢复订阅"""
        return await self.subscription_manager.update_subscription_status(
            subscription_id, SubscriptionStatus.ACTIVE
        )
    
    async def get_subscriber_subscriptions(self, subscriber_id: str) -> List[Dict[str, Any]]:
        """获取订阅者的所有订阅"""
        subscriptions = await self.subscription_manager.get_subscriber_subscriptions(subscriber_id)
        return [
            {
                'subscription_id': sub.subscription_id,
                'data_types': [dt.value for dt in sub.data_types],
                'symbols': list(sub.symbols),
                'exchanges': list(sub.exchanges),
                'status': sub.status.value,
                'priority': sub.priority,
                'messages_sent': sub.messages_sent,
                'messages_dropped': sub.messages_dropped,
                'buffer_size': sub.get_buffer_size(),
                'created_time': sub.created_time,
                'last_activity_time': sub.last_activity_time
            }
            for sub in subscriptions
        ]
    
    async def _update_performance_metrics(self):
        """更新性能指标"""
        current_time = time.time()
        
        # 每5秒更新一次指标
        if current_time - self.last_metrics_update >= 5.0:
            # 计算路由性能
            if self.routing_stats['total_routed'] > 0:
                avg_routing_time = self.routing_stats['routing_time_ms'] / self.routing_stats['total_routed']
                success_rate = self.routing_stats['successful_routed'] / self.routing_stats['total_routed']
                
                self.performance_metrics.messages_per_second = self.routing_stats['total_routed'] / 5.0
                self.performance_metrics.error_rate = 1.0 - success_rate
            
            # 重置计数器
            self.routing_stats['routing_time_ms'] = 0.0
            self.routing_stats['total_routed'] = 0
            self.routing_stats['successful_routed'] = 0
            self.routing_stats['failed_routed'] = 0
            
            self.last_metrics_update = current_time
    
    def get_stats(self) -> Dict[str, Any]:
        """获取路由统计信息"""
        return {
            'routing_stats': self.routing_stats,
            'subscription_stats': self.subscription_manager.get_stats(),
            'distribution_stats': self.distributor.get_stats(),
            'performance_metrics': {
                'messages_per_second': self.performance_metrics.messages_per_second,
                'error_rate': self.performance_metrics.error_rate,
                'uptime_seconds': self.performance_metrics.uptime_seconds
            }
        } 