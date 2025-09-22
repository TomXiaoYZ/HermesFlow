"""
Sentix情绪指数连接器

Sentix是德国领先的投资者情绪调研机构，提供：
- 多市场投资者情绪指数和研究报告
- 股票、大宗商品、外汇、加密货币情绪数据
- 每日/每周情绪指数更新
- 专业投资者行为分析

数据来源: https://sentix.de/
API文档: https://sentix.de/api-documentation (需要订阅)

作者: HermesFlow Team
创建时间: 2024年12月21日
"""

import asyncio
import aiohttp
import logging
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any, Union
from dataclasses import dataclass

from ..base_connector import BaseConnector, ConnectionConfig, ConnectionStatus, DataType
from ...models.sentiment_data import (
    SentimentData, SentimentScore, SentimentPolarity, DataSource, 
    ContentType, MarketSentimentIndex
)

# 配置日志
logger = logging.getLogger(__name__)

@dataclass
class SentixConfig(ConnectionConfig):
    """Sentix连接器配置"""
    api_key: str
    base_url: str = "https://api.sentix.de/v1"
    timeout: int = 30
    rate_limit_requests: int = 100  # 每小时请求限制
    rate_limit_window: int = 3600   # 时间窗口(秒)

class SentixConnector(BaseConnector):
    """
    Sentix情绪指数连接器
    
    提供专业投资者情绪数据接入功能：
    - 情绪指数查询
    - 历史情绪数据
    - 多市场情绪对比
    - 专业投资者行为分析
    """
    
    def __init__(self, config: SentixConfig):
        """
        初始化Sentix连接器
        
        Args:
            config: Sentix连接器配置
        """
        super().__init__(config, "sentix")
        self.config = config
        self._session: Optional[aiohttp.ClientSession] = None
        self._last_request_time = 0
        self._request_count = 0
        
        # Sentix支持的情绪指数类型
        self.SENTIMENT_INDICES = {
            'overall': '总体情绪指数',
            'stocks': '股票市场情绪',
            'currencies': '外汇市场情绪',
            'commodities': '大宗商品情绪',
            'crypto': '加密货币情绪',
            'bonds': '债券市场情绪'
        }
        
        # 支持的时间周期
        self.TIMEFRAMES = {
            'daily': '每日',
            'weekly': '每周',
            'monthly': '每月'
        }
        
    async def connect(self) -> bool:
        """建立连接"""
        try:
            if self._session is None:
                timeout = aiohttp.ClientTimeout(total=self.config.timeout)
                self._session = aiohttp.ClientSession(
                    timeout=timeout,
                    headers={
                        'User-Agent': 'HermesFlow/1.0',
                        'Authorization': f'Bearer {self.config.api_key}',
                        'Content-Type': 'application/json'
                    }
                )
            
            # 测试API连接
            test_result = await self._test_connection()
            
            if test_result:
                self.status = ConnectionStatus.CONNECTED
                logger.info("Sentix连接器连接成功")
                return True
            else:
                self.status = ConnectionStatus.ERROR
                logger.error("Sentix连接器连接失败")
                return False
                
        except Exception as e:
            logger.error(f"Sentix连接建立失败: {e}")
            self.status = ConnectionStatus.ERROR
            return False
    
    async def disconnect(self) -> bool:
        """断开连接"""
        try:
            if self._session:
                await self._session.close()
                self._session = None
            
            self.status = ConnectionStatus.DISCONNECTED
            logger.info("Sentix连接器已断开")
            return True
            
        except Exception as e:
            logger.error(f"Sentix连接断开失败: {e}")
            return False
    
    async def _test_connection(self) -> bool:
        """测试API连接"""
        try:
            url = f"{self.config.base_url}/health"
            async with self._session.get(url) as response:
                if response.status == 200:
                    return True
                else:
                    logger.error(f"Sentix API健康检查失败，状态码: {response.status}")
                    return False
                    
        except Exception as e:
            logger.error(f"Sentix连接测试失败: {e}")
            return False
    
    async def _rate_limit_check(self):
        """速率限制检查"""
        now = datetime.now().timestamp()
        
        # 重置计数器（如果超过时间窗口）
        if now - self._last_request_time > self.config.rate_limit_window:
            self._request_count = 0
            self._last_request_time = now
        
        # 检查是否超过限制
        if self._request_count >= self.config.rate_limit_requests:
            wait_time = self.config.rate_limit_window - (now - self._last_request_time)
            if wait_time > 0:
                logger.warning(f"Sentix API速率限制，等待 {wait_time:.2f} 秒")
                await asyncio.sleep(wait_time)
                self._request_count = 0
                self._last_request_time = datetime.now().timestamp()
        
        self._request_count += 1
    
    async def _make_request(self, endpoint: str, params: Dict[str, Any] = None) -> Dict[str, Any]:
        """发起API请求"""
        await self._rate_limit_check()
        
        url = f"{self.config.base_url}/{endpoint}"
        
        try:
            async with self._session.get(url, params=params) as response:
                if response.status == 200:
                    return await response.json()
                elif response.status == 401:
                    raise Exception("Sentix API认证失败，请检查API密钥")
                elif response.status == 429:
                    raise Exception("Sentix API请求过于频繁")
                else:
                    raise Exception(f"Sentix API请求失败，状态码: {response.status}")
                    
        except aiohttp.ClientError as e:
            raise Exception(f"Sentix API网络请求失败: {e}")
    
    async def get_sentiment_index(self, index_type: str = 'overall', timeframe: str = 'daily') -> Optional[MarketSentimentIndex]:
        """
        获取情绪指数
        
        Args:
            index_type: 指数类型 (overall, stocks, currencies, commodities, crypto, bonds)
            timeframe: 时间周期 (daily, weekly, monthly)
            
        Returns:
            MarketSentimentIndex对象或None
        """
        if not self._session:
            await self.connect()
        
        try:
            params = {
                'type': index_type,
                'timeframe': timeframe
            }
            
            data = await self._make_request('sentiment/index', params)
            
            # 转换为标准格式
            sentiment_index = MarketSentimentIndex(
                symbol=f"SENTIX_{index_type.upper()}",
                index_value=data.get('value', 0),
                calculation_method='sentix_professional_survey',
                component_scores={
                    'individual_investors': data.get('individual_investors', 0),
                    'institutional_investors': data.get('institutional_investors', 0),
                    'overall_sentiment': data.get('overall_sentiment', 0)
                },
                data_sources_weights={DataSource.SENTIX: 1.0},
                previous_value=data.get('previous_value'),
                change_24h=data.get('change_24h'),
                change_7d=data.get('change_7d'),
                sample_size=data.get('sample_size', 0),
                data_quality=0.95  # Sentix数据质量很高
            )
            
            logger.info(f"获取Sentix情绪指数成功: {index_type}")
            return sentiment_index
            
        except Exception as e:
            logger.error(f"获取Sentix情绪指数失败: {e}")
            return None
    
    async def get_historical_sentiment(self, index_type: str, start_date: datetime, end_date: datetime) -> List[Dict[str, Any]]:
        """
        获取历史情绪数据
        
        Args:
            index_type: 指数类型
            start_date: 开始日期
            end_date: 结束日期
            
        Returns:
            历史情绪数据列表
        """
        if not self._session:
            await self.connect()
        
        try:
            params = {
                'type': index_type,
                'start_date': start_date.strftime('%Y-%m-%d'),
                'end_date': end_date.strftime('%Y-%m-%d')
            }
            
            data = await self._make_request('sentiment/historical', params)
            
            logger.info(f"获取Sentix历史情绪数据成功: {len(data.get('data', []))} 条记录")
            return data.get('data', [])
            
        except Exception as e:
            logger.error(f"获取Sentix历史情绪数据失败: {e}")
            return []
    
    async def get_market_comparison(self) -> Dict[str, float]:
        """
        获取多市场情绪对比
        
        Returns:
            各市场情绪指数字典
        """
        if not self._session:
            await self.connect()
        
        try:
            comparison_data = {}
            
            # 获取各市场情绪指数
            for index_type in self.SENTIMENT_INDICES.keys():
                sentiment_index = await self.get_sentiment_index(index_type)
                if sentiment_index:
                    comparison_data[index_type] = sentiment_index.index_value
                    
                # 添加小延迟避免频率限制
                await asyncio.sleep(0.1)
            
            logger.info(f"获取Sentix市场对比数据成功: {len(comparison_data)} 个市场")
            return comparison_data
            
        except Exception as e:
            logger.error(f"获取Sentix市场对比数据失败: {e}")
            return {}
    
    async def get_investor_behavior_analysis(self, market: str = 'stocks') -> Dict[str, Any]:
        """
        获取投资者行为分析
        
        Args:
            market: 市场类型
            
        Returns:
            投资者行为分析数据
        """
        if not self._session:
            await self.connect()
        
        try:
            params = {'market': market}
            data = await self._make_request('sentiment/behavior', params)
            
            logger.info(f"获取Sentix投资者行为分析成功: {market}")
            return data
            
        except Exception as e:
            logger.error(f"获取Sentix投资者行为分析失败: {e}")
            return {}
    
    # 实现BaseConnector抽象方法
    async def get_symbols(self) -> List[str]:
        """获取支持的情绪指数类型"""
        return list(self.SENTIMENT_INDICES.keys())
    
    async def get_klines(self, symbol: str, interval: str, start_time=None, end_time=None, limit: int = 500):
        """情绪连接器不支持K线数据"""
        raise NotImplementedError("情绪连接器不支持K线数据")
    
    async def get_ticker(self, symbol: str):
        """获取情绪指数当前值"""
        sentiment_index = await self.get_sentiment_index(symbol)
        if sentiment_index:
            return {
                'symbol': symbol,
                'value': sentiment_index.index_value,
                'timestamp': sentiment_index.calculation_timestamp
            }
        return None
    
    async def get_orderbook(self, symbol: str, depth: int = 20):
        """情绪连接器不支持订单簿数据"""
        self.logger.warning("Sentix情绪连接器不支持订单簿数据")
        return None
    
    async def subscribe_real_time(self, symbols: List[str], data_types: List[DataType], callback: callable) -> bool:
        """情绪连接器不支持实时订阅"""
        self.logger.warning("Sentix情绪连接器不支持实时订阅")
        return False
    
    async def unsubscribe_real_time(self, symbols: List[str], data_types: List[DataType]) -> bool:
        """情绪连接器不支持实时订阅"""
        self.logger.warning("Sentix情绪连接器不支持实时订阅")
        return False
    
    async def health_check(self) -> Dict[str, Any]:
        """健康检查"""
        try:
            if not self._session:
                await self.connect()
            
            # 测试基本连接
            test_result = await self._test_connection()
            
            # 测试获取数据
            test_data = await self.get_sentiment_index('overall')
            
            return {
                'status': 'healthy' if test_result and test_data else 'unhealthy',
                'connection': test_result,
                'data_access': test_data is not None,
                'supported_indices': len(self.SENTIMENT_INDICES),
                'last_check': datetime.now().isoformat(),
                'api_status': 'active' if test_result else 'inactive'
            }
            
        except Exception as e:
            logger.error(f"Sentix健康检查失败: {e}")
            return {
                'status': 'unhealthy',
                'error': str(e),
                'last_check': datetime.now().isoformat()
            }

# 工厂函数
def create_sentix_connector(api_key: str, **kwargs) -> SentixConnector:
    """
    创建Sentix连接器实例
    
    Args:
        api_key: Sentix API密钥
        **kwargs: 其他配置参数
        
    Returns:
        SentixConnector实例
    """
    config = SentixConfig(
        api_key=api_key,
        **kwargs
    )
    return SentixConnector(config) 