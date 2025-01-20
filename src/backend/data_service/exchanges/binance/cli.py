"""
Binance命令行工具
"""
import argparse
import json
import logging
import os
from typing import Optional
from ...common.exceptions import (
    APIError,
    NetworkError,
    ValidationError,
    AuthenticationError,
    PermissionError,
    RateLimitError,
    OrderError,
    PositionError
)
from ...common.models import (
    OrderSide,
    OrderType,
    PositionSide,
    TimeInForce,
    MarginType
)
from .client import BinanceAPI

logger = logging.getLogger(__name__)

def main():
    """主函数"""
    parser = argparse.ArgumentParser(description="Binance命令行工具")
    
    # 全局参数
    parser.add_argument("--testnet", action="store_true", help="使用测试网")
    parser.add_argument("--api-key", help="API Key")
    parser.add_argument("--api-secret", help="API Secret")
    
    # 子命令
    subparsers = parser.add_subparsers(dest="command", help="子命令")
    
    # 获取合约信息
    contract_info_parser = subparsers.add_parser("contract_info", help="获取合约信息")
    contract_info_parser.add_argument("--symbol", help="交易对")
    
    # 获取资金费率
    funding_rate_parser = subparsers.add_parser("funding_rate", help="获取资金费率")
    funding_rate_parser.add_argument("--symbol", required=True, help="交易对")
    
    # 获取24小时价格变动
    contract_ticker_parser = subparsers.add_parser("contract_ticker", help="获取24小时价格变动")
    contract_ticker_parser.add_argument("--symbol", required=True, help="交易对")
    
    # 获取K线数据
    klines_parser = subparsers.add_parser("contract_klines", help="获取K线数据")
    klines_parser.add_argument("--symbol", required=True, help="交易对")
    klines_parser.add_argument("--interval", required=True, help="K线间隔")
    klines_parser.add_argument("--start-time", type=int, help="开始时间(毫秒时间戳)")
    klines_parser.add_argument("--end-time", type=int, help="结束时间(毫秒时间戳)")
    klines_parser.add_argument("--limit", type=int, default=500, help="返回记录数量")
    
    # 获取深度数据
    depth_parser = subparsers.add_parser("contract_depth", help="获取深度数据")
    depth_parser.add_argument("--symbol", required=True, help="交易对")
    depth_parser.add_argument("--limit", type=int, default=100, help="返回记录数量")
    
    # 获取最近成交
    trades_parser = subparsers.add_parser("contract_trades", help="获取最近成交")
    trades_parser.add_argument("--symbol", required=True, help="交易对")
    trades_parser.add_argument("--limit", type=int, default=500, help="返回记录数量")
    
    # 调整杠杆倍数
    leverage_parser = subparsers.add_parser("change_leverage", help="调整杠杆倍数")
    leverage_parser.add_argument("--symbol", required=True, help="交易对")
    leverage_parser.add_argument("--leverage", type=int, required=True, help="杠杆倍数")
    
    # 调整保证金类型
    margin_type_parser = subparsers.add_parser("change_margin_type", help="调整保证金类型")
    margin_type_parser.add_argument("--symbol", required=True, help="交易对")
    margin_type_parser.add_argument("--type", required=True, choices=[t.value for t in MarginType], help="保证金类型")
    
    # 获取持仓信息
    position_parser = subparsers.add_parser("position_info", help="获取持仓信息")
    position_parser.add_argument("--symbol", help="交易对")
    
    # 创建合约订单
    order_parser = subparsers.add_parser("contract_order", help="创建合约订单")
    order_parser.add_argument("--symbol", required=True, help="交易对")
    order_parser.add_argument("--side", required=True, choices=[s.value for s in OrderSide], help="订单方向")
    order_parser.add_argument("--position-side", required=True, choices=[s.value for s in PositionSide], help="持仓方向")
    order_parser.add_argument("--type", required=True, choices=[t.value for t in OrderType], help="订单类型")
    order_parser.add_argument("--quantity", type=float, required=True, help="数量")
    order_parser.add_argument("--price", type=float, help="价格")
    order_parser.add_argument("--stop-price", type=float, help="触发价格")
    order_parser.add_argument("--time-in-force", choices=[t.value for t in TimeInForce], default=TimeInForce.GTC.value, help="有效方式")
    order_parser.add_argument("--reduce-only", action="store_true", help="是否只减仓")
    order_parser.add_argument("--working-type", default="CONTRACT_PRICE", help="触发价格类型")
    order_parser.add_argument("--client-order-id", help="客户端订单ID")
    
    # 撤销合约订单
    cancel_order_parser = subparsers.add_parser("cancel_contract_order", help="撤销合约订单")
    cancel_order_parser.add_argument("--symbol", required=True, help="交易对")
    cancel_order_parser.add_argument("--order-id", help="订单ID")
    cancel_order_parser.add_argument("--client-order-id", help="客户端订单ID")
    
    # 查询合约订单
    get_order_parser = subparsers.add_parser("get_contract_order", help="查询合约订单")
    get_order_parser.add_argument("--symbol", required=True, help="交易对")
    get_order_parser.add_argument("--order-id", help="订单ID")
    get_order_parser.add_argument("--client-order-id", help="客户端订单ID")
    
    # 查询当前挂单
    open_orders_parser = subparsers.add_parser("open_contract_orders", help="查询当前挂单")
    open_orders_parser.add_argument("--symbol", help="交易对")
    
    # 解析命令行参数
    args = parser.parse_args()
    
    # 如果没有指定子命令，显示帮助信息
    if not args.command:
        parser.print_help()
        return
        
    # 获取API密钥
    api_key = args.api_key or os.environ.get("BINANCE_API_KEY")
    api_secret = args.api_secret or os.environ.get("BINANCE_API_SECRET")
    
    # 创建API客户端
    client = BinanceAPI(
        api_key=api_key,
        api_secret=api_secret,
        testnet=args.testnet
    )
    
    # 打印环境信息
    print("使用测试网" if args.testnet else "使用主网")
    print("API Key:", "已配置" if api_key else "未配置")
    print("API Secret:", "已配置" if api_secret else "未配置")
    
    try:
        # 执行命令
        if args.command == "contract_info":
            result = client.get_contract_info(symbol=args.symbol)
            print(json.dumps(result, default=lambda x: x.__dict__, indent=2, ensure_ascii=False))
            
        elif args.command == "funding_rate":
            result = client.get_funding_rate(symbol=args.symbol)
            print(json.dumps(result, default=lambda x: x.__dict__, indent=2, ensure_ascii=False))
            
        elif args.command == "contract_ticker":
            result = client.get_contract_ticker(symbol=args.symbol)
            print(json.dumps(result, default=lambda x: x.__dict__, indent=2, ensure_ascii=False))
            
        elif args.command == "contract_klines":
            result = client.get_contract_klines(
                symbol=args.symbol,
                interval=args.interval,
                start_time=args.start_time,
                end_time=args.end_time,
                limit=args.limit
            )
            print(json.dumps(result, default=lambda x: x.__dict__, indent=2, ensure_ascii=False))
            
        elif args.command == "contract_depth":
            result = client.get_contract_depth(
                symbol=args.symbol,
                limit=args.limit
            )
            print(json.dumps(result, default=lambda x: x.__dict__, indent=2, ensure_ascii=False))
            
        elif args.command == "contract_trades":
            result = client.get_recent_trades(
                symbol=args.symbol,
                limit=args.limit
            )
            print(json.dumps(result, default=lambda x: x.__dict__, indent=2, ensure_ascii=False))
            
        elif args.command == "change_leverage":
            result = client.change_leverage(
                symbol=args.symbol,
                leverage=args.leverage
            )
            print(json.dumps(result, indent=2, ensure_ascii=False))
            
        elif args.command == "change_margin_type":
            result = client.change_margin_type(
                symbol=args.symbol,
                margin_type=MarginType(args.type)
            )
            print(json.dumps(result, indent=2, ensure_ascii=False))
            
        elif args.command == "position_info":
            result = client.get_position_info(symbol=args.symbol)
            print(json.dumps(result, default=lambda x: x.__dict__, indent=2, ensure_ascii=False))
            
        elif args.command == "contract_order":
            result = client.create_contract_order(
                symbol=args.symbol,
                side=OrderSide(args.side),
                position_side=PositionSide(args.position_side),
                order_type=OrderType(args.type),
                quantity=args.quantity,
                price=args.price,
                stop_price=args.stop_price,
                time_in_force=TimeInForce(args.time_in_force),
                reduce_only=args.reduce_only,
                working_type=args.working_type,
                client_order_id=args.client_order_id
            )
            print(json.dumps(result, default=lambda x: x.__dict__, indent=2, ensure_ascii=False))
            
        elif args.command == "cancel_contract_order":
            result = client.cancel_contract_order(
                symbol=args.symbol,
                order_id=args.order_id,
                client_order_id=args.client_order_id
            )
            print(json.dumps(result, default=lambda x: x.__dict__, indent=2, ensure_ascii=False))
            
        elif args.command == "get_contract_order":
            result = client.get_contract_order(
                symbol=args.symbol,
                order_id=args.order_id,
                client_order_id=args.client_order_id
            )
            print(json.dumps(result, default=lambda x: x.__dict__, indent=2, ensure_ascii=False))
            
        elif args.command == "open_contract_orders":
            result = client.get_open_contract_orders(symbol=args.symbol)
            print(json.dumps(result, default=lambda x: x.__dict__, indent=2, ensure_ascii=False))
            
    except AuthenticationError as e:
        print(f"认证失败: {e.message}")
    except PermissionError as e:
        print(f"权限不足: {e.message}")
    except RateLimitError as e:
        print(f"请求频率超限: {e.message}")
    except ValidationError as e:
        print(f"参数验证失败: {e.message}")
    except NetworkError as e:
        print(f"网络错误: {e.message}")
    except APIError as e:
        print(f"API错误: {e.message}")
    except Exception as e:
        print(f"未知错误: {str(e)}")
        
if __name__ == "__main__":
    main() 