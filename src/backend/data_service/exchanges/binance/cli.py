"""
Binance命令行工具
"""
import os
import asyncio
import argparse
from decimal import Decimal
from datetime import datetime, timedelta
from dotenv import load_dotenv

from ...common.models import Market, OrderType, OrderSide
from .client import BinanceAPI
from .config import BINANCE_KLINE_INTERVALS, BINANCE_WS_TOPICS
from .handlers import MarketDataHandler

async def main():
    """主函数"""
    # 加载环境变量
    load_dotenv()
    
    # 创建参数解析器
    parser = argparse.ArgumentParser(description="Binance命令行工具")
    parser.add_argument("--testnet", action="store_true", help="使用测试网")
    
    # 创建子命令解析器
    subparsers = parser.add_subparsers(dest="command", help="子命令")
    
    # symbols命令
    symbols_parser = subparsers.add_parser("symbols", help="获取交易对信息")
    symbols_parser.add_argument("--market", choices=[m.value for m in Market], default=Market.SPOT.value, help="市场类型")
    
    # ticker命令
    ticker_parser = subparsers.add_parser("ticker", help="获取行情数据")
    ticker_parser.add_argument("--market", choices=[m.value for m in Market], default=Market.SPOT.value, help="市场类型")
    ticker_parser.add_argument("--symbol", required=True, help="交易对")
    
    # orderbook命令
    orderbook_parser = subparsers.add_parser("orderbook", help="获取订单簿数据")
    orderbook_parser.add_argument("--market", choices=[m.value for m in Market], default=Market.SPOT.value, help="市场类型")
    orderbook_parser.add_argument("--symbol", required=True, help="交易对")
    orderbook_parser.add_argument("--limit", type=int, default=10, help="深度")
    
    # trades命令
    trades_parser = subparsers.add_parser("trades", help="获取最近成交")
    trades_parser.add_argument("--market", choices=[m.value for m in Market], default=Market.SPOT.value, help="市场类型")
    trades_parser.add_argument("--symbol", required=True, help="交易对")
    trades_parser.add_argument("--limit", type=int, default=10, help="数量")
    
    # klines命令
    klines_parser = subparsers.add_parser("klines", help="获取K线数据")
    klines_parser.add_argument("--market", choices=[m.value for m in Market], default=Market.SPOT.value, help="市场类型")
    klines_parser.add_argument("--symbol", required=True, help="交易对")
    klines_parser.add_argument("--interval", choices=BINANCE_KLINE_INTERVALS, default="1h", help="K线间隔")
    klines_parser.add_argument("--limit", type=int, default=100, help="数量")
    klines_parser.add_argument("--start-time", type=str, help="开始时间 (YYYY-MM-DD HH:MM:SS)")
    klines_parser.add_argument("--end-time", type=str, help="结束时间 (YYYY-MM-DD HH:MM:SS)")
    
    # balances命令
    balances_parser = subparsers.add_parser("balances", help="获取账户余额")
    
    # order命令
    order_parser = subparsers.add_parser("order", help="下单")
    order_parser.add_argument("--market", choices=[m.value for m in Market], default=Market.SPOT.value, help="市场类型")
    order_parser.add_argument("--symbol", required=True, help="交易对")
    order_parser.add_argument("--type", choices=["limit", "market"], required=True, help="订单类型")
    order_parser.add_argument("--side", choices=["buy", "sell"], required=True, help="订单方向")
    order_parser.add_argument("--price", type=float, help="价格")
    order_parser.add_argument("--quantity", type=float, required=True, help="数量")
    
    # cancel_order命令
    cancel_order_parser = subparsers.add_parser("cancel_order", help="取消订单")
    cancel_order_parser.add_argument("--market", choices=[m.value for m in Market], default=Market.SPOT.value, help="市场类型")
    cancel_order_parser.add_argument("--symbol", required=True, help="交易对")
    cancel_order_parser.add_argument("--order-id", help="订单ID")
    cancel_order_parser.add_argument("--client-order-id", help="客户端订单ID")
    
    # get_order命令
    get_order_parser = subparsers.add_parser("get_order", help="获取订单信息")
    get_order_parser.add_argument("--market", choices=[m.value for m in Market], default=Market.SPOT.value, help="市场类型")
    get_order_parser.add_argument("--symbol", required=True, help="交易对")
    get_order_parser.add_argument("--order-id", help="订单ID")
    get_order_parser.add_argument("--client-order-id", help="客户端订单ID")
    
    # subscribe命令
    subscribe_parser = subparsers.add_parser("subscribe", help="订阅数据")
    subscribe_parser.add_argument("--type", choices=["trade", "ticker", "depth", "kline", "order"], required=True, help="订阅类型")
    subscribe_parser.add_argument("--symbol", help="交易对")
    subscribe_parser.add_argument("--interval", choices=BINANCE_KLINE_INTERVALS, help="K线间隔")
    subscribe_parser.add_argument("--depth", type=int, default=20, help="深度")
    
    # 解析命令行参数
    args = parser.parse_args()
    
    # 获取API密钥
    api_key = os.getenv("BINANCE_API_KEY", "")
    api_secret = os.getenv("BINANCE_API_SECRET", "")
    
    # 创建API客户端
    client = BinanceAPI(api_key, api_secret, args.testnet)
    
    try:
        if args.command == "symbols":
            market = Market(args.market)
            symbols = await client.get_symbols(market)
            for symbol in symbols:
                print(f"交易对: {symbol.symbol}")
                print(f"基础资产: {symbol.base_asset}")
                print(f"计价资产: {symbol.quote_asset}")
                print(f"状态: {symbol.status}")
                print(f"最小价格: {symbol.min_price}")
                print(f"最大价格: {symbol.max_price}")
                print(f"价格精度: {symbol.tick_size}")
                print(f"最小数量: {symbol.min_qty}")
                print(f"最大数量: {symbol.max_qty}")
                print(f"数量精度: {symbol.step_size}")
                print(f"最小名义价值: {symbol.min_notional}")
                print()
        
        elif args.command == "ticker":
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
            print(f"开盘价: {ticker.open_price}")
            print(f"最高价: {ticker.high_price}")
            print(f"最低价: {ticker.low_price}")
            print(f"收盘价: {ticker.close_price}")
            print(f"时间: {ticker.timestamp}")
        
        elif args.command == "orderbook":
            market = Market(args.market)
            order_book = await client.get_order_book(market, args.symbol, args.limit)
            print(f"交易对: {order_book.symbol}")
            print(f"更新ID: {order_book.update_id}")
            print(f"时间: {order_book.timestamp}")
            print("\n买盘:")
            for bid in order_book.bids:
                print(f"价格: {bid['price']}, 数量: {bid['quantity']}")
            print("\n卖盘:")
            for ask in order_book.asks:
                print(f"价格: {ask['price']}, 数量: {ask['quantity']}")
        
        elif args.command == "trades":
            market = Market(args.market)
            trades = await client.get_recent_trades(market, args.symbol, args.limit)
            for trade in trades:
                print(f"ID: {trade.id}")
                print(f"价格: {trade.price}")
                print(f"数量: {trade.quantity}")
                print(f"成交额: {trade.amount}")
                print(f"时间: {trade.timestamp}")
                print(f"买方是否是挂单方: {trade.is_buyer_maker}")
                print(f"方向: {trade.side}")
                print()
        
        elif args.command == "klines":
            market = Market(args.market)
            start_time = datetime.strptime(args.start_time, "%Y-%m-%d %H:%M:%S") if args.start_time else None
            end_time = datetime.strptime(args.end_time, "%Y-%m-%d %H:%M:%S") if args.end_time else None
            
            klines = await client.get_klines(
                market,
                args.symbol,
                args.interval,
                start_time=start_time,
                end_time=end_time,
                limit=args.limit
            )
            
            for kline in klines:
                print(f"开盘时间: {kline.open_time}")
                print(f"收盘时间: {kline.close_time}")
                print(f"开盘价: {kline.open_price}")
                print(f"最高价: {kline.high_price}")
                print(f"最低价: {kline.low_price}")
                print(f"收盘价: {kline.close_price}")
                print(f"成交量: {kline.volume}")
                print(f"成交额: {kline.amount}")
                print(f"成交笔数: {kline.trades_count}")
                print()
        
        elif args.command == "balances":
            if not api_key or not api_secret:
                print("错误: 请在.env文件中设置BINANCE_API_KEY和BINANCE_API_SECRET")
                return
            
            balances = await client.get_balances()
            for balance in balances:
                if balance.free > 0 or balance.locked > 0:
                    print(f"资产: {balance.asset}")
                    print(f"可用数量: {balance.free}")
                    print(f"冻结数量: {balance.locked}")
                    print(f"总数量: {balance.total}")
                    print(f"时间: {balance.timestamp}")
                    print()
        
        elif args.command == "order":
            if not api_key or not api_secret:
                print("错误: 请在.env文件中设置BINANCE_API_KEY和BINANCE_API_SECRET")
                return
            
            market = Market(args.market)
            order_type = OrderType.LIMIT if args.type == "limit" else OrderType.MARKET
            side = OrderSide.BUY if args.side == "buy" else OrderSide.SELL
            
            if order_type == OrderType.LIMIT and not args.price:
                print("错误: 限价单需要指定价格")
                return
            
            order = await client.create_order(
                market,
                args.symbol,
                order_type,
                side,
                price=Decimal(str(args.price)) if args.price else None,
                quantity=Decimal(str(args.quantity))
            )
            
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
            if not api_key or not api_secret:
                print("错误: 请在.env文件中设置BINANCE_API_KEY和BINANCE_API_SECRET")
                return
            
            if not args.order_id and not args.client_order_id:
                print("错误: 需要指定order_id或client_order_id")
                return
            
            market = Market(args.market)
            order = await client.cancel_order(
                market,
                args.symbol,
                order_id=args.order_id,
                client_order_id=args.client_order_id
            )
            
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
            if not api_key or not api_secret:
                print("错误: 请在.env文件中设置BINANCE_API_KEY和BINANCE_API_SECRET")
                return
            
            if not args.order_id and not args.client_order_id:
                print("错误: 需要指定order_id或client_order_id")
                return
            
            market = Market(args.market)
            order = await client.get_order(
                market,
                args.symbol,
                order_id=args.order_id,
                client_order_id=args.client_order_id
            )
            
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
            try:
                print("正在初始化WebSocket客户端...")
                
                # 创建市场数据处理器
                market_handler = MarketDataHandler()
                
                # 注册处理器
                if args.type == "trade":
                    if not args.symbol:
                        print("错误: 订阅trade需要指定symbol参数")
                        return
                    stream = BINANCE_WS_TOPICS["spot"]["trade"].format(symbol=args.symbol.lower())
                    client.ws_client.add_handler("trade", market_handler.handle_trade)
                    print(f"已注册trade处理器")
                
                elif args.type == "ticker":
                    if not args.symbol:
                        print("错误: 订阅ticker需要指定symbol参数")
                        return
                    stream = BINANCE_WS_TOPICS["spot"]["ticker"].format(symbol=args.symbol.lower())
                    client.ws_client.add_handler("24hrTicker", market_handler.handle_ticker)
                    print(f"已注册ticker处理器")
                
                elif args.type == "depth":
                    if not args.symbol:
                        print("错误: 订阅depth需要指定symbol参数")
                        return
                    stream = BINANCE_WS_TOPICS["spot"]["depth"].format(symbol=args.symbol.lower(), level=args.depth)
                    client.ws_client.add_handler("depthUpdate", market_handler.handle_depth)
                    print(f"已注册depth处理器")
                
                elif args.type == "kline":
                    if not args.symbol or not args.interval:
                        print("错误: 订阅kline需要指定symbol和interval参数")
                        return
                    stream = BINANCE_WS_TOPICS["spot"]["kline"].format(
                        symbol=args.symbol.lower(),
                        interval=args.interval
                    )
                    client.ws_client.add_handler("kline", market_handler.handle_kline)
                    print(f"已注册kline处理器")
                
                elif args.type == "order":
                    if not api_key or not api_secret:
                        print("错误: 请在.env文件中设置BINANCE_API_KEY和BINANCE_API_SECRET")
                        return
                
                print(f"正在订阅 {stream}...")
                
                # 启动WebSocket客户端
                ws_task = asyncio.create_task(client.ws_client.start())
                
                try:
                    # 等待WebSocket连接成功
                    print("等待WebSocket连接...")
                    await asyncio.wait_for(client.ws_client.connected.wait(), timeout=10)
                    print("WebSocket连接成功")
                    
                    # 订阅数据流
                    if args.type != "order":
                        print(f"正在订阅数据流: {stream}")
                        await client.ws_client.subscribe([stream])
                        print("订阅成功")
                    
                    print("\n按Ctrl+C停止订阅...")
                    # 等待用户中断
                    await asyncio.Event().wait()
                    
                except asyncio.TimeoutError:
                    print("错误: WebSocket连接超时")
                except KeyboardInterrupt:
                    print("\n正在停止订阅...")
                finally:
                    # 停止WebSocket客户端
                    print("正在关闭WebSocket连接...")
                    await client.ws_client.stop()
                    # 取消WebSocket任务
                    ws_task.cancel()
                    try:
                        await ws_task
                    except asyncio.CancelledError:
                        pass
                    print("WebSocket连接已关闭")
            
            except Exception as e:
                print(f"错误: {str(e)}")
                return
    
    finally:
        # 停止API客户端
        await client.stop()

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        pass 