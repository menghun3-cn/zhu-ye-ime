//! 领域提权（FR-033/FR-034，场景 8，第六期）。
//!
//! 领域识别限定在**已启用**领域包内（P-12：未启用包不加载、不参与识别），对组合串的
//! **完整词**（整串拼音，`lookup` 精确命中）做命中检测（D-15：前缀命中不参与提权）；
//! 多包同时命中时取**包 id 字典序首个**（D-16，同一输入永远同结果）。
//!
//! 命中的领域候选标注 `CandidateSource::Domain`，由调用方（引擎）按 D-13 位次
//! （插基础候选之后、追加组之前）重排；未命中任何领域包时调用方保持既有路径
//! （T-050「领域包只追加、不改基础排序」基线逐位不变）。

use crate::candidate::{Candidate, CandidateSource};
use crate::dict::Dictionary;

/// 领域提权候选上限：单个领域包一次最多提权的候选条数（防极端词条膨胀）。
pub const DOMAIN_BOOST_CAP: usize = 8;

/// 对已按 **id 字典序** 排序的启用领域包做完整词命中检测（D-16 依赖入参有序）。
///
/// 返回首个命中领域包的候选列表（候选 `source = Domain`、保留词条 pinyin/translation）；
/// 无任何包整词命中（或 composing 为空）时返回 `None` —— 调用方保持既有候选路径。
///
/// 调用方保证：
/// - `packs` 已按包 id 升序排列（字典序，D-16；本函数按序遍历取首个命中）；
/// - 只传**已启用**的领域包（未启用包不参与识别，P-12 不突破）。
#[must_use]
pub fn domain_boost_candidates(
    packs: &[(String, &dyn Dictionary)],
    composing: &str,
) -> Option<Vec<Candidate>> {
    if composing.is_empty() {
        return None;
    }
    for (_id, pack) in packs {
        let entries = pack.lookup(composing);
        if entries.is_empty() {
            continue;
        }
        let boosted: Vec<Candidate> = entries
            .into_iter()
            .take(DOMAIN_BOOST_CAP)
            .map(|entry| {
                let mut candidate = Candidate::new(
                    entry.word.clone(),
                    i64::try_from(entry.frequency).unwrap_or(i64::MAX),
                )
                .with_source(CandidateSource::Domain);
                candidate = candidate.with_pinyin(entry.pinyin.clone());
                if let Some(translation) = &entry.translation {
                    candidate = candidate.with_translation(translation.clone());
                }
                candidate
            })
            .collect();
        // 整词命中但词条全被截断为空的情况不会发生（entries 非空即至少取 1 条）。
        return Some(boosted);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::domain_boost_candidates;
    use crate::candidate::{CandidateSorter, CandidateSource};
    use crate::dict::{DictionaryEntry, InMemoryDictionary};

    fn pack(id: &str, entries: Vec<DictionaryEntry>) -> (String, InMemoryDictionary) {
        (id.to_owned(), InMemoryDictionary::from_entries(entries))
    }

    fn text_of(candidates: &[crate::candidate::Candidate]) -> Vec<String> {
        candidates.iter().map(|c| c.text.clone()).collect()
    }

    #[test]
    fn 单包整词命中产出领域候选() {
        let it = pack(
            "it",
            vec![
                DictionaryEntry::new("局部变量", "jububianliang", 100),
                DictionaryEntry::new("局域网", "juyuwang", 80),
            ],
        );
        let packs: Vec<(String, &dyn crate::dict::Dictionary)> = vec![(it.0, &it.1)];
        let boosted = domain_boost_candidates(&packs, "jububianliang").expect("应命中");
        assert_eq!(text_of(&boosted), vec!["局部变量"]);
        assert_eq!(boosted[0].source, CandidateSource::Domain);
        assert_eq!(boosted[0].pinyin.as_deref(), Some("jububianliang"));
    }

    #[test]
    fn 多包同命中按包id字典序取首个() {
        // id 顺序：b_med < it；两者都命中时取 b_med。
        let med = pack(
            "b_med",
            vec![DictionaryEntry::new("神经内科", "shenjingneike", 90)],
        );
        let it = pack(
            "it",
            vec![DictionaryEntry::new("神经网络", "shenjingwangluo", 95)],
        );
        let packs: Vec<(String, &dyn crate::dict::Dictionary)> =
            vec![(med.0, &med.1), (it.0, &it.1)];
        let boosted = domain_boost_candidates(&packs, "shenjingwangluo").expect("应命中");
        // 只有 it 整词命中（med 的 pinyin 不同）；若两个都命中则取 b_med。
        assert_eq!(text_of(&boosted), vec!["神经网络"]);
        assert_eq!(boosted[0].source, CandidateSource::Domain);
    }

    #[test]
    fn 多包同时整词命中取字典序首个() {
        let a = pack("a_first", vec![DictionaryEntry::new("甲词", "jubu", 10)]);
        let z = pack("z_last", vec![DictionaryEntry::new("乙词", "jubu", 20)]);
        let packs: Vec<(String, &dyn crate::dict::Dictionary)> = vec![(a.0, &a.1), (z.0, &z.1)];
        let boosted = domain_boost_candidates(&packs, "jubu").expect("应命中");
        assert_eq!(
            text_of(&boosted),
            vec!["甲词"],
            "应取 id 字典序首个包（a_first）"
        );
    }

    #[test]
    fn 无整词命中或空串返回空() {
        let it = pack(
            "it",
            vec![DictionaryEntry::new("局部变量", "jububianliang", 100)],
        );
        let packs: Vec<(String, &dyn crate::dict::Dictionary)> = vec![(it.0, &it.1)];
        // 前缀串不整词命中（D-15：前缀不参与提权）。
        assert!(domain_boost_candidates(&packs, "jubu").is_none());
        // 空串不识别。
        assert!(domain_boost_candidates(&packs, "").is_none());
        // 完全无关串不命中。
        assert!(domain_boost_candidates(&packs, "nihao").is_none());
        // 无领域包。
        assert!(domain_boost_candidates(&[], "jububianliang").is_none());
    }

    #[test]
    fn 领域候选按词频排序确定() {
        let it = pack(
            "it",
            vec![
                DictionaryEntry::new("低词", "jubu", 5),
                DictionaryEntry::new("高词", "jubu", 900),
            ],
        );
        let packs: Vec<(String, &dyn crate::dict::Dictionary)> = vec![(it.0, &it.1)];
        let boosted = domain_boost_candidates(&packs, "jubu").unwrap();
        // 组内排序确定：词频降序（与主候选一致）；两次结果逐位一致。
        let sorted = CandidateSorter::sort(boosted.clone());
        assert_eq!(text_of(&sorted), vec!["高词", "低词"]);
        let again = domain_boost_candidates(&packs, "jubu").unwrap();
        assert_eq!(boosted, again, "领域识别与候选必须确定性");
    }

    #[test]
    fn 领域候选保留译文() {
        let it = pack(
            "it",
            vec![DictionaryEntry::new("队列", "duilie", 60).with_translation("queue")],
        );
        let packs: Vec<(String, &dyn crate::dict::Dictionary)> = vec![(it.0, &it.1)];
        let boosted = domain_boost_candidates(&packs, "duilie").unwrap();
        assert_eq!(boosted[0].translation.as_deref(), Some("queue"));
    }
}
