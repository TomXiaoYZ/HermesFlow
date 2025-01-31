"""
Binance API 使用示例
"""
import os
import asyncio
import logging
from decimal import Decimal

from src.backend.data_service.exchanges.binance.client import BinanceAPI
from src.backend.data_service.exchanges.binance.websocket import BinanceWebSocketClient
from src.backend.data_service.common.models import OrderType, OrderSide

# 配置日志
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

async def test_rest_api():
    """测试REST API"""
    # 创建API客户端
    client = BinanceAPI(
        api_key=os.getenv("BINANCE_API_KEY", ""),
        api_secret=os.getenv("BINANCE_API_SECRET", ""),
        testnet=True
    )
    
    try:
        # 获取交易所信息
        logger.info("获取交易所信息...")
        info = await client.get_exchange_info()
        logger.info(f"交易所信息: {info}")
        
        # 获取订单簿
        symbol = "BTCUSDT"
        logger.info(f"获取{symbol}订单簿...")
        order_book = await client.get_order_book(symbol)
        logger.info(f"订单簿: {order_book}")
        
        # 获取最近成交
        logger.info(f"获取{symbol}最近成交...")
        trades = await client.get_recent_trades(symbol)
        logger.info(f"最近成交: {trades}")
        
        # 获取K线数据
        logger.info(f"获取{symbol}K线数据...")
        klines = await client.get_klines(symbol, "1m")
        logger.info(f"K线数据: {klines}")
        
        # 获取行情数据
        logger.info(f"获取{symbol}行情数据...")
        ticker = await client.get_ticker(symbol)
        logger.info(f"行情数据: {ticker}")
        
        # 如果有API Key，测试交易接口
        if client.api_key and client.api_secret:
            # 获取账户信息
            logger.info("获取账户信息...")
            account = await client.get_account()
            logger.info(f"账户信息: {account}")
            
            # 创建测试订单
            logger.info("创建测试订单...")
            order = await client.create_order(
                symbol=symbol,
                side=OrderSide.BUY,
                type=OrderType.LIMIT,
                quantity=Decimal("0.001"),
                price=Decimal("20000")
            )
            logger.info(f"订单信息: {order}")
            
            # 查询订单
            logger.info(f"查询订单{order.order_id}...")
            order = await client.get_order(
                symbol=symbol,
                order_id=order.order_id
            )
            logger.info(f"订单信息: {order}")
            
            # 取消订单
            logger.info(f"取消订单{order.order_id}...")
            order = await client.cancel_order(
                symbol=symbol,
                order_id=order.order_id
            )
            logger.info(f"订单信息: {order}")
            
            # 查询当前挂单
            logger.info("查询当前挂单...")
            orders = await client.get_open_orders()
            logger.info(f"当前挂单: {orders}")
            
    finally:
        await client.close()

async def test_websocket():
    """测试WebSocket"""
    # 创建WebSocket客户端
    ws_client = BinanceWebSocketClient(
        api_key=os.getenv("BINANCE_API_KEY", ""),
        api_secret=os.getenv("BINANCE_API_SECRET", ""),
        testnet=True
    )
    
    try:
        # 定义消息处理器
        async def handle_trade(trade):
            logger.info(f"收到成交: {trade}")
            
        async def handle_kline(kline):
            logger.info(f"收到K线: {kline}")
            
        async def handle_depth(depth):
            logger.info(f"收到深度: {depth}")
            
        async def handle_ticker(ticker):
            logger.info(f"收到行情: {ticker}")
            
        async def handle_account(balances):
            logger.info(f"收到账户更新: {balances}")
            
        async def handle_order(order):
            logger.info(f"收到订单更新: {order}")
        
        # 订阅数据
        symbol = "btcusdt"
        await ws_client.subscribe_trade(symbol, handle_trade)
        await ws_client.subscribe_kline(symbol, "1m", handle_kline)
        await ws_client.subscribe_depth(symbol, handle_depth)
        await ws_client.subscribe_ticker(symbol, handle_ticker)
        
        # 如果有API Key，订阅用户数据
        if ws_client.api_key and ws_client.api_secret:
            await ws_client.subscribe_user_data(
                on_account=handle_account,
                on_order=handle_order
            )
        
        # 启动客户端
        await ws_client.start()
        
        # 运行60秒
        await asyncio.sleep(60)
        
    finally:
        await ws_client.stop()

async def main():
    """主函数"""
    # 测试REST API
    logger.info("测试REST API...")
    await test_rest_api()
    
    # 测试WebSocket
    logger.info("测试WebSocket...")
    await test_websocket()

if __name__ == "__main__":
    asyncio.run(main()) 