"""
OKX命令行工具
"""
import os
import sys
import asyncio
import logging
import click
from typing import Optional
from decimal import Decimal

from ....common.models import OrderType, OrderSide, Market
from .client import OKXAPI
from .websocket import OKXWebSocket
from .handlers import OKXMessageHandler

# 配置日志
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

@click.group()
def cli():
    """OKX交易所命令行工具"""
    pass

@cli.command()
@click.option('--symbol', required=True, help='交易对')
async def get_ticker(symbol: str):
    """获取行情数据"""
    api_key = os.getenv("OKX_API_KEY")
    api_secret = os.getenv("OKX_API_SECRET")
    passphrase = os.getenv("OKX_PASSPHRASE")
    
    async with OKXAPI(api_key, api_secret, passphrase) as client:
        try:
            ticker = await client.get_ticker(symbol)
            click.echo(f"行情数据: {ticker}")
        except Exception as e:
            click.echo(f"获取行情数据失败: {str(e)}")

@cli.command()
@click.option('--symbol', required=True, help='交易对')
@click.option('--limit', default=100, help='深度')
async def get_depth(symbol: str, limit: int):
    """获取深度数据"""
    api_key = os.getenv("OKX_API_KEY")
    api_secret = os.getenv("OKX_API_SECRET")
    passphrase = os.getenv("OKX_PASSPHRASE")
    
    async with OKXAPI(api_key, api_secret, passphrase) as client:
        try:
            depth = await client.get_depth(symbol, limit)
            click.echo(f"深度数据: {depth}")
        except Exception as e:
            click.echo(f"获取深度数据失败: {str(e)}")

@cli.command()
@click.option('--symbol', required=True, help='交易对')
@click.option('--interval', default='1m', help='K线间隔')
@click.option('--limit', default=100, help='数量')
async def get_klines(symbol: str, interval: str, limit: int):
    """获取K线数据"""
    api_key = os.getenv("OKX_API_KEY")
    api_secret = os.getenv("OKX_API_SECRET")
    passphrase = os.getenv("OKX_PASSPHRASE")
    
    async with OKXAPI(api_key, api_secret, passphrase) as client:
        try:
            klines = await client.get_klines(symbol, interval, limit)
            click.echo(f"K线数据: {klines}")
        except Exception as e:
            click.echo(f"获取K线数据失败: {str(e)}")

@cli.command()
@click.option('--symbol', required=True, help='交易对')
@click.option('--type', type=click.Choice(['limit', 'market']), required=True, help='订单类型')
@click.option('--side', type=click.Choice(['buy', 'sell']), required=True, help='订单方向')
@click.option('--price', type=float, help='价格')
@click.option('--quantity', type=float, required=True, help='数量')
async def create_order(
    symbol: str,
    type: str,
    side: str,
    price: Optional[float],
    quantity: float
):
    """创建订单"""
    api_key = os.getenv("OKX_API_KEY")
    api_secret = os.getenv("OKX_API_SECRET")
    passphrase = os.getenv("OKX_PASSPHRASE")
    
    if not all([api_key, api_secret, passphrase]):
        click.echo("请先设置环境变量: OKX_API_KEY, OKX_API_SECRET, OKX_PASSPHRASE")
        return
        
    async with OKXAPI(api_key, api_secret, passphrase) as client:
        try:
            order = await client.create_order(
                symbol=symbol,
                type=OrderType[type.upper()],
                side=OrderSide[side.upper()],
                price=Decimal(str(price)) if price else None,
                quantity=Decimal(str(quantity))
            )
            click.echo(f"订单创建成功: {order}")
        except Exception as e:
            click.echo(f"创建订单失败: {str(e)}")

@cli.command()
@click.option('--symbol', required=True, help='交易对')
@click.option('--order-id', help='订单ID')
async def cancel_order(symbol: str, order_id: Optional[str]):
    """取消订单"""
    api_key = os.getenv("OKX_API_KEY")
    api_secret = os.getenv("OKX_API_SECRET")
    passphrase = os.getenv("OKX_PASSPHRASE")
    
    if not all([api_key, api_secret, passphrase]):
        click.echo("请先设置环境变量: OKX_API_KEY, OKX_API_SECRET, OKX_PASSPHRASE")
        return
        
    async with OKXAPI(api_key, api_secret, passphrase) as client:
        try:
            result = await client.cancel_order(symbol, order_id)
            click.echo(f"订单取消成功: {result}")
        except Exception as e:
            click.echo(f"取消订单失败: {str(e)}")

@cli.command()
@click.option('--symbol', required=True, help='交易对')
async def subscribe_ticker(symbol: str):
    """订阅行情数据"""
    ws = OKXWebSocket()
    handler = OKXMessageHandler()
    
    async def on_ticker(ticker):
        click.echo(f"收到行情数据: {ticker}")
        
    handler.register_callback(f"tickers:{symbol}", on_ticker)
    
    try:
        await ws.start()
        await ws.subscribe("tickers", symbol, handler.handle_ticker)
        
        # 保持运行直到用户中断
        while True:
            await asyncio.sleep(1)
            
    except KeyboardInterrupt:
        await ws.stop()
        click.echo("已停止订阅")
    except Exception as e:
        click.echo(f"订阅失败: {str(e)}")
        await ws.stop()

def main():
    """入口函数"""
    loop = asyncio.get_event_loop()
    try:
        loop.run_until_complete(cli())
    except Exception as e:
        click.echo(f"执行失败: {str(e)}")
    finally:
        loop.close()

if __name__ == '__main__':
    main() 