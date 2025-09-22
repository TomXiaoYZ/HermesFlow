"""
Reddit RSS连接器

通过RSS源监控Reddit金融版块，提供：
- r/investing, r/stocks, r/cryptocurrency等热门版块
- 帖子热度、评论情感、投票比例分析
- 用户影响力评估和KOL识别
- 话题趋势和病毒式传播检测

数据来源: Reddit RSS Feeds
相关文档: https://www.reddit.com/wiki/rss

作者: HermesFlow Team
创建时间: 2024年12月21日
"""

import asyncio
import aiohttp
import logging
import feedparser
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any, Union
from dataclasses import dataclass
import hashlib
import re
from urllib.parse import urljoin, urlparse

from ..base_connector import BaseConnector, ConnectionConfig, ConnectionStatus, DataType
from ...models.sentiment_data import (
    SentimentData, SentimentScore, SentimentPolarity, DataSource, 
    ContentType, SocialPost, EntityMention, KeywordInfo, TrendingTopic
)

# 配置日志
logger = logging.getLogger(__name__)

@dataclass
class RedditRSSConfig(ConnectionConfig):
    """Reddit RSS连接器配置"""
    user_agent: str = "HermesFlow/1.0 (Financial Analysis Bot)"
    timeout: int = 30
    rate_limit_delay: float = 2.0  # 请求间隔秒数
    max_posts_per_subreddit: int = 50
    monitor_intervals: int = 300  # 监控间隔秒数 (5分钟)

class RedditRSSConnector(BaseConnector):
    """
    Reddit RSS连接器
    
    提供Reddit社区数据监控功能：
    - 多版块RSS监控
    - 帖子数据解析
    - 热度和趋势分析
    - 用户行为分析
    """
    
    def __init__(self, config: RedditRSSConfig):
        """
        初始化Reddit RSS连接器
        
        Args:
            config: Reddit RSS连接器配置
        """
        super().__init__(config, "reddit_rss")
        self.config = config
        self._session: Optional[aiohttp.ClientSession] = None
        
        # 监控的Reddit版块列表
        self.MONITORED_SUBREDDITS = {
            'investing': {
                'url': 'https://www.reddit.com/r/investing/.rss',
                'description': '投资讨论版块',
                'relevance': 0.9,
                'categories': ['stocks', 'bonds', 'investment']
            },
            'stocks': {
                'url': 'https://www.reddit.com/r/stocks/.rss',
                'description': '股票讨论版块',
                'relevance': 0.95,
                'categories': ['stocks', 'trading', 'market']
            },
            'SecurityAnalysis': {
                'url': 'https://www.reddit.com/r/SecurityAnalysis/.rss',
                'description': '证券分析版块',
                'relevance': 0.85,
                'categories': ['analysis', 'valuation', 'research']
            },
            'ValueInvesting': {
                'url': 'https://www.reddit.com/r/ValueInvesting/.rss',
                'description': '价值投资版块',
                'relevance': 0.8,
                'categories': ['value', 'long-term', 'fundamentals']
            },
            'CryptoCurrency': {
                'url': 'https://www.reddit.com/r/CryptoCurrency/.rss',
                'description': '加密货币版块',
                'relevance': 0.9,
                'categories': ['crypto', 'bitcoin', 'blockchain']
            },
            'Bitcoin': {
                'url': 'https://www.reddit.com/r/Bitcoin/.rss',
                'description': '比特币版块',
                'relevance': 0.88,
                'categories': ['bitcoin', 'crypto']
            },
            'ethereum': {
                'url': 'https://www.reddit.com/r/ethereum/.rss',
                'description': '以太坊版块',
                'relevance': 0.85,
                'categories': ['ethereum', 'crypto', 'defi']
            },
            'DeFi': {
                'url': 'https://www.reddit.com/r/DeFi/.rss',
                'description': 'DeFi版块',
                'relevance': 0.8,
                'categories': ['defi', 'crypto', 'yield']
            },
            'wallstreetbets': {
                'url': 'https://www.reddit.com/r/wallstreetbets/.rss',
                'description': '华尔街赌场版块',
                'relevance': 0.7,
                'categories': ['meme', 'options', 'yolo']
            },
            'financialindependence': {
                'url': 'https://www.reddit.com/r/financialindependence/.rss',
                'description': '财务独立版块',
                'relevance': 0.75,
                'categories': ['fire', 'retirement', 'savings']
            }
        }
        
        # 金融相关关键词模式
        self.FINANCIAL_PATTERNS = {
            'stock_symbols': r'\$([A-Z]{1,5})\b',
            'crypto_symbols': r'\b(BTC|ETH|ADA|SOL|AVAX|MATIC|DOT|LINK|UNI)\b',
            'price_mentions': r'\$[\d,]+(?:\.\d{2})?',
            'percentage_change': r'[+-]?\d+(?:\.\d+)?%',
            'market_terms': r'\b(bull|bear|rally|crash|moon|pump|dump|hodl|dip)\b'
        }
        
        # 情绪指示词
        self.SENTIMENT_INDICATORS = {
            'positive': ['bullish', 'moon', 'pump', 'rally', 'buy', 'long', 'calls', 'rocket'],
            'negative': ['bearish', 'crash', 'dump', 'sell', 'short', 'puts', 'red', 'loss'],
            'neutral': ['hold', 'sideways', 'flat', 'analysis', 'research', 'study']
        }
        
    async def connect(self) -> bool:
        """建立连接"""
        try:
            if self._session is None:
                timeout = aiohttp.ClientTimeout(total=self.config.timeout)
                self._session = aiohttp.ClientSession(
                    timeout=timeout,
                    headers={
                        'User-Agent': self.config.user_agent
                    }
                )
            
            # 测试连接 - 尝试获取一个版块的RSS
            test_result = await self._test_connection()
            
            if test_result:
                self.status = ConnectionStatus.CONNECTED
                logger.info("Reddit RSS连接器连接成功")
                return True
            else:
                self.status = ConnectionStatus.ERROR
                logger.error("Reddit RSS连接器连接失败")
                return False
                
        except Exception as e:
            logger.error(f"Reddit RSS连接建立失败: {e}")
            self.status = ConnectionStatus.ERROR
            return False
    
    async def disconnect(self) -> bool:
        """断开连接"""
        try:
            if self._session and not self._session.closed:
                await self._session.close()
                # 等待一小段时间确保连接完全关闭
                await asyncio.sleep(0.1)
            self._session = None
            
            self.status = ConnectionStatus.DISCONNECTED
            logger.info("Reddit RSS连接器已断开")
            return True
            
        except Exception as e:
            logger.error(f"Reddit RSS连接断开失败: {e}")
            return False
    
    async def _test_connection(self) -> bool:
        """测试RSS连接"""
        try:
            # 测试获取投资版块的RSS，使用更短的超时时间
            test_url = self.MONITORED_SUBREDDITS['investing']['url']
            
            # 使用更短的超时时间进行测试
            timeout = aiohttp.ClientTimeout(total=10, connect=5)
            async with self._session.get(test_url, timeout=timeout) as response:
                if response.status == 200:
                    content = await response.text()
                    # 验证RSS内容格式
                    is_valid = '<?xml' in content and 'rss' in content.lower()
                    if is_valid:
                        logger.info("Reddit RSS连接测试成功")
                    else:
                        logger.warning("Reddit RSS响应格式不正确")
                    return is_valid
                else:
                    logger.error(f"Reddit RSS测试失败，状态码: {response.status}")
                    return False
                    
        except asyncio.TimeoutError:
            logger.error("Reddit RSS连接测试超时")
            return False
        except Exception as e:
            logger.error(f"Reddit RSS连接测试失败: {e}")
            return False
    
    def _extract_post_metrics(self, entry: Dict[str, Any]) -> Dict[str, int]:
        """从RSS条目中提取帖子指标"""
        metrics = {
            'upvotes': 0,
            'downvotes': 0,
            'comments': 0,
            'score': 0
        }
        
        # 从标题或内容中提取分数信息
        title = entry.get('title', '')
        summary = entry.get('summary', '')
        content = f"{title} {summary}"
        
        # 查找评论数
        comments_pattern = r'(\d+)\s*comments?'
        comments_match = re.search(comments_pattern, content, re.IGNORECASE)
        if comments_match:
            metrics['comments'] = int(comments_match.group(1))
        
        # 查找分数信息（如果有的话）
        score_pattern = r'(\d+)\s*points?'
        score_match = re.search(score_pattern, content, re.IGNORECASE)
        if score_match:
            metrics['score'] = int(score_match.group(1))
            metrics['upvotes'] = metrics['score']  # 简化处理
        
        return metrics
    
    def _extract_financial_entities(self, text: str) -> List[EntityMention]:
        """提取金融实体（股票代码、加密货币等）"""
        entities = []
        
        for pattern_name, pattern in self.FINANCIAL_PATTERNS.items():
            matches = re.findall(pattern, text, re.IGNORECASE)
            
            for match in set(matches):
                entity_type = 'stock' if pattern_name == 'stock_symbols' else \
                             'crypto' if pattern_name == 'crypto_symbols' else \
                             'financial_term'
                
                entities.append(EntityMention(
                    entity=match,
                    entity_type=entity_type,
                    mention_count=text.count(match),
                    relevance_score=0.8 if entity_type in ['stock', 'crypto'] else 0.5
                ))
        
        return entities
    
    def _calculate_sentiment_score(self, text: str) -> SentimentScore:
        """计算简单的情绪分数"""
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
        
        # 计算正面、负面词汇数量
        positive_count = sum(1 for word in self.SENTIMENT_INDICATORS['positive'] 
                           if word in text_lower)
        negative_count = sum(1 for word in self.SENTIMENT_INDICATORS['negative']
                           if word in text_lower)
        neutral_count = sum(1 for word in self.SENTIMENT_INDICATORS['neutral']
                          if word in text_lower)
        
        total_indicators = positive_count + negative_count + neutral_count
        
        if total_indicators == 0:
            polarity = SentimentPolarity.NEUTRAL
            compound_score = 0.0
            confidence = 0.1
        else:
            # 计算情绪极性
            if positive_count > negative_count:
                polarity = SentimentPolarity.POSITIVE if positive_count > negative_count * 1.5 else SentimentPolarity.NEUTRAL
                compound_score = (positive_count - negative_count) / total_indicators
            elif negative_count > positive_count:
                polarity = SentimentPolarity.NEGATIVE if negative_count > positive_count * 1.5 else SentimentPolarity.NEUTRAL
                compound_score = (positive_count - negative_count) / total_indicators
            else:
                polarity = SentimentPolarity.NEUTRAL
                compound_score = 0.0
            
            confidence = min(0.9, total_indicators / 10.0)
        
        # 计算概率分布
        total_words = len(text_lower.split())
        if total_words == 0:
            positive_prob = negative_prob = neutral_prob = 0.33
        else:
            positive_prob = min(0.9, positive_count / total_words * 10)
            negative_prob = min(0.9, negative_count / total_words * 10)
            neutral_prob = 1.0 - positive_prob - negative_prob
        
        return SentimentScore(
            polarity=polarity,
            confidence=confidence,
            strength=abs(compound_score),
            compound_score=compound_score,
            positive_prob=positive_prob,
            negative_prob=negative_prob,
            neutral_prob=neutral_prob,
            analyzer_model='reddit_keyword_based'
        )
    
    async def _fetch_rss_feed(self, url: str) -> Optional[Dict[str, Any]]:
        """获取RSS数据"""
        try:
            async with self._session.get(url) as response:
                if response.status == 200:
                    content = await response.text()
                    # 使用feedparser解析RSS
                    feed = feedparser.parse(content)
                    return feed
                else:
                    logger.error(f"RSS获取失败，URL: {url}, 状态码: {response.status}")
                    return None
                    
        except Exception as e:
            logger.error(f"RSS获取失败: {e}")
            return None
    
    async def get_subreddit_posts(self, subreddit: str, limit: int = None) -> List[SocialPost]:
        """
        获取指定版块的帖子
        
        Args:
            subreddit: 版块名称
            limit: 最大帖子数量
            
        Returns:
            SocialPost对象列表
        """
        if not self._session:
            await self.connect()
        
        if subreddit not in self.MONITORED_SUBREDDITS:
            logger.warning(f"不支持的版块: {subreddit}")
            return []
        
        try:
            subreddit_info = self.MONITORED_SUBREDDITS[subreddit]
            rss_url = subreddit_info['url']
            
            feed = await self._fetch_rss_feed(rss_url)
            if not feed:
                return []
            
            posts = []
            max_posts = limit or self.config.max_posts_per_subreddit
            
            for entry in feed.entries[:max_posts]:
                # 解析帖子时间
                published_time = datetime.now()
                if hasattr(entry, 'published_parsed') and entry.published_parsed:
                    published_time = datetime(*entry.published_parsed[:6])
                
                # 提取帖子内容
                title = entry.get('title', '')
                content = entry.get('summary', '')
                full_content = f"{title}\n{content}"
                
                # 提取用户名（从链接中）
                author = 'unknown'
                if hasattr(entry, 'link'):
                    # Reddit链接通常包含用户信息
                    link_parts = entry.link.split('/')
                    if 'user' in link_parts:
                        try:
                            user_idx = link_parts.index('user')
                            if user_idx + 1 < len(link_parts):
                                author = link_parts[user_idx + 1]
                        except:
                            pass
                
                # 提取帖子指标
                metrics = self._extract_post_metrics(entry)
                
                # 创建SocialPost对象
                post = SocialPost(
                    post_id=hashlib.md5(entry.link.encode()).hexdigest(),
                    platform='reddit',
                    author=author,
                    content=full_content,
                    post_timestamp=published_time,
                    upvotes=metrics['upvotes'],
                    downvotes=metrics['downvotes'],
                    comments_count=metrics['comments'],
                    subreddit=subreddit
                )
                
                # 添加情绪分析
                post.sentiment_score = self._calculate_sentiment_score(full_content)
                
                # 提取话题标签和提及
                hashtag_pattern = r'#(\w+)'
                mention_pattern = r'@(\w+)'
                post.hashtags = re.findall(hashtag_pattern, full_content)
                post.mentions = re.findall(mention_pattern, full_content)
                
                # 计算病毒传播评分
                engagement = metrics['upvotes'] + metrics['comments']
                post.virality_score = min(1.0, engagement / 1000.0)
                post.is_viral = engagement > 500
                
                posts.append(post)
            
            logger.info(f"Reddit版块 {subreddit} 获取成功: {len(posts)} 篇帖子")
            return posts
            
        except Exception as e:
            logger.error(f"Reddit版块获取失败: {e}")
            return []
    
    async def monitor_all_subreddits(self) -> Dict[str, List[SocialPost]]:
        """
        监控所有版块
        
        Returns:
            按版块分组的帖子数据
        """
        if not self._session:
            await self.connect()
        
        all_posts = {}
        
        for subreddit in self.MONITORED_SUBREDDITS.keys():
            posts = await self.get_subreddit_posts(subreddit)
            all_posts[subreddit] = posts
            
            # 避免请求过快
            await asyncio.sleep(self.config.rate_limit_delay)
        
        total_posts = sum(len(posts) for posts in all_posts.values())
        logger.info(f"Reddit全版块监控完成: {total_posts} 篇帖子")
        
        return all_posts
    
    async def get_trending_topics(self, time_window: int = 3600) -> List[TrendingTopic]:
        """
        获取趋势话题
        
        Args:
            time_window: 时间窗口（秒）
            
        Returns:
            TrendingTopic对象列表
        """
        all_posts = await self.monitor_all_subreddits()
        
        # 统计关键词频率
        keyword_count = {}
        all_entities = []
        
        cutoff_time = datetime.now() - timedelta(seconds=time_window)
        
        for subreddit, posts in all_posts.items():
            for post in posts:
                if post.post_timestamp >= cutoff_time:
                    # 提取关键词
                    words = re.findall(r'\b\w+\b', post.content.lower())
                    for word in words:
                        if len(word) > 3:  # 过滤短词
                            keyword_count[word] = keyword_count.get(word, 0) + 1
                    
                    # 收集实体
                    entities = self._extract_financial_entities(post.content)
                    all_entities.extend(entities)
        
        # 生成趋势话题
        trending_topics = []
        
        # 按频率排序，取前10个
        top_keywords = sorted(keyword_count.items(), key=lambda x: x[1], reverse=True)[:10]
        
        for keyword, count in top_keywords:
            if count >= 3:  # 至少被提及3次
                trend_score = min(1.0, count / 50.0)  # 标准化趋势分数
                
                trending_topic = TrendingTopic(
                    topic=keyword,
                    keywords=[keyword],
                    mention_count=count,
                    trend_score=trend_score,
                    growth_rate=0.0,  # 需要历史数据计算
                    peak_timestamp=datetime.now(),
                    average_sentiment=SentimentScore(
                        polarity=SentimentPolarity.NEUTRAL,
                        confidence=0.5,
                        strength=0.5,
                        compound_score=0.0,
                        positive_prob=0.33,
                        negative_prob=0.33,
                        neutral_prob=0.34
                    ),
                    start_time=cutoff_time,
                    end_time=datetime.now(),
                    timeframe=f"{time_window}s"
                )
                
                trending_topics.append(trending_topic)
        
        logger.info(f"Reddit趋势话题分析完成: {len(trending_topics)} 个话题")
        return trending_topics
    
    # 实现BaseConnector抽象方法
    async def get_symbols(self) -> List[str]:
        """获取监控的版块列表"""
        return list(self.MONITORED_SUBREDDITS.keys())
    
    async def get_klines(self, symbol: str, interval: str, start_time=None, end_time=None, limit: int = 500):
        """Reddit连接器不支持K线数据"""
        raise NotImplementedError("Reddit连接器不支持K线数据")
    
    async def get_ticker(self, symbol: str):
        """获取版块的最新帖子信息"""
        if symbol in self.MONITORED_SUBREDDITS:
            posts = await self.get_subreddit_posts(symbol, limit=1)
            if posts:
                return {
                    'symbol': symbol,
                    'latest_post': posts[0].content[:100],
                    'timestamp': posts[0].post_timestamp
                }
        return None
    
    async def get_orderbook(self, symbol: str, depth: int = 20):
        """Reddit连接器不支持订单簿数据"""
        self.logger.warning("Reddit RSS连接器不支持订单簿数据")
        return None
    
    async def subscribe_real_time(self, symbols: List[str], data_types: List[DataType], callback: callable) -> bool:
        """Reddit连接器不支持实时订阅"""
        self.logger.warning("Reddit RSS连接器不支持实时订阅")
        return False
    
    async def unsubscribe_real_time(self, symbols: List[str], data_types: List[DataType]) -> bool:
        """Reddit连接器不支持实时订阅"""
        self.logger.warning("Reddit RSS连接器不支持实时订阅")
        return False
    
    async def health_check(self) -> Dict[str, Any]:
        """健康检查"""
        try:
            if not self._session:
                await self.connect()
            
            # 测试基本连接
            test_result = await self._test_connection()
            
            # 测试获取数据
            test_posts = await self.get_subreddit_posts('investing', limit=1)
            
            return {
                'status': 'healthy' if test_result and test_posts else 'unhealthy',
                'connection': test_result,
                'data_access': len(test_posts) > 0,
                'monitored_subreddits': len(self.MONITORED_SUBREDDITS),
                'last_check': datetime.now().isoformat(),
                'rss_status': 'active' if test_result else 'inactive'
            }
            
        except Exception as e:
            logger.error(f"Reddit RSS健康检查失败: {e}")
            return {
                'status': 'unhealthy',
                'error': str(e),
                'last_check': datetime.now().isoformat()
            }

# 工厂函数
def create_reddit_rss_connector(**kwargs) -> RedditRSSConnector:
    """
    创建Reddit RSS连接器实例
    
    Args:
        **kwargs: 配置参数
        
    Returns:
        RedditRSSConnector实例
    """
    config = RedditRSSConfig(**kwargs)
    return RedditRSSConnector(config) 