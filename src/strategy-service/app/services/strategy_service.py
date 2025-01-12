"""
策略管理服务
"""
import asyncio
from datetime import datetime
from typing import Dict, List, Optional, Set, Type

from app.core.config import settings
from app.core.logging import logger
from app.models.market_data import (
    Exchange,
    Interval,
    Kline,
    OrderBook,
    Ticker,
    Trade,
)
from app.models.strategy import BaseStrategy, Signal, StrategyState
from app.services.postgresql_service import PostgresqlService
from app.services.redis_service import RedisService


class StrategyService:
    """策略管理服务"""

    def __init__(self) -> None:
        """初始化服务"""
        self.strategies: Dict[str, BaseStrategy] = {}
        self.redis_service: Optional[RedisService] = None
        self.postgresql_service: Optional[PostgresqlService] = None

    async def initialize(self) -> None:
        """初始化服务"""
        try:
            # 初始化Redis服务
            self.redis_service = RedisService()
            await self.redis_service.initialize()

            # 初始化PostgreSQL服务
            self.postgresql_service = PostgresqlService()
            await self.postgresql_service.initialize()

            logger.info("strategy_service_initialized")
        except Exception as e:
            logger.error(
                "strategy_service_initialization_failed",
                error=str(e)
            )
            raise

    async def close(self) -> None:
        """关闭服务"""
        if self.redis_service:
            await self.redis_service.close()
        if self.postgresql_service:
            await self.postgresql_service.close()
        logger.info("strategy_service_closed")

    async def register_strategy(
        self,
        strategy_class: Type[BaseStrategy],
        name: str,
        exchanges: Set[Exchange],
        symbols: Set[str],
        parameters: Dict
    ) -> None:
        """注册策略"""
        if name in self.strategies:
            raise ValueError(f"Strategy {name} already exists")

        # 创建策略实例
        strategy = strategy_class(name, exchanges, symbols, parameters)
        self.strategies[name] = strategy

        # 初始化策略
        try:
            await strategy.initialize()
            strategy.state = StrategyState.RUNNING
            logger.info(
                "strategy_registered",
                strategy=name,
                exchanges=[e.value for e in exchanges],
                symbols=list(symbols)
            )
        except Exception as e:
            strategy.state = StrategyState.ERROR
            logger.error(
                "strategy_initialization_failed",
                strategy=name,
                error=str(e)
            )
            raise

    async def unregister_strategy(self, name: str) -> None:
        """注销策略"""
        if name not in self.strategies:
            raise ValueError(f"Strategy {name} not found")

        strategy = self.strategies[name]
        strategy.state = StrategyState.STOPPED
        del self.strategies[name]
        logger.info("strategy_unregistered", strategy=name)

    async def get_strategy(self, name: str) -> Optional[BaseStrategy]:
        """获取策略"""
        return self.strategies.get(name)

    async def list_strategies(self) -> List[BaseStrategy]:
        """获取策略列表"""
        return list(self.strategies.values())

    async def on_ticker(self, ticker: Ticker) -> List[Signal]:
        """处理Ticker数据"""
        signals: List[Signal] = []
        for strategy in self.strategies.values():
            if (
                strategy.state == StrategyState.RUNNING
                and ticker.exchange in strategy.exchanges
                and ticker.symbol in strategy.symbols
            ):
                try:
                    # 更新策略缓存
                    strategy.update_ticker(ticker)
                    # 处理数据
                    signal = await strategy.on_ticker(ticker)
                    if signal:
                        signals.append(signal)
                except Exception as e:
                    strategy.state = StrategyState.ERROR
                    logger.error(
                        "strategy_ticker_processing_failed",
                        strategy=strategy.name,
                        error=str(e)
                    )
        return signals

    async def on_kline(self, kline: Kline) -> List[Signal]:
        """处理K线数据"""
        signals: List[Signal] = []
        for strategy in self.strategies.values():
            if (
                strategy.state == StrategyState.RUNNING
                and kline.exchange in strategy.exchanges
                and kline.symbol in strategy.symbols
            ):
                try:
                    # 更新策略缓存
                    strategy.update_kline(kline)
                    # 处理数据
                    signal = await strategy.on_kline(kline)
                    if signal:
                        signals.append(signal)
                except Exception as e:
                    strategy.state = StrategyState.ERROR
                    logger.error(
                        "strategy_kline_processing_failed",
                        strategy=strategy.name,
                        error=str(e)
                    )
        return signals

    async def on_orderbook(self, orderbook: OrderBook) -> List[Signal]:
        """处理订单簿数据"""
        signals: List[Signal] = []
        for strategy in self.strategies.values():
            if (
                strategy.state == StrategyState.RUNNING
                and orderbook.exchange in strategy.exchanges
                and orderbook.symbol in strategy.symbols
            ):
                try:
                    # 更新策略缓存
                    strategy.update_orderbook(orderbook)
                    # 处理数据
                    signal = await strategy.on_orderbook(orderbook)
                    if signal:
                        signals.append(signal)
                except Exception as e:
                    strategy.state = StrategyState.ERROR
                    logger.error(
                        "strategy_orderbook_processing_failed",
                        strategy=strategy.name,
                        error=str(e)
                    )
        return signals

    async def on_trade(self, trade: Trade) -> List[Signal]:
        """处理成交记录"""
        signals: List[Signal] = []
        for strategy in self.strategies.values():
            if (
                strategy.state == StrategyState.RUNNING
                and trade.exchange in strategy.exchanges
                and trade.symbol in strategy.symbols
            ):
                try:
                    # 更新策略缓存
                    strategy.update_trade(trade)
                    # 处理数据
                    signal = await strategy.on_trade(trade)
                    if signal:
                        signals.append(signal)
                except Exception as e:
                    strategy.state = StrategyState.ERROR
                    logger.error(
                        "strategy_trade_processing_failed",
                        strategy=strategy.name,
                        error=str(e)
                    )
        return signals 