//! 候选类型与排序。

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CandidateSource {
    /// 静态词典候选。
    Static,
    /// 用户词候选中被提升。
    User,
    /// AI 建议（第一版不启用）。
    Ai,
}

/// 输入法候选。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    /// 上屏文本。
    pub text: String,
    /// 对应译文（若有）。
    pub translation: Option<String>,
    /// 候选中划分数值，越大越优先。
    pub score: i64,
    /// 来源。
    pub source: CandidateSource,
}

impl Candidate {
    /// 创建一个无译文的候选。
    #[must_use]
    pub fn new(text: impl Into<String>, score: i64) -> Self {
        Self {
            text: text.into(),
            translation: None,
            score,
            source: CandidateSource::Static,
        }
    }

    /// 设置译文后返回自身（便于链式构建）。
    #[must_use]
    pub fn with_translation(mut self, translation: impl Into<String>) -> Self {
        self.translation = Some(translation.into());
        self
    }

    /// 设置来源。
    #[must_use]
    pub fn with_source(mut self, source: CandidateSource) -> Self {
        self.source = source;
        self
    }
}

/// 候选排序器：当前按静态分数降序，用户词加分由调用方预处理。
/// 排序必须确定，相同输入产生相同顺序。
#[derive(Debug, Clone, Default)]
pub struct CandidateSorter;

impl CandidateSorter {
    /// 对候选按分数降序排列。
    #[must_use]
    pub fn sort(mut candidates: Vec<Candidate>) -> Vec<Candidate> {
        candidates.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.text.cmp(&b.text)));
        candidates
    }
}

#[cfg(test)]
mod tests {
    use super::{Candidate, CandidateSorter};

    #[test]
    fn 排序确定且高分优先() {
        let candidates = vec![
            Candidate::new("低", 1),
            Candidate::new("高", 10),
            Candidate::new("中", 5),
        ];
        let sorted = CandidateSorter::sort(candidates);
        let texts: Vec<&str> = sorted.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(texts, vec!["高", "中", "低"]);
    }

    #[test]
    fn 分数相同时文本定序() {
        let candidates = vec![Candidate::new("b", 1), Candidate::new("a", 1)];
        let sorted = CandidateSorter::sort(candidates);
        assert_eq!(sorted[0].text, "a");
    }
}
