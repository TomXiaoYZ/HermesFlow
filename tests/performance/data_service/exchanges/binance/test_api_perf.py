"""
Binance API性能测试
测试API的并发性能、响应时间和错误处理能力
"""
import os
import pytest
import asyncio
import time
import statistics
from datetime import datetime, timedelta
from concurrent.futures import ThreadPoolExecutor

from src.backend.data_service.exchanges.binance.client import BinanceAPI
from src.backend.data_service.common.models import Market

@pytest.fixture
async def api_client():
    """创建API测试客户端"""
    api_key = os.getenv("BINANCE_API_KEY", "")
    api_secret = os.getenv("BINANCE_API_SECRET", "")
    client = BinanceAPI(api_key, api_secret, testnet=True)
    yield client

@pytest.mark.asyncio
async def test_concurrent_requests(api_client):
    """测试并发请求性能"""
    symbol = "BTCUSDT"
    request_count = 50
    response_times = []
    errors = []
    
    async def make_request():
        try:
            start_time = time.time()
            await api_client.get_ticker(Market.SPOT, symbol)
            end_time = time.time()
            response_times.append(end_time - start_time)
        except Exception as e:
            errors.append(e)
    
    # 创建并发请求
    tasks = [make_request() for _ in range(request_count)]
    await asyncio.gather(*tasks)
    
    # 计算性能指标
    success_rate = (request_count - len(errors)) / request_count * 100
    avg_response_time = statistics.mean(response_times)
    max_response_time = max(response_times)
    min_response_time = min(response_times)
    p95_response_time = statistics.quantiles(response_times, n=20)[18]  # 95th percentile
    
    print(f"\nAPI并发性能测试结果:")
    print(f"总请求数: {request_count}")
    print(f"成功率: {success_rate:.2f}%")
    print(f"平均响应时间: {avg_response_time:.3f}秒")
    print(f"最大响应时间: {max_response_time:.3f}秒")
    print(f"最小响应时间: {min_response_time:.3f}秒")
    print(f"95%响应时间: {p95_response_time:.3f}秒")
    
    assert success_rate >= 95  # 成功率应不低于95%
    assert avg_response_time < 1.0  # 平均响应时间应小于1秒
    assert p95_response_time < 2.0  # 95%的请求应在2秒内完成

@pytest.mark.asyncio
async def test_rate_limiting(api_client):
    """测试频率限制处理"""
    symbol = "BTCUSDT"
    request_count = 100
    interval = 1  # 1秒内发送所有请求
    errors = []
    
    async def make_request(i):
        try:
            await api_client.get_ticker(Market.SPOT, symbol)
            return True
        except Exception as e:
            errors.append((i, str(e)))
            return False
    
    start_time = time.time()
    tasks = [make_request(i) for i in range(request_count)]
    results = await asyncio.gather(*tasks)
    end_time = time.time()
    
    success_count = sum(1 for r in results if r)
    rate_limit_errors = sum(1 for _, err in errors if "rate limit" in err.lower())
    
    print(f"\n频率限制测试结果:")
    print(f"总请求数: {request_count}")
    print(f"成功请求数: {success_count}")
    print(f"频率限制错误数: {rate_limit_errors}")
    print(f"其他错误数: {len(errors) - rate_limit_errors}")
    print(f"总执行时间: {end_time - start_time:.2f}秒")
    
    # 验证错误处理
    assert rate_limit_errors > 0  # 应该触发一些频率限制
    assert success_count > 0  # 部分请求应该成功

@pytest.mark.asyncio
async def test_error_rate(api_client):
    """测试错误率"""
    test_cases = [
        ("BTCUSDT", True),  # 有效交易对
        ("INVALIDPAIR", False),  # 无效交易对
        ("ETHUSDT", True),  # 有效交易对
        ("BTC-USD", False),  # 格式错误的交易对
    ]
    
    results = []
    for symbol, should_succeed in test_cases:
        try:
            await api_client.get_ticker(Market.SPOT, symbol)
            results.append((symbol, True))
        except Exception as e:
            results.append((symbol, False))
    
    # 验证结果
    for (symbol, should_succeed), (_, succeeded) in zip(test_cases, results):
        assert should_succeed == succeeded, f"Symbol {symbol} {'failed' if should_succeed else 'succeeded'} unexpectedly"

@pytest.mark.asyncio
async def test_response_consistency(api_client):
    """测试响应一致性"""
    symbol = "BTCUSDT"
    iterations = 10
    responses = []
    
    for _ in range(iterations):
        ticker = await api_client.get_ticker(Market.SPOT, symbol)
        responses.append(ticker)
        await asyncio.sleep(0.1)  # 间隔100ms
    
    # 验证响应格式一致性
    for r in responses:
        assert hasattr(r, 'symbol')
        assert hasattr(r, 'price')
        assert float(r.price) > 0
    
    # 验证价格变化合理性
    price_changes = [
        abs(float(responses[i+1].price) - float(responses[i].price))
        for i in range(len(responses)-1)
    ]
    max_change = max(price_changes)
    assert max_change < 1000  # 价格变化不应过大 