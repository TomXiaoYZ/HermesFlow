"""
FRED (美联储经济数据) 连接器

支持功能：
- 宏观经济指标数据获取
- 多种经济数据序列
- 历史数据下载
- 数据频率转换
- 经济指标关联分析

数据来源: FRED API (https://fred.stlouisfed.org/docs/api/fred/)

作者: HermesFlow Team
创建时间: 2024年12月20日
"""

import asyncio
import aiohttp
import json
import logging
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any, Union
from dataclasses import dataclass, asdict
import pandas as pd
import re

from .base_connector import BaseConnector, ConnectionConfig

# 配置日志
logger = logging.getLogger(__name__)

@dataclass
class EconomicSeries:
    """经济数据序列模型"""
    series_id: str
    title: str
    units: str
    frequency: str
    seasonal_adjustment: str
    last_updated: datetime
    popularity: int
    group_popularity: int
    notes: str

@dataclass
class EconomicDataPoint:
    """经济数据点模型"""
    series_id: str
    date: datetime
    value: float
    realtime_start: datetime
    realtime_end: datetime

@dataclass
class EconomicIndicator:
    """经济指标模型"""
    name: str
    series_id: str
    category: str
    description: str
    unit: str
    frequency: str
    data_points: List[EconomicDataPoint]
    last_value: float
    last_update: datetime

class FREDConnector(BaseConnector):
    """
    FRED API连接器
    
    提供美联储经济数据的获取和分析功能
    """
    
    # 常用经济指标定义
    ECONOMIC_INDICATORS = {
        # 利率和货币政策
        'federal_funds_rate': {
            'series_id': 'FEDFUNDS',
            'name': '联邦基金利率',
            'category': 'interest_rates',
            'description': '美联储基准利率'
        },
        '10y_treasury': {
            'series_id': 'GS10',
            'name': '10年期国债收益率',
            'category': 'interest_rates',
            'description': '10年期美国国债收益率'
        },
        '2y_treasury': {
            'series_id': 'GS2',
            'name': '2年期国债收益率',
            'category': 'interest_rates',
            'description': '2年期美国国债收益率'
        },
        'yield_curve_spread': {
            'series_id': 'T10Y2Y',
            'name': '收益率曲线利差',
            'category': 'interest_rates',
            'description': '10年期与2年期国债收益率利差'
        },
        
        # 通胀数据
        'cpi': {
            'series_id': 'CPIAUCSL',
            'name': '消费者价格指数',
            'category': 'inflation',
            'description': '所有城市消费者CPI，季节性调整'
        },
        'core_cpi': {
            'series_id': 'CPILFESL',
            'name': '核心CPI',
            'category': 'inflation',
            'description': '核心CPI（排除食品和能源）'
        },
        'pce': {
            'series_id': 'PCEPI',
            'name': '个人消费支出价格指数',
            'category': 'inflation',
            'description': 'PCE价格指数'
        },
        'core_pce': {
            'series_id': 'PCEPILFE',
            'name': '核心PCE',
            'category': 'inflation',
            'description': '核心PCE价格指数'
        },
        
        # 就业数据
        'unemployment_rate': {
            'series_id': 'UNRATE',
            'name': '失业率',
            'category': 'employment',
            'description': '美国失业率，季节性调整'
        },
        'nonfarm_payrolls': {
            'series_id': 'PAYEMS',
            'name': '非农就业人数',
            'category': 'employment',
            'description': '非农就业总人数，季节性调整'
        },
        'labor_force_participation': {
            'series_id': 'CIVPART',
            'name': '劳动参与率',
            'category': 'employment',
            'description': '劳动力参与率'
        },
        
        # GDP和经济增长
        'gdp': {
            'series_id': 'GDP',
            'name': '国内生产总值',
            'category': 'gdp',
            'description': '实际GDP，季节性调整年化率'
        },
        'gdp_growth': {
            'series_id': 'A191RL1Q225SBEA',
            'name': 'GDP增长率',
            'category': 'gdp',
            'description': '实际GDP增长率（年化）'
        },
        
        # 制造业和商业
        'ism_manufacturing': {
            'series_id': 'NAPM',
            'name': 'ISM制造业指数',
            'category': 'manufacturing',
            'description': 'ISM制造业采购经理指数'
        },
        'industrial_production': {
            'series_id': 'INDPRO',
            'name': '工业生产指数',
            'category': 'manufacturing',
            'description': '工业生产指数，季节性调整'
        },
        
        # 消费者信心和支出
        'consumer_sentiment': {
            'series_id': 'UMCSENT',
            'name': '消费者信心指数',
            'category': 'consumer',
            'description': '密歇根大学消费者信心指数'
        },
        'retail_sales': {
            'series_id': 'RSAFS',
            'name': '零售销售',
            'category': 'consumer',
            'description': '零售销售总额，季节性调整'
        },
        
        # 房地产市场
        'housing_starts': {
            'series_id': 'HOUST',
            'name': '新屋开工',
            'category': 'housing',
            'description': '新房开工数量，季节性调整'
        },
        'home_sales': {
            'series_id': 'EXHOSLUSM495S',
            'name': '成屋销售',
            'category': 'housing',
            'description': '成屋销售数量'
        },
        
        # 货币供应量
        'm1_money_supply': {
            'series_id': 'M1SL',
            'name': 'M1货币供应量',
            'category': 'money_supply',
            'description': 'M1货币供应量，季节性调整'
        },
        'm2_money_supply': {
            'series_id': 'M2SL',
            'name': 'M2货币供应量',
            'category': 'money_supply',
            'description': 'M2货币供应量，季节性调整'
        }
    }
    
    def __init__(self, config: ConnectionConfig):
        """
        初始化FRED连接器
        
        Args:
            config: 连接配置，API密钥通过config.api_key传递
        """
        super().__init__(config, "fred")
        self.api_key = config.api_key or "demo_key"  # 使用演示密钥如果没有提供
        self.base_url = getattr(config, 'base_url', 'https://api.stlouisfed.org/fred')
        self.session: Optional[aiohttp.ClientSession] = None
        
        # 数据缓存
        self.series_cache: Dict[str, EconomicSeries] = {}
        self.data_cache: Dict[str, List[EconomicDataPoint]] = {}
        
        logger.info(f"FRED连接器初始化完成，API Key: {self.api_key[:8] if self.api_key != 'demo_key' else 'demo_key'}...")

    async def connect(self) -> bool:
        """建立与FRED API的连接"""
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
                    'User-Agent': 'HermesFlow-FREDConnector/1.0'
                }
            )
            
            # 测试API连接
            test_response = await self._make_request('/category', {'category_id': '0'})
            if test_response and 'categories' in test_response:
                logger.info("FRED API连接测试成功")
                self.is_connected = True
                return True
            else:
                logger.error("FRED API连接测试失败")
                return False
                
        except Exception as e:
            logger.error(f"连接FRED API失败: {e}")
            return False

    async def disconnect(self):
        """断开连接并清理资源"""
        try:
            if self.session:
                await self.session.close()
                self.session = None
            
            self.is_connected = False
            logger.info("FRED连接器已断开")
            
        except Exception as e:
            logger.error(f"断开FRED连接时出错: {e}")

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
        
        url = f"{self.base_url}{endpoint}"
        
        # 添加API密钥和默认参数
        if params is None:
            params = {}
        
        params.update({
            'api_key': self.api_key,
            'file_type': 'json'
        })
        
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
                    error_text = await response.text()
                    logger.error(f"错误响应: {error_text}")
                    return None
                    
        except Exception as e:
            logger.error(f"发起API请求时出错: {e}")
            return None

    async def get_series_info(self, series_id: str) -> Optional[EconomicSeries]:
        """
        获取经济数据序列信息
        
        Args:
            series_id: 数据序列ID
            
        Returns:
            经济数据序列信息或None
        """
        try:
            # 检查缓存
            if series_id in self.series_cache:
                return self.series_cache[series_id]
            
            # 从API获取
            endpoint = '/series'
            params = {'series_id': series_id}
            
            response = await self._make_request(endpoint, params)
            if not response or 'seriess' not in response:
                logger.warning(f"获取序列 {series_id} 信息失败")
                return None
            
            series_data = response['seriess'][0]
            
            series = EconomicSeries(
                series_id=series_data.get('id'),
                title=series_data.get('title'),
                units=series_data.get('units'),
                frequency=series_data.get('frequency'),
                seasonal_adjustment=series_data.get('seasonal_adjustment'),
                last_updated=self._parse_fred_datetime(series_data.get('last_updated')),
                popularity=series_data.get('popularity', 0),
                group_popularity=series_data.get('group_popularity', 0),
                notes=series_data.get('notes', '')
            )
            
            # 缓存数据
            self.series_cache[series_id] = series
            
            logger.debug(f"获取序列 {series_id} 信息成功")
            return series
            
        except Exception as e:
            logger.error(f"获取序列 {series_id} 信息时出错: {e}")
            return None

    async def get_series_data(self, series_id: str, start_date: str = None, 
                             end_date: str = None, limit: int = None) -> List[EconomicDataPoint]:
        """
        获取经济数据序列数据
        
        Args:
            series_id: 数据序列ID
            start_date: 开始日期 (YYYY-MM-DD)
            end_date: 结束日期 (YYYY-MM-DD)
            limit: 数据点数量限制
            
        Returns:
            经济数据点列表
        """
        try:
            # 构建缓存键
            cache_key = f"{series_id}_{start_date}_{end_date}_{limit}"
            
            # 检查缓存
            if cache_key in self.data_cache:
                cached_data = self.data_cache[cache_key]
                # 检查缓存是否过期 (1小时)
                if cached_data and (datetime.now() - cached_data[0].realtime_start).seconds < 3600:
                    return cached_data
            
            # 从API获取
            endpoint = '/series/observations'
            params = {'series_id': series_id}
            
            if start_date:
                params['observation_start'] = start_date
            if end_date:
                params['observation_end'] = end_date
            if limit:
                params['limit'] = limit
            
            response = await self._make_request(endpoint, params)
            if not response or 'observations' not in response:
                logger.warning(f"获取序列 {series_id} 数据失败")
                return []
            
            data_points = []
            for obs in response['observations']:
                try:
                    value = float(obs['value'])
                except (ValueError, TypeError):
                    continue  # 跳过无效数据点
                
                data_point = EconomicDataPoint(
                    series_id=series_id,
                    date=self._parse_fred_datetime(obs['date']),
                    value=value,
                    realtime_start=self._parse_fred_datetime(obs['realtime_start']),
                    realtime_end=self._parse_fred_datetime(obs['realtime_end'])
                )
                data_points.append(data_point)
            
            # 缓存数据
            self.data_cache[cache_key] = data_points
            
            logger.info(f"获取序列 {series_id} 数据成功: {len(data_points)} 个数据点")
            return data_points
            
        except Exception as e:
            logger.error(f"获取序列 {series_id} 数据时出错: {e}")
            return []

    async def get_economic_indicator(self, indicator_name: str, 
                                   start_date: str = None, end_date: str = None) -> Optional[EconomicIndicator]:
        """
        获取预定义的经济指标数据
        
        Args:
            indicator_name: 指标名称 (如 'federal_funds_rate', 'cpi')
            start_date: 开始日期 (YYYY-MM-DD)
            end_date: 结束日期 (YYYY-MM-DD)
            
        Returns:
            经济指标数据或None
        """
        try:
            if indicator_name not in self.ECONOMIC_INDICATORS:
                logger.error(f"未知的经济指标: {indicator_name}")
                return None
            
            indicator_info = self.ECONOMIC_INDICATORS[indicator_name]
            series_id = indicator_info['series_id']
            
            # 获取序列信息
            series_info = await self.get_series_info(series_id)
            if not series_info:
                return None
            
            # 获取数据
            data_points = await self.get_series_data(series_id, start_date, end_date)
            if not data_points:
                return None
            
            # 构建经济指标对象
            indicator = EconomicIndicator(
                name=indicator_info['name'],
                series_id=series_id,
                category=indicator_info['category'],
                description=indicator_info['description'],
                unit=series_info.units,
                frequency=series_info.frequency,
                data_points=data_points,
                last_value=data_points[-1].value if data_points else 0.0,
                last_update=data_points[-1].date if data_points else datetime.now()
            )
            
            logger.info(f"获取经济指标 {indicator_name} 成功")
            return indicator
            
        except Exception as e:
            logger.error(f"获取经济指标 {indicator_name} 时出错: {e}")
            return None

    async def get_multiple_indicators(self, indicator_names: List[str], 
                                    start_date: str = None, end_date: str = None) -> Dict[str, EconomicIndicator]:
        """
        批量获取多个经济指标
        
        Args:
            indicator_names: 指标名称列表
            start_date: 开始日期
            end_date: 结束日期
            
        Returns:
            指标名称到指标数据的映射
        """
        try:
            results = {}
            
            # 并发获取多个指标
            tasks = []
            for name in indicator_names:
                task = self.get_economic_indicator(name, start_date, end_date)
                tasks.append((name, task))
            
            # 等待所有任务完成
            for name, task in tasks:
                try:
                    indicator = await task
                    if indicator:
                        results[name] = indicator
                except Exception as e:
                    logger.error(f"获取指标 {name} 时出错: {e}")
            
            logger.info(f"批量获取 {len(results)}/{len(indicator_names)} 个指标成功")
            return results
            
        except Exception as e:
            logger.error(f"批量获取指标时出错: {e}")
            return {}

    async def search_series(self, search_text: str, limit: int = 100) -> List[EconomicSeries]:
        """
        搜索经济数据序列
        
        Args:
            search_text: 搜索关键词
            limit: 结果数量限制
            
        Returns:
            匹配的序列列表
        """
        try:
            endpoint = '/series/search'
            params = {
                'search_text': search_text,
                'limit': limit,
                'order_by': 'popularity',
                'sort_order': 'desc'
            }
            
            response = await self._make_request(endpoint, params)
            if not response or 'seriess' not in response:
                logger.warning(f"搜索 '{search_text}' 失败")
                return []
            
            series_list = []
            for series_data in response['seriess']:
                try:
                    series = EconomicSeries(
                        series_id=series_data.get('id'),
                        title=series_data.get('title'),
                        units=series_data.get('units'),
                        frequency=series_data.get('frequency'),
                        seasonal_adjustment=series_data.get('seasonal_adjustment', ''),
                        last_updated=self._parse_fred_datetime(series_data.get('last_updated')),
                        popularity=series_data.get('popularity', 0),
                        group_popularity=series_data.get('group_popularity', 0),
                        notes=series_data.get('notes', '')
                    )
                    series_list.append(series)
                except Exception as e:
                    logger.warning(f"解析序列数据时出错: {e}")
                    continue
            
            logger.info(f"搜索 '{search_text}' 找到 {len(series_list)} 个序列")
            return series_list
            
        except Exception as e:
            logger.error(f"搜索序列时出错: {e}")
            return []

    def get_available_indicators(self) -> Dict[str, Dict[str, Any]]:
        """获取所有可用的预定义经济指标"""
        return self.ECONOMIC_INDICATORS.copy()

    def get_indicators_by_category(self, category: str) -> Dict[str, Dict[str, Any]]:
        """
        根据类别获取经济指标
        
        Args:
            category: 类别名称 ('interest_rates', 'inflation', 'employment', 等)
            
        Returns:
            该类别下的指标字典
        """
        return {
            name: info for name, info in self.ECONOMIC_INDICATORS.items()
            if info['category'] == category
        }

    # 实现BaseConnector的抽象方法
    async def get_symbols(self) -> List[str]:
        """获取支持的经济指标列表"""
        return list(self.ECONOMIC_INDICATORS.keys())
    
    async def get_klines(self, symbol: str, interval: str, start_time=None, end_time=None, limit: int = 500):
        """经济数据连接器不支持K线数据"""
        raise NotImplementedError("经济数据连接器不支持K线数据")
    
    async def get_ticker(self, symbol: str):
        """获取经济指标的最新值"""
        if symbol in self.ECONOMIC_INDICATORS:
            indicator = await self.get_economic_indicator(symbol)
            if indicator:
                return {
                    'symbol': symbol,
                    'value': indicator.last_value,
                    'timestamp': indicator.last_update
                }
        return None
    
    async def get_orderbook(self, symbol: str, depth: int = 20):
        """经济数据连接器不支持订单簿数据"""
        raise NotImplementedError("经济数据连接器不支持订单簿数据")
    
    async def subscribe_real_time(self, symbols: List[str], data_types, callback):
        """经济数据连接器不支持实时订阅"""
        raise NotImplementedError("经济数据连接器不支持实时订阅")
    
    async def unsubscribe_real_time(self, symbols: List[str], data_types):
        """经济数据连接器不支持实时订阅"""
        raise NotImplementedError("经济数据连接器不支持实时订阅")

    # BaseConnector接口实现
    async def get_ticker_info(self, symbol: str) -> Dict[str, Any]:
        """获取经济指标信息 (将symbol视为indicator_name)"""
        indicator = await self.get_economic_indicator(symbol)
        if indicator:
            return asdict(indicator)
        return {}

    async def get_market_data(self, symbol: str, data_type: str = 'indicator') -> Dict[str, Any]:
        """获取经济数据"""
        if data_type == 'indicator':
            return await self.get_ticker_info(symbol)
        elif data_type == 'series':
            series_info = await self.get_series_info(symbol)
            if series_info:
                return asdict(series_info)
        return {}

    async def subscribe_realtime_data(self, symbol: str, callback=None):
        """订阅实时数据 (FRED不提供实时数据)"""
        logger.info(f"FRED不提供实时数据订阅: {symbol}")
        pass

    async def get_account_info(self) -> Dict[str, Any]:
        """获取账户信息 (FRED不提供交易功能)"""
        return {'error': 'FRED仅提供经济数据，不支持账户功能'}

    def get_supported_symbols(self) -> List[str]:
        """获取支持的经济指标名称"""
        return list(self.ECONOMIC_INDICATORS.keys())

    def get_supported_markets(self) -> List[str]:
        """获取支持的数据类别"""
        categories = set()
        for info in self.ECONOMIC_INDICATORS.values():
            categories.add(info['category'])
        return list(categories)

    # 健康检查和诊断
    async def health_check(self) -> Dict[str, Any]:
        """检查连接器健康状态"""
        try:
            if not self.is_connected:
                return {
                    'status': 'unhealthy',
                    'message': '未连接到FRED API',
                    'timestamp': datetime.now().isoformat()
                }
            
            # 测试API响应
            test_data = await self._make_request('/category', {'category_id': '0'})
            
            if test_data and 'categories' in test_data:
                return {
                    'status': 'healthy',
                    'message': 'FRED API连接正常',
                    'series_cache_size': len(self.series_cache),
                    'data_cache_size': len(self.data_cache),
                    'available_indicators': len(self.ECONOMIC_INDICATORS),
                    'timestamp': datetime.now().isoformat()
                }
            else:
                return {
                    'status': 'unhealthy',
                    'message': 'FRED API响应异常',
                    'timestamp': datetime.now().isoformat()
                }
                
        except Exception as e:
            return {
                'status': 'error',
                'message': f'健康检查失败: {str(e)}',
                'timestamp': datetime.now().isoformat()
            }

    def _parse_fred_datetime(self, date_str: str) -> datetime:
        """
        解析FRED API返回的日期时间格式
        
        Args:
            date_str: FRED API返回的日期字符串
            
        Returns:
            解析后的datetime对象
        """
        try:
            # 处理带时区的格式: '2025-05-01 16:37:08-05'
            if ' ' in date_str and ('-' in date_str.split(' ')[1] or '+' in date_str.split(' ')[1]):
                # 移除时区信息，只保留日期和时间
                date_part = date_str.split(' ')[0]
                time_part = date_str.split(' ')[1]
                # 移除时区部分
                time_clean = re.sub(r'[+-]\d{2}$', '', time_part)
                clean_str = f"{date_part} {time_clean}"
                return datetime.strptime(clean_str, '%Y-%m-%d %H:%M:%S')
            
            # 处理ISO格式: '2025-05-01T16:37:08Z'
            elif 'T' in date_str:
                clean_str = date_str.replace('Z', '+00:00')
                return datetime.fromisoformat(clean_str.replace('+00:00', ''))
            
            # 处理简单日期格式: '2025-05-01'
            else:
                return datetime.strptime(date_str, '%Y-%m-%d')
                
        except Exception as e:
            logger.warning(f"日期解析失败 '{date_str}': {e}, 使用当前时间")
            return datetime.now()

# 连接器工厂注册
def create_fred_connector(config: Dict[str, Any]) -> FREDConnector:
    """
    创建FRED连接器实例
    
    Args:
        config: 配置字典，包含api_key等参数
        
    Returns:
        FREDConnector实例
    """
    api_key = config.get('api_key')
    if not api_key:
        raise ValueError("FRED连接器需要api_key配置")
    
    base_url = config.get('base_url', 'https://api.stlouisfed.org/fred')
    
    return FREDConnector(api_key=api_key, base_url=base_url) 
 