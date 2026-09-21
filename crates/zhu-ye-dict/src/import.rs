//! 真实词库导入：CC-CEDICT + 开放词频语料 -> v2 词典。
//!
//! 数据源与许可证记录在 `docs/数据清单.md` 与 `docs/licenses.md`。
//! 本模块负责行解析、无调拼音归一化、标准音节校验与词频映射，
//! 清洗结果交给 `build_v2` 编译。真实词条中未命中词频表的词按 1
//! 计入，保证排序可比较且常用词仍由真实频率主导。

use std::collections::{HashMap, HashSet};

use zhu_ye_core::dict::DictionaryEntry;
use zhu_ye_core::pinyin::SyllableTable;

/// 导入统计，供 CLI 输出与测试断言。
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ImportStats {
    /// CC-CEDICT 总行数（含注释与空行）。
    pub cedict_lines: usize,
    /// 通过全部清洗规则并进入构建的词条数。
    pub accepted_entries: usize,
    /// 拼音缺失或包含非拼音字符被丢弃。
    pub dropped_no_pinyin: usize,
    /// 存在标准音节表之外的音节被丢弃。
    pub dropped_bad_syllable: usize,
    /// 简体词不是纯中文词形被丢弃。
    pub dropped_word_shape: usize,
    /// 音节数与汉字数不一致被丢弃。
    pub dropped_syllable_count_mismatch: usize,
    /// 最终词条数（可能受 max_entries 截断）。
    pub entry_count: usize,
    /// 命中词频表的词条数。
    pub frequency_hits: usize,
    /// 参与校验的音节总数。
    pub total_syllables: usize,
    /// 无效音节总数。
    pub invalid_syllables: usize,
    /// 未知音节去重排序后的样本（最多 20 个）。
    pub unknown_syllables: Vec<String>,
}

/// 解析 CC-CEDICT 一行，返回简体词、带声调标记的拼音与首条可读译文。
///
/// 官方 `cedict_ts.u8` 格式：`传统词 简体词 [pin1 yin1] /译1/译2/`。
/// 注释与空行返回 `None`；带 `CL:` 前缀的条目不进入反查候选。
pub fn parse_cedict_line(line: &str) -> Option<(String, String, Option<String>)> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    // 官方导出使用空格分隔，旧版/镜像可能使用制表符；统一按 `[拼音]` 定界。
    let bracket_start = line.find('[')?;
    let bracket_end = line[bracket_start + 1..].find(']')? + bracket_start + 1;
    let mut head = line[..bracket_start].split_whitespace();
    let _traditional = head.next()?;
    let simplified = head.next()?.trim();
    let marked = line[bracket_start + 1..bracket_end].trim();
    let translations = line[bracket_end + 1..].trim_start_matches('/');
    if simplified.is_empty() {
        return None;
    }
    Some((
        simplified.to_owned(),
        marked.to_owned(),
        first_translation(translations),
    ))
}

/// 取翻译字段中第一条可用英文释义。
fn first_translation(field: &str) -> Option<String> {
    for part in field.split('/') {
        let part = part.trim();
        if part.is_empty() || part.starts_with("CL:") {
            continue;
        }
        if part.chars().count() > 200 {
            continue;
        }
        return Some(part.to_owned());
    }
    None
}

/// 加载 `word count` 格式的词频映射；计数超过 v2 字段上限时截断。
pub fn load_frequency_map(text: &str) -> HashMap<String, u64> {
    let mut map = HashMap::new();
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let Some(word) = parts.next() else {
            continue;
        };
        let Some(count) = parts.next().and_then(|value| value.parse::<u64>().ok()) else {
            continue;
        };
        map.insert(word.to_owned(), count.min(u64::from(u32::MAX)));
    }
    map
}

/// 把带声调数字/`ü`/`u:`/撇号的拼音归一化为无调 ASCII 拼音串。
///
/// 声调数字删除，`ü` 与 `u:` 按输入法惯例写作 `v`；空格与撇号是分词界符，
/// 一并删除，输出与引擎拼音键一致的连续全拼串。
/// 返回 `None` 表示存在拼音之外的字符。
pub fn normalize_pinyin(marked: &str) -> Option<String> {
    let marked = marked.replace("u:", "v");
    let mut output = String::new();
    for ch in marked.chars() {
        match ch {
            '0'..='9' => {}
            'ü' => output.push('v'),
            ch if ch == '\'' || ch.is_whitespace() => {}
            ch if ch.is_ascii_alphabetic() => output.push(ch.to_ascii_lowercase()),
            _ => return None,
        }
    }
    let output = output.trim().to_owned();
    if output.is_empty() {
        None
    } else {
        Some(output)
    }
}

/// 按 CC-CEDICT 的空白/撇号分界拆分音节，并逐个归一化。
/// 返回每个音节的无调连续全拼；任一分界片段含非法字符时返回 `None`。
fn split_pinyin_syllables(marked: &str) -> Option<Vec<String>> {
    let mut syllables = Vec::new();
    for part in marked.split([' ', '\'']) {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        syllables.push(normalize_pinyin(part)?);
    }
    if syllables.is_empty() {
        None
    } else {
        Some(syllables)
    }
}

/// 检查简体词是否只由 CJK 统一表意文字组成。
fn is_cjk_word(word: &str) -> bool {
    !word.is_empty()
        && word
            .chars()
            .all(|ch| ('\u{4e00}'..='\u{9fff}').contains(&ch))
}

/// 从原始文本构建真实词条列表。
///
/// 清洗规则：简体词须为纯 CJK 词形；无调拼音全部音节必须在标准全拼表中；
/// 音节数必须与汉字数一致；同词同拼音只保留首个词条。词频未命中时按 1。
pub fn build_real_dictionary(
    cedict_text: &str,
    frequency_text: &str,
    max_entries: Option<usize>,
) -> (Vec<DictionaryEntry>, ImportStats) {
    let table = SyllableTable::standard();
    let frequencies = load_frequency_map(frequency_text);
    let mut entries = Vec::new();
    let mut stats = ImportStats::default();
    let mut seen = HashSet::new();

    for line in cedict_text.lines() {
        stats.cedict_lines += 1;
        let Some((word, marked, translation)) = parse_cedict_line(line) else {
            continue;
        };
        let Some(syllables) = split_pinyin_syllables(&marked) else {
            stats.dropped_no_pinyin += 1;
            continue;
        };
        stats.total_syllables += syllables.len();
        let mut bad = false;
        for syllable in &syllables {
            if !table.is_complete_syllable(syllable.as_str()) {
                bad = true;
                stats.invalid_syllables += 1;
                stats.unknown_syllables.push(syllable.clone());
            }
        }

        if bad {
            stats.dropped_bad_syllable += 1;
            continue;
        }
        if !is_cjk_word(&word) {
            stats.dropped_word_shape += 1;
            continue;
        }
        if syllables.len() != word.chars().count() {
            stats.dropped_syllable_count_mismatch += 1;
            continue;
        }

        let frequency = frequencies.get(&word).copied().unwrap_or(1);
        if frequencies.contains_key(&word) {
            stats.frequency_hits += 1;
        }
        let pinyin = syllables.join("");
        if !seen.insert((word.clone(), pinyin.clone())) {
            continue;
        }

        let mut entry = DictionaryEntry::new(word, pinyin, frequency);
        if let Some(translation) = &translation {
            entry = entry.with_translation(translation.clone());
        }
        entries.push(entry);
        stats.accepted_entries += 1;
    }

    if let Some(max) = max_entries {
        entries.truncate(max);
    }
    stats.entry_count = entries.len();
    stats.unknown_syllables.sort();
    stats.unknown_syllables.dedup();
    stats.unknown_syllables.truncate(20);
    (entries, stats)
}

#[cfg(test)]
mod tests {
    use super::{build_real_dictionary, load_frequency_map, normalize_pinyin, parse_cedict_line};

    #[test]
    fn 解析ccdect数据行() {
        let (word, marked, translation) =
            parse_cedict_line("你\t你\t[ni3]\t/you (informal)/").unwrap();
        assert_eq!(word, "你");
        assert_eq!(marked, "ni3");
        assert_eq!(translation.as_deref(), Some("you (informal)"));

        let (word, marked, translation) =
            parse_cedict_line("你好\t你好\t[ni3 hao3]\t/hello!/").unwrap();
        assert_eq!(word, "你好");
        assert_eq!(marked, "ni3 hao3");
        assert_eq!(translation.as_deref(), Some("hello!"));
    }

    #[test]
    fn 注释与缺字段行不解析() {
        assert!(parse_cedict_line("# CC-CEDICT").is_none());
        assert!(parse_cedict_line("").is_none());
        assert!(parse_cedict_line("只有两个字段").is_none());
    }

    #[test]
    fn 翻译字段跳过cl条目() {
        let (_, _, translation) = parse_cedict_line("三\t三\t[san1]\t/CL:三/three/").unwrap();
        assert_eq!(translation.as_deref(), Some("three"));
    }

    #[test]
    fn 无调拼音去掉声调并处理ü与撇号() {
        assert_eq!(normalize_pinyin("lü3").as_deref(), Some("lv"));
        assert_eq!(normalize_pinyin("nu:3").as_deref(), Some("nv"));
        assert_eq!(normalize_pinyin("ni3 hao3").as_deref(), Some("nihao"));
        assert_eq!(normalize_pinyin("xi'an4").as_deref(), Some("xian"));
        assert!(normalize_pinyin("ninja!").is_none());
    }

    #[test]
    fn 词频解析并截断上限() {
        let map = load_frequency_map("的 3957141\n你好 42\n");
        assert_eq!(map.get("的"), Some(&3_957_141));
        assert_eq!(map.get("你好"), Some(&42));
        assert_eq!(map.get("不在表中"), None);
    }

    #[test]
    fn 真实样例构建并映射词频() {
        let cedict = "你好\t你好\t[ni3 hao3]\t/hello/\n世界\t世界\t[shi4 jie4]\t/world/";
        let frequencies = "你好 42\n";
        let (entries, stats) = build_real_dictionary(cedict, frequencies, None);
        assert_eq!(entries.len(), 2);
        assert_eq!(stats.frequency_hits, 1);
        let hello = entries.iter().find(|entry| entry.word == "你好").unwrap();
        assert_eq!(hello.pinyin, "nihao");
        assert_eq!(hello.translation.as_deref(), Some("hello"));
        assert_eq!(hello.frequency, 42);
        let world = entries.iter().find(|entry| entry.word == "世界").unwrap();
        assert_eq!(world.frequency, 1);
    }

    #[test]
    fn 无效音节与词形被丢弃() {
        let cedict = concat!(
            "好\t好\t[hao3]\t/good/\n",
            "呣\t呣\t[m5]\t/(dialect) hello/\n",
            "AIDS\tAIDS\t[ai4 zi4]\t/AIDS/\n",
            "卡\t卡\t[ka3]\t/to check/\n"
        );
        let (entries, stats) = build_real_dictionary(cedict, "", None);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].word, "好");
        assert_eq!(entries[1].word, "卡");
        assert_eq!(stats.dropped_bad_syllable, 1);
        assert_eq!(stats.dropped_word_shape, 1);
        assert_eq!(stats.invalid_syllables, 1);
        assert_eq!(stats.unknown_syllables, vec!["m".to_owned()]);
    }

    #[test]
    fn 音节数与字数不符被丢弃() {
        let cedict = "设计师\t设计师\t[she4 ji4]\t/designer/";
        let (entries, stats) = build_real_dictionary(cedict, "", None);
        assert!(entries.is_empty());
        assert_eq!(stats.dropped_syllable_count_mismatch, 1);
    }

    #[test]
    fn 重复词拼音去重并支持截断() {
        let cedict = "好\t好\t[hao3]\t/good/\n好\t好\t[hao4]\t/to like/\n";
        let (entries, stats) = build_real_dictionary(cedict, "", Some(1));
        assert_eq!(stats.accepted_entries, 1);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].pinyin, "hao");
    }
}
