//! COM 代理：宿主持有的唯一对象，内部持有真正实现的接口指针。
//!
//! 宿主只加载稳定壳，并在壳里创建本代理；宿主缓存的是代理的 COM 接口指针。
//! 因此**替换代理内部的实现指针**，宿主完全无感——这正是「已打开的宿主在获得
//! 输入焦点时自动换用新版实现」的实现基础。
//!
//! 代理实现的接口与真实实现完全一致（宿主 `QueryInterface` 哪些，我们就有哪些）：
//! - `ITfTextInputProcessorEx`（含 `ITfTextInputProcessor`）
//! - `ITfKeyEventSink`
//! - `ITfCompositionSink`
//! - `ITfFunction` / `ITfFnConfigure`
//!
//! 转发约定：内部实现缺失（尚未加载或加载失败）时返回 `E_NOTIMPL`，不崩溃；
//! 内部实现存在但转发失败时原样返回其错误码。
//!
//! # 共享粒度：模块共享，实现对象**不**共享
//!
//! TSF 的文本服务是**每实例独立状态**的：同一进程里可以有多个文本服务实例
//! （不同线程 / 不同 UI 上下文各一个），每个实例各自持有编辑会话、组合状态，
//! 以及 `AdviseKeyEventSink` 注册用的线程 id。
//!
//! 实现侧把这些状态放在**实现对象自己**里（实测：实现把 `tid` 存进自己的
//! 状态字段，`Deactivate` 时用它 `UnadviseKeyEventSink`）。因此若让进程内所有
//! 代理实例共用**同一个**实现对象，后一个实例的 `Activate` 会覆盖先前实例的
//! `tid`，先前实例的按键事件注册随之失效——TSF 报
//! `EVENT_E_ALL_SUBSCRIBERS_FAILED`（0x80040201）并**回退到别的输入法**
//! （真机实测：展开任务栏隐藏图标后，任务栏输入法指示器从「竹叶输入法」变成
//! 「搜狗拼音输入法」）。
//!
//! 故本模块的分工是：
//! - **进程级共享**：当前实现模块（`Arc<ImplModule>`）与版本 id——模块句柄
//!   是进程级资源，共享它才能让换版本只加载一次；
//! - **每实例私有**：本实例的实现对象（`ITfTextInputProcessorEx`）——由本实例
//!   从共享模块 `CreateInstance` 出来，状态互不干扰。
//!
//! 换版本时，新模块换入共享槽位；各代理实例在自己的 `Activate` /
//! `ActivateEx` 时发现版本变了，就用新模块建自己的新对象并释放自己的旧对象。
//! 旧模块由 `Arc` 引用计数管理：最后一个仍持有旧对象的代理实例换掉之后，
//! 旧模块才被 `FreeLibrary`。

use std::sync::{Arc, Mutex};

use windows::core::{implement, Interface, Ref, BOOL, BSTR, GUID};
use windows::Win32::Foundation::{E_NOTIMPL, LPARAM, WPARAM};
use windows::Win32::UI::TextServices::{
    ITfComposition, ITfCompositionSink, ITfCompositionSink_Impl, ITfContext, ITfFnConfigure,
    ITfFnConfigure_Impl, ITfFunction, ITfFunction_Impl, ITfKeyEventSink, ITfKeyEventSink_Impl,
    ITfTextInputProcessorEx, ITfTextInputProcessorEx_Impl, ITfTextInputProcessor_Impl,
    ITfThreadMgr,
};

use crate::loader::SharedImplModule;

/// 进程级共享的实现槽位。
///
/// 只存**模块**与版本 id，不存实现对象——理由见模块级文档。
pub(crate) struct Inner {
    /// 当前实现模块（`None` = 尚未加载成功）。
    pub(crate) current: Option<SharedImplModule>,
    /// 当前实现对应的版本 id（`versions\current.txt` 的内容）。
    version: Option<String>,
}

// 安全说明：COM 接口指针的引用计数由 COM 运行时保证线程安全（原子增减），
// 且实现对象的所有访问都经由本模块的 `Mutex`。因此跨线程共享是安全的。
unsafe impl Send for Inner {}
unsafe impl Sync for Inner {}

/// 代理共享状态：进程内所有代理对象共用同一个「当前实现模块」。
///
/// 多个宿主窗口可能各建一个代理（COM 每进程一个类工厂，工厂每次
/// `CreateInstance` 出一个新对象）。它们必须**共用同一份实现模块**，否则换
/// 版本时会各加载一份。
pub(crate) type Shared = Arc<Mutex<Inner>>;

/// 单个代理实例私有的实现对象。
struct Mine {
    /// 本实例的实现对象（从 `module` 创建）。
    service: Option<ITfTextInputProcessorEx>,
    /// 创建该对象的模块（持有引用，保证实现代码在对象存活期间不被卸载）。
    module: Option<SharedImplModule>,
    /// 该对象对应的版本 id。
    version: Option<String>,
}

// 安全说明：COM 接口指针的引用计数由 COM 运行时保证线程安全（原子增减），
// 且实现对象的所有访问都经由本模块的 `Mutex`。因此跨线程共享是安全的。
unsafe impl Send for Mine {}
unsafe impl Sync for Mine {}

impl Drop for Mine {
    fn drop(&mut self) {
        // 顺序至关重要：必须**先释放 COM 对象，再释放模块**。
        //
        // 模块被 `FreeLibrary` 卸载后，旧 COM 对象的 `Release` 就会落到已
        // 卸载的代码里（访问冲突 0xC0000005）。故先放 `service`（COM 对象），
        // 再放 `module`（模块引用）。
        self.service = None;
        self.module = None;
    }
}

/// 取得进程级共享实现槽位。
pub(crate) fn shared() -> Shared {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Shared> = OnceLock::new();
    SHARED
        .get_or_init(|| {
            Arc::new(Mutex::new(Inner {
                current: None,
                version: None,
            }))
        })
        .clone()
}

/// 在锁内对本实例的实现对象做一次操作。
///
/// 锁内完成「取指针 + 调用」，避免转发期间本实例被换掉导致的对象失效。
fn with_mine<T>(
    mine: &Mutex<Mine>,
    fallback: impl FnOnce() -> T,
    action: impl FnOnce(&ITfTextInputProcessorEx) -> T,
) -> T {
    let guard = mine.lock().unwrap_or_else(|error| error.into_inner());
    match guard.service.as_ref() {
        Some(service) => action(service),
        None => fallback(),
    }
}

/// 代理对象。
#[implement(
    ITfTextInputProcessorEx,
    ITfKeyEventSink,
    ITfCompositionSink,
    ITfFunction,
    ITfFnConfigure
)]
pub struct Proxy {
    shared: Shared,
    /// 本实例私有的实现对象。
    mine: Arc<Mutex<Mine>>,
}

impl Proxy {
    /// 新建代理（共享进程级实现模块；实现对象由本实例私有）。
    #[must_use]
    pub fn new() -> Self {
        Self {
            shared: shared(),
            mine: Arc::new(Mutex::new(Mine {
                service: None,
                module: None,
                version: None,
            })),
        }
    }

    /// 换版本：若版本指针与本实例当前实现不一致，则用新模块重建本实例的
    /// 实现对象。
    ///
    /// 重建失败（模块缺失、导出不对、`CreateInstance` 失败）时**保留旧对象**
    /// 继续工作——降级可用优于不可用。
    fn refresh(&self) {
        let Some(desired) = crate::swap::desired_version() else {
            return;
        };
        {
            let guard = self.mine.lock().unwrap_or_else(|error| error.into_inner());
            if guard.version.as_deref() == Some(desired.as_str()) {
                return; // 已是目标版本，无需重建。
            }
        }

        // 取（或加载）目标版本的实现模块。多个实例可能同时发现需要换版本，
        // 共享槽位保证只加载一次。
        let module = match crate::swap::module_for(&self.shared, &desired) {
            Ok(module) => module,
            Err(error) => {
                crate::log_line(&format!("zhu-ye-shell: 切换实现失败，保留旧实现: {error}"));
                return;
            }
        };

        // 用新模块建本实例自己的实现对象。
        let service = match module.create_text_service() {
            Ok(service) => service,
            Err(error) => {
                crate::log_line(&format!(
                    "zhu-ye-shell: 创建实现对象失败，保留旧实现: {error}"
                ));
                return;
            }
        };

        // 换入：先放新对象（释放旧的），再放新模块引用。
        let mut guard = self.mine.lock().unwrap_or_else(|error| error.into_inner());
        guard.service = Some(service);
        guard.module = Some(module);
        guard.version = Some(desired);
    }
}

impl Default for Proxy {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------- 文本服务

impl ITfTextInputProcessor_Impl for Proxy_Impl {
    fn Activate(&self, ptim: Ref<'_, ITfThreadMgr>, tid: u32) -> windows::core::Result<()> {
        // ★ 换版本的时机：每次获得输入焦点都检查版本指针。切换只在这里发生，
        // 此时旧实现已处于 `Deactivate` 状态、无编辑会话，替换最安全。
        self.refresh();
        with_mine(
            &self.mine,
            || Err(windows::core::Error::from(E_NOTIMPL)),
            |service| unsafe { service.Activate(ptim.ok()?, tid) },
        )
    }

    fn Deactivate(&self) -> windows::core::Result<()> {
        with_mine(
            &self.mine,
            || Err(windows::core::Error::from(E_NOTIMPL)),
            |service| unsafe { service.Deactivate() },
        )
    }
}

impl ITfTextInputProcessorEx_Impl for Proxy_Impl {
    fn ActivateEx(
        &self,
        ptim: Ref<'_, ITfThreadMgr>,
        tid: u32,
        dwflags: u32,
    ) -> windows::core::Result<()> {
        self.refresh();
        with_mine(
            &self.mine,
            || Err(windows::core::Error::from(E_NOTIMPL)),
            |service| unsafe { service.ActivateEx(ptim.ok()?, tid, dwflags) },
        )
    }
}

// ---------------------------------------------------------------- 按键事件

impl ITfKeyEventSink_Impl for Proxy_Impl {
    fn OnSetFocus(&self, foreground: BOOL) -> windows::core::Result<()> {
        with_mine(
            &self.mine,
            || Err(windows::core::Error::from(E_NOTIMPL)),
            |service| {
                sink_of(service).map_or_else(
                    |_| Err(windows::core::Error::from(E_NOTIMPL)),
                    move |sink| unsafe { sink.OnSetFocus(foreground.as_bool()) },
                )
            },
        )
    }

    fn OnTestKeyDown(
        &self,
        pic: Ref<'_, ITfContext>,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> windows::core::Result<BOOL> {
        with_mine(
            &self.mine,
            || Err(windows::core::Error::from(E_NOTIMPL)),
            |service| {
                sink_of(service).map_or_else(
                    |_| Err(windows::core::Error::from(E_NOTIMPL)),
                    |sink| unsafe { sink.OnTestKeyDown(pic.ok()?, wparam, lparam) },
                )
            },
        )
    }

    fn OnTestKeyUp(
        &self,
        pic: Ref<'_, ITfContext>,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> windows::core::Result<BOOL> {
        with_mine(
            &self.mine,
            || Err(windows::core::Error::from(E_NOTIMPL)),
            |service| {
                sink_of(service).map_or_else(
                    |_| Err(windows::core::Error::from(E_NOTIMPL)),
                    |sink| unsafe { sink.OnTestKeyUp(pic.ok()?, wparam, lparam) },
                )
            },
        )
    }

    fn OnKeyDown(
        &self,
        pic: Ref<'_, ITfContext>,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> windows::core::Result<BOOL> {
        with_mine(
            &self.mine,
            || Err(windows::core::Error::from(E_NOTIMPL)),
            |service| {
                sink_of(service).map_or_else(
                    |_| Err(windows::core::Error::from(E_NOTIMPL)),
                    |sink| unsafe { sink.OnKeyDown(pic.ok()?, wparam, lparam) },
                )
            },
        )
    }

    fn OnKeyUp(
        &self,
        pic: Ref<'_, ITfContext>,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> windows::core::Result<BOOL> {
        with_mine(
            &self.mine,
            || Err(windows::core::Error::from(E_NOTIMPL)),
            |service| {
                sink_of(service).map_or_else(
                    |_| Err(windows::core::Error::from(E_NOTIMPL)),
                    |sink| unsafe { sink.OnKeyUp(pic.ok()?, wparam, lparam) },
                )
            },
        )
    }

    /// TSF 保留键回调（Ctrl+Space 中英切换）。
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn OnPreservedKey(
        &self,
        pic: Ref<'_, ITfContext>,
        rguid: *const GUID,
    ) -> windows::core::Result<BOOL> {
        with_mine(
            &self.mine,
            || Err(windows::core::Error::from(E_NOTIMPL)),
            |service| {
                sink_of(service).map_or_else(
                    |_| Err(windows::core::Error::from(E_NOTIMPL)),
                    |sink| unsafe { sink.OnPreservedKey(pic.ok()?, rguid) },
                )
            },
        )
    }
}

// ---------------------------------------------------------------- 组字事件

impl ITfCompositionSink_Impl for Proxy_Impl {
    fn OnCompositionTerminated(
        &self,
        ecwrite: u32,
        pcomposition: Ref<'_, ITfComposition>,
    ) -> windows::core::Result<()> {
        with_mine(
            &self.mine,
            || Err(windows::core::Error::from(E_NOTIMPL)),
            |service| {
                let sink: ITfCompositionSink = service.cast::<ITfCompositionSink>()?;
                unsafe { sink.OnCompositionTerminated(ecwrite, pcomposition.ok()?) }
            },
        )
    }
}

// ---------------------------------------------------------------- 功能接口

impl ITfFunction_Impl for Proxy_Impl {
    fn GetDisplayName(&self) -> windows::core::Result<BSTR> {
        with_mine(
            &self.mine,
            || Err(windows::core::Error::from(E_NOTIMPL)),
            |service| {
                let function: ITfFunction = service.cast::<ITfFunction>()?;
                unsafe { function.GetDisplayName() }
            },
        )
    }
}

impl ITfFnConfigure_Impl for Proxy_Impl {
    /// 打开设置窗口。
    ///
    /// 签名由 COM 接口固定为安全函数，但参数是裸指针，故需显式解除 lint。
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn Show(
        &self,
        hwndparent: windows::Win32::Foundation::HWND,
        langid: u16,
        pguidprofile: *const GUID,
    ) -> windows::core::Result<()> {
        let _ = (hwndparent, langid, pguidprofile);
        with_mine(
            &self.mine,
            || Err(windows::core::Error::from(E_NOTIMPL)),
            |service| {
                let configure: ITfFnConfigure = service.cast::<ITfFnConfigure>()?;
                unsafe { configure.Show(hwndparent, langid, pguidprofile) }
            },
        )
    }
}

/// 取实现的按键事件接收器接口。
fn sink_of(service: &ITfTextInputProcessorEx) -> windows::core::Result<ITfKeyEventSink> {
    service.cast::<ITfKeyEventSink>()
}

// ---------------------------------------------------------------- 诊断接口

/// 进程级当前实现版本（测试与诊断用）。
#[must_use]
pub(crate) fn current_version(shared: &Shared) -> Option<String> {
    let guard = shared.lock().unwrap_or_else(|error| error.into_inner());
    guard.version.clone()
}

/// 进程级是否已加载实现（测试与诊断用）。
#[must_use]
pub(crate) fn has_impl(shared: &Shared) -> bool {
    let guard = shared.lock().unwrap_or_else(|error| error.into_inner());
    guard.current.is_some()
}

/// 把新模块换入进程级共享槽位。
///
/// 旧模块**不立即卸载**——它可能仍被某些代理实例的实现对象引用着；
/// `Arc` 引用计数保证最后一个引用消失时才 `FreeLibrary`。
pub(crate) fn install_module(shared: &Shared, module: SharedImplModule) {
    let mut guard = shared.lock().unwrap_or_else(|error| error.into_inner());
    guard.version = Some(module.version().to_string());
    guard.current = Some(module);
}
