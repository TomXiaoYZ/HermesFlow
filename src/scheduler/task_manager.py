"""
任务管理器

提供高层次的数据采集任务管理API，包括：
- 预定义的数据采集任务模板
- 任务批量管理
- 任务配置验证和优化
- 任务性能监控

作者: HermesFlow Team
创建时间: 2024年12月26日
"""

import logging
import yaml
from typing import Dict, List, Optional, Any
from dataclasses import asdict
from datetime import datetime

from .scheduler import DataCollectionScheduler, TaskConfig, TaskStatus

logger = logging.getLogger(__name__)

class TaskManager:
    """任务管理器"""
    
    def __init__(self, scheduler: DataCollectionScheduler):
        """
        初始化任务管理器
        
        Args:
            scheduler: 数据采集调度器
        """
        self.scheduler = scheduler
        
        # 预定义任务模板
        self.task_templates = self._load_task_templates()
        
        logger.info("任务管理器初始化完成")
    
    def _load_task_templates(self) -> Dict[str, Dict[str, Any]]:
        """加载预定义任务模板"""
        templates = {
            # CEX 数据采集任务
            'binance_spot_ticker': {
                'connector_name': 'binance',
                'method_name': 'get_ticker',
                'schedule_type': 'interval',
                'schedule_config': {'seconds': 10},
                'description': 'Binance现货行情数据采集'
            },
            'binance_klines_1m': {
                'connector_name': 'binance',
                'method_name': 'get_klines',
                'method_args': {'interval': '1m', 'limit': 1000},
                'schedule_type': 'interval',
                'schedule_config': {'minutes': 1},
                'description': 'Binance 1分钟K线数据采集'
            },
            'okx_spot_ticker': {
                'connector_name': 'okx',
                'method_name': 'get_ticker',
                'schedule_type': 'interval',
                'schedule_config': {'seconds': 10},
                'description': 'OKX现货行情数据采集'
            },
            'bitget_spot_ticker': {
                'connector_name': 'bitget',
                'method_name': 'get_ticker',
                'schedule_type': 'interval',
                'schedule_config': {'seconds': 10},
                'description': 'Bitget现货行情数据采集'
            },
            
            # DEX 数据采集任务
            'gmgn_trending_tokens': {
                'connector_name': 'gmgn',
                'method_name': 'get_trending_tokens',
                'schedule_type': 'interval',
                'schedule_config': {'minutes': 5},
                'description': 'GMGN热门代币数据采集'
            },
            'gmgn_token_analysis': {
                'connector_name': 'gmgn',
                'method_name': 'get_token_analysis',
                'schedule_type': 'interval',
                'schedule_config': {'minutes': 30},
                'description': 'GMGN代币分析数据采集'
            },
            
            # 美股数据采集任务
            'polygon_stock_ticker': {
                'connector_name': 'polygon',
                'method_name': 'get_realtime_quotes',
                'schedule_type': 'interval',
                'schedule_config': {'seconds': 30},
                'description': 'Polygon美股实时报价采集'
            },
            'polygon_options_chain': {
                'connector_name': 'polygon',
                'method_name': 'get_options_chain',
                'schedule_type': 'interval',
                'schedule_config': {'minutes': 15},
                'description': 'Polygon期权链数据采集'
            },
            
            # 宏观经济数据采集任务
            'fred_economic_data': {
                'connector_name': 'fred',
                'method_name': 'get_series_data',
                'schedule_type': 'cron',
                'schedule_config': {'hour': 9, 'minute': 0},  # 每天9点执行
                'description': 'FRED宏观经济数据采集'
            },
            
            # 舆情数据采集任务
            'sentix_sentiment': {
                'connector_name': 'sentix',
                'method_name': 'get_sentiment_data',
                'schedule_type': 'interval',
                'schedule_config': {'hours': 6},
                'description': 'Sentix情绪指数数据采集'
            },
            'newsapi_financial_news': {
                'connector_name': 'newsapi',
                'method_name': 'get_top_headlines',
                'method_args': {'category': 'business'},
                'schedule_type': 'interval',
                'schedule_config': {'minutes': 30},
                'description': 'NewsAPI金融新闻采集'
            },
            'reddit_rss_trending': {
                'connector_name': 'reddit_rss',
                'method_name': 'get_trending_topics',
                'schedule_type': 'interval',
                'schedule_config': {'minutes': 15},
                'description': 'Reddit RSS热门话题采集'
            }
        }
        
        logger.info(f"加载了 {len(templates)} 个预定义任务模板")
        return templates
    
    def create_task_from_template(self, task_id: str, template_name: str, 
                                  custom_args: Optional[Dict[str, Any]] = None) -> Optional[TaskConfig]:
        """
        从模板创建任务配置
        
        Args:
            task_id: 任务ID
            template_name: 模板名称
            custom_args: 自定义参数
            
        Returns:
            TaskConfig: 任务配置对象
        """
        if template_name not in self.task_templates:
            logger.error(f"任务模板不存在: {template_name}")
            return None
        
        template = self.task_templates[template_name].copy()
        
        # 应用自定义参数
        if custom_args:
            template.update(custom_args)
        
        # 创建任务配置
        try:
            task_config = TaskConfig(
                task_id=task_id,
                connector_name=template['connector_name'],
                method_name=template['method_name'],
                method_args=template.get('method_args', {}),
                schedule_type=template['schedule_type'],
                schedule_config=template['schedule_config'],
                max_retries=template.get('max_retries', 3),
                retry_backoff=template.get('retry_backoff', 60),
                enabled=template.get('enabled', True),
                description=template['description']
            )
            
            logger.info(f"从模板 {template_name} 创建任务配置: {task_id}")
            return task_config
            
        except Exception as e:
            logger.error(f"创建任务配置失败: {e}")
            return None
    
    def add_task_from_template(self, task_id: str, template_name: str, 
                               custom_args: Optional[Dict[str, Any]] = None) -> bool:
        """
        从模板添加任务
        
        Args:
            task_id: 任务ID
            template_name: 模板名称
            custom_args: 自定义参数
            
        Returns:
            bool: 添加成功返回True
        """
        task_config = self.create_task_from_template(task_id, template_name, custom_args)
        if not task_config:
            return False
        
        return self.scheduler.add_task(task_config)
    
    def batch_add_tasks(self, task_definitions: List[Dict[str, Any]]) -> Dict[str, bool]:
        """
        批量添加任务
        
        Args:
            task_definitions: 任务定义列表
            
        Returns:
            Dict[str, bool]: 任务ID到添加结果的映射
        """
        results = {}
        
        for task_def in task_definitions:
            task_id = task_def.get('task_id')
            if not task_id:
                logger.error("任务定义缺少task_id")
                continue
            
            if 'template_name' in task_def:
                # 从模板创建
                results[task_id] = self.add_task_from_template(
                    task_id=task_id,
                    template_name=task_def['template_name'],
                    custom_args=task_def.get('custom_args')
                )
            else:
                # 直接创建
                try:
                    task_config = TaskConfig(**task_def)
                    results[task_id] = self.scheduler.add_task(task_config)
                except Exception as e:
                    logger.error(f"创建任务配置失败 {task_id}: {e}")
                    results[task_id] = False
        
        logger.info(f"批量添加任务完成，成功: {sum(results.values())}, 失败: {len(results) - sum(results.values())}")
        return results
    
    def batch_remove_tasks(self, task_ids: List[str]) -> Dict[str, bool]:
        """
        批量移除任务
        
        Args:
            task_ids: 任务ID列表
            
        Returns:
            Dict[str, bool]: 任务ID到移除结果的映射
        """
        results = {}
        
        for task_id in task_ids:
            results[task_id] = self.scheduler.remove_task(task_id)
        
        logger.info(f"批量移除任务完成，成功: {sum(results.values())}, 失败: {len(results) - sum(results.values())}")
        return results
    
    def get_task_statistics(self) -> Dict[str, Any]:
        """获取任务统计信息"""
        tasks = self.scheduler.list_tasks()
        
        total_tasks = len(tasks)
        running_tasks = sum(1 for task in tasks if task['last_execution']['status'] == TaskStatus.RUNNING.value)
        failed_tasks = sum(1 for task in tasks if task['last_execution']['status'] == TaskStatus.FAILED.value)
        success_tasks = sum(1 for task in tasks if task['last_execution']['status'] == TaskStatus.SUCCESS.value)
        
        # 计算整体成功率
        total_executions = sum(task['total_executions'] for task in tasks)
        overall_success_rate = sum(task['success_rate'] * task['total_executions'] for task in tasks) / total_executions if total_executions > 0 else 0
        
        return {
            'total_tasks': total_tasks,
            'running_tasks': running_tasks,
            'failed_tasks': failed_tasks,
            'success_tasks': success_tasks,
            'overall_success_rate': overall_success_rate,
            'total_executions': total_executions,
            'timestamp': datetime.utcnow().isoformat()
        }
    
    def export_task_configs(self, filepath: str):
        """导出任务配置到YAML文件"""
        tasks_data = []
        
        for task_id, task_config in self.scheduler.task_configs.items():
            task_data = asdict(task_config)
            tasks_data.append(task_data)
        
        with open(filepath, 'w', encoding='utf-8') as f:
            yaml.dump({'tasks': tasks_data}, f, default_flow_style=False, allow_unicode=True)
        
        logger.info(f"导出 {len(tasks_data)} 个任务配置到: {filepath}")
    
    def import_task_configs(self, filepath: str) -> Dict[str, bool]:
        """从YAML文件导入任务配置"""
        try:
            with open(filepath, 'r', encoding='utf-8') as f:
                data = yaml.safe_load(f)
            
            tasks_data = data.get('tasks', [])
            
            results = {}
            for task_data in tasks_data:
                task_id = task_data.get('task_id')
                if not task_id:
                    continue
                
                try:
                    task_config = TaskConfig(**task_data)
                    results[task_id] = self.scheduler.add_task(task_config)
                except Exception as e:
                    logger.error(f"导入任务配置失败 {task_id}: {e}")
                    results[task_id] = False
            
            logger.info(f"从 {filepath} 导入任务配置完成，成功: {sum(results.values())}, 失败: {len(results) - sum(results.values())}")
            return results
            
        except Exception as e:
            logger.error(f"导入任务配置文件失败: {e}")
            return {}
    
    def list_templates(self) -> List[str]:
        """获取所有可用的任务模板名称"""
        return list(self.task_templates.keys())
    
    def get_template_info(self, template_name: str) -> Optional[Dict[str, Any]]:
        """获取模板详细信息"""
        return self.task_templates.get(template_name)
    
    def optimize_task_schedules(self) -> Dict[str, Any]:
        """优化任务调度配置（避免资源冲突）"""
        # 获取所有任务
        tasks = self.scheduler.list_tasks()
        
        # 分析任务执行模式
        high_frequency_tasks = []
        medium_frequency_tasks = []
        low_frequency_tasks = []
        
        for task in tasks:
            # 简单的频率分类逻辑
            if 'seconds' in str(task):
                high_frequency_tasks.append(task['task_id'])
            elif 'minutes' in str(task) and int(str(task).split('minutes')[0][-2:]) < 30:
                medium_frequency_tasks.append(task['task_id'])
            else:
                low_frequency_tasks.append(task['task_id'])
        
        optimization_report = {
            'analysis': {
                'high_frequency_tasks': len(high_frequency_tasks),
                'medium_frequency_tasks': len(medium_frequency_tasks),
                'low_frequency_tasks': len(low_frequency_tasks)
            },
            'recommendations': [],
            'warnings': []
        }
        
        # 生成优化建议
        if len(high_frequency_tasks) > 10:
            optimization_report['warnings'].append("高频任务过多，可能导致系统负载过高")
            optimization_report['recommendations'].append("考虑合并部分高频任务或增加执行间隔")
        
        if len(medium_frequency_tasks) > 20:
            optimization_report['warnings'].append("中频任务过多，建议分散执行时间")
            optimization_report['recommendations'].append("使用错峰调度避免资源竞争")
        
        optimization_report['recommendations'].append("定期监控任务成功率和执行时间")
        optimization_report['recommendations'].append("为重要任务设置告警机制")
        
        return optimization_report 