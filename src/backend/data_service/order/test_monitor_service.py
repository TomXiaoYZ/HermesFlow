"""
监控服务测试
"""
import uuid
import pytest
import asyncio
from datetime import datetime, timedelta
from typing import List
import logging
import pytest_asyncio
from unittest.mock import AsyncMock, MagicMock, patch

from data_service.order.service import OrderService
from data_service.order.sync_service import OrderSyncService
from data_service.order.monitor_service import OrderMonitorService
from data_service.order.models import OrderRecord, OrderUpdate, TradeRecord
from data_service.order.enums import OrderType, OrderSide, OrderStatus, TimeInForce

@pytest_asyncio.fixture
async def order_service():
    """创建订单服务实例"""
    # 创建 Redis mock
    mock_redis_instance = AsyncMock()
    mock_redis_instance.get.return_value = "1234567890"
    mock_redis_instance.close = AsyncMock()
    mock_redis = AsyncMock(return_value=mock_redis_instance)
    
    # 创建 PostgreSQL mock
    mock_conn = AsyncMock()
    mock_conn.fetchrow = AsyncMock(return_value=None)
    mock_conn.execute = AsyncMock()

    class AsyncContextManager:
        def __init__(self, conn):
            self.conn = conn

        async def __aenter__(self):
            return self.conn

        async def __aexit__(self, exc_type, exc_val, exc_tb):
            pass

    mock_pool = AsyncMock()
    mock_pool.acquire = AsyncMock(return_value=AsyncContextManager(mock_conn))
    mock_pool.close = AsyncMock()
    
    # 创建 ClickHouse mock
    mock_ch_instance = AsyncMock()
    mock_ch_instance.disconnect = AsyncMock()
    
    # 创建一个异步迭代器类
    class AsyncIterator:
        def __init__(self, data):
            self.data = data
            self.index = 0
            
        def __aiter__(self):
            return self
            
        async def __anext__(self):
            if self.index >= len(self.data):
                raise StopAsyncIteration
            result = self.data[self.index]
            self.index += 1
            return result
    
    # 创建一个完整的指标数据行
    mock_metrics_row = (
        90.0,  # cpu_percent
        5000000000,  # memory_rss
        10000000000,  # memory_vms
        10,  # thread_count
        True,  # pg_connected
        True,  # redis_connected
        True,  # ch_connected
        400000,  # sync_delay
        1234567890000,  # last_sync_time
        100,  # order_count_1m
        50,  # trade_count_1m
        10,  # error_count_1m
        2000.0  # avg_process_time_1m
    )

    # 设置 execute_iter 返回异步迭代器
    mock_ch_instance.execute_iter = AsyncMock(return_value=AsyncIterator([mock_metrics_row]))
    mock_ch = AsyncMock(return_value=mock_ch_instance)
    
    # 创建 Future 对象
    redis_future = asyncio.Future()
    redis_future.set_result(mock_redis_instance)
    mock_redis.return_value = redis_future

    pg_future = asyncio.Future()
    pg_future.set_result(mock_pool)
    mock_pool_create = AsyncMock(return_value=pg_future)

    ch_future = asyncio.Future()
    ch_future.set_result(mock_ch_instance)
    mock_ch.return_value = ch_future

    with patch('redis.Redis.from_url', mock_redis), \
         patch('asyncpg.create_pool', mock_pool_create), \
         patch('clickhouse_driver.Client.from_url', mock_ch):
        
        service = OrderService(
            redis_url="redis://localhost",
            pg_url="postgresql://localhost/test",
            ch_url="clickhouse://localhost"
        )
        await service.start()
        yield service
        await service.stop()

@pytest_asyncio.fixture
async def sync_service(order_service):
    """同步服务"""
    service = OrderSyncService(order_service)
    await service.start()
    yield service
    await service.stop()

@pytest_asyncio.fixture
async def monitor_service(order_service, sync_service):
    """监控服务"""
    service = OrderMonitorService(order_service, sync_service)
    await service.start()
    yield service
    await service.stop()

@pytest.fixture
def caplog(caplog):
    """日志捕获fixture"""
    caplog.set_level(logging.INFO)
    return caplog

@pytest.mark.asyncio
async def test_collect_metrics(monitor_service):
    """测试指标收集"""
    # 等待指标收集
    await monitor_service._collect_metrics()
    
    # 验证指标
    assert "timestamp" in monitor_service._metrics
    assert "system" in monitor_service._metrics
    assert "connections" in monitor_service._metrics
    assert "sync" in monitor_service._metrics
    assert "orders" in monitor_service._metrics
    
    # 验证系统指标
    assert monitor_service._metrics["system"]["cpu_percent"] >= 0
    assert monitor_service._metrics["system"]["memory_rss"] > 0
    assert monitor_service._metrics["system"]["memory_vms"] > 0
    assert monitor_service._metrics["system"]["thread_count"] > 0
    
    # 验证连接状态
    assert isinstance(monitor_service._metrics["connections"]["postgres"], bool)
    assert isinstance(monitor_service._metrics["connections"]["redis"], bool)
    assert isinstance(monitor_service._metrics["connections"]["clickhouse"], bool)
    
    # 验证同步状态
    assert monitor_service._metrics["sync"]["delay"] >= 0
    assert monitor_service._metrics["sync"]["last_sync_time"] > 0
    
    # 验证订单指标
    assert monitor_service._metrics["orders"]["count_1m"] >= 0
    assert monitor_service._metrics["orders"]["trade_count_1m"] >= 0
    assert monitor_service._metrics["orders"]["error_count_1m"] >= 0
    assert monitor_service._metrics["orders"]["avg_process_time_1m"] >= 0

@pytest.mark.asyncio
async def test_save_metrics(monitor_service, order_service):
    """测试指标保存"""
    # 等待指标收集
    await monitor_service._collect_metrics()
    
    # 保存指标
    await monitor_service._save_metrics()
    
    # 验证保存结果
    query = """
        SELECT
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
        FROM system_metrics
        WHERE timestamp >= toUnixTimestamp(now() - INTERVAL 2 MINUTE) * 1000
        ORDER BY timestamp DESC
        LIMIT 1
    """
    
    result = await order_service._ch_client.execute_iter(query)
    async for row in result:
        assert row[0] >= 0  # cpu_percent
        assert row[1] > 0  # memory_rss
        assert row[2] > 0  # memory_vms
        assert row[3] > 0  # thread_count
        assert row[4] in (0, 1)  # pg_connected
        assert row[5] in (0, 1)  # redis_connected
        assert row[6] in (0, 1)  # ch_connected
        assert row[7] >= 0  # sync_delay
        assert row[8] > 0  # last_sync_time
        assert row[9] >= 0  # order_count_1m
        assert row[10] >= 0  # trade_count_1m
        assert row[11] >= 0  # error_count_1m
        assert row[12] >= 0  # avg_process_time_1m

@pytest.mark.asyncio
async def test_alerts(monitor_service, order_service, caplog):
    """测试告警功能"""
    # 创建测试订单
    for i in range(10):
        order = OrderRecord(
            id=str(uuid.uuid4()),
            exchange="binance",
            client_order_id=f"test123{i}",
            symbol="BTCUSDT",
            order_type=OrderType.LIMIT,
            side=OrderSide.BUY,
            price=50000.0,
            quantity=1.0,
            status=OrderStatus.NEW,
            create_time=datetime.now(),
            executed_qty=0.0,
            avg_price=0.0,
            time_in_force=TimeInForce.GTC,
            update_time=datetime.now(),
            is_contract=False,
            position_side=None,
            margin_type=None,
            leverage=None,
            stop_price=None,
            working_type=None,
            reduce_only=False
        )
        await order_service.create_order(order)
    
    # 设置告警条件
    monitor_service._metrics = {
        "system": {
            "cpu_percent": 90,
            "memory_rss": 5 * 1024 * 1024 * 1024,  # 5GB
            "memory_vms": 10 * 1024 * 1024 * 1024,  # 10GB
            "thread_count": 100
        },
        "connections": {
            "postgres": False,
            "redis": False,
            "clickhouse": False
        },
        "sync": {
            "delay": 400,
            "last_sync_time": datetime.now().timestamp()
        },
        "orders": {
            "count_1m": 100,
            "trade_count_1m": 50,
            "error_count_1m": 10,
            "avg_process_time_1m": 2000
        }
    }
    
    # 触发告警检查
    await monitor_service._check_alerts()
    
    # 验证告警日志
    messages = [record.message for record in caplog.records]
    assert any("CPU使用率过高" in msg for msg in messages)
    assert any("内存使用过高" in msg for msg in messages)
    assert any("PostgreSQL连接断开" in msg for msg in messages)
    assert any("Redis连接断开" in msg for msg in messages)
    assert any("ClickHouse连接断开" in msg for msg in messages)
    assert any("数据同步延迟过高" in msg for msg in messages)
    assert any("订单处理错误率过高" in msg for msg in messages)
    assert any("订单处理延迟过高" in msg for msg in messages)

@pytest.mark.asyncio
async def test_connection_check(monitor_service):
    """测试连接检查功能"""
    # 检查PostgreSQL连接
    pg_connected = await monitor_service._check_postgres_connection()
    assert isinstance(pg_connected, bool)
    
    # 检查Redis连接
    redis_connected = await monitor_service._check_redis_connection()
    assert isinstance(redis_connected, bool)
    
    # 检查ClickHouse连接
    ch_connected = await monitor_service._check_clickhouse_connection()
    assert isinstance(ch_connected, bool)

@pytest.mark.asyncio
async def test_sync_delay_calculation(monitor_service):
    """测试同步延迟计算"""
    # 获取同步延迟
    sync_delay = await monitor_service._get_sync_delay()
    assert sync_delay >= 0
    assert isinstance(sync_delay, float)

@pytest.mark.asyncio
async def test_order_metrics_calculation(monitor_service):
    """测试订单指标计算"""
    # 获取订单指标
    order_metrics = await monitor_service._get_order_metrics()
    assert isinstance(order_metrics, dict)
    assert "count_1m" in order_metrics
    assert "trade_count_1m" in order_metrics
    assert "error_count_1m" in order_metrics
    assert "avg_process_time_1m" in order_metrics

@pytest.mark.asyncio
async def test_monitor_service_lifecycle(order_service, sync_service):
    """测试监控服务生命周期"""
    # 创建监控服务
    monitor = OrderMonitorService(order_service, sync_service)
    
    # 启动服务
    await monitor.start()
    assert monitor.running
    assert monitor._monitor_task is not None
    
    # 停止服务
    await monitor.stop()
    assert not monitor.running
    assert monitor._monitor_task is None

@pytest.mark.asyncio
async def test_metrics_persistence(monitor_service, order_service):
    """测试指标持久化"""
    # 收集和保存指标
    await monitor_service._collect_metrics()
    await monitor_service._save_metrics()
    
    # 检查ClickHouse中的指标数据
    query = """
        SELECT count(*)
        FROM system_metrics
        WHERE timestamp >= toUnixTimestamp(now() - INTERVAL 2 MINUTE) * 1000
    """
    
    result = await order_service._ch_client.execute_iter(query)
    async for row in result:
        assert row[0] > 0

@pytest.mark.asyncio
async def test_alert_thresholds(monitor_service, caplog):
    """测试告警阈值"""
    caplog.set_level(logging.WARNING)
    
    # 设置告警条件
    monitor_service._metrics = {
        "system": {
            "cpu_percent": 90,
            "memory_rss": 5 * 1024 * 1024 * 1024,  # 5GB
            "memory_vms": 10 * 1024 * 1024 * 1024,  # 10GB
            "thread_count": 100
        },
        "connections": {
            "postgres": False,
            "redis": False,
            "clickhouse": False
        },
        "sync": {
            "delay": 400,
            "last_sync_time": datetime.now().timestamp()
        },
        "orders": {
            "count_1m": 100,
            "trade_count_1m": 50,
            "error_count_1m": 10,
            "avg_process_time_1m": 2000
        }
    }
    
    # 触发告警检查
    await monitor_service._check_alerts()
    
    # 验证告警日志
    messages = [record.message for record in caplog.records]
    assert any("CPU使用率过高" in msg for msg in messages)
    assert any("内存使用过高" in msg for msg in messages)
    assert any("PostgreSQL连接断开" in msg for msg in messages)
    assert any("Redis连接断开" in msg for msg in messages)
    assert any("ClickHouse连接断开" in msg for msg in messages)
    assert any("数据同步延迟过高" in msg for msg in messages)
    assert any("订单处理错误率过高" in msg for msg in messages)
    assert any("订单处理延迟过高" in msg for msg in messages) 