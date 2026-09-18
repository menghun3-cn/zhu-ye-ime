//! 核心库统一错误类型。

use std::fmt;

/// 核心库错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// 输入串无法按拼音音节完整切分。
    InvalidPinyin(String),
    /// 词典未加载或查询失败。
    Dictionary(String),
    /// 用户词库读写失败。
    UserDictionary(String),
    /// 其他内部错误。
    Internal(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPinyin(s) => write!(f, "无效拼音输入: {s}"),
            Self::Dictionary(s) => write!(f, "词典错误: {s}"),
            Self::UserDictionary(s) => write!(f, "用户词库错误: {s}"),
            Self::Internal(s) => write!(f, "内部错误: {s}"),
        }
    }
}

impl std::error::Error for Error {}

/// 核心库结果类型。
pub type Result<T> = std::result::Result<T, Error>;
