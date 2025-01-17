"""
订单仓库
"""
from typing import List, Optional
from datetime import datetime
from sqlalchemy import select, and_
from sqlalchemy.ext.asyncio import AsyncSession

from ..models.order import Order, OrderUpdate, Trade
from ..db.events import publish_event, batch_publish_events
from ..db.connection import db_manager
from ..common.models import Exchange, Market, OrderStatus

class OrderRepository:
    """订单仓库"""

    def __init__(self):
        self.redis = db_manager.get_redis()

    @publish_event("order.created")
    async def create_order(self, session: AsyncSession, order: Order) -> Order:
        """创建订单

        Args:
            session: 数据库会话
            order: 订单对象

        Returns:
            Order: 创建的订单
        """
        # 保存到数据库
        session.add(order)
        await session.flush()

        # 创建订单更新记录
        order_update = OrderUpdate(
            order_id=order.id,
            update_type="create",
            prev_status=None,
            new_status=order.status,
            executed_quantity=order.executed_quantity,
            remaining_quantity=order.quantity - order.executed_quantity
        )
        session.add(order_update)
        await session.commit()

        # 缓存活跃订单
        if order.status not in [OrderStatus.FILLED, OrderStatus.CANCELED, OrderStatus.REJECTED]:
            await self.redis.hset(
                f"active_orders:{order.exchange}:{order.market}",
                order.exchange_order_id,
                order.id
            )

        return order

    @publish_event("order.updated")
    async def update_order(
        self,
        session: AsyncSession,
        order_id: int,
        new_status: OrderStatus,
        executed_quantity: float
    ) -> Order:
        """更新订单

        Args:
            session: 数据库会话
            order_id: 订单ID
            new_status: 新状态
            executed_quantity: 已成交数量

        Returns:
            Order: 更新后的订单
        """
        # 获取订单
        order = await session.get(Order, order_id)
        if not order:
            raise ValueError(f"订单不存在: {order_id}")

        # 创建更新记录
        order_update = OrderUpdate(
            order_id=order.id,
            update_type="update",
            prev_status=order.status,
            new_status=new_status,
            executed_quantity=executed_quantity,
            remaining_quantity=order.quantity - executed_quantity
        )
        session.add(order_update)

        # 更新订单
        order.status = new_status
        order.executed_quantity = executed_quantity
        order.updated_at = datetime.utcnow()
        await session.commit()

        # 更新缓存
        if new_status in [OrderStatus.FILLED, OrderStatus.CANCELED, OrderStatus.REJECTED]:
            await self.redis.hdel(
                f"active_orders:{order.exchange}:{order.market}",
                order.exchange_order_id
            )

        return order

    @publish_event("order.canceled")
    async def cancel_order(self, session: AsyncSession, order_id: int) -> Order:
        """取消订单

        Args:
            session: 数据库会话
            order_id: 订单ID

        Returns:
            Order: 取消的订单
        """
        return await self.update_order(
            session,
            order_id,
            OrderStatus.CANCELED,
            0
        )

    @publish_event("trade.created")
    async def add_trade(self, session: AsyncSession, trade: Trade) -> Trade:
        """添加成交记录

        Args:
            session: 数据库会话
            trade: 成交记录

        Returns:
            Trade: 创建的成交记录
        """
        session.add(trade)
        await session.commit()
        return trade

    async def get_order(
        self,
        session: AsyncSession,
        exchange: Exchange,
        market: Market,
        exchange_order_id: str
    ) -> Optional[Order]:
        """获取订单

        Args:
            session: 数据库会话
            exchange: 交易所
            market: 市场类型
            exchange_order_id: 交易所订单ID

        Returns:
            Optional[Order]: 订单对象
        """
        stmt = select(Order).where(
            and_(
                Order.exchange == exchange,
                Order.market == market,
                Order.exchange_order_id == exchange_order_id
            )
        )
        result = await session.execute(stmt)
        return result.scalar_one_or_none()

    async def get_active_orders(
        self,
        session: AsyncSession,
        exchange: Exchange,
        market: Market
    ) -> List[Order]:
        """获取活跃订单

        Args:
            session: 数据库会话
            exchange: 交易所
            market: 市场类型

        Returns:
            List[Order]: 活跃订单列表
        """
        # 从Redis获取活跃订单ID
        order_ids = await self.redis.hgetall(f"active_orders:{exchange}:{market}")
        if not order_ids:
            return []

        # 从数据库获取订单详情
        stmt = select(Order).where(Order.id.in_(order_ids.values()))
        result = await session.execute(stmt)
        return result.scalars().all()

    async def get_order_updates(
        self,
        session: AsyncSession,
        order_id: int
    ) -> List[OrderUpdate]:
        """获取订单更新记录

        Args:
            session: 数据库会话
            order_id: 订单ID

        Returns:
            List[OrderUpdate]: 订单更新记录列表
        """
        stmt = select(OrderUpdate).where(OrderUpdate.order_id == order_id)
        result = await session.execute(stmt)
        return result.scalars().all()

    async def get_order_trades(
        self,
        session: AsyncSession,
        order_id: int
    ) -> List[Trade]:
        """获取订单成交记录

        Args:
            session: 数据库会话
            order_id: 订单ID

        Returns:
            List[Trade]: 成交记录列表
        """
        stmt = select(Trade).where(Trade.order_id == order_id)
        result = await session.execute(stmt)
        return result.scalars().all() 