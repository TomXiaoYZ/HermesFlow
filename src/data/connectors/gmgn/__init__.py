"""
GMGN连接器模块

该模块提供与GMGN平台的集成功能，支持：
- Solana链交易API
- ETH/Base/BSC多链交易
- 数据爬取功能  
- 反MEV交易支持

作者: HermesFlow量化交易平台
创建时间: 2024-01
"""

from .gmgn_connector import GMGNConnector
from .models import (
    GMGNConfig,
    TokenInfo,
    TokenPair, 
    SwapRoute,
    SwapQuote,
    TransactionResult
)
from .solana_trading import SolanaTrading
from .eth_trading import ETHTrading
from .scraper import GMGNScraper

__all__ = [
    'GMGNConnector',
    'GMGNConfig',
    'TokenInfo',
    'TokenPair',
    'SwapRoute', 
    'SwapQuote',
    'TransactionResult',
    'SolanaTrading',
    'ETHTrading',
    'GMGNScraper'
]

__version__ = '1.0.0' 