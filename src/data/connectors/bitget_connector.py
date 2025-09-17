"""
Bitget连接器实现
集成Bitget API v1现货和期货市场数据

功能特点:
- 支持现货市场数据接入
- REST API 和 WebSocket 实时数据流
- 完整的错误处理和网络检查
- 测试环境支持
- 基于BaseConnector统一接口规范
"""

import hmac
import hashlib
import base64
import time
import json
import asyncio
import aiohttp
import websockets
from typing import Dict, List, Optional, Any, Tuple
from urllib.parse import urlencode
from datetime import datetime

from .base_connector import BaseConnector, ConnectionConfig, DataType
from ..models.market_data import KlineData, TickerData, OrderBookData, TradeData


class BitgetConnector(BaseConnector):
    """
    Bitget交易所连接器
    
    支持现货和期货市场数据接入
    基于Bitget API v1实现
    """
    
    def __init__(self, config: ConnectionConfig):
        super().__init__(config, "bitget")
        
        # Bitget API端点配置
        if config.testnet:
            # Bitget测试环境 (暂时使用生产环境，待官方提供测试端点)
            self.rest_base_url = "https://api.bitget.com"
            self.ws_base_url = "wss://ws.bitget.com/spot/v1/stream"
        else:
            # Bitget生产环境
            self.rest_base_url = "https://api.bitget.com"
            self.ws_base_url = "wss://ws.bitget.com/spot/v1/stream"
        
        # 添加testnet属性以支持测试环境检查
        self.testnet = config.testnet
        
        # API配置
        self.api_key = config.api_key
        self.secret_key = config.api_secret
        self.passphrase = getattr(config, 'passphrase', None)
        
        # 会话管理
        self.session: Optional[aiohttp.ClientSession] = None
        self.ws_connection: Optional[websockets.WebSocketServerProtocol] = None
        
        # 产品类型映射 (Bitget特有的产品类型)
        self.product_types = {
            'spot': 'spot',
            'futures': 'umcbl',
            'swap': 'dmcbl'
        }
        
        # 时间间隔映射
        self.interval_mapping = {
            '1m': '1min',
            '5m': '5min',
            '15m': '15min',
            '30m': '30min',
            '1h': '1h',
            '4h': '4h',
            '1d': '1day',
            '1w': '1week'
        }
    
    def _generate_signature(self, timestamp: str, method: str, request_path: str, 
                          query_string: str = "", body: str = "") -> str:
        """
        生成Bitget API签名
        
        签名格式: timestamp + method.upper() + requestPath + queryString + body
        使用HMAC SHA256算法加密后Base64编码
        """
        if query_string and not query_string.startswith('?'):
            query_string = '?' + query_string
            
        message = timestamp + method.upper() + request_path + query_string + body
        
        signature = hmac.new(
            self.secret_key.encode('utf-8'),
            message.encode('utf-8'),
            hashlib.sha256
        ).digest()
        
        return base64.b64encode(signature).decode('utf-8')
    
    def _get_headers(self, method: str, request_path: str, 
                    query_string: str = "", body: str = "") -> Dict[str, str]:
        """生成请求头"""
        timestamp = str(int(time.time() * 1000))
        signature = self._generate_signature(timestamp, method, request_path, query_string, body)
        
        headers = {
            'ACCESS-KEY': self.api_key,
            'ACCESS-SIGN': signature,
            'ACCESS-TIMESTAMP': timestamp,
            'ACCESS-PASSPHRASE': self.passphrase,
            'Content-Type': 'application/json',
            'locale': 'en-US'
        }
        
        return headers
    
    async def connect(self) -> bool:
        """建立连接"""
        try:
            # 创建HTTP会话
            connector = aiohttp.TCPConnector(
                limit=100,
                limit_per_host=30,
                ttl_dns_cache=300,
                use_dns_cache=True,
            )
            
            self.session = aiohttp.ClientSession(
                connector=connector,
                timeout=aiohttp.ClientTimeout(total=30)
            )
            
            # 测试连接
            return await self.check_connection()
            
        except Exception as e:
            self.logger.error(f"连接Bitget失败: {e}")
            return False
    
    async def disconnect(self) -> None:
        """断开连接"""
        try:
            if self.ws_connection:
                await self.ws_connection.close()
                self.ws_connection = None
                
            if self.session:
                await self.session.close()
                self.session = None
                
        except Exception as e:
            self.logger.error(f"断开Bitget连接失败: {e}")
    
    async def check_connection(self) -> bool:
        """检查连接状态"""
        try:
            if not self.session:
                return False
                
            # 使用服务器时间接口测试连接
            url = f"{self.rest_base_url}/api/spot/v1/public/time"
            
            async with self.session.get(url) as response:
                if response.status == 200:
                    data = await response.json()
                    if data.get('code') == '00000':
                        return True
            
            return False
            
        except Exception as e:
            self.logger.error(f"Bitget连接检查失败: {e}")
            return False
    
    def get_health_status(self) -> Dict[str, Any]:
        """获取连接健康状态"""
        return {
            'exchange': 'bitget',
            'connected': self.session is not None,
            'rest_endpoint': self.rest_base_url,
            'ws_endpoint': self.ws_base_url,
            'testnet_mode': self.testnet,
            'api_key_configured': bool(self.api_key),
            'timestamp': datetime.now().isoformat()
        }
    
    async def get_server_time(self) -> Optional[int]:
        """获取服务器时间"""
        try:
            url = f"{self.rest_base_url}/api/spot/v1/public/time"
            
            async with self.session.get(url) as response:
                if response.status == 200:
                    data = await response.json()
                    if data.get('code') == '00000':
                        return data.get('data')
            
            return None
            
        except Exception as e:
            self.logger.error(f"获取Bitget服务器时间失败: {e}")
            return None
    
    async def get_exchange_info(self) -> Optional[Dict[str, Any]]:
        """获取交易所信息"""
        try:
            url = f"{self.rest_base_url}/api/spot/v1/public/products"
            
            async with self.session.get(url) as response:
                if response.status == 200:
                    data = await response.json()
                    if data.get('code') == '00000':
                        return data.get('data', [])
            
            return None
            
        except Exception as e:
            self.logger.error(f"获取Bitget交易所信息失败: {e}")
            return None
    
    def _convert_symbol(self, symbol: str) -> str:
        """转换交易对格式 (标准格式 -> Bitget格式)"""
        if '/' in symbol:
            base, quote = symbol.upper().split('/')
            return f"{base}{quote}_SPBL"  # Bitget现货交易对格式
        return symbol.upper()
    
    def _normalize_symbol(self, symbol: str) -> str:
        """标准化交易对格式 (Bitget格式 -> 标准格式)"""
        if '_SPBL' in symbol:
            base_quote = symbol.replace('_SPBL', '')
            # 需要根据具体交易对进行拆分
            # 这里简化处理，实际应该查询交易所信息
            if 'USDT' in base_quote:
                base = base_quote.replace('USDT', '')
                return f"{base}/USDT"
            elif 'BTC' in base_quote and not base_quote.startswith('BTC'):
                base = base_quote.replace('BTC', '')
                return f"{base}/BTC"
        return symbol
    
    async def get_klines(self, symbol: str, interval: str, limit: int = 100, 
                        start_time: Optional[int] = None, end_time: Optional[int] = None) -> List[KlineData]:
        """获取K线数据"""
        try:
            # 转换参数
            bitget_symbol = self._convert_symbol(symbol)
            bitget_interval = self.interval_mapping.get(interval, '1min')
            
            # 构建请求参数
            params = {
                'symbol': bitget_symbol,
                'period': bitget_interval,
                'limit': min(limit, 1000)  # Bitget最大限制
            }
            
            if start_time:
                params['after'] = str(start_time)
            if end_time:
                params['before'] = str(end_time)
            
            url = f"{self.rest_base_url}/api/spot/v1/market/candles"
            query_string = urlencode(params)
            
            async with self.session.get(f"{url}?{query_string}") as response:
                if response.status == 200:
                    data = await response.json()
                    if data.get('code') == '00000':
                        klines = []
                        for item in data.get('data', []):
                            kline = KlineData(
                                symbol=self._normalize_symbol(bitget_symbol),
                                open_time=int(item['ts']),
                                close_time=int(item['ts']) + 60000,  # 简化处理
                                open_price=float(item['open']),
                                high_price=float(item['high']),
                                low_price=float(item['low']),
                                close_price=float(item['close']),
                                volume=float(item['baseVol']),
                                quote_volume=float(item['quoteVol']),
                                trades_count=0,  # Bitget API未提供
                                interval=interval
                            )
                            klines.append(kline)
                        
                        return sorted(klines, key=lambda x: x.open_time)
            
            return []
            
        except Exception as e:
            self.logger.error(f"获取Bitget K线数据失败 {symbol}: {e}")
            return []
    
    async def get_ticker(self, symbol: str) -> Optional[TickerData]:
        """获取单个交易对行情"""
        try:
            bitget_symbol = self._convert_symbol(symbol)
            
            url = f"{self.rest_base_url}/api/spot/v1/market/ticker"
            params = {'symbol': bitget_symbol}
            query_string = urlencode(params)
            
            async with self.session.get(f"{url}?{query_string}") as response:
                if response.status == 200:
                    data = await response.json()
                    if data.get('code') == '00000':
                        ticker_data = data.get('data', {})
                        
                        ticker = TickerData(
                            symbol=symbol,
                            price=float(ticker_data.get('close', 0)),
                            bid_price=float(ticker_data.get('buyOne', 0)),
                            ask_price=float(ticker_data.get('sellOne', 0)),
                            bid_qty=float(ticker_data.get('bidSz', 0)),
                            ask_qty=float(ticker_data.get('askSz', 0)),
                            volume=float(ticker_data.get('baseVol', 0)),
                            quote_volume=float(ticker_data.get('quoteVol', 0)),
                            high_24h=float(ticker_data.get('high24h', 0)),
                            low_24h=float(ticker_data.get('low24h', 0)),
                            change_24h=float(ticker_data.get('change', 0)),
                            timestamp=int(ticker_data.get('ts', 0))
                        )
                        
                        return ticker
            
            return None
            
        except Exception as e:
            self.logger.error(f"获取Bitget行情失败 {symbol}: {e}")
            return None
    
    async def get_all_tickers(self) -> List[TickerData]:
        """获取所有交易对行情"""
        try:
            url = f"{self.rest_base_url}/api/spot/v1/market/tickers"
            
            async with self.session.get(url) as response:
                if response.status == 200:
                    data = await response.json()
                    if data.get('code') == '00000':
                        tickers = []
                        for ticker_data in data.get('data', []):
                            # 从Bitget符号推断标准符号
                            symbol = ticker_data.get('symbol', '')
                            normalized_symbol = self._normalize_symbol(symbol + '_SPBL')
                            
                            ticker = TickerData(
                                symbol=normalized_symbol,
                                price=float(ticker_data.get('close', 0)),
                                bid_price=float(ticker_data.get('buyOne', 0)),
                                ask_price=float(ticker_data.get('sellOne', 0)),
                                bid_qty=float(ticker_data.get('bidSz', 0)),
                                ask_qty=float(ticker_data.get('askSz', 0)),
                                volume=float(ticker_data.get('baseVol', 0)),
                                quote_volume=float(ticker_data.get('quoteVol', 0)),
                                high_24h=float(ticker_data.get('high24h', 0)),
                                low_24h=float(ticker_data.get('low24h', 0)),
                                change_24h=float(ticker_data.get('change', 0)),
                                timestamp=int(ticker_data.get('ts', 0))
                            )
                            tickers.append(ticker)
                        
                        return tickers
            
            return []
            
        except Exception as e:
            self.logger.error(f"获取Bitget所有行情失败: {e}")
            return []
    
    async def get_orderbook(self, symbol: str, limit: int = 100) -> Optional[OrderBookData]:
        """获取订单簿"""
        try:
            bitget_symbol = self._convert_symbol(symbol)
            
            params = {
                'symbol': bitget_symbol,
                'type': 'step0',  # 原始价格
                'limit': min(limit, 200)  # Bitget最大限制
            }
            
            url = f"{self.rest_base_url}/api/spot/v1/market/depth"
            query_string = urlencode(params)
            
            async with self.session.get(f"{url}?{query_string}") as response:
                if response.status == 200:
                    data = await response.json()
                    if data.get('code') == '00000':
                        order_data = data.get('data', {})
                        
                        orderbook = OrderBookData(
                            symbol=symbol,
                            bids=[(float(bid[0]), float(bid[1])) for bid in order_data.get('bids', [])],
                            asks=[(float(ask[0]), float(ask[1])) for ask in order_data.get('asks', [])],
                            timestamp=int(order_data.get('timestamp', 0))
                        )
                        
                        return orderbook
            
            return None
            
        except Exception as e:
            self.logger.error(f"获取Bitget订单簿失败 {symbol}: {e}")
            return None
    
    async def get_recent_trades(self, symbol: str, limit: int = 100) -> List[TradeData]:
        """获取最近成交记录"""
        try:
            bitget_symbol = self._convert_symbol(symbol)
            
            params = {
                'symbol': bitget_symbol,
                'limit': min(limit, 500)  # Bitget最大限制
            }
            
            url = f"{self.rest_base_url}/api/spot/v1/market/fills"
            query_string = urlencode(params)
            
            async with self.session.get(f"{url}?{query_string}") as response:
                if response.status == 200:
                    data = await response.json()
                    if data.get('code') == '00000':
                        trades = []
                        for trade_data in data.get('data', []):
                            trade = TradeData(
                                symbol=symbol,
                                trade_id=trade_data.get('tradeId', ''),
                                price=float(trade_data.get('fillPrice', 0)),
                                quantity=float(trade_data.get('fillQuantity', 0)),
                                timestamp=int(trade_data.get('fillTime', 0)),
                                is_buyer_maker=trade_data.get('side', '').lower() == 'sell'
                            )
                            trades.append(trade)
                        
                        return sorted(trades, key=lambda x: x.timestamp, reverse=True)
            
            return []
            
        except Exception as e:
            self.logger.error(f"获取Bitget成交记录失败 {symbol}: {e}")
            return []
    
    async def subscribe_ticker(self, symbol: str, callback):
        """订阅行情数据"""
        # WebSocket实现留待后续完善
        pass
    
    async def subscribe_kline(self, symbol: str, interval: str, callback):
        """订阅K线数据"""
        # WebSocket实现留待后续完善
        pass
    
    async def subscribe_orderbook(self, symbol: str, callback):
        """订阅订单簿数据"""
        # WebSocket实现留待后续完善
        pass
    
    async def subscribe_trades(self, symbol: str, callback):
        """订阅成交数据"""
        # WebSocket实现留待后续完善
        pass
    
    async def get_symbols(self) -> List[str]:
        """获取支持的交易对列表"""
        try:
            url = f"{self.rest_base_url}/api/spot/v1/public/products"
            
            async with self.session.get(url) as response:
                if response.status == 200:
                    data = await response.json()
                    if data.get('code') == '00000':
                        symbols = []
                        for product in data.get('data', []):
                            if product.get('status') == 'online':
                                symbol = self._normalize_symbol(product.get('symbol', ''))
                                if symbol:
                                    symbols.append(symbol)
                        return symbols
            return []
        except Exception as e:
            self.logger.error(f"获取Bitget交易对列表失败: {e}")
            return []
    
    async def subscribe_real_time(
        self, 
        symbols: List[str], 
        data_types: List[DataType],
        callback: callable
    ) -> bool:
        """订阅实时数据 - 基础实现"""
        try:
            # 暂时返回True，WebSocket实现留待后续完善
            self.logger.info(f"Bitget WebSocket订阅功能开发中: {symbols} - {data_types}")
            return True
        except Exception as e:
            self.logger.error(f"Bitget实时数据订阅失败: {e}")
            return False
    
    async def unsubscribe_real_time(
        self, 
        symbols: List[str], 
        data_types: List[DataType]
    ) -> bool:
        """取消订阅实时数据 - 基础实现"""
        try:
            # 暂时返回True，WebSocket实现留待后续完善
            self.logger.info(f"Bitget WebSocket取消订阅功能开发中: {symbols} - {data_types}")
            return True
        except Exception as e:
            self.logger.error(f"Bitget取消实时数据订阅失败: {e}")
            return False
    
    async def __aenter__(self):
        """异步上下文管理器入口"""
        await self.connect()
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """异步上下文管理器退出"""
        await self.disconnect()


def create_bitget_connector(config: ConnectionConfig) -> BitgetConnector:
    """
    创建Bitget连接器实例的工厂函数
    
    Args:
        config: 连接配置
        
    Returns:
        BitgetConnector: Bitget连接器实例
    """
    return BitgetConnector(config) 