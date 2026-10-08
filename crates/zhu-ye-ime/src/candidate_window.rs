//! 候选窗 Win32 GDI 自绘实现。
//!
//! 本模块负责把 `candidate_ui` 计算出的快照画出来，并提供 TSF 驱动的
//! 受控窗口入口：同线程创建/更新/隐藏/销毁，不占用独立消息循环。

use std::ffi::c_void;
use std::mem;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::OnceLock;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    AlphaBlend, BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateDIBSection,
    CreateFontIndirectW, CreatePen, CreateSolidBrush, DeleteDC, DeleteObject, DrawTextW, EndPaint,
    FillRect, GetCurrentObject, GetDC, GetDIBits, GetMonitorInfoW, GetObjectW, GetStockObject,
    GetSysColor, InvalidateRect, MonitorFromWindow, ReleaseDC, RoundRect, SelectObject, SetBkMode,
    SetTextColor, UpdateWindow, AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
    BLENDFUNCTION, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, COLOR_BTNFACE, COLOR_GRAYTEXT,
    COLOR_HIGHLIGHT, COLOR_HIGHLIGHTTEXT, COLOR_WINDOW, COLOR_WINDOWTEXT, DEFAULT_CHARSET,
    DEFAULT_GUI_FONT, DEFAULT_PITCH, DIB_RGB_COLORS, DT_END_ELLIPSIS, DT_NOPREFIX, DT_RIGHT,
    DT_SINGLELINE, DT_VCENTER, FF_DONTCARE, FW_NORMAL, FW_SEMIBOLD, HBRUSH, HDC, HFONT, HGDIOBJ,
    LOGFONTW, MONITORINFO, MONITOR_DEFAULTTONEAREST, OBJ_FONT, OUT_DEFAULT_PRECIS, PAINTSTRUCT,
    PS_NULL, PS_SOLID, SRCCOPY, TRANSPARENT,
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
    display_main_text, index_marker, theme, theme_from_system_colors, theme_with_candidate,
    CandidateMetrics, CandidateUiTheme, CandidateUiView, SystemColors, UiColor, UiRect,
    UiThemeKind, BASE_DPI,
};

/// 拼音拼注（T-115 后续）：把"词"切分音节并按空格拼接，供候选词**上方**的
/// 小号拼音行显示，如 `生成` + `shengcheng` → `sheng cheng`。
/// 输入非 CJK、拼音为空、无法按标准音节表切分或音节数与字数不符时原样返回。
#[must_use]
fn spell_pinyin(word: &str, pinyin: &str) -> String {
    if word.is_empty() || pinyin.is_empty() || !pinyin.is_ascii() {
        return word.to_owned();
    }
    let word_chars: Vec<char> = word.chars().collect();
    if word_chars.is_empty()
        || word_chars
            .iter()
            .any(|ch| !('一'..='\u{9fff}').contains(ch))
    {
        return word.to_owned();
    }
    let Some(syllables) =
        zhu_ye_core::pinyin::segment_all(&zhu_ye_core::pinyin::SyllableTable::standard(), pinyin)
            .into_iter()
            .next()
    else {
        return word.to_owned();
    };
    if syllables.len() != word_chars.len() {
        return word.to_owned();
    }
    syllables.join(" ")
}

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
    /// 自定义主题文件（T-088 / FR-048）；`Some` 时叠加 `candidate` 节配色。
    pub custom_theme: Option<zhu_ye_core::ThemeFile>,
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
    /// 主题偏好；TSF 侧由 `config.json` 的 `theme` 决定（FR-041），演示工具默认 `Auto`。
    theme_pref: ThemePreference,
    /// 自定义主题文件（T-088 / FR-048）；`Some` 时解析配色叠加 `candidate` 节。
    custom_theme: Option<zhu_ye_core::ThemeFile>,
}

impl CandidateWindow {
    /// 创建候选窗控制器；视图在首次 `update` 时落盘，此处只预留空状态。
    #[must_use]
    pub fn new() -> Self {
        Self::with_theme(ThemePreference::Auto)
    }

    /// 按指定主题偏好创建控制器。
    ///
    /// 偏好只在窗口首次创建时解析为配色；主题属装配项，改动需输入法重新装配才生效
    /// （P-12，设置窗口的"主题"条目即通过重写 `config.json` 达成）。
    #[must_use]
    pub fn with_theme(theme_pref: ThemePreference) -> Self {
        Self {
            hwnd: HWND::default(),
            state_ptr: std::ptr::null_mut(),
            theme_pref,
            custom_theme: None,
        }
    }

    /// 按自定义主题文件创建控制器（T-088 / FR-048）。
    ///
    /// 基础配色按 `Auto` 语义（同 T-030：默认浅色基底，高对比度仍走系统配色），
    /// 主题文件的 `candidate` 节在解析时叠加：缺键回退基础预设。
    #[must_use]
    pub fn with_custom_theme(file: zhu_ye_core::ThemeFile) -> Self {
        Self {
            hwnd: HWND::default(),
            state_ptr: std::ptr::null_mut(),
            theme_pref: ThemePreference::Auto,
            custom_theme: Some(file),
        }
    }

    /// 当前主题偏好。
    #[must_use]
    pub fn theme_preference(&self) -> ThemePreference {
        self.theme_pref
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
                let rows = view.panel_rows();
                let (width, height) =
                    state
                        .metrics
                        .panel_size_for(rows, &view.composition, &view.pinyin_hint);
                let width = width.max(1);
                let height = height.max(1);
                state.view = view;
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
                theme: self.theme_pref,
                dpi: None,
                seconds: None,
                shot_path: None,
                custom_theme: self.custom_theme.clone(),
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
            let (width, height) = state.metrics.panel_size_for(
                state.view.panel_rows(),
                &state.view.composition,
                &state.view.pinyin_hint,
            );
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
    /// 拼音行小号字体（T-115 后续：候选词上方拼音小字）。
    pin_font: HFONT,
    pin_font_is_stock: bool,
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
        // T-126：主文本 16px 加粗（视觉焦点），拼音行 Segoe UI 13px 常规。
        let (font, font_is_stock) =
            create_font(metrics.font_height, FW_SEMIBOLD.0 as i32, "SimSun");
        let (pin_font, pin_font_is_stock) =
            create_font(metrics.pin_font_height, FW_NORMAL.0 as i32, "Segoe UI");
        Self {
            view,
            theme: resolve_with_custom(options.theme, options.custom_theme.as_ref()),
            metrics,
            font,
            font_is_stock,
            pin_font,
            pin_font_is_stock,
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
        let (font, font_is_stock) =
            create_font(metrics.font_height, FW_SEMIBOLD.0 as i32, "SimSun");
        self.font = font;
        self.font_is_stock = font_is_stock;
        let old_pin_font = mem::take(&mut self.pin_font);
        if !self.pin_font_is_stock {
            unsafe {
                let _ = DeleteObject(old_pin_font.into());
            }
        }
        let (pin_font, pin_font_is_stock) =
            create_font(metrics.pin_font_height, FW_NORMAL.0 as i32, "Segoe UI");
        self.pin_font = pin_font;
        self.pin_font_is_stock = pin_font_is_stock;
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
        // T-124：长拼音输入时面板宽按内容扩展，本帧全部矩形按实际
        // 客户区宽度布局（页眉输入串/提示不再按固定 360dp 面板截断）。
        let metrics = self.metrics.with_panel_width(width);
        let old_font = unsafe { SelectObject(hdc, self.font.into()) };
        let selected = self.view.selected_on_page();
        let page_rows = self.view.panel_rows();
        let visible = self.view.visible_items();

        paint_background(
            hdc,
            width,
            height,
            metrics.corner_radius,
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
                    to_win_rect(metrics.highlight_rect(index)),
                    self.theme.highlight_background,
                    metrics.corner_radius,
                );
            }
            let Some(item) = visible.get(index) else {
                continue;
            };

            let row_ui = metrics.row_rect(index);
            let marker_color = if is_selected {
                self.theme.highlight_foreground
            } else {
                self.theme.marker
            };
            // T-122：序号右对齐到标记列右缘（再让出 `marker_text_gap` 间距），
            // 单/双位数字都紧贴候选词、留白稳定在 4-8px 档，不再随数字宽度漂移。
            let mut marker_rect = metrics.marker_rect(row_ui);
            marker_rect.right -= metrics.marker_text_gap;
            draw_text_right(hdc, &index_marker(index), marker_rect, marker_color);

            // M6-R：网络语缩写候选在主文本后追加 `[网络]` 标注；
            // 标注并入主文本，宽度估算（row_split）自然把它计入。
            let base = display_main_text(item, self.view.translation_mode);
            // T-115 后续：候选词拼音行——拼音改到词汇**上方**、小一号字体
            // （仿微软拼音布局，用户点名）；错序纠错候选借此展示正确拼音
            // （如输入 zhagnh 显示 账号 上方 "zhang hao"）。译文层主文本是
            // 英文，拼音行不适用；切分失败（拼音行与原文本相同）不显示。
            // 主文本不再采用内联"词（音节）"拼注，保持纯词汇。
            let pin_text = if self.view.translation_mode {
                String::new()
            } else if !item.pinyin_tone.is_empty() {
                // T-112 后续批四：带调拼音优先（`生成` → `shēng chéng`）；
                // 词级/字级带调表齐备时不再回退无调拼注。
                item.pinyin_tone.clone()
            } else {
                spell_pinyin(&base, &item.pinyin)
            };
            let show_pin = !pin_text.is_empty() && pin_text != base;
            let main = base.as_str();
            let secondary = if self.view.translation_mode && !item.translation.is_empty() {
                if item.text == item.translation {
                    ""
                } else {
                    item.text.as_str()
                }
            } else {
                item.translation.as_str()
            };
            let text_color = if is_selected {
                self.theme.highlight_foreground
            } else {
                self.theme.foreground
            };
            // T-037：动态分栏——译文紧跟主文本（不再固定右侧 1/3 列），
            // 英文译文更靠左、可用宽度更大。
            let (main_col, translation_col) = metrics.row_split(row_ui, main, secondary);
            let (main_rect, translation_rect) = if show_pin {
                // 拼音行占行首上方区（小字体），主文本与译文下移至其下方，
                // 两段互不重叠；无拼音行保持原样（整行垂直居中）。
                // T-126：拼音矩形由 `pin_row_rect` 计算——左缘与汉字列严格
                // 对齐、宽度容纳拼音内容（超出汉字宽时不截断）、底部 = 行顶
                // + 拼音带（字高 + 8px 间距），主文本区随之整体下移。
                let pin_rect = metrics.pin_row_rect(row_ui, main_col, &pin_text);
                let pin_bottom = pin_rect.bottom;
                let _ = unsafe { SelectObject(hdc, self.pin_font.into()) };
                // T-122：拼音行颜色独立为 `theme.pin`；
                // T-126：浅色主题下为辅助提示淡灰 #888888（Segoe UI 渲染声调）。
                draw_text(hdc, &pin_text, pin_rect, self.theme.pin);
                let _ = unsafe { SelectObject(hdc, self.font.into()) };
                (
                    UiRect {
                        left: main_col.left,
                        top: pin_bottom,
                        right: main_col.right,
                        bottom: row_ui.bottom,
                    },
                    UiRect {
                        left: translation_col.left,
                        top: pin_bottom,
                        right: translation_col.right,
                        bottom: row_ui.bottom,
                    },
                )
            } else {
                (main_col, translation_col)
            };
            draw_text(hdc, main, main_rect, text_color);
            draw_text(hdc, secondary, translation_rect, self.theme.secondary);
        }

        let header_text = if !self.view.composition.is_empty() {
            self.view.composition.as_str()
        } else {
            self.view.pinyin_hint.as_str()
        };
        // T-124：页眉提示区按内容分配宽度（长拼音输入时输入串完整优先，
        // 面板随内容扩宽；提示为空/与输入串相同时为 0，输入串占整行）。
        let (_, hint_alloc) = metrics.header_widths(header_text, &self.view.pinyin_hint);
        // T-040：页脚 m/n 翻页指示（总页数 >1 时在面板底部右端显示）。
        if page_rows > 0 {
            if let Some(label) = self.view.footer_label() {
                draw_text_right(
                    hdc,
                    &label,
                    metrics.footer_rect(page_rows),
                    self.theme.secondary,
                );
            }
        }
        draw_text(
            hdc,
            header_text,
            metrics.header_text_rect_with(hint_alloc),
            self.theme.foreground,
        );
        if !self.view.pinyin_hint.is_empty() && self.view.pinyin_hint != header_text {
            draw_text(
                hdc,
                &self.view.pinyin_hint,
                metrics.header_hint_rect_with(hint_alloc),
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
        if !self.pin_font_is_stock && !self.pin_font.is_invalid() {
            unsafe {
                let _ = DeleteObject(self.pin_font.into());
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
        let (width, height) = state.metrics.panel_size_for(
            state.view.panel_rows(),
            &state.view.composition,
            &state.view.pinyin_hint,
        );
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

/// T-074：含 emoji 的文本行经 DirectWrite 彩色路径渲染后预乘 alpha 合成。
/// 字号取自绘制 DC 当前字体（与 GDI 候选行同尺寸）；失败回退 GDI。
fn draw_text(hdc: HDC, text: &str, rect: UiRect, color: UiColor) {
    if text.is_empty() || rect.width() <= 0 || rect.height() <= 0 {
        return;
    }
    if crate::color_text::contains_color_glyph(text) {
        let font_size = current_font_size(hdc);
        if let Some(bmp) = crate::color_text::render_color_text(
            text,
            rect.width(),
            rect.height(),
            font_size,
            color,
        ) {
            alpha_blend_bitmap(hdc, &to_win_rect(rect), &bmp);
            return;
        }
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

/// 读取绘制 DC 当前字体逻辑尺寸（GDI 负 lfHeight ≈ 字符单元高，直接当
/// DirectWrite 字号用；取不到时退 15px 保全路径）。
fn current_font_size(hdc: HDC) -> f32 {
    unsafe {
        let hfont = GetCurrentObject(hdc, OBJ_FONT);
        let mut lf = LOGFONTW::default();
        if !hfont.is_invalid()
            && GetObjectW(
                HGDIOBJ(hfont.0),
                std::mem::size_of::<LOGFONTW>() as i32,
                Some(&mut lf as *mut _ as *mut c_void),
            ) > 0
            && lf.lfHeight != 0
        {
            return lf.lfHeight.unsigned_abs() as f32;
        }
        15.0
    }
}

/// 把彩色位图（预乘 BGRA）经 `AlphaBlend(AC_SRC_ALPHA)` 合成到目标 DC。
fn alpha_blend_bitmap(hdc: HDC, rect: &RECT, bmp: &crate::color_text::ColorBitmap) {
    unsafe {
        let width = bmp.width;
        let height = bmp.height;
        if width <= 0 || height <= 0 {
            return;
        }
        let src_dc = CreateCompatibleDC(Some(hdc));
        if src_dc.is_invalid() {
            return;
        }
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // 自上而下
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits: *mut c_void = std::ptr::null_mut();
        let dib = match CreateDIBSection(
            Some(src_dc),
            &info,
            DIB_RGB_COLORS,
            &mut bits as *mut *mut c_void,
            None,
            0,
        ) {
            Ok(d) => d,
            Err(_) => {
                let _ = DeleteDC(src_dc);
                return;
            }
        };
        if bits.is_null() {
            let _ = DeleteObject(dib.into());
            let _ = DeleteDC(src_dc);
            return;
        }
        let old = SelectObject(src_dc, dib.into());
        std::ptr::copy_nonoverlapping(bmp.pixels.as_ptr(), bits as *mut u8, bmp.pixels.len());
        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: 255,
            AlphaFormat: AC_SRC_ALPHA as u8,
        };
        let _ = AlphaBlend(
            hdc, rect.left, rect.top, width, height, src_dc, 0, 0, width, height, blend,
        );
        SelectObject(src_dc, old);
        let _ = DeleteObject(dib.into());
        let _ = DeleteDC(src_dc);
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

fn create_font(height: i32, weight: i32, face: &str) -> (HFONT, bool) {
    unsafe {
        let mut face_name = [0u16; 32];
        // T-034：候选主文本字体为宋体（SimSun）。此前字体名虽设为雅黑却从未
        // SelectObject，实际渲染的一直是 DC 默认字体；paint() 已修复选择。
        // T-126：拼音行改用 Segoe UI——对 precomposed/组合声调字符（ǐ/ǎ 等）
        // 提供平滑的组合字形，修掉宋体下声调符号偏移割裂的观感。
        for (slot, unit) in face_name.iter_mut().zip(face.encode_utf16().chain(Some(0))) {
            *slot = unit;
        }
        let metrics = LOGFONTW {
            lfHeight: -height,
            // T-122：拼音行半粗（FW_SEMIBOLD）提升小字可读性；主文本保持常规。
            // T-126：主文本常规→ FW_SEMIBOLD（汉字作为视觉焦点加粗）；拼音行
            // 半粗→常规（辅助提示，配合浅灰 #888 与声调字体）。
            lfWeight: weight,
            lfCharSet: DEFAULT_CHARSET,
            lfOutPrecision: OUT_DEFAULT_PRECIS,
            lfClipPrecision: CLIP_DEFAULT_PRECIS,
            lfQuality: CLEARTYPE_QUALITY,
            lfPitchAndFamily: FF_DONTCARE.0 | DEFAULT_PITCH.0,
            lfFaceName: face_name,
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
    // 纯偏好解析；自定义主题文件叠加由 `resolve_with_custom` 承担。
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

/// 主题最终配色：先按偏好取基础主题，再叠加自定义主题文件（T-088 / FR-048）。
///
/// 高对比度由系统接管（D-31），激活时不叠加主题文件；规则与设置窗口一致
/// （`SettingsTheme` 侧在高对比度时同样跳过主题文件）。
fn resolve_with_custom(
    pref: ThemePreference,
    custom: Option<&zhu_ye_core::ThemeFile>,
) -> CandidateUiTheme {
    let base = resolve_theme(pref);
    if system_high_contrast_on() {
        return base; // D-31：高对比度由系统接管，主题文件不覆盖。
    }
    match custom {
        Some(file) => theme_with_candidate(base, &file.candidate),
        None => base,
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
    use super::{
        build_bmp, resolve_theme, resolve_with_custom, spell_pinyin, system_high_contrast_on,
        to_utf16_null, CandidateWindow, ThemePreference,
    };

    #[test]
    fn 拼音拼注逐字插音节() {
        // T-115 后续：全拼候选行展示"词（音节）"，错序纠错候选借此显示正确拼音。
        assert_eq!(spell_pinyin("生成", "shengcheng"), "sheng cheng");
        assert_eq!(spell_pinyin("账号", "zhanghao"), "zhang hao");
        assert_eq!(spell_pinyin("的", "de"), "de");
    }

    #[test]
    fn 拼音拼注异常输入原样返回() {
        // T-115 后续：无拼音/无法切分/字数与音节数不符时不拼注，避免噪声。
        assert_eq!(spell_pinyin("abc", ""), "abc");
        assert_eq!(spell_pinyin("你好", ""), "你好");
        assert_eq!(spell_pinyin("账号", "zhanghaoX"), "账号");
        assert_eq!(spell_pinyin("", "nihao"), "");
        // 非 CJK 主文本不拼注（网络语缩写标注场景）。
        assert_eq!(spell_pinyin("yyds", "yyds"), "yyds");
    }

    #[test]
    fn 主题偏好由构造参数决定且默认自动() {
        assert_eq!(
            CandidateWindow::new().theme_preference(),
            ThemePreference::Auto
        );
        assert_eq!(
            CandidateWindow::default().theme_preference(),
            ThemePreference::Auto
        );
        assert_eq!(
            CandidateWindow::with_theme(ThemePreference::Dark).theme_preference(),
            ThemePreference::Dark
        );
    }

    #[test]
    fn 自定义主题控制器以自动为基底且叠加候选节() {
        use zhu_ye_core::parse_theme_file;
        let file = parse_theme_file(
            r##"{
                "version": 1,
                "name": "取证主题",
                "candidate": { "background": "#112233", "foreground": "#FFEE00" }
            }"##,
        )
        .expect("示例主题应解析");
        let window = CandidateWindow::with_custom_theme(file);
        assert_eq!(
            window.theme_preference(),
            ThemePreference::Auto,
            "自定义主题以 Auto（浅色基底 + 高对比由系统接管）语义运行"
        );
        // 叠加结果：出现主题色即证明文件参与解析；非高对比环境（CI/日常桌面）下
        // 基底为浅色，背景被覆盖为 0x112233。
        if !system_high_contrast_on() {
            let resolved = resolve_with_custom(window.theme_pref, window.custom_theme.as_ref());
            assert_eq!(
                resolved.background,
                crate::candidate_ui::UiColor(0x11_22_33)
            );
            assert_eq!(
                resolved.foreground,
                crate::candidate_ui::UiColor(0xFF_EE_00)
            );
            // 未写键保留浅色基底。
            assert_eq!(
                resolved.marker,
                crate::candidate_ui::theme(crate::candidate_ui::UiThemeKind::Light).marker
            );
        }
        // 无自定义主题时与原路径逐位一致（基础配色不变）。
        let plain = resolve_theme(ThemePreference::Light);
        let base = crate::candidate_ui::theme(crate::candidate_ui::UiThemeKind::Light);
        if !system_high_contrast_on() {
            assert_eq!(plain, base);
        }
    }

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
