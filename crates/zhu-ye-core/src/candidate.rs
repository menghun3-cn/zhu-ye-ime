//! 候选类型与排序。
//!
//! 底层 `CandidateSorter` 只负责确定性基础排序；候选排序模型（trait）
//! 负责把静态词典分、上下文 bigram 与用户词频加权成最终排序分。

use std::sync::Arc;

use crate::bigram::{BigramModel, EmptyBigramModel};
use crate::user_dict::UserDictionary;

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
    /// 排序权重分，越大越优先。
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

/// 候选排序器：按排序分降序、文本升序排列。
/// 排序必须确定，相同输入产生相同顺序。
#[derive(Debug, Clone, Default)]
pub struct CandidateSorter;

impl CandidateSorter {
    /// 对候选按排序分降序排列。
    #[must_use]
    pub fn sort(mut candidates: Vec<Candidate>) -> Vec<Candidate> {
        candidates.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.text.cmp(&b.text)));
        candidates
    }
}

/// 静态排序模型权重配置。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankingConfig {
    /// 静态词典分权重。
    pub unigram_weight: u64,
    /// bigram 计数权重。
    pub bigram_weight: u64,
    /// 单个 bigram 计数的参与上限，防止高频词过度主导。
    pub bigram_frequency_cap: u64,
    /// 用户词频权重。
    pub user_weight: u64,
    /// 单个用户词的参与上限。
    pub user_frequency_cap: u64,
}

impl Default for RankingConfig {
    fn default() -> Self {
        Self {
            unigram_weight: 1,
            bigram_weight: 16,
            bigram_frequency_cap: 100_000,
            user_weight: 48,
            user_frequency_cap: 10_000,
        }
    }
}

/// 排序上下文：前词与当前用户词库。
#[derive(Debug, Clone, Copy)]
pub struct RankingContext<'a> {
    previous_word: Option<&'a str>,
    user_dictionary: &'a UserDictionary,
}

impl<'a> RankingContext<'a> {
    /// 创建排序上下文。
    #[must_use]
    pub fn new(previous_word: Option<&'a str>, user_dictionary: &'a UserDictionary) -> Self {
        Self {
            previous_word,
            user_dictionary,
        }
    }
}

/// 候选排序模型：负责最终排位，输入引擎通过 trait 注入。
pub trait RankingModel: Send + Sync {
    /// 对候选重新计分并排序；必须确定且不修改上下文。
    fn rank(&self, candidates: Vec<Candidate>, context: &RankingContext<'_>) -> Vec<Candidate>;
}

/// 静态候选排序模型：`unigram + bigram + 用户词频`。
///
/// 公式（全部为整数饱和运算，保证跨平台一致）：
/// `最终分 = 静态分 × unigram权重 + bigram计数(min cap) × bigram权重 + 用户词频(min cap) × 用户权重`
#[derive(Debug)]
pub struct StaticRankingModel {
    config: RankingConfig,
    bigram: Arc<dyn BigramModel>,
}

impl StaticRankingModel {
    /// 创建静态排序模型。
    #[must_use]
    pub fn new(config: RankingConfig, bigram: Arc<dyn BigramModel>) -> Self {
        Self { config, bigram }
    }

    /// 当前权重配置。
    #[must_use]
    pub fn config(&self) -> &RankingConfig {
        &self.config
    }
}

impl Default for StaticRankingModel {
    fn default() -> Self {
        Self::new(RankingConfig::default(), Arc::new(EmptyBigramModel))
    }
}

impl RankingModel for StaticRankingModel {
    fn rank(&self, candidates: Vec<Candidate>, context: &RankingContext<'_>) -> Vec<Candidate> {
        let config = &self.config;
        let ranked = candidates
            .into_iter()
            .map(|mut candidate| {
                let static_part = u64::try_from(candidate.score.max(0))
                    .unwrap_or(u64::MAX)
                    .saturating_mul(config.unigram_weight);

                let bigram_part = context
                    .previous_word
                    .map_or(0, |previous| {
                        self.bigram
                            .frequency(previous, &candidate.text)
                            .min(config.bigram_frequency_cap)
                    })
                    .saturating_mul(config.bigram_weight);

                let user_frequency = context.user_dictionary.frequency_by_word(&candidate.text);
                if user_frequency > 0 {
                    candidate.source = CandidateSource::User;
                }
                let user_part = user_frequency
                    .min(config.user_frequency_cap)
                    .saturating_mul(config.user_weight);

                let total = static_part
                    .saturating_add(bigram_part)
                    .saturating_add(user_part)
                    .min(i64::MAX as u64);
                candidate.score = i64::try_from(total).expect("分数已夹紧到 i64 范围");
                candidate
            })
            .collect::<Vec<_>>();

        CandidateSorter::sort(ranked)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::bigram::InMemoryBigramModel;
    use crate::candidate::{
        Candidate, CandidateSorter, RankingConfig, RankingContext, RankingModel, StaticRankingModel,
    };
    use crate::user_dict::UserDictionary;

    fn rank_with(
        model: &StaticRankingModel,
        previous: Option<&str>,
        user: &UserDictionary,
        candidates: Vec<Candidate>,
    ) -> Vec<Candidate> {
        model.rank(candidates, &RankingContext::new(previous, user))
    }

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

    #[test]
    fn 静态分高者优先() {
        let user = UserDictionary::new();
        let model = StaticRankingModel::default();
        let sorted = rank_with(
            &model,
            None,
            &user,
            vec![Candidate::new("低", 1), Candidate::new("高", 10)],
        );
        assert_eq!(sorted[0].text, "高");
    }

    #[test]
    fn bigram命中可提升低频词() {
        let mut bigram = InMemoryBigramModel::new();
        bigram.insert("我们", "的", 5_000);
        let model = StaticRankingModel::new(RankingConfig::default(), Arc::new(bigram));
        let user = UserDictionary::new();
        let sorted = rank_with(
            &model,
            Some("我们"),
            &user,
            vec![Candidate::new("得", 200), Candidate::new("的", 100)],
        );
        assert_eq!(sorted[0].text, "的");
    }

    #[test]
    fn bigram未命中时回退静态分() {
        let model = StaticRankingModel::default();
        let user = UserDictionary::new();
        let sorted = rank_with(
            &model,
            Some("我们"),
            &user,
            vec![Candidate::new("得", 200), Candidate::new("的", 100)],
        );
        assert_eq!(sorted[0].text, "得");
    }

    #[test]
    fn 用户词频可提升候选并标记来源() {
        let mut user = UserDictionary::new();
        for now in 1..=10 {
            user.record_selection("爱", "ai", now);
        }
        let model = StaticRankingModel::default();
        let sorted = rank_with(
            &model,
            None,
            &user,
            vec![Candidate::new("暗", 300), Candidate::new("爱", 10)],
        );
        assert_eq!(sorted[0].text, "爱");
        assert_eq!(sorted[0].source, crate::candidate::CandidateSource::User);
    }

    #[test]
    fn 同一输入两次排序结果一致() {
        let mut bigram = InMemoryBigramModel::new();
        bigram.insert("我们", "的", 9_999);
        let model = StaticRankingModel::new(RankingConfig::default(), Arc::new(bigram));
        let mut user = UserDictionary::new();
        user.record_selection("爱", "ai", 1);

        let candidates = || {
            vec![
                Candidate::new("得", 200),
                Candidate::new("爱", 10),
                Candidate::new("的", 100),
            ]
        };
        let first = rank_with(&model, Some("我们"), &user, candidates());
        let second = rank_with(&model, Some("我们"), &user, candidates());
        assert_eq!(first, second);
    }
}
