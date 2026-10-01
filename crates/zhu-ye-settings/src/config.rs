//! 设置窗口的配置读写（纯逻辑，无 Win32 依赖）。
//!
//! 复用 `zhu_ye_core::pack_config` 的加载与原子保存。本模块只负责"只改主题、不动其他
//! 字段"——`config.json` 同时被 TSF 侧、更新器与设置窗口读写，任何一方整体覆盖都会丢掉
//! 别人的写入。

use std::path::{Path, PathBuf};

use zhu_ye_core::{load_config, save_config, ConfigFile, ThemeChoice};

/// `%APPDATA%` 下的数据目录名。
const APPDATA_DIR: &str = "ai-zhu-ye-ime";
/// 配置文件名。
const CONFIG_FILE: &str = "config.json";

/// 用户数据目录：`%APPDATA%\ai-zhu-ye-ime`（与 TSF 侧、更新器同口径）。
#[must_use]
pub fn data_dir() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|root| PathBuf::from(root).join(APPDATA_DIR))
}

/// 配置文件路径。
#[must_use]
pub fn config_path() -> Option<PathBuf> {
    data_dir().map(|dir| dir.join(CONFIG_FILE))
}

/// 读取配置；缺失或损坏时回退默认并回报诊断。
#[must_use]
pub fn load(path: &Path) -> (ConfigFile, Option<String>) {
    load_config(path)
}

/// 读取当前主题；缺失或损坏回退浅色。
#[must_use]
pub fn load_theme(path: &Path) -> (ThemeChoice, Option<String>) {
    let (config, diagnostic) = load_config(path);
    (config.theme, diagnostic)
}

/// 只更新主题并原子保存。
///
/// 保存前重读配置：更新器会写 `last_check`，若用窗口启动时的旧快照整体覆盖就会丢掉它的
/// 写入（设计文档 S-8：两个写入方都必须在提交前重读）。
pub fn save_theme(path: &Path, theme: ThemeChoice) -> Result<(), String> {
    let (mut config, _) = load_config(path);
    config.theme = theme;
    save_config(path, &config)
}

#[cfg(test)]
mod tests {
    use super::{load_theme, save_theme};
    use std::path::PathBuf;
    use zhu_ye_core::ThemeChoice;

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "zhu-ye-settings-{label}-{}-{}",
            std::process::id(),
            zhu_ye_core::unix_now()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn 缺失配置回退浅色且无诊断() {
        let path = temp_dir("missing").join("config.json");
        let (theme, diagnostic) = load_theme(&path);
        assert_eq!(theme, ThemeChoice::Light);
        assert_eq!(diagnostic, None);
    }

    #[test]
    fn 损坏配置回退浅色并给出诊断() {
        let dir = temp_dir("broken");
        let path = dir.join("config.json");
        std::fs::write(&path, "{ not json").unwrap();
        let (theme, diagnostic) = load_theme(&path);
        assert_eq!(theme, ThemeChoice::Light);
        assert!(diagnostic.is_some(), "损坏配置必须回报诊断而不是静默");
    }

    #[test]
    fn 只改主题不动其他字段() {
        let dir = temp_dir("preserve");
        let path = dir.join("config.json");
        std::fs::write(
            &path,
            r#"{
                "enabled_packs": ["it", "med"],
                "online_update": true,
                "contact_vcards": ["C:/contacts/a.vcf"],
                "enable_domain_boost": false,
                "theme": "light"
            }"#,
        )
        .unwrap();

        save_theme(&path, ThemeChoice::Dark).unwrap();

        let (config, diagnostic) = super::load(&path);
        assert_eq!(diagnostic, None);
        assert_eq!(config.theme, ThemeChoice::Dark, "主题应被写入");
        assert_eq!(
            config.enabled_packs,
            vec!["it", "med"],
            "领域包勾选不得丢失"
        );
        assert!(
            config.online_update,
            "在线更新开关不得被重置（P-03 默认关）"
        );
        assert_eq!(config.contact_vcards.len(), 1, "通讯录路径不得丢失");
        assert!(!config.enable_domain_boost, "领域提权开关不得被重置");
        assert_eq!(config.version, zhu_ye_core::CONFIG_FORMAT_VERSION);
    }

    #[test]
    fn 保存前重读保留更新器写入的检查时间() {
        let dir = temp_dir("reread");
        let path = dir.join("config.json");
        // 模拟更新器先写入 last_check。
        std::fs::write(
            &path,
            r#"{ "enabled_packs": ["slang"], "last_check": 1759420800 }"#,
        )
        .unwrap();

        save_theme(&path, ThemeChoice::Dark).unwrap();

        let (config, _) = super::load(&path);
        assert_eq!(
            config.last_check,
            Some(1_759_420_800),
            "提交前重读，更新器写入的 last_check 必须保留（S-8）"
        );
        assert_eq!(config.enabled_packs, vec!["slang"]);
        assert_eq!(config.theme, ThemeChoice::Dark);
    }

    #[test]
    fn 重复保存幂等() {
        let dir = temp_dir("idempotent");
        let path = dir.join("config.json");
        save_theme(&path, ThemeChoice::Dark).unwrap();
        let first = std::fs::read_to_string(&path).unwrap();
        save_theme(&path, ThemeChoice::Dark).unwrap();
        let second = std::fs::read_to_string(&path).unwrap();
        assert_eq!(first, second);
        assert_eq!(load_theme(&path).0, ThemeChoice::Dark);
    }
}
