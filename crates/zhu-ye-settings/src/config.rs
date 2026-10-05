//! 设置窗口的配置读写（纯逻辑，无 Win32 依赖）。
//!
//! 复用 `zhu_ye_core::pack_config` 的加载与原子保存。本模块只负责"只改主题、不动其他
//! 字段"——`config.json` 同时被 TSF 侧、更新器与设置窗口读写，任何一方整体覆盖都会丢掉
//! 别人的写入。

use std::path::{Path, PathBuf};

use zhu_ye_core::{load_config, save_config, ConfigFile, ModeChoice, ThemeChoice};

/// `%APPDATA%` 下的数据目录名。
const APPDATA_DIR: &str = "zhu-ye-ime";
/// 配置文件名。
const CONFIG_FILE: &str = "config.json";

/// 用户数据目录：`%APPDATA%\zhu-ye-ime`（与 TSF 侧、更新器同口径）。
#[must_use]
pub fn data_dir() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|root| PathBuf::from(root).join(APPDATA_DIR))
}

/// 配置文件路径。
#[must_use]
pub fn config_path() -> Option<PathBuf> {
    data_dir().map(|dir| dir.join(CONFIG_FILE))
}

/// 产品日志目录（FR-060，T-091）：`%LOCALAPPDATA%\zhu-ye-ime\logs`。
///
/// TSF 侧产品模式把文件日志写到该目录（`ime.log`，默认 warn 级别，1 MiB
/// 轮转；见 `crates/zhu-ye-ime/src/tsf.rs` 的 `product_log`）。设置窗口独立
/// 进程可能先于首条日志打开 → 打开前由 `open_target` 幂等创建（设计 §5）。
#[must_use]
pub fn product_log_dir() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|root| PathBuf::from(root).join(APPDATA_DIR).join("logs"))
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

/// 读取新会话默认中英模式（D-32 装配项）；缺失或损坏回退中文。
#[must_use]
pub fn load_mode(path: &Path) -> (ModeChoice, Option<String>) {
    let (config, diagnostic) = load_config(path);
    (config.default_mode, diagnostic)
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

/// 只更新新会话默认中英模式并原子保存（与主题同口径，S-8）。
pub fn save_mode(path: &Path, mode: ModeChoice) -> Result<(), String> {
    let (mut config, _) = load_config(path);
    config.default_mode = mode;
    save_config(path, &config)
}

/// 只更新领域包启停集合并原子保存（S-8 同口径）。
///
/// 列表页的勾选是装配项：写进 `enabled_packs` 后由 TSF DLL 下次装配读取，保存前重读
/// 避免覆盖更新器写入的 `last_check`。
pub fn save_packs(path: &Path, enabled_packs: &[String]) -> Result<(), String> {
    let (mut config, _) = load_config(path);
    config.enabled_packs = enabled_packs.to_vec();
    save_config(path, &config)
}

/// 只更新在线更新开关并原子保存（S-8 同口径，P-03 默认关）。
///
/// 勾选即写：`online_update` 是运行时开关（设置状态三分），由更新器进程读取；
/// 保存前重读避免覆盖其他写入方的字段。
pub fn save_online_update(path: &Path, online_update: bool) -> Result<(), String> {
    let (mut config, _) = load_config(path);
    config.online_update = online_update;
    save_config(path, &config)
}

/// 读取在线更新开关；缺失或损坏回退关闭（P-03）。
#[must_use]
pub fn load_online_update(path: &Path) -> (bool, Option<String>) {
    let (config, diagnostic) = load_config(path);
    (config.online_update, diagnostic)
}

/// 读取通讯录 vCard 路径列表；缺失或损坏回退空（附件为会话期收集，不作为默认配置）。
#[must_use]
pub fn load_contact_vcards(path: &Path) -> (Vec<PathBuf>, Option<String>) {
    let (config, diagnostic) = load_config(path);
    (config.contact_vcards.clone(), diagnostic)
}

/// 只更新通讯录 vCard 路径列表并原子保存（T-088 / FR-048，S-8 同口径）。
///
/// 界面化导入把 .vcf 复制进数据目录后调用本函数「追加路径」：保存前重读配置，
/// 避免覆盖更新器写入的 `last_check`；同路径不重复追加（调用方负责去重）。
pub fn save_contact_vcards(path: &Path, vcards: &[PathBuf]) -> Result<(), String> {
    let (mut config, _) = load_config(path);
    config.contact_vcards = vcards.to_vec();
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

    #[test]
    fn 缺失配置默认中英模式回退中文且无诊断() {
        let path = temp_dir("mode-missing").join("config.json");
        let (mode, diagnostic) = super::load_mode(&path);
        assert_eq!(mode, zhu_ye_core::ModeChoice::Chinese);
        assert_eq!(diagnostic, None);
    }

    #[test]
    fn 只改默认中英模式不动其他字段() {
        let dir = temp_dir("mode-preserve");
        let path = dir.join("config.json");
        std::fs::write(
            &path,
            r#"{
                "enabled_packs": ["it"],
                "theme": "dark",
                "last_check": 1759420800,
                "default_mode": "chinese"
            }"#,
        )
        .unwrap();

        super::save_mode(&path, zhu_ye_core::ModeChoice::English).unwrap();

        let (config, diagnostic) = super::load(&path);
        assert_eq!(diagnostic, None);
        assert_eq!(config.default_mode, zhu_ye_core::ModeChoice::English);
        assert_eq!(config.enabled_packs, vec!["it"], "领域包勾选不得丢失");
        assert_eq!(
            config.theme,
            zhu_ye_core::ThemeChoice::Dark,
            "主题不得被重置"
        );
        assert_eq!(
            config.last_check,
            Some(1_759_420_800),
            "提交前重读，更新器写入的 last_check 必须保留（S-8）"
        );
    }

    #[test]
    fn 默认中英模式保存后序列化可读回() {
        let dir = temp_dir("mode-roundtrip");
        let path = dir.join("config.json");
        super::save_mode(&path, zhu_ye_core::ModeChoice::English).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(
            text.contains(r#""default_mode": "english""#),
            "序列化应写出小写模式值：{text}"
        );
        assert_eq!(super::load_mode(&path).0, zhu_ye_core::ModeChoice::English);
    }

    #[test]
    fn 只改启停集合不动其他字段() {
        let dir = temp_dir("packs-preserve");
        let path = dir.join("config.json");
        std::fs::write(
            &path,
            r#"{
                "enabled_packs": ["it"],
                "theme": "dark",
                "default_mode": "english",
                "last_check": 1759420800
            }"#,
        )
        .unwrap();

        super::save_packs(&path, &["slang".to_owned(), "med".to_owned()]).unwrap();

        let (config, diagnostic) = super::load(&path);
        assert_eq!(diagnostic, None);
        assert_eq!(
            config.enabled_packs,
            vec!["slang", "med"],
            "启停集合应被写入"
        );
        assert_eq!(
            config.theme,
            zhu_ye_core::ThemeChoice::Dark,
            "主题不得被重置"
        );
        assert_eq!(
            config.default_mode,
            zhu_ye_core::ModeChoice::English,
            "默认中英模式不得被重置（D-32）"
        );
        assert_eq!(
            config.last_check,
            Some(1_759_420_800),
            "提交前重读，更新器写入的 last_check 必须保留（S-8）"
        );
    }

    #[test]
    fn 在线更新开关默认关闭且显式开启可往返() {
        let dir = temp_dir("online");
        let path = dir.join("config.json");
        // P-03：默认关闭。
        assert!(!super::load_online_update(&path).0);

        super::save_online_update(&path, true).unwrap();
        let (config, diagnostic) = super::load(&path);
        assert_eq!(diagnostic, None);
        assert!(config.online_update, "开启后 config.json 应写入 true");
        assert!(super::load_online_update(&path).0);

        // 关闭同样可往返。
        super::save_online_update(&path, false).unwrap();
        assert!(!super::load_online_update(&path).0);
    }

    #[test]
    fn 只改在线更新开关不动其他字段() {
        let dir = temp_dir("online-preserve");
        let path = dir.join("config.json");
        std::fs::write(
            &path,
            r#"{
                "enabled_packs": ["it"],
                "theme": "dark",
                "default_mode": "english",
                "last_check": 1759420800
            }"#,
        )
        .unwrap();

        super::save_online_update(&path, true).unwrap();

        let (config, diagnostic) = super::load(&path);
        assert_eq!(diagnostic, None);
        assert!(config.online_update);
        assert_eq!(config.enabled_packs, vec!["it"], "领域包勾选不得丢失");
        assert_eq!(
            config.theme,
            zhu_ye_core::ThemeChoice::Dark,
            "主题不得被重置"
        );
        assert_eq!(
            config.default_mode,
            zhu_ye_core::ModeChoice::English,
            "默认中英模式不得被重置（D-32）"
        );
        assert_eq!(
            config.last_check,
            Some(1_759_420_800),
            "提交前重读，更新器写入的 last_check 必须保留（S-8）"
        );
    }

    #[test]
    fn 只改通讯录路径集合不动其他字段() {
        let dir = temp_dir("vcards-preserve");
        let path = dir.join("config.json");
        std::fs::write(
            &path,
            r#"{
                "enabled_packs": ["it"],
                "theme": "dark",
                "default_mode": "english",
                "last_check": 1759420800
            }"#,
        )
        .unwrap();

        super::save_contact_vcards(
            &path,
            &[
                PathBuf::from("C:/data/contacts/a.vcf"),
                PathBuf::from("b.vcf"),
            ],
        )
        .unwrap();

        let (config, diagnostic) = super::load(&path);
        assert_eq!(diagnostic, None);
        assert_eq!(
            config.contact_vcards,
            vec![
                PathBuf::from("C:/data/contacts/a.vcf"),
                PathBuf::from("b.vcf")
            ],
            "通讯录路径应被写入"
        );
        assert_eq!(config.enabled_packs, vec!["it"], "领域包勾选不得丢失");
        assert_eq!(
            config.theme,
            zhu_ye_core::ThemeChoice::Dark,
            "主题不得被重置"
        );
        assert_eq!(
            config.last_check,
            Some(1_759_420_800),
            "提交前重读，更新器写入的 last_check 必须保留（S-8）"
        );
        // 覆盖整集合：第二次保存后读取以新集合为准。
        super::save_contact_vcards(&path, &[PathBuf::from("c.vcf")]).unwrap();
        assert_eq!(
            super::load_contact_vcards(&path).0,
            vec![PathBuf::from("c.vcf")]
        );
    }

    #[test]
    fn 缺失配置通讯录为空列表() {
        let path = temp_dir("vcards-missing").join("config.json");
        let (vcards, diagnostic) = super::load_contact_vcards(&path);
        assert!(vcards.is_empty());
        assert_eq!(diagnostic, None);
    }

    #[test]
    fn 产品日志目录按本地应用数据目录解析且缺失时为空() {
        // T-091（FR-060）：`%LOCALAPPDATA%\zhu-ye-ime\logs`，与 TSF 侧
        // 产品轨 `ime.log` 路径同源。环境变量在单个测试内串行覆盖/恢复，
        // 规避并行线程互相覆盖。
        let original = std::env::var_os("LOCALAPPDATA");
        let dir = temp_dir("logdir");
        std::env::set_var("LOCALAPPDATA", &dir);
        assert_eq!(
            super::product_log_dir(),
            Some(dir.join("zhu-ye-ime").join("logs"))
        );
        std::env::remove_var("LOCALAPPDATA");
        assert_eq!(super::product_log_dir(), None);
        if let Some(path) = original {
            std::env::set_var("LOCALAPPDATA", path);
        }
    }
}
