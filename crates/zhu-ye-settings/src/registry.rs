//! TSF 注册表读取与重注册（Win32 薄层；T-076 / FR-043）。
//!
//! 标识常量与 `scripts/ime-identity.ps1` 双份存在，由 T-079 门禁
//! `scripts/verify-tsf-identity.ps1` 守护逐字一致；本模块的注册路径与写入值对齐
//! `New-TsfRegistration`（"先删后建、幂等"），避免两侧的行为漂移。
//!
//! 本模块还提供二级修复的独立子命令入口 `run_repair_default`（由主进程经
//! `ShellExecuteExW` 的 `runas` 动态拉起，只做注册表重注册并返回退出码）与
//! `restart_ctfmon`（D-43：重注册后重启输入法进程使新注册生效，重启前由
//! 父窗口弹确认）。

use std::path::PathBuf;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS, HWND};
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteTreeW, RegGetValueW, RegOpenKeyExW, RegSetValueExW,
    HKEY, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_64KEY, KEY_WRITE, REG_CREATE_KEY_DISPOSITION,
    REG_DWORD, REG_OPTION_NON_VOLATILE, REG_SZ, REG_VALUE_TYPE, RRF_RT_REG_DWORD, RRF_RT_REG_SZ,
};
use windows::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, MB_ICONERROR, MB_OK, MESSAGEBOX_RESULT, MESSAGEBOX_STYLE,
};

use zhu_ye_core::identity::{
    CLSID_ZHU_YE_TIP, PROFILE_GUID_ZHU_YE, TFCAT_ZHU_YE_KEYBOARD, TSF_LANGUAGE_ID_HEX,
};

use crate::repair::RegistryProbe;
use crate::shell;
use crate::wide::to_utf16;

/// 注册表显示名（与 `ime-identity.ps1` 的 `DisplayName` 一致；不出现在 T-079 比对项中，
/// 但重注册必须与安装脚本写入的字串一致，否则系统设置里的名称会漂移）。
const DISPLAY_NAME: &str = "竹叶输入法";

/// GUID 的标准大括号文本（与 `ime-identity.ps1` 常量格式一致，全大写）。
/// 身份常量的主源在 `zhu_ye_core::identity`（u128 字面量，T-081），此处按
/// Win32 层需要的 `windows::core::GUID` 形式转换。
fn guid_text(value: u128) -> String {
    let guid = windows::core::GUID::from_u128(value);
    format!(
        "{{{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}}}",
        guid.data1,
        guid.data2,
        guid.data3,
        guid.data4[0],
        guid.data4[1],
        guid.data4[2],
        guid.data4[3],
        guid.data4[4],
        guid.data4[5],
        guid.data4[6],
        guid.data4[7],
    )
}

/// TSF TIP 树：`SOFTWARE\Microsoft\CTF\TIP\{CLSID}`。
#[must_use]
pub fn tip_key_path() -> String {
    format!(
        "SOFTWARE\\Microsoft\\CTF\\TIP\\{}",
        guid_text(CLSID_ZHU_YE_TIP)
    )
}

/// CLSID 树：`SOFTWARE\Classes\CLSID\{CLSID}`。
#[must_use]
pub fn clsid_key_path() -> String {
    format!("SOFTWARE\\Classes\\CLSID\\{}", guid_text(CLSID_ZHU_YE_TIP))
}

/// 语言配置文件键：`TIP\{CLSID}\LanguageProfile\{语言}\{ProfileGuid}`。
#[must_use]
pub fn profile_key_path() -> String {
    format!(
        "{}\\LanguageProfile\\{}\\{}",
        tip_key_path(),
        TSF_LANGUAGE_ID_HEX,
        guid_text(PROFILE_GUID_ZHU_YE)
    )
}

/// 类别注册键：`TIP\{CLSID}\Category\Category\{TFCAT}\{CLSID}`。
#[must_use]
pub fn category_key_path() -> String {
    format!(
        "{}\\Category\\Category\\{}\\{}",
        tip_key_path(),
        guid_text(TFCAT_ZHU_YE_KEYBOARD),
        guid_text(CLSID_ZHU_YE_TIP)
    )
}

/// 类别条目键：`TIP\{CLSID}\Category\Item\{CLSID}\{TFCAT}`。
#[must_use]
pub fn item_key_path() -> String {
    format!(
        "{}\\Category\\Item\\{}\\{}",
        tip_key_path(),
        guid_text(CLSID_ZHU_YE_TIP),
        guid_text(TFCAT_ZHU_YE_KEYBOARD)
    )
}

/// `InProcServer32` 键：`SOFTWARE\Classes\CLSID\{CLSID}\InProcServer32`。
#[must_use]
pub fn inproc_key_path() -> String {
    format!("{}\\InProcServer32", clsid_key_path())
}

/// 以 64 位视图打开 HKLM 键（只读）。
fn open_key_read(path: &str) -> Option<HKEY> {
    let wide = to_utf16(path);
    let mut key = HKEY::default();
    let status = unsafe {
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            PCWSTR(wide.as_ptr()),
            None,
            KEY_READ | KEY_WOW64_64KEY,
            &mut key,
        )
    };
    if status == ERROR_SUCCESS {
        Some(key)
    } else {
        None
    }
}

/// 读一个 REG_SZ 值；`None` = 键/值不存在或类型不符。
fn query_string(key: HKEY, name: Option<&str>) -> Option<String> {
    let name_wide = name.map(to_utf16);
    let name_ptr = match &name_wide {
        Some(wide) => PCWSTR(wide.as_ptr()),
        None => PCWSTR::null(),
    };
    let mut buffer = [0u16; 2048];
    let mut size = (buffer.len() * 2) as u32;
    let mut kind = REG_VALUE_TYPE(0);
    let status = unsafe {
        RegGetValueW(
            key,
            PCWSTR::null(),
            name_ptr,
            RRF_RT_REG_SZ,
            Some(&mut kind),
            Some(buffer.as_mut_ptr().cast()),
            Some(&mut size),
        )
    };
    if status != ERROR_SUCCESS || kind != REG_SZ {
        return None;
    }
    let length = size as usize / 2;
    if length == 0 {
        return None;
    }
    // RegGetValueW 的 REG_SZ 带结尾 NUL，去掉。
    let mut text = String::from_utf16_lossy(&buffer[..length]);
    while text.ends_with('\0') {
        text.pop();
    }
    Some(text)
}

/// 读一个 REG_DWORD 值；`None` = 键/值不存在或类型不符。
fn query_dword(key: HKEY, name: &str) -> Option<u32> {
    let name_wide = to_utf16(name);
    let mut value = 0u32;
    let mut size = 4u32;
    let mut kind = REG_VALUE_TYPE(0);
    let status = unsafe {
        RegGetValueW(
            key,
            PCWSTR::null(),
            PCWSTR(name_wide.as_ptr()),
            RRF_RT_REG_DWORD,
            Some(&mut kind),
            Some((&mut value as *mut u32).cast()),
            Some(&mut size),
        )
    };
    if status == ERROR_SUCCESS && kind == REG_DWORD {
        Some(value)
    } else {
        None
    }
}

/// 探测当前注册状态（供"管理输入法"/"修复输入法"展示；只读，不写盘）。
#[must_use]
pub fn probe_registration() -> RegistryProbe {
    let profile = open_key_read(&profile_key_path());
    let inproc = open_key_read(&inproc_key_path());
    RegistryProbe {
        language_profile_exists: profile.is_some(),
        enable: profile.and_then(|key| query_dword(key, "Enable")),
        inproc_server: inproc.and_then(|key| query_string(key, None)),
        threading_model: inproc.and_then(|key| query_string(key, Some("ThreadingModel"))),
    }
}

/// 读取当前 `InProcServer32` 默认值指向的 DLL（二级修复的第一来源）。
#[must_use]
pub fn current_inproc_dll() -> Option<PathBuf> {
    let key = open_key_read(&inproc_key_path())?;
    let dll = query_string(key, None)?;
    let path = PathBuf::from(dll);
    path.is_file().then_some(path)
}

/// 回退来源：安装目录（`exe 所在目录\tsf`）下最新的 `zhu-ye-ime*.dll`。
///
/// 安装使用版本化文件名（`zhu-ye-ime-<hash8>.dll`），不能假定固定名 `zhu-ye-ime.dll`。
#[must_use]
pub fn find_bundled_dll() -> Option<PathBuf> {
    let dir = shell::exe_dir().join("tsf");
    let entries = std::fs::read_dir(&dir).ok()?;
    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension().and_then(|ext| ext.to_str()) == Some("dll")
                && path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .is_some_and(|stem| stem == "zhu-ye-ime" || stem.starts_with("zhu-ye-ime-"))
        })
        .max_by_key(|path| path.metadata().and_then(|meta| meta.modified()).ok())
}

/// 以 KEY_WOW64_64KEY 写权限创建注册键并返回句柄。
fn create_key_write(path: &str) -> Result<HKEY, String> {
    let wide = to_utf16(path);
    let mut key = HKEY::default();
    let mut disposition = REG_CREATE_KEY_DISPOSITION(0);
    let status = unsafe {
        RegCreateKeyExW(
            HKEY_LOCAL_MACHINE,
            PCWSTR(wide.as_ptr()),
            None,
            None,
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE | KEY_READ | KEY_WOW64_64KEY,
            None,
            &mut key,
            Some(&mut disposition),
        )
    };
    if status == ERROR_SUCCESS {
        Ok(key)
    } else {
        Err(format!(
            "创建注册键 HKLM\\{path} 失败（Win32 {}）",
            status.0
        ))
    }
}

fn set_string(key: HKEY, name: Option<&str>, value: &str) -> Result<(), String> {
    let name_wide = name.map(to_utf16);
    let name_ptr = match &name_wide {
        Some(wide) => PCWSTR(wide.as_ptr()),
        None => PCWSTR::null(),
    };
    let value_wide = to_utf16(value);
    let bytes: &[u8] =
        unsafe { std::slice::from_raw_parts(value_wide.as_ptr().cast(), value_wide.len() * 2) };
    let status = unsafe { RegSetValueExW(key, name_ptr, None, REG_SZ, Some(bytes)) };
    if status == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(format!("写入注册表字符串失败（Win32 {}）", status.0))
    }
}

fn set_dword(key: HKEY, name: &str, value: u32) -> Result<(), String> {
    let name_wide = to_utf16(name);
    let bytes = value.to_le_bytes();
    let status = unsafe {
        RegSetValueExW(
            key,
            PCWSTR(name_wide.as_ptr()),
            None,
            REG_DWORD,
            Some(&bytes),
        )
    };
    if status == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(format!("写入注册表 DWORD 失败（Win32 {}）", status.0))
    }
}

/// 删除两棵 HKLM 树后再重建（与安装脚本"先清理旧注册再重建"的幂等语义一致）。
pub fn re_register(dll_path: &std::path::Path) -> Result<(), String> {
    let dll_text = dll_path.to_string_lossy().into_owned();
    let roots = [tip_key_path(), clsid_key_path()];
    for root in &roots {
        let wide = to_utf16(root);
        let status = unsafe { RegDeleteTreeW(HKEY_LOCAL_MACHINE, PCWSTR(wide.as_ptr())) };
        if status != ERROR_SUCCESS && status != ERROR_FILE_NOT_FOUND {
            return Err(format!(
                "删除注册树 HKLM\\{root} 失败（Win32 {}）",
                status.0
            ));
        }
    }

    // 先建所有键；任一失败立即返回（幂等：下次重跑会先删后建）。
    let tip = create_key_write(&tip_key_path())?;
    let category = create_key_write(&category_key_path())?;
    let item = create_key_write(&item_key_path())?;
    let profile = create_key_write(&profile_key_path())?;
    let clsid = create_key_write(&clsid_key_path())?;
    let inproc = create_key_write(&inproc_key_path())?;

    let keys = [tip, category, item, profile, clsid, inproc];
    let result = (|| {
        set_string(item, Some("Description"), DISPLAY_NAME)?;
        set_string(profile, Some("Description"), DISPLAY_NAME)?;
        set_string(profile, Some("Display Description"), DISPLAY_NAME)?;
        set_dword(profile, "Enable", 1)?;
        set_string(profile, Some("IconFile"), &dll_text)?;
        set_dword(profile, "IconIndex", 0)?;
        set_string(clsid, None, DISPLAY_NAME)?;
        set_string(inproc, None, &dll_text)?;
        set_string(inproc, Some("ThreadingModel"), "Apartment")
    })();
    for key in &keys {
        let _ = unsafe { RegCloseKey(*key) };
    }
    result.map_err(|error| format!("重建 TSF 注册失败（{error}；请重试）"))
}

/// 二级修复独立子命令入口：`zhu-ye-settings --repair-registry`。
///
/// 由主进程经 `ShellExecuteExW` 的 `runas` 动词在提权环境拉起；只做两棵 HKLM 树的重注册
/// （不接受任何路径参数），成功静默退出码 0，失败弹窗说明并退出码 1。安全边界见
/// 设计文档 §6.3 与风险表"提权子命令攻击面"。
pub fn run_repair_default() -> std::process::ExitCode {
    let dll = current_inproc_dll().or_else(find_bundled_dll);
    let Some(dll) = dll else {
        let _ = message_box(
            None,
            "输入法文件缺失，请重新安装",
            "竹叶输入法 修复",
            MB_ICONERROR | MB_OK,
        );
        return std::process::ExitCode::from(1);
    };
    match re_register(&dll) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            let _ = message_box(
                None,
                &format!("注册表修复失败：{error}"),
                "竹叶输入法 修复",
                MB_ICONERROR | MB_OK,
            );
            std::process::ExitCode::from(1)
        }
    }
}

/// 弹模态消息框；返回是否点击了"是"（`MB_YESNO` 场景）。
#[must_use]
pub fn message_box(hwnd: Option<HWND>, text: &str, title: &str, style: MESSAGEBOX_STYLE) -> bool {
    let text_wide = to_utf16(text);
    let title_wide = to_utf16(title);
    let result = unsafe {
        MessageBoxW(
            hwnd,
            PCWSTR(text_wide.as_ptr()),
            PCWSTR(title_wide.as_ptr()),
            style,
        )
    };
    // IDYES == 6。
    result == MESSAGEBOX_RESULT(6)
}

/// 重启 ctfmon（终止后由系统按需自动重新加载）。
///
/// 这是设计文档 §6.4 的**开放实现项**：ctfmon 重启的确切行为（终止后系统是否立即自动
/// 拉起）依赖真实 TSF 环境验证，目前以"终止进程树 + 提示系统将自动重新加载"为准，
/// VM 恢复后实测并回填设计文档。
pub fn restart_ctfmon() -> Result<(), String> {
    use windows::core::PWSTR;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        CreateProcessW, WaitForSingleObject, INFINITE, PROCESS_CREATION_FLAGS, PROCESS_INFORMATION,
        STARTUPINFOW,
    };

    let executable = to_utf16(r"C:\Windows\System32\taskkill.exe");
    let mut command_line = to_utf16("taskkill /f /im ctfmon.exe");
    let mut process_info = PROCESS_INFORMATION::default();
    let result = unsafe {
        let startup = STARTUPINFOW {
            cb: std::mem::size_of::<STARTUPINFOW>() as u32,
            ..Default::default()
        };
        CreateProcessW(
            PCWSTR(executable.as_ptr()),
            Some(PWSTR(command_line.as_mut_ptr())),
            None,
            None,
            false,
            PROCESS_CREATION_FLAGS(0),
            None,
            None,
            &startup,
            &mut process_info,
        )
    };
    if result.is_err() {
        return Err(format!(
            "无法启动 taskkill 终止 ctfmon：{}",
            std::io::Error::last_os_error()
        ));
    }
    unsafe {
        let _ = WaitForSingleObject(process_info.hProcess, INFINITE);
        let _ = CloseHandle(process_info.hThread);
        let _ = CloseHandle(process_info.hProcess);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        category_key_path, clsid_key_path, guid_text, inproc_key_path, item_key_path,
        profile_key_path, tip_key_path,
    };
    use zhu_ye_core::identity::{CLSID_ZHU_YE_TIP, PROFILE_GUID_ZHU_YE, TFCAT_ZHU_YE_KEYBOARD};

    #[test]
    fn guid文本与门禁侧一致() {
        assert_eq!(
            guid_text(CLSID_ZHU_YE_TIP),
            "{E54D6682-8650-40E7-A9EE-6FD1137849AE}"
        );
        assert_eq!(
            guid_text(PROFILE_GUID_ZHU_YE),
            "{6315FE74-92C3-439B-8CDF-FDB6E43EDAF1}"
        );
        assert_eq!(
            guid_text(TFCAT_ZHU_YE_KEYBOARD),
            "{34745C63-B2F0-4784-8B67-5E12C8701A31}"
        );
    }

    #[test]
    fn 注册路径与安装脚本布局一致() {
        assert_eq!(
            tip_key_path(),
            r"SOFTWARE\Microsoft\CTF\TIP\{E54D6682-8650-40E7-A9EE-6FD1137849AE}"
        );
        assert_eq!(
            clsid_key_path(),
            r"SOFTWARE\Classes\CLSID\{E54D6682-8650-40E7-A9EE-6FD1137849AE}"
        );
        assert_eq!(
            profile_key_path(),
            r"SOFTWARE\Microsoft\CTF\TIP\{E54D6682-8650-40E7-A9EE-6FD1137849AE}\LanguageProfile\0x00000804\{6315FE74-92C3-439B-8CDF-FDB6E43EDAF1}"
        );
        assert_eq!(
            category_key_path(),
            r"SOFTWARE\Microsoft\CTF\TIP\{E54D6682-8650-40E7-A9EE-6FD1137849AE}\Category\Category\{34745C63-B2F0-4784-8B67-5E12C8701A31}\{E54D6682-8650-40E7-A9EE-6FD1137849AE}"
        );
        assert_eq!(
            item_key_path(),
            r"SOFTWARE\Microsoft\CTF\TIP\{E54D6682-8650-40E7-A9EE-6FD1137849AE}\Category\Item\{E54D6682-8650-40E7-A9EE-6FD1137849AE}\{34745C63-B2F0-4784-8B67-5E12C8701A31}"
        );
        assert_eq!(
            inproc_key_path(),
            r"SOFTWARE\Classes\CLSID\{E54D6682-8650-40E7-A9EE-6FD1137849AE}\InProcServer32"
        );
    }
}
