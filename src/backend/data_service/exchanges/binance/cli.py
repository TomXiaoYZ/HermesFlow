"""
Binance API命令行工具
"""
import os
import sys
import asyncio
import argparse
from datetime import datetime, timedelta
from typing import Optional
from dotenv import load_dotenv

from src.backend.data_service.common.models import Market, OrderType, OrderSide
from src.backend.data_service.exchanges.binance.client import BinanceAPI

def parse_args():
    """解析命令行参数"""
    parser = argparse.ArgumentParser(description="Binance API命令行工具")
    parser.add_argument("--testnet", action="store_true", help="使用测试网络")
    
    subparsers = parser.add_subparsers(dest="command", help="命令")
    
    # 获取交易对信息
    symbols_parser = subparsers.add_parser("symbols", help="获取交易对信息")
    symbols_parser.add_argument("--market", type=str, default="spot", help="市场类型")
    
    # 获取行情数据
    ticker_parser = subparsers.add_parser("ticker", help="获取行情数据")
    ticker_parser.add_argument("--market", type=str, default="spot", help="市场类型")
    ticker_parser.add_argument("--symbol", type=str, required=True, help="交易对")
    
    # 获取订单簿数据
    orderbook_parser = subparsers.add_parser("orderbook", help="获取订单簿数据")
    orderbook_parser.add_argument("--market", type=str, default="spot", help="市场类型")
    orderbook_parser.add_argument("--symbol", type=str, required=True, help="交易对")
    orderbook_parser.add_argument("--limit", type=int, default=10, help="深度")
    
    # 获取最近成交
    trades_parser = subparsers.add_parser("trades", help="获取最近成交")
    trades_parser.add_argument("--market", type=str, default="spot", help="市场类型")
    trades_parser.add_argument("--symbol", type=str, required=True, help="交易对")
    trades_parser.add_argument("--limit", type=int, default=10, help="数量")
    
    # 获取K线数据
    klines_parser = subparsers.add_parser("klines", help="获取K线数据")
    klines_parser.add_argument("--market", type=str, default="spot", help="市场类型")
    klines_parser.add_argument("--symbol", type=str, required=True, help="交易对")
    klines_parser.add_argument("--interval", type=str, default="1h", help="时间间隔")
    klines_parser.add_argument("--limit", type=int, default=24, help="数量")
    
    # 获取账户余额
    balances_parser = subparsers.add_parser("balances", help="获取账户余额")
    
    # 创建订单
    order_parser = subparsers.add_parser("order", help="创建订单")
    order_parser.add_argument("--market", type=str, default="spot", help="市场类型")
    order_parser.add_argument("--symbol", type=str, required=True, help="交易对")
    order_parser.add_argument("--type", type=str, default="limit", help="订单类型")
    order_parser.add_argument("--side", type=str, required=True, help="订单方向")
    order_parser.add_argument("--price", type=float, help="价格")
    order_parser.add_argument("--quantity", type=float, required=True, help="数量")
    
    # 查询订单状态
    get_order_parser = subparsers.add_parser("get_order", help="查询订单状态")
    get_order_parser.add_argument("--market", type=str, default="spot", help="市场类型")
    get_order_parser.add_argument("--symbol", type=str, required=True, help="交易对")
    get_order_parser.add_argument("--order-id", type=str, help="订单ID")
    get_order_parser.add_argument("--client-order-id", type=str, help="客户端订单ID")
    
    # 取消订单
    cancel_order_parser = subparsers.add_parser("cancel_order", help="取消订单")
    cancel_order_parser.add_argument("--market", type=str, default="spot", help="市场类型")
    cancel_order_parser.add_argument("--symbol", type=str, required=True, help="交易对")
    cancel_order_parser.add_argument("--order-id", type=str, help="订单ID")
    cancel_order_parser.add_argument("--client-order-id", type=str, help="客户端订单ID")
    
    # 订阅命令
    subscribe_parser = subparsers.add_parser("subscribe", help="订阅WebSocket数据")
    subscribe_parser.add_argument("--type", choices=["order"], required=True, help="订阅类型")
    
    return parser.parse_args()

async def main():
    """主函数"""
    # 加载环境变量
    load_dotenv()
    
    # 获取API密钥
    api_key = os.getenv("BINANCE_API_KEY")
    api_secret = os.getenv("BINANCE_API_SECRET")
    
    if not api_key or not api_secret:
        print("错误: 请在.env文件中设置BINANCE_API_KEY和BINANCE_API_SECRET")
        return
    
    # 解析命令行参数
    args = parse_args()
    
    # 创建API客户端
    client = BinanceAPI(api_key, api_secret, args.testnet)
    
    try:
        if args.command == "symbols":
            # 获取交易对信息
            market = Market(args.market)
            symbols = await client.get_symbols(market)
            for symbol in symbols:
                print(f"交易对: {symbol.symbol}")
                print(f"  基础资产: {symbol.base_asset}")
                print(f"  计价资产: {symbol.quote_asset}")
                print(f"  最小价格: {symbol.min_price}")
                print(f"  最大价格: {symbol.max_price}")
                print(f"  价格精度: {symbol.tick_size}")
                print(f"  最小数量: {symbol.min_qty}")
                print(f"  最大数量: {symbol.max_qty}")
                print(f"  数量精度: {symbol.step_size}")
                print(f"  最小名义价值: {symbol.min_notional}")
                print(f"  状态: {symbol.status}")
                print()
        
        elif args.command == "ticker":
            # 获取行情数据
            market = Market(args.market)
            ticker = await client.get_ticker(market, args.symbol)
            print(f"交易对: {ticker.symbol}")
            print(f"最新价格: {ticker.price}")
            print(f"24h成交量: {ticker.volume}")
            print(f"24h成交额: {ticker.amount}")
            print(f"买一价: {ticker.bid_price}")
            print(f"买一量: {ticker.bid_qty}")
            print(f"卖一价: {ticker.ask_price}")
            print(f"卖一量: {ticker.ask_qty}")
            print(f"24h开盘价: {ticker.open_price}")
            print(f"24h最高价: {ticker.high_price}")
            print(f"24h最低价: {ticker.low_price}")
            print(f"24h收盘价: {ticker.close_price}")
            print(f"时间: {ticker.timestamp}")
        
        elif args.command == "orderbook":
            # 获取订单簿数据
            market = Market(args.market)
            order_book = await client.get_order_book(market, args.symbol, args.limit)
            print(f"交易对: {order_book.symbol}")
            print(f"时间: {order_book.timestamp}")
            print("\n买单:")
            for bid in order_book.bids:
                print(f"  价格: {bid['price']}, 数量: {bid['quantity']}")
            print("\n卖单:")
            for ask in order_book.asks:
                print(f"  价格: {ask['price']}, 数量: {ask['quantity']}")
        
        elif args.command == "trades":
            # 获取最近成交
            market = Market(args.market)
            trades = await client.get_recent_trades(market, args.symbol, args.limit)
            for trade in trades:
                print(f"成交ID: {trade.id}")
                print(f"  价格: {trade.price}")
                print(f"  数量: {trade.quantity}")
                print(f"  金额: {trade.amount}")
                print(f"  时间: {trade.timestamp}")
                print(f"  方向: {trade.side}")
                print()
        
        elif args.command == "klines":
            # 获取K线数据
            market = Market(args.market)
            end_time = datetime.now()
            start_time = end_time - timedelta(hours=args.limit)
            klines = await client.get_klines(
                market,
                args.symbol,
                args.interval,
                start_time=start_time,
                end_time=end_time,
                limit=args.limit
            )
            for kline in klines:
                print(f"时间: {kline.open_time} - {kline.close_time}")
                print(f"  开盘价: {kline.open_price}")
                print(f"  最高价: {kline.high_price}")
                print(f"  最低价: {kline.low_price}")
                print(f"  收盘价: {kline.close_price}")
                print(f"  成交量: {kline.volume}")
                print(f"  成交额: {kline.amount}")
                print(f"  成交笔数: {kline.trades_count}")
                print()
        
        elif args.command == "balances":
            # 获取账户余额
            balances = await client.get_balances()
            for balance in balances:
                if balance.total > 0:
                    print(f"资产: {balance.asset}")
                    print(f"  可用: {balance.free}")
                    print(f"  冻结: {balance.locked}")
                    print(f"  总额: {balance.total}")
                    print(f"  时间: {balance.timestamp}")
                    print()
        
        elif args.command == "order":
            # 创建订单
            market = Market(args.market)
            order_type = OrderType(args.type)
            side = OrderSide(args.side)
            
            if order_type == OrderType.LIMIT and not args.price:
                print("错误: 限价单需要指定价格")
                return
            
            order = await client.create_order(
                market,
                args.symbol,
                order_type,
                side,
                price=args.price,
                quantity=args.quantity
            )
            print(f"订单ID: {order.id}")
            print(f"客户端订单ID: {order.client_order_id}")
            print(f"交易对: {order.symbol}")
            print(f"类型: {order.type}")
            print(f"方向: {order.side}")
            print(f"价格: {order.price}")
            print(f"数量: {order.original_quantity}")
            print(f"已成交数量: {order.executed_quantity}")
            print(f"剩余数量: {order.remaining_quantity}")
            print(f"状态: {order.status}")
            print(f"创建时间: {order.created_at}")
            print(f"更新时间: {order.updated_at}")
        
        elif args.command == "get_order":
            # 查询订单状态
            market = Market(args.market)
            order = await client.get_order(
                market,
                args.symbol,
                order_id=args.order_id,
                client_order_id=args.client_order_id
            )
            print(f"订单ID: {order.id}")
            print(f"客户端订单ID: {order.client_order_id}")
            print(f"交易对: {order.symbol}")
            print(f"类型: {order.type}")
            print(f"方向: {order.side}")
            print(f"价格: {order.price}")
            print(f"数量: {order.original_quantity}")
            print(f"已成交数量: {order.executed_quantity}")
            print(f"剩余数量: {order.remaining_quantity}")
            print(f"状态: {order.status}")
            print(f"创建时间: {order.created_at}")
            print(f"更新时间: {order.updated_at}")
        
        elif args.command == "cancel_order":
            # 取消订单
            market = Market(args.market)
            order = await client.cancel_order(
                market,
                args.symbol,
                order_id=args.order_id,
                client_order_id=args.client_order_id
            )
            print(f"订单ID: {order.id}")
            print(f"客户端订单ID: {order.client_order_id}")
            print(f"交易对: {order.symbol}")
            print(f"类型: {order.type}")
            print(f"方向: {order.side}")
            print(f"价格: {order.price}")
            print(f"数量: {order.original_quantity}")
            print(f"已成交数量: {order.executed_quantity}")
            print(f"剩余数量: {order.remaining_quantity}")
            print(f"状态: {order.status}")
            print(f"创建时间: {order.created_at}")
            print(f"更新时间: {order.updated_at}")
        
        elif args.command == "subscribe":
            # 启动WebSocket客户端
            print(f"开始订阅{args.type}数据...")
            await client.start()
            
            # 保持运行直到用户中断
            try:
                while True:
                    await asyncio.sleep(1)
            except KeyboardInterrupt:
                print("\n停止订阅...")
                await client.stop()
    
    except Exception as e:
        print(f"错误: {str(e)}")
        sys.exit(1)

if __name__ == "__main__":
    asyncio.run(main()) 