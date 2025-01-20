"""
基础数据模型
"""
from enum import Enum
from dataclasses import dataclass
from datetime import datetime
from decimal import Decimal
from typing import Optional, Dict, Any

class Market(str, Enum):
    """市场类型"""
    SPOT = "spot"  # 现货
    FUTURES = "futures"  # 合约
    MARGIN = "margin"  # 杠杆
    OPTIONS = "options"  # 期权

class OrderStatus(str, Enum):
    """订单状态"""
    NEW = "new"  # 新建订单
    PARTIALLY_FILLED = "partially_filled"  # 部分成交
    FILLED = "filled"  # 完全成交
    CANCELED = "canceled"  # 已取消
    REJECTED = "rejected"  # 已拒绝
    EXPIRED = "expired"  # 已过期

class OrderSide(str, Enum):
    """订单方向"""
    BUY = "buy"  # 买入
    SELL = "sell"  # 卖出

class OrderType(str, Enum):
    """订单类型"""
    MARKET = "market"  # 市价单
    LIMIT = "limit"  # 限价单
    STOP = "stop"  # 止损单
    STOP_MARKET = "stop_market"  # 市价止损单
    TAKE_PROFIT = "take_profit"  # 止盈单
    TAKE_PROFIT_MARKET = "take_profit_market"  # 市价止盈单

class TimeInForce(str, Enum):
    """订单有效期"""
    GTC = "gtc"  # Good Till Cancel 成交为止
    IOC = "ioc"  # Immediate or Cancel 立即成交或取消
    FOK = "fok"  # Fill or Kill 完全成交或取消
    GTX = "gtx"  # Good Till Crossing 无法成为挂单方就撤销

class PositionSide(str, Enum):
    """持仓方向"""
    LONG = "long"  # 多头
    SHORT = "short"  # 空头
    BOTH = "both"  # 双向

class MarginType(str, Enum):
    """保证金类型"""
    ISOLATED = "isolated"  # 逐仓
    CROSS = "cross"  # 全仓

@dataclass
class Symbol:
    """交易对信息"""
    exchange: str  # 交易所
    market: Market  # 市场类型
    base_asset: str  # 基础资产
    quote_asset: str  # 计价资产
    status: str  # 状态
    min_price: Decimal  # 最小价格
    max_price: Decimal  # 最大价格
    tick_size: Decimal  # 价格精度
    min_qty: Decimal  # 最小数量
    max_qty: Decimal  # 最大数量
    step_size: Decimal  # 数量精度
    min_notional: Decimal  # 最小交易额

@dataclass
class Ticker:
    """行情数据"""
    exchange: str  # 交易所
    market: Market  # 市场类型
    symbol: str  # 交易对
    price: Decimal  # 最新价
    volume: Decimal  # 成交量
    amount: Decimal  # 成交额
    timestamp: datetime  # 时间戳
    bid_price: Decimal  # 买一价
    bid_qty: Decimal  # 买一量
    ask_price: Decimal  # 卖一价
    ask_qty: Decimal  # 卖一量
    open_price: Decimal  # 开盘价
    high_price: Decimal  # 最高价
    low_price: Decimal  # 最低价
    close_price: Decimal  # 收盘价

@dataclass
class Trade:
    """成交记录"""
    exchange: str  # 交易所
    market: Market  # 市场类型
    symbol: str  # 交易对
    id: str  # 成交ID
    price: Decimal  # 成交价格
    quantity: Decimal  # 成交数量
    amount: Decimal  # 成交金额
    side: OrderSide  # 成交方向
    timestamp: datetime  # 成交时间

@dataclass
class OrderBook:
    """订单簿"""
    exchange: str  # 交易所
    market: Market  # 市场类型
    symbol: str  # 交易对
    timestamp: datetime  # 时间戳
    bids: Dict[Decimal, Decimal]  # 买盘 {价格: 数量}
    asks: Dict[Decimal, Decimal]  # 卖盘 {价格: 数量}

@dataclass
class Kline:
    """K线数据"""
    exchange: str  # 交易所
    market: Market  # 市场类型
    symbol: str  # 交易对
    interval: str  # 时间间隔
    open_time: datetime  # 开盘时间
    close_time: datetime  # 收盘时间
    open_price: Decimal  # 开盘价
    high_price: Decimal  # 最高价
    low_price: Decimal  # 最低价
    close_price: Decimal  # 收盘价
    volume: Decimal  # 成交量
    amount: Decimal  # 成交额
    trades_count: int  # 成交笔数

@dataclass
class ContractInfo:
    """合约信息"""
    exchange: str  # 交易所
    symbol: str  # 交易对
    status: str  # 状态
    base_asset: str  # 基础资产
    quote_asset: str  # 计价资产
    margin_asset: str  # 保证金资产
    price_precision: int  # 价格精度
    quantity_precision: int  # 数量精度
    min_leverage: int  # 最小杠杆
    max_leverage: int  # 最大杠杆
    min_price: Decimal  # 最小价格
    max_price: Decimal  # 最大价格
    tick_size: Decimal  # 价格步长
    min_qty: Decimal  # 最小数量
    max_qty: Decimal  # 最大数量
    step_size: Decimal  # 数量步长
    min_notional: Decimal  # 最小名义价值
    maintenance_margin_rate: Decimal  # 维持保证金率
    required_margin_rate: Decimal  # 初始保证金率

@dataclass
class FundingRate:
    """资金费率"""
    exchange: str  # 交易所
    symbol: str  # 交易对
    funding_rate: Decimal  # 当前资金费率
    estimated_rate: Decimal  # 预测资金费率
    next_funding_time: datetime  # 下次结算时间

@dataclass
class ContractOrder:
    """合约订单"""
    exchange: str  # 交易所
    symbol: str  # 交易对
    order_id: str  # 订单ID
    client_order_id: Optional[str]  # 客户端订单ID
    price: Decimal  # 价格
    quantity: Decimal  # 数量
    executed_qty: Decimal  # 已成交数量
    executed_price: Optional[Decimal]  # 成交均价
    side: OrderSide  # 订单方向
    position_side: PositionSide  # 持仓方向
    type: OrderType  # 订单类型
    status: OrderStatus  # 订单状态
    time_in_force: TimeInForce  # 有效期
    margin_type: MarginType  # 保证金类型
    leverage: int  # 杠杆倍数
    stop_price: Optional[Decimal]  # 触发价格
    timestamp: datetime  # 创建时间
    update_time: datetime  # 更新时间

@dataclass
class PositionInfo:
    """持仓信息"""
    exchange: str  # 交易所
    symbol: str  # 交易对
    position_side: PositionSide  # 持仓方向
    margin_type: MarginType  # 保证金类型
    leverage: int  # 杠杆倍数
    quantity: Decimal  # 持仓数量
    entry_price: Decimal  # 开仓均价
    mark_price: Decimal  # 标记价格
    unrealized_pnl: Decimal  # 未实现盈亏
    margin: Decimal  # 保证金
    maintenance_margin: Decimal  # 维持保证金
    timestamp: datetime  # 更新时间 