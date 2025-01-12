"""
Redis 实时数据存储服务
"""
import json
from datetime import datetime, timedelta
from decimal import Decimal
from typing import Dict, List, Optional, Union

import orjson
import redis.asyncio as redis
from redis.asyncio.client import Redis
from redis.asyncio.connection import ConnectionPool

from app.core.config import settings
from app.core.logging import logger
from app.core.metrics import (
    CACHE_HIT_COUNT,
    CACHE_MISS_COUNT,
    DATA_PROCESSING_LATENCY,
)
from app.models.market_data import (
    DataType,
    Exchange,
    Interval,
    Kline,
    OrderBook,
    OrderBookLevel,
    Ticker,
    Trade,
)


class RedisService:
    """Redis 服务类"""

    def __init__(self) -> None:
        """初始化服务"""
        self.pool: Optional[ConnectionPool] = None
        self.client: Optional[Redis] = None

    async def initialize(self) -> None:
        """初始化 Redis 连接"""
        try:
            self.pool = redis.ConnectionPool(
                host=settings.REDIS_HOST,
                port=settings.REDIS_PORT,
                password=settings.REDIS_PASSWORD,
                db=settings.REDIS_DB,
                decode_responses=True,
            )
            self.client = redis.Redis(connection_pool=self.pool)
            await self.client.ping()
            logger.info("redis_service_initialized")
        except Exception as e:
            logger.error(
                "redis_service_initialization_failed",
                error=str(e)
            )
            raise

    async def close(self) -> None:
        """关闭连接"""
        if self.client:
            await self.client.close()
        if self.pool:
            await self.pool.disconnect()
        logger.info("redis_service_closed")

    def _get_ticker_key(self, exchange: Exchange, symbol: str) -> str:
        """获取Ticker的Redis键"""
        return f"market:ticker:{exchange.value}:{symbol}"

    def _get_kline_key(self, exchange: Exchange, symbol: str, interval: Interval) -> str:
        """获取K线的Redis键"""
        return f"market:kline:{exchange.value}:{symbol}:{interval.value}"

    def _get_orderbook_key(self, exchange: Exchange, symbol: str) -> str:
        """获取订单簿的Redis键"""
        return f"market:orderbook:{exchange.value}:{symbol}"

    def _get_trade_key(self, exchange: Exchange, symbol: str) -> str:
        """获取成交记录的Redis键"""
        return f"market:trade:{exchange.value}:{symbol}"

    async def save_ticker(self, ticker: Ticker) -> None:
        """保存Ticker数据"""
        if not self.client:
            raise Exception("Service not initialized")

        key = self._get_ticker_key(ticker.exchange, ticker.symbol)
        with DATA_PROCESSING_LATENCY.labels(
            exchange=ticker.exchange.value,
            data_type=DataType.TICKER.value
        ).time():
            # 使用Hash存储，便于部分字段更新
            await self.client.hset(
                key,
                mapping={
                    "price": str(ticker.price),
                    "volume": str(ticker.volume),
                    "timestamp": int(ticker.timestamp.timestamp() * 1000),
                    "bid_price": str(ticker.bid_price),
                    "bid_volume": str(ticker.bid_volume),
                    "ask_price": str(ticker.ask_price),
                    "ask_volume": str(ticker.ask_volume),
                    "high_24h": str(ticker.high_24h),
                    "low_24h": str(ticker.low_24h),
                    "volume_24h": str(ticker.volume_24h),
                    "quote_volume_24h": str(ticker.quote_volume_24h),
                    "price_change_24h": str(ticker.price_change_24h),
                    "price_change_percent_24h": ticker.price_change_percent_24h,
                }
            )
            # 设置过期时间为1小时
            await self.client.expire(key, 3600)

    async def get_ticker(
        self,
        exchange: Exchange,
        symbol: str
    ) -> Optional[Ticker]:
        """获取Ticker数据"""
        if not self.client:
            raise Exception("Service not initialized")

        key = self._get_ticker_key(exchange, symbol)
        with DATA_PROCESSING_LATENCY.labels(
            exchange=exchange.value,
            data_type=DataType.TICKER.value
        ).time():
            data = await self.client.hgetall(key)
            if not data:
                CACHE_MISS_COUNT.labels(
                    exchange=exchange.value,
                    data_type=DataType.TICKER.value
                ).inc()
                return None

            CACHE_HIT_COUNT.labels(
                exchange=exchange.value,
                data_type=DataType.TICKER.value
            ).inc()
            return Ticker(
                exchange=exchange,
                symbol=symbol,
                price=Decimal(data["price"]),
                volume=Decimal(data["volume"]),
                timestamp=datetime.fromtimestamp(int(data["timestamp"]) / 1000),
                bid_price=Decimal(data["bid_price"]),
                bid_volume=Decimal(data["bid_volume"]),
                ask_price=Decimal(data["ask_price"]),
                ask_volume=Decimal(data["ask_volume"]),
                high_24h=Decimal(data["high_24h"]),
                low_24h=Decimal(data["low_24h"]),
                volume_24h=Decimal(data["volume_24h"]),
                quote_volume_24h=Decimal(data["quote_volume_24h"]),
                price_change_24h=Decimal(data["price_change_24h"]),
                price_change_percent_24h=float(data["price_change_percent_24h"]),
            )

    async def save_kline(self, kline: Kline) -> None:
        """保存K线数据"""
        if not self.client:
            raise Exception("Service not initialized")

        key = self._get_kline_key(kline.exchange, kline.symbol, kline.interval)
        with DATA_PROCESSING_LATENCY.labels(
            exchange=kline.exchange.value,
            data_type=DataType.KLINE.value
        ).time():
            # 使用Sorted Set存储，score为开盘时间戳
            score = int(kline.open_time.timestamp() * 1000)
            await self.client.zadd(
                key,
                {
                    orjson.dumps(kline.model_dump()).decode(): score
                }
            )
            # 只保留最近1000根K线
            await self.client.zremrangebyrank(key, 0, -1001)
            # 设置过期时间为7天
            await self.client.expire(key, 7 * 24 * 3600)

    async def get_klines(
        self,
        exchange: Exchange,
        symbol: str,
        interval: Interval,
        start_time: Optional[datetime] = None,
        end_time: Optional[datetime] = None,
        limit: int = 100
    ) -> List[Kline]:
        """获取K线数据"""
        if not self.client:
            raise Exception("Service not initialized")

        key = self._get_kline_key(exchange, symbol, interval)
        with DATA_PROCESSING_LATENCY.labels(
            exchange=exchange.value,
            data_type=DataType.KLINE.value
        ).time():
            # 设置时间范围
            min_score = "-inf"
            max_score = "+inf"
            if start_time:
                min_score = int(start_time.timestamp() * 1000)
            if end_time:
                max_score = int(end_time.timestamp() * 1000)

            # 获取指定范围的数据
            data = await self.client.zrangebyscore(
                key,
                min_score,
                max_score,
                start=0,
                num=limit
            )

            if not data:
                CACHE_MISS_COUNT.labels(
                    exchange=exchange.value,
                    data_type=DataType.KLINE.value
                ).inc()
                return []

            CACHE_HIT_COUNT.labels(
                exchange=exchange.value,
                data_type=DataType.KLINE.value
            ).inc()
            return [
                Kline(**orjson.loads(item))
                for item in data
            ]

    async def save_orderbook(self, orderbook: OrderBook) -> None:
        """保存订单簿数据"""
        if not self.client:
            raise Exception("Service not initialized")

        key = self._get_orderbook_key(orderbook.exchange, orderbook.symbol)
        with DATA_PROCESSING_LATENCY.labels(
            exchange=orderbook.exchange.value,
            data_type=DataType.ORDERBOOK.value
        ).time():
            # 使用Hash存储，分别存储买卖盘和时间戳
            await self.client.hset(
                key,
                mapping={
                    "timestamp": int(orderbook.timestamp.timestamp() * 1000),
                    "bids": orjson.dumps([
                        [str(level.price), str(level.volume)]
                        for level in orderbook.bids
                    ]).decode(),
                    "asks": orjson.dumps([
                        [str(level.price), str(level.volume)]
                        for level in orderbook.asks
                    ]).decode(),
                }
            )
            # 设置过期时间为1小时
            await self.client.expire(key, 3600)

    async def get_orderbook(
        self,
        exchange: Exchange,
        symbol: str
    ) -> Optional[OrderBook]:
        """获取订单簿数据"""
        if not self.client:
            raise Exception("Service not initialized")

        key = self._get_orderbook_key(exchange, symbol)
        with DATA_PROCESSING_LATENCY.labels(
            exchange=exchange.value,
            data_type=DataType.ORDERBOOK.value
        ).time():
            data = await self.client.hgetall(key)
            if not data:
                CACHE_MISS_COUNT.labels(
                    exchange=exchange.value,
                    data_type=DataType.ORDERBOOK.value
                ).inc()
                return None

            CACHE_HIT_COUNT.labels(
                exchange=exchange.value,
                data_type=DataType.ORDERBOOK.value
            ).inc()

            bids_data = orjson.loads(data["bids"])
            asks_data = orjson.loads(data["asks"])

            return OrderBook(
                exchange=exchange,
                symbol=symbol,
                timestamp=datetime.fromtimestamp(int(data["timestamp"]) / 1000),
                bids=[
                    OrderBookLevel(
                        price=Decimal(price),
                        volume=Decimal(volume)
                    )
                    for price, volume in bids_data
                ],
                asks=[
                    OrderBookLevel(
                        price=Decimal(price),
                        volume=Decimal(volume)
                    )
                    for price, volume in asks_data
                ],
            )

    async def save_trade(self, trade: Trade) -> None:
        """保存成交记录"""
        if not self.client:
            raise Exception("Service not initialized")

        key = self._get_trade_key(trade.exchange, trade.symbol)
        with DATA_PROCESSING_LATENCY.labels(
            exchange=trade.exchange.value,
            data_type=DataType.TRADE.value
        ).time():
            # 使用Sorted Set存储，score为成交时间戳
            score = int(trade.timestamp.timestamp() * 1000)
            await self.client.zadd(
                key,
                {
                    orjson.dumps(trade.model_dump()).decode(): score
                }
            )
            # 只保留最近1000条成交记录
            await self.client.zremrangebyrank(key, 0, -1001)
            # 设置过期时间为1天
            await self.client.expire(key, 24 * 3600)

    async def get_trades(
        self,
        exchange: Exchange,
        symbol: str,
        start_time: Optional[datetime] = None,
        end_time: Optional[datetime] = None,
        limit: int = 100
    ) -> List[Trade]:
        """获取成交记录"""
        if not self.client:
            raise Exception("Service not initialized")

        key = self._get_trade_key(exchange, symbol)
        with DATA_PROCESSING_LATENCY.labels(
            exchange=exchange.value,
            data_type=DataType.TRADE.value
        ).time():
            # 设置时间范围
            min_score = "-inf"
            max_score = "+inf"
            if start_time:
                min_score = int(start_time.timestamp() * 1000)
            if end_time:
                max_score = int(end_time.timestamp() * 1000)

            # 获取指定范围的数据
            data = await self.client.zrangebyscore(
                key,
                min_score,
                max_score,
                start=0,
                num=limit
            )

            if not data:
                CACHE_MISS_COUNT.labels(
                    exchange=exchange.value,
                    data_type=DataType.TRADE.value
                ).inc()
                return []

            CACHE_HIT_COUNT.labels(
                exchange=exchange.value,
                data_type=DataType.TRADE.value
            ).inc()
            return [
                Trade(**orjson.loads(item))
                for item in data
            ] 