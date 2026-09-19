//! 输入状态机：纯 Rust，不依赖 Windows，负责组合串维护、候选生成与提交。
//!
//! TSF 适配层只负责把按键翻译成这里的调用，并把返回的提交文本写入文档；
//! 候选排序、拼音切分与词典查询全部复用 `zhu-ye-core`，保证行为可单测。

use std::collections::HashMap;
use std::sync::Arc;

use std::path::Path;

use zhu_ye_core::bigram::BigramModel;
use zhu_ye_core::candidate::{
    Candidate, CandidateSorter, RankingConfig, RankingContext, RankingModel, StaticRankingModel,
};
use zhu_ye_core::dict::{Dictionary, DictionaryEntry, InMemoryDictionary};
use zhu_ye_core::pinyin::{segment_all, SyllableTable};
use zhu_ye_core::{unix_now, DictionaryFile, Result, UserDictStore, UserDictionary};

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
/// 数据与 T-006 词典管线共用 core 的种子词表；当磁盘上的 v1 词典文件
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

    /// 从 v1 词典文件创建引擎；词典与 bigram 共用同一份 mmap 数据。
    pub fn with_dictionary_file(path: &Path) -> Result<Self> {
        let file = DictionaryFile::open(path)?;
        let dictionary: Arc<dyn Dictionary> = Arc::new(file.clone());
        let bigram: Arc<dyn BigramModel> = Arc::new(file);
        Ok(Self::with_bigram(dictionary, bigram))
    }

    /// 从 v1 词典文件创建引擎，并接入用户词持久化。
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

    /// 参与上下文排序的前词；由最近一次成功提交维护。
    #[must_use]
    pub fn previous_word(&self) -> Option<&str> {
        self.previous_word.as_deref()
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
        match self.candidates.first() {
            Some(candidate) => {
                let text = candidate.text.clone();
                let pinyin = candidate.pinyin.clone();
                self.commit_text(text, pinyin)
            }
            None => self.commit_text(self.composing.clone(), None),
        }
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

    /// 按 1-9 选择对应候选（index 从 0 开始）；越界时回退到拼音原文。
    pub fn select_index(&mut self, index: usize) -> Option<String> {
        if !self.is_active() {
            return None;
        }
        match self.candidates.get(index) {
            Some(candidate) => {
                let text = candidate.text.clone();
                let pinyin = candidate.pinyin.clone();
                self.commit_text(text, pinyin)
            }
            None => self.commit_text(self.composing.clone(), None),
        }
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

    fn commit_text(&mut self, text: String, pinyin: Option<String>) -> Option<String> {
        if let Some(pinyin) = pinyin {
            if !text.is_empty() {
                self.user_dictionary
                    .record_selection(&text, pinyin, unix_now());
                if let Some(store) = &self.user_store {
                    // 保存失败不打断输入；词条仍保留在内存中供本次会话排序。
                    let _ = store.save(&self.user_dictionary);
                }
            }
        }
        self.clear_composition();
        self.previous_word = Some(text.clone());
        Some(text)
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

    fn clear_composition(&mut self) {
        self.composing.clear();
        self.candidates.clear();
    }

    fn refresh_candidates(&mut self) {
        let candidates =
            generate_candidates(&self.table, self.dictionary.as_ref(), &self.composing);
        let context = RankingContext::new(self.previous_word.as_deref(), &self.user_dictionary);
        self.candidates = self.ranking.rank(candidates, &context);
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
        let mut combined_pinyin = String::new();
        let mut total = 0i64;
        let mut complete = true;
        for syllable in &segments {
            match dictionary.lookup(syllable).first() {
                Some(entry) => {
                    combined.push_str(&entry.word);
                    combined_pinyin.push_str(syllable);
                    total += i64::try_from(entry.frequency).unwrap_or(i64::MAX);
                }
                None => {
                    complete = false;
                    break;
                }
            }
        }
        if complete {
            collected.push(Candidate::new(combined, total).with_pinyin(combined_pinyin));
        }
    }

    deduplicate_and_sort(collected)
}

fn candidate_from_entry(entry: &DictionaryEntry) -> Candidate {
    let mut candidate = Candidate::new(
        entry.word.clone(),
        i64::try_from(entry.frequency).unwrap_or(i64::MAX),
    );
    candidate = candidate.with_pinyin(entry.pinyin.clone());
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
                if existing.pinyin.is_none() {
                    existing.pinyin = candidate.pinyin.clone();
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
    use std::sync::Arc;

    use super::{generate_candidates, m1_seed_dictionary, InputEngine, InputMode};
    use zhu_ye_core::bigram::InMemoryBigramModel;
    use zhu_ye_core::pinyin::SyllableTable;
    use zhu_ye_core::UserDictStore;
    use zhu_ye_core::{build_v1, seed_bigrams, seed_entries};

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
    fn v1词典文件驱动候选生成() {
        let dir = temp_dir("dict-file");
        let path = dir.join("seed.zyct");
        let bytes = build_v1(&seed_entries(), &seed_bigrams()).unwrap();
        std::fs::write(&path, bytes).unwrap();

        let mut engine = InputEngine::with_dictionary_file(&path).unwrap();
        type_text(&mut engine, "nihao");
        assert_eq!(engine.candidates()[0].text, "你好");
        assert_eq!(engine.candidates()[0].translation.as_deref(), Some("hello"));

        drop(engine);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
