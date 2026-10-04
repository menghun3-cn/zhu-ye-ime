//! T-089（FR-049）proptest 属性测试：对 core 的切分、候选排序、bigram 与整句
//! beam 四个核心模块做属性化测试（方案设计 §14.6）。
//!
//! 覆盖口径：
//! - 切分：合法音节拼接必可切分、任意方案拼接还原原串、任意 ASCII 小写串不
//!   panic 且结果确定、非 ASCII 恒为空；
//! - 排序：同输入两次排序一致（确定性）、比较结果与 (score 降序, text 升序)
//!   全序一致（比较器全序）；
//! - bigram：随机模型插入后查询与语料一致、后继列表排序（频率降序 + 同频
//!   字典序）与截断、随机/空输入不 panic；
//! - beam：随机（字典, 输入）下候选不重复、不超过 SENTENCE_TOP_N、pinyin 与
//!   输入一致（启动时），任意输入不 panic。
//!
//! 全部随机序列由 proptest 固定种子重放（确定性重现、失败最小反例收缩）；
//! 本文件仅测试期编译（dev-dependency）。

use proptest::prelude::*;
use zhu_ye_core::pinyin::STANDARD_SYLLABLES;
use zhu_ye_core::{
    segment_all, sentence_candidates, BigramModel, Candidate, CandidateSorter, DictionaryEntry,
    EmptyBigramModel, InMemoryBigramModel, InMemoryDictionary, SyllableTable, SENTENCE_TOP_N,
};

/// 标准音节表（与实现单测同源）。
fn standard_table() -> SyllableTable {
    SyllableTable::standard()
}

/// 随机抽取 1..=`max` 个标准音节拼接成合法拼音输入（确定性重放）。
fn valid_pinyin_input(max_syllables: usize) -> impl Strategy<Value = String> {
    prop::collection::vec(prop::sample::select(STANDARD_SYLLABLES), 1..=max_syllables)
        .prop_map(|parts| parts.concat())
}

/// 随机候选条目（text 可为空串/重复，score 全覆盖）。
fn any_candidate() -> impl Strategy<Value = Candidate> {
    ("[a-z0-9-]{0,8}", any::<i64>()).prop_map(|(text, score)| Candidate::new(text, score))
}

/// 随机小词典：随机音节 → 1..3 个汉字的词条，词频任意。
fn any_dictionary(max_entries: usize) -> impl Strategy<Value = InMemoryDictionary> {
    let syllable = prop::sample::select(STANDARD_SYLLABLES);
    let word = prop::collection::vec(prop::char::range('一', '龥'), 1..=3)
        .prop_map(|chars| chars.into_iter().collect::<String>());
    prop::collection::vec((syllable, word, 0..10_000u64), 0..max_entries).prop_map(|rows| {
        InMemoryDictionary::from_entries(
            rows.into_iter()
                .map(|(pinyin, text, frequency)| DictionaryEntry::new(text, pinyin, frequency))
                .collect(),
        )
    })
}

proptest! {
    // ---- 1. 切分（pinyin::segment_all）----

    #[test]
    fn 合法音节拼接必可切分且方案拼接还原原串(
        parts in prop::collection::vec(prop::sample::select(STANDARD_SYLLABLES), 1..=8),
    ) {
        let input = parts.concat();
        let table = standard_table();
        let plans = segment_all(&table, &input);
        prop_assert!(!plans.is_empty(), "合法音节拼接必可切分: {:?}", input);
        for plan in &plans {
            let joined = plan.concat();
            prop_assert_eq!(joined.as_str(), input.as_str(), "方案拼接必须还原输入");
            for syllable in plan {
                prop_assert!(
                    table.is_complete_syllable(syllable),
                    "方案元素必须是完整音节: {:?}",
                    syllable
                );
            }
        }
        // 确定性：同输入两次切分逐位一致。
        prop_assert_eq!(plans, segment_all(&table, &input));
    }

    #[test]
    fn 任意小写串切分不panic且结果拼接守恒(input in "[a-z]{0,24}") {
        let table = standard_table();
        let plans = segment_all(&table, &input);
        for plan in &plans {
            let joined = plan.concat();
            prop_assert_eq!(joined.as_str(), input.as_str(), "任意可切分输入方案拼接守恒");
        }
        prop_assert_eq!(plans, segment_all(&table, &input), "切分结果确定性");
    }

    #[test]
    fn 非ascii输入恒为空(input in ".*") {
        if input.is_ascii() {
            return Ok(()); // 本属性只测含非 ASCII 的样本
        }
        prop_assert!(segment_all(&standard_table(), &input).is_empty());
    }

    // ---- 2. 排序（candidate::CandidateSorter）----

    #[test]
    fn 排序确定性且同输入两次结果一致(input in prop::collection::vec(any_candidate(), 0..64)) {
        let once = CandidateSorter::sort(input.clone());
        let twice = CandidateSorter::sort(once.clone());
        prop_assert_eq!(&once, &twice, "二次排序幂等（比较器为全序）");
        // 相邻项满足 (score 降序, text 升序) 的字典序全序。
        for pair in once.windows(2) {
            let a = &pair[0];
            let b = &pair[1];
            prop_assert!(
                a.score > b.score || (a.score == b.score && a.text <= b.text),
                "相邻项违反全序: {:?} vs {:?}",
                a,
                b
            );
        }
    }

    #[test]
    fn 排序结果与元组键全序排序一致(input in prop::collection::vec(any_candidate(), 0..64)) {
        let mut expected = input.clone();
        expected.sort_by_key(|c| (std::cmp::Reverse(c.score), c.text.clone()));
        prop_assert_eq!(CandidateSorter::sort(input), expected);
    }

    // ---- 3. bigram（bigram::InMemoryBigramModel）----

    #[test]
    fn 随机模型插入后查询与语料一致(
        rows in prop::collection::vec(
            ("[a-z]{0,6}", "[a-z]{0,6}", 0..1_000_000u64),
            0..32,
        ),
    ) {
        let mut model = InMemoryBigramModel::new();
        for (previous, word, frequency) in &rows {
            model.insert(previous.clone(), word.clone(), *frequency);
        }
        // insert 为覆盖语义：同键多次插入以最后一次为准（既有行为）。
        let mut last_per_key: std::collections::HashMap<(String, String), u64> =
            std::collections::HashMap::new();
        for (previous, word, frequency) in &rows {
            last_per_key.insert((previous.clone(), word.clone()), *frequency);
        }
        for ((previous, word), frequency) in last_per_key {
            prop_assert_eq!(
                model.frequency(&previous, &word),
                frequency,
                "插入后查询必须与语料一致: {:?}",
                (previous, word)
            );
        }
        // 未收录前词查询为 0（不 panic）。
        prop_assert_eq!(model.frequency("zzzz", "zzzz"), 0);
        prop_assert_eq!(EmptyBigramModel.frequency("zzzz", "zzzz"), 0);
    }

    #[test]
    fn 后继列表按频率降序同频字典序并截断(
        rows in prop::collection::vec(
            ("[a-z]{1,5}", "[a-z]{1,8}", 0..1_000_000u64),
            0..32,
        ),
        limit in 0usize..16,
        previous in "[a-z]{0,5}",
    ) {
        let mut model = InMemoryBigramModel::new();
        for (p, w, f) in &rows {
            model.insert(p.clone(), w.clone(), *f);
        }
        let successors = model.successors(&previous, limit);
        prop_assert!(successors.len() <= limit, "后继条数不得超过上限");
        if limit == 0 {
            prop_assert!(successors.is_empty(), "limit=0 必为空");
        }
        for pair in successors.windows(2) {
            match pair[0].1.cmp(&pair[1].1) {
                std::cmp::Ordering::Greater => {}
                std::cmp::Ordering::Equal => {
                    prop_assert!(
                        pair[0].0 <= pair[1].0,
                        "同频必须按词形字典序: {:?} vs {:?}",
                        pair[0],
                        pair[1]
                    );
                }
                std::cmp::Ordering::Less => {
                    prop_assert!(false, "后继必须按频率降序: {:?} vs {:?}", pair[0], pair[1]);
                }
            }
        }
        // 确定性：同模型两次查询一致。
        prop_assert_eq!(successors, model.successors(&previous, limit));
    }

    // ---- 4. 整句 beam（candidate::sentence_candidates）----

    #[test]
    fn beam随机输入永不panic且候选不重复不越界(
        input in valid_pinyin_input(8),
        dictionary in any_dictionary(24),
    ) {
        let candidates =
            sentence_candidates(&standard_table(), &dictionary, &EmptyBigramModel, &input);
        prop_assert!(candidates.len() <= SENTENCE_TOP_N, "候选数量不越界");
        let mut texts = std::collections::HashSet::new();
        for candidate in &candidates {
            prop_assert!(!candidate.text.is_empty(), "候选文本非空");
            prop_assert!(
                texts.insert(&candidate.text),
                "候选文本不重复: {:?}",
                candidate.text
            );
            if !input.is_empty() && candidate.pinyin.is_some() {
                prop_assert_eq!(
                    candidate.pinyin.as_deref(),
                    Some(input.as_str()),
                    "整句候选 pinyin 与输入一致"
                );
            }
        }
        // 确定性：同输入同字典两次调用一致。
        prop_assert_eq!(
            candidates,
            sentence_candidates(&standard_table(), &dictionary, &EmptyBigramModel, &input)
        );
    }
}
