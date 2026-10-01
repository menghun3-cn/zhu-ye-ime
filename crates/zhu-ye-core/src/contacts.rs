//! 联系人索引（场景 9 通讯录，FR-036/FR-037）。
//!
//! 纯算法、确定、可测：由 `vcard::parse_vcard` 产出的姓名建立可检索索引。
//!
//! - 建键：中文姓名**全部读音形态**逐字组合（D-22，多音字任一读音可命中）；
//!   非中文（英文名/数字）按原文小写作键（D-20 仅姓名）；组合键数量受
//!   [`CONTACT_KEYS_CAP`] 上限保护（确定性截断）。
//! - 结构：键按字节序升序的静态表（与 `en_words` 同款 `partition_point` 前缀扫描）。
//! - 查询：`contact_candidates` 返回 `CandidateSource::Contact` 候选（按键序去重），
//!   不参与分数排序——提权位次由 ime 装配层的协调层决定（D-21 与 D-13 同层）。
//!
//! 联系人不写入用户词学习流水线（FR-003 行为不回退）。

use crate::candidate::{Candidate, CandidateSource};
use crate::char_pinyin::char_pinyin;
use crate::vcard::VCardContact;

/// 单个联系人的检索键上限（组合爆炸保护；超出取前 64，确定性）。
pub const CONTACT_KEYS_CAP: usize = 64;

/// 索引容量上限（联系人条数，NFR-006 常驻内存预算内）；超出按出现顺序截断。
pub const CONTACT_INDEX_CAP: usize = 10_000;

/// 一个检索键及其命中的联系人姓名（去重保序）。
#[derive(Debug, Clone, PartialEq, Eq)]
struct KeyEntry {
    key: String,
    names: Vec<String>,
}

/// 联系人索引：键升序静态表，前缀扫描查询。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContactIndex {
    keys: Vec<KeyEntry>,
}

impl ContactIndex {
    /// 索引中的联系人姓名总数（诊断/测试用）。
    #[must_use]
    pub fn len(&self) -> usize {
        self.keys.iter().map(|entry| entry.names.len()).sum()
    }

    /// 索引是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
}

/// 为姓名生成检索键集合：逐字符读音笛卡尔积（多音全形态） + 非中文原文归一。
///
/// 规则（确定性）：
/// - 汉字 → `char_pinyin` 全部读音（无调 ASCII，ü→v）；
/// - ASCII 字母/数字 → 小写原形（单个候选）；
/// - 空格与其它符号 → 不产生候选（跳过，如 `John Ma` → `johnma`）；
/// - 产物为所有字符候选的笛卡尔积拼接，按输入序截断到 [`CONTACT_KEYS_CAP`]。
#[must_use]
pub fn annotate_name(name: &str) -> Vec<String> {
    let mut slots: Vec<Vec<String>> = Vec::with_capacity(name.chars().count());
    for ch in name.chars() {
        if let Some(readings) = char_pinyin(ch) {
            if !readings.is_empty() {
                slots.push(readings.iter().map(|r| (*r).to_owned()).collect());
                continue;
            }
        }
        if ch.is_ascii_alphanumeric() {
            slots.push(vec![ch.to_ascii_lowercase().to_string()]);
        }
        // 空格/符号：跳过。
    }
    if slots.is_empty() {
        return Vec::new();
    }
    let mut keys = vec![String::new()];
    for slot in slots {
        let mut next = Vec::with_capacity(keys.len().saturating_mul(slot.len()));
        for prefix in &keys {
            for reading in &slot {
                let mut joined = prefix.clone();
                joined.push_str(reading);
                next.push(joined);
            }
        }
        keys = next;
        if keys.len() >= CONTACT_KEYS_CAP {
            keys.truncate(CONTACT_KEYS_CAP);
            break;
        }
    }
    keys.sort();
    keys.dedup();
    keys
}

/// 由 vCard 姓名建立联系人索引（T-071-2；keys 在此填充，超过
/// [`CONTACT_INDEX_CAP`] 条按出现顺序截断）。
#[must_use]
pub fn build_contact_index(contacts: &[VCardContact]) -> ContactIndex {
    let mut by_key: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for contact in contacts.iter().take(CONTACT_INDEX_CAP) {
        if contact.name.is_empty() {
            continue;
        }
        for key in annotate_name(&contact.name) {
            let names = by_key.entry(key).or_default();
            if !names.iter().any(|existing| existing == &contact.name) {
                names.push(contact.name.clone());
            }
        }
    }
    ContactIndex {
        keys: by_key
            .into_iter()
            .map(|(key, names)| KeyEntry { key, names })
            .collect(),
    }
}

/// 前缀查询：返回命中联系人候选（`CandidateSource::Contact`），按键序去重。
///
/// 组内顺序 = 键序（拼音序）下首次出现顺序，确定可复现；与主候选/领域候选的
/// 位次关系由 ime 协调层决定（D-21），本函数不参与任何分数排序。
#[must_use]
pub fn contact_candidates(index: &ContactIndex, pinyin_prefix: &str, cap: usize) -> Vec<Candidate> {
    if pinyin_prefix.is_empty() || index.keys.is_empty() || cap == 0 {
        return Vec::new();
    }
    // 键按字节序升序：前缀区间 [lo, hi)（hi 用词典序上界 "prefix+\u{10FFFF}"）。
    let lo = index
        .keys
        .partition_point(|entry| entry.key.as_str() < pinyin_prefix);
    let mut upper = String::with_capacity(pinyin_prefix.len() + 4);
    upper.push_str(pinyin_prefix);
    upper.push('\u{10FFFF}');
    let hi = index
        .keys
        .partition_point(|entry| entry.key.as_str() <= upper.as_str());
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for entry in &index.keys[lo..hi] {
        for name in &entry.names {
            if seen.insert(name.clone()) {
                out.push(Candidate::new(name.clone(), 0).with_source(CandidateSource::Contact));
                if out.len() >= cap {
                    return out;
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{annotate_name, build_contact_index, contact_candidates, CONTACT_KEYS_CAP};
    use crate::candidate::CandidateSource;
    use crate::vcard::VCardContact;

    fn contact(name: impl Into<String>) -> VCardContact {
        VCardContact {
            name: name.into(),
            keys: Vec::new(),
        }
    }

    #[test]
    fn 全拼键_张三() {
        assert_eq!(annotate_name("张三"), vec!["zhangsan"]);
    }

    #[test]
    fn 多音字全形态键() {
        // 曾 = zeng/ceng（D-22：全部读音形态建键）。
        let keys = annotate_name("曾");
        assert!(keys.contains(&"zeng".to_owned()) && keys.contains(&"ceng".to_owned()));
        // 组合：曾子 → zengzi / cengzi。
        let keys = annotate_name("曾子");
        assert!(keys.contains(&"zengzi".to_owned()) && keys.contains(&"cengzi".to_owned()));
    }

    #[test]
    fn 英文名原文小写键() {
        assert_eq!(annotate_name("Alice"), vec!["alice"]);
        // 空格不产生候选（John Ma → johnma）。
        assert_eq!(annotate_name("John Ma"), vec!["johnma"]);
    }

    #[test]
    fn 键唯一有序() {
        let keys = annotate_name("张三");
        let mut sorted = keys.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(keys, sorted);
    }

    #[test]
    fn 未收录生僻字不建键() {
        // 首字无注音（𠀀），第二字正常 → 只剩第二字读音 + 无原文键（不在 ASCII）。
        assert_eq!(annotate_name("𠀀三"), vec!["san"]);
    }

    #[test]
    fn 索引前缀命中与未命中() {
        let contacts = vec![contact("张三"), contact("李四"), contact("王五")];
        let index = build_contact_index(&contacts);
        let hits = contact_candidates(&index, "zhang", 8);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].text, "张三");
        assert_eq!(hits[0].source, CandidateSource::Contact);
        assert!(contact_candidates(&index, "xyz", 8).is_empty());
        assert!(contact_candidates(&index, "", 8).is_empty());
    }

    #[test]
    fn 同名不同键去重() {
        // 多音字双键同前缀（如 曾 + 张 的 c/z 前缀）只输出一次姓名。
        let contacts = vec![contact("曾"), contact("张")];
        let index = build_contact_index(&contacts);
        let hits = contact_candidates(&index, "z", 8);
        let names: Vec<&str> = hits.iter().map(|c| c.text.as_str()).collect();
        assert!(names.contains(&"曾"));
        assert!(names.contains(&"张"));
        let mut deduped = names.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(names.len(), deduped.len(), "同一联系人不得重复输出");
    }

    #[test]
    fn 查询上限截断() {
        let contacts: Vec<VCardContact> = (0..20).map(|i| contact(format!("测试{i:02}"))).collect();
        let index = build_contact_index(&contacts);
        let hits = contact_candidates(&index, "c", 5);
        assert!(hits.len() <= 5);
    }

    #[test]
    fn 确定性两次查询逐位一致() {
        let contacts = vec![contact("张三"), contact("李四"), contact("曾子")];
        let index = build_contact_index(&contacts);
        assert_eq!(
            contact_candidates(&index, "z", 8),
            contact_candidates(&index, "z", 8)
        );
        assert_eq!(index, index);
    }

    #[test]
    fn 建键组合上限() {
        let keys = annotate_name("张张张张张张张");
        assert!(keys.len() <= CONTACT_KEYS_CAP);
    }

    #[test]
    fn 索引容量上限() {
        let contacts: Vec<VCardContact> = (0..12_000)
            .map(|i| contact(format!("容量{i:05}")))
            .collect();
        let index = build_contact_index(&contacts);
        assert!(index.len() <= 10_000);
    }
}
