use thiserror::Error;

#[derive(Error, Debug)]
pub enum CurveError {
    #[error("API请求错误: {0}")]
    RequestError(String),

    #[error("JSON解析错误: {0}")]
    JsonError(#[from] reqwest::Error),

    #[error("数据解析错误: {0}")]
    ParseError(String),

    #[error("合约调用错误: {0}")]
    ContractError(String),

    #[error("数据验证错误: {0}")]
    ValidationError(String),

    #[error("数据转换错误: {0}")]
    ConversionError(String),

    #[error("未知错误: {0}")]
    Unknown(String),
} 