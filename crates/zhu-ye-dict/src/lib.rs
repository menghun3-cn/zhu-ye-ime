//! 词典数据管线。
//!
//! 本 crate 负责把受控种子/清洗后的词表编译为 v1 二进制词典包，并提供
//! `build` / `inspect` / `verify` 三个 CLI 命令。字节布局、哈希与加载器
//! 全部由 `zhu-ye-core` 统一承载，避免构建方与加载方出现两套格式。

pub use zhu_ye_core::demo::{seed_bigrams, seed_entries};
pub use zhu_ye_core::dict_builder::build_v1;
pub use zhu_ye_core::dict_format::DICT_VERSION;

/// 词典二进制格式版本。
#[must_use]
pub const fn dict_schema_version() -> u32 {
    DICT_VERSION
}

/// 返回当前数据管线状态，供自检使用。
#[must_use]
pub fn pipeline_status() -> &'static str {
    "v1 格式构建/检查可用：build、inspect、verify"
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use zhu_ye_core::bigram::BigramModel;
    use zhu_ye_core::dict::Dictionary;
    use zhu_ye_core::dict_format::DictHeader;
    use zhu_ye_core::dict_loader::DictionaryFile;

    use super::{build_v1, dict_schema_version, pipeline_status, seed_bigrams, seed_entries};

    #[test]
    fn 版本与状态可用于自检() {
        assert_eq!(dict_schema_version(), 1);
        assert!(pipeline_status().contains("build"));
    }

    #[test]
    fn 种子数据量与字典接口一致() {
        assert_eq!(seed_entries().len(), 20);
        assert_eq!(seed_bigrams().len(), 10);
        assert_eq!(
            seed_entries()
                .iter()
                .find(|entry| entry.word == "你好")
                .map(|entry| entry.translation.as_deref())
                .unwrap(),
            Some("hello")
        );
    }

    #[test]
    fn 构建加载查询闭环() {
        let entries = seed_entries();
        let bigrams = seed_bigrams();
        let bytes = build_v1(&entries, &bigrams).unwrap();
        let header = DictHeader::from_bytes(&bytes).unwrap();
        assert_eq!(header.entry_count, 20);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("zhu-ye-dict-pipeline-{}-{now}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("seed.zyct");
        std::fs::write(&path, &bytes).unwrap();

        let file = DictionaryFile::open(Path::new(&path)).unwrap();
        let nihao = file.lookup("nihao");
        assert_eq!(nihao.len(), 2);
        assert_eq!(nihao[0].word, "你好");
        assert_eq!(file.frequency("你好", "世界"), 120);

        drop(file);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
