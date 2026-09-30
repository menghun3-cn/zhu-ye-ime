//! 竹叶输入法核心算法库。
//!
//! 本 crate 不依赖任何 Windows 专有 API，算法行为确定、可测试，
//! 后续输入法 DLL/CLI 全部通过这里的纯数据接口取结果。

pub mod ai;
pub mod bigram;
pub mod candidate;
pub mod composite;
pub mod demo;
pub mod dict;
pub mod dict_builder;
pub mod dict_format;
pub mod dict_loader;
pub mod emoji;
pub mod en_words;
pub mod error;
pub mod format;
pub mod manifest;
pub mod pack_config;
pub mod pinyin;
pub mod suggestion;
pub mod symbols;
pub mod translate;
pub mod update;
pub mod user_dict;
pub mod user_store;

pub use ai::{AiService, OfflineAiService};
pub use bigram::{BigramModel, EmptyBigramModel, InMemoryBigramModel};
pub use candidate::{
    abbreviation_candidates, append_abbreviation_group, corrected_candidates, generate_candidates,
    generate_prefix_candidates, initial_candidates, is_abbreviation_input, merge_candidate_groups,
    sentence_candidates, Candidate, CandidateSorter, PrefixCandidateGroups, RankingConfig,
    RankingContext, RankingModel, StaticRankingModel, ABBREVIATION_MIN_LEN, BEAM_WIDTH,
    BEAM_WORD_CAP, CORRECTION_VARIANT_CAP, INITIAL_COMPLETION_CAP, INITIAL_MAX_LEN,
    INITIAL_MIN_LEN, SENTENCE_MAX_WORD_CHARS, SENTENCE_TOP_N,
};
pub use composite::{any_exists, pack_path, CompositeDictionary};
pub use dict::{Dictionary, DictionaryEntry, InMemoryDictionary};
pub use emoji::{emoji_for, EmojiEntry, EMOJI_TABLE};
pub use en_words::{en_words_with_prefix, EnWordEntry, EN_WORDS};
pub use error::{Error, Result};
pub use format::{cn_numeral, format_candidates, FormatCandidate};
pub use manifest::{
    canonical_bytes, parse_manifest, parse_public_key, sha256_file, sha256_hex, sign_manifest,
    verify_pack_contents, verify_signature, verify_signature_with_key, version_at_least, Manifest,
    ManifestSignature, PackMeta, SignatureError, MANIFEST_SCHEMA, SIGNATURE_ALGORITHM,
};
pub use pack_config::{
    is_distributable_pack, load_config, plan_packs, save_config, ConfigFile, PackPlan,
    BASE_PACK_FILE_NAME, CONFIG_FORMAT_VERSION, DISTRIBUTABLE_PACK_IDS, KNOWN_PACK_IDS,
    PACKS_DIR_NAME,
};
pub use pinyin::{
    fuzzy_variants, initial_syllables, segment_all, valid_prefix, FullPinyinScheme, PinyinScheme,
    SyllableTable,
};
pub use suggestion::{
    suggest_phrases, suggest_words, suggestion_candidates, SUGGESTION_CAP, SUGGESTION_PHRASE_CAP,
    SUGGESTION_WORD_CAP,
};
pub use symbols::{symbol_group, MATH_SYMBOLS, NUMBER_SYMBOLS, PUNCT_SYMBOLS};
pub use translate::{InMemoryTranslator, TranslationDirection, Translator};
pub use update::{
    apply_release, apply_staged, backup_path, clean_staging, find_outdated, rollback_pack,
    stage_packs, staging_dir, verify_installed, verify_release, ApplyOutcome, UpdateError,
    BACKUP_SUFFIX, STAGING_DIR_NAME,
};
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
