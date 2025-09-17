"""
GMGN数据爬取模块

提供备用的数据爬取功能，当API不可用时使用:
- 热门代币数据爬取
- 新币列表爬取
- 交易对数据爬取
- 市场指标爬取
"""

import asyncio
import time
import re
from typing import Dict, List, Optional, Any, Set
from datetime import datetime
import logging

import aiohttp
from bs4 import BeautifulSoup

from .models import (
    ChainType, TokenInfo, TokenPair, GMGNMarketData,
    create_token_info_from_gmgn_data
)

logger = logging.getLogger(__name__)


class GMGNScraper:
    """
    GMGN网站数据爬取器
    
    备用数据获取方案，当API限制或不可用时使用网页爬取。
    注意：爬取功能仅作为备用方案，优先使用官方API。
    """
    
    def __init__(self, connector, enable_scraping: bool = False):
        """
        初始化GMGN爬取器
        
        Args:
            connector: GMGN连接器实例
            enable_scraping: 是否启用爬取功能 (默认关闭)
        """
        self.connector = connector
        self.enable_scraping = enable_scraping
        self.base_url = "https://gmgn.ai"
        
        # 爬取配置
        self.config = {
            'interval': connector.config.scraping_interval,
            'batch_size': connector.config.scraping_batch_size,
            'max_pages': 5,
            'timeout': 30
        }
        
        # 缓存和状态
        self._scraped_data: Dict[str, Any] = {}
        self._last_scrape_time: Dict[str, float] = {}
        self._user_agents = [
            'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36',
            'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
            'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36'
        ]
        
        if not enable_scraping:
            logger.warning("GMGN爬取功能已禁用，仅作为API备用方案")
        else:
            logger.info("GMGN爬取模块初始化完成")
    
    async def _check_rate_limit(self, scrape_type: str) -> bool:
        """
        检查爬取频率限制
        
        Args:
            scrape_type: 爬取类型
            
        Returns:
            bool: 是否允许爬取
        """
        if not self.enable_scraping:
            logger.warning("爬取功能已禁用")
            return False
        
        current_time = time.time()
        last_time = self._last_scrape_time.get(scrape_type, 0)
        
        if current_time - last_time < self.config['interval']:
            logger.debug(f"爬取频率限制: {scrape_type}")
            return False
        
        return True
    
    async def _make_scrape_request(
        self,
        url: str,
        params: Optional[Dict] = None,
        headers: Optional[Dict] = None
    ) -> Optional[str]:
        """
        发起爬取请求
        
        Args:
            url: 目标URL
            params: URL参数
            headers: 请求头
            
        Returns:
            Optional[str]: HTML内容，失败时返回None
        """
        try:
            # 准备请求头
            request_headers = {
                'User-Agent': self._user_agents[0],  # 使用第一个User-Agent
                'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8',
                'Accept-Language': 'en-US,en;q=0.5',
                'Accept-Encoding': 'gzip, deflate',
                'Connection': 'keep-alive',
                'Upgrade-Insecure-Requests': '1'
            }
            
            if headers:
                request_headers.update(headers)
            
            # 使用连接器的session
            if not self.connector.session:
                logger.error("连接器session未建立")
                return None
            
            async with self.connector.session.get(
                url, 
                params=params, 
                headers=request_headers
            ) as response:
                
                if response.status == 200:
                    content = await response.text()
                    logger.debug(f"爬取成功: {url}")
                    return content
                else:
                    logger.warning(f"爬取失败 {response.status}: {url}")
                    return None
                    
        except Exception as e:
            logger.error(f"爬取请求异常: {e}")
            return None
    
    async def scrape_trending_tokens(
        self,
        chain: ChainType = ChainType.SOLANA,
        limit: int = 50
    ) -> Optional[List[TokenInfo]]:
        """
        爬取热门代币列表
        
        Args:
            chain: 目标链
            limit: 返回数量限制
            
        Returns:
            Optional[List[TokenInfo]]: 热门代币列表
        """
        scrape_type = f"trending_{chain.value}"
        
        if not await self._check_rate_limit(scrape_type):
            # 返回缓存数据
            cached = self._scraped_data.get(scrape_type)
            if cached:
                return cached[:limit]
            return None
        
        try:
            url = f"{self.base_url}/boardroom/{chain.value}"
            
            logger.info(f"爬取{chain.value}热门代币")
            
            html_content = await self._make_scrape_request(url)
            if not html_content:
                return None
            
            # 解析HTML
            soup = BeautifulSoup(html_content, 'html.parser')
            
            # 寻找代币列表 (这里需要根据实际页面结构调整)
            token_elements = soup.find_all('div', class_=re.compile(r'token.*item|row.*token'))
            
            tokens = []
            for element in token_elements[:limit]:
                try:
                    # 提取代币信息 (需要根据实际HTML结构调整)
                    token_data = self._extract_token_from_element(element, chain)
                    if token_data:
                        tokens.append(token_data)
                except Exception as e:
                    logger.debug(f"解析代币元素失败: {e}")
                    continue
            
            # 更新缓存
            self._scraped_data[scrape_type] = tokens
            self._last_scrape_time[scrape_type] = time.time()
            
            logger.info(f"成功爬取 {len(tokens)} 个热门代币")
            return tokens
            
        except Exception as e:
            logger.error(f"爬取热门代币异常: {e}")
            return None
    
    async def scrape_new_tokens(
        self,
        chain: ChainType = ChainType.SOLANA,
        limit: int = 30
    ) -> Optional[List[TokenInfo]]:
        """
        爬取新币列表
        
        Args:
            chain: 目标链
            limit: 返回数量限制
            
        Returns:
            Optional[List[TokenInfo]]: 新币列表
        """
        scrape_type = f"new_{chain.value}"
        
        if not await self._check_rate_limit(scrape_type):
            cached = self._scraped_data.get(scrape_type)
            if cached:
                return cached[:limit]
            return None
        
        try:
            url = f"{self.base_url}/new/{chain.value}"
            
            logger.info(f"爬取{chain.value}新币列表")
            
            html_content = await self._make_scrape_request(url)
            if not html_content:
                return None
            
            soup = BeautifulSoup(html_content, 'html.parser')
            
            # 寻找新币列表
            token_elements = soup.find_all('div', class_=re.compile(r'new.*token|token.*new'))
            
            tokens = []
            for element in token_elements[:limit]:
                try:
                    token_data = self._extract_token_from_element(element, chain, is_new=True)
                    if token_data:
                        tokens.append(token_data)
                except Exception as e:
                    logger.debug(f"解析新币元素失败: {e}")
                    continue
            
            self._scraped_data[scrape_type] = tokens
            self._last_scrape_time[scrape_type] = time.time()
            
            logger.info(f"成功爬取 {len(tokens)} 个新币")
            return tokens
            
        except Exception as e:
            logger.error(f"爬取新币列表异常: {e}")
            return None
    
    async def scrape_market_overview(self) -> Optional[Dict[str, Any]]:
        """
        爬取市场概览数据
        
        Returns:
            Optional[Dict]: 市场概览数据
        """
        scrape_type = "market_overview"
        
        if not await self._check_rate_limit(scrape_type):
            cached = self._scraped_data.get(scrape_type)
            if cached:
                return cached
            return None
        
        try:
            url = f"{self.base_url}"
            
            logger.info("爬取市场概览数据")
            
            html_content = await self._make_scrape_request(url)
            if not html_content:
                return None
            
            soup = BeautifulSoup(html_content, 'html.parser')
            
            # 提取市场统计数据
            market_data = {}
            
            # 寻找统计元素 (需要根据实际页面调整)
            stats_elements = soup.find_all('div', class_=re.compile(r'stat|metric|overview'))
            
            for element in stats_elements:
                try:
                    label = element.find('span', class_=re.compile(r'label|title'))
                    value = element.find('span', class_=re.compile(r'value|number'))
                    
                    if label and value:
                        key = label.get_text(strip=True).lower().replace(' ', '_')
                        val = value.get_text(strip=True)
                        market_data[key] = self._parse_metric_value(val)
                        
                except Exception as e:
                    logger.debug(f"解析统计元素失败: {e}")
                    continue
            
            self._scraped_data[scrape_type] = market_data
            self._last_scrape_time[scrape_type] = time.time()
            
            logger.info(f"成功爬取市场概览: {len(market_data)} 项指标")
            return market_data
            
        except Exception as e:
            logger.error(f"爬取市场概览异常: {e}")
            return None
    
    def _extract_token_from_element(
        self, 
        element, 
        chain: ChainType, 
        is_new: bool = False
    ) -> Optional[TokenInfo]:
        """
        从HTML元素提取代币信息
        
        Args:
            element: BeautifulSoup元素
            chain: 链类型
            is_new: 是否为新币
            
        Returns:
            Optional[TokenInfo]: 代币信息
        """
        try:
            # 提取基础信息 (需要根据实际HTML结构调整)
            symbol = self._extract_text_by_class(element, ['symbol', 'ticker'])
            name = self._extract_text_by_class(element, ['name', 'title'])
            address = self._extract_text_by_class(element, ['address', 'contract'])
            
            if not (symbol and address):
                return None
            
            # 提取市场数据
            price = self._extract_number_by_class(element, ['price', 'value'])
            market_cap = self._extract_number_by_class(element, ['mcap', 'market_cap'])
            volume_24h = self._extract_number_by_class(element, ['volume', 'vol24h'])
            
            return TokenInfo(
                address=address,
                symbol=symbol,
                name=name or symbol,
                decimals=18,  # 默认精度
                chain=chain,
                price_usd=price,
                market_cap=market_cap,
                volume_24h=volume_24h,
                updated_at=datetime.now(),
                is_verified=False  # 爬取的数据默认未验证
            )
            
        except Exception as e:
            logger.debug(f"提取代币信息失败: {e}")
            return None
    
    def _extract_text_by_class(self, element, class_patterns: List[str]) -> Optional[str]:
        """
        通过class模式提取文本
        
        Args:
            element: BeautifulSoup元素
            class_patterns: class名称模式列表
            
        Returns:
            Optional[str]: 提取的文本
        """
        for pattern in class_patterns:
            found = element.find('span', class_=re.compile(pattern, re.I))
            if not found:
                found = element.find('div', class_=re.compile(pattern, re.I))
            
            if found:
                text = found.get_text(strip=True)
                if text:
                    return text
        
        return None
    
    def _extract_number_by_class(self, element, class_patterns: List[str]) -> Optional[float]:
        """
        通过class模式提取数字
        
        Args:
            element: BeautifulSoup元素
            class_patterns: class名称模式列表
            
        Returns:
            Optional[float]: 提取的数字
        """
        text = self._extract_text_by_class(element, class_patterns)
        if text:
            return self._parse_metric_value(text)
        return None
    
    def _parse_metric_value(self, value_str: str) -> Optional[float]:
        """
        解析度量值字符串
        
        Args:
            value_str: 值字符串 (如 "1.2K", "$50.3M")
            
        Returns:
            Optional[float]: 解析后的数值
        """
        try:
            # 移除货币符号和空格
            clean_str = re.sub(r'[,$%\s]', '', value_str.upper())
            
            # 处理K, M, B后缀
            if clean_str.endswith('K'):
                return float(clean_str[:-1]) * 1_000
            elif clean_str.endswith('M'):
                return float(clean_str[:-1]) * 1_000_000
            elif clean_str.endswith('B'):
                return float(clean_str[:-1]) * 1_000_000_000
            else:
                return float(clean_str)
                
        except (ValueError, IndexError):
            return None
    
    async def get_scraped_market_data(self) -> Optional[GMGNMarketData]:
        """
        获取聚合的爬取市场数据
        
        Returns:
            Optional[GMGNMarketData]: 聚合市场数据
        """
        try:
            # 获取各类数据
            trending_sol = await self.scrape_trending_tokens(ChainType.SOLANA, 20)
            trending_eth = await self.scrape_trending_tokens(ChainType.ETHEREUM, 10)
            new_tokens = await self.scrape_new_tokens(ChainType.SOLANA, 15)
            market_overview = await self.scrape_market_overview()
            
            # 合并热门代币
            all_trending = []
            if trending_sol:
                all_trending.extend(trending_sol)
            if trending_eth:
                all_trending.extend(trending_eth)
            
            # 创建市场数据对象
            market_data = GMGNMarketData(
                trending_tokens=all_trending,
                new_tokens=new_tokens or [],
                active_pairs=[],  # 爬取中暂不支持交易对数据
                updated_at=datetime.now()
            )
            
            # 添加概览统计
            if market_overview:
                market_data.total_volume_24h = market_overview.get('total_volume_24h')
                market_data.total_transactions_24h = market_overview.get('total_transactions_24h')
            
            logger.info(f"聚合市场数据完成: {len(all_trending)} 热门, {len(new_tokens or [])} 新币")
            return market_data
            
        except Exception as e:
            logger.error(f"获取聚合市场数据异常: {e}")
            return None
    
    def get_scraping_status(self) -> Dict[str, Any]:
        """
        获取爬取状态信息
        
        Returns:
            Dict[str, Any]: 状态信息
        """
        return {
            'enabled': self.enable_scraping,
            'config': self.config,
            'last_scrape_times': self._last_scrape_time,
            'cached_data_types': list(self._scraped_data.keys()),
            'cache_sizes': {
                key: len(value) if isinstance(value, list) else 1
                for key, value in self._scraped_data.items()
            }
        }
    
    def clear_cache(self, scrape_type: Optional[str] = None):
        """
        清理缓存数据
        
        Args:
            scrape_type: 指定清理的类型，None表示清理全部
        """
        if scrape_type:
            self._scraped_data.pop(scrape_type, None)
            self._last_scrape_time.pop(scrape_type, None)
            logger.info(f"已清理缓存: {scrape_type}")
        else:
            self._scraped_data.clear()
            self._last_scrape_time.clear()
            logger.info("已清理全部缓存")
    
    async def health_check(self) -> Dict[str, Any]:
        """
        爬取器健康检查
        
        Returns:
            Dict[str, Any]: 健康状态
        """
        try:
            # 测试访问首页
            start_time = time.time()
            html_content = await self._make_scrape_request(self.base_url)
            latency = time.time() - start_time
            
            is_healthy = html_content is not None and len(html_content) > 1000
            
            return {
                'healthy': is_healthy,
                'latency_ms': round(latency * 1000, 2),
                'enabled': self.enable_scraping,
                'last_check': datetime.now().isoformat()
            }
            
        except Exception as e:
            logger.error(f"爬取器健康检查异常: {e}")
            return {
                'healthy': False,
                'error': str(e),
                'enabled': self.enable_scraping,
                'last_check': datetime.now().isoformat()
            } 