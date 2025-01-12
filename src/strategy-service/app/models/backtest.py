"""
回测相关的数据模型
"""
from datetime import datetime
from decimal import Decimal
from enum import Enum
from typing import Dict, List, Optional, Set, Union

from pydantic import BaseModel, Field

from app.models.market_data import Exchange, Interval
from app.models.strategy import SignalType


class TradeStatus(str, Enum):
    """交易状态"""
    PENDING = "pending"  # 等待成交
    FILLED = "filled"  # 已成交
    CANCELLED = "cancelled"  # 已取消
    REJECTED = "rejected"  # 已拒绝


class TradeRecord(BaseModel):
    """交易记录"""
    exchange: Exchange  # 交易所
    symbol: str  # 交易对
    trade_type: SignalType  # 交易类型
    order_time: datetime  # 下单时间
    fill_time: datetime  # 成交时间
    status: TradeStatus  # 交易状态
    price: Decimal  # 成交价格
    volume: Decimal  # 成交数量
    fee: Decimal  # 手续费
    pnl: Decimal = Decimal("0")  # 收益


class Position(BaseModel):
    """持仓信息"""
    exchange: Exchange  # 交易所
    symbol: str  # 交易对
    position_type: SignalType  # 持仓类型
    volume: Decimal  # 持仓数量
    avg_price: Decimal  # 平均持仓价格
    unrealized_pnl: Decimal  # 未实现盈亏
    realized_pnl: Decimal  # 已实现盈亏
    open_time: datetime  # 开仓时间
    last_update_time: datetime  # 最后更新时间


class BacktestConfig(BaseModel):
    """回测配置"""
    exchanges: Set[Exchange]  # 交易所列表
    symbols: Set[str]  # 交易对列表
    intervals: Set[Interval]  # K线周期列表
    data_types: Set[str]  # 数据类型列表
    start_time: datetime  # 回测开始时间
    end_time: datetime  # 回测结束时间
    initial_capital: Decimal  # 初始资金
    trading_fee: Decimal  # 交易手续费率
    slippage: Decimal  # 滑点率


class PerformanceMetrics(BaseModel):
    """绩效指标"""
    # 基础统计
    total_trades: int = 0  # 总交易次数
    total_fees: Decimal = Decimal("0")  # 总手续费
    total_pnl: Decimal = Decimal("0")  # 总盈亏

    # 盈亏统计
    winning_trades: int = 0  # 盈利交易次数
    losing_trades: int = 0  # 亏损交易次数
    win_rate: float = 0  # 胜率
    avg_winning_trade: Decimal = Decimal("0")  # 平均盈利
    avg_losing_trade: Decimal = Decimal("0")  # 平均亏损
    largest_winning_trade: Decimal = Decimal("0")  # 最大单笔盈利
    largest_losing_trade: Decimal = Decimal("0")  # 最大单笔亏损
    avg_trade_pnl: Decimal = Decimal("0")  # 平均每笔盈亏

    # 连续盈亏统计
    max_consecutive_wins: int = 0  # 最大连续盈利次数
    max_consecutive_losses: int = 0  # 最大连续亏损次数

    # 持仓时间统计
    avg_trade_duration: float = 0  # 平均持仓时间(分钟)

    # 回撤统计
    max_drawdown: Decimal = Decimal("0")  # 最大回撤
    max_drawdown_duration: int = 0  # 最大回撤持续时间(分钟)

    # 收益率统计
    sharpe_ratio: float = 0  # 夏普比率
    sortino_ratio: float = 0  # 索提诺比率
    calmar_ratio: float = 0  # 卡玛比率
    profit_factor: float = 0  # 盈亏比


class BacktestResult(BaseModel):
    """回测结果"""
    config: BacktestConfig  # 回测配置
    trades: List[TradeRecord] = []  # 交易记录
    positions: Dict[str, Position] = {}  # 持仓信息
    metrics: Optional[PerformanceMetrics] = None  # 绩效指标
    equity_curve: List[Dict[str, Union[datetime, Decimal]]] = []  # 权益曲线
    drawdown_curve: List[Dict[str, Union[datetime, Decimal]]] = []  # 回撤曲线 