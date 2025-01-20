"""
Binance消息处理器单元测试
"""
import pytest
from decimal import Decimal
from datetime import datetime
from src.backend.data_service.exchanges.binance.handlers import (
    TickerHandler,
    TradeHandler,
    OrderBookHandler,
    KlineHandler,
    UserDataHandler
)
from src.backend.data_service.common.models import OrderSide, OrderStatus
from tests.common.exchange_test_base import BaseExchangeTest

class TestBinanceHandlers(BaseExchangeTest):
    """Binance消息处理器测试类"""
    
    def test_ticker_handler(self):
        """测试行情数据处理器"""
        handler = TickerHandler()
        data = {
            "e": "24hrTicker",
            "E": 1672502400000,
            "s": "BTCUSDT",
            "p": "100.0",
            "P": "1.0",
            "w": "20000.0",
            "c": "20100.0",
            "Q": "0.001",
            "o": "20000.0",
            "h": "20200.0",
            "l": "19800.0",
            "v": "100.0",
            "q": "2000000.0",
            "b": "20090.0",
            "B": "1.0",
            "a": "20110.0",
            "A": "1.0"
        }
        
        ticker = handler.handle(data)
        assert ticker.symbol == "BTCUSDT"
        assert ticker.last_price == Decimal("20100.0")
        assert ticker.volume == Decimal("100.0")
        assert ticker.amount == Decimal("2000000.0")
        assert ticker.bid_price == Decimal("20090.0")
        assert ticker.ask_price == Decimal("20110.0")
        assert isinstance(ticker.timestamp, datetime)
    
    def test_trade_handler(self):
        """测试成交数据处理器"""
        handler = TradeHandler()
        data = {
            "e": "trade",
            "E": 1672502400000,
            "s": "BTCUSDT",
            "t": 12345,
            "p": "20100.0",
            "q": "0.001",
            "b": 88888,
            "a": 99999,
            "T": 1672502400000,
            "m": True,
            "M": True
        }
        
        trade = handler.handle(data)
        assert trade.symbol == "BTCUSDT"
        assert trade.id == "12345"
        assert trade.price == Decimal("20100.0")
        assert trade.quantity == Decimal("0.001")
        assert trade.amount == Decimal("20.1")
        assert trade.buyer_order_id == "88888"
        assert trade.seller_order_id == "99999"
        assert isinstance(trade.timestamp, datetime)
        assert trade.is_buyer_maker is True
    
    def test_order_book_handler(self):
        """测试订单簿数据处理器"""
        handler = OrderBookHandler()
        data = {
            "lastUpdateId": 12345,
            "bids": [
                ["20090.0", "1.0"],
                ["20080.0", "2.0"]
            ],
            "asks": [
                ["20110.0", "1.0"],
                ["20120.0", "2.0"]
            ]
        }
        
        order_book = handler.handle(data)
        assert len(order_book.bids) == 2
        assert len(order_book.asks) == 2
        assert order_book.bids[0].price == Decimal("20090.0")
        assert order_book.bids[0].quantity == Decimal("1.0")
        assert order_book.asks[0].price == Decimal("20110.0")
        assert order_book.asks[0].quantity == Decimal("1.0")
        assert order_book.update_id == 12345
    
    def test_kline_handler(self):
        """测试K线数据处理器"""
        handler = KlineHandler()
        data = {
            "e": "kline",
            "E": 1672502400000,
            "s": "BTCUSDT",
            "k": {
                "t": 1672502400000,
                "T": 1672502459999,
                "s": "BTCUSDT",
                "i": "1m",
                "o": "20000.0",
                "c": "20100.0",
                "h": "20200.0",
                "l": "19800.0",
                "v": "100.0",
                "n": 100,
                "q": "2000000.0",
                "V": "50.0",
                "Q": "1000000.0",
                "B": "0"
            }
        }
        
        kline = handler.handle(data)
        assert kline.symbol == "BTCUSDT"
        assert kline.interval == "1m"
        assert kline.open_price == Decimal("20000.0")
        assert kline.close_price == Decimal("20100.0")
        assert kline.high_price == Decimal("20200.0")
        assert kline.low_price == Decimal("19800.0")
        assert kline.volume == Decimal("100.0")
        assert kline.amount == Decimal("2000000.0")
        assert kline.trades_count == 100
        assert isinstance(kline.open_time, datetime)
        assert isinstance(kline.close_time, datetime)
    
    def test_user_data_handler(self):
        """测试用户数据处理器"""
        handler = UserDataHandler()
        
        # 测试订单更新
        order_data = {
            "e": "executionReport",
            "E": 1672502400000,
            "s": "BTCUSDT",
            "c": "test",
            "S": "BUY",
            "o": "LIMIT",
            "f": "GTC",
            "q": "0.001",
            "p": "20000.0",
            "X": "NEW",
            "i": 12345,
            "l": "0.0",
            "z": "0.0",
            "L": "0.0",
            "n": "0.0",
            "N": "USDT",
            "T": 1672502400000,
            "t": 0,
            "b": "0.0",
            "a": "0.0",
            "m": False,
            "R": False,
            "wt": "CONTRACT_PRICE",
            "ot": "LIMIT",
            "ps": "BOTH",
            "cp": False,
            "rp": "0.0",
            "pP": False,
            "si": 0,
            "ss": 0
        }
        
        order = handler.handle_order(order_data)
        assert order.symbol == "BTCUSDT"
        assert order.side == OrderSide.BUY
        assert order.type == "LIMIT"
        assert order.quantity == Decimal("0.001")
        assert order.price == Decimal("20000.0")
        assert order.status == OrderStatus.NEW
        assert order.order_id == "12345"
        assert isinstance(order.timestamp, datetime)
        
        # 测试账户更新
        account_data = {
            "e": "outboundAccountPosition",
            "E": 1672502400000,
            "u": 1672502400000,
            "B": [
                {
                    "a": "BTC",
                    "f": "1.0",
                    "l": "0.5"
                },
                {
                    "a": "USDT",
                    "f": "10000.0",
                    "l": "5000.0"
                }
            ]
        }
        
        balances = handler.handle_account(account_data)
        assert len(balances) == 2
        assert balances[0].asset == "BTC"
        assert balances[0].free == Decimal("1.0")
        assert balances[0].locked == Decimal("0.5")
        assert balances[1].asset == "USDT"
        assert balances[1].free == Decimal("10000.0")
        assert balances[1].locked == Decimal("5000.0") 