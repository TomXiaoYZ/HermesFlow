"""
基础数据模型
"""
from enum import Enum, auto

class ExchangeType(Enum):
    """交易所类型"""
    CRYPTO = "CRYPTO"  # 加密货币交易所
    STOCK_US = "STOCK_US"  # 美股
    STOCK_CN = "STOCK_CN"  # A股
    STOCK_HK = "STOCK_HK"  # 港股

class Exchange(Enum):
    """交易所"""
    # 加密货币交易所
    BINANCE = "BINANCE"
    OKX = "OKX"
    BITGET = "BITGET"
    
    # 美股券商
    IBKR = "IBKR"  # 盈透证券
    TD = "TD"  # TD Ameritrade
    
    # A股券商
    HUATAI = "HUATAI"  # 华泰证券
    CITICS = "CITICS"  # 中信证券
    
    # 港股券商
    FUTU = "FUTU"  # 富途证券
    TIGER = "TIGER"  # 老虎证券

class Market(Enum):
    """市场类型"""
    # 加密货币市场
    SPOT = "SPOT"  # 现货
    MARGIN = "MARGIN"  # 杠杆
    FUTURES = "FUTURES"  # 合约
    OPTIONS = "OPTIONS"  # 期权
    
    # 股票市场
    STOCK = "STOCK"  # 股票
    STOCK_MARGIN = "STOCK_MARGIN"  # 融资融券
    STOCK_OPTIONS = "STOCK_OPTIONS"  # 股票期权
    INDEX_FUTURES = "INDEX_FUTURES"  # 股指期货
    
    # 其他市场
    BONDS = "BONDS"  # 债券
    FUNDS = "FUNDS"  # 基金

class ProductType(Enum):
    """产品类型"""
    # 基础产品
    STOCK = "STOCK"  # 股票
    CRYPTO = "CRYPTO"  # 加密货币
    BOND = "BOND"  # 债券
    FUND = "FUND"  # 基金
    
    # 衍生品
    FUTURES = "FUTURES"  # 期货
    OPTION = "OPTION"  # 期权
    WARRANT = "WARRANT"  # 权证
    
    # 组合产品
    PORTFOLIO = "PORTFOLIO"  # 投资组合
    STRATEGY = "STRATEGY"  # 策略组合

class OrderType(Enum):
    """订单类型"""
    # 基础订单类型
    MARKET = "MARKET"  # 市价单
    LIMIT = "LIMIT"  # 限价单
    
    # 高级订单类型
    STOP = "STOP"  # 止损单
    STOP_LIMIT = "STOP_LIMIT"  # 止损限价单
    TRAILING_STOP = "TRAILING_STOP"  # 追踪止损单
    
    # 算法订单
    TWAP = "TWAP"  # 时间加权平均价格
    VWAP = "VWAP"  # 成交量加权平均价格
    ICEBERG = "ICEBERG"  # 冰山单
    
    # 组合订单
    OCO = "OCO"  # One-Cancels-Other
    BRACKET = "BRACKET"  # 括号单

class OrderSide(Enum):
    """订单方向"""
    BUY = "BUY"  # 买入
    SELL = "SELL"  # 卖出
    
    # 融资融券
    MARGIN_BUY = "MARGIN_BUY"  # 融资买入
    SHORT_SELL = "SHORT_SELL"  # 融券卖出
    
    # 期权特有
    BUY_TO_OPEN = "BUY_TO_OPEN"  # 买入开仓
    SELL_TO_CLOSE = "SELL_TO_CLOSE"  # 卖出平仓
    SELL_TO_OPEN = "SELL_TO_OPEN"  # 卖出开仓
    BUY_TO_CLOSE = "BUY_TO_CLOSE"  # 买入平仓

class OrderStatus(Enum):
    """订单状态"""
    # 基础状态
    NEW = "NEW"  # 新建
    PARTIALLY_FILLED = "PARTIALLY_FILLED"  # 部分成交
    FILLED = "FILLED"  # 完全成交
    CANCELED = "CANCELED"  # 已撤销
    REJECTED = "REJECTED"  # 已拒绝
    EXPIRED = "EXPIRED"  # 已过期
    
    # 中间状态
    PENDING_NEW = "PENDING_NEW"  # 待提交
    PENDING_CANCEL = "PENDING_CANCEL"  # 待撤销
    
    # 特殊状态
    SUSPENDED = "SUSPENDED"  # 已暂停
    TRIGGERED = "TRIGGERED"  # 已触发

class TimeInForce(Enum):
    """订单有效期"""
    GTC = "GTC"  # Good Till Cancel
    IOC = "IOC"  # Immediate or Cancel
    FOK = "FOK"  # Fill or Kill
    GTD = "GTD"  # Good Till Date
    DAY = "DAY"  # 当日有效

class PositionSide(Enum):
    """持仓方向"""
    LONG = "LONG"  # 多头
    SHORT = "SHORT"  # 空头
    BOTH = "BOTH"  # 双向持仓

class AccountType(Enum):
    """账户类型"""
    # 基础账户
    SPOT = "SPOT"  # 现货账户
    MARGIN = "MARGIN"  # 保证金账户
    FUTURES = "FUTURES"  # 期货账户
    OPTIONS = "OPTIONS"  # 期权账户
    
    # 特殊账户
    PORTFOLIO = "PORTFOLIO"  # 组合账户
    FUND = "FUND"  # 基金账户
    TRUST = "TRUST"  # 信托账户
    
    # 其他
    DEMO = "DEMO"  # 模拟账户
    SUB = "SUB"  # 子账户 