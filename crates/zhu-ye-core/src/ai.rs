//! AI 能力扩展接口。
//!
//! 第一版只提供离线实现，保证零网络；后续的远程 AI 服务必须实现
//! 同一接口，并且调用方始终以后台任务方式使用，避免阻塞输入热路径。

use crate::translate::TranslationDirection;

/// AI 建议来源。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiSource {
    /// 本地/离线提供。
    Offline,
    /// 远程模型（第一版不可用）。
    Remote,
}

/// AI 建议类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiSuggestionKind {
    /// 候选联想/预判。
    Suggest,
    /// 翻译建议。
    Translate,
    /// 整句润色。
    Polish,
}

/// 输入上下文。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputContext {
    /// 当前拼音串。
    pub pinyin: String,
    /// 最近上屏的若干词。
    pub previous_words: Vec<String>,
}

/// AI 建议。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiSuggestion {
    /// 建议文本。
    pub text: String,
    /// 建议类型。
    pub kind: AiSuggestionKind,
    /// 来源：本地或远程。
    pub source: AiSource,
}

/// AI 服务接口：翻译、润色、联想/预判、建议统一入口。
/// 实现须是 Send + Sync，便于宿主放到后台任务。
pub trait AiService: Send + Sync {
    /// 根据输入上下文生成建议。
    fn suggest(&self, ctx: &InputContext) -> Vec<AiSuggestion>;

    /// 按方向翻译文本；未接入远程/本地模型时返回 None。
    fn translate(&self, _text: &str, _direction: TranslationDirection) -> Option<String> {
        None
    }

    /// 润色文本。
    fn polish(&self, _text: &str) -> Option<String> {
        None
    }
}

/// 第一版离线 AI 服务：不做任何远程请求，只返回确定性的本地结果。
/// 当前不提供智能建议，保持普通输入完全不被 AI 影响。
#[derive(Debug, Clone, Copy, Default)]
pub struct OfflineAiService;

impl AiService for OfflineAiService {
    fn suggest(&self, _ctx: &InputContext) -> Vec<AiSuggestion> {
        Vec::new()
    }

    fn translate(&self, _text: &str, _direction: TranslationDirection) -> Option<String> {
        None
    }

    fn polish(&self, _text: &str) -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{AiService, AiSuggestion, OfflineAiService};
    use crate::translate::TranslationDirection;

    #[test]
    fn 离线服务返回空且可判别类型() {
        let service = OfflineAiService;
        let ctx = super::InputContext {
            pinyin: "nihao".to_owned(),
            previous_words: Vec::new(),
        };
        let suggestions = service.suggest(&ctx);
        assert!(suggestions.is_empty());
        let _typed: Box<dyn AiService> = Box::new(service);
        let _: Vec<AiSuggestion> = suggestions;
        assert_eq!(
            service.translate("你好", TranslationDirection::ZhToEn),
            None
        );
        assert_eq!(
            service.translate("hello", TranslationDirection::EnToZh),
            None
        );
        assert_eq!(service.polish("今天天气很好"), None);
    }
}
