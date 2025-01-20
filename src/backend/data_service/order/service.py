"""订单服务实现。"""

import json
import asyncio
from datetime import datetime, timedelta
from decimal import Decimal
from typing import List, Optional, Dict, Any

import redis
import asyncpg
import clickhouse_driver

from .models import OrderRecord, OrderUpdate, TradeRecord, OrderSummary
from .enums import OrderStatus, OrderUpdateType

class OrderService:
    """订单服务类，提供订单相关的操作。"""
    
    def __init__(self, redis_url: str, pg_url: str, ch_url: str):
        """初始化订单服务。
        
        Args:
            redis_url: Redis连接URL
            pg_url: PostgreSQL连接URL
            ch_url: ClickHouse连接URL
        """
        self._redis_url = redis_url
        self._pg_url = pg_url
        self._ch_url = ch_url
        self._redis = None
        self._pg_pool = None
        self._ch_client = None
        
    @property
    def redis(self):
        """获取Redis客户端"""
        return self._redis
        
    @property
    def ch(self):
        """获取ClickHouse客户端"""
        return self._ch_client
        
    @property
    def clickhouse(self):
        """获取ClickHouse客户端（别名）"""
        return self._ch_client
        
    async def start(self):
        """启动服务，初始化数据库连接。"""
        # 初始化Redis连接
        redis_future = await redis.Redis.from_url(self._redis_url)
        self._redis = redis_future.result() if hasattr(redis_future, 'result') else redis_future

        # 初始化PostgreSQL连接池
        pg_pool_future = await asyncpg.create_pool(self._pg_url)
        self._pg_pool = pg_pool_future.result() if hasattr(pg_pool_future, 'result') else pg_pool_future

        # 验证PostgreSQL连接
        acquire_context = await self._pg_pool.acquire()
        async with acquire_context as conn:
            await conn.execute("SELECT 1")

        # 初始化ClickHouse客户端
        ch_client_future = await clickhouse_driver.Client.from_url(self._ch_url)
        self._ch_client = ch_client_future.result() if hasattr(ch_client_future, 'result') else ch_client_future
        
    async def stop(self):
        """停止服务，关闭数据库连接。"""
        if self._pg_pool:
            await self._pg_pool.close()
        if self._ch_client:
            await self._ch_client.disconnect()
        if self._redis:
            await self._redis.close()
        
    async def create_order(self, order: OrderRecord) -> str:
        """创建新订单。
        
        Args:
            order: 订单记录对象
            
        Returns:
            str: 订单ID
        """
        try:
            acquire_context = await self._pg_pool.acquire()
            async with acquire_context as conn:
                order_id = await conn.fetchval(
                    """
                    INSERT INTO orders (
                        exchange, client_order_id, symbol, order_type, 
                        side, price, quantity, status, create_time
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                    RETURNING id
                    """,
                    order.exchange, order.client_order_id, order.symbol,
                    order.order_type.value, order.side.value,
                    str(order.price), str(order.quantity),
                    order.status.value, order.create_time
                )
                
                if order.status != OrderStatus.CANCELED:
                    order_key = f"order:{order.exchange}:{order.symbol}:{order_id}"
                    await self._redis.hset(
                        order_key,
                        mapping={
                            "id": order_id,
                            "exchange": order.exchange,
                            "symbol": order.symbol,
                            "status": order.status.value,
                            "price": str(order.price),
                            "quantity": str(order.quantity),
                            "create_time": order.create_time.isoformat()
                        }
                    )
                    await self._redis.expire(order_key, timedelta(days=1))
                    
                return order_id
        except Exception as e:
            # TODO: Add proper logging
            print(f"Error creating order: {e}")
            raise
            
    async def update_order(self, order_id: str, update: OrderUpdate):
        """更新订单状态。
        
        Args:
            order_id: 订单ID
            update: 订单更新对象
        """
        try:
            acquire_context = await self._pg_pool.acquire()
            async with acquire_context as conn:
                await conn.execute(
                    """
                    INSERT INTO order_updates (
                        order_id, status, executed_qty, avg_price, update_time
                    ) VALUES ($1, $2, $3, $4, $5)
                    """,
                    order_id, update.status.value,
                    str(update.executed_qty), str(update.avg_price),
                    update.update_time
                )
                
                if update.status != OrderStatus.CANCELED:
                    order = await self.get_order(order_id)
                    if order:
                        order_key = f"order:{order.exchange}:{order.symbol}:{order_id}"
                        await self._redis.hset(
                            order_key,
                            mapping={
                                "status": update.status.value,
                                "executed_qty": str(update.executed_qty),
                                "avg_price": str(update.avg_price),
                                "update_time": update.update_time.isoformat()
                            }
                        )
                        await self._redis.expire(order_key, timedelta(days=1))
        except Exception as e:
            # TODO: Add proper logging
            print(f"Error updating order: {e}")
            raise
            
    async def add_trade(self, trade: TradeRecord):
        """添加成交记录。
        
        Args:
            trade: 成交记录对象
        """
        try:
            acquire_context = await self._pg_pool.acquire()
            async with acquire_context as conn:
                await conn.execute(
                    """
                    INSERT INTO trades (
                        order_id, trade_id, price, quantity, commission,
                        commission_asset, trade_time
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7)
                    """,
                    trade.order_id, trade.trade_id,
                    str(trade.price), str(trade.quantity),
                    str(trade.commission), trade.commission_asset,
                    trade.trade_time
                )
        except Exception as e:
            # TODO: Add proper logging
            print(f"Error adding trade: {e}")
            raise
            
    async def get_order(self, order_id: str) -> Optional[OrderRecord]:
        """获取订单信息。
        
        Args:
            order_id: 订单ID
            
        Returns:
            Optional[OrderRecord]: 订单记录对象，如果不存在则返回None
        """
        try:
            # 先从Redis获取
            order = None
            redis_result = await self._redis.hgetall(f"order:*:{order_id}")
            if redis_result:
                order = OrderRecord(
                    id=order_id,
                    exchange=redis_result[b"exchange"].decode(),
                    symbol=redis_result[b"symbol"].decode(),
                    status=OrderStatus(redis_result[b"status"].decode()),
                    price=Decimal(redis_result[b"price"].decode()),
                    quantity=Decimal(redis_result[b"quantity"].decode()),
                    create_time=datetime.fromisoformat(redis_result[b"create_time"].decode())
                )
            else:
                # 从PostgreSQL获取
                acquire_context = await self._pg_pool.acquire()
                async with acquire_context as conn:
                    row = await conn.fetchrow(
                        """
                        SELECT * FROM orders WHERE id = $1
                        """,
                        order_id
                    )
                    if row:
                        order = OrderRecord(
                            id=row["id"],
                            exchange=row["exchange"],
                            symbol=row["symbol"],
                            status=OrderStatus(row["status"]),
                            price=Decimal(str(row["price"])),
                            quantity=Decimal(str(row["quantity"])),
                            create_time=row["create_time"]
                        )
            return order
        except Exception as e:
            # TODO: Add proper logging
            print(f"Error getting order: {e}")
            raise
            
    async def get_order_updates(
        self,
        start_time: Optional[datetime] = None,
        end_time: Optional[datetime] = None
    ) -> List[OrderUpdate]:
        """获取订单更新记录。
        
        Args:
            start_time: 开始时间
            end_time: 结束时间
            
        Returns:
            List[OrderUpdate]: 订单更新记录列表
        """
        try:
            acquire_context = await self._pg_pool.acquire()
            async with acquire_context as conn:
                query = "SELECT * FROM order_updates WHERE 1=1"
                params = []
                if start_time:
                    query += " AND update_time >= $1"
                    params.append(start_time)
                if end_time:
                    query += f" AND update_time <= ${len(params) + 1}"
                    params.append(end_time)
                query += " ORDER BY update_time DESC"
                
                rows = await conn.fetch(query, *params)
                return [
                    OrderUpdate(
                        order_id=row["order_id"],
                        status=OrderStatus(row["status"]),
                        executed_qty=Decimal(str(row["executed_qty"])),
                        avg_price=Decimal(str(row["avg_price"])),
                        update_time=row["update_time"]
                    )
                    for row in rows
                ]
        except Exception as e:
            # TODO: Add proper logging
            print(f"Error getting order updates: {e}")
            raise
            
    async def get_order_trades(
        self,
        order_id: str,
        start_time: Optional[datetime] = None,
        end_time: Optional[datetime] = None
    ) -> List[TradeRecord]:
        """获取订单成交记录。
        
        Args:
            order_id: 订单ID
            start_time: 开始时间
            end_time: 结束时间
            
        Returns:
            List[TradeRecord]: 成交记录列表
        """
        try:
            query = """
                SELECT * FROM trades 
                WHERE order_id = $1
            """
            params = [order_id]
            
            if start_time:
                query += " AND trade_time >= $2"
                params.append(start_time)
            if end_time:
                query += " AND trade_time <= $3"
                params.append(end_time)
                
            query += " ORDER BY trade_time ASC"
            
            async with self._pg_pool.acquire() as conn:
                rows = await conn.fetch(query, *params)
                return [
                    TradeRecord(
                        exchange_trade_id=row["exchange_trade_id"],
                        price=Decimal(str(row["price"])),
                        quantity=Decimal(str(row["quantity"])),
                        commission=Decimal(str(row["commission"])),
                        commission_asset=row["commission_asset"],
                        realized_pnl=Decimal(str(row["realized_pnl"])),
                        trade_time=row["trade_time"]
                    )
                    for row in rows
                ]
        except Exception as e:
            # TODO: Add proper logging
            print(f"Error getting order trades: {e}")
            raise
            
    async def get_active_orders(
        self,
        exchange: Optional[str] = None,
        symbol: Optional[str] = None
    ) -> List[OrderRecord]:
        """获取活动订单列表。
        
        Args:
            exchange: 交易所名称
            symbol: 交易对
            
        Returns:
            List[OrderRecord]: 活动订单列表
        """
        try:
            pattern = "order:"
            if exchange:
                pattern += f"{exchange}:"
            else:
                pattern += "*:"
            if symbol:
                pattern += f"{symbol}:*"
            else:
                pattern += "*:*"
                
            keys = self._redis.keys(pattern)
            if not keys:
                return []
                
            orders = []
            for key in keys:
                data = self._redis.hgetall(key)
                if data:
                    orders.append(
                        OrderRecord(
                            id=data["id"],
                            exchange=data["exchange"],
                            symbol=data["symbol"],
                            status=OrderStatus(data["status"]),
                            price=Decimal(data["price"]),
                            quantity=Decimal(data["quantity"]),
                            create_time=datetime.fromisoformat(data["create_time"])
                        )
                    )
            return orders
        except Exception as e:
            # TODO: Add proper logging
            print(f"Error getting active orders: {e}")
            raise
            
    async def get_order_summary(
        self,
        exchange: str,
        symbol: str,
        start_time: datetime,
        end_time: datetime
    ) -> OrderSummary:
        """获取订单汇总信息。
        
        Args:
            exchange: 交易所名称
            symbol: 交易对
            start_time: 开始时间
            end_time: 结束时间
            
        Returns:
            OrderSummary: 订单汇总信息
        """
        try:
            query = """
                SELECT
                    COUNT(*) as total_orders,
                    COUNT(*) FILTER (WHERE status = 'FILLED') as filled_orders,
                    COUNT(*) FILTER (WHERE status = 'CANCELED') as canceled_orders,
                    COUNT(*) FILTER (WHERE status = 'REJECTED') as rejected_orders,
                    SUM(CASE WHEN status = 'FILLED' THEN quantity ELSE 0 END) as total_filled_qty,
                    SUM(CASE WHEN status = 'FILLED' THEN price * quantity ELSE 0 END) as total_filled_amount
                FROM orders
                WHERE exchange = $1
                    AND symbol = $2
                    AND create_time BETWEEN $3 AND $4
            """
            
            async with self._pg_pool.acquire() as conn:
                row = await conn.fetchrow(query, exchange, symbol, start_time, end_time)
                
                if not row:
                    return OrderSummary(
                        exchange=exchange,
                        symbol=symbol,
                        start_time=start_time,
                        end_time=end_time,
                        total_orders=0,
                        filled_orders=0,
                        canceled_orders=0,
                        rejected_orders=0,
                        total_filled_qty=Decimal("0"),
                        total_filled_amount=Decimal("0"),
                        success_rate=Decimal("0")
                    )
                    
                total_orders = row["total_orders"]
                filled_orders = row["filled_orders"]
                success_rate = Decimal(str(filled_orders / total_orders)) if total_orders > 0 else Decimal("0")
                
                return OrderSummary(
                    exchange=exchange,
                    symbol=symbol,
                    start_time=start_time,
                    end_time=end_time,
                    total_orders=total_orders,
                    filled_orders=filled_orders,
                    canceled_orders=row["canceled_orders"],
                    rejected_orders=row["rejected_orders"],
                    total_filled_qty=Decimal(str(row["total_filled_qty"] or 0)),
                    total_filled_amount=Decimal(str(row["total_filled_amount"] or 0)),
                    success_rate=success_rate
                )
        except Exception as e:
            # TODO: Add proper logging
            print(f"Error getting order summary: {e}")
            raise 