//! 拼音音节切分。
//!
//! T-007 提供 M2 的全拼输入核心：标准全拼音节表、前缀查询、
//! 动态规划全量切分，以及为双拼预留的输入方案接口。
//! T-006 数据管线后续会用公开权威数据重新生成音节表并做唯一性校验，
//! 当前常量先以语言事实形式入库，来源登记见 `docs/licenses.md`。

/// 标准普通话无调全拼音节表（ASCII 小写，ü 按输入法惯例写作 v）。
///
/// 拼音音节属于语言事实而非实现版权；表项按声母分组便于人工核对，
/// `SyllableTable::from_slices` 会排序去重，不依赖这里的排列顺序。
pub const STANDARD_SYLLABLES: &[&str] = &[
    // 零声母
    "a", "ai", "an", "ang", "ao", "e", "ei", "en", "eng", "er", "o", "ou", //
    // b
    "ba", "bai", "ban", "bang", "bao", "bei", "ben", "beng", "bi", "bian", "biao", "bie", "bin",
    "bing", "bo", "bu", //
    // p
    "pa", "pai", "pan", "pang", "pao", "pei", "pen", "peng", "pi", "pian", "piao", "pie", "pin",
    "ping", "po", "pou", "pu", //
    // m
    "ma", "mai", "man", "mang", "mao", "me", "mei", "men", "meng", "mi", "mian", "miao", "mie",
    "min", "ming", "miu", "mo", "mou", "mu", //
    // f
    "fa", "fan", "fang", "fei", "fen", "feng", "fo", "fou", "fu", //
    // d
    "da", "dai", "dan", "dang", "dao", "de", "dei", "den", "deng", "di", "dia", "dian", "diao",
    "die", "ding", "diu", "dong", "dou", "du", "duan", "dui", "dun", "duo", //
    // t
    "ta", "tai", "tan", "tang", "tao", "te", "teng", "ti", "tian", "tiao", "tie", "ting", "tong",
    "tou", "tu", "tuan", "tui", "tun", "tuo", //
    // n
    "na", "nai", "nan", "nang", "nao", "ne", "nei", "nen", "neng", "ni", "nian", "niang", "niao",
    "nie", "nin", "ning", "niu", "nong", "nou", "nu", "nuan", "nuo", "nv", "nve", "nue", //
    // l
    "la", "lai", "lan", "lang", "lao", "le", "lei", "leng", "li", "lia", "lian", "liang", "liao",
    "lie", "lin", "ling", "liu", "long", "lou", "lu", "luan", "lun", "luo", "lv", "lve",
    "lue", //
    // g
    "ga", "gai", "gan", "gang", "gao", "ge", "gei", "gen", "geng", "gong", "gou", "gu", "gua",
    "guai", "guan", "guang", "gui", "gun", "guo", //
    // k
    "ka", "kai", "kan", "kang", "kao", "ke", "ken", "keng", "kong", "kou", "ku", "kua", "kuai",
    "kuan", "kuang", "kui", "kun", "kuo", //
    // h
    "ha", "hai", "han", "hang", "hao", "he", "hei", "hen", "heng", "hong", "hou", "hu", "hua",
    "huai", "huan", "huang", "hui", "hun", "huo", //
    // j
    "ji", "jia", "jian", "jiang", "jiao", "jie", "jin", "jing", "jiong", "jiu", "ju", "juan", "jue",
    "jun", //
    // q
    "qi", "qia", "qian", "qiang", "qiao", "qie", "qin", "qing", "qiong", "qiu", "qu", "quan", "que",
    "qun", //
    // x
    "xi", "xia", "xian", "xiang", "xiao", "xie", "xin", "xing", "xiong", "xiu", "xu", "xuan", "xue",
    "xun", //
    // zh
    "zha", "zhai", "zhan", "zhang", "zhao", "zhe", "zhei", "zhen", "zheng", "zhi", "zhong", "zhou",
    "zhu", "zhua", "zhuai", "zhuan", "zhuang", "zhui", "zhun", "zhuo", //
    // ch
    "cha", "chai", "chan", "chang", "chao", "che", "chen", "cheng", "chi", "chong", "chou", "chu",
    "chua", "chuai", "chuan", "chuang", "chui", "chun", "chuo", //
    // sh
    "sha", "shai", "shan", "shang", "shao", "she", "shei", "shen", "sheng", "shi", "shou", "shu",
    "shua", "shuai", "shuan", "shuang", "shui", "shun", "shuo", //
    // r
    "ran", "rang", "rao", "re", "ren", "reng", "ri", "rong", "rou", "ru", "ruan", "rui", "run",
    "ruo", //
    // z
    "za", "zai", "zan", "zang", "zao", "ze", "zei", "zen", "zeng", "zi", "zong", "zou", "zu",
    "zuan", "zui", "zun", "zuo", //
    // c
    "ca", "cai", "can", "cang", "cao", "ce", "cei", "cen", "ceng", "ci", "cong", "cou", "cu",
    "cuan", "cui", "cun", "cuo", //
    // s
    "sa", "sai", "san", "sang", "sao", "se", "sen", "seng", "si", "song", "sou", "su", "suan",
    "sui", "sun", "suo", //
    // y
    "ya", "yan", "yang", "yao", "ye", "yi", "yin", "ying", "yo", "yong", "you", "yu", "yuan", "yue",
    "yun", //
    // w
    "wa", "wai", "wan", "wang", "wei", "wen", "weng", "wo", "wu", //
];

/// 输入方案接口。
///
/// 第一版只启用全拼；双拼码表后续作为独立实现接入，切分核心不感知方案细节。
pub trait PinyinScheme: Send + Sync {
    /// 将按键串归一化为待切分的拼音串。
    fn normalize(&self, keys: &str) -> String;
}

/// 全拼输入方案：仅做小写归一化。
#[derive(Debug, Clone, Copy, Default)]
pub struct FullPinyinScheme;

impl PinyinScheme for FullPinyinScheme {
    fn normalize(&self, keys: &str) -> String {
        keys.to_ascii_lowercase()
    }
}

/// 音节表工具。
#[derive(Debug, Clone, Default)]
pub struct SyllableTable {
    syllables: Vec<String>,
}

impl SyllableTable {
    /// 从字符串切片创建音节表，自动排序并去重。
    #[must_use]
    pub fn from_slices(syllables: &[&str]) -> Self {
        let mut items: Vec<String> = syllables.iter().map(|s| String::from(*s)).collect();
        items.sort();
        items.dedup();
        Self { syllables: items }
    }

    /// 标准全拼音节表。
    #[must_use]
    pub fn standard() -> Self {
        Self::from_slices(STANDARD_SYLLABLES)
    }

    /// 检查输入是否为某个音节的完整匹配。
    #[must_use]
    pub fn is_complete_syllable(&self, input: &str) -> bool {
        self.syllables
            .binary_search_by(|syllable| syllable.as_str().cmp(input))
            .is_ok()
    }

    /// 检查输入是否可能成为某音节前缀（空输入视为所有音节的前缀）。
    #[must_use]
    pub fn has_prefix(&self, input: &str) -> bool {
        if input.is_empty() {
            return !self.syllables.is_empty();
        }
        let lower = self.lower_bound(input);
        self.syllables
            .get(lower)
            .is_some_and(|syllable| syllable.starts_with(input))
    }

    /// 返回以输入为前缀的完整音节（有序）。
    #[must_use]
    pub fn complete_syllables_with_prefix(&self, prefix: &str) -> Vec<&str> {
        if prefix.is_empty() {
            return self.syllables.iter().map(String::as_str).collect();
        }
        let lower = self.lower_bound(prefix);
        self.syllables[lower..]
            .iter()
            .take_while(|syllable| syllable.starts_with(prefix))
            .map(String::as_str)
            .collect()
    }

    /// 返回输入本身的完整音节（若存在）；否则返回以输入为首的最小完整音节。
    #[must_use]
    pub fn as_complete(&self, input: &str) -> Option<String> {
        if self.is_complete_syllable(input) {
            Some(input.to_owned())
        } else {
            self.complete_syllables_with_prefix(input)
                .first()
                .map(|s| (*s).to_owned())
        }
    }

    fn lower_bound(&self, prefix: &str) -> usize {
        self.syllables
            .partition_point(|syllable| syllable.as_str() < prefix)
    }
}

/// 判断输入串是否为合法音节前缀。
#[must_use]
pub fn valid_prefix(table: &SyllableTable, input: &str) -> bool {
    !input.is_empty() && table.has_prefix(input)
}

/// 将输入串切分为完整音节的全部可行方案。
///
/// 使用动态规划自底向上枚举：`dp[end]` 保存前 `end` 个字符的全部切分，
/// 最终方案顺序由结束位置和起始位置递增决定，结果确定。
/// 输入包含非 ASCII 或无法完整切分时返回空列表。
#[must_use]
pub fn segment_all(table: &SyllableTable, input: &str) -> Vec<Vec<String>> {
    if !input.is_ascii() {
        return Vec::new();
    }

    let length = input.len();
    let mut dp: Vec<Vec<Vec<String>>> = (0..=length).map(|_| Vec::new()).collect();
    dp[0].push(Vec::new());

    for end in 1..=length {
        for start in 0..end {
            if dp[start].is_empty() {
                continue;
            }
            let candidate = &input[start..end];
            if !table.is_complete_syllable(candidate) {
                continue;
            }
            let tail = candidate.to_owned();
            let prefixes = dp[start].clone();
            for prefix in &prefixes {
                let mut next = prefix.clone();
                next.push(tail.clone());
                dp[end].push(next);
            }
        }
    }

    std::mem::take(&mut dp[length])
}

/// 简拼展开用的静态常用音节表（M7，方案设计 12.2.2）。
///
/// 每个 ASCII 小写字母对应 4-6 个**最常用**音节，按口语常用度人工排序。
/// 属于语言事实（非版权数据），自研维护；某字母无表项表示该首字母无简拼。
pub const INITIAL_SYLLABLE_TABLE: &[(char, &[&str])] = &[
    ('b', &["bu", "ba", "bei", "ben", "bian"]),
    ('c', &["ci", "ca", "cai", "can", "ceng", "cuo"]),
    ('d', &["de", "da", "dan", "dang", "di", "duo"]),
    ('e', &["er", "en"]),
    ('f', &["fa", "fan", "fang", "fei", "fu"]),
    ('g', &["ge", "gao", "gan", "gang", "gong", "guo"]),
    ('h', &["hao", "he", "hai", "han", "hui", "hua"]),
    ('j', &["ji", "jian", "jiang", "jiao", "jin", "ju"]),
    ('k', &["ke", "kai", "kan", "kuai", "kuang"]),
    ('l', &["li", "la", "lai", "lan", "liang", "lu"]),
    ('m', &["me", "ma", "ming", "men", "mu"]),
    ('n', &["ni", "na", "nan", "neng", "nian"]),
    ('o', &["ou"]),
    ('p', &["pa", "pai", "pan", "pi", "ping"]),
    ('q', &["qi", "qian", "qing", "qiu", "qu"]),
    ('r', &["ran", "ren", "ri", "rong", "ru"]),
    ('s', &["shi", "shen", "sheng", "sui", "suo"]),
    ('t', &["ta", "tian", "tiao", "ting", "tong", "tu"]),
    ('w', &["wo", "wei", "wan", "wang", "wen"]),
    ('x', &["xi", "xia", "xian", "xiang", "xiao", "xin"]),
    ('y', &["you", "yi", "yan", "yang", "yao", "yu"]),
    ('z', &["zuo", "zi", "zai", "zan", "zheng", "zhi"]),
];

/// 查询简拼音节表中某字母对应的常用音节；未收录返回空切片。
#[must_use]
pub fn initial_syllables(initial: char) -> &'static [&'static str] {
    INITIAL_SYLLABLE_TABLE
        .iter()
        .find(|(letter, _)| *letter == initial)
        .map_or(&[], |(_, syllables)| *syllables)
}

/// 模糊音等价映射组（M7，方案设计 12.3.2）。
///
/// 每组内音节片段互为模糊等价；映射为语言事实（方言习惯），自研维护。
/// 覆盖文档枚举的 7 组：zh↔z、ch↔c、sh↔s、n↔l、f↔h、an↔ang、en↔eng、in↔ing。
pub const FUZZY_GROUPS: &[&[&str]] = &[
    &["zh", "z"],
    &["ch", "c"],
    &["sh", "s"],
    &["n", "l"],
    &["f", "h"],
    &["an", "ang"],
    &["en", "eng"],
    &["in", "ing"],
];

/// 返回单个音节的全部模糊变体（不含原音节本身），顺序确定。
///
/// 只做**单处替换**：对每个映射组，若音节内含该组任一片段，替换为组内
/// 其余片段生成变体。例如 `zong` → `zhong`（zh↔z）、`lan` → `nan`（n↔l）。
/// 音节不含任何映射片段时返回空。
#[must_use]
pub fn fuzzy_variants(syllable: &str) -> Vec<String> {
    let mut variants = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for group in FUZZY_GROUPS {
        for piece in *group {
            if !syllable.contains(piece) {
                continue;
            }
            for alternative in *group {
                if alternative == piece {
                    continue;
                }
                let variant = syllable.replace(piece, alternative);
                if seen.insert(variant.clone()) {
                    variants.push(variant);
                }
            }
        }
    }
    variants
}

#[cfg(test)]
mod tests {
    use super::{
        fuzzy_variants, initial_syllables, segment_all, valid_prefix, FullPinyinScheme,
        PinyinScheme, SyllableTable, INITIAL_SYLLABLE_TABLE, STANDARD_SYLLABLES,
    };
    use std::collections::HashSet;

    fn table() -> SyllableTable {
        SyllableTable::standard()
    }

    #[test]
    fn 标准表无重复且覆盖公共音节() {
        let table = table();
        let unique: HashSet<&&str> = STANDARD_SYLLABLES.iter().collect();
        assert_eq!(unique.len(), STANDARD_SYLLABLES.len());
        assert!(STANDARD_SYLLABLES.len() >= 400);

        let common = [
            "duo", "cha", "guan", "juan", "quan", "cong", "chuang", "zhou", "shuang", "lve",
        ];
        assert!(
            common
                .iter()
                .all(|syllable| table.is_complete_syllable(syllable)),
            "常用全拼音节应全部存在"
        );
    }

    #[test]
    fn 标准表全部音节可单独切分() {
        let table = table();
        for syllable in STANDARD_SYLLABLES {
            let segments = segment_all(&table, syllable);
            assert!(
                segments.contains(&vec![(*syllable).to_owned()]),
                "音节 {syllable} 应能完整切分"
            );
        }
    }

    #[test]
    fn 基础音节完整识别() {
        let table = table();
        assert!(table.is_complete_syllable("ni"));
        assert!(table.is_complete_syllable("hao"));
        assert!(!table.is_complete_syllable("nihao"));
    }

    #[test]
    fn 前缀识别正确() {
        let table = table();
        assert!(valid_prefix(&table, "zh"));
        assert!(valid_prefix(&table, "zhong"));
        assert!(table.has_prefix(""));
        assert!(!valid_prefix(&table, "zz"));
        assert!(!valid_prefix(&table, ""));
    }

    #[test]
    fn 前缀查询返回有序完整音节() {
        let table = table();
        let all = table.complete_syllables_with_prefix("");
        assert_eq!(all.len(), STANDARD_SYLLABLES.len());
        assert!(all.windows(2).all(|pair| pair[0] < pair[1]));

        let xi = table.complete_syllables_with_prefix("xi");
        assert!(xi.contains(&"xiang"));
        assert!(xi.contains(&"xian"));
    }

    #[test]
    fn nihao可切分为你好拼音() {
        let table = table();
        let segments = segment_all(&table, "nihao");
        assert!(segments.contains(&vec!["ni".to_owned(), "hao".to_owned()]));
        assert!(segments.contains(&vec!["ni".to_owned(), "ha".to_owned(), "o".to_owned()]));
    }

    #[test]
    fn xian支持西安与先两种切分() {
        let table = table();
        let segments = segment_all(&table, "xian");
        assert!(segments.contains(&vec!["xi".to_owned(), "an".to_owned()]));
        assert!(segments.contains(&vec!["xian".to_owned()]));
    }

    #[test]
    fn 多音节串保留全部歧义切分() {
        let table = table();
        let segments = segment_all(&table, "xiange");
        assert!(segments.contains(&vec!["xian".to_owned(), "ge".to_owned()]));
        assert!(segments.contains(&vec!["xi".to_owned(), "an".to_owned(), "ge".to_owned()]));
        assert!(segments.contains(&vec!["xiang".to_owned(), "e".to_owned()]));
    }

    #[test]
    fn 无法切分返回空() {
        let table = table();
        assert!(segment_all(&table, "zzzz").is_empty());
        assert!(segment_all(&table, "a1").is_empty());
        assert!(segment_all(&table, "你好").is_empty());
    }

    #[test]
    fn 全拼方案仅做小写归一化() {
        let scheme = FullPinyinScheme;
        assert_eq!(scheme.normalize("NiHao"), "nihao");
        assert_eq!(scheme.normalize("zhongguo"), "zhongguo");
    }

    #[test]
    fn 简拼音节表覆盖常用首字母且音节合法() {
        let table = table();
        let mut letters: Vec<char> = INITIAL_SYLLABLE_TABLE
            .iter()
            .map(|(letter, _)| *letter)
            .collect();
        letters.sort_unstable();
        let expected: Vec<char> = "bcdefghjklmnopqrstwxyz".chars().collect();
        assert_eq!(letters, expected, "简拼表应覆盖全部常用声母字母");
        for (letter, syllables) in INITIAL_SYLLABLE_TABLE {
            assert!(!syllables.is_empty(), "字母 {letter} 不应有空音节列表");
            assert!(
                syllables
                    .iter()
                    .all(|s| s.starts_with(*letter) && table.is_complete_syllable(s)),
                "字母 {letter} 的音节必须以该字母开头且为合法音节"
            );
        }
    }

    #[test]
    fn 简拼音节查询命中与未收录() {
        assert!(initial_syllables('n').contains(&"ni"));
        assert!(initial_syllables('h').contains(&"hao"));
        assert!(initial_syllables('w').contains(&"wei"));
        assert!(initial_syllables('v').is_empty(), "未收录字母返回空");
        assert!(initial_syllables('0').is_empty(), "非字母返回空");
    }

    #[test]
    fn 模糊变体单处替换且确定() {
        let mut variants = fuzzy_variants("zong");
        variants.sort();
        assert!(variants.contains(&"zhong".to_owned()));
        assert!(!variants.contains(&"zong".to_owned()), "不含原音节本身");
        assert!(variants.len() <= 4, "单处替换变体数受控");

        let lan = fuzzy_variants("lan");
        assert!(lan.contains(&"nan".to_owned()), "n↔l 映射：lan→nan");

        let feng = fuzzy_variants("feng");
        assert!(feng.contains(&"heng".to_owned()), "f↔h 映射：feng→heng");
        assert!(!feng.contains(&"feng".to_owned()));

        let first = fuzzy_variants("zong");
        let second = fuzzy_variants("zong");
        assert_eq!(first, second, "变体枚举确定");
    }

    #[test]
    fn 无映射音节无变体() {
        assert!(fuzzy_variants("wo").is_empty());
        assert!(fuzzy_variants("duo").is_empty());
        assert!(fuzzy_variants("").is_empty());
    }
}
