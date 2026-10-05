//! 语言栏中英模式图标（TSF 语言栏项目）。
//!
//! T-046：任务栏/语言栏显示输入法当前模式——中文模式为"中"、英文模式为"英"，
//! 图标随引擎实际模式切换。Windows 语言栏没有纯注册表方案支持动态图标，
//! 唯一正道是 TSF 语言栏项目（`ITfLangBarItemButton`）：
//!
//! - 文本服务在 `Activate` 时经 `ITfThreadMgr → ITfLangBarItemMgr::AddItem`
//!   注册一个项目按钮，`Deactivate` 时 `RemoveItem` 注销；
//! - 微软文档明确：语言栏管理器会 `QueryInterface` 项目上的 `ITfSource`
//!   并 advise `ITfLangBarItemSink`；项目内部状态变化时调用
//!   `ITfLangBarItemSink::OnUpdate(TF_LBI_ICON)`，语言栏随即重查
//!   `GetIcon`/`GetInfo` 完成刷新；
//! - `windows` crate 0.61 未绑定 `ITfSource`，本模块按
//!   `define_interface!` + 自写 Vtbl 的惯例补上该胶水接口；
//! - 中/英 16×16 图标在运行时用 GDI 绘制（宋体白字 + 品牌蓝/中性灰底），
//!   进程级静态缓存；图标位图与 HICON 同生命周期存活，避免
//!   `CreateIconIndirect` 引用位图被提前释放。
//!
//! 点击语言栏按钮（左键）切换中英模式：`OnClick` 通过构造时注入的
//! `click_handler` 闭包完成。ctfmon 可能从任意 RPC 线程进入 COM 方法，
//! 因此闭包只捕获共享引擎状态的 `Weak` 引用（引擎状态已 Arc 化且
//! `EngineState: Send`，见 tsf.rs T-046 注释），切换在引擎锁内完成、
//! 图标通知在锁外发送，与 `sync_engine` 的 ToggleMode 分支语义一致
//! （组合中的内容在切换时保留）；`upgrade` 失败（文本服务已停用）时
//! 点击安全忽略。

use std::ffi::c_void;
use std::sync::{Mutex, OnceLock};

use windows::core::{implement, ComObject, IUnknown, Interface, Result, GUID, HRESULT, PCWSTR};
use windows::Win32::Foundation::{E_NOINTERFACE, E_POINTER, POINT, RECT};
use windows::Win32::Graphics::Gdi::{
    CreateBitmap, CreateCompatibleDC, CreateDIBSection, CreateFontIndirectW, DeleteDC,
    DeleteObject, DrawTextW, GetDC, GetStockObject, ReleaseDC, SelectObject, SetBkMode,
    SetTextColor, ANTIALIASED_QUALITY, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CLIP_DEFAULT_PRECIS,
    DEFAULT_CHARSET, DEFAULT_GUI_FONT, DEFAULT_PITCH, DIB_RGB_COLORS, DT_CENTER, DT_NOPREFIX,
    DT_SINGLELINE, DT_VCENTER, FF_DONTCARE, FW_BOLD, HBITMAP, HFONT, LOGFONTW, OUT_DEFAULT_PRECIS,
    TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::TextServices::{
    ITfLangBarItem, ITfLangBarItemButton, ITfLangBarItemButton_Impl, ITfLangBarItemMgr,
    ITfLangBarItemSink, ITfLangBarItem_Impl, ITfMenu, ITfThreadMgr, TfLBIClick, TF_LANGBARITEMINFO,
    TF_LBI_DESC_MAXLEN, TF_LBI_ICON, TF_LBI_STYLE_BTN_BUTTON, TF_LBI_STYLE_BTN_MENU,
    TF_LBI_STYLE_SHOWNINTRAY,
};
use windows::Win32::UI::WindowsAndMessaging::{CreateIconIndirect, HICON, ICONINFO, SW_SHOWNORMAL};

use crate::input::InputMode;
use crate::tsf::CLSID_ZHU_YE_TIP;

/// 语言栏项目 GUID（自编固定值，供语言栏区分本项目；不与其他组件冲突）。
pub const LANG_BAR_ITEM_GUID: GUID = GUID::from_u128(0x8C4E3F2A_1D9B_4E57_A6C0_2B9D8E7F1C3A);

/// 语言栏图标物理尺寸（小图标，与 `SM_CXSMICON` 一致，不随 DPI 缩放）。
const ICON_PX: i32 = 16;
/// `TfLBIClick` 左键值（windows 0.61 未导出该公开常量，值来自 windows-sys）。
const TF_LBI_CLK_LEFT: u32 = 1;
/// 中文模式图标底色（品牌蓝 #1E88E5；32bpp 像素 0xAABBGGRR，最高字节 alpha=0xFF 不透明）。
const BG_COLOR_CHINESE: u32 = 0xFFE5_881E;
/// 英文模式图标底色（中性灰 #757575；同上，alpha=0xFF 不透明）。
const BG_COLOR_ENGLISH: u32 = 0xFF75_7575;

/// 语言栏右击菜单中"设置"项的菜单 ID（T-114）。
const SETTINGS_MENU_ID: u32 = 1;

/// 查找设置窗口可执行文件：优先 DLL 同目录，其次上级 bin\ 子目录（发行包布局）。
/// 纯函数便于单测（DLL 路径由调用方注入）。
fn find_settings_exe(dll_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let candidates = [
        dll_dir.join("zhu-ye-settings.exe"),
        dll_dir.join("bin").join("zhu-ye-settings.exe"),
    ];
    candidates.into_iter().find(|p| p.is_file())
}

/// 打开竹叶输入法设置窗口（D-26 已规划的 ITfFnConfigure 拉起逻辑，T-114）。
/// 定位不到设置程序时只记日志，不做 UI 报错（DLL 热路径禁止打扰输入）。
pub(crate) fn launch_settings() {
    let mut buf = [0u16; 1024];
    let len = unsafe { GetModuleFileNameW(None, &mut buf) };
    if len == 0 {
        return;
    }
    let dll_path = std::path::PathBuf::from(String::from_utf16_lossy(&buf[..len as usize]));
    let Some(dll_dir) = dll_path.parent() else {
        return;
    };
    let Some(exe) = find_settings_exe(dll_dir) else {
        crate::tsf::debug_log("zhu-ye: settings exe not found beside dll");
        return;
    };
    let wide = to_utf16_null(&exe.to_string_lossy());
    unsafe {
        ShellExecuteW(None, None, PCWSTR(wide.as_ptr()), None, None, SW_SHOWNORMAL);
    }
}

/// 中文文本转以空字符结尾的 UTF-16（供 PCWSTR 类 API 使用）。
fn to_utf16_null(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

// ---------------------------------------------------------------------------
// ITfSource 胶水接口（windows 0.61 未绑定，按 crate 惯例补定义）
// ---------------------------------------------------------------------------

// `ITfSource`（IID 4EA48A35-60AE-446F-8FD6-E6A8D82459F7）：语言栏管理器
// 通过它向本项目 advise `ITfLangBarItemSink`。仅接受该 sink 接口。
windows_core::imp::define_interface!(
    ITfSource,
    ITfSource_Vtbl,
    0x4EA48A35_60AE_446F_8FD6_E6A8D82459F7
);
windows_core::imp::interface_hierarchy!(ITfSource, windows_core::IUnknown);

#[repr(C)]
#[doc(hidden)]
// COM 方法名/字段沿用接口原义（AdviseSink 等），与 windows 生成代码一致。
#[allow(non_snake_case)]
pub struct ITfSource_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub AdviseSink:
        unsafe extern "system" fn(*mut c_void, *const GUID, *mut c_void, *mut u32) -> HRESULT,
    pub UnadviseSink: unsafe extern "system" fn(*mut c_void, u32) -> HRESULT,
}

/// `ITfSource` 的实现侧契约；`riid` 为非空输入、`punk` 可能为空（COM 规范）。
#[allow(non_camel_case_types, non_snake_case)]
pub trait ITfSource_Impl: windows_core::IUnknownImpl {
    fn AdviseSink(&self, riid: &GUID, punk: Option<&IUnknown>) -> Result<u32>;
    fn UnadviseSink(&self, dwcookie: u32) -> Result<()>;
}

#[allow(non_snake_case)]
impl ITfSource_Vtbl {
    pub const fn new<Identity: ITfSource_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn AdviseSink<Identity: ITfSource_Impl, const OFFSET: isize>(
            this: *mut c_void,
            riid: *const GUID,
            punk: *mut c_void,
            pdwcookie: *mut u32,
        ) -> HRESULT {
            unsafe {
                if riid.is_null() {
                    return E_POINTER;
                }
                if pdwcookie.is_null() {
                    return E_POINTER;
                }
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                let riid: &GUID = &*riid;
                let punk: Option<&IUnknown> = if punk.is_null() {
                    None
                } else {
                    // 关键：`&IUnknown` 是“局部指针变量”的透明视图（windows-core
                    // `Ref::deref` 即 `transmute(&self.0)`），绝不能解引用接口单元
                    // 本身——那里存的是 vtable 指针，不是 this 指针。
                    Some(core::mem::transmute::<&*mut c_void, &IUnknown>(&punk))
                };
                match ITfSource_Impl::AdviseSink(this, riid, punk) {
                    Ok(cookie) => {
                        pdwcookie.write(cookie);
                        HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn UnadviseSink<Identity: ITfSource_Impl, const OFFSET: isize>(
            this: *mut c_void,
            dwcookie: u32,
        ) -> HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                ITfSource_Impl::UnadviseSink(this, dwcookie).into()
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            AdviseSink: AdviseSink::<Identity, OFFSET>,
            UnadviseSink: UnadviseSink::<Identity, OFFSET>,
        }
    }

    pub fn matches(iid: &GUID) -> bool {
        iid == &<ITfSource as Interface>::IID
    }
}

#[allow(non_snake_case)]
impl ITfSource {
    /// 便捷包装：向项目安装/language bar 的 advise sink。
    ///
    /// # Safety
    ///
    /// `riid` 必须指向一个有效的 `GUID`；`pdwcookie` 必须指向可写内存；
    /// `punk` 必须是一个合法的 COM 接口指针（空指针会被拒绝）。
    pub unsafe fn AdviseSink<P0>(
        &self,
        riid: *const GUID,
        punk: P0,
        pdwcookie: *mut u32,
    ) -> Result<()>
    where
        P0: windows_core::Param<IUnknown>,
    {
        unsafe {
            (Interface::vtable(self).AdviseSink)(
                Interface::as_raw(self),
                riid,
                punk.param().abi(),
                pdwcookie,
            )
            .ok()
        }
    }

    /// 便捷包装：撤销语言栏 sink 安装（`dwcookie` 为之前 `AdviseSink` 的返回值）。
    ///
    /// # Safety
    ///
    /// 仅当 `self` 指向真实 COM 对象时调用。
    pub unsafe fn UnadviseSink(&self, dwcookie: u32) -> Result<()> {
        unsafe { (Interface::vtable(self).UnadviseSink)(Interface::as_raw(self), dwcookie).ok() }
    }
}

// ---------------------------------------------------------------------------
// 中/英图标（GDI 运行时绘制，进程级静态缓存）
// ---------------------------------------------------------------------------

/// 一枚语言栏图标及其引用位图（与 HICON 同生命周期，防止提前释放）。
struct ModeIcon {
    icon: HICON,
    /// 32bpp 颜色位图（`CreateIconIndirect` 引用，进程退出时一并回收）。
    _color: HBITMAP,
    /// 单色掩码位图（同上）。
    _mask: HBITMAP,
}

// GDI 句柄在进程内就是地址值（UINT_PTR），可安全跨线程传递/共享：
// 图标只创建一次、永不销毁，也没有线程相关的所有权语义。
unsafe impl Send for ModeIcon {}
unsafe impl Sync for ModeIcon {}

struct ModeIcons {
    chinese: ModeIcon,
    english: ModeIcon,
}

static MODE_ICONS: OnceLock<core::result::Result<ModeIcons, HRESULT>> = OnceLock::new();

/// 取进程级"中/英"图标对；首次调用时用 GDI 绘制（失败则返回错误，
/// 语言栏按 `GetIcon` 失败处理即不显示图标，不影响输入功能）。
fn mode_icons() -> Result<&'static ModeIcons> {
    let stored = MODE_ICONS.get_or_init(|| match render_both_icons() {
        Ok(icons) => Ok(icons),
        Err(err) => Err(err.code()),
    });
    match stored {
        Ok(icons) => Ok(icons),
        Err(hr) => Err(windows::core::Error::from(*hr)),
    }
}

/// 绘制"中/英"两枚图标（仅首次调用，失败整体重试由调用方承担）。
fn render_both_icons() -> Result<ModeIcons> {
    let chinese = render_mode_icon(BG_COLOR_CHINESE, "中")?;
    let english = render_mode_icon(BG_COLOR_ENGLISH, "英")?;
    Ok(ModeIcons { chinese, english })
}

/// 绘制一枚 16×16 图标：底色整块填充 + 白色宋体单位置中。
///
/// 位图与掩码返回后交给调用方持有（与 HICON 同生命周期）；临时 DC 用完即删。
fn render_mode_icon(bg_bgr: u32, glyph: &str) -> Result<ModeIcon> {
    // GDI 调用均为 unsafe FFI；整套绘制放进单个 unsafe 块（与候选窗一致）。
    unsafe {
        let screen = GetDC(None);
        if screen.is_invalid() {
            return Err(windows::core::Error::from_win32());
        }
        let result = (|| -> Result<ModeIcon> {
            let mem = CreateCompatibleDC(Some(screen));
            if mem.is_invalid() {
                return Err(windows::core::Error::from_win32());
            }
            // 32bpp 自顶向下 DIB，直接拿到像素缓冲用于底色填充。
            let bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: ICON_PX,
                    biHeight: -ICON_PX,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                bmiColors: [Default::default()],
            };
            let mut bits: *mut c_void = std::ptr::null_mut();
            let color = CreateDIBSection(Some(mem), &bmi, DIB_RGB_COLORS, &mut bits, None, 0)?;
            if bits.is_null() {
                return Err(windows::core::Error::from_win32());
            }
            let old_bmp = SelectObject(mem, color.into());
            let pixels: &mut [u32] =
                std::slice::from_raw_parts_mut(bits.cast::<u32>(), (ICON_PX * ICON_PX) as usize);
            pixels.fill(bg_bgr);

            // 白色单位置中。直接用 LOGFONTW（与候选窗一致的宋体路径）。
            let mut face = [0u16; 32];
            for (slot, unit) in face.iter_mut().zip("SimSun".encode_utf16().chain(Some(0))) {
                *slot = unit;
            }
            let metrics = LOGFONTW {
                lfHeight: -12,
                lfWeight: FW_BOLD.0 as i32,
                lfCharSet: DEFAULT_CHARSET,
                lfOutPrecision: OUT_DEFAULT_PRECIS,
                lfClipPrecision: CLIP_DEFAULT_PRECIS,
                // 灰度抗锯齿：在 32bpp DIB 上可靠写入 alpha（CLEARTYPE 的
                // 次像素渲染在透明底上可能留下 alpha=0 的不可见图元）。
                lfQuality: ANTIALIASED_QUALITY,
                lfPitchAndFamily: FF_DONTCARE.0 | DEFAULT_PITCH.0,
                lfFaceName: face,
                ..Default::default()
            };
            let font = CreateFontIndirectW(&metrics);
            let font_is_stock = font.is_invalid();
            let font: HFONT = if font_is_stock {
                HFONT(GetStockObject(DEFAULT_GUI_FONT).0)
            } else {
                font
            };
            let old_font = SelectObject(mem, font.into());

            let mut wide: Vec<u16> = glyph.encode_utf16().collect();
            let mut rect = RECT {
                left: 0,
                top: 0,
                right: ICON_PX,
                bottom: ICON_PX,
            };
            SetBkMode(mem, TRANSPARENT);
            SetTextColor(mem, windows::Win32::Foundation::COLORREF(0x00FF_FFFF));
            DrawTextW(
                mem,
                &mut wide,
                &mut rect,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
            );
            // GDI 文本在 32bpp DIB 上只写 RGB 不写 alpha（命中字形区域的像素
            // alpha 落到 0x00）；图标的透明由 alpha 决定，因此把这些"区别于
            // 底色"的像素显式置为不透明，白字才可见。
            for p in pixels.iter_mut() {
                if (*p >> 24) & 0xFF == 0 && *p & 0x00FF_FFFF != bg_bgr {
                    *p |= 0xFF00_0000;
                }
            }
            SelectObject(mem, old_font);
            if !font_is_stock {
                let _ = DeleteObject(font.into());
            }

            // 全 0 单色掩码：尺寸必须与颜色位图一致（同为 16×16），
            // 32bpp 图标的透明由颜色位图的 alpha 通道决定。
            // 1bpp 行字节 = ((16+15)/16)*2 = 2，16 行共 32 字节。
            let mask_bits = [0u8; 32];
            let mask = CreateBitmap(ICON_PX, ICON_PX, 1, 1, Some(mask_bits.as_ptr().cast()));
            if mask.is_invalid() {
                SelectObject(mem, old_bmp);
                let _ = DeleteObject(color.into());
                let _ = DeleteDC(mem);
                return Err(windows::core::Error::from_win32());
            }

            let info = ICONINFO {
                fIcon: true.into(),
                xHotspot: 0,
                yHotspot: 0,
                hbmMask: mask,
                hbmColor: color,
            };
            let icon = CreateIconIndirect(&info)?;

            SelectObject(mem, old_bmp);
            let _ = DeleteDC(mem);
            Ok(ModeIcon {
                icon,
                _color: color,
                _mask: mask,
            })
        })();
        let _ = ReleaseDC(None, screen);
        result
    }
}

// ---------------------------------------------------------------------------
// 语言栏模式按钮项目
// ---------------------------------------------------------------------------

/// 语言栏中英模式按钮：图标与文本随引擎模式切换。
///
/// 线程模型：ctfmon 可能经 RPC 线程调用本对象的 COM 方法，内部状态全部
/// 用 `Mutex` 保护；`set_mode` 通知 sink 时先释放自锁，避免与语言栏的
/// 回查（`OnUpdate → GetIcon/GetInfo`）在单线程上成环。
#[implement(ITfLangBarItem, ITfLangBarItemButton, ITfSource)]
pub struct LangBarModeButton {
    mode: Mutex<InputMode>,
    sink: Mutex<Option<ITfLangBarItemSink>>,
    /// 左键点击处理：返回切换后的模式；`None` 表示忽略（引擎不可达等）。
    /// 只捕获共享状态的 `Weak` 引用，保证 `Send + Sync`（CTF 可能经
    /// 任意 RPC 线程调用 `OnClick`）。
    click_handler: Box<dyn Fn() -> Option<InputMode> + Send + Sync>,
}

impl LangBarModeButton {
    fn new(
        mode: InputMode,
        click_handler: Box<dyn Fn() -> Option<InputMode> + Send + Sync>,
    ) -> Self {
        Self {
            mode: Mutex::new(mode),
            sink: Mutex::new(None),
            click_handler,
        }
    }

    fn current_mode(&self) -> InputMode {
        *self.mode.lock().unwrap()
    }

    /// 切换项目模式并通知语言栏刷新图标；sink 调用在自锁释放后进行。
    pub fn set_mode(&self, mode: InputMode) {
        let sink = {
            let mut guard = self.mode.lock().unwrap();
            *guard = mode;
            self.sink.lock().unwrap().clone()
        };
        let Some(sink) = sink else {
            return;
        };
        // TF_LBI_ICON：语言栏收到后重查 GetIcon/GetInfo 完成图标切换。
        let _ = unsafe { sink.OnUpdate(TF_LBI_ICON) };
    }
}

impl ITfLangBarItem_Impl for LangBarModeButton_Impl {
    // 语言栏保证 pinfo 非空可写；写入下沉到 unsafe 助手执行。
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn GetInfo(
        &self,
        pinfo: *mut windows::Win32::UI::TextServices::TF_LANGBARITEMINFO,
    ) -> Result<()> {
        if pinfo.is_null() {
            return Err(E_POINTER.into());
        }
        // szDescription 是定长数组：逐元素拷贝 + 空终止，不做定长转换。
        // 单字"竹"作为显示文字：Win11 任务栏输入法指示器/系统托盘以本字段
        // 为品牌简称（微信输入法"微"同款机制，2026-10-06 用户点名：指示器
        // 不再显示语言层面回退的"简体"）；完整模式说明在 GetTooltipString。
        let mut desc = [0u16; TF_LBI_DESC_MAXLEN as usize];
        for (slot, unit) in desc.iter_mut().zip("竹".encode_utf16().chain(Some(0))) {
            *slot = unit;
        }
        let info = TF_LANGBARITEMINFO {
            clsidService: CLSID_ZHU_YE_TIP,
            guidItem: LANG_BAR_ITEM_GUID,
            // 普通按钮 + 允许在系统托盘/任务栏指示器显示（Win10；Win11 忽略）
            // + 支持右击菜单（T-114 设置入口）。
            dwStyle: TF_LBI_STYLE_BTN_BUTTON | TF_LBI_STYLE_SHOWNINTRAY | TF_LBI_STYLE_BTN_MENU,
            ulSort: 0,
            szDescription: desc,
        };
        // 写入动作下沉到 unsafe 助手（pinfo 非空且由语言栏保证可写）。
        unsafe { write_lang_bar_info(pinfo, info) };
        Ok(())
    }

    fn GetStatus(&self) -> Result<u32> {
        // 图标本身已表达模式，无需 pressed/隐藏等状态位。
        Ok(0)
    }

    fn Show(&self, _fshow: windows::core::BOOL) -> Result<()> {
        Ok(())
    }

    fn GetTooltipString(&self) -> Result<windows::core::BSTR> {
        let tip = match self.current_mode() {
            InputMode::Chinese => "竹叶输入法：中文模式（按 Shift 切换）",
            InputMode::English => "竹叶输入法：英文模式（按 Shift 切换）",
        };
        Ok(windows::core::BSTR::from(tip))
    }
}

/// 把语言栏项目信息写入调用方缓冲区。
///
/// # Safety
///
/// `pinfo` 必须非空且指向至少 `size_of::<TF_LANGBARITEMINFO>` 的可写内存。
unsafe fn write_lang_bar_info(pinfo: *mut TF_LANGBARITEMINFO, info: TF_LANGBARITEMINFO) {
    unsafe { pinfo.write(info) };
}

impl ITfLangBarItemButton_Impl for LangBarModeButton_Impl {
    fn OnClick(&self, click: TfLBIClick, _pt: &POINT, _prcarea: *const RECT) -> Result<()> {
        // TF_LBI_CLK_LEFT（1）：切换中英模式并刷新图标（T-046）。
        // 右击由语言栏经 InitMenu/OnMenuSelect 走菜单（T-114 设置项）。
        if click == TfLBIClick(TF_LBI_CLK_LEFT as i32) {
            if let Some(mode) = (self.click_handler)() {
                self.set_mode(mode);
            }
        }
        Ok(())
    }

    fn InitMenu(&self, pmenu: windows::core::Ref<'_, ITfMenu>) -> Result<()> {
        // T-114：语言栏右击菜单提供"设置"入口（D-26）。
        let Some(menu) = pmenu.cloned() else {
            return Err(E_POINTER.into());
        };
        let label = to_utf16_null("设置(&S)…");
        unsafe {
            menu.AddMenuItem(
                SETTINGS_MENU_ID,
                0,
                HBITMAP::default(),
                HBITMAP::default(),
                &label[..label.len() - 1],
                std::ptr::null_mut(),
            )?;
        }
        Ok(())
    }

    fn OnMenuSelect(&self, wid: u32) -> Result<()> {
        if wid == SETTINGS_MENU_ID {
            launch_settings();
        }
        Ok(())
    }

    fn GetIcon(&self) -> Result<HICON> {
        Ok(match self.current_mode() {
            InputMode::Chinese => &mode_icons()?.chinese,
            InputMode::English => &mode_icons()?.english,
        }
        .icon)
    }

    fn GetText(&self) -> Result<windows::core::BSTR> {
        let label = match self.current_mode() {
            InputMode::Chinese => "中",
            InputMode::English => "英",
        };
        Ok(windows::core::BSTR::from(label))
    }
}

impl ITfSource_Impl for LangBarModeButton_Impl {
    fn AdviseSink(&self, riid: &GUID, punk: Option<&IUnknown>) -> Result<u32> {
        let Some(punk) = punk else {
            return Err(E_POINTER.into());
        };
        // 语言栏只会安装 ITfLangBarItemSink；其他接口直接拒绝。
        if *riid != <ITfLangBarItemSink as Interface>::IID {
            return Err(E_NOINTERFACE.into());
        }
        let sink: ITfLangBarItemSink = punk.cast()?;
        // 重复 advise 视为替换（语言栏正常只在加载时安装一次）。
        *self.sink.lock().unwrap() = Some(sink);
        Ok(1)
    }

    fn UnadviseSink(&self, _dwcookie: u32) -> Result<()> {
        *self.sink.lock().unwrap() = None;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 文本服务侧绑定：注册/更新/注销
// ---------------------------------------------------------------------------

/// 文本服务实例注册到语言栏的句柄：管理器中已注册的项目 + 项目对象引用。
#[derive(Clone)]
pub struct LangBarHandle {
    mgr: ITfLangBarItemMgr,
    item: ComObject<LangBarModeButton>,
}

impl LangBarHandle {
    /// 在线程管理器上注册中英模式按钮；失败返回错误（调用方按非致命处理）。
    ///
    /// `click_handler` 供左键点击切换模式：只应捕获共享引擎状态的 `Weak`
    /// 引用（`Send + Sync`），返回切换后的模式；`None` 表示忽略本次点击。
    pub fn register(
        thread_mgr: &ITfThreadMgr,
        mode: InputMode,
        click_handler: Box<dyn Fn() -> Option<InputMode> + Send + Sync>,
    ) -> Result<Self> {
        // ITfLangBarItemMgr 不是 ITfThreadMgr 的父子接口，需要显式 QI。
        let mgr: ITfLangBarItemMgr = thread_mgr.cast()?;
        let item: ComObject<LangBarModeButton> =
            ComObject::new(LangBarModeButton::new(mode, click_handler));
        let item_lang: ITfLangBarItem = item.to_interface();
        unsafe { mgr.AddItem(&item_lang) }?;
        Ok(Self { mgr, item })
    }

    /// 引擎模式变化后通知语言栏刷新图标。
    pub fn set_mode(&self, mode: InputMode) {
        self.item.set_mode(mode);
    }

    /// 注销项目（消耗句柄；同时释放管理器对项目的引用）。
    pub fn unregister(self) {
        let item_lang: ITfLangBarItem = self.item.to_interface();
        let _ = unsafe { self.mgr.RemoveItem(&item_lang) };
    }
}

// ---------------------------------------------------------------------------
// 单元测试：不依赖 ctfmon，直接驱动项目对象验证状态机与通知
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::Graphics::Gdi::GetDIBits;
    use windows::Win32::UI::TextServices::ITfLangBarItemSink_Impl;
    use windows::Win32::UI::WindowsAndMessaging::GetIconInfo;

    /// T-114：设置可执行文件定位——同目录优先、发行包 bin\ 备选、缺失返回 None。
    #[test]
    fn find设置可执行文件路径() {
        let base = std::env::temp_dir().join("zhu-ye-langbar-test");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("tsf")).unwrap();
        std::fs::create_dir_all(base.join("bin")).unwrap();
        std::fs::write(base.join("tsf").join("zhu-ye-settings.exe"), b"x").unwrap();
        std::fs::write(base.join("bin").join("zhu-ye-settings.exe"), b"x").unwrap();
        // 同目录命中优先于 bin\。
        let found = find_settings_exe(&base.join("tsf")).unwrap();
        assert_eq!(found, base.join("tsf").join("zhu-ye-settings.exe"));
        // 同目录无、bin\ 有 → 取 bin\。
        let found = find_settings_exe(&base.join("bin")).unwrap();
        assert_eq!(found, base.join("bin").join("zhu-ye-settings.exe"));
        // 均缺失 → None。
        assert!(find_settings_exe(&base.join("missing")).is_none());
        let _ = std::fs::remove_dir_all(&base);
    }
    /// 桩 sink：记录收到的 OnUpdate 标志位。
    #[implement(ITfLangBarItemSink)]
    struct FakeSink {
        updates: Mutex<Vec<u32>>,
    }

    impl ITfLangBarItemSink_Impl for FakeSink_Impl {
        fn OnUpdate(&self, dwflags: u32) -> Result<()> {
            self.updates.lock().unwrap().push(dwflags);
            Ok(())
        }
    }

    /// 默认无操作点击处理器（多数测试只关注图标/通知行为）。
    fn noop_handler() -> Box<dyn Fn() -> Option<InputMode> + Send + Sync> {
        Box::new(|| None)
    }

    fn new_button(mode: InputMode) -> ComObject<LangBarModeButton> {
        ComObject::new(LangBarModeButton::new(mode, noop_handler()))
    }

    /// 经 GetDIBits 把图标颜色位图按 32bpp 自顶向下回读为像素（0xAABBGGRR）。
    fn read_bitmap_pixels(hbm: HBITMAP) -> Vec<u32> {
        let hdc = unsafe { GetDC(None) };
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: ICON_PX,
                biHeight: -ICON_PX,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            bmiColors: [Default::default()],
        };
        let mut buf = vec![0u32; (ICON_PX * ICON_PX) as usize];
        unsafe {
            GetDIBits(
                hdc,
                hbm,
                0,
                ICON_PX as u32,
                Some(buf.as_mut_ptr().cast()),
                &mut bmi,
                DIB_RGB_COLORS,
            );
            let _ = ReleaseDC(None, hdc);
        }
        buf
    }

    fn new_fake_sink() -> (ComObject<FakeSink>, ITfLangBarItemSink) {
        let com = ComObject::new(FakeSink {
            updates: Mutex::new(Vec::new()),
        });
        let iface = com.to_interface::<ITfLangBarItemSink>();
        (com, iface)
    }

    /// 在按钮上 advise 一个桩 sink，返回其 ComObject（保证通知期间存活）。
    fn advised_sink(button: &ComObject<LangBarModeButton>) -> ComObject<FakeSink> {
        let (com, sink_iface) = new_fake_sink();
        let src: ITfSource = button.to_interface();
        let mut cookie = 0u32;
        assert!(
            unsafe {
                src.AdviseSink(
                    &<ITfLangBarItemSink as Interface>::IID,
                    &sink_iface,
                    &mut cookie,
                )
            }
            .is_ok(),
            "AdviseSink 应成功"
        );
        assert_eq!(cookie, 1);
        com
    }

    /// 图标随模式切换，且"中/英"为两枚不同的图标。
    #[test]
    fn 图标与文本随模式切换() {
        let button = new_button(InputMode::Chinese);
        let btn: ITfLangBarItemButton = button.to_interface();
        let icon_cn = unsafe { btn.GetIcon() }.unwrap();
        let text_cn = unsafe { btn.GetText() }.unwrap().to_string();
        assert_eq!(text_cn, "中");
        assert!(!icon_cn.is_invalid());

        button.set_mode(InputMode::English);
        let icon_en = unsafe { btn.GetIcon() }.unwrap();
        let text_en = unsafe { btn.GetText() }.unwrap().to_string();
        assert_eq!(text_en, "英");
        assert!(!icon_en.is_invalid());
        assert_ne!(icon_cn, icon_en, "中/英应使用不同图标句柄");
    }

    /// 模式变化通知已注册的语言栏 sink（TF_LBI_ICON）。
    #[test]
    fn 模式切换通知sink刷新图标() {
        let button = new_button(InputMode::Chinese);
        let fake = advised_sink(&button);

        button.set_mode(InputMode::English);
        button.set_mode(InputMode::Chinese);
        let updates = fake.updates.lock().unwrap().clone();
        assert_eq!(updates.len(), 2);
        assert!(updates.iter().all(|&f| f & TF_LBI_ICON != 0));
    }

    /// 非 ITfLangBarItemSink 的 riid 一律拒绝。
    #[test]
    fn 拒绝非语言栏sink的riid() {
        let button = new_button(InputMode::Chinese);
        let src: ITfSource = button.to_interface();
        let (_com, sink_iface) = new_fake_sink();
        let mut cookie = 0u32;
        let err =
            unsafe { src.AdviseSink(&<IUnknown as Interface>::IID, &sink_iface, &mut cookie) }
                .unwrap_err();
        assert_eq!(err.code().0, E_NOINTERFACE.0);
    }

    /// UnadviseSink 后停止通知。
    #[test]
    fn unadvise后不再通知() {
        let button = new_button(InputMode::Chinese);
        let fake = advised_sink(&button);
        let src: ITfSource = button.to_interface();
        assert!(unsafe { src.UnadviseSink(1) }.is_ok());

        button.set_mode(InputMode::English);
        assert!(fake.updates.lock().unwrap().is_empty());
    }

    /// 重复 AdviseSink 替换旧 sink：只有最后一个收到通知。
    #[test]
    fn 重复advise替换旧sink() {
        let button = new_button(InputMode::Chinese);
        let (first, first_iface) = new_fake_sink();
        let src: ITfSource = button.to_interface();
        let mut cookie = 0u32;
        let _ = unsafe {
            src.AdviseSink(
                &<ITfLangBarItemSink as Interface>::IID,
                &first_iface,
                &mut cookie,
            )
        };

        let second = advised_sink(&button);
        button.set_mode(InputMode::English);
        assert!(first.updates.lock().unwrap().is_empty());
        assert_eq!(second.updates.lock().unwrap().len(), 1);
    }

    /// GetInfo 提供正确的身份信息（CLSID/项目 GUID/按钮样式/描述）。
    #[test]
    fn getinfo身份与描述正确() {
        let button = new_button(InputMode::Chinese);
        let item: ITfLangBarItem = button.to_interface();
        let mut info = TF_LANGBARITEMINFO::default();
        unsafe { item.GetInfo(&mut info) }.unwrap();
        assert_eq!(info.clsidService, CLSID_ZHU_YE_TIP);
        assert_eq!(info.guidItem, LANG_BAR_ITEM_GUID);
        assert_ne!(info.dwStyle & TF_LBI_STYLE_BTN_BUTTON, 0);
        assert_ne!(info.dwStyle & TF_LBI_STYLE_SHOWNINTRAY, 0);
        assert_eq!(info.szDescription[0], '竹' as u16);
        // 单字品牌简称：第二元素即空终止（2026-10-06 起指示器显示"竹"）。
        assert_eq!(info.szDescription[1], 0);
    }

    /// 图标可渲染：中/英两枚图标创建成功，且像素级校验——底色
    /// alpha=0xFF（托盘上不隐身）、存在白色字形像素（"中"/"英"可见）。
    #[test]
    fn 中英图标可渲染() {
        let icons = &mode_icons().expect("图标应能在桌面会话中创建");
        let expect_bg = [
            BG_COLOR_CHINESE & 0x00FF_FFFF,
            BG_COLOR_ENGLISH & 0x00FF_FFFF,
        ];
        for (set, bg_rgb) in [&icons.chinese, &icons.english].into_iter().zip(expect_bg) {
            let mut info = ICONINFO::default();
            assert!(
                unsafe { GetIconInfo(set.icon, &mut info) }.is_ok(),
                "GetIconInfo 应成功"
            );
            let pixels = read_bitmap_pixels(info.hbmColor);
            assert_eq!(pixels.len(), 256, "应为 16×16 像素回读");
            // 背景色（取四角）：alpha 必须为 0xFF（不透明），RGB 必须命中底色。
            for idx in [0usize, 15, 240, 255] {
                let p = pixels[idx];
                assert_eq!((p >> 24) & 0xFF, 0xFF, "四角像素 alpha 必须不透明");
                assert_eq!(p & 0x00FF_FFFF, bg_rgb, "四角像素底色应精确匹配");
            }
            // 字形：白（RGB 全 ≥ 180）且 alpha ≥ 200 的像素应成片存在。
            let white = pixels
                .iter()
                .filter(|&&p| {
                    (p >> 24) & 0xFF >= 200
                        && p & 0xFF >= 180
                        && (p >> 8) & 0xFF >= 180
                        && (p >> 16) & 0xFF >= 180
                })
                .count();
            assert!(white >= 5, "应存在白色字形像素，实际 {white} 个");
            // GetIconInfo 返回的位图是新拷贝，用完必须删除以免句柄泄漏。
            let _ = unsafe { DeleteObject(info.hbmColor.into()) };
            let _ = unsafe { DeleteObject(info.hbmMask.into()) };
        }
    }

    /// 进程级图标缓存只初始化一次（两次取用同一句柄）。
    #[test]
    fn 图标缓存单例() {
        let a = mode_icons().unwrap();
        let b = mode_icons().unwrap();
        assert_eq!(a.chinese.icon, b.chinese.icon);
        assert_eq!(a.english.icon, b.english.icon);
    }

    /// GetStatus 恒为 0（无额外状态位）。
    #[test]
    fn 状态无扩展位() {
        let button = new_button(InputMode::Chinese);
        let item: ITfLangBarItem = button.to_interface();
        assert_eq!(unsafe { item.GetStatus() }.unwrap(), 0);
    }

    /// 左键点击触发注入的处理器并刷新图标；返回的模式写入按钮状态。
    #[test]
    fn 左键点击切换模式并通知() {
        let button = ComObject::new(LangBarModeButton::new(
            InputMode::Chinese,
            Box::new(|| Some(InputMode::English)),
        ));
        let sink = advised_sink(&button);
        let btn: ITfLangBarItemButton = button.to_interface();
        let pt = POINT::default();
        let rect = RECT::default();
        assert!(unsafe { btn.OnClick(TfLBIClick(TF_LBI_CLK_LEFT as i32), pt, &rect) }.is_ok());
        assert_eq!(button.current_mode(), InputMode::English);
        // 左键点击必须触发 OnUpdate(TF_LBI_ICON) 通知语言栏刷新。
        let updates = sink.updates.lock().unwrap();
        assert!(
            updates.contains(&TF_LBI_ICON),
            "expected TF_LBI_ICON update, got {updates:?}"
        );
    }

    /// 处理器返回 None（引擎不可达）时点击被安全忽略，模式不变。
    #[test]
    fn 左键点击处理器忽略则不切换() {
        let button = new_button(InputMode::Chinese);
        let btn: ITfLangBarItemButton = button.to_interface();
        let pt = POINT::default();
        let rect = RECT::default();
        assert!(unsafe { btn.OnClick(TfLBIClick(TF_LBI_CLK_LEFT as i32), pt, &rect) }.is_ok());
        assert_eq!(button.current_mode(), InputMode::Chinese);
    }

    /// 非左键点击（右击等）不触发切换，保证菜单类事件不被吞掉。
    #[test]
    fn 非左键点击不切换() {
        let button = ComObject::new(LangBarModeButton::new(
            InputMode::Chinese,
            Box::new(|| panic!("handler must not run for non-left clicks")),
        ));
        let btn: ITfLangBarItemButton = button.to_interface();
        let pt = POINT::default();
        let rect = RECT::default();
        assert!(unsafe { btn.OnClick(TfLBIClick(2), pt, &rect) }.is_ok());
        assert_eq!(button.current_mode(), InputMode::Chinese);
    }
}
