//! TSF COM 服务与最小输入处理器。
//!
//! 本模块负责输入法 DLL 的 COM 生命周期、类工厂、文本服务实例，以及 M1 的
//! 按键 → 组合 → 上屏最小闭环：
//! - `ITfKeyEventSink` 必须经 `ITfKeystrokeMgr::AdviseKeyEventSink` 注册后
//!   才能接收键盘事件并决定是否吃键（经 `ITfThreadMgr::AdviseSink` 注册会失败）；
//! - `ITfEditSession` 在 TSF 编辑会话内写入组合文本或提交文本；
//! - `ITfCompositionSink` 在宿主终止组合时同步清理输入引擎状态。
//!
//! TSF 注册表写入与清理由 `scripts/` 下的安装/卸载脚本完成，本模块不直接改注册表。

use std::ffi::c_void;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::ptr;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;

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
    GetKeyState, VIRTUAL_KEY, VK_1, VK_9, VK_A, VK_BACK, VK_CONTROL, VK_ESCAPE, VK_MENU,
    VK_OEM_COMMA, VK_OEM_PERIOD, VK_RETURN, VK_SHIFT, VK_SPACE, VK_TAB, VK_Z,
};
use windows::Win32::UI::TextServices::{
    ITfComposition, ITfCompositionSink, ITfCompositionSink_Impl, ITfContext, ITfContextComposition,
    ITfEditSession, ITfEditSession_Impl, ITfInsertAtSelection, ITfKeyEventSink,
    ITfKeyEventSink_Impl, ITfKeystrokeMgr, ITfTextInputProcessorEx, ITfTextInputProcessorEx_Impl,
    ITfTextInputProcessor_Impl, ITfThreadMgr, TF_ES_READWRITE, TF_ES_SYNC, TF_IAS_QUERYONLY,
};
use windows_core::{
    implement, IUnknown, IUnknownImpl, Interface, Ref, Result, BOOL, HRESULT, PCWSTR,
};
use zhu_ye_core::{core_version, BigramModel, Dictionary, DictionaryFile, UserDictStore};

use crate::candidate_window::{CandidateWindow, CandidateWindowPlacement};
use crate::input::{m1_seed_dictionary, InputEngine, InputMode};

/// 输入法 TIP 的 CLSID，与 `scripts/ime-identity.ps1` 中的 `TipClsid` 保持一致。
pub const CLSID_ZHU_YE_TIP: windows::core::GUID =
    windows::core::GUID::from_u128(0xE54D6682_8650_40E7_A9EE_6FD1137849AE);

/// 安装/便携包随带的 v2 词典文件名，与 `scripts/ime-identity.ps1` 保持一致。
const DICTIONARY_FILE_NAME: &str = "dictionary.zyct";

/// 简体中文（zh-CN，LCID 0x0804）下的语言配置文件 GUID，
/// 与 `scripts/ime-identity.ps1` 中的 `ProfileGuid` 保持一致。
pub const PROFILE_GUID_ZHU_YE: windows::core::GUID =
    windows::core::GUID::from_u128(0x6315FE74_92C3_439B_8CDF_FDB6E43EDAF1);

/// 当前由 DLL 创建且尚未释放的 COM 对象数（含类工厂与文本服务）。
static ACTIVE_OBJECTS: AtomicUsize = AtomicUsize::new(0);

/// `IClassFactory::LockServer` 锁定的层数。
static SERVER_LOCKS: AtomicUsize = AtomicUsize::new(0);

/// TSF 键盘事件对应的输入动作。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyAction {
    /// 小写英文字母进入组合。
    Letter(char),
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
}

impl KeyAction {
    /// 是否需要 TSF 编辑会话；纯状态动作（Shift/Tab/翻页）在会话外执行。
    fn needs_edit_session(self) -> bool {
        matches!(
            self,
            KeyAction::Letter(_)
                | KeyAction::Backspace
                | KeyAction::Space
                | KeyAction::Enter
                | KeyAction::Escape
                | KeyAction::Select(_)
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
}

impl EngineState {
    fn new() -> Self {
        Self {
            engine: create_engine(None),
            tid: 0,
            keystroke_mgr: None,
            composition: None,
            candidate_window: CandidateWindow::new(),
        }
    }

    fn with_user_store(store: UserDictStore) -> Self {
        Self {
            engine: create_engine(Some(store)),
            tid: 0,
            keystroke_mgr: None,
            composition: None,
            candidate_window: CandidateWindow::new(),
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
    state: Rc<Mutex<EngineState>>,
}

impl TextService {
    fn state(&self) -> &Rc<Mutex<EngineState>> {
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
            debug_log("zhu-ye: Activate tm=none");
            return Ok(());
        };

        // `ITfKeyEventSink` 只能经 `ITfKeystrokeMgr::AdviseKeyEventSink` 注册：
        // 通过 `ITfThreadMgr::AdviseSink` 注册会返回 CONNECT_E_CANNOTCONNECT
        // (0x80040202)，按下按键永远不会送达文本服务。
        let keystroke_mgr = match unsafe { get_keystroke_mgr(&thread_mgr) } {
            Ok(km) => km,
            Err(err) => {
                debug_log(&format!("zhu-ye: Activate keystroke-mgr FAILED {err:?}"));
                return Err(err);
            }
        };
        let key_sink = self.to_object().to_interface::<ITfKeyEventSink>();
        match unsafe { keystroke_mgr.AdviseKeyEventSink(tid, &key_sink, true) } {
            Ok(()) => {
                let mut state = state_lock(self);
                state.tid = tid;
                state.keystroke_mgr = Some(keystroke_mgr);
                debug_log(&format!("zhu-ye: Activate tid={tid} key-sink ok"));
                Ok(())
            }
            Err(err) => {
                debug_log(&format!("zhu-ye: Activate key-sink FAILED {err:?}"));
                Err(err)
            }
        }
    }

    fn Deactivate(&self) -> Result<()> {
        debug_log("zhu-ye: Deactivate");
        let mut state = state_lock(self);
        state.composition = None;
        state.engine.cancel_input();
        state.candidate_window.hide();

        if let Some(keystroke_mgr) = state.keystroke_mgr.take() {
            let _ = unsafe { keystroke_mgr.UnadviseKeyEventSink(state.tid) };
        }
        Ok(())
    }
}

impl ITfTextInputProcessorEx_Impl for TextService_Impl {
    fn ActivateEx(&self, ptim: Ref<'_, ITfThreadMgr>, tid: u32, dwflags: u32) -> Result<()> {
        debug_log(&format!("zhu-ye: ActivateEx flags=0x{dwflags:X} tid={tid}"));
        ITfTextInputProcessor_Impl::Activate(self, ptim, tid)
    }
}

impl ITfKeyEventSink_Impl for TextService_Impl {
    fn OnSetFocus(&self, fforeground: BOOL) -> Result<()> {
        debug_log(&format!("zhu-ye: OnSetFocus fg={}", fforeground.0));
        Ok(())
    }

    fn OnTestKeyDown(
        &self,
        _pic: Ref<'_, ITfContext>,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> Result<BOOL> {
        let action = plan_action(wparam, lparam, key_modifiers_down(), self.state());
        debug_log(&format!(
            "zhu-ye: TestKeyDown 0x{:X} -> {:?}",
            wparam.0, action
        ));
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
        let Some(action) = plan_action(wparam, lparam, key_modifiers_down(), self.state()) else {
            return Ok(BOOL(0));
        };

        // 只改引擎/候选窗状态、不写文档文本的动作不需要编辑会话。
        if !action.needs_edit_session() {
            sync_engine(self.state(), action);
            refresh_candidate_window(self.state(), None);
            debug_log(&format!(
                "zhu-ye: key 0x{:X} state action {:?}",
                wparam.0, action
            ));
            return Ok(BOOL(1));
        }

        let Some(context) = pic.cloned() else {
            return Ok(BOOL(0));
        };

        debug_log(&format!("zhu-ye: key 0x{:X} action {:?}", wparam.0, action));

        let tid = state_lock(self).tid;
        let sink = self.to_object().to_interface::<ITfCompositionSink>();
        let state = Rc::clone(self.state());
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
fn classify_key(wparam: WPARAM, lparam: LPARAM) -> Option<KeyAction> {
    let code = VIRTUAL_KEY(wparam.0 as u16).0;
    match code {
        code if (VK_A.0..=VK_Z.0).contains(&code) => {
            let letter = u16::from(b'a') + (code - VK_A.0);
            Some(KeyAction::Letter(
                char::from_u32(u32::from(letter)).unwrap(),
            ))
        }
        code if code == VK_BACK.0 => Some(KeyAction::Backspace),
        code if code == VK_SPACE.0 => Some(KeyAction::Space),
        code if code == VK_RETURN.0 => Some(KeyAction::Enter),
        code if code == VK_ESCAPE.0 => Some(KeyAction::Escape),
        code if (VK_1.0..=VK_9.0).contains(&code) => {
            Some(KeyAction::Select(usize::from(code - VK_1.0)))
        }
        code if code == VK_SHIFT.0 && !is_repeat(lparam) => Some(KeyAction::ToggleMode),
        code if code == VK_TAB.0 => Some(KeyAction::ToggleLayer),
        code if code == VK_OEM_COMMA.0 => Some(KeyAction::PageUp),
        code if code == VK_OEM_PERIOD.0 => Some(KeyAction::PageDown),
        _ => None,
    }
}

/// 判断 Ctrl 或 Alt 修饰键是否按下；按住时系统组合键（复制/粘贴/保存等）
/// 必须放行给宿主应用，避免输入法吞掉 Ctrl+A/C/S/V 等快捷键。
fn key_modifiers_down() -> bool {
    // GetKeyState 返回 SHORT，高位为 1 表示按下，即 i16 值为负。
    unsafe { GetKeyState(i32::from(VK_CONTROL.0)) < 0 || GetKeyState(i32::from(VK_MENU.0)) < 0 }
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
fn plan_action(
    wparam: WPARAM,
    lparam: LPARAM,
    modifier_held: bool,
    state: &Rc<Mutex<EngineState>>,
) -> Option<KeyAction> {
    if modifier_held {
        return None;
    }
    let action = classify_key(wparam, lparam)?;
    let engine = &state.lock().unwrap().engine;
    match action {
        KeyAction::Letter(_) => (engine.mode() == InputMode::Chinese).then_some(action),
        KeyAction::ToggleMode => Some(action),
        _ if engine.is_active() => Some(action),
        _ => None,
    }
}

/// 应用一次输入动作：先写 TSF 组合/提交文本，再同步引擎状态。
fn apply_action(
    state: &Rc<Mutex<EngineState>>,
    context: &ITfContext,
    sink: &ITfCompositionSink,
    ec: u32,
    action: KeyAction,
) -> Result<()> {
    match action {
        KeyAction::Letter(_) | KeyAction::Backspace => {
            let text = compose_text(state, action);
            update_composition(state, context, sink, ec, &text)?;
        }
        KeyAction::Space | KeyAction::Enter | KeyAction::Escape | KeyAction::Select(_) => {
            let text = commit_text(state, action);
            finish_composition(state, context, ec, &text)?;
        }
        KeyAction::ToggleMode
        | KeyAction::ToggleLayer
        | KeyAction::PageUp
        | KeyAction::PageDown => {
            sync_engine(state, action);
            refresh_candidate_window(state, Some((context, ec)));
            return Ok(());
        }
    }
    sync_engine(state, action);
    refresh_candidate_window(state, Some((context, ec)));
    Ok(())
}

/// 计算下一次组合串文本；不修改引擎，真实状态在 TSF 写入完成后同步。
fn compose_text(state: &Rc<Mutex<EngineState>>, action: KeyAction) -> String {
    let text = {
        let engine = &mut state.lock().unwrap().engine;
        match action {
            KeyAction::Letter(c) => format!("{}{}", engine.composing(), c),
            KeyAction::Backspace => engine.preview_after_backspace().unwrap_or_default(),
            _ => String::new(),
        }
    };
    debug_log(&format!("zhu-ye: compose-text {action:?} -> {text:?}"));
    text
}

/// 计算本次提交文本；清空引擎状态交给 TSF 写入完成后的 `sync_engine`。
fn commit_text(state: &Rc<Mutex<EngineState>>, action: KeyAction) -> String {
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
    debug_log(&format!(
        "zhu-ye: commit-text {action:?} -> {text:?} ({} utf8)",
        text.len()
    ));
    text
}

/// 将引擎状态推进到动作后的实际状态。
fn sync_engine(state: &Rc<Mutex<EngineState>>, action: KeyAction) {
    let engine = &mut state.lock().unwrap().engine;
    match action {
        KeyAction::Letter(c) => {
            let _ = engine.handle_letter(c);
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
            engine.toggle_mode();
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
    }
}

/// 用引擎最新状态刷新候选窗。`edit` 提供编辑会话内的上下文以计算组合区坐标。
fn refresh_candidate_window(state: &Rc<Mutex<EngineState>>, edit: Option<(&ITfContext, u32)>) {
    let view = state.lock().unwrap().engine.candidate_ui_view();
    if view.visible_items().is_empty() {
        debug_log("zhu-ye: cand-hide (no items)");
        state.lock().unwrap().candidate_window.hide();
        return;
    }
    let placement = edit.and_then(|(context, ec)| composition_placement(state, context, ec));
    debug_log(&format!(
        "zhu-ye: cand-show items={} first={:?}",
        view.visible_items().len(),
        view.visible_items().first()
    ));
    state
        .lock()
        .unwrap()
        .candidate_window
        .update(view, placement);
}

/// 编辑会话内取组合范围在屏幕上的底部坐标，用于候选窗定位。
fn composition_placement(
    state: &Rc<Mutex<EngineState>>,
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

/// 更新组合文本：已有组合直接替换，否则插入文本并启动新组合。
fn update_composition(
    state: &Rc<Mutex<EngineState>>,
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
            debug_log(&format!("zhu-ye: comp-update {text:?}"));
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
    state: &Rc<Mutex<EngineState>>,
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
                debug_log(&format!("zhu-ye: commit {text:?} ({} utf8)", text.len()));
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

/// 输出调试日志：先写入调试器输出通道，再在“验收期文件日志”开关
/// （`C:\zhu-ye-test\tsf-debug.enable` 存在）时追加写
/// `C:\zhu-ye-test\tsf-debug.log`，便于在没有调试器的远程虚拟机上观察
/// TSF 生命周期与按键流程。验收结束后移除文件日志部分。
fn debug_log(message: &str) {
    let wide = to_wide(message);
    unsafe { OutputDebugStringW(PCWSTR(wide.as_ptr())) };
    if !file_log_enabled() {
        return;
    }
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

/// 验收期文件日志路径与开关缓存。
const FILE_LOG_PATH: &str = r"C:\zhu-ye-test\tsf-debug.log";
static FILE_LOG_ENABLED: AtomicBool = AtomicBool::new(false);
static FILE_LOG_CHECKED: AtomicBool = AtomicBool::new(false);

fn file_log_enabled() -> bool {
    if !FILE_LOG_CHECKED.load(Ordering::Relaxed) {
        FILE_LOG_ENABLED.store(
            PathBuf::from(r"C:\zhu-ye-test\tsf-debug.enable").exists(),
            Ordering::Relaxed,
        );
        FILE_LOG_CHECKED.store(true, Ordering::Relaxed);
    }
    FILE_LOG_ENABLED.load(Ordering::Relaxed)
}

/// 创建输入引擎：优先加载 `%APPDATA%\ai-zhu-ye-ime\seed.zyct`，
/// 缺失或损坏时回退 M1 内置演示词典，保证输入法始终可启动。
fn create_engine(user_store: Option<UserDictStore>) -> InputEngine {
    let dict_path = dictionary_path();
    match DictionaryFile::open(&dict_path) {
        Ok(file) => {
            debug_log(&format!("zhu-ye: dict-ok path={dict_path:?}"));
            let dictionary: Arc<dyn Dictionary> = Arc::new(file.clone());
            let bigram: Arc<dyn BigramModel> = Arc::new(file);
            match user_store {
                Some(store) => InputEngine::with_user_store_and_bigram(dictionary, store, bigram),
                None => InputEngine::with_bigram(dictionary, bigram),
            }
        }
        Err(_) => {
            debug_log(&format!("zhu-ye: dict-fallback path={dict_path:?}"));
            match user_store {
                Some(store) => InputEngine::with_user_store(m1_seed_dictionary(), store),
                None => InputEngine::with_m1_seed(),
            }
        }
    }
}

/// 词典运行时路径解析顺序：显式环境变量 > DLL 同目录（安装器写入）
/// > 用户数据目录 > 工作目录回退。缺失或损坏时由 `create_engine` 回退内置演示词典。
fn dictionary_path() -> PathBuf {
    if let Some(override_path) = std::env::var_os("ZHU_YE_DICT_PATH") {
        return PathBuf::from(override_path);
    }
    resolve_dictionary_path(installed_dictionary_path(), appdata_dictionary_path())
}

/// 纯路径取舍，便于单测；调用方传入已经确认存在的候选路径。
fn resolve_dictionary_path(installed: Option<PathBuf>, appdata: Option<PathBuf>) -> PathBuf {
    installed
        .or(appdata)
        .unwrap_or_else(|| PathBuf::from(DICTIONARY_FILE_NAME))
}

/// 本 DLL 内的锚点函数：取其地址经 `GetModuleHandleExW(FROM_ADDRESS)`
/// 反查 DLL 所在目录，与 DLL 文件名解耦（安装脚本用版本化文件名
/// `zhu-ye-ime-vN.dll` 规避“被 explorer 锁定”问题，不能按名查找）。
fn dictionary_module_anchor() {}

/// 返回 DLL 同目录存在的 `dictionary.zyct`；便于安装器做机器级部署。
fn installed_dictionary_path() -> Option<PathBuf> {
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
        let candidate = directory.join(DICTIONARY_FILE_NAME);
        candidate.exists().then_some(candidate)
    }
}

/// 用户数据目录里的词典，保留旧版手动放词典的开发流程。
fn appdata_dictionary_path() -> Option<PathBuf> {
    let candidate = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .map(|root| root.join("ai-zhu-ye-ime").join(DICTIONARY_FILE_NAME))?;
    candidate.exists().then_some(candidate)
}

/// 用户词库 JSON 路径：`%APPDATA%\ai-zhu-ye-ime\user_words.json`。
fn user_words_path() -> PathBuf {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .map(|root| root.join("ai-zhu-ye-ime").join("user_words.json"))
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
        state: Rc::new(Mutex::new(match user_store {
            Some(store) => EngineState::with_user_store(store),
            None => EngineState::new(),
        })),
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

    let factory: IUnknown =
        create_class_factory(Some(UserDictStore::new(user_words_path()))).into();
    unsafe { factory.query(riid, ppv) }
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
    use windows::Win32::UI::Input::KeyboardAndMouse::VK_0;
    use windows::Win32::UI::TextServices::{ITfTextInputProcessor, ITfThreadMgr};

    /// 生命周期计数是全局状态；测试并行运行时互斥，避免相互干扰。
    static TEST_LOCK: Mutex<()> = Mutex::new(());

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
            classify_key(WPARAM(VK_A.0 as usize), LPARAM(0)),
            Some(KeyAction::Letter('a'))
        );
        assert_eq!(
            classify_key(WPARAM(VK_Z.0 as usize), LPARAM(0)),
            Some(KeyAction::Letter('z'))
        );
        assert_eq!(
            classify_key(WPARAM(VK_BACK.0 as usize), LPARAM(0)),
            Some(KeyAction::Backspace)
        );
        assert_eq!(
            classify_key(WPARAM(VK_SPACE.0 as usize), LPARAM(0)),
            Some(KeyAction::Space)
        );
        assert_eq!(
            classify_key(WPARAM(VK_RETURN.0 as usize), LPARAM(0)),
            Some(KeyAction::Enter)
        );
        assert_eq!(
            classify_key(WPARAM(VK_ESCAPE.0 as usize), LPARAM(0)),
            Some(KeyAction::Escape)
        );
        assert_eq!(
            classify_key(WPARAM(VK_1.0 as usize), LPARAM(0)),
            Some(KeyAction::Select(0))
        );
        assert_eq!(
            classify_key(WPARAM(VK_9.0 as usize), LPARAM(0)),
            Some(KeyAction::Select(8))
        );
        assert_eq!(classify_key(WPARAM(VK_0.0 as usize), LPARAM(0)), None);
        assert_eq!(classify_key(WPARAM(0x00A0), LPARAM(0)), None);
    }

    #[test]
    fn 未激活组合时功能键放行字母进入引擎() {
        let state = Rc::new(Mutex::new(EngineState::new()));
        assert_eq!(
            plan_action(WPARAM(VK_A.0 as usize), LPARAM(0), false, &state),
            Some(KeyAction::Letter('a'))
        );
        assert_eq!(
            plan_action(WPARAM(VK_SPACE.0 as usize), LPARAM(0), false, &state),
            None
        );
        assert_eq!(
            plan_action(WPARAM(VK_RETURN.0 as usize), LPARAM(0), false, &state),
            None
        );
        assert_eq!(
            plan_action(WPARAM(VK_ESCAPE.0 as usize), LPARAM(0), false, &state),
            None
        );

        state.lock().unwrap().engine.handle_letter('a');
        assert_eq!(
            plan_action(WPARAM(VK_SPACE.0 as usize), LPARAM(0), false, &state),
            Some(KeyAction::Space)
        );
        assert_eq!(
            plan_action(WPARAM(VK_1.0 as usize), LPARAM(0), false, &state),
            Some(KeyAction::Select(0))
        );
    }

    #[test]
    fn ctrl或alt修饰键一律放行给宿主() {
        let state = Rc::new(Mutex::new(EngineState::new()));
        assert_eq!(
            plan_action(WPARAM(VK_A.0 as usize), LPARAM(0), true, &state),
            None
        );
        assert_eq!(
            plan_action(WPARAM(VK_SPACE.0 as usize), LPARAM(0), true, &state),
            None
        );

        // 组合进行中按 Ctrl+S 等系统组合键也不被输入法吞掉。
        state.lock().unwrap().engine.handle_letter('n');
        assert_eq!(plan_action(WPARAM(0x53), LPARAM(0), true, &state), None);
        // 无修饰键时字母仍正常进入组合。
        assert_eq!(
            plan_action(WPARAM(0x53), LPARAM(0), false, &state),
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
    fn 词典路径优先安装目录并回退默认文件名() {
        let installed = Some(PathBuf::from(
            "C:\\Program Files\\ai-zhu-ye-ime\\tsf\\dictionary.zyct",
        ));
        let appdata = Some(PathBuf::from("%APPDATA%\\ai-zhu-ye-ime\\dictionary.zyct"));
        assert_eq!(
            resolve_dictionary_path(installed.clone(), None).as_path(),
            installed.as_deref().unwrap()
        );
        assert_eq!(
            resolve_dictionary_path(None, appdata.clone()).as_path(),
            appdata.as_deref().unwrap()
        );
        assert_eq!(
            resolve_dictionary_path(None, None),
            PathBuf::from(DICTIONARY_FILE_NAME)
        );
    }

    #[test]
    fn 引擎推进与组合文本预览一致() {
        let state = Rc::new(Mutex::new(EngineState::new()));
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
            classify_key(WPARAM(VK_SHIFT.0 as usize), LPARAM(0)),
            Some(KeyAction::ToggleMode)
        );
        assert_eq!(
            classify_key(WPARAM(VK_TAB.0 as usize), LPARAM(0)),
            Some(KeyAction::ToggleLayer)
        );
        assert_eq!(
            classify_key(WPARAM(VK_OEM_COMMA.0 as usize), LPARAM(0)),
            Some(KeyAction::PageUp)
        );
        assert_eq!(
            classify_key(WPARAM(VK_OEM_PERIOD.0 as usize), LPARAM(0)),
            Some(KeyAction::PageDown)
        );
    }

    #[test]
    fn shift长按重复事件不重复切换() {
        assert_eq!(
            classify_key(WPARAM(VK_SHIFT.0 as usize), LPARAM(0)),
            Some(KeyAction::ToggleMode)
        );
        assert_eq!(
            classify_key(WPARAM(VK_SHIFT.0 as usize), LPARAM(0x4000_0000)),
            None
        );
        assert!(is_repeat(LPARAM(0x4000_0000)));
        assert!(!is_repeat(LPARAM(0)));
    }

    #[test]
    fn shift与组合内功能键放行策略正确() {
        let state = Rc::new(Mutex::new(EngineState::new()));
        assert_eq!(
            plan_action(WPARAM(VK_SHIFT.0 as usize), LPARAM(0), false, &state),
            Some(KeyAction::ToggleMode)
        );
        assert_eq!(
            plan_action(WPARAM(VK_TAB.0 as usize), LPARAM(0), false, &state),
            None
        );

        state.lock().unwrap().engine.handle_letter('n');
        assert_eq!(
            plan_action(WPARAM(VK_TAB.0 as usize), LPARAM(0), false, &state),
            Some(KeyAction::ToggleLayer)
        );
        assert_eq!(
            plan_action(WPARAM(VK_OEM_PERIOD.0 as usize), LPARAM(0), false, &state),
            Some(KeyAction::PageDown)
        );
    }

    #[test]
    fn 状态动作推进图层与页码() {
        let state = Rc::new(Mutex::new(EngineState::new()));
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
        let state = Rc::new(Mutex::new(EngineState::new()));
        sync_engine(&state, KeyAction::Letter('n'));
        sync_engine(&state, KeyAction::ToggleMode);
        assert_eq!(state.lock().unwrap().engine.mode(), InputMode::English);
        assert_eq!(
            plan_action(WPARAM(VK_TAB.0 as usize), LPARAM(0), false, &state),
            None
        );
        assert_eq!(
            plan_action(WPARAM(VK_OEM_PERIOD.0 as usize), LPARAM(0), false, &state),
            None
        );
    }
}
