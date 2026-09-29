//! 候选类型与排序。
//!
//! 底层 `CandidateSorter` 只负责确定性基础排序；候选排序模型（trait）
//! 负责把静态词典分、上下文 bigram 与用户词频加权成最终排序分。
use std::collections::HashMap;

use std::sync::Arc;

use crate::bigram::{BigramModel, EmptyBigramModel};
use crate::dict::{Dictionary, DictionaryEntry};
use crate::pinyin::{segment_all, SyllableTable};
use crate::user_dict::UserDictionary;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CandidateSource {
    /// 静态词典候选。
    Static,
    /// 用户词候选中被提升。
    User,
    /// AI 建议（第一版不启用）。
    Ai,
    /// 网络语缩写路径候选（M6-R）；UI 以 `[网络]` 标注。
    Slang,
}

/// 输入法候选。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    /// 上屏文本。
    pub text: String,
    /// 对应译文（若有）。
    pub translation: Option<String>,
    /// 上屏文本对应的拼音串（用户词学习使用；AI 候选可能为空）。
    pub pinyin: Option<String>,
    /// 排序权重分，越大越优先。
    pub score: i64,
    /// 来源。
    pub source: CandidateSource,
}

impl Candidate {
    /// 创建一个无译文的候选。
    #[must_use]
    pub fn new(text: impl Into<String>, score: i64) -> Self {
        Self {
            text: text.into(),
            translation: None,
            pinyin: None,
            score,
            source: CandidateSource::Static,
        }
    }

    /// 设置译文后返回自身（便于链式构建）。
    #[must_use]
    pub fn with_translation(mut self, translation: impl Into<String>) -> Self {
        self.translation = Some(translation.into());
        self
    }

    /// 设置来源。
    #[must_use]
    pub fn with_source(mut self, source: CandidateSource) -> Self {
        self.source = source;
        self
    }

    /// 设置拼音串后返回自身（便于链式构建）。
    #[must_use]
    pub fn with_pinyin(mut self, pinyin: impl Into<String>) -> Self {
        self.pinyin = Some(pinyin.into());
        self
    }
}

/// 候选排序器：按排序分降序、文本升序排列。
/// 排序必须确定，相同输入产生相同顺序。
#[derive(Debug, Clone, Default)]
pub struct CandidateSorter;

/// 前缀候选分组（T-029）：输入串无法完整切分时使用。
///
/// - `completions`：组 2，拼音以输入串为前缀的完整词（如 `nih` → 你好），展示在前；
/// - `completed`：组 1，输入串尾部残缺音节之前的最后完整音节的候选（如 `nih` → ni 的你/泥），展示在后。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrefixCandidateGroups {
    /// 组 2：前缀补全词。
    pub completions: Vec<Candidate>,
    /// 组 1：最后完整音节的候选。
    pub completed: Vec<Candidate>,
}

/// 前缀候选生成（T-029）：输入串存在尾部残缺音节时，返回补全组与完成组；
/// 输入为空、可完整切分（调用方应走 `generate_candidates`）或开头无完整音节时两组皆空。
/// 组 2 按词频降序截断到 `completion_cap` 条以内，控制查询成本。
#[must_use]
pub fn generate_prefix_candidates(
    table: &SyllableTable,
    dictionary: &dyn Dictionary,
    pinyin: &str,
    completion_cap: usize,
) -> PrefixCandidateGroups {
    let empty = PrefixCandidateGroups::default();
    if pinyin.is_empty() || !segment_all(table, pinyin).is_empty() {
        return empty;
    }
    // 找最长可完整切分前缀 P；找不到（如 `z`/`zh`）则两组皆空，避免前缀泛滥。
    let mut complete_len = 0usize;
    for cut in (1..pinyin.len()).rev() {
        if !segment_all(table, &pinyin[..cut]).is_empty() {
            complete_len = cut;
            break;
        }
    }
    if complete_len == 0 {
        return empty;
    }
    let completions = dictionary
        .lookup_prefix(pinyin)
        .into_iter()
        .map(|entry| candidate_from_entry(&entry))
        .take(completion_cap.max(1))
        .collect();
    let completed = generate_candidates(table, dictionary, &pinyin[..complete_len]);
    PrefixCandidateGroups {
        completions,
        completed,
    }
}

/// 组 2 + 组 1 有序融合：保留组间顺序（补全组在前），同文本去重、优先保留补全组。
/// 与 `deduplicate_and_sort` 的去重粒度一致（按文本），保证展示确定。
#[must_use]
pub fn merge_candidate_groups(
    completions: Vec<Candidate>,
    completed: Vec<Candidate>,
) -> Vec<Candidate> {
    let mut seen = std::collections::HashSet::new();
    let mut merged = Vec::with_capacity(completions.len() + completed.len());
    for candidate in completions.into_iter().chain(completed) {
        if seen.insert(candidate.text.clone()) {
            merged.push(candidate);
        }
    }
    merged
}

/// 根据拼音串生成候选：整词优先，无整词时按音节切分组合，最后合并去重并确定性排序。
pub fn generate_candidates(
    table: &SyllableTable,
    dictionary: &dyn Dictionary,
    pinyin: &str,
) -> Vec<Candidate> {
    if pinyin.is_empty() {
        return Vec::new();
    }

    let mut collected: Vec<Candidate> = Vec::new();
    let direct_entries = dictionary.lookup(pinyin);
    for entry in &direct_entries {
        collected.push(candidate_from_entry(entry));
    }

    if direct_entries.is_empty() {
        for segments in segment_all(table, pinyin) {
            if segments.len() < 2 {
                continue;
            }
            let mut combined = String::new();
            let mut combined_pinyin = String::new();
            let mut total = 0i64;
            let mut complete = true;
            for syllable in &segments {
                match dictionary.lookup(syllable).first() {
                    Some(entry) => {
                        combined.push_str(&entry.word);
                        combined_pinyin.push_str(syllable);
                        total += i64::try_from(entry.frequency).unwrap_or(i64::MAX);
                    }
                    None => {
                        complete = false;
                        break;
                    }
                }
            }
            if complete {
                collected.push(Candidate::new(combined, total).with_pinyin(combined_pinyin));
            }
        }
    }
    deduplicate_and_sort(collected)
}

fn candidate_from_entry(entry: &DictionaryEntry) -> Candidate {
    let mut candidate = Candidate::new(
        entry.word.clone(),
        i64::try_from(entry.frequency).unwrap_or(i64::MAX),
    );
    candidate = candidate.with_pinyin(entry.pinyin.clone());
    if let Some(translation) = &entry.translation {
        candidate = candidate.with_translation(translation.clone());
    }
    candidate
}

/// 缩写路径的最小触发长度（S-2 定稿：≥2 位，防单字母泛滥）。
pub const ABBREVIATION_MIN_LEN: usize = 2;

/// 判断输入串是否应进入缩写路径（M6-R，方案设计 11.4）。
///
/// 必须**全部**满足：
/// 1. 长度 ≥ `ABBREVIATION_MIN_LEN`（按字符计，数字缩写如 `88` 同样满足）；
/// 2. 仅由 ASCII 小写字母或数字组成（大小写不敏感由调用方归一）；
/// 3. 整串**完全无法切分为标准拼音**（`segment_all` 为空）——可切分串
///    （如 `wo`、`emo`）绝不进入缩写路径，防止污染正常拼音候选。
#[must_use]
pub fn is_abbreviation_input(table: &SyllableTable, input: &str) -> bool {
    if input.chars().count() < ABBREVIATION_MIN_LEN {
        return false;
    }
    if !input
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    {
        return false;
    }
    segment_all(table, input).is_empty()
}

/// 缩写候选组（M6-R）：整串直查 + 前缀补全，标注 `Slang` 来源。
///
/// 只在 `is_abbreviation_input` 为真时调用。返回的候选**不参与默认排序竞争**，
/// 由调用方追加到候选尾部（方案设计 11.4）。
#[must_use]
pub fn abbreviation_candidates(
    table: &SyllableTable,
    slang: &dyn Dictionary,
    input: &str,
    completion_cap: usize,
) -> Vec<Candidate> {
    if !is_abbreviation_input(table, input) {
        return Vec::new();
    }
    let mut collected: Vec<Candidate> = slang
        .lookup(input)
        .iter()
        .map(|entry| candidate_from_entry(entry).with_source(CandidateSource::Slang))
        .collect();
    // 前缀补全：`yy` → yyds；整串直查命中的词条不去重丢弃（下面统一去重）。
    let completions: Vec<Candidate> = slang
        .lookup_prefix(input)
        .into_iter()
        .map(|entry| candidate_from_entry(&entry).with_source(CandidateSource::Slang))
        .collect();
    let mut seen: std::collections::HashSet<String> = collected
        .iter()
        .map(|candidate| candidate.text.clone())
        .collect();
    for candidate in completions.into_iter().take(completion_cap.max(1)) {
        if seen.insert(candidate.text.clone()) {
            collected.push(candidate);
        }
    }
    // 组内按词频降序、同频按文本升序，保证确定性。
    collected.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.text.cmp(&b.text)));
    collected
}

/// 把缩写组追加到主候选尾部（M6-R，方案设计 11.4）。
///
/// 网络语包同时参与拼音路径（纯中文网络词如「内卷」走正常排序），因此缩写键
/// 可能已经被主路径查到。此时**以缩写组为准**：把主组里的同文本候选移除，
/// 改用缩写组版本追加到尾部——这样 `[网络]` 标注与"独立组排尾"同时成立，
/// 且不产生重复候选。
#[must_use]
pub fn append_abbreviation_group(main: Vec<Candidate>, slang: Vec<Candidate>) -> Vec<Candidate> {
    if slang.is_empty() {
        return main;
    }
    let slang_texts: std::collections::HashSet<&str> = slang
        .iter()
        .map(|candidate| candidate.text.as_str())
        .collect();
    // 主组中与缩写组同文本的候选让位给缩写组版本。
    let mut merged: Vec<Candidate> = main
        .into_iter()
        .filter(|candidate| !slang_texts.contains(candidate.text.as_str()))
        .collect();
    merged.extend(slang);
    merged
}

fn deduplicate_and_sort(candidates: Vec<Candidate>) -> Vec<Candidate> {
    let mut by_text: HashMap<String, Candidate> = HashMap::new();
    for candidate in candidates {
        match by_text.get_mut(&candidate.text) {
            Some(existing) => {
                if candidate.score > existing.score {
                    existing.score = candidate.score;
                }
                if existing.translation.is_none() {
                    existing.translation = candidate.translation.clone();
                }
                if existing.pinyin.is_none() {
                    existing.pinyin = candidate.pinyin.clone();
                }
            }
            None => {
                by_text.insert(candidate.text.clone(), candidate);
            }
        }
    }
    CandidateSorter::sort(by_text.into_values().collect())
}

impl CandidateSorter {
    /// 对候选按排序分降序排列。
    #[must_use]
    pub fn sort(mut candidates: Vec<Candidate>) -> Vec<Candidate> {
        candidates.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.text.cmp(&b.text)));
        candidates
    }
}

/// 静态排序模型权重配置。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankingConfig {
    /// 静态词典分权重。
    pub unigram_weight: u64,
    /// bigram 计数权重。
    pub bigram_weight: u64,
    /// 单个 bigram 计数的参与上限，防止高频词过度主导。
    pub bigram_frequency_cap: u64,
    /// 用户词频权重。
    pub user_weight: u64,
    /// 单个用户词的参与上限。
    pub user_frequency_cap: u64,
}

impl Default for RankingConfig {
    fn default() -> Self {
        Self {
            unigram_weight: 1,
            bigram_weight: 16,
            bigram_frequency_cap: 100_000,
            user_weight: 48,
            user_frequency_cap: 10_000,
        }
    }
}

/// 排序上下文：前词与当前用户词库。
#[derive(Debug, Clone, Copy)]
pub struct RankingContext<'a> {
    previous_word: Option<&'a str>,
    user_dictionary: &'a UserDictionary,
}

impl<'a> RankingContext<'a> {
    /// 创建排序上下文。
    #[must_use]
    pub fn new(previous_word: Option<&'a str>, user_dictionary: &'a UserDictionary) -> Self {
        Self {
            previous_word,
            user_dictionary,
        }
    }
}

/// 候选排序模型：负责最终排位，输入引擎通过 trait 注入。
pub trait RankingModel: Send + Sync {
    /// 对候选重新计分并排序；必须确定且不修改上下文。
    fn rank(&self, candidates: Vec<Candidate>, context: &RankingContext<'_>) -> Vec<Candidate>;
}

/// 静态候选排序模型：`unigram + bigram + 用户词频`。
///
/// 公式（全部为整数饱和运算，保证跨平台一致）：
/// `最终分 = 静态分 × unigram权重 + bigram计数(min cap) × bigram权重 + 用户词频(min cap) × 用户权重`
#[derive(Debug)]
pub struct StaticRankingModel {
    config: RankingConfig,
    bigram: Arc<dyn BigramModel>,
}

impl StaticRankingModel {
    /// 创建静态排序模型。
    #[must_use]
    pub fn new(config: RankingConfig, bigram: Arc<dyn BigramModel>) -> Self {
        Self { config, bigram }
    }

    /// 当前权重配置。
    #[must_use]
    pub fn config(&self) -> &RankingConfig {
        &self.config
    }
}

impl Default for StaticRankingModel {
    fn default() -> Self {
        Self::new(RankingConfig::default(), Arc::new(EmptyBigramModel))
    }
}

impl RankingModel for StaticRankingModel {
    fn rank(&self, candidates: Vec<Candidate>, context: &RankingContext<'_>) -> Vec<Candidate> {
        let config = &self.config;
        let ranked = candidates
            .into_iter()
            .map(|mut candidate| {
                let static_part = u64::try_from(candidate.score.max(0))
                    .unwrap_or(u64::MAX)
                    .saturating_mul(config.unigram_weight);

                let bigram_part = context
                    .previous_word
                    .map_or(0, |previous| {
                        self.bigram
                            .frequency(previous, &candidate.text)
                            .min(config.bigram_frequency_cap)
                    })
                    .saturating_mul(config.bigram_weight);

                let user_frequency = context.user_dictionary.frequency_by_word(&candidate.text);
                if user_frequency > 0 {
                    candidate.source = CandidateSource::User;
                }
                let user_part = user_frequency
                    .min(config.user_frequency_cap)
                    .saturating_mul(config.user_weight);

                let total = static_part
                    .saturating_add(bigram_part)
                    .saturating_add(user_part)
                    .min(i64::MAX as u64);
                candidate.score = i64::try_from(total).expect("分数已夹紧到 i64 范围");
                candidate
            })
            .collect::<Vec<_>>();

        CandidateSorter::sort(ranked)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::bigram::InMemoryBigramModel;
    use crate::candidate::{
        abbreviation_candidates, append_abbreviation_group, generate_candidates,
        generate_prefix_candidates, is_abbreviation_input, merge_candidate_groups, Candidate,
        CandidateSorter, CandidateSource, RankingConfig, RankingContext, RankingModel,
        StaticRankingModel,
    };
    use crate::dict::{DictionaryEntry, InMemoryDictionary};
    use crate::pinyin::SyllableTable;
    use crate::user_dict::UserDictionary;

    fn rank_with(
        model: &StaticRankingModel,
        previous: Option<&str>,
        user: &UserDictionary,
        candidates: Vec<Candidate>,
    ) -> Vec<Candidate> {
        model.rank(candidates, &RankingContext::new(previous, user))
    }

    // ---- 缩写输入路径（M6-R）----

    fn slang_dictionary() -> InMemoryDictionary {
        InMemoryDictionary::from_entries(vec![
            DictionaryEntry::new("永远的神", "yyds", 5000),
            DictionaryEntry::new("笑死我了", "xswl", 5000),
            DictionaryEntry::new("有一说一", "u1s1", 5000),
            DictionaryEntry::new("九九六", "996", 5000),
            DictionaryEntry::new("内卷", "neijuan", 5000),
        ])
    }

    #[test]
    fn 缩写输入判定要求不可切分且长度达标() {
        let table = SyllableTable::standard();
        // 不可切分的字母/数字串进入缩写路径。
        assert!(is_abbreviation_input(&table, "yyds"));
        assert!(is_abbreviation_input(&table, "u1s1"));
        assert!(is_abbreviation_input(&table, "996"));
        assert!(is_abbreviation_input(&table, "88"));
        // 单字母不触发（防泛滥）。
        assert!(!is_abbreviation_input(&table, "y"));
        assert!(!is_abbreviation_input(&table, "9"));
        // 可切分串绝不进入缩写路径（防污染）。
        assert!(!is_abbreviation_input(&table, "wo"));
        assert!(!is_abbreviation_input(&table, "ni"));
        assert!(!is_abbreviation_input(&table, "neijuan"));
        // 非小写字母/数字不进入。
        assert!(!is_abbreviation_input(&table, "YYDS"));
        assert!(!is_abbreviation_input(&table, "中国"));
        assert!(!is_abbreviation_input(&table, ""));
    }

    #[test]
    fn 缩写精确命中并标注网络来源() {
        let table = SyllableTable::standard();
        let slang = slang_dictionary();
        let found = abbreviation_candidates(&table, &slang, "yyds", 32);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].text, "永远的神");
        assert_eq!(found[0].source, CandidateSource::Slang);
    }

    #[test]
    fn 缩写前缀补全命中() {
        let table = SyllableTable::standard();
        let slang = slang_dictionary();
        let found = abbreviation_candidates(&table, &slang, "yy", 32);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].text, "永远的神");
        assert_eq!(found[0].source, CandidateSource::Slang);
    }

    #[test]
    fn 缩写数字键可达() {
        let table = SyllableTable::standard();
        let slang = slang_dictionary();
        let found = abbreviation_candidates(&table, &slang, "996", 32);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].text, "九九六");
        let mixed = abbreviation_candidates(&table, &slang, "u1s1", 32);
        assert_eq!(mixed.len(), 1);
        assert_eq!(mixed[0].text, "有一说一");
    }

    #[test]
    fn 可切分串不产生缩写候选() {
        let table = SyllableTable::standard();
        let slang = slang_dictionary();
        assert!(abbreviation_candidates(&table, &slang, "wo", 32).is_empty());
        assert!(abbreviation_candidates(&table, &slang, "y", 32).is_empty());
    }

    #[test]
    fn 缩写组追加尾部且同文本以缩写组为准() {
        let main = vec![Candidate::new("你好", 100)];
        let slang = vec![
            Candidate::new("你好", 5000).with_source(CandidateSource::Slang),
            Candidate::new("永远的神", 5000).with_source(CandidateSource::Slang),
        ];
        let merged = append_abbreviation_group(main, slang);
        // 主组的「你好」被缩写组同文本候选取代，整体只剩两条且顺序为
        // 缩写组内部顺序（词频同分按文本定序：你好 < 永远的神）。
        assert_eq!(merged.len(), 2, "同文本候选应去重");
        assert!(merged.iter().all(|c| c.source == CandidateSource::Slang));
        assert_eq!(
            merged.iter().filter(|c| c.text == "你好").count(),
            1,
            "不得出现重复候选"
        );
        assert!(merged.iter().any(|c| c.text == "永远的神"));
    }

    #[test]
    fn 缩写组不与主组重复且标注保留() {
        let main = vec![
            Candidate::new("你好", 100),
            Candidate::new("永远的神", 9000),
            Candidate::new("世界", 80),
        ];
        let slang = vec![Candidate::new("永远的神", 5000).with_source(CandidateSource::Slang)];
        let merged = append_abbreviation_group(main, slang);
        let texts: Vec<&str> = merged.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(texts, vec!["你好", "世界", "永远的神"]);
        assert_eq!(merged.last().unwrap().source, CandidateSource::Slang);
        assert_eq!(
            merged.iter().filter(|c| c.text == "永远的神").count(),
            1,
            "不得出现重复候选"
        );
    }

    #[test]
    fn 空缩写组不改变主组() {
        let main = vec![Candidate::new("你好", 100)];
        let merged = append_abbreviation_group(main.clone(), Vec::new());
        assert_eq!(merged, main);
    }

    #[test]
    fn 排序确定且高分优先() {
        let candidates = vec![
            Candidate::new("低", 1),
            Candidate::new("高", 10),
            Candidate::new("中", 5),
        ];
        let sorted = CandidateSorter::sort(candidates);
        let texts: Vec<&str> = sorted.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(texts, vec!["高", "中", "低"]);
    }

    #[test]
    fn 分数相同时文本定序() {
        let candidates = vec![Candidate::new("b", 1), Candidate::new("a", 1)];
        let sorted = CandidateSorter::sort(candidates);
        assert_eq!(sorted[0].text, "a");
    }

    #[test]
    fn 静态分高者优先() {
        let user = UserDictionary::new();
        let model = StaticRankingModel::default();
        let sorted = rank_with(
            &model,
            None,
            &user,
            vec![Candidate::new("低", 1), Candidate::new("高", 10)],
        );
        assert_eq!(sorted[0].text, "高");
    }

    #[test]
    fn bigram命中可提升低频词() {
        let mut bigram = InMemoryBigramModel::new();
        bigram.insert("我们", "的", 5_000);
        let model = StaticRankingModel::new(RankingConfig::default(), Arc::new(bigram));
        let user = UserDictionary::new();
        let sorted = rank_with(
            &model,
            Some("我们"),
            &user,
            vec![Candidate::new("得", 200), Candidate::new("的", 100)],
        );
        assert_eq!(sorted[0].text, "的");
    }

    #[test]
    fn bigram未命中时回退静态分() {
        let model = StaticRankingModel::default();
        let user = UserDictionary::new();
        let sorted = rank_with(
            &model,
            Some("我们"),
            &user,
            vec![Candidate::new("得", 200), Candidate::new("的", 100)],
        );
        assert_eq!(sorted[0].text, "得");
    }

    #[test]
    fn 用户词频可提升候选并标记来源() {
        let mut user = UserDictionary::new();
        for now in 1..=10 {
            user.record_selection("爱", "ai", now);
        }
        let model = StaticRankingModel::default();
        let sorted = rank_with(
            &model,
            None,
            &user,
            vec![Candidate::new("暗", 300), Candidate::new("爱", 10)],
        );
        assert_eq!(sorted[0].text, "爱");
        assert_eq!(sorted[0].source, crate::candidate::CandidateSource::User);
    }

    #[test]
    fn 同一输入两次排序结果一致() {
        let mut bigram = InMemoryBigramModel::new();
        bigram.insert("我们", "的", 9_999);
        let model = StaticRankingModel::new(RankingConfig::default(), Arc::new(bigram));
        let mut user = UserDictionary::new();
        user.record_selection("爱", "ai", 1);

        let candidates = || {
            vec![
                Candidate::new("得", 200),
                Candidate::new("爱", 10),
                Candidate::new("的", 100),
            ]
        };
        let first = rank_with(&model, Some("我们"), &user, candidates());
        let second = rank_with(&model, Some("我们"), &user, candidates());
        assert_eq!(first, second);
    }

    #[test]
    fn 候选生成确定且整词与切分合并去重() {
        use crate::demo::seed_entries;
        use crate::dict::InMemoryDictionary;
        use crate::pinyin::SyllableTable;

        let table = SyllableTable::standard();
        let dictionary = InMemoryDictionary::from_entries(seed_entries());
        let first = generate_candidates(&table, &dictionary, "xian");
        let second = generate_candidates(&table, &dictionary, "xian");
        assert_eq!(first, second);
        let texts: Vec<&str> = first.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(texts.iter().filter(|t| **t == "西安").count(), 1);
    }

    #[test]
    fn 整词存在时不生成音节切分噪声() {
        use crate::demo::seed_entries;
        use crate::dict::{DictionaryEntry, InMemoryDictionary};
        use crate::pinyin::SyllableTable;

        let table = SyllableTable::standard();
        let mut entries = seed_entries();
        entries.push(DictionaryEntry::new("给", "ji", 1000));
        entries.push(DictionaryEntry::new("啊", "a", 1000));
        entries.push(DictionaryEntry::new("哦", "o", 1000));
        entries.push(DictionaryEntry::new("叫", "jiao", 100));
        let dictionary = InMemoryDictionary::from_entries(entries);
        let candidates = generate_candidates(&table, &dictionary, "jiao");
        let texts: Vec<&str> = candidates.iter().map(|c| c.text.as_str()).collect();
        assert!(texts.contains(&"叫"));
        assert!(!texts.contains(&"给啊哦"));
    }

    #[test]
    fn 无整词时按音节组合保留回退候选() {
        use crate::dict::{DictionaryEntry, InMemoryDictionary};
        use crate::pinyin::SyllableTable;

        let table = SyllableTable::standard();
        let dictionary = InMemoryDictionary::from_entries(vec![
            DictionaryEntry::new("你", "ni", 100),
            DictionaryEntry::new("好", "hao", 90),
        ]);
        let candidates = generate_candidates(&table, &dictionary, "nihao");
        let texts: Vec<&str> = candidates.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(texts, vec!["你好"]);
    }

    #[test]
    fn 前缀候选补全组优先于完成组() {
        use crate::dict::{DictionaryEntry, InMemoryDictionary};
        use crate::pinyin::SyllableTable;

        let table = SyllableTable::standard();
        let dictionary = InMemoryDictionary::from_entries(vec![
            DictionaryEntry::new("你好", "nihao", 100).with_translation("hello"),
            DictionaryEntry::new("泥好", "nihao", 30),
            DictionaryEntry::new("你", "ni", 200),
            DictionaryEntry::new("泥", "ni", 50),
        ]);
        let groups = generate_prefix_candidates(&table, &dictionary, "nih", 32);
        let completions: Vec<&str> = groups.completions.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(completions, vec!["你好", "泥好"]);
        let completed: Vec<&str> = groups.completed.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(completed, vec!["你", "泥"]);

        let merged = merge_candidate_groups(groups.completions, groups.completed);
        let texts: Vec<&str> = merged.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(texts, vec!["你好", "泥好", "你", "泥"]);
    }

    #[test]
    fn 前缀候选补全截断到上限() {
        use crate::dict::{DictionaryEntry, InMemoryDictionary};
        use crate::pinyin::SyllableTable;

        let table = SyllableTable::standard();
        let dictionary = InMemoryDictionary::from_entries(vec![
            DictionaryEntry::new("你好", "nihao", 100),
            DictionaryEntry::new("泥好", "nihao", 30),
            DictionaryEntry::new("你", "ni", 200),
        ]);
        let groups = generate_prefix_candidates(&table, &dictionary, "nih", 1);
        assert_eq!(groups.completions.len(), 1);
        assert_eq!(groups.completions[0].text, "你好");
        // 完成组不受补全上限影响。
        assert_eq!(
            groups
                .completed
                .iter()
                .map(|c| c.text.as_str())
                .collect::<Vec<_>>(),
            vec!["你"]
        );
    }

    #[test]
    fn 前缀候选无完整音节或可完整切分时为空() {
        use crate::dict::{DictionaryEntry, InMemoryDictionary};
        use crate::pinyin::SyllableTable;

        let table = SyllableTable::standard();
        let dictionary = InMemoryDictionary::from_entries(vec![
            DictionaryEntry::new("你好", "nihao", 100),
            DictionaryEntry::new("你", "ni", 200),
        ]);
        // `zh` 前无完整音节：两组皆空，避免前缀泛滥。
        let groups = generate_prefix_candidates(&table, &dictionary, "zh", 32);
        assert!(groups.completions.is_empty());
        assert!(groups.completed.is_empty());
        // 可完整切分时不由前缀逻辑处理。
        let groups = generate_prefix_candidates(&table, &dictionary, "nihao", 32);
        assert!(groups.completions.is_empty());
        assert!(groups.completed.is_empty());
    }

    #[test]
    fn 前缀候选生成确定性() {
        use crate::dict::{DictionaryEntry, InMemoryDictionary};
        use crate::pinyin::SyllableTable;

        let table = SyllableTable::standard();
        let dictionary = InMemoryDictionary::from_entries(vec![
            DictionaryEntry::new("你好", "nihao", 100),
            DictionaryEntry::new("泥好", "nihao", 30),
            DictionaryEntry::new("你", "ni", 200),
        ]);
        let first = generate_prefix_candidates(&table, &dictionary, "nih", 32);
        let second = generate_prefix_candidates(&table, &dictionary, "nih", 32);
        assert_eq!(first, second);
        assert_eq!(
            merge_candidate_groups(first.completions, first.completed),
            merge_candidate_groups(second.completions, second.completed)
        );
    }

    #[test]
    fn 融合去重保留补全组() {
        let merged = merge_candidate_groups(
            vec![Candidate::new("你好", 100)],
            vec![Candidate::new("你好", 1), Candidate::new("泥", 50)],
        );
        let texts: Vec<&str> = merged.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(texts, vec!["你好", "泥"]);
    }
}
