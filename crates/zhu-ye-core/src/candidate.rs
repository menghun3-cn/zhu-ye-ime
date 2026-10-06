//! 候选类型与排序。
//!
//! 底层 `CandidateSorter` 只负责确定性基础排序；候选排序模型（trait）
//! 负责把静态词典分、上下文 bigram 与用户词频加权成最终排序分。
use std::collections::HashMap;

use std::sync::Arc;

use crate::bigram::{BigramModel, EmptyBigramModel};
use crate::dict::{Dictionary, DictionaryEntry};
use crate::pinyin::{fuzzy_variants, initial_syllables, segment_all, SyllableTable};
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
    /// 模糊音/纠错候选（M7，FR-024）；UI 不新增标签，仅 source 区分。
    Corrected,
    /// 上屏联想候选（T-058/T-059，场景5）；仅出现在空闲候选窗，UI 不新增标签。
    Suggestion,
    /// 数字格式候选（FR-027，场景7）；仅出现在空闲候选窗，UI 不新增标签。
    NumberFormat,
    /// v 模式符号候选（FR-028，场景7）；仅出现在空闲候选窗，UI 不新增标签。
    Symbol,
    /// emoji 推荐候选（FR-029，场景7）；追在拼音候选尾部，UI 不新增标签。
    Emoji,
    /// 英文词候选（FR-030，场景6）；主候选之后追加英文组，UI 不新增标签。
    EnWord,
    /// 邮箱/网址补全候选（FR-031，场景6）；组合态 @/www./http 前缀路径，UI 不新增标签。
    EmailUrl,
    /// 领域提权候选（FR-033/FR-034，场景8）；完整词命中启用领域包时按 D-13 位次
    /// 插基础候选之后、追加组之前，UI 不新增标签。
    Domain,
    /// 联系人提权候选（FR-036/FR-037，场景9）；姓名拼音/简拼命中时与领域提权
    /// 共用协调层（D-21），UI 不新增标签。
    Contact,
    /// 混合串整句/分段候选（FR-050，T-086，场景6）；`python代码` 类输入置首
    /// 整句 + 分段候选，UI 不新增标签。
    Mixed,
    /// 前缀组词展开候选（FR-059，T-090）；完整拼音整词命中不足一页时按拼音前缀
    /// 补足更深组词（D-70/D-71），独立追加组不参与主排序，UI 不新增标签。
    PrefixExpand,
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
    // 找最长可完整切分前缀 P；找不到（如 `d`/`z`/`zh`）时无完成组，
    // 但补全组仍照常给出（单字母/声母前缀即时出候选：`d` → 的/多/到……，
    // 搜狗/微软拼音同款体验），长度由 `completion_cap` 封顶避免前缀泛滥。
    let mut complete_len = 0usize;
    for cut in (1..pinyin.len()).rev() {
        if !segment_all(table, &pinyin[..cut]).is_empty() {
            complete_len = cut;
            break;
        }
    }
    let completions = dictionary
        .lookup_prefix(pinyin)
        .into_iter()
        .map(|entry| candidate_from_entry(&entry))
        .take(completion_cap.max(1))
        .collect();
    if complete_len == 0 {
        return PrefixCandidateGroups {
            completions,
            completed: Vec::new(),
        };
    }
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

/// 前缀组词展开（FR-059，T-090）：完整拼音整词命中不足一页时，按拼音前缀把
/// 更深的组词补足候选（D-70 候选不足一页才展开、D-71 独立追加组按词频取前 N）。
///
/// 触发条件（完整音节串且有整词命中）由调用方（引擎）判定并给出 `fill`；本函数只做
/// "展开"本身：
/// 1. `fill == 0` 或空拼音 → 返回空（不展开）；
/// 2. `lookup_prefix(pinyin)` 取拼音以输入串开头的词条（自然含 `shui` 更深音节组词），
///    `candidate_from_entry` 转候选并标注 `PrefixExpand` 来源；
/// 3. 与 `already`（整词命中组）同文本剔除，展开组内部同文本只保留第一个；
/// 4. 组内按词频降序、同频文本升序（与 `CandidateSorter` 全序一致）后截断到 `fill` 条。
///
/// 展开组**不参与主候选排序**，由调用方追加到整词命中组之后；任何失败返回空，不 panic。
#[must_use]
pub fn prefix_expand_candidates(
    dictionary: &dyn Dictionary,
    pinyin: &str,
    fill: usize,
    already: &[Candidate],
) -> Vec<Candidate> {
    if fill == 0 || pinyin.is_empty() {
        return Vec::new();
    }
    let mut seen: std::collections::HashSet<String> = already
        .iter()
        .map(|candidate| candidate.text.clone())
        .collect();
    let mut collected: Vec<Candidate> = dictionary
        .lookup_prefix(pinyin)
        .into_iter()
        .map(|entry| candidate_from_entry(&entry).with_source(CandidateSource::PrefixExpand))
        .filter(|candidate| seen.insert(candidate.text.clone()))
        .collect();
    collected.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.text.cmp(&b.text)));
    collected.truncate(fill);
    collected
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

/// 简拼最小触发长度（M7，方案设计 12.2.3）。
pub const INITIAL_MIN_LEN: usize = 2;
/// 简拼最大触发长度（超过放弃，防组合爆炸）。
pub const INITIAL_MAX_LEN: usize = 4;
/// 简拼展开组合最多进入结果的总条数。
pub const INITIAL_COMPLETION_CAP: usize = 32;

/// 英文词候选（FR-030，场景6）：查 `EN_WORDS` 前缀得到英文原形候选组。
///
/// 只在调用方确认"整串不可按拼音切分"后调用（与缩写路径同判定，见 `is_abbreviation_input`）；
/// 候选**不参与默认排序竞争**，由调用方追加到主候选尾部（D-10），`score` 取组内负排名
/// 仅用于保持 freq_rank 升序展示。`pinyin = None`，不进入用户词学习。
///
/// 本函数走第五期内嵌静态表（`en_words.rs`）；发行运行时经 `en_word_candidates_from`
/// 走 `en.zyen` 文件词表（T-085），静态表仅为无文件回退，行为完全一致。
#[must_use]
pub fn en_word_candidates(input: &str, limit: usize) -> Vec<Candidate> {
    crate::en_words::en_words_with_prefix(input, limit)
        .into_iter()
        .map(|(word, rank)| {
            Candidate::new(word, -(rank as i64)).with_source(CandidateSource::EnWord)
        })
        .collect()
}

/// 英文词候选（FR-030，场景6）：查 `en.zyen` 文件词表前缀得到英文原形候选组。
///
/// 语义与 `en_word_candidates` 完全一致（D-09/D-10 保持），仅数据源换为
/// T-085 的 mmap 词表；rank 为文件内记录下标（构建期按词频排序，越小越常用）。
#[must_use]
pub fn en_word_candidates_from(
    lexicon: &crate::en_lexicon::EnLexicon,
    input: &str,
    limit: usize,
) -> Vec<Candidate> {
    lexicon
        .words_with_prefix(input, limit)
        .into_iter()
        .map(|(word, rank)| {
            Candidate::new(word, -(rank as i64)).with_source(CandidateSource::EnWord)
        })
        .collect()
}

/// 简拼/首字母候选（M7，FR-023，方案设计 12.2）。
///
/// 输入为 2-4 位纯小写 ASCII 首字母串（如 `nh`），按 `pinyin::INITIAL_SYLLABLE_TABLE`
/// 前序笛卡尔积展开为完整拼音串，整词命中词典即产候选。
///
/// 触发前置由调用方保证（不可完整切分、长度合规、纯字母），本函数内部仍做防御：
/// - 任一字母无简拼音节 → 返回空
/// - 展开组合数超过上限 → 返回空（防组合爆炸：4 位 × 6 音节 = 1296 种，超预算）
///
/// 返回顺序 = 表序展开顺序（确定性）；由调用方并入主候选后统一排序。
#[must_use]
pub fn initial_candidates(dictionary: &dyn Dictionary, initials: &str) -> Vec<Candidate> {
    let chars: Vec<char> = initials.chars().collect();
    if chars.len() < INITIAL_MIN_LEN || chars.len() > INITIAL_MAX_LEN {
        return Vec::new();
    }
    if !chars.iter().all(|c| c.is_ascii_lowercase()) {
        return Vec::new();
    }
    let sets: Vec<&'static [&'static str]> = chars.iter().map(|c| initial_syllables(*c)).collect();
    if sets.iter().any(|set| set.is_empty()) {
        return Vec::new();
    }
    // 前序笛卡尔积：控制组合数上限，超出即放弃（防长串组合爆炸）。
    const MAX_COMBINATIONS: usize = 128;
    let mut combos: Vec<String> = vec![String::new()];
    for set in &sets {
        let mut next = Vec::with_capacity(combos.len().saturating_mul(set.len()));
        for prefix in &combos {
            for syllable in *set {
                let mut combo = prefix.clone();
                combo.push_str(syllable);
                next.push(combo);
            }
        }
        combos = next;
        if combos.len() > MAX_COMBINATIONS {
            return Vec::new();
        }
    }
    let mut collected: Vec<Candidate> = Vec::new();
    for combo in combos {
        let entries = dictionary.lookup(&combo);
        for entry in entries {
            let mut candidate = candidate_from_entry(&entry);
            candidate.pinyin = Some(combo.clone());
            collected.push(candidate);
            if collected.len() >= INITIAL_COMPLETION_CAP {
                return collected;
            }
        }
    }
    collected
}

/// 纠错变体上限（M7，方案设计 12.3.3）。
pub const CORRECTION_VARIANT_CAP: usize = 24;

/// 模糊音与纠错候选（M7，FR-024，方案设计 12.3）。
///
/// 输入串必须**可完整切分**且**无整词命中**（由调用方保证，本函数内部防御）。
/// 两类纠错：
/// 1. **模糊替换**：对切分中每个音节做 `fuzzy_variants`（如 `zong`→`zhong`）；
/// 2. **少字母补全**：对最后一个音节枚举以它为前缀的完整音节（如 `ha`→`hao`）。
///
/// 变体拼音串命中词典的候选标注 `CandidateSource::Corrected`，作为独立组
/// 追加在主候选之后（UI 不新增标签）。数量受 `CORRECTION_VARIANT_CAP` 约束。
#[must_use]
pub fn corrected_candidates(
    table: &SyllableTable,
    dictionary: &dyn Dictionary,
    pinyin: &str,
) -> Vec<Candidate> {
    if pinyin.is_empty() || !dictionary.lookup(pinyin).is_empty() {
        return Vec::new();
    }
    let segments = segment_all(table, pinyin);
    let Some(segments) = segments.first() else {
        return Vec::new();
    };
    let syllable_count = segments.len();
    let mut collected: Vec<Candidate> = Vec::new();
    let mut seen_pinyin: std::collections::HashSet<String> = std::collections::HashSet::new();

    for (index, syllable) in segments.iter().enumerate() {
        let variants = if index + 1 == syllable_count {
            // 尾音节：模糊替换 + 少字母补全。
            let mut variants = fuzzy_variants(syllable);
            variants.extend(
                table
                    .complete_syllables_with_prefix(syllable)
                    .into_iter()
                    .filter(|completed| *completed != syllable)
                    .map(str::to_owned),
            );
            variants
        } else {
            fuzzy_variants(syllable)
        };
        for variant in variants {
            if !table.is_complete_syllable(&variant) {
                continue;
            }
            let mut replaced = segments.clone();
            replaced[index] = variant.clone();
            let new_pinyin = replaced.concat();
            if !seen_pinyin.insert(new_pinyin.clone()) {
                continue;
            }
            for entry in dictionary.lookup(&new_pinyin) {
                let mut candidate = candidate_from_entry(&entry);
                candidate.source = CandidateSource::Corrected;
                candidate.pinyin = Some(new_pinyin.clone());
                collected.push(candidate);
                if collected.len() >= CORRECTION_VARIANT_CAP {
                    return collected;
                }
            }
        }
    }
    collected
}

/// 错序容错候选上限（T-115 后续）：与 `CORRECTION_VARIANT_CAP` 同量级，
/// 防止变体枚举把候选列表撑爆。
pub const TRANSPOSITION_VARIANT_CAP: usize = 16;

/// 错序容错（T-115 后续）：输入串**无法完整切分**时（快打常见的相邻字母
/// 颠倒，如 `zhegnq`→`zhengq`、`shegnc`→`shengc`、`xiagnzhe`→`xiangzhe`、
/// `zhagnh`→`zhangh`），枚举每对相邻字母交换后的变体出候选：
///
/// - 变体**可完整切分** → 走主链路 `generate_candidates`（整词/音节切分组合）；
/// - 变体**仍不可切分**（如 `zhangh` 尾音节残）→ 走前缀候选链路
///   （补全组优先，`账号` 即由 `zhangh` 补全命中）。
///
/// 与 `corrected_candidates`（FR-024）互补：后者只处理**可完整切分**串的
/// 模糊替换/少字母补全；错位串此前被两条链路都漏掉，用户点名报缺陷。
/// 候选标注 `CandidateSource::Corrected` 且 `pinyin` 为变体（正确拼音），
/// 候选窗据此展示"词（正确拼音）"。数量受 `TRANSPOSITION_VARIANT_CAP` 约束。
#[must_use]
pub fn transposed_candidates(
    table: &SyllableTable,
    dictionary: &dyn Dictionary,
    pinyin: &str,
    completion_cap: usize,
) -> Vec<Candidate> {
    if pinyin.is_empty()
        || !pinyin.is_ascii()
        || !dictionary.lookup(pinyin).is_empty()
        || !segment_all(table, pinyin).is_empty()
    {
        // 整词已命中或可完整切分：无需错序容错（由主链路承担）。
        return Vec::new();
    }
    let mut collected: Vec<Candidate> = Vec::new();
    let mut seen_pinyin: std::collections::HashSet<String> = std::collections::HashSet::new();
    for variant in transposition_variants(pinyin) {
        if !seen_pinyin.insert(variant.clone()) {
            continue;
        }
        let variant_candidates = if segment_all(table, &variant).is_empty() {
            let groups = generate_prefix_candidates(table, dictionary, &variant, completion_cap);
            let mut all = groups.completions;
            all.extend(groups.completed);
            all
        } else {
            generate_candidates(table, dictionary, &variant)
        };
        for mut candidate in variant_candidates {
            candidate.source = CandidateSource::Corrected;
            candidate.pinyin = Some(variant.clone());
            collected.push(candidate);
            if collected.len() >= TRANSPOSITION_VARIANT_CAP {
                return collected;
            }
        }
    }
    collected
}

/// 相邻字母交换变体：对输入串每对相邻位置交换一次生成候选串。
fn transposition_variants(input: &str) -> Vec<String> {
    let bytes = input.as_bytes();
    let mut variants = Vec::new();
    for index in 0..bytes.len().saturating_sub(1) {
        if bytes[index] == bytes[index + 1] {
            continue;
        }
        let mut swapped = Vec::with_capacity(bytes.len());
        swapped.extend_from_slice(&bytes[..index]);
        swapped.push(bytes[index + 1]);
        swapped.push(bytes[index]);
        swapped.extend_from_slice(&bytes[index + 2..]);
        variants.push(String::from_utf8(swapped).expect("纯 ASCII 输入交换后仍为 ASCII"));
    }
    variants
}

/// Beam Search 参数（M7，方案设计 12.4.2）。
pub const BEAM_WIDTH: usize = 8;
/// 每个（子串）最多参与搜索的候选词数。
pub const BEAM_WORD_CAP: usize = 4;
/// 整句候选最多返回条数。
pub const SENTENCE_TOP_N: usize = 5;
/// 单个词最多覆盖的拼音字符数（4 个音节 × 平均 3-4 字符，防长串搜索爆炸）。
pub const SENTENCE_MAX_WORD_CHARS: usize = 12;
/// 词单频（unigram）进入整句评分前的上限，与 `bigram_frequency_cap` 同量级。
/// 修复 M7-A 实测暴露的问题：真实词库中超高频单字（如「被」）若放行原始词频，
/// 会凭 unigram 碾压多音节整词，导致 beam 退化为逐字拼接。
pub const SENTENCE_UNIGRAM_CAP: i64 = 100_000;
/// 前词→本词无任何 bigram 证据（频率为 0）时的路径惩罚。
/// 整句评分按"词间转移"建模：`想去北京` 有证据得分，`去被敬` 无证据被罚，
/// 使随机单字拼接无法通过累加小分值胜过自然搭配。
/// 罚额与 `SENTENCE_UNIGRAM_CAP` 等值：M7-A 实测发现若罚 < cap，超高频单字
/// （如介词「被」）罚后残值仍为正，两个残值累积即可压过低频整词（如「北京」）。
/// 罚 = cap 使无证据词贡献 ≤ 0，随机拼接路径整体必然深于有证据路径。
pub const SENTENCE_BIGRAM_MISS_PENALTY: i64 = SENTENCE_UNIGRAM_CAP;

/// 整句 Beam Search 候选（M7，FR-025，方案设计 12.4）。
///
/// 输入串必须**可完整切分**、**音节数 ≥3**且**无整词命中**（由调用方保证，
/// 本函数内部防御）。采用**跨音节整词匹配**的 beam 搜索：
/// - 在每个拼音位置上，枚举所有「可完整切分的子串」（1 至 `SENTENCE_MAX_WORD_CHARS`
///   字符），直接 `lookup` 命中词典词条——`mingtian` 这类两音节整词可被选中；
/// - 评分（与 `StaticRankingModel` 同权重体系，M7-A 修订）：
///   首词 = min(词频, `SENTENCE_UNIGRAM_CAP`) × unigram权重；
///   后续词 = 前词→本词 bigram 有证据时
///   min(词频, cap) × unigram权重 + min(bigram, cap) × bigram权重，
///   无 bigram 证据时 min(词频, cap) × unigram权重 − `SENTENCE_BIGRAM_MISS_PENALTY`；
/// - 每步保 `BEAM_WIDTH` 条路径，每个子串取 `BEAM_WORD_CAP` 个候选词；
/// - 合并所有完整路径，同文本去重取最高分，返回前 `SENTENCE_TOP_N`。
///
/// 整词命中与音节数不足的输入返回空（调用方走现状路径）。整句候选
/// `source = Static`、`pinyin = 原输入串`，由调用方置于主候选最前。
/// 全部路径耗尽返回空（调用方回退单字拼接，保证 ≥1 候选）。
#[must_use]
pub fn sentence_candidates(
    table: &SyllableTable,
    dictionary: &dyn Dictionary,
    bigram: &dyn BigramModel,
    pinyin: &str,
) -> Vec<Candidate> {
    if pinyin.is_empty() || !dictionary.lookup(pinyin).is_empty() {
        return Vec::new();
    }
    // 防抖：少于 3 个音节的长串不进 beam（短串沿用现状，避免行为漂移）。
    let mut all_segments = segment_all(table, pinyin);
    let min_syllables = all_segments
        .iter()
        .map(Vec::len)
        .min()
        .unwrap_or(usize::MAX);
    if min_syllables < 3 {
        return Vec::new();
    }
    let _ = std::mem::take(&mut all_segments); // 已用 min 统计，释放中间结果

    let config = RankingConfig::default();
    // beam: (词序列, 已消费字符数, 累计分)。字符数保证终止（每步至少消费 1 字符）。
    let mut beam: Vec<(Vec<String>, usize, i64)> = vec![(Vec::new(), 0, 0)];
    let len = pinyin.len();
    // 至多 len 轮（每轮至少消费 1 字符），保证有限步终止。
    // `completed` 跨轮保留已消费到串尾的路径——整词步长（如「明天」8 字符）会让
    // 高分局 路径提前完成；若只从当轮 `next_beam` 收集，慢（逐字）路径会在后续
    // 轮次把提前完成的路径挤出 beam，导致 top 结果退化（M7-A 实测发现）。
    let mut completed: Vec<(Vec<String>, usize, i64)> = Vec::new();
    for _ in 0..len {
        let mut next_beam: Vec<(Vec<String>, usize, i64)> = Vec::new();
        for (words, pos, score) in &beam {
            let end_max = (*pos + SENTENCE_MAX_WORD_CHARS).min(len);
            for end in (*pos + 1)..=end_max {
                let sub = &pinyin[*pos..end];
                if segment_all(table, sub).is_empty() {
                    continue;
                }
                let entries = dictionary.lookup(sub);
                if entries.is_empty() {
                    continue;
                }
                for entry in entries.iter().take(BEAM_WORD_CAP) {
                    // unigram 上限：防止超高频单字以原始词频垄断整句评分（M7-A 修订）。
                    let unigram_capped = i64::try_from(entry.frequency)
                        .unwrap_or(i64::MAX)
                        .min(SENTENCE_UNIGRAM_CAP)
                        .saturating_mul(i64::try_from(config.unigram_weight).unwrap_or(1));
                    let bigram_score = words.last().map_or(0, |previous| {
                        bigram
                            .frequency(previous, &entry.word)
                            .min(config.bigram_frequency_cap)
                    });
                    let add = if words.is_empty() || bigram_score > 0 {
                        // 首词或存在词间转移证据：unigram + bigram 加成。
                        unigram_capped.saturating_add(
                            i64::try_from(bigram_score)
                                .unwrap_or(i64::MAX)
                                .saturating_mul(i64::try_from(config.bigram_weight).unwrap_or(1)),
                        )
                    } else {
                        // 前词→本词无证据：随机拼接惩罚，防止「去被敬」类路径胜出。
                        unigram_capped.saturating_sub(SENTENCE_BIGRAM_MISS_PENALTY)
                    };
                    let mut next_words = words.clone();
                    next_words.push(entry.word.clone());
                    next_beam.push((next_words, end, score.saturating_add(add)));
                }
            }
        }
        if next_beam.is_empty() {
            break; // 全部路径耗尽，回退由调用方负责
        }
        // 分离「已到串尾」与「继续展开」：完成路径并入 completed 跨轮保留。
        let (mut done, mut pending): (Vec<_>, Vec<_>) =
            next_beam.into_iter().partition(|(_, pos, _)| *pos == len);
        if !done.is_empty() {
            done.append(&mut completed);
            done.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.join("").cmp(&b.0.join(""))));
            done.truncate(BEAM_WIDTH);
            completed = done;
        }
        if pending.is_empty() {
            break; // 全部路径均已完成
        }
        // 截断到 BEAM_WIDTH：累计分降序，同分按词文本序（确定性）。
        pending.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.join("").cmp(&b.0.join(""))));
        pending.truncate(BEAM_WIDTH);
        beam = pending;
    }

    let mut sentence_scores: HashMap<String, i64> = HashMap::new();
    for (words, pos, score) in beam.iter().chain(completed.iter()) {
        if *pos != len {
            continue;
        }
        let sentence = words.join("");
        let existing = sentence_scores.entry(sentence.clone()).or_insert(i64::MIN);
        if *score > *existing {
            *existing = *score;
        }
    }

    let mut sentences: Vec<(String, i64)> = sentence_scores.into_iter().collect();
    sentences.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    sentences.truncate(SENTENCE_TOP_N);
    sentences
        .into_iter()
        .map(|(sentence, score)| Candidate::new(sentence, score).with_pinyin(pinyin.to_owned()))
        .collect()
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
        abbreviation_candidates, append_abbreviation_group, corrected_candidates,
        generate_candidates, generate_prefix_candidates, initial_candidates, is_abbreviation_input,
        merge_candidate_groups, prefix_expand_candidates, sentence_candidates,
        transposed_candidates, Candidate, CandidateSorter, CandidateSource, RankingConfig,
        RankingContext, RankingModel, StaticRankingModel,
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
    fn 前缀候选无完整音节时只给补全组可完整切分时为空() {
        use crate::dict::{DictionaryEntry, InMemoryDictionary};
        use crate::pinyin::SyllableTable;

        let table = SyllableTable::standard();
        let dictionary = InMemoryDictionary::from_entries(vec![
            DictionaryEntry::new("你好", "nihao", 100),
            DictionaryEntry::new("你", "ni", 200),
            DictionaryEntry::new("中", "zhong", 60),
            DictionaryEntry::new("这", "zhe", 40),
        ]);
        // `zh` 前无完整音节：无完成组，但补全组仍给出（单字母/声母前缀即时出候选，
        // `d`/`z`/`zh` 一按就出现 的/多/到 之类，搜狗/微软同款；上限由 cap 封顶）。
        let groups = generate_prefix_candidates(&table, &dictionary, "zh", 32);
        assert!(groups.completed.is_empty());
        let completions: Vec<&str> = groups.completions.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(completions, vec!["中", "这"]);
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

    // ---- 前缀组词展开（FR-059，T-090，第十一期）----

    fn shui_expand_dictionary() -> InMemoryDictionary {
        InMemoryDictionary::from_entries(vec![
            DictionaryEntry::new("说", "shui", 413852),
            DictionaryEntry::new("谁", "shui", 127180),
            DictionaryEntry::new("睡", "shui", 80000),
            DictionaryEntry::new("水果", "shuiguo", 90000),
            DictionaryEntry::new("水平", "shuiping", 85000),
            DictionaryEntry::new("睡觉", "shuijiao", 70000),
            DictionaryEntry::new("水稻", "shuidao", 120000),
            // 拼音前缀更长的无关词不命中：
            DictionaryEntry::new("说明", "shuoming", 200000),
        ])
    }

    #[test]
    fn 前缀组词展开取更深组词并标注来源() {
        let dictionary = shui_expand_dictionary();
        // already 模拟真实主组：全部单音节整词命中（说/谁/睡）。
        let already = vec![
            Candidate::new("说", 413852),
            Candidate::new("谁", 127180),
            Candidate::new("睡", 80000),
        ];
        let expanded = prefix_expand_candidates(&dictionary, "shui", 4, &already);
        let texts: Vec<&str> = expanded.iter().map(|c| c.text.as_str()).collect();
        // 词频降序：水稻(120000) > 水果(90000) > 水平(85000) > 睡觉(70000)。
        assert_eq!(texts, vec!["水稻", "水果", "水平", "睡觉"]);
        assert!(expanded
            .iter()
            .all(|c| c.source == CandidateSource::PrefixExpand));
        // 拼音以 shui 开头的单音节命中已被 already（整词命中组）剔除，
        // 更窄前缀的 shuoming 不命中。
    }

    #[test]
    fn 前缀组词展开与整词命中同文本剔除() {
        let dictionary = shui_expand_dictionary();
        let already: Vec<Candidate> = vec![
            Candidate::new("说", 413852),
            Candidate::new("水果", 90000).with_source(CandidateSource::PrefixExpand),
        ];
        let expanded = prefix_expand_candidates(&dictionary, "shui", 8, &already);
        assert!(!expanded.iter().any(|c| c.text == "水果"));
        // 其余更深组词与未覆盖的单音节命中仍按词频展开（说 已剔除；谁/睡 未被 already 覆盖）。
        let texts: Vec<&str> = expanded.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(texts, vec!["谁", "水稻", "水平", "睡", "睡觉"]);
    }

    #[test]
    fn 前缀组词展开截断到填充上限() {
        let dictionary = shui_expand_dictionary();
        let already = vec![
            Candidate::new("说", 413852),
            Candidate::new("谁", 127180),
            Candidate::new("睡", 80000),
        ];
        let expanded = prefix_expand_candidates(&dictionary, "shui", 2, &already);
        let texts: Vec<&str> = expanded.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(texts, vec!["水稻", "水果"]);
        // fill 大于可用量时给出全部（不满一页是允许的）。
        let expanded = prefix_expand_candidates(&dictionary, "shui", 8, &already);
        assert_eq!(expanded.len(), 4);
    }

    #[test]
    fn 前缀组词展开空输入或零填充为空() {
        let dictionary = shui_expand_dictionary();
        assert!(prefix_expand_candidates(&dictionary, "shui", 0, &[]).is_empty());
        assert!(prefix_expand_candidates(&dictionary, "", 4, &[]).is_empty());
        // 无前缀命中的拼音 → 空。
        let empty_dict = InMemoryDictionary::default();
        assert!(prefix_expand_candidates(&empty_dict, "shui", 4, &[]).is_empty());
    }

    #[test]
    fn 前缀组词展开内部同文本去重且确定性() {
        // 词典含同词形两条（词频不同）：展开组内只保留第一条（按文本去重语义）。
        let dictionary = InMemoryDictionary::from_entries(vec![
            DictionaryEntry::new("水平", "shuiping", 85000),
            DictionaryEntry::new("水平", "shuiping", 100),
            DictionaryEntry::new("睡觉", "shuijiao", 70000),
        ]);
        let expanded = prefix_expand_candidates(&dictionary, "shui", 8, &[]);
        let texts: Vec<&str> = expanded.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(texts, vec!["水平", "睡觉"]);
        assert_eq!(
            expanded,
            prefix_expand_candidates(&dictionary, "shui", 8, &[])
        );
    }

    // ---- 输入体验优化（M7，FR-023 至 FR-025）----

    fn m7_dictionary() -> InMemoryDictionary {
        InMemoryDictionary::from_entries(vec![
            DictionaryEntry::new("你好", "nihao", 100),
            DictionaryEntry::new("泥好", "nihao", 30),
            DictionaryEntry::new("为什么", "weishenme", 80),
            DictionaryEntry::new("我是", "woshi", 70),
            DictionaryEntry::new("我们", "women", 60),
            DictionaryEntry::new("你", "ni", 90),
            DictionaryEntry::new("好", "hao", 85),
            DictionaryEntry::new("我", "wo", 88),
            DictionaryEntry::new("想", "xiang", 75),
            DictionaryEntry::new("明天", "mingtian", 72),
            DictionaryEntry::new("去", "qu", 68),
            DictionaryEntry::new("北京", "beijing", 95),
            DictionaryEntry::new("中国", "zhongguo", 92),
            DictionaryEntry::new("难", "nan", 50),
            DictionaryEntry::new("发", "fa", 66),
        ])
    }

    #[test]
    fn 简拼展开命中整词() {
        let dictionary = m7_dictionary();
        let candidates = initial_candidates(&dictionary, "nh");
        assert!(
            candidates.iter().any(|c| c.text == "你好"),
            "nh 应展开 nihao 命中你好，实际: {:?}",
            candidates
                .iter()
                .map(|c| c.text.as_str())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn 简拼三字展开() {
        let dictionary = m7_dictionary();
        let candidates = initial_candidates(&dictionary, "wsm");
        assert!(
            candidates.iter().any(|c| c.text == "为什么"),
            "wsm 应展开 weishenme 命中为什么"
        );
    }

    #[test]
    fn 简拼单字符与非法输入不触发() {
        let dictionary = m7_dictionary();
        assert!(
            initial_candidates(&dictionary, "n").is_empty(),
            "单字符不触发"
        );
        assert!(
            initial_candidates(&dictionary, "N").is_empty(),
            "大写不触发"
        );
        assert!(
            initial_candidates(&dictionary, "u1s1").is_empty(),
            "含数字不触发"
        );
        assert!(initial_candidates(&dictionary, "").is_empty());
    }

    #[test]
    fn 简拼未知字母返回空() {
        let dictionary = m7_dictionary();
        assert!(
            initial_candidates(&dictionary, "qq").is_empty(),
            "字母 q 不在简拼表（表中无 q）则整组放弃"
        );
    }

    #[test]
    fn 简拼结果确定性() {
        let dictionary = m7_dictionary();
        let first = initial_candidates(&dictionary, "nh");
        let second = initial_candidates(&dictionary, "nh");
        assert_eq!(first, second);
    }

    #[test]
    fn 模糊替换纠错出中国() {
        let table = SyllableTable::standard();
        let dictionary = m7_dictionary();
        let candidates = corrected_candidates(&table, &dictionary, "zongguo");
        let zhongguo = candidates
            .iter()
            .find(|c| c.text == "中国")
            .expect("zongguo 应纠错出中国");
        assert_eq!(zhongguo.source, CandidateSource::Corrected);
        assert_eq!(zhongguo.pinyin.as_deref(), Some("zhongguo"));
    }

    #[test]
    fn 少字母补全尾音节() {
        let table = SyllableTable::standard();
        let dictionary = m7_dictionary();
        let candidates = corrected_candidates(&table, &dictionary, "niha");
        assert!(
            candidates.iter().any(|c| c.text == "你好"),
            "niha 应补全 ha→hao 出你好，实际: {:?}",
            candidates
                .iter()
                .map(|c| c.text.as_str())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn 整词命中不触发纠错() {
        let table = SyllableTable::standard();
        let dictionary = m7_dictionary();
        assert!(
            corrected_candidates(&table, &dictionary, "nihao").is_empty(),
            "nihao 整词命中则不纠错"
        );
        assert!(corrected_candidates(&table, &dictionary, "").is_empty());
    }

    // ---- 错序容错（T-115 后续）----

    fn errata_dictionary() -> InMemoryDictionary {
        InMemoryDictionary::from_entries(vec![
            DictionaryEntry::new("正确", "zhengque", 8000),
            DictionaryEntry::new("争取", "zhengqu", 6000),
            DictionaryEntry::new("生成", "shengcheng", 7000),
            DictionaryEntry::new("账号", "zhanghao", 5000),
            DictionaryEntry::new("想着", "xiangzhe", 6000),
            DictionaryEntry::new("想", "xiang", 9000),
            DictionaryEntry::new("着", "zhe", 7000),
            DictionaryEntry::new("账", "zhang", 5000),
            DictionaryEntry::new("号", "hao", 7000),
            DictionaryEntry::new("生", "sheng", 8000),
            DictionaryEntry::new("成", "cheng", 7000),
        ])
    }

    #[test]
    fn 错序容错相邻交换变体出正确候选() {
        let table = SyllableTable::standard();
        let dict = errata_dictionary();
        // zhegnq -> 变体 zhengq（仍不可切分）-> 前缀链路补全出"正确"（用户点名）。
        let out = transposed_candidates(&table, &dict, "zhegnq", 8);
        let hit = out.iter().find(|c| c.text == "正确");
        assert!(hit.is_some(), "zhegnq 应纠出 正确，实际: {out:?}");
        // 拼音标注为变体（正确拼音），来源标 Corrected 供 UI 区分。
        let hit = hit.unwrap();
        assert_eq!(hit.pinyin.as_deref(), Some("zhengq"));
        assert_eq!(hit.source, CandidateSource::Corrected);
        // 同一错误两个变体可同时命中（shegnc -> shengc -> 生成，用户点名）。
        let out = transposed_candidates(&table, &dict, "shegnc", 8);
        assert!(
            out.iter().any(|c| c.text == "生成"),
            "shegnc 应纠出 生成，实际: {out:?}"
        );
    }

    #[test]
    fn 错序容错完整切分变体走主链路() {
        let table = SyllableTable::standard();
        let dict = errata_dictionary();
        // xiagnzhe -> 变体 xiangzhe（可完整切分）-> 主链路整词"想着"。
        let out = transposed_candidates(&table, &dict, "xiagnzhe", 8);
        assert!(
            out.iter().any(|c| c.text == "想着"),
            "xiagnzhe 应纠出 想着，实际: {out:?}"
        );
        // zhagnh -> 变体 zhangh -> 前缀链路补全出"账号"（用户点名"账（zhang）号（hao）"）。
        let out = transposed_candidates(&table, &dict, "zhagnh", 8);
        assert!(
            out.iter().any(|c| c.text == "账号"),
            "zhagnh 应纠出 账号，实际: {out:?}"
        );
    }

    #[test]
    fn 错序容错整词命中或可切分输入不触发() {
        let table = SyllableTable::standard();
        let dict = errata_dictionary();
        assert!(
            transposed_candidates(&table, &dict, "zhengqu", 8).is_empty(),
            "整词命中不触发错序容错"
        );
        // nihao 不在 errata 词典，但输入可完整切分（不触发错序容错本身）。
        assert!(transposed_candidates(&table, &dict, "nihao", 8).is_empty());
        assert!(transposed_candidates(&table, &dict, "", 8).is_empty());
    }

    #[test]
    fn 整句beam搜索全局最优() {
        let table = SyllableTable::standard();
        let dictionary = m7_dictionary();
        let bigram = InMemoryBigramModel::new();
        let candidates =
            sentence_candidates(&table, &dictionary, &bigram, "woxiangmingtianqubeijing");
        assert!(
            !candidates.is_empty(),
            "长串应产出整句候选: {:?}",
            candidates
                .iter()
                .map(|c| c.text.as_str())
                .collect::<Vec<_>>()
        );
        assert!(
            candidates.iter().any(|c| c.text == "我想明天去北京"),
            "应包含我想明天去北京: {:?}",
            candidates
                .iter()
                .map(|c| c.text.as_str())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            candidates[0].pinyin.as_deref(),
            Some("woxiangmingtianqubeijing")
        );
    }

    #[test]
    fn 短串与整词命中不启动beam() {
        let table = SyllableTable::standard();
        let dictionary = m7_dictionary();
        let bigram = InMemoryBigramModel::new();
        assert!(
            sentence_candidates(&table, &dictionary, &bigram, "nihao").is_empty(),
            "整词命中不启动 beam"
        );
        assert!(
            sentence_candidates(&table, &dictionary, &bigram, "ni").is_empty(),
            "单音节不启动 beam"
        );
        assert!(sentence_candidates(&table, &dictionary, &bigram, "").is_empty());
    }

    #[test]
    fn 整句beam借助bigram选出自然搭配() {
        let table = SyllableTable::standard();
        let mut dictionary = m7_dictionary();
        // 人为构造：xiang 的高频词是"想"，qu 只跟"去"。
        dictionary.push(DictionaryEntry::new("响", "xiang", 2000));
        dictionary.push(DictionaryEntry::new("趣", "qu", 2000));
        let mut bigram = InMemoryBigramModel::new();
        bigram.insert("想", "去", 1000);
        bigram.insert("明天", "去", 1000);
        bigram.insert("去", "北京", 1000);
        let candidates =
            sentence_candidates(&table, &dictionary, &bigram, "woxiangmingtianqubeijing");
        assert!(
            candidates.iter().any(|c| c.text == "我想明天去北京"),
            "beam 应借助 bigram 选我想明天去北京而非响/趣组合: {:?}",
            candidates
                .iter()
                .map(|c| c.text.as_str())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn 整句beam超高频单字不压过整词() {
        // M7-A 真实词库实测回归：单字「名/天/被/敬」词频远超整词「明天/北京」时，
        // 旧评分（unigram 无上限 + 无 bigram 惩罚）会让 beam 退化为逐字拼接。
        let table = SyllableTable::standard();
        let mut dictionary = m7_dictionary();
        dictionary.push(DictionaryEntry::new("名", "ming", 90_000));
        dictionary.push(DictionaryEntry::new("天", "tian", 90_000));
        dictionary.push(DictionaryEntry::new("被", "bei", 90_000));
        dictionary.push(DictionaryEntry::new("敬", "jing", 90_000));
        let mut bigram = InMemoryBigramModel::new();
        // 自然搭配有强 bigram 证据；各单字组合均无证据（被罚）。
        bigram.insert("想", "明天", 8_000);
        bigram.insert("明天", "去", 9_000);
        bigram.insert("去", "北京", 8_000);
        let candidates =
            sentence_candidates(&table, &dictionary, &bigram, "woxiangmingtianqubeijing");
        assert_eq!(
            candidates.first().map(|c| c.text.as_str()),
            Some("我想明天去北京"),
            "超高频单字不得压过整词: {:?}",
            candidates
                .iter()
                .map(|c| c.text.as_str())
                .collect::<Vec<_>>()
        );
    }
}
