//! Win32 侧的系统信息读取、进程级设置与"用系统默认方式打开路径"（不含 UI 逻辑）。

use std::path::{Path, PathBuf};

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
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::{
    SystemParametersInfoW, SHOW_WINDOW_CMD, SPI_GETHIGHCONTRAST, SW_SHOWNORMAL,
    SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
};

use zhu_ye_ui::{SystemColors, UiThemeKind};

use crate::model::OpenTarget;
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

/// 设置程序 exe 所在目录；失败时回退当前目录。
///
/// 安装器把设置窗口与 DLL、基础包放在同一目录，因此 exe 目录就是基础包目录的
/// 等价物（与 TSF 侧 `resolve_base_dir` 的"DLL 同目录"同级）。
#[must_use]
pub fn exe_dir() -> PathBuf {
    use windows::Win32::System::LibraryLoader::GetModuleFileNameW;
    unsafe {
        let mut buffer = [0u16; 4096];
        let length = GetModuleFileNameW(None, &mut buffer);
        if length == 0 || length as usize >= buffer.len() {
            return PathBuf::from(".");
        }
        let mut path = PathBuf::from(String::from_utf16_lossy(&buffer[..length as usize]));
        path.pop();
        path
    }
}

/// 解析「更多设置」的目标路径；未设置 `APPDATA` 时返回描述。
pub fn resolve_target(target: OpenTarget) -> Result<PathBuf, String> {
    match target {
        OpenTarget::ConfigFile => crate::config::config_path()
            .ok_or_else(|| "未设置 APPDATA，无法定位配置文件".to_owned()),
        OpenTarget::DataDir => {
            crate::config::data_dir().ok_or_else(|| "未设置 APPDATA，无法定位数据目录".to_owned())
        }
        OpenTarget::LogDir => Ok(crate::config::acceptance_log_dir()),
    }
}

/// 用系统协议处理器打开一个 URI（如 `ms-settings:keyboard` 系统设置页）。
///
/// 与 `open_path` 同机制（`ShellExecuteW` 的 `open` 动作），只是入参是 URI 字符串而非
/// 文件路径。失败同样以返回值 `<= 32` 判定。
///
/// # Errors
/// 系统拒绝打开时返回带返回码的描述。
pub fn open_uri(uri: &str) -> Result<(), String> {
    let operation = to_utf16("open");
    let target = to_utf16(uri);
    let result = unsafe {
        ShellExecuteW(
            None,
            PCWSTR(operation.as_ptr()),
            PCWSTR(target.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SHOW_WINDOW_CMD(1),
        )
    };
    if result.0 as usize <= 32 {
        Err(format!(
            "系统无法打开「{uri}」（返回码 {}）",
            result.0 as usize
        ))
    } else {
        Ok(())
    }
}

/// 用系统默认方式打开一个路径（文件用关联程序，目录用资源管理器）。
///
/// `ShellExecuteW` 以**返回值 `<= 32`** 表示失败，不设置 `GetLastError`，因此必须检查返回值。
///
/// # Errors
/// 系统拒绝打开时返回带返回码的描述。
pub fn open_path(path: &Path) -> Result<(), String> {
    let operation = to_utf16("open");
    let file = to_utf16(&path.to_string_lossy());
    let result = unsafe {
        ShellExecuteW(
            None,
            PCWSTR(operation.as_ptr()),
            PCWSTR(file.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    let code = result.0 as isize;
    if code <= 32 {
        return Err(format!("系统未能打开该路径（ShellExecute 返回 {code}）"));
    }
    Ok(())
}
