"""
同步服务测试
"""
import uuid
import pytest
import asyncio
from datetime import datetime, timedelta
from typing import List

from .service import OrderService
from .sync_service import OrderSyncService
from .models import OrderRecord, OrderUpdate, TradeRecord
from .enums import OrderType, OrderSide, OrderStatus, TimeInForce

@pytest.fixture
async def order_service():
    """创建订单服务实例"""
    service = OrderService(
        redis_url="redis://localhost:6379/0",
        postgres_dsn="postgresql://user:pass@localhost:5432/test",
        clickhouse_settings={
            "host": "localhost",
            "port": 9000,
            "database": "test",
            "user": "default",
            "password": ""
        }
    )
    await service.init()
    yield service
    await service.close()

@pytest.fixture
async def sync_service(order_service):
    """创建同步服务实例"""
    service = OrderSyncService(order_service)
    await service.start()
    yield service
    await service.stop()

@pytest.mark.asyncio
async def test_sync_orders(sync_service, order_service):
    """测试订单同步"""
    # 创建测试订单
    orders = []
    for i in range(5):
        order = OrderRecord(
            id=str(uuid.uuid4()),
            exchange="binance",
            exchange_order_id=f"123456{i}",
            client_order_id=f"test123{i}",
            symbol="BTCUSDT",
            type=OrderType.LIMIT,
            side=OrderSide.BUY,
            price=50000.0,
            quantity=1.0,
            executed_qty=1.0 if i < 3 else 0.0,  # 3个订单已成交
            avg_price=50000.0 if i < 3 else 0.0,
            status=OrderStatus.FILLED if i < 3 else OrderStatus.NEW,
            time_in_force=TimeInForce.GTC,
            created_time=int(datetime.now().timestamp() * 1000),
            updated_time=int(datetime.now().timestamp() * 1000),
            is_contract=False,
            position_side=None,
            margin_type=None,
            leverage=None,
            stop_price=None,
            working_type=None,
            reduce_only=False
        )
        await order_service.create_order(order)
        orders.append(order)
    
    # 等待同步完成
    await asyncio.sleep(65)  # 等待一次同步周期
    
    # 验证同步结果
    query = """
        SELECT
            count(*) as total_orders,
            sum(case when status = 'FILLED' then 1 else 0 end) as filled_orders,
            sum(executed_qty) as total_volume,
            sum(executed_qty * avg_price) as total_value
        FROM orders
        WHERE exchange = 'binance'
            AND symbol = 'BTCUSDT'
            AND created_time >= toUnixTimestamp(now() - INTERVAL 1 HOUR) * 1000
    """
    
    result = await order_service.ch.execute_iter(query)
    async for row in result:
        assert row[0] == 5  # 总订单数
        assert row[1] == 3  # 已成交订单数
        assert float(row[2]) == 3.0  # 总成交量
        assert float(row[3]) == 150000.0  # 总成交额

@pytest.mark.asyncio
async def test_sync_trades(sync_service, order_service):
    """测试成交记录同步"""
    # 创建测试订单
    order = OrderRecord(
        id=str(uuid.uuid4()),
        exchange="binance",
        exchange_order_id="123456",
        client_order_id="test123",
        symbol="BTCUSDT",
        type=OrderType.LIMIT,
        side=OrderSide.BUY,
        price=50000.0,
        quantity=1.0,
        executed_qty=1.0,
        avg_price=50000.0,
        status=OrderStatus.FILLED,
        time_in_force=TimeInForce.GTC,
        created_time=int(datetime.now().timestamp() * 1000),
        updated_time=int(datetime.now().timestamp() * 1000),
        is_contract=False,
        position_side=None,
        margin_type=None,
        leverage=None,
        stop_price=None,
        working_type=None,
        reduce_only=False
    )
    await order_service.create_order(order)
    
    # 添加成交记录
    trades = []
    for i in range(2):
        trade = TradeRecord(
            id=str(uuid.uuid4()),
            order_id=order.id,
            exchange="binance",
            exchange_trade_id=f"t123456{i}",
            symbol="BTCUSDT",
            price=50000.0,
            quantity=0.5,
            commission=0.1,
            commission_asset="USDT",
            created_time=int(datetime.now().timestamp() * 1000),
            is_buyer=True,
            is_maker=False,
            is_contract=False,
            position_side=None,
            realized_pnl=None
        )
        await order_service.add_trade(trade)
        trades.append(trade)
    
    # 等待同步完成
    await asyncio.sleep(65)  # 等待一次同步周期
    
    # 验证同步结果
    query = """
        SELECT
            count(*) as trade_count,
            sum(quantity) as total_volume,
            sum(quantity * price) as total_value,
            sum(commission) as total_commission
        FROM trades
        WHERE exchange = 'binance'
            AND symbol = 'BTCUSDT'
            AND created_time >= toUnixTimestamp(now() - INTERVAL 1 HOUR) * 1000
    """
    
    result = await order_service.ch.execute_iter(query)
    async for row in result:
        assert row[0] == 2  # 成交笔数
        assert float(row[1]) == 1.0  # 总成交量
        assert float(row[2]) == 50000.0  # 总成交额
        assert float(row[3]) == 0.2  # 总手续费

@pytest.mark.asyncio
async def test_calculate_summary(sync_service, order_service):
    """测试订单汇总计算"""
    # 创建测试订单和成交记录
    order = OrderRecord(
        id=str(uuid.uuid4()),
        exchange="binance",
        exchange_order_id="123456",
        client_order_id="test123",
        symbol="BTCUSDT",
        type=OrderType.LIMIT,
        side=OrderSide.BUY,
        price=50000.0,
        quantity=1.0,
        executed_qty=1.0,
        avg_price=50000.0,
        status=OrderStatus.FILLED,
        time_in_force=TimeInForce.GTC,
        created_time=int(datetime.now().timestamp() * 1000),
        updated_time=int(datetime.now().timestamp() * 1000),
        is_contract=False,
        position_side=None,
        margin_type=None,
        leverage=None,
        stop_price=None,
        working_type=None,
        reduce_only=False
    )
    await order_service.create_order(order)
    
    trade = TradeRecord(
        id=str(uuid.uuid4()),
        order_id=order.id,
        exchange="binance",
        exchange_trade_id="t123456",
        symbol="BTCUSDT",
        price=50000.0,
        quantity=1.0,
        commission=0.1,
        commission_asset="USDT",
        created_time=int(datetime.now().timestamp() * 1000),
        is_buyer=True,
        is_maker=False,
        is_contract=False,
        position_side=None,
        realized_pnl=None
    )
    await order_service.add_trade(trade)
    
    # 等待汇总计算完成
    await asyncio.sleep(305)  # 等待一次汇总计算周期
    
    # 验证小时汇总
    query = """
        SELECT
            total_orders,
            filled_orders,
            total_volume,
            total_value,
            total_commission,
            success_rate
        FROM order_summary
        WHERE exchange = 'binance'
            AND symbol = 'BTCUSDT'
            AND start_time >= toStartOfHour(now())
        ORDER BY start_time DESC
        LIMIT 1
    """
    
    result = await order_service.ch.execute_iter(query)
    async for row in result:
        assert row[0] == 1  # 总订单数
        assert row[1] == 1  # 已成交订单数
        assert float(row[2]) == 1.0  # 总成交量
        assert float(row[3]) == 50000.0  # 总成交额
        assert float(row[4]) == 0.1  # 总手续费
        assert float(row[5]) == 100.0  # 成功率 