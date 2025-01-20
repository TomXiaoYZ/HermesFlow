"""
Binance内存性能测试
测试长时间运行和内存泄漏情况
"""
import os
import pytest
import asyncio
import time
import psutil
import gc
import weakref
from datetime import datetime
import matplotlib.pyplot as plt
import numpy as np

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

def plot_memory_usage(timestamps, memory_usage, title):
    """绘制内存使用图表"""
    plt.figure(figsize=(10, 6))
    plt.plot(timestamps, memory_usage)
    plt.title(title)
    plt.xlabel('时间 (秒)')
    plt.ylabel('内存使用 (MB)')
    plt.grid(True)
    
    # 保存图表
    filename = f"memory_usage_{datetime.now().strftime('%Y%m%d_%H%M%S')}.png"
    plt.savefig(filename)
    plt.close()
    print(f"内存使用图表已保存到: {filename}")

@pytest.mark.asyncio
async def test_long_running_memory(ws_client):
    """测试长时间运行的内存使用情况"""
    symbols = ["BTCUSDT", "ETHUSDT", "BNBUSDT", "ADAUSDT", "DOGEUSDT"]
    test_duration = 3600  # 测试1小时
    sample_interval = 10  # 每10秒采样一次
    
    process = psutil.Process()
    memory_samples = []
    timestamps = []
    start_time = time.time()
    
    # 强制进行垃圾回收
    gc.collect()
    initial_memory = process.memory_info().rss / 1024 / 1024
    
    # 订阅多个交易对
    for symbol in symbols:
        await ws_client.subscribe_ticker(Market.SPOT, symbol, lambda x: None)
        await ws_client.subscribe_depth(Market.SPOT, symbol, lambda x: None)
        await ws_client.subscribe_kline(Market.SPOT, symbol, "1m", lambda x: None)
    
    print(f"初始内存使用: {initial_memory:.2f}MB")
    
    try:
        while time.time() - start_time < test_duration:
            current_time = time.time() - start_time
            memory_used = process.memory_info().rss / 1024 / 1024
            memory_samples.append(memory_used)
            timestamps.append(current_time)
            
            if int(current_time) % 60 == 0:  # 每分钟输出一次
                print(f"\n运行时间: {int(current_time)}秒")
                print(f"当前内存使用: {memory_used:.2f}MB")
                print(f"内存增长: {memory_used - initial_memory:.2f}MB")
            
            await asyncio.sleep(sample_interval)
    
    finally:
        # 绘制内存使用图表
        plot_memory_usage(timestamps, memory_samples, "长时间运行内存使用")
        
        # 计算统计数据
        memory_growth = memory_samples[-1] - initial_memory
        max_memory = max(memory_samples)
        avg_memory = sum(memory_samples) / len(memory_samples)
        
        print(f"\n长时间运行内存统计:")
        print(f"运行时间: {test_duration}秒")
        print(f"初始内存: {initial_memory:.2f}MB")
        print(f"最终内存: {memory_samples[-1]:.2f}MB")
        print(f"内存增长: {memory_growth:.2f}MB")
        print(f"最大内存: {max_memory:.2f}MB")
        print(f"平均内存: {avg_memory:.2f}MB")
        
        # 计算内存增长率
        growth_rate = memory_growth / (test_duration / 3600)  # MB/小时
        print(f"内存增长率: {growth_rate:.2f}MB/小时")
        
        assert growth_rate < 50  # 每小时内存增长应小于50MB
        assert max_memory < 1024  # 最大内存使用应小于1GB

@pytest.mark.asyncio
async def test_memory_leak_detection(ws_client):
    """测试内存泄漏检测"""
    symbol = "BTCUSDT"
    test_cycles = 10
    cycle_duration = 60  # 每个周期60秒
    
    process = psutil.Process()
    memory_samples = []
    cycle_memories = []
    
    # 用于检测对象引用
    callback_refs = []
    
    def create_callback():
        """创建带有本地状态的回调函数"""
        local_data = {"count": 0}
        def callback(data):
            local_data["count"] += 1
        return callback, weakref.ref(callback)
    
    print("\n开始内存泄漏测试...")
    
    for cycle in range(test_cycles):
        print(f"\n周期 {cycle + 1}/{test_cycles}")
        
        # 强制垃圾回收
        gc.collect()
        start_memory = process.memory_info().rss / 1024 / 1024
        
        # 创建新的回调并保持弱引用
        callback, ref = create_callback()
        callback_refs.append(ref)
        
        # 订阅和取消订阅
        await ws_client.subscribe_ticker(Market.SPOT, symbol, callback)
        
        # 监控一段时间的内存使用
        cycle_start = time.time()
        while time.time() - cycle_start < cycle_duration:
            memory_used = process.memory_info().rss / 1024 / 1024
            memory_samples.append(memory_used)
            await asyncio.sleep(1)
        
        # 取消订阅
        await ws_client.unsubscribe_ticker(Market.SPOT, symbol)
        
        # 强制垃圾回收
        gc.collect()
        end_memory = process.memory_info().rss / 1024 / 1024
        cycle_memories.append(end_memory)
        
        # 检查回调是否被正确清理
        live_callbacks = sum(1 for ref in callback_refs if ref() is not None)
        print(f"活跃回调数量: {live_callbacks}/{len(callback_refs)}")
        
        print(f"周期内存变化: {end_memory - start_memory:.2f}MB")
    
    # 分析内存使用趋势
    memory_trend = np.polyfit(range(len(cycle_memories)), cycle_memories, 1)
    trend_slope = memory_trend[0]
    
    print(f"\n内存泄漏测试结果:")
    print(f"内存趋势斜率: {trend_slope:.2f}MB/周期")
    print(f"活跃回调数量: {live_callbacks}")
    
    # 绘制内存使用图表
    plot_memory_usage(range(len(memory_samples)), memory_samples, "内存泄漏测试")
    
    # 验证测试结果
    assert trend_slope < 1.0  # 内存增长趋势应该很小
    assert live_callbacks == 0  # 所有回调都应该被清理

@pytest.mark.asyncio
async def test_gc_effectiveness(ws_client):
    """测试垃圾回收效果"""
    symbol = "BTCUSDT"
    test_duration = 300  # 测试5分钟
    
    process = psutil.Process()
    memory_samples = []
    gc_times = []
    start_time = time.time()
    
    def create_data_consumer():
        """创建消耗内存的数据消费者"""
        data_buffer = []
        def consumer(data):
            data_buffer.append(data)
            if len(data_buffer) > 1000:
                data_buffer.clear()
        return consumer
    
    print("\n开始垃圾回收效果测试...")
    
    # 订阅数据
    consumer = create_data_consumer()
    await ws_client.subscribe_ticker(Market.SPOT, symbol, consumer)
    
    try:
        while time.time() - start_time < test_duration:
            # 记录当前内存
            memory_used = process.memory_info().rss / 1024 / 1024
            memory_samples.append(memory_used)
            
            if len(memory_samples) % 30 == 0:  # 每30秒触发一次GC
                gc_start_time = time.time()
                gc.collect()
                gc_duration = time.time() - gc_start_time
                gc_times.append(gc_duration)
                
                print(f"\n触发GC:")
                print(f"GC耗时: {gc_duration*1000:.2f}ms")
                print(f"当前内存: {memory_used:.2f}MB")
            
            await asyncio.sleep(1)
    
    finally:
        # 取消订阅
        await ws_client.unsubscribe_ticker(Market.SPOT, symbol)
        
        # 最终GC
        gc.collect()
        final_memory = process.memory_info().rss / 1024 / 1024
        
        # 统计分析
        avg_gc_time = sum(gc_times) / len(gc_times)
        max_gc_time = max(gc_times)
        memory_after_gc = final_memory
        
        print(f"\nGC效果统计:")
        print(f"平均GC时间: {avg_gc_time*1000:.2f}ms")
        print(f"最大GC时间: {max_gc_time*1000:.2f}ms")
        print(f"最终内存使用: {memory_after_gc:.2f}MB")
        
        # 绘制内存使用图表
        plot_memory_usage(range(len(memory_samples)), memory_samples, "GC效果测试")
        
        assert avg_gc_time < 1.0  # 平均GC时间应小于1秒
        assert max_gc_time < 2.0  # 最大GC时间应小于2秒 