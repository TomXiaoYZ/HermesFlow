"""
测试工具类
提供通用的测试辅助功能
"""
import time
import asyncio
import statistics
from typing import List, Callable, Any, Dict
from datetime import datetime
import psutil

class PerformanceMetrics:
    """性能指标收集器"""
    def __init__(self):
        self.response_times = []
        self.error_count = 0
        self.success_count = 0
        self.start_time = None
        self.end_time = None
    
    def start(self):
        """开始计时"""
        self.start_time = time.time()
    
    def stop(self):
        """停止计时"""
        self.end_time = time.time()
    
    def add_response(self, response_time: float, success: bool):
        """添加响应时间"""
        self.response_times.append(response_time)
        if success:
            self.success_count += 1
        else:
            self.error_count += 1
    
    @property
    def total_requests(self) -> int:
        """总请求数"""
        return self.success_count + self.error_count
    
    @property
    def success_rate(self) -> float:
        """成功率"""
        return self.success_count / self.total_requests if self.total_requests > 0 else 0
    
    @property
    def avg_response_time(self) -> float:
        """平均响应时间"""
        return statistics.mean(self.response_times) if self.response_times else 0
    
    @property
    def p95_response_time(self) -> float:
        """95分位响应时间"""
        if len(self.response_times) >= 20:
            return statistics.quantiles(self.response_times, n=20)[18]
        return 0
    
    @property
    def total_time(self) -> float:
        """总执行时间"""
        if self.start_time and self.end_time:
            return self.end_time - self.start_time
        return 0
    
    def print_summary(self):
        """打印性能统计摘要"""
        print(f"性能测试结果:")
        print(f"总请求数: {self.total_requests}")
        print(f"成功请求数: {self.success_count}")
        print(f"失败请求数: {self.error_count}")
        print(f"成功率: {self.success_rate:.2%}")
        print(f"平均响应时间: {self.avg_response_time:.3f}秒")
        print(f"P95响应时间: {self.p95_response_time:.3f}秒")
        print(f"总执行时间: {self.total_time:.3f}秒")

class ResourceMonitor:
    """资源使用监控器"""
    def __init__(self):
        self.process = psutil.Process()
        self.memory_samples = []
        self.cpu_samples = []
        self.start_time = None
        self.end_time = None
    
    def start(self):
        """开始监控"""
        self.start_time = time.time()
        self.memory_samples = []
        self.cpu_samples = []
    
    def sample(self):
        """采集样本"""
        self.memory_samples.append(self.process.memory_info().rss)
        self.cpu_samples.append(self.process.cpu_percent())
    
    def stop(self):
        """停止监控"""
        self.end_time = time.time()
    
    @property
    def avg_memory(self) -> float:
        """平均内存使用(MB)"""
        return statistics.mean(self.memory_samples) / 1024 / 1024 if self.memory_samples else 0
    
    @property
    def max_memory(self) -> float:
        """最大内存使用(MB)"""
        return max(self.memory_samples) / 1024 / 1024 if self.memory_samples else 0
    
    @property
    def avg_cpu(self) -> float:
        """平均CPU使用率"""
        return statistics.mean(self.cpu_samples) if self.cpu_samples else 0
    
    def print_summary(self):
        """打印资源使用摘要"""
        print(f"资源使用统计:")
        print(f"平均内存使用: {self.avg_memory:.2f}MB")
        print(f"最大内存使用: {self.max_memory:.2f}MB")
        print(f"平均CPU使用率: {self.avg_cpu:.1f}%")
        print(f"监控时长: {self.end_time - self.start_time:.1f}秒")

async def run_concurrent_tasks(
    task_func: Callable,
    count: int,
    batch_size: int,
    metrics: PerformanceMetrics
):
    """
    并发执行任务
    
    Args:
        task_func: 任务函数
        count: 总任务数
        batch_size: 批次大小
        metrics: 性能指标收集器
    """
    metrics.start()
    
    for i in range(0, count, batch_size):
        batch = []
        for j in range(min(batch_size, count - i)):
            batch.append(task_func())
        
        results = await asyncio.gather(*batch, return_exceptions=True)
        for result in results:
            if isinstance(result, tuple):
                response_time, success = result
                metrics.add_response(response_time, success)
            else:
                metrics.add_response(0, False)
    
    metrics.stop()

async def monitor_ws_connection(
    ws_client: Any,
    duration: int,
    monitor: ResourceMonitor,
    sample_interval: int = 1
):
    """
    监控WebSocket连接
    
    Args:
        ws_client: WebSocket客户端
        duration: 监控时长(秒)
        monitor: 资源监控器
        sample_interval: 采样间隔(秒)
    """
    monitor.start()
    end_time = time.time() + duration
    
    while time.time() < end_time:
        monitor.sample()
        await asyncio.sleep(sample_interval)
    
    monitor.stop() 