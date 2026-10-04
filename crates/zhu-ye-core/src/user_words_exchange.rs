//! 用户词表导入导出（T-088，FR-048 用户词表导入导出）。
//!
//! 磁盘上的 `user_words.json`（`UserDictStore` 的 `FileFormat`）是内部格式，字段含
//! `last_used` 等学习元数据；导入导出使用独立的 v1 交换 Schema（方案设计 §14.5.1）：
//!
//! ```json
//! {
//!   "format": "zhu-ye-user-words",
//!   "version": 1,
//!   "items": [ { "pinyin": "zhuye", "word": "竹叶", "freq": 3 } ],
//!   "exported_at": 1759420800
//! }
//! ```
//!
//! 口径：
//! - 导出 = 按 `words_sorted` 序（词频降序、同频按词与拼音稳定定序）写出；
//! - 导入 = `format`/`version` 须精确匹配，否则整体拒绝（不触碰磁盘）；合并在内存
//!   词典上执行（同 `(word, pinyin)` 取词频较大值，沿用既有学习合并口径；新增条目
//!   `last_used` 记 0）；调用方校验通过后才 `UserDictStore::save` 原子落盘
//!   （先校验后写，对齐 FR-042）。

use serde::{Deserialize, Serialize};

use crate::user_dict::UserDictionary;
use crate::user_store::unix_now;

/// 交换格式标识；导入时须精确匹配。
pub const USER_WORDS_EXCHANGE_FORMAT: &str = "zhu-ye-user-words";
/// 交换格式版本。
pub const USER_WORDS_EXCHANGE_VERSION: u32 = 1;

/// 交换格式中的一条用户词（不含内部学习元数据）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExchangeItem {
    /// 全拼（缺省空串；合并时按非法条目忽略）。
    #[serde(default)]
    pub pinyin: String,
    /// 词文本（缺省空串；合并时按非法条目忽略）。
    #[serde(default)]
    pub word: String,
    /// 词频（累计选择次数；缺省 0，合并时按非法条目忽略）。
    #[serde(default)]
    pub freq: u64,
}

/// v1 交换文件结构。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExchangeFile {
    /// 格式标识（须等于 [`USER_WORDS_EXCHANGE_FORMAT`]）。
    pub format: String,
    /// 版本（须等于 [`USER_WORDS_EXCHANGE_VERSION`]）。
    pub version: u32,
    /// 词条。
    pub items: Vec<ExchangeItem>,
    /// 导出时间（Unix 秒）。
    pub exported_at: u64,
}

/// 从用户词库生成交换 JSON 文本（带缩进，便于人工查看与 diff）。
pub fn export_user_words(dictionary: &UserDictionary) -> Result<String, String> {
    let file = ExchangeFile {
        format: USER_WORDS_EXCHANGE_FORMAT.to_owned(),
        version: USER_WORDS_EXCHANGE_VERSION,
        items: dictionary
            .words_sorted()
            .into_iter()
            .map(|word| ExchangeItem {
                pinyin: word.pinyin,
                word: word.word,
                freq: word.frequency,
            })
            .collect(),
        exported_at: unix_now(),
    };
    serde_json::to_string_pretty(&file)
        .map_err(|error| format!("序列化用户词交换文件失败: {error}"))
}

/// 解析交换 JSON 文本并校验格式/版本；任一不匹配返回错误（调用方不落盘）。
pub fn parse_exchange_file(text: &str) -> Result<ExchangeFile, String> {
    let file: ExchangeFile = serde_json::from_str(text)
        .map_err(|error| format!("导入文件不是合法的用户词表 JSON: {error}"))?;
    if file.format != USER_WORDS_EXCHANGE_FORMAT {
        return Err(format!(
            "导入文件格式不是“{}”（实际：{}）",
            USER_WORDS_EXCHANGE_FORMAT, file.format
        ));
    }
    if file.version != USER_WORDS_EXCHANGE_VERSION {
        return Err(format!(
            "导入文件版本 {} 不是当前支持的 {}",
            file.version, USER_WORDS_EXCHANGE_VERSION
        ));
    }
    Ok(file)
}

/// 把交换条目合并进本地用户词库：同 `(word, pinyin)` 取词频较大值（保留本地
/// `last_used`），其余新增（`last_used` 记 0）；非法条目忽略。
///
/// 返回新增条目数。只在内存词典上操作，调用方校验通过后才落盘。
pub fn merge_exchange_items(dictionary: &mut UserDictionary, items: &[ExchangeItem]) -> usize {
    items.iter().fold(0, |added, item| {
        added + usize::from(dictionary.merge_external(&item.word, &item.pinyin, item.freq))
    })
}

#[cfg(test)]
mod tests {
    use super::{
        export_user_words, merge_exchange_items, parse_exchange_file, ExchangeFile, ExchangeItem,
        USER_WORDS_EXCHANGE_FORMAT, USER_WORDS_EXCHANGE_VERSION,
    };
    use crate::user_dict::{UserDictionary, UserWord};
    use serde_json::{json, Value};

    fn dict_with(words: &[(&str, &str, u64)]) -> UserDictionary {
        UserDictionary::from_entries(
            words
                .iter()
                .map(|&(word, pinyin, frequency)| UserWord {
                    word: word.to_owned(),
                    pinyin: pinyin.to_owned(),
                    frequency,
                    last_used: 0,
                })
                .collect::<Vec<_>>(),
        )
    }

    #[test]
    fn 导出可回读且字段齐全() {
        let dict = dict_with(&[("竹叶", "zhuye", 3), ("竹", "zhu", 1)]);
        let text = export_user_words(&dict).unwrap();
        let root: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(root["format"], USER_WORDS_EXCHANGE_FORMAT);
        assert_eq!(root["version"], USER_WORDS_EXCHANGE_VERSION);
        assert!(root["exported_at"].is_u64());
        let items = root["items"].as_array().unwrap();
        // words_sorted 按词频降序。
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["word"], "竹叶");
        assert_eq!(items[0]["freq"], 3);
        assert_eq!(items[1]["word"], "竹");
        let file: ExchangeFile = parse_exchange_file(&text).unwrap();
        assert_eq!(file.items[0].pinyin, "zhuye");
    }

    #[test]
    fn 格式或版本不匹配整体拒绝() {
        let good = export_user_words(&dict_with(&[("竹", "zhu", 1)])).unwrap();
        let wrong_format = good.replace(USER_WORDS_EXCHANGE_FORMAT, "other-format");
        assert!(parse_exchange_file(&wrong_format).is_err());
        let wrong_version = good.replace(
            &format!(r#""version": {USER_WORDS_EXCHANGE_VERSION}"#),
            r#""version": 99"#,
        );
        assert!(parse_exchange_file(&wrong_version).is_err());
        assert!(parse_exchange_file("not json").is_err());
        assert!(parse_exchange_file("{}").is_err(), "缺 format 应拒绝");
    }

    #[test]
    fn 合并同词取较大词频且新增数正确() {
        let mut local = dict_with(&[("竹叶", "zhuye", 3)]);
        let items: Vec<ExchangeItem> = serde_json::from_value(json!([
            {"pinyin": "zhuye", "word": "竹叶", "freq": 5},
            {"pinyin": "zhuye", "word": "竹叶", "freq": 2},
            {"pinyin": "zhu", "word": "竹", "freq": 2}
        ]))
        .unwrap();
        let added = merge_exchange_items(&mut local, &items);
        assert_eq!(local.len(), 2, "同键并一条，总共 2 条");
        assert_eq!(added, 1, "只有“竹”是新增");
        assert_eq!(local.frequency("竹叶", "zhuye"), 5, "同词取较大词频");
        assert_eq!(local.frequency("竹", "zhu"), 2);
    }

    #[test]
    fn 非法条目被过滤且零新增() {
        let mut local = UserDictionary::new();
        let items: Vec<ExchangeItem> = serde_json::from_value(json!([
            {"pinyin": "", "word": "空拼音", "freq": 1},
            {"pinyin": "w", "word": "", "freq": 1},
            {"pinyin": "w", "word": "零词频", "freq": 0},
            {"pinyin": "ok", "word": "正常", "freq": 1}
        ]))
        .unwrap();
        let added = merge_exchange_items(&mut local, &items);
        assert_eq!(local.len(), 1);
        assert_eq!(added, 1);
    }

    #[test]
    fn 导入保留本地已学词且同键取本地无损失() {
        // 调用方口径：load 本地词典 → 合并 → 整体保存。
        let mut local = dict_with(&[("竹叶", "zhuye", 2), ("本地词", "bendi", 1)]);
        let items: Vec<ExchangeItem> = serde_json::from_value(json!([
            {"pinyin": "zhuye", "word": "竹叶", "freq": 7},
            {"pinyin": "xinci", "word": "新词", "freq": 1}
        ]))
        .unwrap();
        let added = merge_exchange_items(&mut local, &items);
        assert_eq!(added, 1, "新词为新增，竹叶为同键 max");
        assert_eq!(local.frequency("竹叶", "zhuye"), 7);
        assert_eq!(local.frequency("本地词", "bendi"), 1, "本地词保留");
        assert_eq!(local.frequency("新词", "xinci"), 1);
        assert_eq!(local.len(), 3);
    }
}
