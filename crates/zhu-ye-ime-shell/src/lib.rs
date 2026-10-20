//! 竹叶输入法 TSF **稳定壳**。
//!
//! 壳 DLL 是宿主的唯一加载目标（固定文件名 `<InstallDir>\zhu-ye-ime.dll`），
//! 本身不含输入法逻辑。它做两件事：
//!
//! 1. **加载版本化实现**：读 `versions\current.txt` 得到版本 id，加载
//!    `versions\<id>\zhu-ye-impl.dll`，通过标准 TSF 入口 `DllGetClassObject`
//!    取其类工厂。
//! 2. **COM 代理转发**：宿主创建的是壳的代理对象；代理把宿主对
//!    `ITfTextInputProcessorEx` / `ITfKeyEventSink` / `ITfCompositionSink` /
//!    `ITfFunction` / `ITfFnConfigure` 的每次调用转发给当前实现。
//!
//! 这样做的原因（实测）：宿主在失焦再聚焦时**复用已缓存的 COM 对象**，不会
//! 重新调用 `DllGetClassObject`。因此只有把替换粒度下沉到 **COM 对象内部**，
//! 已打开的宿主才能在获得输入焦点时自动换用新版实现。详见
//! `docs/方案设计.md` §17。
//!
//! 升级流程见 `scripts/ime-identity.ps1`：写 `versions\current.txt` 即发布。

#![allow(clippy::missing_safety_doc)]

pub mod loader;
pub mod proxy;
pub mod swap;
pub mod version;

use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};

use windows::core::{implement, Interface, HRESULT};
use windows::Win32::Foundation::{
    CLASS_E_CLASSNOTAVAILABLE, CLASS_E_NOAGGREGATION, E_POINTER, S_FALSE, S_OK,
};
use windows::Win32::System::Com::{IClassFactory, IClassFactory_Impl};

use proxy::Proxy;

/// 竹叶输入法 TIP 的 CLSID（与实现侧一致）。
pub const CLSID_ZHU_YE_TIP: windows::core::GUID =
    windows::core::GUID::from_u128(0xE54D6682_8650_40E7_A9EE_6FD1137849AE);

/// 活动对象计数（`DllCanUnloadNow` 用）。
static ACTIVE_OBJECTS: AtomicBool = AtomicBool::new(false);

/// 日志文件路径：与实现侧共用同一份，便于统一排查。
fn log_path() -> std::path::PathBuf {
    std::env::temp_dir()
        .join("zhu-ye-test")
        .join("shell-debug.log")
}

/// 写一行调试日志（失败静默：日志不可用不应影响输入法）。
pub(crate) fn log_line(message: &str) {
    use std::io::Write;
    let path = log_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = writeln!(file, "{}", message);
    }
}

/// 壳的类工厂：按 CLSID 返回代理对象。
#[implement(IClassFactory)]
struct ShellClassFactory;

impl IClassFactory_Impl for ShellClassFactory_Impl {
    fn CreateInstance(
        &self,
        punkouter: windows::core::Ref<'_, windows::core::IUnknown>,
        riid: *const windows::core::GUID,
        ppvobject: *mut *mut c_void,
    ) -> windows::core::Result<()> {
        if riid.is_null() || ppvobject.is_null() {
            return Err(E_POINTER.into());
        }
        if !punkouter.is_null() {
            return Err(CLASS_E_NOAGGREGATION.into());
        }
        // 首次创建对象时按指针加载实现（之后每次 Activate 再检查版本）。
        swap::ensure_loaded(&proxy::shared());
        let object: Proxy = Proxy::new();
        let unknown: windows::core::IUnknown = object.into();
        let hr = unsafe { unknown.query(riid, ppvobject) };
        hr.ok()
    }

    fn LockServer(&self, _flock: windows::core::BOOL) -> windows::core::Result<()> {
        Ok(())
    }
}

/// DLL 标准导出：按 CLSID 返回类工厂对象。
///
/// # Safety
/// 参数由 TSF 宿主按 COM 规范传入。
#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: *const windows::core::GUID,
    riid: *const windows::core::GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if rclsid.is_null() || riid.is_null() || ppv.is_null() {
        return E_POINTER;
    }
    unsafe {
        *ppv = std::ptr::null_mut();
    }
    if unsafe { *rclsid } != CLSID_ZHU_YE_TIP {
        return CLASS_E_CLASSNOTAVAILABLE;
    }
    let factory: windows::core::IUnknown = ShellClassFactory.into();
    unsafe { factory.query(riid, ppv) }
}

/// DLL 标准导出：无活动对象且未被 LockServer 时允许卸载。
#[no_mangle]
pub extern "system" fn DllCanUnloadNow() -> HRESULT {
    if !ACTIVE_OBJECTS.load(Ordering::Relaxed) {
        S_OK
    } else {
        S_FALSE
    }
}

/// DLL 标准导出：实例计数（宿主可能调用）。
#[no_mangle]
pub extern "system" fn DllGetInstanceCount() -> HRESULT {
    S_OK
}

/// 探针导出：供部署校验脚本确认壳已就位。
#[no_mangle]
pub extern "system" fn dll_probe() -> u32 {
    0x5A48_4559
}

/// 供测试与诊断：当前实现版本 id。
#[must_use]
pub fn current_impl_version() -> Option<String> {
    proxy::current_version(&proxy::shared())
}
