//! Win32 侧的系统信息读取与进程级设置（不含 UI 逻辑）。

use windows::core::PCWSTR;
use windows::Win32::Foundation::{ERROR_SUCCESS, HWND};
use windows::Win32::Graphics::Gdi::{
    GetSysColor, COLOR_BTNFACE, COLOR_GRAYTEXT, COLOR_HIGHLIGHT, COLOR_HIGHLIGHTTEXT, COLOR_WINDOW,
    COLOR_WINDOWTEXT,
};
use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};
use windows::Win32::UI::Accessibility::{HCF_HIGHCONTRASTON, HIGHCONTRASTW};
use windows::Win32::UI::HiDpi::{
    GetDpiForSystem, GetDpiForWindow, SetProcessDpiAwarenessContext,
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows::Win32::UI::WindowsAndMessaging::{
    SystemParametersInfoW, SPI_GETHIGHCONTRAST, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
};

use zhu_ye_ime::candidate_ui::{SystemColors, UiThemeKind};

use crate::wide::to_utf16;

/// 系统深浅色偏好的注册表位置。
const PERSONALIZE_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
/// `AppsUseLightTheme`：`0` 表示应用使用深色。
const APPS_USE_LIGHT_THEME: &str = "AppsUseLightTheme";

/// 启用 Per-Monitor V2 DPI 感知；与候选窗同口径，避免两套缩放语义。
///
/// 失败只说明进程的感知级别已被固定或已设置过，不影响运行，因此忽略结果。
pub fn enable_per_monitor_v2() {
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
}

/// 系统 DPI。
#[must_use]
pub fn system_dpi() -> u32 {
    unsafe { GetDpiForSystem() }
}

/// 窗口所在显示器的 DPI。
#[must_use]
pub fn window_dpi(hwnd: HWND) -> u32 {
    unsafe { GetDpiForWindow(hwnd) }
}

/// 系统是否开启高对比度。
#[must_use]
pub fn high_contrast_on() -> bool {
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

/// 系统是否要求应用使用深色主题。
///
/// 读不到该值时按浅色处理（旧系统无此项），不因此报错。
#[must_use]
pub fn system_dark_mode() -> bool {
    unsafe {
        let sub_key = to_utf16(PERSONALIZE_KEY);
        let value_name = to_utf16(APPS_USE_LIGHT_THEME);
        let mut use_light: u32 = 1;
        let mut size = u32::try_from(std::mem::size_of::<u32>()).unwrap_or(4);
        let status = RegGetValueW(
            HKEY_CURRENT_USER,
            PCWSTR(sub_key.as_ptr()),
            PCWSTR(value_name.as_ptr()),
            RRF_RT_REG_DWORD,
            None,
            Some((&mut use_light as *mut u32).cast()),
            Some(&mut size),
        );
        status == ERROR_SUCCESS && use_light == 0
    }
}

/// 读取系统配色（高对比度主题使用）。
#[must_use]
pub fn system_colors() -> SystemColors {
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

/// 解析设置窗口应使用的主题种类。
///
/// 高对比度优先于深浅色：可访问性设置不得被自定义配色盖过（D-30 的窗口跟随系统）。
#[must_use]
pub fn resolve_theme_kind() -> UiThemeKind {
    if high_contrast_on() {
        UiThemeKind::HighContrast
    } else if system_dark_mode() {
        UiThemeKind::Dark
    } else {
        UiThemeKind::Light
    }
}
