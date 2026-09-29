//! 词典包运行时配置：`config.json` 解析与包路径解析（M6-R，FR-015/FR-022）。
//!
//! 配置与用户词库同目录（`%APPDATA%\ai-zhu-ye-ime\config.json`）：
//!
//! ```json
//! { "enabled_packs": ["it","med","slang"], "online_update": false, "last_check": null }
//! ```
//!
//! 设计约束（方案设计 11.3）：
//! - 未知包 id 忽略并记日志，不报错；
//! - 损坏配置回退默认（仅 base），不让输入法不可用；
//! - 基础包随安装只读，领域包在 `packs/` 可写目录；
//! - 配置变更在下次启动装配（P-12，不做热切换）。

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// 配置文件版本；格式变更时递增并在加载时拒绝更高版本。
pub const CONFIG_FORMAT_VERSION: u32 = 1;

/// 基础包文件名（随安装只读，DLL 同目录）。
pub const BASE_PACK_FILE_NAME: &str = "dictionary.zyct";

/// 领域包目录名（`%APPDATA%\ai-zhu-ye-ime\packs`）。
pub const PACKS_DIR_NAME: &str = "packs";

/// 已知领域包 id（用于过滤未知 id；新增包需同步此处与文档）。
pub const KNOWN_PACK_IDS: &[&str] = &["base", "it", "med", "slang", "real", "seed"];

/// 可通过在线更新分发的包 id（方案设计 11.7）。
///
/// 基础包随安装/发版只读交付，不在更新范围内；`real`/`seed` 是开发期构建产物，
/// 从不发布。更新器只应下载并应用此列表内的包——否则会去拉取根本不存在的
/// 发布文件（实测会得到 404）。
pub const DISTRIBUTABLE_PACK_IDS: &[&str] = &["it", "med", "slang"];

/// 判断某个包 id 是否属于在线更新分发范围。
#[must_use]
pub fn is_distributable_pack(pack_id: &str) -> bool {
    DISTRIBUTABLE_PACK_IDS.contains(&pack_id)
}

/// 磁盘上的配置格式。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigFile {
    /// 格式版本。
    #[serde(default = "default_version")]
    pub version: u32,
    /// 启用的领域包 id 列表；基础包始终加载，不出现在此列表。
    #[serde(default)]
    pub enabled_packs: Vec<String>,
    /// 是否允许在线更新；默认 `false`（需求 P-03 默认关闭原则）。
    #[serde(default)]
    pub online_update: bool,
    /// 上次检查更新时间（Unix 秒）；`null` 表示从未检查。
    #[serde(default)]
    pub last_check: Option<u64>,
}

fn default_version() -> u32 {
    CONFIG_FORMAT_VERSION
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            version: CONFIG_FORMAT_VERSION,
            enabled_packs: Vec::new(),
            online_update: false,
            last_check: None,
        }
    }
}

impl ConfigFile {
    /// 从 JSON 文本解析；版本高于支持范围时返回错误，其余损坏由调用方回退默认。
    pub fn from_json(text: &str) -> Result<Self, String> {
        let parsed: Self =
            serde_json::from_str(text).map_err(|error| format!("配置解析失败: {error}"))?;
        if parsed.version > CONFIG_FORMAT_VERSION {
            return Err(format!(
                "配置版本 {} 高于当前支持的 {}，请升级输入法",
                parsed.version, CONFIG_FORMAT_VERSION
            ));
        }
        Ok(parsed)
    }

    /// 序列化为 JSON 文本（带缩进，便于用户手改）。
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|error| format!("配置序列化失败: {error}"))
    }

    /// 过滤出已知包 id，保持用户给定顺序并去重；返回 (有效, 被忽略)。
    #[must_use]
    pub fn partition_packs(&self) -> (Vec<String>, Vec<String>) {
        let mut known = Vec::new();
        let mut unknown = Vec::new();
        for id in &self.enabled_packs {
            if KNOWN_PACK_IDS.contains(&id.as_str()) {
                if !known.contains(id) {
                    known.push(id.clone());
                }
            } else if !unknown.contains(id) {
                unknown.push(id.clone());
            }
        }
        (known, unknown)
    }
}

/// 运行时装配计划：基础包路径 + 已启用领域包路径。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PackPlan {
    /// 基础包路径（只读，随安装）；不存在时为空。
    pub base: Option<PathBuf>,
    /// 已启用的领域包路径，按配置顺序。
    pub packs: Vec<PathBuf>,
    /// 被忽略的未知包 id。
    pub unknown: Vec<String>,
    /// 配置了但文件缺失的包 id。
    pub missing: Vec<String>,
}

impl PackPlan {
    /// 全部待加载路径：基础包在前，领域包按配置顺序在后。
    #[must_use]
    pub fn paths(&self) -> Vec<PathBuf> {
        let mut all = Vec::new();
        if let Some(base) = &self.base {
            all.push(base.clone());
        }
        all.extend(self.packs.iter().cloned());
        all
    }
}

/// 计算装配计划：`base_dir` 放基础包（DLL 同目录/安装目录），
/// `packs_dir` 放领域包（`%APPDATA%\ai-zhu-ye-ime\packs`）。
///
/// 缺失与未知的包只记录不报错，保证输入法始终能启动。
#[must_use]
pub fn plan_packs(config: &ConfigFile, base_dir: &Path, packs_dir: &Path) -> PackPlan {
    let base_candidate = base_dir.join(BASE_PACK_FILE_NAME);
    let base = base_candidate.is_file().then_some(base_candidate);

    let (known, unknown) = config.partition_packs();
    let mut packs = Vec::new();
    let mut missing = Vec::new();
    for id in known {
        let path = crate::composite::pack_path(packs_dir, &id);
        if path.is_file() {
            packs.push(path);
        } else {
            missing.push(id);
        }
    }
    PackPlan {
        base,
        packs,
        unknown,
        missing,
    }
}

/// 读取配置文件；不存在或损坏时回退默认配置并返回诊断信息。
///
/// 返回值第二项非空表示发生了回退，调用方应记日志。
#[must_use]
pub fn load_config(path: &Path) -> (ConfigFile, Option<String>) {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return (ConfigFile::default(), None);
        }
        Err(error) => {
            return (
                ConfigFile::default(),
                Some(format!("读取配置 {} 失败: {error}", path.display())),
            );
        }
    };
    match ConfigFile::from_json(&text) {
        Ok(config) => (config, None),
        Err(reason) => (
            ConfigFile::default(),
            Some(format!("{}（已回退默认配置：仅基础包）", reason)),
        ),
    }
}

/// 原子写入配置：先写同目录临时文件再替换，避免半截文件。
pub fn save_config(path: &Path, config: &ConfigFile) -> Result<(), String> {
    let text = config.to_json()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("创建配置目录 {} 失败: {error}", parent.display()))?;
    }
    let temp = path.with_extension("json.tmp");
    std::fs::write(&temp, text.as_bytes())
        .map_err(|error| format!("写入临时配置 {} 失败: {error}", temp.display()))?;
    std::fs::rename(&temp, path).map_err(|error| {
        let _ = std::fs::remove_file(&temp);
        format!("替换配置 {} 失败: {error}", path.display())
    })
}

#[cfg(test)]
mod tests {
    use super::{
        load_config, plan_packs, save_config, ConfigFile, PackPlan, CONFIG_FORMAT_VERSION,
    };
    use std::path::PathBuf;

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "zhu-ye-config-{label}-{}-{}",
            std::process::id(),
            crate::unix_now()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn 分发范围只含领域包不含基础包与开发产物() {
        // 在线更新只覆盖可写的领域包。
        assert!(super::is_distributable_pack("it"));
        assert!(super::is_distributable_pack("med"));
        assert!(super::is_distributable_pack("slang"));
        // 基础包随安装只读交付，不参与在线更新。
        assert!(!super::is_distributable_pack("base"));
        // 开发期产物从不发布。
        assert!(!super::is_distributable_pack("real"));
        assert!(!super::is_distributable_pack("seed"));
        // 分发范围必须是已知包的子集。
        for id in super::DISTRIBUTABLE_PACK_IDS {
            assert!(
                super::KNOWN_PACK_IDS.contains(id),
                "{id} 在分发范围内但不在已知包列表"
            );
        }
    }

    #[test]
    fn 默认配置仅基础包且在线更新关闭() {
        let config = ConfigFile::default();
        assert!(config.enabled_packs.is_empty());
        assert!(!config.online_update, "默认必须关闭在线更新（P-03）");
        assert_eq!(config.last_check, None);
        assert_eq!(config.version, CONFIG_FORMAT_VERSION);
    }

    #[test]
    fn 解析完整配置() {
        let text = r#"{
            "version": 1,
            "enabled_packs": ["it", "med"],
            "online_update": true,
            "last_check": 1234567890
        }"#;
        let config = ConfigFile::from_json(text).unwrap();
        assert_eq!(config.enabled_packs, vec!["it", "med"]);
        assert!(config.online_update);
        assert_eq!(config.last_check, Some(1_234_567_890));
    }

    #[test]
    fn 缺省字段回退默认值() {
        let config = ConfigFile::from_json("{}").unwrap();
        assert_eq!(config.version, CONFIG_FORMAT_VERSION);
        assert!(config.enabled_packs.is_empty());
        assert!(!config.online_update);
    }

    #[test]
    fn 高版本配置被拒绝() {
        let err = ConfigFile::from_json(r#"{"version": 99}"#).unwrap_err();
        assert!(
            err.contains("高于当前支持"),
            "错误信息应说明版本过高: {err}"
        );
    }

    #[test]
    fn 非法json解析失败() {
        assert!(ConfigFile::from_json("{ not json").is_err());
    }

    #[test]
    fn 未知包id被过滤且去重() {
        let config = ConfigFile {
            enabled_packs: vec![
                "it".to_owned(),
                "不存在".to_owned(),
                "it".to_owned(),
                "med".to_owned(),
                "另一个未知".to_owned(),
            ],
            ..ConfigFile::default()
        };
        let (known, unknown) = config.partition_packs();
        assert_eq!(known, vec!["it", "med"]);
        assert_eq!(unknown, vec!["不存在", "另一个未知"]);
    }

    #[test]
    fn 配置往返序列化一致() {
        let config = ConfigFile {
            enabled_packs: vec!["slang".to_owned()],
            online_update: true,
            last_check: Some(42),
            ..ConfigFile::default()
        };
        let text = config.to_json().unwrap();
        assert_eq!(ConfigFile::from_json(&text).unwrap(), config);
    }

    #[test]
    fn 装配计划含基础包与已启用领域包() {
        let dir = temp_dir("plan");
        let packs = dir.join("packs");
        std::fs::create_dir_all(&packs).unwrap();
        std::fs::write(dir.join("dictionary.zyct"), b"base").unwrap();
        std::fs::write(packs.join("it.zyct"), b"it").unwrap();
        std::fs::write(packs.join("med.zyct"), b"med").unwrap();

        let config = ConfigFile {
            enabled_packs: vec!["it".to_owned(), "med".to_owned()],
            ..ConfigFile::default()
        };
        let plan = plan_packs(&config, &dir, &packs);
        assert_eq!(plan.base, Some(dir.join("dictionary.zyct")));
        assert_eq!(plan.packs.len(), 2);
        assert!(plan.missing.is_empty());
        assert!(plan.unknown.is_empty());

        // 路径顺序：基础包在前。
        let paths = plan.paths();
        assert_eq!(paths.len(), 3);
        assert!(paths[0].ends_with("dictionary.zyct"));
        assert!(paths[1].ends_with("it.zyct"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 装配计划记录缺失包与未知包() {
        let dir = temp_dir("plan-missing");
        let packs = dir.join("packs");
        std::fs::create_dir_all(&packs).unwrap();
        std::fs::write(packs.join("it.zyct"), b"it").unwrap();

        let config = ConfigFile {
            enabled_packs: vec!["it".to_owned(), "med".to_owned(), "不存在".to_owned()],
            ..ConfigFile::default()
        };
        let plan = plan_packs(&config, &dir, &packs);
        assert_eq!(plan.packs.len(), 1, "只有 it 存在");
        assert_eq!(plan.missing, vec!["med"], "med 配置了但文件缺失");
        assert_eq!(plan.unknown, vec!["不存在"]);
        assert_eq!(plan.base, None, "基础包缺失时不报错");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 空计划路径为空() {
        let plan = PackPlan::default();
        assert!(plan.paths().is_empty());
    }

    #[test]
    fn 配置缺失回退默认且无诊断() {
        let dir = temp_dir("absent");
        let (config, diagnostic) = load_config(&dir.join("config.json"));
        assert_eq!(config, ConfigFile::default());
        assert!(diagnostic.is_none(), "文件不存在属正常情况，不应报错");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 配置损坏回退默认并给出诊断() {
        let dir = temp_dir("corrupt");
        let path = dir.join("config.json");
        std::fs::write(&path, b"{ broken").unwrap();
        let (config, diagnostic) = load_config(&path);
        assert_eq!(config, ConfigFile::default(), "损坏必须回退默认（仅 base）");
        assert!(diagnostic.is_some(), "回退应留下诊断信息");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 保存后可重新加载() {
        let dir = temp_dir("roundtrip");
        let path = dir.join("config.json");
        let config = ConfigFile {
            enabled_packs: vec!["slang".to_owned()],
            online_update: true,
            last_check: Some(7),
            ..ConfigFile::default()
        };
        save_config(&path, &config).unwrap();
        let (loaded, diagnostic) = load_config(&path);
        assert_eq!(loaded, config);
        assert!(diagnostic.is_none());
        // 临时文件不应残留。
        assert!(!path.with_extension("json.tmp").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
