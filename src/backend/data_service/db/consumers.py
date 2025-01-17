"""
事件消费者服务
"""
import json
import asyncio
from typing import List, Dict, Any
from datetime import datetime
from decimal import Decimal

from .connection import db_manager
from .config import KAFKA_TOPICS
from ..models.order import Order, OrderUpdate, Trade
from ..common.models import Exchange, Market, OrderType, OrderSide, OrderStatus

class EventConsumer:
    """事件消费者"""

    def __init__(self):
        """初始化事件消费者"""
        self.consumer = None
        self.running = False

    async def start(self):
        """启动消费者"""
        if self.running:
            return

        # 创建消费者
        self.consumer = await db_manager.create_kafka_consumer([
            KAFKA_TOPICS["order_events"],
            KAFKA_TOPICS["trade_events"]
        ])
        self.running = True

        # 开始消费
        try:
            async for message in self.consumer:
                await self._process_message(message)
        finally:
            await self.stop()

    async def stop(self):
        """停止消费者"""
        self.running = False
        if self.consumer:
            await self.consumer.stop()

    async def _process_message(self, message):
        """处理消息

        Args:
            message: Kafka消息
        """
        try:
            # 解析消息
            event = json.loads(message.value.decode())
            event_type = event["event_type"]
            data = event["data"]

            # 根据事件类型处理
            if "order" in event_type:
                await self._handle_order_event(event_type, data)
            elif "trade" in event_type:
                await self._handle_trade_event(data)

        except Exception as e:
            print(f"处理消息失败: {str(e)}")
            # TODO: 实现错误重试机制
            # TODO: 发送告警

    async def _handle_order_event(self, event_type: str, data: Dict[str, Any]):
        """处理订单事件

        Args:
            event_type: 事件类型
            data: 事件数据
        """
        async with db_manager.get_session() as session:
            if event_type == "order.created":
                # 创建订单
                order = Order(
                    exchange_order_id=data["exchange_order_id"],
                    client_order_id=data["client_order_id"],
                    exchange=Exchange[data["exchange"]],
                    market=Market[data["market"]],
                    symbol=data["symbol"],
                    type=OrderType[data["type"]],
                    side=OrderSide[data["side"]],
                    price=Decimal(str(data["price"])),
                    quantity=Decimal(str(data["quantity"])),
                    executed_quantity=Decimal(str(data.get("executed_quantity", 0))),
                    status=OrderStatus[data["status"]],
                    created_at=datetime.fromisoformat(data["created_at"]),
                    updated_at=datetime.fromisoformat(data["updated_at"])
                )
                session.add(order)
                await session.flush()

                # 创建订单更新记录
                order_update = OrderUpdate(
                    order_id=order.id,
                    update_type="create",
                    prev_status=None,
                    new_status=order.status,
                    executed_quantity=order.executed_quantity,
                    remaining_quantity=order.quantity - order.executed_quantity,
                    created_at=datetime.utcnow()
                )
                session.add(order_update)

            elif event_type == "order.updated":
                # 更新订单
                stmt = select(Order).where(
                    and_(
                        Order.exchange == Exchange[data["exchange"]],
                        Order.exchange_order_id == data["exchange_order_id"]
                    )
                )
                result = await session.execute(stmt)
                order = result.scalar_one()

                # 创建更新记录
                order_update = OrderUpdate(
                    order_id=order.id,
                    update_type="update",
                    prev_status=order.status,
                    new_status=OrderStatus[data["status"]],
                    executed_quantity=Decimal(str(data["executed_quantity"])),
                    remaining_quantity=order.quantity - Decimal(str(data["executed_quantity"])),
                    created_at=datetime.utcnow()
                )
                session.add(order_update)

                # 更新订单
                order.status = OrderStatus[data["status"]]
                order.executed_quantity = Decimal(str(data["executed_quantity"]))
                order.updated_at = datetime.fromisoformat(data["updated_at"])

            elif event_type == "order.canceled":
                # 取消订单
                stmt = select(Order).where(
                    and_(
                        Order.exchange == Exchange[data["exchange"]],
                        Order.exchange_order_id == data["exchange_order_id"]
                    )
                )
                result = await session.execute(stmt)
                order = result.scalar_one()

                # 创建更新记录
                order_update = OrderUpdate(
                    order_id=order.id,
                    update_type="cancel",
                    prev_status=order.status,
                    new_status=OrderStatus.CANCELED,
                    executed_quantity=order.executed_quantity,
                    remaining_quantity=order.quantity - order.executed_quantity,
                    created_at=datetime.utcnow()
                )
                session.add(order_update)

                # 更新订单
                order.status = OrderStatus.CANCELED
                order.updated_at = datetime.utcnow()

            await session.commit()

    async def _handle_trade_event(self, data: Dict[str, Any]):
        """处理成交事件

        Args:
            data: 事件数据
        """
        async with db_manager.get_session() as session:
            # 查找订单
            stmt = select(Order).where(
                and_(
                    Order.exchange == Exchange[data["exchange"]],
                    Order.exchange_order_id == data["order_id"]
                )
            )
            result = await session.execute(stmt)
            order = result.scalar_one()

            # 创建成交记录
            trade = Trade(
                order_id=order.id,
                exchange_trade_id=data["trade_id"],
                price=Decimal(str(data["price"])),
                quantity=Decimal(str(data["quantity"])),
                fee=Decimal(str(data["fee"])),
                fee_asset=data["fee_asset"],
                created_at=datetime.fromisoformat(data["created_at"])
            )
            session.add(trade)
            await session.commit()

# 全局事件消费者实例
event_consumer = EventConsumer() 