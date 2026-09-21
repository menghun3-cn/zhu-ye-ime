//! 竹叶输入法核心算法库。
//!
//! 本 crate 不依赖任何 Windows 专有 API，算法行为确定、可测试，
//! 后续输入法 DLL/CLI 全部通过这里的纯数据接口取结果。

pub mod ai;
pub mod bigram;
pub mod candidate;
pub mod demo;
pub mod dict;
pub mod dict_builder;
pub mod dict_format;
pub mod dict_loader;
pub mod error;
pub mod pinyin;
pub mod translate;
pub mod user_dict;
pub mod user_store;

pub use ai::{AiService, OfflineAiService};
pub use bigram::{BigramModel, EmptyBigramModel, InMemoryBigramModel};
pub use candidate::{
    generate_candidates, Candidate, CandidateSorter, RankingConfig, RankingContext, RankingModel,
    StaticRankingModel,
};
pub use dict::{Dictionary, DictionaryEntry, InMemoryDictionary};
pub use error::{Error, Result};
pub use pinyin::{segment_all, valid_prefix, FullPinyinScheme, PinyinScheme, SyllableTable};
pub use translate::{InMemoryTranslator, Translator};
pub use user_dict::UserDictionary;
pub use user_store::{unix_now, UserDictStore};

pub use demo::{seed_bigrams, seed_entries};
pub use dict_builder::build_v2;
pub use dict_loader::DictionaryFile;

/// 返回核心库版本标识，用于自检输出。
#[must_use]
pub fn core_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
