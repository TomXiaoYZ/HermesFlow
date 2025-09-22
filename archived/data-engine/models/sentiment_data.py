"""
舆情数据模型

定义舆情分析系统使用的标准化数据结构，包括：
- 情绪数据基础模型
- 新闻文章数据结构
- 社交媒体帖子模型
- 情绪评分计算模型
- 热点话题数据模型
- 影响者指标模型

作者: HermesFlow Team
创建时间: 2024年12月21日
"""

from dataclasses import dataclass, field
from datetime import datetime
from typing import Optional, List, Dict, Any, Union
from decimal import Decimal
from enum import Enum

class SentimentPolarity(Enum):
    """情绪极性枚举"""
    VERY_NEGATIVE = -2
    NEGATIVE = -1
    NEUTRAL = 0
    POSITIVE = 1
    VERY_POSITIVE = 2

class DataSource(Enum):
    """数据源类型枚举"""
    # Tier 1: 专业金融舆情源
    SENTIX = "sentix"
    ALPHAVANTAGE_NEWS = "alphavantage_news"
    FINNHUB = "finnhub"
    STOCKTWITS = "stocktwits"
    
    # Tier 2: 通用新闻聚合源
    NEWSAPI = "newsapi"
    RSS_NEWS = "rss_news"
    REUTERS = "reuters"
    BLOOMBERG = "bloomberg"
    CNBC = "cnbc"
    
    # Tier 3: 开放社区数据源
    REDDIT = "reddit"
    HACKERNEWS = "hackernews"
    MEDIUM = "medium"
    GITHUB = "github"
    DISCORD = "discord"
    
    # Tier 4: 加密货币专属源
    COINGECKO = "coingecko"
    LUNARCRUSH = "lunarcrush"
    CRYPTOPANIC = "cryptopanic"
    COINMARKETCAP = "coinmarketcap"

class ContentType(Enum):
    """内容类型枚举"""
    NEWS_ARTICLE = "news_article"
    SOCIAL_POST = "social_post"
    FORUM_POST = "forum_post"
    BLOG_POST = "blog_post"
    COMMENT = "comment"
    RESEARCH_REPORT = "research_report"
    SENTIMENT_INDEX = "sentiment_index"

@dataclass
class SentimentScore:
    """情绪评分模型"""
    polarity: SentimentPolarity
    confidence: float  # 0.0 - 1.0
    strength: float   # 0.0 - 1.0，情绪强度
    compound_score: float  # -1.0 to 1.0，复合情绪分数
    positive_prob: float  # 正面情绪概率
    negative_prob: float  # 负面情绪概率
    neutral_prob: float   # 中性情绪概率
    
    # 详细情绪分析
    emotions: Optional[Dict[str, float]] = None  # 恐惧、贪婪、希望等
    
    # 元数据
    analyzer_model: str = "default"
    analysis_timestamp: datetime = field(default_factory=datetime.now)

@dataclass
class EntityMention:
    """实体提及模型"""
    entity: str  # 实体名称（如股票代码、加密货币符号）
    entity_type: str  # 实体类型（stock, crypto, company, person）
    mention_count: int  # 提及次数
    sentiment_score: Optional[SentimentScore] = None
    relevance_score: float = 0.0  # 相关性评分 0.0-1.0
    
@dataclass
class KeywordInfo:
    """关键词信息模型"""
    keyword: str
    frequency: int
    tfidf_score: float
    category: str  # 类别（financial, technical, emotion等）
    importance_score: float  # 重要性评分

@dataclass
class SentimentData:
    """标准化情绪数据结构"""
    # 基础信息（无默认值字段）
    id: str
    source: DataSource
    content_type: ContentType
    timestamp: datetime
    title: str
    content: str
    sentiment_score: SentimentScore
    
    # 可选字段（有默认值）
    author: Optional[str] = None
    url: Optional[str] = None
    
    # 实体和关键词分析
    entities: List[EntityMention] = field(default_factory=list)
    keywords: List[KeywordInfo] = field(default_factory=list)
    
    # 社交指标
    engagement_metrics: Optional[Dict[str, Any]] = None  # 点赞、分享、评论等
    reach_metrics: Optional[Dict[str, Any]] = None       # 影响力、传播范围等
    
    # 元数据
    language: str = "en"
    data_quality_score: float = 1.0  # 数据质量评分
    processing_timestamp: datetime = field(default_factory=datetime.now)
    
    # 原始数据存储
    raw_data: Optional[Dict[str, Any]] = None

@dataclass
class NewsArticle:
    """新闻文章数据模型"""
    # 基础信息
    title: str
    content: str
    summary: str
    source: str
    publication_date: datetime
    url: str
    category: str
    
    # 可选基础信息
    author: Optional[str] = None
    
    # 分类信息
    tags: List[str] = field(default_factory=list)
    
    # 影响评估
    market_impact_score: float = 0.0  # 市场影响评分
    urgency_score: float = 0.0        # 紧急程度评分
    credibility_score: float = 1.0    # 可信度评分
    
    # 关联资产
    related_symbols: List[str] = field(default_factory=list)
    
    # 情绪分析
    sentiment_score: Optional[SentimentScore] = None
    
    # 元数据
    news_id: str = field(default_factory=lambda: f"news_{datetime.now().timestamp()}")
    crawl_timestamp: datetime = field(default_factory=datetime.now)

@dataclass
class SocialPost:
    """社交媒体帖子模型"""
    # 基础信息（无默认值字段）
    post_id: str
    platform: str  # reddit, discord, telegram等
    author: str
    content: str
    post_timestamp: datetime
    
    # 可选基础信息
    author_id: Optional[str] = None
    
    # 社交指标
    upvotes: int = 0
    downvotes: int = 0
    comments_count: int = 0
    shares_count: int = 0
    views_count: int = 0
    
    # 作者信息
    author_followers: int = 0
    author_reputation: float = 0.0
    author_verification: bool = False
    
    # 情绪分析
    sentiment_score: Optional[SentimentScore] = None
    
    # 话题标签
    hashtags: List[str] = field(default_factory=list)
    mentions: List[str] = field(default_factory=list)
    
    # 传播信息
    is_viral: bool = False
    virality_score: float = 0.0
    
    # 元数据
    thread_id: Optional[str] = None
    parent_post_id: Optional[str] = None
    subreddit: Optional[str] = None  # Reddit专用
    
@dataclass 
class TrendingTopic:
    """热点话题数据模型"""
    # 话题信息（无默认值字段）
    topic: str
    keywords: List[str]
    mention_count: int
    trend_score: float  # 趋势评分
    growth_rate: float  # 增长率
    peak_timestamp: datetime
    average_sentiment: SentimentScore
    start_time: datetime
    end_time: datetime
    
    # 可选字段（有默认值）
    hashtags: List[str] = field(default_factory=list)
    sentiment_distribution: Dict[SentimentPolarity, int] = field(default_factory=dict)
    
    # 传播分析
    source_distribution: Dict[DataSource, int] = field(default_factory=dict)
    geographic_distribution: Dict[str, int] = field(default_factory=dict)
    
    # 相关资产
    related_symbols: List[str] = field(default_factory=list)
    market_correlation: Optional[float] = None
    
    # 时间窗口
    timeframe: str = "1h"  # 1h, 1d, 1w等

@dataclass
class InfluencerMetrics:
    """影响者指标模型"""
    # 基础信息
    influencer_id: str
    username: str
    platform: str
    
    # 影响力指标
    followers_count: int
    engagement_rate: float
    reach_score: float
    authority_score: float
    
    # 内容分析
    posts_count: int
    average_sentiment: float
    topic_expertise: List[str] = field(default_factory=list)
    
    # 市场影响
    market_moving_posts: int = 0
    prediction_accuracy: float = 0.0
    track_record_score: float = 0.0
    
    # 时间相关
    measurement_period: str = "30d"
    last_updated: datetime = field(default_factory=datetime.now)

@dataclass
class SentimentTimeSeriesPoint:
    """情绪时间序列数据点"""
    timestamp: datetime
    sentiment_score: float
    volume: int  # 该时间点的数据量
    confidence: float
    source_breakdown: Dict[DataSource, int] = field(default_factory=dict)

@dataclass
class MarketSentimentIndex:
    """市场情绪指数"""
    # 基础信息
    symbol: str  # 股票代码或加密货币符号
    index_value: float  # -100 to 100
    calculation_method: str
    
    # 组成部分
    component_scores: Dict[str, float] = field(default_factory=dict)
    data_sources_weights: Dict[DataSource, float] = field(default_factory=dict)
    
    # 历史对比
    previous_value: Optional[float] = None
    change_24h: Optional[float] = None
    change_7d: Optional[float] = None
    
    # 元数据
    calculation_timestamp: datetime = field(default_factory=datetime.now)
    data_quality: float = 1.0
    sample_size: int = 0

@dataclass
class SentimentAlert:
    """情绪告警模型"""
    # 告警信息
    alert_id: str
    alert_type: str  # spike, crash, anomaly等
    severity: str    # low, medium, high, critical
    
    # 触发条件
    symbol: str
    threshold_value: float
    actual_value: float
    trigger_timestamp: datetime
    
    # 描述信息
    title: str
    description: str
    recommended_actions: List[str] = field(default_factory=list)
    
    # 状态管理
    is_resolved: bool = False
    resolution_timestamp: Optional[datetime] = None
    resolution_notes: Optional[str] = None 