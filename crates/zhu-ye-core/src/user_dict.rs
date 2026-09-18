//! 用户词学习（内存模型）。
//!
//! M3 里程碑增加本地 JSON 持久化、损坏自动恢复与配置接口。

use std::collections::HashMap;

/// 用户词条。
#[derive(Debug, Clone, PartialEq, Eq)]
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

    /// 返回全部用户词条（按词频降序）。
    #[must_use]
    pub fn words_sorted(&self) -> Vec<UserWord> {
        let mut words: Vec<UserWord> = self.entries.values().cloned().collect();
        words.sort_by_key(|entry| std::cmp::Reverse(entry.frequency));
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
    use super::UserDictionary;

    #[test]
    fn 记录选择后词频可见() {
        let mut u = UserDictionary::new();
        u.record_selection("竹叶", "zhuye", 1);
        u.record_selection("竹叶", "zhuye", 2);
        assert_eq!(u.frequency("竹叶", "zhuye"), 2);
        assert_eq!(u.words_sorted()[0].last_used, 2);
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
}
