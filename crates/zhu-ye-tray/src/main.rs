#![cfg(windows)]
#![windows_subsystem = "windows"]
//! 竹叶输入法系统托盘（T-112 后续批六）。
//!
//! 进程子系统：`windows_subsystem = "windows"`（T-133）——托盘按 GUI
//! subsystem 链接，登录自启/手动运行时**不分配控制台**；此前缺省 console
//! subsystem，每次启动都闪现一个 cmd 命令行窗口。
//!
//! 输入法本体是 TSF DLL、被各宿主进程加载，无法自持托盘图标（批四已论证
//! `Shell_NotifyIcon` 无常驻进程不可行）。本程序是**常驻的独立进程**
//! （部署位 `C:\Program Files\zhu-ye-ime\bin\zhu-ye-tray.exe`，HKCU Run
//! 自启），按搜狗/QQ 拼音同款做法持有托盘状态图标：
//!
//! - 白"中"（资源 101）= 中文输入模式；
//! - 白"英"（资源 102）= 英文输入模式；
//!
//! 模式来源：引擎在模式切换时把状态原子写入
//! `%APPDATA%\zhu-ye-ime\tray-state`（`zhu_ye_core::tray_state`，单行
//! `Chinese`/`English`），本程序每 500ms 轮询该文件更新图标（原子写保证
//! 不会读到半文件）。托盘进程退出不影响输入法本体（独立进程、只读状态
//! 文件、失败静默降级）。
//!
//! 自第十四期（T-151，FR-070）起，托盘同时承担**工具箱面板的全局呼出热键**
//! 宿主：设置窗口"关窗即退"，唯有托盘进程常驻（HKCU Run 自启），注册表
//! `RegisterHotKey` 必须由它持有时才可靠。热键按下 → `WM_HOTKEY` →
//! 全局单面板互斥（窗口类 `ZhuYeSettingsPanel` 唯一）：已打开则关闭（toggle），
//! 未打开则按上次页签/位置记忆创建（详见 S-30、S-31、S-33）。

use std::ffi::c_void;
use std::sync::Mutex;
use std::time::SystemTime;

use windows::core::{w, PCWSTR, PWSTR};
use windows::Win32::Foundation::{GetLastError, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::System::LibraryLoader::{GetModuleFileNameW, GetModuleHandleW};
use windows::Win32::System::Threading::{
    CreateMutexW, CreateProcessW, PROCESS_CREATION_FLAGS, PROCESS_INFORMATION, STARTUPINFOW,
};
use windows::Win32::UI::HiDpi::{
    SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS,
};
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyIcon, DestroyMenu,
    DispatchMessageW, FindWindowW, GetCursorPos, GetMessageW, GetSystemMetrics, LoadImageW,
    PostMessageW, PostQuitMessage, RegisterClassW, SetTimer, TrackPopupMenu, TranslateMessage,
    CW_USEDEFAULT, HICON, IMAGE_ICON, LR_DEFAULTCOLOR, MF_GRAYED, MF_SEPARATOR, MF_STRING, MSG,
    SM_CXSMICON, SM_CYSMICON, TPM_BOTTOMALIGN, TPM_RIGHTBUTTON, WINDOW_EX_STYLE, WINDOW_STYLE,
    WM_APP, WM_CLOSE, WM_COMMAND, WM_CONTEXTMENU, WM_DESTROY, WM_HOTKEY, WM_LBUTTONDBLCLK,
    WM_LBUTTONUP, WM_RBUTTONUP, WM_TIMER, WNDCLASSW,
};

use zhu_ye_settings::panel::PanelView;
use zhu_ye_settings::panel_window::{self, PANEL_CLASS_NAME};

/// 托盘图标资源 ID（与 build.rs 的 101/102 对应）。
const ICON_ZH: u16 = 101;
const ICON_EN: u16 = 102;

/// 托盘标识与回调消息。
const TRAY_ID: u32 = 1;
const WM_TRAY: u32 = WM_APP + 1;

/// 状态轮询周期。
const POLL_MS: u32 = 500;

/// 单实例互斥名（会话级：每个登录用户一个托盘，与 HKCU Run 自启一致）。
const MUTEX_NAME: &str = "Local\\ZhuYeImeTraySingleton";
const CLASS_NAME: &str = "ZhuYeTrayHostWindow";

/// 菜单项 ID。
const MENU_OPEN_SETTINGS: usize = 1;
const MENU_EXIT: usize = 2;

/// 面板呼出热键的注册 ID（本进程内唯一，用于 RegisterHotKey/UnregisterHotKey 配对）。
const PANEL_HOTKEY_ID: i32 = 0x5A7A;

/// 托盘运行状态（经全局静态供窗口回调访问；窗口句柄以指针值存储以保持 Send）。
struct TrayState {
    window: isize,
    icon_mode: String,
    last_mtime: Option<SystemTime>,
}

static TRAY: Mutex<Option<TrayState>> = Mutex::new(None);

fn mode_to_icon(mode: &str) -> u16 {
    if mode == zhu_ye_core::tray_state::MODE_ENGLISH {
        ICON_EN
    } else {
        ICON_ZH
    }
}

fn mode_tip(mode: &str) -> PCWSTR {
    if mode == zhu_ye_core::tray_state::MODE_ENGLISH {
        w!("竹叶输入法（英文）")
    } else {
        w!("竹叶输入法（中文）")
    }
}

/// 状态文件路径：`%APPDATA%\zhu-ye-ime\tray-state`。
fn state_path() -> Option<std::path::PathBuf> {
    std::env::var_os("APPDATA").map(|root| {
        std::path::PathBuf::from(root)
            .join("zhu-ye-ime")
            .join(zhu_ye_core::tray_state::TRAY_STATE_FILE_NAME)
    })
}

/// 读当前模式；文件缺失/损坏时回退中文标识（保持图标为"中"）。
fn read_mode() -> String {
    let Some(path) = state_path() else {
        return zhu_ye_core::tray_state::MODE_CHINESE.to_string();
    };
    let Ok(text) = std::fs::read_to_string(path) else {
        return zhu_ye_core::tray_state::MODE_CHINESE.to_string();
    };
    zhu_ye_core::tray_state::parse_tray_state(&text)
        .unwrap_or(zhu_ye_core::tray_state::MODE_CHINESE)
        .to_string()
}

/// 从内嵌资源加载小图标（尺寸按系统小图标指标，DPI 自适应）。
fn load_icon(id: u16) -> windows::core::Result<HICON> {
    unsafe {
        let hinstance = GetModuleHandleW(None)?;
        let cx = GetSystemMetrics(SM_CXSMICON);
        let cy = GetSystemMetrics(SM_CYSMICON);
        let handle = LoadImageW(
            Some(hinstance.into()),
            PCWSTR(id as *const u16),
            IMAGE_ICON,
            cx,
            cy,
            LR_DEFAULTCOLOR,
        )?;
        Ok(HICON(handle.0))
    }
}

/// 注册（NIM_ADD）或更新（NIM_MODIFY）托盘图标；加载失败的旧 HICON 由系统持有。
fn set_tray_icon(window: HWND, mode: &str, modify: bool) -> windows::core::Result<()> {
    let icon = load_icon(mode_to_icon(mode))?;
    let tip = mode_tip(mode);
    let tip_utf16: Vec<u16> = unsafe {
        let mut len = 0;
        while *tip.as_ptr().add(len) != 0 {
            len += 1;
        }
        std::slice::from_raw_parts(tip.as_ptr(), len).to_vec()
    };
    let mut data: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    data.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    data.hWnd = window;
    data.uID = TRAY_ID;
    data.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
    data.uCallbackMessage = WM_TRAY;
    data.hIcon = icon;
    let n = tip_utf16.len().min(data.szTip.len() - 1);
    data.szTip[..n].copy_from_slice(&tip_utf16[..n]);
    let flags = if modify { NIM_MODIFY } else { NIM_ADD };
    let ok = unsafe { Shell_NotifyIconW(flags, &data) }.as_bool();
    // LoadImage 返回的图标句柄在交给 Shell 后即可释放（系统已取得副本）。
    unsafe {
        let _ = DestroyIcon(icon);
    }
    if ok {
        Ok(())
    } else {
        Err(windows::core::Error::from_win32())
    }
}

/// 移除托盘图标（退出路径）。
fn remove_tray_icon(window: HWND) {
    let mut data: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    data.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    data.hWnd = window;
    data.uID = TRAY_ID;
    unsafe {
        let _ = Shell_NotifyIconW(NIM_DELETE, &data);
    }
}

/// 解析 `config.json` 的 `panel_hotkey`；未配置/解析失败回默认 `ctrl+alt+e`（S-30）。
fn panel_hotkey_spec() -> (u32, u16) {
    use zhu_ye_core::hotkey::{parse_hotkey, DEFAULT_PANEL_HOTKEY};
    let fallback = || parse_hotkey(DEFAULT_PANEL_HOTKEY).expect("默认热键必须可解析");
    let configured = zhu_ye_settings::config::config_path()
        .and_then(|path| zhu_ye_core::load_config(&path).0.panel_hotkey);
    match configured.as_deref().and_then(parse_hotkey) {
        Some(spec) => (spec.modifiers, spec.vk),
        None => {
            let spec = fallback();
            (spec.modifiers, spec.vk)
        }
    }
}

/// 注册面板呼出热键（FR-070）。返回是否注册成功；失败（如被其他程序占用）时
/// 写一行日志，托盘其余功能照常。
fn register_panel_hotkey(window: HWND) -> bool {
    let (modifiers, vk) = panel_hotkey_spec();
    let result = unsafe {
        RegisterHotKey(
            Some(window),
            PANEL_HOTKEY_ID,
            HOT_KEY_MODIFIERS(modifiers),
            u32::from(vk),
        )
    };
    if result.is_ok() {
        return true;
    }
    // 无控制台（windows_subsystem）：失败原因落到 `%APPDATA%\zhu-ye-ime\panel-hotkey.log`。
    if let Some(mut path) = std::env::var_os("APPDATA").map(std::path::PathBuf::from) {
        path.push("zhu-ye-ime");
        let _ = std::fs::create_dir_all(&path);
        path.push("panel-hotkey.log");
        let message = format!(
            "[error] 面板呼出热键注册失败（modifiers={modifiers:#x}, vk={vk:#x}），可能被其他程序占用；静默降级为仅托盘菜单可用\n",
        );
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .and_then(|mut file| std::io::Write::write_all(&mut file, message.as_bytes()));
    }
    false
}

/// 呼出热键递达：面板已打开（任意进程）→ 关闭；未打开 → 按上次页签/位置创建
/// （S-31 全局单面板：窗口类 `ZhuYeSettingsPanel` 全局唯一）。
fn toggle_panel() {
    let class: Vec<u16> = PANEL_CLASS_NAME.encode_utf16().collect();
    match unsafe { FindWindowW(PCWSTR(class.as_ptr()), None) } {
        Ok(existing) if !existing.is_invalid() => {
            // 已打开：PostMessage(WM_CLOSE) 关闭（面板侧保持不变动窗口语义）。
            unsafe {
                let _ = PostMessageW(Some(existing), WM_CLOSE, WPARAM(0), LPARAM(0));
            }
        }
        _ => {
            // 未打开：按记忆的页签/位置创建；托盘为宿主时不通知面板关闭事件。
            let kind = panel_window::remembered_kind();
            if let Err(error) = panel_window::PanelWindow::open(PanelView::new(kind), None) {
                if let Some(mut path) = std::env::var_os("APPDATA").map(std::path::PathBuf::from) {
                    path.push("zhu-ye-ime");
                    path.push("panel-hotkey.log");
                    let message = format!("[error] 呼出面板失败：{error}\n",);
                    let _ = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(path)
                        .and_then(|mut file| {
                            std::io::Write::write_all(&mut file, message.as_bytes())
                        });
                }
            }
        }
    }
}

/// 打开设置窗口：与托盘同目录的 `zhu-ye-settings.exe`（worker copy-exe 部署位）。
fn open_settings() {
    let Ok(module) = (unsafe { GetModuleHandleW(None) }) else {
        return;
    };
    let mut path = [0u16; 1024];
    let written = unsafe { GetModuleFileNameW(Some(module), &mut path) };
    if written == 0 {
        return;
    }
    let exe =
        String::from_utf16_lossy(&path[..path.iter().position(|&c| c == 0).unwrap_or(path.len())]);
    let mut settings = std::path::PathBuf::from(&exe);
    settings.set_file_name("zhu-ye-settings.exe");
    if !settings.exists() {
        return;
    }
    let mut command_line = settings
        .to_string_lossy()
        .encode_utf16()
        .collect::<Vec<u16>>();
    command_line.push(0);
    let startup = STARTUPINFOW {
        cb: std::mem::size_of::<STARTUPINFOW>() as u32,
        ..Default::default()
    };
    let mut info = PROCESS_INFORMATION::default();
    unsafe {
        if CreateProcessW(
            None,
            Some(PWSTR(command_line.as_mut_ptr())),
            None,
            None,
            false,
            PROCESS_CREATION_FLAGS(0),
            None,
            None,
            &startup,
            &mut info,
        )
        .is_ok()
        {
            let _ = windows::Win32::Foundation::CloseHandle(info.hThread);
            let _ = windows::Win32::Foundation::CloseHandle(info.hProcess);
        }
    }
}

/// 右键菜单（左键单击同弹，便于查看当前模式）。
fn show_menu(window: HWND) {
    let mode = read_mode();
    let status = if mode == zhu_ye_core::tray_state::MODE_ENGLISH {
        w!("模式：英文输入")
    } else {
        w!("模式：中文输入")
    };
    let Ok(menu) = (unsafe { CreatePopupMenu() }) else {
        return;
    };
    unsafe {
        let _ = AppendMenuW(menu, MF_GRAYED, 0, status);
        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
        let _ = AppendMenuW(menu, MF_STRING, MENU_OPEN_SETTINGS, w!("打开设置(&S)…"));
        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
        let _ = AppendMenuW(menu, MF_STRING, MENU_EXIT, w!("退出(&X)"));
        let mut point = POINT::default();
        let _ = GetCursorPos(&mut point);
        let _ = TrackPopupMenu(
            menu,
            TPM_RIGHTBUTTON | TPM_BOTTOMALIGN,
            point.x,
            point.y,
            None,
            window,
            None,
        );
        let _ = DestroyMenu(menu);
    }
}

/// 轮询状态文件：mtime 变化才读内容，模式变化才换图标（原子写保证内容完整）。
fn poll() {
    let Some(path) = state_path() else {
        return;
    };
    let Ok(meta) = std::fs::metadata(&path) else {
        return; // 状态文件尚未生成（无宿主激活过）→ 保持当前图标。
    };
    let mtime = meta.modified().unwrap_or(std::time::UNIX_EPOCH);
    let mut guard = match TRAY.lock() {
        Ok(g) => g,
        Err(_) => return,
    };
    let Some(state) = guard.as_mut() else {
        return;
    };
    if state.last_mtime == Some(mtime) {
        return;
    }
    state.last_mtime = Some(mtime);
    let mode = read_mode();
    if mode != state.icon_mode {
        let window = HWND(state.window as *mut c_void);
        if set_tray_icon(window, &mode, true).is_ok() {
            state.icon_mode = mode;
        }
    }
}

fn main() -> windows::core::Result<()> {
    // 高 DPI：托盘尺寸按系统小图标缩放（失败静默，图标仍可用默认尺寸）。
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }

    // 单实例：已有托盘在跑则第二个进程直接返回 0（不弹窗、不重复注册图标）。
    // 互斥句柄必须保持打开到进程结束：初版创建后立即 CloseHandle，导致互斥对象
    // 在第一个进程里已被销毁，第二个进程会新建同名对象（GetLastError≠183）并照常
    // 运行——用户实测托盘出现两个图标。HANDLE 无 Drop，`_mutex` 绑定在 main 作用域
    // 内一直保活，句柄只随进程退出由系统回收。
    let mutex_name: Vec<u16> = MUTEX_NAME.encode_utf16().collect();
    unsafe {
        let _mutex = CreateMutexW(None, false, PCWSTR(mutex_name.as_ptr()))?;
        let already = GetLastError().0 == 183; // ERROR_ALREADY_EXISTS
        if already {
            return Ok(());
        }
    }

    let hinstance = unsafe { GetModuleHandleW(None)? };
    let class_name: Vec<u16> = CLASS_NAME.encode_utf16().collect();
    unsafe {
        let class_def = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: hinstance.into(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };
        if RegisterClassW(&class_def) == 0 {
            return Err(windows::core::Error::from_win32());
        }
    }

    let window = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(class_name.as_ptr()),
            PCWSTR::null(),
            WINDOW_STYLE(0),
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            0,
            0,
            None,
            None,
            Some(hinstance.into()),
            None,
        )?
    };
    if window.0.is_null() {
        return Err(windows::core::Error::from_win32());
    }

    let mode = read_mode();
    set_tray_icon(window, &mode, false)?;
    // 面板呼出热键（FR-070）：注册失败（占用/系统拒绝）不阻断托盘启动。
    register_panel_hotkey(window);
    let mut state = TrayState {
        window: window.0 as isize,
        icon_mode: mode,
        last_mtime: None,
    };
    // 启动即记一次 mtime，避免第一帧重复更新图标。
    if let Some(path) = state_path() {
        if let Ok(meta) = std::fs::metadata(path) {
            state.last_mtime = meta.modified().ok();
        }
    }
    *TRAY.lock().unwrap() = Some(state);

    let _ = unsafe { SetTimer(Some(window), 1, POLL_MS, None) };
    unsafe {
        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
    Ok(())
}

unsafe extern "system" fn window_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_TRAY => match lparam.0 as u32 {
            WM_LBUTTONUP | WM_RBUTTONUP | WM_CONTEXTMENU => show_menu(window),
            WM_LBUTTONDBLCLK => open_settings(),
            _ => {}
        },
        WM_COMMAND => match wparam.0 & 0xFFFF {
            id if id == MENU_OPEN_SETTINGS => open_settings(),
            id if id == MENU_EXIT => {
                remove_tray_icon(window);
                unsafe {
                    let _ = UnregisterHotKey(Some(window), PANEL_HOTKEY_ID);
                }
                PostQuitMessage(0);
            }
            _ => {}
        },
        WM_HOTKEY => toggle_panel(),
        WM_TIMER => poll(),
        WM_DESTROY => {
            unsafe {
                let _ = UnregisterHotKey(Some(window), PANEL_HOTKEY_ID);
            }
            PostQuitMessage(0);
        }
        _ => return DefWindowProcW(window, message, wparam, lparam),
    }
    LRESULT(0)
}
