"""
性能监控服务
"""
import asyncio
import time
import psutil
from datetime import datetime, timedelta
from typing import Dict, Any, Optional
import logging

from .service import OrderService
from .sync_service import OrderSyncService
from ..common.logger import get_logger

logger = get_logger("monitor_service")

class OrderMonitorService:
    """订单监控服务"""
    
    def __init__(self, order_service: OrderService, sync_service: OrderSyncService):
        """初始化监控服务
        
        Args:
            order_service: 订单服务实例
            sync_service: 同步服务实例
        """
        self.order_service = order_service
        self.sync_service = sync_service
        self.running = False
        self._monitor_task: Optional[asyncio.Task] = None
        self._metrics: Dict[str, Any] = {}
        
    async def start(self):
        """启动监控服务"""
        if self.running:
            return
            
        self.running = True
        self._monitor_task = asyncio.create_task(self._monitor_loop())
        logger.info("监控服务已启动")
    
    async def stop(self):
        """停止监控服务"""
        if not self.running:
            return
        
        self.running = False
        if self._monitor_task:
            self._monitor_task.cancel()
            try:
                await self._monitor_task
            except asyncio.CancelledError:
                pass
            self._monitor_task = None
        
        logging.info("监控服务已停止")
    
    async def _monitor_loop(self):
        """监控循环"""
        while self.running:
            try:
                # 收集性能指标
                await self._collect_metrics()
                # 保存监控数据
                await self._save_metrics()
                # 检查告警条件
                await self._check_alerts()
                # 等待下一次监控
                await asyncio.sleep(60)  # 每分钟监控一次
            except Exception as e:
                logger.error(f"监控出错: {str(e)}")
                await asyncio.sleep(5)  # 出错后等待5秒重试
    
    async def _collect_metrics(self):
        """收集性能指标"""
        try:
            # 系统资源使用
            process = psutil.Process()
            cpu_percent = process.cpu_percent()
            memory_info = process.memory_info()
            
            # 数据库连接状态
            pg_connected = await self._check_postgres_connection()
            redis_connected = await self._check_redis_connection()
            ch_connected = await self._check_clickhouse_connection()
            
            # 同步延迟
            sync_delay = await self._get_sync_delay()
            
            # 订单处理性能
            order_metrics = await self._get_order_metrics()
            
            # 更新指标
            self._metrics.update({
                "timestamp": int(datetime.now().timestamp() * 1000),
                "system": {
                    "cpu_percent": cpu_percent,
                    "memory_rss": memory_info.rss,
                    "memory_vms": memory_info.vms,
                    "thread_count": process.num_threads()
                },
                "connections": {
                    "postgres": pg_connected,
                    "redis": redis_connected,
                    "clickhouse": ch_connected
                },
                "sync": {
                    "delay": sync_delay,
                    "last_sync_time": await self.sync_service._get_last_sync_time()
                },
                "orders": order_metrics
            })
            
            logger.info("性能指标收集完成")
        except Exception as e:
            logger.error(f"性能指标收集失败: {str(e)}")
            raise
    
    async def _save_metrics(self):
        """保存监控指标到ClickHouse。"""
        try:
            query = """
                INSERT INTO system_metrics (
                    timestamp,
                    cpu_percent,
                    memory_rss,
                    memory_vms,
                    thread_count,
                    pg_connected,
                    redis_connected,
                    ch_connected,
                    sync_delay,
                    last_sync_time,
                    order_count_1m,
                    trade_count_1m,
                    error_count_1m,
                    avg_process_time_1m
                ) VALUES (
                    %(timestamp)s,
                    %(cpu_percent)s,
                    %(memory_rss)s,
                    %(memory_vms)s,
                    %(thread_count)s,
                    %(pg_connected)s,
                    %(redis_connected)s,
                    %(ch_connected)s,
                    %(sync_delay)s,
                    %(last_sync_time)s,
                    %(order_count_1m)s,
                    %(trade_count_1m)s,
                    %(error_count_1m)s,
                    %(avg_process_time_1m)s
                )
            """
            params = {
                "timestamp": int(datetime.now().timestamp() * 1000),
                "cpu_percent": self._metrics["system"]["cpu_percent"],
                "memory_rss": self._metrics["system"]["memory_rss"],
                "memory_vms": self._metrics["system"]["memory_vms"],
                "thread_count": self._metrics["system"]["thread_count"],
                "pg_connected": int(self._metrics["connections"]["postgres"]),
                "redis_connected": int(self._metrics["connections"]["redis"]),
                "ch_connected": int(self._metrics["connections"]["clickhouse"]),
                "sync_delay": self._metrics["sync"]["delay"],
                "last_sync_time": int(self._metrics["sync"]["last_sync_time"] * 1000),
                "order_count_1m": self._metrics["orders"]["count_1m"],
                "trade_count_1m": self._metrics["orders"]["trade_count_1m"],
                "error_count_1m": self._metrics["orders"]["error_count_1m"],
                "avg_process_time_1m": self._metrics["orders"]["avg_process_time_1m"]
            }
            result = await self.order_service.ch.execute_iter(query, params)
            async for _ in result:
                pass
            logging.info("监控数据保存完成")
        except Exception as e:
            logging.error(f"保存监控数据失败: {e}")
            raise
    
    async def _check_alerts(self):
        """检查告警条件"""
        if not self._metrics:
            return
        
        # 系统资源告警
        system = self._metrics.get("system", {})
        if system.get("cpu_percent", 0) > 80:
            logging.warning(f"CPU使用率过高: {system['cpu_percent']}%")
        if system.get("memory_rss", 0) > 4 * 1024 * 1024 * 1024:  # 4GB
            logging.warning(f"内存使用过高: {system['memory_rss'] / 1024 / 1024 / 1024:.2f}GB")
        
        # 连接状态告警
        connections = self._metrics.get("connections", {})
        if not connections.get("postgres", True):
            logging.error("PostgreSQL连接断开")
        if not connections.get("redis", True):
            logging.error("Redis连接断开")
        if not connections.get("clickhouse", True):
            logging.error("ClickHouse连接断开")
        
        # 同步延迟告警
        sync = self._metrics.get("sync", {})
        if sync.get("delay", 0) > 300:  # 5分钟
            logging.warning(f"数据同步延迟过高: {sync['delay']}秒")
        
        # 订单处理告警
        orders = self._metrics.get("orders", {})
        error_rate = orders.get("error_count_1m", 0) / max(orders.get("count_1m", 1), 1) * 100
        if error_rate > 5:
            logging.warning(f"订单处理错误率过高: {error_rate:.2f}%")
        if orders.get("avg_process_time_1m", 0) > 1000:  # 1秒
            logging.warning(f"订单处理延迟过高: {orders['avg_process_time_1m']}ms")
    
    async def _check_postgres_connection(self) -> bool:
        """检查PostgreSQL连接状态"""
        try:
            await self.order_service.pg.fetchrow("SELECT 1")
            return True
        except:
            return False
    
    async def _check_redis_connection(self) -> bool:
        """检查Redis连接状态"""
        try:
            await self.order_service.redis.get("test")
            return True
        except:
            return False
    
    async def _check_clickhouse_connection(self) -> bool:
        """检查ClickHouse连接状态。"""
        try:
            result = await self.order_service.ch.execute_iter("SELECT 1")
            async for _ in result:
                return True
            return False
        except Exception as e:
            logging.error(f"ClickHouse连接检查失败: {e}")
            return False
    
    async def _get_sync_delay(self) -> float:
        """获取同步延迟
        
        Returns:
            float: 同步延迟(秒)
        """
        try:
            last_sync_time = await self.sync_service._get_last_sync_time()
            if last_sync_time is None:
                return 0.0
            current_time = int(time.time() * 1000)
            return float(current_time - last_sync_time) / 1000
        except Exception as e:
            logging.error(f"获取同步时间失败: {e}")
            return 0.0
    
    async def _get_order_metrics(self) -> dict:
        """获取订单相关指标。"""
        try:
            # 获取最近1分钟的订单数量
            query = """
                SELECT
                    count(*) as order_count,
                    countIf(status = 'FILLED') as trade_count,
                    countIf(status = 'REJECTED' OR status = 'EXPIRED') as error_count,
                    avg(update_time - create_time) as avg_process_time
                FROM orders
                WHERE create_time >= now() - INTERVAL 1 MINUTE
            """
            result = await self.order_service.ch.execute_iter(query)
            metrics = {"count_1m": 0, "trade_count_1m": 0, "error_count_1m": 0, "avg_process_time_1m": 0}
            async for row in result:
                metrics["count_1m"] = row[0]
                metrics["trade_count_1m"] = row[1]
                metrics["error_count_1m"] = row[2]
                metrics["avg_process_time_1m"] = row[3] if row[3] is not None else 0
                break
            return metrics
        except Exception as e:
            logging.error(f"获取订单指标失败: {e}")
            return {
                "count_1m": 0,
                "trade_count_1m": 0,
                "error_count_1m": 0,
                "avg_process_time_1m": 0
            } 