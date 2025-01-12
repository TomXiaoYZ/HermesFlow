"""
数据验证服务
"""
from datetime import datetime, timedelta
from decimal import Decimal
from typing import List, Optional

from app.core.logging import logger
from app.core.metrics import DATA_VALIDATION_COUNT
from app.models.market_data import Kline, OrderBook, Ticker, Trade


class DataValidator:
    """数据验证服务类"""

    @staticmethod
    def validate_ticker(ticker: Ticker) -> bool:
        """验证 Ticker 数据"""
        try:
            # 基础字段验证
            if not all([
                ticker.exchange,
                ticker.symbol,
                ticker.price > 0,
                ticker.volume >= 0,
                ticker.timestamp,
                ticker.bid_price > 0,
                ticker.bid_volume >= 0,
                ticker.ask_price > 0,
                ticker.ask_volume >= 0,
            ]):
                logger.warning(
                    "ticker_validation_failed_basic",
                    exchange=ticker.exchange.value,
                    symbol=ticker.symbol
                )
                DATA_VALIDATION_COUNT.labels(
                    data_type="ticker",
                    validation_type="basic",
                    status="failed"
                ).inc()
                return False

            # 业务规则验证
            if not all([
                ticker.bid_price <= ticker.ask_price,  # 买价应小于卖价
                ticker.timestamp <= datetime.now() + timedelta(minutes=5),  # 时间戳不应该超前太多
                ticker.timestamp >= datetime.now() - timedelta(days=1),  # 数据不应该太旧
                ticker.high_24h >= ticker.low_24h,  # 24小时最高价应大于最低价
            ]):
                logger.warning(
                    "ticker_validation_failed_business",
                    exchange=ticker.exchange.value,
                    symbol=ticker.symbol
                )
                DATA_VALIDATION_COUNT.labels(
                    data_type="ticker",
                    validation_type="business",
                    status="failed"
                ).inc()
                return False

            DATA_VALIDATION_COUNT.labels(
                data_type="ticker",
                validation_type="all",
                status="success"
            ).inc()
            return True

        except Exception as e:
            logger.error(
                "ticker_validation_error",
                exchange=ticker.exchange.value,
                symbol=ticker.symbol,
                error=str(e)
            )
            DATA_VALIDATION_COUNT.labels(
                data_type="ticker",
                validation_type="all",
                status="error"
            ).inc()
            return False

    @staticmethod
    def validate_kline(kline: Kline) -> bool:
        """验证 K线数据"""
        try:
            # 基础字段验证
            if not all([
                kline.exchange,
                kline.symbol,
                kline.interval,
                kline.open_time,
                kline.close_time,
                kline.open > 0,
                kline.high > 0,
                kline.low > 0,
                kline.close > 0,
                kline.volume >= 0,
            ]):
                logger.warning(
                    "kline_validation_failed_basic",
                    exchange=kline.exchange.value,
                    symbol=kline.symbol
                )
                DATA_VALIDATION_COUNT.labels(
                    data_type="kline",
                    validation_type="basic",
                    status="failed"
                ).inc()
                return False

            # 业务规则验证
            if not all([
                kline.high >= kline.low,  # 最高价应大于最低价
                kline.high >= kline.open,  # 最高价应大于开盘价
                kline.high >= kline.close,  # 最高价应大于收盘价
                kline.low <= kline.open,  # 最低价应小于开盘价
                kline.low <= kline.close,  # 最低价应小于收盘价
                kline.close_time > kline.open_time,  # 收盘时间应大于开盘时间
                kline.trades_count >= 0,  # 成交笔数应大于等于0
            ]):
                logger.warning(
                    "kline_validation_failed_business",
                    exchange=kline.exchange.value,
                    symbol=kline.symbol
                )
                DATA_VALIDATION_COUNT.labels(
                    data_type="kline",
                    validation_type="business",
                    status="failed"
                ).inc()
                return False

            DATA_VALIDATION_COUNT.labels(
                data_type="kline",
                validation_type="all",
                status="success"
            ).inc()
            return True

        except Exception as e:
            logger.error(
                "kline_validation_error",
                exchange=kline.exchange.value,
                symbol=kline.symbol,
                error=str(e)
            )
            DATA_VALIDATION_COUNT.labels(
                data_type="kline",
                validation_type="all",
                status="error"
            ).inc()
            return False

    @staticmethod
    def validate_trade(trade: Trade) -> bool:
        """验证成交数据"""
        try:
            # 基础字段验证
            if not all([
                trade.exchange,
                trade.symbol,
                trade.trade_id,
                trade.price > 0,
                trade.quantity > 0,
                trade.timestamp,
            ]):
                logger.warning(
                    "trade_validation_failed_basic",
                    exchange=trade.exchange.value,
                    symbol=trade.symbol
                )
                DATA_VALIDATION_COUNT.labels(
                    data_type="trade",
                    validation_type="basic",
                    status="failed"
                ).inc()
                return False

            # 业务规则验证
            if not all([
                trade.timestamp <= datetime.now() + timedelta(minutes=5),  # 时间戳不应该超前太多
                trade.timestamp >= datetime.now() - timedelta(days=1),  # 数据不应该太旧
            ]):
                logger.warning(
                    "trade_validation_failed_business",
                    exchange=trade.exchange.value,
                    symbol=trade.symbol
                )
                DATA_VALIDATION_COUNT.labels(
                    data_type="trade",
                    validation_type="business",
                    status="failed"
                ).inc()
                return False

            DATA_VALIDATION_COUNT.labels(
                data_type="trade",
                validation_type="all",
                status="success"
            ).inc()
            return True

        except Exception as e:
            logger.error(
                "trade_validation_error",
                exchange=trade.exchange.value,
                symbol=trade.symbol,
                error=str(e)
            )
            DATA_VALIDATION_COUNT.labels(
                data_type="trade",
                validation_type="all",
                status="error"
            ).inc()
            return False

    @staticmethod
    def clean_ticker(ticker: Ticker) -> Optional[Ticker]:
        """清洗 Ticker 数据"""
        try:
            # 移除异常值
            if ticker.volume < 0:
                ticker.volume = Decimal('0')
            if ticker.bid_volume < 0:
                ticker.bid_volume = Decimal('0')
            if ticker.ask_volume < 0:
                ticker.ask_volume = Decimal('0')

            # 确保买卖价格合理
            if ticker.bid_price > ticker.ask_price:
                ticker.bid_price, ticker.ask_price = ticker.ask_price, ticker.bid_price

            # 确保时间戳合理
            if ticker.timestamp > datetime.now() + timedelta(minutes=5):
                ticker.timestamp = datetime.now()

            return ticker
        except Exception as e:
            logger.error(
                "ticker_clean_error",
                exchange=ticker.exchange.value,
                symbol=ticker.symbol,
                error=str(e)
            )
            return None

    @staticmethod
    def clean_kline(kline: Kline) -> Optional[Kline]:
        """清洗 K线数据"""
        try:
            # 确保 OHLC 价格合理
            prices = [kline.open, kline.high, kline.low, kline.close]
            kline.high = max(prices)
            kline.low = min(prices)

            # 确保成交量非负
            if kline.volume < 0:
                kline.volume = Decimal('0')
            if kline.quote_volume < 0:
                kline.quote_volume = Decimal('0')

            # 确保时间戳合理
            if kline.close_time <= kline.open_time:
                kline.close_time = kline.open_time + timedelta(minutes=1)

            return kline
        except Exception as e:
            logger.error(
                "kline_clean_error",
                exchange=kline.exchange.value,
                symbol=kline.symbol,
                error=str(e)
            )
            return None

    @staticmethod
    def clean_trade(trade: Trade) -> Optional[Trade]:
        """清洗成交数据"""
        try:
            # 确保数量为正
            if trade.quantity <= 0:
                return None

            # 确保价格为正
            if trade.price <= 0:
                return None

            # 确保时间戳合理
            if trade.timestamp > datetime.now() + timedelta(minutes=5):
                trade.timestamp = datetime.now()

            return trade
        except Exception as e:
            logger.error(
                "trade_clean_error",
                exchange=trade.exchange.value,
                symbol=trade.symbol,
                error=str(e)
            )
            return None 