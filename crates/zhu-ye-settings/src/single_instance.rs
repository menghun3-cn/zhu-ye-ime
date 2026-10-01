//! 单实例控制：命名互斥体。
//!
//! 设置窗口可以有多个窗口同时写 `config.json`，互相覆盖对方刚提交的主题（设计文档 S-8
//! 只约定写入方在提交前重读，未约定互斥）。因此第二次唤起只激活已有窗口并退出。

use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE};
use windows::Win32::System::Threading::CreateMutexW;

use crate::wide::to_utf16;

/// 互斥体名；`Local\` 前缀把作用域限定在当前登录会话。
pub const MUTEX_NAME: &str = r"Local\ZhuYeSettingsWindow";

/// 单实例所有权凭据；进程存活期间必须保留，丢弃即释放所有权。
#[derive(Debug)]
pub struct InstanceGuard {
    handle: HANDLE,
}

impl Drop for InstanceGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}

/// 取得单实例的结果。
#[derive(Debug)]
pub enum InstanceState {
    /// 本进程是唯一实例，持有所有权凭据。
    Primary(InstanceGuard),
    /// 已有实例在运行。
    AlreadyRunning,
}

/// 尝试取得单实例所有权。
///
/// # Errors
/// 互斥体创建失败时返回描述；调用方仍可继续启动（单实例保证降级，但窗口可用）。
pub fn acquire() -> Result<InstanceState, String> {
    unsafe {
        let name = to_utf16(MUTEX_NAME);
        let handle = CreateMutexW(None, true, PCWSTR(name.as_ptr()))
            .map_err(|error| format!("创建单实例互斥体失败: {error}"))?;
        // 对象已存在时本次调用拿到的是既有对象的句柄，必须立刻关闭：所有权仍归先到者，
        // 若把该句柄当成自己的凭据，先到者退出后就会误判为"无人持有"。
        if GetLastError() == ERROR_ALREADY_EXISTS {
            let _ = CloseHandle(handle);
            return Ok(InstanceState::AlreadyRunning);
        }
        Ok(InstanceState::Primary(InstanceGuard { handle }))
    }
}
