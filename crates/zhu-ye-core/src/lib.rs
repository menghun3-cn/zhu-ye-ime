//! 竹叶输入法核心算法库。
//!
//! 本 crate 不依赖任何 Windows 专有 API，算法行为确定、可测试，
//! 后续输入法 DLL/CLI 全部通过这里的纯数据接口取结果。

pub mod ai;
pub mod candidate;
pub mod dict;
pub mod error;
pub mod pinyin;
pub mod translate;
pub mod user_dict;

pub use ai::{AiService, OfflineAiService};
pub use candidate::{Candidate, CandidateSorter};
pub use dict::{Dictionary, DictionaryEntry, InMemoryDictionary};
pub use error::{Error, Result};
pub use pinyin::{segment_all, valid_prefix, FullPinyinScheme, PinyinScheme, SyllableTable};
pub use translate::{InMemoryTranslator, Translator};
pub use user_dict::UserDictionary;

/// 返回核心库版本标识，用于自检输出。
#[must_use]
pub fn core_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
