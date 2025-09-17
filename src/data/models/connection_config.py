#!/usr/bin/env python3
"""
连接配置模型模块 (Connection Configuration Models)

定义各种连接器的配置类，包括：
- 基础连接配置
- 交易所连接配置
- 数据源连接配置
- 舆情数据连接配置
"""

from dataclasses import dataclass, field
from typing import Optional, Dict, Any, List
from enum import Enum


class Environment(Enum):
    """环境类型枚举"""
    PRODUCTION = "production"
    TESTNET = "testnet"
    SANDBOX = "sandbox"
    LOCAL = "local"


@dataclass
class ConnectionConfig:
    """基础连接配置类"""
    api_key: str
    api_secret: str
    testnet: bool = False
    environment: Environment = Environment.PRODUCTION
    timeout: int = 30
    max_retries: int = 3
    retry_delay: float = 1.0
    rate_limit: Optional[int] = None
    extra_params: Dict[str, Any] = field(default_factory=dict)
    
    def __post_init__(self):
        """初始化后处理"""
        if self.testnet:
            self.environment = Environment.TESTNET


@dataclass
class ExchangeConfig(ConnectionConfig):
    """交易所连接配置类"""
    passphrase: Optional[str] = None  # OKX等交易所需要
    sandbox: bool = False
    enable_rate_limit: bool = True
    enable_websocket: bool = True
    websocket_timeout: int = 60
    
    def __post_init__(self):
        super().__post_init__()
        if self.sandbox:
            self.environment = Environment.SANDBOX


@dataclass
class BinanceConfig(ExchangeConfig):
    """Binance连接配置"""
    base_url: Optional[str] = None
    websocket_url: Optional[str] = None
    
    def __post_init__(self):
        super().__post_init__()
        if self.testnet:
            self.base_url = "https://testnet.binance.vision"
            self.websocket_url = "wss://testnet.binance.vision"
        else:
            self.base_url = "https://api.binance.com"
            self.websocket_url = "wss://stream.binance.com:9443"


@dataclass
class OKXConfig(ExchangeConfig):
    """OKX连接配置"""
    base_url: Optional[str] = None
    websocket_url: Optional[str] = None
    
    def __post_init__(self):
        super().__post_init__()
        if self.sandbox:
            self.base_url = "https://www.okx.com"
            self.websocket_url = "wss://wspap.okx.com:8443/ws/v5/public"
        else:
            self.base_url = "https://www.okx.com"
            self.websocket_url = "wss://ws.okx.com:8443/ws/v5/public"


@dataclass
class BitgetConfig(ExchangeConfig):
    """Bitget连接配置"""
    base_url: Optional[str] = None
    websocket_url: Optional[str] = None
    
    def __post_init__(self):
        super().__post_init__()
        # Bitget目前没有测试网络
        self.base_url = "https://api.bitget.com"
        self.websocket_url = "wss://ws.bitget.com/spot/v1/stream"


@dataclass
class DataSourceConfig(ConnectionConfig):
    """数据源连接配置基类"""
    base_url: str = ""
    endpoints: Dict[str, str] = field(default_factory=dict)
    headers: Dict[str, str] = field(default_factory=dict)


@dataclass
class FREDConfig(DataSourceConfig):
    """FRED (Federal Reserve Economic Data) 配置"""
    base_url: str = "https://api.stlouisfed.org/fred"
    
    def __post_init__(self):
        super().__post_init__()
        self.endpoints = {
            "series": "/series",
            "observations": "/series/observations",
            "search": "/series/search"
        }


@dataclass
class PolygonConfig(DataSourceConfig):
    """Polygon.io 配置"""
    base_url: str = "https://api.polygon.io"
    
    def __post_init__(self):
        super().__post_init__()
        self.endpoints = {
            "stocks": "/v2/aggs/ticker",
            "options": "/v3/reference/options/contracts",
            "real_time": "/v1/last/stocks"
        }


@dataclass
class SentimentConfig(ConnectionConfig):
    """舆情数据连接配置基类"""
    language: str = "en"
    sentiment_threshold: float = 0.5
    keywords: List[str] = field(default_factory=list)


@dataclass
class NewsAPIConfig(SentimentConfig):
    """NewsAPI 配置"""
    base_url: str = "https://newsapi.org/v2"
    country: str = "us"
    category: str = "business"
    page_size: int = 100
    
    def __post_init__(self):
        super().__post_init__()
        self.keywords = ["bitcoin", "cryptocurrency", "blockchain", "trading", "finance"]


@dataclass
class RedditRSSConfig(SentimentConfig):
    """Reddit RSS 配置"""
    subreddits: List[str] = field(default_factory=lambda: [
        "CryptoCurrency", "Bitcoin", "ethereum", "investing", 
        "stocks", "SecurityAnalysis", "ValueInvesting", "financialindependence"
    ])
    max_posts: int = 50
    
    def __post_init__(self):
        super().__post_init__()
        self.keywords = ["crypto", "bitcoin", "ethereum", "trading", "investment"]


@dataclass
class SentixConfig(SentimentConfig):
    """Sentix 配置"""
    base_url: str = "https://api.sentix.de"
    indicators: List[str] = field(default_factory=lambda: [
        "EURUSD", "GBPUSD", "USDJPY", "GOLD", "OIL", "DAX", "SPX"
    ])
    
    def __post_init__(self):
        super().__post_init__()
        self.endpoints = {
            "sentiment": "/sentiment",
            "indicators": "/indicators"
        }


@dataclass
class GMGNConfig(DataSourceConfig):
    """GMGN (DEX数据) 配置"""
    base_url: str = "https://gmgn.ai/api"
    supported_chains: List[str] = field(default_factory=lambda: ["solana", "ethereum", "bsc"])
    
    def __post_init__(self):
        super().__post_init__()
        self.endpoints = {
            "tokens": "/tokens",
            "pairs": "/pairs",
            "trades": "/trades"
        }


# 配置工厂函数
def create_config(connector_type: str, **kwargs) -> ConnectionConfig:
    """
    创建连接配置实例
    
    Args:
        connector_type: 连接器类型
        **kwargs: 配置参数
    
    Returns:
        ConnectionConfig: 配置实例
    """
    config_map = {
        "binance": BinanceConfig,
        "okx": OKXConfig,
        "bitget": BitgetConfig,
        "fred": FREDConfig,
        "polygon": PolygonConfig,
        "newsapi": NewsAPIConfig,
        "reddit": RedditRSSConfig,
        "sentix": SentixConfig,
        "gmgn": GMGNConfig,
    }
    
    config_class = config_map.get(connector_type.lower(), ConnectionConfig)
    return config_class(**kwargs)


# 默认配置
DEFAULT_CONFIGS = {
    "binance": {
        "api_key": "test_key",
        "api_secret": "test_secret",
        "testnet": True
    },
    "okx": {
        "api_key": "test_key",
        "api_secret": "test_secret",
        "passphrase": "test_passphrase",
        "sandbox": True
    },
    "bitget": {
        "api_key": "test_key",
        "api_secret": "test_secret",
        "testnet": False  # Bitget没有测试网络
    },
    "fred": {
        "api_key": "test_fred_key",
        "api_secret": ""
    },
    "polygon": {
        "api_key": "test_polygon_key",
        "api_secret": ""
    },
    "newsapi": {
        "api_key": "test_newsapi_key",
        "api_secret": ""
    },
    "reddit": {
        "api_key": "",
        "api_secret": ""
    },
    "sentix": {
        "api_key": "test_sentix_key",
        "api_secret": ""
    },
    "gmgn": {
        "api_key": "",
        "api_secret": ""
    }
} 