"""
Binance交易所性能测试
"""
import pytest
import asyncio
from decimal import Decimal
from src.backend.data_service.common.models import OrderType, OrderSide
from tests.common.exchange_test_base import BaseExchangeTest
from tests.common.test_utils import PerformanceMetrics, ResourceMonitor, run_concurrent_tasks

class TestBinancePerformance(BaseExchangeTest):
    """Binance交易所性能测试类"""
    
    @pytest.fixture(autouse=True)
    async def setup(self, binance_api, binance_ws, performance_metrics, resource_monitor):
        """测试初始化"""
        self.api_client = binance_api
        self.ws_client = binance_ws
        self.metrics = performance_metrics
        self.monitor = resource_monitor
    
    async def test_api_throughput(self):
        """测试API吞吐量"""
        symbol = self.get_test_symbol()
        
        # 测试不同API端点的性能
        endpoints = [
            (self.api_client.get_ticker, "ticker"),
            (self.api_client.get_order_book, "orderbook"),
            (self.api_client.get_recent_trades, "trades"),
            (self.api_client.get_klines, "klines")
        ]
        
        for endpoint, name in endpoints:
            async def make_request():
                start_time = asyncio.get_event_loop().time()
                try:
                    if name == "klines":
                        result = await endpoint(symbol, "1m")
                    else:
                        result = await endpoint(symbol)
                    success = result is not None
                    end_time = asyncio.get_event_loop().time()
                    return end_time - start_time, success
                except Exception:
                    end_time = asyncio.get_event_loop().time()
                    return end_time - start_time, False
            
            # 执行并发请求
            await run_concurrent_tasks(
                make_request,
                count=100,
                batch_size=10,
                metrics=self.metrics
            )
            
            # 验证性能指标
            assert self.metrics.success_rate > 0.95
            assert self.metrics.avg_response_time < 1.0
            assert self.metrics.p95_response_time < 2.0
            
            # 打印性能统计
            print(f"\n{name}性能统计:")
            self.metrics.print_summary()
            self.metrics.reset()
    
    async def test_websocket_throughput(self):
        """测试WebSocket吞吐量"""
        symbol = self.get_test_symbol()
        updates = {
            "ticker": [],
            "depth": [],
            "kline": [],
            "trades": []
        }
        
        # 启动资源监控
        self.monitor.start()
        
        # 订阅多个数据流
        await self.ws_client.subscribe_ticker(
            symbol,
            lambda x: updates["ticker"].append(x)
        )
        await self.ws_client.subscribe_depth(
            symbol,
            lambda x: updates["depth"].append(x)
        )
        await self.ws_client.subscribe_kline(
            symbol,
            "1m",
            lambda x: updates["kline"].append(x)
        )
        await self.ws_client.subscribe_trades(
            symbol,
            lambda x: updates["trades"].append(x)
        )
        
        # 收集数据60秒
        await asyncio.sleep(60)
        
        # 停止监控
        self.monitor.stop()
        
        # 计算每个数据流的消息速率
        for name, data in updates.items():
            messages_per_second = len(data) / 60
            print(f"\n{name}消息速率: {messages_per_second:.2f} 消息/秒")
            assert messages_per_second > 0  # 每个数据流都应该有消息
        
        # 验证资源使用
        assert self.monitor.avg_cpu < 50  # CPU使用率小于50%
        assert self.monitor.max_memory < 1024  # 内存使用小于1GB
        
        # 打印资源使用统计
        self.monitor.print_summary()
    
    async def test_order_performance(self):
        """测试订单处理性能"""
        # 测试不同类型订单的性能
        order_types = [
            (OrderType.LIMIT, "限价单"),
            (OrderType.MARKET, "市价单"),
            (OrderType.STOP_LOSS, "止损单"),
            (OrderType.TAKE_PROFIT, "止盈单")
        ]
        
        for order_type, name in order_types:
            async def create_and_cancel_order():
                start_time = asyncio.get_event_loop().time()
                try:
                    # 创建订单
                    params = self.get_test_order_params(order_type)
                    order = await self.api_client.create_order(**params)
                    
                    # 取消订单
                    await self.api_client.cancel_order(
                        symbol=params["symbol"],
                        order_id=order.order_id
                    )
                    
                    end_time = asyncio.get_event_loop().time()
                    return end_time - start_time, True
                except Exception:
                    end_time = asyncio.get_event_loop().time()
                    return end_time - start_time, False
            
            # 执行并发订单操作
            await run_concurrent_tasks(
                create_and_cancel_order,
                count=50,
                batch_size=5,
                metrics=self.metrics
            )
            
            # 验证性能指标
            assert self.metrics.success_rate > 0.95
            assert self.metrics.avg_response_time < 2.0
            assert self.metrics.p95_response_time < 3.0
            
            # 打印性能统计
            print(f"\n{name}性能统计:")
            self.metrics.print_summary()
            self.metrics.reset()
    
    async def test_connection_stability(self):
        """测试连接稳定性"""
        symbol = self.get_test_symbol()
        updates = []
        disconnections = 0
        last_update_time = None
        
        def on_update(data):
            nonlocal last_update_time
            updates.append(data)
            last_update_time = asyncio.get_event_loop().time()
        
        # 订阅数据
        await self.ws_client.subscribe_ticker(
            symbol,
            on_update
        )
        
        # 监控10分钟
        start_time = asyncio.get_event_loop().time()
        while asyncio.get_event_loop().time() - start_time < 600:  # 10分钟
            if last_update_time and \
               asyncio.get_event_loop().time() - last_update_time > 5:
                disconnections += 1
            await asyncio.sleep(1)
        
        # 计算消息间隔
        intervals = []
        for i in range(1, len(updates)):
            interval = updates[i].timestamp - updates[i-1].timestamp
            intervals.append(interval.total_seconds())
        
        avg_interval = sum(intervals) / len(intervals)
        
        # 验证稳定性指标
        assert disconnections < 3  # 10分钟内断连次数少于3次
        assert avg_interval < 2  # 平均消息间隔小于2秒
        
        # 打印稳定性统计
        print(f"断连次数: {disconnections}")
        print(f"平均消息间隔: {avg_interval:.2f}秒")
        print(f"总消息数: {len(updates)}")
        print(f"消息处理速率: {len(updates)/600:.2f} 消息/秒")
    
    async def test_error_recovery(self):
        """测试错误恢复"""
        symbol = self.get_test_symbol()
        
        # 测试API错误恢复
        for _ in range(10):
            try:
                await self.api_client.get_ticker(symbol)
                await asyncio.sleep(0.01)  # 快速请求触发限流
            except Exception as e:
                assert "rate limit" in str(e).lower()
                await asyncio.sleep(1)  # 等待限流恢复
                continue
        
        # 测试WebSocket错误恢复
        updates = []
        await self.ws_client.subscribe_ticker(
            symbol,
            lambda x: updates.append(x)
        )
        
        # 等待初始数据
        success = await self.wait_for_data(updates)
        assert success
        
        # 模拟多次断连
        for _ in range(3):
            await self.ws_client._ws.close()
            await asyncio.sleep(5)  # 等待重连
            updates.clear()
            success = await self.wait_for_data(updates)
            assert success  # 每次断连后都应该能恢复数据流 