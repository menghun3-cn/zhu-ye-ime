//! 竹叶输入法核心算法库。
//!
//! 本 crate 不依赖任何 Windows 专有 API，算法行为确定、可测试，
//! 后续输入法 DLL/CLI 全部通过这里的纯数据接口取结果。

pub mod ai;
pub mod bigram;
pub mod candidate;
pub mod char_pinyin;
pub mod composite;
pub mod contacts;
pub mod demo;
pub mod dict;
pub mod dict_builder;
pub mod dict_format;
pub mod dict_loader;
pub mod domain_boost;
pub mod email_url;
pub mod emoji;
pub mod en_lexicon;
pub mod en_words;
pub mod error;
pub mod file_log;
pub mod format;
pub mod identity;
pub mod log_level;
pub mod manifest;
pub mod mixed;
pub mod pack_config;
pub mod pinyin;
pub mod spoken;
pub mod suggestion;
pub mod symbols;
pub mod theme_file;
pub mod time;
pub mod tone;
pub mod translate;
pub mod tray_state;
pub mod units;
pub mod update;
pub mod update_status;
pub mod user_dict;
pub mod user_store;
pub mod user_words_exchange;
pub mod vcard;

pub use ai::{AiService, OfflineAiService};
pub use bigram::{BigramModel, EmptyBigramModel, InMemoryBigramModel};
pub use candidate::{
    abbreviation_candidates, append_abbreviation_group, constrained_segment_candidates,
    corrected_candidates, en_word_candidates, en_word_candidates_from, generate_candidates,
    generate_prefix_candidates, initial_candidates, is_abbreviation_input, merge_candidate_groups,
    prefix_expand_candidates, sentence_candidates, transposed_candidates, Candidate,
    CandidateSorter, PrefixCandidateGroups, RankingConfig, RankingContext, RankingModel,
    StaticRankingModel, ABBREVIATION_MIN_LEN, BEAM_WIDTH, BEAM_WORD_CAP, CORRECTION_VARIANT_CAP,
    INITIAL_COMPLETION_CAP, INITIAL_MAX_LEN, INITIAL_MIN_LEN, SENTENCE_MAX_WORD_CHARS,
    SENTENCE_TOP_N, TRANSPOSITION_VARIANT_CAP,
};
pub use char_pinyin::{char_pinyin, CharPinyinEntry, CHAR_PINYIN};
pub use composite::{any_exists, pack_path, CompositeDictionary};
pub use contacts::{
    abbreviation_key, annotate_name, build_contact_index, contact_candidates, ContactIndex,
    CONTACT_INDEX_CAP, CONTACT_KEYS_CAP,
};
pub use dict::{Dictionary, DictionaryEntry, InMemoryDictionary};
pub use domain_boost::domain_boost_candidates;
pub use email_url::{detect_format, email_candidates, url_candidates, FormatKind};
pub use emoji::{emoji_for, EmojiEntry, EMOJI_TABLE};
pub use en_lexicon::{compile_en_records, EnHeader, EnLexicon, EnRecord};
pub use en_words::{en_words_with_prefix, EnWordEntry, EN_WORDS};
pub use error::{Error, Result};
pub use file_log::{FileLogger, DEFAULT_LOG_SIZE_LIMIT};
pub use format::{cn_numeral, format_candidates, FormatCandidate};
pub use log_level::{deserialize_log_level, LogLevel};
pub use manifest::{
    canonical_bytes, generate_keypair, parse_manifest, parse_public_key, sha256_file, sha256_hex,
    sign_manifest, verify_pack_contents, verify_signature, verify_signature_with_key,
    version_at_least, Manifest, ManifestSignature, PackMeta, SignatureError, MANIFEST_SCHEMA,
    SIGNATURE_ALGORITHM,
};
pub use mixed::{is_mixed_input, mixed_candidates};
pub use pack_config::{
    is_distributable_pack, load_config, pack_display, plan_packs, save_config, ConfigFile,
    ModeChoice, PackDisplay, PackPlan, ThemeChoice, BASE_PACK_FILE_NAME, CONFIG_FORMAT_VERSION,
    DISTRIBUTABLE_PACK_IDS, KNOWN_PACK_IDS, PACKS_DIR_NAME,
};
pub use pinyin::{
    fuzzy_variants, initial_syllables, segment_all, valid_prefix, FullPinyinScheme, PinyinScheme,
    SyllableTable,
};
pub use spoken::{spoken_overrides, SPOKEN_SUGGESTIONS};
pub use suggestion::{
    suggest_phrases, suggest_words, suggestion_candidates, SUGGESTION_CAP, SUGGESTION_PHRASE_CAP,
    SUGGESTION_WORD_CAP,
};
pub use symbols::{
    all_panel_groups, symbol_group, EXTRA_SYMBOL_GROUPS, MATH_SYMBOLS, NUMBER_SYMBOLS,
    PUNCT_SYMBOLS,
};
pub use theme_file::{
    is_safe_theme_name, load_theme_file, parse_theme_file, CandidatePalette, SettingsPalette,
    ThemeColor, ThemeFile, THEME_FILE_VERSION,
};
pub use time::{civil_from_days, format_date, today_compact};
pub use translate::{InMemoryTranslator, TranslationDirection, Translator};
pub use units::{unit_candidates, unit_key_prefix, UNIT_CONVERSIONS};
pub use update::{
    apply_release, apply_staged, backup_path, clean_staging, find_outdated, rollback_pack,
    stage_packs, staging_dir, verify_installed, verify_release, ApplyOutcome, UpdateError,
    BACKUP_SUFFIX, STAGING_DIR_NAME,
};
pub use update_status::{
    check_due, parse_update_status, read_update_status, to_json as update_status_to_json,
    write_update_status, UpdateStatus, CHECK_INTERVAL_DAYS, DAY_SECONDS, UPDATE_STATUS_FILE_NAME,
    UPDATE_STATUS_FORMAT, UPDATE_STATUS_VERSION,
};
pub use user_dict::UserDictionary;
pub use user_store::{probe_user_words_file, unix_now, UserDictStore, UserWordsProbe};
pub use user_words_exchange::{
    export_user_words, merge_exchange_items, parse_exchange_file, ExchangeFile, ExchangeItem,
    USER_WORDS_EXCHANGE_FORMAT, USER_WORDS_EXCHANGE_VERSION,
};
pub use vcard::{parse_vcard, VCardContact, VCardError};

pub use demo::{seed_bigrams, seed_entries};
pub use dict_builder::build_v2;
pub use dict_loader::DictionaryFile;

/// 返回核心库版本标识，用于自检输出。
#[must_use]
pub fn core_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
