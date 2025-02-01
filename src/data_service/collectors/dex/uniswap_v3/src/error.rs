use std::fmt;
use std::error::Error;

/// Uniswap V3 解析器错误类型
#[derive(Debug)]
pub enum UniswapV3Error {
    /// 数据解析错误
    ParseError(String),
    /// JSON解析错误
    JsonError(serde_json::Error),
    /// 数值转换错误
    DecimalError(rust_decimal::Error),
    /// 网络请求错误
    RequestError(String),
    /// 数据验证错误
    ValidationError(String),
    /// 数据转换错误
    ConversionError(String),
}

impl fmt::Display for UniswapV3Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UniswapV3Error::ParseError(msg) => write!(f, "解析错误: {}", msg),
            UniswapV3Error::JsonError(e) => write!(f, "JSON错误: {}", e),
            UniswapV3Error::DecimalError(e) => write!(f, "数值转换错误: {}", e),
            UniswapV3Error::RequestError(msg) => write!(f, "网络请求错误: {}", msg),
            UniswapV3Error::ValidationError(msg) => write!(f, "数据验证错误: {}", msg),
            UniswapV3Error::ConversionError(msg) => write!(f, "数据转换错误: {}", msg),
        }
    }
}

impl Error for UniswapV3Error {}

impl From<serde_json::Error> for UniswapV3Error {
    fn from(err: serde_json::Error) -> Self {
        UniswapV3Error::JsonError(err)
    }
}

impl From<rust_decimal::Error> for UniswapV3Error {
    fn from(err: rust_decimal::Error) -> Self {
        UniswapV3Error::DecimalError(err)
    }
} 