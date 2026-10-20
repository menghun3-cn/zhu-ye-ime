//! 版本检测与实现切换。
//!
//! 触发时机：每次 `Activate`（获得输入焦点）。此时旧实现已 `Deactivate`、
//! 无编辑会话，替换最安全；且每次聚焦都检查，用户**无需任何操作**即可用上
//! 新版实现。
//!
//! 失败策略：**降级可用优于不可用**。新版本加载失败时保留旧实现继续工作，
//! 只记录错误；指针未变时不做任何额外工作（热路径零开销）。

use std::path::{Path, PathBuf};

use crate::loader::LoadedImpl;
use crate::proxy::{self, Shared};
use crate::version::{impl_path, read_pointer};

/// 本模块内的锚点函数：取其地址经 `GetModuleHandleExW(FROM_ADDRESS)`
/// 反查壳 DLL 所在目录（与 DLL 文件名解耦）。
fn shell_anchor() {}

/// 壳所在的安装目录。
///
/// 壳 DLL 与 `versions\` 同级；由 `std::env::current_exe` 不适用（DLL 不是
/// exe），因此用 `GetModuleHandleW(NULL)` 取宿主进程主模块不可靠。改用
/// `GetModuleHandleExW` 以地址取自身路径——这是取得 DLL 自身所在目录的
/// 标准做法。
fn shell_dir() -> Option<PathBuf> {
    use windows::Win32::Foundation::HMODULE;
    use windows::Win32::System::LibraryLoader::{
        GetModuleHandleExW, GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
        GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
    };

    // 用本函数自身的地址反查所属模块——这是取得「本 DLL 所在目录」的标准
    // 做法（`GetModuleHandleEx(NULL)` 返回的是宿主主模块，不是壳 DLL）。
    let mut handle = HMODULE::default();
    let address = (shell_anchor as fn()) as usize;
    let ok = unsafe {
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            windows::core::PCWSTR(address as *const u16),
            &mut handle,
        )
    };
    if ok.is_err() {
        return None;
    }
    module_path(handle)
}

/// 取模块文件路径。
fn module_path(module: windows::Win32::Foundation::HMODULE) -> Option<PathBuf> {
    use windows::Win32::Foundation::MAX_PATH;
    use windows::Win32::System::LibraryLoader::GetModuleFileNameW;

    let mut buffer = [0u16; MAX_PATH as usize];
    let len = unsafe { GetModuleFileNameW(Some(module), &mut buffer) };
    if len == 0 || len as usize >= buffer.len() {
        return None;
    }
    let path = PathBuf::from(String::from_utf16_lossy(&buffer[..len as usize]));
    path.parent().map(Path::to_path_buf)
}

/// `versions\` 目录。
fn versions_dir() -> Option<PathBuf> {
    shell_dir().map(|dir| dir.join("versions"))
}

/// 若版本指针已变，则加载新实现并换入代理。
///
/// 无指针时不做任何事（保留当前实现）；指针与当前一致时直接返回。
pub(crate) fn refresh_if_outdated(shared: &Shared) {
    let Some(versions) = versions_dir() else {
        return;
    };
    let Some(desired) = read_pointer(&versions) else {
        return;
    };
    if proxy::current_version(shared).as_deref() == Some(desired.as_str()) {
        return;
    }
    if let Err(error) = swap_to(shared, &versions, &desired) {
        // 降级可用：保留旧实现继续工作。
        crate::log_line(&format!("zhu-ye-shell: 切换实现失败，保留旧实现: {error}"));
    }
}

/// 加载指定版本并换入代理槽位。
fn swap_to(shared: &Shared, versions: &Path, version: &str) -> Result<(), String> {
    let path = impl_path(versions, version);
    if !path.is_file() {
        return Err(format!("实现文件不存在: {}", path.display()));
    }
    let module = LoadedImpl::load(&path)?;
    let service = module.create_text_service()?;
    proxy::install(shared, module, service, version.to_string());
    crate::log_line(&format!("zhu-ye-shell: 已换用实现 {version}"));
    Ok(())
}

/// 首次加载实现（`CreateInstance` 时用）。
///
/// 与 [`refresh_if_outdated`] 的区别：这里在**还没有任何实现**时按指针加载；
/// 已有实现时只做版本检查。
pub(crate) fn ensure_loaded(shared: &Shared) {
    if proxy::has_impl(shared) {
        refresh_if_outdated(shared);
        return;
    }
    let Some(versions) = versions_dir() else {
        return;
    };
    let Some(desired) = read_pointer(&versions) else {
        return;
    };
    if let Err(error) = swap_to(shared, &versions, &desired) {
        crate::log_line(&format!("zhu-ye-shell: 首次加载实现失败: {error}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 壳目录可解析() {
        // 本测试运行在测试二进制里，`GetModuleHandleExW(NULL)` 返回的是测试
        // 可执行文件而非壳 DLL——因此只断言「能取到某个模块路径」，不断言具体目录。
        let path = shell_dir();
        assert!(path.is_some(), "应能取到模块所在目录");
    }
}
