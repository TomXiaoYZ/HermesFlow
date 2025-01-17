"""
事件发布装饰器
"""
import json
import functools
import asyncio
from datetime import datetime
from typing import Any, Dict, Optional, Callable
from .connection import db_manager
from .config import KAFKA_TOPICS

def publish_event(event_type: str, topic: Optional[str] = None):
    """
    事件发布装饰器

    Args:
        event_type: 事件类型
        topic: Kafka主题，如果不指定则根据事件类型自动选择

    Example:
        @publish_event("order.created")
        async def create_order(...):
            ...
    """
    def decorator(func: Callable):
        @functools.wraps(func)
        async def wrapper(*args, **kwargs):
            # 执行原函数
            result = await func(*args, **kwargs)

            # 构建事件数据
            event_data = {
                "event_type": event_type,
                "timestamp": datetime.utcnow().isoformat(),
                "data": result.to_dict() if hasattr(result, "to_dict") else result
            }

            # 确定主题
            event_topic = topic
            if event_topic is None:
                if "order" in event_type:
                    event_topic = KAFKA_TOPICS["order_events"]
                elif "trade" in event_type:
                    event_topic = KAFKA_TOPICS["trade_events"]
                else:
                    raise ValueError(f"未知的事件类型: {event_type}")

            # 异步发布事件
            try:
                producer = db_manager.get_kafka_producer()
                if producer:
                    await producer.send_and_wait(
                        event_topic,
                        json.dumps(event_data).encode()
                    )
            except Exception as e:
                # 记录错误但不影响主流程
                print(f"发布事件失败: {str(e)}")

            return result
        return wrapper
    return decorator

def batch_publish_events(event_type: str, topic: Optional[str] = None):
    """
    批量事件发布装饰器

    Args:
        event_type: 事件类型
        topic: Kafka主题，如果不指定则根据事件类型自动选择

    Example:
        @batch_publish_events("order.batch_created")
        async def create_orders(orders):
            ...
    """
    def decorator(func: Callable):
        @functools.wraps(func)
        async def wrapper(*args, **kwargs):
            # 执行原函数
            results = await func(*args, **kwargs)

            # 构建批量事件数据
            events = []
            for result in results:
                event_data = {
                    "event_type": event_type,
                    "timestamp": datetime.utcnow().isoformat(),
                    "data": result.to_dict() if hasattr(result, "to_dict") else result
                }
                events.append(event_data)

            # 确定主题
            event_topic = topic
            if event_topic is None:
                if "order" in event_type:
                    event_topic = KAFKA_TOPICS["order_events"]
                elif "trade" in event_type:
                    event_topic = KAFKA_TOPICS["trade_events"]
                else:
                    raise ValueError(f"未知的事件类型: {event_type}")

            # 异步发布事件
            try:
                producer = db_manager.get_kafka_producer()
                if producer:
                    # 批量发送
                    await asyncio.gather(*[
                        producer.send_and_wait(
                            event_topic,
                            json.dumps(event).encode()
                        )
                        for event in events
                    ])
            except Exception as e:
                # 记录错误但不影响主流程
                print(f"批量发布事件失败: {str(e)}")

            return results
        return wrapper
    return decorator 