//! 设置窗口的 Win32 层：窗口类、消息循环、GDI 双缓冲绘制与命中测试。
//!
//! 绘制与命中测试共用 `layout` 的同一份矩形，避免"看得见的地方点不到"。窗口跟随系统
//! 深浅色与高对比度（D-30），主题变化时重算配色并重绘。

use std::mem::size_of;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU16, Ordering};

use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontIndirectW, CreatePen,
    CreateSolidBrush, DeleteDC, DeleteObject, DrawTextW, EndPaint, FillRect, GetDIBits,
    InvalidateRect, RoundRect, SelectObject, SetBkMode, SetTextColor, UpdateWindow, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, CLEARTYPE_QUALITY, DEFAULT_CHARSET, DEFAULT_PITCH, DIB_RGB_COLORS,
    DRAW_TEXT_FORMAT, DT_CENTER, DT_END_ELLIPSIS, DT_LEFT, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER,
    DT_WORDBREAK, FF_DONTCARE, FW_NORMAL, FW_SEMIBOLD, HBRUSH, HDC, HFONT, HGDIOBJ, LOGFONTW,
    PS_NULL, SRCCOPY, TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::AdjustWindowRectExForDpi;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_ESCAPE;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, FindWindowW, GetClientRect,
    GetMessageW, GetWindowLongPtrW, LoadCursorW, PostMessageW, PostQuitMessage, RegisterClassW,
    SetForegroundWindow, SetWindowLongPtrW, ShowWindow, SystemParametersInfoW, TranslateMessage,
    CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA, IDC_ARROW,
    NONCLIENTMETRICSW, SPI_GETNONCLIENTMETRICS, SW_RESTORE, SW_SHOW,
    SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, WINDOW_EX_STYLE, WM_CLOSE, WM_DESTROY, WM_DPICHANGED,
    WM_ERASEBKGND, WM_KEYDOWN, WM_LBUTTONDOWN, WM_NCCREATE, WM_PAINT, WM_SETTINGCHANGE,
    WM_SYSKEYDOWN, WM_THEMECHANGED, WNDCLASSW, WS_OVERLAPPEDWINDOW,
};
use zhu_ye_core::ThemeChoice;
use zhu_ye_ime::candidate_ui::{UiColor, UiRect, UiThemeKind, BASE_DPI};

use crate::config;
use crate::layout::{self, SettingsMetrics};
use crate::model::{ItemControl, Page, SettingsState};
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
}

impl WindowState {
    fn new(theme: ThemeChoice, config_path: Option<PathBuf>, options: &RunOptions) -> Self {
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

        let state = Box::new(WindowState::new(theme, config_file_path, &options));
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
            if items[row.index].control == ItemControl::ThemeChoice {
                // 主题行本身不切换展开，避免与控件块点击混淆。
                return;
            }
            state.settings.click_item(row.index);
            invalidate(hwnd);
            return;
        }
    }
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

unsafe fn fill(hdc: HDC, rect: UiRect, color: UiColor) {
    unsafe {
        let brush = CreateSolidBrush(COLORREF(color.to_colorref()));
        let native = to_native(rect);
        FillRect(hdc, &native, brush);
        let _ = DeleteObject(brush.into());
    }
}

unsafe fn fill_round(hdc: HDC, rect: UiRect, radius: i32, color: UiColor) {
    unsafe {
        let brush = CreateSolidBrush(COLORREF(color.to_colorref()));
        let pen = CreatePen(PS_NULL, 0, COLORREF(0));
        let previous_brush = SelectObject(hdc, brush.into());
        let previous_pen = SelectObject(hdc, pen.into());
        let _ = RoundRect(
            hdc,
            rect.left,
            rect.top,
            rect.right,
            rect.bottom,
            radius.max(1) * 2,
            radius.max(1) * 2,
        );
        let _ = SelectObject(hdc, previous_brush);
        let _ = SelectObject(hdc, previous_pen);
        let _ = DeleteObject(brush.into());
        let _ = DeleteObject(pen.into());
    }
}

unsafe fn draw_text(
    hdc: HDC,
    text: &str,
    rect: UiRect,
    color: UiColor,
    font: HFONT,
    format: DRAW_TEXT_FORMAT,
) {
    unsafe {
        // `DrawTextW` 的窗口绑定按切片长度传参，因此这里不追加结尾 NUL。
        let mut wide: Vec<u16> = text.encode_utf16().collect();
        let mut native = to_native(rect);
        if !font.is_invalid() {
            let _ = SelectObject(hdc, font.into());
        }
        let _ = SetBkMode(hdc, TRANSPARENT);
        let _ = SetTextColor(hdc, COLORREF(color.to_colorref()));
        let _ = DrawTextW(hdc, &mut wide, &mut native, format);
    }
}

const fn to_native(rect: UiRect) -> RECT {
    RECT {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    }
}

/// GDI 后备缓冲；尺寸变化时重建位图。
#[derive(Default)]
struct BackBuffer {
    memory_dc: HDC,
    bitmap: windows::Win32::Graphics::Gdi::HBITMAP,
    previous: HGDIOBJ,
    width: i32,
    height: i32,
}

impl BackBuffer {
    /// 准备一个不小于给定尺寸的内存 DC。
    unsafe fn prepare(&mut self, window_dc: HDC, width: i32, height: i32) -> HDC {
        unsafe {
            if self.memory_dc.is_invalid() {
                self.memory_dc = CreateCompatibleDC(Some(window_dc));
                if self.memory_dc.is_invalid() {
                    return HDC::default();
                }
            }
            if self.bitmap.is_invalid() || self.width < width || self.height < height {
                if !self.bitmap.is_invalid() {
                    let _ = SelectObject(self.memory_dc, self.previous);
                    let _ = DeleteObject(self.bitmap.into());
                }
                self.bitmap = CreateCompatibleBitmap(window_dc, width, height);
                if self.bitmap.is_invalid() {
                    return HDC::default();
                }
                self.previous = SelectObject(self.memory_dc, self.bitmap.into());
                self.width = width;
                self.height = height;
            }
            self.memory_dc
        }
    }

    /// 把后备缓冲写成 32 位 BMP（自顶向下行序，便于肉眼与像素取证核对）。
    unsafe fn save_bmp(&self, width: i32, height: i32, path: &Path) -> Result<(), String> {
        unsafe {
            if self.bitmap.is_invalid() {
                return Err("没有可导出的位图".to_owned());
            }
            let mut info = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: u32::try_from(size_of::<BITMAPINFOHEADER>()).unwrap_or(40),
                    biWidth: width,
                    biHeight: -height,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut pixels = vec![0u8; (width as usize) * (height as usize) * 4];
            let lines = GetDIBits(
                self.memory_dc,
                self.bitmap,
                0,
                u32::try_from(height).unwrap_or(0),
                Some(pixels.as_mut_ptr().cast()),
                &mut info,
                DIB_RGB_COLORS,
            );
            if lines == 0 {
                return Err("读取窗口像素失败".to_owned());
            }
            let mut bmp = Vec::with_capacity(54 + pixels.len());
            bmp.extend_from_slice(b"BM");
            bmp.extend_from_slice(&u32::try_from(54 + pixels.len()).unwrap_or(0).to_le_bytes());
            bmp.extend_from_slice(&0u16.to_le_bytes());
            bmp.extend_from_slice(&0u16.to_le_bytes());
            bmp.extend_from_slice(&54u32.to_le_bytes());
            bmp.extend_from_slice(&40u32.to_le_bytes());
            bmp.extend_from_slice(&width.to_le_bytes());
            bmp.extend_from_slice(&(-height).to_le_bytes());
            bmp.extend_from_slice(&1u16.to_le_bytes());
            bmp.extend_from_slice(&32u16.to_le_bytes());
            bmp.extend_from_slice(&0u32.to_le_bytes());
            bmp.extend_from_slice(&u32::try_from(pixels.len()).unwrap_or(0).to_le_bytes());
            bmp.extend_from_slice(&0i32.to_le_bytes());
            bmp.extend_from_slice(&0i32.to_le_bytes());
            bmp.extend_from_slice(&0u32.to_le_bytes());
            bmp.extend_from_slice(&0u32.to_le_bytes());
            bmp.extend_from_slice(&pixels);
            std::fs::write(path, bmp).map_err(|error| format!("写出截图失败: {error}"))
        }
    }
}

impl Drop for BackBuffer {
    fn drop(&mut self) {
        unsafe {
            if !self.bitmap.is_invalid() {
                let _ = SelectObject(self.memory_dc, self.previous);
                let _ = DeleteObject(self.bitmap.into());
            }
            if !self.memory_dc.is_invalid() {
                let _ = DeleteDC(self.memory_dc);
            }
        }
    }
}

/// 字体集合；随 DPI 变化重建。
#[derive(Default)]
struct Fonts {
    title: HFONT,
    body: HFONT,
    small: HFONT,
    body_height: i32,
    small_height: i32,
    dpi: u32,
}

impl Fonts {
    /// 按 DPI 准备字体；DPI 未变时复用。
    unsafe fn ensure(&mut self, dpi: u32) {
        unsafe {
            if self.dpi == dpi && !self.body.is_invalid() {
                return;
            }
            self.release();
            let base = system_message_font();
            let system_dpi = shell::system_dpi().max(1) as i32;
            let target_dpi = i32::try_from(dpi).unwrap_or(96);
            let scale = |logical: i32, numerator: i32, denominator: i32| -> i32 {
                (logical * numerator * target_dpi / (denominator * system_dpi)).max(1)
            };
            // `lfHeight` 为负表示字符高度（正数表示单元格高度）；先取绝对值再缩放，
            // 否则 `.max(1)` 会把负值钳成 1，字体退化成 1 像素高。
            let base_height = base.lfHeight.abs().max(1);

            let mut body_font = base;
            body_font.lfHeight = -scale(base_height, 1, 1);
            let mut title_font = base;
            title_font.lfHeight = -scale(base_height, 5, 4);
            title_font.lfWeight = FW_SEMIBOLD.0 as i32;
            let mut small_font = base;
            small_font.lfHeight = -scale(base_height, 4, 5);

            self.body = CreateFontIndirectW(&body_font);
            self.title = CreateFontIndirectW(&title_font);
            self.small = CreateFontIndirectW(&small_font);
            self.body_height = body_font.lfHeight.abs();
            self.small_height = small_font.lfHeight.abs();
            self.dpi = dpi;
        }
    }

    /// 释放字体；DPI 变化或窗口销毁时调用。
    unsafe fn release(&mut self) {
        unsafe {
            for font in [self.title, self.body, self.small] {
                if !font.is_invalid() {
                    let _ = DeleteObject(font.into());
                }
            }
            self.title = HFONT::default();
            self.body = HFONT::default();
            self.small = HFONT::default();
            self.body_height = 0;
            self.small_height = 0;
            self.dpi = 0;
        }
    }
}

impl Drop for Fonts {
    fn drop(&mut self) {
        unsafe {
            self.release();
        }
    }
}

/// 读取系统消息字体（即系统默认 UI 字体，D-30）。
unsafe fn system_message_font() -> LOGFONTW {
    unsafe {
        let mut metrics = NONCLIENTMETRICSW {
            cbSize: u32::try_from(size_of::<NONCLIENTMETRICSW>()).unwrap_or(0),
            ..Default::default()
        };
        let _ = SystemParametersInfoW(
            SPI_GETNONCLIENTMETRICS,
            metrics.cbSize,
            Some((&mut metrics as *mut NONCLIENTMETRICSW).cast()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS::default(),
        );
        let mut font = metrics.lfMessageFont;
        if font.lfHeight == 0 {
            // 读不到系统字体时回退到 GDI 默认 GUI 字体的字面名，避免画出零高字体。
            font = LOGFONTW {
                lfHeight: -12,
                lfWeight: FW_NORMAL.0 as i32,
                lfCharSet: DEFAULT_CHARSET,
                lfQuality: CLEARTYPE_QUALITY,
                lfPitchAndFamily: DEFAULT_PITCH.0 | FF_DONTCARE.0,
                ..Default::default()
            };
            set_face_name(&mut font, &to_utf16("Microsoft YaHei UI"));
        }
        font
    }
}

/// 写入字体字面名（最多 `LF_FACESIZE - 1` 个 UTF-16 单元，含结尾 NUL）。
fn set_face_name(font: &mut LOGFONTW, name: &[u16]) {
    let capacity = font.lfFaceName.len();
    if capacity == 0 {
        return;
    }
    let limit = capacity - 1;
    for (index, unit) in name.iter().take(limit).enumerate() {
        font.lfFaceName[index] = *unit;
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
