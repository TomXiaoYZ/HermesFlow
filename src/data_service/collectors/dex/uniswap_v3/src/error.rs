use thiserror::Error;

/// Uniswap V3 数据采集器错误类型
#[derive(Debug, Error)]
pub enum UniswapV3Error {
    /// 网络错误
    #[error("网络错误: {0}")]
    NetworkError(String),

    /// 合约调用错误
    #[error("合约调用错误: {0}")]
    ContractError(String),

    /// 无效的地址
    #[error("无效的地址: {0}")]
    InvalidAddress(String),

    /// 区块未找到
    #[error("区块未找到")]
    BlockNotFound,

    /// 价格计算错误
    #[error("价格计算错误: {0}")]
    PriceCalculationError(String),

    /// 数据解析错误
    #[error("数据解析错误: {0}")]
    ParseError(String),

    /// 缓存错误
    #[error("缓存错误: {0}")]
    CacheError(String),

    /// 配置错误
    #[error("配置错误: {0}")]
    ConfigError(String),

    /// 以太坊客户端错误
    #[error("以太坊客户端错误: {0}")]
    EthersError(#[from] ethers::prelude::ProviderError),

    /// 合约ABI错误
    #[error("合约ABI错误: {0}")]
    AbiError(#[from] ethers::contract::AbiError),

    /// 事件解析错误
    #[error("事件解析错误: {0}")]
    EventError(#[from] ethers::contract::ContractError<ethers::providers::Provider<ethers::providers::Http>>),
}

/// 结果类型别名
pub type UniswapV3Result<T> = Result<T, UniswapV3Error>; 