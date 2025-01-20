"""
订单相关枚举类型
"""
from enum import Enum

class OrderType(str, Enum):
    """订单类型"""
    MARKET = "market"  # 市价单
    LIMIT = "limit"  # 限价单
    STOP = "stop"  # 止损单
    STOP_MARKET = "stop_market"  # 市价止损单
    TAKE_PROFIT = "take_profit"  # 止盈单
    TAKE_PROFIT_MARKET = "take_profit_market"  # 市价止盈单

class OrderSide(str, Enum):
    """订单方向"""
    BUY = "buy"  # 买入
    SELL = "sell"  # 卖出

class OrderStatus(str, Enum):
    """订单状态"""
    NEW = "new"  # 新建订单
    PARTIALLY_FILLED = "partially_filled"  # 部分成交
    FILLED = "filled"  # 完全成交
    CANCELED = "canceled"  # 已取消
    REJECTED = "rejected"  # 已拒绝
    EXPIRED = "expired"  # 已过期

class TimeInForce(str, Enum):
    """订单有效期"""
    GTC = "gtc"  # Good Till Cancel 成交为止
    IOC = "ioc"  # Immediate or Cancel 立即成交或取消
    FOK = "fok"  # Fill or Kill 完全成交或取消
    GTX = "gtx"  # Good Till Crossing 无法成为挂单方就撤销

class OrderUpdateType(str, Enum):
    """订单更新类型"""
    STATUS = "status"  # 状态更新
    EXECUTION = "execution"  # 成交更新
    CANCEL = "cancel"  # 取消更新
    EXPIRE = "expire"  # 过期更新
    REJECT = "reject"  # 拒绝更新 