"""
连接器监控模块

功能：
- 定义连接器健康检查标准
- 实现批量连接器状态监控
- 为后续监控系统提供基础
- 连接器性能指标收集

作者: HermesFlow Team
创建时间: 2024年12月21日
"""

import asyncio
import logging
import time
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any, Callable
from dataclasses import dataclass, field
from enum import Enum
import json

from . import CONNECTOR_REGISTRY
from .base_connector import BaseConnector, ConnectionStatus

# 配置日志
logger = logging.getLogger(__name__)

class HealthStatus(Enum):
    """健康状态枚举"""
    HEALTHY = "healthy"          # 健康
    DEGRADED = "degraded"        # 降级
    UNHEALTHY = "unhealthy"      # 不健康
    CRITICAL = "critical"        # 严重问题
    UNKNOWN = "unknown"          # 未知状态

class MetricType(Enum):
    """指标类型枚举"""
    RESPONSE_TIME = "response_time"      # 响应时间
    ERROR_RATE = "error_rate"           # 错误率
    SUCCESS_RATE = "success_rate"       # 成功率
    API_CALLS = "api_calls"             # API调用次数
    DATA_QUALITY = "data_quality"       # 数据质量
    CONNECTION_STATUS = "connection_status"  # 连接状态

@dataclass
class HealthCheckResult:
    """健康检查结果"""
    connector_name: str
    status: HealthStatus
    timestamp: datetime
    response_time: float
    error_message: Optional[str] = None
    metrics: Dict[str, Any] = field(default_factory=dict)
    warnings: List[str] = field(default_factory=list)
    
    def to_dict(self) -> Dict[str, Any]:
        """转换为字典格式"""
        return {
            'connector_name': self.connector_name,
            'status': self.status.value,
            'timestamp': self.timestamp.isoformat(),
            'response_time': self.response_time,
            'error_message': self.error_message,
            'metrics': self.metrics,
            'warnings': self.warnings
        }

@dataclass
class PerformanceMetric:
    """性能指标"""
    metric_type: MetricType
    value: float
    timestamp: datetime
    connector_name: str
    tags: Dict[str, str] = field(default_factory=dict)
    
    def to_dict(self) -> Dict[str, Any]:
        """转换为字典格式"""
        return {
            'metric_type': self.metric_type.value,
            'value': self.value,
            'timestamp': self.timestamp.isoformat(),
            'connector_name': self.connector_name,
            'tags': self.tags
        }

@dataclass
class MonitoringConfig:
    """监控配置"""
    check_interval: int = 60  # 检查间隔（秒）
    timeout: int = 30  # 超时时间
    retry_count: int = 3  # 重试次数
    alert_thresholds: Dict[str, float] = field(default_factory=lambda: {
        'response_time': 5.0,  # 响应时间阈值（秒）
        'error_rate': 0.1,     # 错误率阈值（10%）
        'success_rate': 0.9    # 成功率阈值（90%）
    })
    enabled_connectors: List[str] = field(default_factory=lambda: list(CONNECTOR_REGISTRY.keys()))

class ConnectorMonitor:
    """连接器监控器"""
    
    def __init__(self, config: MonitoringConfig):
        """
        初始化监控器
        
        Args:
            config: 监控配置
        """
        self.config = config
        self.metrics_history: Dict[str, List[PerformanceMetric]] = {}
        self.health_history: Dict[str, List[HealthCheckResult]] = {}
        self.last_check_time: Dict[str, datetime] = {}
        self.alert_callbacks: List[Callable[[HealthCheckResult], None]] = []
        self._running = False
        self._monitor_task: Optional[asyncio.Task] = None
        
        logger.info("连接器监控器初始化完成")
    
    def add_alert_callback(self, callback: Callable[[HealthCheckResult], None]):
        """
        添加告警回调函数
        
        Args:
            callback: 告警回调函数
        """
        self.alert_callbacks.append(callback)
    
    async def check_connector_health(self, connector_name: str, connector_instance: BaseConnector) -> HealthCheckResult:
        """
        检查单个连接器健康状态
        
        Args:
            connector_name: 连接器名称
            connector_instance: 连接器实例
            
        Returns:
            HealthCheckResult: 健康检查结果
        """
        start_time = time.time()
        
        try:
            # 检查连接状态
            if hasattr(connector_instance, 'health_check'):
                health_data = await asyncio.wait_for(
                    connector_instance.health_check(),
                    timeout=self.config.timeout
                )
            else:
                # 基础健康检查
                health_data = {
                    'status': 'unknown',
                    'message': '连接器不支持health_check方法'
                }
            
            response_time = time.time() - start_time
            
            # 分析健康状态
            status = self._analyze_health_status(connector_name, health_data, response_time)
            
            # 收集性能指标
            metrics = self._collect_metrics(connector_name, health_data, response_time)
            
            # 检查警告
            warnings = self._check_warnings(connector_name, health_data, response_time)
            
            result = HealthCheckResult(
                connector_name=connector_name,
                status=status,
                timestamp=datetime.now(),
                response_time=response_time,
                metrics=metrics,
                warnings=warnings
            )
            
            # 记录性能指标
            self._record_metrics(connector_name, response_time, True)
            
            return result
            
        except asyncio.TimeoutError:
            response_time = time.time() - start_time
            result = HealthCheckResult(
                connector_name=connector_name,
                status=HealthStatus.CRITICAL,
                timestamp=datetime.now(),
                response_time=response_time,
                error_message=f"健康检查超时 ({self.config.timeout}秒)"
            )
            
            # 记录错误指标
            self._record_metrics(connector_name, response_time, False)
            
            return result
            
        except Exception as e:
            response_time = time.time() - start_time
            result = HealthCheckResult(
                connector_name=connector_name,
                status=HealthStatus.UNHEALTHY,
                timestamp=datetime.now(),
                response_time=response_time,
                error_message=str(e)
            )
            
            # 记录错误指标
            self._record_metrics(connector_name, response_time, False)
            
            return result
    
    def _analyze_health_status(self, connector_name: str, health_data: Dict[str, Any], response_time: float) -> HealthStatus:
        """
        分析健康状态
        
        Args:
            connector_name: 连接器名称
            health_data: 健康数据
            response_time: 响应时间
            
        Returns:
            HealthStatus: 健康状态
        """
        # 检查响应时间
        if response_time > self.config.alert_thresholds['response_time']:
            return HealthStatus.DEGRADED
        
        # 检查健康数据中的状态
        if health_data.get('status') == 'healthy':
            return HealthStatus.HEALTHY
        elif health_data.get('status') == 'degraded':
            return HealthStatus.DEGRADED
        elif health_data.get('status') in ['unhealthy', 'error']:
            return HealthStatus.UNHEALTHY
        elif health_data.get('status') == 'critical':
            return HealthStatus.CRITICAL
        
        # 检查历史错误率
        error_rate = self._calculate_error_rate(connector_name)
        if error_rate > self.config.alert_thresholds['error_rate']:
            return HealthStatus.UNHEALTHY
        
        return HealthStatus.HEALTHY
    
    def _collect_metrics(self, connector_name: str, health_data: Dict[str, Any], response_time: float) -> Dict[str, Any]:
        """
        收集性能指标
        
        Args:
            connector_name: 连接器名称
            health_data: 健康数据
            response_time: 响应时间
            
        Returns:
            Dict[str, Any]: 性能指标
        """
        metrics = {
            'response_time': response_time,
            'error_rate': self._calculate_error_rate(connector_name),
            'success_rate': self._calculate_success_rate(connector_name),
            'api_calls_count': self._get_api_calls_count(connector_name)
        }
        
        # 从健康数据中提取额外指标
        if 'metrics' in health_data:
            metrics.update(health_data['metrics'])
        
        return metrics
    
    def _check_warnings(self, connector_name: str, health_data: Dict[str, Any], response_time: float) -> List[str]:
        """
        检查警告条件
        
        Args:
            connector_name: 连接器名称
            health_data: 健康数据
            response_time: 响应时间
            
        Returns:
            List[str]: 警告列表
        """
        warnings = []
        
        # 响应时间警告
        if response_time > self.config.alert_thresholds['response_time'] * 0.8:
            warnings.append(f"响应时间较慢: {response_time:.2f}秒")
        
        # 错误率警告
        error_rate = self._calculate_error_rate(connector_name)
        if error_rate > self.config.alert_thresholds['error_rate'] * 0.8:
            warnings.append(f"错误率较高: {error_rate:.2%}")
        
        # 从健康数据中提取警告
        if 'warnings' in health_data:
            warnings.extend(health_data['warnings'])
        
        return warnings
    
    def _record_metrics(self, connector_name: str, response_time: float, success: bool):
        """
        记录性能指标
        
        Args:
            connector_name: 连接器名称
            response_time: 响应时间
            success: 是否成功
        """
        timestamp = datetime.now()
        
        # 初始化历史记录
        if connector_name not in self.metrics_history:
            self.metrics_history[connector_name] = []
        
        # 记录响应时间指标
        response_metric = PerformanceMetric(
            metric_type=MetricType.RESPONSE_TIME,
            value=response_time,
            timestamp=timestamp,
            connector_name=connector_name
        )
        self.metrics_history[connector_name].append(response_metric)
        
        # 记录API调用指标
        api_metric = PerformanceMetric(
            metric_type=MetricType.API_CALLS,
            value=1,
            timestamp=timestamp,
            connector_name=connector_name,
            tags={'success': str(success)}
        )
        self.metrics_history[connector_name].append(api_metric)
        
        # 清理旧指标（保留最近1小时）
        cutoff_time = timestamp - timedelta(hours=1)
        self.metrics_history[connector_name] = [
            m for m in self.metrics_history[connector_name]
            if m.timestamp > cutoff_time
        ]
    
    def _calculate_error_rate(self, connector_name: str) -> float:
        """
        计算错误率
        
        Args:
            connector_name: 连接器名称
            
        Returns:
            float: 错误率 (0.0-1.0)
        """
        if connector_name not in self.metrics_history:
            return 0.0
        
        # 获取最近5分钟的API调用
        cutoff_time = datetime.now() - timedelta(minutes=5)
        recent_calls = [
            m for m in self.metrics_history[connector_name]
            if m.metric_type == MetricType.API_CALLS and m.timestamp > cutoff_time
        ]
        
        if not recent_calls:
            return 0.0
        
        error_calls = [m for m in recent_calls if m.tags.get('success') == 'False']
        return len(error_calls) / len(recent_calls)
    
    def _calculate_success_rate(self, connector_name: str) -> float:
        """
        计算成功率
        
        Args:
            connector_name: 连接器名称
            
        Returns:
            float: 成功率 (0.0-1.0)
        """
        return 1.0 - self._calculate_error_rate(connector_name)
    
    def _get_api_calls_count(self, connector_name: str) -> int:
        """
        获取API调用次数
        
        Args:
            connector_name: 连接器名称
            
        Returns:
            int: 最近5分钟的API调用次数
        """
        if connector_name not in self.metrics_history:
            return 0
        
        cutoff_time = datetime.now() - timedelta(minutes=5)
        recent_calls = [
            m for m in self.metrics_history[connector_name]
            if m.metric_type == MetricType.API_CALLS and m.timestamp > cutoff_time
        ]
        
        return len(recent_calls)
    
    async def check_all_connectors(self) -> Dict[str, HealthCheckResult]:
        """
        检查所有启用的连接器健康状态
        
        Returns:
            Dict[str, HealthCheckResult]: 所有连接器的健康检查结果
        """
        results = {}
        
        for connector_name in self.config.enabled_connectors:
            if connector_name not in CONNECTOR_REGISTRY:
                logger.warning(f"连接器 {connector_name} 未在注册表中找到")
                continue
            
            try:
                # 创建连接器实例进行测试
                # 注意：这里使用测试配置，不进行实际连接
                connector_class = CONNECTOR_REGISTRY[connector_name]
                
                # 根据不同连接器类型创建测试实例
                if connector_name == 'polygon':
                    connector_instance = connector_class(api_key='test_key')
                elif connector_name == 'fred':
                    connector_instance = connector_class(api_key='test_key')
                elif connector_name in ['binance', 'okx', 'bitget']:
                    connector_instance = connector_class(
                        api_key='test_key',
                        api_secret='test_secret'
                    )
                elif connector_name == 'gmgn':
                    from .gmgn.models import GMGNConfig, ChainType
                    config = GMGNConfig()
                    connector_instance = connector_class(config)
                else:
                    logger.warning(f"未知的连接器类型: {connector_name}")
                    continue
                
                result = await self.check_connector_health(connector_name, connector_instance)
                results[connector_name] = result
                
                # 更新历史记录
                if connector_name not in self.health_history:
                    self.health_history[connector_name] = []
                
                self.health_history[connector_name].append(result)
                
                # 清理旧历史（保留最近24小时）
                cutoff_time = datetime.now() - timedelta(hours=24)
                self.health_history[connector_name] = [
                    h for h in self.health_history[connector_name]
                    if h.timestamp > cutoff_time
                ]
                
                # 更新最后检查时间
                self.last_check_time[connector_name] = datetime.now()
                
                # 触发告警回调
                if result.status in [HealthStatus.UNHEALTHY, HealthStatus.CRITICAL]:
                    for callback in self.alert_callbacks:
                        try:
                            callback(result)
                        except Exception as e:
                            logger.error(f"告警回调执行失败: {e}")
                
                logger.debug(f"连接器 {connector_name} 健康检查完成: {result.status.value}")
                
            except Exception as e:
                logger.error(f"检查连接器 {connector_name} 时发生异常: {e}")
                result = HealthCheckResult(
                    connector_name=connector_name,
                    status=HealthStatus.CRITICAL,
                    timestamp=datetime.now(),
                    response_time=0.0,
                    error_message=str(e)
                )
                results[connector_name] = result
        
        return results
    
    async def start_monitoring(self):
        """启动持续监控"""
        if self._running:
            logger.warning("监控已在运行中")
            return
        
        self._running = True
        self._monitor_task = asyncio.create_task(self._monitoring_loop())
        logger.info("连接器监控已启动")
    
    async def stop_monitoring(self):
        """停止监控"""
        if not self._running:
            return
        
        self._running = False
        if self._monitor_task:
            self._monitor_task.cancel()
            try:
                await self._monitor_task
            except asyncio.CancelledError:
                pass
        
        logger.info("连接器监控已停止")
    
    async def _monitoring_loop(self):
        """监控循环"""
        while self._running:
            try:
                logger.debug("开始定期健康检查")
                results = await self.check_all_connectors()
                
                # 输出摘要
                healthy_count = sum(1 for r in results.values() if r.status == HealthStatus.HEALTHY)
                total_count = len(results)
                logger.info(f"健康检查完成: {healthy_count}/{total_count} 连接器健康")
                
                # 等待下次检查
                await asyncio.sleep(self.config.check_interval)
                
            except asyncio.CancelledError:
                break
            except Exception as e:
                logger.error(f"监控循环异常: {e}")
                await asyncio.sleep(5)  # 短暂等待后重试
    
    def get_monitoring_summary(self) -> Dict[str, Any]:
        """
        获取监控摘要
        
        Returns:
            Dict[str, Any]: 监控摘要
        """
        summary = {
            'enabled_connectors': self.config.enabled_connectors,
            'total_connectors': len(self.config.enabled_connectors),
            'last_check_times': {
                name: time.isoformat() for name, time in self.last_check_time.items()
            },
            'health_summary': {},
            'metrics_summary': {},
            'timestamp': datetime.now().isoformat()
        }
        
        # 健康状态摘要
        for connector_name in self.config.enabled_connectors:
            if connector_name in self.health_history and self.health_history[connector_name]:
                latest_health = self.health_history[connector_name][-1]
                summary['health_summary'][connector_name] = {
                    'status': latest_health.status.value,
                    'last_check': latest_health.timestamp.isoformat(),
                    'response_time': latest_health.response_time,
                    'error_message': latest_health.error_message
                }
        
        # 指标摘要
        for connector_name in self.config.enabled_connectors:
            if connector_name in self.metrics_history:
                summary['metrics_summary'][connector_name] = {
                    'error_rate': self._calculate_error_rate(connector_name),
                    'success_rate': self._calculate_success_rate(connector_name),
                    'api_calls_count': self._get_api_calls_count(connector_name)
                }
        
        return summary

# 默认告警回调函数
def default_alert_callback(result: HealthCheckResult):
    """
    默认告警回调函数
    
    Args:
        result: 健康检查结果
    """
    if result.status in [HealthStatus.UNHEALTHY, HealthStatus.CRITICAL]:
        logger.warning(
            f"连接器告警: {result.connector_name} 状态为 {result.status.value}"
            f"{'，错误信息: ' + result.error_message if result.error_message else ''}"
        )

# 创建默认监控器实例
def create_default_monitor() -> ConnectorMonitor:
    """
    创建默认监控器实例
    
    Returns:
        ConnectorMonitor: 监控器实例
    """
    config = MonitoringConfig()
    monitor = ConnectorMonitor(config)
    monitor.add_alert_callback(default_alert_callback)
    return monitor 