//! 设置窗口的 Win32 层：窗口类、消息循环、GDI 双缓冲绘制与命中测试。
//!
//! 绘制与命中测试共用 `layout` 的同一份矩形，避免"看得见的地方点不到"。窗口跟随系统
//! 深浅色与高对比度（D-30），主题变化时重算配色并重绘。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread;

use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{CloseHandle, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, EndPaint, InvalidateRect, UpdateWindow, DT_CENTER, DT_END_ELLIPSIS,
    DT_LEFT, DT_NOPREFIX, DT_RIGHT, DT_SINGLELINE, DT_VCENTER, DT_WORDBREAK, HBRUSH, HDC, HFONT,
    SRCCOPY,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::{GetExitCodeProcess, WaitForSingleObject, INFINITE};
use windows::Win32::UI::Controls::Dialogs::{
    GetOpenFileNameW, GetSaveFileNameW, OFN_FILEMUSTEXIST, OFN_HIDEREADONLY, OFN_NOCHANGEDIR,
    OFN_OVERWRITEPROMPT, OFN_PATHMUSTEXIST, OPENFILENAMEW,
};
use windows::Win32::UI::HiDpi::AdjustWindowRectExForDpi;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_ESCAPE;
use windows::Win32::UI::Shell::{ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, FindWindowW, GetClientRect,
    GetForegroundWindow, GetMessageW, GetWindowLongPtrW, LoadCursorW, PostMessageW,
    PostQuitMessage, RegisterClassW, SetForegroundWindow, SetWindowLongPtrW, ShowWindow,
    TranslateMessage, CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA,
    IDC_ARROW, MB_DEFBUTTON2, MB_ICONQUESTION, MB_YESNO, SW_RESTORE, SW_SHOW, SW_SHOWNORMAL,
    WINDOW_EX_STYLE, WM_CLOSE, WM_DESTROY, WM_DPICHANGED, WM_ERASEBKGND, WM_KEYDOWN,
    WM_LBUTTONDOWN, WM_NCCREATE, WM_PAINT, WM_SETTINGCHANGE, WM_SYSKEYDOWN, WM_THEMECHANGED,
    WM_USER, WNDCLASSW, WS_OVERLAPPEDWINDOW,
};
use zhu_ye_core::identity::DICTIONARY_FILE_NAME;
use zhu_ye_core::{ModeChoice, ThemeChoice};
use zhu_ye_ui::{UiRect, UiThemeKind, BASE_DPI};

use crate::config;
use crate::gdi::{draw_text, fill, fill_round, BackBuffer, Fonts};
use crate::installed::{self, InstalledPack, PackSource};
use crate::inventory::{self, list_packs, PackInfo};
use crate::layout::{self, Chip, ChipValue, SettingsMetrics};
use crate::model::{ItemControl, OpenTarget, Page, SettingsState, Subview};
use crate::panel::PanelView;
use crate::panel_window::{PanelWindow, WM_PANEL_CLOSED};
use crate::registry;
use crate::repair::{self, evaluate_registration, L1Outcome, RegistrationStatus, RepairScan};
use crate::shell;
use crate::theme::{settings_theme, settings_theme_from_system_colors, SettingsTheme};
use crate::updater;
use crate::wide::to_utf16;

/// 窗口类名；第二条实例用它查找已有窗口。
pub const WINDOW_CLASS_NAME: &str = "ZhuYeSettingsWindow";
/// 窗口标题。
pub const WINDOW_TITLE: &str = "竹叶输入法 设置";

/// 后台更新器任务完成的回执消息（spawn 的线程完成后投递，结果经 mpsc 取回）。
const WM_UPDATER_DONE: u32 = WM_USER + 0x120;

/// 后台更新器任务的类型（决定线程的更新器参数与界面的结果去向）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UpdateKind {
    /// `zhu-ye-updater check`：只检查，不下载不应用。
    Check,
    /// `zhu-ye-updater apply`：下载并应用可用更新（联网发生在子进程内，D-44）。
    Apply,
}

impl UpdateKind {
    /// 任务进行中的提示文案。
    const fn busy_label(self) -> &'static str {
        match self {
            Self::Check => "正在检查更新…",
            Self::Apply => "正在下载并应用更新…",
        }
    }

    /// 更新器子命令参数。
    const fn args(self) -> &'static [&'static str] {
        match self {
            Self::Check => &["check"],
            Self::Apply => &["apply"],
        }
    }

    /// 完成后写入结果区的标题行。
    const fn done_title(self) -> &'static str {
        match self {
            Self::Check => "检查更新结果",
            Self::Apply => "应用更新结果",
        }
    }
}

/// 后台任务完成时送回的内容。
struct UpdateOutcome {
    kind: UpdateKind,
    result: Result<String, String>,
}

/// 启动选项。
#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    /// 截图输出路径；`Some` 时在首帧绘制后写出 BMP 并关闭窗口（验收取证）。
    pub shot_path: Option<PathBuf>,
    /// 截图时显示的页面。
    pub shot_page: Option<Page>,
    /// 截图时展开的条目下标。
    pub shot_expanded: Option<usize>,
    /// 截图时进入「添加词库」子视图（与 `shot_page` 互斥，优先于条目展开）。
    pub shot_packs: bool,
    /// 截图时进入「管理输入法」子视图（同上互斥规则）。
    pub shot_manage: bool,
    /// 截图时进入「修复输入法」子视图（同上互斥规则）。
    pub shot_repair: bool,
    /// 截图时进入「检查更新」子视图（同上互斥规则）。
    pub shot_update: bool,
    /// 截图时进入「版本与诊断信息」子视图（同上互斥规则）。
    pub shot_diag: bool,
    /// 截图时进入「用户词表」子视图（T-088 取证，同上互斥规则）。
    pub shot_user_words: bool,
    /// 截图时进入「通讯录」子视图（T-088 取证，同上互斥规则）。
    pub shot_contacts: bool,
    /// 截图时进入「自定义主题」子视图（T-088 取证，同上互斥规则）。
    pub shot_themes: bool,
}

/// 窗口运行状态。
struct WindowState {
    settings: SettingsState,
    theme_kind: UiThemeKind,
    config_path: Option<PathBuf>,
    /// 一次性反馈（保存结果等），显示在底部提示条。
    hint: Option<String>,
    shot_path: Option<PathBuf>,
    shot_done: bool,
    dpi: u32,
    back: BackBuffer,
    fonts: Fonts,
    /// 设置窗口出现前的前台窗口。打开工具箱面板时把焦点还给它，面板才能把字符
    /// 送进用户真正在打字的那个应用。
    previous_foreground: HWND,
    /// 已打开的工具箱面板；关闭时置空。
    panel: Option<PanelWindow>,
    /// 「添加词库」子视图的包清单；进入子视图时从磁盘重扫（FR-042）。
    packs: Vec<PackInfo>,
    /// 「管理输入法」子视图的注册状态；进入时探测（T-076 / FR-043）。
    manage: Option<RegistrationStatus>,
    /// 「修复输入法」子视图的检测结果；进入时扫描、修复后重扫（T-076 / FR-043）。
    repair_scan: Option<RepairScan>,
    /// 正在运行的后台更新器任务（None 表示空闲）；完成后回执到主线程再清空。
    update_busy: Option<UpdateKind>,
    /// 最近一次后台任务的结果（含错误）；进入子视图或任务完成时更新。
    update_result: Option<(UpdateKind, Result<String, String>)>,
    /// 后台任务完成回执的接收端；spawn 新任务时重建。
    update_rx: Option<Receiver<UpdateOutcome>>,
    /// 已派发的任务序号；回执消息用它识别"当前任务"（旧任务迟到回执忽略）。
    update_seq: u32,
    /// 「版本与诊断信息」子视图的逐行内容；进入时从本地收集（不联网）。
    diagnostics: Option<Vec<String>>,
    /// 自定义主题（`Custom(name)`）解析出的设置窗配色与覆盖键数；非自定义主题为
    /// `None`。文件缺失/解析失败回退系统深浅预设（T-088 / FR-048）。
    custom_theme: Option<(SettingsTheme, usize)>,
    /// 「用户词表」子视图的上次导入/导出结果消息。
    user_words_message: Option<String>,
    /// 「通讯录」子视图的已登记 .vcf 路径列表；进入子视图时从配置装载（T-088）。
    contacts: Vec<PathBuf>,
    /// 「自定义主题」子视图的主题文件清单；进入子视图时扫描 themes 目录（T-088）。
    theme_files: Vec<ThemeInfo>,
}

/// 「自定义主题」子视图中的一个主题文件。
#[derive(Debug, Clone)]
struct ThemeInfo {
    /// 文件名（不含 `.json` 后缀，即主题 id，写进配置 `theme: Custom(name)`）。
    id: String,
    /// 显示名（取文件内 `name` 字段，缺省用文件名）。
    display_name: String,
    /// 完整解析结果；`None` 表示文件损坏或版本过高，选择时给出错误提示。
    file: Option<zhu_ye_core::ThemeFile>,
}

impl WindowState {
    fn new(
        theme: ThemeChoice,
        config_path: Option<PathBuf>,
        options: &RunOptions,
        previous_foreground: HWND,
    ) -> Self {
        // D-32 装配项：新会话默认中英模式从配置装载，运行中的 Shift 模式与之无关。
        let default_mode = match &config_path {
            Some(path) => config::load_mode(path).0,
            None => ModeChoice::Chinese,
        };
        // P-03：在线更新默认关闭；开启与否都从配置装载，窗口如实反映当前状态。
        let online_update = match &config_path {
            Some(path) => config::load_online_update(path).0,
            None => false,
        };
        // T-127：候选框拼音行默认显示（历史行为）；从配置装载，窗口如实反映。
        let candidate_show_pin = match &config_path {
            Some(path) => config::load_candidate_show_pin(path).0,
            None => true,
        };
        let mut settings = SettingsState::with_config(
            theme.clone(),
            default_mode,
            online_update,
            candidate_show_pin,
        );
        if let Some(page) = options.shot_page {
            settings.page = page;
            settings.expanded = options.shot_expanded;
        }
        if options.shot_packs {
            settings.page = Page::Common;
            settings.open_packs();
        }
        if options.shot_manage {
            settings.page = Page::Common;
            settings.open_manage();
        }
        if options.shot_repair {
            settings.page = Page::Common;
            settings.open_repair();
        }
        if options.shot_update {
            settings.page = Page::About;
            settings.open_update();
        }
        if options.shot_diag {
            settings.page = Page::About;
            settings.open_diagnostics();
        }
        if options.shot_user_words {
            settings.page = Page::Common;
            settings.open_user_words();
        }
        if options.shot_contacts {
            settings.page = Page::Common;
            settings.open_contacts();
        }
        if options.shot_themes {
            settings.page = Page::Common;
            settings.open_themes();
        }
        let packs = if settings.subview == Subview::Packs {
            list_packs_now(config_path.as_deref())
        } else {
            Vec::new()
        };
        // 取证模式直接进入子视图时，首帧就需要真实数据。
        let manage = if settings.subview == Subview::Manage {
            Some(evaluate_registration(&registry::probe_registration()))
        } else {
            None
        };
        let repair_scan = if settings.subview == Subview::Repair {
            Some(repair::scan_l1(
                &repair_packs_dir(config_path.as_deref()),
                config_path.as_deref(),
                &repair_user_words_path(),
                &shell::exe_dir().join(DICTIONARY_FILE_NAME),
            ))
        } else {
            None
        };
        // 取证模式直接进入诊断子视图时，首帧就需要真实信息。
        let diagnostics = if settings.subview == Subview::Diagnostics {
            Some(build_diagnostics(config_path.as_deref()))
        } else {
            None
        };
        let custom_theme =
            resolve_custom_theme(shell::resolve_theme_kind(), &theme, config_path.as_deref());
        // 取证模式直接进入 T-088 三子视图时，首帧就需要真实数据。
        let user_words_message = None;
        let contacts = if settings.subview == Subview::Contacts {
            load_contacts(config_path.as_deref())
        } else {
            Vec::new()
        };
        let theme_files = if settings.subview == Subview::Themes {
            list_theme_files(config_path.as_deref())
        } else {
            Vec::new()
        };
        Self {
            settings,
            theme_kind: shell::resolve_theme_kind(),
            config_path,
            hint: None,
            shot_path: options.shot_path.clone(),
            shot_done: false,
            dpi: shell::system_dpi().max(BASE_DPI),
            back: BackBuffer::default(),
            fonts: Fonts::default(),
            previous_foreground,
            panel: None,
            packs,
            manage,
            repair_scan,
            update_busy: None,
            update_result: None,
            update_rx: None,
            update_seq: 0,
            diagnostics,
            custom_theme,
            user_words_message,
            contacts,
            theme_files,
        }
    }

    /// 当前配色。
    fn theme(&self) -> SettingsTheme {
        if self.theme_kind == UiThemeKind::HighContrast {
            // D-31：高对比度由系统接管，自定义主题文件不覆盖。
            return settings_theme_from_system_colors(shell::system_colors());
        }
        if let ThemeChoice::Custom(_) = self.settings.theme {
            if let Some((theme, _)) = &self.custom_theme {
                return *theme;
            }
        }
        settings_theme(self.theme_kind)
    }
}

/// 运行设置窗口直到关闭。
///
/// # Errors
/// 注册窗口类或创建窗口失败时返回描述。
pub fn run(options: RunOptions) -> Result<(), String> {
    unsafe {
        shell::enable_per_monitor_v2();
        let module = GetModuleHandleW(PCWSTR::null())
            .map_err(|error| format!("获取模块句柄失败: {error}"))?;
        let instance = HINSTANCE(module.0);
        register_window_class(instance)?;

        let dpi = shell::system_dpi().max(BASE_DPI);
        let metrics = SettingsMetrics::new(dpi);
        let (client_width, client_height) = metrics.desired_client_size();
        let mut frame = RECT {
            left: 0,
            top: 0,
            right: client_width,
            bottom: client_height,
        };
        // 客户区尺寸是布局基准，窗口外框按当前 DPI 补足边框。
        let _ = AdjustWindowRectExForDpi(
            &mut frame,
            WS_OVERLAPPEDWINDOW,
            false,
            WINDOW_EX_STYLE(0),
            dpi,
        );

        let (config_file_path, theme) = match config::config_path() {
            Some(path) => {
                let (theme, _) = config::load_theme(&path);
                (Some(path), theme)
            }
            None => (None, ThemeChoice::Light),
        };

        let state = Box::new(WindowState::new(
            theme,
            config_file_path,
            &options,
            // 必须在设置窗口显示前取：显示之后前台窗口就是我们自己了。
            GetForegroundWindow(),
        ));
        let state_ptr = Box::into_raw(state);

        let class = to_utf16(WINDOW_CLASS_NAME);
        let title = to_utf16(WINDOW_TITLE);
        let hwnd = match CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(class.as_ptr()),
            PCWSTR(title.as_ptr()),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            frame.right - frame.left,
            frame.bottom - frame.top,
            None,
            None,
            Some(instance),
            Some(state_ptr.cast()),
        ) {
            Ok(hwnd) => hwnd,
            Err(error) => {
                // 窗口未创建，`WM_NCCREATE` 不会到达，所有权仍在本地。
                drop(Box::from_raw(state_ptr));
                return Err(format!("创建设置窗口失败: {error}"));
            }
        };

        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = UpdateWindow(hwnd);

        let mut message = windows::Win32::UI::WindowsAndMessaging::MSG::default();
        while GetMessageW(&mut message, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
        Ok(())
    }
}

/// 激活已有实例的窗口；返回是否找到。
pub fn focus_existing() -> bool {
    unsafe {
        let class = to_utf16(WINDOW_CLASS_NAME);
        match FindWindowW(PCWSTR(class.as_ptr()), PCWSTR::null()) {
            Ok(hwnd) if !hwnd.is_invalid() => {
                let _ = ShowWindow(hwnd, SW_RESTORE);
                let _ = SetForegroundWindow(hwnd);
                true
            }
            _ => false,
        }
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_NCCREATE => {
            let create = lparam.0 as *const CREATESTRUCTW;
            if !create.is_null() {
                let state = unsafe { (*create).lpCreateParams } as *mut WindowState;
                unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize) };
            }
            LRESULT(1)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_PAINT => {
            unsafe {
                if let Some(state) = state_mut(hwnd) {
                    paint(hwnd, state);
                } else {
                    let mut paint = windows::Win32::Graphics::Gdi::PAINTSTRUCT::default();
                    let _ = BeginPaint(hwnd, &mut paint);
                    let _ = EndPaint(hwnd, &paint);
                }
            }
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
            unsafe {
                if let Some(state) = state_mut(hwnd) {
                    on_click(hwnd, state, x, y);
                }
            }
            LRESULT(0)
        }
        WM_DPICHANGED => {
            unsafe {
                if let Some(state) = state_mut(hwnd) {
                    state.dpi = u32::from((wparam.0 & 0xFFFF) as u16).max(BASE_DPI);
                    state.fonts.release();
                }
                let rect = &*(lparam.0 as *const RECT);
                let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowPos(
                    hwnd,
                    None,
                    rect.left,
                    rect.top,
                    rect.right - rect.left,
                    rect.bottom - rect.top,
                    windows::Win32::UI::WindowsAndMessaging::SWP_NOZORDER
                        | windows::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE,
                );
                invalidate(hwnd);
            }
            LRESULT(0)
        }
        WM_THEMECHANGED | WM_SETTINGCHANGE => {
            unsafe {
                if let Some(state) = state_mut(hwnd) {
                    state.theme_kind = shell::resolve_theme_kind();
                }
                invalidate(hwnd);
            }
            LRESULT(0)
        }
        // 面板关闭：销毁它并把焦点收回到设置窗口。
        WM_PANEL_CLOSED => {
            unsafe {
                if let Some(state) = state_mut(hwnd) {
                    state.panel = None;
                    state.hint = Some("面板已关闭".to_owned());
                }
                let _ = SetForegroundWindow(hwnd);
                invalidate(hwnd);
            }
            LRESULT(0)
        }
        // 后台更新器任务完成（T-077 / FR-044）：取回结果并重绘。
        // 任务线程只负责运行更新器与投递回执；一切输出处理都在主线程完成。
        // `wparam` 携带任务序号：旧任务（已被新任务顶替）的迟到回执直接忽略。
        WM_UPDATER_DONE => {
            unsafe {
                if let Some(state) = state_mut(hwnd) {
                    if (wparam.0 as u32) == state.update_seq {
                        if let Some(rx) = state.update_rx.take() {
                            // 线程在投递回执前完成 send，这里必能取到；竞态时清 busy 自愈。
                            if let Ok(outcome) = rx.try_recv() {
                                state.update_busy = None;
                                state.update_result = Some((outcome.kind, outcome.result));
                            } else {
                                state.update_busy = None;
                            }
                        } else {
                            state.update_busy = None;
                        }
                    }
                }
                invalidate(hwnd);
            }
            LRESULT(0)
        }
        WM_KEYDOWN | WM_SYSKEYDOWN => {
            if u16::try_from(wparam.0).ok() == Some(VK_ESCAPE.0) {
                unsafe {
                    let _ = DestroyWindow(hwnd);
                }
                return LRESULT(0);
            }
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
        WM_CLOSE => {
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            unsafe {
                if let Some(state) = state_mut(hwnd) {
                    drop(Box::from_raw(state as *mut WindowState));
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                }
                PostQuitMessage(0);
            }
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

unsafe fn state_mut(hwnd: HWND) -> Option<&'static mut WindowState> {
    let pointer = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut WindowState;
    unsafe { pointer.as_mut() }
}

unsafe fn invalidate(hwnd: HWND) {
    unsafe {
        let _ = InvalidateRect(Some(hwnd), None, false);
    }
}

/// 处理左键点击：先命中二选一控件，再命中条目行，最后命中导航。
/// 打开子视图时只响应子视图内的命中（返回 / 动作按钮 / 行内开关）。
unsafe fn on_click(hwnd: HWND, state: &mut WindowState, x: i32, y: i32) {
    let client = match client_rect(hwnd) {
        Some(rect) => rect,
        None => return,
    };
    let metrics = SettingsMetrics::new(state.dpi);

    match state.settings.subview {
        Subview::Packs => {
            on_packs_click(hwnd, state, &metrics, client, x, y);
            return;
        }
        Subview::Manage => {
            on_manage_click(state, &metrics, client, x, y);
            return;
        }
        Subview::Repair => {
            on_repair_click(hwnd, state, &metrics, client, x, y);
            return;
        }
        Subview::Update => {
            on_update_click(hwnd, state, &metrics, client, x, y);
            return;
        }
        Subview::Diagnostics => {
            on_diagnostics_click(state, &metrics, client, x, y);
            return;
        }
        Subview::UserWords => {
            on_user_words_click(hwnd, state, &metrics, client, x, y);
            return;
        }
        Subview::Contacts => {
            on_contacts_click(hwnd, state, &metrics, client, x, y);
            return;
        }
        Subview::Themes => {
            on_themes_click(hwnd, state, &metrics, client, x, y);
            return;
        }
        Subview::None => {}
    }

    for (page, rect) in layout::nav_rows(&metrics, client) {
        if layout::contains(rect, x, y) {
            state.settings.select_page(page);
            invalidate(hwnd);
            return;
        }
    }

    let items = state.settings.page.items();
    for row in layout::item_rows(&metrics, client, items, state.settings.expanded) {
        for chip in &row.chips {
            if layout::contains(chip.rect, x, y) {
                apply_chip(state, chip);
                invalidate(hwnd);
                return;
            }
        }
        if layout::contains(row.rect, x, y) {
            match items[row.index].control {
                // 二选一行本身不切换展开，避免与控件块点击混淆。
                ItemControl::ThemeChoice | ItemControl::ModeChoice | ItemControl::CandidatePin => {}
                ItemControl::OpenPanel(kind) => open_panel(hwnd, state, kind),
                ItemControl::OpenPath(target) => open_target(state, target),
                ItemControl::OpenPacks => {
                    state.packs = list_packs_now(state.config_path.as_deref());
                    state.settings.open_packs();
                    invalidate(hwnd);
                }
                ItemControl::OpenManage => {
                    state.manage = Some(evaluate_registration(&registry::probe_registration()));
                    state.settings.open_manage();
                    invalidate(hwnd);
                }
                ItemControl::OpenRepair => {
                    state.repair_scan = Some(scan_repair_now(state));
                    state.settings.open_repair();
                    invalidate(hwnd);
                }
                ItemControl::OpenUpdate => {
                    state.settings.open_update();
                    invalidate(hwnd);
                }
                ItemControl::OpenDiagnostics => {
                    state.diagnostics = Some(build_diagnostics(state.config_path.as_deref()));
                    state.settings.open_diagnostics();
                    invalidate(hwnd);
                }
                ItemControl::OpenUserWords => {
                    state.settings.open_user_words();
                    state.hint = None;
                    invalidate(hwnd);
                }
                ItemControl::OpenContacts => {
                    state.contacts = load_contacts(state.config_path.as_deref());
                    state.settings.open_contacts();
                    invalidate(hwnd);
                }
                ItemControl::OpenThemes => {
                    state.theme_files = list_theme_files(state.config_path.as_deref());
                    state.settings.open_themes();
                    invalidate(hwnd);
                }
                ItemControl::OnlineUpdate => {}
                ItemControl::RestoreLangBar => {
                    restore_lang_bar(hwnd, state);
                    invalidate(hwnd);
                }
                ItemControl::None => {
                    state.settings.click_item(row.index);
                    invalidate(hwnd);
                }
            }
            return;
        }
    }
}

/// 「添加词库」子视图内的命中：返回按钮、导入按钮、各行的启用开关。
unsafe fn on_packs_click(
    hwnd: HWND,
    state: &mut WindowState,
    metrics: &SettingsMetrics,
    client: UiRect,
    x: i32,
    y: i32,
) {
    let layout = layout::packs_layout(metrics, client, state.packs.len());
    if layout::contains(layout.back, x, y) {
        state.settings.close_subview();
        invalidate(hwnd);
        return;
    }
    if layout::contains(layout.import, x, y) {
        import_zyct(hwnd, state);
        return;
    }
    // 先把命中拷贝出来再改 state：循环持有 `state.packs` 的不可变借用，不能同时
    // 传 `&mut state` 给 `toggle_pack`（包数量小，克隆代价可忽略）。
    let mut hit = None;
    for (index, pack) in layout.rows.iter().enumerate() {
        if layout::contains(pack.toggle, x, y) {
            if let Some(info) = state.packs.get(index) {
                if info.toggleable() {
                    hit = Some((info.id.clone(), info.name.clone(), info.enabled));
                }
            }
            break;
        }
    }
    if let Some((id, name, enabled)) = hit {
        toggle_pack(state, &id, &name, enabled);
        invalidate(hwnd);
    }
}

/// 「管理输入法」子视图内的命中：返回 / 打开系统输入法设置。
///
/// "打开系统输入法设置"（FR-043 / D-39）用 `ms-settings:keyboard` 协议 URI 交给系统
/// 设置应用，展示 TSF 视角的输入法列表。
fn on_manage_click(
    state: &mut WindowState,
    metrics: &SettingsMetrics,
    client: UiRect,
    x: i32,
    y: i32,
) {
    let layout_rows = layout::manage_layout(metrics, client);
    if layout::contains(layout_rows.back, x, y) {
        state.settings.close_subview();
        return;
    }
    if layout::contains(layout_rows.sys_settings, x, y) {
        state.hint = Some(match shell::open_uri("ms-settings:keyboard") {
            Ok(()) => "已打开系统输入法设置".to_owned(),
            Err(error) => error,
        });
    }
}

/// 「修复输入法」子视图内的命中：返回 / 一级修复 / 二级修复。
///
/// 先报告后动手（D-41）：检测结果在上方先行展示；一级修复是用户显式触发后才执行，
/// 执行完重扫并把逐项结果滚动进底部提示条。二级修复走提权子进程（UAC，D-40），
/// 完成后询问是否重启输入法进程（D-43）。
fn on_repair_click(
    hwnd: HWND,
    state: &mut WindowState,
    metrics: &SettingsMetrics,
    client: UiRect,
    x: i32,
    y: i32,
) {
    unsafe {
        let row_count = state
            .repair_scan
            .as_ref()
            .map_or(0, |scan| scan.summary_lines().len());
        let layout_rows = layout::repair_layout(metrics, client, row_count);
        if layout::contains(layout_rows.back, x, y) {
            state.settings.close_subview();
            return;
        }
        if layout::contains(layout_rows.l1, x, y) {
            let Some(scan) = &state.repair_scan else {
                return;
            };
            let outcomes = repair::apply_l1(
                scan,
                &repair_packs_dir(state.config_path.as_deref()),
                state.config_path.as_deref(),
                &repair_user_words_path(),
            );
            state.hint = Some(outcomes_summary(&outcomes));
            state.repair_scan = Some(scan_repair_now(state));
            invalidate(hwnd);
            return;
        }
        if layout::contains(layout_rows.l2, x, y) {
            run_repair_l2(hwnd, state);
            invalidate(hwnd);
        }
    }
}

/// 「检查更新」子视图内的命中（T-077 / FR-044）：返回 / 检查更新 / 应用更新。
///
/// 检查与应用按钮有共同的可点条件：在线更新已开启（P-03）、更新器存在、当前无任务在跑。
/// 未开启或更新器缺失时按钮不可点，界面在说明区给出原因——保证"关闭时零出站连接"可测。
fn on_update_click(
    hwnd: HWND,
    state: &mut WindowState,
    metrics: &SettingsMetrics,
    client: UiRect,
    x: i32,
    y: i32,
) {
    let layout_rows = layout::update_layout(metrics, client);
    if layout::contains(layout_rows.back, x, y) {
        state.settings.close_subview();
        return;
    }
    let can_run = can_run_updater(state);
    if layout::contains(layout_rows.check, x, y) {
        if can_run {
            unsafe { start_update_task(hwnd, state, UpdateKind::Check) };
        }
        return;
    }
    if layout::contains(layout_rows.apply, x, y) {
        // 应用更新是下载并覆盖本机安装的动作：二次确认（FR-044 / 验收 13.2）。
        if can_run && confirm_apply_update(hwnd) {
            unsafe { start_update_task(hwnd, state, UpdateKind::Apply) };
        }
    }
}

/// 检查/应用按钮是否可点：在线更新已开启（P-03）+ 更新器存在 + 无任务在跑。
///
/// `online_update` 未开启时**不 spawn 任何进程**，这是"关闭时零出站连接"的保证边界。
fn can_run_updater(state: &WindowState) -> bool {
    state.settings.online_update
        && updater::updater_exe_path().is_some()
        && state.update_busy.is_none()
}

/// 应用更新的二次确认（D-44 / FR-044）。
fn confirm_apply_update(hwnd: HWND) -> bool {
    registry::message_box(
        Some(hwnd),
        "将下载并应用可用更新。\n更新会替换本机程序文件，可能需要重启输入法，确定继续吗？",
        "应用更新",
        MB_YESNO | MB_ICONQUESTION | MB_DEFBUTTON2,
    )
}

/// 派发一个后台更新器任务：spawn 线程运行 `zhu-ye-updater`，完成后投递回执。
///
/// 联网只发生在更新器**子进程**内（D-44）；主线程不阻塞：结果经 mpsc 送还，
/// 由 `WM_UPDATER_DONE` 在消息循环里取回并重绘。
unsafe fn start_update_task(hwnd: HWND, state: &mut WindowState, kind: UpdateKind) {
    let Some(program) = updater::updater_exe_path() else {
        state.hint =
            Some("未找到更新器程序：请安装完整发行包（含 bin\\zhu-ye-updater.exe）".to_owned());
        invalidate(hwnd);
        return;
    };
    let args: Vec<&'static str> = kind.args().to_vec();
    state.update_seq = state.update_seq.wrapping_add(1);
    let seq = state.update_seq;
    let (tx, rx) = mpsc::channel();
    state.update_busy = Some(kind);
    state.update_result = None; // 新任务清掉旧结果，避免把上一次结果误读为新任务输出。
    state.update_rx = Some(rx);
    state.hint = Some(kind.busy_label().to_owned());
    // HWND 不是 Send，把它降为句柄数值传入线程，线程内重建（仅用于投递回执消息）。
    let hwnd_raw = hwnd.0 as isize;
    let _ = thread::spawn(move || {
        // 先送结果、后投递回执：回执到达时结果必然已在 channel 里（见 WM_UPDATER_DONE）。
        let result = updater::run(&program, &args);
        let _ = tx.send(UpdateOutcome { kind, result });
        let target = HWND(hwnd_raw as *mut core::ffi::c_void);
        unsafe {
            let _ = PostMessageW(
                Some(target),
                WM_UPDATER_DONE,
                WPARAM(seq as usize),
                LPARAM(0),
            );
        }
    });
    invalidate(hwnd);
}

/// 「版本与诊断信息」子视图内的命中：返回。
fn on_diagnostics_click(
    state: &mut WindowState,
    metrics: &SettingsMetrics,
    client: UiRect,
    x: i32,
    y: i32,
) {
    let row_count = state.diagnostics.as_ref().map_or(0, Vec::len);
    let layout_rows = layout::diagnostics_layout(metrics, client, row_count);
    if layout::contains(layout_rows.back, x, y) {
        state.settings.close_subview();
    }
}

/// 收集诊断信息（T-077 / FR-044）：版本、配置/数据/日志路径与已装包列表。
///
/// 全部本地读取，不 spawn 更新器、不发网络请求（D-44）。已装包清单复用
/// `list_packs_now` 的本地盘面（清单 + 配置 + exe 目录），与「添加词库」同源。
fn build_diagnostics(config_path: Option<&std::path::Path>) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!(
        "竹叶输入法 v{}（引擎与设置窗口同源发布）",
        zhu_ye_core::core_version()
    ));
    lines.push("联机更新由独立更新器进程完成（D-44），本窗口自身不联网。".to_owned());
    match config_path {
        Some(path) => lines.push(format!("配置文件：{}", path.display())),
        None => lines.push("配置文件：（未找到，APPDATA 未设置）".to_owned()),
    }
    match config::data_dir() {
        Some(dir) => lines.push(format!("数据目录：{}", dir.display())),
        None => lines.push("数据目录：（未找到，APPDATA 未设置）".to_owned()),
    }
    match config::product_log_dir() {
        Some(dir) => lines.push(format!(
            "日志目录：{}（默认记录错误，级别见 config.json log_level）",
            dir.display()
        )),
        None => lines.push("日志目录：（未找到，LOCALAPPDATA 未设置）".to_owned()),
    }
    let packs = list_packs_now(config_path);
    if packs.is_empty() {
        lines.push("已装包：（无）".to_owned());
    } else {
        lines.push(format!("已装包（{}）：", packs.len()));
        for pack in packs {
            let flag = if pack.enabled { "已启用" } else { "停用" };
            lines.push(format!("  {}（{}，{flag}）", pack.name, pack.id));
        }
    }
    lines
}

/// 汇总一级修复的逐项结果为底部提示（每项一行，行数有限时截断）。
fn outcomes_summary(outcomes: &[L1Outcome]) -> String {
    if outcomes.is_empty() {
        return "检测无异常，无需修复".to_owned();
    }
    outcomes
        .iter()
        .map(|outcome| match outcome {
            L1Outcome::Done(text) => format!("✓ {text}"),
            L1Outcome::Failed(text) => format!("✗ {text}"),
        })
        .collect::<Vec<_>>()
        .join("；")
}

/// 执行一级检测（进入「修复输入法」子视图与修复后重扫共用）。
fn scan_repair_now(state: &mut WindowState) -> RepairScan {
    repair::scan_l1(
        &repair_packs_dir(state.config_path.as_deref()),
        state.config_path.as_deref(),
        &repair_user_words_path(),
        &shell::exe_dir().join(DICTIONARY_FILE_NAME),
    )
}

/// 领域包目录：`%APPDATA%\zhu-ye-ime\packs`（与清单/导入同口径）。
fn repair_packs_dir(config_path: Option<&std::path::Path>) -> std::path::PathBuf {
    let _ = config_path; // 清单另有用途；检测直接以数据目录为准。
    config::data_dir()
        .map(|dir| dir.join("packs"))
        .unwrap_or_else(|| std::path::PathBuf::from("packs"))
}

/// 用户词库路径：`%APPDATA%\zhu-ye-ime\user_words.json`（与 TSF 侧同口径）。
fn repair_user_words_path() -> std::path::PathBuf {
    config::data_dir()
        .map(|dir| dir.join("user_words.json"))
        .unwrap_or_else(|| std::path::PathBuf::from("user_words.json"))
}

/// 恢复状态栏：重启输入法进程（ctfmon）重建语言栏；执行前弹确认（D-43）。
///
/// 语言栏由 ctfmon 托管，终止后系统按需自动重新加载（开放实现项，见 §6.4）。
fn restore_lang_bar(hwnd: HWND, state: &mut WindowState) {
    let confirmed = registry::message_box(
        Some(hwnd),
        "将重启输入法进程（ctfmon）以重建语言栏。\n当前输入法会短暂中断，确定继续吗？",
        "恢复状态栏",
        MB_YESNO | MB_ICONQUESTION | MB_DEFBUTTON2,
    );
    if !confirmed {
        state.hint = Some("已取消".to_owned());
        return;
    }
    state.hint = Some(match registry::restart_ctfmon() {
        Ok(()) => "输入法进程已重启，系统将自动重新加载语言栏".to_owned(),
        Err(error) => error,
    });
}

/// 二级修复：经 `runas` 拉起 `--repair-registry` 子命令（UAC，D-40），等待退出码。
///
/// 用 `ShellExecuteExW`+`SEE_MASK_NOCLOSEPROCESS` 拿到进程句柄，等待结束后读退出码
/// （`ShellExecuteW` 拿不到退出码，这是必须升级为 Ex 的原因）。成功则重探注册状态并
/// 询问是否重启输入法进程（D-43）；失败说明原因。
fn run_repair_l2(hwnd: HWND, state: &mut WindowState) {
    let executable = shell::exe_dir().join("zhu-ye-settings.exe");
    let verb = to_utf16("runas");
    let file = to_utf16(&executable.to_string_lossy());
    let parameters = to_utf16("--repair-registry");
    let mut info = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS,
        hwnd,
        lpVerb: PCWSTR(verb.as_ptr()),
        lpFile: PCWSTR(file.as_ptr()),
        lpParameters: PCWSTR(parameters.as_ptr()),
        nShow: SW_SHOWNORMAL.0,
        ..Default::default()
    };
    if unsafe { ShellExecuteExW(&mut info) }.is_err() {
        state.hint = Some(format!(
            "无法启动修复程序（拒绝提权或系统限制）：{}",
            std::io::Error::last_os_error()
        ));
        return;
    }
    let process = info.hProcess;
    unsafe {
        let _ = WaitForSingleObject(process, INFINITE);
    }
    let mut exit_code = 0u32;
    if unsafe { GetExitCodeProcess(process, &mut exit_code) }.is_err() {
        let _ = unsafe { CloseHandle(process) };
        state.hint = Some("读取修复结果失败，请重试".to_owned());
        return;
    }
    let _ = unsafe { CloseHandle(process) };
    if exit_code != 0 {
        state.hint = Some(format!("二级修复未完成（退出码 {exit_code}），可重试"));
        return;
    }
    // 先报告后动手的闭环：重探注册状态后再问是否重启输入法进程。
    state.manage = Some(evaluate_registration(&registry::probe_registration()));
    let confirmed = registry::message_box(
        Some(hwnd),
        "注册表已重建。\n是否立即重启输入法进程（ctfmon）使新注册生效？",
        "修复输入法",
        MB_YESNO | MB_ICONQUESTION | MB_DEFBUTTON2,
    );
    if !confirmed {
        state.hint = Some("注册表已重建；重启输入法后生效".to_owned());
        return;
    }
    state.hint = Some(match registry::restart_ctfmon() {
        Ok(()) => "注册表已重建，输入法进程已重启".to_owned(),
        Err(error) => error,
    });
}

/// 打开工具箱面板。
///
/// 关键一步是把焦点还给"打开设置前的那个窗口"：面板本身不激活，目标应用保持前台，
/// 面板里的 `SendInput` 才落得到用户真正在打字的应用上（FR-040 的"选中即上屏"）。
unsafe fn open_panel(hwnd: HWND, state: &mut WindowState, kind: crate::panel::PanelKind) {
    if state.panel.is_some() {
        return;
    }
    match PanelWindow::open(PanelView::new(kind), hwnd) {
        Ok(panel) => {
            state.panel = Some(panel);
            state.hint = Some(format!(
                "{} 已打开：点击即上屏，点“返回设置”关闭",
                kind.title()
            ));
            let previous = state.previous_foreground;
            if !previous.is_invalid() {
                unsafe {
                    let _ = SetForegroundWindow(previous);
                }
            }
        }
        Err(error) => state.hint = Some(format!("打开面板失败：{error}")),
    }
    invalidate(hwnd);
}

/// 执行「更多设置」的打开动作。
///
/// 配置与日志可能尚未生成（首次改动设置前没有 `config.json`；产品日志目录由
/// TSF 首次写日志时创建），因此先判断存在性再调用系统打开，避免弹出系统的
/// "找不到文件"对话框；日志目录则先幂等创建（设计 §5「先判断/创建再打开」，
/// T-075 ⑤ 既有原则）。
fn open_target(state: &mut WindowState, target: OpenTarget) {
    state.hint = Some(match shell::resolve_target(target) {
        Err(error) => error,
        Ok(path) => match target {
            OpenTarget::LogDir => open_log_dir(&path),
            _ if path.exists() => match shell::open_path(&path) {
                Ok(()) => format!("已打开 {}", path.display()),
                Err(error) => error,
            },
            OpenTarget::ConfigFile => "配置文件尚未生成：改动任一设置后即会写入".to_owned(),
            OpenTarget::DataDir => format!("目录不存在：{}", path.display()),
        },
    });
}

/// 打开日志目录：不存在时先创建（创建失败仅提示，不弹系统"找不到文件"框）。
fn open_log_dir(path: &std::path::Path) -> String {
    if !path.exists() && std::fs::create_dir_all(path).is_err() {
        return format!("无法创建日志目录：{}", path.display());
    }
    match shell::open_path(path) {
        Ok(()) => format!("已打开 {}", path.display()),
        Err(error) => error,
    }
}

/// 应用主题选择并持久化。
fn apply_theme(state: &mut WindowState, choice: ThemeChoice) {
    // 自定义主题：立即解析进 `custom_theme`（缺键回退当前深浅预设），窗口即时重绘；
    // 候选窗与设置窗同源，TSF DLL 下次装配时按配置的值读取同一文件。
    state.custom_theme =
        resolve_custom_theme(state.theme_kind, &choice, state.config_path.as_deref());
    state.settings.theme = choice.clone();
    state.hint = Some(match &state.config_path {
        Some(path) => match config::save_theme(path, choice.clone()) {
            // 主题是装配项：由 TSF DLL 在下次装配时读取（P-12）。
            Ok(()) => {
                let base = match &choice {
                    ThemeChoice::Custom(name) => match &state.custom_theme {
                        Some((_, covered)) => {
                            format!("已保存主题：自定义（{name}），应用 {covered}/15 键；重启输入法后候选窗生效")
                        }
                        None => format!(
                            "已保存主题：自定义（{name}）；未找到可用主题文件，回退当前深浅预设"
                        ),
                    },
                    _ => format!(
                        "已保存主题：{}，重启输入法后候选窗生效",
                        theme_label(&choice)
                    ),
                };
                base
            }
            Err(error) => format!("保存失败：{error}"),
        },
        None => "未找到配置目录（APPDATA 未设置），本次选择不会保留".to_owned(),
    });
}

fn theme_label(choice: &ThemeChoice) -> std::borrow::Cow<'static, str> {
    match choice {
        ThemeChoice::Light => std::borrow::Cow::Borrowed("浅色"),
        ThemeChoice::Dark => std::borrow::Cow::Borrowed("深色"),
        ThemeChoice::Custom(name) => std::borrow::Cow::Owned(format!("自定义（{name}）")),
    }
}

const fn mode_label(mode: ModeChoice) -> &'static str {
    match mode {
        ModeChoice::Chinese => "中文",
        ModeChoice::English => "英文",
    }
}

/// 二选一控件命中后的分发：主题、默认中英模式（D-32）、在线更新开关（P-03）
/// 或候选框拼音行开关（T-127）。
fn apply_chip(state: &mut WindowState, chip: &Chip) {
    match &chip.value {
        ChipValue::Theme(choice) => apply_theme(state, choice.clone()),
        ChipValue::Mode(mode) => apply_mode(state, *mode),
        ChipValue::OnlineUpdate(on) => apply_online_update(state, *on),
        ChipValue::CandidatePin(show) => apply_candidate_show_pin(state, *show),
    }
}

/// 应用候选框拼音行开关并持久化（T-127）。
///
/// 装配项：写进 `config.json` 的 `candidate_show_pin`，由 TSF DLL 下次装配读取
/// （候选窗随输入法实例创建），设置窗口不回传"立即生效"的假象——与主题同口径。
fn apply_candidate_show_pin(state: &mut WindowState, show: bool) {
    state.settings.candidate_show_pin = show;
    state.hint = Some(match &state.config_path {
        Some(path) => match config::save_candidate_show_pin(path, show) {
            Ok(()) => {
                if show {
                    "已开启候选拼音：候选框显示汉字上方的拼音及声调，重启输入法后生效".to_owned()
                } else {
                    "已关闭候选拼音：候选框不再显示拼音及声调，重启输入法后生效".to_owned()
                }
            }
            Err(error) => format!("保存失败：{error}"),
        },
        None => "未找到配置目录（APPDATA 未设置），本次选择不会保留".to_owned(),
    });
}

/// 应用在线更新开关并持久化（T-077 / FR-044，P-03）。
///
/// 勾选即写 `config.json`（运行时开关，设置状态三分）：更新器进程下次读取；
/// 写入失败时如实提示，不假装已开启。
fn apply_online_update(state: &mut WindowState, on: bool) {
    state.settings.online_update = on;
    state.hint = Some(match &state.config_path {
        Some(path) => match config::save_online_update(path, on) {
            Ok(()) => {
                if on {
                    "已开启在线更新：检查/应用将经独立更新器进程联网（D-44）".to_owned()
                } else {
                    "已关闭在线更新：更新器不再发起任何网络请求".to_owned()
                }
            }
            Err(error) => format!("保存失败：{error}"),
        },
        None => "未找到配置目录（APPDATA 未设置），本次选择不会保留".to_owned(),
    });
}

/// 应用默认中英模式选择并持久化（D-32 装配项）。
fn apply_mode(state: &mut WindowState, mode: ModeChoice) {
    state.settings.default_mode = mode;
    state.hint = Some(match &state.config_path {
        Some(path) => match config::save_mode(path, mode) {
            // 装配项：由 TSF DLL 在下次装配（Activate）时作为新会话起始模式读取。
            Ok(()) => format!(
                "已保存默认模式：{}，重启输入法后新会话生效",
                mode_label(mode)
            ),
            Err(error) => format!("保存失败：{error}"),
        },
        None => "未找到配置目录（APPDATA 未设置），本次选择不会保留".to_owned(),
    });
}

/// 从磁盘重扫包清单（进入子视图、勾选、导入后调用）。
fn list_packs_now(config_path: Option<&std::path::Path>) -> Vec<PackInfo> {
    let Some(path) = config_path else {
        return Vec::new();
    };
    let Some(data_dir) = config::data_dir() else {
        return Vec::new();
    };
    let packs_sub = inventory::packs_dir(&data_dir);
    let (config, _) = config::load(path);
    let (record, _) = installed::load(&packs_sub);
    list_packs(&config, &shell::exe_dir(), &packs_sub, &record)
}

/// 勾选/取消勾选一个领域包：只改 `enabled_packs` 并原子保存，随后刷新列表。
///
/// 包启停是装配项：写进 `config.json` 后由 TSF DLL 下次装配读取，不回传"立即生效"
/// 的假象（与主题同口径）。
fn toggle_pack(state: &mut WindowState, id: &str, name: &str, enabled: bool) {
    let Some(config_path) = &state.config_path else {
        state.hint = Some("未找到配置目录（APPDATA 未设置），本次勾选不会保留".to_owned());
        return;
    };
    let (mut config, _) = config::load(config_path);
    if enabled {
        config.enabled_packs.retain(|pack| pack != id);
    } else if !config.enabled_packs.iter().any(|pack| pack == id) {
        config.enabled_packs.push(id.to_owned());
    }
    state.hint = Some(
        match config::save_packs(config_path, &config.enabled_packs) {
            Ok(()) => {
                state.packs = list_packs_now(state.config_path.as_deref());
                if enabled {
                    format!("已停用「{name}」，重启输入法后生效")
                } else {
                    format!("已启用「{name}」，重启输入法后生效")
                }
            }
            Err(error) => format!("保存失败：{error}"),
        },
    );
}

/// 导入本地 `.zyct`（FR-042）：选文件 → 完整校验 → 复制进 `packs/` → 记入清单。
///
/// 校验用 `DictionaryFile::open`（魔数、头部、内容 SHA-256、分区布局逐项查），任一失败
/// 就直接拒绝且不动磁盘；复制成功后写 `installed.json`（source: Import，无版本），并
/// 在界面标注"不验签、不参与在线更新的签名信任链"（D-38）。
unsafe fn import_zyct(hwnd: HWND, state: &mut WindowState) {
    let Some(source) = pick_zyct_file(hwnd) else {
        return; // 用户取消对话框，不打扰。
    };
    if let Err(reason) = zhu_ye_core::DictionaryFile::open(&source) {
        state.hint = Some(format!("导入失败：{reason}（文件未改动）"));
        invalidate(hwnd);
        return;
    }
    let Some(data_dir) = config::data_dir() else {
        state.hint = Some("未找到配置目录（APPDATA 未设置），无法导入".to_owned());
        invalidate(hwnd);
        return;
    };
    let packs_sub = inventory::packs_dir(&data_dir);
    let stem = source
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| "import".to_owned());
    let target = packs_sub.join(format!("{stem}.zyct"));
    if let Err(error) = std::fs::create_dir_all(&packs_sub)
        .and_then(|()| std::fs::copy(&source, &target))
        .map_err(|error| error.to_string())
    {
        state.hint = Some(format!("导入失败：无法复制文件：{error}"));
        invalidate(hwnd);
        return;
    }
    let sha256 = match zhu_ye_core::sha256_file(&target) {
        Ok(sha) => sha,
        Err(error) => {
            state.hint = Some(format!("导入失败：计算校验值出错：{error}"));
            invalidate(hwnd);
            return;
        }
    };
    let (mut record, _) = installed::load(&packs_sub);
    record.upsert(InstalledPack {
        id: stem.clone(),
        version: None,
        sha256,
        installed_at: zhu_ye_core::unix_now(),
        source: PackSource::Import,
    });
    if let Err(error) = installed::save(&packs_sub, &record) {
        state.hint = Some(format!(
            "导入完成但清单记录失败：{error}（下次进入设置会重新列出）"
        ));
        state.packs = list_packs_now(state.config_path.as_deref());
        invalidate(hwnd);
        return;
    }
    state.packs = list_packs_now(state.config_path.as_deref());
    state.hint = Some(format!(
        "已导入「{stem}」（本地导入，不参与签名校验），勾选启用后重启生效"
    ));
    invalidate(hwnd);
}

/// 弹出系统打开对话框挑选 `.zyct`；取消返回 `None`（调用方保持静默）。
fn pick_zyct_file(hwnd: HWND) -> Option<PathBuf> {
    unsafe {
        let filter = to_utf16("词典包 (*.zyct)\0*.zyct\0全部文件 (*.*)\0*.*\0");
        let mut file_buffer = [0u16; 1024];
        let mut ofn = OPENFILENAMEW {
            lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
            hwndOwner: hwnd,
            lpstrFilter: PCWSTR(filter.as_ptr()),
            lpstrFile: PWSTR(file_buffer.as_mut_ptr()),
            nMaxFile: file_buffer.len() as u32,
            Flags: OFN_FILEMUSTEXIST | OFN_HIDEREADONLY | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR,
            ..Default::default()
        };
        if !GetOpenFileNameW(&mut ofn).as_bool() {
            return None;
        }
        let length = file_buffer.iter().position(|unit| *unit == 0).unwrap_or(0);
        Some(PathBuf::from(String::from_utf16_lossy(
            &file_buffer[..length],
        )))
    }
}

// ---------------------------------------------------------------------------
// T-088 / FR-048：用户词表、通讯录、自定义主题三子视图
// ---------------------------------------------------------------------------

/// themes 目录：`%APPDATA%\zhu-ye-ime\themes`。
fn themes_dir(config_path: Option<&std::path::Path>) -> Option<PathBuf> {
    let data_dir = config_path?.parent()?;
    Some(data_dir.join("themes"))
}

/// 解析自定义主题文件为设置窗配色：文件缺失/解析失败回退系统深浅预设（`None`）；
/// 高对比度由系统接管（D-31），不在本函数处理（调用方在 `HighContrast` 时跳过）。
fn resolve_custom_theme(
    kind: UiThemeKind,
    theme: &ThemeChoice,
    config_path: Option<&std::path::Path>,
) -> Option<(SettingsTheme, usize)> {
    let ThemeChoice::Custom(name) = theme else {
        return None;
    };
    // 主题 id 校验见 zhu_ye_core::is_safe_theme_name：配置里手写的 `../x` 之类一律
    // 视为"无此主题"，回退系统深浅预设（防目录逃逸）。
    if !zhu_ye_core::is_safe_theme_name(name) {
        return None;
    }
    let file = zhu_ye_core::load_theme_file(&themes_dir(config_path)?.join(format!("{name}.json")))
        .ok()?;
    Some(settings_theme(kind).with_theme_file(&file))
}

/// 扫描 themes 目录列出 `*.json` 主题（按显示名排序）；目录不存在或不可读返回空。
fn list_theme_files(config_path: Option<&std::path::Path>) -> Vec<ThemeInfo> {
    let Some(dir) = themes_dir(config_path) else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut infos = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let Some(id) = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .map(str::to_owned)
        else {
            continue;
        };
        let file = zhu_ye_core::load_theme_file(&path).ok();
        let display_name = file
            .as_ref()
            .and_then(|theme| theme.name.clone())
            .unwrap_or_else(|| id.clone());
        infos.push(ThemeInfo {
            id,
            display_name,
            file,
        });
    }
    infos.sort_by(|a, b| a.display_name.cmp(&b.display_name));
    infos
}

/// 装载已登记的 .vcf 路径列表（来自 `config.contact_vcards`；无配置时为空）。
fn load_contacts(config_path: Option<&std::path::Path>) -> Vec<PathBuf> {
    match config_path {
        Some(path) => config::load_contact_vcards(path).0,
        None => Vec::new(),
    }
}

/// 追加一个 .vcf 路径（同路径不重复，按规范化路径比较）；保存前重读配置（S-8）。
fn append_contact_vcard(
    config_path: Option<&std::path::Path>,
    vcard: &std::path::Path,
) -> Result<bool, String> {
    let Some(path) = config_path else {
        return Err("未找到配置目录（APPDATA 未设置），无法保存".to_owned());
    };
    let mut vcards = config::load_contact_vcards(path).0;
    let canonical = vcard.to_path_buf();
    let existed = vcards.iter().any(|existing| {
        if let (Ok(a), Ok(b)) = (existing.canonicalize(), canonical.canonicalize()) {
            a == b
        } else {
            existing == &canonical
        }
    });
    if existed {
        return Ok(false);
    }
    vcards.push(canonical);
    config::save_contact_vcards(path, &vcards)?;
    Ok(true)
}

/// 弹出系统保存对话框挑导出路径（默认 `user_words_YYYYMMDD.json`，覆盖前确认）；
/// 取消返回 `None`（调用方保持静默）。
unsafe fn pick_user_words_save_file(hwnd: HWND) -> Option<PathBuf> {
    let filter = to_utf16("用户词表 (*.json)\0*.json\0全部文件 (*.*)\0*.*\0");
    let mut file_buffer = [0u16; 1024];
    let default_name = to_utf16(&format!("user_words_{}.json", zhu_ye_core::today_compact()));
    let copied = default_name.len().min(file_buffer.len() - 1);
    file_buffer[..copied].copy_from_slice(&default_name[..copied]);
    let mut ofn = OPENFILENAMEW {
        lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
        hwndOwner: hwnd,
        lpstrFilter: PCWSTR(filter.as_ptr()),
        lpstrFile: PWSTR(file_buffer.as_mut_ptr()),
        nMaxFile: file_buffer.len() as u32,
        Flags: OFN_OVERWRITEPROMPT | OFN_PATHMUSTEXIST | OFN_HIDEREADONLY | OFN_NOCHANGEDIR,
        ..Default::default()
    };
    if !GetSaveFileNameW(&mut ofn).as_bool() {
        return None;
    }
    let length = file_buffer.iter().position(|unit| *unit == 0).unwrap_or(0);
    Some(PathBuf::from(String::from_utf16_lossy(
        &file_buffer[..length],
    )))
}

/// 通用打开对话框挑选 `.json` / `.vcf` / `.zyct` 文件；取消返回 `None`。
unsafe fn pick_any_file(hwnd: HWND, filter: &str) -> Option<PathBuf> {
    let filter = to_utf16(filter);
    let mut file_buffer = [0u16; 1024];
    let mut ofn = OPENFILENAMEW {
        lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
        hwndOwner: hwnd,
        lpstrFilter: PCWSTR(filter.as_ptr()),
        lpstrFile: PWSTR(file_buffer.as_mut_ptr()),
        nMaxFile: file_buffer.len() as u32,
        Flags: OFN_FILEMUSTEXIST | OFN_HIDEREADONLY | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR,
        ..Default::default()
    };
    if !GetOpenFileNameW(&mut ofn).as_bool() {
        return None;
    }
    let length = file_buffer.iter().position(|unit| *unit == 0).unwrap_or(0);
    Some(PathBuf::from(String::from_utf16_lossy(
        &file_buffer[..length],
    )))
}

/// 导出用户词表：保存对话框 → 序列化 v1 交换 JSON → 写入目标路径。
///
/// 先序列化成功再写文件（写失败提示，不影响磁盘上的词表）。
unsafe fn export_user_words_now(hwnd: HWND, state: &mut WindowState) {
    let Some(target) = pick_user_words_save_file(hwnd) else {
        return; // 用户取消，不打扰。
    };
    let store = zhu_ye_core::UserDictStore::new(repair_user_words_path());
    let dictionary = match store.load() {
        Ok(dictionary) => dictionary,
        Err(error) => {
            state.user_words_message = Some(format!("导出失败：{error}"));
            invalidate(hwnd);
            return;
        }
    };
    let text = match zhu_ye_core::export_user_words(&dictionary) {
        Ok(text) => text,
        Err(error) => {
            state.user_words_message = Some(error);
            invalidate(hwnd);
            return;
        }
    };
    if let Err(error) = std::fs::write(&target, text) {
        state.user_words_message =
            Some(format!("导出失败：无法写入 {}：{error}", target.display()));
        invalidate(hwnd);
        return;
    }
    state.user_words_message = Some(format!(
        "已导出 {} 条到 {}",
        dictionary.len(),
        target.display()
    ));
    invalidate(hwnd);
}

/// 导入用户词表：选 JSON → 整体校验（格式/版本/逐条合法性在解析时过滤）→ 内存合并
/// → 原子落盘。先校验后写：解析失败不动磁盘；保存失败也保持原词表。
unsafe fn import_user_words_now(hwnd: HWND, state: &mut WindowState) {
    let Some(source) = pick_any_file(hwnd, "用户词表 (*.json)\0*.json\0全部文件 (*.*)\0*.*\0")
    else {
        return; // 用户取消，不打扰。
    };
    let text = match std::fs::read_to_string(&source) {
        Ok(text) => text,
        Err(error) => {
            state.user_words_message =
                Some(format!("导入失败：无法读取 {}：{error}", source.display()));
            invalidate(hwnd);
            return;
        }
    };
    let items = match zhu_ye_core::parse_exchange_file(&text) {
        Ok(file) => file.items,
        Err(reason) => {
            state.user_words_message = Some(format!("导入失败：{reason}（未改动本地词表）"));
            invalidate(hwnd);
            return;
        }
    };
    let store = zhu_ye_core::UserDictStore::new(repair_user_words_path());
    let mut dictionary = match store.load() {
        Ok(dictionary) => dictionary,
        Err(error) => {
            state.user_words_message = Some(format!("导入失败：{error}"));
            invalidate(hwnd);
            return;
        }
    };
    let added = zhu_ye_core::merge_exchange_items(&mut dictionary, &items);
    // 已整体校验（解析）+ 内存合并；保存是原子写（.tmp + rename），失败不动原文件。
    match store.save(&dictionary) {
        Ok(()) => {
            state.user_words_message = Some(format!(
                "导入完成：新增 {added} 条，本地用户词表共 {} 条",
                dictionary.len()
            ));
        }
        Err(error) => {
            state.user_words_message = Some(format!("导入失败：{error}（未改动本地词表）"));
        }
    }
    invalidate(hwnd);
}

/// 通讯录 .vcf 界面化导入：选文件 → 解析 → 确认 → 复制进数据目录 + 追加配置。
///
/// 任何失败都发生在写入前（解析失败/确认取消），不会留下半截状态；复制失败同样不
/// 碰配置。已登记的同路径不重复追加（S-8 保存前重读）。
unsafe fn import_vcf_now(hwnd: HWND, state: &mut WindowState) {
    let Some(source) = pick_any_file(hwnd, "通讯录 (*.vcf)\0*.vcf\0全部文件 (*.*)\0*.*\0")
    else {
        return; // 用户取消，不打扰。
    };
    let text = match std::fs::read_to_string(&source) {
        Ok(text) => text,
        Err(error) => {
            state.hint = Some(format!("导入失败：无法读取 {}：{error}", source.display()));
            invalidate(hwnd);
            return;
        }
    };
    let contacts = match zhu_ye_core::parse_vcard(&text) {
        Ok(contacts) => contacts,
        Err(error) => {
            state.hint = Some(format!("导入失败：不是有效的 vCard：{error}"));
            invalidate(hwnd);
            return;
        }
    };
    if contacts.is_empty() {
        state.hint = Some("导入失败：文件中没有联系人记录".to_owned());
        invalidate(hwnd);
        return;
    }
    let confirmed = registry::message_box(
        Some(hwnd),
        &format!(
            "检测到 {} 条联系人，导入后将生成对应的联系人拼音候选。\n是否继续？",
            contacts.len()
        ),
        "通讯录导入",
        MB_YESNO | MB_ICONQUESTION | MB_DEFBUTTON2,
    );
    if !confirmed {
        return; // 用户取消，不打扰。
    }
    let Some(data_dir) = config::data_dir() else {
        state.hint = Some("未找到配置目录（APPDATA 未设置），无法导入".to_owned());
        invalidate(hwnd);
        return;
    };
    let contacts_dir = data_dir.join("contacts");
    if let Err(error) = std::fs::create_dir_all(&contacts_dir) {
        state.hint = Some(format!("导入失败：无法创建通讯录目录：{error}"));
        invalidate(hwnd);
        return;
    }
    let stem = source
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| "contacts".to_owned());
    let extension = source
        .extension()
        .map(|ext| ext.to_string_lossy().into_owned())
        .unwrap_or_else(|| "vcf".to_owned());
    // 同名文件冲突时追加序号，避免覆盖既有导入。
    let mut target = contacts_dir.join(format!("{stem}.{extension}"));
    let mut serial = 1;
    while target.exists() {
        target = contacts_dir.join(format!("{stem}({serial}).{extension}"));
        serial += 1;
    }
    if let Err(error) = std::fs::copy(&source, &target) {
        state.hint = Some(format!("导入失败：无法复制文件：{error}"));
        invalidate(hwnd);
        return;
    }
    match append_contact_vcard(state.config_path.as_deref(), &target) {
        Ok(true) => {
            state.hint = Some(format!(
                "已导入 {} 条联系人（{}），重启输入法后生效",
                contacts.len(),
                target.display()
            ));
        }
        Ok(false) => {
            state.hint = Some(format!(
                "该 .vcf 已在列表中（{}），未重复登记",
                target.display()
            ));
        }
        Err(error) => {
            state.hint = Some(format!("已复制文件但登记失败：{error}"));
        }
    }
    state.contacts = load_contacts(state.config_path.as_deref());
    invalidate(hwnd);
}

/// 「用户词表」子视图内命中：返回 / 导出 / 导入。
unsafe fn on_user_words_click(
    hwnd: HWND,
    state: &mut WindowState,
    metrics: &SettingsMetrics,
    client: UiRect,
    x: i32,
    y: i32,
) {
    let layout = layout::userwords_layout(metrics, client);
    if layout::contains(layout.back, x, y) {
        state.settings.close_subview();
        invalidate(hwnd);
        return;
    }
    if layout::contains(layout.export, x, y) {
        export_user_words_now(hwnd, state);
        return;
    }
    if layout::contains(layout.import, x, y) {
        import_user_words_now(hwnd, state);
    }
}

/// 「通讯录」子视图内命中：返回 / 导入 .vcf。
unsafe fn on_contacts_click(
    hwnd: HWND,
    state: &mut WindowState,
    metrics: &SettingsMetrics,
    client: UiRect,
    x: i32,
    y: i32,
) {
    let layout = layout::contacts_layout(metrics, client, state.contacts.len());
    if layout::contains(layout.back, x, y) {
        state.settings.close_subview();
        invalidate(hwnd);
        return;
    }
    if layout::contains(layout.import, x, y) {
        import_vcf_now(hwnd, state);
    }
}

/// 「自定义主题」子视图内命中：返回 / 选择主题行（损坏行给出提示保持当前主题）。
unsafe fn on_themes_click(
    hwnd: HWND,
    state: &mut WindowState,
    metrics: &SettingsMetrics,
    client: UiRect,
    x: i32,
    y: i32,
) {
    let layout = layout::themes_layout(metrics, client, state.theme_files.len());
    if layout::contains(layout.back, x, y) {
        state.settings.close_subview();
        invalidate(hwnd);
        return;
    }
    for (index, row) in layout.rows.iter().enumerate() {
        if !layout::contains(*row, x, y) {
            continue;
        }
        let Some(info) = state.theme_files.get(index) else {
            break;
        };
        if state.theme_kind == UiThemeKind::HighContrast {
            state.hint =
                Some("系统处于高对比度模式，由系统接管配色（D-31），自定义主题不生效".to_owned());
            invalidate(hwnd);
            return;
        }
        if info.file.is_none() {
            state.hint = Some(format!(
                "「{}」文件损坏或版本过高，无法应用（保持当前主题）",
                info.display_name
            ));
            invalidate(hwnd);
            return;
        }
        state.hint = None;
        apply_theme(state, ThemeChoice::Custom(info.id.clone()));
        invalidate(hwnd);
        return;
    }
}

/// 绘制「用户词表」子视图：说明、结果消息、导出/导入/返回按钮。
unsafe fn draw_user_words(
    hdc: HDC,
    state: &mut WindowState,
    theme: SettingsTheme,
    metrics: &SettingsMetrics,
    client: UiRect,
) {
    let layout = layout::userwords_layout(metrics, client);
    draw_text(
        hdc,
        "导出：把本地用户词表按词频导出为 JSON（zhu-ye-user-words v1 交换格式）",
        layout.info,
        theme.item_text,
        state.fonts.small,
        DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
    );
    let second = UiRect {
        left: layout.info.left,
        top: layout.info.top + layout.info.height() / 3,
        right: layout.info.right,
        bottom: layout.info.bottom,
    };
    draw_text(
        hdc,
        "导入：先整体校验（格式/版本），合并在内存完成（同拼音同词取较大词频），通过后才原子写入",
        second,
        theme.secondary_text,
        state.fonts.small,
        DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
    );
    draw_button(
        hdc,
        state.fonts.small,
        theme,
        metrics,
        layout.export,
        "导出用户词表…",
    );
    draw_button(
        hdc,
        state.fonts.small,
        theme,
        metrics,
        layout.import,
        "从文件导入…",
    );
    draw_button(
        hdc,
        state.fonts.small,
        theme,
        metrics,
        layout.back,
        "← 返回常用设置",
    );
    if let Some(message) = &state.user_words_message {
        let result_rect = UiRect {
            left: layout.result.left + metrics.gap,
            top: layout.result.top,
            right: layout.result.right,
            bottom: layout.result.bottom,
        };
        draw_text(
            hdc,
            message,
            result_rect,
            theme.item_text,
            state.fonts.small,
            DT_LEFT | DT_WORDBREAK | DT_NOPREFIX,
        );
    }
}

/// 绘制「通讯录」子视图：说明、已登记 .vcf 列表、导入/返回按钮。
unsafe fn draw_contacts(
    hdc: HDC,
    state: &mut WindowState,
    theme: SettingsTheme,
    metrics: &SettingsMetrics,
    client: UiRect,
) {
    let layout = layout::contacts_layout(metrics, client, state.contacts.len());
    draw_text(
        hdc,
        "导入 .vcf 通讯录文件，生成联系人拼音候选（TSF 下次装配时读取，需重启输入法）",
        layout.info,
        theme.item_text,
        state.fonts.small,
        DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
    );
    let empty_note = UiRect {
        left: layout.info.left,
        top: layout.info.top + layout.info.height() / 3,
        right: layout.info.right,
        bottom: layout.info.bottom,
    };
    if state.contacts.is_empty() {
        draw_text(
            hdc,
            "尚未登记任何 .vcf（导入后路径记录在 config.json 的 contact_vcards）",
            empty_note,
            theme.placeholder_text,
            state.fonts.small,
            DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
        );
    } else {
        draw_text(
            hdc,
            &format!("已登记 {} 个 .vcf：", state.contacts.len()),
            empty_note,
            theme.secondary_text,
            state.fonts.small,
            DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
        );
        for (index, row) in layout.rows.iter().enumerate() {
            let Some(vcard) = state.contacts.get(index) else {
                break;
            };
            let rect = UiRect {
                left: row.left + metrics.gap,
                top: row.top,
                right: row.right,
                bottom: row.bottom,
            };
            draw_text(
                hdc,
                &vcard.to_string_lossy(),
                rect,
                theme.item_text,
                state.fonts.small,
                DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
            );
        }
        if state.contacts.len() > layout.rows.len() {
            let overflow = UiRect {
                left: layout.info.left,
                top: layout
                    .rows
                    .last()
                    .map(|row| row.bottom)
                    .unwrap_or(layout.info.bottom),
                right: layout.info.right,
                bottom: layout.info.bottom + layout.info.height() / 3,
            };
            draw_text(
                hdc,
                &format!(
                    "…另有 {} 个未显示",
                    state.contacts.len() - layout.rows.len()
                ),
                overflow,
                theme.secondary_text,
                state.fonts.small,
                DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
            );
        }
    }
    draw_button(
        hdc,
        state.fonts.small,
        theme,
        metrics,
        layout.import,
        "导入 .vcf…",
    );
    draw_button(
        hdc,
        state.fonts.small,
        theme,
        metrics,
        layout.back,
        "← 返回常用设置",
    );
}

/// 绘制「自定义主题」子视图：说明、主题文件行（选中/损坏标注）、返回按钮。
unsafe fn draw_themes(
    hdc: HDC,
    state: &mut WindowState,
    theme: SettingsTheme,
    metrics: &SettingsMetrics,
    client: UiRect,
) {
    let layout = layout::themes_layout(metrics, client, state.theme_files.len());
    draw_text(
        hdc,
        "主题文件位于 %APPDATA%\\zhu-ye-ime\\themes\\*.json，每文件一个主题",
        layout.info,
        theme.item_text,
        state.fonts.small,
        DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
    );
    let empty_note = UiRect {
        left: layout.info.left,
        top: layout.info.top + layout.info.height() / 3,
        right: layout.info.right,
        bottom: layout.info.bottom,
    };
    if state.theme_files.is_empty() {
        draw_text(
            hdc,
            "no themes：把主题 JSON 放进该目录后重新进入本页（轻点“返回”再进入）",
            empty_note,
            theme.placeholder_text,
            state.fonts.small,
            DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
        );
    } else {
        draw_text(
            hdc,
            "选择即应用并写入 config.json；缺键回退预设，候选窗重启输入法生效",
            empty_note,
            theme.secondary_text,
            state.fonts.small,
            DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
        );
    }
    for (index, row) in layout.rows.iter().enumerate() {
        let Some(info) = state.theme_files.get(index) else {
            break;
        };
        let selected =
            matches!(&state.settings.theme, ThemeChoice::Custom(name) if name == &info.id);
        if selected {
            fill_round(hdc, *row, metrics.gap / 2, theme.control_selected);
        }
        let name_rect = UiRect {
            left: row.left + metrics.gap,
            top: row.top,
            right: row.right - metrics.pack_button_width - metrics.gap,
            bottom: row.bottom,
        };
        draw_text(
            hdc,
            &info.display_name,
            name_rect,
            if selected {
                theme.control_selected_text
            } else {
                theme.item_text
            },
            state.fonts.small,
            DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
        );
        let status = if info.file.is_none() {
            "文件损坏或版本过高"
        } else if selected {
            "当前"
        } else {
            "未应用"
        };
        let status_rect = UiRect {
            left: row.right - metrics.pack_button_width - metrics.gap,
            top: row.top,
            right: row.right - metrics.gap,
            bottom: row.bottom,
        };
        draw_text(
            hdc,
            status,
            status_rect,
            if info.file.is_none() {
                theme.warn_text
            } else if selected {
                theme.control_selected_text
            } else {
                theme.secondary_text
            },
            state.fonts.small,
            DT_RIGHT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
        );
    }
    draw_button(
        hdc,
        state.fonts.small,
        theme,
        metrics,
        layout.back,
        "← 返回常用设置",
    );
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

unsafe fn paint(hwnd: HWND, state: &mut WindowState) {
    unsafe {
        let mut paint_struct = windows::Win32::Graphics::Gdi::PAINTSTRUCT::default();
        let hdc = BeginPaint(hwnd, &mut paint_struct);
        if hdc.is_invalid() {
            return;
        }
        let client = client_rect(hwnd);
        if let Some(client) = client {
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
                                    eprintln!("zhu-ye-settings: 截图已写出 {}", path.display())
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

unsafe fn draw(hdc: HDC, state: &mut WindowState, client: UiRect) {
    let theme = state.theme();
    let metrics = SettingsMetrics::new(state.dpi);
    unsafe {
        state.fonts.ensure(state.dpi);
    }

    unsafe {
        fill(hdc, client, theme.window);
        let nav_area = UiRect {
            left: client.left,
            top: client.top,
            right: metrics.nav_width,
            bottom: client.bottom,
        };
        fill(hdc, nav_area, theme.nav_background);

        // 导航
        for (page, rect) in layout::nav_rows(&metrics, client) {
            let selected = page == state.settings.page;
            if selected {
                fill_round(hdc, rect, metrics.gap, theme.nav_selected);
            }
            let text_rect = UiRect {
                left: rect.left + metrics.padding,
                right: rect.right,
                ..rect
            };
            let color = if selected {
                theme.nav_selected_text
            } else {
                theme.nav_text
            };
            // 选中的导航项加粗（字重 600，T-125），未选中用正文。
            let font = if selected {
                state.fonts.nav_active
            } else {
                state.fonts.body
            };
            draw_text(
                hdc,
                page.title(),
                text_rect,
                color,
                font,
                DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
            );
        }

        // 页标题
        let title = layout::title_rect(&metrics, client);
        draw_text(
            hdc,
            state.settings.page.title(),
            title,
            theme.title_text,
            state.fonts.title,
            DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
        );
        fill(
            hdc,
            UiRect {
                left: title.left,
                top: title.bottom - 1,
                right: title.right,
                bottom: title.bottom,
            },
            theme.border,
        );

        match state.settings.subview {
            Subview::Packs => draw_packs(hdc, state, theme, &metrics, client),
            Subview::Manage => draw_manage(hdc, state, theme, &metrics, client),
            Subview::Repair => draw_repair(hdc, state, theme, &metrics, client),
            Subview::Update => draw_update(hdc, state, theme, &metrics, client),
            Subview::Diagnostics => draw_diagnostics(hdc, state, theme, &metrics, client),
            Subview::UserWords => draw_user_words(hdc, state, theme, &metrics, client),
            Subview::Contacts => draw_contacts(hdc, state, theme, &metrics, client),
            Subview::Themes => draw_themes(hdc, state, theme, &metrics, client),
            Subview::None => draw_items(hdc, state, theme, &metrics, client),
        }

        // 底部提示
        if let Some(hint) = state.hint.clone() {
            let rect = layout::hint_rect(&metrics, client);
            draw_text(
                hdc,
                &hint,
                rect,
                theme.secondary_text,
                state.fonts.small,
                DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
            );
        }
    }
}

/// 绘制普通条目列表（与 `item_rows` 对应的既有视图，含主题/模式二选一控件）。
unsafe fn draw_items(
    hdc: HDC,
    state: &mut WindowState,
    theme: SettingsTheme,
    metrics: &SettingsMetrics,
    client: UiRect,
) {
    let items = state.settings.page.items();
    let rows = layout::item_rows(metrics, client, items, state.settings.expanded);
    for row in &rows {
        let item = &items[row.index];
        // 标题 + 描述作为一块垂直居中于行内（T-125：行高 76，上下留白 16-20px 呼吸感）。
        let block_top = row.rect.top
            + (row.rect.height()
                - state.fonts.body_height
                - state.fonts.small_height
                - metrics.gap)
                / 2;
        let title_rect = UiRect {
            left: row.rect.left,
            top: block_top,
            right: row.rect.right,
            bottom: block_top + state.fonts.body_height,
        };
        draw_text(
            hdc,
            item.title,
            title_rect,
            theme.item_text,
            state.fonts.body,
            DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
        );
        let summary_rect = UiRect {
            left: row.rect.left,
            top: title_rect.bottom + metrics.gap / 2,
            right: row.rect.right,
            bottom: title_rect.bottom + metrics.gap / 2 + state.fonts.small_height,
        };
        let summary_color = if item.state.is_planned() {
            theme.placeholder_text
        } else {
            theme.secondary_text
        };
        draw_text(
            hdc,
            item.summary,
            summary_rect,
            summary_color,
            state.fonts.small,
            DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
        );

        for chip in &row.chips {
            let selected = match &chip.value {
                // 自定义主题不是二选一预设，两 Chip 均不选中（选中态展示在自定义主题子视图）。
                ChipValue::Theme(choice) => *choice == state.settings.theme,
                ChipValue::Mode(mode) => *mode == state.settings.default_mode,
                ChipValue::OnlineUpdate(on) => *on == state.settings.online_update,
                ChipValue::CandidatePin(show) => *show == state.settings.candidate_show_pin,
            };
            let background = if selected {
                theme.control_selected
            } else {
                theme.control_background
            };
            fill_round(hdc, chip.rect, metrics.gap / 2, background);
            let foreground = if selected {
                theme.control_selected_text
            } else {
                theme.item_text
            };
            draw_text(
                hdc,
                chip.label,
                chip.rect,
                foreground,
                state.fonts.small,
                DT_CENTER | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
            );
        }

        if let Some(note_rect) = row.expanded {
            let text_rect = UiRect {
                left: note_rect.left,
                top: note_rect.top,
                right: note_rect.right,
                bottom: note_rect.bottom - metrics.gap / 2,
            };
            if let Some(message) = item.state.message() {
                draw_text(
                    hdc,
                    &message,
                    text_rect,
                    theme.placeholder_text,
                    state.fonts.small,
                    DT_LEFT | DT_WORDBREAK | DT_NOPREFIX,
                );
            }
        }

        fill(
            hdc,
            UiRect {
                left: row.rect.left,
                top: row.rect.bottom - 1,
                right: row.rect.right,
                bottom: row.rect.bottom,
            },
            theme.border,
        );
    }
}

/// 绘制「添加词库」子视图（FR-022 / FR-042）：不验签说明、包行、底部按钮。
unsafe fn draw_packs(
    hdc: HDC,
    state: &mut WindowState,
    theme: SettingsTheme,
    metrics: &SettingsMetrics,
    client: UiRect,
) {
    let layout = layout::packs_layout(metrics, client, state.packs.len());

    // 不验签说明（D-38：导入不参与在线更新的签名信任链）。
    draw_text(
        hdc,
        "导入包以本地文件为准，不参与在线更新的签名信任链（不验签）",
        layout.note,
        theme.secondary_text,
        state.fonts.small,
        DT_LEFT | DT_WORDBREAK | DT_NOPREFIX,
    );
    fill(
        hdc,
        UiRect {
            left: layout.note.left,
            top: layout.note.bottom - 1,
            right: layout.note.right,
            bottom: layout.note.bottom,
        },
        theme.border,
    );

    for (index, pack) in layout.rows.iter().enumerate() {
        let Some(info) = state.packs.get(index) else {
            break; // 超出可用高度的行不绘制（布局不压缩，宁可截断）。
        };
        let title_rect = UiRect {
            left: pack.label.left,
            top: pack.label.top + metrics.gap,
            right: pack.label.right,
            bottom: pack.label.top + metrics.gap + state.fonts.body_height,
        };
        draw_text(
            hdc,
            &info.name,
            title_rect,
            theme.item_text,
            state.fonts.body,
            DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
        );
        let summary_rect = UiRect {
            left: pack.label.left,
            top: title_rect.bottom,
            right: pack.label.right,
            bottom: pack.label.bottom - metrics.gap / 2,
        };
        draw_text(
            hdc,
            &info.summary,
            summary_rect,
            theme.secondary_text,
            state.fonts.small,
            DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
        );

        // 元信息列：上排"词条数 · 体积"，下排版本/来源。
        let count_and_size = format!("{} · {}", info.entry_label(), info.size_label());
        let meta_first = UiRect {
            left: pack.meta.left,
            top: pack.meta.top + metrics.gap,
            right: pack.meta.right,
            bottom: pack.meta.top + metrics.gap + state.fonts.body_height,
        };
        draw_text(
            hdc,
            &count_and_size,
            meta_first,
            theme.secondary_text,
            state.fonts.small,
            DT_RIGHT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
        );
        let version_line = match info.source {
            Some(crate::installed::PackSource::Import) => "本地导入 · 未签名".to_owned(),
            _ => info.version_label(),
        };
        let meta_second = UiRect {
            left: pack.meta.left,
            top: meta_first.bottom,
            right: pack.meta.right,
            bottom: pack.meta.bottom - metrics.gap / 2,
        };
        draw_text(
            hdc,
            &version_line,
            meta_second,
            theme.secondary_text,
            state.fonts.small,
            DT_RIGHT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
        );

        // 启用开关；基础包只标注不可停用。
        if info.toggleable() {
            let selected = info.enabled;
            fill_round(
                hdc,
                pack.toggle,
                metrics.gap / 2,
                if selected {
                    theme.control_selected
                } else {
                    theme.control_background
                },
            );
            let foreground = if selected {
                theme.control_selected_text
            } else {
                theme.item_text
            };
            draw_text(
                hdc,
                if info.enabled { "已启用" } else { "停用" },
                pack.toggle,
                foreground,
                state.fonts.small,
                DT_CENTER | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
            );
        } else if info.base {
            draw_text(
                hdc,
                "基础包",
                pack.toggle,
                theme.placeholder_text,
                state.fonts.small,
                DT_CENTER | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
            );
        }

        fill(
            hdc,
            UiRect {
                left: pack.row.left,
                top: pack.row.bottom - 1,
                right: pack.row.right,
                bottom: pack.row.bottom,
            },
            theme.border,
        );
    }

    draw_button(
        hdc,
        state.fonts.small,
        theme,
        metrics,
        layout.back,
        "← 返回常用设置",
    );
    draw_button(
        hdc,
        state.fonts.small,
        theme,
        metrics,
        layout.import,
        "导入本地 .zyct…",
    );
}

/// 绘制「管理输入法」子视图：注册状态详情 + 系统设置入口 + 返回。
///
/// 状态区按 `RegistrationStatus::detail_lines()` 逐行绘制（自上而下，超出截断）；
/// 头部一行用状态色（已注册=常规文字，异常=警示文案 `headline()`）。
unsafe fn draw_manage(
    hdc: HDC,
    state: &mut WindowState,
    theme: SettingsTheme,
    metrics: &SettingsMetrics,
    client: UiRect,
) {
    let layout_rows = layout::manage_layout(metrics, client);

    // 「已注册」状态行 + 展开行。
    let status = state.manage.as_ref().map_or_else(
        || "正在读取注册状态…".to_owned(),
        |status| status.headline(),
    );
    let status_rect = UiRect {
        left: layout_rows.status.left,
        top: layout_rows.status.top + metrics.gap,
        right: layout_rows.status.right,
        bottom: layout_rows.status.top + metrics.gap + state.fonts.body_height,
    };
    let status_color = match &state.manage {
        Some(reg) if reg.registered() => theme.item_text,
        Some(_) => theme.warn_text,
        None => theme.placeholder_text,
    };
    draw_text(
        hdc,
        &status,
        status_rect,
        status_color,
        state.fonts.body,
        DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
    );
    if let Some(status) = &state.manage {
        let details = status.detail_lines();
        if !details.is_empty() {
            let row_height = state.fonts.small_height;
            let lines = details
                .iter()
                .take((layout_rows.status.height() / row_height.max(1)).max(1) as usize);
            for (index, line) in lines.enumerate() {
                let rect = UiRect {
                    left: layout_rows.status.left,
                    top: status_rect.bottom + index as i32 * row_height,
                    right: layout_rows.status.right,
                    bottom: status_rect.bottom + (index as i32 + 1) * row_height,
                };
                draw_text(
                    hdc,
                    line,
                    rect,
                    theme.secondary_text,
                    state.fonts.small,
                    DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
                );
            }
        }
        fill(
            hdc,
            UiRect {
                left: layout_rows.status.left,
                top: status_rect.bottom,
                right: layout_rows.status.right,
                bottom: status_rect.bottom + 1,
            },
            theme.border,
        );
    }

    draw_button(
        hdc,
        state.fonts.small,
        theme,
        metrics,
        layout_rows.back,
        "← 返回常用设置",
    );
    draw_button(
        hdc,
        state.fonts.small,
        theme,
        metrics,
        layout_rows.sys_settings,
        "打开系统输入法设置",
    );
}

/// 绘制「修复输入法」子视图：检测结果逐行 + 三个底部按钮。
///
/// 先报告后动手（D-41）：进入即扫描并逐行展示；按钮只有显式点击才执行。
/// 一级（无需提权）与二级（UAC 提权）分开，符合 D-40。
unsafe fn draw_repair(
    hdc: HDC,
    state: &mut WindowState,
    theme: SettingsTheme,
    metrics: &SettingsMetrics,
    client: UiRect,
) {
    let lines = state
        .repair_scan
        .as_ref()
        .map_or_else(Vec::new, |scan| scan.summary_lines());
    let layout_rows = layout::repair_layout(metrics, client, lines.len());
    for (index, row_rect) in layout_rows.rows.iter().enumerate() {
        let Some(line) = lines.get(index) else {
            break;
        };
        let rect = UiRect {
            left: row_rect.left + metrics.gap,
            top: row_rect.top,
            right: row_rect.right,
            bottom: row_rect.bottom,
        };
        draw_text(
            hdc,
            line,
            rect,
            theme.secondary_text,
            state.fonts.small,
            DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
        );
        fill(
            hdc,
            UiRect {
                left: row_rect.left,
                top: row_rect.bottom - 1,
                right: row_rect.right,
                bottom: row_rect.bottom,
            },
            theme.border,
        );
    }

    draw_button(
        hdc,
        state.fonts.small,
        theme,
        metrics,
        layout_rows.back,
        "← 返回常用设置",
    );
    // 提权动作与普通动作在视觉上区分：二级修复不套强调色。
    draw_button(
        hdc,
        state.fonts.small,
        theme,
        metrics,
        layout_rows.l1,
        "一级修复（无需管理员）",
    );
    fill_round(
        hdc,
        layout_rows.l2,
        metrics.gap / 2,
        theme.control_background,
    );
    draw_text(
        hdc,
        "二级修复（需管理员，UAC）",
        layout_rows.l2,
        theme.item_text,
        state.fonts.small,
        DT_CENTER | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
    );
}

/// 绘制「检查更新」子视图（T-077 / FR-044）。
///
/// 说明区给出开关状态与不可点的原因；结果区逐行展示更新器输出；
/// 底部「检查更新」为主按钮，未开启/更新器缺失时置灰且不响应。
unsafe fn draw_update(
    hdc: HDC,
    state: &mut WindowState,
    theme: SettingsTheme,
    metrics: &SettingsMetrics,
    client: UiRect,
) {
    let layout_rows = layout::update_layout(metrics, client);
    let online = state.settings.online_update;

    // 说明区第一行：在线更新开关状态（P-03 默认关）。
    let status_rect = UiRect {
        left: layout_rows.info.left,
        top: layout_rows.info.top + metrics.gap,
        right: layout_rows.info.right,
        bottom: layout_rows.info.top + metrics.gap + state.fonts.body_height,
    };
    draw_text(
        hdc,
        if online {
            "在线更新：已开启"
        } else {
            "在线更新：已关闭"
        },
        status_rect,
        if online {
            theme.item_text
        } else {
            theme.warn_text
        },
        state.fonts.body,
        DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
    );
    // 说明区第二行：不可点的原因或更新器位置（D-44 边界说明）。
    let hint_rect = UiRect {
        left: layout_rows.info.left,
        top: status_rect.bottom,
        right: layout_rows.info.right,
        bottom: layout_rows.info.bottom - metrics.gap / 2,
    };
    let hint = if !online {
        "未开启时不发起任何网络请求（D-44）：请在「启用在线更新」中开启".to_owned()
    } else {
        match updater::updater_exe_path() {
            Some(path) => format!("更新器：{}", path.display()),
            None => "更新器未找到：请安装完整发行包（含 bin\\zhu-ye-updater.exe）".to_owned(),
        }
    };
    draw_text(
        hdc,
        &hint,
        hint_rect,
        theme.secondary_text,
        state.fonts.small,
        DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
    );
    fill(
        hdc,
        UiRect {
            left: layout_rows.info.left,
            top: layout_rows.info.bottom - 1,
            right: layout_rows.info.right,
            bottom: layout_rows.info.bottom,
        },
        theme.border,
    );

    // 结果区：任务中显示进行中；否则显示最近一次结果（如实反映错误，D-44 无假共识）。
    let lines = update_result_lines(state);
    let row_height = state.fonts.small_height;
    let visible = (layout_rows.result.height() / row_height.max(1)).max(0) as usize;
    for (index, line) in lines.iter().take(visible).enumerate() {
        let rect = UiRect {
            left: layout_rows.result.left,
            top: layout_rows.result.top + index as i32 * row_height,
            right: layout_rows.result.right,
            bottom: layout_rows.result.top + (index as i32 + 1) * row_height,
        };
        draw_text(
            hdc,
            line,
            rect,
            if index == 0 {
                theme.item_text
            } else {
                theme.secondary_text
            },
            state.fonts.small,
            DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
        );
    }

    // 底部按钮：检查 / 应用 / 返回。
    let can_run = online && updater::updater_exe_path().is_some() && state.update_busy.is_none();
    let label = if state.update_busy.is_some() {
        "正在运行…"
    } else {
        "检查更新"
    };
    if can_run {
        draw_button(
            hdc,
            state.fonts.small,
            theme,
            metrics,
            layout_rows.check,
            label,
        );
    } else {
        draw_disabled_button(
            hdc,
            state.fonts.small,
            theme,
            metrics,
            layout_rows.check,
            label,
        );
    }
    fill_round(
        hdc,
        layout_rows.apply,
        metrics.gap / 2,
        theme.control_background,
    );
    draw_text(
        hdc,
        "应用更新",
        layout_rows.apply,
        if can_run {
            theme.item_text
        } else {
            theme.placeholder_text
        },
        state.fonts.small,
        DT_CENTER | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
    );
    draw_button(
        hdc,
        state.fonts.small,
        theme,
        metrics,
        layout_rows.back,
        "← 返回关于与更新",
    );
}

/// 结果区的逐行内容：任务进行中 / 最近一次手动检查结果 / 启动异步检查状态文件 /
/// 都为空时的占位文案。手动检查优先，其次展示 update_status.json（T-088 / FR-048）。
fn update_result_lines(state: &WindowState) -> Vec<String> {
    if let Some(kind) = state.update_busy {
        return vec![kind.busy_label().to_owned()];
    }
    if let Some((kind, result)) = &state.update_result {
        let mut lines = vec![format!("{}：", kind.done_title())];
        match result {
            Ok(text) => {
                let output = updater::output_lines(text);
                if output.is_empty() {
                    lines.push("（无输出）".to_owned());
                } else {
                    lines.extend(output);
                }
            }
            Err(text) => {
                lines.push("更新器执行失败：".to_owned());
                let output = updater::output_lines(text);
                if output.is_empty() {
                    lines.push("（无输出）".to_owned());
                } else {
                    lines.extend(output);
                }
            }
        }
        return lines;
    }
    if let Some(lines) = update_status_lines(state.config_path.as_deref()) {
        return lines;
    }
    vec!["尚未执行检查。".to_owned()]
}

/// 读取启动异步检查的状态文件（update_status.json），生成摘要行；文件不存在或损坏
/// 返回 `None`（调用方显示占位文案）。本函数不联网（S-4）。
fn update_status_lines(config_path: Option<&std::path::Path>) -> Option<Vec<String>> {
    let path = config_path
        .and_then(|path| path.parent())
        .map(|dir| dir.join(zhu_ye_core::UPDATE_STATUS_FILE_NAME))?;
    let status = zhu_ye_core::read_update_status(&path).ok()??;
    let mut lines = vec!["启动时异步检查（update_status.json）：".to_owned()];
    if let Some(error) = &status.error {
        lines.push(format!("检查失败：{error}"));
    } else if status.available {
        let packs = if status.outdated_packs.is_empty() {
            "但清单为空".to_owned()
        } else {
            format!("（{}）", status.outdated_packs.join("、"))
        };
        lines.push(format!(
            "发现可用更新：可更新 {} 个包{packs}，最新版本 {}",
            status.outdated_packs.len(),
            status.latest_version.as_deref().unwrap_or("未知")
        ));
    } else {
        lines.push("未发现可用更新。".to_owned());
    }
    lines.push(format!(
        "检查时间：{}",
        zhu_ye_core::format_date(status.last_checked)
    ));
    Some(lines)
}

/// 不可点按钮：底色与前景都用"禁用"色，命中端不响应。
unsafe fn draw_disabled_button(
    hdc: HDC,
    font: HFONT,
    theme: SettingsTheme,
    metrics: &SettingsMetrics,
    rect: UiRect,
    label: &str,
) {
    fill_round(hdc, rect, metrics.gap / 2, theme.control_background);
    draw_text(
        hdc,
        label,
        rect,
        theme.placeholder_text,
        font,
        DT_CENTER | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
    );
}

/// 绘制「版本与诊断信息」子视图（T-077 / FR-044）：逐行展示诊断内容。
unsafe fn draw_diagnostics(
    hdc: HDC,
    state: &mut WindowState,
    theme: SettingsTheme,
    metrics: &SettingsMetrics,
    client: UiRect,
) {
    let lines = state
        .diagnostics
        .as_ref()
        .map_or_else(Vec::new, |lines| lines.clone());
    let layout_rows = layout::diagnostics_layout(metrics, client, lines.len());
    for (index, row_rect) in layout_rows.rows.iter().enumerate() {
        let Some(line) = lines.get(index) else {
            break;
        };
        let rect = UiRect {
            left: row_rect.left + metrics.gap,
            top: row_rect.top,
            right: row_rect.right,
            bottom: row_rect.bottom,
        };
        draw_text(
            hdc,
            line,
            rect,
            if index == 0 {
                theme.item_text
            } else {
                theme.secondary_text
            },
            state.fonts.small,
            DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
        );
    }
    draw_button(
        hdc,
        state.fonts.small,
        theme,
        metrics,
        layout_rows.back,
        "← 返回关于与更新",
    );
}

/// 底部强调色按钮。
unsafe fn draw_button(
    hdc: HDC,
    font: HFONT,
    theme: SettingsTheme,
    metrics: &SettingsMetrics,
    rect: UiRect,
    label: &str,
) {
    fill_round(hdc, rect, metrics.gap / 2, theme.accent);
    draw_text(
        hdc,
        label,
        rect,
        theme.control_selected_text,
        font,
        DT_CENTER | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
    );
}

fn register_window_class(instance: HINSTANCE) -> Result<(), String> {
    static REGISTERED_ATOM: AtomicU16 = AtomicU16::new(0);
    if REGISTERED_ATOM.load(Ordering::Relaxed) != 0 {
        return Ok(());
    }
    let class = to_utf16(WINDOW_CLASS_NAME);
    let cursor = unsafe { LoadCursorW(None, IDC_ARROW) }.unwrap_or_default();
    let class_def = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wnd_proc),
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
            "注册设置窗口类失败: {}",
            std::io::Error::last_os_error()
        ));
    }
    REGISTERED_ATOM.store(atom, Ordering::Relaxed);
    Ok(())
}
