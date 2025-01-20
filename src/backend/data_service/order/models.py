"""订单相关数据模型。"""

from dataclasses import dataclass
from datetime import datetime
from decimal import Decimal
from typing import Optional

from .enums import OrderType, OrderSide, OrderStatus, TimeInForce, OrderUpdateType

@dataclass
class OrderRecord:
    """订单记录。"""
    id: str
    exchange: str
    client_order_id: str
    symbol: str
    order_type: OrderType
    side: OrderSide
    price: Decimal
    quantity: Decimal
    status: OrderStatus
    create_time: datetime
    executed_qty: Optional[Decimal] = None
    avg_price: Optional[Decimal] = None
    time_in_force: Optional[TimeInForce] = None
    update_time: Optional[datetime] = None
    is_contract: bool = False
    position_side: Optional[str] = None
    margin_type: Optional[str] = None
    leverage: Optional[int] = None
    stop_price: Optional[Decimal] = None
    working_type: Optional[str] = None
    reduce_only: bool = False

@dataclass
class OrderUpdate:
    """订单更新记录。"""
    update_type: OrderUpdateType
    prev_status: OrderStatus
    curr_status: OrderStatus
    executed_qty: Decimal
    update_time: datetime
    reason: Optional[str] = None

@dataclass
class TradeRecord:
    """成交记录。"""
    exchange_trade_id: str
    price: Decimal
    quantity: Decimal
    commission: Decimal
    commission_asset: str
    trade_time: datetime
    realized_pnl: Optional[Decimal] = None

@dataclass
class OrderSummary:
    """订单汇总信息。"""
    exchange: str
    symbol: str
    start_time: datetime
    end_time: datetime
    total_orders: int
    filled_orders: int
    canceled_orders: int
    rejected_orders: int
    total_filled_qty: Decimal
    total_filled_amount: Decimal
    success_rate: Decimal 