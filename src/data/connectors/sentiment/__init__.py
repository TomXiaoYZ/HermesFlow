"""
舆情数据连接器包

此包包含各种舆情数据源的连接器，用于收集和分析市场情绪数据。

支持的舆情数据源：
- Sentix: 专业情绪指数和市场情绪分析
- NewsAPI: 全球新闻聚合和情绪分析
- Reddit RSS: 社区讨论监控和情绪分析

作者: HermesFlow Team
创建时间: 2024年12月21日
"""

from .sentix_connector import SentixConnector, create_sentix_connector
from .newsapi_connector import NewsAPIConnector, create_newsapi_connector
from .reddit_rss_connector import RedditRSSConnector, create_reddit_rss_connector

# 连接器注册表
SENTIMENT_CONNECTORS = {
    'sentix': {
        'class': SentixConnector,
        'factory': create_sentix_connector,
        'description': 'Sentix专业情绪指数连接器',
        'data_type': 'professional_sentiment'
    },
    'newsapi': {
        'class': NewsAPIConnector,
        'factory': create_newsapi_connector,
        'description': 'NewsAPI全球新闻聚合连接器',
        'data_type': 'news_sentiment'
    },
    'reddit_rss': {
        'class': RedditRSSConnector,
        'factory': create_reddit_rss_connector,
        'description': 'Reddit社区RSS情绪连接器',
        'data_type': 'social_sentiment'
    }
}

# 导出的公共API
__all__ = [
    'SentixConnector',
    'NewsAPIConnector',
    'RedditRSSConnector',
    'create_sentix_connector',
    'create_newsapi_connector',
    'create_reddit_rss_connector',
    'SENTIMENT_CONNECTORS'
]

def get_available_connectors():
    """获取所有可用的舆情连接器"""
    return list(SENTIMENT_CONNECTORS.keys())

def create_connector(connector_type: str, **kwargs):
    """
    工厂函数：根据类型创建舆情连接器
    
    Args:
        connector_type: 连接器类型 ('sentix', 'newsapi', 'reddit_rss')
        **kwargs: 连接器配置参数
        
    Returns:
        对应的连接器实例
        
    Raises:
        ValueError: 不支持的连接器类型
    """
    if connector_type not in SENTIMENT_CONNECTORS:
        raise ValueError(f"不支持的舆情连接器类型: {connector_type}")
    
    factory_func = SENTIMENT_CONNECTORS[connector_type]['factory']
    return factory_func(**kwargs)

def create_sentiment_connector(connector_type: str = 'sentix', **kwargs):
    """
    创建舆情连接器的统一工厂函数
    
    Args:
        connector_type: 连接器类型，默认为'sentix'
        **kwargs: 连接器配置参数
        
    Returns:
        对应的连接器实例
    """
    return create_connector(connector_type, **kwargs) 