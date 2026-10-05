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
use crate::pinyin::{segment_all, SyllableTable};
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

/// 从全拼键派生简拼键（FR-037 简拼可达）：对键做音节切分，取每音节**首字母**。
///
/// - 切分用标准音节表最长匹配（`segment_all` 首个切分，确定性）；
/// - 非拼音键（英文名原形、含数字）切分失败 → 不派生（原文键已可命中）；
/// - 单音节键不派生（`zs` 类简拼至少两音节才有区分度）。
#[must_use]
pub fn abbreviation_key(name_key: &str, table: &SyllableTable) -> Option<String> {
    if !name_key.is_ascii() {
        return None;
    }
    let first = segment_all(table, name_key).into_iter().next()?;
    if first.len() < 2 {
        return None;
    }
    let mut abbr = String::with_capacity(first.len());
    for syllable in &first {
        abbr.push(syllable.chars().next()?);
    }
    (abbr.bytes().all(|b| b.is_ascii_lowercase())).then_some(abbr)
}

/// 由 vCard 联系人建立联系人索引（T-071-2；keys 在此填充，超过
/// [`CONTACT_INDEX_CAP`] 条按出现顺序截断）。
///
/// 建键来源（D-20 扩展，T-102）：姓名 + 组织（`ORG`）+ 地址（`ADR`）+ 电邮（`EMAIL`）
/// 四者各自注音/归一生成键集，合并去重后索引同一联系人；命中任意来源均上屏姓名。
#[must_use]
pub fn build_contact_index(contacts: &[VCardContact]) -> ContactIndex {
    let table = SyllableTable::standard();
    let mut by_key: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for contact in contacts.iter().take(CONTACT_INDEX_CAP) {
        if contact.name.is_empty() {
            continue;
        }
        for key in contact_keys(contact) {
            let names = by_key.entry(key.clone()).or_default();
            if !names.iter().any(|existing| existing == &contact.name) {
                names.push(contact.name.clone());
            }
            // 简拼键（FR-037）：全拼键派生首字母键，指向同一姓名。
            if let Some(abbr) = abbreviation_key(&key, &table) {
                let names = by_key.entry(abbr).or_default();
                if !names.iter().any(|existing| existing == &contact.name) {
                    names.push(contact.name.clone());
                }
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

/// 单个联系人的全部检索键：姓名/组织/地址/电邮四源各自 [`annotate_name`] 后合并去重。
///
/// - 中文（公司名/地址）经注音建键：`北京朝阳` → `beijingchaoyang`，可拼音命中；
/// - 电邮/ASCII 按原文归一（`@` `.` 等符号被跳过）：`zhangsan@acme.com` → `zhangsanacmecom`，
///   输入 `zhangsan` 前缀即命中；
/// - 同联系人跨源产生的同键只保留一次（确定性去重）。
fn contact_keys(contact: &VCardContact) -> Vec<String> {
    let mut keys = Vec::new();
    keys.extend(annotate_name(&contact.name));
    if !contact.org.is_empty() {
        keys.extend(annotate_name(&contact.org));
    }
    if !contact.address.is_empty() {
        keys.extend(annotate_name(&contact.address));
    }
    if !contact.email.is_empty() {
        keys.extend(annotate_name(&contact.email));
    }
    keys.sort();
    keys.dedup();
    keys
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
        VCardContact::new(name.into())
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
    fn 简拼键命中() {
        // FR-037：`zs` 前缀命中 张三（zhangsan → zs）。
        let contacts = vec![contact("张三")];
        let index = build_contact_index(&contacts);
        let hits = contact_candidates(&index, "zs", 8);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].text, "张三");
    }

    #[test]
    fn 多音简拼键_多形态() {
        // 曾子 → zengzi(zz) / cengzi(cz)：两个简拼键都可达（D-22 全形态原则延伸）。
        let contacts = vec![contact("曾子")];
        let index = build_contact_index(&contacts);
        assert!(!contact_candidates(&index, "zz", 8).is_empty());
        assert!(!contact_candidates(&index, "cz", 8).is_empty());
    }

    #[test]
    fn 英文名不派生简拼键() {
        let contacts = vec![contact("Alice")];
        let index = build_contact_index(&contacts);
        // alice 无拼音音节切分 → 不派生简拼键，原文小写键仍可前缀命中（D-20）。
        assert!(!contact_candidates(&index, "al", 8).is_empty());
        assert!(contact_candidates(&index, "zs", 8).is_empty());
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

    // ---------- T-102：D-20 扩展——公司/地址/邮箱建索引 ----------

    fn contact_with(org: &str, email: &str, address: &str) -> VCardContact {
        VCardContact {
            org: org.to_owned(),
            email: email.to_owned(),
            address: address.to_owned(),
            ..VCardContact::new("王五")
        }
    }

    #[test]
    fn 公司名注音键命中() {
        // ORG 为中文公司名 → 拼音前缀命中联系人（上屏姓名）。
        let c = contact_with("竹叶科技", "", "");
        let index = build_contact_index(&[c]);
        let hits = contact_candidates(&index, "zhuye", 8);
        assert_eq!(
            hits.iter().map(|c| c.text.as_str()).collect::<Vec<_>>(),
            vec!["王五"]
        );
        assert_eq!(hits[0].source, CandidateSource::Contact);
    }

    #[test]
    fn 公司名简拼键命中() {
        // 中文公司名全拼键派生首字母键（竹叶科技 → zykj）。
        let c = contact_with("竹叶科技", "", "");
        let index = build_contact_index(&[c]);
        assert_eq!(contact_candidates(&index, "zykj", 8)[0].text, "王五");
    }

    #[test]
    fn 英文公司名原形键命中() {
        let c = contact_with("Alice Tech", "", "");
        let index = build_contact_index(&[c]);
        assert_eq!(contact_candidates(&index, "alice", 8)[0].text, "王五");
    }

    #[test]
    fn 邮箱前缀命中() {
        // EMAIL 归一（符号跳过）后按前缀命中：`zhangsanacmecom`，「zhangsan」可及。
        let c = contact_with("", "zhangsan@acme.com", "");
        let index = build_contact_index(&[c]);
        let hits = contact_candidates(&index, "zhangsan", 8);
        assert_eq!(
            hits.iter().map(|c| c.text.as_str()).collect::<Vec<_>>(),
            vec!["王五"]
        );
        // 邮箱尾巴不出键（符号之后的部分不被提取为独立键）。
        assert!(contact_candidates(&index, "acmecom", 8).is_empty());
    }

    #[test]
    fn 地址注音键命中() {
        // ADR 中文地址 → 拼音前缀命中（地址组件分号被跳过 → 连续字符建键）。
        let c = contact_with("", "", "北京市朝阳区;建国路 88 号");
        let index = build_contact_index(&[c]);
        assert_eq!(contact_candidates(&index, "beijing", 8)[0].text, "王五");
    }

    #[test]
    fn 四源同键去重() {
        // 姓名与公司命中同一键时只产生一条候选（去重保序）。
        let c = VCardContact {
            org: "张三工作室".to_owned(),
            email: "zhangsan@x.com".to_owned(),
            address: "".to_owned(),
            ..VCardContact::new("张三")
        };
        let index = build_contact_index(&[c]);
        let hits = contact_candidates(&index, "zhangsan", 8);
        assert_eq!(hits.iter().filter(|c| c.text == "张三").count(), 1);
    }
}
