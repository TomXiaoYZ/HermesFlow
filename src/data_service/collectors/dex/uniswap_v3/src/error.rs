use thiserror::Error;

/// Uniswap V3 解析器错误类型
#[derive(Error, Debug)]
pub enum UniswapV3Error {
    #[error("解析错误: {0}")]
    ParseError(String),

    #[error("合约错误: {0}")]
    ContractError(String),

    #[error("网络错误: {0}")]
    NetworkError(String),
}

pub type Result<T> = std::result::Result<T, UniswapV3Error>; 