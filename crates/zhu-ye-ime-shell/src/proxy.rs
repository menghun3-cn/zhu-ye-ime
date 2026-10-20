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

use std::sync::{Arc, Mutex};

use windows::core::{implement, Interface, Ref, BOOL, BSTR, GUID};
use windows::Win32::Foundation::{E_NOTIMPL, LPARAM, WPARAM};
use windows::Win32::UI::TextServices::{
    ITfComposition, ITfCompositionSink, ITfCompositionSink_Impl, ITfContext, ITfFnConfigure,
    ITfFnConfigure_Impl, ITfFunction, ITfFunction_Impl, ITfKeyEventSink, ITfKeyEventSink_Impl,
    ITfTextInputProcessorEx, ITfTextInputProcessorEx_Impl, ITfTextInputProcessor_Impl,
    ITfThreadMgr,
};

use crate::loader::LoadedImpl;

/// 代理内部持有的实现状态。
///
/// `impl_module` 与 `text_service` 一起被替换：换版本时先建好新实现，再整体
/// 换入，旧实现随旧模块一起释放。`Mutex` 保证切换与转发互斥——转发时短暂
/// 持锁（转发本身是同步 COM 调用，锁只覆盖取指针与调用，不引入长持锁）。
pub(crate) struct Inner {
    /// 当前实现模块（持有模块句柄与类工厂）。
    impl_module: Option<LoadedImpl>,
    /// 当前实现创建出的文本服务对象（`ITfTextInputProcessorEx`）。
    text_service: Option<ITfTextInputProcessorEx>,
    /// 当前实现对应的版本 id（`versions\current.txt` 的内容）。
    version: Option<String>,
}

/// 代理共享状态：进程内所有代理对象共用同一个实现槽位。
///
/// 多个宿主窗口可能各建一个代理（COM 每进程一个类工厂，工厂每次
/// `CreateInstance` 出一个新对象）。它们必须**共用同一份实现**，否则换版本时
/// 只会换掉其中一个代理的实现。因此实现槽位是进程级全局。
pub(crate) type Shared = Arc<Mutex<Inner>>;

// 安全说明：COM 接口指针的引用计数由 COM 运行时保证线程安全（原子增减），
// 且实现对象的所有访问都经由本模块的 `Mutex`。因此跨线程共享是安全的。
unsafe impl Send for Inner {}
unsafe impl Sync for Inner {}

/// 取得进程级共享实现槽位。
pub(crate) fn shared() -> Shared {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Shared> = OnceLock::new();
    SHARED
        .get_or_init(|| {
            Arc::new(Mutex::new(Inner {
                impl_module: None,
                text_service: None,
                version: None,
            }))
        })
        .clone()
}

/// 在锁内对当前实现做一次操作。
///
/// 锁内完成「取指针 + 调用」，避免转发期间实现被换掉导致的对象失效。
fn with_impl<T>(
    shared: &Shared,
    fallback: impl FnOnce() -> T,
    action: impl FnOnce(&ITfTextInputProcessorEx) -> T,
) -> T {
    let guard = shared.lock().unwrap_or_else(|error| error.into_inner());
    match guard.text_service.as_ref() {
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
}

impl Proxy {
    /// 新建代理（共享进程级实现槽位）。
    #[must_use]
    pub fn new() -> Self {
        Self { shared: shared() }
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
        crate::swap::refresh_if_outdated(&self.shared);
        with_impl(
            &self.shared,
            || Err(windows::core::Error::from(E_NOTIMPL)),
            |service| unsafe { service.Activate(ptim.ok()?, tid) },
        )
    }

    fn Deactivate(&self) -> windows::core::Result<()> {
        with_impl(
            &self.shared,
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
        crate::swap::refresh_if_outdated(&self.shared);
        with_impl(
            &self.shared,
            || Err(windows::core::Error::from(E_NOTIMPL)),
            |service| unsafe { service.ActivateEx(ptim.ok()?, tid, dwflags) },
        )
    }
}

// ---------------------------------------------------------------- 按键事件

impl ITfKeyEventSink_Impl for Proxy_Impl {
    fn OnSetFocus(&self, foreground: BOOL) -> windows::core::Result<()> {
        with_impl(
            &self.shared,
            || Err(windows::core::Error::from(E_NOTIMPL)),
            move |service| {
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
        with_impl(
            &self.shared,
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
        with_impl(
            &self.shared,
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
        with_impl(
            &self.shared,
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
        with_impl(
            &self.shared,
            || Err(windows::core::Error::from(E_NOTIMPL)),
            |service| {
                sink_of(service).map_or_else(
                    |_| Err(windows::core::Error::from(E_NOTIMPL)),
                    |sink| unsafe { sink.OnKeyUp(pic.ok()?, wparam, lparam) },
                )
            },
        )
    }

    // `rguid` 是裸指针（COM 规范签名），本方法必须按 trait 要求保持 safe fn；
    // 实际解引用发生在 `ITfKeyEventSink::OnPreservedKey` 内部，其指针由宿主
    // 按 COM 规范传入。
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn OnPreservedKey(
        &self,
        pic: Ref<'_, ITfContext>,
        rguid: *const GUID,
    ) -> windows::core::Result<BOOL> {
        with_impl(
            &self.shared,
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

/// 取实现的按键事件接收器接口。
fn sink_of(service: &ITfTextInputProcessorEx) -> windows::core::Result<ITfKeyEventSink> {
    service.cast::<ITfKeyEventSink>()
}

// ---------------------------------------------------------------- 组字事件

impl ITfCompositionSink_Impl for Proxy_Impl {
    fn OnCompositionTerminated(
        &self,
        ecwrite: u32,
        pcomposition: Ref<'_, ITfComposition>,
    ) -> windows::core::Result<()> {
        with_impl(
            &self.shared,
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
        with_impl(
            &self.shared,
            || Err(windows::core::Error::from(E_NOTIMPL)),
            |service| {
                let function: ITfFunction = service.cast::<ITfFunction>()?;
                unsafe { function.GetDisplayName() }
            },
        )
    }
}

impl ITfFnConfigure_Impl for Proxy_Impl {
    // `rguidprofile` 是裸指针（COM 规范签名），本方法必须按 trait 要求保持
    // safe fn；实际解引用发生在 `ITfFnConfigure::Show` 内部。
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn Show(
        &self,
        hwndparent: windows::Win32::Foundation::HWND,
        langid: u16,
        rguidprofile: *const GUID,
    ) -> windows::core::Result<()> {
        with_impl(
            &self.shared,
            || Err(windows::core::Error::from(E_NOTIMPL)),
            |service| {
                let configure: ITfFnConfigure = service.cast::<ITfFnConfigure>()?;
                unsafe { configure.Show(hwndparent, langid, rguidprofile) }
            },
        )
    }
}

/// 供其它模块检查代理是否已有可用实现（测试与诊断用）。
#[must_use]
pub(crate) fn has_impl(shared: &Shared) -> bool {
    let guard = shared.lock().unwrap_or_else(|error| error.into_inner());
    guard.text_service.is_some()
}

/// 当前实现对应的版本 id（测试与诊断用）。
#[must_use]
pub(crate) fn current_version(shared: &Shared) -> Option<String> {
    let guard = shared.lock().unwrap_or_else(|error| error.into_inner());
    guard.version.clone()
}

/// 替换进程级实现槽位（供 `swap` 模块调用）。
///
/// 供 `swap` 在加载新版本成功后原子换入；旧实现随旧模块一起释放。
pub(crate) fn install(
    shared: &Shared,
    module: LoadedImpl,
    service: ITfTextInputProcessorEx,
    version: String,
) {
    let mut guard = shared.lock().unwrap_or_else(|error| error.into_inner());
    // ★ 顺序至关重要：必须先释放旧的 COM 对象，再释放旧模块。
    //
    // `LoadedImpl::drop` 会 `FreeLibrary` 旧实现模块——一旦模块被卸载，旧 COM
    // 对象的 `Release` 就会落到已卸载的代码里（访问冲突 0xC0000005）。
    // 因此赋值顺序必须是：先 `text_service`（放 COM 对象），再 `impl_module`
    // （放模块句柄）。
    //
    // 同理，新对象是**先构造好再换入**（见 `swap::swap_to`），所以换入时不会有
    // 「新实现还没加载好、旧实现已被卸载」的窗口。
    guard.text_service = Some(service);
    guard.impl_module = Some(module);
    guard.version = Some(version);
}
