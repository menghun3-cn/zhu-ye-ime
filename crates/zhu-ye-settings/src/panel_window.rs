//! 工具箱面板：非激活浮层窗口。
//!
//! 面板用 `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST` 浮在最上层，并且
//! **永不夺取焦点**（`WM_MOUSEACTIVATE` 回 `MA_NOACTIVATE`）。因此用户的目标应用在整个
//! 浏览过程中保持前台，点击格子时 `deliver` 送出的字符才落得到那里——这是"选中即上屏"
//! 的实现前提。
//!
//! 代价：非激活窗口收不到键盘输入，所以面板没有 Esc 关闭，只能点"返回设置"。

use std::path::{Path, PathBuf};

use windows::core::PCWSTR;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, EndPaint, InvalidateRect, UpdateWindow, DT_CENTER, DT_END_ELLIPSIS,
    DT_LEFT, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, HBRUSH, HDC, SRCCOPY,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::AdjustWindowRectExForDpi;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetClientRect, GetMessageW,
    GetWindowLongPtrW, LoadCursorW, PostMessageW, PostQuitMessage, RegisterClassW,
    SetWindowLongPtrW, SetWindowPos, ShowWindow, SystemParametersInfoW, TranslateMessage,
    CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, GWLP_USERDATA, HWND_TOPMOST, IDC_ARROW, MA_NOACTIVATE,
    SPI_GETWORKAREA, SWP_NOACTIVATE, SWP_SHOWWINDOW, SW_SHOWNOACTIVATE,
    SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, WINDOW_EX_STYLE, WM_APP, WM_CLOSE, WM_DESTROY,
    WM_ERASEBKGND, WM_LBUTTONDOWN, WM_MOUSEACTIVATE, WM_NCCREATE, WM_PAINT, WM_SETTINGCHANGE,
    WM_THEMECHANGED, WNDCLASSW, WS_POPUP,
};

use zhu_ye_ui::{UiRect, UiThemeKind, BASE_DPI};

use crate::deliver::{deliver_text, Delivery};
use crate::gdi::{draw_text, fill, fill_round, BackBuffer, Fonts};
use crate::layout::{self, PanelMetrics};
use crate::panel::PanelView;
use crate::shell;
use crate::theme::{settings_theme, settings_theme_from_system_colors, SettingsTheme};
use crate::wide::to_utf16;

/// 面板窗口类名。
pub const PANEL_CLASS_NAME: &str = "ZhuYeSettingsPanel";

/// 面板关闭时通知设置窗口的自定义消息。
///
/// 面板不激活，因此无法靠"焦点回到设置窗口"来判断关闭时机；关闭动作显式通知。
pub const WM_PANEL_CLOSED: u32 = WM_APP + 1;

/// 已打开的面板窗口；`Drop` 销毁窗口。
pub struct PanelWindow {
    hwnd: HWND,
}

impl PanelWindow {
    /// 创建并显示面板。不启动消息循环：面板与设置窗口同线程，共用设置窗口的消息循环。
    ///
    /// # Errors
    /// 注册窗口类或创建窗口失败时返回描述。
    pub fn open(view: PanelView, settings_hwnd: HWND) -> Result<Self, String> {
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

            let state = Box::new(PanelState::new(view, settings_hwnd, None));
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
            position_panel(hwnd, frame.right - frame.left, frame.bottom - frame.top);
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
/// 供验收取证，与设置窗口的 `--shot` 同路数。
///
/// # Errors
/// 建窗、绘制或写文件失败时返回描述。
pub fn run_shot(view: PanelView, path: &Path) -> Result<(), String> {
    let path = path.to_path_buf();
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
        let state = Box::new(PanelState::new(view, HWND::default(), Some(path)));
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
        position_panel(hwnd, frame.right - frame.left, frame.bottom - frame.top);
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

/// 把面板居中到工作区（避开任务栏）。
unsafe fn position_panel(hwnd: HWND, width: i32, height: i32) {
    unsafe {
        let mut work = RECT::default();
        let ok = SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            Some((&mut work as *mut RECT).cast()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS::default(),
        )
        .is_ok();
        let (left, top) = if ok {
            (
                work.left + (work.right - work.left - width) / 2,
                work.top + (work.bottom - work.top - height) / 2,
            )
        } else {
            (80, 80)
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
    settings_hwnd: HWND,
    hint: Option<String>,
    shot_path: Option<PathBuf>,
    shot_done: bool,
    back: BackBuffer,
    fonts: Fonts,
}

impl PanelState {
    fn new(view: PanelView, settings_hwnd: HWND, shot_path: Option<PathBuf>) -> Self {
        Self {
            view,
            theme_kind: shell::resolve_theme_kind(),
            dpi: shell::system_dpi().max(BASE_DPI),
            settings_hwnd,
            hint: None,
            shot_path,
            shot_done: false,
            back: BackBuffer::default(),
            fonts: Fonts::default(),
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
        // 非激活：点击面板既不激活它、也不把焦点从用户的目标应用拿走。
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
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
                    let settings = state.settings_hwnd;
                    drop(Box::from_raw(state as *mut PanelState));
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                    // 通知设置窗口：面板已关，该把焦点收回来了。
                    if !settings.is_invalid() {
                        let _ = PostMessageW(Some(settings), WM_PANEL_CLOSED, WPARAM(0), LPARAM(0));
                    } else {
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

/// 处理点击：格子投递字符，翻页改页，返回关闭面板。
unsafe fn on_click(hwnd: HWND, state: &mut PanelState, x: i32, y: i32) {
    let Some(client) = (unsafe { client_rect(hwnd) }) else {
        return;
    };
    let metrics = PanelMetrics::new(state.dpi);
    let layout = layout::panel_layout(&metrics, client, state.view.visible_len());

    if layout::contains(layout.back, x, y) {
        unsafe { close_panel(hwnd) };
        return;
    }
    if layout::contains(layout.prev, x, y) {
        state.view.prev_page();
        unsafe { invalidate(hwnd) };
        return;
    }
    if layout::contains(layout.next, x, y) {
        state.view.next_page();
        unsafe { invalidate(hwnd) };
        return;
    }
    for (index, cell) in layout.cells.iter().enumerate() {
        if layout::contains(*cell, x, y) {
            if let Some(entry) = state.view.entry_at(index) {
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
    let layout = layout::panel_layout(&metrics, client, state.view.visible_len());
    let entries = state.view.visible();

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
            theme.border,
        );

        // 标题：面板名 + 分组跨度提示
        let title = UiRect {
            left: layout.header.left + metrics.padding,
            top: layout.header.top,
            right: layout.header.right - metrics.padding,
            bottom: layout.header.bottom,
        };
        let first = entries.first().map(|entry| entry.group).unwrap_or("");
        let last = entries.last().map(|entry| entry.group).unwrap_or("");
        let heading = if first.is_empty() {
            state.view.kind.title().to_owned()
        } else if first == last {
            format!("{}（{} 组）", state.view.kind.title(), first)
        } else {
            format!("{}（{}–{} 组）", state.view.kind.title(), first, last)
        };
        draw_text(
            hdc,
            &heading,
            title,
            theme.title_text,
            state.fonts.title,
            DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
        );

        // 格子：字符居中，副标签在下方（emoji 为拼音别名，符号为组名）
        for (index, cell) in layout.cells.iter().enumerate() {
            let Some(entry) = entries.get(index) else {
                continue;
            };
            fill_round(hdc, *cell, metrics.gap, theme.control_background);
            let glyph = UiRect {
                top: cell.top + metrics.gap / 2,
                bottom: cell.bottom - state.fonts.small_height - metrics.gap / 2,
                ..*cell
            };
            draw_text(
                hdc,
                entry.text,
                glyph,
                theme.item_text,
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
                theme.secondary_text,
                state.fonts.small,
                DT_CENTER | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
            );
        }

        // 提示
        if let Some(hint) = state.hint.clone() {
            draw_text(
                hdc,
                &hint,
                layout.hint,
                theme.placeholder_text,
                state.fonts.small,
                DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
            );
        }

        // 页脚
        draw_button(hdc, state, layout.back, "返回设置", false);
        draw_button(hdc, state, layout.prev, "上一页", false);
        draw_button(hdc, state, layout.next, "下一页", false);
        draw_text(
            hdc,
            &state.view.page_label(),
            layout.page_label,
            theme.secondary_text,
            state.fonts.small,
            DT_CENTER | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
        );

        // 外边框
        fill(
            hdc,
            UiRect {
                left: client.left,
                top: client.top,
                right: client.right,
                bottom: client.top + 1,
            },
            theme.border,
        );
        fill(
            hdc,
            UiRect {
                left: client.left,
                top: client.bottom - 1,
                right: client.right,
                bottom: client.bottom,
            },
            theme.border,
        );
        fill(
            hdc,
            UiRect {
                left: client.left,
                top: client.top,
                right: client.left + 1,
                bottom: client.bottom,
            },
            theme.border,
        );
        fill(
            hdc,
            UiRect {
                left: client.right - 1,
                top: client.top,
                right: client.right,
                bottom: client.bottom,
            },
            theme.border,
        );
    }
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
                theme.control_selected
            } else {
                theme.control_background
            },
        );
        draw_text(
            hdc,
            label,
            rect,
            if primary {
                theme.control_selected_text
            } else {
                theme.item_text
            },
            state.fonts.small,
            DT_CENTER | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
        );
    }
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
