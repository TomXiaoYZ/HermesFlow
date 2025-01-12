"""
Binance 数据接入服务
"""
import asyncio
from datetime import datetime
from decimal import Decimal
from typing import Any, Dict, List, Optional

from binance import AsyncClient, BinanceSocketManager
from binance.exceptions import BinanceAPIException

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


class BinanceService:
    """Binance 数据服务类"""

    def __init__(self, kafka_producer: KafkaProducerService) -> None:
        """初始化服务"""
        self.client: Optional[AsyncClient] = None
        self.bm: Optional[BinanceSocketManager] = None
        self.ws_tasks: List[asyncio.Task] = []
        self.symbols: List[str] = []
        self.kafka_producer = kafka_producer

    async def initialize(self) -> None:
        """初始化 Binance 客户端"""
        try:
            self.client = await AsyncClient.create(
                api_key=settings.BINANCE_API_KEY,
                api_secret=settings.BINANCE_API_SECRET,
                testnet=settings.BINANCE_TESTNET,
            )
            self.bm = BinanceSocketManager(self.client)
            # 获取所有交易对
            exchange_info = await self.client.get_exchange_info()
            self.symbols = [
                symbol["symbol"]
                for symbol in exchange_info["symbols"]
                if symbol["status"] == "TRADING"
            ]
            logger.info(
                "binance_service_initialized",
                symbols_count=len(self.symbols)
            )
        except BinanceAPIException as e:
            logger.error(
                "binance_service_initialization_failed",
                error=str(e)
            )
            raise

    async def close(self) -> None:
        """关闭连接"""
        if self.client:
            await self.client.close_connection()
        for task in self.ws_tasks:
            task.cancel()
        logger.info("binance_service_closed")

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
        if not self.bm:
            return
        
        WEBSOCKET_CONNECTION_COUNT.labels(
            exchange=Exchange.BINANCE.value,
            status="started"
        ).inc()
        
        async with self.bm.symbol_ticker_socket(symbol=symbol) as stream:
            ACTIVE_WEBSOCKET_CONNECTIONS.labels(
                exchange=Exchange.BINANCE.value
            ).inc()
            
            try:
                while True:
                    res = await stream.recv()
                    with DATA_PROCESSING_LATENCY.labels(
                        exchange=Exchange.BINANCE.value,
                        data_type=DataType.TICKER.value
                    ).time():
                        ticker = Ticker(
                            exchange=Exchange.BINANCE,
                            symbol=res["s"],
                            price=Decimal(res["c"]),
                            volume=Decimal(res["v"]),
                            timestamp=datetime.fromtimestamp(res["E"] / 1000),
                            bid_price=Decimal(res["b"]),
                            bid_volume=Decimal(res["B"]),
                            ask_price=Decimal(res["a"]),
                            ask_volume=Decimal(res["A"]),
                            high_24h=Decimal(res["h"]),
                            low_24h=Decimal(res["l"]),
                            volume_24h=Decimal(res["v"]),
                            quote_volume_24h=Decimal(res["q"]),
                            price_change_24h=Decimal(res["p"]),
                            price_change_percent_24h=float(res["P"]),
                        )
                        # 发送到 Kafka
                        self.kafka_producer.send_market_data(
                            exchange=Exchange.BINANCE,
                            data_type=DataType.TICKER,
                            symbol=symbol,
                            data=ticker.model_dump(),
                        )
                        DATA_PROCESSING_COUNT.labels(
                            exchange=Exchange.BINANCE.value,
                            data_type=DataType.TICKER.value,
                            status="success"
                        ).inc()
            except Exception as e:
                logger.error(
                    "binance_ticker_stream_error",
                    symbol=symbol,
                    error=str(e)
                )
                DATA_PROCESSING_COUNT.labels(
                    exchange=Exchange.BINANCE.value,
                    data_type=DataType.TICKER.value,
                    status="error"
                ).inc()
            finally:
                ACTIVE_WEBSOCKET_CONNECTIONS.labels(
                    exchange=Exchange.BINANCE.value
                ).dec()

    async def _start_kline_stream(self, symbol: str) -> None:
        """启动K线数据流"""
        if not self.bm:
            return
        
        WEBSOCKET_CONNECTION_COUNT.labels(
            exchange=Exchange.BINANCE.value,
            status="started"
        ).inc()
        
        async with self.bm.kline_socket(symbol=symbol) as stream:
            ACTIVE_WEBSOCKET_CONNECTIONS.labels(
                exchange=Exchange.BINANCE.value
            ).inc()
            
            try:
                while True:
                    res = await stream.recv()
                    with DATA_PROCESSING_LATENCY.labels(
                        exchange=Exchange.BINANCE.value,
                        data_type=DataType.KLINE.value
                    ).time():
                        k = res["k"]
                        kline = Kline(
                            exchange=Exchange.BINANCE,
                            symbol=k["s"],
                            interval=k["i"],
                            open_time=datetime.fromtimestamp(k["t"] / 1000),
                            close_time=datetime.fromtimestamp(k["T"] / 1000),
                            open=Decimal(k["o"]),
                            high=Decimal(k["h"]),
                            low=Decimal(k["l"]),
                            close=Decimal(k["c"]),
                            volume=Decimal(k["v"]),
                            quote_volume=Decimal(k["q"]),
                            trades_count=k["n"],
                            taker_buy_volume=Decimal(k["V"]),
                            taker_buy_quote_volume=Decimal(k["Q"]),
                        )
                        # 发送到 Kafka
                        self.kafka_producer.send_market_data(
                            exchange=Exchange.BINANCE,
                            data_type=DataType.KLINE,
                            symbol=symbol,
                            data=kline.model_dump(),
                        )
                        DATA_PROCESSING_COUNT.labels(
                            exchange=Exchange.BINANCE.value,
                            data_type=DataType.KLINE.value,
                            status="success"
                        ).inc()
            except Exception as e:
                logger.error(
                    "binance_kline_stream_error",
                    symbol=symbol,
                    error=str(e)
                )
                DATA_PROCESSING_COUNT.labels(
                    exchange=Exchange.BINANCE.value,
                    data_type=DataType.KLINE.value,
                    status="error"
                ).inc()
            finally:
                ACTIVE_WEBSOCKET_CONNECTIONS.labels(
                    exchange=Exchange.BINANCE.value
                ).dec()

    async def _start_depth_stream(self, symbol: str) -> None:
        """启动深度数据流"""
        if not self.bm:
            return
        
        WEBSOCKET_CONNECTION_COUNT.labels(
            exchange=Exchange.BINANCE.value,
            status="started"
        ).inc()
        
        async with self.bm.depth_socket(symbol=symbol) as stream:
            ACTIVE_WEBSOCKET_CONNECTIONS.labels(
                exchange=Exchange.BINANCE.value
            ).inc()
            
            try:
                while True:
                    res = await stream.recv()
                    with DATA_PROCESSING_LATENCY.labels(
                        exchange=Exchange.BINANCE.value,
                        data_type=DataType.ORDERBOOK.value
                    ).time():
                        orderbook = OrderBook(
                            exchange=Exchange.BINANCE,
                            symbol=res["s"],
                            timestamp=datetime.fromtimestamp(res["E"] / 1000),
                            last_update_id=res["u"],
                            bids=[
                                OrderBookLevel(
                                    price=Decimal(price),
                                    quantity=Decimal(qty)
                                )
                                for price, qty in res["b"]
                            ],
                            asks=[
                                OrderBookLevel(
                                    price=Decimal(price),
                                    quantity=Decimal(qty)
                                )
                                for price, qty in res["a"]
                            ],
                        )
                        # 发送到 Kafka
                        self.kafka_producer.send_market_data(
                            exchange=Exchange.BINANCE,
                            data_type=DataType.ORDERBOOK,
                            symbol=symbol,
                            data=orderbook.model_dump(),
                        )
                        DATA_PROCESSING_COUNT.labels(
                            exchange=Exchange.BINANCE.value,
                            data_type=DataType.ORDERBOOK.value,
                            status="success"
                        ).inc()
            except Exception as e:
                logger.error(
                    "binance_depth_stream_error",
                    symbol=symbol,
                    error=str(e)
                )
                DATA_PROCESSING_COUNT.labels(
                    exchange=Exchange.BINANCE.value,
                    data_type=DataType.ORDERBOOK.value,
                    status="error"
                ).inc()
            finally:
                ACTIVE_WEBSOCKET_CONNECTIONS.labels(
                    exchange=Exchange.BINANCE.value
                ).dec()

    async def _start_trade_stream(self, symbol: str) -> None:
        """启动成交数据流"""
        if not self.bm:
            return
        
        WEBSOCKET_CONNECTION_COUNT.labels(
            exchange=Exchange.BINANCE.value,
            status="started"
        ).inc()
        
        async with self.bm.trade_socket(symbol=symbol) as stream:
            ACTIVE_WEBSOCKET_CONNECTIONS.labels(
                exchange=Exchange.BINANCE.value
            ).inc()
            
            try:
                while True:
                    res = await stream.recv()
                    with DATA_PROCESSING_LATENCY.labels(
                        exchange=Exchange.BINANCE.value,
                        data_type=DataType.TRADE.value
                    ).time():
                        trade = Trade(
                            exchange=Exchange.BINANCE,
                            symbol=res["s"],
                            id=str(res["t"]),
                            price=Decimal(res["p"]),
                            quantity=Decimal(res["q"]),
                            timestamp=datetime.fromtimestamp(res["T"] / 1000),
                            is_buyer_maker=res["m"],
                            quote_quantity=Decimal(str(
                                float(res["p"]) * float(res["q"])
                            )),
                        )
                        # 发送到 Kafka
                        self.kafka_producer.send_market_data(
                            exchange=Exchange.BINANCE,
                            data_type=DataType.TRADE,
                            symbol=symbol,
                            data=trade.model_dump(),
                        )
                        DATA_PROCESSING_COUNT.labels(
                            exchange=Exchange.BINANCE.value,
                            data_type=DataType.TRADE.value,
                            status="success"
                        ).inc()
            except Exception as e:
                logger.error(
                    "binance_trade_stream_error",
                    symbol=symbol,
                    error=str(e)
                )
                DATA_PROCESSING_COUNT.labels(
                    exchange=Exchange.BINANCE.value,
                    data_type=DataType.TRADE.value,
                    status="error"
                ).inc()
            finally:
                ACTIVE_WEBSOCKET_CONNECTIONS.labels(
                    exchange=Exchange.BINANCE.value
                ).dec()

    async def get_historical_klines(
        self,
        symbol: str,
        interval: Interval,
        start_time: Optional[datetime] = None,
        end_time: Optional[datetime] = None,
        limit: int = 500
    ) -> List[Kline]:
        """获取历史K线数据"""
        if not self.client:
            raise RuntimeError("Client not initialized")

        try:
            klines = await self.client.get_historical_klines(
                symbol=symbol,
                interval=interval.value,
                start_str=str(int(start_time.timestamp() * 1000)) if start_time else None,
                end_str=str(int(end_time.timestamp() * 1000)) if end_time else None,
                limit=limit
            )
            
            return [
                Kline(
                    exchange=Exchange.BINANCE,
                    symbol=symbol,
                    interval=interval,
                    open_time=datetime.fromtimestamp(k[0] / 1000),
                    close_time=datetime.fromtimestamp(k[6] / 1000),
                    open=Decimal(k[1]),
                    high=Decimal(k[2]),
                    low=Decimal(k[3]),
                    close=Decimal(k[4]),
                    volume=Decimal(k[5]),
                    quote_volume=Decimal(k[7]),
                    trades_count=int(k[8]),
                    taker_buy_volume=Decimal(k[9]),
                    taker_buy_quote_volume=Decimal(k[10]),
                )
                for k in klines
            ]
        except BinanceAPIException as e:
            logger.error(
                "get_historical_klines_error",
                symbol=symbol,
                interval=interval.value,
                error=str(e)
            )
            raise 