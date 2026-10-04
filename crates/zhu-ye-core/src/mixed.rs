//! 中英混合串解码（FR-050，T-086，方案设计 §14.3）。
//!
//! # 触发（§14.3.1，`is_mixed_input`，纯函数、确定性）
//!
//! 段 = 连续 `[A-Za-z]` 游程 / 其余字符游程。输入同时满足任一组合时触发：
//! - **英文/缩写成分**：字母段含大写字母（`API`/`iPhone`，缩写/专名原形）或整段
//!   不可按拼音切分但可整段匹配英文词（`python`）；
//! - **可拼音成分**：非 ASCII 段（`代码`）或字母段可完整拼音切分（`daima`）；
//! - 同段混合：字母段 = 英文词前缀 + 拼音后缀（`pythondaima`，§14.3.2 贪婪匹配）。
//!
//! 不回退清单（§14.3.5）：纯拼音（`nihao`）、纯缩写（`yyds`，FR-017）、纯英文
//! （`python`，FR-030）、纯中文（`代码`）、`xie` 类双可切串均不触发，走既有路径。
//!
//! # 解码（§14.3.2/14.3.3）
//!
//! 字母段按"整段英文词 → 英文词前缀+拼音后缀 → 最长拼音前缀（可尾随英文段）→
//! 英文词前缀+剩余递归 → 字面保底"的有序策略拆解；**命中即完整英文词原形大小写**
//! （`api`→`API`、`iphone`→`iPhone`、`python`→`python`，14.3.2"未命中逐前缀减一
//! 重查"的递减目标是**词**而非任意前缀），拼音段走既有全拼切分首候选（T-007），
//! 其余段字面保底（不丢输入）。输出 `[整句候选] + 各段最优候选`：**整句 = 各段
//! 最优解码的拼接**（`pythondaima` → `python代码`）置首，分段候选按段出现顺序
//! 随后（`python` / `代码`）。不引入统计模型/语言模型（D-56）。
use crate::candidate::{Candidate, CandidateSource};
use crate::dict::Dictionary;
use crate::en_lexicon::EnLexicon;
use crate::pinyin::{segment_all, SyllableTable};

/// 整句候选的置顶基础分（保证排在一切正常分值候选之前）。
const MIXED_WHOLE_SCORE: i64 = i64::MAX;
/// 分段候选（非英文）的基础分（整句之下、英文负分之上）。
const MIXED_SEGMENT_SCORE: i64 = i64::MAX - 16;

/// 混合串判别（§14.3.1）：返回是否应进入混合解码路径。
#[must_use]
pub fn is_mixed_input(table: &SyllableTable, input: &str) -> bool {
    if input.is_empty() {
        return false;
    }
    let mut has_pinyin_body = false; // 非 ASCII 段（中文等）= 可拼音段
    let mut has_pinyin_letters = false; // 字母段整体可拼音切分
    let mut has_cased = false; // 含大写字母段（缩写/专名成分）
    let mut has_english = false; // 整段不可切但完整匹配英文词
    let mut strong_mixed = false; // 同段"英文词前缀 + 拼音后缀"
    for run in split_runs(input) {
        if run.letters {
            let lower = run.text.to_ascii_lowercase();
            let cased = run.text != lower;
            if cased {
                has_cased = true;
                if is_pinyin_ok(table, &lower) {
                    has_pinyin_letters = true;
                } else if en_is_word(&lower) {
                    has_english = true;
                }
            } else if is_pinyin_ok(table, run.text) {
                has_pinyin_letters = true;
            } else if en_prefix_then_pinyin(table, run.text) {
                strong_mixed = true;
            } else if en_is_word(run.text) {
                has_english = true;
            }
        } else if !run.text.is_ascii() {
            has_pinyin_body = true;
        }
    }
    strong_mixed
        || (has_pinyin_body && (has_cased || has_english))
        || (has_cased && has_pinyin_letters)
}

/// 串（小写化后）能否完整切分为标准拼音。
fn is_pinyin_ok(table: &SyllableTable, lower: &str) -> bool {
    !segment_all(table, lower).is_empty()
}

/// `probe` 是否为**完整英文词**：静态词表前缀查询的 top1 小写化后等于 `probe`
/// 小写化（core 纯函数，`en.zyen` 不可依赖；触发判定略保守：文件词表更大也不
/// 影响"不小看既有路径"的正确性）。
fn en_is_word(probe: &str) -> bool {
    crate::en_words::en_words_with_prefix(probe, 1)
        .first()
        .is_some_and(|(word, _)| word.eq_ignore_ascii_case(probe))
}

/// 同段"英文词前缀 + 拼音后缀"是否存在（§14.3.2 贪婪判定）。
fn en_prefix_then_pinyin(table: &SyllableTable, letter_run: &str) -> bool {
    let n = letter_run.len();
    for k in (2..=n).rev() {
        let (prefix, rest) = letter_run.split_at(k);
        if !rest.is_empty() && en_is_word(prefix) && is_pinyin_ok(table, rest) {
            return true;
        }
    }
    false
}

/// 单个段：`letters = true` 表示连续 `[A-Za-z]` 游程。
struct SegmentRun<'a> {
    letters: bool,
    text: &'a str,
}

/// 按字符游程切分输入，返回有序段序列（字母段与非字母段交替）。
fn split_runs(input: &str) -> Vec<SegmentRun<'_>> {
    let mut runs: Vec<SegmentRun<'_>> = Vec::new();
    if input.is_empty() {
        return runs;
    }
    let mut start = 0usize;
    let mut letters = input
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic());
    for (idx, c) in input.char_indices() {
        let is_letters = c.is_ascii_alphabetic();
        if is_letters != letters {
            runs.push(SegmentRun {
                letters,
                text: &input[start..idx],
            });
            start = idx;
            letters = is_letters;
        }
    }
    runs.push(SegmentRun {
        letters,
        text: &input[start..],
    });
    runs
}

/// 混合串解码候选（§14.3.3）：`[整句] + 各段最优候选`，顺序确定。
///
/// - 调用方保证 `is_mixed_input` 为真（否则整句 = 输入原样、分段 = 全部字面）；
/// - `dictionary` 用于可拼音切分字母段的全拼切分（无则不产出该段中文候选）；
/// - `en_lexicon` 有值走 `en.zyen` 词表，否则回退静态表（同 FR-030 双源口径）；
/// - 英文段候选每段取 top1（"各段最优候选组合"，§14.3.3 上限约束）。
#[must_use]
pub fn mixed_candidates(
    table: &SyllableTable,
    dictionary: Option<&dyn Dictionary>,
    en_lexicon: Option<&EnLexicon>,
    input: &str,
) -> Vec<Candidate> {
    if input.is_empty() {
        return Vec::new();
    }
    let mut whole = String::with_capacity(input.len().saturating_mul(2));
    let mut segments: Vec<Candidate> = Vec::new();
    for run in split_runs(input) {
        if run.letters {
            let (part, part_candidates) =
                decode_letter_run(table, dictionary, en_lexicon, run.text);
            whole.push_str(&part);
            segments.extend(part_candidates);
        } else {
            whole.push_str(run.text);
            segments.push(literal(run.text, segments.len()));
        }
    }
    let mut out = Vec::with_capacity(1 + segments.len());
    out.push(Candidate::new(whole, MIXED_WHOLE_SCORE).with_source(CandidateSource::Mixed));
    out.extend(segments);
    out
}

/// 解码单个字母段：返回（整句拼接文本, 分段候选）。
///
/// 有序策略：① 整段英文词 → ② 英文词前缀+拼音后缀（最长词优先）→ ③ 最长拼音
/// 前缀（可尾随英文段）→ ④ 英文词前缀+剩余递归（`pythonxyz` → python + xyz 字面）
/// → ⑤ 字面保底（不丢输入）。均只取各段最优 top1。
fn decode_letter_run(
    table: &SyllableTable,
    dictionary: Option<&dyn Dictionary>,
    en_lexicon: Option<&EnLexicon>,
    run: &str,
) -> (String, Vec<Candidate>) {
    let n = run.len();
    // ① 整段英文词。
    if let Some(top) = best_english_word(en_lexicon, run) {
        if top.text.eq_ignore_ascii_case(run) {
            return (top.text.clone(), vec![top]);
        }
    }
    // ② 英文词前缀 + 拼音后缀（最长词优先，§14.3.2 贪婪）。
    for k in (2..=n).rev() {
        let (prefix, rest) = run.split_at(k);
        if rest.is_empty() || !en_is_word(prefix) || !is_pinyin_ok(table, rest) {
            continue;
        }
        if let Some(top) = best_english_word(en_lexicon, prefix) {
            let tail = decode_pinyin_tail(table, dictionary, rest);
            let mut text = String::with_capacity(top.text.len() + tail.text.len());
            text.push_str(&top.text);
            text.push_str(&tail.text);
            let mut candidates = vec![top];
            candidates.extend(tail.candidates);
            return (text, candidates);
        }
    }
    // ③ 最长拼音前缀（可尾随英文段）。
    for k in (2..=n).rev() {
        let (prefix, rest) = run.split_at(k);
        if !is_pinyin_ok(table, prefix) {
            continue;
        }
        if rest.is_empty() {
            if let Some(top) = top_pinyin(table, dictionary, prefix) {
                return (top.text.clone(), vec![top]);
            }
            continue; // 整段拼音但词典无词：试更短前缀
        }
        if let (Some(zh), Some(en)) = (
            top_pinyin(table, dictionary, prefix),
            best_english_word(en_lexicon, rest),
        ) {
            let mut text = zh.text.clone();
            text.push_str(&en.text);
            return (text, vec![zh, en]);
        }
    }
    // ④ 英文词前缀 + 剩余递归（保证剩余不丢）。
    if let Some(top) = best_english_word(en_lexicon, run) {
        let consumed = top.text.len();
        if consumed < n {
            let (tail_text, tail_candidates) =
                decode_letter_run(table, dictionary, en_lexicon, &run[consumed..]);
            let mut text = top.text.clone();
            text.push_str(&tail_text);
            let mut candidates = vec![top];
            candidates.extend(tail_candidates);
            return (text, candidates);
        }
    }
    // ⑤ 字面保底：不丢输入。
    (run.to_owned(), vec![literal(run, 0)])
}

/// 拼音后缀解码：既有全拼切分首候选；无词典命中则字面保底。
fn decode_pinyin_tail(
    table: &SyllableTable,
    dictionary: Option<&dyn Dictionary>,
    text: &str,
) -> DecodedTail {
    match top_pinyin(table, dictionary, text) {
        Some(top) => DecodedTail {
            text: top.text.clone(),
            candidates: vec![top],
        },
        None => DecodedTail {
            text: text.to_owned(),
            candidates: vec![literal(text, 0)],
        },
    }
}

struct DecodedTail {
    text: String,
    candidates: Vec<Candidate>,
}

/// 拼音串的首选候选（T-007 全拼切分首个；无字典词典词则不产出）。
fn top_pinyin(
    table: &SyllableTable,
    dictionary: Option<&dyn Dictionary>,
    pinyin: &str,
) -> Option<Candidate> {
    dictionary.and_then(|dict| {
        crate::candidate::generate_candidates(table, dict, pinyin)
            .into_iter()
            .next()
    })
}

/// 英文段最优候选：整段前缀查询；未命中**完整英文词**时逐前缀减一重查
/// （§14.3.2，递减目标是词，`pyx` 不会回退到非词前缀 `py`）。
fn best_english_word(en_lexicon: Option<&EnLexicon>, seg: &str) -> Option<Candidate> {
    let mut probe = seg;
    while probe.chars().count() >= 2 {
        let found = match en_lexicon {
            Some(lexicon) => crate::candidate::en_word_candidates_from(lexicon, probe, 1),
            None => crate::candidate::en_word_candidates(probe, 1),
        };
        if let Some(top) = found.into_iter().next() {
            if top.text.eq_ignore_ascii_case(probe) {
                return Some(top);
            }
        }
        probe = &probe[..probe.len() - 1];
    }
    None
}

/// 段字面候选（源 Mixed，排在整句之下）。
fn literal(text: &str, seg_idx: usize) -> Candidate {
    Candidate::new(text.to_owned(), MIXED_SEGMENT_SCORE - seg_idx as i64)
        .with_source(CandidateSource::Mixed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dict::{DictionaryEntry, InMemoryDictionary};
    use crate::pinyin::SyllableTable;

    fn table() -> SyllableTable {
        SyllableTable::standard()
    }

    fn dict_with_daima() -> InMemoryDictionary {
        InMemoryDictionary::from_entries(vec![DictionaryEntry {
            word: "代码".to_owned(),
            pinyin: "daima".to_owned(),
            translation: None,
            frequency: 100,
        }])
    }

    #[test]
    fn is_mixed_triggers_on_mixed_input() {
        let t = table();
        // 验收用例：python代码 / API接口 / iPhone / Web 类
        assert!(is_mixed_input(&t, "python代码"));
        assert!(is_mixed_input(&t, "API接口"));
        assert!(is_mixed_input(&t, "iPhone价格"));
        assert!(is_mixed_input(&t, "Web开发"));
        // 真实组合串（全 ASCII）：同段英文词+拼音后缀
        assert!(is_mixed_input(&t, "pythondaima"));
        assert!(is_mixed_input(&t, "nihaoAPI"));
    }

    #[test]
    fn is_mixed_does_not_hijack_existing_paths() {
        let t = table();
        // §14.3.5 不回退清单：纯拼音 / 纯缩写 / 纯英文 / 纯中文
        assert!(!is_mixed_input(&t, "nihao"));
        assert!(!is_mixed_input(&t, "yyds"));
        assert!(!is_mixed_input(&t, "python"));
        assert!(!is_mixed_input(&t, "代码"));
        // 双可切串（§14.8 风险表）：拼音可切且无英文成分 → 不走混合
        assert!(!is_mixed_input(&t, "xie"));
        assert!(!is_mixed_input(&t, "xiedaima"));
        // 可切拼音段 + 中文（无英文成分）→ 不走混合
        assert!(!is_mixed_input(&t, "nihao代码"));
        // 小写可切缩写 + 中文 → 不走混合（验收仅大写 `API接口`）
        assert!(!is_mixed_input(&t, "api接口"));
        // 数字段不算可拼音段，英文在前无拼音段也不触发
        assert!(!is_mixed_input(&t, "python123"));
        // 拼音在前的同段混合（`daimapython`）暂不触发：英文词需在拼音段之前
        assert!(!is_mixed_input(&t, "daimapython"));
        assert!(!is_mixed_input(&t, ""));
    }

    #[test]
    fn mixed_ascii_core_case_pythondaima() {
        let t = table();
        let dict = dict_with_daima();
        let cands = mixed_candidates(&t, Some(&dict), None, "pythondaima");
        let texts: Vec<&str> = cands.iter().map(|c| c.text.as_str()).collect();
        // 整句 = python（词表原形小写）+ 代码（拼音段词典命中）
        assert_eq!(texts[0], "python代码");
        assert_eq!(texts.len(), 3);
        assert_eq!(texts[1], "python");
        assert_eq!(texts[2], "代码");
    }

    #[test]
    fn mixed_no_dictionary_falls_back_to_literal() {
        let t = table();
        let cands = mixed_candidates(&t, None, None, "pythondaima");
        let texts: Vec<&str> = cands.iter().map(|c| c.text.as_str()).collect();
        // 无词典：拼音段字面保底，输入内容不丢
        assert_eq!(texts[0], "pythondaima");
        assert_eq!(texts[1], "python");
        assert_eq!(texts[2], "daima");
    }

    #[test]
    fn mixed_segments_with_static_en() {
        let t = table();
        let cands = mixed_candidates(&t, None, None, "iphone接口");
        let texts: Vec<&str> = cands.iter().map(|c| c.text.as_str()).collect();
        // iphone → iPhone（静态表 en-capitals 原形）+ 接口（中文段字面）
        assert_eq!(texts[0], "iPhone接口");
        assert_eq!(texts[1], "iPhone");
        assert_eq!(texts[2], "接口");
    }

    #[test]
    fn mixed_uppercase_abbreviation() {
        let t = table();
        let cands = mixed_candidates(&t, None, None, "API接口");
        let texts: Vec<&str> = cands.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(texts[0], "API接口");
        assert!(texts.contains(&"API"));
        assert!(texts.contains(&"接口"));
    }

    #[test]
    fn mixed_pinyin_then_abbreviation() {
        let t = table();
        let dict = InMemoryDictionary::from_entries(vec![DictionaryEntry {
            word: "你好".to_owned(),
            pinyin: "nihao".to_owned(),
            translation: None,
            frequency: 100,
        }]);
        let cands = mixed_candidates(&t, Some(&dict), None, "nihaoAPI");
        let texts: Vec<&str> = cands.iter().map(|c| c.text.as_str()).collect();
        // ③ 最长拼音前缀 nihao → 你好，尾随英文段 API → API
        assert_eq!(texts[0], "你好API");
        assert!(texts.contains(&"你好"));
        assert!(texts.contains(&"API"));
    }

    #[test]
    fn mixed_word_prefix_then_unresolvable_tail_kept() {
        let t = table();
        // ④ 英文词前缀 python + 剩余 xyz 不可解 → 字面递归，输入不丢
        let cands = mixed_candidates(&t, None, None, "pythonxyz");
        let texts: Vec<&str> = cands.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(texts[0], "pythonxyz");
        assert!(texts.contains(&"python"));
        assert!(texts.contains(&"xyz"));
    }

    #[test]
    fn mixed_deterministic() {
        let t = table();
        let a = mixed_candidates(&t, None, None, "python代码");
        let b = mixed_candidates(&t, None, None, "python代码");
        assert_eq!(a, b);
        let first = a.first().unwrap();
        assert_eq!(first.text, "python代码");
        assert_eq!(first.source, CandidateSource::Mixed);
    }

    #[test]
    fn mixed_unresolvable_segment_literal_no_loss() {
        let t = table();
        // pyx 非完整英文词（不会回退到非词前缀 py）→ 字面保底
        let cands = mixed_candidates(&t, None, None, "pyx1接口");
        let texts: Vec<&str> = cands.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(texts[0], "pyx1接口");
        assert!(texts.contains(&"pyx"));
        assert!(texts.contains(&"1接口"));
    }

    #[test]
    fn mixed_empty_input() {
        let t = table();
        assert!(mixed_candidates(&t, None, None, "",).is_empty());
        assert!(!is_mixed_input(&t, ""));
    }
}
