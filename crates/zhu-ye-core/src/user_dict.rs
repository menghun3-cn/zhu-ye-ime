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

/// 用户级二元转移词条（T-174）：「前词 → 当前词」的个人转移频次。
///
/// 与全局 bigram（词典统计）互补——它记录的是**该用户**的输入习惯。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserTransition {
    /// 前一个已上屏的词。
    pub previous: String,
    /// 当前词。
    pub word: String,
    /// 当前词的拼音。
    pub pinyin: String,
    /// 该转移的累计发生次数。
    pub frequency: u64,
    /// 最近发生时间（Unix 秒，0 表示未记录）。
    pub last_used: u64,
}

/// 用户级二元转移表容量上限（T-174）。
///
/// 超出后按「频次最低 + 最久未用」淘汰，防止文件无限增长。
pub const USER_TRANSITION_CAP: usize = 2_000;

/// 用户词库：选择即记忆，删除/重置接口先行到位。
#[derive(Debug, Clone, Default)]
pub struct UserDictionary {
    entries: HashMap<(String, String), UserWord>,
    transitions: HashMap<(String, String), UserTransition>,
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

    /// 从条目与转移集合重建词库（T-174，存储层读取 v2 文件用）。
    ///
    /// 非法条目（空文本/空拼音/零频次）与非法转移（空前词/空词/空拼音/零频次）
    /// 一律丢弃。
    #[must_use]
    pub fn from_entries_and_transitions(
        entries: impl IntoIterator<Item = UserWord>,
        transitions: impl IntoIterator<Item = UserTransition>,
    ) -> Self {
        let mut dictionary = Self::from_entries(entries);
        for entry in transitions {
            if entry.previous.is_empty()
                || entry.word.is_empty()
                || entry.pinyin.is_empty()
                || entry.frequency == 0
            {
                continue;
            }
            let key = (entry.previous.clone(), entry.word.clone());
            dictionary.transitions.insert(key, entry);
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
        self.transitions.clear();
    }

    /// 按拼音查用户词条（T-173，第 1 层用户词独立候选组的查询入口）。
    ///
    /// 返回按频次降序、同频按词文本稳定定序的词条——**不掺入词典频次**，
    /// 调用方据此把用户词作为独立候选组前置（组内只按个人偏好排）。
    #[must_use]
    pub fn words_by_pinyin(&self, pinyin: &str) -> Vec<UserWord> {
        if pinyin.is_empty() {
            return Vec::new();
        }
        let mut found: Vec<UserWord> = self
            .entries
            .values()
            .filter(|entry| entry.pinyin == pinyin)
            .cloned()
            .collect();
        found.sort_by(|a, b| {
            b.frequency
                .cmp(&a.frequency)
                .then_with(|| a.word.cmp(&b.word))
        });
        found
    }

    /// 记录一次「前词 → 当前词」的个人转移（T-174，第 2 层用户级二元语法）。
    ///
    /// 前词或当前词为空时忽略（首词无前词，不构成转移）。超出容量上限时
    /// 按「频次最低 + 最久未用」淘汰一条。
    pub fn record_transition(&mut self, previous: &str, word: &str, pinyin: &str, now: u64) {
        if previous.is_empty() || word.is_empty() || pinyin.is_empty() {
            return;
        }
        let key = (previous.to_owned(), word.to_owned());
        let entry = self.transitions.entry(key).or_insert(UserTransition {
            previous: previous.to_owned(),
            word: word.to_owned(),
            pinyin: pinyin.to_owned(),
            frequency: 0,
            last_used: 0,
        });
        entry.pinyin = pinyin.to_owned();
        entry.frequency += 1;
        entry.last_used = now;
        self.evict_transitions_if_needed();
    }

    /// 查询「前词 → 当前词」的个人转移频次；未记录返回 0（T-174）。
    #[must_use]
    pub fn transition_frequency(&self, previous: &str, word: &str) -> u64 {
        if previous.is_empty() || word.is_empty() {
            return 0;
        }
        self.transitions
            .get(&(previous.to_owned(), word.to_owned()))
            .map_or(0, |entry| entry.frequency)
    }

    /// 转移表超出容量上限时淘汰「频次最低 + 最久未用」的一条。
    fn evict_transitions_if_needed(&mut self) {
        while self.transitions.len() > USER_TRANSITION_CAP {
            let victim = self
                .transitions
                .values()
                .min_by(|a, b| {
                    a.frequency
                        .cmp(&b.frequency)
                        .then_with(|| a.last_used.cmp(&b.last_used))
                        .then_with(|| a.word.cmp(&b.word))
                })
                .map(|entry| (entry.previous.clone(), entry.word.clone()));
            let Some(key) = victim else {
                break;
            };
            self.transitions.remove(&key);
        }
    }

    /// 全部用户级转移词条（按频次降序，同频按前词/词稳定定序）。
    #[must_use]
    pub fn transitions_sorted(&self) -> Vec<UserTransition> {
        let mut all: Vec<UserTransition> = self.transitions.values().cloned().collect();
        all.sort_by(|a, b| {
            b.frequency
                .cmp(&a.frequency)
                .then_with(|| a.previous.cmp(&b.previous))
                .then_with(|| a.word.cmp(&b.word))
        });
        all
    }

    /// 用户级转移条数。
    #[must_use]
    pub fn transition_count(&self) -> usize {
        self.transitions.len()
    }

    /// 合并一条外部词频条目（导入口径，T-088 / FR-048"同拼音同词频取 max"）：
    /// 键已存在时词频取本地与外部较大值（保留本地 `last_used`），否则新增
    /// （`last_used` 记 0）。非法条目（空词/空拼音/零词频）忽略。
    ///
    /// 返回 `true` 表示键为新增。调用方（设置窗口导入）校验通过后才落盘。
    pub fn merge_external(&mut self, word: &str, pinyin: &str, frequency: u64) -> bool {
        if word.is_empty() || pinyin.is_empty() || frequency == 0 {
            return false;
        }
        let key = (word.to_owned(), pinyin.to_owned());
        match self.entries.get_mut(&key) {
            Some(entry) => {
                entry.frequency = entry.frequency.max(frequency);
                false
            }
            None => {
                self.entries.insert(
                    key,
                    UserWord {
                        word: word.to_owned(),
                        pinyin: pinyin.to_owned(),
                        frequency,
                        last_used: 0,
                    },
                );
                true
            }
        }
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
    use super::{UserDictionary, UserTransition, UserWord, USER_TRANSITION_CAP};

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

    /// T-173 第 1 层：按拼音查词条，按频次降序、同频按词文本稳定定序。
    #[test]
    fn 按拼音查词条按频次降序() {
        let mut u = UserDictionary::new();
        u.record_selection("逆势", "nishi", 1);
        u.record_selection("你是", "nishi", 1);
        u.record_selection("逆势", "nishi", 2);
        u.record_selection("别的", "other", 1);
        let words = u.words_by_pinyin("nishi");
        assert_eq!(words.len(), 2, "只返回该拼音的词条");
        assert_eq!(words[0].word, "逆势", "频次高的在前");
        assert_eq!(words[1].word, "你是");
        assert!(u.words_by_pinyin("不存在").is_empty());
        assert!(u.words_by_pinyin("").is_empty());
    }

    /// T-174 第 2 层：记录与查询「前词 → 当前词」转移。
    #[test]
    fn 记录与查询二元转移() {
        let mut u = UserDictionary::new();
        u.record_transition("我说", "你好", "nihao", 100);
        u.record_transition("我说", "你好", "nihao", 101);
        assert_eq!(u.transition_frequency("我说", "你好"), 2);
        assert_eq!(u.transition_frequency("别人", "你好"), 0);
        assert_eq!(u.transition_count(), 1);
    }

    /// T-174：空前词不构成转移（首词无前词）。
    #[test]
    fn 空前词不记转移() {
        let mut u = UserDictionary::new();
        u.record_transition("", "你好", "nihao", 100);
        assert_eq!(u.transition_count(), 0);
    }

    /// T-174：转移表超容量时淘汰「频次最低 + 最久未用」的一条。
    #[test]
    fn 转移表超容量淘汰最弱项() {
        let mut u = UserDictionary::new();
        for index in 0..=USER_TRANSITION_CAP {
            u.record_transition("前", &format!("词{index}"), "py", index as u64);
        }
        assert_eq!(
            u.transition_count(),
            USER_TRANSITION_CAP,
            "超出上限后应淘汰到上限"
        );
        assert_eq!(
            u.transition_frequency("前", "词0"),
            0,
            "频次最低且最久未用的应被淘汰"
        );
    }

    /// T-174：重建词库时过滤非法转移条目。
    #[test]
    fn 重建词库过滤非法转移() {
        let u = UserDictionary::from_entries_and_transitions(
            vec![UserWord {
                word: "你好".into(),
                pinyin: "nihao".into(),
                frequency: 3,
                last_used: 1,
            }],
            vec![
                UserTransition {
                    previous: "我说".into(),
                    word: "你好".into(),
                    pinyin: "nihao".into(),
                    frequency: 2,
                    last_used: 1,
                },
                UserTransition {
                    previous: String::new(),
                    word: "坏".into(),
                    pinyin: "py".into(),
                    frequency: 5,
                    last_used: 1,
                },
            ],
        );
        assert_eq!(u.transition_count(), 1, "空前词的转移被丢弃");
        assert_eq!(u.transition_frequency("我说", "你好"), 2);
    }

    /// T-174：重置同时清空词条与转移。
    #[test]
    fn 重置清空转移表() {
        let mut u = UserDictionary::new();
        u.record_selection("你好", "nihao", 1);
        u.record_transition("我说", "你好", "nihao", 1);
        u.reset();
        assert!(u.is_empty());
        assert_eq!(u.transition_count(), 0);
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

    #[test]
    fn 外部合并同键取最大词频并保留本地学习时间() {
        let mut u = UserDictionary::new();
        u.record_selection("竹叶", "zhuye", 1_700_000_000);
        // 外部词频更低 → 保持本地 1。
        assert!(!u.merge_external("竹叶", "zhuye", 0));
        assert!(!u.merge_external("竹叶", "zhuye", 0), "零词频忽略");
        assert_eq!(u.frequency("竹叶", "zhuye"), 1);
        assert_eq!(
            u.words_sorted()[0].last_used,
            1_700_000_000,
            "既有条目的学习时间不被外部合并覆盖"
        );
        // 外部词频更高 → 取外部值。
        assert!(!u.merge_external("竹叶", "zhuye", 9));
        assert_eq!(u.frequency("竹叶", "zhuye"), 9);
        // 新键 → 新增。
        assert!(u.merge_external("新词", "xinci", 2));
        assert_eq!(u.frequency("新词", "xinci"), 2);
        // 非法条目丢弃。
        assert!(!u.merge_external("", "kong", 1));
        assert!(!u.merge_external("无拼音", "", 1));
        assert_eq!(u.len(), 2);
    }
}
