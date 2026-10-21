//! 加载版本化实现 DLL 并取到它的类工厂。
//!
//! 壳通过标准 TSF 入口 `DllGetClassObject` 从实现模块取类工厂，因此不需要
//! 任何自定义 ABI——COM 本身就是跨模块稳定 ABI。实现模块由 Windows 加载器
//! 映射，其 `DllMain`/全局初始化照常执行。
//!
//! # 共享粒度：模块共享，实现对象不共享
//!
//! TSF 的文本服务是**每实例独立状态**的：同一进程里可以有多个文本服务实例
//! （不同线程 / 不同 UI 上下文各一个），每个实例各自持有编辑会话、组合状态、
//! 以及 `AdviseKeyEventSink` 注册用的线程 id。
//!
//! 因此本模块提供 [`ImplModule`]（模块 + 类工厂 + 版本 id），它可以被多个
//! 代理实例**共享**（模块句柄是进程级资源）；但每个代理实例必须**自己**
//! `CreateInstance` 出一个实现对象。若让所有实例共用同一个实现对象，实现侧
//! 的单值状态（实测：实现把 `AdviseKeyEventSink` 的 `tid` 存进自己的状态
//! 字段）会被后一个实例覆盖，导致先前实例的按键事件注册失效——TSF 随之
//! 报 `EVENT_E_ALL_SUBSCRIBERS_FAILED`（0x80040201）并回退到别的输入法。

use std::ffi::c_void;
use std::path::Path;
use std::sync::Arc;

use windows::core::PCWSTR;
use windows::core::{Interface, GUID, PCSTR};
use windows::Win32::Foundation::FreeLibrary;
use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::Com::IClassFactory;
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
use windows::Win32::UI::TextServices::ITfTextInputProcessorEx;

/// 竹叶输入法 TIP 的 CLSID（与实现侧一致）。
pub const CLSID_ZHU_YE_TIP: GUID = GUID::from_u128(0xE54D6682_8650_40E7_A9EE_6FD1137849AE);

/// `DllGetClassObject` 的函数签名（COM 规范）。
type DllGetClassObject = unsafe extern "system" fn(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> windows::core::HRESULT;

/// 已加载的实现模块及其类工厂。
///
/// 持有 `HMODULE` 与 `IClassFactory`：模块句柄保证实现代码在工厂存活期间
/// 不会被卸载；COM 引用计数保证工厂存活期间实现模块不被释放。
///
/// 多个代理实例通过 [`Arc`] 共享同一个 `ImplModule`——这样换版本时新模块
/// 只需加载一次，而旧模块会在**最后一个**引用它的代理实例换掉自己的实现
/// 对象之后才被 `FreeLibrary`（`Arc` 的 `Drop` 自动完成）。
pub struct ImplModule {
    module: HMODULE,
    /// 用 `Option` 包裹以便 `Drop` 中显式取出并释放（`IClassFactory` 无
    /// `Default` 实现，不能用 `mem::replace` 的占位技巧）。
    factory: Option<IClassFactory>,
    /// 本模块对应的版本 id（`versions\current.txt` 的内容）。
    version: String,
}

impl ImplModule {
    /// 加载指定路径的实现模块并取其类工厂。
    ///
    /// 失败原因以字符串返回（供日志），不 panic：壳必须能在实现不可用时
    /// 继续用旧实现工作，而不是让宿主崩溃。
    pub fn load(path: &Path, version: &str) -> Result<Self, String> {
        let wide: Vec<u16> = path
            .as_os_str()
            .to_string_lossy()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let module = unsafe { LoadLibraryW(PCWSTR(wide.as_ptr())) }
            .map_err(|error| format!("LoadLibraryW 失败: {error:?}"))?;

        let symbol = {
            let name = b"DllGetClassObject\0";
            unsafe { GetProcAddress(module, PCSTR::from_raw(name.as_ptr())) }
        };
        let Some(symbol) = symbol else {
            let _ = unsafe { FreeLibrary(module) };
            return Err("实现模块未导出 DllGetClassObject".to_string());
        };

        let get_class_object: DllGetClassObject = unsafe { std::mem::transmute(symbol) };
        let mut factory_ptr: *mut c_void = std::ptr::null_mut();
        let hr =
            unsafe { get_class_object(&CLSID_ZHU_YE_TIP, &IClassFactory::IID, &mut factory_ptr) };
        if hr.is_err() || factory_ptr.is_null() {
            let _ = unsafe { FreeLibrary(module) };
            return Err(format!("实现的 DllGetClassObject 失败: {hr:?}"));
        }

        // 安全说明：`factory_ptr` 由 COM 规范保证是有效的 IClassFactory 指针；
        // 接管其引用计数（from_raw 不 AddRef），失败时释放模块。
        let factory = unsafe { IClassFactory::from_raw(factory_ptr) };

        Ok(Self {
            module,
            factory: Some(factory),
            version: version.to_string(),
        })
    }

    /// 本模块的版本 id。
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// 用该模块创建一个**新的**文本服务对象。
    ///
    /// 每个文本服务实例都必须有自己的实现对象（见模块级文档说明的状态
    /// 隔离要求）。
    pub fn create_text_service(&self) -> Result<ITfTextInputProcessorEx, String> {
        let factory = self
            .factory
            .as_ref()
            .ok_or_else(|| "类工厂已释放".to_string())?;
        unsafe {
            factory
                .CreateInstance::<Option<&windows::core::IUnknown>, ITfTextInputProcessorEx>(None)
        }
        .map_err(|error| format!("CreateInstance 失败: {error:?}"))
    }
}

// 安全说明：模块句柄与 COM 类工厂指针的跨线程使用由 COM 运行时保证原子性，
// 且本类型的所有访问都经由调用方的 `Mutex` 保护。
unsafe impl Send for ImplModule {}
unsafe impl Sync for ImplModule {}

impl Drop for ImplModule {
    fn drop(&mut self) {
        // 先放工厂引用，再放模块引用；顺序反了会让模块在工厂还活着时被卸载。
        self.factory = None;
        let _ = unsafe { FreeLibrary(self.module) };
    }
}

/// 共享的已加载实现模块。
pub type SharedImplModule = Arc<ImplModule>;
