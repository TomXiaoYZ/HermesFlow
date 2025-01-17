"""
通用装饰器
"""
import asyncio
import functools
from typing import Type, Optional, Callable, Any, List, Union
from datetime import datetime, timedelta

class RetryError(Exception):
    """重试错误"""
    pass

def retry(
    max_retries: int = 3,
    retry_delay: float = 1.0,
    max_delay: float = 10.0,
    exponential_base: float = 2.0,
    exceptions: Union[Type[Exception], List[Type[Exception]]] = Exception,
    should_retry: Optional[Callable[[Exception], bool]] = None
):
    """重试装饰器
    
    Args:
        max_retries: 最大重试次数
        retry_delay: 初始重试延迟（秒）
        max_delay: 最大重试延迟（秒）
        exponential_base: 指数退避基数
        exceptions: 需要重试的异常类型
        should_retry: 自定义重试判断函数
    """
    def decorator(func):
        @functools.wraps(func)
        async def wrapper(*args, **kwargs):
            last_exception = None
            delay = retry_delay

            for attempt in range(max_retries + 1):
                try:
                    return await func(*args, **kwargs)
                except Exception as e:
                    last_exception = e
                    
                    # 检查是否是需要重试的异常
                    if not isinstance(e, exceptions):
                        raise
                    
                    # 检查是否需要重试
                    if should_retry and not should_retry(e):
                        raise
                    
                    # 最后一次尝试失败
                    if attempt == max_retries:
                        raise RetryError(
                            f"达到最大重试次数 {max_retries}，"
                            f"最后一次错误: {str(last_exception)}"
                        ) from last_exception
                    
                    # 计算下一次重试延迟
                    delay = min(delay * exponential_base, max_delay)
                    
                    # 记录重试信息
                    print(f"第{attempt + 1}次重试失败: {str(e)}")
                    print(f"等待{delay}秒后重试...")
                    
                    # 等待后重试
                    await asyncio.sleep(delay)
            
            # 不应该到达这里
            raise RetryError("未知错误")
        
        return wrapper
    
    return decorator 