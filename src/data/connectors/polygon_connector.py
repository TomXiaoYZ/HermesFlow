"""
Polygon.io连接器 - 美股和期权数据接入

支持功能：
- 实时股票行情数据
- 期权链数据和Greeks计算
- 历史数据回填
- 盘前盘后交易数据
- 隐含波动率分析

作者: HermesFlow Team
创建时间: 2024年12月20日
"""

import asyncio
import aiohttp
import json
import logging
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any, Tuple
from dataclasses import dataclass, asdict
import time
import math

from .base_connector import BaseConnector, ConnectionConfig

# 配置日志
logger = logging.getLogger(__name__)

@dataclass
class StockQuote:
    """股票报价数据模型"""
    symbol: str
    price: float
    volume: int
    timestamp: datetime
    high: float
    low: float
    open: float
    close: float
    change: float
    change_percent: float
    market_status: str  # 'open', 'closed', 'pre', 'after'

@dataclass
class OptionContract:
    """期权合约数据模型"""
    symbol: str
    underlying_symbol: str
    contract_type: str  # 'call' or 'put'
    strike_price: float
    expiration_date: datetime
    price: float
    bid: float
    ask: float
    volume: int
    open_interest: int
    implied_volatility: float
    delta: float
    gamma: float
    theta: float
    vega: float
    rho: float
    timestamp: datetime

@dataclass
class OptionsChain:
    """期权链数据模型"""
    underlying_symbol: str
    expiration_date: datetime
    calls: List[OptionContract]
    puts: List[OptionContract]
    underlying_price: float
    timestamp: datetime

class PolygonConnector(BaseConnector):
    """
    Polygon.io API连接器
    
    提供美股和期权数据的实时和历史数据接入功能
    """
    
    def __init__(self, config: ConnectionConfig):
        """
        初始化Polygon连接器
        
        Args:
            config: 连接配置，API密钥通过config.api_key传递
        """
        super().__init__(config, "polygon")
        self.api_key = config.api_key or "demo_key"  # 使用演示密钥如果没有提供
        self.base_url = "https://api.polygon.io"
        self.session: Optional[aiohttp.ClientSession] = None
        self.ws_session: Optional[aiohttp.ClientSession] = None
        self.websocket_url = "wss://socket.polygon.io/stocks"
        
        # API限制控制
        self.rate_limit_calls = 0
        self.rate_limit_window_start = time.time()
        self.max_calls_per_minute = 5  # 免费账户限制
        
        # 支持的市场和产品类型
        self.supported_markets = {
            'stocks': ['NASDAQ', 'NYSE', 'AMEX'],
            'options': ['OPRA']  # Options Price Reporting Authority
        }
        
        # 数据缓存
        self.quote_cache: Dict[str, StockQuote] = {}
        self.options_cache: Dict[str, OptionsChain] = {}
        
        logger.info(f"Polygon连接器初始化完成，API Key: {self.api_key[:8] if self.api_key != 'demo_key' else 'demo_key'}...")

    async def connect(self) -> bool:
        """建立与Polygon API的连接"""
        try:
            # 创建HTTP会话
            connector = aiohttp.TCPConnector(
                limit=100,
                limit_per_host=30,
                ttl_dns_cache=300,
                use_dns_cache=True,
            )
            
            timeout = aiohttp.ClientTimeout(total=30, connect=10)
            self.session = aiohttp.ClientSession(
                connector=connector,
                timeout=timeout,
                headers={
                    'User-Agent': 'HermesFlow-PolygonConnector/1.0',
                    'Authorization': f'Bearer {self.api_key}'
                }
            )
            
            # 测试API连接
            test_response = await self._make_request('/v2/reference/markets')
            if test_response and test_response.get('status') == 'OK':
                logger.info("Polygon API连接测试成功")
                self.is_connected = True
                return True
            else:
                logger.error("Polygon API连接测试失败")
                return False
                
        except Exception as e:
            logger.error(f"连接Polygon API失败: {e}")
            return False

    async def disconnect(self):
        """断开连接并清理资源"""
        try:
            if self.session:
                await self.session.close()
                self.session = None
            
            if self.ws_session:
                await self.ws_session.close()
                self.ws_session = None
            
            self.is_connected = False
            logger.info("Polygon连接器已断开")
            
        except Exception as e:
            logger.error(f"断开Polygon连接时出错: {e}")

    async def _make_request(self, endpoint: str, params: Dict[str, Any] = None) -> Optional[Dict]:
        """
        发起HTTP请求
        
        Args:
            endpoint: API端点
            params: 请求参数
            
        Returns:
            响应数据或None
        """
        if not self.session:
            logger.error("HTTP会话未初始化")
            return None
        
        # 检查API限制
        await self._check_rate_limit()
        
        url = f"{self.base_url}{endpoint}"
        
        # 添加API密钥到参数
        if params is None:
            params = {}
        params['apikey'] = self.api_key
        
        try:
            async with self.session.get(url, params=params) as response:
                if response.status == 200:
                    data = await response.json()
                    return data
                elif response.status == 429:
                    logger.warning("API请求频率限制，等待重试...")
                    await asyncio.sleep(60)  # 等待1分钟后重试
                    return await self._make_request(endpoint, params)
                else:
                    logger.error(f"API请求失败，状态码: {response.status}")
                    return None
                    
        except Exception as e:
            logger.error(f"发起API请求时出错: {e}")
            return None

    async def _check_rate_limit(self):
        """检查和控制API请求频率"""
        current_time = time.time()
        
        # 重置速率限制窗口
        if current_time - self.rate_limit_window_start >= 60:
            self.rate_limit_calls = 0
            self.rate_limit_window_start = current_time
        
        # 检查是否超过限制
        if self.rate_limit_calls >= self.max_calls_per_minute:
            sleep_time = 60 - (current_time - self.rate_limit_window_start)
            if sleep_time > 0:
                logger.info(f"达到API速率限制，等待 {sleep_time:.1f} 秒")
                await asyncio.sleep(sleep_time)
                self.rate_limit_calls = 0
                self.rate_limit_window_start = time.time()
        
        self.rate_limit_calls += 1

    async def get_stock_quote(self, symbol: str) -> Optional[StockQuote]:
        """
        获取股票实时报价
        
        Args:
            symbol: 股票代码 (如 'AAPL')
            
        Returns:
            股票报价数据或None
        """
        try:
            # 获取实时报价
            endpoint = f"/v2/last/trade/{symbol}"
            trade_data = await self._make_request(endpoint)
            
            # 获取当日统计数据
            endpoint = f"/v1/open-close/{symbol}/{datetime.now().strftime('%Y-%m-%d')}"
            daily_data = await self._make_request(endpoint)
            
            if not trade_data or trade_data.get('status') != 'OK':
                logger.warning(f"获取 {symbol} 交易数据失败")
                return None
            
            # 解析数据
            trade_result = trade_data.get('results', {})
            daily_result = daily_data.get('results', {}) if daily_data else {}
            
            # 计算变化
            current_price = trade_result.get('p', 0)
            prev_close = daily_result.get('close', current_price)
            change = current_price - prev_close
            change_percent = (change / prev_close * 100) if prev_close > 0 else 0
            
            # 确定市场状态
            market_status = self._get_market_status()
            
            quote = StockQuote(
                symbol=symbol,
                price=current_price,
                volume=trade_result.get('v', 0),
                timestamp=datetime.fromtimestamp(trade_result.get('t', 0) / 1000),
                high=daily_result.get('high', current_price),
                low=daily_result.get('low', current_price),
                open=daily_result.get('open', current_price),
                close=prev_close,
                change=change,
                change_percent=change_percent,
                market_status=market_status
            )
            
            # 缓存数据
            self.quote_cache[symbol] = quote
            
            logger.debug(f"获取 {symbol} 报价成功: ${current_price:.2f}")
            return quote
            
        except Exception as e:
            logger.error(f"获取 {symbol} 股票报价时出错: {e}")
            return None

    async def get_options_chain(self, underlying_symbol: str, expiration_date: str = None) -> Optional[OptionsChain]:
        """
        获取期权链数据
        
        Args:
            underlying_symbol: 标的股票代码
            expiration_date: 到期日期 (YYYY-MM-DD)，默认获取最近到期的期权
            
        Returns:
            期权链数据或None
        """
        try:
            # 如果没有指定到期日，获取最近的到期日
            if not expiration_date:
                expiration_date = await self._get_next_expiration_date(underlying_symbol)
                if not expiration_date:
                    logger.error(f"无法获取 {underlying_symbol} 的期权到期日")
                    return None
            
            # 获取期权合约列表
            endpoint = "/v3/reference/options/contracts"
            params = {
                'underlying_ticker': underlying_symbol,
                'expiration_date': expiration_date,
                'limit': 1000
            }
            
            contracts_data = await self._make_request(endpoint, params)
            if not contracts_data or contracts_data.get('status') != 'OK':
                logger.warning(f"获取 {underlying_symbol} 期权合约列表失败")
                return None
            
            # 获取标的股票价格
            underlying_quote = await self.get_stock_quote(underlying_symbol)
            underlying_price = underlying_quote.price if underlying_quote else 0
            
            # 解析期权合约
            calls = []
            puts = []
            
            contracts = contracts_data.get('results', [])
            
            # 并发获取期权报价数据
            option_tasks = []
            for contract in contracts:
                task = self._get_option_contract_data(contract, underlying_price)
                option_tasks.append(task)
            
            # 限制并发数量以避免API限制
            batch_size = 10
            option_contracts = []
            
            for i in range(0, len(option_tasks), batch_size):
                batch = option_tasks[i:i + batch_size]
                batch_results = await asyncio.gather(*batch, return_exceptions=True)
                
                for result in batch_results:
                    if isinstance(result, OptionContract):
                        option_contracts.append(result)
                
                # 批次之间延迟以避免API限制
                if i + batch_size < len(option_tasks):
                    await asyncio.sleep(1)
            
            # 分类期权合约
            for contract in option_contracts:
                if contract.contract_type == 'call':
                    calls.append(contract)
                elif contract.contract_type == 'put':
                    puts.append(contract)
            
            # 按执行价格排序
            calls.sort(key=lambda x: x.strike_price)
            puts.sort(key=lambda x: x.strike_price)
            
            options_chain = OptionsChain(
                underlying_symbol=underlying_symbol,
                expiration_date=datetime.strptime(expiration_date, '%Y-%m-%d'),
                calls=calls,
                puts=puts,
                underlying_price=underlying_price,
                timestamp=datetime.now()
            )
            
            # 缓存数据
            cache_key = f"{underlying_symbol}_{expiration_date}"
            self.options_cache[cache_key] = options_chain
            
            logger.info(f"获取 {underlying_symbol} 期权链成功: {len(calls)} calls, {len(puts)} puts")
            return options_chain
            
        except Exception as e:
            logger.error(f"获取 {underlying_symbol} 期权链时出错: {e}")
            return None

    async def _get_option_contract_data(self, contract: Dict, underlying_price: float) -> Optional[OptionContract]:
        """
        获取单个期权合约的详细数据
        
        Args:
            contract: 期权合约基础信息
            underlying_price: 标的股票价格
            
        Returns:
            期权合约数据或None
        """
        try:
            ticker = contract.get('ticker')
            if not ticker:
                return None
            
            # 获取期权报价
            endpoint = f"/v2/last/trade/{ticker}"
            quote_data = await self._make_request(endpoint)
            
            if not quote_data or quote_data.get('status') != 'OK':
                return None
            
            quote_result = quote_data.get('results', {})
            
            # 解析合约信息
            strike_price = contract.get('strike_price', 0)
            expiration_date = datetime.strptime(contract.get('expiration_date'), '%Y-%m-%d')
            contract_type = contract.get('contract_type', '').lower()
            
            # 获取期权价格和Greek数据
            price = quote_result.get('p', 0)
            
            # 计算隐含波动率和Greeks (简化版本，实际项目中应使用专业期权定价模型)
            time_to_expiry = (expiration_date - datetime.now()).days / 365.0
            iv = self._calculate_implied_volatility(price, underlying_price, strike_price, time_to_expiry, contract_type)
            greeks = self._calculate_greeks(underlying_price, strike_price, time_to_expiry, iv, contract_type)
            
            option_contract = OptionContract(
                symbol=ticker,
                underlying_symbol=contract.get('underlying_ticker'),
                contract_type=contract_type,
                strike_price=strike_price,
                expiration_date=expiration_date,
                price=price,
                bid=quote_result.get('bid', 0),
                ask=quote_result.get('ask', 0),
                volume=quote_result.get('v', 0),
                open_interest=contract.get('open_interest', 0),
                implied_volatility=iv,
                delta=greeks.get('delta', 0),
                gamma=greeks.get('gamma', 0),
                theta=greeks.get('theta', 0),
                vega=greeks.get('vega', 0),
                rho=greeks.get('rho', 0),
                timestamp=datetime.fromtimestamp(quote_result.get('t', 0) / 1000)
            )
            
            return option_contract
            
        except Exception as e:
            logger.error(f"获取期权合约数据时出错: {e}")
            return None

    async def _get_next_expiration_date(self, underlying_symbol: str) -> Optional[str]:
        """获取下一个期权到期日"""
        try:
            # 获取今天的日期
            today = datetime.now().date()
            
            # 期权通常在每月第三个星期五到期
            # 这里简化处理，获取当月和下月的第三个星期五
            current_month = today.replace(day=1)
            next_month = (current_month + timedelta(days=32)).replace(day=1)
            
            expiration_dates = []
            
            for month_start in [current_month, next_month]:
                # 找到第三个星期五
                day = month_start
                friday_count = 0
                
                while day.month == month_start.month:
                    if day.weekday() == 4:  # 星期五
                        friday_count += 1
                        if friday_count == 3:
                            if day >= today:
                                expiration_dates.append(day.strftime('%Y-%m-%d'))
                            break
                    day += timedelta(days=1)
            
            # 返回最近的到期日
            return expiration_dates[0] if expiration_dates else None
            
        except Exception as e:
            logger.error(f"计算期权到期日时出错: {e}")
            return None

    def _calculate_implied_volatility(self, option_price: float, stock_price: float, 
                                     strike_price: float, time_to_expiry: float, 
                                     option_type: str) -> float:
        """
        计算隐含波动率 (简化版Black-Scholes)
        
        Args:
            option_price: 期权价格
            stock_price: 股票价格
            strike_price: 执行价格
            time_to_expiry: 到期时间 (年)
            option_type: 期权类型 ('call' or 'put')
            
        Returns:
            隐含波动率
        """
        try:
            # 这是一个简化的隐含波动率计算
            # 实际项目中应使用Newton-Raphson方法或其他数值方法
            
            if time_to_expiry <= 0 or option_price <= 0:
                return 0.0
            
            # 使用简化的启发式公式
            if option_type == 'call':
                intrinsic_value = max(stock_price - strike_price, 0)
            else:
                intrinsic_value = max(strike_price - stock_price, 0)
            
            time_value = max(option_price - intrinsic_value, 0)
            
            if time_value <= 0:
                return 0.0
            
            # 简化的隐含波动率估算
            iv = (time_value / stock_price) / math.sqrt(time_to_expiry) * 2
            
            return min(max(iv, 0.01), 5.0)  # 限制在合理范围内
            
        except Exception:
            return 0.2  # 默认20%波动率

    def _calculate_greeks(self, stock_price: float, strike_price: float, 
                         time_to_expiry: float, volatility: float, 
                         option_type: str) -> Dict[str, float]:
        """
        计算期权Greeks (简化版Black-Scholes)
        
        Args:
            stock_price: 股票价格
            strike_price: 执行价格
            time_to_expiry: 到期时间 (年)
            volatility: 波动率
            option_type: 期权类型
            
        Returns:
            Greeks字典
        """
        try:
            if time_to_expiry <= 0 or volatility <= 0:
                return {'delta': 0, 'gamma': 0, 'theta': 0, 'vega': 0, 'rho': 0}
            
            # 简化的Greeks计算 (实际应使用完整的Black-Scholes公式)
            risk_free_rate = 0.05  # 假设5%无风险利率
            
            d1 = (math.log(stock_price / strike_price) + 
                  (risk_free_rate + 0.5 * volatility ** 2) * time_to_expiry) / (
                  volatility * math.sqrt(time_to_expiry))
            
            d2 = d1 - volatility * math.sqrt(time_to_expiry)
            
            # 标准正态分布累积分布函数 (简化版)
            def norm_cdf(x):
                return 0.5 * (1 + math.erf(x / math.sqrt(2)))
            
            # 标准正态分布概率密度函数
            def norm_pdf(x):
                return math.exp(-0.5 * x ** 2) / math.sqrt(2 * math.pi)
            
            # Delta
            if option_type == 'call':
                delta = norm_cdf(d1)
            else:
                delta = norm_cdf(d1) - 1
            
            # Gamma
            gamma = norm_pdf(d1) / (stock_price * volatility * math.sqrt(time_to_expiry))
            
            # Theta
            if option_type == 'call':
                theta = (-stock_price * norm_pdf(d1) * volatility / (2 * math.sqrt(time_to_expiry)) -
                        risk_free_rate * strike_price * math.exp(-risk_free_rate * time_to_expiry) * norm_cdf(d2)) / 365
            else:
                theta = (-stock_price * norm_pdf(d1) * volatility / (2 * math.sqrt(time_to_expiry)) +
                        risk_free_rate * strike_price * math.exp(-risk_free_rate * time_to_expiry) * norm_cdf(-d2)) / 365
            
            # Vega
            vega = stock_price * norm_pdf(d1) * math.sqrt(time_to_expiry) / 100
            
            # Rho
            if option_type == 'call':
                rho = strike_price * time_to_expiry * math.exp(-risk_free_rate * time_to_expiry) * norm_cdf(d2) / 100
            else:
                rho = -strike_price * time_to_expiry * math.exp(-risk_free_rate * time_to_expiry) * norm_cdf(-d2) / 100
            
            return {
                'delta': round(delta, 4),
                'gamma': round(gamma, 4),
                'theta': round(theta, 4),
                'vega': round(vega, 4),
                'rho': round(rho, 4)
            }
            
        except Exception as e:
            logger.error(f"计算Greeks时出错: {e}")
            return {'delta': 0, 'gamma': 0, 'theta': 0, 'vega': 0, 'rho': 0}

    def _get_market_status(self) -> str:
        """
        获取市场状态
        
        Returns:
            市场状态: 'pre', 'open', 'after', 'closed'
        """
        try:
            now = datetime.now()
            weekday = now.weekday()
            
            # 周末
            if weekday >= 5:  # 星期六和星期日
                return 'closed'
            
            # 市场时间 (EST)
            market_open = now.replace(hour=9, minute=30, second=0, microsecond=0)
            market_close = now.replace(hour=16, minute=0, second=0, microsecond=0)
            pre_market_start = now.replace(hour=4, minute=0, second=0, microsecond=0)
            after_market_end = now.replace(hour=20, minute=0, second=0, microsecond=0)
            
            if pre_market_start <= now < market_open:
                return 'pre'
            elif market_open <= now < market_close:
                return 'open'
            elif market_close <= now < after_market_end:
                return 'after'
            else:
                return 'closed'
                
        except Exception:
            return 'unknown'

    async def get_historical_data(self, symbol: str, timeframe: str = '1D', 
                                 start_date: str = None, end_date: str = None) -> List[Dict]:
        """
        获取历史数据
        
        Args:
            symbol: 股票代码
            timeframe: 时间周期 ('1T', '5T', '1H', '1D')
            start_date: 开始日期 (YYYY-MM-DD)
            end_date: 结束日期 (YYYY-MM-DD)
            
        Returns:
            历史数据列表
        """
        try:
            # 设置默认日期范围
            if not end_date:
                end_date = datetime.now().strftime('%Y-%m-%d')
            if not start_date:
                start_date = (datetime.now() - timedelta(days=30)).strftime('%Y-%m-%d')
            
            # 构建API端点
            if timeframe in ['1T', '5T', '15T', '30T', '1H']:
                # 分钟级数据
                multiplier = 1
                timespan = 'minute'
                if timeframe == '5T':
                    multiplier = 5
                elif timeframe == '15T':
                    multiplier = 15
                elif timeframe == '30T':
                    multiplier = 30
                elif timeframe == '1H':
                    multiplier = 60
                    
                endpoint = f"/v2/aggs/ticker/{symbol}/range/{multiplier}/{timespan}/{start_date}/{end_date}"
            else:
                # 日级数据
                endpoint = f"/v2/aggs/ticker/{symbol}/range/1/day/{start_date}/{end_date}"
            
            params = {
                'adjusted': 'true',
                'sort': 'asc',
                'limit': 50000
            }
            
            data = await self._make_request(endpoint, params)
            
            if not data or data.get('status') != 'OK':
                logger.warning(f"获取 {symbol} 历史数据失败")
                return []
            
            results = data.get('results', [])
            
            # 格式化数据
            formatted_data = []
            for bar in results:
                formatted_data.append({
                    'timestamp': datetime.fromtimestamp(bar.get('t', 0) / 1000),
                    'open': bar.get('o', 0),
                    'high': bar.get('h', 0),
                    'low': bar.get('l', 0),
                    'close': bar.get('c', 0),
                    'volume': bar.get('v', 0),
                    'vwap': bar.get('vw', 0)  # 成交量加权平均价
                })
            
            logger.info(f"获取 {symbol} 历史数据成功: {len(formatted_data)} 条记录")
            return formatted_data
            
        except Exception as e:
            logger.error(f"获取 {symbol} 历史数据时出错: {e}")
            return []

    # 实现BaseConnector的抽象方法
    async def get_symbols(self) -> List[str]:
        """获取支持的股票代码列表"""
        return self.get_supported_symbols()
    
    async def get_klines(self, symbol: str, interval: str, start_time=None, end_time=None, limit: int = 500):
        """获取K线数据（使用历史数据）"""
        # 转换时间格式
        start_date = start_time.strftime('%Y-%m-%d') if start_time else None
        end_date = end_time.strftime('%Y-%m-%d') if end_time else None
        
        # 转换时间间隔格式
        timeframe_map = {
            '1m': '1T', '5m': '5T', '15m': '15T', '30m': '30T',
            '1h': '1H', '1d': '1D'
        }
        timeframe = timeframe_map.get(interval, '1D')
        
        historical_data = await self.get_historical_data(symbol, timeframe, start_date, end_date)
        return historical_data[:limit] if historical_data else []
    
    async def get_ticker(self, symbol: str):
        """获取股票报价"""
        quote = await self.get_stock_quote(symbol)
        if quote:
            return {
                'symbol': symbol,
                'price': quote.price,
                'volume': quote.volume,
                'timestamp': quote.timestamp,
                'change': quote.change,
                'change_percent': quote.change_percent
            }
        return None
    
    async def get_orderbook(self, symbol: str, depth: int = 20):
        """股票数据连接器不支持订单簿数据"""
        raise NotImplementedError("股票数据连接器不支持订单簿数据")
    
    async def subscribe_real_time(self, symbols: List[str], data_types, callback):
        """订阅实时数据（WebSocket功能开发中）"""
        for symbol in symbols:
            await self.subscribe_realtime_data(symbol, callback)
        return True
    
    async def unsubscribe_real_time(self, symbols: List[str], data_types):
        """取消订阅实时数据"""
        # TODO: 实现WebSocket取消订阅
        logger.info(f"取消订阅实时数据: {symbols}")
        return True

    # BaseConnector接口实现
    async def get_ticker_info(self, symbol: str) -> Dict[str, Any]:
        """获取股票基本信息"""
        quote = await self.get_stock_quote(symbol)
        if quote:
            return asdict(quote)
        return {}

    async def get_market_data(self, symbol: str, data_type: str = 'quote') -> Dict[str, Any]:
        """获取市场数据"""
        if data_type == 'quote':
            return await self.get_ticker_info(symbol)
        elif data_type == 'options':
            options_chain = await self.get_options_chain(symbol)
            if options_chain:
                return asdict(options_chain)
        return {}

    async def subscribe_realtime_data(self, symbol: str, callback=None):
        """订阅实时数据 (WebSocket)"""
        # TODO: 实现WebSocket实时数据订阅
        logger.info(f"实时数据订阅功能开发中: {symbol}")
        pass

    async def get_account_info(self) -> Dict[str, Any]:
        """获取账户信息 (Polygon不提供交易功能)"""
        return {'error': 'Polygon仅提供市场数据，不支持账户功能'}

    def get_supported_symbols(self) -> List[str]:
        """获取支持的股票代码"""
        # 返回一些常见的股票代码作为示例
        return [
            'AAPL', 'MSFT', 'GOOGL', 'AMZN', 'TSLA', 
            'NVDA', 'META', 'NFLX', 'ORCL', 'CRM'
        ]

    def get_supported_markets(self) -> List[str]:
        """获取支持的市场"""
        return ['NASDAQ', 'NYSE', 'AMEX']

    # 健康检查和诊断
    async def health_check(self) -> Dict[str, Any]:
        """检查连接器健康状态"""
        try:
            if not self.is_connected:
                return {
                    'status': 'unhealthy',
                    'message': '未连接到Polygon API',
                    'timestamp': datetime.now().isoformat()
                }
            
            # 测试API响应
            test_data = await self._make_request('/v2/reference/markets')
            
            if test_data and test_data.get('status') == 'OK':
                return {
                    'status': 'healthy',
                    'message': 'Polygon API连接正常',
                    'api_calls_used': self.rate_limit_calls,
                    'cache_size': len(self.quote_cache),
                    'timestamp': datetime.now().isoformat()
                }
            else:
                return {
                    'status': 'unhealthy',
                    'message': 'Polygon API响应异常',
                    'timestamp': datetime.now().isoformat()
                }
                
        except Exception as e:
            return {
                'status': 'error',
                'message': f'健康检查失败: {str(e)}',
                'timestamp': datetime.now().isoformat()
            }

# 连接器工厂注册
def create_polygon_connector(config: Dict[str, Any]) -> PolygonConnector:
    """
    创建Polygon连接器实例
    
    Args:
        config: 配置字典，包含api_key等参数
        
    Returns:
        PolygonConnector实例
    """
    api_key = config.get('api_key')
    if not api_key:
        raise ValueError("Polygon连接器需要api_key配置")
    
    base_url = config.get('base_url', 'https://api.polygon.io')
    
    return PolygonConnector(api_key=api_key, base_url=base_url) 
 