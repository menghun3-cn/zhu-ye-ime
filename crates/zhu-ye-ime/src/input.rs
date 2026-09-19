//! 输入状态机：纯 Rust，不依赖 Windows，负责组合串维护、候选生成与提交。
//!
//! TSF 适配层只负责把按键翻译成这里的调用，并把返回的提交文本写入文档；
//! 候选排序、拼音切分与词典查询全部复用 `zhu-ye-core`，保证行为可单测。

use std::collections::HashMap;
use std::sync::Arc;

use zhu_ye_core::candidate::{Candidate, CandidateSorter};
use zhu_ye_core::dict::{Dictionary, DictionaryEntry, InMemoryDictionary};
use zhu_ye_core::pinyin::{segment_all, SyllableTable};

/// 输入模式。T-013 接入 Shift 切换；这里先提供状态与切换方法。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// 中文模式：字母进入拼音组合。
    Chinese,
    /// 英文字母模式：字母直接透传，不进入组合。
    English,
}

/// M1 内置演示词表，供 T-011 上屏闭环使用。
///
/// 这些词条属于脚手架，正式词典由 T-006 数据管线接管；词条格式与
/// `zhu-ye-core::dict::DictionaryEntry` 保持一致，因此后续替换时引擎无需改动。
const M1_SEED_WORDS: &[(&str, &str, u64, Option<&str>)] = &[
    ("你好", "nihao", 100, Some("hello")),
    ("尼好", "nihao", 1, None),
    ("世界", "shijie", 90, Some("world")),
    ("中国", "zhongguo", 95, Some("China")),
    ("先", "xian", 70, Some("first")),
    ("西安", "xian", 60, Some("Xi'an")),
    ("西", "xi", 55, Some("west")),
    ("安", "an", 45, Some("safe")),
    ("我", "wo", 80, Some("I")),
    ("你", "ni", 65, Some("you")),
    ("好", "hao", 58, Some("good")),
    ("的", "de", 100, Some("of")),
    ("得", "de", 55, Some("get")),
    ("地", "de", 40, Some("land")),
    ("爱", "ai", 70, Some("love")),
    ("输入", "shuru", 55, Some("input")),
    ("打字", "dazi", 50, Some("type")),
    ("谢谢", "xiexie", 65, Some("thank you")),
    ("再见", "zaijian", 50, Some("goodbye")),
    ("早上好", "zaoshanghao", 42, Some("good morning")),
];

/// 构建 M1 演示词典。
#[must_use]
pub fn m1_seed_dictionary() -> Arc<dyn Dictionary> {
    let mut dictionary = InMemoryDictionary::default();
    for &(word, pinyin, frequency, translation) in M1_SEED_WORDS {
        let mut entry = DictionaryEntry::new(word, pinyin, frequency);
        if let Some(translation) = translation {
            entry = entry.with_translation(translation);
        }
        dictionary.push(entry);
    }
    Arc::new(dictionary)
}

/// 输入状态机。
pub struct InputEngine {
    table: SyllableTable,
    dictionary: Arc<dyn Dictionary>,
    composing: String,
    candidates: Vec<Candidate>,
    mode: InputMode,
}

impl InputEngine {
    /// 使用指定词典创建引擎；词典通过 trait 注入，未来可无缝切换 mmap 实现。
    #[must_use]
    pub fn new(dictionary: Arc<dyn Dictionary>) -> Self {
        Self {
            table: SyllableTable::standard(),
            dictionary,
            composing: String::new(),
            candidates: Vec::new(),
            mode: InputMode::Chinese,
        }
    }

    /// 使用 M1 内置演示词表创建引擎。
    #[must_use]
    pub fn with_m1_seed() -> Self {
        Self::new(m1_seed_dictionary())
    }

    /// 当前输入模式。
    #[must_use]
    pub fn mode(&self) -> InputMode {
        self.mode
    }

    /// 切换中英文模式；组合中的内容在切换时保留，由上层决定是否结束组合。
    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            InputMode::Chinese => InputMode::English,
            InputMode::English => InputMode::Chinese,
        };
    }

    /// 是否存在活动组合。
    #[must_use]
    pub fn is_active(&self) -> bool {
        !self.composing.is_empty() && self.mode == InputMode::Chinese
    }

    /// 当前拼音组合串（小写 a-z）。
    #[must_use]
    pub fn composing(&self) -> &str {
        &self.composing
    }

    /// 当前候选列表；候选窗渲染与选择都从这里取数。
    #[must_use]
    pub fn candidates(&self) -> &[Candidate] {
        &self.candidates
    }

    /// 字母进入组合；英文模式或非小写字母返回 `false`。
    pub fn handle_letter(&mut self, c: char) -> bool {
        if self.mode != InputMode::Chinese || !c.is_ascii_lowercase() {
            return false;
        }
        self.composing.push(c);
        self.refresh_candidates();
        true
    }

    /// Backspace 删除最后一个拼音字母；无组合时返回 `false`。
    pub fn handle_backspace(&mut self) -> bool {
        if !self.is_active() {
            return false;
        }
        self.composing.pop();
        self.refresh_candidates();
        true
    }

    /// 空格上屏第一候选；无候选时按设计上屏拼音原文。
    pub fn handle_space(&mut self) -> Option<String> {
        if !self.is_active() {
            return None;
        }
        let text = self
            .candidates
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_else(|| self.composing.clone());
        self.clear_composition();
        Some(text)
    }

    /// Enter 上屏拼音原文。
    pub fn handle_enter(&mut self) -> Option<String> {
        if !self.is_active() {
            return None;
        }
        let text = self.composing.clone();
        self.clear_composition();
        Some(text)
    }

    /// Esc 取消本次组合，不产生提交文本。
    pub fn handle_escape(&mut self) -> bool {
        if !self.is_active() {
            return false;
        }
        self.clear_composition();
        true
    }

    /// 按 1-9 选择对应候选（index 从 0 开始）；越界时回退到拼音原文。
    pub fn select_index(&mut self, index: usize) -> Option<String> {
        if !self.is_active() {
            return None;
        }
        let text = self
            .candidates
            .get(index)
            .map(|c| c.text.clone())
            .unwrap_or_else(|| self.composing.clone());
        self.clear_composition();
        Some(text)
    }

    /// 组合被 TSF 宿主终止时清空内部分组状态。
    pub fn cancel_input(&mut self) {
        self.clear_composition();
    }

    /// 为 TSF 层提供提交预览：空格应上屏的文本。
    #[must_use]
    pub fn preview_space(&self) -> Option<String> {
        self.is_active().then(|| {
            self.candidates
                .first()
                .map(|c| c.text.clone())
                .unwrap_or_else(|| self.composing.clone())
        })
    }

    /// 为 TSF 层提供数字选择预览：第 index 个候选或拼音原文。
    #[must_use]
    pub fn preview_selection(&self, index: usize) -> Option<String> {
        self.is_active().then(|| {
            self.candidates
                .get(index)
                .map(|c| c.text.clone())
                .unwrap_or_else(|| self.composing.clone())
        })
    }

    /// 为 TSF 层提供组合串预览：删除末尾字母后的内容。
    #[must_use]
    pub fn preview_after_backspace(&self) -> Option<String> {
        self.is_active()
            .then(|| self.composing[..self.composing.len() - 1].to_owned())
    }

    /// 为 TSF 层提供提交预览：回车应上屏的拼音原文。
    #[must_use]
    pub fn preview_enter(&self) -> Option<String> {
        self.is_active().then(|| self.composing.clone())
    }

    fn clear_composition(&mut self) {
        self.composing.clear();
        self.candidates.clear();
    }

    fn refresh_candidates(&mut self) {
        self.candidates =
            generate_candidates(&self.table, self.dictionary.as_ref(), &self.composing);
    }
}

/// 根据拼音串生成候选：整词优先，再按音节切分组合，最后合并去重并确定性排序。
fn generate_candidates(
    table: &SyllableTable,
    dictionary: &dyn Dictionary,
    pinyin: &str,
) -> Vec<Candidate> {
    if pinyin.is_empty() {
        return Vec::new();
    }

    let mut collected: Vec<Candidate> = Vec::new();
    for entry in dictionary.lookup(pinyin) {
        collected.push(candidate_from_entry(&entry));
    }

    for segments in segment_all(table, pinyin) {
        if segments.len() < 2 {
            continue;
        }
        let mut combined = String::new();
        let mut total = 0i64;
        let mut complete = true;
        for syllable in &segments {
            match dictionary.lookup(syllable).first() {
                Some(entry) => {
                    combined.push_str(&entry.word);
                    total += i64::try_from(entry.frequency).unwrap_or(i64::MAX);
                }
                None => {
                    complete = false;
                    break;
                }
            }
        }
        if complete {
            collected.push(Candidate::new(combined, total));
        }
    }

    deduplicate_and_sort(collected)
}

fn candidate_from_entry(entry: &DictionaryEntry) -> Candidate {
    let mut candidate = Candidate::new(
        entry.word.clone(),
        i64::try_from(entry.frequency).unwrap_or(i64::MAX),
    );
    if let Some(translation) = &entry.translation {
        candidate = candidate.with_translation(translation.clone());
    }
    candidate
}

fn deduplicate_and_sort(candidates: Vec<Candidate>) -> Vec<Candidate> {
    let mut by_text: HashMap<String, Candidate> = HashMap::new();
    for candidate in candidates {
        match by_text.get_mut(&candidate.text) {
            Some(existing) => {
                if candidate.score > existing.score {
                    existing.score = candidate.score;
                }
                if existing.translation.is_none() {
                    existing.translation = candidate.translation.clone();
                }
            }
            None => {
                by_text.insert(candidate.text.clone(), candidate);
            }
        }
    }
    CandidateSorter::sort(by_text.into_values().collect())
}

#[cfg(test)]
mod tests {
    use super::{generate_candidates, m1_seed_dictionary, InputEngine, InputMode};
    use zhu_ye_core::pinyin::SyllableTable;

    fn engine() -> InputEngine {
        InputEngine::with_m1_seed()
    }

    fn type_text(engine: &mut InputEngine, text: &str) {
        assert!(text.chars().all(|c| engine.handle_letter(c)));
    }

    #[test]
    fn 内置词典可查询演示词() {
        let dictionary = m1_seed_dictionary();
        let found = dictionary.lookup("nihao");
        assert!(found.iter().any(|e| e.word == "你好"));
    }

    #[test]
    fn nihao第一候选是你好且带译文() {
        let mut engine = engine();
        type_text(&mut engine, "nihao");
        let candidate = &engine.candidates()[0];
        assert_eq!(candidate.text, "你好");
        assert_eq!(candidate.translation.as_deref(), Some("hello"));
    }

    #[test]
    fn xian同时包含西安与先() {
        let mut engine = engine();
        type_text(&mut engine, "xian");
        let texts: Vec<&str> = engine
            .candidates()
            .iter()
            .map(|c| c.text.as_str())
            .collect();
        assert!(texts.contains(&"先"));
        assert!(texts.contains(&"西安"));
    }

    #[test]
    fn 空格上屏第一候选并清空() {
        let mut engine = engine();
        type_text(&mut engine, "nihao");
        assert_eq!(engine.handle_space().as_deref(), Some("你好"));
        assert!(!engine.is_active());
        assert!(engine.candidates().is_empty());
    }

    #[test]
    fn 回车上屏拼音原文() {
        let mut engine = engine();
        type_text(&mut engine, "nihao");
        assert_eq!(engine.handle_enter().as_deref(), Some("nihao"));
        assert!(!engine.is_active());
    }

    #[test]
    fn 回车预览与提交一致() {
        let mut engine = engine();
        type_text(&mut engine, "nihao");
        assert_eq!(engine.preview_enter().as_deref(), Some("nihao"));
        assert_eq!(engine.handle_enter().as_deref(), Some("nihao"));
    }

    #[test]
    fn esc取消且不提交() {
        let mut engine = engine();
        type_text(&mut engine, "nihao");
        assert!(engine.handle_escape());
        assert!(!engine.is_active());
        assert_eq!(engine.composing(), "");
    }

    #[test]
    fn 数字选择第二候选() {
        let mut engine = engine();
        type_text(&mut engine, "de");
        let second = engine.candidates().get(1).map(|c| c.text.clone());
        let selected = engine.select_index(1);
        assert_eq!(selected, second);
        assert!(!engine.is_active());
    }

    #[test]
    fn backspace逐步删除并在空串时停用() {
        let mut engine = engine();
        type_text(&mut engine, "nihao");
        assert!(engine.handle_backspace());
        assert_eq!(engine.composing(), "niha");
        while engine.is_active() {
            assert!(engine.handle_backspace());
        }
        assert!(!engine.handle_backspace());
    }

    #[test]
    fn 英文模式不进入组合() {
        let mut engine = engine();
        engine.toggle_mode();
        assert_eq!(engine.mode(), InputMode::English);
        assert!(!engine.handle_letter('n'));
        assert!(!engine.is_active());
        engine.toggle_mode();
        assert!(engine.handle_letter('n'));
    }

    #[test]
    fn 候选生成确定且整词与切分合并去重() {
        let table = SyllableTable::standard();
        let dictionary = m1_seed_dictionary();
        let first = generate_candidates(&table, dictionary.as_ref(), "xian");
        let second = generate_candidates(&table, dictionary.as_ref(), "xian");
        assert_eq!(first, second);
        let texts: Vec<&str> = first.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(texts.iter().filter(|t| **t == "西安").count(), 1);
    }

    #[test]
    fn 无候选时空格回退拼音原文() {
        let mut engine = engine();
        type_text(&mut engine, "zzzz");
        assert!(engine.candidates().is_empty());
        assert_eq!(engine.handle_space().as_deref(), Some("zzzz"));
    }
}
