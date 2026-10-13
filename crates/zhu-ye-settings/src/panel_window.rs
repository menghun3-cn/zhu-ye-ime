//! 工具箱面板：非激活浮层窗口。
//!
//! 面板用 `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST` 浮在最上层，并且
//! **常态永不夺取焦点**（`WM_MOUSEACTIVATE` 回 `MA_NOACTIVATE`）。因此用户的目标应用在整个
//! 浏览过程中保持前台，点击格子时 `deliver` 送出的字符才落得到那里——这是"选中即上屏"
//! 的实现前提。
//!
//! 第十四期（FR-070，T-151）交互重构：
//! - 顶部 **emoji / 符号双页签**（点击或 Tab 切换，`layout.tabs`）；
//! - **搜索框**：点击进入搜索态 = 临时激活的窄化例外（S-32）——进入前记录前台窗口为
//!   目标，`WM_CHAR` 输入即按拼音别名/字符/组名子串过滤（`panel::filter_entries`），
//!   Esc 退出搜索并还焦目标；搜索态选中 → 还焦后 `deliver` 上屏；
//! - 呼出热键（托盘 `RegisterHotKey`）**toggle** 关闭（S-31），补上非激活窗口收不到
//!   键盘的关闭位；面板窗口类全局唯一，`FindWindow` 互斥。
//! - 面板位置记忆（S-33）：关闭时把窗口矩形写回 `panel.json`，下次呼出原位弹出；
//!   首次无记忆则贴鼠标光标。

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, EndPaint, InvalidateRect, ScreenToClient, UpdateWindow, DT_CENTER,
    DT_END_ELLIPSIS, DT_LEFT, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, HBRUSH, HDC, SRCCOPY,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::WM_MOUSELEAVE;
use windows::Win32::UI::HiDpi::AdjustWindowRectExForDpi;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT, VK_BACK, VK_ESCAPE, VK_LEFT, VK_RIGHT, VK_TAB,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetClientRect, GetCursorPos,
    GetForegroundWindow, GetMessageW, GetWindowLongPtrW, GetWindowRect, LoadCursorW, PostMessageW,
    PostQuitMessage, RegisterClassW, SetForegroundWindow, SetWindowLongPtrW, SetWindowPos,
    ShowWindow, SystemParametersInfoW, TranslateMessage, CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW,
    GWLP_USERDATA, GWL_EXSTYLE, HWND_TOPMOST, IDC_ARROW, MA_ACTIVATE, MA_NOACTIVATE,
    SPI_GETWORKAREA, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
    SWP_SHOWWINDOW, SW_SHOWNOACTIVATE, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, WINDOW_EX_STYLE,
    WM_APP, WM_CHAR, WM_CLOSE, WM_DESTROY, WM_ERASEBKGND, WM_KEYDOWN, WM_LBUTTONDOWN,
    WM_MOUSEACTIVATE, WM_MOUSEMOVE, WM_NCCREATE, WM_PAINT, WM_SETTINGCHANGE, WM_THEMECHANGED,
    WNDCLASSW, WS_EX_NOACTIVATE, WS_POPUP,
};

use zhu_ye_ui::{UiRect, UiThemeKind, BASE_DPI};

use crate::deliver::{deliver_text, Delivery};
use crate::gdi::{draw_text, fill, fill_round, BackBuffer, Fonts};
use crate::layout::{self, PanelMetrics};
use crate::panel::{filter_entries, page_of, PanelEntry, PanelKind, PanelView, PANEL_PAGE_SIZE};
use crate::shell;
use crate::theme::{settings_theme, settings_theme_from_system_colors, SettingsTheme};
use crate::wide::to_utf16;

/// 面板窗口类名。
pub const PANEL_CLASS_NAME: &str = "ZhuYeSettingsPanel";

/// 面板关闭时通知设置窗口的自定义消息。
///
/// 面板不激活，因此无法靠"焦点回到设置窗口"来判断关闭时机；关闭动作显式通知
/// （托盘回调 `WM_HOTKEY` toggle 关闭时同样收不到——只有宿主窗口才需要通知）。
pub const WM_PANEL_CLOSED: u32 = WM_APP + 1;

/// 面板窗口状态文件：关闭时记忆位置与页签（S-33；托盘/设置窗口共用，S-8 原子写）。
pub const PANEL_POSITION_FILE: &str = "panel.json";

/// 已打开的面板窗口；`Drop` 销毁窗口。
pub struct PanelWindow {
    hwnd: HWND,
}

impl PanelWindow {
    /// 创建并显示面板。不启动消息循环：面板与宿主（设置窗口或托盘）同线程，
    /// 共用宿主的消息循环。
    ///
    /// `host`：面板关闭时接收 `WM_PANEL_CLOSED` 通知的窗口；托盘宿主传 `None`。
    /// 位置：优先读 `panel.json` 记忆（S-33），无记忆首次贴鼠标光标。
    ///
    /// # Errors
    /// 注册窗口类或创建窗口失败时返回描述。
    pub fn open(view: PanelView, host: Option<HWND>) -> Result<Self, String> {
        unsafe {
            let module = GetModuleHandleW(PCWSTR::null())
                .map_err(|error| format!("获取模块句柄失败: {error}"))?;
            let instance = HINSTANCE(module.0);
            register_panel_class(instance)?;

            let dpi = shell::system_dpi().max(BASE_DPI);
            let metrics = PanelMetrics::new(dpi);
            let (client_width, client_height) = metrics.desired_client_size();
            let mut frame = RECT {
                left: 0,
                top: 0,
                right: client_width,
                bottom: client_height,
            };
            let _ = AdjustWindowRectExForDpi(&mut frame, WS_POPUP, false, WINDOW_EX_STYLE(0), dpi);

            let state = Box::new(PanelState::new(
                view,
                host.unwrap_or_default(),
                None,
                String::new(),
                false,
            ));
            let state_ptr = Box::into_raw(state);
            let class = to_utf16(PANEL_CLASS_NAME);
            let title = to_utf16(PANEL_CLASS_NAME);
            let hwnd = match CreateWindowExW(
                // 不激活 + 工具窗口（不进任务栏与 Alt+Tab）+ 置顶。
                WINDOW_EX_STYLE(0x0800_0000 | 0x0000_0080 | 0x0000_0008),
                PCWSTR(class.as_ptr()),
                PCWSTR(title.as_ptr()),
                WS_POPUP,
                0,
                0,
                frame.right - frame.left,
                frame.bottom - frame.top,
                None,
                None,
                Some(instance),
                Some(state_ptr.cast()),
            ) {
                Ok(hwnd) => hwnd,
                Err(error) => {
                    drop(Box::from_raw(state_ptr));
                    return Err(format!("创建工具箱面板失败: {error}"));
                }
            };
            position_panel(
                hwnd,
                frame.right - frame.left,
                frame.bottom - frame.top,
                true,
            );
            let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            let _ = UpdateWindow(hwnd);
            Ok(Self { hwnd })
        }
    }

    /// 面板窗口句柄。
    #[must_use]
    pub const fn hwnd(&self) -> HWND {
        self.hwnd
    }
}

impl Drop for PanelWindow {
    fn drop(&mut self) {
        unsafe {
            if !self.hwnd.is_invalid() {
                let _ = DestroyWindow(self.hwnd);
            }
        }
        self.hwnd = HWND::default();
    }
}

/// 截图模式：建立面板、画出一帧、写出 BMP 后退出。
///
/// 供验收取证，与设置窗口的 `--shot` 同路数。`query` 预置搜索查询（`--panel-query`），
/// 渲染过滤后的网格帧（搜索态取证）。截图用固定居中位置，不走位置记忆，保证帧可比。
///
/// # Errors
/// 建窗、绘制或写文件失败时返回描述。
pub fn run_shot(view: PanelView, query: &str, path: &Path) -> Result<(), String> {
    let path = path.to_path_buf();
    let query = query.to_owned();
    unsafe {
        // 与设置窗口同口径：不启用 Per-Monitor V2 时 `GetDpiForSystem` 返回 96，
        // 截图就不是真实缩放下的样子。
        shell::enable_per_monitor_v2();
        let module = GetModuleHandleW(PCWSTR::null())
            .map_err(|error| format!("获取模块句柄失败: {error}"))?;
        let instance = HINSTANCE(module.0);
        register_panel_class(instance)?;
        let dpi = shell::system_dpi().max(BASE_DPI);
        let metrics = PanelMetrics::new(dpi);
        let (client_width, client_height) = metrics.desired_client_size();
        let mut frame = RECT {
            left: 0,
            top: 0,
            right: client_width,
            bottom: client_height,
        };
        let _ = AdjustWindowRectExForDpi(&mut frame, WS_POPUP, false, WINDOW_EX_STYLE(0), dpi);
        let state = Box::new(PanelState::new(
            view,
            HWND::default(),
            Some(path),
            query,
            true,
        ));
        let state_ptr = Box::into_raw(state);
        let class = to_utf16(PANEL_CLASS_NAME);
        let hwnd = match CreateWindowExW(
            WINDOW_EX_STYLE(0x0800_0000 | 0x0000_0080 | 0x0000_0008),
            PCWSTR(class.as_ptr()),
            PCWSTR(class.as_ptr()),
            WS_POPUP,
            0,
            0,
            frame.right - frame.left,
            frame.bottom - frame.top,
            None,
            None,
            Some(instance),
            Some(state_ptr.cast()),
        ) {
            Ok(hwnd) => hwnd,
            Err(error) => {
                drop(Box::from_raw(state_ptr));
                return Err(format!("创建工具箱面板失败: {error}"));
            }
        };
        position_panel(
            hwnd,
            frame.right - frame.left,
            frame.bottom - frame.top,
            false,
        );
        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        let _ = UpdateWindow(hwnd);
        let mut message = windows::Win32::UI::WindowsAndMessaging::MSG::default();
        while GetMessageW(&mut message, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
        Ok(())
    }
}

/// 工作区矩形（避开任务栏）；失败返回 `None`（调用方回退默认位置）。
unsafe fn work_area() -> Option<RECT> {
    let mut work = RECT::default();
    unsafe {
        SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            Some((&mut work as *mut RECT).cast()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS::default(),
        )
        .ok()?;
    }
    Some(work)
}

/// 把面板放到目标位置（S-33）：有记忆读出并钳制到工作区，无记忆贴鼠标光标右下，
/// 无论如何都保证完整可见；截图模式固定居中。
unsafe fn position_panel(hwnd: HWND, width: i32, height: i32, use_memory: bool) {
    unsafe {
        let work = work_area();
        let fallback = |default: (i32, i32)| -> (i32, i32) {
            match work {
                Some(rect) if width < rect.right - rect.left && height < rect.bottom - rect.top => {
                    let left = default.0.clamp(rect.left, rect.right - width);
                    let top = default.1.clamp(rect.top, rect.bottom - height);
                    (left, top)
                }
                _ => (80, 80),
            }
        };
        let (left, top) = if use_memory {
            match PanelPosition::load() {
                Some(position) => fallback((position.left, position.top)),
                // 首次呼出：贴鼠标光标右下（偏移 12px，光标不再被面板盖住）。
                None => {
                    let mut point = POINT::default();
                    if GetCursorPos(&mut point).is_ok() {
                        fallback((point.x + 12, point.y + 12))
                    } else {
                        fallback((80, 80))
                    }
                }
            }
        } else {
            match work {
                Some(rect) if width < rect.right - rect.left && height < rect.bottom - rect.top => {
                    (
                        rect.left + (rect.right - rect.left - width) / 2,
                        rect.top + (rect.bottom - rect.top - height) / 2,
                    )
                }
                _ => (80, 80),
            }
        };
        let _ = SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            left,
            top,
            width,
            height,
            SWP_NOACTIVATE | SWP_SHOWWINDOW,
        );
    }
}

/// 面板窗口状态。
struct PanelState {
    view: PanelView,
    theme_kind: UiThemeKind,
    dpi: u32,
    /// 关闭时通知的宿主窗口（设置窗口）；托盘宿主传 `None`（无需通知）。
    host_hwnd: HWND,
    hint: Option<String>,
    shot_path: Option<PathBuf>,
    shot_done: bool,
    back: BackBuffer,
    fonts: Fonts,
    /// 鼠标悬停的格子下标（T-148 选中格视觉；点击即上屏，无驻留选中态）。
    hover: Option<usize>,
    /// 搜索查询（T-151，S-32）：空 = 全量浏览；非空 = `filter_entries` 过滤结果。
    query: String,
    /// 是否处于搜索态（面板已临时激活以接收键盘）。
    search_active: bool,
    /// 进入搜索态前的前台窗口——搜索态还焦与选中上屏的目标（S-32）。
    target_hwnd: HWND,
    /// 独立消息循环场景（截图模式）在面板关闭后 `PostQuitMessage`。
    quit_on_close: bool,
}

impl PanelState {
    fn new(
        view: PanelView,
        host_hwnd: HWND,
        shot_path: Option<PathBuf>,
        query: String,
        quit_on_close: bool,
    ) -> Self {
        Self {
            view,
            theme_kind: shell::resolve_theme_kind(),
            dpi: shell::system_dpi().max(BASE_DPI),
            host_hwnd,
            hint: None,
            shot_path,
            shot_done: false,
            back: BackBuffer::default(),
            fonts: Fonts::default(),
            hover: None,
            query,
            search_active: false,
            target_hwnd: HWND::default(),
            quit_on_close,
        }
    }

    fn theme(&self) -> SettingsTheme {
        if self.theme_kind == UiThemeKind::HighContrast {
            settings_theme_from_system_colors(shell::system_colors())
        } else {
            settings_theme(self.theme_kind)
        }
    }
}

/// 面板状态文件的 JSON 载荷（S-33：位置 + 页签记忆）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct PanelPosition {
    kind: PanelKind,
    left: i32,
    top: i32,
    width: i32,
    height: i32,
}

impl PanelPosition {
    /// 读取记忆位置；无文件/解析失败为 `None`（首次呼出贴光标）。
    fn load() -> Option<Self> {
        let path = crate::config::panel_state_path()?;
        let text = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&text).ok()
    }

    /// 原子写（临时文件 + rename，S-8 口径）：失败静默（记忆是增强项，不打断使用）。
    fn save(&self) {
        let Some(path) = crate::config::panel_state_path() else {
            return;
        };
        let mut tmp = path.clone();
        tmp.set_extension("json.tmp");
        if let Ok(text) = serde_json::to_string(self) {
            if std::fs::write(&tmp, text).is_ok() {
                let _ = std::fs::rename(&tmp, path);
            }
        }
    }
}

/// 读取记忆的页签（托盘热键呼出用；无记忆/损坏回 emoji）。
#[must_use]
pub fn remembered_kind() -> PanelKind {
    PanelPosition::load()
        .map(|position| position.kind)
        .unwrap_or(PanelKind::Emoji)
}

unsafe extern "system" fn panel_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_NCCREATE => {
            let create = lparam.0 as *const CREATESTRUCTW;
            if !create.is_null() {
                let state = unsafe { (*create).lpCreateParams } as *mut PanelState;
                unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize) };
            }
            LRESULT(1)
        }
        // 常态非激活：点击面板既不激活它、也不把焦点从用户的目标应用拿走。
        // 搜索框例外（T-151，S-32）：点搜索框 = 进入搜索态，返回 MA_ACTIVATE 让
        // 鼠标点击合法激活面板以接收键盘；进入前记录前台窗口为目标。
        WM_MOUSEACTIVATE => {
            // lParam 为屏幕坐标：先转客户区再命中测试。
            let mut point = POINT {
                x: (lparam.0 & 0xFFFF) as i16 as i32,
                y: ((lparam.0 >> 16) & 0xFFFF) as i16 as i32,
            };
            unsafe {
                let _ = ScreenToClient(hwnd, &mut point);
                if let Some(state) = panel_state_mut(hwnd) {
                    if hit_search_box(hwnd, state, point.x, point.y) {
                        if !state.search_active {
                            state.search_active = true;
                            state.target_hwnd = GetForegroundWindow();
                            // 移除 NOACTIVATE 使本次点击可激活窗口。
                            set_no_activate(hwnd, false);
                        }
                        return LRESULT(MA_ACTIVATE as isize);
                    }
                }
                LRESULT(MA_NOACTIVATE as isize)
            }
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_PAINT => {
            unsafe {
                if let Some(state) = panel_state_mut(hwnd) {
                    paint(hwnd, state);
                } else {
                    let mut paint_struct = windows::Win32::Graphics::Gdi::PAINTSTRUCT::default();
                    let _ = BeginPaint(hwnd, &mut paint_struct);
                    let _ = EndPaint(hwnd, &paint_struct);
                }
            }
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
            unsafe {
                if let Some(state) = panel_state_mut(hwnd) {
                    on_click(hwnd, state, x, y);
                }
            }
            LRESULT(0)
        }
        // 搜索态（临时激活）下接收键盘：字符进查询、Esc 退出还焦、Tab 切页签、
        // Backspace 删字符、左右键翻页。
        WM_CHAR => {
            let ch = char::from_u32(wparam.0 as u32);
            unsafe {
                if let Some(state) = panel_state_mut(hwnd) {
                    if state.search_active {
                        if let Some(ch) = ch {
                            // 只收可显示字符（含空格）；控制字符由 WM_KEYDOWN 处理。
                            if !ch.is_control() {
                                state.query.push(ch);
                                state.view.page = 0;
                                state.hover = None;
                                invalidate(hwnd);
                            }
                        }
                    }
                }
            }
            LRESULT(0)
        }
        WM_KEYDOWN => {
            let vk = u32::from(wparam.0 as u16);
            unsafe {
                if let Some(state) = panel_state_mut(hwnd) {
                    if state.search_active {
                        match vk {
                            v if v == u32::from(VK_ESCAPE.0) => {
                                let target = state.target_hwnd;
                                leave_search(hwnd, state, true);
                                if !target.is_invalid() {
                                    let _ = SetForegroundWindow(target);
                                }
                            }
                            v if v == u32::from(VK_TAB.0) => {
                                state.view.kind = match state.view.kind {
                                    PanelKind::Emoji => PanelKind::Symbol,
                                    PanelKind::Symbol => PanelKind::Emoji,
                                };
                                state.view.page = 0;
                                state.hover = None;
                                invalidate(hwnd);
                            }
                            v if v == u32::from(VK_BACK.0) => {
                                state.query.pop();
                                state.view.page = 0;
                                state.hover = None;
                                invalidate(hwnd);
                            }
                            v if v == u32::from(VK_LEFT.0) => {
                                // 搜索态翻页按过滤结果的页数（不是全量分页）。
                                if state.view.page > 0 {
                                    state.view.page -= 1;
                                }
                                invalidate(hwnd);
                            }
                            v if v == u32::from(VK_RIGHT.0) => {
                                let filtered = filtered_entries(state);
                                let page_count = page_of(&filtered, 0, PANEL_PAGE_SIZE).page_count;
                                if state.view.page + 1 < page_count {
                                    state.view.page += 1;
                                }
                                invalidate(hwnd);
                            }
                            _ => {}
                        }
                    }
                }
            }
            LRESULT(0)
        }
        // 悬停高亮选中格（T-148 §6.3）：纯绘制，不改变"点击即上屏"的交互。
        WM_MOUSEMOVE => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
            unsafe {
                let mut track = TRACKMOUSEEVENT {
                    cbSize: u32::try_from(std::mem::size_of::<TRACKMOUSEEVENT>()).unwrap_or(0),
                    dwFlags: TME_LEAVE,
                    hwndTrack: hwnd,
                    dwHoverTime: 0,
                };
                let _ = TrackMouseEvent(&mut track);
                if let Some(state) = panel_state_mut(hwnd) {
                    let next = hover_index(hwnd, state, x, y);
                    if next != state.hover {
                        state.hover = next;
                        let _ = InvalidateRect(Some(hwnd), None, false);
                    }
                }
            }
            LRESULT(0)
        }
        WM_MOUSELEAVE => {
            unsafe {
                if let Some(state) = panel_state_mut(hwnd) {
                    if state.hover.is_some() {
                        state.hover = None;
                        let _ = InvalidateRect(Some(hwnd), None, false);
                    }
                }
            }
            LRESULT(0)
        }
        WM_THEMECHANGED | WM_SETTINGCHANGE => {
            unsafe {
                if let Some(state) = panel_state_mut(hwnd) {
                    state.theme_kind = shell::resolve_theme_kind();
                }
                let _ = InvalidateRect(Some(hwnd), None, false);
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            unsafe {
                close_panel(hwnd);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            unsafe {
                if let Some(state) = panel_state_mut(hwnd) {
                    let host = state.host_hwnd;
                    let quit = state.quit_on_close;
                    let target = state.target_hwnd;
                    let search_active = state.search_active;
                    // 位置 + 页签记忆（S-33）：关闭前把窗口矩形写回 panel.json。
                    remember_position(hwnd, state);
                    drop(Box::from_raw(state as *mut PanelState));
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                    // 搜索态下关闭：把焦点还给进入搜索态前的窗口（避免焦点悬空）。
                    if search_active && !target.is_invalid() {
                        let _ = SetForegroundWindow(target);
                    }
                    // 通知宿主（设置窗口）：面板已关。托盘宿主无需通知；
                    // 截图模式的独立消息循环靠 PostQuitMessage 退出。
                    if !host.is_invalid() {
                        let _ = PostMessageW(Some(host), WM_PANEL_CLOSED, WPARAM(0), LPARAM(0));
                    } else if quit {
                        PostQuitMessage(0);
                    }
                } else {
                    PostQuitMessage(0);
                }
            }
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

/// 关闭面板：销毁窗口即可，通知在 `WM_DESTROY` 里发出。
unsafe fn close_panel(hwnd: HWND) {
    unsafe {
        let _ = DestroyWindow(hwnd);
    }
}

unsafe fn invalidate(hwnd: HWND) {
    unsafe {
        let _ = InvalidateRect(Some(hwnd), None, false);
    }
}

unsafe fn panel_state_mut(hwnd: HWND) -> Option<&'static mut PanelState> {
    let pointer = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut PanelState;
    unsafe { pointer.as_mut() }
}

/// 处理点击：格子投递字符（搜索态先还焦），翻页改页，返回/页签/搜索框各自处理。
unsafe fn on_click(hwnd: HWND, state: &mut PanelState, x: i32, y: i32) {
    let Some(client) = (unsafe { client_rect(hwnd) }) else {
        return;
    };
    let metrics = PanelMetrics::new(state.dpi);
    let filtered = filtered_entries(state);
    let slice = page_of(&filtered, state.view.page, PANEL_PAGE_SIZE);
    let layout = layout::panel_layout(&metrics, client, slice.items.len());

    // 页签：点击切换（emoji / 符号），查询与分页状态随新页签重置。
    for (index, tab) in layout.tabs.iter().enumerate() {
        if layout::contains(*tab, x, y) {
            let desired = [PanelKind::Emoji, PanelKind::Symbol][index.min(1)];
            if state.view.kind != desired {
                state.view.kind = desired;
                state.view.page = 0;
                state.hover = None;
                unsafe { invalidate(hwnd) };
            }
            return;
        }
    }
    // 搜索框：进入搜索态的动作已在 WM_MOUSEACTIVATE 完成；这里只刷新（光标提示）。
    if layout::contains(layout.search, x, y) {
        unsafe { invalidate(hwnd) };
        return;
    }
    if layout::contains(layout.back, x, y) {
        unsafe { close_panel(hwnd) };
        return;
    }
    if layout::contains(layout.prev, x, y) {
        if slice.page > 0 {
            state.view.page -= 1;
        }
        unsafe { invalidate(hwnd) };
        return;
    }
    if layout::contains(layout.next, x, y) {
        if slice.page + 1 < slice.page_count {
            state.view.page += 1;
        }
        unsafe { invalidate(hwnd) };
        return;
    }
    for (index, cell) in layout.cells.iter().enumerate() {
        if layout::contains(*cell, x, y) {
            if let Some(entry) = slice.items.get(index).copied() {
                // 搜索态选中：先还焦给进入搜索态前的目标窗口，再投递，字符落在目标应用。
                if state.search_active {
                    let target = state.target_hwnd;
                    unsafe { leave_search(hwnd, state, false) };
                    if !target.is_invalid() {
                        unsafe {
                            let _ = SetForegroundWindow(target);
                        }
                    }
                }
                let delivery = deliver_text(entry.text);
                state.hint = delivery.hint().map(str::to_owned);
                if delivery == Delivery::Typed {
                    // 上屏成功时不打断浏览：面板继续开着，方便连续点选。
                    state.hint = Some(format!("已上屏 {}", entry.text));
                }
                unsafe { invalidate(hwnd) };
            }
            return;
        }
    }
}

/// 当前查询下的条目序列（全量浏览或过滤命中），绘制/命中测试/分页共用。
fn filtered_entries(state: &PanelState) -> Vec<PanelEntry> {
    filter_entries(state.view.kind, &state.query)
}

/// 搜索框命中测试（客户区坐标）；布局失败（无客户区）按未命中处理。
fn hit_search_box(hwnd: HWND, state: &PanelState, x: i32, y: i32) -> bool {
    let Some(client) = (unsafe { client_rect(hwnd) }) else {
        return false;
    };
    let metrics = PanelMetrics::new(state.dpi);
    let layout = layout::panel_layout(&metrics, client, 0);
    layout::contains(layout.search, x, y)
}

/// 退出搜索态：恢复 NOACTIVATE（回到非激活点格即上屏），可选清空查询（Esc 取消）。
unsafe fn leave_search(hwnd: HWND, state: &mut PanelState, clear: bool) {
    if !state.search_active {
        if clear {
            state.query.clear();
        }
        return;
    }
    state.search_active = false;
    if clear {
        state.query.clear();
    }
    state.hover = None;
    set_no_activate(hwnd, true);
    unsafe { invalidate(hwnd) };
}

/// 开关面板窗口的 `WS_EX_NOACTIVATE`（搜索态启用键盘时临时移除，S-32）。
unsafe fn set_no_activate(hwnd: HWND, no_activate: bool) {
    unsafe {
        let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as i32;
        let updated = if no_activate {
            ex | WS_EX_NOACTIVATE.0 as i32
        } else {
            ex & !(WS_EX_NOACTIVATE.0 as i32)
        };
        if updated != ex {
            let _ = SetWindowLongPtrW(hwnd, GWL_EXSTYLE, updated as isize);
            let _ = SetWindowPos(
                hwnd,
                None,
                0,
                0,
                0,
                0,
                SWP_FRAMECHANGED | SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER,
            );
        }
    }
}

/// 关闭前面板的位置/页签记忆（S-33）。
unsafe fn remember_position(hwnd: HWND, state: &PanelState) {
    let mut rect = RECT::default();
    if unsafe { GetWindowRect(hwnd, &mut rect) }.is_err() {
        return;
    }
    PanelPosition {
        kind: state.view.kind,
        left: rect.left,
        top: rect.top,
        width: rect.right - rect.left,
        height: rect.bottom - rect.top,
    }
    .save();
}

unsafe fn client_rect(hwnd: HWND) -> Option<UiRect> {
    let mut rect = RECT::default();
    unsafe { GetClientRect(hwnd, &mut rect) }.ok()?;
    Some(UiRect {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    })
}

unsafe fn paint(hwnd: HWND, state: &mut PanelState) {
    unsafe {
        let mut paint_struct = windows::Win32::Graphics::Gdi::PAINTSTRUCT::default();
        let hdc = BeginPaint(hwnd, &mut paint_struct);
        if hdc.is_invalid() {
            return;
        }
        if let Some(client) = client_rect(hwnd) {
            let width = client.width();
            let height = client.height();
            if width > 0 && height > 0 {
                let memory = state.back.prepare(hdc, width, height);
                if !memory.is_invalid() {
                    draw(memory, state, client);
                    let _ = BitBlt(hdc, 0, 0, width, height, Some(memory), 0, 0, SRCCOPY);
                    if state.shot_path.is_some() && !state.shot_done {
                        state.shot_done = true;
                        if let Some(path) = state.shot_path.clone() {
                            match state.back.save_bmp(width, height, &path) {
                                Ok(()) => {
                                    eprintln!("zhu-ye-settings: 面板截图已写出 {}", path.display())
                                }
                                Err(error) => eprintln!("zhu-ye-settings: {error}"),
                            }
                        }
                        let _ = PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
                    }
                }
            }
        }
        let _ = EndPaint(hwnd, &paint_struct);
    }
}

unsafe fn draw(hdc: HDC, state: &mut PanelState, client: UiRect) {
    let theme = state.theme();
    let metrics = PanelMetrics::new(state.dpi);
    unsafe {
        state.fonts.ensure(state.dpi);
    }
    let filtered = filtered_entries(state);
    let slice = page_of(&filtered, state.view.page, PANEL_PAGE_SIZE);
    let layout = layout::panel_layout(&metrics, client, slice.items.len());
    let entries = slice.items;

    unsafe {
        fill(hdc, client, theme.window);
        fill(hdc, layout.header, theme.nav_background);
        fill(
            hdc,
            UiRect {
                left: layout.header.left,
                top: layout.header.bottom - 1,
                right: layout.header.right,
                bottom: layout.header.bottom,
            },
            theme.chip_border,
        );

        // 页签：选中 = 主字色 + 底边 2px 叶色下划线；未选中 = 次要字色。
        for (index, tab) in layout.tabs.iter().enumerate() {
            let active = index
                == match state.view.kind {
                    PanelKind::Emoji => 0,
                    PanelKind::Symbol => 1,
                };
            draw_text(
                hdc,
                [PanelKind::Emoji, PanelKind::Symbol][index.min(1)].title(),
                *tab,
                if active {
                    theme.title_text
                } else {
                    theme.secondary_text
                },
                state.fonts.title,
                DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
            );
            if active {
                // 页签下划线置于标题基线下方一小段，不压住搜索框。
                let underline = UiRect {
                    left: tab.left + metrics.gap / 2,
                    top: tab.bottom - metrics.gap,
                    right: tab.right - metrics.gap / 2,
                    bottom: tab.bottom - metrics.gap / 2,
                };
                fill(hdc, underline, theme.accent);
            }
        }

        // 搜索框：外框（搜索态用叶色强调）+ 内底；无查询时画占位提示。
        let search_border = if state.search_active {
            theme.accent
        } else {
            theme.chip_border
        };
        fill_round(hdc, layout.search, metrics.gap / 2, search_border);
        fill_round(
            hdc,
            UiRect {
                left: layout.search.left + 1,
                top: layout.search.top + 1,
                right: layout.search.right - 1,
                bottom: layout.search.bottom - 1,
            },
            (metrics.gap / 2 - 1).max(0),
            theme.control_background,
        );
        let search_text = state.query.clone();
        if search_text.is_empty() {
            let placeholder = if state.search_active {
                "输入拼音别名过滤…"
            } else {
                "点击输入关键词过滤（拼音别名/字符）"
            };
            draw_text(
                hdc,
                placeholder,
                layout.search,
                theme.secondary_text,
                state.fonts.small,
                DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
            );
        } else {
            draw_text(
                hdc,
                &search_text,
                layout.search,
                theme.item_text,
                state.fonts.small,
                DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
            );
        }

        // 格子：字符居中，副标签在下方（emoji 为拼音别名，符号为组名）
        for (index, cell) in layout.cells.iter().enumerate() {
            let Some(entry) = entries.get(index) else {
                continue;
            };
            // 选中格（悬停）：实叶底 + 面板字色（T-148 §6.3；深色叶底用墨字保证对比）。
            let selected = state.hover == Some(index);
            let cell_bg = if selected {
                theme.accent
            } else {
                theme.control_background
            };
            let glyph_color = if selected {
                theme.on_accent
            } else {
                theme.item_text
            };
            fill_round(hdc, *cell, metrics.gap, cell_bg);
            let glyph = UiRect {
                top: cell.top + metrics.gap / 2,
                bottom: cell.bottom - state.fonts.small_height - metrics.gap / 2,
                ..*cell
            };
            draw_text(
                hdc,
                entry.text,
                glyph,
                glyph_color,
                state.fonts.title,
                DT_CENTER | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
            );
            let label = UiRect {
                top: cell.bottom - state.fonts.small_height - metrics.gap / 2,
                bottom: cell.bottom - metrics.gap / 4,
                ..*cell
            };
            draw_text(
                hdc,
                entry.label,
                label,
                if selected {
                    theme.on_accent
                } else {
                    theme.secondary_text
                },
                state.fonts.small,
                DT_CENTER | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
            );
        }

        // 提示：默认显示操作提示；搜索无命中时提示"无匹配"。
        let hint = state.hint.clone().unwrap_or_else(|| {
            if !state.query.is_empty() && slice.items.is_empty() {
                format!("无匹配「{}」，请换更短的关键词", state.query)
            } else if state.search_active {
                "输入拼音别名过滤 · Esc 退出 · Tab 切换面板".to_owned()
            } else {
                format!("点击格子上屏 · 呼出热键 {}", hotkey_display())
            }
        });
        draw_text(
            hdc,
            &hint,
            layout.hint,
            theme.secondary_text,
            state.fonts.small,
            DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
        );

        // 页脚
        draw_button(hdc, state, layout.back, "关闭", false);
        draw_button(hdc, state, layout.prev, "上一页", false);
        draw_button(hdc, state, layout.next, "下一页", false);
        draw_text(
            hdc,
            &format!("{}/{}", slice.page + 1, slice.page_count),
            layout.page_label,
            theme.secondary_text,
            state.fonts.small,
            DT_CENTER | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
        );

        // 外边框（T-148 §6.3：面板外框取 chip_border 描边族，与 chips 同源）
        fill(
            hdc,
            UiRect {
                left: client.left,
                top: client.top,
                right: client.right,
                bottom: client.top + 1,
            },
            theme.chip_border,
        );
        fill(
            hdc,
            UiRect {
                left: client.left,
                top: client.bottom - 1,
                right: client.right,
                bottom: client.bottom,
            },
            theme.chip_border,
        );
        fill(
            hdc,
            UiRect {
                left: client.left,
                top: client.top,
                right: client.left + 1,
                bottom: client.bottom,
            },
            theme.chip_border,
        );
        fill(
            hdc,
            UiRect {
                left: client.right - 1,
                top: client.top,
                right: client.right,
                bottom: client.bottom,
            },
            theme.chip_border,
        );
    }
}

/// 配置里的呼出热键描述（解析失败/未配置回默认 "Ctrl+Alt+E"）。
fn hotkey_display() -> String {
    use zhu_ye_core::hotkey::{parse_hotkey, HotkeySpec, DEFAULT_PANEL_HOTKEY};
    let configured = crate::config::config_path()
        .and_then(|path| zhu_ye_core::load_config(&path).0.panel_hotkey)
        .and_then(|spec| parse_hotkey(&spec))
        .map(HotkeySpec::display);
    configured.unwrap_or_else(|| {
        parse_hotkey(DEFAULT_PANEL_HOTKEY)
            .map(HotkeySpec::display)
            .unwrap_or_default()
    })
}

unsafe fn draw_button(hdc: HDC, state: &PanelState, rect: UiRect, label: &str, primary: bool) {
    let theme = state.theme();
    let metrics = PanelMetrics::new(state.dpi);
    unsafe {
        fill_round(
            hdc,
            rect,
            metrics.gap,
            if primary {
                // 主按钮 = 实叶底，字色用面板 on_accent（T-148 §7）。
                theme.accent
            } else {
                theme.control_background
            },
        );
        draw_text(
            hdc,
            label,
            rect,
            if primary {
                theme.on_accent
            } else {
                theme.item_text
            },
            state.fonts.small,
            DT_CENTER | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
        );
    }
}

/// 鼠标位置所在的格子下标（仅本页已有条目；落在空白/外框上为 `None`）。
fn hover_index(hwnd: HWND, state: &PanelState, x: i32, y: i32) -> Option<usize> {
    let mut rect = RECT::default();
    unsafe {
        GetClientRect(hwnd, &mut rect).ok()?;
    }
    let client = UiRect {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    };
    let metrics = PanelMetrics::new(state.dpi);
    let filtered = filtered_entries(state);
    let slice = page_of(&filtered, state.view.page, PANEL_PAGE_SIZE);
    let layout = layout::panel_layout(&metrics, client, slice.items.len());
    layout
        .cells
        .iter()
        .position(|cell| x >= cell.left && x < cell.right && y >= cell.top && y < cell.bottom)
}

fn register_panel_class(instance: HINSTANCE) -> Result<(), String> {
    use std::sync::atomic::{AtomicU16, Ordering};
    static REGISTERED_ATOM: AtomicU16 = AtomicU16::new(0);
    if REGISTERED_ATOM.load(Ordering::Relaxed) != 0 {
        return Ok(());
    }
    let class = to_utf16(PANEL_CLASS_NAME);
    let cursor = unsafe { LoadCursorW(None, IDC_ARROW) }.unwrap_or_default();
    let class_def = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(panel_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: Default::default(),
        hCursor: cursor,
        hbrBackground: HBRUSH::default(),
        lpszMenuName: PCWSTR::null(),
        lpszClassName: PCWSTR(class.as_ptr()),
    };
    let atom = unsafe { RegisterClassW(&class_def) };
    if atom == 0 {
        return Err(format!(
            "注册面板窗口类失败: {}",
            std::io::Error::last_os_error()
        ));
    }
    REGISTERED_ATOM.store(atom, Ordering::Relaxed);
    Ok(())
}
