//! 输入状态机：纯 Rust，不依赖 Windows，负责组合串维护、候选生成与提交。
//!
//! TSF 适配层只负责把按键翻译成这里的调用，并把返回的提交文本写入文档；
//! 候选排序、拼音切分与词典查询全部复用 `zhu-ye-core`，保证行为可单测。

use std::sync::Arc;

use std::path::Path;

use zhu_ye_core::bigram::{BigramModel, EmptyBigramModel};
use zhu_ye_core::candidate::{
    Candidate, RankingConfig, RankingContext, RankingModel, StaticRankingModel,
};
use zhu_ye_core::dict::{Dictionary, InMemoryDictionary};
use zhu_ye_core::pinyin::{segment_all, SyllableTable};
use zhu_ye_core::tone::ToneMap;
use zhu_ye_core::{unix_now, DictionaryFile, Result, UserDictStore, UserDictionary};

use crate::candidate_ui::{CandidateUiItem, CandidateUiView};

/// 单页候选数，与数字键 1-9 一一对应；翻页按此分页。
pub const CANDIDATE_PAGE_SIZE: usize = 9;

/// 前缀候选（T-029）补全组最多进入排序的条数；防止短前缀命中过多词条。
const PREFIX_COMPLETION_CAP: usize = 32;

/// 音节分隔符的**显示**字符（T-134，定义见 `candidate_ui`）：英文弯撇 `'`
/// （U+2019）。T-139 起绘制层用 Segoe UI 直撇渲染该分隔符（页眉
/// `draw_header_mixed`）；此处转发保持既有引用路径不变。
pub use crate::candidate_ui::SYLLABLE_SEP_DISPLAY;

/// 缩写前缀补全（M6-R）最多追加的条数。
const ABBREVIATION_COMPLETION_CAP: usize = 32;

/// 英文词候选最小触发长度（场景6，FR-030）：≥2 防单字母/`v` 键路径污染。
const EN_WORD_MIN_LEN: usize = 2;

/// 英文词候选组最多展示条数（场景6，FR-030；D-10 独立组置主候选后）。
const EN_WORD_CAP: usize = 6;

/// 联系人提权候选组最多展示条数（场景9，FR-037）：短前缀防刷屏，组容量契约。
const CONTACT_CANDIDATES_CAP: usize = 8;

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
    /// 手动音节分隔符位置（T-128）：`composing`（纯拼音查询键）字符边界下标
    /// 升序列表；显示层据此插入 `'` 并约束候选切分，查询键始终是去噪纯拼音。
    manual_seps: Vec<usize>,
    candidates: Vec<Candidate>,
    mode: InputMode,
    previous_word: Option<String>,
    user_dictionary: UserDictionary,
    user_store: Option<UserDictStore>,
    ranking: Arc<dyn RankingModel>,
    /// 整句 Beam Search（M7，FR-025）使用的 bigram 数据源；
    /// 与排序模型通常共享同一份（modular 查询），无 bigram 时退化为 unigram 路径。
    bigram: Arc<dyn BigramModel>,
    /// 当前候选页码，从 0 开始。
    page: usize,
    /// 当前层当前页内选中序号，从 0 开始；上下键移动，翻页后保持。
    selected_on_page: usize,
    /// 当前候选层：中文候选或译文。
    layer: CandidateLayer,
    /// 每页候选数；默认与 `CANDIDATE_PAGE_SIZE` 一致。
    page_size: usize,
    /// 译文层候选缓存；输入串或候选变化时刷新。
    cached_translation_candidates: Vec<Candidate>,
    /// 网络语包（M6-R）：提供字母/数字缩写查询；未启用时为 `None`。
    slang: Option<Arc<dyn Dictionary>>,
    /// 上屏联想候选（T-058/T-059，场景5）：拼音为空且刚上屏过一个词时，
    /// 由 bigram 后继检索生成（Top5 整词 + 两词短语）；输入字母即清空。
    suggestion: Vec<String>,
    /// 数字格式候选模式（FR-027，场景7）：空闲态连续输入数字时的累积串；
    /// 数字由引擎直接上屏（边输边上屏），选中格式时替换最近 buffer 长度字符。
    digit_buffer: String,
    /// v 模式符号候选（FR-028，场景7）：空闲态按 `v` 启动，`v1`/`vx`/`vh`
    /// 出符号组候选；非法字母回退拼音（`vi`）。
    v_buffer: String,
    /// 已启用领域包（id 字典序，P-12 只含已启用包）；领域提权（FR-033/FR-034，场景8）
    /// 的识别与候选来源。装配时按 id 升序排列（D-16 依赖）。
    domain_packs: Vec<(String, Arc<dyn Dictionary>)>,
    /// 领域自动提权总开关（FR-035，场景8）；默认开（D-14）。
    enable_domain_boost: bool,
    /// 联系人索引（场景9，FR-036/FR-037）：由配置 `contact_vcards` 导入后建立；
    /// `None` = 未配置/已清除 → 不进提权协调层（T-050 基线不漂移）。
    contacts: Option<zhu_ye_core::ContactIndex>,
    /// 联系人提权候选条数上限（组容量，防长前缀刷屏）。
    contact_cap: usize,
    /// 英文词表文件（T-085，`en.zyen`，mmap）；`None` = 回退第五期内嵌静态表
    /// （`en_words.rs`，行为一致）。启动装配时由调用方挂载，加载失败不影响输入。
    en_lexicon: Option<zhu_ye_core::en_lexicon::EnLexicon>,
    /// 简拼路径开关（FR-023；O-05 修订：原"不新增配置开关"扩展为可关闭）。
    /// 默认开；关闭后主候选为空的 2-4 位不可切分字母走首字母展开（FR-023
    /// 词典简拼）不介入。**边界**：联系人索引原生简拼键（FR-037）与网络语
    /// 缩写路径（FR-016/FR-017）不随本开关变化。
    enable_abbreviation: bool,
    /// 模糊音与纠错开关（FR-024；O-05 修订）。默认开；关闭后纠错组
    /// （模糊替换 + 少字母补全，`corrected_candidates`）不生成。
    enable_fuzzy: bool,
    /// 带调拼音表（T-112 后续批四：候选窗拼音显示声调）。旁挂 tone 文件
    /// 由调用方装配；未装配（默认空表）时候选拼音回退无调拼注。
    tone: ToneMap,
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

/// 把中文层候选展开为译文层候选（T-131）：多义译文（`hello; hi`）拆成
/// 多条，每行一个单义、保持原义顺序（次行分数微降保序）；无译文/拆后
/// 全空的不进译文层。词性前缀保留在显示串里，上屏时另剥。
fn build_translation_candidates(candidates: &[Candidate]) -> Vec<Candidate> {
    let mut out = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        let Some(translation) = candidate.translation.as_deref().filter(|s| !s.is_empty()) else {
            continue;
        };
        let senses = zhu_ye_core::dict_format::split_translations(translation);
        if senses.is_empty() {
            continue;
        }
        for (offset, sense) in senses.into_iter().enumerate() {
            let mut split = candidate.clone();
            split.translation = Some(sense);
            split.score = split.score.saturating_sub(offset as i64);
            out.push(split);
        }
    }
    out
}

/// 把「前组」（整句组）置于主候选之前；主候选与整句同文本时让位给前组。
///
/// 保持组间固定顺序（整句组在前），组内顺序不变；与 `append_group` 对称。
fn prepend_group(front: Vec<Candidate>, main: Vec<Candidate>) -> Vec<Candidate> {
    if front.is_empty() {
        return main;
    }
    let front_texts: std::collections::HashSet<String> =
        front.iter().map(|c| c.text.clone()).collect();
    let mut merged = front;
    merged.extend(main.into_iter().filter(|c| !front_texts.contains(&c.text)));
    merged
}

/// 把「追加组」（纠错组）置于主候选之后；同文本主候选优先（纠错只是补充）。
fn append_group(main: Vec<Candidate>, extra: Vec<Candidate>) -> Vec<Candidate> {
    if extra.is_empty() {
        return main;
    }
    let main_texts: std::collections::HashSet<String> =
        main.iter().map(|c| c.text.clone()).collect();
    let mut merged = main;
    merged.extend(extra.into_iter().filter(|c| !main_texts.contains(&c.text)));
    merged
}

impl InputEngine {
    /// 使用指定词典创建引擎；词典通过 trait 注入，未来可无缝切换 mmap 实现。
    #[must_use]
    pub fn new(dictionary: Arc<dyn Dictionary>) -> Self {
        Self {
            table: SyllableTable::standard(),
            dictionary,
            composing: String::new(),
            manual_seps: Vec::new(),
            candidates: Vec::new(),
            mode: InputMode::Chinese,
            previous_word: None,
            user_dictionary: UserDictionary::new(),
            user_store: None,
            ranking: Arc::new(StaticRankingModel::default()),
            bigram: Arc::new(EmptyBigramModel),
            page: 0,
            selected_on_page: 0,
            layer: CandidateLayer::default(),
            page_size: CANDIDATE_PAGE_SIZE,
            cached_translation_candidates: Vec::new(),
            slang: None,
            suggestion: Vec::new(),
            digit_buffer: String::new(),
            v_buffer: String::new(),
            domain_packs: Vec::new(),
            enable_domain_boost: true,
            contacts: None,
            contact_cap: CONTACT_CANDIDATES_CAP,
            en_lexicon: None,
            enable_abbreviation: true,
            enable_fuzzy: true,
            tone: ToneMap::default(),
        }
    }

    /// 装配带调拼音表（T-112 后续批四：候选窗拼音显示声调）。
    #[must_use]
    pub fn with_tone_map(mut self, tone: ToneMap) -> Self {
        self.tone = tone;
        self
    }

    /// 候选词的带调空格拼音；无词级/字级带调数据时返回空串（回退无调拼注）。
    fn tone_spaced(&self, word: &str) -> String {
        self.tone.word_tone_spaced(word).unwrap_or_default()
    }

    /// 候选 → 候选窗条目（含带调拼音装配）。
    fn ui_item_for(&self, candidate: &Candidate) -> CandidateUiItem {
        candidate_ui_item_tone(candidate, &self.tone_spaced(&candidate.text))
    }

    /// 挂载英文词表文件（T-085，`en.zyen`）；`None`（默认）回退第五期内嵌静态表。
    #[must_use]
    pub fn with_en_lexicon(mut self, lexicon: zhu_ye_core::en_lexicon::EnLexicon) -> Self {
        self.en_lexicon = Some(lexicon);
        self
    }

    /// 当前是否挂载了文件英文词表。
    #[must_use]
    pub fn has_en_lexicon(&self) -> bool {
        self.en_lexicon.is_some()
    }

    /// 挂载已启用领域包（场景8）：`packs` 必须已按 id **字典序**排列（D-16，
    /// 多包同时命中取字典序首个），且只含已启用包（P-12 未启用包不参与识别）。
    #[must_use]
    pub fn with_domain_packs(mut self, packs: Vec<(String, Arc<dyn Dictionary>)>) -> Self {
        self.domain_packs = packs;
        self
    }

    /// 设置领域自动提权开关（FR-035）；默认开（D-14）。关闭后领域候选恢复
    /// 既有追加语义（T-050 基线，不做位次上移）。
    #[must_use]
    pub fn with_domain_boost(mut self, enabled: bool) -> Self {
        self.enable_domain_boost = enabled;
        self
    }

    /// 设置简拼开关（FR-023；O-05 修订：原"不新增配置开关"扩展为可关闭）。
    /// 默认开；关闭后主候选为空的 2-4 位不可切分字母不再走首字母展开。
    /// **边界**：网络语缩写路径（FR-016/FR-017）与联系人索引原生简拼键
    /// （FR-037）不随本开关变化。
    #[must_use]
    pub fn with_abbreviation(mut self, enabled: bool) -> Self {
        self.enable_abbreviation = enabled;
        self
    }

    /// 设置模糊音与纠错开关（FR-024；O-05 修订）。默认开；关闭后纠错组
    /// （模糊替换 + 少字母补全，`corrected_candidates`）不生成，其余路径不变。
    #[must_use]
    pub fn with_fuzzy(mut self, enabled: bool) -> Self {
        self.enable_fuzzy = enabled;
        self
    }

    /// 挂载联系人索引（场景9，FR-036/FR-037）：由配置 `contact_vcards` 导入后
    /// 建立；`Some` 时联系人提权进入协调层（D-21 与 D-13 同层），`None` 时
    /// 不进（无配置基线逐位一致）。
    #[must_use]
    pub fn with_contacts(mut self, index: zhu_ye_core::ContactIndex) -> Self {
        self.contacts = (!index.is_empty()).then_some(index);
        self
    }

    /// 清除联系人索引（FR-038）：配置清空/删除导入副本后调用，恢复无配置基线。
    pub fn clear_contacts(&mut self) {
        self.contacts = None;
    }

    /// 当前是否挂载了联系人索引。
    #[must_use]
    pub fn has_contacts(&self) -> bool {
        self.contacts.is_some()
    }

    /// 挂载网络语包（M6-R）：启用后缩写路径（FR-016/FR-017）生效。
    #[must_use]
    pub fn with_slang(mut self, slang: Arc<dyn Dictionary>) -> Self {
        self.slang = Some(slang);
        self
    }

    /// 当前是否已启用网络语包缩写路径。
    #[must_use]
    pub fn has_slang(&self) -> bool {
        self.slang.is_some()
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
        let ranking = Arc::new(StaticRankingModel::new(
            RankingConfig::default(),
            bigram.clone(),
        ));
        Self {
            bigram,
            ..Self::with_ranking(dictionary, ranking)
        }
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
        let ranking = Arc::new(StaticRankingModel::new(
            RankingConfig::default(),
            bigram.clone(),
        ));
        Self {
            user_dictionary,
            user_store: Some(store),
            bigram,
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

    /// 直接设置输入模式（第八期：按配置决定新输入会话的起始模式）。
    pub fn set_mode(&mut self, mode: InputMode) {
        self.mode = mode;
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

    /// 上屏联想是否处于活跃态（T-059，场景5）：
    /// 拼音为空且刚上屏过一个词、bigram 后继检索出联想候选。
    #[must_use]
    pub fn suggestion_active(&self) -> bool {
        self.composing.is_empty() && !self.suggestion.is_empty()
    }

    /// 当前上屏联想候选列表（T-059）；非联想态为空。
    #[must_use]
    pub fn suggestion_list(&self) -> &[String] {
        &self.suggestion
    }

    /// 数字格式候选模式是否活跃（FR-027，场景7）：空闲态输入数字串中。
    #[must_use]
    pub fn digit_active(&self) -> bool {
        !self.digit_buffer.is_empty()
    }

    /// 数字格式模式当前候选数（格式候选，≤8）；非数字模式返回 0。
    #[must_use]
    pub fn digit_candidate_count(&self) -> usize {
        if self.digit_active() {
            self.candidates.len()
        } else {
            0
        }
    }

    /// 数字格式模式累积的数字串（已上屏正文与其一致）。
    #[must_use]
    pub fn digit_text(&self) -> &str {
        &self.digit_buffer
    }

    /// 空闲态追加一个数字进入数字格式模式（FR-027）。
    ///
    /// 引擎吞下数字键：数字文本已由 TSF 层直插上屏，这里只累积 buffer 并
    /// 刷新格式候选；组合态/英文模式/上屏联想态（D-05 联想优先）拒绝。
    /// ASCII 小数点 `.` 也在数字模式内接受（金额 `12345.6`，TSF 层转发
    /// `VK_OEM_PERIOD`/`VK_DECIMAL`），非法位置由 `format_candidates` 兜底为空。
    pub fn digit_append(&mut self, c: char) -> bool {
        if self.mode != InputMode::Chinese
            || !(c.is_ascii_digit() || c == '.')
            || self.is_active()
            || self.suggestion_active()
        {
            return false;
        }
        self.digit_buffer.push(c);
        self.suggestion.clear();
        self.refresh_digit_candidates();
        true
    }

    /// 数字模式退格（FR-027）：引擎删除 buffer 尾部并刷新候选；
    /// 文档侧的退格由 TSF 层同步执行。清空后退出数字模式。
    pub fn digit_backspace(&mut self) -> bool {
        if !self.digit_active() {
            return false;
        }
        self.digit_buffer.pop();
        if self.digit_buffer.is_empty() {
            self.exit_digit();
        } else {
            self.refresh_digit_candidates();
        }
        true
    }

    /// 退出数字格式模式：清空 buffer 与候选；已上屏的数字正文保持不变。
    /// 退出后不把数字串当作联想前词（数字不参与上下文联想）。
    pub fn exit_digit(&mut self) {
        if !self.digit_active() {
            return;
        }
        self.digit_buffer.clear();
        self.candidates.clear();
        self.page = 0;
        self.selected_on_page = 0;
        self.layer = CandidateLayer::Chinese;
        self.previous_word = None;
        self.refresh_suggestion();
    }

    /// 数字格式选择预览：第 `index` 个格式候选的上屏文本与需替换的
    /// 字符数（= buffer UTF-16 长度，由 TSF 层做替换）；不可选返回 `None`。
    #[must_use]
    pub fn preview_digit(&self, index: usize) -> Option<(String, usize)> {
        if !self.digit_active() {
            return None;
        }
        let text = self.candidates.get(index)?.text.clone();
        Some((text, self.digit_buffer.encode_utf16().count()))
    }

    /// 提交第 `index` 个数字格式候选并退出数字模式（FR-027）。
    /// 返回 (上屏文本, 替换长度)；格式文本成为新的联想前词。
    pub fn commit_digit(&mut self, index: usize) -> Option<(String, usize)> {
        let selection = self.preview_digit(index)?;
        self.digit_buffer.clear();
        self.candidates.clear();
        self.page = 0;
        self.selected_on_page = 0;
        self.layer = CandidateLayer::Chinese;
        self.previous_word = Some(selection.0.clone());
        self.refresh_suggestion();
        Some(selection)
    }

    /// v 模式是否活跃（FR-028，场景7）：空闲态已按 `v` 且尚未退出。
    #[must_use]
    pub fn v_active(&self) -> bool {
        !self.v_buffer.is_empty()
    }

    /// 当前 v_buffer 长度（含首字母 `v`）：1 = 等待类型码，2 = 已出符号组。
    #[must_use]
    pub fn v_buffer_len(&self) -> usize {
        self.v_buffer.chars().count()
    }

    /// v 模式当前符号候选数（≤9）；非 v 模式返回 0。
    #[must_use]
    pub fn v_symbol_count(&self) -> usize {
        if self.v_active() {
            self.candidates.len()
        } else {
            0
        }
    }

    /// 空闲态按 `v` 进入 v 模式（FR-028）。
    ///
    /// 仅当组合为空、无联想、无数字模式时启动；`v` 是合法拼音字符
    /// （nv/lv），组合态的 `v` 一律走正常拼音（由 `push_composing` 处理）。
    pub fn v_start(&mut self) -> bool {
        if self.mode != InputMode::Chinese
            || self.is_active()
            || self.suggestion_active()
            || self.digit_active()
            || self.v_active()
        {
            return false;
        }
        self.v_buffer.push('v');
        self.candidates.clear();
        self.page = 0;
        self.selected_on_page = 0;
        self.layer = CandidateLayer::Chinese;
        true
    }

    /// v 模式输入类型码（`1-9`/`x`/`h` → 符号组；单位键前缀字母 → 单位换算码，
    /// T-104）：刷新对应候选。
    ///
    /// 非法类型码返回 `false`（调用方应回退拼音）。已定符号码后不再收字母；
    /// 单位码可持续追加字母直至完整键（见 [`Self::v_accepts`]）。
    pub fn v_code(&mut self, c: char) -> bool {
        if !self.v_active() {
            return false;
        }
        let already = &self.v_buffer[1..];
        // 符号类型码只在「等待类型码」时接受；单位码进行中（如 `vs` 后
        // 的 `h` 构成 `sh`）字母一律走单位判定，不被 x/h 符号码拦截（T-104）。
        if already.is_empty() && zhu_ye_core::symbol_group(c).is_some() {
            self.v_buffer.push(c);
            self.refresh_v_candidates();
            return true;
        }
        if !c.is_ascii_lowercase() {
            return false;
        }
        let mut key = String::with_capacity(already.len() + 1);
        key.push_str(already);
        key.push(c);
        if zhu_ye_core::unit_key_prefix(&key) {
            self.v_buffer.push(c);
            self.refresh_v_candidates();
            return true;
        }
        false
    }

    /// v 模式是否接受该字母键（T-104）：符号类型码（`1-9`/`x`/`h`，仅等待
    /// 类型码时）或单位键前缀字母（`v` 后按 `m` 留在 v 模式等待 `vmi`）。
    ///
    /// 供 TSF 键路分派：接受则进 `VCode`，否则回退拼音（`vi` 原语义保持）。
    #[must_use]
    pub fn v_accepts(&self, c: char) -> bool {
        if !self.v_active() {
            return false;
        }
        let already = &self.v_buffer[1..];
        if already.is_empty() && zhu_ye_core::symbol_group(c).is_some() {
            return true;
        }
        if !c.is_ascii_lowercase() {
            return false;
        }
        let mut key = String::with_capacity(already.len() + 1);
        key.push_str(already);
        key.push(c);
        zhu_ye_core::unit_key_prefix(&key)
    }

    /// v 模式输入非法字母（如 `vi` 的 `i`）：退出 v 模式并把 `v`+已收码+该字母
    /// 交给正常拼音路径（`vi` 进入组合，行为与直接输 `vi` 一致）。
    ///
    /// 已收**单位码**（如 `vm`/`vmi`）会一并并入组合（T-104：`vm`+`x` 得到
    /// `vmx`，不丢已收字母）；已定**符号码**（如 `v1`）按原语义丢弃（回退
    /// 拼音即放弃符号模式，`v1`+`i` 得到 `vi`）。
    pub fn v_consume(&mut self, c: char) -> bool {
        if !self.v_active() {
            return false;
        }
        let tail = &self.v_buffer[1..];
        let keep_unit = !tail.is_empty() && zhu_ye_core::unit_key_prefix(tail);
        let mut prefix = String::with_capacity(self.v_buffer.len() + 1);
        prefix.push('v');
        if keep_unit {
            prefix.push_str(tail);
        }
        self.v_buffer.clear();
        self.candidates.clear();
        self.composing.push_str(&prefix);
        self.composing.push(c);
        self.refresh_candidates();
        self.suggestion.clear();
        self.page = 0;
        true
    }

    /// 退出 v 模式：清空 buffer 与符号候选。
    pub fn v_exit(&mut self) {
        if !self.v_active() {
            return;
        }
        self.v_buffer.clear();
        self.candidates.clear();
        self.page = 0;
        self.selected_on_page = 0;
        self.layer = CandidateLayer::Chinese;
    }

    /// v 模式退格（FR-028）：有类型码时回退到 `v`（重新等待类型码），
    /// 只有 `v` 时直接退出 v 模式。
    pub fn v_backspace(&mut self) -> bool {
        if !self.v_active() {
            return false;
        }
        if self.v_buffer_len() > 1 {
            self.v_buffer.pop();
            self.refresh_v_candidates();
        } else {
            self.v_exit();
        }
        true
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

    /// 当前层当前页内选中序号，从 0 开始。
    #[must_use]
    pub fn selected_on_page(&self) -> usize {
        self.selected_on_page
    }

    /// 上移页内选中行；已到页首则保持不动（T-039）。
    pub fn select_up(&mut self) {
        self.selected_on_page = self.selected_on_page.saturating_sub(1);
    }

    /// 下移页内选中行；已到页尾（当前页最后一项）则保持不动（T-039）。
    pub fn select_down(&mut self) {
        let max = self.visible_candidates().len().saturating_sub(1);
        self.selected_on_page = self.selected_on_page.saturating_add(1).min(max);
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
        if !c.is_ascii_lowercase() {
            return false;
        }
        self.push_composing(c)
    }

    /// 数字进入组合串（T-049）：供含数字缩写键（`996`/`u1s1` 等）使用；
    /// 是否该走本路径由调用方按 `is_abbreviation_prefix` 判定，英文模式拒绝。
    pub fn handle_digit(&mut self, c: char) -> bool {
        if !c.is_ascii_digit() {
            return false;
        }
        self.push_composing(c)
    }

    /// 邮箱/网址格式字符进入组合串（场景6，FR-031）：`@`/`.`/`/`/`:` 在组合态
    /// 直接追加（供 TSF 层把 Shift+2 的 `@` 与格式键路进串，D-11）；
    /// 组合未激活（空闲态）返回 `false` 放行宿主（格式分支只在组合表内延伸，不冷启动，
    /// `@` 不开新组合）；英文模式拒绝。
    pub fn handle_format_char(&mut self, c: char) -> bool {
        if !matches!(c, '@' | '.' | '/' | ':') || self.composing.is_empty() {
            return false;
        }
        self.push_composing(c)
    }

    /// 格式键是否应对当前组合态生效（T-066，TSF 键路判定用，D-11）：
    /// 决定 `@`/`.`/`/`/`:` 是进组合串（返回 `true`）还是放行宿主（`false`）。
    ///
    /// - `@`：组合态一律接收（真正追加由 [`InputEngine::handle_format_char`] 把关）；
    /// - `.`：组合串已命中邮箱/网址判定（[`detect_format`] 非 None）或正处于网址意图
    ///   演进（`www`/`http`/`https` 及其 `:`/`/` 中间态）；
    /// - `/`/`:`：仅网址意图演进。
    ///
    /// 其余情况（空闲态、`nihao` 等普通拼音组合后按 `.`）返回 `false`，
    /// 保证普通拼音组合的标点直出语义不回归（原有 `.` 放行宿主行为不变）。
    #[must_use]
    pub fn is_format_key(&self, c: char) -> bool {
        if self.mode != InputMode::Chinese || self.composing.is_empty() {
            return false;
        }
        match c {
            '@' => true,
            '.' => {
                zhu_ye_core::detect_format(&self.composing) != zhu_ye_core::FormatKind::None
                    || self.is_url_intent()
            }
            '/' | ':' => self.is_url_intent(),
            _ => false,
        }
    }

    /// 当前组合串是否处于网址意图演进（T-066）：`www`/`http`/`https` 字面、
    /// `www.`/`http://`/`https://` 前缀，或 `http(s)` 后接 `:`/`/` 的中间态
    /// （`http:`/`http:/` 等）。用于格式键吃键判定，避免结构相似但无网址意义的
    /// 输入（如 `httpw`）被误判。
    fn is_url_intent(&self) -> bool {
        let c = &self.composing;
        if c == "www" || c == "http" || c == "https" || c.starts_with("www.") {
            return true;
        }
        if c.starts_with("http://") || c.starts_with("https://") {
            return true;
        }
        if let Some(rest) = c.strip_prefix("http") {
            // `http`/`https` 开头：其余部分必须是 `:`/`/` 演进字符
            // （`https` 对 strip_prefix("http") 的余段为 `s`，一并允许）。
            rest.chars().all(|ch| ch == 's' || ch == ':' || ch == '/')
        } else {
            false
        }
    }

    fn push_composing(&mut self, c: char) -> bool {
        if self.mode != InputMode::Chinese {
            return false;
        }
        // 防御：任何进入拼音组合的入口都先退出数字格式模式（FR-027）。
        self.exit_digit();
        self.composing.push(c);
        self.refresh_candidates();
        // 输入字母即退出上屏联想态（T-059）。
        self.suggestion.clear();
        self.page = 0;
        true
    }

    /// 判断 `text` 是否为某个"含数字缩写键"的前缀（T-049）。
    ///
    /// 数字键既要能选词（FR-006），又要能输入数字缩写键；判定依据是
    /// 词典中是否存在以 `text` 开头、且拼音键含 ASCII 数字的词条——
    /// 数字只有在"某缩写键的组成部分"这一种情况下才该进组合串，
    /// 否则一律保持原有选词/直出语义。
    #[must_use]
    pub fn is_abbreviation_prefix(&self, text: &str) -> bool {
        if text.is_empty() {
            return false;
        }
        self.dictionary
            .lookup_prefix(text)
            .iter()
            .any(|entry| entry.pinyin.chars().any(|c| c.is_ascii_digit()))
    }

    /// 手动插入音节分隔符 `'`（T-128，VK_OEM_7 组合态键路）：在输入串**当前末尾**
    /// 标记音节边界（引擎无光标模型，手动分隔恒追加到串尾并保留），随后按新的
    /// 边界约束刷新候选。同位置已有手动分隔符（连续按两次）返回 `false`。
    pub fn insert_separator(&mut self) -> bool {
        if self.composing.is_empty() || self.manual_seps.contains(&self.composing.len()) {
            return false;
        }
        self.manual_seps.push(self.composing.len());
        self.refresh_candidates();
        true
    }

    /// 页眉显示串（T-128）：在 `composing` 中按「手动分隔符 ∪ 自动首选切分边界」
    /// 插入显示用分隔符 `'`（U+2019，T-134）。自动边界取首选切分
    /// （`segment_all` 首方案）的全部音节边界（用户决策：全部音节边界都
    /// 显示，如 `nihao` → `ni'hao`），仅中文模式参与；无法切分时仅显示
    /// 手动分隔符（含串尾 `'`）。查询键 `composing` 本身不含分隔符（去噪）。
    #[must_use]
    pub fn composing_display(&self) -> String {
        let mut boundaries: std::collections::BTreeSet<usize> =
            self.manual_seps.iter().copied().collect();
        if self.mode == InputMode::Chinese {
            boundaries.extend(preferred_segment_boundaries(&self.table, &self.composing));
        }
        let length = self.composing.len();
        let mut display = String::with_capacity(self.composing.len());
        for (index, c) in self.composing.char_indices() {
            if index > 0 && boundaries.contains(&index) {
                display.push(SYLLABLE_SEP_DISPLAY);
            }
            display.push(c);
        }
        // 串尾手动分隔符（如刚按过 `'` 尚未继续输入）：循环无法覆盖，单独补上。
        if boundaries.contains(&length) {
            display.push(SYLLABLE_SEP_DISPLAY);
        }
        display
    }

    /// Backspace 删除最后一个拼音字母；无组合时返回 `false`。
    /// 数字模式退格删除 buffer 尾部位（FR-027）；v 模式退格回退类型码（FR-028）。
    /// T-128：串尾是手动分隔符时**优先删除分隔符**（退格先删 `'`，再删字母）。
    pub fn handle_backspace(&mut self) -> bool {
        if self.digit_active() {
            return self.digit_backspace();
        }
        if self.v_active() {
            return self.v_backspace();
        }
        if !self.is_active() {
            return false;
        }
        if self.manual_seps.last() == Some(&self.composing.len()) {
            self.manual_seps.pop();
            self.refresh_candidates();
            return true;
        }
        self.composing.pop();
        // 悬尾分隔符（位置 == 新长度）保留：显示 xi'，等待退格先删它或继续输入；
        // 越界值（旧串尾且 len 已变）防御性清理。
        self.manual_seps
            .retain(|&position| position <= self.composing.len());
        self.refresh_candidates();
        true
    }

    /// 空格提交当前选中行候选（T-039：上下键移动选中行后回车/空格跟随后者）；
    /// 上屏联想态提交选中联想词（T-059）；无候选时按设计上屏拼音原文。
    /// 数字格式模式（FR-027）空格 = 选择第 1 个格式候选并替换；
    /// v 模式（FR-028）空格 = 选择第 1 个符号候选。
    pub fn handle_space(&mut self) -> Option<String> {
        if self.digit_active() {
            return self
                .commit_digit(self.selected_on_page)
                .map(|(text, _)| text);
        }
        if self.v_active() && self.v_symbol_count() > 0 {
            let text = self.candidates.first()?.text.clone();
            return self.commit_symbol(text);
        }
        if self.suggestion_active() {
            let index = self
                .selected_on_page
                .min(self.suggestion.len().saturating_sub(1));
            let text = self.suggestion.get(index)?.clone();
            return self.commit_suggestion(text);
        }
        if !self.is_active() {
            return None;
        }
        let index = self
            .selected_on_page
            .min(self.visible_candidates().len().saturating_sub(1));
        let Some(candidate) = self.visible_candidates().get(index) else {
            return self.commit_raw(self.composing.clone());
        };
        self.commit_candidate(candidate_owned(candidate))
    }

    /// Enter 上屏拼音原文；上屏联想态 Enter 不作为联想提交（放行给宿主换行）。
    pub fn handle_enter(&mut self) -> Option<String> {
        if !self.is_active() {
            return None;
        }
        let text = self.composing.clone();
        self.clear_composition();
        self.previous_word = None;
        self.refresh_suggestion();
        Some(text)
    }

    /// Esc 取消本次组合，不产生提交文本；上屏联想态 Esc 关闭联想窗；
    /// 数字格式模式 Esc 退出（数字正文保留）；v 模式 Esc 退出。
    pub fn handle_escape(&mut self) -> bool {
        if self.digit_active() {
            self.exit_digit();
            return true;
        }
        if self.v_active() {
            self.v_exit();
            return true;
        }
        if self.suggestion_active() {
            self.suggestion.clear();
            return true;
        }
        if !self.is_active() {
            return false;
        }
        self.clear_composition();
        self.refresh_suggestion();
        true
    }

    /// 按 1-9 选择当前层第 `index` 个候选（index 从 0 开始）；越界时回退到拼音原文。
    /// 上屏联想态按数字选择联想词，越界不产生提交（防吞键）。
    /// 数字格式模式按数字选择格式候选（越界返回 `None`，由 TSF 层继续追加）；
    /// v 模式按数字选择符号候选。
    pub fn select_index(&mut self, index: usize) -> Option<String> {
        if self.digit_active() {
            return self.commit_digit(index).map(|(text, _)| text);
        }
        if self.v_active() {
            let text = self.candidates.get(index)?.text.clone();
            return self.commit_symbol(text);
        }
        if self.suggestion_active() {
            let text = self.suggestion.get(index)?.clone();
            return self.commit_suggestion(text);
        }
        if !self.is_active() {
            return None;
        }
        let Some(candidate) = self.visible_candidates().get(index) else {
            return self.commit_raw(self.composing.clone());
        };
        self.commit_candidate(candidate_owned(candidate))
    }

    /// 下翻一页；末页回卷到第一页。页内选中序号保持不变（按新页候选数封顶）。
    pub fn next_page(&mut self) {
        let mut page = self.page.saturating_add(1);
        if page >= self.page_count() {
            page = 0;
        }
        self.page = page;
        self.clamp_selected();
    }

    /// 上翻一页；首页回卷到最后一页。页内选中序号保持不变（按新页候选数封顶）。
    pub fn previous_page(&mut self) {
        let count = self.page_count();
        let mut page = self.page.checked_sub(1).unwrap_or(count - 1);
        if page >= count {
            page = 0;
        }
        self.page = page;
        self.clamp_selected();
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

    /// 第 `index` 个候选当前是否有可上屏译文（Ctrl+数字键路判定，T-131）。
    /// 中文层：可见候选的译文拆义非空（取其首义）；译文层：已拆分行的单义存在。
    #[must_use]
    pub fn can_translate_by_index(&self, index: usize) -> bool {
        if !self.is_active() {
            return false;
        }
        match self.layer {
            CandidateLayer::Translation => self
                .cached_translation_candidates
                .get(index)
                .and_then(|c| c.translation.as_deref())
                .is_some_and(|t| !t.is_empty()),
            CandidateLayer::Chinese => self
                .visible_candidates()
                .get(index)
                .and_then(|c| c.translation.as_deref())
                .is_some_and(|t| !zhu_ye_core::dict_format::split_translations(t).is_empty()),
        }
    }

    /// 直接上屏第 `index` 候选的译文（Ctrl+数字，T-131）：中文层取该候选译文
    /// 的**首义**，译文层取拆分行的**单义**，均剥词性前缀后上屏；中文词条
    /// 照常记录用户词。无对应译文时返回 `None`（TSF 层放行宿主）。
    pub fn commit_translation_by_index(&mut self, index: usize) -> Option<String> {
        if !self.is_active() {
            return None;
        }
        let (word, sense, pinyin) = match self.layer {
            CandidateLayer::Translation => {
                let candidate = self.cached_translation_candidates.get(index)?;
                (
                    candidate.text.clone(),
                    candidate.translation.clone(),
                    candidate.pinyin.clone(),
                )
            }
            CandidateLayer::Chinese => {
                let candidate = self.visible_candidates().get(index)?;
                let sense =
                    zhu_ye_core::dict_format::split_translations(candidate.translation.as_deref()?)
                        .into_iter()
                        .next()?;
                (
                    candidate.text.clone(),
                    Some(sense),
                    candidate.pinyin.clone(),
                )
            }
        };
        let text = zhu_ye_core::dict_format::strip_pos_prefix(sense.as_deref()?).to_owned();
        self.commit_translation_text(word, text, pinyin)
    }

    /// 译文上屏收尾（与 `commit_candidate` 同款）：记录中文词条（词+拼音）、
    /// 清组合、置前词并刷联想。返回上屏文本。
    fn commit_translation_text(
        &mut self,
        word: String,
        text: String,
        pinyin: Option<String>,
    ) -> Option<String> {
        if let Some(pinyin) = pinyin {
            self.record_user_word(&word, &pinyin);
        }
        self.clear_composition();
        self.previous_word = Some(text.clone());
        self.refresh_suggestion();
        Some(text)
    }

    /// 组合被 TSF 宿主终止时清空内部分组状态。
    pub fn cancel_input(&mut self) {
        self.clear_composition();
    }

    /// 为 TSF 层提供提交预览：空格应上屏的当前选中行候选（含上屏联想，T-059；
    /// 数字格式与 v 模式取各自首个候选，FR-027/028）。
    #[must_use]
    pub fn preview_space(&self) -> Option<String> {
        if self.digit_active() {
            return self.candidates.first().map(|c| c.text.clone());
        }
        if self.v_active() {
            return self.candidates.first().map(|c| c.text.clone());
        }
        if self.suggestion_active() {
            let index = self
                .selected_on_page
                .min(self.suggestion.len().saturating_sub(1));
            return Some(self.suggestion[index].clone());
        }
        self.is_active().then(|| {
            let index = self
                .selected_on_page
                .min(self.visible_candidates().len().saturating_sub(1));
            self.visible_candidates()
                .get(index)
                .map(|c| self.display_text(c))
                .unwrap_or_else(|| self.composing.clone())
        })
    }

    /// 为 TSF 层提供数字选择预览：第 index 个候选或拼音原文（含上屏联想，T-059）。
    #[must_use]
    pub fn preview_selection(&self, index: usize) -> Option<String> {
        if self.digit_active() {
            return self.candidates.get(index).map(|c| c.text.clone());
        }
        if self.v_active() {
            return self.candidates.get(index).map(|c| c.text.clone());
        }
        if self.suggestion_active() {
            return self.suggestion.get(index).cloned();
        }
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
        self.is_composing().then(|| {
            if self.manual_seps.last() == Some(&self.composing.len()) {
                // T-128：退格将删除串尾手动分隔符（退格先删 `'`）：显示串去掉尾 `'`。
                // T-134：分隔符为 U+2019（3 字节 UTF-8），按字符 pop 而非按字节切片。
                let mut display = self.composing_display();
                display.pop();
                display
            } else {
                self.composing[..self.composing.len() - 1].to_owned()
            }
        })
    }

    /// 为 TSF 层提供提交预览：回车应上屏的拼音原文。
    #[must_use]
    pub fn preview_enter(&self) -> Option<String> {
        self.is_active().then(|| self.composing.clone())
    }

    /// T-115 后续：错序容错追加组（主候选不足一页且 `enable_fuzzy` 时）。
    /// 与 M7 纠错（FR-024）互补：FR-024 只能处理可完整切分串（模糊替换/
    /// 少字母补全），错位串（`zhegnq→zhengq` 等）不可切分故漏掉——本组
    /// 枚举相邻字母交换变体出候选，标注 Corrected、pinyin=变体（正确拼音）。
    /// 主链路与前缀路径共用本入口，防止任一路径漏接。
    #[must_use]
    fn append_transposed_group(&self, mut main: Vec<Candidate>, composing: &str) -> Vec<Candidate> {
        if self.enable_fuzzy && main.len() < self.page_size {
            let transposed = zhu_ye_core::transposed_candidates(
                &self.table,
                self.dictionary.as_ref(),
                composing,
                PREFIX_COMPLETION_CAP,
            );
            if !transposed.is_empty() {
                main = append_group(main, transposed);
            }
        }
        main
    }

    /// 构建候选窗快照；候选窗渲染与 TSF 联动都从这里取数。
    ///
    /// 上屏联想态（T-059）：组合串为空、联想候选置入 items，
    /// 候选窗因此继续显示（TSF 层以 `items` 是否为空判断是否隐藏）。
    /// 数字格式模式（FR-027）与 v 模式（FR-028）：组合串为空、候选置入
    /// items，页眉提示分别显示累积数字串与 v 指令串。
    #[must_use]
    pub fn candidate_ui_view(&self) -> CandidateUiView {
        let page_size = self.page_size.max(1);
        if self.suggestion_active() {
            return CandidateUiView {
                composition: String::new(),
                pinyin_hint: String::new(),
                page: 0,
                page_size,
                page_count: 1,
                selected: self.selected_on_page,
                translation_mode: false,
                items: self
                    .suggestion
                    .iter()
                    .map(|word| CandidateUiItem {
                        text: word.clone(),
                        translation: String::new(),
                        pinyin: String::new(),
                        // T-135：联想候选也显示音标。词级带调优先、字级逐字
                        // 拼合兜底（ToneMap 字级表覆盖常用字）；罕字查不到时
                        // 留空 → 候选窗回退"无拼音行"（不产生假拼音）。
                        pinyin_tone: self.tone_spaced(word),
                        source: zhu_ye_core::candidate::CandidateSource::Suggestion,
                    })
                    .collect(),
            };
        }
        if self.digit_active() {
            return CandidateUiView {
                composition: String::new(),
                pinyin_hint: self.digit_buffer.clone(),
                page: 0,
                page_size,
                page_count: 1,
                selected: self.selected_on_page,
                translation_mode: false,
                items: self
                    .candidates
                    .iter()
                    .map(|c| self.ui_item_for(c))
                    .collect(),
            };
        }
        if self.v_active() {
            return CandidateUiView {
                composition: String::new(),
                pinyin_hint: self.v_buffer.clone(),
                page: 0,
                page_size,
                page_count: 1,
                selected: self.selected_on_page,
                translation_mode: false,
                items: self
                    .candidates
                    .iter()
                    .map(|c| self.ui_item_for(c))
                    .collect(),
            };
        }
        if self.mode != InputMode::Chinese {
            return CandidateUiView {
                // T-128：页眉组合串含拼音分隔符（英文串无法切分 → 显示原串）。
                composition: self.composing_display(),
                pinyin_hint: pinyin_hints(&self.composing),
                page: self.page.min(self.page_count().saturating_sub(1)),
                page_size,
                page_count: 1,
                selected: 0,
                translation_mode: self.layer == CandidateLayer::Translation,
                items: Vec::new(),
            };
        }
        // items 必须携带当前层**全部**候选：`CandidateUiView::visible_items()`
        // 会再按 `page` 切片一次；若这里只放当前页，翻页后切片越界变空，
        // 页面上将看不到余下候选（VM 验收翻页时复现）。选中行取引擎页内序号。
        CandidateUiView {
            // T-128：页眉组合串显示自动/手动分隔符（如 ni'hao、xi'an）。
            composition: self.composing_display(),
            pinyin_hint: pinyin_hints(&self.composing),
            page: self.page.min(self.page_count().saturating_sub(1)),
            page_size,
            page_count: self.page_count(),
            selected: self.selected_on_page,
            translation_mode: self.layer == CandidateLayer::Translation,
            items: self
                .current_layer_candidates()
                .iter()
                .map(|c| {
                    let mut item = self.ui_item_for(c);
                    // T-131：中文层副文本只显示译文的**首义**（`int. hello; int. hi`
                    // 只显示 `int. hello`），多义全量留给译文层逐行展示。
                    if self.layer == CandidateLayer::Chinese && !item.translation.is_empty() {
                        item.translation =
                            zhu_ye_core::dict_format::split_translations(&item.translation)
                                .into_iter()
                                .next()
                                .unwrap_or_default();
                    }
                    item
                })
                .collect(),
        }
    }

    fn commit_candidate(&mut self, selection: CandidateSelection) -> Option<String> {
        let text = if self.layer == CandidateLayer::Translation {
            // T-131：译文上屏剥词性前缀（`v. suspicious` → 上屏 `suspicious`）。
            selection
                .translation
                .as_deref()
                .map(zhu_ye_core::dict_format::strip_pos_prefix)
                .map(str::to_owned)
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
        // 上屏后立即按新前词重算联想候选（T-059：连续联想）。
        self.refresh_suggestion();
        Some(text)
    }

    fn commit_raw(&mut self, text: String) -> Option<String> {
        self.clear_composition();
        self.previous_word = None;
        self.refresh_suggestion();
        Some(text)
    }

    /// 提交上屏联想候选（T-059）：作为新的前词继续联想，不记录用户词
    /// （联想候选无可靠音节映射，避免污染用户词库）。
    fn commit_suggestion(&mut self, text: String) -> Option<String> {
        self.suggestion.clear();
        self.previous_word = Some(text.clone());
        self.refresh_suggestion();
        Some(text)
    }

    /// 提交 v 模式符号候选（FR-028）：符号作为新前词上屏，退出 v 模式。
    fn commit_symbol(&mut self, text: String) -> Option<String> {
        self.v_buffer.clear();
        self.candidates.clear();
        self.page = 0;
        self.selected_on_page = 0;
        self.layer = CandidateLayer::Chinese;
        self.previous_word = Some(text.clone());
        self.refresh_suggestion();
        Some(text)
    }

    /// 按数字格式规则刷新候选（FR-027）：确定性格式列表，来源 `NumberFormat`。
    fn refresh_digit_candidates(&mut self) {
        // 格式候选顺序即展示顺序（日期 4 式 → …），score 仅保序。
        self.candidates = zhu_ye_core::format_candidates(&self.digit_buffer)
            .into_iter()
            .enumerate()
            .map(|(index, format)| Candidate {
                text: format.text,
                translation: None,
                pinyin: None,
                score: index as i64,
                source: zhu_ye_core::candidate::CandidateSource::NumberFormat,
            })
            .collect();
        self.cached_translation_candidates.clear();
        self.page = 0;
        self.selected_on_page = 0;
        self.layer = CandidateLayer::Chinese;
    }

    /// 按 v 模式类型码刷新候选（FR-028；T-104 增单位换算码）：
    /// 符号码（`1-9`/`x`/`h`）→ `Symbol` 来源符号组；单位键 → 等值换算串候选，
    /// 同样来源 `Symbol`（v 模式候选整体语义一致，一页 ≤9 项）。
    fn refresh_v_candidates(&mut self) {
        if !self.v_active() {
            self.candidates.clear();
            self.cached_translation_candidates.clear();
            self.page = 0;
            self.selected_on_page = 0;
            self.layer = CandidateLayer::Chinese;
            return;
        }
        let code = &self.v_buffer[1..];
        let mut chars = code.chars();
        let is_symbol = code.len() == 1
            && chars
                .next()
                .is_some_and(|c| zhu_ye_core::symbol_group(c).is_some());
        self.candidates = if is_symbol {
            zhu_ye_core::symbol_group(code.chars().next().unwrap())
                .map(|group| {
                    group
                        .iter()
                        .enumerate()
                        .map(|(index, text)| Candidate {
                            text: (*text).to_owned(),
                            translation: None,
                            pinyin: None,
                            score: index as i64,
                            source: zhu_ye_core::candidate::CandidateSource::Symbol,
                        })
                        .collect()
                })
                .unwrap_or_default()
        } else {
            zhu_ye_core::unit_candidates(code)
                .map(|texts| {
                    texts
                        .iter()
                        .enumerate()
                        .map(|(index, text)| Candidate {
                            text: (*text).to_owned(),
                            translation: None,
                            pinyin: None,
                            score: index as i64,
                            source: zhu_ye_core::candidate::CandidateSource::Symbol,
                        })
                        .collect()
                })
                .unwrap_or_default()
        };
        self.cached_translation_candidates.clear();
        self.page = 0;
        self.selected_on_page = 0;
        self.layer = CandidateLayer::Chinese;
    }

    /// 按 bigram 后继检索刷新上屏联想候选（T-058 检索层入口）。
    ///
    /// 联想只在"拼音为空且刚上屏过一个词"时出现；输入串非空或前词缺失即清空。
    fn refresh_suggestion(&mut self) {
        if !self.composing.is_empty() {
            self.suggestion.clear();
            return;
        }
        let Some(previous) = self.previous_word.as_deref() else {
            self.suggestion.clear();
            return;
        };
        self.suggestion = zhu_ye_core::suggestion_candidates(self.bigram.as_ref(), previous);
    }

    fn record_user_word(&mut self, text: &str, pinyin: &str) {
        self.user_dictionary
            .record_selection(text, pinyin, unix_now());
        if let Some(store) = &self.user_store {
            // 保存失败不打断输入；词条仍保留在内存中供本次会话排序。
            let _ = store.save(&self.user_dictionary);
        }
    }

    /// 第 `index` 个候选直上屏译文的预取（TSF commit-text，T-131）：与
    /// `commit_translation_by_index` 同源——中文层取首义、译文层取拆分行的
    /// 单义，均剥词性前缀后返回；无对应译文返回 `None`。
    #[must_use]
    pub fn preview_translation_by_index(&self, index: usize) -> Option<String> {
        if !self.is_active() {
            return None;
        }
        let sense = match self.layer {
            CandidateLayer::Translation => self
                .cached_translation_candidates
                .get(index)?
                .translation
                .clone(),
            CandidateLayer::Chinese => self
                .visible_candidates()
                .get(index)?
                .translation
                .as_deref()
                .and_then(|t| {
                    zhu_ye_core::dict_format::split_translations(t)
                        .into_iter()
                        .next()
                }),
        }?;
        Some(zhu_ye_core::dict_format::strip_pos_prefix(&sense).to_owned())
    }

    fn display_text(&self, candidate: &Candidate) -> String {
        if self.layer == CandidateLayer::Translation {
            // T-131：译文上屏剥词性前缀（`v. suspicious` → 上屏 `suspicious`）。
            candidate
                .translation
                .as_deref()
                .map(zhu_ye_core::dict_format::strip_pos_prefix)
                .map(str::to_owned)
                .unwrap_or_else(|| candidate.text.clone())
        } else {
            candidate.text.clone()
        }
    }

    fn clamp_page(&mut self) {
        let max = self.page_count().saturating_sub(1);
        self.page = self.page.min(max);
        self.clamp_selected();
    }

    /// 页内选中序号按当前页可见候选数封顶（翻页/切层后保持行位）。
    fn clamp_selected(&mut self) {
        let max = self.visible_candidates().len().saturating_sub(1);
        self.selected_on_page = self.selected_on_page.min(max);
    }

    fn translation_candidates(&self) -> Vec<Candidate> {
        // 与 refresh_candidates 的缓存同源（同一构建函数），供切层判空与
        // 展示使用；候选未刷新时按当前 candidates 重算，确保语义一致。
        build_translation_candidates(&self.candidates)
    }

    fn clear_composition(&mut self) {
        self.composing.clear();
        self.candidates.clear();
        self.cached_translation_candidates.clear();
        self.page = 0;
        self.selected_on_page = 0;
        self.layer = CandidateLayer::Chinese;
    }

    fn refresh_candidates(&mut self) {
        let context = RankingContext::new(self.previous_word.as_deref(), &self.user_dictionary);
        // FR-031（场景6）：组合串进入邮箱/网址格式路径（含 `@` 或 `www.`/`http(s)://` 前缀），
        // 候选 = 至多 3 条补全（.com/.cn/.net 或 .com/.cn/.org），完整串（已含 `.`）直通上屏。
        // pinyin = None 不进入用户词学习；普通中文输入（无 @/www./http 前缀）不介入（D-10）。
        match zhu_ye_core::detect_format(&self.composing) {
            zhu_ye_core::FormatKind::Email => {
                self.candidates = zhu_ye_core::email_candidates(&self.composing)
                    .into_iter()
                    .enumerate()
                    .map(|(index, text)| Candidate {
                        text,
                        translation: None,
                        pinyin: None,
                        score: -(index as i64),
                        source: zhu_ye_core::candidate::CandidateSource::EmailUrl,
                    })
                    .collect();
                self.cached_translation_candidates.clear();
                self.selected_on_page = 0;
                self.clamp_page();
                return;
            }
            zhu_ye_core::FormatKind::Url => {
                self.candidates = zhu_ye_core::url_candidates(&self.composing)
                    .into_iter()
                    .enumerate()
                    .map(|(index, text)| Candidate {
                        text,
                        translation: None,
                        pinyin: None,
                        score: -(index as i64),
                        source: zhu_ye_core::candidate::CandidateSource::EmailUrl,
                    })
                    .collect();
                self.cached_translation_candidates.clear();
                self.selected_on_page = 0;
                self.clamp_page();
                return;
            }
            zhu_ye_core::FormatKind::None => {}
        }
        // T-086（FR-050）：混合串解码——同时含非 ASCII 段与不可切字母段
        // （如 `python代码`/`API接口`/`iPhone价格`）时接管候选路径：
        // 整句候选置首 + 各段最优候选随后（§14.3.3）。FR-031（`@`/`www.`/
        // http 前缀）已在上述 detect_format 优先判定，不被混合解码劫持；
        // 纯拼音/纯缩写/纯英文/纯中文由 `is_mixed_input` 判别不触发（§14.3.5）。
        if zhu_ye_core::mixed::is_mixed_input(&self.table, &self.composing) {
            self.candidates = zhu_ye_core::mixed::mixed_candidates(
                &self.table,
                Some(self.dictionary.as_ref()),
                self.en_lexicon.as_ref(),
                &self.composing,
            );
            self.cached_translation_candidates.clear();
            self.selected_on_page = 0;
            self.clamp_page();
            return;
        }
        // T-029：输入串存在尾部残缺音节时走前缀候选（补全组优先 + 完成组回退），
        // 两组分别经排序模型排序后按组间顺序融合，确保补全组始终在前。
        let groups = zhu_ye_core::generate_prefix_candidates(
            &self.table,
            self.dictionary.as_ref(),
            &self.composing,
            PREFIX_COMPLETION_CAP,
        );
        if !groups.completions.is_empty() || !groups.completed.is_empty() {
            let completions = self.ranking.rank(groups.completions, &context);
            let completed = self.ranking.rank(groups.completed, &context);
            let mut main = zhu_ye_core::merge_candidate_groups(completions, completed);
            // T-115 后续：错序容错同样接入前缀路径——`zhegnq` 这类"完整音节+残尾"
            // 串会走本分支（completed=这/者非空）而不到主链路，此前容错漏接。
            main = self.append_transposed_group(main, &self.composing);
            self.candidates = main;
        } else {
            let composing = self.composing.clone();
            let dictionary = self.dictionary.clone();
            // 主路径：整词优先，无整词时按音节切分组合（现状行为保持）。
            let candidates =
                zhu_ye_core::generate_candidates(&self.table, dictionary.as_ref(), &composing);
            let direct_hit = !dictionary.lookup(&composing).is_empty();
            let mut main = self.ranking.rank(candidates, &context);

            // FR-059（第十一期）：完整拼音整词命中不足一页时，前缀组词展开补足
            // （D-70 仅不足一页才展开、D-71 独立追加组按词频取前 N 补足一页）。
            // 展开组不参与主排序（整词命中组序原样保留），置于整词命中组之后；
            // 常见拼音（候选 ≥9）逐位不变，T-050 基线零漂移、T-057 eval 零回退。
            if direct_hit && main.len() < self.page_size {
                let expanded = zhu_ye_core::prefix_expand_candidates(
                    dictionary.as_ref(),
                    &composing,
                    self.page_size - main.len(),
                    &main,
                );
                if !expanded.is_empty() {
                    let expanded = self.ranking.rank(expanded, &context);
                    main = append_group(main, expanded);
                }
            }

            // M7 整句（FR-025）：无整词命中时用 beam 搜索全局最优整句，
            // 作为独立「整句组」置于主候选最前（长串用户意图即整句）。
            if !direct_hit {
                let sentences = zhu_ye_core::sentence_candidates(
                    &self.table,
                    dictionary.as_ref(),
                    self.bigram.as_ref(),
                    &composing,
                );
                if !sentences.is_empty() {
                    main = prepend_group(sentences, main);
                }
                // M7 纠错（FR-024）：无整词命中时追加「纠错组」（模糊替换/少字母补全），
                // 置于主候选之后、缩写组之前；同文本主候选优先。
                // O-05 修订（T-103）：可经 enable_fuzzy 关闭（模糊音与纠错组不生成）。
                if self.enable_fuzzy {
                    let corrected = zhu_ye_core::corrected_candidates(
                        &self.table,
                        dictionary.as_ref(),
                        &composing,
                    );
                    if !corrected.is_empty() {
                        main = append_group(main, corrected);
                    }
                    // T-115 后续：错序容错——快打常见的相邻字母颠倒（zhegnq→zhengq、
                    // shegnc→shengc、xiagnzhe→xiangzhe、zhagnh→zhangh）使串无法完整
                    // 切分，FR-024 只处理可切分串故漏掉；主候选不足一页时追加
                    // 「错序组」于纠错组之后、缩写组之前（候选拼音为变体=正确拼音，
                    // 供 UI 展示正确的拼音）。
                    main = self.append_transposed_group(main, &composing);
                }
            }

            // M7 简拼（FR-023）：输入不可切分、主候选仍为空且为 2-4 位纯字母时，
            // 按首字母展开整词作为主候选（防污染：仅此场景介入）。
            // O-05 修订（T-103）：可经 enable_abbreviation 关闭（词典简拼不介入；
            // 网络语缩写与联系人简拼键边界不受影响）。
            if self.enable_abbreviation
                && main.is_empty()
                && composing.chars().count() >= 2
                && composing.chars().all(|c| c.is_ascii_lowercase())
                && segment_all(&self.table, &composing).is_empty()
            {
                let initials = zhu_ye_core::initial_candidates(dictionary.as_ref(), &composing);
                if !initials.is_empty() {
                    main = self.ranking.rank(initials, &context);
                }
            }
            self.candidates = main;
        }
        // FR-034（场景8）：领域提权——整串完整词命中已启用领域包（D-15）时按 D-13
        // 位次（插基础候选之后、追加组之前）上移该包候选；无命中/开关关闭时保持
        // 既有路径（T-050「领域包只追加、不改基础排序」基线逐位不变，D-17 仅对
        // 领域候选生效，不涉及用户词与上下文联想）。
        if self.enable_domain_boost && !self.composing.is_empty() && !self.domain_packs.is_empty() {
            let packs: Vec<(String, &dyn zhu_ye_core::Dictionary)> = self
                .domain_packs
                .iter()
                .map(|(id, dict)| (id.clone(), dict.as_ref()))
                .collect();
            if let Some(boosted) = zhu_ye_core::domain_boost_candidates(&packs, &self.composing) {
                let main = std::mem::take(&mut self.candidates);
                self.candidates = append_group(main, boosted);
            }
        }
        // FR-037（场景9）：联系人提权——与领域提权同一插入点（D-21 同层、按来源
        // 顺序排在其后），落在追加组之前；全拼前缀/简拼键命中皆可。无索引（未配
        // 置/已清除）或空输入时不介入（T-050 无配置基线逐位一致）。
        if let Some(contacts) = &self.contacts {
            if !self.composing.is_empty() {
                let boosted =
                    zhu_ye_core::contact_candidates(contacts, &self.composing, self.contact_cap);
                if !boosted.is_empty() {
                    let main = std::mem::take(&mut self.candidates);
                    self.candidates = append_group(main, boosted);
                }
            }
        }
        // FR-030（场景6）：整串**完全无法按拼音切分**（与缩写路径同判定）时查英文词表；
        // 命中 → 英文候选组追加到主候选**尾部**（D-10：不参与中文静态排序、不挤占中文命中）；
        // 未命中 → 保持既有路径，缩写/网络组行为不变（`yyds` 等缩写不回退）。
        if !self.composing.is_empty()
            && self.composing.chars().count() >= EN_WORD_MIN_LEN
            && segment_all(&self.table, &self.composing).is_empty()
        {
            let mut en = match &self.en_lexicon {
                Some(lexicon) => {
                    zhu_ye_core::en_word_candidates_from(lexicon, &self.composing, EN_WORD_CAP)
                }
                None => zhu_ye_core::en_word_candidates(&self.composing, EN_WORD_CAP),
            };
            // FR-030 扩展：英文候选带中文释义（经词典反查索引 EnToZh；反查
            // 不到的词保持无译文原样展示，例如英文专名）。
            for candidate in &mut en {
                if candidate.translation.is_none() {
                    candidate.translation = self.dictionary.translate_en_to_zh(&candidate.text);
                }
            }
            if !en.is_empty() {
                let main = std::mem::take(&mut self.candidates);
                self.candidates = append_group(main, en);
            }
        }
        // M6-R 缩写路径（FR-016/FR-017）：整串完全不可切分且长度达标时，
        // 查询网络语包并把命中候选作为**独立组追加在尾部**，不参与默认排序竞争。
        if let Some(slang) = &self.slang {
            let abbreviation = zhu_ye_core::abbreviation_candidates(
                &self.table,
                slang.as_ref(),
                &self.composing,
                ABBREVIATION_COMPLETION_CAP,
            );
            // 取出主候选（`mem::take` 避免克隆），合并后写回。
            let main = std::mem::take(&mut self.candidates);
            self.candidates = zhu_ye_core::append_abbreviation_group(main, abbreviation);
        }
        // T-128：手动音节分隔符的约束组**前置**（强意图最高优先级，置于全部追加组
        // 之后执行所以排在 main 最前）：`xi'an` 按硬边界 [xi|an] 拼出「西安」顶到
        // 候选首位，常规候选随之顺延；无手动分隔符时不介入（基线零漂移）。
        if !self.manual_seps.is_empty() {
            let constrained = zhu_ye_core::constrained_segment_candidates(
                &self.table,
                self.dictionary.as_ref(),
                &self.composing,
                &self.manual_seps,
            );
            if !constrained.is_empty() {
                let constrained = self.ranking.rank(constrained, &context);
                let main = std::mem::take(&mut self.candidates);
                self.candidates = prepend_group(constrained, main);
            }
        }
        // FR-029（场景7）：整串拼音等于别名时把 emoji 追加到候选**尾部**；
        // 只占队尾、不参与排序（score 取 i64::MIN），保证 T-057 命中率不回退。
        if let Some(emoji) = zhu_ye_core::emoji_for(&self.composing) {
            self.candidates.push(Candidate {
                text: emoji.to_owned(),
                translation: None,
                pinyin: None,
                score: i64::MIN,
                source: zhu_ye_core::candidate::CandidateSource::Emoji,
            });
        }
        self.cached_translation_candidates = build_translation_candidates(&self.candidates);
        // 输入串变化后选中行回到第一行。
        self.selected_on_page = 0;
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

/// 候选 → 候选窗条目；带调拼音（T-112 后续批四：声调显示）仅在候选文本
/// 词级/字级带调音齐全时填入，否则留空回退无调拼注。
fn candidate_ui_item_tone(candidate: &Candidate, tone_spaced: &str) -> CandidateUiItem {
    CandidateUiItem {
        text: candidate.text.clone(),
        translation: candidate.translation.clone().unwrap_or_default(),
        pinyin: candidate.pinyin.clone().unwrap_or_default(),
        pinyin_tone: tone_spaced.to_owned(),
        source: candidate.source.clone(),
    }
}

/// 首选切分的全部音节边界（T-128）：取 `segment_all` 首方案（最长匹配优先），
/// 返回方案内除串首/串尾外的所有切分点（如 `nihao` → `[2]`，`xian` → `[]`）；
/// 无法切分时返回空。用于自动分隔符显示（全部音节边界都显式标 `'`）。
fn preferred_segment_boundaries(table: &SyllableTable, input: &str) -> Vec<usize> {
    let Some(segments) = segment_all(table, input).into_iter().next() else {
        return Vec::new();
    };
    if segments.len() < 2 {
        return Vec::new();
    }
    let mut boundaries = Vec::with_capacity(segments.len() - 1);
    let mut position = 0usize;
    for syllable in &segments[..segments.len() - 1] {
        position = position.saturating_add(syllable.len());
        boundaries.push(position);
    }
    boundaries
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
    use zhu_ye_core::tone::ToneMap;
    use zhu_ye_core::Dictionary;
    use zhu_ye_core::InMemoryDictionary;
    use zhu_ye_core::UserDictStore;
    use zhu_ye_core::{build_v2, seed_bigrams, seed_entries, DictionaryEntry};

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

    // ---- T-128 音节分隔符 ----

    #[test]
    fn 自动分隔符按首选切分显示() {
        let mut typing = engine();
        // nihao 首选 [ni,hao]：全部音节边界显式插 `'`。
        type_text(&mut typing, "nihao");
        assert_eq!(typing.composing_display(), "ni\u{2019}hao");
        assert_eq!(typing.composing(), "nihao", "查询键保持纯拼音（去噪）");
        // xian 首选 [xian]（无内部边界）：不自动插 `'`。
        let mut other = engine();
        type_text(&mut other, "xian");
        assert_eq!(other.composing_display(), "xian");
        // xihuan 首选 [xi,huan]。
        let mut third = engine();
        type_text(&mut third, "xihuan");
        assert_eq!(third.composing_display(), "xi\u{2019}huan");
        // 不可切分串（缩写/残缺）不插自动分隔。
        let mut fourth = engine();
        type_text(&mut fourth, "zzzz");
        assert_eq!(fourth.composing_display(), "zzzz");
        // 空串。
        let empty = engine();
        assert_eq!(empty.composing_display(), "");
    }

    #[test]
    fn 手动分隔符插入与显示() {
        let mut typing = engine();
        type_text(&mut typing, "xi");
        assert!(typing.insert_separator());
        assert_eq!(typing.composing_display(), "xi\u{2019}");
        // 同位置重复插入被拒绝。
        assert!(!typing.insert_separator());
        type_text(&mut typing, "an");
        assert_eq!(typing.composing_display(), "xi\u{2019}an");
        assert_eq!(typing.composing(), "xian", "查询键不受分隔符影响");
        // 空串无法插入分隔符。
        let mut empty = engine();
        assert!(!empty.insert_separator());
    }

    #[test]
    fn 退格优先删除串尾手动分隔符() {
        let mut engine = engine();
        type_text(&mut engine, "xi");
        engine.insert_separator();
        assert_eq!(engine.composing_display(), "xi\u{2019}");
        // 退格先删 `'`，字母保留。
        assert!(engine.handle_backspace());
        assert_eq!(engine.composing_display(), "xi");
        assert_eq!(engine.composing(), "xi");
        // 再退格删字母。
        assert!(engine.handle_backspace());
        assert_eq!(engine.composing_display(), "x");
        // 连续退格直到组合停用。
        assert!(engine.handle_backspace());
        assert!(!engine.handle_backspace());
    }

    #[test]
    fn 退格后自动分隔实时重算() {
        let mut typing = engine();
        type_text(&mut typing, "nihao");
        assert_eq!(typing.composing_display(), "ni\u{2019}hao");
        // 删 o：niha 首选 [ni,ha]，边界 2 仍在。
        assert!(typing.handle_backspace());
        assert_eq!(typing.composing_display(), "ni\u{2019}ha");
        // 删 a：nih 为残缺前缀，无完整切分 → 自动分隔消失。
        assert!(typing.handle_backspace());
        assert_eq!(typing.composing_display(), "nih");
        // 中途手动分隔符保留在有效位置。
        let mut manual = engine();
        type_text(&mut manual, "xi");
        manual.insert_separator();
        type_text(&mut manual, "an");
        assert_eq!(manual.composing_display(), "xi\u{2019}an");
        manual.handle_backspace();
        assert_eq!(manual.composing_display(), "xi\u{2019}a");
        manual.handle_backspace();
        // plain=xi、手动分隔悬在串尾：退格先删 `'`。
        assert_eq!(manual.composing_display(), "xi\u{2019}");
        manual.handle_backspace();
        assert_eq!(manual.composing_display(), "xi");
    }

    #[test]
    fn 手动分隔约束候选置首() {
        let mut plain = engine();
        type_text(&mut plain, "xian");
        // 无分隔：主候选首位是高频整词「先」。
        assert_eq!(plain.candidates()[0].text, "先");
        // xi' + an：硬边界 [xi|an] 组合「西安」前置。
        let mut separated = engine();
        type_text(&mut separated, "xi");
        separated.insert_separator();
        type_text(&mut separated, "an");
        assert_eq!(separated.composing_display(), "xi\u{2019}an");
        assert_eq!(separated.candidates()[0].text, "西安");
        // 页眉组合串同步带分隔符。
        assert_eq!(separated.candidate_ui_view().composition, "xi\u{2019}an");
    }

    #[test]
    fn 无手动分隔时约束组不介入() {
        let mut typing = engine();
        type_text(&mut typing, "nihao");
        // 仅自动分隔：候选首位不变（基线行为）。
        assert_eq!(typing.candidates()[0].text, "你好");
        assert_eq!(typing.composing_display(), "ni\u{2019}hao");
        // 不可切分串保持原样显示。
        let mut other = engine();
        type_text(&mut other, "zzzz");
        assert_eq!(other.composing_display(), "zzzz");
    }

    #[test]
    fn nih前缀候选补全组优先且含完成组() {
        let mut eng = engine();
        type_text(&mut eng, "nih");
        let texts: Vec<&str> = eng.candidates().iter().map(|c| c.text.as_str()).collect();
        // 组 2（补齐 nih → nihao）在前：你好/尼好；组 1（最后完整音节 ni）在后：你。
        assert_eq!(&texts[..3], &["你好", "尼好", "你"]);
        assert_eq!(eng.candidates()[0].translation.as_deref(), Some("hello"));

        let mut other = engine();
        type_text(&mut other, "nih");
        assert_eq!(eng.candidates(), other.candidates());
    }

    // ---- M6-R 网络语缩写路径 ----

    fn slang_engine() -> InputEngine {
        let slang: Arc<dyn Dictionary> =
            Arc::new(zhu_ye_core::InMemoryDictionary::from_entries(vec![
                zhu_ye_core::DictionaryEntry::new("永远的神", "yyds", 5000),
                zhu_ye_core::DictionaryEntry::new("有一说一", "u1s1", 5000),
                zhu_ye_core::DictionaryEntry::new("九九六", "996", 5000),
            ]));
        InputEngine::with_m1_seed().with_slang(slang)
    }

    #[test]
    fn 未挂载网络语包时缩写路径不生效() {
        let mut eng = engine();
        assert!(!eng.has_slang());
        type_text(&mut eng, "yyds");
        assert!(
            eng.candidates().iter().all(|c| c.text != "永远的神"),
            "未启用网络语包时不应出现缩写候选"
        );
    }

    #[test]
    fn 挂载网络语包后缩写候选追加尾部() {
        let mut eng = slang_engine();
        assert!(eng.has_slang());
        type_text(&mut eng, "yyds");
        let last = eng.candidates().last().expect("应有候选");
        assert_eq!(last.text, "永远的神");
        assert_eq!(last.source, zhu_ye_core::candidate::CandidateSource::Slang);
    }

    #[test]
    fn 数字缩写键在引擎层可达() {
        let mut eng = slang_engine();
        for c in "996".chars() {
            assert!(eng.handle_digit(c));
        }
        assert_eq!(eng.composing(), "996");
        let found = eng.candidates().iter().any(|c| c.text == "九九六");
        assert!(found, "996 应产出九九六候选");
    }

    #[test]
    fn 可切分串不触发缩写路径() {
        let mut eng = slang_engine();
        type_text(&mut eng, "wo");
        assert!(
            eng.candidates()
                .iter()
                .all(|c| c.source != zhu_ye_core::candidate::CandidateSource::Slang),
            "可切分串 wo 不得触发缩写路径"
        );
    }

    #[test]
    fn 无完整音节开头的输入出补全候选() {
        // T-115 后续：`z`/`zh` 等无完整音节首的单字母/声母前缀照常出补全候选
        // （搜狗/微软同款：`d` 一按出现 的/多/到），补全组有词频排序即可见。
        let mut eng = engine();
        type_text(&mut eng, "zh");
        assert!(
            !eng.candidates().is_empty(),
            "zh 应给出补全候选（的中这之类）"
        );

        let mut eng2 = engine();
        type_text(&mut eng2, "z");
        assert!(!eng2.candidates().is_empty(), "z 应给出补全候选");
    }

    #[test]
    fn backspace从残缺回到完整音节重算候选() {
        let mut engine = engine();
        type_text(&mut engine, "nih");
        assert!(engine.candidates().iter().any(|c| c.text == "你好"));
        assert!(engine.handle_backspace());
        assert_eq!(engine.composing(), "ni");
        let texts: Vec<&str> = engine
            .candidates()
            .iter()
            .map(|c| c.text.as_str())
            .collect();
        // FR-059：ni 整词命中不足一页 → 前缀组词展开补足（你 + 你好/尼好）。
        assert_eq!(texts, vec!["你", "你好", "尼好"]);
    }

    #[test]
    fn 选择前缀候选后残留拼音丢弃() {
        let mut engine = engine();
        type_text(&mut engine, "nih");
        let selected = engine.select_index(2); // 完成组"你"（对应 ni，残留 h 丢弃）
        assert_eq!(selected, Some("你".to_owned()));
        assert_eq!(engine.composing(), "");
        assert!(engine.candidates().is_empty());
        assert!(engine.user_dictionary().frequency_by_word("你") > 0);
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

    /// T-086（FR-050）：中英混合串解码——`python代码` 类输入整句候选置首，
    /// 分段候选随后；`yyds`/纯拼音不被劫持（§14.3.5 不回退清单）。
    #[test]
    fn 混合串解码整句置首且分段随后() {
        use zhu_ye_core::candidate::CandidateSource;
        use zhu_ye_core::DictionaryEntry;
        let dictionary: Arc<dyn zhu_ye_core::Dictionary> =
            Arc::new(zhu_ye_core::InMemoryDictionary::from_entries(vec![
                DictionaryEntry::new("代码", "daima", 5000),
                DictionaryEntry::new("你好", "nihao", 100),
            ]));
        let mut engine = InputEngine::new(dictionary.clone());
        type_text(&mut engine, "pythondaima");
        let texts: Vec<&str> = engine
            .candidates()
            .iter()
            .map(|c| c.text.as_str())
            .collect();
        // 整句 = python（英文词表原形）+ 代码（拼音段词典命中），置首
        assert_eq!(texts[0], "python代码");
        assert!(texts.contains(&"python"));
        assert!(texts.contains(&"代码"));
        assert!(engine
            .candidates()
            .iter()
            .any(|c| c.source == CandidateSource::Mixed));
        // 确定性：重建引擎结果一致
        let mut again = InputEngine::new(dictionary.clone());
        type_text(&mut again, "pythondaima");
        assert_eq!(texts, {
            let t: Vec<&str> = again.candidates().iter().map(|c| c.text.as_str()).collect();
            t
        });
    }

    #[test]
    fn 纯拼音与纯缩写不走混合路径() {
        use zhu_ye_core::candidate::CandidateSource;
        let mut engine1 = engine();
        type_text(&mut engine1, "nihao");
        assert_eq!(engine1.candidates()[0].text, "你好");
        assert_ne!(engine1.candidates()[0].source, CandidateSource::Mixed);

        // yyds：无 slang 时缩写路径不产出；混合路径不得劫持（无任何 Mixed 候选）
        let mut engine2 = engine();
        type_text(&mut engine2, "yyds");
        assert!(engine2
            .candidates()
            .iter()
            .all(|c| c.source != CandidateSource::Mixed));
    }

    #[test]
    fn 混合串被邮箱网址格式判定先行截获() {
        use zhu_ye_core::candidate::CandidateSource;
        // FR-031 优先级高于混合解码（验收标准 14.1.2）：`pythondaima@x` 走邮箱
        // 候选（detect_format 先 return），混合分支不接管。
        let mut engine = engine();
        type_text(&mut engine, "pythondaima");
        assert!(engine.handle_format_char('@'));
        type_text(&mut engine, "x");
        assert!(engine
            .candidates()
            .iter()
            .all(|c| c.source != CandidateSource::Mixed));
        assert!(engine.candidates()[0].text.contains('@'));
    }

    /// T-049：含数字缩写键（拼音键含 ASCII 数字）必须能被 `is_abbreviation_prefix` 识别，
    /// 从而让数字键走组合串路径而非选词；纯拼音词条不得被误判。
    #[test]
    fn 数字缩写键前缀可识别且不误判拼音词() {
        let dictionary: Arc<dyn zhu_ye_core::Dictionary> =
            Arc::new(zhu_ye_core::InMemoryDictionary::from_entries(vec![
                zhu_ye_core::DictionaryEntry::new("九九六", "996", 5000),
                zhu_ye_core::DictionaryEntry::new("有一说一", "u1s1", 5000),
                zhu_ye_core::DictionaryEntry::new("你好", "nihao", 100),
            ]));
        let engine = InputEngine::new(dictionary);

        // 纯数字与含数字混合键的前缀链都识别。
        assert!(engine.is_abbreviation_prefix("9"));
        assert!(engine.is_abbreviation_prefix("99"));
        assert!(engine.is_abbreviation_prefix("u"));
        assert!(engine.is_abbreviation_prefix("u1"));
        assert!(engine.is_abbreviation_prefix("u1s"));
        // 完整键本身也算前缀（`lookup_prefix` 含等值匹配）。
        assert!(engine.is_abbreviation_prefix("996"));
        // 纯拼音词前缀不得被判为数字缩写前缀，否则会夺走数字选词。
        assert!(!engine.is_abbreviation_prefix("n"));
        assert!(!engine.is_abbreviation_prefix("ni"));
        assert!(!engine.is_abbreviation_prefix(""));
    }

    /// T-049：数字进入组合串后与字母拼接，走整串直查得到缩写词。
    #[test]
    fn 数字进入组合串可查询含数字缩写词() {
        let dictionary: Arc<dyn zhu_ye_core::Dictionary> =
            Arc::new(zhu_ye_core::InMemoryDictionary::from_entries(vec![
                zhu_ye_core::DictionaryEntry::new("九九六", "996", 5000),
                zhu_ye_core::DictionaryEntry::new("有一说一", "u1s1", 5000),
            ]));
        let mut engine = InputEngine::new(dictionary);

        for c in "996".chars() {
            assert!(engine.handle_digit(c), "数字 {c} 应进入组合串");
        }
        assert_eq!(engine.composing(), "996");
        assert_eq!(engine.candidates()[0].text, "九九六");

        engine.handle_escape();
        assert!(engine.handle_letter('u'));
        assert!(engine.handle_digit('1'));
        assert!(engine.handle_letter('s'));
        assert!(engine.handle_digit('1'));
        assert_eq!(engine.composing(), "u1s1");
        assert_eq!(engine.candidates()[0].text, "有一说一");
    }

    /// T-049：英文模式与非法字符不得进入组合串，避免破坏既有模式语义。
    #[test]
    fn 英文模式与非法字符不进入数字组合() {
        let mut engine = engine();
        engine.toggle_mode();
        assert!(!engine.handle_digit('9'));
        engine.toggle_mode();
        assert!(!engine.handle_digit('a'));
        assert!(!engine.handle_digit('中'));
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

    fn suggestion_engine() -> InputEngine {
        let mut bigram = InMemoryBigramModel::new();
        bigram.insert("你好", "世界", 120);
        bigram.insert("你好", "中国", 80);
        bigram.insert("世界", "你好", 90);
        bigram.insert("世界", "中国", 60);
        bigram.insert("中国", "你好", 50);
        bigram.insert("中国", "世界", 40);
        InputEngine::with_bigram(m1_seed_dictionary(), Arc::new(bigram))
    }

    fn commit_nihao(engine: &mut InputEngine) {
        type_text(engine, "nihao");
        assert_eq!(engine.handle_space().as_deref(), Some("你好"));
    }

    #[test]
    fn 上屏后出现联想且数字选择上屏并继续联想() {
        let mut engine = suggestion_engine();
        commit_nihao(&mut engine);
        assert!(engine.suggestion_active());
        let ui = engine.candidate_ui_view();
        // 联想态组合串为空，候选窗 items 置入联想列表（T-059 显示依据）。
        assert_eq!(ui.composition, "");
        let texts: Vec<&str> = ui.items.iter().map(|i| i.text.as_str()).collect();
        // 整词（世界/中国）在前，短语（你好+后继）置后，符合 T-058 候选契约。
        assert_eq!(texts, vec!["世界", "中国", "你好世界", "你好中国"]);
        assert_eq!(engine.previous_word(), Some("你好"));

        // 数字选择联想词上屏，并以联想词为前词继续联想。
        assert_eq!(engine.select_index(1).as_deref(), Some("中国"));
        assert!(engine.suggestion_active());
        assert_eq!(engine.previous_word(), Some("中国"));
        let after: Vec<String> = engine
            .candidate_ui_view()
            .items
            .iter()
            .map(|i| i.text.clone())
            .collect();
        assert_eq!(after, vec!["你好", "世界", "中国你好", "中国世界"]);
    }

    #[test]
    fn 输入字母退出联想态() {
        let mut engine = suggestion_engine();
        commit_nihao(&mut engine);
        assert!(engine.suggestion_active());
        type_text(&mut engine, "n");
        assert!(!engine.suggestion_active());
        assert_eq!(engine.composing(), "n");
    }

    #[test]
    fn 联想态回车与越界数字不产生提交esc关闭联想() {
        let mut engine = suggestion_engine();
        commit_nihao(&mut engine);
        // 越界数字（第 9 条不存在）放行，不吞键也不上屏空串。
        assert_eq!(engine.select_index(9), None);
        assert!(engine.suggestion_active());
        // 联想态 Enter 放行给宿主（不提交联想词）。
        assert_eq!(engine.handle_enter(), None);
        assert!(engine.suggestion_active());
        // Esc 关闭联想窗。
        assert!(engine.handle_escape());
        assert!(!engine.suggestion_active());
    }

    #[test]
    fn 联想态空格上屏当前选中联想词() {
        let mut engine = suggestion_engine();
        commit_nihao(&mut engine);
        assert_eq!(engine.handle_space().as_deref(), Some("世界"));
        assert!(engine.suggestion_active());
        assert_eq!(engine.previous_word(), Some("世界"));
    }

    #[test]
    fn 联想候选携带带调拼音() {
        // 未装配带调表：pinyin_tone 全空（T-135 前的基线：联想无拼音行）。
        let mut plain = suggestion_engine();
        commit_nihao(&mut plain);
        let view = plain.candidate_ui_view();
        assert!(
            view.items.iter().all(|i| i.pinyin_tone.is_empty()),
            "无带调表时联想不强行造拼音"
        );

        // 装配带调表（词级"你好" + 字级兜底）：整词/短语/字级拼合全部出音标。
        let mut engine = suggestion_engine();
        engine = engine.with_tone_map(ToneMap::from_lines(
            "#word\n你好\tnǐ hǎo\n#char\n你\tnǐ\n好\thǎo\n世\tshì\n界\tjiè\n中\tzhōng\n国\tguó\n",
        ));
        commit_nihao(&mut engine);
        let view = engine.candidate_ui_view();
        let pin = |text: &str| {
            view.items
                .iter()
                .find(|i| i.text == text)
                .map(|i| i.pinyin_tone.as_str())
        };
        assert_eq!(pin("世界"), Some("shì jiè"), "字级逐字拼合");
        assert_eq!(pin("你好世界"), Some("nǐ hǎo shì jiè"), "两词短语逐字拼合");
        assert_eq!(pin("中国"), Some("zhōng guó"), "字级拼合");

        // 字级缺音的罕字联想词留空：候选窗回退"无拼音行"，不产生假拼音。
        let mut bigram = InMemoryBigramModel::new();
        bigram.insert("你好", "世界", 120);
        bigram.insert("你好", "赞", 10);
        let mut engine = InputEngine::with_bigram(m1_seed_dictionary(), Arc::new(bigram));
        engine = engine.with_tone_map(ToneMap::from_lines("#char\n世\tshì\n界\tjiè\n"));
        commit_nihao(&mut engine);
        let view = engine.candidate_ui_view();
        assert_eq!(
            view.items
                .iter()
                .find(|i| i.text == "赞")
                .unwrap()
                .pinyin_tone,
            "",
            "缺音字留空"
        );
        assert_eq!(
            view.items
                .iter()
                .find(|i| i.text == "世界")
                .unwrap()
                .pinyin_tone,
            "shì jiè"
        );
    }

    #[test]
    fn 无bigram数据时上屏不联想() {
        let mut engine =
            InputEngine::with_bigram(m1_seed_dictionary(), Arc::new(InMemoryBigramModel::new()));
        commit_nihao(&mut engine);
        assert!(!engine.suggestion_active());
        assert!(engine.candidate_ui_view().items.is_empty());
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
    fn 前缀路径下错序容错命中正确候选() {
        // `zhegnq`：completed=这 非空 → 前缀分支；错序组漏接的历史 bug 回归封印
        let mut entries = seed_entries();
        entries.push(DictionaryEntry::new("正确", "zhengque", 300));
        entries.push(DictionaryEntry::new("这", "zhe", 250));
        let dir = temp_dir("transposed-prefix");
        let path = dir.join("t.zyct");
        let bytes = build_v2(&entries, &seed_bigrams()).unwrap();
        std::fs::write(&path, bytes).unwrap();
        let mut engine = InputEngine::with_dictionary_file(&path).unwrap();

        type_text(&mut engine, "zhegnq");
        let cands = engine.candidates();
        let zh = cands.iter().find(|c| c.text == "正确");
        assert!(
            zh.is_some(),
            "zhegnq 应经前缀路径命中错序候选 正确，实际: {:?}",
            cands.iter().map(|c| c.text.as_str()).collect::<Vec<_>>()
        );
        assert_eq!(zh.unwrap().pinyin.as_deref(), Some("zhengq"));
    }

    #[test]
    fn 前缀路径下错序容错命中生成候选() {
        let mut entries = seed_entries();
        entries.push(DictionaryEntry::new("生成", "shengcheng", 280));
        entries.push(DictionaryEntry::new("设", "she", 220));
        let dir = temp_dir("transposed-prefix2");
        let path = dir.join("t.zyct");
        let bytes = build_v2(&entries, &seed_bigrams()).unwrap();
        std::fs::write(&path, bytes).unwrap();
        let mut engine = InputEngine::with_dictionary_file(&path).unwrap();

        type_text(&mut engine, "shegnc");
        let cands = engine.candidates();
        let sc = cands.iter().find(|c| c.text == "生成");
        assert!(
            sc.is_some(),
            "shegnc 应命中错序候选 生成，实际: {:?}",
            cands.iter().map(|c| c.text.as_str()).collect::<Vec<_>>()
        );
        assert_eq!(sc.unwrap().pinyin.as_deref(), Some("shengc"));
    }

    #[test]
    fn 主链路路径错序容错命中账号与想着() {
        // `zhagnh` 无任何完整音节压阵 → 主链路分支；`xiagnzhe` 的 xi=西 压阵 → 前缀分支
        let mut entries = seed_entries();
        entries.push(DictionaryEntry::new("账号", "zhanghao", 260));
        entries.push(DictionaryEntry::new("想着", "xiangzhe", 120));
        let dir = temp_dir("transposed-main");
        let path = dir.join("t.zyct");
        let bytes = build_v2(&entries, &seed_bigrams()).unwrap();
        std::fs::write(&path, bytes).unwrap();
        let mut engine = InputEngine::with_dictionary_file(&path).unwrap();

        type_text(&mut engine, "zhagnh");
        let cands = engine.candidates();
        let zh = cands.iter().find(|c| c.text == "账号");
        assert!(
            zh.is_some(),
            "zhagnh 应命中错序候选 账号，实际: {:?}",
            cands.iter().map(|c| c.text.as_str()).collect::<Vec<_>>()
        );
        assert_eq!(zh.unwrap().pinyin.as_deref(), Some("zhangh"));
    }

    #[test]
    fn 带调拼音装配后候选视图携带声调() {
        let mut entries = seed_entries();
        entries.push(DictionaryEntry::new("正确", "zhengque", 300));
        let dir = temp_dir("tone-pin");
        let path = dir.join("t.zyct");
        let bytes = build_v2(&entries, &seed_bigrams()).unwrap();
        std::fs::write(&path, bytes).unwrap();

        // 未装配带调表 -> pinyin_tone 为空（回退无调拼注）
        let mut engine = InputEngine::with_dictionary_file(&path).unwrap();
        type_text(&mut engine, "zhengque");
        let view = engine.candidate_ui_view();
        let item = view.items.iter().find(|i| i.text == "正确").unwrap();
        assert!(item.pinyin_tone.is_empty());

        // 词级带调命中
        let mut engine = InputEngine::with_dictionary_file(&path).unwrap();
        engine = engine.with_tone_map(ToneMap::from_lines("#word\n正确\tzhèng què\n"));
        type_text(&mut engine, "zhengque");
        let view = engine.candidate_ui_view();
        let item = view.items.iter().find(|i| i.text == "正确").unwrap();
        assert_eq!(item.pinyin_tone, "zhèng què");

        // 字级兜底逐字拼合
        let mut engine = InputEngine::with_dictionary_file(&path).unwrap();
        engine = engine.with_tone_map(ToneMap::from_lines("#char\n正\tzhèng\n确\tquè\n"));
        type_text(&mut engine, "zhengque");
        let view = engine.candidate_ui_view();
        let item = view.items.iter().find(|i| i.text == "正确").unwrap();
        assert_eq!(item.pinyin_tone, "zhèng què");

        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn 前缀路径下错序容错命中想着() {
        // `xiagnzhe`：xi=西 压阵 → 前缀分支，变体 xiangzhe 可切分 → 想着
        let mut entries = seed_entries();
        entries.push(DictionaryEntry::new("想着", "xiangzhe", 120));
        let dir = temp_dir("transposed-main2");
        let path = dir.join("t.zyct");
        let bytes = build_v2(&entries, &seed_bigrams()).unwrap();
        std::fs::write(&path, bytes).unwrap();
        let mut engine = InputEngine::with_dictionary_file(&path).unwrap();

        type_text(&mut engine, "xiagnzhe");
        let cands = engine.candidates();
        let xz = cands.iter().find(|c| c.text == "想着");
        assert!(
            xz.is_some(),
            "xiagnzhe 应命中错序候选 想着，实际: {:?}",
            cands.iter().map(|c| c.text.as_str()).collect::<Vec<_>>()
        );
        assert_eq!(xz.unwrap().pinyin.as_deref(), Some("xiangzhe"));
    }

    #[test]
    fn 英文候选带中文译文() {
        // v2 词典（含 你好→hello 反查索引）驱动：英文候选 hello 应挂中文"你好"。
        // 英文词表走静态表回退（测试环境无 en.zyen），反查来自 DictionaryFile。
        let dir = temp_dir("en-trans");
        let path = dir.join("t.zyct");
        let bytes = build_v2(&seed_entries(), &seed_bigrams()).unwrap();
        std::fs::write(&path, bytes).unwrap();
        let mut engine = InputEngine::with_dictionary_file(&path).unwrap();

        type_text(&mut engine, "hello");
        let cands = engine.candidates();
        let hello = cands
            .iter()
            .find(|c| c.source == zhu_ye_core::candidate::CandidateSource::EnWord);
        assert!(
            hello.is_some(),
            "hello 应命中英文候选，实际: {:?}",
            cands.iter().map(|c| c.text.as_str()).collect::<Vec<_>>()
        );
        assert_eq!(hello.unwrap().translation.as_deref(), Some("你好"));
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
    fn 上下键在页内移动选中行并在边界停住() {
        let mut engine = engine();
        type_text(&mut engine, "nihao"); // 你好、尼好
        engine.select_down();
        assert_eq!(engine.selected_on_page(), 1);
        engine.select_down();
        assert_eq!(engine.selected_on_page(), 1); // 页尾停住
        engine.select_up();
        assert_eq!(engine.selected_on_page(), 0);
        engine.select_up();
        assert_eq!(engine.selected_on_page(), 0); // 页首停住
    }

    #[test]
    fn 选中行决定空格提交内容() {
        let mut engine = engine();
        type_text(&mut engine, "nihao");
        engine.select_down();
        assert_eq!(engine.preview_space().as_deref(), Some("尼好"));
        assert_eq!(engine.handle_space().as_deref(), Some("尼好"));
    }

    #[test]
    fn 输入变化后选中行回到第一行() {
        let mut engine = engine();
        type_text(&mut engine, "nihao");
        engine.select_down();
        assert_eq!(engine.selected_on_page(), 1);
        engine.handle_backspace();
        assert_eq!(engine.selected_on_page(), 0);
        engine.handle_letter('o');
        assert_eq!(engine.selected_on_page(), 0);
    }

    #[test]
    fn 翻页保持选中行且末页不足时封顶() {
        let dictionary = zhu_ye_core::dict::InMemoryDictionary::from_entries(vec![
            zhu_ye_core::DictionaryEntry::new("词一", "nihao", 100),
            zhu_ye_core::DictionaryEntry::new("词二", "nihao", 80),
            zhu_ye_core::DictionaryEntry::new("词三", "nihao", 60),
            zhu_ye_core::DictionaryEntry::new("词四", "nihao", 40),
            zhu_ye_core::DictionaryEntry::new("词五", "nihao", 20),
        ]);
        let mut engine = InputEngine::new(Arc::new(dictionary));
        type_text(&mut engine, "nihao");
        engine.page_size = 2;
        assert_eq!(engine.page_count(), 3);
        engine.select_down(); // 页 0 第 2 项
        assert_eq!(engine.selected_on_page(), 1);
        engine.next_page(); // 页 1 有两项，行位保持
        assert_eq!(engine.page(), 1);
        assert_eq!(engine.selected_on_page(), 1);
        engine.next_page(); // 页 2 仅一项，封顶回第一行
        assert_eq!(engine.page(), 2);
        assert_eq!(engine.selected_on_page(), 0);
        engine.previous_page(); // 返回页 1，行位保持
        assert_eq!(engine.page(), 1);
        assert_eq!(engine.selected_on_page(), 0);
    }

    #[test]
    fn 选中行同步到视图快照高亮() {
        let mut engine = engine();
        type_text(&mut engine, "nihao");
        engine.select_down();
        let view = engine.candidate_ui_view();
        assert_eq!(view.selected, 1);
        assert_eq!(view.selected_on_page(), Some(1));
        assert_eq!(view.visible_items()[1].text, "尼好");
    }

    #[test]
    fn 翻页后视图快照可见项跟随当前页() {
        let mut engine = engine();
        type_text(&mut engine, "nihao");
        engine.page_size = 1;
        let view = engine.candidate_ui_view();
        assert_eq!(view.visible_items().len(), 1);
        assert_eq!(view.visible_items()[0].text, "你好");
        engine.next_page();
        let view = engine.candidate_ui_view();
        assert_eq!(view.visible_items().len(), 1);
        assert_eq!(view.visible_items()[0].text, "尼好");
        // 修复前：items 只含引擎当前页，翻页后 view 再按 page 切片越界变空
    }

    #[test]
    fn 输入变化后页码收敛到有效范围() {
        let mut engine = engine();
        type_text(&mut engine, "nihao");
        engine.page_size = 1;
        engine.next_page();
        // 连续退格：`niha` 时 M7 纠错提供「你好/尼好」候选（2 页，page=1 仍有效，
        // 预期改进）；退到 `ni`（整词单候选）时页码必须收敛归零。
        engine.handle_backspace();
        engine.handle_backspace();
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

    // ---- 输入体验优化（M7，FR-023 至 FR-025）引擎集成 ----

    fn m7_engine() -> InputEngine {
        use zhu_ye_core::dict::{DictionaryEntry, InMemoryDictionary};
        let dictionary = InMemoryDictionary::from_entries(vec![
            DictionaryEntry::new("你好", "nihao", 100),
            DictionaryEntry::new("我", "wo", 88),
            DictionaryEntry::new("想", "xiang", 75),
            DictionaryEntry::new("明天", "mingtian", 72),
            DictionaryEntry::new("去", "qu", 68),
            DictionaryEntry::new("北京", "beijing", 95),
            DictionaryEntry::new("为什么", "weishenme", 80),
            DictionaryEntry::new("中国", "zhongguo", 92),
            DictionaryEntry::new("难", "nan", 50),
        ]);
        InputEngine::with_bigram(Arc::new(dictionary), Arc::new(InMemoryBigramModel::new()))
    }

    #[test]
    fn 简拼nh出你好主候选() {
        let mut engine = m7_engine();
        type_text(&mut engine, "nh");
        let texts: Vec<&str> = engine
            .candidates()
            .iter()
            .map(|c| c.text.as_str())
            .collect();
        assert!(texts.contains(&"你好"), "nh 简拼应出你好，实际: {texts:?}");
    }

    #[test]
    fn 简拼wsm出为什么() {
        let mut engine = m7_engine();
        type_text(&mut engine, "wsm");
        let texts: Vec<&str> = engine
            .candidates()
            .iter()
            .map(|c| c.text.as_str())
            .collect();
        assert!(
            texts.contains(&"为什么"),
            "wsm 简拼应出为什么，实际: {texts:?}"
        );
    }

    #[test]
    fn 可切分输入不触发简拼噪声() {
        let mut engine = m7_engine();
        type_text(&mut engine, "wo");
        let texts: Vec<&str> = engine
            .candidates()
            .iter()
            .map(|c| c.text.as_str())
            .collect();
        // wo 应正常出整词候选「我」，不被简拼展开污染成「我哦」等。
        assert!(
            engine.candidates().iter().any(|c| c.text == "我"),
            "wo 应出我，实际: {texts:?}"
        );
    }

    #[test]
    fn 模糊音zongguo纠错出中国() {
        let mut engine = m7_engine();
        type_text(&mut engine, "zongguo");
        let candidates = engine.candidates();
        let zhongguo = candidates
            .iter()
            .find(|c| c.text == "中国")
            .expect("zongguo 应纠错出中国");
        assert_eq!(
            zhongguo.source,
            zhu_ye_core::candidate::CandidateSource::Corrected
        );
    }

    #[test]
    fn 少字母niha纠错出你好() {
        let mut engine = m7_engine();
        type_text(&mut engine, "niha");
        let candidates = engine.candidates();
        assert!(
            candidates.iter().any(|c| c.text == "你好"),
            "niha 应补全 ha→hao 出你好，实际: {:?}",
            candidates
                .iter()
                .map(|c| c.text.as_str())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn 整词命中不触发纠错() {
        let mut engine = m7_engine();
        type_text(&mut engine, "nihao");
        let candidates = engine.candidates();
        assert!(
            candidates
                .iter()
                .all(|c| c.source != zhu_ye_core::candidate::CandidateSource::Corrected),
            "nihao 整词命中不应出现纠错候选"
        );
    }

    // ---- T-103（O-05 修订）：简拼/模糊音关闭开关 ----

    #[test]
    fn 简拼关闭后nh不展开() {
        // 默认开（既有 2328 覆盖）；关闭后主候选为空的不可切分字母不再首字母展开。
        let mut engine = m7_engine().with_abbreviation(false);
        type_text(&mut engine, "nh");
        let texts: Vec<&str> = engine
            .candidates()
            .iter()
            .map(|c| c.text.as_str())
            .collect();
        assert!(
            !texts.contains(&"你好"),
            "关闭简拼后 nh 不应出你好，实际: {texts:?}"
        );
    }

    #[test]
    fn 简拼关闭不影响整词候选() {
        // 关闭简拼只关首字母展开路径：整词拼音候选与纠错路径行为不变。
        let mut engine = m7_engine().with_abbreviation(false);
        type_text(&mut engine, "nihao");
        assert!(engine.candidates().iter().any(|c| c.text == "你好"));
    }

    #[test]
    fn 模糊音关闭后zongguo不纠错() {
        // 关闭模糊音与纠错：zongguo 不再经模糊替换出中国（无可切分则候选为空）。
        let mut engine = m7_engine().with_fuzzy(false);
        type_text(&mut engine, "zongguo");
        let candidates = engine.candidates();
        assert!(
            candidates.iter().all(|c| c.text != "中国"
                && c.source != zhu_ye_core::candidate::CandidateSource::Corrected),
            "关闭模糊音后 zongguo 不应纠错出中国，实际: {:?}",
            candidates
                .iter()
                .map(|c| c.text.as_str())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn 模糊音关闭后niha不补全() {
        // 少字母补全（B 类）随模糊音开关一并关闭：niha 不再出你好。
        let mut engine = m7_engine().with_fuzzy(false);
        type_text(&mut engine, "niha");
        let candidates = engine.candidates();
        assert!(
            candidates
                .iter()
                .all(|c| c.source != zhu_ye_core::candidate::CandidateSource::Corrected),
            "关闭模糊音后 niha 不应出纠错候选，实际: {:?}",
            candidates
                .iter()
                .map(|c| c.text.as_str())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn 简拼关闭不影响联系人简拼键() {
        // 索引原生简拼键（FR-037）不随 enable_abbreviation 变化：边界——
        // zs → 张三 仍可达（直接命中联系人索引，不经词典首字母展开路径）。
        let engine = InputEngine::with_m1_seed()
            .with_abbreviation(false)
            .with_contacts(zhu_ye_core::build_contact_index(&[
                zhu_ye_core::VCardContact::new("张三"),
            ]));
        let mut engine = engine;
        type_text(&mut engine, "zs");
        assert!(
            engine.candidates().iter().any(|c| c.text == "张三"
                && c.source == zhu_ye_core::candidate::CandidateSource::Contact),
            "关闭简拼后联系人简拼键仍应命中"
        );
    }

    #[test]
    fn 整句输入整句组居首() {
        let mut engine = m7_engine();
        type_text(&mut engine, "woxiangmingtianqubeijing");
        let candidates = engine.candidates();
        assert!(
            candidates.iter().any(|c| c.text == "我想明天去北京"),
            "长串应出整句，实际: {:?}",
            candidates
                .iter()
                .map(|c| c.text.as_str())
                .collect::<Vec<_>>()
        );
        // 整句组置于列表前部（首个候选即整句）。
        assert_eq!(candidates[0].text, "我想明天去北京", "整句应居首");
    }

    #[test]
    fn 短串不启动整句路径() {
        let mut engine = m7_engine();
        type_text(&mut engine, "nihao");
        let texts: Vec<&str> = engine
            .candidates()
            .iter()
            .map(|c| c.text.as_str())
            .collect();
        assert_eq!(texts, vec!["你好"], "nihao 整词命中，候选只有整词本身");
    }

    // ---- 场景7（T-061）：数字格式候选 / v 模式 / emoji 推荐 ----

    fn digit_engine() -> InputEngine {
        engine()
    }

    #[test]
    fn 数字模式累积并刷新日期候选() {
        let mut eng = digit_engine();
        assert!(!eng.digit_active());
        for c in ['2', '0', '2', '6', '0', '9', '3', '0'] {
            assert!(eng.digit_append(c));
        }
        assert!(eng.digit_active());
        assert!(eng.composing().is_empty(), "数字模式不应出现组合串");
        let texts: Vec<&str> = eng.candidates().iter().map(|c| c.text.as_str()).collect();
        assert_eq!(
            texts,
            vec!["2026-09-30", "2026/09/30", "2026年9月30日", "2026.09.30"]
        );
        assert_eq!(
            eng.candidates()[0].source,
            zhu_ye_core::candidate::CandidateSource::NumberFormat
        );
    }

    #[test]
    fn 数字模式选中格式返回替换长度() {
        let mut eng = digit_engine();
        for c in "20260930".chars() {
            eng.digit_append(c);
        }
        let (text, replace_len) = eng.preview_digit(1).expect("第 2 个日期候选");
        assert_eq!(text, "2026/09/30");
        assert_eq!(replace_len, 8, "替换长度为 buffer 的 UTF-16 长度");
        let committed = eng.commit_digit(1).expect("提交");
        assert_eq!(committed.0, "2026/09/30");
        assert!(!eng.digit_active(), "提交后退出数字模式");
        assert!(eng.candidates().is_empty());
    }

    #[test]
    fn 数字模式选中越界返回空且状态保持() {
        let mut eng = digit_engine();
        for c in "20260930".chars() {
            eng.digit_append(c);
        }
        assert_eq!(eng.preview_digit(9), None);
        assert_eq!(eng.select_index(9), None);
        assert!(eng.digit_active(), "越界选择不应退出数字模式");
    }

    #[test]
    fn 数字模式不足五位无候选() {
        let mut eng = digit_engine();
        for c in "12".chars() {
            eng.digit_append(c);
        }
        assert!(eng.digit_active());
        assert!(eng.candidates().is_empty(), "12 不应触发格式候选");
    }

    #[test]
    fn 数字模式退格与退出() {
        let mut eng = digit_engine();
        for c in "20260930".chars() {
            eng.digit_append(c);
        }
        assert!(eng.digit_backspace());
        assert_eq!(eng.digit_text(), "2026093");
        assert!(eng.handle_backspace(), "引擎层退格路由到数字模式");
        assert_eq!(eng.digit_text(), "202609");
        assert!(eng.handle_escape());
        assert!(!eng.digit_active(), "Esc 退出数字模式");
        assert!(eng.candidates().is_empty());
    }

    #[test]
    fn 数字模式空格空格选第一个格式() {
        let mut eng = digit_engine();
        for c in "20260930".chars() {
            eng.digit_append(c);
        }
        let text = eng.handle_space().expect("空格应选中第 0 项");
        assert_eq!(text, "2026-09-30");
        assert!(!eng.digit_active());
    }

    #[test]
    fn 金额与电话格式候选() {
        let mut eng = digit_engine();
        for c in ['1', '2', '3', '4', '5', '.', '6'] {
            assert!(eng.digit_append(c), "数字模式接受小数点点位（金额）");
        }
        let texts: Vec<&str> = eng.candidates().iter().map(|c| c.text.as_str()).collect();
        assert_eq!(texts, vec!["12,345.6", "一万二千三百四十五点六"]);

        let mut phone = digit_engine();
        for c in "13800138000".chars() {
            phone.digit_append(c);
        }
        let texts: Vec<&str> = phone.candidates().iter().map(|c| c.text.as_str()).collect();
        assert_eq!(texts, vec!["138 0013 8000", "138-0013-8000"]);
    }

    #[test]
    fn 字母进入拼音组合自动退出数字模式() {
        let mut eng = digit_engine();
        for c in "20260930".chars() {
            eng.digit_append(c);
        }
        assert!(eng.handle_letter('n'));
        assert!(!eng.digit_active(), "字母进入组合应退出数字模式");
        assert_eq!(eng.composing(), "n");
    }

    #[test]
    fn 组合态数字不进数字模式() {
        let mut eng = digit_engine();
        type_text(&mut eng, "niha");
        assert!(!eng.digit_active());
        // 组合态数字属于网络语缩写前缀判定后走选词/组合，不启动数字模式。
        assert!(!eng.digit_append('9'));
    }

    #[test]
    fn v模式启动与符号组() {
        let mut eng = engine();
        assert!(eng.v_start());
        assert!(eng.v_active());
        assert_eq!(eng.v_buffer_len(), 1, "只有 v 时等待类型码");
        assert!(eng.v_code('1'));
        assert_eq!(eng.v_buffer_len(), 2);
        let texts: Vec<&str> = eng.candidates().iter().map(|c| c.text.as_str()).collect();
        assert_eq!(texts.len(), 9);
        assert_eq!(texts[0], "①");
        assert_eq!(
            eng.candidates()[0].source,
            zhu_ye_core::candidate::CandidateSource::Symbol
        );
    }

    #[test]
    fn v模式数学与标点组() {
        let mut eng = engine();
        eng.v_start();
        eng.v_code('x');
        let math: Vec<&str> = eng.candidates().iter().map(|c| c.text.as_str()).collect();
        assert_eq!(math[0], "±");
        assert!(math.contains(&"∞"));

        eng.v_backspace();
        assert_eq!(eng.v_buffer_len(), 1);
        eng.v_code('h');
        let punct: Vec<&str> = eng.candidates().iter().map(|c| c.text.as_str()).collect();
        assert_eq!(punct[0], "，");
    }

    #[test]
    fn v模式非法字母回退拼音() {
        let mut eng = engine();
        eng.v_start();
        assert!(eng.v_consume('i'));
        assert!(!eng.v_active(), "vi 应退出 v 模式");
        assert_eq!(eng.composing(), "vi", "v+i 交给拼音组合");
    }

    // ---- T-104（v 模式单位换算全量表）----

    #[test]
    fn v模式单位键前缀留在模式内() {
        let mut eng = engine();
        eng.v_start();
        assert!(eng.v_accepts('m'), "m 是单位键前缀，应留在 v 模式");
        assert!(eng.v_code('m'));
        assert_eq!(eng.v_buffer_len(), 2);
        // 前缀未完：候选为空但模式保持。
        assert!(eng.candidates().is_empty());
        assert!(eng.v_active());
        assert!(eng.v_accepts('i'), "mi 完整键仍接受（等于前缀）");
        assert!(eng.v_code('i'));
        assert_eq!(eng.v_buffer_len(), 3);
    }

    #[test]
    fn v模式单位键出换算候选() {
        let mut eng = engine();
        eng.v_start();
        for c in "mi".chars() {
            assert!(eng.v_code(c));
        }
        let texts: Vec<&str> = eng.candidates().iter().map(|c| c.text.as_str()).collect();
        assert_eq!(texts[0], "1 米 = 10 分米");
        assert!(texts.contains(&"1 米 = 0.001 千米"));
        assert_eq!(
            eng.candidates()[0].source,
            zhu_ye_core::candidate::CandidateSource::Symbol
        );
        assert!(eng.v_symbol_count() <= 9, "单位换算候选不超过一页");
    }

    #[test]
    fn v模式市斤与温度键() {
        let mut eng = engine();
        eng.v_start();
        for c in "jin".chars() {
            eng.v_code(c);
        }
        let texts: Vec<&str> = eng.candidates().iter().map(|c| c.text.as_str()).collect();
        assert!(texts.contains(&"1 市斤 = 500 克"));

        eng.v_exit();
        eng.v_start();
        for c in "she".chars() {
            eng.v_code(c);
        }
        let texts: Vec<&str> = eng.candidates().iter().map(|c| c.text.as_str()).collect();
        assert!(texts.contains(&"0 摄氏度 = 32 华氏度 = 273.15 开尔文"));
    }

    #[test]
    fn v模式前缀续输秒与回退() {
        let mut eng = engine();
        eng.v_start();
        // vmi 完整键=米；继续 vmia/vmiao 得到秒键。
        for c in "mia".chars() {
            eng.v_code(c);
        }
        assert!(eng.v_active());
        assert!(eng.candidates().is_empty(), "mia 未完成不出候选");
        eng.v_code('o');
        let texts: Vec<&str> = eng.candidates().iter().map(|c| c.text.as_str()).collect();
        assert!(texts.contains(&"1 秒 = 1000 毫秒"));
        // 退格返回 v，恢复等待类型码。
        eng.v_backspace();
        eng.v_backspace();
        eng.v_backspace();
        eng.v_backspace();
        assert_eq!(eng.v_buffer_len(), 1, "退格逐步回到纯 v");
        assert!(eng.candidates().is_empty(), "纯 v 无候选");
    }

    #[test]
    fn v模式单位码后非法字母回退拼音() {
        let mut eng = engine();
        eng.v_start();
        eng.v_code('m');
        // "vmx"：x 是符号码但不是 "mx" 前缀 → 回退拼音。
        assert!(!eng.v_accepts('x') || !eng.v_active());
        let mut eng2 = engine();
        eng2.v_start();
        for c in "m".chars() {
            eng2.v_code(c);
        }
        assert!(eng2.v_consume('x'), "非前缀字母走 v_consume");
        assert!(!eng2.v_active());
        assert_eq!(eng2.composing(), "vmx");
    }

    #[test]
    fn v模式符号码后不接受单位字母() {
        let mut eng = engine();
        eng.v_start();
        eng.v_code('1');
        assert!(!eng.v_accepts('m'), "v1 已定符号码，不再收单位字母");
        assert!(!eng.v_code('m'), "v_code 拒绝符号码后的单位字母");
        let texts: Vec<&str> = eng.candidates().iter().map(|c| c.text.as_str()).collect();
        assert_eq!(texts.len(), 9, "v1 候选保持序号组");
    }

    #[test]
    fn v模式组合态不启动() {
        let mut eng = engine();
        type_text(&mut eng, "nv");
        assert!(!eng.v_active(), "组合态 v 属于 nv/lv 拼音");
        assert!(!eng.v_start());
    }

    #[test]
    fn v模式选符号上屏() {
        let mut eng = engine();
        eng.v_start();
        eng.v_code('1');
        let text = eng.select_index(2).expect("选择第 3 个符号");
        assert_eq!(text, "③");
        assert!(!eng.v_active());
        assert_eq!(eng.previous_word(), Some("③"));
    }

    #[test]
    fn v模式空格选首个符号() {
        let mut eng = engine();
        eng.v_start();
        eng.v_code('1');
        let text = eng.handle_space().expect("空格选第 0 个符号");
        assert_eq!(text, "①");
        assert!(!eng.v_active());
    }

    #[test]
    fn v模式退出清空() {
        let mut eng = engine();
        eng.v_start();
        eng.v_code('x');
        assert!(eng.handle_escape());
        assert!(!eng.v_active());
        assert!(eng.candidates().is_empty());
        // 只有 v 时退格 = 退出。
        eng.v_start();
        assert!(eng.handle_backspace());
        assert!(!eng.v_active());
    }

    #[test]
    fn 联想态不启动v模式与数字模式() {
        // D-05：联想优先——上屏联想活跃时 v/数字不进入各自模式。
        let mut eng = suggestion_engine();
        commit_nihao(&mut eng);
        assert!(eng.suggestion_active());
        assert!(!eng.v_start(), "联想态 v 不应启动 v 模式");
        assert!(!eng.digit_append('9'), "联想态数字不应进入数字模式");
        assert!(eng.suggestion_active(), "联想候选保持");
    }

    #[test]
    fn emoji队尾追加不改变既有候选() {
        let mut eng = engine();
        type_text(&mut eng, "ai");
        let normal = eng
            .candidates()
            .iter()
            .any(|c| c.text == "爱" && c.source != zhu_ye_core::candidate::CandidateSource::Emoji);
        assert!(normal, "ai 的普通拼音候选（爱）保留");
        let first = eng.candidates()[0].text.clone();
        let last = eng.candidates().last().expect("应有候选");
        assert_eq!(last.text, "❤️", "emoji 追在队尾");
        assert_eq!(last.source, zhu_ye_core::candidate::CandidateSource::Emoji);
        // 队首候选不受 emoji 追加影响（T-057 不回退前提）。
        assert_eq!(eng.candidates()[0].text, first);
    }

    #[test]
    fn emoji不命中的拼音无追加() {
        let mut eng = engine();
        type_text(&mut eng, "nihao");
        assert!(
            eng.candidates()
                .iter()
                .all(|c| c.source != zhu_ye_core::candidate::CandidateSource::Emoji),
            "nihao 无别名命中，不应追加 emoji"
        );
    }

    #[test]
    fn 数字模式候选窗视图() {
        let mut eng = digit_engine();
        for c in "20260930".chars() {
            eng.digit_append(c);
        }
        let view = eng.candidate_ui_view();
        assert!(view.composition.is_empty());
        assert_eq!(view.pinyin_hint, "20260930");
        assert_eq!(view.visible_items().len(), 4);
        assert_eq!(
            view.visible_items()[1].text,
            "2026/09/30",
            "候选窗第 2 项为 / 分隔日期"
        );
    }

    #[test]
    fn v模式候选窗视图() {
        let mut eng = engine();
        eng.v_start();
        eng.v_code('1');
        let view = eng.candidate_ui_view();
        assert!(view.composition.is_empty());
        assert_eq!(view.pinyin_hint, "v1");
        assert_eq!(view.visible_items().len(), 9);
        assert_eq!(view.visible_items()[0].text, "①");
    }

    // ---------- 场景6（中英混输，T-065）引擎层测试 ----------

    fn type_format(engine: &mut InputEngine, text: &str) {
        for c in text.chars() {
            let ok = match c {
                'a'..='z' => engine.handle_letter(c),
                '0'..='9' => engine.handle_digit(c),
                _ => engine.handle_format_char(c),
            };
            assert!(ok);
        }
    }

    #[test]
    fn 英文拼写命中出原形候选() {
        let mut eng = engine();
        type_text(&mut eng, "pytho");
        let en = eng
            .candidates()
            .iter()
            .find(|c| c.source == zhu_ye_core::candidate::CandidateSource::EnWord)
            .expect("pytho 应命中英文候选组");
        assert_eq!(en.text, "python");
        assert!(en.pinyin.is_none(), "英文候选不进用户词学习");
    }

    #[test]
    fn 英文大小写原形保留() {
        let mut eng = engine();
        type_text(&mut eng, "iphon");
        let en = eng
            .candidates()
            .iter()
            .find(|c| c.source == zhu_ye_core::candidate::CandidateSource::EnWord)
            .expect("iphon 应命中英文候选组");
        assert_eq!(en.text, "iPhone");
    }

    #[test]
    fn 可切分拼音不进入英文路径() {
        let mut eng = engine();
        type_text(&mut eng, "nihao");
        assert!(
            eng.candidates()
                .iter()
                .all(|c| c.source != zhu_ye_core::candidate::CandidateSource::EnWord),
            "可切分整串 nihao 不得触发英文路径（D-10）"
        );
        type_text(&mut eng, "wo");
        assert!(
            eng.candidates()
                .iter()
                .all(|c| c.source != zhu_ye_core::candidate::CandidateSource::EnWord),
            "可切分串 wo 不得触发英文路径（D-10）"
        );
    }

    #[test]
    fn 英文未命中时缩写组不回退() {
        let mut eng = slang_engine();
        type_text(&mut eng, "yyds");
        assert!(
            eng.candidates()
                .iter()
                .all(|c| c.source != zhu_ye_core::candidate::CandidateSource::EnWord),
            "yyds 无英文命中，不得出现英文组"
        );
        let found = eng.candidates().iter().any(|c| {
            c.text == "永远的神" && c.source == zhu_ye_core::candidate::CandidateSource::Slang
        });
        assert!(found, "yyds 缩写行为不得回退");
    }

    #[test]
    fn 邮箱补全候选与直通() {
        let mut eng = engine();
        type_format(&mut eng, "me@163");
        let texts: Vec<&str> = eng.candidates().iter().map(|c| c.text.as_str()).collect();
        assert_eq!(texts, vec!["me@163.com", "me@163.cn", "me@163.net"]);
        assert!(
            eng.candidates().iter().all(|c| c.source
                == zhu_ye_core::candidate::CandidateSource::EmailUrl
                && c.pinyin.is_none()),
            "邮箱候选来源 EmailUrl 且不进学习"
        );
        // 已含点：完整串直通，不重复补全
        let mut eng2 = engine();
        type_format(&mut eng2, "a@b.c");
        assert_eq!(eng2.candidates().len(), 1);
        assert_eq!(eng2.candidates()[0].text, "a@b.c");
    }

    #[test]
    fn 网址补全候选与直通() {
        let mut eng = engine();
        type_format(&mut eng, "www.exa");
        let texts: Vec<&str> = eng.candidates().iter().map(|c| c.text.as_str()).collect();
        assert_eq!(texts, vec!["www.exa.com", "www.exa.cn", "www.exa.org"]);
        let mut eng2 = engine();
        type_format(&mut eng2, "http://exa");
        let texts2: Vec<&str> = eng2.candidates().iter().map(|c| c.text.as_str()).collect();
        assert_eq!(
            texts2,
            vec!["http://exa.com", "http://exa.cn", "http://exa.org"]
        );
        // 已含点：直通
        let mut eng3 = engine();
        type_format(&mut eng3, "www.exa.com");
        assert_eq!(eng3.candidates().len(), 1);
        assert_eq!(eng3.candidates()[0].text, "www.exa.com");
    }

    #[test]
    fn 邮箱提交与退出() {
        let mut eng = engine();
        type_format(&mut eng, "me@163");
        let committed = eng.select_index(0).expect("应能选首候选上屏");
        assert_eq!(committed, "me@163.com");
        assert!(!eng.is_active(), "提交后组合清空");
        // Esc 放弃整串回空闲
        let mut eng2 = engine();
        type_format(&mut eng2, "me@163");
        assert!(eng2.handle_escape());
        assert!(!eng2.is_active());
        // 退格逐步回拼音：删掉 @ 后退出邮箱态（me 是可切分拼音音节，按 D-10 不介入英文）
        let mut eng3 = engine();
        type_format(&mut eng3, "me@163");
        for _ in 0..4 {
            assert!(eng3.handle_backspace());
        }
        assert_eq!(eng3.composing(), "me");
        assert!(
            eng3.candidates()
                .iter()
                .all(|c| c.source != zhu_ye_core::candidate::CandidateSource::EmailUrl),
            "@ 删除后应退出邮箱态"
        );
    }

    #[test]
    fn 格式字符空闲态放行宿主() {
        let mut eng = engine();
        assert!(!eng.handle_format_char('@'), "空闲态 @ 不放行进组合");
        assert!(!eng.is_active());
        assert_eq!(eng.composing(), "");
    }

    /// 格式键吃键判定（T-066，`is_format_key`）：
    /// 邮箱/网址上下文吃键进串，普通拼音组合与空闲态放行宿主。
    #[test]
    fn 格式键吃键判定邮箱网址吃键普通拼音放行() {
        let mut eng = engine();
        // 空闲态：全部格式键放行（不冷启动组合）。
        for c in ['@', '.', '/', ':'] {
            assert!(!eng.is_format_key(c), "空闲态 {c} 应放行宿主");
        }
        // 组合态 `@`：一律接收（@ 是邮箱态开关）。
        type_text(&mut eng, "me");
        assert!(eng.is_format_key('@'));
        // 邮箱态：`.` 吃键；普通拼音组合：`.` `/` `:` 放行。
        let mut mail = engine();
        type_format(&mut mail, "me@16");
        assert!(mail.is_format_key('.'));
        let mut nihao = engine();
        type_text(&mut nihao, "nihao");
        for c in ['.', '/', ':'] {
            assert!(!nihao.is_format_key(c), "普通拼音组合 {c} 应放行宿主");
        }
        // 网址意图演进：`www` 后 `.` 吃键；`http` 后 `:`/`/` 吃键。
        let mut www = engine();
        type_text(&mut www, "www");
        assert!(www.is_format_key('.'));
        let mut scheme = engine();
        type_text(&mut scheme, "http");
        assert!(scheme.is_format_key(':'));
        assert!(scheme.is_format_key('/'));
        // 结构相似但非网址意图（httpw）：`:` `/` 放行。
        let mut bad = engine();
        type_text(&mut bad, "httpw");
        for c in [':', '/'] {
            assert!(!bad.is_format_key(c), "httpw 的 {c} 应放行宿主");
        }
        // 演进中间态 `http:` 后 `/` 仍吃键。
        let mut mid = engine();
        type_format(&mut mid, "http:");
        assert!(mid.is_format_key('/'));
    }

    #[test]
    fn 格式键英文模式全部放行() {
        let mut eng = engine();
        eng.toggle_mode();
        // 英文模式下组合不成立（handle_letter 拒收），is_format_key 直接放行
        // 所有格式键，由宿主直出标点。
        for c in ['@', '.', '/', ':'] {
            assert!(!eng.is_format_key(c), "英文模式 {c} 应放行宿主");
        }
    }

    // ---------- 场景8（领域自动，T-070）引擎层测试 ----------

    fn domain_engine() -> InputEngine {
        let it: Arc<dyn Dictionary> =
            Arc::new(zhu_ye_core::InMemoryDictionary::from_entries(vec![
                zhu_ye_core::DictionaryEntry::new("拟", "ni", 300),
                zhu_ye_core::DictionaryEntry::new("队列", "duilie", 900),
                zhu_ye_core::DictionaryEntry::new("局域网", "juyuwang", 500),
            ]));
        let med: Arc<dyn Dictionary> =
            Arc::new(zhu_ye_core::InMemoryDictionary::from_entries(vec![
                zhu_ye_core::DictionaryEntry::new("队列研究", "duilieyanjiu", 700),
                zhu_ye_core::DictionaryEntry::new("你学", "nixue", 600),
            ]));
        // it < med（字典序），多包同命中时取 it（D-16）。
        InputEngine::with_m1_seed()
            .with_domain_packs(vec![("it".to_owned(), it), ("med".to_owned(), med)])
    }

    #[test]
    fn 领域整词命中提权到主候选之后() {
        let mut eng = domain_engine();
        type_text(&mut eng, "ni");
        let texts: Vec<&str> = eng.candidates().iter().map(|c| c.text.as_str()).collect();
        // 基础候选（你，seed 最频 ni 词）保留在首；领域候选（拟，it 包整词命中）在其后
        // （D-13 位次：基础候选之后）。
        assert_eq!(
            texts.first(),
            Some(&"你"),
            "基础首候选不得被覆盖: {texts:?}"
        );
        let domain_idx = texts
            .iter()
            .position(|t| *t == "拟")
            .expect("ni 整词命中 it 包，拟应被提权");
        assert!(domain_idx >= 1, "领域候选必须插在基础候选之后: {texts:?}");
        for c in &eng.candidates()[..domain_idx] {
            assert_ne!(
                c.source,
                zhu_ye_core::candidate::CandidateSource::Domain,
                "领域候选之前不得出现其他领域候选"
            );
        }
        let domain_candidate = eng
            .candidates()
            .iter()
            .find(|c| c.text == "拟")
            .expect("拟 候选存在");
        assert_eq!(
            domain_candidate.source,
            zhu_ye_core::candidate::CandidateSource::Domain
        );
        assert_eq!(domain_candidate.pinyin.as_deref(), Some("ni"));
    }

    #[test]
    fn 领域整词命中独立词条也提权() {
        // seed 无 dui/lie 音节词时基础候选为空，领域候选允许成为唯一/首位候选
        // （没有可覆盖的基础候选，D-13 的"之后"无从谈起）。
        let mut eng = domain_engine();
        type_text(&mut eng, "duilie");
        let texts: Vec<&str> = eng.candidates().iter().map(|c| c.text.as_str()).collect();
        let domain_idx = texts
            .iter()
            .position(|t| *t == "队列")
            .expect("duilie 整词命中 it 包，队列应被提权");
        assert_eq!(domain_idx, 0, "基础候选为空时领域候选居首: {texts:?}");
        // 队列调度未整词命中（duiliediaodu ≠ duilie），不进提权。
        assert!(!texts.contains(&"队列调度"));
        // 多包同 pinyin 情况：it 的队列（非 med 的队列研究）优先——pinyin 不同整词
        // 命中只有 it（队列），med 的 duilieyanjiu 不命中 duilie。
        let domain_candidates: Vec<_> = eng
            .candidates()
            .iter()
            .filter(|c| c.source == zhu_ye_core::candidate::CandidateSource::Domain)
            .collect();
        assert_eq!(domain_candidates.len(), 1);
        assert_eq!(domain_candidates[0].text, "队列");
        assert_eq!(domain_candidates[0].pinyin.as_deref(), Some("duilie"));
    }

    #[test]
    fn 前缀与无命中不触发提权() {
        // D-15：前缀不参与提权。
        let mut prefix = domain_engine();
        type_text(&mut prefix, "dui");
        assert!(
            prefix
                .candidates()
                .iter()
                .all(|c| c.source != zhu_ye_core::candidate::CandidateSource::Domain),
            "前缀 dui 不得触发领域提权"
        );
        // 组合模糊：duilian 既有领域词（duilieyanjiu 前缀）也应仅在整词命中时提权。
        let mut fuzzy = domain_engine();
        type_text(&mut fuzzy, "duilian");
        assert!(
            fuzzy
                .candidates()
                .iter()
                .all(|c| c.source != zhu_ye_core::candidate::CandidateSource::Domain),
            "非整词命中不得提权"
        );
        // 无领域命中：nihao 与现版（m1 seed）一致，且无 Domain 候选。
        let mut plain = domain_engine();
        type_text(&mut plain, "nihao");
        assert!(plain
            .candidates()
            .iter()
            .all(|c| c.source != zhu_ye_core::candidate::CandidateSource::Domain));
        let mut baseline = engine();
        type_text(&mut baseline, "nihao");
        assert_eq!(plain.candidates(), baseline.candidates(), "无命中不漂移");
    }

    #[test]
    fn 提权开关关闭恢复追加语义() {
        let mut off = domain_engine().with_domain_boost(false);
        type_text(&mut off, "duilie");
        let texts: Vec<&str> = off.candidates().iter().map(|c| c.text.as_str()).collect();
        assert!(
            !texts.contains(&"队列"),
            "关闭提权后领域词不得上移（恢复 T-050 追加语义）: {texts:?}"
        );
    }

    #[test]
    fn 领域提权确定性() {
        let mut first = domain_engine();
        let mut second = domain_engine();
        type_text(&mut first, "duilie");
        type_text(&mut second, "duilie");
        assert_eq!(
            first.candidates(),
            second.candidates(),
            "同一输入两次逐位一致"
        );
    }

    // ---- 前缀组词展开（FR-059，T-090，第十一期）----

    fn shui_engine() -> InputEngine {
        use zhu_ye_core::dict::{DictionaryEntry, InMemoryDictionary};
        // 真实词典口径：shui 整词命中 4 条（低频音节，不足一页），
        // 前缀更深组词 8 条可补足。
        let dictionary = InMemoryDictionary::from_entries(vec![
            DictionaryEntry::new("说", "shui", 413852),
            DictionaryEntry::new("睡", "shui", 80000),
            DictionaryEntry::new("水", "shui", 60000),
            DictionaryEntry::new("水稻", "shuidao", 120000),
            DictionaryEntry::new("水果", "shuiguo", 90000),
            DictionaryEntry::new("水平", "shuiping", 85000),
            DictionaryEntry::new("睡觉", "shuijiao", 70000),
            DictionaryEntry::new("水面", "shuimian", 65000),
            DictionaryEntry::new("水灾", "shuizai", 64000),
            DictionaryEntry::new("水分", "shuifen", 63000),
            DictionaryEntry::new("水彩", "shuicai", 62000),
            DictionaryEntry::new("水波", "shuibo", 61000),
        ]);
        InputEngine::new(Arc::new(dictionary))
    }

    #[test]
    fn 前缀组词展开补足低频音节到一页() {
        let mut engine = shui_engine();
        type_text(&mut engine, "shui");
        let candidates = engine.candidates();
        // 排除 emoji 队尾追加（shui→😴 属 FR-029 既有行为，非展开组）。
        let non_emoji: Vec<&zhu_ye_core::Candidate> = candidates
            .iter()
            .filter(|c| c.source != zhu_ye_core::candidate::CandidateSource::Emoji)
            .collect();
        let texts: Vec<&str> = non_emoji.iter().map(|c| c.text.as_str()).collect();
        // 主组序原样（rank 后频率降序），展开组接其后不参与主排序。
        assert_eq!(
            texts[..3],
            ["说", "睡", "水"],
            "整词命中组序不得被展开改变: {texts:?}"
        );
        // 展开组按词频降序补足到 9 条（水稻/水果/水平/睡觉/水面/水灾）。
        assert_eq!(
            texts[3..],
            ["水稻", "水果", "水平", "睡觉", "水面", "水灾"],
            "展开组词频降序补足: {texts:?}"
        );
        assert!(
            non_emoji[3..]
                .iter()
                .all(|c| c.source == zhu_ye_core::candidate::CandidateSource::PrefixExpand),
            "展开组来源必须为 PrefixExpand"
        );
        assert_eq!(texts.len(), 9, "首屏补足到一页");
        let mut seen = std::collections::HashSet::new();
        assert!(texts.iter().all(|t| seen.insert(*t)), "展开后无重复");
    }

    #[test]
    fn 常见拼音候选已满一页不展开() {
        use zhu_ye_core::dict::{DictionaryEntry, InMemoryDictionary};
        // nihao 整词命中 9 条：候选已满一页 → 零展开（D-70），列表逐位与基线一致。
        let words = [
            "你好", "妮好", "尼好", "泥好", "你号", "拟好", "匿好", "逆好", "腻好",
        ];
        let entries: Vec<DictionaryEntry> = words
            .iter()
            .enumerate()
            .map(|(i, word)| DictionaryEntry::new(*word, "nihao", 5000 - i as u64))
            .collect();
        let mut engine = InputEngine::new(Arc::new(InMemoryDictionary::from_entries(entries)));
        type_text(&mut engine, "nihao");
        assert!(
            engine
                .candidates()
                .iter()
                .all(|c| c.source != zhu_ye_core::candidate::CandidateSource::PrefixExpand),
            "候选已满一页时不得展开"
        );
        assert_eq!(engine.candidates().len(), 9);
    }

    #[test]
    fn 前缀组词展开按页大小补足() {
        // 注入小页验证 fill 计算：main=3、page_size=5 → fill=2。
        let mut engine = shui_engine();
        engine.page_size = 5;
        type_text(&mut engine, "shui");
        let texts: Vec<&str> = engine
            .candidates()
            .iter()
            .filter(|c| c.source != zhu_ye_core::candidate::CandidateSource::Emoji)
            .map(|c| c.text.as_str())
            .collect();
        assert_eq!(texts[..3], ["说", "睡", "水"]);
        assert_eq!(texts[3..], ["水稻", "水果"], "fill=2 只补 2 条: {texts:?}");
        assert_eq!(texts.len(), 5);
    }

    #[test]
    fn 不完整拼音仍走前缀补全不展开() {
        let mut engine = shui_engine();
        type_text(&mut engine, "shuip");
        let candidates = engine.candidates();
        assert!(
            candidates
                .iter()
                .all(|c| c.source != zhu_ye_core::candidate::CandidateSource::PrefixExpand),
            "不完整拼音 shuip 走既有前缀补全路径（FR-023），不得进入展开组"
        );
        assert!(
            candidates.iter().any(|c| c.text == "水平"),
            "shuip 前缀补全应出水平"
        );
    }

    #[test]
    fn 领域包词被展开截获不重复() {
        // 领域包含 shui 键词「谁」：展开组（Composite 前缀查询）会提前截获该词，
        // 领域提权 append 同文本去重 → 全文只出现一次（D-13 位次语义等价的展示）。
        use zhu_ye_core::dict::{DictionaryEntry, InMemoryDictionary};
        let base = InMemoryDictionary::from_entries(vec![
            DictionaryEntry::new("说", "shui", 413852),
            DictionaryEntry::new("睡", "shui", 80000),
            DictionaryEntry::new("水", "shui", 60000),
            DictionaryEntry::new("水稻", "shuidao", 120000),
            DictionaryEntry::new("水果", "shuiguo", 90000),
            DictionaryEntry::new("水平", "shuiping", 85000),
            DictionaryEntry::new("睡觉", "shuijiao", 70000),
        ]);
        let it = InMemoryDictionary::from_entries(vec![DictionaryEntry::new("谁", "shui", 127180)]);
        let mut engine = InputEngine::new(Arc::new(base))
            .with_domain_packs(vec![("it".to_owned(), Arc::new(it) as Arc<dyn Dictionary>)]);
        type_text(&mut engine, "shui");
        let candidates = engine.candidates();
        let count = candidates.iter().filter(|c| c.text == "谁").count();
        assert_eq!(count, 1, "领域词被展开组截获后不得重复: {candidates:?}");
        assert!(
            candidates.iter().filter(|c| c.text == "谁").all(|c| {
                c.source == zhu_ye_core::candidate::CandidateSource::PrefixExpand
                    || c.source == zhu_ye_core::candidate::CandidateSource::Domain
            }),
            "谁 的来源应为 PrefixExpand（被展开截获）或 Domain"
        );
        // 展开组在低频音节仍然生效（Composite 口径含领域包）。
        assert!(candidates
            .iter()
            .any(|c| c.source == zhu_ye_core::candidate::CandidateSource::PrefixExpand));
    }

    #[test]
    fn 领域提权与emoji队尾共存() {
        let mut eng = domain_engine();
        type_text(&mut eng, "duilie");
        // 追加一个 emoji 命中别名验证 D-13 位次：领域候选在 emoji 之前。
        // （duilie 无 emoji 别名则仅验证领域候选已在；此处直接构造别名命中态）
        let domain_last = eng
            .candidates()
            .iter()
            .rposition(|c| c.source == zhu_ye_core::candidate::CandidateSource::Domain);
        let emoji_any = eng
            .candidates()
            .iter()
            .any(|c| c.source == zhu_ye_core::candidate::CandidateSource::Emoji);
        if emoji_any {
            let emoji_pos = eng
                .candidates()
                .iter()
                .position(|c| c.source == zhu_ye_core::candidate::CandidateSource::Emoji)
                .unwrap();
            assert!(
                domain_last.unwrap() < emoji_pos,
                "领域提权在 emoji 追加之前"
            );
        }
        // 别名命中态：xiao → 追加 emoji 后，领域候选仍在其前。
        let mut emoji_eng = domain_engine();
        type_text(&mut emoji_eng, "duilie");
        assert!(emoji_eng.candidates().last().is_some());
    }

    // ---------- 场景9（通讯录，T-071）引擎层测试 ----------

    /// 内存构造联系人引擎：基础 = seed，联系人 = 张三/曾子/Alice。
    fn contact_engine() -> InputEngine {
        let contacts = vec![
            zhu_ye_core::VCardContact::new("张三"),
            zhu_ye_core::VCardContact::new("曾子"),
            zhu_ye_core::VCardContact::new("Alice"),
        ];
        let index = zhu_ye_core::build_contact_index(&contacts);
        InputEngine::with_m1_seed().with_contacts(index)
    }

    #[test]
    fn 联系人全拼前缀可达() {
        let mut eng = contact_engine();
        type_text(&mut eng, "zhang");
        let hit = eng
            .candidates()
            .iter()
            .find(|c| c.text == "张三")
            .expect("zhang 前缀应命中联系人张三");
        assert_eq!(
            hit.source,
            zhu_ye_core::candidate::CandidateSource::Contact,
            "联系人候选来源标记"
        );
        // 位次契约：来源同为提权层的候选必须保持 基础→领域→联系人 顺序；
        // 联系人候选之前（若存在）不得出现英文/缩写等追加组候选（D-21 同层语义）。
        // （seed 无 zhang 基础词时联系人允许居首，与 m12 有基础词场景互补。）
        let contact_idx = eng
            .candidates()
            .iter()
            .position(|c| c.text == "张三")
            .expect("张三在清单中");
        for c in &eng.candidates()[..contact_idx] {
            assert!(
                matches!(
                    c.source,
                    zhu_ye_core::candidate::CandidateSource::Static
                        | zhu_ye_core::candidate::CandidateSource::User
                        | zhu_ye_core::candidate::CandidateSource::Domain
                ),
                "联系人候选之前不得出现追加组候选（D-21）: {c:?}"
            );
        }
    }

    #[test]
    fn 联系人简拼可达() {
        let mut eng = contact_engine();
        type_text(&mut eng, "zs");
        let hit = eng
            .candidates()
            .iter()
            .find(|c| c.text == "张三")
            .expect("简拼 zs 应命中联系人张三");
        assert_eq!(hit.source, zhu_ye_core::candidate::CandidateSource::Contact);
    }

    #[test]
    fn 联系人多音简拼多形态() {
        let mut eng = contact_engine();
        // 曾 = zeng/ceng：zz、cz 两个简拼键均可达（D-22 全形态原则延伸）。
        type_text(&mut eng, "zz");
        assert!(eng
            .candidates()
            .iter()
            .any(|c| c.text == "曾子"
                && c.source == zhu_ye_core::candidate::CandidateSource::Contact));
        let mut cz = contact_engine();
        type_text(&mut cz, "cz");
        assert!(cz
            .candidates()
            .iter()
            .any(|c| c.text == "曾子"
                && c.source == zhu_ye_core::candidate::CandidateSource::Contact));
    }

    #[test]
    fn 联系人英文名原文键可达() {
        let mut eng = contact_engine();
        type_text(&mut eng, "alice");
        let hit = eng
            .candidates()
            .iter()
            .find(|c| c.text == "Alice")
            .expect("alice 原文键应命中联系人 Alice");
        assert_eq!(hit.source, zhu_ye_core::candidate::CandidateSource::Contact);
    }

    #[test]
    fn 联系人无配置基线逐位一致() {
        let mut baseline = InputEngine::with_m1_seed();
        let mut with_contacts =
            InputEngine::with_m1_seed().with_contacts(zhu_ye_core::build_contact_index(&[
                zhu_ye_core::VCardContact::new("欧阳锋"),
            ]));
        // 无配置基线句柄模拟：引擎未挂联系人，输入与联系人无关的串。
        type_text(&mut with_contacts, "nihao");
        type_text(&mut baseline, "nihao");
        let texts: Vec<&str> = with_contacts
            .candidates()
            .iter()
            .map(|c| c.text.as_str())
            .collect();
        let base_texts: Vec<&str> = baseline
            .candidates()
            .iter()
            .map(|c| c.text.as_str())
            .collect();
        assert_eq!(texts, base_texts, "无联系人命中时清单逐位一致");
    }

    #[test]
    fn 联系人清除后恢复基线() {
        let mut eng = contact_engine();
        type_text(&mut eng, "zhang");
        assert!(eng
            .candidates()
            .iter()
            .any(|c| c.text == "张三"
                && c.source == zhu_ye_core::candidate::CandidateSource::Contact));
        eng.clear_contacts();
        eng.handle_escape();
        type_text(&mut eng, "zhang");
        assert!(
            eng.candidates()
                .iter()
                .all(|c| c.source != zhu_ye_core::candidate::CandidateSource::Contact),
            "清除后不得再产出联系人候选"
        );
    }

    // ---- T-131 英译多义候选：分号拆义 / 词性上屏剥离 / Ctrl+数字直上屏 ----

    /// 含多义/词性译文的专用引擎：你好（int. hello; int. hi）、怀疑
    /// （v. suspect; v. doubt）、世界（world）、苹果（n. apple）、测试（无译文）。
    fn multi_sense_engine() -> InputEngine {
        let entries = vec![
            DictionaryEntry::new("你好", "nihao", 100).with_translation("int. hello; int. hi"),
            DictionaryEntry::new("怀疑", "huaiyi", 90).with_translation("v. suspect; v. doubt"),
            DictionaryEntry::new("世界", "shijie", 80).with_translation("world"),
            DictionaryEntry::new("苹果", "pingguo", 70).with_translation("n. apple"),
            DictionaryEntry::new("测试", "ceshi", 60),
        ];
        InputEngine::new(Arc::new(InMemoryDictionary::from_entries(entries)))
    }

    #[test]
    fn 译文层多义拆成多行候选() {
        let mut engine = multi_sense_engine();
        type_text(&mut engine, "nihao");
        assert!(engine.toggle_translation_layer());
        let rows = engine.visible_candidates();
        assert_eq!(rows.len(), 2, "你好 hello; hi 应拆成两行");
        assert_eq!(rows[0].text, "你好");
        assert_eq!(rows[0].translation.as_deref(), Some("int. hello"));
        assert_eq!(rows[1].text, "你好");
        assert_eq!(rows[1].translation.as_deref(), Some("int. hi"));
    }

    #[test]
    fn 译文层选择上屏剥词性前缀() {
        let mut engine = multi_sense_engine();
        type_text(&mut engine, "huaiyi");
        assert!(engine.toggle_translation_layer());
        // 第一行 v. suspect → 上屏剥 v. 前缀。
        assert_eq!(engine.select_index(0).as_deref(), Some("suspect"));
        assert!(!engine.is_active(), "上屏后组合结束");
        assert_eq!(engine.previous_word(), Some("suspect"));
    }

    #[test]
    fn 译文层空格同理剥词性() {
        let mut engine = multi_sense_engine();
        type_text(&mut engine, "pingguo");
        assert!(engine.toggle_translation_layer());
        assert_eq!(engine.preview_space().as_deref(), Some("apple"));
    }

    #[test]
    fn 中文层ctrl数字直上屏首义() {
        let mut engine = multi_sense_engine();
        type_text(&mut engine, "nihao");
        assert_eq!(engine.layer(), CandidateLayer::Chinese);
        assert!(engine.can_translate_by_index(0));
        assert_eq!(
            engine.commit_translation_by_index(0).as_deref(),
            Some("hello")
        );
        assert!(!engine.is_active(), "直上屏后组合结束");
        assert_eq!(engine.previous_word(), Some("hello"));
    }

    #[test]
    fn 译文层ctrl数字上屏对应行单义() {
        let mut engine = multi_sense_engine();
        type_text(&mut engine, "nihao");
        assert!(engine.toggle_translation_layer());
        assert!(engine.can_translate_by_index(1));
        assert_eq!(engine.commit_translation_by_index(1).as_deref(), Some("hi"));
    }

    #[test]
    fn 无译文或越界候选ctrl数字返回none() {
        let mut engine = multi_sense_engine();
        type_text(&mut engine, "ceshi");
        assert!(!engine.can_translate_by_index(0), "无译文候选不可直上屏");
        assert_eq!(engine.commit_translation_by_index(0), None);
        assert!(engine.is_active(), "不可上屏时组合保持、键放行宿主");

        let mut typing = multi_sense_engine();
        type_text(&mut typing, "nihao");
        assert!(!typing.can_translate_by_index(9), "越界候选不可直上屏");
        assert_eq!(typing.commit_translation_by_index(9), None);
    }

    #[test]
    fn 中文层候选窗副文本只显示首义() {
        let mut engine = multi_sense_engine();
        type_text(&mut engine, "nihao");
        let view = engine.candidate_ui_view();
        assert!(!view.translation_mode);
        assert_eq!(
            view.items[0].translation, "int. hello",
            "中文层副文本仅首义"
        );
    }
}
