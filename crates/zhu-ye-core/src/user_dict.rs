//! 用户词学习（内存模型）。
//!
//! 选择即记忆，删除/重置接口先行到位；持久化由 `UserDictStore` 负责，
//! 本模块只维护内存状态与条目的合法性校验。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// 用户词条。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserWord {
    /// 词文本。
    pub word: String,
    /// 全拼。
    pub pinyin: String,
    /// 累计选择次数。
    pub frequency: u64,
    /// 最近选择时间（Unix 秒，0 表示未记录）。
    pub last_used: u64,
}

/// 用户词库：选择即记忆，删除/重置接口先行到位。
#[derive(Debug, Clone, Default)]
pub struct UserDictionary {
    entries: HashMap<(String, String), UserWord>,
}

impl UserDictionary {
    /// 创建空用户词库。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 从条目集合重建词库；空文本、空拼音或零词频的非法条目会被丢弃。
    #[must_use]
    pub fn from_entries(entries: impl IntoIterator<Item = UserWord>) -> Self {
        let mut dictionary = Self::new();
        for entry in entries {
            if entry.word.is_empty() || entry.pinyin.is_empty() || entry.frequency == 0 {
                continue;
            }
            let key = (entry.word.clone(), entry.pinyin.clone());
            dictionary.entries.insert(key, entry);
        }
        dictionary
    }

    /// 记录一次选择，词频加一。
    pub fn record_selection(
        &mut self,
        word: impl Into<String>,
        pinyin: impl Into<String>,
        now: u64,
    ) {
        let word = word.into();
        let pinyin = pinyin.into();
        let key = (word.clone(), pinyin.clone());
        let entry = self.entries.entry(key).or_insert(UserWord {
            word,
            pinyin,
            frequency: 0,
            last_used: 0,
        });
        entry.frequency += 1;
        entry.last_used = now;
    }

    /// 删除一个用户词；不存在时返回 false。
    pub fn delete(&mut self, word: &str, pinyin: &str) -> bool {
        self.entries
            .remove(&(word.to_owned(), pinyin.to_owned()))
            .is_some()
    }

    /// 清空全部用户词。
    pub fn reset(&mut self) {
        self.entries.clear();
    }

    /// 查询用户词频；未记录返回 0。
    #[must_use]
    pub fn frequency(&self, word: &str, pinyin: &str) -> u64 {
        self.entries
            .get(&(word.to_owned(), pinyin.to_owned()))
            .map_or(0, |entry| entry.frequency)
    }

    /// 查询按文本汇总的词频（同一文本跨拼音求和）；未记录返回 0。
    ///
    /// 当前用户词条量较小，直接线性汇总即可满足排序查询；
    /// 词条规模增长后，再按文本建索引。
    #[must_use]
    pub fn frequency_by_word(&self, word: &str) -> u64 {
        self.entries
            .iter()
            .filter_map(|((text, _), entry)| (text == word).then_some(entry.frequency))
            .sum()
    }

    /// 返回全部用户词条（按词频降序，同频按词与拼音稳定定序）。
    #[must_use]
    pub fn words_sorted(&self) -> Vec<UserWord> {
        let mut words: Vec<UserWord> = self.entries.values().cloned().collect();
        words.sort_by(|a, b| {
            b.frequency
                .cmp(&a.frequency)
                .then_with(|| a.word.cmp(&b.word))
                .then_with(|| a.pinyin.cmp(&b.pinyin))
        });
        words
    }

    /// 用户词条数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 用户词库是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{UserDictionary, UserWord};

    #[test]
    fn 记录选择后词频可见() {
        let mut u = UserDictionary::new();
        u.record_selection("竹叶", "zhuye", 1);
        u.record_selection("竹叶", "zhuye", 2);
        assert_eq!(u.frequency("竹叶", "zhuye"), 2);
        assert_eq!(u.words_sorted()[0].last_used, 2);
    }

    #[test]
    fn 按文本汇总跨拼音词频() {
        let mut u = UserDictionary::new();
        u.record_selection("竹叶", "zhuye", 1);
        u.record_selection("竹叶", "zhuye", 2);
        u.record_selection("竹叶", "zhuye_ye", 3);
        assert_eq!(u.frequency_by_word("竹叶"), 3);
        assert_eq!(u.frequency_by_word("竹"), 0);
    }

    #[test]
    fn 删除与重置() {
        let mut u = UserDictionary::new();
        u.record_selection("竹叶", "zhuye", 1);
        assert!(u.delete("竹叶", "zhuye"));
        assert!(!u.delete("竹叶", "zhuye"));
        u.record_selection("竹叶", "zhuye", 1);
        u.reset();
        assert!(u.is_empty());
    }

    #[test]
    fn 重建词库过滤非法条目() {
        let u = UserDictionary::from_entries(vec![
            UserWord {
                word: "竹叶".into(),
                pinyin: "zhuye".into(),
                frequency: 3,
                last_used: 1,
            },
            UserWord {
                word: String::new(),
                pinyin: "kong".into(),
                frequency: 1,
                last_used: 1,
            },
            UserWord {
                word: "无拼音".into(),
                pinyin: String::new(),
                frequency: 1,
                last_used: 1,
            },
            UserWord {
                word: "零词频".into(),
                pinyin: "ling".into(),
                frequency: 0,
                last_used: 1,
            },
        ]);
        assert_eq!(u.len(), 1);
        assert_eq!(u.frequency("竹叶", "zhuye"), 3);
    }

    #[test]
    fn 排序结果确定性稳定() {
        let mut u = UserDictionary::new();
        u.record_selection("竹叶", "zhuye", 1);
        u.record_selection("竹", "zhu", 2);
        u.record_selection("叶子", "yezi", 2);
        let first = u.words_sorted();
        let second = u.words_sorted();
        assert_eq!(first, second);
        assert_eq!(first[0].word, "叶子");
        assert_eq!(first[1].word, "竹");
        assert_eq!(first[2].word, "竹叶");
    }
}
