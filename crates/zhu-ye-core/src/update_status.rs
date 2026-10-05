//! 更新状态文件（T-088，FR-048 启动时异步检查一次）。
//!
//! 启动异步检查的结果只写状态文件（无 UI、无弹窗，方案设计 §14.5.4），供关于页
//! 顺带展示。文件为 `%APPDATA%\zhu-ye-ime\update_status.json`：
//!
//! ```json
//! {
//!   "format": "zhu-ye-update-status",
//!   "version": 1,
//!   "last_checked": 1759420800,
//!   "available": true,
//!   "outdated_packs": ["it", "med"],
//!   "latest_version": "2026.10.4",
//!   "error": null
//! }
//! ```
//!
//! 写入方 = 更新器 `check-once` 子命令；读取方 = 设置窗口关于页。文件不存在或损坏
//! 时读取方按"从未检查/无结果"处理，不弹窗、不打扰。

use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// 更新状态文件名（在 `%APPDATA%\zhu-ye-ime` 目录下）。
pub const UPDATE_STATUS_FILE_NAME: &str = "update_status.json";
/// 状态文件格式标识。
pub const UPDATE_STATUS_FORMAT: &str = "zhu-ye-update-status";
/// 状态文件格式版本。
pub const UPDATE_STATUS_VERSION: u32 = 1;

/// 启动异步检查的间隔（天）：距上次检查不足此间隔时 `check-once` 直接退出。
pub const CHECK_INTERVAL_DAYS: u64 = 7;
/// 一天对应的秒数。
pub const DAY_SECONDS: u64 = 24 * 60 * 60;

/// 更新状态。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateStatus {
    /// 格式标识。
    #[serde(default)]
    pub format: String,
    /// 格式版本。
    #[serde(default)]
    pub version: u32,
    /// 上次检查时间（Unix 秒）。
    #[serde(default)]
    pub last_checked: u64,
    /// 是否存在可应用更新。
    #[serde(default)]
    pub available: bool,
    /// 可更新的包 id 列表（为空 = 无可用更新或不可用）。
    #[serde(default)]
    pub outdated_packs: Vec<String>,
    /// 可更新包中的最大版本号（无可用更新时为 `None`）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_version: Option<String>,
    /// 检查失败的摘要（成功或未检查时为 `None`）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl UpdateStatus {
    /// 构造"发现可用更新"状态。
    #[must_use]
    pub fn available(
        last_checked: u64,
        outdated_packs: Vec<String>,
        latest_version: String,
    ) -> Self {
        Self {
            format: UPDATE_STATUS_FORMAT.to_owned(),
            version: UPDATE_STATUS_VERSION,
            last_checked,
            available: true,
            outdated_packs,
            latest_version: Some(latest_version),
            error: None,
        }
    }

    /// 构造"没有可用更新"状态。
    #[must_use]
    pub fn up_to_date(last_checked: u64) -> Self {
        Self {
            format: UPDATE_STATUS_FORMAT.to_owned(),
            version: UPDATE_STATUS_VERSION,
            last_checked,
            available: false,
            outdated_packs: Vec::new(),
            latest_version: None,
            error: None,
        }
    }

    /// 构造"检查失败"状态（仍记录时间，避免失败后立即反复重试）。
    #[must_use]
    pub fn failed(last_checked: u64, error: impl Into<String>) -> Self {
        Self {
            format: UPDATE_STATUS_FORMAT.to_owned(),
            version: UPDATE_STATUS_VERSION,
            last_checked,
            available: false,
            outdated_packs: Vec::new(),
            latest_version: None,
            error: Some(error.into()),
        }
    }
}

/// 序列化为 JSON 文本。
pub fn to_json(status: &UpdateStatus) -> Result<String, String> {
    serde_json::to_string_pretty(status).map_err(|error| format!("序列化更新状态失败: {error}"))
}

/// 从 JSON 文本解析并校验格式/版本；不合法返回 `Err`。
pub fn parse_update_status(text: &str) -> Result<UpdateStatus, String> {
    let status: UpdateStatus = serde_json::from_str(text)
        .map_err(|error| format!("更新状态文件不是合法 JSON: {error}"))?;
    if status.format != UPDATE_STATUS_FORMAT {
        return Err(format!(
            "更新状态文件格式不是“{}”（实际：{}）",
            UPDATE_STATUS_FORMAT, status.format
        ));
    }
    if status.version != UPDATE_STATUS_VERSION {
        return Err(format!(
            "更新状态文件版本 {} 不是当前支持的 {}",
            status.version, UPDATE_STATUS_VERSION
        ));
    }
    Ok(status)
}

/// 读取更新状态文件：不存在返回 `Ok(None)`；损坏返回 `Err`（调用方静默按无结果处理）。
pub fn read_update_status(path: &Path) -> Result<Option<UpdateStatus>, String> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!("读取更新状态 {} 失败: {error}", path.display()));
        }
    };
    parse_update_status(&text).map(Some)
}

/// 原子写入更新状态文件（临时文件 + rename，失败不破坏旧文件）。
pub fn write_update_status(path: &Path, status: &UpdateStatus) -> Result<(), String> {
    let bytes = to_json(status)?.into_bytes();
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent)
        .map_err(|error| format!("创建目录 {} 失败: {error}", parent.display()))?;

    let tmp = path.with_extension("tmp");
    let write_result = (|| -> std::io::Result<()> {
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    })();
    if let Err(error) = write_result {
        let _ = std::fs::remove_file(&tmp);
        return Err(format!("写入更新状态 {} 失败: {error}", path.display()));
    }
    Ok(())
}

/// 是否到期检查：从未检查（`None`）或距上次检查已满间隔天数。
#[must_use]
pub fn check_due(last_check: Option<u64>, now: u64) -> bool {
    match last_check {
        None => true,
        Some(last) => now >= last.saturating_add(CHECK_INTERVAL_DAYS * DAY_SECONDS),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        check_due, parse_update_status, read_update_status, write_update_status, UpdateStatus,
        UPDATE_STATUS_FORMAT, UPDATE_STATUS_VERSION,
    };
    use crate::update_status::{CHECK_INTERVAL_DAYS, DAY_SECONDS};

    #[test]
    fn 到期判断覆盖从未检查与间隔边界() {
        assert!(check_due(None, 1), "从未检查应立即检查");
        let last = 1_000;
        let interval = CHECK_INTERVAL_DAYS * DAY_SECONDS;
        assert!(!check_due(Some(last), last + interval - 1), "差一秒未到期");
        assert!(check_due(Some(last), last + interval), "满间隔到期");
        assert!(check_due(Some(last), last + interval + 10_000));
        // 时钟回拨：now 小于 last 时不触发（saturating 保护）。
        assert!(!check_due(Some(last), last - 100));
    }

    #[test]
    fn 可用更新状态往返() {
        let status =
            UpdateStatus::available(1_759_420_800, vec!["it".to_owned()], "2026.10.4".into());
        let text = super::to_json(&status).unwrap();
        let back = parse_update_status(&text).unwrap();
        assert_eq!(back, status);
        assert!(back.available);
        assert_eq!(back.outdated_packs, vec!["it"]);
        assert_eq!(back.latest_version.as_deref(), Some("2026.10.4"));
        assert_eq!(back.error, None);
    }

    #[test]
    fn 无更新与失败状态() {
        let uptodate =
            parse_update_status(&super::to_json(&UpdateStatus::up_to_date(1)).unwrap()).unwrap();
        assert!(!uptodate.available);
        assert!(uptodate.outdated_packs.is_empty());
        let failed =
            parse_update_status(&super::to_json(&UpdateStatus::failed(1, "网络不通")).unwrap())
                .unwrap();
        assert_eq!(failed.error.as_deref(), Some("网络不通"));
    }

    #[test]
    fn 格式或版本不匹配整体拒绝() {
        let text = r#"{"format": "other", "version": 1, "last_checked": 1, "available": false}"#;
        assert!(parse_update_status(text).is_err());
        let text =
            format!(r#"{{"format": "{UPDATE_STATUS_FORMAT}", "version": 99, "last_checked": 1}}"#);
        assert!(parse_update_status(&text).is_err());
        assert!(parse_update_status("not json").is_err());
    }

    #[test]
    fn 读写文件原子往返() {
        let dir = std::env::temp_dir().join(format!(
            "zhu-ye-update-status-{}-{}",
            std::process::id(),
            crate::user_store::unix_now()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(super::UPDATE_STATUS_FILE_NAME);
        assert_eq!(read_update_status(&path).unwrap(), None, "不存在返回 None");
        let status = UpdateStatus::available(42, vec!["med".to_owned()], "2026.10.1".into());
        write_update_status(&path, &status).unwrap();
        assert_eq!(read_update_status(&path).unwrap(), Some(status.clone()));
        // 写入后无 tmp 残留。
        assert!(!path.with_extension("tmp").exists());
        // 损坏文件返回 Err。
        std::fs::write(&path, "{broken").unwrap();
        assert!(read_update_status(&path).is_err());
    }

    #[test]
    fn 构造默认值即当前支持的格式() {
        for status in [
            UpdateStatus::available(1, vec![], "1".into()),
            UpdateStatus::up_to_date(1),
            UpdateStatus::failed(1, "x"),
        ] {
            assert_eq!(status.format, UPDATE_STATUS_FORMAT);
            assert_eq!(status.version, UPDATE_STATUS_VERSION);
        }
    }
}
