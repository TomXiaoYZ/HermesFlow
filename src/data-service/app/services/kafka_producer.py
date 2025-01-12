"""
Kafka 生产者服务
"""
import json
from typing import Any, Dict

from kafka import KafkaProducer
from kafka.errors import KafkaError

from app.core.config import settings
from app.core.logging import logger
from app.core.metrics import KAFKA_MESSAGE_COUNT
from app.models.market_data import DataType, Exchange, MarketDataUpdate


class KafkaProducerService:
    """Kafka 生产者服务类"""

    def __init__(self) -> None:
        """初始化服务"""
        self.producer: KafkaProducer | None = None
        self.topics = {
            DataType.TICKER: "market.ticker",
            DataType.KLINE: "market.kline",
            DataType.ORDERBOOK: "market.orderbook",
            DataType.TRADE: "market.trade",
        }

    def initialize(self) -> None:
        """初始化 Kafka 生产者"""
        try:
            self.producer = KafkaProducer(
                bootstrap_servers=settings.KAFKA_BROKERS,
                value_serializer=lambda v: json.dumps(v).encode("utf-8"),
                key_serializer=lambda v: v.encode("utf-8"),
                acks="all",
                retries=3,
                max_in_flight_requests_per_connection=1,
            )
            logger.info("kafka_producer_initialized")
        except KafkaError as e:
            logger.error("kafka_producer_initialization_failed", error=str(e))
            raise

    def close(self) -> None:
        """关闭生产者"""
        if self.producer:
            self.producer.close()
            logger.info("kafka_producer_closed")

    def send_market_data(
        self,
        exchange: Exchange,
        data_type: DataType,
        symbol: str,
        data: Dict[str, Any],
    ) -> None:
        """发送市场数据到 Kafka"""
        if not self.producer:
            raise RuntimeError("Producer not initialized")

        try:
            # 创建市场数据更新对象
            market_data = MarketDataUpdate(
                exchange=exchange,
                data_type=data_type,
                symbol=symbol,
                data=data,
            )

            # 获取目标主题
            topic = self.topics[data_type]

            # 生成消息键（用于分区）
            key = f"{exchange.value}.{symbol}"

            # 发送消息
            self.producer.send(
                topic=topic,
                key=key,
                value=market_data.model_dump(),
            )

            # 更新指标
            KAFKA_MESSAGE_COUNT.labels(
                topic=topic,
                operation="send",
                status="success",
            ).inc()

            logger.debug(
                "kafka_message_sent",
                topic=topic,
                key=key,
                data_type=data_type.value,
            )
        except Exception as e:
            logger.error(
                "kafka_message_send_error",
                topic=self.topics[data_type],
                key=f"{exchange.value}.{symbol}",
                error=str(e),
            )
            KAFKA_MESSAGE_COUNT.labels(
                topic=self.topics[data_type],
                operation="send",
                status="error",
            ).inc()
            raise 