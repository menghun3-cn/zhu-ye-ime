//! 加载版本化实现 DLL 并取到它的类工厂。
//!
//! 壳通过标准 TSF 入口 `DllGetClassObject` 从实现模块取类工厂，因此不需要
//! 任何自定义 ABI——COM 本身就是跨模块稳定 ABI。实现模块由 Windows 加载器
//! 映射，其 `DllMain`/全局初始化照常执行。
//!
//! 模块句柄由 [`LoadedImpl`] 持有；实现对象释放后调用 `FreeLibrary` 卸载该
//! 模块（仅当没有其它引用时才会真正卸载）。

use std::ffi::c_void;
use std::path::Path;

use windows::core::PCWSTR;
use windows::core::{Interface, GUID, PCSTR};
use windows::Win32::Foundation::FreeLibrary;
use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::Com::IClassFactory;
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};

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
pub struct LoadedImpl {
    module: HMODULE,
    /// 用 `Option` 包裹以便 `Drop` 中显式取出并释放（`IClassFactory` 无
    /// `Default` 实现，不能用 `mem::replace` 的占位技巧）。
    factory: Option<IClassFactory>,
}

impl LoadedImpl {
    /// 加载指定路径的实现模块并取其类工厂。
    ///
    /// 失败原因以字符串返回（供日志），不 panic：壳必须能在实现不可用时
    /// 继续用旧实现工作，而不是让宿主崩溃。
    pub fn load(path: &Path) -> Result<Self, String> {
        let wide: Vec<u16> = path
            .as_os_str()
            .to_string_lossy()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let module = unsafe { LoadLibraryW(PCWSTR(wide.as_ptr())) }
            .map_err(|error| format!("加载 {} 失败: {error}", path.display()))?;

        let symbol = {
            let name = b"DllGetClassObject\0";
            unsafe { GetProcAddress(module, PCSTR::from_raw(name.as_ptr())) }
        };
        let Some(symbol) = symbol else {
            let _ = unsafe { FreeLibrary(module) };
            return Err(format!("{} 未导出 DllGetClassObject", path.display()));
        };

        let get_class_object: DllGetClassObject = unsafe { std::mem::transmute(symbol) };
        let iid_class_factory = GUID::from_u128(0x0000_0001_0000_0000_C000_0000_0000_0046);
        let mut factory_ptr: *mut c_void = std::ptr::null_mut();
        let hr =
            unsafe { get_class_object(&CLSID_ZHU_YE_TIP, &iid_class_factory, &mut factory_ptr) };
        if hr.is_err() || factory_ptr.is_null() {
            let _ = unsafe { FreeLibrary(module) };
            return Err(format!(
                "{} 的 DllGetClassObject 失败: {hr:?}",
                path.display()
            ));
        }

        // 安全说明：`factory_ptr` 由 COM 规范保证是有效的 IClassFactory 指针；
        // 接管其引用计数（from_raw 不 AddRef），失败时释放模块。
        let factory = unsafe { IClassFactory::from_raw(factory_ptr) };

        Ok(Self {
            module,
            factory: Some(factory),
        })
    }

    /// 用该实现创建文本服务对象。
    ///
    /// 返回 `ITfTextInputProcessorEx`——宿主只会 `QueryInterface` 这几个接口，
    /// 代理按需再向具体实现取其它接口指针。
    pub fn create_text_service(
        &self,
    ) -> Result<windows::Win32::UI::TextServices::ITfTextInputProcessorEx, String> {
        use windows::Win32::UI::TextServices::ITfTextInputProcessorEx;
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

impl Drop for LoadedImpl {
    fn drop(&mut self) {
        // 先放工厂引用，再放模块引用；顺序反了会让模块在工厂还活着时被卸载。
        self.factory = None;
        let _ = unsafe { FreeLibrary(self.module) };
    }
}
