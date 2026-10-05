//! TSF COM 服务与最小输入处理器。
//!
//! 本模块负责输入法 DLL 的 COM 生命周期、类工厂、文本服务实例，以及 M1 的
//! 按键 → 组合 → 上屏最小闭环：
//! - `ITfKeyEventSink` 必须经 `ITfKeystrokeMgr::AdviseKeyEventSink` 注册后
//!   才能接收键盘事件并决定是否吃键（经 `ITfThreadMgr::AdviseSink` 注册会失败）；
//! - `ITfEditSession` 在 TSF 编辑会话内写入组合文本或提交文本；
//! - `ITfCompositionSink` 在宿主终止组合时同步清理输入引擎状态。
//!
//! T-046：`Activate` 时经 `ITfLangBarItemMgr::AddItem` 注册语言栏中英模式
//! 图标（见 [`crate::lang_bar`]），`Deactivate` 时注销；`sync_engine` 的
//! `ToggleMode` 分支在引擎锁外通知语言栏刷新图标。
//!
//! TSF 注册表写入与清理由 `scripts/` 下的安装/卸载脚本完成，本模块不直接改注册表。

use std::ffi::c_void;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;

use windows::Win32::Foundation::{
    CLASS_E_CLASSNOTAVAILABLE, CLASS_E_NOAGGREGATION, E_POINTER, HMODULE, LPARAM, POINT, RECT,
    S_FALSE, S_OK, WPARAM,
};
use windows::Win32::System::Com::{IClassFactory, IClassFactory_Impl};
use windows::Win32::System::Diagnostics::Debug::OutputDebugStringW;
use windows::Win32::System::LibraryLoader::{
    GetModuleFileNameW, GetModuleHandleExW, GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
    GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, VIRTUAL_KEY, VK_0, VK_2, VK_9, VK_A, VK_BACK, VK_CONTROL, VK_DECIMAL, VK_DOWN,
    VK_ESCAPE, VK_MENU, VK_OEM_1, VK_OEM_2, VK_OEM_MINUS, VK_OEM_PERIOD, VK_OEM_PLUS, VK_RETURN,
    VK_SHIFT, VK_SPACE, VK_TAB, VK_UP, VK_Z,
};
use windows::Win32::UI::TextServices::{
    ITfComposition, ITfCompositionSink, ITfCompositionSink_Impl, ITfContext, ITfContextComposition,
    ITfEditSession, ITfEditSession_Impl, ITfInsertAtSelection, ITfKeyEventSink,
    ITfKeyEventSink_Impl, ITfKeystrokeMgr, ITfTextInputProcessorEx, ITfTextInputProcessorEx_Impl,
    ITfTextInputProcessor_Impl, ITfThreadMgr, TfAnchor, TF_AE_END, TF_DEFAULT_SELECTION,
    TF_ES_READWRITE, TF_ES_SYNC, TF_IAS_QUERYONLY, TF_SELECTION, TF_SELECTIONSTYLE,
};
use windows_core::{
    implement, IUnknown, IUnknownImpl, Interface, Ref, Result, BOOL, HRESULT, PCWSTR,
};
use zhu_ye_core::{
    build_contact_index, core_version, parse_vcard, BigramModel, CompositeDictionary, Dictionary,
    DictionaryFile, UserDictStore,
};

use crate::candidate_window::{CandidateWindow, CandidateWindowPlacement, ThemePreference};
use crate::input::{m1_seed_dictionary, InputEngine, InputMode};
use crate::lang_bar::LangBarHandle;

/// 输入法 TIP 的 CLSID，与 `scripts/ime-identity.ps1` 中的 `TipClsid` 保持一致。
/// 主源在 `zhu_ye_core::identity`（T-081 收拢 Rust 侧身份值为单一主源），
/// 此处按本 crate 消费方需要的 `windows::core::GUID` 形式转换。
pub const CLSID_ZHU_YE_TIP: windows::core::GUID =
    windows::core::GUID::from_u128(zhu_ye_core::identity::CLSID_ZHU_YE_TIP);

/// 安装/便携包随带的 v2 词典文件名，与 `scripts/ime-identity.ps1` 保持一致。
pub use zhu_ye_core::identity::DICTIONARY_FILE_NAME;

/// 安装/便携包随带的英文词表文件名（T-085），与
/// `scripts/ime-identity.ps1` 保持一致。
pub use zhu_ye_core::identity::EN_WORDBOOK_FILE_NAME;

/// 简体中文（zh-CN，LCID 0x0804）下的语言配置文件 GUID，
/// 与 `scripts/ime-identity.ps1` 中的 `ProfileGuid` 保持一致。
pub const PROFILE_GUID_ZHU_YE: windows::core::GUID =
    windows::core::GUID::from_u128(zhu_ye_core::identity::PROFILE_GUID_ZHU_YE);

/// 键盘输入处理器（TIP）类别的 TFCAT GUID，与 `scripts/ime-identity.ps1` 中的
/// `KeyboardCategoryGuid` 保持一致；二级修复重建 `Category\Category` 与
/// `Category\Item` 两棵子树时使用。
pub const TFCAT_ZHU_YE_KEYBOARD: windows::core::GUID =
    windows::core::GUID::from_u128(zhu_ye_core::identity::TFCAT_ZHU_YE_KEYBOARD);

/// TSF 语言配置文件注册路径中的语言段（简体中文 zh-CN），与
/// `scripts/ime-identity.ps1` 中的 `LanguageIdHex` 保持一致。
pub use zhu_ye_core::identity::TSF_LANGUAGE_ID_HEX;

/// 安装目录相对 `Program Files` 的路径段，与 `scripts/ime-identity.ps1` 的
/// `Get-TsfInstallDir` 保持一致；二级修复在读不到 `InProcServer32` 时用它在
/// 自身安装目录下推导 DLL 搜索位置。
pub use zhu_ye_core::identity::TSF_INSTALL_DIR_RELATIVE;

/// 当前由 DLL 创建且尚未释放的 COM 对象数（含类工厂与文本服务）。
static ACTIVE_OBJECTS: AtomicUsize = AtomicUsize::new(0);

/// `IClassFactory::LockServer` 锁定的层数。
static SERVER_LOCKS: AtomicUsize = AtomicUsize::new(0);

/// TSF 键盘事件对应的输入动作。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyAction {
    /// 小写英文字母进入组合。
    Letter(char),
    /// 数字进入组合串（T-049）：仅当它是某含数字缩写键的组成部分。
    Digit(char),
    /// 邮箱/网址格式字符 `@`/`.`/`/`/`:` 进入组合串（场景6，T-065/T-066）：
    /// 由 `plan_action` 按引擎 `is_format_key` 判定吃键，组合态进串、其余放行宿主。
    FormatChar(char),
    /// 退格删除组合末尾字母。
    Backspace,
    /// 空格提交第一候选或拼音原文。
    Space,
    /// 回车提交拼音原文。
    Enter,
    /// Esc 取消当前组合。
    Escape,
    /// 数字选择候选，index 从 0 开始。
    Select(usize),
    /// Shift 单击切换中英模式。
    ToggleMode,
    /// Tab 在中文候选层与译文层之间切换。
    ToggleLayer,
    /// 逗号上翻页。
    PageUp,
    /// 句号下翻页。
    PageDown,
    /// 上方向键移动页内选中行（T-039）。
    SelectUp,
    /// 下方向键移动页内选中行（T-039）。
    SelectDown,
    /// 小数点键（VK_OEM_PERIOD/VK_DECIMAL）：仅在数字格式模式内追加（FR-027 金额）。
    Dot,
    /// 数字格式模式：追加一个数字/小数点（TSF 直插该字符上屏，引擎累积 buffer）。
    BufferDigit(char),
    /// 数字格式模式退格：文档侧删插入点前 1 字符 + 引擎 buffer 回退（FR-027）。
    DigitBackspace,
    /// 数字格式模式选择：替换 buffer 长度字符后插入格式文本（FR-027 替换链）。
    SelectAndReplace(usize),
    /// 空闲态 `v`：进入 v 模式（FR-028）。
    VStart,
    /// v 模式类型码（`1-9`/`x`/`h`）。
    VCode(char),
    /// v 模式非法字母（如 `vi` 的 `i`）：`v`+该字母转入拼音组合（FR-028 回退）。
    VConsume(char),
    /// v 模式退格（有类型码时回退，只有 `v` 时退出）。
    VBackspace,
}

impl KeyAction {
    /// 是否需要 TSF 编辑会话；纯状态动作（Shift/Tab/翻页）在会话外执行。
    /// 场景7 的数字/v 模式动作都在会话内执行：直插文本（BufferDigit）、
    /// 替换文本（SelectAndReplace/DigitBackspace）、开组合（VConsume）以及
    /// 需要候选窗锚点定位（selection_placement）的纯状态动作。
    fn needs_edit_session(self) -> bool {
        matches!(
            self,
            KeyAction::Letter(_)
                | KeyAction::Digit(_)
                | KeyAction::FormatChar(_)
                | KeyAction::Backspace
                | KeyAction::Space
                | KeyAction::Enter
                | KeyAction::Escape
                | KeyAction::Select(_)
                | KeyAction::BufferDigit(_)
                | KeyAction::DigitBackspace
                | KeyAction::SelectAndReplace(_)
                | KeyAction::VStart
                | KeyAction::VCode(_)
                | KeyAction::VConsume(_)
                | KeyAction::VBackspace
        )
    }
}

/// 文本服务的共享状态。COM 回调可能由宿主在任意时刻进入，
/// 因此引擎、线程管理器引用与组合状态统一放在互斥锁内。
struct EngineState {
    engine: InputEngine,
    tid: u32,
    keystroke_mgr: Option<ITfKeystrokeMgr>,
    composition: Option<ITfComposition>,
    candidate_window: CandidateWindow,
    /// 语言栏中英模式图标（T-046）；仅在激活且有线程管理器时存在。
    lang_bar: Option<LangBarHandle>,
}

/// T-046：跨线程共享的引擎状态（语言栏点击切换模式需要）。
///
/// `unsafe impl Send/Sync` 的安全论证：
/// 1. **锁内访问**：`EngineState` 的全部非 `Send` 字段（COM 接口、
///    候选窗裸指针）只在本锁持有期内被访问，`Mutex` 提供互斥——
///    跨线程移动的只是锁拥有者，不是锁内数据的所有权；
/// 2. **Drop 线程**：最后一个强引用只在文本服务键盘/激活线程释放
///    （语言栏 `OnClick` 仅经 `upgrade` 产生临时强引用；按钮存活期间
///    `self.state` 必有常驻强引用），与 Rc 时代的销毁线程一致，
///    候选窗 `Drop` 中的 `DestroyWindow` 不会跨线程执行；
/// 3. **COM 释放**：`IUnknown::Release` 线程无关，可在任意线程调用。
struct SharedEngine(Mutex<EngineState>);

impl SharedEngine {
    /// 与 `Mutex::lock` 完全同语义（返回 `LockResult`，调用方沿用
    /// `state.lock().unwrap()` / `.ok()?` 惯例）。
    fn lock(&self) -> std::sync::LockResult<std::sync::MutexGuard<'_, EngineState>> {
        self.0.lock()
    }
}

// 安全论证见类型注释：非 Send 字段全部锁内访问 + Drop/COM 释放线程无关。
unsafe impl Send for SharedEngine {}
unsafe impl Sync for SharedEngine {}

/// 从配置解析候选窗主题偏好（FR-041，第八期设置窗口写入）。
///
/// `Light` 映射为 `Auto`：保留 T-030 的"候选窗默认固定浅色、高对比度仍走系统配色"路径；
/// `Dark` 映射为显式深色。高对比度是否应覆盖显式深色不由本函数决定——`resolve_theme`
/// 的既有语义保持不变，改动它属于独立决策。
///
/// 这里独立读取一次配置（`create_engine` 也读一次）：`config.json` 仅数百字节，装配期
/// 两次读取的代价可忽略；相比把配置贯穿进装配函数签名，这样更不易漏改。
fn configured_theme_preference() -> ThemePreference {
    let (config, _) = zhu_ye_core::load_config(&config_path());
    match config.theme {
        zhu_ye_core::ThemeChoice::Dark => ThemePreference::Dark,
        zhu_ye_core::ThemeChoice::Light => ThemePreference::Auto,
        // 自定义主题：以 Auto 为基底（浅色、高对比由系统接管 D-31），
        // 具体配色由 configured_custom_theme 读主题文件叠加（T-088 / FR-048）。
        zhu_ye_core::ThemeChoice::Custom(_) => ThemePreference::Auto,
    }
}

/// 读取配置里的自定义主题文件（T-088 / FR-048）。
///
/// `config.theme` 为 `Custom(name)` 时从 `%APPDATA%\zhu-ye-ime\themes\<name>.json`
/// 读取并解析；名称不合法（见 `zhu_ye_core::is_safe_theme_name`，防目录逃逸）、文件
/// 缺失或损坏 → `None`，候选窗回退浅色基底（与设置窗口口径一致，配置不失败）。
fn configured_custom_theme() -> Option<zhu_ye_core::ThemeFile> {
    let (config, _) = zhu_ye_core::load_config(&config_path());
    let zhu_ye_core::ThemeChoice::Custom(name) = &config.theme else {
        return None;
    };
    if !zhu_ye_core::is_safe_theme_name(name) {
        return None;
    }
    let themes = appdata_root()?.join("themes");
    zhu_ye_core::load_theme_file(&themes.join(format!("{name}.json"))).ok()
}

/// 从配置解析装配候选窗的控制器：无自定义主题走偏好路径，有则用主题文件（T-088）。
fn configured_candidate_window() -> CandidateWindow {
    match configured_custom_theme() {
        Some(file) => CandidateWindow::with_custom_theme(file),
        None => CandidateWindow::with_theme(configured_theme_preference()),
    }
}

/// 从配置解析新输入会话的默认中英模式（FR-041）。
///
/// 中英模式本身是每个宿主进程各自的 IME 实例状态，跨进程没有单一的"当前模式"；设置窗口
/// 能表达的唯一有意义语义就是"新会话从哪种模式开始"，因此这条配置是**装配项**——每次
/// `create_text_service` 实例化输入法时读取一次，之后本会话里的 Shift 切换照旧。
fn configured_default_mode() -> crate::input::InputMode {
    let (config, _) = zhu_ye_core::load_config(&config_path());
    match config.default_mode {
        zhu_ye_core::ModeChoice::English => crate::input::InputMode::English,
        zhu_ye_core::ModeChoice::Chinese => crate::input::InputMode::Chinese,
    }
}

impl EngineState {
    /// 纯逻辑构造：不读任何配置（单元测试与无配置环境统一从中文模式开始）。
    #[cfg(test)]
    fn new() -> Self {
        Self {
            engine: create_engine(None),
            tid: 0,
            keystroke_mgr: None,
            composition: None,
            candidate_window: CandidateWindow::with_theme(configured_theme_preference()),
            lang_bar: None,
        }
    }

    /// 生产装配：新输入法实例的起始模式按配置决定（FR-041，装配项）。
    fn with_start_mode(user_store: Option<UserDictStore>, mode: crate::input::InputMode) -> Self {
        let mut engine = create_engine(user_store);
        engine.set_mode(mode);
        Self {
            engine,
            tid: 0,
            keystroke_mgr: None,
            composition: None,
            candidate_window: configured_candidate_window(),
            lang_bar: None,
        }
    }
}

#[implement(IClassFactory)]
struct ClassFactory {
    user_store: Option<UserDictStore>,
}

impl Drop for ClassFactory {
    fn drop(&mut self) {
        object_released();
    }
}

impl IClassFactory_Impl for ClassFactory_Impl {
    fn CreateInstance(
        &self,
        punkouter: Ref<'_, IUnknown>,
        riid: *const windows::core::GUID,
        ppvobject: *mut *mut c_void,
    ) -> Result<()> {
        if riid.is_null() || ppvobject.is_null() {
            return Err(E_POINTER.into());
        }
        if !punkouter.is_null() {
            return Err(CLASS_E_NOAGGREGATION.into());
        }

        let service = create_text_service(self.user_store.clone());
        let hr = unsafe { service.query(riid, ppvobject) };
        hr.ok()
    }

    fn LockServer(&self, flock: BOOL) -> Result<()> {
        if !flock.as_bool() {
            SERVER_LOCKS.fetch_sub(1, Ordering::Relaxed);
        } else {
            SERVER_LOCKS.fetch_add(1, Ordering::Relaxed);
        }
        Ok(())
    }
}

#[implement(ITfTextInputProcessorEx, ITfKeyEventSink, ITfCompositionSink)]
struct TextService {
    state: Arc<SharedEngine>,
}

impl TextService {
    fn state(&self) -> &Arc<SharedEngine> {
        &self.state
    }
}

impl Drop for TextService {
    fn drop(&mut self) {
        object_released();
    }
}

impl ITfTextInputProcessor_Impl for TextService_Impl {
    fn Activate(&self, ptim: Ref<'_, ITfThreadMgr>, tid: u32) -> Result<()> {
        // 自检等场景允许空线程管理器；此时不注册按键事件，其余状态照常可用。
        let Some(thread_mgr) = ptim.cloned() else {
            product_log(zhu_ye_core::LogLevel::Info, "zhu-ye: Activate tm=none");
            return Ok(());
        };

        // `ITfKeyEventSink` 只能经 `ITfKeystrokeMgr::AdviseKeyEventSink` 注册：
        // 通过 `ITfThreadMgr::AdviseSink` 注册会返回 CONNECT_E_CANNOTCONNECT
        // (0x80040202)，按下按键永远不会送达文本服务。
        let keystroke_mgr = match unsafe { get_keystroke_mgr(&thread_mgr) } {
            Ok(km) => km,
            Err(err) => {
                product_log(
                    zhu_ye_core::LogLevel::Error,
                    &format!("zhu-ye: Activate keystroke-mgr FAILED {err:?}"),
                );
                return Err(err);
            }
        };
        let key_sink = self.to_object().to_interface::<ITfKeyEventSink>();
        match unsafe { keystroke_mgr.AdviseKeyEventSink(tid, &key_sink, true) } {
            Ok(()) => {
                let mut state = state_lock(self);
                state.tid = tid;
                state.keystroke_mgr = Some(keystroke_mgr);
                product_log(
                    zhu_ye_core::LogLevel::Info,
                    &format!("zhu-ye: Activate tid={tid} key-sink ok"),
                );
                // T-046：注册语言栏中英模式图标。语言栏/ctfmon 不可用时
                // 不阻断激活（图标属增强反馈，缺了不影响输入闭环）。
                // 左键点击切换：注入只捕获状态弱引用的处理器——ctfmon 可能
                // 经任意 RPC 线程回调 OnClick，切换在引擎锁内完成、图标通知
                // 由处理器返回值触发（与 sync_engine::ToggleMode 同语义：
                // 组合内容在切换时保留）。
                let state_weak = Arc::downgrade(&self.state.clone());
                let click_handler = Box::new(move || {
                    let state = state_weak.upgrade()?;
                    let mut guard = state.lock().ok()?;
                    guard.engine.toggle_mode();
                    Some(guard.engine.mode())
                });
                match LangBarHandle::register(&thread_mgr, state.engine.mode(), click_handler) {
                    Ok(handle) => {
                        state.lang_bar = Some(handle);
                        product_log(zhu_ye_core::LogLevel::Info, "zhu-ye: langbar added");
                    }
                    Err(err) => product_log(
                        zhu_ye_core::LogLevel::Error,
                        &format!("zhu-ye: langbar skip {err:?}"),
                    ),
                }
                Ok(())
            }
            Err(err) => {
                product_log(
                    zhu_ye_core::LogLevel::Error,
                    &format!("zhu-ye: Activate key-sink FAILED {err:?}"),
                );
                Err(err)
            }
        }
    }

    fn Deactivate(&self) -> Result<()> {
        product_log(zhu_ye_core::LogLevel::Info, "zhu-ye: Deactivate");
        let mut state = state_lock(self);
        state.composition = None;
        state.engine.cancel_input();
        state.candidate_window.hide();

        // T-046：注销语言栏项目（消耗句柄持有），优先于按键 sink 清理。
        if let Some(handle) = state.lang_bar.take() {
            handle.unregister();
            product_log(zhu_ye_core::LogLevel::Info, "zhu-ye: langbar removed");
        }
        if let Some(keystroke_mgr) = state.keystroke_mgr.take() {
            let _ = unsafe { keystroke_mgr.UnadviseKeyEventSink(state.tid) };
        }
        Ok(())
    }
}

impl ITfTextInputProcessorEx_Impl for TextService_Impl {
    fn ActivateEx(&self, ptim: Ref<'_, ITfThreadMgr>, tid: u32, dwflags: u32) -> Result<()> {
        product_log(
            zhu_ye_core::LogLevel::Info,
            &format!("zhu-ye: ActivateEx flags=0x{dwflags:X} tid={tid}"),
        );
        ITfTextInputProcessor_Impl::Activate(self, ptim, tid)
    }
}

impl ITfKeyEventSink_Impl for TextService_Impl {
    fn OnSetFocus(&self, fforeground: BOOL) -> Result<()> {
        product_log(
            zhu_ye_core::LogLevel::Info,
            &format!("zhu-ye: OnSetFocus fg={}", fforeground.0),
        );
        Ok(())
    }

    fn OnTestKeyDown(
        &self,
        _pic: Ref<'_, ITfContext>,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> Result<BOOL> {
        let action = plan_action(
            wparam,
            lparam,
            key_modifiers_down(),
            shift_key_down(),
            self.state(),
        );
        if should_log(zhu_ye_core::LogLevel::Debug) {
            debug_log(&format!(
                "zhu-ye: TestKeyDown 0x{:X} -> {:?}",
                wparam.0, action
            ));
        }
        Ok(BOOL(action.is_some() as i32))
    }

    fn OnTestKeyUp(
        &self,
        _pic: Ref<'_, ITfContext>,
        _wparam: WPARAM,
        _lparam: LPARAM,
    ) -> Result<BOOL> {
        Ok(BOOL(0))
    }

    fn OnKeyDown(&self, pic: Ref<'_, ITfContext>, wparam: WPARAM, lparam: LPARAM) -> Result<BOOL> {
        let Some(action) = plan_action(
            wparam,
            lparam,
            key_modifiers_down(),
            shift_key_down(),
            self.state(),
        ) else {
            return Ok(BOOL(0));
        };

        // 只改引擎/候选窗状态、不写文档文本的动作不需要编辑会话。
        if !action.needs_edit_session() {
            sync_engine(self.state(), action);
            refresh_candidate_window(self.state(), None);
            if should_log(zhu_ye_core::LogLevel::Debug) {
                debug_log(&format!(
                    "zhu-ye: key 0x{:X} state action {:?}",
                    wparam.0, action
                ));
            }
            return Ok(BOOL(1));
        }

        let Some(context) = pic.cloned() else {
            return Ok(BOOL(0));
        };

        if should_log(zhu_ye_core::LogLevel::Debug) {
            debug_log(&format!("zhu-ye: key 0x{:X} action {:?}", wparam.0, action));
        }

        let tid = state_lock(self).tid;
        let sink = self.to_object().to_interface::<ITfCompositionSink>();
        let state = Arc::clone(self.state());
        let session_context = context.clone();
        let session: ITfEditSession = EditSession {
            callback: Mutex::new(Some(Box::new(move |ec| {
                apply_action(&state, &session_context, &sink, ec, action)
            }))),
        }
        .into();

        let session_hr =
            unsafe { context.RequestEditSession(tid, &session, TF_ES_SYNC | TF_ES_READWRITE) };
        let session_failed = match session_hr {
            Ok(hr) => hr.is_err(),
            Err(_) => true,
        };
        // 编辑会话失败时仍同步引擎，避免后续按键基于漂移状态继续输入。
        if session_failed {
            sync_engine(self.state(), action);
            refresh_candidate_window(self.state(), None);
        }
        Ok(BOOL(1))
    }
    fn OnKeyUp(&self, _pic: Ref<'_, ITfContext>, _wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        Ok(BOOL(0))
    }

    fn OnPreservedKey(
        &self,
        _pic: Ref<'_, ITfContext>,
        _rguid: *const windows::core::GUID,
    ) -> Result<BOOL> {
        Ok(BOOL(0))
    }
}

impl ITfCompositionSink_Impl for TextService_Impl {
    fn OnCompositionTerminated(
        &self,
        _ecwrite: u32,
        _pcomposition: Ref<'_, ITfComposition>,
    ) -> Result<()> {
        let mut state = state_lock(self);
        state.composition = None;
        state.engine.cancel_input();
        state.candidate_window.hide();
        Ok(())
    }
}

/// 一次编辑会话回调。TSF 会以同步方式调用 `DoEditSession`，
/// 因此只需保存一个闭包并在首次调用时执行。
type EditSessionCallback = Box<dyn FnOnce(u32) -> Result<()>>;

#[implement(ITfEditSession)]
struct EditSession {
    callback: Mutex<Option<EditSessionCallback>>,
}

impl ITfEditSession_Impl for EditSession_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        if let Some(callback) = self.callback.lock().unwrap().take() {
            callback(ec)
        } else {
            Ok(())
        }
    }
}

/// 判断按键是否为长按重复事件（lparam 第 30 位）。
fn is_repeat(lparam: LPARAM) -> bool {
    (lparam.0 as u32) & 0x4000_0000 != 0
}

/// 将虚拟键码归类为输入动作；与本输入法无关的键返回 `None`。
/// `shift` 为按键时刻 Shift 修饰是否按下（英文布局决定 `@`/`:` 等上档字符，
/// 由调用方用真实键盘状态查询后传入，便于测试注入）。
fn classify_key(wparam: WPARAM, lparam: LPARAM, shift: bool) -> Option<KeyAction> {
    let code = VIRTUAL_KEY(wparam.0 as u16).0;
    match code {
        code if (VK_A.0..=VK_Z.0).contains(&code) => {
            let letter = u16::from(b'a') + (code - VK_A.0);
            Some(KeyAction::Letter(
                char::from_u32(u32::from(letter)).unwrap(),
            ))
        }
        // T-066：英文布局 Shift+2 = `@`（场景6 邮箱键路；是否进组合由
        // `plan_action` 按引擎 `is_format_key` 判定，空闲态放行宿主）。
        code if code == VK_2.0 && shift => Some(KeyAction::FormatChar('@')),
        // T-066：英文布局 `:`（Shift+`;`，VK_OEM_1+Shift）；无 Shift 的 `;` 放行宿主。
        code if code == VK_OEM_1.0 && shift => Some(KeyAction::FormatChar(':')),
        // T-066：英文布局 `/`（VK_OEM_2，无 Shift；Shift+`/` 的 `?` 放行宿主）。
        code if code == VK_OEM_2.0 && !shift => Some(KeyAction::FormatChar('/')),
        code if code == VK_BACK.0 => Some(KeyAction::Backspace),
        code if code == VK_SPACE.0 => Some(KeyAction::Space),
        code if code == VK_RETURN.0 => Some(KeyAction::Enter),
        code if code == VK_ESCAPE.0 => Some(KeyAction::Escape),
        code if (VK_0.0..=VK_9.0).contains(&code) => {
            // T-049：数字键先归类为"数字"，由 `plan_action` 决定它是选词还是进组合串。
            char::from_u32(u32::from(b'0') + u32::from(code - VK_0.0)).map(KeyAction::Digit)
        }
        // FR-027：英文布局小数点（`.`/小键盘 `.`）归为 Dot，仅在数字格式模式内
        // 追加（金额小数位）；数字模式外放行给宿主直出标点。
        code if code == VK_OEM_PERIOD.0 || code == VK_DECIMAL.0 => Some(KeyAction::Dot),
        code if code == VK_SHIFT.0 && !is_repeat(lparam) => Some(KeyAction::ToggleMode),
        code if code == VK_TAB.0 => Some(KeyAction::ToggleLayer),
        // T-033：候选翻页键改用 `-`（上一页）与 `=`/`+`（下一页，VK_OEM_PLUS
        // 同时覆盖 Shift+`=` 的 `+`），替换原 `,`（上翻）与 `.`（下翻）。
        code if code == VK_OEM_MINUS.0 => Some(KeyAction::PageUp),
        code if code == VK_OEM_PLUS.0 => Some(KeyAction::PageDown),
        // T-039：上下方向键移动页内选中行。
        code if code == VK_UP.0 => Some(KeyAction::SelectUp),
        code if code == VK_DOWN.0 => Some(KeyAction::SelectDown),
        _ => None,
    }
}

/// 判断 Ctrl 或 Alt 修饰键是否按下；按住时系统组合键（复制/粘贴/保存等）
/// 必须放行给宿主应用，避免输入法吞掉 Ctrl+A/C/S/V 等快捷键。
fn key_modifiers_down() -> bool {
    // GetKeyState 返回 SHORT，高位为 1 表示按下，即 i16 值为负。
    unsafe { GetKeyState(i32::from(VK_CONTROL.0)) < 0 || GetKeyState(i32::from(VK_MENU.0)) < 0 }
}

/// 判断 Shift 修饰键是否按下（英文布局上档字符判定，T-066）。
fn shift_key_down() -> bool {
    unsafe { GetKeyState(i32::from(VK_SHIFT.0)) < 0 }
}

/// 从 `ITfThreadMgr` 取出按键管理器。
///
/// TSF 中 `ITfKeystrokeMgr` 由线程管理器对象一并实现，通过
/// `QueryInterface(IID_ITfKeystrokeMgr)` 获取（windows crate 生成的
/// `ITfThreadMgr` 只导出 11 个自身方法，vtable 槽 3..13，再往后访问就会越界，
/// 因此绝不能按槽位手工读取——此前按“第 14 槽”假设实现，导致 conhost/MSCTF
/// 在 TIP 激活时进程崩溃）。
unsafe fn get_keystroke_mgr(thread_mgr: &ITfThreadMgr) -> Result<ITfKeystrokeMgr> {
    thread_mgr.cast()
}

/// 决定是否吃下按键。字母仅在中文模式下进入组合；
/// 功能键只在已有组合时处理，避免键盘事件被无谓吞掉。
/// Shift 只在无活动组合时切换模式：按 `+`（Shift+`=`）翻页时，先落下的 Shift
/// 只作为修饰键放行给宿主，避免翻页顺带把中文模式切走（T-033）。
///
/// 场景7（FR-027/028）：空闲态数字启动数字格式模式、空闲态 `v` 启动 v 模式；
/// 数字/v 模式激活时按优先级处理，遇到无关键（Enter/Tab/翻页/其它字母）先退出
/// 该模式再按常规语义处理，保证模式窗口不会被卡住。
fn plan_action(
    wparam: WPARAM,
    lparam: LPARAM,
    modifier_held: bool,
    shift_held: bool,
    state: &Arc<SharedEngine>,
) -> Option<KeyAction> {
    if modifier_held {
        return None;
    }
    let action = classify_key(wparam, lparam, shift_held)?;
    let engine = &mut state.lock().unwrap().engine;

    // 数字/v 模式的自动退出：遇到与模式无关的键先退出模式，按键本身按常规处理
    // （Enter 放行宿主、字母进拼音、翻页/选行作用于常规候选等）。
    if engine.digit_active() && !keeps_digit_mode(action) {
        engine.exit_digit();
    }
    if engine.v_active() && !keeps_v_mode(action) {
        engine.v_exit();
    }

    match action {
        KeyAction::Letter(c) if engine.v_active() => {
            // FR-028 + T-104：符号类型码（x/h）与单位键前缀字母进 v 模式
            // （`vmi` 单位换算码），其余字母（含 `v`/`i`）回退拼音。
            if engine.v_accepts(c) {
                Some(KeyAction::VCode(c))
            } else {
                Some(KeyAction::VConsume(c))
            }
        }
        KeyAction::Letter('v')
            if engine.mode() == InputMode::Chinese
                && !engine.is_active()
                && !engine.suggestion_active() =>
        {
            // FR-028：空闲态 `v` 进入 v 模式；`v` 同时是 nv/lv 的合法拼音字符，
            // 组合态/联想态一律走常规拼音路径（联想优先，D-05）。
            Some(KeyAction::VStart)
        }
        KeyAction::Letter(_) => (engine.mode() == InputMode::Chinese).then_some(action),
        KeyAction::Dot => {
            // FR-027：数字格式模式内追加小数点（金额小数位）。
            // T-066：否则组合态邮箱/网址上下文中 `.` 进组合串（按引擎 `is_format_key`），
            // 普通拼音组合后的 `.` 照旧放行宿主直出标点。
            if engine.mode() == InputMode::Chinese && engine.digit_active() {
                Some(KeyAction::BufferDigit('.'))
            } else if engine.mode() == InputMode::Chinese && engine.is_format_key('.') {
                Some(KeyAction::FormatChar('.'))
            } else {
                None
            }
        }
        // T-066：`@`/`:`/`/` 是否进组合串由引擎判定（组合态邮箱/网址上下文为真，
        // 空闲态/普通拼音/英文模式为假并放行宿主）。
        KeyAction::FormatChar(c)
            if engine.mode() == InputMode::Chinese && engine.is_format_key(c) =>
        {
            Some(action)
        }
        KeyAction::FormatChar(_) => None,
        // T-049：数字键在中文模式下先判断是否为"含数字缩写键"的组成部分
        // （如 `996`/`u1s1`），是则进组合串；否则回落原有语义——
        // 有候选时选词（FR-006），空闲态数字进入数字格式模式（FR-027）。
        KeyAction::Digit(digit) => plan_digit(engine, digit),
        // FR-027：数字格式模式退格由引擎删除 buffer 尾部位，文档侧同步删 1 字符。
        KeyAction::Backspace if engine.digit_active() => Some(KeyAction::DigitBackspace),
        // FR-028：v 模式退格回退类型码（只有 `v` 时退出）。
        KeyAction::Backspace if engine.v_active() => Some(KeyAction::VBackspace),
        // FR-027：数字格式模式空格 = 选择当前选中行格式候选并替换。
        KeyAction::Space if engine.digit_active() => {
            Some(KeyAction::SelectAndReplace(engine.selected_on_page()))
        }
        // FR-028：v 模式空格 = 选择第 1 个符号候选。
        KeyAction::Space if engine.v_active() && engine.v_symbol_count() > 0 => Some(action),
        // T-059：上屏联想态空格提交选中联想词、Esc 关闭联想窗；
        // 其余功能键（Enter/Backspace/翻页/选择）放行给宿主。
        KeyAction::Space | KeyAction::Escape if engine.suggestion_active() => Some(action),
        // FR-027/028：Esc 退出数字/v 模式（已上屏的数字/符号正文保留）。
        KeyAction::Escape if engine.digit_active() || engine.v_active() => Some(action),
        KeyAction::ToggleMode if !engine.is_active() => Some(action),
        KeyAction::ToggleMode => None,
        _ if engine.is_active() => Some(action),
        _ => None,
    }
}

/// 数字键在中文模式下的分派（FR-027 与既有 T-049 语义）。
fn plan_digit(engine: &mut InputEngine, digit: char) -> Option<KeyAction> {
    if engine.mode() != InputMode::Chinese {
        return None;
    }
    let index = usize::from(digit as u8).checked_sub(usize::from(b'1'));
    // v 模式优先：等待类型码时 `1-9` 出符号组，已出组后 `1-9` 选择符号；
    // `0` 无义，退出 v 模式并放行宿主。
    if engine.v_active() {
        if engine.v_buffer_len() == 1 {
            return if digit != '0' {
                Some(KeyAction::VCode(digit))
            } else {
                engine.v_exit();
                None
            };
        }
        if let Some(index) = index.filter(|i| *i < engine.v_symbol_count()) {
            return Some(KeyAction::Select(index));
        }
        engine.v_exit();
        return None;
    }
    // 数字格式模式：`1-9` 且未越界选择格式候选；越界/无候选（如不足 5 位）继续追加。
    if engine.digit_active() {
        return index
            .filter(|i| *i < engine.digit_candidate_count())
            .map(KeyAction::SelectAndReplace)
            .or(Some(KeyAction::BufferDigit(digit)));
    }
    // T-059：上屏联想态数字键直接选择联想候选；越界放行给宿主（不吞键、不上屏空串）。
    if engine.suggestion_active() {
        return index
            .filter(|i| *i < engine.suggestion_list().len())
            .map(KeyAction::Select);
    }
    if should_compose_digit(engine, digit) {
        return Some(KeyAction::Digit(digit));
    }
    // 空闲态数字：启动数字格式模式（边输边上屏），不再放行宿主直出。
    if !engine.is_active() {
        return Some(KeyAction::BufferDigit(digit));
    }
    match index {
        Some(index) if engine.is_active() => Some(KeyAction::Select(index)),
        _ => None,
    }
}

/// 数字格式模式是否保留该键（FR-027）：数字/小数点追加、选格式、退格、Esc。
fn keeps_digit_mode(action: KeyAction) -> bool {
    matches!(
        action,
        KeyAction::Digit(_)
            | KeyAction::Dot
            | KeyAction::Space
            | KeyAction::Escape
            | KeyAction::Backspace
    )
}

/// v 模式是否保留该键（FR-028）：类型码、选符号、回退拼音、退格、Esc。
fn keeps_v_mode(action: KeyAction) -> bool {
    match action {
        KeyAction::Digit(_) | KeyAction::Backspace | KeyAction::Escape => true,
        KeyAction::Space => true,
        KeyAction::Letter(c) => c.is_ascii_lowercase(),
        _ => false,
    }
}

/// 判断数字键是否应进入组合串而非选词（T-049）。
///
/// 两种情况进入组合：① 组合串接上该数字后仍是某个含数字缩写键的前缀；
/// ② 组合串本身已是一串纯数字（用户正在输入电话号码/日期等），继续累积。
/// 其余情况返回 `false`，由调用方回落到选词或直出。
fn should_compose_digit(engine: &InputEngine, digit: char) -> bool {
    let candidate = format!("{}{digit}", engine.composing());
    if engine.is_abbreviation_prefix(&candidate) {
        return true;
    }
    let composing = engine.composing();
    !composing.is_empty() && composing.chars().all(|c| c.is_ascii_digit())
}

/// 应用一次输入动作：先写 TSF 组合/提交文本，再同步引擎状态。
fn apply_action(
    state: &Arc<SharedEngine>,
    context: &ITfContext,
    sink: &ITfCompositionSink,
    ec: u32,
    action: KeyAction,
) -> Result<()> {
    match action {
        KeyAction::Letter(_)
        | KeyAction::Digit(_)
        | KeyAction::FormatChar(_)
        | KeyAction::Backspace => {
            let text = compose_text(state, action);
            update_composition(state, context, sink, ec, &text)?;
        }
        // FR-028：`vi` 等回退拼音——`v`+字母直接开组合显示，进正常拼音路径。
        KeyAction::VConsume(c) => {
            let text = compose_text(state, action);
            update_composition(state, context, sink, ec, &text)?;
            if should_log(zhu_ye_core::LogLevel::Debug) {
                debug_log(&format!("zhu-ye: v-consume {c} -> {text:?}"));
            }
        }
        KeyAction::Space | KeyAction::Enter | KeyAction::Escape | KeyAction::Select(_) => {
            let text = commit_text(state, action);
            finish_composition(state, context, ec, &text)?;
        }
        // FR-027：数字格式模式——每键直插该数字上屏（无组合，边输边上屏）。
        KeyAction::BufferDigit(c) => {
            let text = c.to_string();
            finish_composition(state, context, ec, &text)?;
        }
        // FR-027：数字格式模式退格——文档侧删插入点前 1 字符。
        KeyAction::DigitBackspace => {
            replace_last_chars(context, ec, 1, "")?;
        }
        // FR-027：数字格式模式选择——替换最近 buffer 长度字符后插入格式文本。
        KeyAction::SelectAndReplace(index) => {
            let (text, replace_len) = {
                let engine = &mut state.lock().unwrap().engine;
                engine.preview_digit(index).unwrap_or_default()
            };
            replace_last_chars(context, ec, replace_len, &text)?;
        }
        KeyAction::ToggleMode
        | KeyAction::ToggleLayer
        | KeyAction::PageUp
        | KeyAction::PageDown
        | KeyAction::SelectUp
        | KeyAction::SelectDown
        | KeyAction::VStart
        | KeyAction::VCode(_)
        | KeyAction::VBackspace
        | KeyAction::Dot => {
            sync_engine(state, action);
            refresh_candidate_window(state, Some((context, ec)));
            return Ok(());
        }
    }
    sync_engine(state, action);
    refresh_candidate_window(state, Some((context, ec)));
    Ok(())
}

/// 编辑会话内把文档当前插入点前的 `replace_len` 个 UTF-16 单元替换为 `text`
/// （FR-027 数字格式选择替换链，备选 A 之外的唯一实施路径）。
///
/// 步骤：取插入点 range → 起点前移 replace_len → SetText 替换 → 折叠到末尾并
/// SetSelection 修正光标。`replace_len` 为 0 时退化为纯插入（防御）；文档当前
/// 无选择点时静默跳过（与 `selection_placement` 同级的防御）。
fn replace_last_chars(context: &ITfContext, ec: u32, replace_len: usize, text: &str) -> Result<()> {
    let mut selection = [TF_SELECTION::default()];
    let mut fetched: u32 = 0;
    unsafe {
        context.GetSelection(ec, TF_DEFAULT_SELECTION, &mut selection, &mut fetched)?;
    }
    let Some(range) = (*selection[0].range).clone() else {
        return Ok(());
    };
    if let Ok(shift) = i32::try_from(replace_len) {
        if shift > 0 {
            unsafe {
                range.ShiftStart(ec, -shift, std::ptr::null_mut(), std::ptr::null())?;
            }
        }
    }
    let wide = to_wide(text);
    unsafe {
        range.SetText(ec, 0, &wide)?;
    }
    // SetText 后把选区折叠到替换文本末尾（场景7 VM 验收断言"无残留"）。
    unsafe {
        range.Collapse(ec, TfAnchor(1))?;
    }
    let selection = TF_SELECTION {
        range: std::mem::ManuallyDrop::new(Some(range)),
        style: TF_SELECTIONSTYLE {
            ase: TF_AE_END,
            fInterimChar: BOOL(0),
        },
    };
    unsafe {
        context.SetSelection(ec, &[selection])?;
    }
    if should_log(zhu_ye_core::LogLevel::Debug) {
        debug_log(&format!("zhu-ye: replace-last {replace_len} -> {text:?}"));
    }
    Ok(())
}

/// 计算下一次组合串文本；不修改引擎，真实状态在 TSF 写入完成后同步。
fn compose_text(state: &Arc<SharedEngine>, action: KeyAction) -> String {
    let text = {
        let engine = &mut state.lock().unwrap().engine;
        match action {
            KeyAction::Letter(c) | KeyAction::Digit(c) | KeyAction::FormatChar(c) => {
                format!("{}{}", engine.composing(), c)
            }
            // FR-028：v 模式回退拼音时组合串为 `v`+字母（引擎尚未同步）。
            KeyAction::VConsume(c) => format!("v{c}"),
            KeyAction::Backspace => engine.preview_after_backspace().unwrap_or_default(),
            _ => String::new(),
        }
    };
    if should_log(zhu_ye_core::LogLevel::Debug) {
        debug_log(&format!("zhu-ye: compose-text {action:?} -> {text:?}"));
    }
    text
}

/// 计算本次提交文本；清空引擎状态交给 TSF 写入完成后的 `sync_engine`。
fn commit_text(state: &Arc<SharedEngine>, action: KeyAction) -> String {
    let text = {
        let engine = &mut state.lock().unwrap().engine;
        match action {
            KeyAction::Space => engine.preview_space().unwrap_or_default(),
            KeyAction::Enter => engine.preview_enter().unwrap_or_default(),
            KeyAction::Escape => String::new(),
            KeyAction::Select(index) => engine.preview_selection(index).unwrap_or_default(),
            _ => String::new(),
        }
    };
    if should_log(zhu_ye_core::LogLevel::Debug) {
        debug_log(&format!(
            "zhu-ye: commit-text {action:?} -> {text:?} ({} utf8)",
            text.len()
        ));
    }
    text
}

/// 将引擎状态推进到动作后的实际状态。
fn sync_engine(state: &Arc<SharedEngine>, action: KeyAction) {
    // T-046：模式切换额外需要把新模式同步到语言栏图标。语言栏接口
    // 与引擎状态分属不同锁域，切换动作在锁内完成、通知在锁外发送，
    // 避免在 ctfmon 回调线程上持锁等待语言栏。
    if action == KeyAction::ToggleMode {
        let (mode, lang_bar) = {
            let mut guard = state.lock().unwrap();
            guard.engine.toggle_mode();
            let mode = guard.engine.mode();
            (mode, guard.lang_bar.clone())
        };
        if let Some(handle) = lang_bar {
            handle.set_mode(mode);
        }
        return;
    }

    let engine = &mut state.lock().unwrap().engine;
    match action {
        KeyAction::Letter(c) => {
            let _ = engine.handle_letter(c);
        }
        KeyAction::Digit(c) => {
            let _ = engine.handle_digit(c);
        }
        KeyAction::FormatChar(c) => {
            let _ = engine.handle_format_char(c);
        }
        KeyAction::Backspace => {
            let _ = engine.handle_backspace();
        }
        KeyAction::Space => {
            let _ = engine.handle_space();
        }
        KeyAction::Enter => {
            let _ = engine.handle_enter();
        }
        KeyAction::Escape => {
            let _ = engine.handle_escape();
        }
        KeyAction::Select(index) => {
            let _ = engine.select_index(index);
        }
        KeyAction::ToggleMode => {
            // 已在上方专门分支处理，这里不可达。
            unreachable!("ToggleMode 在 sync_engine 入口已拦截");
        }
        KeyAction::ToggleLayer => {
            let _ = engine.toggle_translation_layer();
        }
        KeyAction::PageUp => {
            engine.previous_page();
        }
        KeyAction::PageDown => {
            engine.next_page();
        }
        KeyAction::SelectUp => {
            engine.select_up();
        }
        KeyAction::SelectDown => {
            engine.select_down();
        }
        // FR-027：数字格式模式，引擎侧同步 buffer 与候选（文档已先写入）。
        KeyAction::Dot => {
            let _ = engine.digit_append('.');
        }
        KeyAction::BufferDigit(c) => {
            let _ = engine.digit_append(c);
        }
        KeyAction::DigitBackspace => {
            let _ = engine.digit_backspace();
        }
        KeyAction::SelectAndReplace(index) => {
            let _ = engine.commit_digit(index);
        }
        // FR-028：v 模式状态推进（文档侧无写入，VConsume 的组合已在 apply_action 更新）。
        KeyAction::VStart => {
            let _ = engine.v_start();
        }
        KeyAction::VCode(c) => {
            let _ = engine.v_code(c);
        }
        KeyAction::VConsume(c) => {
            let _ = engine.v_consume(c);
        }
        KeyAction::VBackspace => {
            let _ = engine.v_backspace();
        }
    }
}

/// 用引擎最新状态刷新候选窗。`edit` 提供编辑会话内的上下文以计算组合区坐标。
///
/// T-031：组合串非空即显示候选窗；无候选词时只画页眉条（组合串与拼音提示）。
/// T-059：上屏联想态（组合串为空但联想 items 非空）也显示候选窗，定位在
/// 文档插入点（selection 顶部）；仅当组合串为空且无任何候选时才隐藏窗口。
fn refresh_candidate_window(state: &Arc<SharedEngine>, edit: Option<(&ITfContext, u32)>) {
    let view = state.lock().unwrap().engine.candidate_ui_view();
    if view.composition.is_empty() && view.items.is_empty() {
        if should_log(zhu_ye_core::LogLevel::Debug) {
            debug_log("zhu-ye: cand-hide (no composition, no items)");
        }
        state.lock().unwrap().candidate_window.hide();
        return;
    }
    let placement = if view.composition.is_empty() {
        edit.and_then(|(context, ec)| selection_placement(context, ec))
    } else {
        edit.and_then(|(context, ec)| composition_placement(state, context, ec))
    };
    if should_log(zhu_ye_core::LogLevel::Debug) {
        debug_log(&format!(
            "zhu-ye: cand-show items={} first={:?}",
            view.visible_items().len(),
            view.visible_items().first()
        ));
    }
    state
        .lock()
        .unwrap()
        .candidate_window
        .update(view, placement);
}

/// 编辑会话内取组合范围在屏幕上的底部坐标，用于候选窗定位。
fn composition_placement(
    state: &Arc<SharedEngine>,
    context: &ITfContext,
    ec: u32,
) -> Option<CandidateWindowPlacement> {
    let composition = state.lock().unwrap().composition.clone()?;
    let view = unsafe { context.GetActiveView() }.ok()?;
    let range = unsafe { composition.GetRange() }.ok()?;
    let mut rect = RECT::default();
    let mut clipped = BOOL(0);
    unsafe {
        view.GetTextExt(ec, &range, &mut rect, &mut clipped).ok()?;
    }
    Some(CandidateWindowPlacement {
        anchor: POINT {
            x: rect.left,
            y: rect.bottom,
        },
    })
}

/// 编辑会话内取文档当前插入点（selection 起点）在屏幕上的底部坐标，
/// 用于上屏联想候选窗（T-059，组合串为空无组成区范围）定位。
fn selection_placement(context: &ITfContext, ec: u32) -> Option<CandidateWindowPlacement> {
    let mut selection = [TF_SELECTION::default()];
    let mut fetched: u32 = 0;
    unsafe {
        context
            .GetSelection(ec, TF_DEFAULT_SELECTION, &mut selection, &mut fetched)
            .ok()?;
    }
    let range = (*selection[0].range).clone()?;
    let view = unsafe { context.GetActiveView() }.ok()?;
    let mut rect = RECT::default();
    let mut clipped = BOOL(0);
    unsafe {
        view.GetTextExt(ec, &range, &mut rect, &mut clipped).ok()?;
    }
    Some(CandidateWindowPlacement {
        anchor: POINT {
            x: rect.left,
            y: rect.bottom,
        },
    })
}

/// 更新组合文本：已有组合直接替换，否则插入文本并启动新组合。
fn update_composition(
    state: &Arc<SharedEngine>,
    context: &ITfContext,
    sink: &ITfCompositionSink,
    ec: u32,
    text: &str,
) -> Result<()> {
    let wide = to_wide(text);
    let existing = state.lock().unwrap().composition.clone();
    match existing {
        Some(composition) => {
            let range = unsafe { composition.GetRange() }?;
            unsafe { range.SetText(ec, 0, &wide) }?;
            if should_log(zhu_ye_core::LogLevel::Debug) {
                debug_log(&format!("zhu-ye: comp-update {text:?}"));
            }
        }
        None => {
            debug_log("zhu-ye: comp-insert-begin");
            let insert = context.cast::<ITfInsertAtSelection>()?;
            debug_log("zhu-ye: comp-insert-cast-ok");
            // 注意：不能用 TF_IAS_NOQUERY 直接写入——msctf 的
            // InsertTextAtSelection 写入分支在本机与 Win10/1809 上都会
            // 在非空 pprange 下崩溃（c0000005），本地探针 tsf_min_host
            // 已验证。改为 QUERYONLY 取得插入点 range，再由
            // StartComposition + range.SetText 写入（组合范围会覆盖新文本，
            // 探针已用 GetText 逐键验证）。
            let wide_text = &wide[..wide.len() - 1];
            let range =
                match unsafe { insert.InsertTextAtSelection(ec, TF_IAS_QUERYONLY, wide_text) } {
                    Ok(range) => {
                        debug_log("zhu-ye: comp-insert-ok");
                        range
                    }
                    Err(err) => {
                        debug_log(&format!("zhu-ye: comp-insert-err {err:?}"));
                        return Err(err);
                    }
                };
            debug_log("zhu-ye: comp-ccomp-begin");
            let composition_services = context.cast::<ITfContextComposition>()?;
            debug_log("zhu-ye: comp-ccomp-cast-ok");
            let composition =
                match unsafe { composition_services.StartComposition(ec, &range, sink) } {
                    Ok(composition) => {
                        debug_log("zhu-ye: comp-start-ok");
                        composition
                    }
                    Err(err) => {
                        debug_log(&format!("zhu-ye: comp-start-err {err:?}"));
                        return Err(err);
                    }
                };
            if let Err(err) = unsafe { range.SetText(ec, 0, wide_text) } {
                debug_log(&format!("zhu-ye: comp-settext-err {err:?}"));
                // 组合已启动但写入失败：立即结束空组合，避免悬挂。
                let _ = unsafe { composition.EndComposition(ec) };
                return Err(err);
            }
            debug_log("zhu-ye: comp-settext-ok");
            state.lock().unwrap().composition = Some(composition);
            debug_log(&format!("zhu-ye: comp-start {text:?}"));
        }
    }
    Ok(())
}

/// 结束组合并提交文本；空文本按取消处理，若原本没有组合则直接插入提交文本。
fn finish_composition(
    state: &Arc<SharedEngine>,
    context: &ITfContext,
    ec: u32,
    text: &str,
) -> Result<()> {
    let composition = state.lock().unwrap().composition.take();
    match composition {
        Some(composition) => {
            let wide: Vec<u16> = if text.is_empty() {
                Vec::new()
            } else {
                to_wide(text)
            };
            let range = unsafe { composition.GetRange() }?;
            unsafe { range.SetText(ec, 0, &wide) }?;
            unsafe { composition.EndComposition(ec) }?;
            if text.is_empty() {
                debug_log("zhu-ye: commit-cancel (empty)");
            } else {
                if should_log(zhu_ye_core::LogLevel::Debug) {
                    debug_log(&format!("zhu-ye: commit {text:?} ({} utf8)", text.len()));
                }
            }
        }
        None if !text.is_empty() => {
            let insert = context.cast::<ITfInsertAtSelection>()?;
            let wide = to_wide(text);
            let wide_text = &wide[..wide.len() - 1];
            // 同 update_composition：QUERYONLY + SetText 绕过崩溃的写入分支。
            let range = unsafe { insert.InsertTextAtSelection(ec, TF_IAS_QUERYONLY, wide_text) }?;
            unsafe { range.SetText(ec, 0, wide_text) }?;
            debug_log(&format!("zhu-ye: commit-no-comp {text:?}"));
        }
        None => {
            debug_log("zhu-ye: commit-noop (no comp, empty)");
        }
    }
    Ok(())
}

/// 将 UTF-8 文本转为以空字符结尾的 UTF-16 序列。
fn to_wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

/// 输出调试日志（第十一期 FR-060，T-091 分级后的 debug 级入口）：
/// 先写入调试器输出通道（`OutputDebugStringW`，无调试器时空转无成本），
/// 再按 D-73 双轨写文件——哨兵存在时全量写 `C:\zhu-ye-test\tsf-debug.log`
/// （VM 取证零改动），否则按产品轨级别门写用户目录日志。调用点默认包
/// `should_log` 前置守卫（热路径构造前短路）。
#[inline]
fn debug_log(message: &str) {
    product_log(zhu_ye_core::LogLevel::Debug, message);
}

/// 产品化日志（FR-060，T-091）：按级别写文件日志。双轨（D-73）：
/// - 哨兵 `C:\zhu-ye-test\tsf-debug.enable` 存在 → 全量写原路径（格式不变，
///   vm-accept-sop 零改动）；
/// - 否则 → 产品轨：级别门（config `log_level`，默认 warn，D-72）短路后
///   `FileLogger` 写 `%LOCALAPPDATA%\zhu-ye-ime\logs\ime.log`（1 MiB 轮转）。
///
/// 目录创建与文件写只发生在达到级别的消息上（热路径由调用点守卫短路）。
fn product_log(level: zhu_ye_core::LogLevel, message: &str) {
    let wide = to_wide(message);
    unsafe { OutputDebugStringW(PCWSTR(wide.as_ptr())) };
    if file_log_enabled() {
        write_sentinel_log(message);
        return;
    }
    if level > product_log_level() {
        return;
    }
    let mut guard = PRODUCT_LOGGER
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if guard.is_none() {
        *guard = build_product_logger();
    }
    if let Some(logger) = guard.as_mut() {
        logger.write(level, unsafe { GetCurrentThreadId() }, message);
    }
}

/// 哨兵轨：全量追加写 `C:\zhu-ye-test\tsf-debug.log`（现状格式，无级别过滤）。
fn write_sentinel_log(message: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(FILE_LOG_PATH)
    {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let pid = std::process::id();
        let tid = unsafe { GetCurrentThreadId() };
        let _ = writeln!(file, "[{now}] pid={pid} tid={tid} {message}");
    }
}

/// 产品轨配置级别（进程生命周期内缓存；P-12 配置不做热切换）。
fn product_log_level() -> zhu_ye_core::LogLevel {
    static LEVEL: OnceLock<zhu_ye_core::LogLevel> = OnceLock::new();
    *LEVEL.get_or_init(|| zhu_ye_core::load_config(&config_path()).0.log_level)
}

/// `level` 消息当前是否值得构造与落盘（热路径调用点前置守卫用）：
/// 哨兵模式恒真（全量写），产品模式按级别门判断。
fn should_log(level: zhu_ye_core::LogLevel) -> bool {
    file_log_enabled() || level <= product_log_level()
}

/// 产品日志主文件路径：`%LOCALAPPDATA%\zhu-ye-ime\logs\ime.log`
/// （与设置窗口 `product_log_dir` 同源；`%LOCALAPPDATA%` 缺失返回 `None`）。
fn product_log_path() -> Option<PathBuf> {
    let root = std::env::var_os("LOCALAPPDATA")?;
    Some(
        PathBuf::from(root)
            .join("zhu-ye-ime")
            .join("logs")
            .join("ime.log"),
    )
}

/// 产品轨 `FileLogger` 惰性装配：首次真正需要写时解析路径（设计 §4.2）；
/// `%LOCALAPPDATA%` 缺失时返回 `None`（尽力而为，仅保留调试器输出）。
fn build_product_logger() -> Option<zhu_ye_core::FileLogger> {
    Some(zhu_ye_core::FileLogger::new(
        product_log_path()?,
        product_log_level(),
    ))
}

/// 产品轨 logger 静态持有：TSF 事件线程内串行访问，`Mutex` 兜底跨线程回调
/// （core `FileLogger` 本身无锁，见诊断产品化设计 §3）。
static PRODUCT_LOGGER: Mutex<Option<zhu_ye_core::FileLogger>> = Mutex::new(None);

/// 验收期哨兵文件路径与开关缓存（D-73 双轨之哨兵轨）。
const FILE_LOG_PATH: &str = r"C:\zhu-ye-test\tsf-debug.log";
const FILE_LOG_SENTINEL: &str = r"C:\zhu-ye-test\tsf-debug.enable";
static FILE_LOG_ENABLED: AtomicBool = AtomicBool::new(false);
static FILE_LOG_CHECKED: AtomicBool = AtomicBool::new(false);

/// 哨兵存在性判定（D-73）：`tsf-debug.enable` 存在即启用全量写旧路径。
/// 抽成参数化纯函数便于单测（真实哨兵路径是 VM 取证契约，不得注入/改动）。
fn sentinel_enabled(sentinel: &std::path::Path) -> bool {
    sentinel.exists()
}

fn file_log_enabled() -> bool {
    if !FILE_LOG_CHECKED.load(Ordering::Relaxed) {
        FILE_LOG_ENABLED.store(
            sentinel_enabled(Path::new(FILE_LOG_SENTINEL)),
            Ordering::Relaxed,
        );
        FILE_LOG_CHECKED.store(true, Ordering::Relaxed);
    }
    FILE_LOG_ENABLED.load(Ordering::Relaxed)
}

#[cfg(test)]
mod log_tests {
    //! 产品化日志（FR-060，T-091）路径与哨兵判定的单元测试。
    //!
    //! 只测纯函数（路径拼接、哨兵存在性），不触碰进程级静态缓存
    //! （`FILE_LOG_CHECKED` 首次判定后锁定）与真实 `C:\zhu-ye-test`。
    //! 环境变量操作在单个测试内串行完成，规避并行线程互相覆盖。

    use super::{product_log_path, sentinel_enabled};

    /// 测试专属临时目录（创建后保留；进程内计数唯一，产物不入库）。
    fn temp_dir() -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!(
            "zy-tsf-log-test-{}-{}-{}",
            std::process::id(),
            stamp,
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).expect("创建测试临时目录");
        dir
    }

    #[test]
    fn 产品日志路径随本地应用数据目录解析且缺失时为空() {
        let original = std::env::var_os("LOCALAPPDATA");
        let dir = temp_dir();
        std::env::set_var("LOCALAPPDATA", &dir);
        let expected = dir.join("zhu-ye-ime").join("logs").join("ime.log");
        assert_eq!(product_log_path(), Some(expected));
        std::env::remove_var("LOCALAPPDATA");
        assert_eq!(product_log_path(), None);
        if let Some(path) = original {
            std::env::set_var("LOCALAPPDATA", path);
        }
    }

    #[test]
    fn 哨兵文件存在性判定随文件创建与删除变化() {
        let sentinel = temp_dir().join("tsf-debug.enable");
        assert!(!sentinel_enabled(&sentinel));
        std::fs::write(&sentinel, "").expect("创建临时哨兵");
        assert!(sentinel_enabled(&sentinel));
        std::fs::remove_file(&sentinel).expect("删除临时哨兵");
        assert!(!sentinel_enabled(&sentinel));
    }
}

/// 创建输入引擎（M6-R 多包装配）。
///
/// 装配顺序（方案设计 11.3）：
/// 1. 基础包：`ZHU_YE_DICT_PATH` 覆盖 > DLL 同目录 `dictionary.zyct` > `%APPDATA%`；
/// 2. 读 `%APPDATA%\zhu-ye-ime\config.json` 的 `enabled_packs`，在
///    `%APPDATA%\zhu-ye-ime\packs\` 下按 `<id>.zyct` 解析领域包；
/// 3. 全部包合并为 `CompositeDictionary`（词频取 max）；
/// 4. 网络语包单独挂到引擎上，供缩写路径（FR-016/FR-017）查询。
///
/// 任一包打开失败只跳过该包并记日志，绝不阻断输入；无任何可用包时回退内置演示词典。
fn create_engine(user_store: Option<UserDictStore>) -> InputEngine {
    let (config, config_diagnostic) = zhu_ye_core::load_config(&config_path());
    if let Some(diagnostic) = config_diagnostic {
        product_log(
            zhu_ye_core::LogLevel::Error,
            &format!("zhu-ye: config-warn {diagnostic}"),
        );
    }

    let plan = assembly_plan(&config);
    for id in &plan.unknown {
        product_log(
            zhu_ye_core::LogLevel::Warn,
            &format!("zhu-ye: config-unknown-pack id={id}"),
        );
    }
    for id in &plan.missing {
        product_log(
            zhu_ye_core::LogLevel::Warn,
            &format!("zhu-ye: config-missing-pack id={id}"),
        );
    }

    let paths = plan.paths();
    let (composite, skipped) = CompositeDictionary::from_paths(&paths);
    for diagnostic in &skipped {
        product_log(
            zhu_ye_core::LogLevel::Error,
            &format!("zhu-ye: pack-skipped {diagnostic}"),
        );
    }

    // 英文词表文件（T-085）：缺失/失败回退内嵌静态表，不阻断装配。
    let en_lexicon = en_lexicon();

    if composite.is_empty() {
        product_log(
            zhu_ye_core::LogLevel::Error,
            &format!(
                "zhu-ye: dict-fallback path={:?}",
                plan.base.clone().unwrap_or_default()
            ),
        );
        let mut engine = match user_store {
            Some(store) => InputEngine::with_user_store(m1_seed_dictionary(), store),
            None => InputEngine::with_m1_seed(),
        };
        if let Some(lexicon) = en_lexicon {
            engine = engine.with_en_lexicon(lexicon);
        }
        return engine;
    }

    for (name, entries) in composite.describe() {
        product_log(
            zhu_ye_core::LogLevel::Info,
            &format!("zhu-ye: pack-ok path={name:?} entries={entries}"),
        );
    }
    product_log(
        zhu_ye_core::LogLevel::Info,
        &format!(
            "zhu-ye: composite-ok packs={} enabled={:?} online_update={}",
            composite.file_count(),
            config.enabled_packs,
            config.online_update
        ),
    );

    let slang = slang_pack(&plan);
    let dictionary: Arc<dyn Dictionary> = Arc::new(composite.clone());
    let bigram: Arc<dyn BigramModel> = Arc::new(composite);
    let engine = match user_store {
        Some(store) => InputEngine::with_user_store_and_bigram(dictionary, store, bigram),
        None => InputEngine::with_bigram(dictionary, bigram),
    };
    let engine = domain_engine(engine, &plan, config.enable_domain_boost);
    // O-05 修订（T-103）：简拼与模糊音可配置关闭；默认开（ConfigFile 缺省 true）。
    let engine = engine.with_abbreviation(config.enable_abbreviation);
    let engine = engine.with_fuzzy(config.enable_fuzzy);
    let engine = contact_engine(engine, &config);
    let engine = if let Some(lexicon) = en_lexicon {
        engine.with_en_lexicon(lexicon)
    } else {
        engine
    };
    match slang {
        Some(slang) => {
            product_log(zhu_ye_core::LogLevel::Info, "zhu-ye: slang-path enabled");
            engine.with_slang(slang)
        }
        None => engine,
    }
}

/// 挂载领域提权（FR-033/FR-034，场景8）：把已启用领域包按 **id 字典序** 排序后
/// 单独挂到引擎（D-16：多包同命中取字典序首个）；仅当总开关打开时参与提权
/// （D-14 默认开；D-17 仅对领域候选生效，不改用户词/联想）。
///
/// 领域包文件独立再开一份只读映射（composite 已合并过一份）；mmap 共享页缓存，
/// 只对已启用包（P-12），内存代价可忽略。打开失败静默跳过——composite 阶段
/// 已对同一批文件记过 `pack-skipped` 日志，不重复报。
fn domain_engine(
    engine: InputEngine,
    plan: &zhu_ye_core::PackPlan,
    boost_enabled: bool,
) -> InputEngine {
    if !boost_enabled || plan.pack_ids.is_empty() {
        return engine;
    }
    let mut paired: Vec<(&str, &std::path::Path)> = plan
        .pack_ids
        .iter()
        .zip(plan.packs.iter())
        .map(|(id, path)| (id.as_str(), path.as_path()))
        .collect();
    paired.sort_by(|a, b| a.0.cmp(b.0));
    let mut packs: Vec<(String, Arc<dyn Dictionary>)> = Vec::with_capacity(paired.len());
    for (id, path) in paired {
        // 打开失败静默跳过——composite 阶段已对同一批文件记过 `pack-skipped`。
        if let Ok(file) = DictionaryFile::open(path) {
            packs.push((id.to_owned(), Arc::new(file)));
        }
    }
    if packs.is_empty() {
        return engine;
    }
    product_log(
        zhu_ye_core::LogLevel::Info,
        &format!(
            "zhu-ye: domain-boost enabled packs={}",
            packs
                .iter()
                .map(|(id, _)| id.as_str())
                .collect::<Vec<_>>()
                .join(",")
        ),
    );
    engine.with_domain_packs(packs)
}

/// 挂载联系人索引（FR-036/FR-037，场景9）：按配置 `contact_vcards` 路径逐个读取
/// vCard 文件 → 解析（无效卡/不可读文件记录日志并跳过）→ 汇总姓名 → 建索引
/// （kTGHZ2013 注音全读形 D-22 + 简拼键）。无配置或全部文件无效时保持无联系人
/// 基线（T-050 不漂移）。
fn contact_engine(engine: InputEngine, config: &zhu_ye_core::ConfigFile) -> InputEngine {
    use std::io::Read as _;
    if config.contact_vcards.is_empty() {
        return engine;
    }
    let mut contacts = Vec::new();
    for path in &config.contact_vcards {
        let Ok(mut file) = std::fs::File::open(path) else {
            product_log(
                zhu_ye_core::LogLevel::Warn,
                &format!("zhu-ye: contact-vcf-missing path={path:?}"),
            );
            continue;
        };
        let mut text = String::new();
        if file.read_to_string(&mut text).is_err() {
            product_log(
                zhu_ye_core::LogLevel::Warn,
                &format!("zhu-ye: contact-vcf-unreadable path={path:?}"),
            );
            continue;
        }
        match parse_vcard(&text) {
            Ok(mut parsed) => {
                product_log(
                    zhu_ye_core::LogLevel::Info,
                    &format!(
                        "zhu-ye: contact-vcf-ok path={path:?} entries={}",
                        parsed.len()
                    ),
                );
                contacts.append(&mut parsed);
            }
            Err(error) => product_log(
                zhu_ye_core::LogLevel::Warn,
                &format!("zhu-ye: contact-vcf-parse-err path={path:?} {error:?}"),
            ),
        }
    }
    if contacts.is_empty() {
        return engine;
    }
    let index = build_contact_index(&contacts);
    product_log(
        zhu_ye_core::LogLevel::Info,
        &format!(
            "zhu-ye: contact-index index={} contacts={}",
            index.len(),
            contacts.len()
        ),
    );
    engine.with_contacts(index)
}

/// 计算多包装配计划：基础包目录取 DLL 同目录（或环境变量覆盖的父目录）。
fn assembly_plan(config: &zhu_ye_core::ConfigFile) -> zhu_ye_core::PackPlan {
    let (base_dir, packs_dir) = assembly_dirs();
    zhu_ye_core::plan_packs(config, &base_dir, &packs_dir)
}

/// 基础包所在目录与领域包目录。
fn assembly_dirs() -> (PathBuf, PathBuf) {
    let packs_dir = appdata_root()
        .map(|root| root.join(zhu_ye_core::PACKS_DIR_NAME))
        .unwrap_or_else(|| PathBuf::from(zhu_ye_core::PACKS_DIR_NAME));
    let base_dir = resolve_base_dir(
        std::env::var_os("ZHU_YE_DICT_PATH").map(PathBuf::from),
        installed_dictionary_path(),
        appdata_root(),
    );
    (base_dir, packs_dir)
}

/// 网络语包路径：装配计划里文件名为 `slang.zyct` 的包。
fn slang_pack(plan: &zhu_ye_core::PackPlan) -> Option<Arc<dyn Dictionary>> {
    let path = plan
        .packs
        .iter()
        .find(|path| path.file_stem().and_then(|s| s.to_str()) == Some("slang"))?;
    match DictionaryFile::open(path) {
        Ok(file) => Some(Arc::new(file)),
        Err(error) => {
            product_log(
                zhu_ye_core::LogLevel::Warn,
                &format!("zhu-ye: slang-open-failed {error}"),
            );
            None
        }
    }
}

/// 配置目录：`%APPDATA%\zhu-ye-ime`。
fn appdata_root() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|root| PathBuf::from(root).join("zhu-ye-ime"))
}

/// 配置文件路径：`%APPDATA%\zhu-ye-ime\config.json`。
fn config_path() -> PathBuf {
    appdata_root()
        .map(|root| root.join("config.json"))
        .unwrap_or_else(|| PathBuf::from("config.json"))
}

/// 基础包目录解析（M6-R）：显式环境变量 > DLL 同目录（安装器写入）
/// > 用户数据目录。返回目录由调用方拼接 `dictionary.zyct`。
///
/// 保留 T-022 确立的优先级，只是从"选出一个文件"改为"选出一个目录"——
/// 多包装配需要目录，而不是单个文件路径。
fn resolve_base_dir(
    override_path: Option<PathBuf>,
    installed: Option<PathBuf>,
    appdata: Option<PathBuf>,
) -> PathBuf {
    if let Some(path) = override_path {
        if let Some(parent) = path.parent() {
            return parent.to_path_buf();
        }
    }
    if let Some(path) = installed {
        if let Some(parent) = path.parent() {
            return parent.to_path_buf();
        }
    }
    appdata.unwrap_or_else(|| PathBuf::from("."))
}

/// 本 DLL 内的锚点函数：取其地址经 `GetModuleHandleExW(FROM_ADDRESS)`
/// 反查 DLL 所在目录，与 DLL 文件名解耦（安装脚本用版本化文件名
/// `zhu-ye-ime-vN.dll` 规避“被 explorer 锁定”问题，不能按名查找）。
fn dictionary_module_anchor() {}

/// zhu-ye-updater.exe 路径：与 DLL 同目录（安装/便携结构一致）。
fn installed_updater_path() -> Option<PathBuf> {
    installed_module_file("zhu-ye-updater.exe")
}

/// 返回 DLL 同目录存在的 `dictionary.zyct`；便于安装器做机器级部署。
fn installed_dictionary_path() -> Option<PathBuf> {
    installed_module_file(DICTIONARY_FILE_NAME)
}

/// 返回 DLL 同目录存在的 `en.zyen`（T-085，安装器/便携包随带）。
fn installed_en_wordbook_path() -> Option<PathBuf> {
    installed_module_file(EN_WORDBOOK_FILE_NAME)
}

/// 反查本 DLL 所在目录，返回其中存在的指定文件名（机器级部署）。
fn installed_module_file(name: &str) -> Option<PathBuf> {
    unsafe {
        let mut module = HMODULE::default();
        let address = (dictionary_module_anchor as fn()) as usize;
        let ok = GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            PCWSTR(address as *const u16),
            &mut module,
        );
        if ok.is_err() {
            return None;
        }
        let mut buffer = [0u16; 4096];
        let length = GetModuleFileNameW(Some(module), &mut buffer);
        if length == 0 {
            return None;
        }
        let module_path = String::from_utf16_lossy(&buffer[..length as usize]);
        let mut directory = PathBuf::from(module_path);
        directory.pop();
        let candidate = directory.join(name);
        candidate.exists().then_some(candidate)
    }
}

/// 装载英文词表（T-085）：DLL 同目录 `en.zyen` > `%APPDATA%\zhu-ye-ime\en.zyen`。
///
/// 任一候选存在但加载/校验失败 → `debug_log` 记录并回退（引擎侧走内嵌静态表）；
/// 都不存在 → 静默 `None`（与词典包打开失败同策略：绝不阻断输入）。
fn en_lexicon() -> Option<zhu_ye_core::EnLexicon> {
    let installed = installed_en_wordbook_path();
    let appdata = appdata_root().map(|root| root.join(EN_WORDBOOK_FILE_NAME));
    for candidate in [installed, appdata].into_iter().flatten() {
        match zhu_ye_core::EnLexicon::open(&candidate) {
            Ok(lexicon) => {
                product_log(
                    zhu_ye_core::LogLevel::Info,
                    &format!(
                        "zhu-ye: en-wordbook-ok path={:?} entries={}",
                        candidate,
                        lexicon.count()
                    ),
                );
                return Some(lexicon);
            }
            Err(error) => product_log(
                zhu_ye_core::LogLevel::Warn,
                &format!(
                    "zhu-ye: en-wordbook-failed path={:?} error={}",
                    candidate, error
                ),
            ),
        }
    }
    None
}

/// 用户词库 JSON 路径：`%APPDATA%\zhu-ye-ime\user_words.json`。
fn user_words_path() -> PathBuf {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .map(|root| root.join("zhu-ye-ime").join("user_words.json"))
        .unwrap_or_else(|| PathBuf::from("user_words.json"))
}

fn state_lock(service: &TextService_Impl) -> impl std::ops::DerefMut<Target = EngineState> + '_ {
    service.state().lock().unwrap()
}

/// 活动对象计数加一；创建成功后必须由对应对象的 `Drop` 减一。
fn object_created() {
    ACTIVE_OBJECTS.fetch_add(1, Ordering::Relaxed);
}

/// 活动对象计数减一；只在对象 `Drop` 时调用。
fn object_released() {
    ACTIVE_OBJECTS.fetch_sub(1, Ordering::Relaxed);
}

/// 创建类工厂并计入活动对象数；失败路径由 `Drop` 回滚计数。
fn create_class_factory(user_store: Option<UserDictStore>) -> IClassFactory {
    object_created();
    ClassFactory { user_store }.into()
}

/// 创建文本服务并计入活动对象数；失败路径由 `Drop` 回滚计数。
fn create_text_service(user_store: Option<UserDictStore>) -> IUnknown {
    object_created();
    TextService {
        state: Arc::new(SharedEngine(Mutex::new(EngineState::with_start_mode(
            user_store,
            configured_default_mode(),
        )))),
    }
    .into()
}

/// DLL 标准导出：按 CLSID 返回类工厂对象。
///
/// # Safety
/// 参数由 TSF 宿主按 COM 规范传入：非空指针写出接口指针。
#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: *const windows::core::GUID,
    riid: *const windows::core::GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if rclsid.is_null() || riid.is_null() || ppv.is_null() {
        return E_POINTER;
    }
    unsafe {
        *ppv = ptr::null_mut();
    }
    if unsafe { *rclsid } != CLSID_ZHU_YE_TIP {
        return CLASS_E_CLASSNOTAVAILABLE;
    }

    // 启动异步更新检查（T-088 / FR-048）：DLL 进入宿主进程（ctfmon/explorer/
    // 编辑器）时最多尝试一次；更新器子进程 detached + 低优先级，<10ms 返回。
    spawn_update_check_once();

    let factory: IUnknown =
        create_class_factory(Some(UserDictStore::new(user_words_path()))).into();
    unsafe { factory.query(riid, ppv) }
}

/// 启动异步更新检查的一次性开关（T-088 / FR-048）：每个宿主进程至多尝试一次，
/// 失败也不重试；进程内不做任何网络请求（S-4，检查发生在 updater 子进程）。
static UPDATE_CHECK_SPAWNED: AtomicBool = AtomicBool::new(false);

/// 启动异步更新检查（每宿主进程一次）。
///
/// 前置判定全部为只读：`online_update` 关闭 → 零 spawn（验收：默认关闭零网络零进程）；
/// 开启但距上次检查不足 7 天（`check_due`）→ 零 spawn 零网络。满足条件才
/// detach 拉起 `zhu-ye-updater.exe check-once`：`CREATE_NO_WINDOW` 无黑窗、
/// `BELOW_NORMAL_PRIORITY_CLASS` 低优先级（≤10ms 返回）；写盘（update_status.json、
/// `last_check`）由 updater 子进程完成，IME 进程内零写。
fn spawn_update_check_once() {
    use std::os::windows::process::CommandExt;
    if UPDATE_CHECK_SPAWNED.swap(true, Ordering::Relaxed) {
        return; // 每宿主进程一次，避免反复 spawn。
    }
    let (config, _) = zhu_ye_core::load_config(&config_path());
    if !config.online_update {
        return; // 默认关闭：零 spawn。
    }
    if !zhu_ye_core::check_due(config.last_check, zhu_ye_core::unix_now()) {
        return; // 距上次检查不足 7 天：零网络。
    }
    let Some(updater_exe) = installed_updater_path() else {
        return; // 便携/开发目录没有更新器：静默跳过。
    };
    let _ = std::process::Command::new(updater_exe)
        .arg("check-once")
        // CREATE_NO_WINDOW(0x0800_0000) | BELOW_NORMAL_PRIORITY_CLASS(0x0000_4000)。
        .creation_flags(0x0800_0000 | 0x0000_4000)
        .spawn();
}

/// DLL 标准导出：无活动对象且未被 LockServer 时允许卸载。
#[no_mangle]
pub extern "system" fn DllCanUnloadNow() -> HRESULT {
    if ACTIVE_OBJECTS.load(Ordering::Relaxed) == 0 && SERVER_LOCKS.load(Ordering::Relaxed) == 0 {
        S_OK
    } else {
        S_FALSE
    }
}

/// 加载探针：宿主进程可通过导出符号确认 DLL 已加载。
#[must_use]
#[no_mangle]
pub extern "system" fn dll_probe() -> u32 {
    0x5A48_4559
}

/// 返回适配层依赖的核心库版本，供自检输出。
#[must_use]
pub fn paired_core_version() -> &'static str {
    core_version()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        VK_0, VK_1, VK_2, VK_5, VK_I, VK_OEM_COMMA, VK_OEM_PERIOD, VK_V, VK_X,
    };
    use windows::Win32::UI::TextServices::{ITfTextInputProcessor, ITfThreadMgr};

    /// 生命周期计数是全局状态；测试并行运行时互斥，避免相互干扰。
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    /// T-046：共享引擎状态必须可跨线程迁移——语言栏/ctfmon 可能经任意
    /// RPC 线程回调 `OnClick` 进入 `Arc<SharedEngine>` 切换模式。
    #[test]
    fn 引擎共享状态可跨线程迁移() {
        fn assert_send<T: Send + Sync>() {}
        assert_send::<SharedEngine>();
    }

    /// T-046：模式切换处理器闭包（Weak<…> 捕获）必须可跨线程。
    #[test]
    fn 语言栏点击处理器可跨线程() {
        let state = Arc::new(SharedEngine(Mutex::new(EngineState::new())));
        let weak = Arc::downgrade(&state);
        let handler = move || {
            let state = weak.upgrade()?;
            let mut guard = state.lock().ok()?;
            guard.engine.toggle_mode();
            Some(guard.engine.mode())
        };
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Box<dyn Fn() -> Option<crate::input::InputMode> + Send + Sync>>();
        let _ = handler;
    }

    #[test]
    fn 类工厂可创建文本服务() {
        let _guard = TEST_LOCK.lock().unwrap();
        let factory = create_class_factory(None);
        let service: ITfTextInputProcessorEx =
            unsafe { factory.CreateInstance(Option::<&IUnknown>::None) }.unwrap();
        let activated = unsafe { service.ActivateEx(Option::<&ITfThreadMgr>::None, 0, 0) };
        assert!(activated.is_ok());
    }

    #[test]
    fn 拒绝聚合创建() {
        let _guard = TEST_LOCK.lock().unwrap();
        let factory = create_class_factory(None);
        let aggregate = factory.cast::<IUnknown>().unwrap();
        let result: windows::core::Result<ITfTextInputProcessor> =
            unsafe { factory.CreateInstance(Some(&aggregate)) };
        assert!(result.is_err());
    }

    #[test]
    fn 探针与版本可工作() {
        assert_eq!(dll_probe(), 0x5A48_4559);
        assert!(!paired_core_version().is_empty());
    }

    #[test]
    fn 活动对象为零时可卸载() {
        let _guard = TEST_LOCK.lock().unwrap();
        assert_eq!(DllCanUnloadNow(), S_OK);
        let factory = create_class_factory(None);
        assert_eq!(DllCanUnloadNow(), S_FALSE);
        drop(factory);
        assert_eq!(DllCanUnloadNow(), S_OK);
    }

    #[test]
    fn 键分类覆盖字母与功能键() {
        assert_eq!(
            classify_key(WPARAM(VK_A.0 as usize), LPARAM(0), false),
            Some(KeyAction::Letter('a'))
        );
        assert_eq!(
            classify_key(WPARAM(VK_Z.0 as usize), LPARAM(0), false),
            Some(KeyAction::Letter('z'))
        );
        assert_eq!(
            classify_key(WPARAM(VK_BACK.0 as usize), LPARAM(0), false),
            Some(KeyAction::Backspace)
        );
        assert_eq!(
            classify_key(WPARAM(VK_SPACE.0 as usize), LPARAM(0), false),
            Some(KeyAction::Space)
        );
        assert_eq!(
            classify_key(WPARAM(VK_RETURN.0 as usize), LPARAM(0), false),
            Some(KeyAction::Enter)
        );
        assert_eq!(
            classify_key(WPARAM(VK_ESCAPE.0 as usize), LPARAM(0), false),
            Some(KeyAction::Escape)
        );
        assert_eq!(
            classify_key(WPARAM(VK_1.0 as usize), LPARAM(0), false),
            Some(KeyAction::Digit('1'))
        );
        assert_eq!(
            classify_key(WPARAM(VK_9.0 as usize), LPARAM(0), false),
            Some(KeyAction::Digit('9'))
        );
        assert_eq!(
            classify_key(WPARAM(VK_0.0 as usize), LPARAM(0), false),
            Some(KeyAction::Digit('0'))
        );
        assert_eq!(classify_key(WPARAM(0x00A0), LPARAM(0), false), None);
    }

    #[test]
    fn 未激活组合时功能键放行字母进入引擎() {
        let state = Arc::new(SharedEngine(Mutex::new(EngineState::new())));
        assert_eq!(
            plan_action(WPARAM(VK_A.0 as usize), LPARAM(0), false, false, &state),
            Some(KeyAction::Letter('a'))
        );
        assert_eq!(
            plan_action(WPARAM(VK_SPACE.0 as usize), LPARAM(0), false, false, &state),
            None
        );
        assert_eq!(
            plan_action(
                WPARAM(VK_RETURN.0 as usize),
                LPARAM(0),
                false,
                false,
                &state
            ),
            None
        );
        assert_eq!(
            plan_action(
                WPARAM(VK_ESCAPE.0 as usize),
                LPARAM(0),
                false,
                false,
                &state
            ),
            None
        );

        state.lock().unwrap().engine.handle_letter('a');
        assert_eq!(
            plan_action(WPARAM(VK_SPACE.0 as usize), LPARAM(0), false, false, &state),
            Some(KeyAction::Space)
        );
        assert_eq!(
            plan_action(WPARAM(VK_1.0 as usize), LPARAM(0), false, false, &state),
            Some(KeyAction::Select(0))
        );
    }

    /// T-049 + FR-027：数字键在"非缩写键前缀"场景保持原语义——有候选时选词；
    /// 空闲态数字进入数字格式模式（BufferDigit 边输边上屏，替代放行直出）。
    #[test]
    fn 数字键非缩写前缀时保持选词与直出语义() {
        let state = Arc::new(SharedEngine(Mutex::new(EngineState::new())));
        // 空组合、无候选：进入数字格式模式（FR-027，引擎替代宿主直出数字）。
        assert_eq!(
            plan_action(WPARAM(VK_1.0 as usize), LPARAM(0), false, false, &state),
            Some(KeyAction::BufferDigit('1'))
        );
        // 有候选：数字仍是选词（FR-006 不回归）。
        state.lock().unwrap().engine.handle_letter('n');
        state.lock().unwrap().engine.handle_letter('i');
        assert_eq!(
            plan_action(WPARAM(VK_1.0 as usize), LPARAM(0), false, false, &state),
            Some(KeyAction::Select(0))
        );
        // 英文模式下数字一律放行。
        state.lock().unwrap().engine.toggle_mode();
        assert_eq!(
            plan_action(WPARAM(VK_1.0 as usize), LPARAM(0), false, false, &state),
            None
        );
    }

    /// T-049：当数字是某含数字缩写键的组成部分时进入组合串，而非选词。
    #[test]
    fn 数字键为缩写前缀时进入组合串() {
        let mut state = EngineState::new();
        state.engine = InputEngine::new(Arc::new(zhu_ye_core::InMemoryDictionary::from_entries(
            vec![
                zhu_ye_core::DictionaryEntry::new("九九六", "996", 5000),
                zhu_ye_core::DictionaryEntry::new("有一说一", "u1s1", 5000),
            ],
        )));
        let state = Arc::new(SharedEngine(Mutex::new(state)));

        // `9` 是 `996` 的前缀：进组合串而非选词。
        assert_eq!(
            plan_action(WPARAM(VK_9.0 as usize), LPARAM(0), false, false, &state),
            Some(KeyAction::Digit('9'))
        );
        // `u` 是 `u1s1` 的前缀：字母照常进入组合。
        assert_eq!(
            plan_action(WPARAM(0x55), LPARAM(0), false, false, &state),
            Some(KeyAction::Letter('u'))
        );
        state.lock().unwrap().engine.handle_letter('u');
        // 组合 `u` 之后按 `1`：仍是 `u1s1` 前缀，进组合串。
        assert_eq!(
            plan_action(WPARAM(VK_1.0 as usize), LPARAM(0), false, false, &state),
            Some(KeyAction::Digit('1'))
        );
    }

    #[test]
    fn ctrl或alt修饰键一律放行给宿主() {
        let state = Arc::new(SharedEngine(Mutex::new(EngineState::new())));
        assert_eq!(
            plan_action(WPARAM(VK_A.0 as usize), LPARAM(0), true, false, &state),
            None
        );
        assert_eq!(
            plan_action(WPARAM(VK_SPACE.0 as usize), LPARAM(0), true, false, &state),
            None
        );

        // 组合进行中按 Ctrl+S 等系统组合键也不被输入法吞掉。
        state.lock().unwrap().engine.handle_letter('n');
        assert_eq!(
            plan_action(WPARAM(0x53), LPARAM(0), true, false, &state),
            None
        );
        // 无修饰键时字母仍正常进入组合。
        assert_eq!(
            plan_action(WPARAM(0x53), LPARAM(0), false, false, &state),
            Some(KeyAction::Letter('s'))
        );
    }

    #[test]
    fn 宽字符转换以空字符结尾() {
        let wide = to_wide("你好");
        assert_eq!(wide, vec![0x4F60, 0x597D, 0]);
        assert_eq!(to_wide(""), vec![0]);
    }

    #[test]
    fn 基础包目录优先环境变量再安装目录再用户目录() {
        let override_path = Some(PathBuf::from("D:\\custom\\my.zyct"));
        let installed = Some(PathBuf::from(
            "C:\\Program Files\\zhu-ye-ime\\tsf\\dictionary.zyct",
        ));
        let appdata = Some(PathBuf::from("%APPDATA%\\zhu-ye-ime"));

        // 环境变量优先，取其父目录（VM 逐包切换测试依赖此语义）。
        assert_eq!(
            resolve_base_dir(override_path.clone(), installed.clone(), appdata.clone()),
            PathBuf::from("D:\\custom")
        );
        // 其次 DLL 同目录。
        assert_eq!(
            resolve_base_dir(None, installed.clone(), appdata.clone()),
            PathBuf::from("C:\\Program Files\\zhu-ye-ime\\tsf")
        );
        // 再次用户数据目录。
        assert_eq!(
            resolve_base_dir(None, None, appdata.clone()),
            appdata.clone().unwrap()
        );
        // 全空时回退当前目录，保证仍可启动。
        assert_eq!(resolve_base_dir(None, None, None), PathBuf::from("."));
    }

    #[test]
    fn 引擎推进与组合文本预览一致() {
        let state = Arc::new(SharedEngine(Mutex::new(EngineState::new())));
        assert_eq!(compose_text(&state, KeyAction::Letter('n')), "n");
        sync_engine(&state, KeyAction::Letter('n'));
        assert_eq!(compose_text(&state, KeyAction::Letter('i')), "ni");
        sync_engine(&state, KeyAction::Letter('i'));
        assert_eq!(commit_text(&state, KeyAction::Space), "你");
        sync_engine(&state, KeyAction::Space);
        assert!(!state.lock().unwrap().engine.is_active());
    }

    #[test]
    fn 键分类覆盖shift_tab与翻页键() {
        assert_eq!(
            classify_key(WPARAM(VK_SHIFT.0 as usize), LPARAM(0), false),
            Some(KeyAction::ToggleMode)
        );
        assert_eq!(
            classify_key(WPARAM(VK_TAB.0 as usize), LPARAM(0), false),
            Some(KeyAction::ToggleLayer)
        );
        // T-033：`-` 上翻、`=`/`+` 下翻；逗号句号不再映射翻页。
        assert_eq!(
            classify_key(WPARAM(VK_OEM_MINUS.0 as usize), LPARAM(0), false),
            Some(KeyAction::PageUp)
        );
        assert_eq!(
            classify_key(WPARAM(VK_OEM_PLUS.0 as usize), LPARAM(0), false),
            Some(KeyAction::PageDown)
        );
        assert_eq!(
            classify_key(WPARAM(VK_OEM_COMMA.0 as usize), LPARAM(0), false),
            None
        );
        // FR-027：`.`（VK_OEM_PERIOD/小键盘 VK_DECIMAL）归为 Dot，
        // 数字格式模式内追加小数、模式外放行宿主。
        assert_eq!(
            classify_key(WPARAM(VK_OEM_PERIOD.0 as usize), LPARAM(0), false),
            Some(KeyAction::Dot)
        );
        assert_eq!(
            classify_key(WPARAM(VK_DECIMAL.0 as usize), LPARAM(0), false),
            Some(KeyAction::Dot)
        );
        // T-039：上下方向键移动页内选中行。
        assert_eq!(
            classify_key(WPARAM(VK_UP.0 as usize), LPARAM(0), false),
            Some(KeyAction::SelectUp)
        );
        assert_eq!(
            classify_key(WPARAM(VK_DOWN.0 as usize), LPARAM(0), false),
            Some(KeyAction::SelectDown)
        );
    }

    #[test]
    fn shift长按重复事件不重复切换() {
        assert_eq!(
            classify_key(WPARAM(VK_SHIFT.0 as usize), LPARAM(0), false),
            Some(KeyAction::ToggleMode)
        );
        assert_eq!(
            classify_key(WPARAM(VK_SHIFT.0 as usize), LPARAM(0x4000_0000), false),
            None
        );
        assert!(is_repeat(LPARAM(0x4000_0000)));
        assert!(!is_repeat(LPARAM(0)));
    }

    #[test]
    fn shift与组合内功能键放行策略正确() {
        let state = Arc::new(SharedEngine(Mutex::new(EngineState::new())));
        assert_eq!(
            plan_action(WPARAM(VK_SHIFT.0 as usize), LPARAM(0), false, false, &state),
            Some(KeyAction::ToggleMode)
        );
        assert_eq!(
            plan_action(WPARAM(VK_TAB.0 as usize), LPARAM(0), false, false, &state),
            None
        );
        // T-039：无组合时方向键放行给宿主（不干扰光标移动）。
        assert_eq!(
            plan_action(WPARAM(VK_DOWN.0 as usize), LPARAM(0), false, false, &state),
            None
        );

        // T-033：组合进行中 Shift 不切换模式（`+` 翻页时 Shift 只是修饰键），
        // Tab 与 `=`/`+` 翻页照常吃下。
        state.lock().unwrap().engine.handle_letter('n');
        assert_eq!(
            plan_action(WPARAM(VK_SHIFT.0 as usize), LPARAM(0), false, false, &state),
            None
        );
        assert_eq!(
            plan_action(WPARAM(VK_TAB.0 as usize), LPARAM(0), false, false, &state),
            Some(KeyAction::ToggleLayer)
        );
        assert_eq!(
            plan_action(
                WPARAM(VK_OEM_PLUS.0 as usize),
                LPARAM(0),
                false,
                false,
                &state
            ),
            Some(KeyAction::PageDown)
        );
        assert_eq!(
            plan_action(
                WPARAM(VK_OEM_MINUS.0 as usize),
                LPARAM(0),
                false,
                false,
                &state
            ),
            Some(KeyAction::PageUp)
        );
        // T-039：组合活跃时方向键吃下并移动选中行；无组合时放行给宿主。
        assert_eq!(
            plan_action(WPARAM(VK_DOWN.0 as usize), LPARAM(0), false, false, &state),
            Some(KeyAction::SelectDown)
        );
        assert_eq!(
            plan_action(WPARAM(VK_UP.0 as usize), LPARAM(0), false, false, &state),
            Some(KeyAction::SelectUp)
        );
        // 修饰键按下时放行（方向键不参与系统组合，这里保证 Ctrl/Alt 场景不吞键）。
        assert_eq!(
            plan_action(WPARAM(VK_DOWN.0 as usize), LPARAM(0), true, false, &state),
            None
        );
    }

    #[test]
    fn 上下键推进页内选中行() {
        let state = Arc::new(SharedEngine(Mutex::new(EngineState::new())));
        sync_engine(&state, KeyAction::Letter('n'));
        sync_engine(&state, KeyAction::Letter('i'));
        sync_engine(&state, KeyAction::Letter('h'));
        sync_engine(&state, KeyAction::Letter('a'));
        sync_engine(&state, KeyAction::Letter('o'));
        sync_engine(&state, KeyAction::SelectDown);
        assert_eq!(state.lock().unwrap().engine.selected_on_page(), 1);
        sync_engine(&state, KeyAction::SelectUp);
        assert_eq!(state.lock().unwrap().engine.selected_on_page(), 0);
    }

    #[test]
    fn 状态动作推进图层与页码() {
        let state = Arc::new(SharedEngine(Mutex::new(EngineState::new())));
        sync_engine(&state, KeyAction::Letter('n'));
        sync_engine(&state, KeyAction::Letter('i'));
        sync_engine(&state, KeyAction::ToggleLayer);
        assert_eq!(
            state.lock().unwrap().engine.layer(),
            crate::input::CandidateLayer::Translation
        );
        let before = state.lock().unwrap().engine.page();
        sync_engine(&state, KeyAction::PageDown);
        assert_eq!(state.lock().unwrap().engine.page(), before);
        sync_engine(&state, KeyAction::ToggleMode);
        assert_eq!(state.lock().unwrap().engine.mode(), InputMode::English);
    }

    #[test]
    fn 英文模式放行制表与翻页键给宿主() {
        let state = Arc::new(SharedEngine(Mutex::new(EngineState::new())));
        sync_engine(&state, KeyAction::Letter('n'));
        sync_engine(&state, KeyAction::ToggleMode);
        assert_eq!(state.lock().unwrap().engine.mode(), InputMode::English);
        assert_eq!(
            plan_action(WPARAM(VK_TAB.0 as usize), LPARAM(0), false, false, &state),
            None
        );
        assert_eq!(
            plan_action(
                WPARAM(VK_OEM_PLUS.0 as usize),
                LPARAM(0),
                false,
                false,
                &state
            ),
            None
        );
    }

    // ---- 场景7（T-061）：数字格式模式 / v 模式 / emoji 的 TSF 键路 ----

    /// 构造进入数字格式模式（8 位日期串已累积）的 state。
    fn digit_state() -> Arc<SharedEngine> {
        let state = Arc::new(SharedEngine(Mutex::new(EngineState::new())));
        for digit in "20260930".chars() {
            sync_engine(&state, KeyAction::BufferDigit(digit));
        }
        assert!(state.lock().unwrap().engine.digit_active());
        state
    }

    /// 构造进入 v 模式且已输类型码（v1）的 state。
    fn v_code_state() -> Arc<SharedEngine> {
        let state = Arc::new(SharedEngine(Mutex::new(EngineState::new())));
        sync_engine(&state, KeyAction::VStart);
        sync_engine(&state, KeyAction::VCode('1'));
        assert!(state.lock().unwrap().engine.v_active());
        state
    }

    #[test]
    fn 空闲态数字启动数字格式模式() {
        let state = Arc::new(SharedEngine(Mutex::new(EngineState::new())));
        // 首数字键：启动数字模式并直插上屏（引擎累积），不再放行宿主。
        assert_eq!(
            plan_action(WPARAM(VK_1.0 as usize), LPARAM(0), false, false, &state),
            Some(KeyAction::BufferDigit('1'))
        );
        sync_engine(&state, KeyAction::BufferDigit('1'));
        assert_eq!(state.lock().unwrap().engine.digit_text(), "1");
        assert!(state.lock().unwrap().engine.digit_active());
        // 后续数字继续累积。
        assert_eq!(
            plan_action(WPARAM(VK_0.0 as usize), LPARAM(0), false, false, &state),
            Some(KeyAction::BufferDigit('0'))
        );
        // 不足 5 位时无格式候选，数字越界仍继续追加。
        assert_eq!(
            plan_action(WPARAM(VK_1.0 as usize), LPARAM(0), false, false, &state),
            Some(KeyAction::BufferDigit('1'))
        );
    }

    #[test]
    fn 数字模式选择替换与越界追加() {
        let state = digit_state();
        // 8 位日期、4 个格式候选：`1` 选中第 0 项走替换。
        assert_eq!(
            plan_action(WPARAM(VK_1.0 as usize), LPARAM(0), false, false, &state),
            Some(KeyAction::SelectAndReplace(0))
        );
        // 越界数字（5 及以上）继续追加，不吞键。
        assert_eq!(
            plan_action(WPARAM(VK_5.0 as usize), LPARAM(0), false, false, &state),
            Some(KeyAction::BufferDigit('5'))
        );
        // 替换预览：第 2 个候选 + 替换长度为 buffer UTF-16 长度（8 位日期）。
        let (text, len) = state
            .lock()
            .unwrap()
            .engine
            .preview_digit(1)
            .expect("第 2 个日期候选");
        assert_eq!(text, "2026/09/30");
        assert_eq!(len, 8, "20260930 为 8 位");
    }

    #[test]
    fn 数字模式小数点追加与退格() {
        let state = Arc::new(SharedEngine(Mutex::new(EngineState::new())));
        for digit in "12345".chars() {
            sync_engine(&state, KeyAction::BufferDigit(digit));
        }
        // `.` 在数字模式内追加（金额小数）。
        assert_eq!(
            plan_action(
                WPARAM(VK_OEM_PERIOD.0 as usize),
                LPARAM(0),
                false,
                false,
                &state
            ),
            Some(KeyAction::BufferDigit('.'))
        );
        sync_engine(&state, KeyAction::BufferDigit('.'));
        assert_eq!(state.lock().unwrap().engine.digit_text(), "12345.");
        // 数字模式退格删尾部位。
        assert_eq!(
            plan_action(WPARAM(VK_BACK.0 as usize), LPARAM(0), false, false, &state),
            Some(KeyAction::DigitBackspace)
        );
        sync_engine(&state, KeyAction::DigitBackspace);
        assert_eq!(state.lock().unwrap().engine.digit_text(), "12345");
    }

    #[test]
    fn 数字模式空格选当前行esc退出() {
        let state = digit_state();
        // 空格 = 选中当前选中行（默认 0）。
        assert_eq!(
            plan_action(WPARAM(VK_SPACE.0 as usize), LPARAM(0), false, false, &state),
            Some(KeyAction::SelectAndReplace(0))
        );
        // Esc 退出数字模式（数字正文保留，引擎清 buffer）。
        assert_eq!(
            plan_action(
                WPARAM(VK_ESCAPE.0 as usize),
                LPARAM(0),
                false,
                false,
                &state
            ),
            Some(KeyAction::Escape)
        );
        sync_engine(&state, KeyAction::Escape);
        assert!(!state.lock().unwrap().engine.digit_active());
        assert!(state.lock().unwrap().engine.candidates().is_empty());
    }

    #[test]
    fn 数字模式字母退出后进拼音组合() {
        let state = digit_state();
        // 字母键：退出数字模式并把字母交给拼音（plan 已同步退出）。
        assert_eq!(
            plan_action(WPARAM(VK_A.0 as usize), LPARAM(0), false, false, &state),
            Some(KeyAction::Letter('a'))
        );
        assert!(!state.lock().unwrap().engine.digit_active());
        sync_engine(&state, KeyAction::Letter('a'));
        assert_eq!(state.lock().unwrap().engine.composing(), "a");
    }

    #[test]
    fn 数字模式提交后清空状态() {
        let state = digit_state();
        sync_engine(&state, KeyAction::SelectAndReplace(1));
        {
            let engine = &state.lock().unwrap().engine;
            assert!(!engine.digit_active());
            assert_eq!(engine.previous_word(), Some("2026/09/30"));
        }
    }

    #[test]
    fn 空闲态v进入v模式组合态v走拼音() {
        let state = Arc::new(SharedEngine(Mutex::new(EngineState::new())));
        assert_eq!(
            plan_action(WPARAM(VK_V.0 as usize), LPARAM(0), false, false, &state),
            Some(KeyAction::VStart)
        );
        sync_engine(&state, KeyAction::VStart);
        assert!(state.lock().unwrap().engine.v_active());
        assert_eq!(state.lock().unwrap().engine.v_buffer_len(), 1);
        // 组合态（nv/lv）：v 是正常拼音字符，不进入 v 模式。
        let nv = Arc::new(SharedEngine(Mutex::new(EngineState::new())));
        sync_engine(&nv, KeyAction::Letter('n'));
        sync_engine(&nv, KeyAction::Letter('v'));
        assert!(!nv.lock().unwrap().engine.v_active());
    }

    #[test]
    fn v模式类型码与选择键() {
        let state = Arc::new(SharedEngine(Mutex::new(EngineState::new())));
        sync_engine(&state, KeyAction::VStart);
        // 等待类型码：`x` 出数学符号组。
        assert_eq!(
            plan_action(WPARAM(VK_X.0 as usize), LPARAM(0), false, false, &state),
            Some(KeyAction::VCode('x'))
        );
        sync_engine(&state, KeyAction::VCode('x'));
        assert_eq!(state.lock().unwrap().engine.v_symbol_count(), 9);
        // 已出组：数字 = 选择符号。
        assert_eq!(
            plan_action(WPARAM(VK_2.0 as usize), LPARAM(0), false, false, &state),
            Some(KeyAction::Select(1))
        );
        // `0` 无义：退出 v 模式并放行宿主。
        let zero = v_code_state();
        assert_eq!(
            plan_action(WPARAM(VK_0.0 as usize), LPARAM(0), false, false, &zero),
            None
        );
        assert!(!zero.lock().unwrap().engine.v_active());
    }

    #[test]
    fn v模式非法字母回退拼音组合() {
        let state = v_code_state();
        // `i`（vi）回退拼音：动作是 VConsume，组合文本 = "vi"。
        assert_eq!(
            plan_action(WPARAM(VK_I.0 as usize), LPARAM(0), false, false, &state),
            Some(KeyAction::VConsume('i'))
        );
        sync_engine(&state, KeyAction::VConsume('i'));
        assert!(!state.lock().unwrap().engine.v_active());
        assert_eq!(state.lock().unwrap().engine.composing(), "vi");
        assert_eq!(compose_text(&state, KeyAction::VConsume('i')), "vi");
    }

    #[test]
    fn v模式空格选首个符号esc与无效键退出() {
        let state = v_code_state();
        // 空格 = 选第 1 个符号（commit 直插后引擎提交符号）。
        assert_eq!(
            plan_action(WPARAM(VK_SPACE.0 as usize), LPARAM(0), false, false, &state),
            Some(KeyAction::Space)
        );
        // Esc 退出 v 模式。
        let esc = v_code_state();
        assert_eq!(
            plan_action(WPARAM(VK_ESCAPE.0 as usize), LPARAM(0), false, false, &esc),
            Some(KeyAction::Escape)
        );
        sync_engine(&esc, KeyAction::Escape);
        assert!(!esc.lock().unwrap().engine.v_active());
        // Enter 与 v 模式无关：先退出 v 模式再放行宿主。
        let enter = v_code_state();
        assert_eq!(
            plan_action(
                WPARAM(VK_RETURN.0 as usize),
                LPARAM(0),
                false,
                false,
                &enter
            ),
            None
        );
        assert!(!enter.lock().unwrap().engine.v_active());
    }

    #[test]
    fn v模式退格回退类型码() {
        let state = v_code_state();
        assert_eq!(
            plan_action(WPARAM(VK_BACK.0 as usize), LPARAM(0), false, false, &state),
            Some(KeyAction::VBackspace)
        );
        sync_engine(&state, KeyAction::VBackspace);
        assert!(state.lock().unwrap().engine.v_active());
        assert_eq!(state.lock().unwrap().engine.v_buffer_len(), 1);
        // 只剩 `v` 时再退格退出 v 模式。
        sync_engine(&state, KeyAction::VBackspace);
        assert!(!state.lock().unwrap().engine.v_active());
    }

    #[test]
    fn v模式再按v回退拼音不重复启动() {
        let state = v_code_state();
        assert_eq!(
            plan_action(WPARAM(VK_V.0 as usize), LPARAM(0), false, false, &state),
            Some(KeyAction::VConsume('v'))
        );
        sync_engine(&state, KeyAction::VConsume('v'));
        assert_eq!(state.lock().unwrap().engine.composing(), "vv");
        assert!(!state.lock().unwrap().engine.v_active());
    }

    #[test]
    fn 数字模式外小数点放行宿主() {
        let state = Arc::new(SharedEngine(Mutex::new(EngineState::new())));
        assert_eq!(
            plan_action(
                WPARAM(VK_OEM_PERIOD.0 as usize),
                LPARAM(0),
                false,
                false,
                &state
            ),
            None,
            "非数字模式 `.` 放行给宿主直出标点"
        );
    }

    /// 构造组合串为 `text` 的 state（T-066）：字母走 Letter、数字走 Digit、
    /// 其余（`@`/`.`/`/`/`:`）走 FormatChar。
    fn format_state(text: &str) -> Arc<SharedEngine> {
        let state = Arc::new(SharedEngine(Mutex::new(EngineState::new())));
        for c in text.chars() {
            let action = match c {
                'a'..='z' => KeyAction::Letter(c),
                '0'..='9' => KeyAction::Digit(c),
                _ => KeyAction::FormatChar(c),
            };
            sync_engine(&state, action);
        }
        state
    }

    #[test]
    fn shift加2归类at组合态进串空闲放行() {
        // 键分类：Shift+2 => `@`；无 Shift 的 2 仍是数字。
        assert_eq!(
            classify_key(WPARAM(VK_2.0 as usize), LPARAM(0), true),
            Some(KeyAction::FormatChar('@'))
        );
        assert_eq!(
            classify_key(WPARAM(VK_2.0 as usize), LPARAM(0), false),
            Some(KeyAction::Digit('2'))
        );
        // 组合态 Shift+2：吃键进 `@`。
        let state = format_state("me");
        assert_eq!(
            plan_action(WPARAM(VK_2.0 as usize), LPARAM(0), false, true, &state),
            Some(KeyAction::FormatChar('@'))
        );
        // 空闲态：放行宿主（`@` 不冷启动组合，D-11）。
        let idle = Arc::new(SharedEngine(Mutex::new(EngineState::new())));
        assert_eq!(
            plan_action(WPARAM(VK_2.0 as usize), LPARAM(0), false, true, &idle),
            None
        );
        // 英文模式：放行宿主。
        let english = Arc::new(SharedEngine(Mutex::new(EngineState::new())));
        sync_engine(&english, KeyAction::ToggleMode);
        assert_eq!(
            plan_action(WPARAM(VK_2.0 as usize), LPARAM(0), false, true, &english),
            None
        );
    }

    #[test]
    fn at进串后邮箱态候选与提交预览() {
        let state = format_state("me");
        sync_engine(&state, KeyAction::FormatChar('@'));
        assert_eq!(state.lock().unwrap().engine.composing(), "me@");
        let candidates = state
            .lock()
            .unwrap()
            .engine
            .candidates()
            .iter()
            .map(|c| c.text.clone())
            .collect::<Vec<_>>();
        assert_eq!(candidates, vec!["me@.com", "me@.cn", "me@.net"]);
        // 空格提交预览返回符合预期（邮箱态首候选）。
        assert_eq!(
            commit_text(&state, KeyAction::Space),
            "me@.com",
            "空格预览 = 邮箱补全首候选"
        );
    }

    #[test]
    fn 冒号与斜杠仅网址意图进串() {
        // 键分类：Shift+`;` => `:`；`;` 放行宿主；`/` 无 Shift；Shift+`/`（`?`）放行。
        assert_eq!(
            classify_key(WPARAM(VK_OEM_1.0 as usize), LPARAM(0), true),
            Some(KeyAction::FormatChar(':'))
        );
        assert_eq!(
            classify_key(WPARAM(VK_OEM_1.0 as usize), LPARAM(0), false),
            None
        );
        assert_eq!(
            classify_key(WPARAM(VK_OEM_2.0 as usize), LPARAM(0), false),
            Some(KeyAction::FormatChar('/'))
        );
        assert_eq!(
            classify_key(WPARAM(VK_OEM_2.0 as usize), LPARAM(0), true),
            None
        );
        // `http` 演进：`:` 与 `/` 逐键进串，最终命中网址态。
        let state = format_state("http");
        assert_eq!(
            plan_action(WPARAM(VK_OEM_1.0 as usize), LPARAM(0), false, true, &state),
            Some(KeyAction::FormatChar(':'))
        );
        sync_engine(&state, KeyAction::FormatChar(':'));
        assert_eq!(state.lock().unwrap().engine.composing(), "http:");
        for _ in 0..2 {
            assert_eq!(
                plan_action(WPARAM(VK_OEM_2.0 as usize), LPARAM(0), false, false, &state),
                Some(KeyAction::FormatChar('/'))
            );
            sync_engine(&state, KeyAction::FormatChar('/'));
        }
        assert_eq!(state.lock().unwrap().engine.composing(), "http://");
        // 结构相似但非网址意图（httpw）：`:` 放行宿主。
        let bad = format_state("httpw");
        assert_eq!(
            plan_action(WPARAM(VK_OEM_1.0 as usize), LPARAM(0), false, true, &bad),
            None
        );
        // 普通拼音组合后的 `/`：放行宿主（不改变既有标点直出语义）。
        let nihao = format_state("nihao");
        assert_eq!(
            plan_action(WPARAM(VK_OEM_2.0 as usize), LPARAM(0), false, false, &nihao),
            None
        );
    }

    #[test]
    fn 组合态点键邮箱网址进串() {
        // 邮箱态：`me@16` 后按 `.` 进串。
        let email = format_state("me@16");
        assert_eq!(
            plan_action(
                WPARAM(VK_OEM_PERIOD.0 as usize),
                LPARAM(0),
                false,
                false,
                &email
            ),
            Some(KeyAction::FormatChar('.'))
        );
        // 网址意图：`www` 后按 `.` 进串。
        let www = format_state("www");
        assert_eq!(
            plan_action(
                WPARAM(VK_OEM_PERIOD.0 as usize),
                LPARAM(0),
                false,
                false,
                &www
            ),
            Some(KeyAction::FormatChar('.'))
        );
        // 普通拼音：`nihao` 后按 `.` 放行宿主（既有行为不回退）。
        let nihao = format_state("nihao");
        assert_eq!(
            plan_action(
                WPARAM(VK_OEM_PERIOD.0 as usize),
                LPARAM(0),
                false,
                false,
                &nihao
            ),
            None
        );
        // 完整邮箱串继续演进：me@163.com 已含点 → 单候选直通。
        let full = format_state("me@163.c");
        assert_eq!(
            plan_action(
                WPARAM(VK_OEM_PERIOD.0 as usize),
                LPARAM(0),
                false,
                false,
                &full
            ),
            Some(KeyAction::FormatChar('.'))
        );
    }
}
