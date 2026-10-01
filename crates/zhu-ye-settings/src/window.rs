//! 设置窗口的 Win32 层：窗口类、消息循环、GDI 双缓冲绘制与命中测试。
//!
//! 绘制与命中测试共用 `layout` 的同一份矩形，避免"看得见的地方点不到"。窗口跟随系统
//! 深浅色与高对比度（D-30），主题变化时重算配色并重绘。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU16, Ordering};

use windows::core::PCWSTR;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, EndPaint, InvalidateRect, UpdateWindow, DT_CENTER, DT_END_ELLIPSIS,
    DT_LEFT, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, DT_WORDBREAK, HBRUSH, HDC, SRCCOPY,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::AdjustWindowRectExForDpi;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_ESCAPE;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, FindWindowW, GetClientRect,
    GetForegroundWindow, GetMessageW, GetWindowLongPtrW, LoadCursorW, PostMessageW,
    PostQuitMessage, RegisterClassW, SetForegroundWindow, SetWindowLongPtrW, ShowWindow,
    TranslateMessage, CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA,
    IDC_ARROW, SW_RESTORE, SW_SHOW, WINDOW_EX_STYLE, WM_CLOSE, WM_DESTROY, WM_DPICHANGED,
    WM_ERASEBKGND, WM_KEYDOWN, WM_LBUTTONDOWN, WM_NCCREATE, WM_PAINT, WM_SETTINGCHANGE,
    WM_SYSKEYDOWN, WM_THEMECHANGED, WNDCLASSW, WS_OVERLAPPEDWINDOW,
};
use zhu_ye_core::ThemeChoice;
use zhu_ye_ime::candidate_ui::{UiRect, UiThemeKind, BASE_DPI};

use crate::config;
use crate::gdi::{draw_text, fill, fill_round, BackBuffer, Fonts};
use crate::layout::{self, SettingsMetrics};
use crate::model::{ItemControl, Page, SettingsState};
use crate::panel::PanelView;
use crate::panel_window::{PanelWindow, WM_PANEL_CLOSED};
use crate::shell;
use crate::theme::{settings_theme, settings_theme_from_system_colors, SettingsTheme};
use crate::wide::to_utf16;

/// 窗口类名；第二条实例用它查找已有窗口。
pub const WINDOW_CLASS_NAME: &str = "ZhuYeSettingsWindow";
/// 窗口标题。
pub const WINDOW_TITLE: &str = "竹叶输入法 设置";

/// 启动选项。
#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    /// 截图输出路径；`Some` 时在首帧绘制后写出 BMP 并关闭窗口（验收取证）。
    pub shot_path: Option<PathBuf>,
    /// 截图时显示的页面。
    pub shot_page: Option<Page>,
    /// 截图时展开的条目下标。
    pub shot_expanded: Option<usize>,
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
}

impl WindowState {
    fn new(
        theme: ThemeChoice,
        config_path: Option<PathBuf>,
        options: &RunOptions,
        previous_foreground: HWND,
    ) -> Self {
        let mut settings = SettingsState::new(theme);
        if let Some(page) = options.shot_page {
            settings.page = page;
            settings.expanded = options.shot_expanded;
        }
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
        }
    }

    /// 当前配色。
    fn theme(&self) -> SettingsTheme {
        if self.theme_kind == UiThemeKind::HighContrast {
            settings_theme_from_system_colors(shell::system_colors())
        } else {
            settings_theme(self.theme_kind)
        }
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
unsafe fn on_click(hwnd: HWND, state: &mut WindowState, x: i32, y: i32) {
    let client = match client_rect(hwnd) {
        Some(rect) => rect,
        None => return,
    };
    let metrics = SettingsMetrics::new(state.dpi);

    for (page, rect) in layout::nav_rows(&metrics, client) {
        if layout::contains(rect, x, y) {
            state.settings.select_page(page);
            invalidate(hwnd);
            return;
        }
    }

    let items = state.settings.page.items();
    for row in layout::item_rows(&metrics, client, items, state.settings.expanded) {
        for (choice, chip) in &row.chips {
            if layout::contains(*chip, x, y) {
                apply_theme(state, *choice);
                invalidate(hwnd);
                return;
            }
        }
        if layout::contains(row.rect, x, y) {
            match items[row.index].control {
                // 主题行本身不切换展开，避免与控件块点击混淆。
                ItemControl::ThemeChoice => {}
                ItemControl::OpenPanel(kind) => open_panel(hwnd, state, kind),
                ItemControl::None => {
                    state.settings.click_item(row.index);
                    invalidate(hwnd);
                }
            }
            return;
        }
    }
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

/// 应用主题选择并持久化。
fn apply_theme(state: &mut WindowState, choice: ThemeChoice) {
    state.settings.theme = choice;
    state.hint = Some(match &state.config_path {
        Some(path) => match config::save_theme(path, choice) {
            // 主题是装配项：由 TSF DLL 在下次装配时读取（P-12）。
            Ok(()) => format!(
                "已保存主题：{}，重启输入法后候选窗生效",
                theme_label(choice)
            ),
            Err(error) => format!("保存失败：{error}"),
        },
        None => "未找到配置目录（APPDATA 未设置），本次选择不会保留".to_owned(),
    });
}

const fn theme_label(choice: ThemeChoice) -> &'static str {
    match choice {
        ThemeChoice::Light => "浅色",
        ThemeChoice::Dark => "深色",
    }
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
            draw_text(
                hdc,
                page.title(),
                text_rect,
                color,
                state.fonts.body,
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

        // 条目
        let items = state.settings.page.items();
        let rows = layout::item_rows(&metrics, client, items, state.settings.expanded);
        for row in &rows {
            let item = &items[row.index];
            let title_rect = UiRect {
                left: row.rect.left,
                top: row.rect.top + metrics.gap,
                right: row.rect.right,
                bottom: row.rect.top + metrics.gap + state.fonts.body_height,
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
                top: title_rect.bottom,
                right: row.rect.right,
                bottom: row.rect.bottom - metrics.gap / 2,
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

            for (choice, chip) in &row.chips {
                let selected = *choice == state.settings.theme;
                let background = if selected {
                    theme.control_selected
                } else {
                    theme.control_background
                };
                fill_round(hdc, *chip, metrics.gap / 2, background);
                let foreground = if selected {
                    theme.control_selected_text
                } else {
                    theme.item_text
                };
                draw_text(
                    hdc,
                    theme_label(*choice),
                    *chip,
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
