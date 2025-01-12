"""
ClickHouse 数据存储服务
"""
from datetime import datetime
from typing import List, Optional

from clickhouse_driver import Client
from clickhouse_driver.errors import Error as ClickHouseError

from app.core.config import settings
from app.core.logging import logger
from app.core.metrics import (
    DATABASE_OPERATION_COUNT,
    DATABASE_OPERATION_LATENCY,
)
from app.models.market_data import (
    DataType,
    Exchange,
    Interval,
    Kline,
    OrderBook,
    Ticker,
    Trade,
)


class ClickHouseService:
    """ClickHouse 数据服务类"""

    def __init__(self) -> None:
        """初始化服务"""
        self.client = Client(
            host=settings.CLICKHOUSE_HOST,
            port=settings.CLICKHOUSE_PORT,
            user=settings.CLICKHOUSE_USER,
            password=settings.CLICKHOUSE_PASSWORD,
            database=settings.CLICKHOUSE_DATABASE,
        )
        self._ensure_tables()

    def _ensure_tables(self) -> None:
        """确保所需的表都已创建"""
        try:
            # 创建 tickers 表
            self.client.execute("""
                CREATE TABLE IF NOT EXISTS tickers (
                    exchange String,
                    symbol String,
                    price Decimal64(8),
                    volume Decimal64(8),
                    timestamp DateTime,
                    bid_price Decimal64(8),
                    bid_volume Decimal64(8),
                    ask_price Decimal64(8),
                    ask_volume Decimal64(8),
                    high_24h Decimal64(8),
                    low_24h Decimal64(8),
                    volume_24h Decimal64(8),
                    quote_volume_24h Decimal64(8),
                    price_change_24h Decimal64(8),
                    price_change_percent_24h Float64
                ) ENGINE = MergeTree()
                ORDER BY (exchange, symbol, timestamp)
            """)

            # 创建 klines 表
            self.client.execute("""
                CREATE TABLE IF NOT EXISTS klines (
                    exchange String,
                    symbol String,
                    interval String,
                    open_time DateTime,
                    close_time DateTime,
                    open Decimal64(8),
                    high Decimal64(8),
                    low Decimal64(8),
                    close Decimal64(8),
                    volume Decimal64(8),
                    quote_volume Decimal64(8),
                    trades_count UInt32,
                    taker_buy_volume Decimal64(8),
                    taker_buy_quote_volume Decimal64(8)
                ) ENGINE = MergeTree()
                ORDER BY (exchange, symbol, interval, open_time)
            """)

            # 创建 trades 表
            self.client.execute("""
                CREATE TABLE IF NOT EXISTS trades (
                    exchange String,
                    symbol String,
                    trade_id String,
                    price Decimal64(8),
                    quantity Decimal64(8),
                    timestamp DateTime,
                    is_buyer_maker Bool,
                    is_best_match Bool
                ) ENGINE = MergeTree()
                ORDER BY (exchange, symbol, timestamp)
            """)

            logger.info("clickhouse_tables_created")
        except ClickHouseError as e:
            logger.error(
                "clickhouse_table_creation_failed",
                error=str(e)
            )
            raise

    def insert_tickers(self, tickers: List[Ticker]) -> None:
        """批量插入 Ticker 数据"""
        with DATABASE_OPERATION_LATENCY.labels(
            database="clickhouse",
            operation="insert",
            table="tickers"
        ).time():
            try:
                data = [
                    (
                        t.exchange.value,
                        t.symbol,
                        float(t.price),
                        float(t.volume),
                        t.timestamp,
                        float(t.bid_price),
                        float(t.bid_volume),
                        float(t.ask_price),
                        float(t.ask_volume),
                        float(t.high_24h),
                        float(t.low_24h),
                        float(t.volume_24h),
                        float(t.quote_volume_24h),
                        float(t.price_change_24h),
                        t.price_change_percent_24h,
                    )
                    for t in tickers
                ]
                self.client.execute(
                    """
                    INSERT INTO tickers (
                        exchange, symbol, price, volume, timestamp,
                        bid_price, bid_volume, ask_price, ask_volume,
                        high_24h, low_24h, volume_24h, quote_volume_24h,
                        price_change_24h, price_change_percent_24h
                    ) VALUES
                    """,
                    data
                )
                DATABASE_OPERATION_COUNT.labels(
                    database="clickhouse",
                    operation="insert",
                    table="tickers",
                    status="success"
                ).inc(len(tickers))
            except ClickHouseError as e:
                logger.error(
                    "clickhouse_insert_tickers_failed",
                    error=str(e)
                )
                DATABASE_OPERATION_COUNT.labels(
                    database="clickhouse",
                    operation="insert",
                    table="tickers",
                    status="error"
                ).inc(len(tickers))
                raise

    def insert_klines(self, klines: List[Kline]) -> None:
        """批量插入 K线数据"""
        with DATABASE_OPERATION_LATENCY.labels(
            database="clickhouse",
            operation="insert",
            table="klines"
        ).time():
            try:
                data = [
                    (
                        k.exchange.value,
                        k.symbol,
                        k.interval,
                        k.open_time,
                        k.close_time,
                        float(k.open),
                        float(k.high),
                        float(k.low),
                        float(k.close),
                        float(k.volume),
                        float(k.quote_volume),
                        k.trades_count,
                        float(k.taker_buy_volume),
                        float(k.taker_buy_quote_volume),
                    )
                    for k in klines
                ]
                self.client.execute(
                    """
                    INSERT INTO klines (
                        exchange, symbol, interval, open_time, close_time,
                        open, high, low, close, volume, quote_volume,
                        trades_count, taker_buy_volume, taker_buy_quote_volume
                    ) VALUES
                    """,
                    data
                )
                DATABASE_OPERATION_COUNT.labels(
                    database="clickhouse",
                    operation="insert",
                    table="klines",
                    status="success"
                ).inc(len(klines))
            except ClickHouseError as e:
                logger.error(
                    "clickhouse_insert_klines_failed",
                    error=str(e)
                )
                DATABASE_OPERATION_COUNT.labels(
                    database="clickhouse",
                    operation="insert",
                    table="klines",
                    status="error"
                ).inc(len(klines))
                raise

    def insert_trades(self, trades: List[Trade]) -> None:
        """批量插入成交数据"""
        with DATABASE_OPERATION_LATENCY.labels(
            database="clickhouse",
            operation="insert",
            table="trades"
        ).time():
            try:
                data = [
                    (
                        t.exchange.value,
                        t.symbol,
                        t.trade_id,
                        float(t.price),
                        float(t.quantity),
                        t.timestamp,
                        t.is_buyer_maker,
                        t.is_best_match,
                    )
                    for t in trades
                ]
                self.client.execute(
                    """
                    INSERT INTO trades (
                        exchange, symbol, trade_id, price, quantity,
                        timestamp, is_buyer_maker, is_best_match
                    ) VALUES
                    """,
                    data
                )
                DATABASE_OPERATION_COUNT.labels(
                    database="clickhouse",
                    operation="insert",
                    table="trades",
                    status="success"
                ).inc(len(trades))
            except ClickHouseError as e:
                logger.error(
                    "clickhouse_insert_trades_failed",
                    error=str(e)
                )
                DATABASE_OPERATION_COUNT.labels(
                    database="clickhouse",
                    operation="insert",
                    table="trades",
                    status="error"
                ).inc(len(trades))
                raise

    def get_klines(
        self,
        exchange: Exchange,
        symbol: str,
        interval: Interval,
        start_time: Optional[datetime] = None,
        end_time: Optional[datetime] = None,
        limit: int = 500
    ) -> List[Kline]:
        """查询K线数据"""
        with DATABASE_OPERATION_LATENCY.labels(
            database="clickhouse",
            operation="select",
            table="klines"
        ).time():
            try:
                query = """
                    SELECT *
                    FROM klines
                    WHERE exchange = %(exchange)s
                    AND symbol = %(symbol)s
                    AND interval = %(interval)s
                """
                params = {
                    "exchange": exchange.value,
                    "symbol": symbol,
                    "interval": interval,
                }

                if start_time:
                    query += " AND open_time >= %(start_time)s"
                    params["start_time"] = start_time

                if end_time:
                    query += " AND open_time <= %(end_time)s"
                    params["end_time"] = end_time

                query += """
                    ORDER BY open_time DESC
                    LIMIT %(limit)s
                """
                params["limit"] = limit

                rows = self.client.execute(query, params)
                klines = [
                    Kline(
                        exchange=Exchange(row[0]),
                        symbol=row[1],
                        interval=row[2],
                        open_time=row[3],
                        close_time=row[4],
                        open=row[5],
                        high=row[6],
                        low=row[7],
                        close=row[8],
                        volume=row[9],
                        quote_volume=row[10],
                        trades_count=row[11],
                        taker_buy_volume=row[12],
                        taker_buy_quote_volume=row[13],
                    )
                    for row in rows
                ]
                DATABASE_OPERATION_COUNT.labels(
                    database="clickhouse",
                    operation="select",
                    table="klines",
                    status="success"
                ).inc()
                return klines
            except ClickHouseError as e:
                logger.error(
                    "clickhouse_get_klines_failed",
                    error=str(e)
                )
                DATABASE_OPERATION_COUNT.labels(
                    database="clickhouse",
                    operation="select",
                    table="klines",
                    status="error"
                ).inc()
                raise 