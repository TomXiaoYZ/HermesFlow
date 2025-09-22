"""
NewsAPI新闻聚合连接器

NewsAPI.org提供全球新闻聚合服务，包括：
- 80,000+新闻源实时聚合
- 多语言新闻内容和分类
- 关键词搜索和实时筛选  
- 新闻源可信度评估
- 实时新闻事件监控

数据来源: https://newsapi.org/
API文档: https://newsapi.org/docs

作者: HermesFlow Team
创建时间: 2024年12月21日
"""

import asyncio
import aiohttp
import logging
import re
import hashlib
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any, Union
from dataclasses import dataclass
from urllib.parse import urlencode

from ..base_connector import BaseConnector, ConnectionConfig, ConnectionStatus, DataType
from ...models.sentiment_data import (
    SentimentData, SentimentScore, SentimentPolarity, DataSource, 
    ContentType, NewsArticle, EntityMention, KeywordInfo
)

# 配置日志
logger = logging.getLogger(__name__)

@dataclass
class NewsAPIConfig(ConnectionConfig):
    """NewsAPI连接器配置"""
    api_key: str
    base_url: str = "https://newsapi.org/v2"
    timeout: int = 30
    rate_limit_requests: int = 1000  # 每日请求限制 (免费版)
    rate_limit_window: int = 86400   # 时间窗口(秒) - 24小时
    default_language: str = "en"
    default_page_size: int = 20
    max_page_size: int = 100

class NewsAPIConnector(BaseConnector):
    """
    NewsAPI新闻聚合连接器
    
    提供全球新闻数据接入功能：
    - 实时新闻聚合
    - 新闻搜索和筛选
    - 多语言新闻处理
    - 源可信度评估
    """
    
    def __init__(self, config: NewsAPIConfig):
        """
        初始化NewsAPI连接器
        
        Args:
            config: NewsAPI连接器配置
        """
        super().__init__(config, "newsapi")
        self.config = config
        self._session: Optional[aiohttp.ClientSession] = None
        self._request_count = 0
        self._request_reset_time = datetime.now()
        
        # 支持的新闻类别
        self.NEWS_CATEGORIES = {
            'business': '商业财经',
            'entertainment': '娱乐',
            'general': '综合新闻',
            'health': '健康医疗',
            'science': '科学技术',
            'sports': '体育运动',
            'technology': '科技'
        }
        
        # 支持的国家/地区
        self.SUPPORTED_COUNTRIES = {
            'us': '美国', 'gb': '英国', 'ca': '加拿大', 'au': '澳大利亚',
            'de': '德国', 'fr': '法国', 'jp': '日本', 'cn': '中国',
            'kr': '韩国', 'in': '印度', 'br': '巴西', 'ru': '俄罗斯'
        }
        
        # 支持的语言
        self.SUPPORTED_LANGUAGES = {
            'en': '英语', 'zh': '中文', 'es': '西班牙语', 'fr': '法语',
            'de': '德语', 'it': '意大利语', 'pt': '葡萄牙语', 'ru': '俄语',
            'ja': '日语', 'ko': '韩语', 'ar': '阿拉伯语'
        }
        
        # 可信度评估权重（基于新闻源质量）
        self.SOURCE_CREDIBILITY = {
            'reuters.com': 0.95,
            'bloomberg.com': 0.93,
            'wsj.com': 0.92,
            'ft.com': 0.91,
            'cnbc.com': 0.89,
            'bbc.com': 0.90,
            'cnn.com': 0.85,
            'nytimes.com': 0.88,
            'washingtonpost.com': 0.87,
            'apnews.com': 0.92,
            'theguardian.com': 0.86
        }
        
        # 金融关键词模式
        self.FINANCIAL_KEYWORDS = {
            'stocks': r'\b(stock|share|equity|securities)\b',
            'crypto': r'\b(bitcoin|cryptocurrency|crypto|blockchain|ethereum)\b',
            'market': r'\b(market|trading|exchange|nasdaq|nyse|dow)\b',
            'economy': r'\b(economy|economic|gdp|inflation|recession|growth)\b',
            'finance': r'\b(finance|financial|bank|investment|fund)\b'
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
                        'X-API-Key': self.config.api_key,
                        'Content-Type': 'application/json'
                    }
                )
            
            # 测试API连接
            test_result = await self._test_connection()
            
            if test_result:
                self.status = ConnectionStatus.CONNECTED
                logger.info("NewsAPI连接器连接成功")
                return True
            else:
                self.status = ConnectionStatus.ERROR
                logger.error("NewsAPI连接器连接失败")
                return False
                
        except Exception as e:
            logger.error(f"NewsAPI连接建立失败: {e}")
            self.status = ConnectionStatus.ERROR
            return False
    
    async def disconnect(self) -> bool:
        """断开连接"""
        try:
            if self._session:
                await self._session.close()
                self._session = None
            
            self.status = ConnectionStatus.DISCONNECTED
            logger.info("NewsAPI连接器已断开")
            return True
            
        except Exception as e:
            logger.error(f"NewsAPI连接断开失败: {e}")
            return False
    
    async def _test_connection(self) -> bool:
        """测试API连接"""
        try:
            # 使用sources端点测试连接
            url = f"{self.config.base_url}/sources"
            params = {'pageSize': 1}
            
            async with self._session.get(url, params=params) as response:
                if response.status == 200:
                    data = await response.json()
                    return data.get('status') == 'ok'
                elif response.status == 401:
                    logger.error("NewsAPI认证失败，请检查API密钥")
                    return False
                else:
                    logger.error(f"NewsAPI连接测试失败，状态码: {response.status}")
                    return False
                    
        except Exception as e:
            logger.error(f"NewsAPI连接测试失败: {e}")
            return False
    
    async def _rate_limit_check(self):
        """速率限制检查"""
        now = datetime.now()
        
        # 检查是否需要重置计数器（每日重置）
        if (now - self._request_reset_time).total_seconds() > self.config.rate_limit_window:
            self._request_count = 0
            self._request_reset_time = now
        
        # 检查是否超过限制
        if self._request_count >= self.config.rate_limit_requests:
            wait_time = self.config.rate_limit_window - (now - self._request_reset_time).total_seconds()
            if wait_time > 0:
                logger.warning(f"NewsAPI每日限制已达上限，等待 {wait_time/3600:.2f} 小时")
                await asyncio.sleep(min(3600, wait_time))  # 最多等待1小时
                return
        
        self._request_count += 1
    
    async def _make_request(self, endpoint: str, params: Dict[str, Any] = None) -> Dict[str, Any]:
        """发起API请求"""
        await self._rate_limit_check()
        
        url = f"{self.config.base_url}/{endpoint}"
        
        try:
            async with self._session.get(url, params=params) as response:
                if response.status == 200:
                    data = await response.json()
                    if data.get('status') == 'ok':
                        return data
                    else:
                        raise Exception(f"NewsAPI错误: {data.get('message', '未知错误')}")
                elif response.status == 401:
                    raise Exception("NewsAPI认证失败，请检查API密钥")
                elif response.status == 429:
                    raise Exception("NewsAPI请求过于频繁，已达到速率限制")
                elif response.status == 426:
                    raise Exception("NewsAPI需要升级到付费计划")
                else:
                    raise Exception(f"NewsAPI请求失败，状态码: {response.status}")
                    
        except aiohttp.ClientError as e:
            raise Exception(f"NewsAPI网络请求失败: {e}")
    
    def _extract_financial_entities(self, text: str) -> List[EntityMention]:
        """提取金融实体"""
        entities = []
        
        for entity_type, pattern in self.FINANCIAL_KEYWORDS.items():
            matches = re.findall(pattern, text, re.IGNORECASE)
            
            for match in set(matches):
                entities.append(EntityMention(
                    entity=match,
                    entity_type=f"financial_{entity_type}",
                    mention_count=text.count(match),
                    relevance_score=0.8
                ))
        
        # 提取股票代码
        stock_pattern = r'\$([A-Z]{1,5})\b'
        stock_matches = re.findall(stock_pattern, text)
        for stock in set(stock_matches):
            entities.append(EntityMention(
                entity=stock,
                entity_type="stock_symbol",
                mention_count=text.count(f"${stock}"),
                relevance_score=0.9
            ))
        
        return entities
    
    def _calculate_source_credibility(self, source_name: str, source_url: str) -> float:
        """计算新闻源可信度"""
        # 基于域名的可信度评估
        for domain, credibility in self.SOURCE_CREDIBILITY.items():
            if domain in source_url:
                return credibility
        
        # 基于源名称的启发式评估
        trusted_indicators = ['reuters', 'bloomberg', 'associated press', 'bbc', 'cnn']
        for indicator in trusted_indicators:
            if indicator.lower() in source_name.lower():
                return 0.85
        
        # 默认可信度
        return 0.7
    
    def _calculate_market_impact_score(self, title: str, content: str) -> float:
        """计算市场影响评分"""
        text = f"{title} {content}".lower()
        
        # 高影响关键词
        high_impact_keywords = [
            'fed', 'federal reserve', 'interest rate', 'inflation', 'recession',
            'earnings', 'gdp', 'unemployment', 'merger', 'acquisition',
            'ipo', 'bankruptcy', 'regulation', 'sanctions'
        ]
        
        # 中等影响关键词
        medium_impact_keywords = [
            'market', 'stock', 'trading', 'investment', 'economy',
            'financial', 'bank', 'currency', 'commodity'
        ]
        
        score = 0.0
        for keyword in high_impact_keywords:
            if keyword in text:
                score += 0.3
        
        for keyword in medium_impact_keywords:
            if keyword in text:
                score += 0.1
        
        return min(1.0, score)
    
    def _calculate_simple_sentiment(self, text: str) -> SentimentScore:
        """计算简单情绪分数"""
        if not text:
            return SentimentScore(
                polarity=SentimentPolarity.NEUTRAL,
                confidence=0.0,
                strength=0.0,
                compound_score=0.0,
                positive_prob=0.33,
                negative_prob=0.33,
                neutral_prob=0.34
            )
        
        text_lower = text.lower()
        
        # 正面情绪词汇
        positive_words = [
            'growth', 'profit', 'gain', 'rise', 'increase', 'boost', 'success',
            'strong', 'positive', 'bull', 'rally', 'surge', 'soar'
        ]
        
        # 负面情绪词汇
        negative_words = [
            'loss', 'decline', 'fall', 'drop', 'crash', 'recession', 'crisis',
            'weak', 'negative', 'bear', 'plunge', 'collapse', 'fail'
        ]
        
        positive_count = sum(1 for word in positive_words if word in text_lower)
        negative_count = sum(1 for word in negative_words if word in text_lower)
        total_sentiment_words = positive_count + negative_count
        
        if total_sentiment_words == 0:
            polarity = SentimentPolarity.NEUTRAL
            compound_score = 0.0
            confidence = 0.1
        else:
            if positive_count > negative_count:
                polarity = SentimentPolarity.POSITIVE
                compound_score = (positive_count - negative_count) / total_sentiment_words
            elif negative_count > positive_count:
                polarity = SentimentPolarity.NEGATIVE
                compound_score = (positive_count - negative_count) / total_sentiment_words
            else:
                polarity = SentimentPolarity.NEUTRAL
                compound_score = 0.0
            
            confidence = min(0.8, total_sentiment_words / 10.0)
        
        return SentimentScore(
            polarity=polarity,
            confidence=confidence,
            strength=abs(compound_score),
            compound_score=compound_score,
            positive_prob=positive_count / max(1, total_sentiment_words),
            negative_prob=negative_count / max(1, total_sentiment_words),
            neutral_prob=1.0 - (positive_count + negative_count) / max(1, total_sentiment_words),
            analyzer_model='newsapi_keyword_based'
        )
    
    async def get_top_headlines(self, 
                               country: str = None, 
                               category: str = None,
                               sources: str = None,
                               q: str = None,
                               page_size: int = None) -> List[NewsArticle]:
        """
        获取头条新闻
        
        Args:
            country: 国家代码 (如 'us', 'gb')
            category: 新闻类别
            sources: 新闻源
            q: 搜索关键词
            page_size: 每页大小
            
        Returns:
            NewsArticle对象列表
        """
        if not self._session:
            await self.connect()
        
        try:
            params = {}
            
            if country:
                params['country'] = country
            if category:
                params['category'] = category
            if sources:
                params['sources'] = sources
            if q:
                params['q'] = q
            if page_size:
                params['pageSize'] = min(page_size, self.config.max_page_size)
            else:
                params['pageSize'] = self.config.default_page_size
            
            data = await self._make_request('top-headlines', params)
            articles = []
            
            for article_data in data.get('articles', []):
                if not article_data.get('title') or article_data.get('title') == '[Removed]':
                    continue
                
                # 解析发布时间
                published_at = datetime.now()
                if article_data.get('publishedAt'):
                    try:
                        published_at = datetime.fromisoformat(
                            article_data['publishedAt'].replace('Z', '+00:00')
                        )
                    except:
                        pass
                
                # 计算可信度
                source_name = article_data.get('source', {}).get('name', '')
                source_url = article_data.get('url', '')
                credibility = self._calculate_source_credibility(source_name, source_url)
                
                # 计算市场影响
                title = article_data.get('title', '')
                content = article_data.get('content', '') or article_data.get('description', '')
                market_impact = self._calculate_market_impact_score(title, content)
                
                # 创建NewsArticle对象
                article = NewsArticle(
                    title=title,
                    content=content,
                    summary=article_data.get('description', ''),
                    author=article_data.get('author'),
                    source=source_name,
                    publication_date=published_at,
                    url=source_url,
                    category=category or 'general',
                    market_impact_score=market_impact,
                    credibility_score=credibility,
                    sentiment_score=self._calculate_simple_sentiment(f"{title} {content}")
                )
                
                articles.append(article)
            
            logger.info(f"NewsAPI头条新闻获取成功: {len(articles)} 篇文章")
            return articles
            
        except Exception as e:
            logger.error(f"NewsAPI头条新闻获取失败: {e}")
            return []
    
    async def search_news(self, 
                         query: str,
                         from_date: datetime = None,
                         to_date: datetime = None,
                         language: str = None,
                         sort_by: str = 'relevancy',
                         page: int = 1,
                         page_size: int = None) -> List[NewsArticle]:
        """
        搜索新闻
        
        Args:
            query: 搜索关键词
            from_date: 开始日期
            to_date: 结束日期
            language: 语言代码
            sort_by: 排序方式 ('relevancy', 'popularity', 'publishedAt')
            page: 页码
            page_size: 每页大小
            
        Returns:
            NewsArticle对象列表
        """
        if not self._session:
            await self.connect()
        
        try:
            params = {
                'q': query,
                'sortBy': sort_by,
                'page': page,
                'pageSize': page_size or self.config.default_page_size
            }
            
            if from_date:
                params['from'] = from_date.strftime('%Y-%m-%d')
            if to_date:
                params['to'] = to_date.strftime('%Y-%m-%d')
            if language:
                params['language'] = language
            
            data = await self._make_request('everything', params)
            articles = []
            
            for article_data in data.get('articles', []):
                if not article_data.get('title') or article_data.get('title') == '[Removed]':
                    continue
                
                # 解析发布时间
                published_at = datetime.now()
                if article_data.get('publishedAt'):
                    try:
                        published_at = datetime.fromisoformat(
                            article_data['publishedAt'].replace('Z', '+00:00')
                        )
                    except:
                        pass
                
                # 计算相关性和影响评分
                source_name = article_data.get('source', {}).get('name', '')
                source_url = article_data.get('url', '')
                credibility = self._calculate_source_credibility(source_name, source_url)
                
                title = article_data.get('title', '')
                content = article_data.get('content', '') or article_data.get('description', '')
                market_impact = self._calculate_market_impact_score(title, content)
                
                # 创建NewsArticle对象
                article = NewsArticle(
                    title=title,
                    content=content,
                    summary=article_data.get('description', ''),
                    author=article_data.get('author'),
                    source=source_name,
                    publication_date=published_at,
                    url=source_url,
                    category='search_result',
                    market_impact_score=market_impact,
                    credibility_score=credibility,
                    sentiment_score=self._calculate_simple_sentiment(f"{title} {content}"),
                    tags=[query]  # 添加搜索关键词作为标签
                )
                
                articles.append(article)
            
            logger.info(f"NewsAPI新闻搜索成功: '{query}' 找到 {len(articles)} 篇文章")
            return articles
            
        except Exception as e:
            logger.error(f"NewsAPI新闻搜索失败: {e}")
            return []
    
    async def get_news_sources(self, 
                              category: str = None,
                              language: str = None,
                              country: str = None) -> List[Dict[str, Any]]:
        """
        获取新闻源列表
        
        Args:
            category: 新闻类别
            language: 语言代码
            country: 国家代码
            
        Returns:
            新闻源信息列表
        """
        if not self._session:
            await self.connect()
        
        try:
            params = {}
            
            if category:
                params['category'] = category
            if language:
                params['language'] = language
            if country:
                params['country'] = country
            
            data = await self._make_request('sources', params)
            sources = data.get('sources', [])
            
            # 添加可信度评估
            for source in sources:
                source['credibility_score'] = self._calculate_source_credibility(
                    source.get('name', ''),
                    source.get('url', '')
                )
            
            logger.info(f"NewsAPI新闻源获取成功: {len(sources)} 个源")
            return sources
            
        except Exception as e:
            logger.error(f"NewsAPI新闻源获取失败: {e}")
            return []
    
    async def get_financial_news(self, keywords: List[str] = None) -> List[NewsArticle]:
        """
        获取金融相关新闻
        
        Args:
            keywords: 搜索关键词列表
            
        Returns:
            金融新闻列表
        """
        if not keywords:
            keywords = ['finance', 'stock market', 'economy', 'investment', 'cryptocurrency']
        
        all_articles = []
        
        for keyword in keywords:
            articles = await self.search_news(
                query=keyword,
                from_date=datetime.now() - timedelta(days=1),
                sort_by='publishedAt',
                page_size=20
            )
            all_articles.extend(articles)
            
            # 避免请求过快
            await asyncio.sleep(0.5)
        
        # 去重和排序
        unique_articles = {}
        for article in all_articles:
            if article.url not in unique_articles:
                unique_articles[article.url] = article
        
        # 按市场影响评分排序
        sorted_articles = sorted(
            unique_articles.values(),
            key=lambda x: x.market_impact_score,
            reverse=True
        )
        
        logger.info(f"金融新闻获取完成: {len(sorted_articles)} 篇文章")
        return sorted_articles
    
    # 实现BaseConnector抽象方法
    async def get_symbols(self) -> List[str]:
        """获取支持的新闻类别"""
        return list(self.NEWS_CATEGORIES.keys())
    
    async def get_klines(self, symbol: str, interval: str, start_time=None, end_time=None, limit: int = 500):
        """新闻连接器不支持K线数据"""
        raise NotImplementedError("新闻连接器不支持K线数据")
    
    async def get_ticker(self, symbol: str):
        """获取指定类别的最新新闻"""
        if symbol in self.NEWS_CATEGORIES:
            articles = await self.get_top_headlines(category=symbol, page_size=1)
            if articles:
                return {
                    'symbol': symbol,
                    'latest_news': articles[0].title,
                    'timestamp': articles[0].publication_date
                }
        return None
    
    async def get_orderbook(self, symbol: str, depth: int = 20):
        """新闻连接器不支持订单簿数据"""
        self.logger.warning("NewsAPI新闻连接器不支持订单簿数据")
        return None
    
    async def subscribe_real_time(self, symbols: List[str], data_types: List[DataType], callback: callable) -> bool:
        """新闻连接器不支持实时订阅"""
        self.logger.warning("NewsAPI新闻连接器不支持实时订阅")
        return False
    
    async def unsubscribe_real_time(self, symbols: List[str], data_types: List[DataType]) -> bool:
        """新闻连接器不支持实时订阅"""
        self.logger.warning("NewsAPI新闻连接器不支持实时订阅")
        return False
    
    async def health_check(self) -> Dict[str, Any]:
        """健康检查"""
        try:
            if not self._session:
                await self.connect()
            
            # 测试基本连接
            test_result = await self._test_connection()
            
            # 测试获取数据
            test_articles = await self.get_top_headlines(page_size=1)
            
            return {
                'status': 'healthy' if test_result and test_articles else 'unhealthy',
                'connection': test_result,
                'data_access': len(test_articles) > 0,
                'supported_categories': len(self.NEWS_CATEGORIES),
                'supported_languages': len(self.SUPPORTED_LANGUAGES),
                'request_count': self._request_count,
                'rate_limit_remaining': self.config.rate_limit_requests - self._request_count,
                'last_check': datetime.now().isoformat(),
                'api_status': 'active' if test_result else 'inactive'
            }
            
        except Exception as e:
            logger.error(f"NewsAPI健康检查失败: {e}")
            return {
                'status': 'unhealthy',
                'error': str(e),
                'last_check': datetime.now().isoformat()
            }

# 工厂函数
def create_newsapi_connector(api_key: str, **kwargs) -> NewsAPIConnector:
    """
    创建NewsAPI连接器实例
    
    Args:
        api_key: NewsAPI API密钥
        **kwargs: 其他配置参数
        
    Returns:
        NewsAPIConnector实例
    """
    config = NewsAPIConfig(
        api_key=api_key,
        **kwargs
    )
    return NewsAPIConnector(config) 