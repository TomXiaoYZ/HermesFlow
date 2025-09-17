"""
数据采集调度器

基于APScheduler实现的数据采集任务调度系统，支持：
- Cron表达式定时调度
- 任务失败重试机制
- 动态任务管理
- 实时状态监控

作者: HermesFlow Team
创建时间: 2024年12月26日
"""

import asyncio
import logging
import sys
import traceback
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Callable, Any
from dataclasses import dataclass, field
from enum import Enum

from apscheduler.schedulers.asyncio import AsyncIOScheduler
from apscheduler.triggers.cron import CronTrigger
from apscheduler.triggers.interval import IntervalTrigger
from apscheduler.events import EVENT_JOB_EXECUTED, EVENT_JOB_ERROR, EVENT_JOB_MISSED
from apscheduler.jobstores.sqlalchemy import SQLAlchemyJobStore
from apscheduler.executors.asyncio import AsyncIOExecutor

# 添加项目根目录到Python路径
sys.path.append('.')

from src.config.config_manager import ConfigManager
from src.data.connectors import CONNECTOR_REGISTRY

logger = logging.getLogger(__name__)

class TaskStatus(Enum):
    """任务状态枚举"""
    PENDING = "pending"
    RUNNING = "running"
    SUCCESS = "success"
    FAILED = "failed"
    RETRYING = "retrying"
    CANCELLED = "cancelled"

@dataclass
class TaskConfig:
    """任务配置"""
    task_id: str
    connector_name: str
    method_name: str
    method_args: Dict[str, Any] = field(default_factory=dict)
    schedule_type: str = "interval"  # interval, cron
    schedule_config: Dict[str, Any] = field(default_factory=dict)
    max_retries: int = 3
    retry_backoff: int = 60  # 秒
    enabled: bool = True
    description: str = ""

@dataclass
class TaskExecution:
    """任务执行记录"""
    task_id: str
    execution_id: str
    start_time: datetime
    end_time: Optional[datetime] = None
    status: TaskStatus = TaskStatus.PENDING
    result: Optional[Any] = None
    error: Optional[str] = None
    retry_count: int = 0

class DataCollectionScheduler:
    """数据采集调度器"""
    
    def __init__(self, config_manager: ConfigManager):
        """
        初始化数据采集调度器
        
        Args:
            config_manager: 配置管理器
        """
        self.config_manager = config_manager
        self.scheduler_config = config_manager.get_scheduler_config()
        
        # 初始化调度器
        jobstores = {
            'default': SQLAlchemyJobStore(url=self.scheduler_config.get('database_url', 'sqlite:///jobs.sqlite'))
        }
        executors = {
            'default': AsyncIOExecutor()
        }
        job_defaults = {
            'coalesce': False,
            'max_instances': 3
        }
        
        self.scheduler = AsyncIOScheduler(
            jobstores=jobstores,
            executors=executors,
            job_defaults=job_defaults,
            timezone='UTC'
        )
        
        # 任务配置和执行记录
        self.task_configs: Dict[str, TaskConfig] = {}
        self.task_executions: Dict[str, List[TaskExecution]] = {}
        
        # 连接器注册表
        self.connector_registry = CONNECTOR_REGISTRY
        
        # 绑定事件监听器
        self._setup_event_listeners()
        
        logger.info("数据采集调度器初始化完成")
    
    def _setup_event_listeners(self):
        """设置调度器事件监听器"""
        self.scheduler.add_listener(
            self._on_job_executed,
            EVENT_JOB_EXECUTED
        )
        self.scheduler.add_listener(
            self._on_job_error,
            EVENT_JOB_ERROR
        )
        self.scheduler.add_listener(
            self._on_job_missed,
            EVENT_JOB_MISSED
        )
    
    async def start(self):
        """启动调度器"""
        try:
            self.scheduler.start()
            logger.info("数据采集调度器已启动")
        except Exception as e:
            logger.error(f"启动调度器失败: {e}")
            raise
    
    async def shutdown(self):
        """关闭调度器"""
        try:
            self.scheduler.shutdown(wait=True)
            logger.info("数据采集调度器已关闭")
        except Exception as e:
            logger.error(f"关闭调度器失败: {e}")
    
    def add_task(self, task_config: TaskConfig) -> bool:
        """
        添加数据采集任务
        
        Args:
            task_config: 任务配置
            
        Returns:
            bool: 添加成功返回True
        """
        try:
            # 验证任务配置
            if not self._validate_task_config(task_config):
                return False
            
            # 创建调度触发器
            trigger = self._create_trigger(task_config)
            if not trigger:
                return False
            
            # 添加任务到调度器
            self.scheduler.add_job(
                func=self._execute_task,
                trigger=trigger,
                args=[task_config.task_id],
                id=task_config.task_id,
                name=f"数据采集任务: {task_config.description}",
                replace_existing=True
            )
            
            # 保存任务配置
            self.task_configs[task_config.task_id] = task_config
            self.task_executions[task_config.task_id] = []
            
            logger.info(f"添加数据采集任务: {task_config.task_id}")
            return True
            
        except Exception as e:
            logger.error(f"添加任务失败 {task_config.task_id}: {e}")
            return False
    
    def remove_task(self, task_id: str) -> bool:
        """
        移除数据采集任务
        
        Args:
            task_id: 任务ID
            
        Returns:
            bool: 移除成功返回True
        """
        try:
            # 从调度器移除任务
            self.scheduler.remove_job(task_id)
            
            # 移除任务配置
            if task_id in self.task_configs:
                del self.task_configs[task_id]
            
            # 保留执行历史（用于统计分析）
            
            logger.info(f"移除数据采集任务: {task_id}")
            return True
            
        except Exception as e:
            logger.error(f"移除任务失败 {task_id}: {e}")
            return False
    
    def pause_task(self, task_id: str) -> bool:
        """暂停任务"""
        try:
            self.scheduler.pause_job(task_id)
            logger.info(f"暂停数据采集任务: {task_id}")
            return True
        except Exception as e:
            logger.error(f"暂停任务失败 {task_id}: {e}")
            return False
    
    def resume_task(self, task_id: str) -> bool:
        """恢复任务"""
        try:
            self.scheduler.resume_job(task_id)
            logger.info(f"恢复数据采集任务: {task_id}")
            return True
        except Exception as e:
            logger.error(f"恢复任务失败 {task_id}: {e}")
            return False
    
    def get_task_status(self, task_id: str) -> Optional[Dict[str, Any]]:
        """获取任务状态"""
        if task_id not in self.task_configs:
            return None
        
        task_config = self.task_configs[task_id]
        executions = self.task_executions.get(task_id, [])
        
        # 获取最近的执行记录
        last_execution = executions[-1] if executions else None
        
        # 获取调度器中的任务信息
        job = self.scheduler.get_job(task_id)
        
        return {
            'task_id': task_id,
            'connector_name': task_config.connector_name,
            'method_name': task_config.method_name,
            'enabled': task_config.enabled,
            'description': task_config.description,
            'next_run_time': job.next_run_time.isoformat() if job and job.next_run_time else None,
            'last_execution': {
                'start_time': last_execution.start_time.isoformat() if last_execution else None,
                'end_time': last_execution.end_time.isoformat() if last_execution and last_execution.end_time else None,
                'status': last_execution.status.value if last_execution else None,
                'retry_count': last_execution.retry_count if last_execution else 0
            },
            'total_executions': len(executions),
            'success_rate': self._calculate_success_rate(executions)
        }
    
    def list_tasks(self) -> List[Dict[str, Any]]:
        """获取所有任务状态"""
        return [self.get_task_status(task_id) for task_id in self.task_configs.keys()]
    
    async def _execute_task(self, task_id: str):
        """
        执行数据采集任务
        
        Args:
            task_id: 任务ID
        """
        execution_id = f"{task_id}_{datetime.utcnow().strftime('%Y%m%d_%H%M%S')}"
        execution = TaskExecution(
            task_id=task_id,
            execution_id=execution_id,
            start_time=datetime.utcnow(),
            status=TaskStatus.RUNNING
        )
        
        # 记录执行开始
        self.task_executions.setdefault(task_id, []).append(execution)
        logger.info(f"开始执行数据采集任务: {task_id}")
        
        try:
            # 获取任务配置
            task_config = self.task_configs[task_id]
            
            # 获取连接器
            connector = self.connector_registry.get_connector(task_config.connector_name)
            if not connector:
                raise Exception(f"连接器不存在: {task_config.connector_name}")
            
            # 获取方法
            method = getattr(connector, task_config.method_name, None)
            if not method:
                raise Exception(f"方法不存在: {task_config.method_name}")
            
            # 执行方法
            if asyncio.iscoroutinefunction(method):
                result = await method(**task_config.method_args)
            else:
                result = method(**task_config.method_args)
            
            # 记录执行成功
            execution.end_time = datetime.utcnow()
            execution.status = TaskStatus.SUCCESS
            execution.result = result
            
            logger.info(f"数据采集任务执行成功: {task_id}")
            
        except Exception as e:
            # 记录执行失败
            execution.end_time = datetime.utcnow()
            execution.status = TaskStatus.FAILED
            execution.error = str(e)
            
            logger.error(f"数据采集任务执行失败 {task_id}: {e}")
            logger.error(f"错误详情: {traceback.format_exc()}")
            
            # 判断是否需要重试
            await self._handle_task_retry(task_id, execution)
    
    async def _handle_task_retry(self, task_id: str, execution: TaskExecution):
        """处理任务重试"""
        task_config = self.task_configs[task_id]
        
        if execution.retry_count < task_config.max_retries:
            execution.retry_count += 1
            execution.status = TaskStatus.RETRYING
            
            # 计算退避时间
            backoff_time = task_config.retry_backoff * (2 ** (execution.retry_count - 1))
            retry_time = datetime.utcnow() + timedelta(seconds=backoff_time)
            
            # 安排重试
            self.scheduler.add_job(
                func=self._execute_task,
                trigger='date',
                run_date=retry_time,
                args=[task_id],
                id=f"{task_id}_retry_{execution.retry_count}",
                name=f"重试任务: {task_config.description} (第{execution.retry_count}次)"
            )
            
            logger.info(f"安排任务重试 {task_id}: 第{execution.retry_count}次，将在{retry_time}执行")
    
    def _validate_task_config(self, task_config: TaskConfig) -> bool:
        """验证任务配置"""
        if not task_config.task_id:
            logger.error("任务ID不能为空")
            return False
        
        if not task_config.connector_name:
            logger.error("连接器名称不能为空")
            return False
        
        if not task_config.method_name:
            logger.error("方法名称不能为空")
            return False
        
        if task_config.schedule_type not in ['interval', 'cron']:
            logger.error(f"不支持的调度类型: {task_config.schedule_type}")
            return False
        
        return True
    
    def _create_trigger(self, task_config: TaskConfig):
        """创建调度触发器"""
        try:
            if task_config.schedule_type == 'interval':
                return IntervalTrigger(**task_config.schedule_config)
            elif task_config.schedule_type == 'cron':
                return CronTrigger(**task_config.schedule_config)
            else:
                logger.error(f"不支持的调度类型: {task_config.schedule_type}")
                return None
        except Exception as e:
            logger.error(f"创建触发器失败: {e}")
            return None
    
    def _calculate_success_rate(self, executions: List[TaskExecution]) -> float:
        """计算任务成功率"""
        if not executions:
            return 0.0
        
        success_count = sum(1 for exec in executions if exec.status == TaskStatus.SUCCESS)
        return success_count / len(executions) * 100
    
    def _on_job_executed(self, event):
        """任务执行完成事件处理"""
        logger.debug(f"任务执行完成: {event.job_id}")
    
    def _on_job_error(self, event):
        """任务执行错误事件处理"""
        logger.error(f"任务执行错误: {event.job_id}, 错误: {event.exception}")
    
    def _on_job_missed(self, event):
        """任务执行错过事件处理"""
        logger.warning(f"任务执行错过: {event.job_id}") 