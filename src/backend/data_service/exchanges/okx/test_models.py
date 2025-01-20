"""
OKX数据模型转换器测试
"""
import pytest
from datetime import datetime
from decimal import Decimal

from ....common.models import (
    Market, Exchange, OrderType, OrderSide, OrderStatus
)
from .models import (
    parse_ticker, parse_order_book, parse_trade,
    parse_kline, parse_contract_info, parse_position,
    parse_funding_rate, parse_order, parse_order_status,
    parse_order_type, get_interval_seconds
)

def test_parse_ticker():
    """测试行情数据解析"""
    data = {
        "instId": "BTC-USDT",
        "last": "50000",
        "vol24h": "1000",
        "volCcy24h": "50000000",
        "ts": "1616679000000",
        "bidPx": "49999",
        "bidSz": "1",
        "askPx": "50001",
        "askSz": "1",
        "open24h": "49000",
        "high24h": "51000",
        "low24h": "48000"
    }
    
    ticker = parse_ticker(data)
    assert ticker.exchange == Exchange.OKX
    assert ticker.market == Market.SPOT
    assert ticker.symbol == "BTC-USDT"
    assert ticker.price == Decimal("50000")
    assert ticker.volume == Decimal("1000")
    assert ticker.amount == Decimal("50000000")
    assert isinstance(ticker.timestamp, datetime)
    assert ticker.bid_price == Decimal("49999")
    assert ticker.bid_qty == Decimal("1")
    assert ticker.ask_price == Decimal("50001")
    assert ticker.ask_qty == Decimal("1")
    assert ticker.open_price == Decimal("49000")
    assert ticker.high_price == Decimal("51000")
    assert ticker.low_price == Decimal("48000")
    assert ticker.close_price == Decimal("50000")

def test_parse_order_book():
    """测试订单簿数据解析"""
    data = {
        "instId": "BTC-USDT",
        "bids": [
            ["50000", "1", "0", "1"],
            ["49999", "2", "0", "1"]
        ],
        "asks": [
            ["50001", "1", "0", "1"],
            ["50002", "2", "0", "1"]
        ],
        "ts": "1616679000000"
    }
    
    order_book = parse_order_book(data)
    assert order_book.exchange == Exchange.OKX
    assert order_book.market == Market.SPOT
    assert order_book.symbol == "BTC-USDT"
    assert isinstance(order_book.timestamp, datetime)
    assert len(order_book.bids) == 2
    assert len(order_book.asks) == 2
    assert order_book.bids[0]['price'] == Decimal("50000")
    assert order_book.bids[0]['quantity'] == Decimal("1")
    assert order_book.bids[0]['orders'] == 1
    assert order_book.asks[0]['price'] == Decimal("50001")
    assert order_book.asks[0]['quantity'] == Decimal("1")
    assert order_book.asks[0]['orders'] == 1

def test_parse_trade():
    """测试成交记录解析"""
    data = {
        "instId": "BTC-USDT",
        "tradeId": "1",
        "px": "50000",
        "sz": "1",
        "side": "buy",
        "ts": "1616679000000"
    }
    
    trade = parse_trade(data)
    assert trade.exchange == Exchange.OKX
    assert trade.market == Market.SPOT
    assert trade.symbol == "BTC-USDT"
    assert trade.id == "1"
    assert trade.price == Decimal("50000")
    assert trade.quantity == Decimal("1")
    assert trade.amount == Decimal("50000")
    assert isinstance(trade.timestamp, datetime)
    assert trade.is_buyer_maker is True
    assert trade.side == OrderSide.BUY

def test_parse_kline():
    """测试K线数据解析"""
    data = ["1616679000000", "50000", "51000", "49000", "50500", "100", "5000000", "1000"]
    
    kline = parse_kline(data, "BTC-USDT", "1m")
    assert kline.exchange == Exchange.OKX
    assert kline.market == Market.SPOT
    assert kline.symbol == "BTC-USDT"
    assert kline.interval == "1m"
    assert isinstance(kline.open_time, datetime)
    assert isinstance(kline.close_time, datetime)
    assert kline.open_price == Decimal("50000")
    assert kline.high_price == Decimal("51000")
    assert kline.low_price == Decimal("49000")
    assert kline.close_price == Decimal("50500")
    assert kline.volume == Decimal("100")
    assert kline.amount == Decimal("5000000")
    assert kline.trades_count == 1000

def test_parse_contract_info():
    """测试合约信息解析"""
    data = {
        "instId": "BTC-USDT-SWAP",
        "uly": "BTC-USDT",
        "ctVal": "0.01",
        "tickSz": "0.1",
        "lotSz": "1",
        "lever": "100",
        "maintMarginRatio": "0.005",
        "maxIsolatedLoan": "1000000",
        "minSz": "1",
        "maxSz": "1000000",
        "maxTranInstCount": "100",
        "minTranInstCount": "1"
    }
    
    contract = parse_contract_info(data)
    assert contract.exchange == Exchange.OKX
    assert contract.symbol == "BTC-USDT-SWAP"
    assert contract.underlying == "BTC-USDT"
    assert contract.contract_type == "perpetual"
    assert contract.contract_size == Decimal("0.01")
    assert contract.price_precision == 1
    assert contract.quantity_precision == 0
    assert contract.min_leverage == Decimal("1")
    assert contract.max_leverage == Decimal("100")
    assert contract.maintenance_margin_rate == Decimal("0.005")
    assert contract.max_price == Decimal("1000000")
    assert contract.min_price == Decimal("1")
    assert contract.max_quantity == Decimal("1000000")
    assert contract.min_quantity == Decimal("1")
    assert contract.max_amount == Decimal("100")
    assert contract.min_amount == Decimal("1")

def test_parse_position():
    """测试持仓信息解析"""
    data = {
        "instId": "BTC-USDT-SWAP",
        "posSide": "long",
        "pos": "1",
        "avgPx": "50000",
        "lever": "10",
        "upl": "100",
        "mgnMode": "isolated",
        "margin": "5000",
        "liqPx": "45000",
        "mgnRatio": "0.1",
        "cTime": "1616679000000"
    }
    
    position = parse_position(data)
    assert position.exchange == Exchange.OKX
    assert position.symbol == "BTC-USDT-SWAP"
    assert position.position_side == "long"
    assert position.position_amount == Decimal("1")
    assert position.entry_price == Decimal("50000")
    assert position.leverage == Decimal("10")
    assert position.unrealized_pnl == Decimal("100")
    assert position.margin_mode == "isolated"
    assert position.isolated_margin == Decimal("5000")
    assert position.liquidation_price == Decimal("45000")
    assert position.margin_ratio == Decimal("0.1")
    assert isinstance(position.timestamp, datetime)

def test_parse_funding_rate():
    """测试资金费率解析"""
    data = {
        "instId": "BTC-USDT-SWAP",
        "fundingRate": "0.0001",
        "nextFundingRate": "0.0002",
        "fundingTime": "1616679000000"
    }
    
    funding = parse_funding_rate(data)
    assert funding.exchange == Exchange.OKX
    assert funding.symbol == "BTC-USDT-SWAP"
    assert funding.funding_rate == Decimal("0.0001")
    assert funding.estimated_rate == Decimal("0.0002")
    assert isinstance(funding.next_funding_time, datetime)
    assert isinstance(funding.timestamp, datetime)

def test_parse_order():
    """测试订单信息解析"""
    data = {
        "instId": "BTC-USDT",
        "ordId": "12345",
        "clOrdId": "test123",
        "px": "50000",
        "sz": "1",
        "accFillSz": "0.5",
        "state": "partially_filled",
        "ordType": "limit",
        "side": "buy",
        "posSide": "long",
        "cTime": "1616679000000",
        "uTime": "1616679000000"
    }
    
    order = parse_order(data)
    assert order.exchange == Exchange.OKX
    assert order.market == Market.SPOT
    assert order.symbol == "BTC-USDT"
    assert order.id == "12345"
    assert order.client_order_id == "test123"
    assert order.price == Decimal("50000")
    assert order.quantity == Decimal("1")
    assert order.executed_quantity == Decimal("0.5")
    assert order.remaining_quantity == Decimal("0.5")
    assert order.status == OrderStatus.PARTIALLY_FILLED
    assert order.type == OrderType.LIMIT
    assert order.side == OrderSide.BUY
    assert order.position_side == "long"
    assert isinstance(order.created_at, datetime)
    assert isinstance(order.updated_at, datetime)

def test_parse_order_status():
    """测试订单状态解析"""
    assert parse_order_status("live") == OrderStatus.NEW
    assert parse_order_status("partially_filled") == OrderStatus.PARTIALLY_FILLED
    assert parse_order_status("filled") == OrderStatus.FILLED
    assert parse_order_status("canceled") == OrderStatus.CANCELED
    assert parse_order_status("rejected") == OrderStatus.REJECTED
    assert parse_order_status("unknown") == OrderStatus.UNKNOWN

def test_parse_order_type():
    """测试订单类型解析"""
    assert parse_order_type("limit") == OrderType.LIMIT
    assert parse_order_type("market") == OrderType.MARKET
    assert parse_order_type("post_only") == OrderType.POST_ONLY
    assert parse_order_type("fok") == OrderType.FOK
    assert parse_order_type("ioc") == OrderType.IOC
    assert parse_order_type("unknown") == OrderType.UNKNOWN

def test_get_interval_seconds():
    """测试K线间隔转换"""
    assert get_interval_seconds("1m") == 60
    assert get_interval_seconds("1h") == 3600
    assert get_interval_seconds("1d") == 86400
    assert get_interval_seconds("1w") == 604800
    assert get_interval_seconds("1M") == 2592000 