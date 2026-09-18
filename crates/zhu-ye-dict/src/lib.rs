//! 词典数据管线。
//!
//! M2 里程碑实现：原始开放词表清洗、合并去重、拼音注音、
//! 词频统计与二进制词典编译。当前先固定格式版本与表结构。

/// 词典二进制格式版本（v0 草案）。
#[must_use]
pub const fn dict_schema_version() -> u32 {
    0
}

/// 返回当前数据管线状态，供自检使用。
#[must_use]
pub fn pipeline_status() -> &'static str {
    "骨架就绪，等待 M2 数据实现"
}

#[cfg(test)]
mod tests {
    use super::{dict_schema_version, pipeline_status};

    #[test]
    fn 格式版本与状态可用() {
        assert_eq!(dict_schema_version(), 0);
        assert!(pipeline_status().contains("骨架"));
    }
}
