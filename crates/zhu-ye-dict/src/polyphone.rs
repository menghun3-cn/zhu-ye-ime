//! 多音缺读审计与补丁（T-056）。
//!
//! 背景：CC-CEDICT 对多音字常只保留一个读音（如「谁」仅标 shei），而输入法用户
//! 大量按日常口语音输入（shui→谁、shou→熟），导致候选缺失。本模块提供：
//!
//! - `strip_tone_letters`：把《通用规范汉字表读音》(kTGHZ2013) 的预组声调字符
//!   （hǎo、shuí、nǚ）映射回无调字母，与 CEDICT 侧的 `normalize_pinyin` 对齐；
//! - `load_standard_readings`：解析 kTGHZ2013 行（`U+8C01: shéi,shuí  # 谁`）；
//! - `polyphone_gaps`：对照 CEDICT 单字读音集，输出规范读音中词库缺失的项；
//! - `PatchEntry` / `load_patch_table`：人工把关的补丁表（`data/patches/polyphone.tsv`），
//!   构建期由 `import --polyphone` 应用（见 `import::build_real_dictionary`）。
//!
//! 补丁应用规则（构建期，保证确定性且不污染）：音节必须通过标准全拼表校验；
//! 字必须已存在于词库（不引入规范表外汉字）；同字同音已存在时跳过；
//! 词频取该字现有最高词频（不人为改写排序尺度）。

use std::collections::{HashMap, HashSet};
use std::path::Path;

use zhu_ye_core::pinyin::SyllableTable;

/// 预组声调字符 -> 基础字母（kTGHZ2013 字母调符；ü 按输入法惯例写作 v）。
fn tone_char_to_base(ch: char) -> Option<char> {
    Some(match ch {
        'á' | 'à' | 'ǎ' | 'ā' => 'a',
        'é' | 'è' | 'ě' | 'ē' | 'ê' => 'e',
        'í' | 'ì' | 'ǐ' | 'ī' => 'i',
        'ó' | 'ò' | 'ǒ' | 'ō' => 'o',
        'ú' | 'ù' | 'ǔ' | 'ū' => 'u',
        'ǘ' | 'ǚ' | 'ǜ' | 'ǖ' | 'ü' => 'v',
        'ń' | 'ň' | 'ǹ' => 'n',
        'ḿ' => 'm',
        _ => return None,
    })
}

/// 组合式变音符（如 `ê̌` 的 U+030C、`m̀` 的 U+0300）直接删除。
fn is_combining_tone(ch: char) -> bool {
    matches!(ch, '\u{0300}'..='\u{030F}')
}

/// 把字母调符拼音归一化为无调 ASCII 拼音（与 CEDICT 侧 `normalize_pinyin` 对齐）。
///
/// `hǎo` -> `hao`；`shuí` -> `shui`；`nǚ` -> `nv`；`ǹg` -> `ng`（能否入库由
/// 标准音节表校验决定）；含字母调符之外的非 ASCII 拼音字符时原样保留并由调用方
/// 决定拒绝（如小数点、注音符号等不应出现）。
pub fn strip_tone_letters(marked: &str) -> String {
    let mut output = String::new();
    for ch in marked.chars() {
        if let Some(base) = tone_char_to_base(ch) {
            output.push(base);
        } else if is_combining_tone(ch) {
            // 组合变音符删除
        } else {
            output.push(ch);
        }
    }
    output
}

/// 一条规范读音条目：汉字与其规范读音（无调全拼，去重保序）。
pub struct StandardReading {
    pub character: String,
    pub readings: Vec<String>,
}

/// 解析 kTGHZ2013：`U+8C01: shéi,shuí  # 谁`。
///
/// 无法定位汉字或没有可解析读音的行跳过（某些附加区字符行无读音）。
pub fn load_standard_readings(text: &str) -> Vec<StandardReading> {
    let mut out = Vec::new();
    for line in text.lines() {
        let Some(hash_pos) = line.find('#') else {
            continue;
        };
        let head = line[..hash_pos].trim();
        let character = line[hash_pos + 1..].trim().chars().next().map(String::from);
        let Some(character) = character else { continue };
        let Some(colon) = head.find(':') else {
            continue;
        };
        let mut readings = Vec::new();
        for part in head[colon + 1..].split(',') {
            let normalized = strip_tone_letters(part.trim());
            if !normalized.is_empty()
                && normalized.chars().all(|c| c.is_ascii_alphabetic())
                && !readings.contains(&normalized)
            {
                readings.push(normalized);
            }
        }
        if readings.is_empty() {
            continue;
        }
        out.push(StandardReading {
            character,
            readings,
        });
    }
    out
}

/// 一条缺读记录：规范读音存在而词库该字读音集合中缺失的读音。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolyphoneGap {
    pub character: String,
    /// 缺失读音（无调全拼）。
    pub missing: String,
    /// 词库中该字已有的全部读音（无调全拼，排序拼接展示）。
    pub existing: Vec<String>,
}

/// 对照 CEDICT 单字读音集与规范读音，输出词库缺失的读音（按给定顺序）。
///
/// `entry_readings`：字 -> 已有无调读音集合（来自 CEDICT 解析的同口径结果）。
pub fn polyphone_gaps(
    entry_readings: &HashMap<String, HashSet<String>>,
    standard: &[StandardReading],
) -> Vec<PolyphoneGap> {
    let mut gaps = Vec::new();
    for reading in standard {
        let Some(have) = entry_readings.get(&reading.character) else {
            continue; // 词库无此字（规范表外的生僻/非常用字形），不参与
        };
        for missing in &reading.readings {
            if !have.contains(missing) {
                gaps.push(PolyphoneGap {
                    character: reading.character.clone(),
                    missing: missing.clone(),
                    existing: {
                        let mut list: Vec<String> = have.iter().cloned().collect();
                        list.sort();
                        list
                    },
                });
            }
        }
    }
    gaps
}

/// 补丁表条目：`字<TAB>读音<TAB>备注`，读音为无调全拼。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchEntry {
    pub character: String,
    pub pinyin: String,
}

/// 解析补丁表 TSV（`data/patches/polyphone.tsv`）。
///
/// 规则：# 开头为注释；每行两列（字、读音）至少满足；读音必须通过标准全拼表
/// 校验，否则整行拒绝（防脏数据进入构建）；列数不足或汉字/读音为空的行跳过。
pub fn load_patch_table(path: &Path) -> Result<Vec<PatchEntry>, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| format!("读取补丁表失败 {}: {error}", path.display()))?;
    parse_patch_table(&text)
}

/// 解析补丁表文本（供测试直接调用，避免依赖磁盘路径）。
pub fn parse_patch_table(text: &str) -> Result<Vec<PatchEntry>, String> {
    let table = SyllableTable::standard();
    let mut entries = Vec::new();
    let mut rejected = Vec::new();
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut columns = line.split('\t');
        let Some(character) = columns.next().map(str::trim).filter(|s| !s.is_empty()) else {
            continue;
        };
        let Some(pinyin) = columns.next().map(str::trim).filter(|s| !s.is_empty()) else {
            continue;
        };
        if character.chars().count() != 1 {
            rejected.push(format!("第 {} 行：字必须是单个汉字", index + 1));
            continue;
        }
        if !table.is_complete_syllable(pinyin) {
            rejected.push(format!(
                "第 {} 行：读音 `{pinyin}` 不在标准全拼表中（拒绝）",
                index + 1
            ));
            continue;
        }
        entries.push(PatchEntry {
            character: character.to_owned(),
            pinyin: pinyin.to_owned(),
        });
    }
    if !rejected.is_empty() {
        return Err(format!("补丁表存在无效行：{}", rejected.join("；")));
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import::normalize_pinyin;

    #[test]
    fn 预组声调字符逐一对齐无调字母() {
        assert_eq!(strip_tone_letters("hǎo"), "hao");
        assert_eq!(strip_tone_letters("shuí"), "shui");
        assert_eq!(strip_tone_letters("shéi"), "shei");
        assert_eq!(strip_tone_letters("shuō"), "shuo");
        assert_eq!(strip_tone_letters("nǚ"), "nv");
        assert_eq!(strip_tone_letters("ǹg"), "ng");
        assert_eq!(strip_tone_letters("lǜ"), "lv");
        assert_eq!(strip_tone_letters("wǔ"), "wu");
    }

    #[test]
    fn 与cedict侧数字调归一化口径一致() {
        // kTGHZ 字母调符 vs CEDICT 数字调，归一化后应相同
        assert_eq!(
            strip_tone_letters("shuí"),
            normalize_pinyin("shui2").unwrap()
        );
        assert_eq!(strip_tone_letters("nǚ"), normalize_pinyin("nu:3").unwrap());
        assert_eq!(strip_tone_letters("lǜ"), normalize_pinyin("lü4").unwrap());
        assert_eq!(strip_tone_letters("hǎo"), normalize_pinyin("hao3").unwrap());
    }

    #[test]
    fn 规范读音表解析单行() {
        let readings = load_standard_readings("U+8C01: shéi,shuí  # 谁");
        assert_eq!(readings.len(), 1);
        assert_eq!(readings[0].character, "谁");
        assert_eq!(readings[0].readings, vec!["shei", "shui"]);
    }

    #[test]
    fn 规范读音表忽略无读音行() {
        let text = "U+3447: zhòu  # 㑇\nU+8C01: shéi,shuí  # 谁\n# 注释行";
        let readings = load_standard_readings(text);
        assert_eq!(readings.len(), 2);
    }

    #[test]
    fn 缺读审计谁案例() {
        let mut have = HashMap::new();
        have.insert("谁".to_owned(), HashSet::from(["shei".to_owned()]));
        let std = load_standard_readings("U+8C01: shéi,shuí  # 谁");
        let gaps = polyphone_gaps(&have, &std);
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].character, "谁");
        assert_eq!(gaps[0].missing, "shui");
        assert_eq!(gaps[0].existing, vec!["shei"]);
    }

    #[test]
    fn 缺读审计忽略词库外字() {
        let have = HashMap::new(); // 词库无任何字
        let std = load_standard_readings("U+8C01: shéi,shuí  # 谁");
        assert!(polyphone_gaps(&have, &std).is_empty());
    }

    #[test]
    fn 补丁表解析与应用面样例() {
        let text = "# 多音补丁\n谁\tshui\t备注\n熟\tshou\t备注\n";
        let entries = parse_patch_table(text).unwrap();
        assert_eq!(
            entries,
            vec![
                PatchEntry {
                    character: "谁".into(),
                    pinyin: "shui".into()
                },
                PatchEntry {
                    character: "熟".into(),
                    pinyin: "shou".into()
                },
            ]
        );
    }

    #[test]
    fn 补丁表拒绝非标准音节() {
        let text = "嗯\tng\t鼻音不在标准表\n谁\tshui\tok\n";
        assert!(parse_patch_table(text).is_err());
    }
}
