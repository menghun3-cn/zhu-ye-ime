//! 设置窗口的 Win32 层：窗口类、消息循环、GDI 双缓冲绘制与命中测试。
//!
//! 绘制与命中测试共用 `layout` 的同一份矩形，避免"看得见的地方点不到"。窗口跟随系统
//! 深浅色与高对比度（D-30），主题变化时重算配色并重绘。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU16, Ordering};

use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, EndPaint, InvalidateRect, UpdateWindow, DT_CENTER, DT_END_ELLIPSIS,
    DT_LEFT, DT_NOPREFIX, DT_RIGHT, DT_SINGLELINE, DT_VCENTER, DT_WORDBREAK, HBRUSH, HDC, HFONT,
    SRCCOPY,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::Dialogs::{
    GetOpenFileNameW, OFN_FILEMUSTEXIST, OFN_HIDEREADONLY, OFN_NOCHANGEDIR, OFN_PATHMUSTEXIST,
    OPENFILENAMEW,
};
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
use zhu_ye_core::{ModeChoice, ThemeChoice};
use zhu_ye_ime::candidate_ui::{UiRect, UiThemeKind, BASE_DPI};

use crate::config;
use crate::gdi::{draw_text, fill, fill_round, BackBuffer, Fonts};
use crate::installed::{self, InstalledPack, PackSource};
use crate::inventory::{self, list_packs, PackInfo};
use crate::layout::{self, Chip, ChipValue, SettingsMetrics};
use crate::model::{ItemControl, OpenTarget, Page, SettingsState};
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
    /// 截图时进入「添加词库」子视图（与 `shot_page` 互斥，优先于条目展开）。
    pub shot_packs: bool,
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
        let mut settings = SettingsState::with_defaults(theme, default_mode);
        if let Some(page) = options.shot_page {
            settings.page = page;
            settings.expanded = options.shot_expanded;
        }
        if options.shot_packs {
            settings.page = Page::Common;
            settings.open_packs();
        }
        let packs = if settings.packs_view {
            list_packs_now(config_path.as_deref())
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
/// 「添加词库」子视图打开时只响应子视图内的命中（返回 / 导入 / 启用开关）。
unsafe fn on_click(hwnd: HWND, state: &mut WindowState, x: i32, y: i32) {
    let client = match client_rect(hwnd) {
        Some(rect) => rect,
        None => return,
    };
    let metrics = SettingsMetrics::new(state.dpi);

    if state.settings.packs_view {
        on_packs_click(hwnd, state, &metrics, client, x, y);
        return;
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
                ItemControl::ThemeChoice | ItemControl::ModeChoice => {}
                ItemControl::OpenPanel(kind) => open_panel(hwnd, state, kind),
                ItemControl::OpenPath(target) => open_target(state, target),
                ItemControl::OpenPacks => {
                    state.packs = list_packs_now(state.config_path.as_deref());
                    state.settings.open_packs();
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
        state.settings.close_packs();
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
/// 配置与日志可能尚未生成（首次改动设置前没有 `config.json`；文件日志仅在验收期启用），
/// 因此先判断存在性再调用系统打开，避免弹出系统的"找不到文件"对话框。
fn open_target(state: &mut WindowState, target: OpenTarget) {
    state.hint = Some(match shell::resolve_target(target) {
        Err(error) => error,
        Ok(path) if path.exists() => match shell::open_path(&path) {
            Ok(()) => format!("已打开 {}", path.display()),
            Err(error) => error,
        },
        Ok(path) => match target {
            OpenTarget::ConfigFile => "配置文件尚未生成：改动任一设置后即会写入".to_owned(),
            OpenTarget::LogDir => "未找到日志目录：文件日志仅在验收期启用".to_owned(),
            OpenTarget::DataDir => format!("目录不存在：{}", path.display()),
        },
    });
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

const fn mode_label(mode: ModeChoice) -> &'static str {
    match mode {
        ModeChoice::Chinese => "中文",
        ModeChoice::English => "英文",
    }
}

/// 二选一控件命中后的分发：主题或默认中英模式（D-32 装配项）。
fn apply_chip(state: &mut WindowState, chip: &Chip) {
    match chip.value {
        ChipValue::Theme(choice) => apply_theme(state, choice),
        ChipValue::Mode(mode) => apply_mode(state, mode),
    }
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

        if state.settings.packs_view {
            draw_packs(hdc, state, theme, &metrics, client);
        } else {
            draw_items(hdc, state, theme, &metrics, client);
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

        for chip in &row.chips {
            let selected = match chip.value {
                ChipValue::Theme(choice) => choice == state.settings.theme,
                ChipValue::Mode(mode) => mode == state.settings.default_mode,
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
