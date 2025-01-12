"""
OKX 数据接入服务
"""
import asyncio
from datetime import datetime
from decimal import Decimal
from typing import Any, Dict, List, Optional

import aiohttp
import orjson
from websockets.client import connect as ws_connect

from app.core.config import settings
from app.core.logging import logger
from app.core.metrics import (
    ACTIVE_WEBSOCKET_CONNECTIONS,
    DATA_PROCESSING_COUNT,
    DATA_PROCESSING_LATENCY,
    WEBSOCKET_CONNECTION_COUNT,
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
from app.services.kafka_producer import KafkaProducerService


class OKXService:
    """OKX 数据服务类"""

    def __init__(self, kafka_producer: KafkaProducerService) -> None:
        """初始化服务"""
        self.session: Optional[aiohttp.ClientSession] = None
        self.ws_tasks: List[asyncio.Task] = []
        self.symbols: List[str] = []
        self.kafka_producer = kafka_producer
        self.base_url = "https://www.okx.com"
        self.ws_url = "wss://ws.okx.com:8443/ws/v5/public"

    async def initialize(self) -> None:
        """初始化 OKX 客户端"""
        try:
            self.session = aiohttp.ClientSession(
                headers={
                    "Content-Type": "application/json",
                    "OK-ACCESS-KEY": settings.OKX_API_KEY,
                }
            )
            # 获取所有交易对
            async with self.session.get(f"{self.base_url}/api/v5/market/tickers?instType=SPOT") as response:
                if response.status == 200:
                    data = await response.json()
                    self.symbols = [
                        item["instId"]
                        for item in data["data"]
                        if item["state"] == "live"
                    ]
                    logger.info(
                        "okx_service_initialized",
                        symbols_count=len(self.symbols)
                    )
                else:
                    raise Exception(f"Failed to get symbols: {response.status}")
        except Exception as e:
            logger.error(
                "okx_service_initialization_failed",
                error=str(e)
            )
            raise

    async def close(self) -> None:
        """关闭连接"""
        if self.session:
            await self.session.close()
        for task in self.ws_tasks:
            task.cancel()
        logger.info("okx_service_closed")

    async def start_market_data_stream(self) -> None:
        """启动市场数据流"""
        # 创建 WebSocket 连接任务
        tasks = []
        for symbol in self.symbols[:10]:  # 先订阅前10个交易对
            tasks.extend([
                self._start_ticker_stream(symbol),
                self._start_kline_stream(symbol),
                self._start_depth_stream(symbol),
                self._start_trade_stream(symbol),
            ])
        self.ws_tasks = tasks
        await asyncio.gather(*tasks)

    async def _start_ticker_stream(self, symbol: str) -> None:
        """启动行情数据流"""
        WEBSOCKET_CONNECTION_COUNT.labels(
            exchange=Exchange.OKX.value,
            status="started"
        ).inc()
        
        async with ws_connect(self.ws_url) as ws:
            ACTIVE_WEBSOCKET_CONNECTIONS.labels(
                exchange=Exchange.OKX.value
            ).inc()
            
            try:
                # 发送订阅请求
                await ws.send(orjson.dumps({
                    "op": "subscribe",
                    "args": [{
                        "channel": "tickers",
                        "instId": symbol
                    }]
                }))
                
                while True:
                    msg = await ws.recv()
                    data = orjson.loads(msg)
                    
                    if "event" in data:  # 订阅确认消息
                        continue
                        
                    if "data" not in data:
                        continue
                        
                    with DATA_PROCESSING_LATENCY.labels(
                        exchange=Exchange.OKX.value,
                        data_type=DataType.TICKER.value
                    ).time():
                        ticker_data = data["data"][0]
                        ticker = Ticker(
                            exchange=Exchange.OKX,
                            symbol=ticker_data["instId"],
                            price=Decimal(ticker_data["last"]),
                            volume=Decimal(ticker_data["vol24h"]),
                            timestamp=datetime.fromtimestamp(int(ticker_data["ts"]) / 1000),
                            bid_price=Decimal(ticker_data["bidPx"]),
                            bid_volume=Decimal(ticker_data["bidSz"]),
                            ask_price=Decimal(ticker_data["askPx"]),
                            ask_volume=Decimal(ticker_data["askSz"]),
                            high_24h=Decimal(ticker_data["high24h"]),
                            low_24h=Decimal(ticker_data["low24h"]),
                            volume_24h=Decimal(ticker_data["vol24h"]),
                            quote_volume_24h=Decimal(ticker_data["volCcy24h"]),
                            price_change_24h=Decimal(ticker_data["last"]) - Decimal(ticker_data["open24h"]),
                            price_change_percent_24h=float(
                                (Decimal(ticker_data["last"]) - Decimal(ticker_data["open24h"])) 
                                / Decimal(ticker_data["open24h"]) * 100
                            ),
                        )
                        # 发送到 Kafka
                        await self.kafka_producer.send_market_data(
                            exchange=Exchange.OKX,
                            data_type=DataType.TICKER,
                            symbol=symbol,
                            data=ticker.model_dump(),
                        )
                        DATA_PROCESSING_COUNT.labels(
                            exchange=Exchange.OKX.value,
                            data_type=DataType.TICKER.value,
                            status="success"
                        ).inc()
            except Exception as e:
                logger.error(
                    "okx_ticker_stream_error",
                    symbol=symbol,
                    error=str(e)
                )
                DATA_PROCESSING_COUNT.labels(
                    exchange=Exchange.OKX.value,
                    data_type=DataType.TICKER.value,
                    status="error"
                ).inc()
            finally:
                ACTIVE_WEBSOCKET_CONNECTIONS.labels(
                    exchange=Exchange.OKX.value
                ).dec()

    async def _start_kline_stream(self, symbol: str) -> None:
        """启动K线数据流"""
        WEBSOCKET_CONNECTION_COUNT.labels(
            exchange=Exchange.OKX.value,
            status="started"
        ).inc()
        
        async with ws_connect(self.ws_url) as ws:
            ACTIVE_WEBSOCKET_CONNECTIONS.labels(
                exchange=Exchange.OKX.value
            ).inc()
            
            try:
                # 发送订阅请求
                await ws.send(orjson.dumps({
                    "op": "subscribe",
                    "args": [{
                        "channel": "candle1m",
                        "instId": symbol
                    }]
                }))
                
                while True:
                    msg = await ws.recv()
                    data = orjson.loads(msg)
                    
                    if "event" in data:  # 订阅确认消息
                        continue
                        
                    if "data" not in data:
                        continue
                        
                    with DATA_PROCESSING_LATENCY.labels(
                        exchange=Exchange.OKX.value,
                        data_type=DataType.KLINE.value
                    ).time():
                        k = data["data"][0]
                        kline = Kline(
                            exchange=Exchange.OKX,
                            symbol=data["arg"]["instId"],
                            interval=Interval.MIN_1,
                            open_time=datetime.fromtimestamp(int(k[0]) / 1000),
                            close_time=datetime.fromtimestamp(int(k[0]) / 1000 + 60),
                            open=Decimal(k[1]),
                            high=Decimal(k[2]),
                            low=Decimal(k[3]),
                            close=Decimal(k[4]),
                            volume=Decimal(k[5]),
                            quote_volume=Decimal(k[6]),
                            trades_count=int(k[7]),
                            taker_buy_volume=Decimal(k[8]),
                            taker_buy_quote_volume=Decimal(k[9]),
                        )
                        # 发送到 Kafka
                        await self.kafka_producer.send_market_data(
                            exchange=Exchange.OKX,
                            data_type=DataType.KLINE,
                            symbol=symbol,
                            data=kline.model_dump(),
                        )
                        DATA_PROCESSING_COUNT.labels(
                            exchange=Exchange.OKX.value,
                            data_type=DataType.KLINE.value,
                            status="success"
                        ).inc()
            except Exception as e:
                logger.error(
                    "okx_kline_stream_error",
                    symbol=symbol,
                    error=str(e)
                )
                DATA_PROCESSING_COUNT.labels(
                    exchange=Exchange.OKX.value,
                    data_type=DataType.KLINE.value,
                    status="error"
                ).inc()
            finally:
                ACTIVE_WEBSOCKET_CONNECTIONS.labels(
                    exchange=Exchange.OKX.value
                ).dec()

    async def _start_depth_stream(self, symbol: str) -> None:
        """启动深度数据流"""
        WEBSOCKET_CONNECTION_COUNT.labels(
            exchange=Exchange.OKX.value,
            status="started"
        ).inc()
        
        async with ws_connect(self.ws_url) as ws:
            ACTIVE_WEBSOCKET_CONNECTIONS.labels(
                exchange=Exchange.OKX.value
            ).inc()
            
            try:
                # 发送订阅请求
                await ws.send(orjson.dumps({
                    "op": "subscribe",
                    "args": [{
                        "channel": "books",
                        "instId": symbol
                    }]
                }))
                
                while True:
                    msg = await ws.recv()
                    data = orjson.loads(msg)
                    
                    if "event" in data:  # 订阅确认消息
                        continue
                        
                    if "data" not in data:
                        continue
                        
                    with DATA_PROCESSING_LATENCY.labels(
                        exchange=Exchange.OKX.value,
                        data_type=DataType.ORDERBOOK.value
                    ).time():
                        book_data = data["data"][0]
                        orderbook = OrderBook(
                            exchange=Exchange.OKX,
                            symbol=data["arg"]["instId"],
                            timestamp=datetime.fromtimestamp(int(book_data["ts"]) / 1000),
                            bids=[
                                OrderBookLevel(
                                    price=Decimal(price),
                                    volume=Decimal(volume)
                                )
                                for price, volume in book_data["bids"]
                            ],
                            asks=[
                                OrderBookLevel(
                                    price=Decimal(price),
                                    volume=Decimal(volume)
                                )
                                for price, volume in book_data["asks"]
                            ],
                        )
                        # 发送到 Kafka
                        await self.kafka_producer.send_market_data(
                            exchange=Exchange.OKX,
                            data_type=DataType.ORDERBOOK,
                            symbol=symbol,
                            data=orderbook.model_dump(),
                        )
                        DATA_PROCESSING_COUNT.labels(
                            exchange=Exchange.OKX.value,
                            data_type=DataType.ORDERBOOK.value,
                            status="success"
                        ).inc()
            except Exception as e:
                logger.error(
                    "okx_depth_stream_error",
                    symbol=symbol,
                    error=str(e)
                )
                DATA_PROCESSING_COUNT.labels(
                    exchange=Exchange.OKX.value,
                    data_type=DataType.ORDERBOOK.value,
                    status="error"
                ).inc()
            finally:
                ACTIVE_WEBSOCKET_CONNECTIONS.labels(
                    exchange=Exchange.OKX.value
                ).dec()

    async def _start_trade_stream(self, symbol: str) -> None:
        """启动成交数据流"""
        WEBSOCKET_CONNECTION_COUNT.labels(
            exchange=Exchange.OKX.value,
            status="started"
        ).inc()
        
        async with ws_connect(self.ws_url) as ws:
            ACTIVE_WEBSOCKET_CONNECTIONS.labels(
                exchange=Exchange.OKX.value
            ).inc()
            
            try:
                # 发送订阅请求
                await ws.send(orjson.dumps({
                    "op": "subscribe",
                    "args": [{
                        "channel": "trades",
                        "instId": symbol
                    }]
                }))
                
                while True:
                    msg = await ws.recv()
                    data = orjson.loads(msg)
                    
                    if "event" in data:  # 订阅确认消息
                        continue
                        
                    if "data" not in data:
                        continue
                        
                    with DATA_PROCESSING_LATENCY.labels(
                        exchange=Exchange.OKX.value,
                        data_type=DataType.TRADE.value
                    ).time():
                        for trade_data in data["data"]:
                            trade = Trade(
                                exchange=Exchange.OKX,
                                symbol=data["arg"]["instId"],
                                id=str(trade_data["tradeId"]),
                                price=Decimal(trade_data["px"]),
                                volume=Decimal(trade_data["sz"]),
                                timestamp=datetime.fromtimestamp(int(trade_data["ts"]) / 1000),
                                is_buyer_maker=trade_data["side"] == "buy",
                            )
                            # 发送到 Kafka
                            await self.kafka_producer.send_market_data(
                                exchange=Exchange.OKX,
                                data_type=DataType.TRADE,
                                symbol=symbol,
                                data=trade.model_dump(),
                            )
                            DATA_PROCESSING_COUNT.labels(
                                exchange=Exchange.OKX.value,
                                data_type=DataType.TRADE.value,
                                status="success"
                            ).inc()
            except Exception as e:
                logger.error(
                    "okx_trade_stream_error",
                    symbol=symbol,
                    error=str(e)
                )
                DATA_PROCESSING_COUNT.labels(
                    exchange=Exchange.OKX.value,
                    data_type=DataType.TRADE.value,
                    status="error"
                ).inc()
            finally:
                ACTIVE_WEBSOCKET_CONNECTIONS.labels(
                    exchange=Exchange.OKX.value
                ).dec()

    async def get_historical_klines(
        self,
        symbol: str,
        interval: Interval,
        start_time: Optional[datetime] = None,
        end_time: Optional[datetime] = None,
        limit: int = 100
    ) -> List[Kline]:
        """获取历史K线数据"""
        if not self.session:
            raise Exception("Service not initialized")
            
        # 转换时间间隔
        interval_map = {
            Interval.MIN_1: "1m",
            Interval.MIN_5: "5m",
            Interval.MIN_15: "15m",
            Interval.MIN_30: "30m",
            Interval.HOUR_1: "1H",
            Interval.HOUR_4: "4H",
            Interval.DAY_1: "1D",
        }
        
        params = {
            "instId": symbol,
            "bar": interval_map[interval],
            "limit": limit
        }
        
        if start_time:
            params["after"] = int(start_time.timestamp() * 1000)
        if end_time:
            params["before"] = int(end_time.timestamp() * 1000)
            
        async with self.session.get(
            f"{self.base_url}/api/v5/market/candles",
            params=params
        ) as response:
            if response.status == 200:
                data = await response.json()
                return [
                    Kline(
                        exchange=Exchange.OKX,
                        symbol=symbol,
                        interval=interval,
                        open_time=datetime.fromtimestamp(int(k[0]) / 1000),
                        close_time=datetime.fromtimestamp(int(k[0]) / 1000 + interval.to_seconds()),
                        open=Decimal(k[1]),
                        high=Decimal(k[2]),
                        low=Decimal(k[3]),
                        close=Decimal(k[4]),
                        volume=Decimal(k[5]),
                        quote_volume=Decimal(k[6]),
                        trades_count=int(k[7]),
                        taker_buy_volume=Decimal(k[8]),
                        taker_buy_quote_volume=Decimal(k[9]),
                    )
                    for k in data["data"]
                ]
            else:
                raise Exception(f"Failed to get historical klines: {response.status}") 