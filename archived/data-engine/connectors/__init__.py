"""
数据连接器模块

提供统一的交易所数据接入接口
支持多个主流交易所的REST API和WebSocket连接
"""

from .base_connector import BaseConnector, ConnectionConfig, DataType, DataPoint
from .binance_connector import BinanceConnector, create_binance_connector
from .okx_connector import OKXConnector, create_okx_connector
from .bitget_connector import BitgetConnector, create_bitget_connector
from .fred_connector import FREDConnector, create_fred_connector
from .gmgn.gmgn_connector import GMGNConnector, create_gmgn_connector
from .sentiment import create_sentiment_connector

# 可选导入IBKR connector
try:
    from .ibkr_connector import IBKRConnector, create_ibkr_connector
    IBKR_AVAILABLE = True
except ImportError:
    IBKRConnector = None
    create_ibkr_connector = None
    IBKR_AVAILABLE = False

__all__ = [
    'BaseConnector',
    'ConnectionConfig', 
    'DataType',
    'DataPoint',
    'BinanceConnector',
    'OKXConnector',
    'BitgetConnector',
    'FREDConnector',
    'GMGNConnector',
    'CONNECTOR_REGISTRY',
    'ORDERBOOK_SUPPORTED_CONNECTORS',
    'DATA_SOURCE_CATEGORIES',
    'create_connector',
    'get_supported_connectors',
    'get_orderbook_connectors',
    'get_connector_category',
]

# 如果IBKR可用，添加到__all__
if IBKR_AVAILABLE:
    __all__.append('IBKRConnector')

# 连接器注册表
CONNECTOR_REGISTRY = {
    'binance': create_binance_connector,
    'okx': create_okx_connector,
    'bitget': create_bitget_connector,
    'fred': create_fred_connector,
    'gmgn': create_gmgn_connector,
    'sentix': create_sentiment_connector,
    'newsapi': create_sentiment_connector,
    'reddit_rss': create_sentiment_connector,
}

# 如果IBKR可用，添加到注册表
if IBKR_AVAILABLE:
    CONNECTOR_REGISTRY['ibkr'] = create_ibkr_connector

# 支持订单簿数据的连接器
ORDERBOOK_SUPPORTED_CONNECTORS = {
    'binance': '完整的加密货币订单簿',
    'okx': '完整的加密货币订单簿',
    'bitget': '完整的加密货币订单簿',
    'gmgn': 'DeFi协议订单簿',
}

# 如果IBKR可用，添加到订单簿支持列表
if IBKR_AVAILABLE:
    ORDERBOOK_SUPPORTED_CONNECTORS['ibkr'] = 'Level I/II美股订单簿'

# 数据源分类
DATA_SOURCE_CATEGORIES = {
    'crypto_exchanges': ['binance', 'okx', 'bitget'],
    'traditional_finance': ['fred'],
    'defi': ['gmgn'],
    'sentiment': ['sentix', 'newsapi', 'reddit_rss'],
}

# 如果IBKR可用，添加到传统金融分类
if IBKR_AVAILABLE:
    DATA_SOURCE_CATEGORIES['traditional_finance'].append('ibkr')


def create_connector(connector_type: str, config: dict):
    """
    创建指定类型的连接器实例
    
    Args:
        connector_type: 连接器类型
        config: 配置字典
        
    Returns:
        连接器实例
        
    Raises:
        ValueError: 不支持的连接器类型
    """
    if connector_type not in CONNECTOR_REGISTRY:
        raise ValueError(f"不支持的连接器类型: {connector_type}")
    
    factory_func = CONNECTOR_REGISTRY[connector_type]
    return factory_func(config)


def get_supported_connectors():
    """获取所有支持的连接器类型"""
    return list(CONNECTOR_REGISTRY.keys())


def get_orderbook_connectors():
    """获取支持订单簿数据的连接器"""
    return ORDERBOOK_SUPPORTED_CONNECTORS


def get_connector_category(connector_type: str):
    """获取连接器所属分类"""
    for category, connectors in DATA_SOURCE_CATEGORIES.items():
        if connector_type in connectors:
            return category
    return 'other'


def get_connector(exchange: str, config: ConnectionConfig):
    """
    工厂函数：根据交易所名称获取对应的连接器实例
    
    Args:
        exchange: 交易所名称 ('binance', 'okx', 'bitget', 'gmgn', 'fred', 'ibkr', 'sentix', 'newsapi', 'reddit_rss')
        config: 连接配置
        
    Returns:
        对应的连接器实例
        
    Raises:
        ValueError: 如果交易所名称不支持
    """
    exchange_lower = exchange.lower()
    
    if exchange_lower not in CONNECTOR_REGISTRY:
        supported_exchanges = ', '.join(CONNECTOR_REGISTRY.keys())
        raise ValueError(f"不支持的交易所: {exchange}. 支持的交易所: {supported_exchanges}")
    
    connector_class = CONNECTOR_REGISTRY[exchange_lower]
    return connector_class(config)


def get_supported_exchanges():
    """
    获取支持的交易所列表
    
    Returns:
        支持的交易所名称列表
    """
    return list(CONNECTOR_REGISTRY.keys()) 