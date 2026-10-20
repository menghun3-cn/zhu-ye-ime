//! 壳层的端到端集成测试。
//!
//! 覆盖 T-176 的核心能力：
//! 1. 壳能加载版本化实现并返回代理对象；
//! 2. 代理能被 `QueryInterface` 出宿主真正使用的接口；
//! 3. **同一进程内**换版本指针即换实现，无需重启宿主。
//!
//! 第 3 条是本方案的全部价值所在：宿主缓存 COM 对象，只有把替换粒度下沉到
//! COM 对象内部，已打开的宿主才能在获得输入焦点时自动换用新版实现。
//!
//! 「换到了哪个版本」用**模块加载状态**判定：换实现后，旧版本 DLL 的模块句柄
//! 应被卸载（引用计数归零），新版本 DLL 的模块句柄应存在。直接读 Windows
//! 加载器状态，不依赖壳的日志。
//!
//! 测试需要两个真实 DLL，由环境变量提供：
//! - `ZHU_YE_SHELL_DLL`：壳 DLL（`zhu-ye-ime-shell` crate 的产物）
//! - `ZHU_YE_IMPL_DLL`：实现 DLL（`zhu-ye-ime` crate 的产物）
//!
//! 未设置时用例跳过（不硬失败），以便在未构建 DLL 的环境下也能跑测试套件。

use std::path::{Path, PathBuf};

use windows::core::PCWSTR;
use windows::core::{Interface, PCSTR};
use windows::Win32::System::Com::{
    CoInitializeEx, CoUninitialize, IClassFactory, COINIT_APARTMENTTHREADED,
};
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress, LoadLibraryW};
use windows::Win32::UI::TextServices::{
    ITfFnConfigure, ITfFunction, ITfKeyEventSink, ITfTextInputProcessor, ITfTextInputProcessorEx,
};

const CLSID_ZHU_YE_TIP: windows::core::GUID =
    windows::core::GUID::from_u128(0xE54D6682_8650_40E7_A9EE_6FD1137849AE);
const IID_CLASS_FACTORY: windows::core::GUID =
    windows::core::GUID::from_u128(0x0000_0001_0000_0000_C000_0000_0000_0046);

type DllGetClassObject = unsafe extern "system" fn(
    *const windows::core::GUID,
    *const windows::core::GUID,
    *mut *mut std::ffi::c_void,
) -> windows::core::HRESULT;

/// 测试所需 DLL 路径（未设置则跳过用例）。
struct Fixtures {
    shell: PathBuf,
    impl_dll: PathBuf,
}

fn fixtures() -> Option<Fixtures> {
    let shell = std::env::var_os("ZHU_YE_SHELL_DLL")?;
    let impl_dll = std::env::var_os("ZHU_YE_IMPL_DLL")?;
    Some(Fixtures {
        shell: PathBuf::from(shell),
        impl_dll: PathBuf::from(impl_dll),
    })
}

/// 测试用目录。
fn test_root(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("zy-shell-it-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("建测试目录");
    dir
}

/// 搭一个「安装目录」：固定名壳 + `versions\<id>\zhu-ye-impl.dll` + 指针。
///
/// 返回 `(versions 目录, 各版本实现文件路径)`。
fn build_layout(
    root: &Path,
    shell: &Path,
    impl_dll: &Path,
    ids: &[&str],
) -> (PathBuf, Vec<PathBuf>) {
    let versions = root.join("versions");
    let mut impls = Vec::new();
    for id in ids {
        let version_dir = versions.join(id);
        std::fs::create_dir_all(&version_dir).expect("建版本目录");
        let target = version_dir.join("zhu-ye-impl.dll");
        std::fs::copy(impl_dll, &target).expect("复制实现");
        impls.push(target);
    }
    std::fs::copy(shell, root.join("zhu-ye-ime.dll")).expect("复制壳");
    std::fs::write(versions.join("current.txt"), format!("{}\n", ids[0])).expect("写指针");
    (versions, impls)
}

/// 从壳取一个代理对象。
fn create_proxy(shell: &Path) -> Option<ITfTextInputProcessorEx> {
    let wide: Vec<u16> = shell
        .as_os_str()
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let module = unsafe { LoadLibraryW(PCWSTR(wide.as_ptr())) }.ok()?;
    let symbol = {
        let name = b"DllGetClassObject\0";
        unsafe { GetProcAddress(module, PCSTR::from_raw(name.as_ptr())) }
    }?;
    let get: DllGetClassObject = unsafe { std::mem::transmute(symbol) };
    let mut factory_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
    let hr = unsafe { get(&CLSID_ZHU_YE_TIP, &IID_CLASS_FACTORY, &mut factory_ptr) };
    if hr.is_err() || factory_ptr.is_null() {
        return None;
    }
    let factory = unsafe { IClassFactory::from_raw(factory_ptr) };
    unsafe {
        factory.CreateInstance::<Option<&windows::core::IUnknown>, ITfTextInputProcessorEx>(None)
    }
    .ok()
}

/// 该 DLL 当前是否已加载进本进程。
fn is_loaded(path: &Path) -> bool {
    let wide: Vec<u16> = path
        .as_os_str()
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    unsafe { GetModuleHandleW(PCWSTR(wide.as_ptr())).is_ok() }
}

#[test]
fn 壳能加载版本化实现并返回代理() {
    let Some(fixtures) = fixtures() else {
        eprintln!("跳过：需设置 ZHU_YE_SHELL_DLL 与 ZHU_YE_IMPL_DLL");
        return;
    };
    let _ = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }.ok();
    let root = test_root("load");
    let id = "1aaaaaaa";
    let (_versions, impls) = build_layout(&root, &fixtures.shell, &fixtures.impl_dll, &[id]);

    let proxy = create_proxy(&root.join("zhu-ye-ime.dll")).expect("壳应返回代理对象");
    // 代理必须能被 QI 成宿主真正使用的接口。
    assert!(proxy.cast::<ITfTextInputProcessorEx>().is_ok());
    assert!(proxy.cast::<ITfTextInputProcessor>().is_ok());
    assert!(proxy.cast::<ITfKeyEventSink>().is_ok());
    assert!(proxy.cast::<ITfFunction>().is_ok());
    assert!(proxy.cast::<ITfFnConfigure>().is_ok());
    // 版本化实现确实被加载进本进程。
    assert!(is_loaded(&impls[0]), "版本化实现应已加载");

    let _ = std::fs::remove_dir_all(&root);
    unsafe { CoUninitialize() };
}

#[test]
fn 同进程内换版本指针即换实现() {
    let Some(fixtures) = fixtures() else {
        eprintln!("跳过：需设置 ZHU_YE_SHELL_DLL 与 ZHU_YE_IMPL_DLL");
        return;
    };
    let _ = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }.ok();
    let root = test_root("swap");
    let v1 = "1aaaaaaa";
    let v2 = "2bbbbbbb";
    let (versions, impls) = build_layout(&root, &fixtures.shell, &fixtures.impl_dll, &[v1, v2]);

    let proxy = create_proxy(&root.join("zhu-ye-ime.dll")).expect("壳应返回代理对象");
    assert!(is_loaded(&impls[0]), "初始应加载 v1");
    assert!(!is_loaded(&impls[1]), "初始不应加载 v2");

    // 换指针（模拟「升级发布」），然后触发代理的版本检查路径。
    std::fs::write(versions.join("current.txt"), format!("{v2}\n")).expect("改指针");
    // `ActivateEx` 会先做版本检查再转发；这里用空线程管理器，预期转发失败，
    // 但版本切换必须已经发生且【不崩溃】。
    let _ = unsafe {
        proxy.ActivateEx(
            Option::<&windows::Win32::UI::TextServices::ITfThreadMgr>::None,
            0,
            0,
        )
    };

    // ★ 核心断言：新版本已加载、旧版本已卸载 —— 同一进程内完成切换。
    assert!(is_loaded(&impls[1]), "换指针后应加载 v2");
    assert!(!is_loaded(&impls[0]), "换指针后 v1 应已卸载");

    // 换版本后壳仍可用（没有因换实现而损坏）。
    let proxy2 = create_proxy(&root.join("zhu-ye-ime.dll")).expect("换版本后壳仍应可用");
    assert!(proxy2.cast::<ITfTextInputProcessorEx>().is_ok());

    let _ = std::fs::remove_dir_all(&root);
    unsafe { CoUninitialize() };
}

#[test]
fn 指针非法时壳不崩溃且仍返回代理() {
    let Some(fixtures) = fixtures() else {
        eprintln!("跳过：需设置 ZHU_YE_SHELL_DLL 与 ZHU_YE_IMPL_DLL");
        return;
    };
    let _ = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }.ok();
    let root = test_root("badptr");
    let (versions, _impls) =
        build_layout(&root, &fixtures.shell, &fixtures.impl_dll, &["1aaaaaaa"]);
    for bad in ["", "zzzzzzzz", "short", "1aaaaaaa extra"] {
        std::fs::write(versions.join("current.txt"), bad).expect("写脏指针");
        let proxy = create_proxy(&root.join("zhu-ye-ime.dll")).expect("壳应仍能返回代理");
        assert!(proxy.cast::<ITfTextInputProcessorEx>().is_ok());
    }
    let _ = std::fs::remove_dir_all(&root);
    unsafe { CoUninitialize() };
}

#[test]
fn 指向不存在的实现时壳不崩溃() {
    let Some(fixtures) = fixtures() else {
        eprintln!("跳过：需设置 ZHU_YE_SHELL_DLL 与 ZHU_YE_IMPL_DLL");
        return;
    };
    let _ = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }.ok();
    let root = test_root("missingimpl");
    let id = "1aaaaaaa";
    let (versions, _impls) = build_layout(&root, &fixtures.shell, &fixtures.impl_dll, &[id]);
    // 指针指向一个不存在的版本目录。
    std::fs::write(versions.join("current.txt"), "deadbeef\n").expect("改指针");
    let proxy = create_proxy(&root.join("zhu-ye-ime.dll")).expect("壳应仍能返回代理");
    assert!(proxy.cast::<ITfTextInputProcessorEx>().is_ok());
    let _ = std::fs::remove_dir_all(&root);
    unsafe { CoUninitialize() };
}
