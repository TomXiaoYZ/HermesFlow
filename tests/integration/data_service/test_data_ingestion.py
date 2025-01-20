"""
数据接入服务集成测试
测试数据接入服务的数据落库和查询功能
"""
import os
import pytest
import asyncio
import psycopg2
from datetime import datetime, timezone

from src.backend.data_service.exchanges.binance.client import BinanceAPI
from src.backend.data_service.exchanges.binance.websocket import BinanceWebsocketClient
from src.backend.data_service.common.models import Market
from src.backend.data_service.db.connection import DatabaseManager

@pytest.fixture
async def setup_service():
    """设置测试环境"""
    # 创建数据库连接
    db_params = {
        "host": os.getenv("DB_HOST", "postgres"),
        "port": int(os.getenv("DB_PORT", "5432")),
        "user": os.getenv("DB_USER", "test"),
        "password": os.getenv("DB_PASSWORD", "test"),
        "database": os.getenv("DB_NAME", "test_db")
    }
    
    # 创建WebSocket客户端
    ws_client = BinanceWebsocketClient(
        api_key=os.getenv("BINANCE_API_KEY", ""),
        api_secret=os.getenv("BINANCE_API_SECRET", ""),
        testnet=True
    )
    
    # 定义市场行情数据处理器
    async def handle_market_ticker(data):
        print(f"处理市场行情数据: {data}")
        with psycopg2.connect(**db_params) as conn:
            with conn.cursor() as cur:
                cur.execute("""
                    INSERT INTO market_tickers (
                        exchange, market, symbol, price, volume, amount, timestamp
                    ) VALUES (
                        'binance', 'spot', %s, %s, %s, %s, %s
                    )
                """, (
                    data["s"],  # symbol
                    float(data["c"]),  # price
                    float(data["v"]),  # volume
                    float(data["q"]),  # amount
                    datetime.fromtimestamp(data["E"] / 1000, tz=timezone.utc)  # timestamp
                ))
                conn.commit()
    
    # 定义交易数据处理器
    async def handle_trade(data):
        print(f"处理交易数据: {data}")
        with psycopg2.connect(**db_params) as conn:
            with conn.cursor() as cur:
                cur.execute("""
                    INSERT INTO trades (
                        exchange, market, symbol, trade_id, price, quantity, amount,
                        side, timestamp
                    ) VALUES (
                        'binance', 'spot', %s, %s, %s, %s, %s, %s, %s
                    )
                """, (
                    data["s"],  # symbol
                    data["t"],  # trade_id
                    float(data["p"]),  # price
                    float(data["q"]),  # quantity
                    float(data["p"]) * float(data["q"]),  # amount
                    data["m"],  # side (true=sell, false=buy)
                    datetime.fromtimestamp(data["E"] / 1000, tz=timezone.utc)  # timestamp
                ))
                conn.commit()
    
    # 添加消息处理器
    ws_client.handlers["market_ticker"] = [handle_market_ticker]
    ws_client.handlers["trade"] = [handle_trade]
    
    # 启动WebSocket客户端
    await ws_client.start()
    
    # 订阅数据流
    symbol = "BTCUSDT"
    await ws_client.subscribe_market_ticker(symbol)
    await ws_client.subscribe_trade(symbol)
    
    yield ws_client
    
    # 清理
    await ws_client.stop()

@pytest.fixture
def db_connection():
    """创建数据库连接"""
    conn = psycopg2.connect(
        host=os.getenv("DB_HOST", "postgres"),
        port=int(os.getenv("DB_PORT", "5432")),
        user=os.getenv("DB_USER", "test"),
        password=os.getenv("DB_PASSWORD", "test"),
        database=os.getenv("DB_NAME", "test_db")
    )
    yield conn
    conn.close()

@pytest.mark.asyncio
async def test_market_data_ingestion(setup_service, db_connection):
    """测试市场数据接入"""
    symbol = "BTCUSDT"
    test_duration = 60  # 测试1分钟
    
    # 等待数据接入
    print(f"\n等待{test_duration}秒接收数据...")
    await asyncio.sleep(test_duration)
    
    # 验证行情数据
    with db_connection.cursor() as cur:
        # 检查最近的行情数据
        cur.execute("""
            SELECT COUNT(*), 
                   MAX(timestamp) as latest_time,
                   MIN(timestamp) as earliest_time
            FROM market_tickers
            WHERE symbol = %s
            AND exchange = 'binance'
            AND timestamp >= NOW() - INTERVAL '2 minutes'
        """, (symbol,))
        
        ticker_stats = cur.fetchone()
        ticker_count, latest_time, earliest_time = ticker_stats
        
        print(f"\n行情数据统计:")
        print(f"数据点数量: {ticker_count}")
        print(f"最新数据时间: {latest_time}")
        print(f"最早数据时间: {earliest_time}")
        
        assert ticker_count > 0, "应该有行情数据入库"
        if latest_time and earliest_time:
            assert (latest_time - earliest_time).total_seconds() > 30, "数据应该持续接收"

@pytest.mark.asyncio
async def test_trade_data_ingestion(setup_service, db_connection):
    """测试交易数据接入"""
    symbol = "BTCUSDT"
    
    # 等待数据接入
    await asyncio.sleep(30)
    
    # 验证交易数据
    with db_connection.cursor() as cur:
        cur.execute("""
            SELECT COUNT(*), 
                   MAX(timestamp) as latest_time,
                   MIN(timestamp) as earliest_time
            FROM trades
            WHERE symbol = %s
            AND exchange = 'binance'
            AND timestamp >= NOW() - INTERVAL '1 minute'
        """, (symbol,))
        
        trade_stats = cur.fetchone()
        trade_count, latest_time, earliest_time = trade_stats
        
        print(f"\n交易数据统计:")
        print(f"交易笔数: {trade_count}")
        print(f"最新交易时间: {latest_time}")
        print(f"最早交易时间: {earliest_time}")
        
        assert trade_count > 0, "应该有交易数据入库"

@pytest.mark.asyncio
async def test_data_consistency(setup_service, db_connection):
    """测试数据一致性"""
    symbol = "BTCUSDT"
    
    with db_connection.cursor() as cur:
        # 检查价格数据一致性
        cur.execute("""
            WITH price_changes AS (
                SELECT 
                    timestamp,
                    price,
                    LAG(price) OVER (ORDER BY timestamp) as prev_price
                FROM market_tickers
                WHERE symbol = %s
                AND exchange = 'binance'
                AND timestamp >= NOW() - INTERVAL '2 minutes'
                ORDER BY timestamp
            )
            SELECT 
                COUNT(*) as total_changes,
                MAX(ABS(price - prev_price)) as max_change,
                AVG(ABS(price - prev_price)) as avg_change
            FROM price_changes
            WHERE prev_price IS NOT NULL
        """, (symbol,))
        
        stats = cur.fetchone()
        total_changes, max_change, avg_change = stats
        
        print(f"\n价格变化统计:")
        print(f"总变化次数: {total_changes}")
        if max_change is not None:
            print(f"最大变化: {max_change:.2f}")
        if avg_change is not None:
            print(f"平均变化: {avg_change:.2f}")
        
        if max_change is not None:
            assert max_change < 1000, "价格变化不应过大"
        if avg_change is not None:
            assert avg_change < 100, "平均价格变化应在合理范围内"
        
        # 检查数据完整性
        cur.execute("""
            SELECT 
                COUNT(*) as total_records,
                COUNT(DISTINCT timestamp) as unique_timestamps
            FROM market_tickers
            WHERE symbol = %s
            AND exchange = 'binance'
            AND timestamp >= NOW() - INTERVAL '2 minutes'
        """, (symbol,))
        
        total_records, unique_timestamps = cur.fetchone()
        
        print(f"\n数据完整性统计:")
        print(f"总记录数: {total_records}")
        print(f"唯一时间戳数: {unique_timestamps}")
        
        assert total_records == unique_timestamps, "不应该有重复数据"

@pytest.mark.asyncio
async def test_data_latency(setup_service, db_connection):
    """测试数据延迟"""
    symbol = "BTCUSDT"
    test_duration = 30  # 测试30秒
    
    # 等待数据接入
    await asyncio.sleep(test_duration)
    
    # 检查数据延迟
    with db_connection.cursor() as cur:
        cur.execute("""
            SELECT 
                AVG(EXTRACT(EPOCH FROM (created_at - timestamp))) as avg_latency,
                MAX(EXTRACT(EPOCH FROM (created_at - timestamp))) as max_latency
            FROM market_tickers
            WHERE symbol = %s
            AND exchange = 'binance'
            AND timestamp >= NOW() - INTERVAL '30 seconds'
        """, (symbol,))
        
        latency_stats = cur.fetchone()
        avg_latency, max_latency = latency_stats
        
        print(f"\n数据延迟统计:")
        if avg_latency is not None:
            print(f"平均延迟: {avg_latency:.3f}秒")
        if max_latency is not None:
            print(f"最大延迟: {max_latency:.3f}秒")
        
        if avg_latency is not None:
            assert avg_latency < 1.0, "平均延迟应小于1秒"
        if max_latency is not None:
            assert max_latency < 2.0, "最大延迟应小于2秒" 