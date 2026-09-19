//! 上下文 bigram 模型接口与内存实现。
//!
//! T-006 数据管线会生成语料 bigram 表；候选排序通过 trait 接入，
//! 不依赖具体存储，未来可无缝替换为 mmap 或压缩索引实现。

use std::fmt::Debug;

/// bigram 频率数据源。
pub trait BigramModel: Send + Sync + Debug {
    /// 返回前词 `previous` 后出现 `word` 的语料计数；未收录返回 0。
    fn frequency(&self, previous: &str, word: &str) -> u64;
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
}
