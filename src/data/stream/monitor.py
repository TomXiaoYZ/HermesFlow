#!/usr/bin/env python3
"""
监控系统模块 (Monitor System Module)

负责数据流的监控和告警，包括：
- 实时性能指标收集和分析
- 健康状态监控和检查
- 异常检测和智能告警
- 多渠道告警通知
- 监控数据可视化导出
- 基于阈值的自动化响应

支持全方位的系统监控，确保数据流的稳定运行
"""

import asyncio
import time
import json
import logging
import statistics
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Any, Callable, Union, Tuple
from enum import Enum
from collections import deque, defaultdict
import aiohttp
import smtplib
from email.mime.text import MIMEText
from email.mime.multipart import MIMEMultipart

from .models import PerformanceMetrics, DataQuality, QualityLevel
from .config import MonitorConfig

# 设置日志
logger = logging.getLogger(__name__)

class AlertLevel(Enum):
    """告警级别枚举"""
    INFO = "info"               # 信息
    WARNING = "warning"         # 警告
    ERROR = "error"             # 错误
    CRITICAL = "critical"       # 严重

class MonitorStatus(Enum):
    """监控状态枚举"""
    HEALTHY = "healthy"         # 健康
    WARNING = "warning"         # 警告状态
    CRITICAL = "critical"       # 严重状态
    UNKNOWN = "unknown"         # 未知状态

class MetricType(Enum):
    """指标类型枚举"""
    COUNTER = "counter"         # 计数器
    GAUGE = "gauge"             # 仪表盘
    HISTOGRAM = "histogram"     # 直方图
    SUMMARY = "summary"         # 摘要

@dataclass
class Alert:
    """告警信息类"""
    # 基本信息
    alert_id: str
    level: AlertLevel
    title: str
    message: str
    source: str
    
    # 时间信息
    created_time: float = field(default_factory=time.time)
    resolved_time: Optional[float] = None
    
    # 告警状态
    is_resolved: bool = False
    acknowledgment: bool = False
    
    # 相关数据
    metric_name: str = ""
    metric_value: Optional[float] = None
    threshold: Optional[float] = None
    
    # 告警规则
    rule_id: Optional[str] = None
    
    def to_dict(self) -> Dict[str, Any]:
        """转换为字典"""
        return {
            'alert_id': self.alert_id,
            'level': self.level.value,
            'title': self.title,
            'message': self.message,
            'source': self.source,
            'created_time': self.created_time,
            'resolved_time': self.resolved_time,
            'is_resolved': self.is_resolved,
            'acknowledgment': self.acknowledgment,
            'metric_name': self.metric_name,
            'metric_value': self.metric_value,
            'threshold': self.threshold,
            'rule_id': self.rule_id
        }

@dataclass
class MonitorMetric:
    """监控指标类"""
    name: str
    metric_type: MetricType
    value: Union[float, int]
    timestamp: float = field(default_factory=time.time)
    labels: Dict[str, str] = field(default_factory=dict)
    description: str = ""
    unit: str = ""
    
    def to_prometheus_format(self) -> str:
        """转换为Prometheus格式"""
        labels_str = ""
        if self.labels:
            label_pairs = [f'{k}="{v}"' for k, v in self.labels.items()]
            labels_str = "{" + ",".join(label_pairs) + "}"
        
        return f"# HELP {self.name} {self.description}\n" \
               f"# TYPE {self.name} {self.metric_type.value}\n" \
               f"{self.name}{labels_str} {self.value} {int(self.timestamp * 1000)}"

class AlertRule:
    """告警规则类"""
    
    def __init__(self, rule_id: str, metric_name: str, condition: str,
                 threshold: float, level: AlertLevel = AlertLevel.WARNING,
                 duration_seconds: int = 60, cooldown_seconds: int = 300):
        self.rule_id = rule_id
        self.metric_name = metric_name
        self.condition = condition  # gt, lt, eq, ge, le
        self.threshold = threshold
        self.level = level
        self.duration_seconds = duration_seconds
        self.cooldown_seconds = cooldown_seconds
        
        # 状态跟踪
        self.triggered_time: Optional[float] = None
        self.last_alert_time: Optional[float] = None
        self.is_active = False
        
        # 历史数据
        self.metric_history: deque = deque(maxlen=100)
    
    def evaluate(self, metric_value: float) -> bool:
        """评估告警规则"""
        current_time = time.time()
        
        # 记录指标历史
        self.metric_history.append((current_time, metric_value))
        
        # 评估条件
        condition_met = False
        if self.condition == "gt":
            condition_met = metric_value > self.threshold
        elif self.condition == "lt":
            condition_met = metric_value < self.threshold
        elif self.condition == "ge":
            condition_met = metric_value >= self.threshold
        elif self.condition == "le":
            condition_met = metric_value <= self.threshold
        elif self.condition == "eq":
            condition_met = abs(metric_value - self.threshold) < 0.001
        
        # 状态管理
        if condition_met:
            if not self.is_active:
                self.triggered_time = current_time
                self.is_active = True
            
            # 检查持续时间
            if (current_time - self.triggered_time >= self.duration_seconds and
                (not self.last_alert_time or 
                 current_time - self.last_alert_time >= self.cooldown_seconds)):
                self.last_alert_time = current_time
                return True
        else:
            self.is_active = False
            self.triggered_time = None
        
        return False

class BaseNotifier(ABC):
    """通知器基础抽象类"""
    
    @abstractmethod
    async def send_notification(self, alert: Alert) -> bool:
        """发送通知"""
        pass
    
    @abstractmethod
    async def test_connection(self) -> bool:
        """测试连接"""
        pass

class SlackNotifier(BaseNotifier):
    """Slack通知器"""
    
    def __init__(self, webhook_url: str, channel: str = "#alerts"):
        self.webhook_url = webhook_url
        self.channel = channel
        self.session: Optional[aiohttp.ClientSession] = None
    
    async def initialize(self):
        """初始化HTTP会话"""
        self.session = aiohttp.ClientSession()
    
    async def cleanup(self):
        """清理资源"""
        if self.session:
            await self.session.close()
    
    async def send_notification(self, alert: Alert) -> bool:
        """发送Slack通知"""
        if not self.session:
            await self.initialize()
        
        try:
            # 根据告警级别选择颜色
            color_map = {
                AlertLevel.INFO: "#36a64f",      # 绿色
                AlertLevel.WARNING: "#ff9500",   # 橙色
                AlertLevel.ERROR: "#ff0000",     # 红色
                AlertLevel.CRITICAL: "#8B0000"   # 深红色
            }
            
            # 构建Slack消息
            payload = {
                "channel": self.channel,
                "username": "HermesFlow Monitor",
                "icon_emoji": ":warning:",
                "attachments": [{
                    "color": color_map.get(alert.level, "#808080"),
                    "title": alert.title,
                    "text": alert.message,
                    "fields": [
                        {"title": "级别", "value": alert.level.value.upper(), "short": True},
                        {"title": "来源", "value": alert.source, "short": True},
                        {"title": "指标", "value": alert.metric_name, "short": True},
                        {"title": "当前值", "value": str(alert.metric_value), "short": True}
                    ],
                    "ts": int(alert.created_time)
                }]
            }
            
            # 发送HTTP请求
            async with self.session.post(self.webhook_url, json=payload) as response:
                if response.status == 200:
                    logger.info(f"Slack告警发送成功: {alert.alert_id}")
                    return True
                else:
                    logger.error(f"Slack告警发送失败: {response.status}")
                    return False
                    
        except Exception as e:
            logger.error(f"发送Slack通知失败: {e}")
            return False
    
    async def test_connection(self) -> bool:
        """测试Slack连接"""
        test_alert = Alert(
            alert_id="test",
            level=AlertLevel.INFO,
            title="连接测试",
            message="这是一条测试消息",
            source="monitor_test"
        )
        return await self.send_notification(test_alert)

class EmailNotifier(BaseNotifier):
    """邮件通知器"""
    
    def __init__(self, smtp_server: str, smtp_port: int, username: str,
                 password: str, from_email: str, to_emails: List[str]):
        self.smtp_server = smtp_server
        self.smtp_port = smtp_port
        self.username = username
        self.password = password
        self.from_email = from_email
        self.to_emails = to_emails
    
    async def send_notification(self, alert: Alert) -> bool:
        """发送邮件通知"""
        try:
            # 构建邮件内容
            msg = MIMEMultipart()
            msg['From'] = self.from_email
            msg['To'] = ", ".join(self.to_emails)
            msg['Subject'] = f"[{alert.level.value.upper()}] {alert.title}"
            
            # 邮件正文
            body = f"""
HermesFlow 系统告警通知

告警级别: {alert.level.value.upper()}
告警标题: {alert.title}
告警消息: {alert.message}
告警来源: {alert.source}
触发时间: {time.strftime('%Y-%m-%d %H:%M:%S', time.localtime(alert.created_time))}

指标信息:
- 指标名称: {alert.metric_name}
- 当前值: {alert.metric_value}
- 阈值: {alert.threshold}

请及时处理相关问题。

---
HermesFlow 监控系统
            """
            
            msg.attach(MIMEText(body, 'plain', 'utf-8'))
            
            # 发送邮件 (在线程池中执行以避免阻塞)
            await asyncio.get_event_loop().run_in_executor(
                None, self._send_email_sync, msg
            )
            
            logger.info(f"邮件告警发送成功: {alert.alert_id}")
            return True
            
        except Exception as e:
            logger.error(f"发送邮件通知失败: {e}")
            return False
    
    def _send_email_sync(self, msg: MIMEMultipart):
        """同步发送邮件"""
        with smtplib.SMTP(self.smtp_server, self.smtp_port) as server:
            server.starttls()
            server.login(self.username, self.password)
            server.send_message(msg)
    
    async def test_connection(self) -> bool:
        """测试邮件连接"""
        try:
            await asyncio.get_event_loop().run_in_executor(
                None, self._test_connection_sync
            )
            return True
        except Exception as e:
            logger.error(f"邮件连接测试失败: {e}")
            return False
    
    def _test_connection_sync(self):
        """同步测试连接"""
        with smtplib.SMTP(self.smtp_server, self.smtp_port) as server:
            server.starttls()
            server.login(self.username, self.password)

class AlertManager:
    """告警管理器"""
    
    def __init__(self, config: MonitorConfig):
        self.config = config
        self.alert_rules: Dict[str, AlertRule] = {}
        self.active_alerts: Dict[str, Alert] = {}
        self.alert_history: deque = deque(maxlen=1000)
        
        # 通知器
        self.notifiers: Dict[str, BaseNotifier] = {}
        
        # 统计信息
        self.alert_stats = {
            'total_alerts': 0,
            'alerts_by_level': defaultdict(int),
            'resolved_alerts': 0,
            'notification_success': 0,
            'notification_failed': 0
        }
        
        logger.info("告警管理器初始化完成")
    
    def add_alert_rule(self, rule: AlertRule):
        """添加告警规则"""
        self.alert_rules[rule.rule_id] = rule
        logger.info(f"告警规则已添加: {rule.rule_id}")
    
    def remove_alert_rule(self, rule_id: str) -> bool:
        """移除告警规则"""
        if rule_id in self.alert_rules:
            del self.alert_rules[rule_id]
            logger.info(f"告警规则已移除: {rule_id}")
            return True
        return False
    
    def add_notifier(self, name: str, notifier: BaseNotifier):
        """添加通知器"""
        self.notifiers[name] = notifier
        logger.info(f"通知器已添加: {name}")
    
    async def evaluate_metrics(self, metrics: Dict[str, float]):
        """评估指标并触发告警"""
        for rule_id, rule in self.alert_rules.items():
            if rule.metric_name in metrics:
                metric_value = metrics[rule.metric_name]
                
                if rule.evaluate(metric_value):
                    await self._trigger_alert(rule, metric_value)
    
    async def _trigger_alert(self, rule: AlertRule, metric_value: float):
        """触发告警"""
        alert_id = f"{rule.rule_id}_{int(time.time())}"
        
        alert = Alert(
            alert_id=alert_id,
            level=rule.level,
            title=f"指标告警: {rule.metric_name}",
            message=f"指标 {rule.metric_name} 当前值 {metric_value} {rule.condition} 阈值 {rule.threshold}",
            source="stream_monitor",
            metric_name=rule.metric_name,
            metric_value=metric_value,
            threshold=rule.threshold,
            rule_id=rule.rule_id
        )
        
        # 记录告警
        self.active_alerts[alert_id] = alert
        self.alert_history.append(alert)
        
        # 更新统计
        self.alert_stats['total_alerts'] += 1
        self.alert_stats['alerts_by_level'][rule.level.value] += 1
        
        # 发送通知
        await self._send_notifications(alert)
        
        logger.warning(f"告警触发: {alert.title} - {alert.message}")
    
    async def _send_notifications(self, alert: Alert):
        """发送通知"""
        if not self.config.alert_enabled:
            return
        
        notification_tasks = []
        for channel in self.config.alert_channels:
            if channel in self.notifiers:
                task = self.notifiers[channel].send_notification(alert)
                notification_tasks.append(task)
        
        if notification_tasks:
            results = await asyncio.gather(*notification_tasks, return_exceptions=True)
            
            # 统计通知结果
            for result in results:
                if isinstance(result, Exception):
                    self.alert_stats['notification_failed'] += 1
                    logger.error(f"通知发送异常: {result}")
                elif result:
                    self.alert_stats['notification_success'] += 1
                else:
                    self.alert_stats['notification_failed'] += 1
    
    async def resolve_alert(self, alert_id: str) -> bool:
        """解决告警"""
        if alert_id in self.active_alerts:
            alert = self.active_alerts[alert_id]
            alert.is_resolved = True
            alert.resolved_time = time.time()
            
            # 从活跃告警中移除
            del self.active_alerts[alert_id]
            
            # 更新统计
            self.alert_stats['resolved_alerts'] += 1
            
            logger.info(f"告警已解决: {alert_id}")
            return True
        return False
    
    async def acknowledge_alert(self, alert_id: str) -> bool:
        """确认告警"""
        if alert_id in self.active_alerts:
            self.active_alerts[alert_id].acknowledgment = True
            logger.info(f"告警已确认: {alert_id}")
            return True
        return False
    
    def get_active_alerts(self) -> List[Alert]:
        """获取活跃告警"""
        return list(self.active_alerts.values())
    
    def get_alert_history(self, limit: int = 100) -> List[Alert]:
        """获取告警历史"""
        return list(self.alert_history)[-limit:]
    
    def get_stats(self) -> Dict[str, Any]:
        """获取告警统计"""
        return {
            **self.alert_stats,
            'active_alerts_count': len(self.active_alerts),
            'alert_rules_count': len(self.alert_rules),
            'notifiers_count': len(self.notifiers)
        }

class PerformanceMonitor:
    """性能监控器"""
    
    def __init__(self, config: MonitorConfig):
        self.config = config
        self.metrics: Dict[str, MonitorMetric] = {}
        self.metric_history: Dict[str, deque] = defaultdict(lambda: deque(maxlen=1000))
        
        # 监控状态
        self.running = False
        self.last_collection_time = time.time()
        
        # 系统性能指标
        self.performance_metrics = PerformanceMetrics()
        
        logger.info("性能监控器初始化完成")
    
    def record_metric(self, name: str, value: Union[float, int], 
                     metric_type: MetricType = MetricType.GAUGE,
                     labels: Dict[str, str] = None, description: str = "",
                     unit: str = ""):
        """记录指标"""
        metric = MonitorMetric(
            name=name,
            metric_type=metric_type,
            value=value,
            labels=labels or {},
            description=description,
            unit=unit
        )
        
        self.metrics[name] = metric
        self.metric_history[name].append((time.time(), value))
        
        # 更新性能指标
        self._update_performance_metrics(name, value)
    
    def _update_performance_metrics(self, metric_name: str, value: Union[float, int]):
        """更新性能指标"""
        current_time = time.time()
        
        # 更新特定性能指标
        if metric_name == "messages_per_second":
            self.performance_metrics.messages_per_second = float(value)
        elif metric_name == "bytes_per_second":
            self.performance_metrics.bytes_per_second = float(value)
        elif metric_name == "memory_usage_mb":
            self.performance_metrics.memory_usage_mb = float(value)
        elif metric_name == "cpu_usage_percent":
            self.performance_metrics.cpu_usage_percent = float(value)
        elif metric_name == "error_rate":
            self.performance_metrics.error_rate = float(value)
        elif metric_name == "active_connections":
            self.performance_metrics.active_connections = int(value)
        
        self.performance_metrics.last_update = current_time
    
    def get_metric(self, name: str) -> Optional[MonitorMetric]:
        """获取指标"""
        return self.metrics.get(name)
    
    def get_metric_value(self, name: str) -> Optional[Union[float, int]]:
        """获取指标值"""
        metric = self.metrics.get(name)
        return metric.value if metric else None
    
    def get_metric_history(self, name: str, duration_seconds: int = 3600) -> List[Tuple[float, Union[float, int]]]:
        """获取指标历史"""
        if name not in self.metric_history:
            return []
        
        current_time = time.time()
        cutoff_time = current_time - duration_seconds
        
        return [(timestamp, value) for timestamp, value in self.metric_history[name]
                if timestamp >= cutoff_time]
    
    def get_metric_statistics(self, name: str, duration_seconds: int = 3600) -> Dict[str, float]:
        """获取指标统计信息"""
        history = self.get_metric_history(name, duration_seconds)
        if not history:
            return {}
        
        values = [value for _, value in history]
        
        try:
            return {
                'count': len(values),
                'min': min(values),
                'max': max(values),
                'mean': statistics.mean(values),
                'median': statistics.median(values),
                'stdev': statistics.stdev(values) if len(values) > 1 else 0.0,
                'percentile_95': statistics.quantiles(values, n=20)[18] if len(values) >= 20 else max(values),
                'percentile_99': statistics.quantiles(values, n=100)[98] if len(values) >= 100 else max(values)
            }
        except Exception as e:
            logger.warning(f"计算指标统计失败: {e}")
            return {}
    
    def export_prometheus_metrics(self) -> str:
        """导出Prometheus格式指标"""
        lines = []
        for metric in self.metrics.values():
            lines.append(metric.to_prometheus_format())
        
        return "\n".join(lines)
    
    def get_all_metrics(self) -> Dict[str, Any]:
        """获取所有指标"""
        return {
            name: {
                'value': metric.value,
                'type': metric.metric_type.value,
                'timestamp': metric.timestamp,
                'labels': metric.labels,
                'description': metric.description,
                'unit': metric.unit
            }
            for name, metric in self.metrics.items()
        }
    
    def clear_metrics(self):
        """清空指标"""
        self.metrics.clear()
        self.metric_history.clear()
        logger.info("性能指标已清空")

class HealthChecker:
    """健康检查器"""
    
    def __init__(self, config: MonitorConfig):
        self.config = config
        self.health_checks: Dict[str, Callable] = {}
        self.health_status: Dict[str, MonitorStatus] = {}
        self.last_check_time = time.time()
        
        logger.info("健康检查器初始化完成")
    
    def register_health_check(self, name: str, check_func: Callable):
        """注册健康检查函数"""
        self.health_checks[name] = check_func
        self.health_status[name] = MonitorStatus.UNKNOWN
        logger.info(f"健康检查已注册: {name}")
    
    async def run_health_checks(self) -> Dict[str, MonitorStatus]:
        """运行所有健康检查"""
        results = {}
        
        for name, check_func in self.health_checks.items():
            try:
                if asyncio.iscoroutinefunction(check_func):
                    result = await check_func()
                else:
                    result = check_func()
                
                # 结果转换为状态
                if isinstance(result, bool):
                    status = MonitorStatus.HEALTHY if result else MonitorStatus.CRITICAL
                elif isinstance(result, MonitorStatus):
                    status = result
                else:
                    status = MonitorStatus.UNKNOWN
                
                self.health_status[name] = status
                results[name] = status
                
            except Exception as e:
                logger.error(f"健康检查 {name} 执行失败: {e}")
                self.health_status[name] = MonitorStatus.CRITICAL
                results[name] = MonitorStatus.CRITICAL
        
        self.last_check_time = time.time()
        return results
    
    def get_overall_health(self) -> MonitorStatus:
        """获取整体健康状态"""
        if not self.health_status:
            return MonitorStatus.UNKNOWN
        
        status_counts = defaultdict(int)
        for status in self.health_status.values():
            status_counts[status] += 1
        
        # 优先级：CRITICAL > WARNING > HEALTHY > UNKNOWN
        if status_counts[MonitorStatus.CRITICAL] > 0:
            return MonitorStatus.CRITICAL
        elif status_counts[MonitorStatus.WARNING] > 0:
            return MonitorStatus.WARNING
        elif status_counts[MonitorStatus.HEALTHY] > 0:
            return MonitorStatus.HEALTHY
        else:
            return MonitorStatus.UNKNOWN
    
    def get_health_report(self) -> Dict[str, Any]:
        """获取健康报告"""
        return {
            'overall_status': self.get_overall_health().value,
            'components': {name: status.value for name, status in self.health_status.items()},
            'last_check_time': self.last_check_time,
            'total_checks': len(self.health_checks)
        }

class StreamMonitor:
    """数据流监控器主类"""
    
    def __init__(self, config: MonitorConfig):
        self.config = config
        self.performance_monitor = PerformanceMonitor(config)
        self.alert_manager = AlertManager(config)
        self.health_checker = HealthChecker(config)
        
        # 监控状态
        self.running = False
        self.start_time = time.time()
        
        # 后台任务
        self.background_tasks: List[asyncio.Task] = []
        
        # 数据质量监控
        self.data_quality = DataQuality()
        
        logger.info("数据流监控器初始化完成")
    
    async def initialize(self) -> bool:
        """初始化监控器"""
        try:
            # 初始化通知器
            await self._initialize_notifiers()
            
            # 设置默认告警规则
            self._setup_default_alert_rules()
            
            # 注册默认健康检查
            self._register_default_health_checks()
            
            self.running = True
            self.start_time = time.time()
            
            # 启动后台监控任务
            monitoring_tasks = [
                self._metrics_collection_loop(),
                self._health_check_loop(),
                self._alert_evaluation_loop()
            ]
            
            for task_func in monitoring_tasks:
                task = asyncio.create_task(task_func)
                self.background_tasks.append(task)
            
            logger.info("数据流监控器初始化成功")
            return True
            
        except Exception as e:
            logger.error(f"数据流监控器初始化失败: {e}")
            return False
    
    async def _initialize_notifiers(self):
        """初始化通知器"""
        for channel in self.config.alert_channels:
            if channel == "slack":
                # TODO: 从配置获取Slack Webhook URL
                webhook_url = "https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK"
                notifier = SlackNotifier(webhook_url)
                await notifier.initialize()
                self.alert_manager.add_notifier("slack", notifier)
            
            elif channel == "email":
                # TODO: 从配置获取邮件参数
                notifier = EmailNotifier(
                    smtp_server="smtp.gmail.com",
                    smtp_port=587,
                    username="your_email@gmail.com",
                    password="your_password",
                    from_email="your_email@gmail.com",
                    to_emails=["admin@example.com"]
                )
                self.alert_manager.add_notifier("email", notifier)
    
    def _setup_default_alert_rules(self):
        """设置默认告警规则"""
        # 延迟告警规则
        latency_rule = AlertRule(
            rule_id="high_latency",
            metric_name="avg_latency_ms",
            condition="gt",
            threshold=self.config.max_latency_threshold_ms,
            level=AlertLevel.WARNING,
            duration_seconds=30
        )
        self.alert_manager.add_alert_rule(latency_rule)
        
        # 吞吐量告警规则
        throughput_rule = AlertRule(
            rule_id="low_throughput",
            metric_name="messages_per_second",
            condition="lt",
            threshold=self.config.min_throughput_threshold,
            level=AlertLevel.WARNING,
            duration_seconds=60
        )
        self.alert_manager.add_alert_rule(throughput_rule)
        
        # 错误率告警规则
        error_rate_rule = AlertRule(
            rule_id="high_error_rate",
            metric_name="error_rate",
            condition="gt",
            threshold=self.config.max_error_rate_threshold,
            level=AlertLevel.ERROR,
            duration_seconds=30
        )
        self.alert_manager.add_alert_rule(error_rate_rule)
        
        # 内存使用告警规则
        memory_rule = AlertRule(
            rule_id="high_memory_usage",
            metric_name="memory_usage_percent",
            condition="gt",
            threshold=self.config.max_memory_usage_threshold * 100,
            level=AlertLevel.CRITICAL,
            duration_seconds=60
        )
        self.alert_manager.add_alert_rule(memory_rule)
    
    def _register_default_health_checks(self):
        """注册默认健康检查"""
        def check_system_resources():
            # 简化的系统资源检查
            import psutil
            
            memory_percent = psutil.virtual_memory().percent
            cpu_percent = psutil.cpu_percent(interval=1)
            
            if memory_percent > 90 or cpu_percent > 90:
                return MonitorStatus.CRITICAL
            elif memory_percent > 80 or cpu_percent > 80:
                return MonitorStatus.WARNING
            else:
                return MonitorStatus.HEALTHY
        
        self.health_checker.register_health_check("system_resources", check_system_resources)
        
        def check_data_quality():
            if self.data_quality.get_validity_rate() < 0.9:
                return MonitorStatus.CRITICAL
            elif self.data_quality.get_validity_rate() < 0.95:
                return MonitorStatus.WARNING
            else:
                return MonitorStatus.HEALTHY
        
        self.health_checker.register_health_check("data_quality", check_data_quality)
    
    async def _metrics_collection_loop(self):
        """指标收集循环"""
        while self.running:
            try:
                await asyncio.sleep(self.config.metrics_collection_interval)
                
                # 收集系统指标
                await self._collect_system_metrics()
                
                # 收集性能指标
                await self._collect_performance_metrics()
                
            except Exception as e:
                logger.error(f"指标收集循环异常: {e}")
    
    async def _health_check_loop(self):
        """健康检查循环"""
        while self.running:
            try:
                await asyncio.sleep(self.config.health_check_interval)
                
                # 运行健康检查
                await self.health_checker.run_health_checks()
                
            except Exception as e:
                logger.error(f"健康检查循环异常: {e}")
    
    async def _alert_evaluation_loop(self):
        """告警评估循环"""
        while self.running:
            try:
                await asyncio.sleep(5)  # 每5秒评估一次
                
                # 获取当前指标值
                current_metrics = {}
                for name, metric in self.performance_monitor.metrics.items():
                    current_metrics[name] = metric.value
                
                # 评估告警规则
                await self.alert_manager.evaluate_metrics(current_metrics)
                
            except Exception as e:
                logger.error(f"告警评估循环异常: {e}")
    
    async def _collect_system_metrics(self):
        """收集系统指标"""
        try:
            import psutil
            
            # 内存使用率
            memory = psutil.virtual_memory()
            self.performance_monitor.record_metric(
                "memory_usage_percent", memory.percent,
                MetricType.GAUGE, description="Memory usage percentage"
            )
            
            # CPU使用率
            cpu_percent = psutil.cpu_percent(interval=None)
            self.performance_monitor.record_metric(
                "cpu_usage_percent", cpu_percent,
                MetricType.GAUGE, description="CPU usage percentage"
            )
            
            # 磁盘使用率
            disk = psutil.disk_usage('/')
            disk_percent = (disk.used / disk.total) * 100
            self.performance_monitor.record_metric(
                "disk_usage_percent", disk_percent,
                MetricType.GAUGE, description="Disk usage percentage"
            )
            
        except ImportError:
            logger.warning("psutil未安装，无法收集系统指标")
        except Exception as e:
            logger.error(f"收集系统指标失败: {e}")
    
    async def _collect_performance_metrics(self):
        """收集性能指标"""
        try:
            # 运行时间
            uptime_seconds = time.time() - self.start_time
            self.performance_monitor.record_metric(
                "uptime_seconds", uptime_seconds,
                MetricType.COUNTER, description="System uptime in seconds"
            )
            
            # 数据质量指标
            validity_rate = self.data_quality.get_validity_rate()
            self.performance_monitor.record_metric(
                "data_validity_rate", validity_rate,
                MetricType.GAUGE, description="Data validity rate"
            )
            
            # 平均延迟
            if self.data_quality.valid_messages > 0:
                self.performance_monitor.record_metric(
                    "avg_latency_ms", self.data_quality.avg_latency_ms,
                    MetricType.GAUGE, description="Average latency in milliseconds"
                )
            
        except Exception as e:
            logger.error(f"收集性能指标失败: {e}")
    
    async def cleanup(self) -> bool:
        """清理资源"""
        try:
            self.running = False
            
            # 取消后台任务
            for task in self.background_tasks:
                task.cancel()
            
            if self.background_tasks:
                await asyncio.gather(*self.background_tasks, return_exceptions=True)
            
            # 清理通知器
            for notifier in self.alert_manager.notifiers.values():
                if hasattr(notifier, 'cleanup'):
                    await notifier.cleanup()
            
            logger.info("数据流监控器清理完成")
            return True
            
        except Exception as e:
            logger.error(f"清理监控器失败: {e}")
            return False
    
    def record_data_event(self, event_type: str, data_quality: Optional[DataQuality] = None):
        """记录数据事件"""
        if data_quality:
            self.data_quality = data_quality
        
        # 记录事件指标
        self.performance_monitor.record_metric(
            f"data_events_{event_type}", 1,
            MetricType.COUNTER, description=f"Number of {event_type} events"
        )
    
    def get_monitoring_dashboard(self) -> Dict[str, Any]:
        """获取监控仪表盘数据"""
        return {
            'system_status': {
                'running': self.running,
                'uptime_seconds': time.time() - self.start_time,
                'health_status': self.health_checker.get_overall_health().value
            },
            'performance_metrics': self.performance_monitor.get_all_metrics(),
            'alert_summary': {
                'active_alerts': len(self.alert_manager.get_active_alerts()),
                'total_alerts_today': self.alert_manager.alert_stats['total_alerts'],
                'alert_stats': self.alert_manager.get_stats()
            },
            'health_report': self.health_checker.get_health_report(),
            'data_quality': {
                'total_messages': self.data_quality.total_messages,
                'valid_messages': self.data_quality.valid_messages,
                'validity_rate': self.data_quality.get_validity_rate(),
                'avg_latency_ms': self.data_quality.avg_latency_ms
            }
        }
    
    def get_stats(self) -> Dict[str, Any]:
        """获取监控统计信息"""
        return {
            'monitor_status': {
                'running': self.running,
                'start_time': self.start_time,
                'background_tasks': len(self.background_tasks)
            },
            'performance_monitor': {
                'metrics_count': len(self.performance_monitor.metrics),
                'last_collection': self.performance_monitor.last_collection_time
            },
            'alert_manager': self.alert_manager.get_stats(),
            'health_checker': self.health_checker.get_health_report()
        } 
 