//! 输入状态机：纯 Rust，不依赖 Windows，负责组合串维护、候选生成与提交。
//!
//! TSF 适配层只负责把按键翻译成这里的调用，并把返回的提交文本写入文档；
//! 候选排序、拼音切分与词典查询全部复用 `zhu-ye-core`，保证行为可单测。

use std::sync::Arc;

use std::path::Path;

use zhu_ye_core::bigram::BigramModel;
use zhu_ye_core::candidate::{
    Candidate, RankingConfig, RankingContext, RankingModel, StaticRankingModel,
};
use zhu_ye_core::dict::{Dictionary, InMemoryDictionary};
use zhu_ye_core::pinyin::{segment_all, SyllableTable};
use zhu_ye_core::{unix_now, DictionaryFile, Result, UserDictStore, UserDictionary};

use crate::candidate_ui::{CandidateUiItem, CandidateUiView};

/// 单页候选数，与数字键 1-9 一一对应；翻页按此分页。
pub const CANDIDATE_PAGE_SIZE: usize = 9;

/// 输入模式。T-013 接入 Shift 切换；这里先提供状态与切换方法。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// 中文模式：字母进入拼音组合。
    Chinese,
    /// 英文字母模式：字母直接透传，不进入组合。
    English,
}

/// 候选层；Tab 在中文候选与译文之间切换。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CandidateLayer {
    /// 中文候选层。
    #[default]
    Chinese,
    /// 译文层：候选列表只展示带译文的词。
    Translation,
}

/// M1 内置演示词表，供 T-011 上屏闭环使用。
///
/// 数据与 T-006 词典管线共用 core 的种子词表；当磁盘上的 v2 词典文件
/// 缺失或损坏时，用这份内存词表保持输入法可运行。
#[must_use]
pub fn m1_seed_dictionary() -> Arc<dyn Dictionary> {
    Arc::new(InMemoryDictionary::from_entries(zhu_ye_core::seed_entries()))
}

/// 输入状态机。
pub struct InputEngine {
    table: SyllableTable,
    dictionary: Arc<dyn Dictionary>,
    composing: String,
    candidates: Vec<Candidate>,
    mode: InputMode,
    previous_word: Option<String>,
    user_dictionary: UserDictionary,
    user_store: Option<UserDictStore>,
    ranking: Arc<dyn RankingModel>,
    /// 当前候选页码，从 0 开始。
    page: usize,
    /// 当前候选层：中文候选或译文。
    layer: CandidateLayer,
    /// 每页候选数；默认与 `CANDIDATE_PAGE_SIZE` 一致。
    page_size: usize,
    /// 译文层候选缓存；输入串或候选变化时刷新。
    cached_translation_candidates: Vec<Candidate>,
}

/// 提交所需的候选快照；TSF 与引擎内部都以此为单位，避免借用冲突。
struct CandidateSelection {
    text: String,
    translation: Option<String>,
    pinyin: Option<String>,
}

fn candidate_owned(candidate: &Candidate) -> CandidateSelection {
    CandidateSelection {
        text: candidate.text.clone(),
        translation: candidate.translation.clone(),
        pinyin: candidate.pinyin.clone(),
    }
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
            previous_word: None,
            user_dictionary: UserDictionary::new(),
            user_store: None,
            ranking: Arc::new(StaticRankingModel::default()),
            page: 0,
            layer: CandidateLayer::default(),
            page_size: CANDIDATE_PAGE_SIZE,
            cached_translation_candidates: Vec::new(),
        }
    }

    /// 使用指定词典与排序模型创建引擎；测试可注入自定义排序。
    #[must_use]
    pub fn with_ranking(dictionary: Arc<dyn Dictionary>, ranking: Arc<dyn RankingModel>) -> Self {
        Self {
            ranking,
            ..Self::new(dictionary)
        }
    }

    /// 使用指定词典与 bigram 数据创建引擎。
    #[must_use]
    pub fn with_bigram(dictionary: Arc<dyn Dictionary>, bigram: Arc<dyn BigramModel>) -> Self {
        let ranking = Arc::new(StaticRankingModel::new(RankingConfig::default(), bigram));
        Self::with_ranking(dictionary, ranking)
    }

    /// 使用指定词典与用户词持久化创建引擎；启动时加载，提交时自动记录并落盘。
    #[must_use]
    pub fn with_user_store(dictionary: Arc<dyn Dictionary>, store: UserDictStore) -> Self {
        // 加载失败时回退空库，不让持久化故障阻塞输入法启动。
        let user_dictionary = store.load().unwrap_or_default();
        Self {
            user_dictionary,
            user_store: Some(store),
            ..Self::with_ranking(dictionary, Arc::new(StaticRankingModel::default()))
        }
    }

    /// 使用 M1 内置演示词表创建引擎。
    #[must_use]
    pub fn with_m1_seed() -> Self {
        Self::new(m1_seed_dictionary())
    }

    /// 从 v2 词典文件创建引擎；词典、bigram 与翻译共用同一份 mmap 数据。
    pub fn with_dictionary_file(path: &Path) -> Result<Self> {
        let file = DictionaryFile::open(path)?;
        let dictionary: Arc<dyn Dictionary> = Arc::new(file.clone());
        let bigram: Arc<dyn BigramModel> = Arc::new(file);
        Ok(Self::with_bigram(dictionary, bigram))
    }

    /// 从 v2 词典文件创建引擎，并接入用户词持久化。
    pub fn with_dictionary_file_and_user_store(path: &Path, store: UserDictStore) -> Result<Self> {
        let file = DictionaryFile::open(path)?;
        let dictionary: Arc<dyn Dictionary> = Arc::new(file.clone());
        let bigram: Arc<dyn BigramModel> = Arc::new(file);
        Ok(Self::with_user_store_and_bigram(dictionary, store, bigram))
    }

    /// 使用指定词典、bigram 与用户词持久化创建引擎。
    #[must_use]
    pub fn with_user_store_and_bigram(
        dictionary: Arc<dyn Dictionary>,
        store: UserDictStore,
        bigram: Arc<dyn BigramModel>,
    ) -> Self {
        let user_dictionary = store.load().unwrap_or_default();
        let ranking = Arc::new(StaticRankingModel::new(RankingConfig::default(), bigram));
        Self {
            user_dictionary,
            user_store: Some(store),
            ..Self::with_ranking(dictionary, ranking)
        }
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

    /// 是否存在活动组合（不区分中英模式，TSF 层判断组合生命周期使用）。
    #[must_use]
    pub fn is_composing(&self) -> bool {
        !self.composing.is_empty()
    }

    /// 是否存在活动组合且中文模式可输入。
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.is_composing() && self.mode == InputMode::Chinese
    }

    /// 当前拼音组合串（小写 a-z）。
    #[must_use]
    pub fn composing(&self) -> &str {
        &self.composing
    }

    /// 参与上下文排序的前词；由最近一次成功提交维护。
    #[must_use]
    pub fn previous_word(&self) -> Option<&str> {
        self.previous_word.as_deref()
    }

    /// 当前候选层。
    #[must_use]
    pub fn layer(&self) -> CandidateLayer {
        self.layer
    }

    /// 当前页码，从 0 开始。
    #[must_use]
    pub fn page(&self) -> usize {
        self.page
    }

    /// 当前层当前页可见候选；译文中没有译文的词不会出现。
    #[must_use]
    pub fn visible_candidates(&self) -> &[Candidate] {
        let page_size = self.page_size.max(1);
        let start = self.page.saturating_mul(page_size);
        let list = self.current_layer_candidates();
        if start >= list.len() {
            return &[];
        }
        let end = start.saturating_add(page_size).min(list.len());
        &list[start..end]
    }

    /// 当前层全部候选（译文层过滤无译文的词）。
    #[must_use]
    pub fn current_layer_candidates(&self) -> &[Candidate] {
        if self.layer == CandidateLayer::Translation {
            self.cached_translation_candidates.as_slice()
        } else {
            &self.candidates
        }
    }

    /// 全部中文候选；候选窗渲染与选择都从这里取数。
    #[must_use]
    pub fn candidates(&self) -> &[Candidate] {
        &self.candidates
    }

    /// 当前层候选总页数；至少为 1。
    #[must_use]
    pub fn page_count(&self) -> usize {
        self.current_layer_candidates()
            .len()
            .div_ceil(self.page_size.max(1))
            .max(1)
    }

    /// 字母进入组合；英文模式或非小写字母返回 `false`。
    pub fn handle_letter(&mut self, c: char) -> bool {
        if self.mode != InputMode::Chinese || !c.is_ascii_lowercase() {
            return false;
        }
        self.composing.push(c);
        self.refresh_candidates();
        self.page = 0;
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

    /// 空格提交当前层第一候选；无候选时按设计上屏拼音原文。
    pub fn handle_space(&mut self) -> Option<String> {
        if !self.is_active() {
            return None;
        }
        let Some(candidate) = self.visible_candidates().first() else {
            return self.commit_raw(self.composing.clone());
        };
        self.commit_candidate(candidate_owned(candidate))
    }

    /// Enter 上屏拼音原文。
    pub fn handle_enter(&mut self) -> Option<String> {
        if !self.is_active() {
            return None;
        }
        let text = self.composing.clone();
        self.clear_composition();
        self.previous_word = None;
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

    /// 按 1-9 选择当前层第 `index` 个候选（index 从 0 开始）；越界时回退到拼音原文。
    pub fn select_index(&mut self, index: usize) -> Option<String> {
        if !self.is_active() {
            return None;
        }
        let Some(candidate) = self.visible_candidates().get(index) else {
            return self.commit_raw(self.composing.clone());
        };
        self.commit_candidate(candidate_owned(candidate))
    }

    /// 下翻一页；末页回卷到第一页。
    pub fn next_page(&mut self) {
        let mut page = self.page.saturating_add(1);
        if page >= self.page_count() {
            page = 0;
        }
        self.page = page;
    }

    /// 上翻一页；首页回卷到最后一页。
    pub fn previous_page(&mut self) {
        let count = self.page_count();
        let mut page = self.page.checked_sub(1).unwrap_or(count - 1);
        if page >= count {
            page = 0;
        }
        self.page = page;
    }

    /// 切换中文候选层与译文层；无译文候选时保持中文层，避免出现空白页。
    ///
    /// 返回 `true` 表示切换成功。
    pub fn toggle_translation_layer(&mut self) -> bool {
        let next = match self.layer {
            CandidateLayer::Chinese => CandidateLayer::Translation,
            CandidateLayer::Translation => CandidateLayer::Chinese,
        };
        if next == CandidateLayer::Translation && self.translation_candidates().is_empty() {
            return false;
        }
        self.layer = next;
        self.clamp_page();
        true
    }

    /// 组合被 TSF 宿主终止时清空内部分组状态。
    pub fn cancel_input(&mut self) {
        self.clear_composition();
    }

    /// 为 TSF 层提供提交预览：空格应上屏的文本。
    #[must_use]
    pub fn preview_space(&self) -> Option<String> {
        self.is_active().then(|| {
            self.visible_candidates()
                .first()
                .map(|c| self.display_text(c))
                .unwrap_or_else(|| self.composing.clone())
        })
    }

    /// 为 TSF 层提供数字选择预览：第 index 个候选或拼音原文。
    #[must_use]
    pub fn preview_selection(&self, index: usize) -> Option<String> {
        self.is_active().then(|| {
            self.visible_candidates()
                .get(index)
                .map(|c| self.display_text(c))
                .unwrap_or_else(|| self.composing.clone())
        })
    }

    /// 为 TSF 层提供组合串预览：删除末尾字母后的内容。
    #[must_use]
    pub fn preview_after_backspace(&self) -> Option<String> {
        self.is_composing()
            .then(|| self.composing[..self.composing.len() - 1].to_owned())
    }

    /// 为 TSF 层提供提交预览：回车应上屏的拼音原文。
    #[must_use]
    pub fn preview_enter(&self) -> Option<String> {
        self.is_active().then(|| self.composing.clone())
    }

    /// 构建候选窗快照；候选窗渲染与 TSF 联动都从这里取数。
    #[must_use]
    pub fn candidate_ui_view(&self) -> CandidateUiView {
        let page_size = self.page_size.max(1);
        if self.mode != InputMode::Chinese {
            return CandidateUiView {
                composition: self.composing.clone(),
                pinyin_hint: pinyin_hints(&self.composing),
                page: self.page.min(self.page_count().saturating_sub(1)),
                page_size,
                selected: 0,
                translation_mode: self.layer == CandidateLayer::Translation,
                items: Vec::new(),
            };
        }
        CandidateUiView {
            composition: self.composing.clone(),
            pinyin_hint: pinyin_hints(&self.composing),
            page: self.page.min(self.page_count().saturating_sub(1)),
            page_size,
            selected: 0,
            translation_mode: self.layer == CandidateLayer::Translation,
            items: self
                .visible_candidates()
                .iter()
                .map(candidate_ui_item)
                .collect(),
        }
    }

    fn commit_candidate(&mut self, selection: CandidateSelection) -> Option<String> {
        let text = if self.layer == CandidateLayer::Translation {
            selection
                .translation
                .unwrap_or_else(|| selection.text.clone())
        } else {
            selection.text.clone()
        };
        let record_word = if self.layer == CandidateLayer::Translation {
            selection.text.clone()
        } else {
            text.clone()
        };
        if let Some(pinyin) = selection.pinyin {
            self.record_user_word(&record_word, &pinyin);
        }
        self.clear_composition();
        self.previous_word = Some(text.clone());
        Some(text)
    }

    fn commit_raw(&mut self, text: String) -> Option<String> {
        self.clear_composition();
        self.previous_word = None;
        Some(text)
    }

    fn record_user_word(&mut self, text: &str, pinyin: &str) {
        self.user_dictionary
            .record_selection(text, pinyin, unix_now());
        if let Some(store) = &self.user_store {
            // 保存失败不打断输入；词条仍保留在内存中供本次会话排序。
            let _ = store.save(&self.user_dictionary);
        }
    }

    fn display_text(&self, candidate: &Candidate) -> String {
        if self.layer == CandidateLayer::Translation {
            candidate
                .translation
                .clone()
                .unwrap_or_else(|| candidate.text.clone())
        } else {
            candidate.text.clone()
        }
    }

    fn clamp_page(&mut self) {
        let max = self.page_count().saturating_sub(1);
        self.page = self.page.min(max);
    }

    fn translation_candidates(&self) -> Vec<Candidate> {
        self.candidates
            .iter()
            .filter(|c| c.translation.as_deref().is_some_and(|s| !s.is_empty()))
            .cloned()
            .collect()
    }

    fn clear_composition(&mut self) {
        self.composing.clear();
        self.candidates.clear();
        self.cached_translation_candidates.clear();
        self.page = 0;
        self.layer = CandidateLayer::Chinese;
    }

    fn refresh_candidates(&mut self) {
        let candidates = zhu_ye_core::generate_candidates(
            &self.table,
            self.dictionary.as_ref(),
            &self.composing,
        );
        let context = RankingContext::new(self.previous_word.as_deref(), &self.user_dictionary);
        self.candidates = self.ranking.rank(candidates, &context);
        self.cached_translation_candidates = self
            .candidates
            .iter()
            .filter(|c| c.translation.as_deref().is_some_and(|s| !s.is_empty()))
            .cloned()
            .collect();
        self.clamp_page();
    }

    /// 删除一个用户词；删除成功时同步落盘，落盘失败返回错误。
    pub fn delete_user_word(&mut self, word: &str, pinyin: &str) -> Result<bool> {
        if !self.user_dictionary.delete(word, pinyin) {
            return Ok(false);
        }
        if let Some(store) = &self.user_store {
            store.save(&self.user_dictionary)?;
        }
        Ok(true)
    }

    /// 重置全部用户词；内存与磁盘同步清空。
    pub fn reset_user_words(&mut self) -> Result<()> {
        self.user_dictionary.reset();
        if let Some(store) = &self.user_store {
            store.save(&self.user_dictionary)?;
        }
        Ok(())
    }

    /// 当前用户词库引用，供排序与状态展示。
    #[must_use]
    pub fn user_dictionary(&self) -> &UserDictionary {
        &self.user_dictionary
    }
}

fn candidate_ui_item(candidate: &Candidate) -> CandidateUiItem {
    CandidateUiItem {
        text: candidate.text.clone(),
        translation: candidate.translation.clone().unwrap_or_default(),
        source: candidate.source.clone(),
    }
}

/// 拼音分词提示；使用标准音节表生成空格分隔的拼音，无法切分时保持原串。
fn pinyin_hints(composing: &str) -> String {
    let mut parts = Vec::new();
    let mut cursor = 0usize;
    for syllable in segment_all(&SyllableTable::standard(), composing) {
        let start = cursor;
        let end = cursor.saturating_add(syllable.len());
        if end > composing.len() {
            break;
        }
        parts.push(&composing[start..end]);
        cursor = end;
    }
    if cursor == composing.len() && !parts.is_empty() {
        parts.join(" ")
    } else {
        composing.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{m1_seed_dictionary, CandidateLayer, InputEngine, InputMode};
    use zhu_ye_core::bigram::InMemoryBigramModel;
    use zhu_ye_core::generate_candidates;
    use zhu_ye_core::pinyin::SyllableTable;
    use zhu_ye_core::UserDictStore;
    use zhu_ye_core::{build_v2, seed_bigrams, seed_entries};

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("zhu-ye-input-{name}-{}-{now}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

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
    fn 空格上屏第一候选并更新前词() {
        let mut engine = engine();
        type_text(&mut engine, "nihao");
        assert_eq!(engine.handle_space().as_deref(), Some("你好"));
        assert_eq!(engine.previous_word(), Some("你好"));
        assert!(!engine.is_active());
        assert!(engine.candidates().is_empty());
    }

    #[test]
    fn 回车上屏拼音原文并清空前词() {
        let mut engine = engine();
        type_text(&mut engine, "nihao");
        assert_eq!(engine.handle_enter().as_deref(), Some("nihao"));
        assert_eq!(engine.previous_word(), None);
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
    fn 数字选择第二候选取并更新前词() {
        let mut engine = engine();
        type_text(&mut engine, "nihao");
        let second = engine.candidates().get(1).map(|c| c.text.clone());
        let selected = engine.select_index(1);
        assert_eq!(selected, second);
        assert_eq!(engine.previous_word(), second.as_deref());
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

    #[test]
    fn 提交后前词参与bigram排序() {
        let mut bigram = InMemoryBigramModel::new();
        bigram.insert("你好", "得", 100_000);
        let mut engine = InputEngine::with_bigram(m1_seed_dictionary(), Arc::new(bigram));

        type_text(&mut engine, "de");
        assert_eq!(engine.candidates()[0].text, "的");
        engine.handle_escape();

        type_text(&mut engine, "nihao");
        assert_eq!(engine.handle_space().as_deref(), Some("你好"));
        type_text(&mut engine, "de");
        assert_eq!(engine.candidates()[0].text, "得");
    }

    #[test]
    fn 选择候选后写入用户词库并可重新加载() {
        let dir = temp_dir("select-persist");
        let store = UserDictStore::new(dir.join("user_words.json"));
        let mut engine = InputEngine::with_user_store(m1_seed_dictionary(), store.clone());
        type_text(&mut engine, "nihao");
        engine.select_index(1);

        let loaded = store.load().unwrap();
        assert_eq!(loaded.frequency("尼好", "nihao"), 1);
        drop(engine);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn 回车与无候选回退不写入用户词() {
        let dir = temp_dir("enter-no-record");
        let store = UserDictStore::new(dir.join("user_words.json"));
        let mut engine = InputEngine::with_user_store(m1_seed_dictionary(), store.clone());
        type_text(&mut engine, "nihao");
        engine.handle_enter();
        assert!(store.load().unwrap().is_empty());

        type_text(&mut engine, "zzzz");
        engine.handle_space();
        assert!(store.load().unwrap().is_empty());
        drop(engine);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn 用户词选择多次后提升候选排序() {
        let dir = temp_dir("user-promote");
        let store = UserDictStore::new(dir.join("user_words.json"));
        let mut engine = InputEngine::with_user_store(m1_seed_dictionary(), store.clone());

        type_text(&mut engine, "nihao");
        assert_eq!(engine.candidates()[0].text, "你好");
        engine.select_index(1);
        type_text(&mut engine, "nihao");
        engine.select_index(1);
        type_text(&mut engine, "nihao");
        engine.select_index(1);
        type_text(&mut engine, "nihao");
        assert_eq!(engine.candidates()[0].text, "尼好");
        assert_eq!(
            engine.candidates()[0].source,
            zhu_ye_core::candidate::CandidateSource::User
        );
        drop(engine);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn 删除与重置用户词同步落盘() {
        let dir = temp_dir("delete-reset");
        let store = UserDictStore::new(dir.join("user_words.json"));
        let mut engine = InputEngine::with_user_store(m1_seed_dictionary(), store.clone());
        type_text(&mut engine, "nihao");
        engine.select_index(1);

        assert!(engine.delete_user_word("尼好", "nihao").unwrap());
        assert!(!engine.delete_user_word("尼好", "nihao").unwrap());
        assert!(store.load().unwrap().is_empty());

        type_text(&mut engine, "nihao");
        engine.select_index(1);
        engine.reset_user_words().unwrap();
        assert!(store.load().unwrap().is_empty());
        drop(engine);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn v2词典文件驱动候选生成与译文层() {
        let dir = temp_dir("dict-file");
        let path = dir.join("seed.zyct");
        let bytes = build_v2(&seed_entries(), &seed_bigrams()).unwrap();
        std::fs::write(&path, bytes).unwrap();

        let mut engine = InputEngine::with_dictionary_file(&path).unwrap();
        type_text(&mut engine, "nihao");
        assert_eq!(engine.candidates()[0].text, "你好");
        assert_eq!(engine.candidates()[0].translation.as_deref(), Some("hello"));
        assert!(engine.toggle_translation_layer());
        assert_eq!(engine.visible_candidates()[0].text, "你好");
        assert_eq!(
            engine.visible_candidates()[0].translation.as_deref(),
            Some("hello")
        );

        drop(engine);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn 翻页按当前页展示候选并回卷() {
        let mut engine = engine();
        type_text(&mut engine, "nihao");
        engine.page_size = 1;
        assert_eq!(engine.page_count(), 2);
        assert_eq!(engine.visible_candidates()[0].text, "你好");
        engine.next_page();
        assert_eq!(engine.page(), 1);
        assert_eq!(engine.visible_candidates()[0].text, "尼好");
        engine.next_page();
        assert_eq!(engine.page(), 0);
        engine.previous_page();
        assert_eq!(engine.page(), 1);
    }

    #[test]
    fn 翻页后数字选择与预览基于当前页() {
        let mut engine = engine();
        type_text(&mut engine, "nihao");
        engine.page_size = 1;
        engine.next_page();
        assert_eq!(engine.preview_selection(0).as_deref(), Some("尼好"));
        assert_eq!(engine.handle_space().as_deref(), Some("尼好"));
        assert!(!engine.is_active());
    }

    #[test]
    fn 输入变化后页码收敛到有效范围() {
        let mut engine = engine();
        type_text(&mut engine, "nihao");
        engine.page_size = 1;
        engine.next_page();
        engine.handle_backspace();
        assert_eq!(engine.page(), 0);
    }

    #[test]
    fn 译文层只展示带译文的词并可返回中文层() {
        let mut engine = engine();
        type_text(&mut engine, "nihao");
        assert!(engine.toggle_translation_layer());
        assert_eq!(engine.layer(), CandidateLayer::Translation);
        let texts: Vec<&str> = engine
            .visible_candidates()
            .iter()
            .map(|c| c.text.as_str())
            .collect();
        assert_eq!(texts, vec!["你好"]);
        assert!(engine.toggle_translation_layer());
        assert_eq!(engine.layer(), CandidateLayer::Chinese);
    }

    #[test]
    fn 译文层上屏返回中文层且记录中文原词() {
        let dir = temp_dir("translation-persist");
        let store = UserDictStore::new(dir.join("user_words.json"));
        let mut engine = InputEngine::with_user_store(m1_seed_dictionary(), store.clone());
        type_text(&mut engine, "nihao");
        engine.toggle_translation_layer();
        assert_eq!(engine.handle_space().as_deref(), Some("hello"));
        assert_eq!(engine.layer(), CandidateLayer::Chinese);
        assert_eq!(engine.previous_word(), Some("hello"));
        assert_eq!(store.load().unwrap().frequency("你好", "nihao"), 1);
        drop(engine);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn 无译文时不进入译文层也不出现空白页() {
        let dictionary = zhu_ye_core::dict::InMemoryDictionary::from_entries(vec![
            zhu_ye_core::DictionaryEntry::new("尼好", "nihao", 10),
        ]);
        let mut engine = InputEngine::new(Arc::new(dictionary));
        type_text(&mut engine, "nihao");
        assert!(!engine.toggle_translation_layer());
        assert_eq!(engine.layer(), CandidateLayer::Chinese);
        assert!(!engine.visible_candidates().is_empty());
    }

    #[test]
    fn 译文层翻页和视图快照跟随当前层() {
        let mut engine = engine();
        type_text(&mut engine, "nihao");
        engine.page_size = 1;
        engine.toggle_translation_layer();
        assert_eq!(engine.page_count(), 1);
        engine.next_page();
        assert_eq!(engine.page(), 0);
        let view = engine.candidate_ui_view();
        assert!(view.translation_mode);
        assert_eq!(view.items.len(), 1);
        assert_eq!(view.items[0].translation, "hello");
        assert_eq!(view.visible_items().len(), 1);
    }

    #[test]
    fn 英文模式切换保留组合但字母不再进入() {
        let mut engine = engine();
        type_text(&mut engine, "nihao");
        engine.toggle_mode();
        assert_eq!(engine.mode(), InputMode::English);
        assert!(engine.is_composing());
        assert!(!engine.is_active());
        assert!(!engine.handle_letter('x'));
        engine.toggle_mode();
        assert_eq!(engine.mode(), InputMode::Chinese);
        assert!(engine.is_active());
    }

    #[test]
    fn 英文模式视图不显示候选但保留组合串() {
        let mut engine = engine();
        type_text(&mut engine, "nihao");
        engine.toggle_mode();
        let view = engine.candidate_ui_view();
        assert_eq!(view.composition, "nihao");
        assert!(view.items.is_empty());
    }
}
