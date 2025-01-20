"""
数据同步服务
"""
import asyncio
import logging
from datetime import datetime, timedelta
from typing import List, Dict, Any, Optional

from .service import OrderService
from .models import OrderSummary
from ..common.logger import get_logger

logger = get_logger("sync_service")

class OrderSyncService:
    """订单数据同步服务"""
    
    def __init__(self, order_service: OrderService):
        """初始化同步服务
        
        Args:
            order_service: 订单服务实例
        """
        self.order_service = order_service
        self.running = False
        self._sync_task: Optional[asyncio.Task] = None
        self._summary_task: Optional[asyncio.Task] = None
    
    async def start(self):
        """启动同步服务"""
        if self.running:
            return
            
        self.running = True
        self._sync_task = asyncio.create_task(self._sync_loop())
        self._summary_task = asyncio.create_task(self._summary_loop())
        logger.info("同步服务已启动")
    
    async def stop(self):
        """停止同步服务"""
        self.running = False
        if self._sync_task:
            self._sync_task.cancel()
            try:
                await self._sync_task
            except asyncio.CancelledError:
                pass
        if self._summary_task:
            self._summary_task.cancel()
            try:
                await self._summary_task
            except asyncio.CancelledError:
                pass
        logger.info("同步服务已停止")
    
    async def _sync_loop(self):
        """数据同步循环"""
        while self.running:
            try:
                # 同步订单数据
                await self._sync_orders()
                # 同步成交记录
                await self._sync_trades()
                # 等待下一次同步
                await asyncio.sleep(60)  # 每分钟同步一次
            except Exception as e:
                logger.error(f"数据同步出错: {str(e)}")
                await asyncio.sleep(5)  # 出错后等待5秒重试
    
    async def _summary_loop(self):
        """订单汇总计算循环"""
        while self.running:
            try:
                # 计算订单汇总
                await self._calculate_summary()
                # 等待下一次计算
                await asyncio.sleep(300)  # 每5分钟计算一次
            except Exception as e:
                logger.error(f"订单汇总计算出错: {str(e)}")
                await asyncio.sleep(5)  # 出错后等待5秒重试
    
    async def _sync_orders(self):
        """同步订单数据到ClickHouse"""
        query = """
            INSERT INTO order_analysis (
                exchange, symbol, timestamp,
                order_count, trade_count, volume, value,
                avg_price, commission, is_contract,
                position_side, realized_pnl
            )
            SELECT
                exchange,
                symbol,
                toDateTime64(created_time / 1000, 3) as timestamp,
                count(*) as order_count,
                sum(case when status = 'FILLED' then 1 else 0 end) as trade_count,
                sum(executed_qty) as volume,
                sum(executed_qty * avg_price) as value,
                sum(executed_qty * avg_price) / sum(executed_qty) as avg_price,
                0 as commission,
                is_contract,
                position_side,
                0 as realized_pnl
            FROM orders
            WHERE created_time >= {start_time:Float64}
                AND created_time < {end_time:Float64}
            GROUP BY
                exchange,
                symbol,
                timestamp,
                is_contract,
                position_side
        """
        
        # 获取上次同步时间
        last_sync_time = await self._get_last_sync_time()
        current_time = int(datetime.now().timestamp() * 1000)
        
        params = {
            "start_time": last_sync_time,
            "end_time": current_time
        }
        
        try:
            # 执行同步
            await self.order_service.ch.execute(query, params)
            # 更新同步时间
            await self._update_last_sync_time(current_time)
            logger.info(f"订单数据同步完成: {last_sync_time} -> {current_time}")
        except Exception as e:
            logger.error(f"订单数据同步失败: {str(e)}")
            raise
    
    async def _sync_trades(self):
        """同步成交记录到ClickHouse"""
        query = """
            INSERT INTO order_analysis (
                exchange, symbol, timestamp,
                order_count, trade_count, volume, value,
                avg_price, commission, is_contract,
                position_side, realized_pnl
            )
            SELECT
                exchange,
                symbol,
                toDateTime64(created_time / 1000, 3) as timestamp,
                0 as order_count,
                count(*) as trade_count,
                sum(quantity) as volume,
                sum(quantity * price) as value,
                sum(quantity * price) / sum(quantity) as avg_price,
                sum(commission) as commission,
                is_contract,
                position_side,
                sum(realized_pnl) as realized_pnl
            FROM trades
            WHERE created_time >= {start_time:Float64}
                AND created_time < {end_time:Float64}
            GROUP BY
                exchange,
                symbol,
                timestamp,
                is_contract,
                position_side
        """
        
        # 获取上次同步时间
        last_sync_time = await self._get_last_sync_time()
        current_time = int(datetime.now().timestamp() * 1000)
        
        params = {
            "start_time": last_sync_time,
            "end_time": current_time
        }
        
        try:
            # 执行同步
            await self.order_service.ch.execute(query, params)
            logger.info(f"成交记录同步完成: {last_sync_time} -> {current_time}")
        except Exception as e:
            logger.error(f"成交记录同步失败: {str(e)}")
            raise
    
    async def _calculate_summary(self):
        """计算订单汇总数据"""
        # 计算1小时汇总
        await self._calculate_hourly_summary()
        # 计算日汇总
        await self._calculate_daily_summary()
        
    async def _calculate_hourly_summary(self):
        """计算小时汇总数据"""
        query = """
            INSERT INTO order_summary (
                id, exchange, symbol, start_time, end_time,
                total_orders, filled_orders, canceled_orders,
                rejected_orders, expired_orders, total_volume,
                total_value, total_commission, avg_execution_time,
                success_rate, is_contract, position_side, realized_pnl
            )
            SELECT
                generateUUIDv4() as id,
                exchange,
                symbol,
                toStartOfHour(timestamp) as start_time,
                toStartOfHour(timestamp) + INTERVAL 1 HOUR as end_time,
                sum(order_count) as total_orders,
                sum(trade_count) as filled_orders,
                0 as canceled_orders,
                0 as rejected_orders,
                0 as expired_orders,
                sum(volume) as total_volume,
                sum(value) as total_value,
                sum(commission) as total_commission,
                0 as avg_execution_time,
                sum(trade_count) / sum(order_count) * 100 as success_rate,
                is_contract,
                position_side,
                sum(realized_pnl) as realized_pnl
            FROM order_analysis
            WHERE timestamp >= toStartOfHour(now()) - INTERVAL 1 DAY
                AND timestamp < toStartOfHour(now())
            GROUP BY
                exchange,
                symbol,
                start_time,
                end_time,
                is_contract,
                position_side
            ORDER BY start_time DESC
        """
        
        try:
            await self.order_service.ch.execute(query)
            logger.info("小时汇总数据计算完成")
        except Exception as e:
            logger.error(f"小时汇总数据计算失败: {str(e)}")
            raise
    
    async def _calculate_daily_summary(self):
        """计算日汇总数据"""
        query = """
            INSERT INTO order_summary (
                id, exchange, symbol, start_time, end_time,
                total_orders, filled_orders, canceled_orders,
                rejected_orders, expired_orders, total_volume,
                total_value, total_commission, avg_execution_time,
                success_rate, is_contract, position_side, realized_pnl
            )
            SELECT
                generateUUIDv4() as id,
                exchange,
                symbol,
                toStartOfDay(timestamp) as start_time,
                toStartOfDay(timestamp) + INTERVAL 1 DAY as end_time,
                sum(order_count) as total_orders,
                sum(trade_count) as filled_orders,
                0 as canceled_orders,
                0 as rejected_orders,
                0 as expired_orders,
                sum(volume) as total_volume,
                sum(value) as total_value,
                sum(commission) as total_commission,
                0 as avg_execution_time,
                sum(trade_count) / sum(order_count) * 100 as success_rate,
                is_contract,
                position_side,
                sum(realized_pnl) as realized_pnl
            FROM order_analysis
            WHERE timestamp >= toStartOfDay(now()) - INTERVAL 7 DAY
                AND timestamp < toStartOfDay(now())
            GROUP BY
                exchange,
                symbol,
                start_time,
                end_time,
                is_contract,
                position_side
            ORDER BY start_time DESC
        """
        
        try:
            await self.order_service.ch.execute(query)
            logger.info("日汇总数据计算完成")
        except Exception as e:
            logger.error(f"日汇总数据计算失败: {str(e)}")
            raise
    
    async def _get_last_sync_time(self) -> int:
        """获取上次同步时间
        
        Returns:
            int: 上次同步时间戳(毫秒)
        """
        try:
            value = await self.order_service.redis.get("order_sync:last_sync_time")
            if value:
                return int(value)
            # 默认同步最近1小时的数据
            return int((datetime.now() - timedelta(hours=1)).timestamp() * 1000)
        except Exception as e:
            logger.error(f"获取同步时间失败: {str(e)}")
            raise
    
    async def _update_last_sync_time(self, timestamp: int):
        """更新同步时间
        
        Args:
            timestamp: 时间戳(毫秒)
        """
        try:
            await self.order_service.redis.set("order_sync:last_sync_time", str(timestamp))
        except Exception as e:
            logger.error(f"更新同步时间失败: {str(e)}")
            raise 