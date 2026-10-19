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
//! - 白"英"（资源 102）= 英文输入模式。
//!
//! 模式来源：引擎在模式切换时把状态原子写入
//! `%APPDATA%\zhu-ye-ime\tray-state`（`zhu_ye_core::tray_state`，单行
//! `Chinese`/`English`），本程序每 500ms 轮询该文件更新图标（原子写保证
//! 不会读到半文件）；单击托盘弹菜单时**同步再读一次**，菜单里的状态行
//! 与图标因此即点即刷新。托盘进程退出不影响输入法本体（独立进程、只读
//! 状态文件、失败静默降级）。
//!
//! 自第十四期（T-151，FR-070）起，托盘同时承担**工具箱面板的全局呼出热键**
//! 宿主：设置窗口"关窗即退"，唯有托盘进程常驻（HKCU Run 自启），注册表
//! `RegisterHotKey` 必须由它持有时才可靠。热键按下 → `WM_HOTKEY` →
//! 全局单面板互斥（窗口类 `ZhuYeSettingsPanel` 唯一）：已打开则关闭（toggle），
//! 未打开则按上次页签/位置记忆创建（详见 S-30、S-31、S-33）。
//!
//! 第十五期（T-169，FR-074）补齐**鼠标入口**：菜单新增"工具箱(&T)…"（双击托盘
//! 仍是打开设置，保持用户既有习惯）。面板创建/关闭一律投递到自持的**串行任务
//! 队列线程**（本文件 [`task`]）执行，托盘主线程只读"面板已开"缓存并投递任务——
//! 主线程永不被面板消息循环阻塞，面板销毁中途也不会被重入 toggle。

mod task;

use std::ffi::c_void;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use std::time::SystemTime;

use windows::core::{w, PCWSTR, PWSTR};
use windows::Win32::Foundation::{GetLastError, HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::System::LibraryLoader::{GetModuleFileNameW, GetModuleHandleW};
use windows::Win32::System::Threading::{
    CreateMutexW, CreateProcessW, PROCESS_CREATION_FLAGS, PROCESS_INFORMATION, STARTUPINFOW,
};
use windows::Win32::UI::HiDpi::{
    SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    keybd_event, RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, KEYBD_EVENT_FLAGS,
    KEYEVENTF_KEYUP, VK_CONTROL, VK_F24, VK_SPACE,
};
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CallNextHookEx, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyIcon,
    DestroyMenu, DispatchMessageW, GetCursorPos, GetMessageW, GetSystemMetrics, LoadImageW,
    PostQuitMessage, RegisterClassW, SetTimer, SetWindowsHookExW, TrackPopupMenu, TranslateMessage,
    UnhookWindowsHookEx, CW_USEDEFAULT, HHOOK, HICON, IMAGE_ICON, KBDLLHOOKSTRUCT, LR_DEFAULTCOLOR,
    MF_GRAYED, MF_SEPARATOR, MF_STRING, MSG, SM_CXSMICON, SM_CYSMICON, TPM_BOTTOMALIGN,
    TPM_RIGHTBUTTON, WH_KEYBOARD_LL, WINDOW_EX_STYLE, WINDOW_STYLE, WM_APP, WM_CONTEXTMENU,
    WM_DESTROY, WM_ENDSESSION, WM_HOTKEY, WM_KEYDOWN, WM_LBUTTONDBLCLK, WM_LBUTTONUP, WM_RBUTTONUP,
    WM_SYSKEYDOWN, WM_TIMER, WNDCLASSW,
};

use crate::task::PanelAction;

/// 托盘图标资源 ID（与 build.rs 的 101/102 对应）。
const ICON_ZH: u16 = 101;
const ICON_EN: u16 = 102;

/// 托盘标识与回调消息。
const TRAY_ID: u32 = 1;
const WM_TRAY: u32 = WM_APP + 1;
/// 串行任务队列线程回报"一个面板任务已完成"（见 [`task`]）。
const WM_TASK_DONE: u32 = WM_APP + 2;

/// 状态轮询周期。
const POLL_MS: u32 = 500;

/// 从托盘菜单「工具箱」呼出面板后，Esc 关闭的窗口期（T-169，毫秒）：面板小胜前台
/// 并接管键盘这么久，鼠标进入面板或到点即交还，回落"点格即上屏、面板不抢焦"。
const PANEL_ESCAPE_MS: u32 = 1500;

/// 单实例互斥名（会话级：每个登录用户一个托盘，与 HKCU Run 自启一致）。
const MUTEX_NAME: &str = "Local\\ZhuYeImeTraySingleton";
const CLASS_NAME: &str = "ZhuYeTrayHostWindow";

/// 菜单项 ID。
const MENU_OPEN_PANEL: usize = 1;
const MENU_OPEN_SETTINGS: usize = 2;
const MENU_EXIT: usize = 3;

/// 面板呼出热键的注册 ID（本进程内唯一，用于 RegisterHotKey/UnregisterHotKey 配对）。
const PANEL_HOTKEY_ID: i32 = 0x5A7A;

/// T-165：Ctrl 按下状态（LL 键盘钩子回调维护；钩子在托盘消息线程单线程
/// 执行，原子类型仅为跨线程防御性正确）。
static CTRL_PRESSED: AtomicBool = AtomicBool::new(false);
/// T-165：LL 键盘钩子句柄（指针值存储保持 Send；托盘常驻期间持有）。
static KEYBOARD_HOOK: Mutex<Option<isize>> = Mutex::new(None);

/// 托盘运行状态（经全局静态供窗口回调访问；窗口句柄以指针值存储以保持 Send）。
struct TrayState {
    window: isize,
    icon_mode: String,
    last_mtime: Option<SystemTime>,
    /// 工具箱面板当前是否打开（由任务线程回报维护，托盘线程只读，T-169）。
    panel_open: bool,
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
    log_panel_hotkey(&format!(
        "[error] 面板呼出热键注册失败（modifiers={modifiers:#x}, vk={vk:#x}），可能被其他程序占用；静默降级为托盘菜单可用\n",
    ));
    false
}

/// 追加一行面板相关日志（`%APPDATA%\zhu-ye-ime\panel-hotkey.log`，失败静默）。
fn log_panel_hotkey(message: &str) {
    let Some(mut path) = std::env::var_os("APPDATA").map(std::path::PathBuf::from) else {
        return;
    };
    path.push("zhu-ye-ime");
    let _ = std::fs::create_dir_all(&path);
    path.push("panel-hotkey.log");
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| std::io::Write::write_all(&mut file, message.as_bytes()));
}

/// T-165：LL 键盘钩子回调——跟踪 Ctrl 按下状态；检测到 Ctrl+Space 时注入
/// 无修饰转义键 F24（竹叶 TSF 识别后切中英）。**不吞键**（CallNextHookEx
/// 放行），系统把 Ctrl+Space 的保留键路由照常给搜狗，双方中英切换互不干扰。
unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && lparam.0 != 0 {
        let kbd = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
        let vk = kbd.vkCode as u16;
        let msg = wparam.0 as u32;
        let down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
        if vk == VK_CONTROL.0 {
            CTRL_PRESSED.store(down, std::sync::atomic::Ordering::SeqCst);
        } else if vk == VK_SPACE.0 && down && CTRL_PRESSED.load(std::sync::atomic::Ordering::SeqCst)
        {
            keybd_event(VK_F24.0 as u8, 0, KEYBD_EVENT_FLAGS(0), 0);
            keybd_event(VK_F24.0 as u8, 0, KEYEVENTF_KEYUP, 0);
        }
    }
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

/// 安装 LL 键盘钩子（T-165）。失败（罕见）静默降级为仅 Shift 切换。
fn install_keyboard_hook() -> bool {
    let hmod = unsafe { GetModuleHandleW(None) }
        .ok()
        .map(|m| HINSTANCE(m.0));
    match unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), hmod, 0) } {
        Ok(hook) => {
            if let Ok(mut guard) = KEYBOARD_HOOK.lock() {
                *guard = Some(hook.0 as isize);
            }
            true
        }
        Err(err) => {
            log_panel_hotkey(&format!("[error] 键盘钩子安装失败 {err:?}\n"));
            false
        }
    }
}

/// 注销 LL 键盘钩子（退出时）。
fn uninstall_keyboard_hook() {
    if let Ok(mut guard) = KEYBOARD_HOOK.lock() {
        if let Some(hook) = guard.take() {
            let _ = unsafe { UnhookWindowsHookEx(HHOOK(hook as *mut c_void)) };
        }
    }
}

/// 面板已开（本进程任务线程持有）时请求关闭；未开则投递创建任务（T-169 串行队列）。
fn request_panel_toggle() {
    let open = TRAY
        .lock()
        .ok()
        .and_then(|guard| guard.as_ref().map(|state| state.panel_open))
        .unwrap_or(false);
    request_panel(if open {
        PanelAction::Close
    } else {
        PanelAction::Open
    });
}

/// 投递一个面板任务（热键与菜单「工具箱」两个入口共用）。
fn request_panel(action: PanelAction) {
    if !task::dispatch(action) {
        log_panel_hotkey("[error] 工具箱任务队列线程不可用，面板打开/关闭请求被丢弃\n");
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

/// 弹出托盘菜单（左键/右键单击）。弹出前同步刷新模式，保证状态行与图标
/// 即点即见新值；菜单内**只收集动作**，`TrackPopupMenu` 返回（模态循环结束）
/// 后再执行——面板窗口需要自己的消息循环，不能在菜单模态循环里创建（T-169）。
fn show_menu(window: HWND) {
    refresh_mode(window);
    let mode = read_mode();
    let status = if mode == zhu_ye_core::tray_state::MODE_ENGLISH {
        w!("模式：英文输入")
    } else {
        w!("模式：中文输入")
    };
    let Ok(menu) = (unsafe { CreatePopupMenu() }) else {
        return;
    };
    let mut point = POINT::default();
    unsafe {
        let _ = AppendMenuW(menu, MF_GRAYED, 0, status);
        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
        let _ = AppendMenuW(menu, MF_STRING, MENU_OPEN_PANEL, w!("工具箱(&T)…"));
        let _ = AppendMenuW(menu, MF_STRING, MENU_OPEN_SETTINGS, w!("打开设置(&S)…"));
        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
        let _ = AppendMenuW(menu, MF_STRING, MENU_EXIT, w!("退出(&X)"));
        let _ = GetCursorPos(&mut point);
    }
    let command = unsafe {
        TrackPopupMenu(
            menu,
            TPM_RIGHTBUTTON | TPM_BOTTOMALIGN,
            point.x,
            point.y,
            None,
            window,
            None,
        )
        .0
    };
    let _ = unsafe { DestroyMenu(menu) };
    // 模态菜单已收起：这时才允许创建面板。
    #[allow(clippy::cast_sign_loss)]
    let picked = command as usize;
    if picked == MENU_OPEN_PANEL {
        request_panel_toggle();
    } else if picked == MENU_OPEN_SETTINGS {
        open_settings();
    } else if picked == MENU_EXIT {
        request_exit(window);
    }
}

/// 退出：移除图标、注销热键、结束任务队列线程后退出消息循环。
fn request_exit(window: HWND) {
    remove_tray_icon(window);
    unsafe {
        let _ = UnregisterHotKey(Some(window), PANEL_HOTKEY_ID);
    }
    uninstall_keyboard_hook();
    task::request_shutdown();
    unsafe { PostQuitMessage(0) };
}

/// 同步刷新模式与图标：读状态文件，模式变化才 `NIM_MODIFY`（单击/轮询共用）。
fn refresh_mode(window: HWND) {
    let mode = read_mode();
    let mut guard = match TRAY.lock() {
        Ok(g) => g,
        Err(_) => return,
    };
    let Some(state) = guard.as_mut() else {
        return;
    };
    if mode != state.icon_mode && set_tray_icon(window, &mode, true).is_ok() {
        state.icon_mode = mode;
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
    // T-165：LL 键盘钩子（Ctrl+Space 检测 + F24 转义注入，不吞键、不抢搜狗）。
    install_keyboard_hook();
    let mut state = TrayState {
        window: window.0 as isize,
        icon_mode: mode,
        last_mtime: None,
        panel_open: false,
    };
    // 启动即记一次 mtime，避免第一帧重复更新图标。
    if let Some(path) = state_path() {
        if let Ok(meta) = std::fs::metadata(path) {
            state.last_mtime = meta.modified().ok();
        }
    }
    *TRAY.lock().unwrap() = Some(state);
    // 串行任务队列线程（T-169）：面板的创建/关闭全在它上面执行。
    if let Err(error) = task::start(window) {
        log_panel_hotkey(&format!(
            "[error] 工具箱任务队列线程启动失败：{error}；降级为仅热键可用\n"
        ));
    }

    let _ = unsafe { SetTimer(Some(window), 1, POLL_MS, None) };
    unsafe {
        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
    task::request_shutdown();
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
            // 单击（左/右）= 菜单：状态行即点即刷新，动作在菜单收起后执行（T-169）。
            WM_LBUTTONUP | WM_RBUTTONUP | WM_CONTEXTMENU => show_menu(window),
            // 双击 = 打开设置（T-169：双击语义按用户要求保持不变，不占用给工具箱；
            // 工具箱鼠标入口是菜单项，另有 Ctrl+Alt+E 热键）。
            WM_LBUTTONDBLCLK => open_settings(),
            _ => {}
        },
        WM_TASK_DONE => {
            // 任务线程回报（lparam 非 0 = 面板当前打开）：只更新缓存，不碰面板窗口。
            if let Ok(mut guard) = TRAY.lock() {
                if let Some(state) = guard.as_mut() {
                    state.panel_open = lparam.0 != 0;
                }
            }
        }
        WM_HOTKEY => request_panel_toggle(),
        WM_TIMER => poll(),
        // 登录注销/关机：尽量移除图标，避免通知区残留幽灵图标。
        WM_ENDSESSION => {
            if wparam.0 != 0 {
                remove_tray_icon(window);
            }
        }
        WM_DESTROY => {
            unsafe {
                let _ = UnregisterHotKey(Some(window), PANEL_HOTKEY_ID);
            }
            uninstall_keyboard_hook();
            PostQuitMessage(0);
        }
        _ => return DefWindowProcW(window, message, wparam, lparam),
    }
    LRESULT(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 菜单序列（T-169）：状态 → 分隔 → 面板 → 设置 → 分隔 → 退出。
    #[test]
    fn 菜单项标识唯一且面板优先于设置() {
        let ids = [MENU_OPEN_PANEL, MENU_OPEN_SETTINGS, MENU_EXIT];
        assert_ne!(ids[0], ids[1]);
        assert_ne!(ids[1], ids[2]);
        assert_ne!(ids[0], ids[2]);
        // 顺带固定菜单顺序：工具箱 → 打开设置 → 退出。
        let max_index = ids
            .iter()
            .enumerate()
            .max_by_key(|(_, id)| **id)
            .map(|(index, _)| index);
        assert_eq!(max_index, Some(2), "退出应为最后一项");
        assert_eq!(
            ids.iter().copied().min(),
            Some(MENU_OPEN_PANEL),
            "工具箱应排在最前"
        );
    }

    /// 菜单命令只在 `TrackPopupMenu` 返回后执行：源码契约——`TrackPopupMenu` 的
    /// 调用必须先于动作分发，且两者之间只做收起菜单。
    #[test]
    fn 菜单动作在模态菜单收起后执行() {
        // 同样只看生产代码：本测试文件自身也调用 `TrackPopupMenu(`。
        let source = include_str!("main.rs");
        let production = source.split("#[cfg(test)]").next().expect("源码可分段");
        let track = production
            .find("TrackPopupMenu(")
            .expect("必须调用 TrackPopupMenu");
        let tail = &production[track..];
        let dispatch = tail
            .find("if picked == MENU_OPEN_PANEL")
            .expect("分发代码存在");
        let between = &tail[..dispatch];
        assert!(
            between.contains("DestroyMenu"),
            "TrackPopupMenu 与分发之间应先收起菜单"
        );
        for action in [
            "request_panel_toggle()",
            "open_settings()",
            "request_exit(window)",
        ] {
            let index = tail.find(action).expect("三个菜单动作都要分发");
            assert!(index > dispatch, "菜单动作 {action} 必须在菜单收起后执行");
        }
    }

    /// 面板创建不经托盘主线程：`PanelWindow::open` 只允许出现在 task 模块里。
    #[test]
    fn 面板创建隔离在任务队列线程() {
        let source = include_str!("main.rs");
        // 只看生产代码：`cfg(test)` 之后是本模块自身，不参与该契约。
        let production = source.split("#[cfg(test)]").next().expect("源码可分段");
        assert!(
            !production.contains("PanelWindow::open"),
            "托盘主线程不得直接创建面板（S-35：模态菜单期间会收不到消息）"
        );
    }
}
