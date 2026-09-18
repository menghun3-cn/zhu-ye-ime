//! 拼音音节切分。
//!
//! 脚手架阶段内置基础音节表用于验证切分算法；M1 里程碑将改为
//! 由数据管线从公开权威音节数据生成完整表并做唯一性校验。

/// 基础音节表（占位，M1 替换为完整生成表）。
///
/// 拼音音节属于语言事实而非实现版权，后续生成脚本会入库。
pub const BASIC_SYLLABLES: &[&str] = &[
    "a", "ai", "an", "ang", "ao", "ba", "bai", "ban", "bang", "bao", "bei", "ben", "beng", "bi",
    "bian", "biao", "bie", "bin", "bing", "bo", "bu", "ca", "cai", "can", "cang", "cao", "che",
    "chen", "cheng", "chi", "chong", "chou", "chu", "chuan", "chuang", "chun", "ci", "cu", "da",
    "dai", "dan", "dang", "dao", "de", "deng", "di", "dian", "diao", "die", "ding", "dong", "du",
    "duan", "dui", "dun", "e", "en", "er", "fa", "fan", "fang", "fei", "fen", "feng", "fo", "fu",
    "ga", "gan", "gang", "gao", "ge", "gei", "gen", "geng", "gong", "gou", "gu", "guo", "ha",
    "hai", "han", "hang", "hao", "he", "hei", "hen", "heng", "hong", "hou", "hu", "hua", "huai",
    "huan", "huang", "hui", "hun", "huo", "ji", "jia", "jian", "jiang", "jiao", "jie", "jin",
    "jing", "jiong", "jiu", "ju", "jue", "jun", "ka", "kai", "kan", "kang", "kao", "ke", "ken",
    "kong", "kou", "ku", "kuai", "kuan", "kuang", "kui", "kun", "kuo", "la", "lai", "lan", "lang",
    "lao", "le", "lei", "leng", "li", "lian", "liang", "liao", "lie", "lin", "ling", "liu", "long",
    "lou", "lu", "luan", "lun", "luo", "lv", "ma", "mai", "man", "mang", "mao", "me", "mei", "men",
    "meng", "mi", "mian", "miao", "mie", "min", "ming", "miu", "mo", "mou", "mu", "na", "nai",
    "nan", "nang", "nao", "ne", "nei", "nen", "neng", "ni", "nian", "niang", "niao", "nie", "nin",
    "ning", "niu", "nong", "nu", "nuan", "nuo", "nv", "o", "ou", "pa", "pai", "pan", "pang", "pao",
    "pei", "pen", "peng", "pi", "pian", "piao", "pie", "pin", "ping", "po", "pou", "pu", "qi",
    "qia", "qian", "qiang", "qiao", "qie", "qin", "qing", "qiong", "qiu", "qu", "que", "qun",
    "ran", "rang", "rao", "re", "ren", "reng", "ri", "rong", "rou", "ru", "ruan", "rui", "ruo",
    "sa", "sai", "san", "sang", "sao", "se", "sen", "seng", "sha", "shai", "shan", "shang", "shao",
    "she", "shei", "shen", "sheng", "shi", "shou", "shu", "shua", "shuai", "shuan", "shuang",
    "shui", "shun", "shuo", "si", "song", "sou", "su", "suan", "sui", "sun", "suo", "ta", "tai",
    "tan", "tang", "tao", "te", "teng", "ti", "tian", "tiao", "tie", "ting", "tong", "tou", "tu",
    "tuan", "tui", "tun", "tuo", "wa", "wai", "wan", "wang", "wei", "wen", "weng", "wo", "wu",
    "xi", "xia", "xian", "xiang", "xiao", "xie", "xin", "xing", "xiong", "xiu", "xu", "xuan",
    "xue", "xun", "ya", "yan", "yang", "yao", "ye", "yi", "yin", "ying", "yo", "yong", "you", "yu",
    "yuan", "yue", "yun", "za", "zai", "zan", "zang", "zao", "ze", "zei", "zen", "zeng", "zha",
    "zhai", "zhan", "zhang", "zhao", "zhe", "zhen", "zheng", "zhi", "zhong", "zhou", "zhu", "zhua",
    "zhuai", "zhuan", "zhuang", "zhui", "zhun", "zhuo", "zi", "zong", "zou", "zu", "zuan", "zui",
    "zun", "zuo",
];

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

    /// 内置基础音节表。
    #[must_use]
    pub fn basic() -> Self {
        Self::from_slices(BASIC_SYLLABLES)
    }

    /// 检查输入是否为某个音节的完整匹配。
    #[must_use]
    pub fn is_complete_syllable(&self, input: &str) -> bool {
        self.syllables.binary_search(&input.to_owned()).is_ok()
    }

    /// 检查输入是否为某音节前缀。
    #[must_use]
    pub fn has_prefix(&self, input: &str) -> bool {
        let needle = input.to_owned();
        self.syllables
            .iter()
            .any(|syllable| syllable.starts_with(&needle))
    }

    /// 返回以输入为前缀的完整音节（有序）。
    #[must_use]
    pub fn complete_syllables_with_prefix(&self, prefix: &str) -> Vec<&str> {
        let needle = prefix.to_owned();
        self.syllables
            .iter()
            .filter(|syllable| syllable.starts_with(&needle))
            .map(String::as_str)
            .collect()
    }

    /// 返回输入本身的完整音节（若存在）。
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
}

/// 判断输入串是否为合法音节前缀。
#[must_use]
pub fn valid_prefix(table: &SyllableTable, input: &str) -> bool {
    !input.is_empty() && table.has_prefix(input)
}

/// 将输入串切分为完整音节的全部可行方案。
///
/// 返回每个方案视为一组音节；顺序按深度优先遍历确定。
/// 若输入无法完整切分，返回空列表。
#[must_use]
pub fn segment_all(table: &SyllableTable, input: &str) -> Vec<Vec<String>> {
    let mut result = Vec::new();
    let mut current = Vec::new();
    segment_rec(table, input, &mut current, &mut result);
    result
}

fn segment_rec(
    table: &SyllableTable,
    rest: &str,
    current: &mut Vec<String>,
    result: &mut Vec<Vec<String>>,
) {
    if rest.is_empty() {
        result.push(current.clone());
        return;
    }
    for syllable in &table.syllables {
        if rest.starts_with(syllable) {
            current.push(syllable.clone());
            segment_rec(table, &rest[syllable.len()..], current, result);
            current.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{segment_all, SyllableTable};

    #[test]
    fn 基础音节完整识别() {
        let table = SyllableTable::basic();
        assert!(table.is_complete_syllable("ni"));
        assert!(table.is_complete_syllable("hao"));
        assert!(!table.is_complete_syllable("nihao"));
    }

    #[test]
    fn 前缀识别正确() {
        let table = SyllableTable::basic();
        assert!(table.has_prefix("zh"));
        assert!(table.has_prefix("zhong"));
        assert!(!table.has_prefix("zz"));
    }

    #[test]
    fn nihao可切分为你好拼音() {
        let table = SyllableTable::basic();
        let segments = segment_all(&table, "nihao");
        assert!(segments.contains(&vec!["ni".to_owned(), "hao".to_owned()]));
    }

    #[test]
    fn xian支持西安与先两种切分() {
        let table = SyllableTable::basic();
        let segments = segment_all(&table, "xian");
        assert!(segments.contains(&vec!["xi".to_owned(), "an".to_owned()]));
        assert!(segments.contains(&vec!["xian".to_owned()]));
    }

    #[test]
    fn 无法切分返回空() {
        let table = SyllableTable::basic();
        assert!(segment_all(&table, "zzzz").is_empty());
    }
}
