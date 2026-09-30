//! 上下文 bigram 模型接口与内存实现。
//!
//! T-006 数据管线会生成语料 bigram 表；候选排序通过 trait 接入，
//! 不依赖具体存储，未来可无缝替换为 mmap 或压缩索引实现。

use std::fmt::Debug;

/// bigram 频率数据源。
pub trait BigramModel: Send + Sync + Debug {
    /// 返回前词 `previous` 后出现 `word` 的语料计数；未收录返回 0。
    fn frequency(&self, previous: &str, word: &str) -> u64;

    /// 返回前词 `previous` 之后的高频后继词（最多 `limit` 条），按频率降序、
    /// 同频按词形字典序（确定性）；前词未收录返回空。
    ///
    /// 默认实现返回空：未提供后继索引的模型退化为"不联想"，
    /// 不影响现有排序语义（T-058）。
    fn successors(&self, _previous: &str, _limit: usize) -> Vec<(String, u64)> {
        Vec::new()
    }
}

/// 空 bigram 模型：任何查询返回 0，排序退化为 unigram + 用户词。
#[derive(Debug, Clone, Copy, Default)]
pub struct EmptyBigramModel;

impl BigramModel for EmptyBigramModel {
    fn frequency(&self, _previous: &str, _word: &str) -> u64 {
        0
    }
}

/// 内存 bigram 表：按前词 -> 后词两级索引，查询无字符串分配。
#[derive(Debug, Clone, Default)]
pub struct InMemoryBigramModel {
    pairs: std::collections::HashMap<String, std::collections::HashMap<String, u64>>,
}

impl InMemoryBigramModel {
    /// 创建空 bigram 表。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 插入或累加一组前词/后词共现计数。
    pub fn insert(&mut self, previous: impl Into<String>, word: impl Into<String>, frequency: u64) {
        self.pairs
            .entry(previous.into())
            .or_default()
            .insert(word.into(), frequency);
    }

    /// 已收录前词数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    /// 是否没有任何 bigram 记录。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }
}

impl BigramModel for InMemoryBigramModel {
    fn frequency(&self, previous: &str, word: &str) -> u64 {
        self.pairs
            .get(previous)
            .and_then(|by_word| by_word.get(word))
            .copied()
            .unwrap_or(0)
    }

    fn successors(&self, previous: &str, limit: usize) -> Vec<(String, u64)> {
        let mut items: Vec<(String, u64)> = self
            .pairs
            .get(previous)
            .map(|by_word| {
                by_word
                    .iter()
                    .map(|(word, &frequency)| (word.clone(), frequency))
                    .collect()
            })
            .unwrap_or_default();
        items.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        items.truncate(limit);
        items
    }
}

#[cfg(test)]
mod tests {
    use super::{BigramModel, EmptyBigramModel, InMemoryBigramModel};

    #[test]
    fn 内存表按前词与后词命中() {
        let mut model = InMemoryBigramModel::new();
        model.insert("我们", "的", 5000);
        assert_eq!(model.frequency("我们", "的"), 5000);
        assert_eq!(model.frequency("我们", "得"), 0);
        assert_eq!(model.frequency("你们", "的"), 0);
        assert_eq!(model.len(), 1);
    }

    #[test]
    fn 空模型恒返回零() {
        assert_eq!(EmptyBigramModel.frequency("我们", "的"), 0);
    }

    #[test]
    fn 后继按频率降序同频字典序并截取上限() {
        let mut model = InMemoryBigramModel::new();
        model.insert("我们", "的", 5000);
        model.insert("我们", "在", 8000);
        model.insert("我们", "一起", 5000);
        let all = model.successors("我们", 10);
        assert_eq!(
            all,
            vec![
                ("在".to_owned(), 8000),
                ("一起".to_owned(), 5000),
                ("的".to_owned(), 5000),
            ]
        );
        assert_eq!(
            model.successors("我们", 2),
            vec![("在".to_owned(), 8000), ("一起".to_owned(), 5000)]
        );
    }

    #[test]
    fn 空号与未收录前词后继为空() {
        let model = InMemoryBigramModel::new();
        assert!(model.successors("我们", 5).is_empty());
        let mut model = InMemoryBigramModel::new();
        model.insert("我们", "的", 1);
        assert!(model.successors("你们", 5).is_empty());
        assert_eq!(
            EmptyBigramModel.successors("我们", 5),
            Vec::<(String, u64)>::new()
        );
    }
}
