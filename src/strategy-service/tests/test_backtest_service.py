"""
回测引擎单元测试
"""
import asyncio
from datetime import datetime, timedelta
from decimal import Decimal
from typing import Dict, List, Optional, Set, Union
from unittest.mock import AsyncMock, MagicMock, patch

import pytest
from pydantic import BaseModel

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
    OrderBookLevel,
    Ticker,
    Trade,
)
from app.models.strategy import BaseStrategy, Signal, SignalType
from app.services.backtest_service import BacktestService


class MockStrategy(BaseStrategy):
    """模拟策略"""

    def __init__(
        self,
        name: str,
        exchanges: Set[Exchange],
        symbols: Set[str],
        parameters: Dict[str, Union[str, int, float, bool, Dict, List]]
    ) -> None:
        """初始化策略"""
        super().__init__(name, exchanges, symbols, parameters)
        self.signal_counter = 0

    async def initialize(self) -> None:
        """初始化策略"""
        pass

    async def on_kline(self, kline: Kline) -> Optional[Signal]:
        """处理K线数据"""
        self.signal_counter += 1
        if self.signal_counter % 2 == 0:
            return Signal(
                exchange=kline.exchange,
                symbol=kline.symbol,
                signal_type=SignalType.LONG,
                timestamp=kline.timestamp,
                price=kline.close,
                volume=Decimal("1")
            )
        return None

    async def on_ticker(self, ticker: Ticker) -> Optional[Signal]:
        """处理Ticker数据"""
        return None

    async def on_orderbook(self, orderbook: OrderBook) -> Optional[Signal]:
        """处理订单簿数据"""
        return None

    async def on_trade(self, trade: Trade) -> Optional[Signal]:
        """处理成交记录"""
        return None


@pytest.fixture
async def backtest_service():
    """创建回测服务"""
    service = BacktestService()
    await service.initialize()
    yield service
    await service.close()


@pytest.fixture
def mock_clickhouse_service():
    """模拟ClickHouse服务"""
    with patch(
        "app.services.backtest_service.ClickHouseService"
    ) as mock_class:
        mock_instance = mock_class.return_value
        mock_instance.initialize = AsyncMock()
        mock_instance.close = AsyncMock()
        mock_instance.get_klines = AsyncMock(return_value=[
            Kline(
                exchange=Exchange.BINANCE,
                symbol="BTC/USDT",
                interval=Interval.MIN_1,
                timestamp=datetime(2024, 1, 1, 0, i),
                open=Decimal("40000"),
                high=Decimal("40100"),
                low=Decimal("39900"),
                close=Decimal("40050"),
                volume=Decimal("10"),
                turnover=Decimal("400500")
            )
            for i in range(10)
        ])
        mock_instance.get_tickers = AsyncMock(return_value=[
            Ticker(
                exchange=Exchange.BINANCE,
                symbol="BTC/USDT",
                timestamp=datetime(2024, 1, 1, 0, i),
                price=Decimal("40000"),
                volume=Decimal("10"),
                turnover=Decimal("400000")
            )
            for i in range(10)
        ])
        mock_instance.get_orderbooks = AsyncMock(return_value=[
            OrderBook(
                exchange=Exchange.BINANCE,
                symbol="BTC/USDT",
                timestamp=datetime(2024, 1, 1, 0, i),
                asks=[
                    OrderBookLevel(
                        price=Decimal("40100"),
                        volume=Decimal("1")
                    )
                ],
                bids=[
                    OrderBookLevel(
                        price=Decimal("39900"),
                        volume=Decimal("1")
                    )
                ]
            )
            for i in range(10)
        ])
        mock_instance.get_trades = AsyncMock(return_value=[
            Trade(
                exchange=Exchange.BINANCE,
                symbol="BTC/USDT",
                timestamp=datetime(2024, 1, 1, 0, i),
                price=Decimal("40000"),
                volume=Decimal("1"),
                side=SignalType.LONG
            )
            for i in range(10)
        ])
        yield mock_instance


@pytest.mark.asyncio
async def test_initialize(mock_clickhouse_service):
    """测试初始化"""
    service = BacktestService()
    await service.initialize()
    mock_clickhouse_service.initialize.assert_called_once()
    await service.close()
    mock_clickhouse_service.close.assert_called_once()


@pytest.mark.asyncio
async def test_load_historical_data(backtest_service, mock_clickhouse_service):
    """测试加载历史数据"""
    config = BacktestConfig(
        exchanges={Exchange.BINANCE},
        symbols={"BTC/USDT"},
        intervals={Interval.MIN_1},
        data_types={"kline", "ticker", "orderbook", "trade"},
        start_time=datetime(2024, 1, 1),
        end_time=datetime(2024, 1, 2),
        initial_capital=Decimal("10000"),
        trading_fee=Decimal("0.001"),
        slippage=Decimal("0.001")
    )

    queue = await backtest_service._load_historical_data(config)
    assert queue.qsize() == 40  # 4种数据类型 * 10条数据

    mock_clickhouse_service.get_klines.assert_called_once_with(
        exchange=Exchange.BINANCE,
        symbol="BTC/USDT",
        interval=Interval.MIN_1,
        start_time=datetime(2024, 1, 1),
        end_time=datetime(2024, 1, 2)
    )
    mock_clickhouse_service.get_tickers.assert_called_once_with(
        exchange=Exchange.BINANCE,
        symbol="BTC/USDT",
        start_time=datetime(2024, 1, 1),
        end_time=datetime(2024, 1, 2)
    )
    mock_clickhouse_service.get_orderbooks.assert_called_once_with(
        exchange=Exchange.BINANCE,
        symbol="BTC/USDT",
        start_time=datetime(2024, 1, 1),
        end_time=datetime(2024, 1, 2)
    )
    mock_clickhouse_service.get_trades.assert_called_once_with(
        exchange=Exchange.BINANCE,
        symbol="BTC/USDT",
        start_time=datetime(2024, 1, 1),
        end_time=datetime(2024, 1, 2)
    )


@pytest.mark.asyncio
async def test_execute_trade(backtest_service):
    """测试执行交易"""
    signal = Signal(
        exchange=Exchange.BINANCE,
        symbol="BTC/USDT",
        signal_type=SignalType.LONG,
        timestamp=datetime(2024, 1, 1),
        price=Decimal("40000"),
        volume=Decimal("1")
    )

    kline = Kline(
        exchange=Exchange.BINANCE,
        symbol="BTC/USDT",
        interval=Interval.MIN_1,
        timestamp=datetime(2024, 1, 1),
        open=Decimal("40000"),
        high=Decimal("40100"),
        low=Decimal("39900"),
        close=Decimal("40050"),
        volume=Decimal("10"),
        turnover=Decimal("400500")
    )

    config = BacktestConfig(
        exchanges={Exchange.BINANCE},
        symbols={"BTC/USDT"},
        intervals={Interval.MIN_1},
        data_types={"kline"},
        start_time=datetime(2024, 1, 1),
        end_time=datetime(2024, 1, 2),
        initial_capital=Decimal("10000"),
        trading_fee=Decimal("0.001"),
        slippage=Decimal("0.001")
    )

    account_balance = Decimal("10000")
    positions = {}

    trade = await backtest_service._execute_trade(
        signal,
        kline,
        config,
        account_balance,
        positions
    )

    assert trade is not None
    assert trade.exchange == Exchange.BINANCE
    assert trade.symbol == "BTC/USDT"
    assert trade.trade_type == SignalType.LONG
    assert trade.status == TradeStatus.FILLED
    assert trade.price == Decimal("40050") * (1 + config.slippage)
    assert trade.volume == Decimal("1")
    assert trade.fee == trade.price * trade.volume * config.trading_fee
    assert trade.pnl == Decimal("0")


@pytest.mark.asyncio
async def test_calculate_metrics(backtest_service):
    """测试计算绩效指标"""
    config = BacktestConfig(
        exchanges={Exchange.BINANCE},
        symbols={"BTC/USDT"},
        intervals={Interval.MIN_1},
        data_types={"kline"},
        start_time=datetime(2024, 1, 1),
        end_time=datetime(2024, 1, 2),
        initial_capital=Decimal("10000"),
        trading_fee=Decimal("0.001"),
        slippage=Decimal("0.001")
    )

    trades = [
        TradeRecord(
            exchange=Exchange.BINANCE,
            symbol="BTC/USDT",
            trade_type=SignalType.LONG,
            order_time=datetime(2024, 1, 1, 0, i),
            fill_time=datetime(2024, 1, 1, 0, i),
            status=TradeStatus.FILLED,
            price=Decimal("40000"),
            volume=Decimal("1"),
            fee=Decimal("40"),
            pnl=Decimal("100") if i % 2 == 0 else Decimal("-50")
        )
        for i in range(10)
    ]

    equity_curve = [
        {
            "timestamp": datetime(2024, 1, 1, 0, i),
            "equity": Decimal("10000") + Decimal(str(i * 100))
        }
        for i in range(10)
    ]

    drawdown_curve = [
        {
            "timestamp": datetime(2024, 1, 1, 0, i),
            "drawdown": Decimal("0.01") * i
        }
        for i in range(10)
    ]

    metrics = await backtest_service._calculate_metrics(
        config,
        trades,
        equity_curve,
        drawdown_curve
    )

    assert metrics.total_trades == 10
    assert metrics.total_fees == Decimal("400")
    assert metrics.total_pnl == Decimal("250")
    assert metrics.winning_trades == 5
    assert metrics.losing_trades == 5
    assert metrics.win_rate == 0.5
    assert metrics.avg_winning_trade == Decimal("100")
    assert metrics.avg_losing_trade == Decimal("-50")
    assert metrics.largest_winning_trade == Decimal("100")
    assert metrics.largest_losing_trade == Decimal("-50")
    assert metrics.avg_trade_pnl == Decimal("25")
    assert metrics.max_consecutive_wins == 1
    assert metrics.max_consecutive_losses == 1
    assert metrics.avg_trade_duration == 0
    assert metrics.max_drawdown == Decimal("0.09")
    assert metrics.profit_factor == 2.0


@pytest.mark.asyncio
async def test_run_backtest(backtest_service, mock_clickhouse_service):
    """测试运行回测"""
    config = BacktestConfig(
        exchanges={Exchange.BINANCE},
        symbols={"BTC/USDT"},
        intervals={Interval.MIN_1},
        data_types={"kline"},
        start_time=datetime(2024, 1, 1),
        end_time=datetime(2024, 1, 2),
        initial_capital=Decimal("10000"),
        trading_fee=Decimal("0.001"),
        slippage=Decimal("0.001")
    )

    result = await backtest_service.run_backtest(MockStrategy, config)

    assert result.config == config
    assert len(result.trades) == 5  # 10条K线数据，每2条生成1个信号
    assert result.metrics is not None
    assert len(result.equity_curve) == 10
    assert len(result.drawdown_curve) > 0 