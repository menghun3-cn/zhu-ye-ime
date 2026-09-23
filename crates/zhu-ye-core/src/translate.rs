//! 本地翻译接口与内存实现。

use std::collections::HashMap;

/// 翻译方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslationDirection {
    /// 中文到英文。
    ZhToEn,
    /// 英文到中文。
    EnToZh,
}

/// 翻译接口：后续由本地二进制词典或者 AI 服务实现。
pub trait Translator: Send + Sync {
    /// 中文到英文翻译。
    fn translate(&self, text: &str, direction: TranslationDirection) -> Option<String>;

    /// 便捷方法：中文到英文。
    fn zh_to_en(&self, text: &str) -> Option<String> {
        self.translate(text, TranslationDirection::ZhToEn)
    }

    /// 便捷方法：英文到中文。
    fn en_to_zh(&self, text: &str) -> Option<String> {
        self.translate(text, TranslationDirection::EnToZh)
    }
}

/// 小型内存翻译表，用于脚手架与演示。
#[derive(Debug, Clone, Default)]
pub struct InMemoryTranslator {
    zh_to_en: HashMap<String, String>,
    en_to_zh: HashMap<String, String>,
}

impl InMemoryTranslator {
    /// 构造空翻译表。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一对翻译。
    pub fn insert(&mut self, zh: impl Into<String>, en: impl Into<String>) {
        let zh = zh.into();
        let en = en.into();
        self.zh_to_en.insert(zh.clone(), en.clone());
        self.en_to_zh.insert(en, zh);
    }
}

impl Translator for InMemoryTranslator {
    fn translate(&self, text: &str, direction: TranslationDirection) -> Option<String> {
        match direction {
            TranslationDirection::ZhToEn => self.zh_to_en.get(text).cloned(),
            TranslationDirection::EnToZh => self.en_to_zh.get(text).cloned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{InMemoryTranslator, TranslationDirection, Translator};

    #[test]
    fn 记忆翻译器支持双向查询() {
        let mut t = InMemoryTranslator::new();
        t.insert("你好", "hello");
        assert_eq!(
            t.translate("你好", TranslationDirection::ZhToEn).as_deref(),
            Some("hello")
        );
        assert_eq!(
            t.translate("hello", TranslationDirection::EnToZh)
                .as_deref(),
            Some("你好")
        );
        assert_eq!(t.zh_to_en("不在"), None);
    }
}
