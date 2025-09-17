"""
GMGN连接器数据模型

定义与GMGN平台交互所需的所有数据结构和配置类。
"""

from dataclasses import dataclass, field
from typing import Dict, List, Optional, Any, Union
from datetime import datetime
from enum import Enum

from ..base_connector import ConnectionConfig


class ChainType(Enum):
    """支持的区块链类型"""
    SOLANA = "sol"
    ETHEREUM = "eth"
    BASE = "base"
    BSC = "bsc"


class SwapMode(Enum):
    """交换模式"""
    EXACT_IN = "ExactIn"
    EXACT_OUT = "ExactOut"


class TransactionStatus(Enum):
    """交易状态"""
    PENDING = "pending"
    SUCCESS = "success"
    FAILED = "failed"
    EXPIRED = "expired"


@dataclass
class GMGNConfig(ConnectionConfig):
    """
    GMGN连接器配置类
    
    继承自基础连接器配置，添加GMGN特有的配置项
    """
    # 基础配置
    base_url: str = "https://gmgn.ai"
    
    # 支持的链
    supported_chains: List[ChainType] = field(default_factory=lambda: [
        ChainType.SOLANA, 
        ChainType.ETHEREUM, 
        ChainType.BASE, 
        ChainType.BSC
    ])
    
    # API限制
    rate_limit_per_second: float = 2.0  # 每秒最大请求数
    max_retry_attempts: int = 3
    retry_delay: float = 1.0
    request_timeout: int = 30
    
    # 缓存配置
    cache_expire_time: int = 300  # 5分钟
    max_cache_size: int = 1000
    
    # 交易配置
    default_slippage: float = 1.0  # 默认滑点1%
    enable_anti_mev: bool = True  # 默认开启反MEV
    default_gas_fee: float = 0.006  # 默认Gas费用(SOL)
    
    # 数据爬取配置（备用）
    enable_scraping: bool = False
    scraping_interval: int = 60  # 爬取间隔秒数
    scraping_batch_size: int = 10


@dataclass
class TokenInfo:
    """
    代币信息数据结构
    """
    address: str  # 代币合约地址
    symbol: str  # 代币符号
    name: str  # 代币名称
    decimals: int  # 小数位数
    
    # 链信息
    chain: ChainType
    
    # 市场数据
    price_usd: Optional[float] = None  # USD价格
    market_cap: Optional[float] = None  # 市值
    volume_24h: Optional[float] = None  # 24小时交易量
    
    # 流动性信息
    total_supply: Optional[float] = None  # 总供应量
    circulating_supply: Optional[float] = None  # 流通供应量
    liquidity: Optional[float] = None  # 流动性
    
    # 元数据
    created_at: Optional[datetime] = None
    updated_at: Optional[datetime] = None
    
    # 安全信息
    is_verified: bool = False
    risk_level: Optional[str] = None
    
    def to_dict(self) -> Dict[str, Any]:
        """转换为字典格式"""
        return {
            'address': self.address,
            'symbol': self.symbol,
            'name': self.name,
            'decimals': self.decimals,
            'chain': self.chain.value,
            'price_usd': self.price_usd,
            'market_cap': self.market_cap,
            'volume_24h': self.volume_24h,
            'total_supply': self.total_supply,
            'circulating_supply': self.circulating_supply,
            'liquidity': self.liquidity,
            'created_at': self.created_at.isoformat() if self.created_at else None,
            'updated_at': self.updated_at.isoformat() if self.updated_at else None,
            'is_verified': self.is_verified,
            'risk_level': self.risk_level
        }


@dataclass  
class TokenPair:
    """
    交易对信息
    """
    base_token: TokenInfo  # 基础代币
    quote_token: TokenInfo  # 计价代币
    chain: ChainType
    
    # 交易对标识
    pair_address: Optional[str] = None  # 流动性池地址
    
    # 市场数据
    price: Optional[float] = None  # 当前价格
    volume_24h: Optional[float] = None  # 24小时交易量
    liquidity: Optional[float] = None  # 流动性
    
    # 价格变化
    price_change_24h: Optional[float] = None  # 24小时价格变化
    price_change_percent_24h: Optional[float] = None  # 24小时价格变化百分比
    
    def symbol(self) -> str:
        """获取交易对符号"""
        return f"{self.base_token.symbol}/{self.quote_token.symbol}"
    
    def to_dict(self) -> Dict[str, Any]:
        """转换为字典格式"""
        return {
            'base_token': self.base_token.to_dict(),
            'quote_token': self.quote_token.to_dict(),
            'chain': self.chain.value,
            'pair_address': self.pair_address,
            'symbol': self.symbol(),
            'price': self.price,
            'volume_24h': self.volume_24h,
            'liquidity': self.liquidity,
            'price_change_24h': self.price_change_24h,
            'price_change_percent_24h': self.price_change_percent_24h
        }


@dataclass
class RouteStep:
    """
    交换路由步骤
    """
    protocol: str  # 协议名称 (如 "Raydium", "Uniswap")
    input_token: str  # 输入代币地址
    output_token: str  # 输出代币地址
    input_amount: str  # 输入金额
    output_amount: str  # 输出金额
    fee: Optional[float] = None  # 手续费
    
    def to_dict(self) -> Dict[str, Any]:
        """转换为字典格式"""
        return {
            'protocol': self.protocol,
            'input_token': self.input_token,
            'output_token': self.output_token,
            'input_amount': self.input_amount,
            'output_amount': self.output_amount,
            'fee': self.fee
        }


@dataclass
class SwapQuote:
    """
    交换报价信息
    """
    # 基础信息
    input_mint: str  # 输入代币地址
    output_mint: str  # 输出代币地址
    input_amount: str  # 输入金额
    output_amount: str  # 输出金额
    
    # 滑点和影响
    slippage_bps: int  # 滑点基点 (10000为分母)
    price_impact_pct: str  # 价格影响百分比
    other_amount_threshold: str  # 考虑滑点后的最小输出金额
    
    # 交换配置
    swap_mode: SwapMode  # 交换模式
    
    # 路由信息
    route_plan: List[RouteStep] = field(default_factory=list)  # 路由计划
    
    # 元数据
    context_slot: Optional[int] = None  # Solana slot编号
    time_taken: Optional[float] = None  # 计算耗时
    platform_fee: Optional[str] = None  # 平台费用
    
    def to_dict(self) -> Dict[str, Any]:
        """转换为字典格式"""
        return {
            'input_mint': self.input_mint,
            'output_mint': self.output_mint,
            'input_amount': self.input_amount,
            'output_amount': self.output_amount,
            'slippage_bps': self.slippage_bps,
            'price_impact_pct': self.price_impact_pct,
            'other_amount_threshold': self.other_amount_threshold,
            'swap_mode': self.swap_mode.value,
            'route_plan': [step.to_dict() for step in self.route_plan],
            'context_slot': self.context_slot,
            'time_taken': self.time_taken,
            'platform_fee': self.platform_fee
        }


@dataclass
class SwapRoute:
    """
    完整的交换路由信息
    """
    quote: SwapQuote  # 报价信息
    chain: ChainType  # 区块链类型
    
    # 原始交易数据 (Solana专用)
    swap_transaction: Optional[str] = None  # Base64编码的交易
    last_valid_block_height: Optional[int] = None  # 最后有效区块高度
    recent_blockhash: Optional[str] = None  # 最近区块哈希
    prioritization_fee_lamports: Optional[int] = None  # 优先费用
    
    # ETH系链交易数据
    transaction_data: Optional[Dict[str, Any]] = None  # ETH/Base/BSC交易数据
    gas_estimate: Optional[int] = None  # Gas估算
    gas_price: Optional[str] = None  # Gas价格
    
    # 通用字段
    created_at: datetime = field(default_factory=datetime.now)
    
    def to_dict(self) -> Dict[str, Any]:
        """转换为字典格式"""
        return {
            'quote': self.quote.to_dict(),
            'swap_transaction': self.swap_transaction,
            'last_valid_block_height': self.last_valid_block_height,
            'recent_blockhash': self.recent_blockhash,
            'prioritization_fee_lamports': self.prioritization_fee_lamports,
            'transaction_data': self.transaction_data,
            'gas_estimate': self.gas_estimate,
            'gas_price': self.gas_price,
            'chain': self.chain.value,
            'created_at': self.created_at.isoformat()
        }


@dataclass
class TransactionResult:
    """
    交易执行结果
    """
    # 基础信息
    hash: str  # 交易哈希
    chain: ChainType
    status: TransactionStatus
    
    # 交易详情
    from_address: Optional[str] = None  # 发送方地址
    to_address: Optional[str] = None  # 接收方地址
    input_token: Optional[str] = None  # 输入代币
    output_token: Optional[str] = None  # 输出代币
    input_amount: Optional[str] = None  # 输入金额
    output_amount: Optional[str] = None  # 实际输出金额
    
    # 费用信息
    gas_used: Optional[int] = None  # 实际使用的Gas
    gas_price: Optional[str] = None  # Gas价格
    transaction_fee: Optional[str] = None  # 交易费用
    
    # 时间信息
    submitted_at: datetime = field(default_factory=datetime.now)  # 提交时间
    confirmed_at: Optional[datetime] = None  # 确认时间
    
    # 错误信息
    error_message: Optional[str] = None  # 错误消息
    error_code: Optional[str] = None  # 错误代码
    
    def to_dict(self) -> Dict[str, Any]:
        """转换为字典格式"""
        return {
            'hash': self.hash,
            'chain': self.chain.value,
            'status': self.status.value,
            'from_address': self.from_address,
            'to_address': self.to_address,
            'input_token': self.input_token,
            'output_token': self.output_token,
            'input_amount': self.input_amount,
            'output_amount': self.output_amount,
            'gas_used': self.gas_used,
            'gas_price': self.gas_price,
            'transaction_fee': self.transaction_fee,
            'submitted_at': self.submitted_at.isoformat(),
            'confirmed_at': self.confirmed_at.isoformat() if self.confirmed_at else None,
            'error_message': self.error_message,
            'error_code': self.error_code
        }


@dataclass
class GMGNMarketData:
    """
    GMGN市场数据聚合
    """
    # 热门代币
    trending_tokens: List[TokenInfo] = field(default_factory=list)
    
    # 新币列表  
    new_tokens: List[TokenInfo] = field(default_factory=list)
    
    # 交易对列表
    active_pairs: List[TokenPair] = field(default_factory=list)
    
    # 统计信息
    total_volume_24h: Optional[float] = None  # 24小时总交易量
    total_transactions_24h: Optional[int] = None  # 24小时总交易数
    
    # 更新时间
    updated_at: datetime = field(default_factory=datetime.now)
    
    def to_dict(self) -> Dict[str, Any]:
        """转换为字典格式"""
        return {
            'trending_tokens': [token.to_dict() for token in self.trending_tokens],
            'new_tokens': [token.to_dict() for token in self.new_tokens],
            'active_pairs': [pair.to_dict() for pair in self.active_pairs],
            'total_volume_24h': self.total_volume_24h,
            'total_transactions_24h': self.total_transactions_24h,
            'updated_at': self.updated_at.isoformat()
        }


# 常用代币地址常量 (Solana)
class SolanaTokens:
    """Solana常用代币地址"""
    SOL = "So11111111111111111111111111111111111111112"
    USDC = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
    USDT = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"
    RAY = "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R"
    SRM = "SRMuApVNdxXokk5GT7XD5cUUgXMBCoAz2LHeuAoKWRt"


# 常用代币地址常量 (Ethereum)
class EthereumTokens:
    """Ethereum常用代币地址"""
    ETH = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"  # WETH
    USDC = "0xA0b86a33E6417c3cdd45caE3C81C5CF26e3A2D8b"
    USDT = "0xdAC17F958D2ee523a2206206994597C13D831ec7"
    DAI = "0x6B175474E89094C44Da98b954EedeAC495271d0F"


def create_token_info_from_gmgn_data(data: Dict[str, Any], chain: ChainType) -> TokenInfo:
    """
    从GMGN API响应创建TokenInfo对象
    
    Args:
        data: GMGN API返回的代币数据
        chain: 区块链类型
        
    Returns:
        TokenInfo: 代币信息对象
    """
    return TokenInfo(
        address=data.get('address', ''),
        symbol=data.get('symbol', ''),
        name=data.get('name', ''),
        decimals=data.get('decimals', 18),
        chain=chain,
        price_usd=data.get('price_usd'),
        market_cap=data.get('market_cap'),
        volume_24h=data.get('volume_24h'),
        total_supply=data.get('total_supply'),
        circulating_supply=data.get('circulating_supply'),
        liquidity=data.get('liquidity'),
        is_verified=data.get('is_verified', False),
        risk_level=data.get('risk_level'),
        updated_at=datetime.now()
    )


def create_swap_route_from_gmgn_data(data: Dict[str, Any], chain: ChainType) -> SwapRoute:
    """
    从GMGN API响应创建SwapRoute对象
    
    Args:
        data: GMGN API返回的路由数据
        chain: 区块链类型
        
    Returns:
        SwapRoute: 交换路由对象
    """
    quote_data = data.get('quote', {})
    raw_tx_data = data.get('raw_tx', {})
    
    # 创建路由步骤
    route_steps = []
    for step_data in quote_data.get('routePlan', []):
        step = RouteStep(
            protocol=step_data.get('swapInfo', {}).get('label', 'Unknown'),
            input_token=step_data.get('inputMint', ''),
            output_token=step_data.get('outputMint', ''),
            input_amount=step_data.get('inAmount', '0'),
            output_amount=step_data.get('outAmount', '0'),
            fee=step_data.get('feeAmount')
        )
        route_steps.append(step)
    
    # 创建报价
    quote = SwapQuote(
        input_mint=quote_data.get('inputMint', ''),
        output_mint=quote_data.get('outputMint', ''),
        input_amount=quote_data.get('inAmount', '0'),
        output_amount=quote_data.get('outAmount', '0'),
        slippage_bps=quote_data.get('slippageBps', 0),
        price_impact_pct=quote_data.get('priceImpactPct', '0'),
        other_amount_threshold=quote_data.get('otherAmountThreshold', '0'),
        swap_mode=SwapMode(quote_data.get('swapMode', 'ExactIn')),
        route_plan=route_steps,
        context_slot=quote_data.get('contextSlot'),
        time_taken=quote_data.get('timeTaken'),
        platform_fee=quote_data.get('platformFee')
    )
    
    # 创建路由
    route = SwapRoute(
        quote=quote,
        chain=chain
    )
    
    # 根据链类型设置特定字段
    if chain == ChainType.SOLANA:
        route.swap_transaction = raw_tx_data.get('swapTransaction')
        route.last_valid_block_height = raw_tx_data.get('lastValidBlockHeight')
        route.recent_blockhash = raw_tx_data.get('recentBlockhash')
        route.prioritization_fee_lamports = raw_tx_data.get('prioritizationFeeLamports')
    else:
        route.transaction_data = raw_tx_data.get('transactionData')
        route.gas_estimate = raw_tx_data.get('gasEstimate')
        route.gas_price = raw_tx_data.get('gasPrice')
    
    return route 