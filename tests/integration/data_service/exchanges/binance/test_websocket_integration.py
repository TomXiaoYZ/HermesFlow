"""
Binance WebSocket客户端集成测试

测试覆盖:
1. 数据流集成测试 - WebSocket到数据库的完整流程
2. 消息队列集成测试 - 数据分发到Kafka
3. 缓存集成测试 - 实时数据缓存到Redis
4. 多组件协作测试 - 完整业务流程验证
"""
import pytest
import asyncio
import json
from datetime import datetime
from typing import Dict, Any
from src.backend.data_service.binance.websocket_client import BinanceWebsocketClient
from src.backend.data_service.storage import PostgresStorage, RedisStorage
from src.backend.data_service.queue import KafkaProducer

@pytest.fixture
async def postgres_storage():
    """PostgreSQL存储fixture"""
    storage = PostgresStorage(
        host="postgres",
        port=5432,
        user="test",
        password="test",
        database="test_db"
    )
    await storage.connect()
    yield storage
    await storage.close()

@pytest.fixture
async def redis_storage():
    """Redis存储fixture"""
    storage = RedisStorage(
        host="redis",
        port=6379
    )
    await storage.connect()
    yield storage
    await storage.close()

@pytest.fixture
async def kafka_producer():
    """Kafka生产者fixture"""
    producer = KafkaProducer(
        bootstrap_servers="kafka:29092",
        topic_prefix="binance"
    )
    await producer.start()
    yield producer
    await producer.stop()

@pytest.fixture
async def ws_client(postgres_storage, redis_storage, kafka_producer):
    """WebSocket客户端fixture"""
    client = BinanceWebsocketClient()
    client.set_storage(postgres_storage)
    client.set_cache(redis_storage)
    client.set_queue(kafka_producer)
    await client.connect()
    yield client
    await client.close()

@pytest.mark.integration
async def test_market_data_flow(ws_client, postgres_storage, redis_storage, kafka_producer):
    """测试市场数据完整流程"""
    symbol = "btcusdt"
    data_received = asyncio.Event()
    
    async def verify_data_flow(msg: Dict[str, Any]):
        # 验证数据已写入PostgreSQL
        stored_data = await postgres_storage.get_latest_trade(symbol)
        assert stored_data is not None
        assert stored_data["symbol"] == symbol
        
        # 验证数据已缓存到Redis
        cached_data = await redis_storage.get_latest_price(symbol)
        assert cached_data is not None
        assert float(cached_data) > 0
        
        # 标记测试完成
        data_received.set()
    
    # 订阅交易数据
    await ws_client.subscribe(f"{symbol}@trade", callback=verify_data_flow)
    
    # 等待数据处理完成
    try:
        await asyncio.wait_for(data_received.wait(), timeout=30)
    except asyncio.TimeoutError:
        pytest.fail("未在超时时间内收到数据")

@pytest.mark.integration
async def test_order_book_sync(ws_client, redis_storage):
    """测试订单簿同步"""
    symbol = "btcusdt"
    sync_complete = asyncio.Event()
    
    async def verify_order_book(msg: Dict[str, Any]):
        # 验证订单簿数据已正确缓存
        order_book = await redis_storage.get_order_book(symbol)
        assert order_book is not None
        assert len(order_book["bids"]) > 0
        assert len(order_book["asks"]) > 0
        
        # 验证买卖盘价格正确性
        assert float(order_book["bids"][0][0]) < float(order_book["asks"][0][0])
        
        sync_complete.set()
    
    # 订阅深度数据
    await ws_client.subscribe(f"{symbol}@depth", callback=verify_order_book)
    
    # 等待同步完成
    try:
        await asyncio.wait_for(sync_complete.wait(), timeout=30)
    except asyncio.TimeoutError:
        pytest.fail("订单簿同步超时")

@pytest.mark.integration
async def test_kline_storage(ws_client, postgres_storage, kafka_producer):
    """测试K线数据存储和分发"""
    symbol = "btcusdt"
    interval = "1m"
    storage_complete = asyncio.Event()
    
    async def verify_kline_storage(msg: Dict[str, Any]):
        # 验证K线数据已存储到PostgreSQL
        kline = await postgres_storage.get_latest_kline(symbol, interval)
        assert kline is not None
        assert kline["symbol"] == symbol
        assert kline["interval"] == interval
        
        # 验证数据已发送到Kafka
        # 这里需要实现一个Kafka消费者来验证
        
        storage_complete.set()
    
    # 订阅K线数据
    await ws_client.subscribe(f"{symbol}@kline_{interval}", callback=verify_kline_storage)
    
    # 等待存储完成
    try:
        await asyncio.wait_for(storage_complete.wait(), timeout=30)
    except asyncio.TimeoutError:
        pytest.fail("K线数据存储超时")

@pytest.mark.integration
async def test_multi_symbol_processing(ws_client, redis_storage):
    """测试多交易对数据处理"""
    symbols = ["btcusdt", "ethusdt", "bnbusdt"]
    symbols_completed = {symbol: asyncio.Event() for symbol in symbols}
    
    async def verify_symbol_data(symbol: str, msg: Dict[str, Any]):
        # 验证数据已缓存到Redis
        price = await redis_storage.get_latest_price(symbol)
        assert price is not None
        assert float(price) > 0
        
        symbols_completed[symbol].set()
    
    # 订阅多个交易对
    for symbol in symbols:
        await ws_client.subscribe(
            f"{symbol}@trade",
            callback=lambda msg, s=symbol: verify_symbol_data(s, msg)
        )
    
    # 等待所有交易对数据处理完成
    try:
        await asyncio.wait_for(
            asyncio.gather(*(event.wait() for event in symbols_completed.values())),
            timeout=30
        )
    except asyncio.TimeoutError:
        pytest.fail("多交易对数据处理超时")

@pytest.mark.integration
async def test_error_recovery(ws_client, redis_storage):
    """测试错误恢复机制"""
    symbol = "btcusdt"
    recovery_complete = asyncio.Event()
    
    async def verify_recovery(msg: Dict[str, Any]):
        # 验证恢复后数据正常
        price = await redis_storage.get_latest_price(symbol)
        assert price is not None
        assert float(price) > 0
        
        recovery_complete.set()
    
    # 订阅数据
    await ws_client.subscribe(f"{symbol}@trade", callback=verify_recovery)
    
    # 模拟连接断开
    await ws_client.close()
    
    # 重新连接
    await ws_client.connect()
    
    # 验证是否成功恢复
    try:
        await asyncio.wait_for(recovery_complete.wait(), timeout=30)
    except asyncio.TimeoutError:
        pytest.fail("错误恢复超时")

@pytest.mark.integration
async def test_data_consistency(ws_client, postgres_storage, redis_storage):
    """测试数据一致性"""
    symbol = "btcusdt"
    consistency_verified = asyncio.Event()
    
    async def verify_consistency(msg: Dict[str, Any]):
        # 获取PostgreSQL和Redis中的数据
        db_price = await postgres_storage.get_latest_price(symbol)
        cache_price = await redis_storage.get_latest_price(symbol)
        
        # 验证数据一致性
        assert abs(float(db_price) - float(cache_price)) < 0.0001
        
        consistency_verified.set()
    
    # 订阅交易数据
    await ws_client.subscribe(f"{symbol}@trade", callback=verify_consistency)
    
    # 等待验证完成
    try:
        await asyncio.wait_for(consistency_verified.wait(), timeout=30)
    except asyncio.TimeoutError:
        pytest.fail("数据一致性验证超时") 