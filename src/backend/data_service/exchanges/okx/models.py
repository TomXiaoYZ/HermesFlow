"""
OKX数据模型转换器
"""
from typing import Dict, Any, List, Optional
from datetime import datetime
from decimal import Decimal

from ...common.models import (
    Market, Exchange, Trade, OrderBook, Kline, Ticker,
    OrderType, OrderSide, OrderStatus, FundingRate,
    ContractInfo, PositionInfo, ContractOrder
)

def parse_ticker(data: Dict[str, Any], market: Market = Market.SPOT) -> Ticker:
    """解析行情数据
    
    Args:
        data: 原始数据
        market: 市场类型
        
    Returns:
        Ticker: 行情数据对象
    """
    return Ticker(
        exchange=Exchange.OKX,
        market=market,
        symbol=data['instId'],
        price=Decimal(data['last']),
        volume=Decimal(data['vol24h']),
        amount=Decimal(data['volCcy24h']),
        timestamp=datetime.fromtimestamp(int(data['ts']) / 1000),
        bid_price=Decimal(data['bidPx']),
        bid_qty=Decimal(data['bidSz']),
        ask_price=Decimal(data['askPx']),
        ask_qty=Decimal(data['askSz']),
        open_price=Decimal(data['open24h']),
        high_price=Decimal(data['high24h']),
        low_price=Decimal(data['low24h']),
        close_price=Decimal(data['last'])
    )

def parse_order_book(data: Dict[str, Any], market: Market = Market.SPOT) -> OrderBook:
    """解析订单簿数据
    
    Args:
        data: 原始数据
        market: 市场类型
        
    Returns:
        OrderBook: 订单簿对象
    """
    return OrderBook(
        exchange=Exchange.OKX,
        market=market,
        symbol=data['instId'],
        timestamp=datetime.fromtimestamp(int(data['ts']) / 1000),
        bids=[{
            'price': Decimal(price),
            'quantity': Decimal(qty),
            'orders': int(count)
        } for price, qty, _, count in data['bids']],
        asks=[{
            'price': Decimal(price),
            'quantity': Decimal(qty),
            'orders': int(count)
        } for price, qty, _, count in data['asks']],
        update_id=int(data['ts'])
    )

def parse_trade(data: Dict[str, Any], market: Market = Market.SPOT) -> Trade:
    """解析成交记录
    
    Args:
        data: 原始数据
        market: 市场类型
        
    Returns:
        Trade: 成交记录对象
    """
    return Trade(
        exchange=Exchange.OKX,
        market=market,
        symbol=data['instId'],
        id=data['tradeId'],
        price=Decimal(data['px']),
        quantity=Decimal(data['sz']),
        amount=Decimal(data['px']) * Decimal(data['sz']),
        timestamp=datetime.fromtimestamp(int(data['ts']) / 1000),
        is_buyer_maker=data['side'] == 'buy',
        side=OrderSide.BUY if data['side'] == 'buy' else OrderSide.SELL
    )

def parse_kline(data: List[Any], symbol: str, interval: str, market: Market = Market.SPOT) -> Kline:
    """解析K线数据
    
    Args:
        data: 原始数据
        symbol: 交易对
        interval: K线间隔
        market: 市场类型
        
    Returns:
        Kline: K线对象
    """
    return Kline(
        exchange=Exchange.OKX,
        market=market,
        symbol=symbol,
        interval=interval,
        open_time=datetime.fromtimestamp(int(data[0]) / 1000),
        close_time=datetime.fromtimestamp(int(data[0]) / 1000 + get_interval_seconds(interval)),
        open_price=Decimal(data[1]),
        high_price=Decimal(data[2]),
        low_price=Decimal(data[3]),
        close_price=Decimal(data[4]),
        volume=Decimal(data[5]),
        amount=Decimal(data[6]),
        trades_count=int(data[7])
    )

def parse_contract_info(data: Dict[str, Any]) -> ContractInfo:
    """解析合约信息
    
    Args:
        data: 原始数据
        
    Returns:
        ContractInfo: 合约信息对象
    """
    return ContractInfo(
        exchange=Exchange.OKX,
        symbol=data['instId'],
        underlying=data['uly'],
        contract_type='perpetual',
        contract_size=Decimal(data['ctVal']),
        price_precision=int(data['tickSz'].find('1')),
        quantity_precision=int(data['lotSz'].find('1')),
        min_leverage=Decimal('1'),
        max_leverage=Decimal(data['lever']),
        maintenance_margin_rate=Decimal(data['maintMarginRatio']),
        max_price=Decimal(data['maxIsolatedLoan']),
        min_price=Decimal(data['minSz']),
        max_quantity=Decimal(data['maxSz']),
        min_quantity=Decimal(data['minSz']),
        max_amount=Decimal(data['maxTranInstCount']),
        min_amount=Decimal(data['minTranInstCount'])
    )

def parse_position(data: Dict[str, Any]) -> PositionInfo:
    """解析持仓信息
    
    Args:
        data: 原始数据
        
    Returns:
        PositionInfo: 持仓信息对象
    """
    return PositionInfo(
        exchange=Exchange.OKX,
        symbol=data['instId'],
        position_side='long' if data['posSide'] == 'long' else 'short',
        position_amount=Decimal(data['pos']),
        entry_price=Decimal(data['avgPx']),
        leverage=Decimal(data['lever']),
        unrealized_pnl=Decimal(data['upl']),
        margin_mode='isolated' if data['mgnMode'] == 'isolated' else 'cross',
        isolated_margin=Decimal(data['margin']) if data['mgnMode'] == 'isolated' else Decimal('0'),
        liquidation_price=Decimal(data['liqPx']),
        margin_ratio=Decimal(data['mgnRatio']),
        timestamp=datetime.fromtimestamp(int(data['cTime']) / 1000)
    )

def parse_funding_rate(data: Dict[str, Any]) -> FundingRate:
    """解析资金费率
    
    Args:
        data: 原始数据
        
    Returns:
        FundingRate: 资金费率对象
    """
    return FundingRate(
        exchange=Exchange.OKX,
        symbol=data['instId'],
        funding_rate=Decimal(data['fundingRate']),
        estimated_rate=Decimal(data['nextFundingRate']),
        next_funding_time=datetime.fromtimestamp(int(data['fundingTime']) / 1000),
        timestamp=datetime.fromtimestamp(int(data['fundingTime']) / 1000)
    )

def parse_order(data: Dict[str, Any], market: Market = Market.SPOT) -> ContractOrder:
    """解析订单信息
    
    Args:
        data: 原始数据
        market: 市场类型
        
    Returns:
        ContractOrder: 订单对象
    """
    return ContractOrder(
        exchange=Exchange.OKX,
        market=market,
        symbol=data['instId'],
        id=data['ordId'],
        client_order_id=data.get('clOrdId'),
        price=Decimal(data['px']) if data.get('px') else Decimal('0'),
        quantity=Decimal(data['sz']),
        executed_quantity=Decimal(data['accFillSz']),
        remaining_quantity=Decimal(data['sz']) - Decimal(data['accFillSz']),
        status=parse_order_status(data['state']),
        type=parse_order_type(data['ordType']),
        side=OrderSide.BUY if data['side'] == 'buy' else OrderSide.SELL,
        position_side=data.get('posSide'),
        created_at=datetime.fromtimestamp(int(data['cTime']) / 1000),
        updated_at=datetime.fromtimestamp(int(data['uTime']) / 1000)
    )

def parse_order_status(status: str) -> OrderStatus:
    """解析订单状态
    
    Args:
        status: 原始状态
        
    Returns:
        OrderStatus: 标准订单状态
    """
    status_map = {
        'live': OrderStatus.NEW,
        'partially_filled': OrderStatus.PARTIALLY_FILLED,
        'filled': OrderStatus.FILLED,
        'canceled': OrderStatus.CANCELED,
        'rejected': OrderStatus.REJECTED
    }
    return status_map.get(status, OrderStatus.UNKNOWN)

def parse_order_type(type: str) -> OrderType:
    """解析订单类型
    
    Args:
        type: 原始类型
        
    Returns:
        OrderType: 标准订单类型
    """
    type_map = {
        'limit': OrderType.LIMIT,
        'market': OrderType.MARKET,
        'post_only': OrderType.POST_ONLY,
        'fok': OrderType.FOK,
        'ioc': OrderType.IOC
    }
    return type_map.get(type, OrderType.UNKNOWN)

def get_interval_seconds(interval: str) -> int:
    """获取K线间隔的秒数
    
    Args:
        interval: K线间隔
        
    Returns:
        int: 间隔秒数
    """
    units = {
        'm': 60,
        'h': 3600,
        'd': 86400,
        'w': 604800,
        'M': 2592000
    }
    unit = interval[-1]
    number = int(interval[:-1])
    return number * units[unit] 