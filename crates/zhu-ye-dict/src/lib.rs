//! 词典数据管线。
//!
//! 本 crate 负责把受控种子/清洗后的词表编译为 v2 二进制词典包，并提供
//! `build` / `inspect` / `verify` / `source-check` / `build-pack` /
//! `build-manifest` / `verify-manifest` 等 CLI 命令。字节布局、哈希与
//! 加载器全部由 `zhu-ye-core` 统一承载，避免构建方与加载方出现两套格式。
//!
//! `import` 命令负责把 CC-CEDICT 与开放词频语料清洗为真实词条，
//! 数据来源与许可证见 `docs/数据清单.md` 与 `docs/licenses.md`。
//! 第二期（M6）多词包与 manifest 逻辑见 [`m6`] 模块。

pub use zhu_ye_core::demo::{seed_bigrams, seed_entries};
pub use zhu_ye_core::dict_builder::build_v2;
pub use zhu_ye_core::dict_format::DICT_VERSION;

pub mod import;
pub use import::{
    build_real_bigrams, build_real_dictionary, load_frequency_map, normalize_pinyin,
    parse_cedict_line, split_pinyin_syllables,
};
pub use import::{BigramStats, ImportStats};

pub mod polyphone;
pub use polyphone::{
    load_patch_table, load_standard_readings, parse_patch_table, polyphone_gaps,
    strip_tone_letters, PatchEntry, PolyphoneGap, StandardReading,
};

pub mod m6;
pub use m6::{
    audit_coverage, build_base, build_manifest, build_pack, source_check, today, verify_manifest,
    BaseStats, CoverageReport, Manifest, MAX_MISSES,
};

pub mod slang;
pub use slang::{build_slang, SlangReport};

pub mod social;
pub use social::{clean_social, SocialCleanReport, SLANG_SOCIAL};

pub mod en_wordbook;
pub use en_wordbook::{
    build_en_wordbook, EnWordbookInputs, EnWordbookStats, ECDICT_MAX_WORD_LEN, FREQ_WORDS_TOP,
};

pub mod eval;
pub use eval::{generate_word_eval_set, render_eval_set, EvalSample};

/// 词典二进制格式版本。
#[must_use]
pub const fn dict_schema_version() -> u32 {
    DICT_VERSION
}

/// 返回当前数据管线状态，供自检使用。
#[must_use]
pub fn pipeline_status() -> &'static str {
    "v2 格式构建/检查可用：build、inspect、verify、import；M6：source-check、build-pack、build-base、build-slang、build-manifest、verify-manifest"
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use zhu_ye_core::bigram::BigramModel;
    use zhu_ye_core::dict::Dictionary;
    use zhu_ye_core::dict_format::DictHeader;
    use zhu_ye_core::dict_loader::DictionaryFile;
    use zhu_ye_core::translate::Translator;

    use super::{build_v2, dict_schema_version, pipeline_status, seed_bigrams, seed_entries};

    #[test]
    fn 版本与状态可用于自检() {
        assert_eq!(dict_schema_version(), 2);
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
        let bytes = build_v2(&entries, &bigrams).unwrap();
        let header = DictHeader::from_bytes(&bytes).unwrap();
        assert_eq!(header.entry_count, 20);
        assert_eq!(header.word_translation_count, 19);
        assert_eq!(header.reverse_translation_count, 19);

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
        assert_eq!(file.zh_to_en("你好").as_deref(), Some("hello"));
        assert_eq!(file.en_to_zh("china").as_deref(), Some("中国"));

        drop(file);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
