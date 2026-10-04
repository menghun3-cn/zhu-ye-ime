//! 词典接口与内存实现。

/// 词典词条。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictionaryEntry {
    /// 中文词。
    pub word: String,
    /// 全拼（不带声调）。
    pub pinyin: String,
    /// 常见英文译文。
    pub translation: Option<String>,
    /// 静态词频（相对值）。
    pub frequency: u64,
}

impl DictionaryEntry {
    /// 构造词条。
    #[must_use]
    pub fn new(word: impl Into<String>, pinyin: impl Into<String>, frequency: u64) -> Self {
        Self {
            word: word.into(),
            pinyin: pinyin.into(),
            translation: None,
            frequency,
        }
    }

    /// 附加译文。
    #[must_use]
    pub fn with_translation(mut self, translation: impl Into<String>) -> Self {
        self.translation = Some(translation.into());
        self
    }
}

/// 词典查询接口。
///
/// 后续 mmap 二进制词典实现同一接口，算法层不关心物理存储。
pub trait Dictionary: Send + Sync {
    /// 按完整拼音查询词条。
    fn lookup(&self, pinyin: &str) -> Vec<DictionaryEntry>;

    /// 按拼音前缀查询词条（前缀候选/补全用）；实现必须按词频降序返回。
    fn lookup_prefix(&self, pinyin_prefix: &str) -> Vec<DictionaryEntry>;
}

/// 内存词典：用于脚手架、测试与小型演示数据。
#[derive(Debug, Clone, Default)]
pub struct InMemoryDictionary {
    entries: Vec<DictionaryEntry>,
}

impl InMemoryDictionary {
    /// 从词条列表构建。
    #[must_use]
    pub fn from_entries(entries: Vec<DictionaryEntry>) -> Self {
        Self { entries }
    }

    /// 追加词条。
    pub fn push(&mut self, entry: DictionaryEntry) {
        self.entries.push(entry);
    }
}

impl Dictionary for InMemoryDictionary {
    fn lookup(&self, pinyin: &str) -> Vec<DictionaryEntry> {
        let mut found: Vec<DictionaryEntry> = self
            .entries
            .iter()
            .filter(|entry| entry.pinyin == pinyin)
            .cloned()
            .collect();
        found.sort_by_key(|entry| std::cmp::Reverse(entry.frequency));
        found
    }

    fn lookup_prefix(&self, pinyin_prefix: &str) -> Vec<DictionaryEntry> {
        if pinyin_prefix.is_empty() {
            return Vec::new();
        }
        let mut found: Vec<DictionaryEntry> = self
            .entries
            .iter()
            .filter(|entry| entry.pinyin.starts_with(pinyin_prefix))
            .cloned()
            .collect();
        found.sort_by_key(|entry| std::cmp::Reverse(entry.frequency));
        found
    }
}

#[cfg(test)]
mod tests {
    use super::{Dictionary, DictionaryEntry, InMemoryDictionary};

    #[test]
    fn 内存词典按拼音查询并降序排列() {
        let dict = InMemoryDictionary::from_entries(vec![
            DictionaryEntry::new("泥好", "nihao", 1),
            DictionaryEntry::new("你好", "nihao", 100).with_translation("hello"),
        ]);
        let found = dict.lookup("nihao");
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].word, "你好");
        assert_eq!(found[0].translation.as_deref(), Some("hello"));
        assert!(dict.lookup("haha").is_empty());
    }

    #[test]
    fn 内存词典前缀查询按词频降序且空前缀为空() {
        let dict = InMemoryDictionary::from_entries(vec![
            DictionaryEntry::new("泥好", "nihao", 1),
            DictionaryEntry::new("你好", "nihao", 100),
            DictionaryEntry::new("你", "ni", 65),
        ]);
        let found = dict.lookup_prefix("nih");
        let words: Vec<&str> = found.iter().map(|e| e.word.as_str()).collect();
        assert_eq!(words, vec!["你好", "泥好"]);
        assert!(found.iter().all(|e| e.pinyin.starts_with("nih")));
        assert!(dict.lookup_prefix("zzz").is_empty());
        assert!(dict.lookup_prefix("").is_empty());
    }
}
