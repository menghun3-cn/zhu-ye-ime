//! 候选窗 Win32 GDI 自绘实现。
//!
//! 本模块负责把 `candidate_ui` 计算出的快照画出来，并提供 TSF 驱动的
//! 受控窗口入口：同线程创建/更新/隐藏/销毁，不占用独立消息循环。

use std::mem;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::OnceLock;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontIndirectW, CreatePen,
    CreateSolidBrush, DeleteDC, DeleteObject, DrawTextW, EndPaint, FillRect, GetDC, GetDIBits,
    GetMonitorInfoW, GetStockObject, GetSysColor, InvalidateRect, MonitorFromWindow, ReleaseDC,
    RoundRect, SelectObject, SetBkMode, SetTextColor, UpdateWindow, BITMAPINFO, BITMAPINFOHEADER,
    BI_RGB, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, COLOR_BTNFACE, COLOR_GRAYTEXT, COLOR_HIGHLIGHT,
    COLOR_HIGHLIGHTTEXT, COLOR_WINDOW, COLOR_WINDOWTEXT, DEFAULT_CHARSET, DEFAULT_GUI_FONT,
    DEFAULT_PITCH, DIB_RGB_COLORS, DT_END_ELLIPSIS, DT_NOPREFIX, DT_RIGHT, DT_SINGLELINE,
    DT_VCENTER, FF_DONTCARE, FW_NORMAL, HBRUSH, HDC, HFONT, LOGFONTW, MONITORINFO,
    MONITOR_DEFAULTTONEAREST, OUT_DEFAULT_PRECIS, PAINTSTRUCT, PS_NULL, PS_SOLID, SRCCOPY,
    TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Accessibility::{HCF_HIGHCONTRASTON, HIGHCONTRASTW};
use windows::Win32::UI::HiDpi::{
    GetDpiForSystem, SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows::Win32::UI::Input::KeyboardAndMouse::VK_ESCAPE;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetClientRect, GetMessageW,
    GetWindowLongPtrW, PostMessageW, PostQuitMessage, RegisterClassW, SetTimer, SetWindowLongPtrW,
    SetWindowPos, ShowWindow, SystemParametersInfoW, TranslateMessage, CREATESTRUCTW, CS_HREDRAW,
    CS_VREDRAW, GWLP_USERDATA, HWND_TOP, MSG, SPI_GETHIGHCONTRAST, SWP_NOACTIVATE, SWP_NOMOVE,
    SWP_NOZORDER, SW_HIDE, SW_SHOWNOACTIVATE, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, WM_CLOSE,
    WM_DESTROY, WM_DPICHANGED, WM_ERASEBKGND, WM_KEYDOWN, WM_NCCREATE, WM_PAINT, WM_SETTINGCHANGE,
    WM_SYSKEYDOWN, WM_THEMECHANGED, WM_TIMER, WNDCLASSW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    WS_EX_TOPMOST, WS_POPUP,
};

use crate::candidate_ui::{
    display_main_text, index_marker, theme, theme_from_system_colors, CandidateMetrics,
    CandidateUiTheme, CandidateUiView, SystemColors, UiColor, UiRect, UiThemeKind, BASE_DPI,
};

/// 窗口主题偏好；`Auto` 跟随系统深浅色与高对比度。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemePreference {
    #[default]
    Auto,
    Light,
    Dark,
    HighContrast,
}

/// 候选窗演示选项；T-013 接入 TSF 后窗口由输入上下文驱动，不再走 CLI。
/// 演示计时器 ID，仅用于 `--seconds` 自动关闭窗口。
const DEMO_TIMER_ID: usize = 1;
#[derive(Debug, Clone, Default)]
pub struct CandidateWindowOptions {
    pub theme: ThemePreference,
    pub dpi: Option<u32>,
    pub seconds: Option<u64>,
    pub shot_path: Option<PathBuf>,
}

/// 候选窗放置点：`anchor` 为组成区屏幕坐标左边界/底部，窗口显示在其下方。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CandidateWindowPlacement {
    pub anchor: POINT,
}

/// TSF 生命周期内的候选窗控制器。
///
/// 不启动消息循环，所有创建/更新/隐藏都发生在调用线程（TSF 宿主 UI
/// 线程）上。`Drop` 销毁窗口并释放窗口状态。
pub struct CandidateWindow {
    hwnd: HWND,
    state_ptr: *mut CandidateWindowState,
}

impl CandidateWindow {
    /// 创建候选窗控制器；视图在首次 `update` 时落盘，此处只预留空状态。
    #[must_use]
    pub fn new() -> Self {
        Self {
            hwnd: HWND::default(),
            state_ptr: std::ptr::null_mut(),
        }
    }

    /// 更新候选视图；组合串为空且无任何候选项（含上屏联想，T-059）时隐藏窗口，
    /// 否则创建（如未创建）并显示。无候选词时也显示——只画页眉条
    /// （组合串与拼音提示），见 T-031。
    pub fn update(&mut self, view: CandidateUiView, placement: Option<CandidateWindowPlacement>) {
        if view.composition.is_empty() && view.items.is_empty() {
            self.hide();
            return;
        }
        if self.hwnd.is_invalid() {
            self.create_window(&view);
        }
        if self.hwnd.is_invalid() {
            return;
        }
        unsafe {
            if let Some(state) = state_mut(self.hwnd) {
                state.view = view;
                let rows = state.view.panel_rows();
                let (width, height) = state.metrics.panel_size(rows);
                let width = width.max(1);
                let height = height.max(1);
                if let Some(placement) = placement {
                    place_at(self.hwnd, placement.anchor, width, height);
                } else {
                    let _ = SetWindowPos(
                        self.hwnd,
                        Some(HWND_TOP),
                        0,
                        0,
                        width,
                        height,
                        SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOZORDER,
                    );
                }
                let _ = ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
                invalidate(self.hwnd);
            }
        }
    }

    /// 隐藏候选窗，保留窗口与状态供下一次组合复用。
    pub fn hide(&mut self) {
        if self.hwnd.is_invalid() {
            return;
        }
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
    }

    fn create_window(&mut self, view: &CandidateUiView) {
        unsafe {
            let Ok(module) = GetModuleHandleW(PCWSTR::null()) else {
                return;
            };
            let instance = HINSTANCE(module.0);
            if register_window_class(instance).is_err() {
                return;
            }
            let initial_dpi = GetDpiForSystem().max(BASE_DPI);
            let options = CandidateWindowOptions {
                theme: ThemePreference::Auto,
                dpi: None,
                seconds: None,
                shot_path: None,
            };
            let state = Box::new(CandidateWindowState::new_with_quit(
                view.clone(),
                &options,
                initial_dpi,
                false,
            ));
            if state.view.composition.is_empty() {
                return;
            }
            let (width, height) = state.metrics.panel_size(state.view.panel_rows());
            let state_ptr = Box::into_raw(state);
            let Ok(hwnd) = CreateWindowExW(
                WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
                PCWSTR(window_class_name().as_ptr()),
                PCWSTR::null(),
                WS_POPUP,
                0,
                0,
                width.max(1),
                height.max(1),
                None,
                None,
                Some(instance),
                Some(state_ptr.cast()),
            ) else {
                drop(Box::from_raw(state_ptr));
                return;
            };
            self.hwnd = hwnd;
            self.state_ptr = state_ptr;
        }
    }
}

impl Default for CandidateWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for CandidateWindow {
    fn drop(&mut self) {
        unsafe {
            if !self.hwnd.is_invalid() {
                let _ = DestroyWindow(self.hwnd);
            }
        }
        self.hwnd = HWND::default();
        self.state_ptr = std::ptr::null_mut();
    }
}

/// 把候选窗放到组合区下方并限制在显示器工作区内。
fn place_at(hwnd: HWND, anchor: POINT, width: i32, height: i32) {
    unsafe {
        let rect = RECT {
            left: anchor.x,
            top: anchor.y,
            right: anchor.x + width,
            bottom: anchor.y + height,
        };
        // 超出显示器工作区时回到右下角，避免候选窗不可见。
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(monitor, &mut info).as_bool() {
            return;
        }
        let work = info.rcWork;
        let gap = 4i32;
        let mut left = rect.left;
        let mut top = rect.top + gap;
        if left < work.left {
            left = work.left;
        }
        if left + width > work.right {
            left = (work.right - width).max(work.left);
        }
        if top < work.top {
            top = work.top;
        }
        if top + height > work.bottom {
            top = (work.bottom - height).max(work.top);
        }
        let _ = SetWindowPos(
            hwnd,
            Some(HWND_TOP),
            left,
            top,
            width,
            height,
            SWP_NOACTIVATE,
        );
    }
}

struct CandidateWindowState {
    view: CandidateUiView,
    metrics: CandidateMetrics,
    theme: CandidateUiTheme,
    font: HFONT,
    font_is_stock: bool,
    theme_pref: ThemePreference,
    forced_dpi: Option<u32>,
    quit_on_destroy: bool,
}

impl CandidateWindowState {
    fn new(view: CandidateUiView, options: &CandidateWindowOptions, initial_dpi: u32) -> Self {
        Self::new_with_quit(view, options, initial_dpi, true)
    }

    /// 内部构造：`quit_on_destroy` 为 `false` 时不退出宿主消息循环。
    fn new_with_quit(
        view: CandidateUiView,
        options: &CandidateWindowOptions,
        initial_dpi: u32,
        quit_on_destroy: bool,
    ) -> Self {
        let metrics = CandidateMetrics::new(initial_dpi.max(BASE_DPI));
        let (font, font_is_stock) = create_font(metrics.font_height);
        Self {
            view,
            theme: resolve_theme(options.theme),
            metrics,
            font,
            font_is_stock,
            theme_pref: options.theme,
            forced_dpi: options.dpi,
            quit_on_destroy,
        }
    }

    fn apply_dpi(&mut self, dpi: u32) {
        let dpi = dpi.max(BASE_DPI);
        if dpi == self.metrics.dpi {
            return;
        }
        let metrics = CandidateMetrics::new(dpi);
        let old_font = mem::take(&mut self.font);
        if !self.font_is_stock {
            unsafe {
                let _ = DeleteObject(old_font.into());
            }
        }
        let (font, font_is_stock) = create_font(metrics.font_height);
        self.font = font;
        self.font_is_stock = font_is_stock;
        self.metrics = metrics;
    }

    fn refresh_theme(&mut self) {
        self.theme = resolve_theme(self.theme_pref);
    }

    /// 把一帧快照画到指定 DC；`width`/`height` 为窗口客户区尺寸。
    fn paint(&mut self, hdc: HDC, width: i32, height: i32) {
        // 关键修复（T-034）：此前创建了 `create_font` 的 HFONT 却从未
        // `SelectObject` 进绘制 DC，所有 DrawTextW 都用了 DC 默认字体
        // （现代中文 Windows 上为微软雅黑），字体名改动因此不生效。
        let old_font = unsafe { SelectObject(hdc, self.font.into()) };
        let selected = self.view.selected_on_page();
        let page_rows = self.view.panel_rows();
        let visible = self.view.visible_items();

        paint_background(
            hdc,
            width,
            height,
            self.metrics.corner_radius,
            self.theme.background,
            self.theme.border,
        );

        for index in 0..page_rows {
            let is_selected = selected == Some(index);
            if is_selected {
                // 搜狗风选中块：圆角浅蓝块（与窗口圆角一致的半径）。
                // T-043：改用贴面板左右边框的整行高亮矩形，序号仍按内容行
                // 布局，摆脱"序号贴高亮块左缘"的局促观感。
                fill_round_rect(
                    hdc,
                    to_win_rect(self.metrics.highlight_rect(index)),
                    self.theme.highlight_background,
                    self.metrics.corner_radius,
                );
            }
            let Some(item) = visible.get(index) else {
                continue;
            };

            let row_ui = self.metrics.row_rect(index);
            let marker_color = if is_selected {
                self.theme.highlight_foreground
            } else {
                self.theme.marker
            };
            draw_text(
                hdc,
                &index_marker(index),
                self.metrics.marker_rect(row_ui),
                marker_color,
            );

            // M6-R：网络语缩写候选在主文本后追加 `[网络]` 标注；
            // 标注并入主文本，宽度估算（row_split）自然把它计入。
            let main_owned = display_main_text(item, self.view.translation_mode);
            let (main, secondary) = if self.view.translation_mode && !item.translation.is_empty() {
                (
                    main_owned.as_str(),
                    if item.text == item.translation {
                        ""
                    } else {
                        item.text.as_str()
                    },
                )
            } else {
                (main_owned.as_str(), item.translation.as_str())
            };
            let text_color = if is_selected {
                self.theme.highlight_foreground
            } else {
                self.theme.foreground
            };
            // T-037：动态分栏——译文紧跟主文本（不再固定右侧 1/3 列），
            // 英文译文更靠左、可用宽度更大。
            let (main_rect, translation_rect) = self.metrics.row_split(row_ui, main, secondary);
            draw_text(hdc, main, main_rect, text_color);
            draw_text(hdc, secondary, translation_rect, self.theme.secondary);
        }

        let header_text = if !self.view.composition.is_empty() {
            self.view.composition.as_str()
        } else {
            self.view.pinyin_hint.as_str()
        };
        // T-040：页脚 m/n 翻页指示（总页数 >1 时在面板底部右端显示）。
        if page_rows > 0 {
            if let Some(label) = self.view.footer_label() {
                draw_text_right(
                    hdc,
                    &label,
                    self.metrics.footer_rect(page_rows),
                    self.theme.secondary,
                );
            }
        }
        draw_text(
            hdc,
            header_text,
            self.metrics.header_text_rect(),
            self.theme.foreground,
        );
        if !self.view.pinyin_hint.is_empty() && self.view.pinyin_hint != header_text {
            draw_text(
                hdc,
                &self.view.pinyin_hint,
                self.metrics.header_hint_rect(),
                self.theme.secondary,
            );
        }
        // 恢复旧字体；`paint_window` 每帧用全新兼容 DC，恢复是防御性的。
        unsafe {
            SelectObject(hdc, old_font);
        }
    }
}

impl Drop for CandidateWindowState {
    fn drop(&mut self) {
        if !self.font_is_stock && !self.font.is_invalid() {
            unsafe {
                let _ = DeleteObject(self.font.into());
            }
        }
    }
}

/// 运行候选窗演示并进入消息循环，直到窗口关闭。
pub fn run_candidate_demo(
    view: CandidateUiView,
    options: &CandidateWindowOptions,
) -> Result<(), String> {
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let module = GetModuleHandleW(PCWSTR::null())
            .map_err(|error| format!("获取进程模块句柄失败: {error}"))?;
        let instance = HINSTANCE(module.0);
        register_window_class(instance)?;

        let initial_dpi = options.dpi.unwrap_or_else(|| GetDpiForSystem());
        let state = CandidateWindowState::new(view, options, initial_dpi);
        let (width, height) = state.metrics.panel_size(state.view.panel_rows());
        let state_ptr = Box::into_raw(Box::new(state));

        let hwnd = match CreateWindowExW(
            WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
            PCWSTR(window_class_name().as_ptr()),
            PCWSTR::null(),
            WS_POPUP,
            0,
            0,
            width,
            height,
            None,
            None,
            Some(instance),
            Some(state_ptr.cast()),
        ) {
            Ok(hwnd) => hwnd,
            Err(error) => {
                drop(Box::from_raw(state_ptr));
                return Err(format!("创建候选窗失败: {error}"));
            }
        };

        place_window(hwnd, width, height);
        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        let _ = UpdateWindow(hwnd);

        if let Some(seconds) = options.seconds {
            let milliseconds = seconds.saturating_mul(1000).min(u32::MAX as u64) as u32;
            if SetTimer(Some(hwnd), DEMO_TIMER_ID, milliseconds, None) == 0 {
                let _ = DestroyWindow(hwnd);
                return Err("启动演示计时器失败".to_owned());
            }
        }

        if let Some(path) = &options.shot_path {
            match capture_window_bmp(hwnd) {
                Ok(bytes) => {
                    if let Err(error) = std::fs::write(path, bytes) {
                        let _ = DestroyWindow(hwnd);
                        return Err(format!("写入截图失败: {error}"));
                    }
                    if options.seconds.is_none() {
                        let _ = PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
                    }
                }
                Err(error) => {
                    let _ = DestroyWindow(hwnd);
                    return Err(error);
                }
            }
        }

        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&message);
            let _ = DispatchMessageW(&message);
        }
        Ok(())
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
            unsafe {
                let create = lparam.0 as *const CREATESTRUCTW;
                let state = (*create).lpCreateParams as *mut CandidateWindowState;
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize);
            }
            LRESULT(1)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_PAINT => {
            let mut paint = PAINTSTRUCT::default();
            let hdc = unsafe { BeginPaint(hwnd, &mut paint) };
            if !hdc.is_invalid() {
                let mut client = RECT::default();
                let _ = unsafe { GetClientRect(hwnd, &mut client) };
                paint_window(hwnd, hdc, client.right, client.bottom);
                unsafe {
                    let _ = EndPaint(hwnd, &paint);
                }
            }
            LRESULT(0)
        }
        WM_DPICHANGED => {
            unsafe {
                let dpi = u32::from((wparam.0 & 0xFFFF) as u16);
                if let Some(state) = state_mut(hwnd) {
                    if state.forced_dpi.is_none() {
                        state.apply_dpi(dpi);
                    }
                }
                let rect = &*(lparam.0 as *const RECT);
                let _ = SetWindowPos(
                    hwnd,
                    Some(HWND_TOP),
                    rect.left,
                    rect.top,
                    rect.right - rect.left,
                    rect.bottom - rect.top,
                    SWP_NOZORDER,
                );
            }
            LRESULT(0)
        }
        WM_THEMECHANGED | WM_SETTINGCHANGE => {
            unsafe {
                if let Some(state) = state_mut(hwnd) {
                    state.refresh_theme();
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
        WM_TIMER | WM_CLOSE => {
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            unsafe {
                if let Some(state) = state_mut(hwnd) {
                    let quit = state.quit_on_destroy;
                    drop(Box::from_raw(state as *mut CandidateWindowState));
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                    if quit {
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

fn register_window_class(instance: HINSTANCE) -> Result<(), String> {
    static REGISTERED_ATOM: AtomicU16 = AtomicU16::new(0);
    if REGISTERED_ATOM.load(Ordering::Relaxed) != 0 {
        return Ok(());
    }
    let class = window_class_name();
    let class_def = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: Default::default(),
        hCursor: Default::default(),
        hbrBackground: HBRUSH::default(),
        lpszMenuName: PCWSTR::null(),
        lpszClassName: PCWSTR(class.as_ptr()),
    };
    let atom = unsafe { RegisterClassW(&class_def) };
    if atom == 0 {
        return Err(format!(
            "注册候选窗窗口类失败: {}",
            std::io::Error::last_os_error()
        ));
    }
    REGISTERED_ATOM.store(atom, Ordering::Relaxed);
    Ok(())
}

fn window_class_name() -> &'static [u16] {
    static CLASS_NAME: OnceLock<&'static [u16]> = OnceLock::new();
    CLASS_NAME.get_or_init(|| {
        let wide = to_utf16_null("ZhuYe.CandidateWindow.1");
        Box::leak(wide.into_boxed_slice())
    })
}

unsafe fn state_mut(hwnd: HWND) -> Option<&'static mut CandidateWindowState> {
    unsafe {
        let raw = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
        if raw == 0 {
            None
        } else {
            Some(&mut *(raw as *mut CandidateWindowState))
        }
    }
}

fn place_window(hwnd: HWND, width: i32, height: i32) {
    unsafe {
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(monitor, &mut info).as_bool() {
            return;
        }
        let spacing = 16;
        let x = (info.rcWork.right - width - spacing).max(info.rcWork.left);
        let y = (info.rcWork.top + spacing)
            .min(info.rcWork.bottom - height - spacing)
            .max(info.rcWork.top);
        let _ = SetWindowPos(hwnd, Some(HWND_TOP), x, y, width, height, SWP_NOACTIVATE);
    }
}

fn invalidate(hwnd: HWND) {
    unsafe {
        let _ = InvalidateRect(Some(hwnd), None, false);
    }
}

fn paint_window(hwnd: HWND, hdc: HDC, width: i32, height: i32) {
    unsafe {
        let memory_dc = CreateCompatibleDC(Some(hdc));
        if memory_dc.is_invalid() || width <= 0 || height <= 0 {
            return;
        }
        let bitmap = CreateCompatibleBitmap(hdc, width, height);
        if bitmap.is_invalid() {
            let _ = DeleteDC(memory_dc);
            return;
        }
        let old_bitmap = SelectObject(memory_dc, bitmap.into());
        if let Some(state) = state_mut(hwnd) {
            state.paint(memory_dc, width, height);
        }
        let _ = BitBlt(hdc, 0, 0, width, height, Some(memory_dc), 0, 0, SRCCOPY);
        SelectObject(memory_dc, old_bitmap);
        let _ = DeleteObject(bitmap.into());
        let _ = DeleteDC(memory_dc);
    }
}

fn paint_background(
    hdc: HDC,
    width: i32,
    height: i32,
    radius: i32,
    background: UiColor,
    border: UiColor,
) {
    unsafe {
        let brush = CreateSolidBrush(COLORREF(background.to_colorref()));
        let pen = CreatePen(PS_SOLID, 1, COLORREF(border.to_colorref()));
        if brush.is_invalid() || pen.is_invalid() {
            if !brush.is_invalid() {
                let _ = DeleteObject(brush.into());
            }
            if !pen.is_invalid() {
                let _ = DeleteObject(pen.into());
            }
            let fallback = CreateSolidBrush(COLORREF(background.to_colorref()));
            if !fallback.is_invalid() {
                let rect = RECT {
                    right: width,
                    bottom: height,
                    ..Default::default()
                };
                FillRect(hdc, &rect, fallback);
                let _ = DeleteObject(fallback.into());
            }
            return;
        }
        let old_brush = SelectObject(hdc, brush.into());
        let old_pen = SelectObject(hdc, pen.into());
        // T-037：先以背景色填满整个客户区，再画圆角矩形。此前只画圆角矩形，
        // 圆角外侧四角从未填充——内存 DC 初始（黑/杂色）像素直接透出，
        // 表现为四个角的黑点。垫底后四角为背景色。
        let rect = RECT {
            right: width,
            bottom: height,
            ..Default::default()
        };
        FillRect(hdc, &rect, brush);
        let diameter = radius.saturating_mul(2);
        let _ = RoundRect(hdc, 0, 0, width, height, diameter, diameter);
        SelectObject(hdc, old_brush);
        SelectObject(hdc, old_pen);
        let _ = DeleteObject(brush.into());
        let _ = DeleteObject(pen.into());
    }
}

/// 圆角填充矩形（选中块用）；无边框，圆角半径与窗口一致。
fn fill_round_rect(hdc: HDC, rect: RECT, color: UiColor, radius: i32) {
    unsafe {
        let brush = CreateSolidBrush(COLORREF(color.to_colorref()));
        let pen = CreatePen(PS_NULL, 0, COLORREF(color.to_colorref()));
        if brush.is_invalid() || pen.is_invalid() {
            if !brush.is_invalid() {
                let _ = DeleteObject(brush.into());
            }
            if !pen.is_invalid() {
                let _ = DeleteObject(pen.into());
            }
            return;
        }
        let old_brush = SelectObject(hdc, brush.into());
        let old_pen = SelectObject(hdc, pen.into());
        let diameter = radius.saturating_mul(2);
        let _ = RoundRect(
            hdc,
            rect.left,
            rect.top,
            rect.right,
            rect.bottom,
            diameter,
            diameter,
        );
        SelectObject(hdc, old_brush);
        SelectObject(hdc, old_pen);
        let _ = DeleteObject(brush.into());
        let _ = DeleteObject(pen.into());
    }
}

fn draw_text(hdc: HDC, text: &str, rect: UiRect, color: UiColor) {
    if text.is_empty() || rect.width() <= 0 || rect.height() <= 0 {
        return;
    }
    let mut wide: Vec<u16> = text.encode_utf16().collect();
    let mut rect = to_win_rect(rect);
    unsafe {
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, COLORREF(color.to_colorref()));
        DrawTextW(
            hdc,
            &mut wide,
            &mut rect,
            DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
        );
    }
}

/// 右对齐绘制一行文本（T-040 页脚页码用；不做省略截断）。
fn draw_text_right(hdc: HDC, text: &str, rect: UiRect, color: UiColor) {
    if text.is_empty() || rect.width() <= 0 || rect.height() <= 0 {
        return;
    }
    let mut wide: Vec<u16> = text.encode_utf16().collect();
    let mut rect = to_win_rect(rect);
    unsafe {
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, COLORREF(color.to_colorref()));
        DrawTextW(
            hdc,
            &mut wide,
            &mut rect,
            DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_RIGHT,
        );
    }
}

fn create_font(height: i32) -> (HFONT, bool) {
    unsafe {
        let mut face = [0u16; 32];
        // T-034：候选框字体为宋体（SimSun）。此前字体名虽设为雅黑却从未
        // SelectObject，实际渲染的一直是 DC 默认字体；paint() 已修复选择。
        for (slot, unit) in face.iter_mut().zip("SimSun".encode_utf16().chain(Some(0))) {
            *slot = unit;
        }
        let metrics = LOGFONTW {
            lfHeight: -height,
            lfWeight: FW_NORMAL.0 as i32,
            lfCharSet: DEFAULT_CHARSET,
            lfOutPrecision: OUT_DEFAULT_PRECIS,
            lfClipPrecision: CLIP_DEFAULT_PRECIS,
            lfQuality: CLEARTYPE_QUALITY,
            lfPitchAndFamily: FF_DONTCARE.0 | DEFAULT_PITCH.0,
            lfFaceName: face,
            ..Default::default()
        };
        let font = CreateFontIndirectW(&metrics);
        if !font.is_invalid() {
            return (font, false);
        }
        let stock = GetStockObject(DEFAULT_GUI_FONT);
        (HFONT(stock.0), true)
    }
}

fn to_utf16_null(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

fn to_win_rect(rect: UiRect) -> RECT {
    RECT {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    }
}

/// 捕获窗口客户区为 32 位 BGR（带 54 字节 BMP 头、自上而下像素）。
fn capture_window_bmp(hwnd: HWND) -> Result<Vec<u8>, String> {
    unsafe {
        let window_dc = GetDC(Some(hwnd));
        if window_dc.is_invalid() {
            return Err("获取候选窗 DC 失败".to_owned());
        }
        let mut client = RECT::default();
        if GetClientRect(hwnd, &mut client).is_err() || client.right <= 0 || client.bottom <= 0 {
            let _ = ReleaseDC(Some(hwnd), window_dc);
            return Err("候选窗客户区无效".to_owned());
        }
        let width = client.right;
        let height = client.bottom;

        let memory_dc = CreateCompatibleDC(Some(window_dc));
        if memory_dc.is_invalid() {
            let _ = ReleaseDC(Some(hwnd), window_dc);
            return Err("创建兼容 DC 失败".to_owned());
        }
        let bitmap = CreateCompatibleBitmap(window_dc, width, height);
        if bitmap.is_invalid() {
            let _ = DeleteDC(memory_dc);
            let _ = ReleaseDC(Some(hwnd), window_dc);
            return Err("创建兼容位图失败".to_owned());
        }
        let old_bitmap = SelectObject(memory_dc, bitmap.into());
        if BitBlt(
            memory_dc,
            0,
            0,
            width,
            height,
            Some(window_dc),
            0,
            0,
            SRCCOPY,
        )
        .is_err()
        {
            SelectObject(memory_dc, old_bitmap);
            let _ = DeleteObject(bitmap.into());
            let _ = DeleteDC(memory_dc);
            let _ = ReleaseDC(Some(hwnd), window_dc);
            return Err("复制窗口像素失败".to_owned());
        }

        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                biSizeImage: 0,
                ..Default::default()
            },
            ..Default::default()
        };
        let row_bytes = usize::try_from(width)
            .ok()
            .and_then(|width| width.checked_mul(4))
            .ok_or_else(|| "候选窗宽度溢出".to_owned())?;
        let mut pixels = vec![0u8; row_bytes * usize::try_from(height).unwrap_or(0)];
        let lines = GetDIBits(
            memory_dc,
            bitmap,
            0,
            u32::try_from(height).unwrap_or(0),
            Some(pixels.as_mut_ptr().cast()),
            &mut info,
            DIB_RGB_COLORS,
        );
        if lines == 0 {
            SelectObject(memory_dc, old_bitmap);
            let _ = DeleteObject(bitmap.into());
            let _ = DeleteDC(memory_dc);
            let _ = ReleaseDC(Some(hwnd), window_dc);
            return Err("读取窗口像素失败".to_owned());
        }

        SelectObject(memory_dc, old_bitmap);
        let _ = DeleteObject(bitmap.into());
        let _ = DeleteDC(memory_dc);
        let _ = ReleaseDC(Some(hwnd), window_dc);
        Ok(build_bmp(width, height, &pixels))
    }
}

fn build_bmp(width: i32, height: i32, pixels: &[u8]) -> Vec<u8> {
    let mut bmp = Vec::with_capacity(54 + pixels.len());
    bmp.extend_from_slice(b"BM");
    let file_size = u32::try_from(54 + pixels.len()).unwrap_or(u32::MAX);
    bmp.extend_from_slice(&file_size.to_le_bytes());
    bmp.extend_from_slice(&[0, 0, 0, 0]);
    bmp.extend_from_slice(&54u32.to_le_bytes());

    bmp.extend_from_slice(&40u32.to_le_bytes());
    bmp.extend_from_slice(&width.to_le_bytes());
    bmp.extend_from_slice(&(-height).to_le_bytes());
    bmp.extend_from_slice(&1u16.to_le_bytes());
    bmp.extend_from_slice(&32u16.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    bmp.extend_from_slice(
        &u32::try_from(pixels.len())
            .unwrap_or(u32::MAX)
            .to_le_bytes(),
    );
    bmp.extend_from_slice(&0i32.to_le_bytes());
    bmp.extend_from_slice(&0i32.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    bmp.extend_from_slice(pixels);
    bmp
}

fn resolve_theme(pref: ThemePreference) -> CandidateUiTheme {
    let high_contrast = system_high_contrast_on();
    let kind = match pref {
        ThemePreference::Auto if high_contrast => UiThemeKind::HighContrast,
        // T-030：默认固定浅色，不再跟随系统深浅色（深色仅经显式设置使用）。
        ThemePreference::Auto => UiThemeKind::Light,
        ThemePreference::Light => UiThemeKind::Light,
        ThemePreference::Dark => UiThemeKind::Dark,
        ThemePreference::HighContrast => UiThemeKind::HighContrast,
    };
    if kind == UiThemeKind::HighContrast {
        theme_from_system_colors(system_colors())
    } else {
        theme(kind)
    }
}

fn system_high_contrast_on() -> bool {
    unsafe {
        let mut high_contrast = HIGHCONTRASTW {
            cbSize: std::mem::size_of::<HIGHCONTRASTW>() as u32,
            ..Default::default()
        };
        let ok = SystemParametersInfoW(
            SPI_GETHIGHCONTRAST,
            high_contrast.cbSize,
            Some((&mut high_contrast as *mut HIGHCONTRASTW).cast()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS::default(),
        )
        .is_ok();
        ok && high_contrast.dwFlags.contains(HCF_HIGHCONTRASTON)
    }
}

fn system_colors() -> SystemColors {
    unsafe {
        SystemColors {
            window: GetSysColor(COLOR_WINDOW),
            window_text: GetSysColor(COLOR_WINDOWTEXT),
            gray_text: GetSysColor(COLOR_GRAYTEXT),
            highlight: GetSysColor(COLOR_HIGHLIGHT),
            highlight_text: GetSysColor(COLOR_HIGHLIGHTTEXT),
            btn_face: GetSysColor(COLOR_BTNFACE),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{build_bmp, to_utf16_null};

    #[test]
    fn utf16转换以空字符结尾() {
        let wide = to_utf16_null("候选");
        assert_eq!(wide, vec![0x5019, 0x9009, 0]);
    }

    #[test]
    fn bmp头与像素长度一致() {
        let pixels = vec![0u8; 480 * 320 * 4];
        let bmp = build_bmp(480, 320, &pixels);
        assert_eq!(&bmp[..2], b"BM");
        assert_eq!(bmp.len(), 54 + pixels.len());
        assert_eq!(u32::from_le_bytes(bmp[18..22].try_into().unwrap()), 480);
        assert_eq!(i32::from_le_bytes(bmp[22..26].try_into().unwrap()), -320);
    }
}
