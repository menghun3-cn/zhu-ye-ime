//! 托盘状态桥（T-112 后续批六：Windows 系统托盘"中/英"状态图标）。
//!
//! 输入法本体是 TSF DLL，被各宿主进程加载，无法自行持有托盘图标；批量四
//! 曾论证 `Shell_NotifyIcon` 无常驻进程不可实现。本模块把模式状态以极小
//! 文件桥递给独立的常驻托盘程序 `zhu-ye-tray`（部署位 `bin\zhu-ye-tray.exe`，
//! 经 HKCU Run 自启）：
//!
//! - 引擎（TSF DLL）在模式切换时原子写入 `%APPDATA%\zhu-ye-ime\tray-state`；
//! - 托盘程序轮询该文件，按内容切换托盘图标（白"中"=中文模式 / 白"英"=
//!   英文模式）。
//!
//! 文件内容单行，取稳定标识 `Chinese` / `English`（与
//! `zhu_ye_ime::input::InputMode` 的序列化约定）。写入失败只降级（托盘
//! 保持上一次模式），绝不阻断输入闭环。

/// 状态文件名（位于配置目录 `%APPDATA%\zhu-ye-ime`，与 `config.json` 同层）。
pub const TRAY_STATE_FILE_NAME: &str = "tray-state";

/// 中文模式标识。
pub const MODE_CHINESE: &str = "Chinese";

/// 英文模式标识。
pub const MODE_ENGLISH: &str = "English";

/// 原子写模式状态：先写临时文件再改名，托盘轮询读到的永远是整行内容，
/// 不会撞上半写文件。
pub fn write_tray_state(dir: &std::path::Path, mode: &str) -> std::io::Result<()> {
    let target = dir.join(TRAY_STATE_FILE_NAME);
    let tmp = dir.join(format!("{TRAY_STATE_FILE_NAME}.tmp"));
    std::fs::write(&tmp, format!("{mode}\n"))?;
    std::fs::rename(&tmp, &target)?;
    Ok(())
}

/// 解析状态文件内容；只接受两个稳定标识，其余（含空/损坏）返回 `None`。
#[must_use]
pub fn parse_tray_state(text: &str) -> Option<&str> {
    match text.trim() {
        MODE_CHINESE => Some(MODE_CHINESE),
        MODE_ENGLISH => Some(MODE_ENGLISH),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 写读往返一致() {
        let dir =
            std::env::temp_dir().join(format!("zhu-ye-tray-state-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        write_tray_state(&dir, MODE_CHINESE).unwrap();
        let text = std::fs::read_to_string(dir.join(TRAY_STATE_FILE_NAME)).unwrap();
        assert_eq!(parse_tray_state(&text), Some(MODE_CHINESE));

        write_tray_state(&dir, MODE_ENGLISH).unwrap();
        let text = std::fs::read_to_string(dir.join(TRAY_STATE_FILE_NAME)).unwrap();
        assert_eq!(parse_tray_state(&text), Some(MODE_ENGLISH));

        // 原子写：临时文件不得残留。
        assert!(!dir.join(format!("{TRAY_STATE_FILE_NAME}.tmp")).exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 非法内容返回空() {
        assert_eq!(parse_tray_state(""), None);
        assert_eq!(parse_tray_state("ZhEre"), None);
        assert_eq!(parse_tray_state("Chinese\nEnglish\n"), None);
        assert_eq!(parse_tray_state("chinese"), None);
    }

    #[test]
    fn 目录不存在时原子写报错且不产生半文件() {
        let dir = std::env::temp_dir().join("zhu-ye-tray-state-no-such-dir");
        let _ = std::fs::remove_dir_all(&dir);
        assert!(write_tray_state(&dir, MODE_CHINESE).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
