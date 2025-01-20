"""
Binance WebSocket性能测试
测试WebSocket的消息吞吐量、延迟和内存使用情况
"""
import os
import pytest
import asyncio
import time
import psutil
import statistics
from datetime import datetime
from collections import deque

from src.backend.data_service.exchanges.binance.websocket import BinanceWebsocketClient
from src.backend.data_service.common.models import Market

@pytest.fixture
async def ws_client():
    """创建WebSocket测试客户端"""
    api_key = os.getenv("BINANCE_API_KEY", "")
    api_secret = os.getenv("BINANCE_API_SECRET", "")
    client = BinanceWebsocketClient(api_key, api_secret, testnet=True)
    await client.start()
    yield client
    await client.stop()

@pytest.mark.asyncio
async def test_message_throughput(ws_client):
    """测试消息吞吐量"""
    symbols = ["BTCUSDT", "ETHUSDT", "BNBUSDT", "ADAUSDT", "DOGEUSDT"]
    message_counts = {symbol: 0 for symbol in symbols}
    start_time = time.time()
    test_duration = 60  # 测试持续60秒
    
    def on_ticker(symbol, data):
        message_counts[symbol] += 1
    
    # 订阅多个交易对
    for symbol in symbols:
        await ws_client.subscribe_ticker(
            Market.SPOT, 
            symbol, 
            lambda data, s=symbol: on_ticker(s, data)
        )
    
    # 监控消息接收
    while time.time() - start_time < test_duration:
        await asyncio.sleep(1)
        current_time = time.time() - start_time
        total_messages = sum(message_counts.values())
        messages_per_second = total_messages / current_time
        
        # 每10秒输出一次统计
        if int(current_time) % 10 == 0:
            print(f"\n{int(current_time)}秒统计:")
            print(f"总消息数: {total_messages}")
            print(f"每秒消息数: {messages_per_second:.2f}")
            for symbol, count in message_counts.items():
                print(f"{symbol}: {count}消息")
    
    # 计算最终统计
    final_total = sum(message_counts.values())
    final_mps = final_total / test_duration
    
    print(f"\n最终统计:")
    print(f"总消息数: {final_total}")
    print(f"平均每秒消息数: {final_mps:.2f}")
    for symbol, count in message_counts.items():
        print(f"{symbol}: {count}消息, {count/test_duration:.2f}/秒")
    
    assert final_mps > 1.0  # 每秒至少处理1条消息
    assert all(count > 0 for count in message_counts.values())

@pytest.mark.asyncio
async def test_message_latency(ws_client):
    """测试消息延迟"""
    symbol = "BTCUSDT"
    latencies = deque(maxlen=1000)  # 最多记录1000个延迟样本
    
    def on_ticker(data):
        receive_time = time.time()
        # 计算从交易所发出消息到接收到消息的延迟
        event_time = data.event_time / 1000  # 转换为秒
        latency = receive_time - event_time
        latencies.append(latency)
    
    await ws_client.subscribe_ticker(Market.SPOT, symbol, on_ticker)
    await asyncio.sleep(30)  # 收集30秒的数据
    
    # 计算延迟统计
    latency_list = list(latencies)
    avg_latency = statistics.mean(latency_list)
    max_latency = max(latency_list)
    min_latency = min(latency_list)
    p95_latency = statistics.quantiles(latency_list, n=20)[18]
    
    print(f"\n消息延迟统计:")
    print(f"样本数: {len(latency_list)}")
    print(f"平均延迟: {avg_latency*1000:.2f}ms")
    print(f"最大延迟: {max_latency*1000:.2f}ms")
    print(f"最小延迟: {min_latency*1000:.2f}ms")
    print(f"95%延迟: {p95_latency*1000:.2f}ms")
    
    assert avg_latency < 1.0  # 平均延迟应小于1秒
    assert p95_latency < 2.0  # 95%的消息延迟应小于2秒

@pytest.mark.asyncio
async def test_memory_usage(ws_client):
    """测试内存使用情况"""
    symbols = ["BTCUSDT", "ETHUSDT", "BNBUSDT", "ADAUSDT", "DOGEUSDT"]
    memory_samples = []
    start_time = time.time()
    test_duration = 60  # 测试持续60秒
    
    # 获取初始内存使用
    process = psutil.Process()
    initial_memory = process.memory_info().rss / 1024 / 1024  # MB
    memory_samples.append(initial_memory)
    
    # 订阅多个交易对
    for symbol in symbols:
        await ws_client.subscribe_ticker(Market.SPOT, symbol, lambda x: None)
    
    # 监控内存使用
    while time.time() - start_time < test_duration:
        await asyncio.sleep(5)  # 每5秒采样一次
        memory_used = process.memory_info().rss / 1024 / 1024
        memory_samples.append(memory_used)
        
        print(f"\n当前内存使用: {memory_used:.2f}MB")
        print(f"内存增长: {memory_used - initial_memory:.2f}MB")
    
    # 计算内存统计
    avg_memory = statistics.mean(memory_samples)
    max_memory = max(memory_samples)
    memory_growth = max_memory - initial_memory
    
    print(f"\n内存使用统计:")
    print(f"初始内存: {initial_memory:.2f}MB")
    print(f"平均内存: {avg_memory:.2f}MB")
    print(f"最大内存: {max_memory:.2f}MB")
    print(f"内存增长: {memory_growth:.2f}MB")
    
    assert memory_growth < 100  # 内存增长应小于100MB
    assert max_memory < 500  # 最大内存使用应小于500MB

@pytest.mark.asyncio
async def test_connection_stability(ws_client):
    """测试连接稳定性"""
    symbol = "BTCUSDT"
    disconnections = []
    message_gaps = []
    last_message_time = None
    
    def on_ticker(data):
        nonlocal last_message_time
        current_time = time.time()
        
        if last_message_time is not None:
            gap = current_time - last_message_time
            if gap > 5:  # 如果消息间隔超过5秒，可能发生了断连
                disconnections.append((last_message_time, current_time))
            message_gaps.append(gap)
        
        last_message_time = current_time
    
    await ws_client.subscribe_ticker(Market.SPOT, symbol, on_ticker)
    await asyncio.sleep(300)  # 测试5分钟
    
    # 计算统计数据
    avg_gap = statistics.mean(message_gaps)
    max_gap = max(message_gaps)
    disconnect_count = len(disconnections)
    
    print(f"\n连接稳定性统计:")
    print(f"平均消息间隔: {avg_gap:.3f}秒")
    print(f"最大消息间隔: {max_gap:.3f}秒")
    print(f"断连次数: {disconnect_count}")
    for start, end in disconnections:
        duration = end - start
        print(f"断连时间: {datetime.fromtimestamp(start)} - {datetime.fromtimestamp(end)} (持续{duration:.1f}秒)")
    
    assert disconnect_count < 3  # 5分钟内断连次数应少于3次
    assert avg_gap < 1.0  # 平均消息间隔应小于1秒 