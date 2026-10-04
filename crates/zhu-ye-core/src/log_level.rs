//! 日志级别（第十一期 FR-060，T-091）：枚举与宽松解析。
//!
//! 级别序 `Error < Warn < Info < Debug`：文件日志的级别门按
//! “消息级别不高于配置级别才落盘”判定。非法/未知/缺失配置一律回退
//! `Warn`（D-72 默认只记错误级、热路径零文件写），与 `theme`/`default_mode`
//! 的宽松解析模式一致（T-073 教训：配置损坏不得阻断输入法）。

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// 文件日志级别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum LogLevel {
    /// 错误级：失败/降级类（装配失败、回退、配置损坏等）。
    Error,
    /// 警告级：可恢复异常。**默认级别**（D-72）。
    #[default]
    Warn,
    /// 信息级：生命周期/装配事件。
    Info,
    /// 调试级：键击/候选流，仅手动调低级别时落盘。
    Debug,
}

impl LogLevel {
    /// 宽松解析：大小写不敏感；未知/空值回退 `Warn`（D-72）。
    #[must_use]
    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "error" => Self::Error,
            "info" => Self::Info,
            "debug" => Self::Debug,
            // 未知值（含 `trace`/`off` 等历史拼写）一律回退默认级别。
            _ => Self::Warn,
        }
    }

    /// 规范小写文本（配置文件与行格式用）。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for LogLevel {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

/// 宽松反序列化：字段缺失/非字符串（如数字）/未知字符串均回退默认 `warn`
/// （与 `theme` 的宽松模式同款：任何值都不让整体配置解析失败）。
impl<'de> Deserialize<'de> for LogLevel {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Option::<serde_json::Value>::deserialize(deserializer)?;
        Ok(match value {
            Some(serde_json::Value::String(text)) => Self::parse(&text),
            _ => Self::default(),
        })
    }
}

/// `ConfigFile.log_level` 字段的 `deserialize_with` 入口（T-073 同款模式）：
/// 任意值回退默认，不影响其它字段。
pub fn deserialize_log_level<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<LogLevel, D::Error> {
    LogLevel::deserialize(deserializer)
}

#[cfg(test)]
mod tests {
    use super::LogLevel;

    #[test]
    fn 解析合法值大小写不敏感() {
        assert_eq!(LogLevel::parse("error"), LogLevel::Error);
        assert_eq!(LogLevel::parse("warn"), LogLevel::Warn);
        assert_eq!(LogLevel::parse("info"), LogLevel::Info);
        assert_eq!(LogLevel::parse("debug"), LogLevel::Debug);
        assert_eq!(LogLevel::parse("  DEBUG "), LogLevel::Debug);
    }

    #[test]
    fn 解析未知与空值回退默认级别() {
        assert_eq!(LogLevel::parse("trace"), LogLevel::Warn);
        assert_eq!(LogLevel::parse("off"), LogLevel::Warn);
        assert_eq!(LogLevel::parse(""), LogLevel::Warn);
        assert_eq!(LogLevel::parse("级别"), LogLevel::Warn);
    }

    #[test]
    fn 默认级别为警告() {
        assert_eq!(LogLevel::default(), LogLevel::Warn);
        assert_eq!(LogLevel::default().as_str(), "warn");
    }

    #[test]
    fn 序关系支撑级别门() {
        assert!(LogLevel::Error < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Debug);
    }

    #[test]
    fn 宽松反序列化未知值回退() {
        let value: LogLevel = serde_json::from_str(r#""verbose""#).expect("宽松解析");
        assert_eq!(value, LogLevel::Warn);
        let value: LogLevel = serde_json::from_str(r#"null"#).expect("非字符串回退");
        assert_eq!(value, LogLevel::Warn);
        let value: LogLevel = serde_json::from_str(r#""Info""#).expect("大小写不敏感");
        assert_eq!(value, LogLevel::Info);
    }

    #[test]
    fn 序列化输出规范文本() {
        assert_eq!(
            serde_json::to_string(&LogLevel::Info).expect("序列化"),
            r#""info""#
        );
    }
}
