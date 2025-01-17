"""
订单相关的数据模型
"""
from datetime import datetime
from decimal import Decimal
from typing import Optional
from sqlalchemy import (
    Column, Integer, String, Enum, 
    Numeric, DateTime, ForeignKey, Index,
    UniqueConstraint, JSON
)
from sqlalchemy.orm import relationship
from sqlalchemy.ext.declarative import declarative_base

from ..common.models import (
    ExchangeType, Exchange, Market, ProductType,
    OrderType, OrderSide, OrderStatus, TimeInForce,
    PositionSide, AccountType
)

Base = declarative_base()

class Order(Base):
    """订单表"""
    __tablename__ = "orders"

    id = Column(Integer, primary_key=True, autoincrement=True)
    exchange_order_id = Column(String(64), nullable=False)
    client_order_id = Column(String(64), nullable=False)
    
    # 交易所和市场信息
    exchange_type = Column(Enum(ExchangeType), nullable=False)
    exchange = Column(Enum(Exchange), nullable=False)
    market = Column(Enum(Market), nullable=False)
    product_type = Column(Enum(ProductType), nullable=False)
    symbol = Column(String(20), nullable=False)
    
    # 账户信息
    account_id = Column(String(64), nullable=False)
    account_type = Column(Enum(AccountType), nullable=False)
    sub_account_id = Column(String(64), nullable=True)
    
    # 订单基本信息
    type = Column(Enum(OrderType), nullable=False)
    side = Column(Enum(OrderSide), nullable=False)
    position_side = Column(Enum(PositionSide), nullable=True)  # 持仓方向（期货/期权等）
    time_in_force = Column(Enum(TimeInForce), nullable=False)
    
    # 价格和数量
    price = Column(Numeric(20, 8), nullable=False)
    quantity = Column(Numeric(20, 8), nullable=False)
    executed_quantity = Column(Numeric(20, 8), nullable=False, default=0)
    remaining_quantity = Column(Numeric(20, 8), nullable=False)
    average_price = Column(Numeric(20, 8), nullable=True)
    
    # 订单状态
    status = Column(Enum(OrderStatus), nullable=False)
    is_working = Column(Boolean, nullable=False, default=True)
    
    # 时间信息
    created_at = Column(DateTime, nullable=False)
    updated_at = Column(DateTime, nullable=False)
    
    # 扩展信息
    stop_price = Column(Numeric(20, 8), nullable=True)  # 止损/止盈价格
    activation_price = Column(Numeric(20, 8), nullable=True)  # 触发价格
    callback_rate = Column(Numeric(10, 4), nullable=True)  # 回调比例
    close_position = Column(Boolean, nullable=True)  # 是否平仓单
    reduce_only = Column(Boolean, nullable=True)  # 是否只减仓
    leverage = Column(Integer, nullable=True)  # 杠杆倍数
    margin_type = Column(String(20), nullable=True)  # 保证金类型
    working_type = Column(String(20), nullable=True)  # 触发价格类型
    price_protect = Column(Boolean, nullable=True)  # 是否开启价格保护
    
    # 期权特有字段
    option_type = Column(String(10), nullable=True)  # CALL/PUT
    strike_price = Column(Numeric(20, 8), nullable=True)  # 行权价
    expiry_date = Column(DateTime, nullable=True)  # 到期日
    
    # 其他信息
    source = Column(String(50), nullable=True)  # 订单来源
    tags = Column(JSON, nullable=True)  # 标签
    remarks = Column(String(200), nullable=True)  # 备注
    
    # 费用信息
    commission = Column(Numeric(20, 8), nullable=True)  # 手续费
    commission_asset = Column(String(10), nullable=True)  # 手续费资产
    
    # 关联关系
    updates = relationship("OrderUpdate", back_populates="order")
    trades = relationship("Trade", back_populates="order")

    # 索引
    __table_args__ = (
        Index("ix_orders_exchange_order_id", "exchange_type", "exchange", "exchange_order_id", unique=True),
        Index("ix_orders_client_order_id", "exchange_type", "exchange", "client_order_id"),
        Index("ix_orders_account", "exchange_type", "exchange", "account_id", "sub_account_id"),
        Index("ix_orders_symbol", "exchange_type", "exchange", "market", "symbol"),
        Index("ix_orders_created_at", "created_at"),
        Index("ix_orders_status", "status"),
        Index("ix_orders_product_type", "product_type"),
    )

class OrderUpdate(Base):
    """订单更新记录表"""
    __tablename__ = "order_updates"

    id = Column(Integer, primary_key=True, autoincrement=True)
    order_id = Column(Integer, ForeignKey("orders.id"), nullable=False)
    update_type = Column(String(20), nullable=False)  # create, update, cancel
    prev_status = Column(Enum(OrderStatus), nullable=True)
    new_status = Column(Enum(OrderStatus), nullable=False)
    executed_quantity = Column(Numeric(20, 8), nullable=False)
    remaining_quantity = Column(Numeric(20, 8), nullable=False)
    average_price = Column(Numeric(20, 8), nullable=True)
    reason = Column(String(200), nullable=True)  # 更新原因
    created_at = Column(DateTime, nullable=False, default=datetime.utcnow)

    # 关联关系
    order = relationship("Order", back_populates="updates")

    # 索引
    __table_args__ = (
        Index("ix_order_updates_order_id", "order_id"),
        Index("ix_order_updates_created_at", "created_at"),
    )

class Trade(Base):
    """成交记录表"""
    __tablename__ = "trades"

    id = Column(Integer, primary_key=True, autoincrement=True)
    order_id = Column(Integer, ForeignKey("orders.id"), nullable=False)
    exchange_trade_id = Column(String(64), nullable=False)
    price = Column(Numeric(20, 8), nullable=False)
    quantity = Column(Numeric(20, 8), nullable=False)
    fee = Column(Numeric(20, 8), nullable=False)
    fee_asset = Column(String(10), nullable=False)
    realized_pnl = Column(Numeric(20, 8), nullable=True)  # 已实现盈亏
    position_side = Column(Enum(PositionSide), nullable=True)  # 持仓方向
    maker = Column(Boolean, nullable=False, default=False)  # 是否是挂单方
    created_at = Column(DateTime, nullable=False)

    # 关联关系
    order = relationship("Order", back_populates="trades")

    # 索引
    __table_args__ = (
        UniqueConstraint("exchange_trade_id", name="uq_trades_exchange_trade_id"),
        Index("ix_trades_order_id", "order_id"),
        Index("ix_trades_created_at", "created_at"),
    ) 