"""
回测服务
"""
import asyncio
from datetime import datetime, timedelta
from decimal import Decimal
from typing import Dict, List, Optional, Set, Type, Union

import numpy as np
from scipy import stats

from app.core.config import settings
from app.core.logging import logger
from app.models.backtest import (
    BacktestConfig,
    BacktestResult,
    PerformanceMetrics,
    Position,
    TradeRecord,
    TradeStatus,
)
from app.models.market_data import (
    Exchange,
    Interval,
    Kline,
    OrderBook,
    Ticker,
    Trade,
)
from app.models.strategy import BaseStrategy, Signal, SignalType
from app.services.clickhouse_service import ClickHouseService


class BacktestService:
    """回测服务"""

    def __init__(self) -> None:
        """初始化服务"""
        self.clickhouse_service: Optional[ClickHouseService] = None

    async def initialize(self) -> None:
        """初始化服务"""
        try:
            # 初始化ClickHouse服务
            self.clickhouse_service = ClickHouseService()
            await self.clickhouse_service.initialize()
            logger.info("backtest_service_initialized")
        except Exception as e:
            logger.error(
                "backtest_service_initialization_failed",
                error=str(e)
            )
            raise

    async def close(self) -> None:
        """关闭服务"""
        if self.clickhouse_service:
            await self.clickhouse_service.close()
        logger.info("backtest_service_closed")

    async def run_backtest(
        self,
        strategy_class: Type[BaseStrategy],
        config: BacktestConfig
    ) -> BacktestResult:
        """运行回测"""
        # 创建回测结果
        result = BacktestResult(config=config)

        try:
            # 创建策略实例
            strategy = strategy_class(
                name="backtest",
                exchanges=config.exchanges,
                symbols=config.symbols,
                parameters={}
            )

            # 初始化策略
            await strategy.initialize()

            # 加载历史数据
            data_queue = await self._load_historical_data(config)

            # 初始化账户状态
            account_balance = config.initial_capital
            positions: Dict[str, Position] = {}
            equity_curve = []
            drawdown_curve = []
            max_equity = config.initial_capital
            current_drawdown_start = None

            # 回放历史数据
            while not data_queue.empty():
                data = await data_queue.get()
                data_type = data["type"]
                data_obj = data["data"]

                # 更新策略数据缓存
                if data_type == "kline":
                    strategy.update_kline(data_obj)
                elif data_type == "ticker":
                    strategy.update_ticker(data_obj)
                elif data_type == "orderbook":
                    strategy.update_orderbook(data_obj)
                elif data_type == "trade":
                    strategy.update_trade(data_obj)

                # 处理策略信号
                signal = None
                if data_type == "kline":
                    signal = await strategy.on_kline(data_obj)
                elif data_type == "ticker":
                    signal = await strategy.on_ticker(data_obj)
                elif data_type == "orderbook":
                    signal = await strategy.on_orderbook(data_obj)
                elif data_type == "trade":
                    signal = await strategy.on_trade(data_obj)

                if signal:
                    # 模拟交易执行
                    trade = await self._execute_trade(
                        signal,
                        data_obj,
                        config,
                        account_balance,
                        positions
                    )
                    if trade:
                        result.trades.append(trade)
                        # 更新账户状态
                        account_balance += trade.pnl - trade.fee
                        key = f"{trade.exchange.value}:{trade.symbol}"
                        if key in positions:
                            positions[key].volume = Decimal("0")
                            positions[key].unrealized_pnl = Decimal("0")
                            positions[key].realized_pnl += trade.pnl
                        else:
                            positions[key] = Position(
                                exchange=trade.exchange,
                                symbol=trade.symbol,
                                position_type=trade.trade_type,
                                volume=Decimal("0"),
                                avg_price=trade.price,
                                unrealized_pnl=Decimal("0"),
                                realized_pnl=trade.pnl,
                                open_time=trade.fill_time,
                                last_update_time=trade.fill_time
                            )

                # 更新持仓盈亏
                total_equity = account_balance
                for position in positions.values():
                    if position.volume > 0:
                        if data_type == "kline":
                            current_price = data_obj.close
                        elif data_type == "ticker":
                            current_price = data_obj.price
                        elif data_type == "trade":
                            current_price = data_obj.price
                        else:
                            continue

                        if position.position_type == SignalType.LONG:
                            position.unrealized_pnl = (
                                current_price - position.avg_price
                            ) * position.volume
                        else:
                            position.unrealized_pnl = (
                                position.avg_price - current_price
                            ) * position.volume
                        position.last_update_time = data_obj.timestamp
                        total_equity += position.unrealized_pnl

                # 更新权益曲线
                equity_curve.append({
                    "timestamp": data_obj.timestamp,
                    "equity": total_equity
                })

                # 更新回撤曲线
                if total_equity > max_equity:
                    max_equity = total_equity
                    current_drawdown_start = None
                else:
                    drawdown = (max_equity - total_equity) / max_equity
                    if current_drawdown_start is None:
                        current_drawdown_start = data_obj.timestamp
                    drawdown_curve.append({
                        "timestamp": data_obj.timestamp,
                        "drawdown": drawdown
                    })

            # 计算绩效指标
            result.metrics = await self._calculate_metrics(
                config,
                result.trades,
                equity_curve,
                drawdown_curve
            )
            result.positions = positions
            result.equity_curve = equity_curve
            result.drawdown_curve = drawdown_curve

            logger.info(
                "backtest_completed",
                total_trades=result.metrics.total_trades,
                total_pnl=float(result.metrics.total_pnl),
                win_rate=result.metrics.win_rate,
                sharpe_ratio=result.metrics.sharpe_ratio
            )
        except Exception as e:
            logger.error("backtest_failed", error=str(e))
            raise

        return result

    async def _load_historical_data(
        self,
        config: BacktestConfig
    ) -> asyncio.Queue:
        """加载历史数据"""
        if not self.clickhouse_service:
            raise Exception("Service not initialized")

        # 创建数据队列
        queue = asyncio.Queue()

        try:
            # 加载K线数据
            if "kline" in config.data_types:
                for interval in config.intervals:
                    for exchange in config.exchanges:
                        for symbol in config.symbols:
                            klines = await self.clickhouse_service.get_klines(
                                exchange=exchange,
                                symbol=symbol,
                                interval=interval,
                                start_time=config.start_time,
                                end_time=config.end_time
                            )
                            for kline in klines:
                                await queue.put({
                                    "type": "kline",
                                    "data": kline
                                })

            # 加载Ticker数据
            if "ticker" in config.data_types:
                for exchange in config.exchanges:
                    for symbol in config.symbols:
                        tickers = await self.clickhouse_service.get_tickers(
                            exchange=exchange,
                            symbol=symbol,
                            start_time=config.start_time,
                            end_time=config.end_time
                        )
                        for ticker in tickers:
                            await queue.put({
                                "type": "ticker",
                                "data": ticker
                            })

            # 加载订单簿数据
            if "orderbook" in config.data_types:
                for exchange in config.exchanges:
                    for symbol in config.symbols:
                        orderbooks = await self.clickhouse_service.get_orderbooks(
                            exchange=exchange,
                            symbol=symbol,
                            start_time=config.start_time,
                            end_time=config.end_time
                        )
                        for orderbook in orderbooks:
                            await queue.put({
                                "type": "orderbook",
                                "data": orderbook
                            })

            # 加载成交记录
            if "trade" in config.data_types:
                for exchange in config.exchanges:
                    for symbol in config.symbols:
                        trades = await self.clickhouse_service.get_trades(
                            exchange=exchange,
                            symbol=symbol,
                            start_time=config.start_time,
                            end_time=config.end_time
                        )
                        for trade in trades:
                            await queue.put({
                                "type": "trade",
                                "data": trade
                            })

            logger.info(
                "historical_data_loaded",
                queue_size=queue.qsize()
            )
        except Exception as e:
            logger.error(
                "historical_data_loading_failed",
                error=str(e)
            )
            raise

        return queue

    async def _execute_trade(
        self,
        signal: Signal,
        market_data: Union[Kline, Ticker, OrderBook, Trade],
        config: BacktestConfig,
        account_balance: Decimal,
        positions: Dict[str, Position]
    ) -> Optional[TradeRecord]:
        """执行交易"""
        # 获取当前持仓
        key = f"{signal.exchange.value}:{signal.symbol}"
        position = positions.get(key)

        # 检查信号有效性
        if signal.signal_type in [SignalType.CLOSE_LONG, SignalType.CLOSE_SHORT]:
            if not position or position.volume == 0:
                return None
            if (
                signal.signal_type == SignalType.CLOSE_LONG
                and position.position_type != SignalType.LONG
            ):
                return None
            if (
                signal.signal_type == SignalType.CLOSE_SHORT
                and position.position_type != SignalType.SHORT
            ):
                return None

        # 计算交易价格（考虑滑点）
        if isinstance(market_data, (Kline, Ticker, Trade)):
            price = market_data.price
        else:
            # 对于订单簿数据，使用对手方最优价格
            if signal.signal_type in [SignalType.LONG, SignalType.CLOSE_SHORT]:
                price = market_data.asks[0].price
            else:
                price = market_data.bids[0].price

        if signal.signal_type in [SignalType.LONG, SignalType.CLOSE_SHORT]:
            price = price * (1 + config.slippage)
        else:
            price = price * (1 - config.slippage)

        # 计算交易数量
        if signal.volume:
            volume = signal.volume
        else:
            # 默认使用全部可用资金的10%
            available_balance = account_balance
            if position and position.volume > 0:
                available_balance += position.unrealized_pnl
            volume = (available_balance * Decimal("0.1")) / price

        # 计算手续费
        fee = price * volume * config.trading_fee

        # 检查资金是否足够
        if signal.signal_type in [SignalType.LONG, SignalType.SHORT]:
            if fee > account_balance:
                return None

        # 创建交易记录
        trade = TradeRecord(
            exchange=signal.exchange,
            symbol=signal.symbol,
            trade_type=signal.signal_type,
            order_time=signal.timestamp,
            fill_time=market_data.timestamp,
            status=TradeStatus.FILLED,
            price=price,
            volume=volume,
            fee=fee
        )

        # 计算收益
        if signal.signal_type in [SignalType.CLOSE_LONG, SignalType.CLOSE_SHORT]:
            if signal.signal_type == SignalType.CLOSE_LONG:
                trade.pnl = (price - position.avg_price) * volume
            else:
                trade.pnl = (position.avg_price - price) * volume
        else:
            trade.pnl = Decimal("0")

        return trade

    async def _calculate_metrics(
        self,
        config: BacktestConfig,
        trades: List[TradeRecord],
        equity_curve: List[Dict[str, Union[datetime, Decimal]]],
        drawdown_curve: List[Dict[str, Union[datetime, Decimal]]]
    ) -> PerformanceMetrics:
        """计算绩效指标"""
        metrics = PerformanceMetrics()

        if not trades:
            return metrics

        # 基础统计
        metrics.total_trades = len(trades)
        metrics.total_fees = sum(t.fee for t in trades)
        metrics.total_pnl = sum(t.pnl for t in trades)

        # 盈亏统计
        winning_trades = [t for t in trades if t.pnl > 0]
        losing_trades = [t for t in trades if t.pnl < 0]
        metrics.winning_trades = len(winning_trades)
        metrics.losing_trades = len(losing_trades)
        metrics.win_rate = (
            metrics.winning_trades / metrics.total_trades
            if metrics.total_trades > 0 else 0
        )

        if winning_trades:
            metrics.avg_winning_trade = (
                sum(t.pnl for t in winning_trades)
                / len(winning_trades)
            )
            metrics.largest_winning_trade = max(
                t.pnl for t in winning_trades
            )

        if losing_trades:
            metrics.avg_losing_trade = (
                sum(t.pnl for t in losing_trades)
                / len(losing_trades)
            )
            metrics.largest_losing_trade = min(
                t.pnl for t in losing_trades
            )

        metrics.avg_trade_pnl = (
            metrics.total_pnl / metrics.total_trades
            if metrics.total_trades > 0 else Decimal("0")
        )

        # 连续盈亏统计
        current_streak = 0
        max_win_streak = 0
        max_loss_streak = 0
        for trade in trades:
            if trade.pnl > 0:
                if current_streak > 0:
                    current_streak += 1
                else:
                    current_streak = 1
                max_win_streak = max(max_win_streak, current_streak)
            elif trade.pnl < 0:
                if current_streak < 0:
                    current_streak -= 1
                else:
                    current_streak = -1
                max_loss_streak = min(max_loss_streak, current_streak)
            else:
                current_streak = 0

        metrics.max_consecutive_wins = max_win_streak
        metrics.max_consecutive_losses = abs(max_loss_streak)

        # 持仓时间统计
        durations = [
            int((t.fill_time - t.order_time).total_seconds() / 60)
            for t in trades
        ]
        metrics.avg_trade_duration = (
            sum(durations) / len(durations)
            if durations else 0
        )

        # 回撤统计
        if drawdown_curve:
            max_drawdown = max(
                float(d["drawdown"])
                for d in drawdown_curve
            )
            metrics.max_drawdown = Decimal(str(max_drawdown))

            # 计算最大回撤持续时间
            current_drawdown_start = None
            max_drawdown_duration = 0
            current_duration = 0
            for i in range(len(drawdown_curve)):
                if float(drawdown_curve[i]["drawdown"]) > 0:
                    if current_drawdown_start is None:
                        current_drawdown_start = drawdown_curve[i]["timestamp"]
                    current_duration = int(
                        (
                            drawdown_curve[i]["timestamp"]
                            - current_drawdown_start
                        ).total_seconds() / 60
                    )
                    max_drawdown_duration = max(
                        max_drawdown_duration,
                        current_duration
                    )
                else:
                    current_drawdown_start = None
                    current_duration = 0
            metrics.max_drawdown_duration = max_drawdown_duration

        # 收益率统计
        if len(equity_curve) > 1:
            # 计算日收益率序列
            daily_returns = []
            current_day = equity_curve[0]["timestamp"].date()
            day_start_equity = float(equity_curve[0]["equity"])
            for i in range(1, len(equity_curve)):
                date = equity_curve[i]["timestamp"].date()
                if date != current_day:
                    day_end_equity = float(equity_curve[i-1]["equity"])
                    daily_return = (
                        day_end_equity - day_start_equity
                    ) / day_start_equity
                    daily_returns.append(daily_return)
                    current_day = date
                    day_start_equity = float(equity_curve[i]["equity"])

            if daily_returns:
                # 计算年化收益率
                total_days = len(daily_returns)
                annual_return = np.mean(daily_returns) * 252

                # 计算夏普比率
                if np.std(daily_returns) > 0:
                    metrics.sharpe_ratio = (
                        (annual_return - 0.02)  # 假设无风险利率为2%
                        / (np.std(daily_returns) * np.sqrt(252))
                    )

                # 计算索提诺比率
                downside_returns = [r for r in daily_returns if r < 0]
                if downside_returns and np.std(downside_returns) > 0:
                    metrics.sortino_ratio = (
                        (annual_return - 0.02)
                        / (np.std(downside_returns) * np.sqrt(252))
                    )

                # 计算卡玛比率
                if metrics.max_drawdown > 0:
                    metrics.calmar_ratio = (
                        annual_return
                        / float(metrics.max_drawdown)
                    )

        # 计算盈亏比
        total_profit = sum(t.pnl for t in winning_trades)
        total_loss = abs(sum(t.pnl for t in losing_trades))
        metrics.profit_factor = (
            float(total_profit / total_loss)
            if total_loss > 0 else 0
        )

        return metrics 