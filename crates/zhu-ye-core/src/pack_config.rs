//! 词典包运行时配置：`config.json` 解析与包路径解析（M6-R，FR-015/FR-022）。
//!
//! 配置与用户词库同目录（`%APPDATA%\ai-zhu-ye-ime\config.json`）：
//!
//! ```json
//! { "enabled_packs": ["it","med","slang"], "online_update": false, "last_check": null, "theme": "light" }
//! ```
//!
//! 设计约束（方案设计 11.3）：
//! - 未知包 id 忽略并记日志，不报错；
//! - 损坏配置回退默认（仅 base），不让输入法不可用；
//! - 基础包随安装只读，领域包在 `packs/` 可写目录；
//! - 配置变更在下次启动装配（P-12，不做热切换）。

use std::path::{Path, PathBuf};

use serde::{Deserialize, Deserializer, Serialize};

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

/// 词典包的展示信息（第八期 FR-022：设置界面要求列出名称与简介）。
///
/// 名称与简介走这里的静态表，而不是从产物读：`manifest.json` 生成时把 `name` 写成包 id，
/// 中文名此前只是构建期的打印常量，简介在产物与 manifest 中都不存在；`.zyct` 头部也只剩
/// 两段共 20 字节的未写入预留区，装不下中文名与简介（D-37）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackDisplay {
    /// 中文展示名。
    pub name: &'static str,
    /// 一行简介。
    pub summary: &'static str,
    /// 是否为开发期产物：不作为用户可勾选的领域包展示。
    pub development: bool,
}

/// 取词典包的展示信息；未知包 id 返回 `None`。
#[must_use]
pub fn pack_display(pack_id: &str) -> Option<PackDisplay> {
    Some(match pack_id {
        "base" => PackDisplay {
            name: "基础词典",
            summary: "日常常用词与中英译文，始终加载，不可停用",
            development: false,
        },
        "it" => PackDisplay {
            name: "IT/编程",
            summary: "编程与计算机术语（THUOCL_IT + MDN Web 术语表）",
            development: false,
        },
        "med" => PackDisplay {
            name: "医学",
            summary: "医学与临床术语（THUOCL 医学）",
            development: false,
        },
        "slang" => PackDisplay {
            name: "网络语",
            summary: "网络热词与字母缩写（yyds 一类）",
            development: false,
        },
        "real" => PackDisplay {
            name: "真实语料包",
            summary: "开发期真实语料构建产物",
            development: true,
        },
        "seed" => PackDisplay {
            name: "演示种子包",
            summary: "开发期演示用最小词典",
            development: true,
        },
        _ => return None,
    })
}

/// 候选窗主题选择（第八期设置窗口写入，FR-041；T-088 扩展自定义主题文件）。
///
/// 只提供浅色与深色两个预置：高对比度由系统接管，不作为可选值（D-31）。
/// `Custom` 携带主题文件名（不含 `.json` 扩展），对应 `%APPDATA%\ai-zhu-ye-ime\
/// themes\<name>.json`；文件缺失或解析失败时渲染回退浅色（配置本身不失败）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ThemeChoice {
    /// 浅色（默认，沿用 T-030 的候选窗默认浅色口径）。
    #[default]
    Light,
    /// 深色。
    Dark,
    /// 自定义主题文件名（不含扩展名）；任何非预置字符串都按此解释。
    Custom(String),
}

impl ThemeChoice {
    /// 配置字符串（小写）：预置返回字面值，自定义返回主题文件名。
    #[must_use]
    pub fn as_str(&self) -> std::borrow::Cow<'static, str> {
        match self {
            Self::Light => std::borrow::Cow::Borrowed("light"),
            Self::Dark => std::borrow::Cow::Borrowed("dark"),
            Self::Custom(name) => std::borrow::Cow::Owned(name.clone()),
        }
    }

    /// 宽松解析：拼错的预置值视为自定义主题名，渲染时按文件可用性回退浅色。
    #[must_use]
    pub fn parse(value: &str) -> Self {
        match value {
            "light" => Self::Light,
            "dark" => Self::Dark,
            _ if value.is_empty() => Self::Light,
            _ => Self::Custom(value.to_owned()),
        }
    }
}

/// `ThemeChoice` 序列化为单字符串（自定义主题写入文件名本身）。
impl Serialize for ThemeChoice {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.as_str())
    }
}

/// `theme` 的反序列化：宽松解析。
///
/// 主题值由用户手改 `config.json` 时最易写错，而加载器对不可解析的配置整体回退默认
/// （丢掉 `enabled_packs`）。该字段单独接受任意 JSON 值：字符串按 `parse` 解释，其余
/// 类型（数字/对象/数组/null）一律降级为浅色。配置只经 `from_json` 读取，因此这里直接
/// 以 JSON 值接收，而不必为每种标量各写一个访问者。
fn deserialize_theme<'de, D>(deserializer: D) -> Result<ThemeChoice, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(match value {
        Some(serde_json::Value::String(text)) => ThemeChoice::parse(&text),
        _ => ThemeChoice::Light,
    })
}

/// 新输入会话的默认中英模式（第八期设置窗口写入，FR-041）。
///
/// 这是**装配项**而不是会话状态：中英模式是每个宿主进程各自的 IME 实例状态，跨进程没有
/// 单一的"当前模式"可供设置窗口展示或切换，因此设置窗口能表达的唯一有意义语义是
/// "新会话从哪种模式开始"，由 TSF DLL 在下次装配时读取。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ModeChoice {
    /// 中文（默认，沿用既有行为）。
    #[default]
    Chinese,
    /// 英文。
    English,
}

impl ModeChoice {
    /// 配置字符串（小写），与序列化表示一致。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Chinese => "chinese",
            Self::English => "english",
        }
    }

    /// 宽松解析：未知值回退中文。
    #[must_use]
    pub fn parse(value: &str) -> Self {
        match value {
            "english" => Self::English,
            _ => Self::Chinese,
        }
    }
}

/// `default_mode` 的反序列化：宽松解析，理由同 `theme`。
fn deserialize_default_mode<'de, D>(deserializer: D) -> Result<ModeChoice, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(match value {
        Some(serde_json::Value::String(text)) => ModeChoice::parse(&text),
        _ => ModeChoice::Chinese,
    })
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
    /// 领域自动提权总开关（FR-035，场景8）；默认 `true`（D-14）。
    /// 关闭后领域候选恢复既有追加语义（T-050 基线，不做位次上移）。
    #[serde(default = "default_domain_boost")]
    pub enable_domain_boost: bool,
    /// 通讯录 vCard 文件路径列表（FR-038，场景9；D-19 配置触发导入）。
    /// 为空 = 不导入、无联系人候选（清单与现版逐位一致，T-050 基线不漂移）。
    #[serde(default)]
    pub contact_vcards: Vec<PathBuf>,
    /// 候选窗主题（第八期 FR-041）；缺失或未知值时按浅色处理。
    #[serde(default, deserialize_with = "deserialize_theme")]
    pub theme: ThemeChoice,
    /// 新输入会话的默认中英模式（第八期 FR-041）；缺失或未知值时按中文处理。
    #[serde(default, deserialize_with = "deserialize_default_mode")]
    pub default_mode: ModeChoice,
    /// 文件日志级别（第十一期 FR-060，T-091；D-72 默认只记错误级）。
    /// 缺失/非法/未知值回退 `warn`（宽松解析，T-073 同款模式），
    /// **不递增 `CONFIG_FORMAT_VERSION`**（旧配置无此字段按默认处理）。
    #[serde(default, deserialize_with = "crate::log_level::deserialize_log_level")]
    pub log_level: crate::log_level::LogLevel,
}

fn default_version() -> u32 {
    CONFIG_FORMAT_VERSION
}

fn default_domain_boost() -> bool {
    true
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            version: CONFIG_FORMAT_VERSION,
            enabled_packs: Vec::new(),
            online_update: false,
            last_check: None,
            enable_domain_boost: true,
            contact_vcards: Vec::new(),
            theme: ThemeChoice::Light,
            default_mode: ModeChoice::Chinese,
            log_level: crate::log_level::LogLevel::default(),
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
    /// 已启用的领域包路径，按配置顺序（与 `pack_ids` 一一对应）。
    pub packs: Vec<PathBuf>,
    /// 已启用领域包 id，按配置顺序（与 `packs` 一一对应）。
    /// 装配方（IME）可按 id 字典序重排做领域识别（D-16，场景8）。
    pub pack_ids: Vec<String>,
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
    let mut pack_ids = Vec::new();
    let mut missing = Vec::new();
    for id in known {
        let path = crate::composite::pack_path(packs_dir, &id);
        if path.is_file() {
            packs.push(path);
            pack_ids.push(id);
        } else {
            missing.push(id);
        }
    }
    PackPlan {
        base,
        packs,
        pack_ids,
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
        // 领域提权默认开（D-14）。
        assert!(config.enable_domain_boost);
        // 主题缺失时按浅色（T-030 候选窗默认浅色口径）。
        assert_eq!(config.theme, super::ThemeChoice::Light);
        // 新会话默认中英模式缺失时按中文（D-32 装配项口径）。
        assert_eq!(config.default_mode, super::ModeChoice::Chinese);
    }

    #[test]
    fn 默认中英模式解析与序列化往返() {
        for (text, expected) in [
            ("chinese", super::ModeChoice::Chinese),
            ("english", super::ModeChoice::English),
        ] {
            let json = format!(r#"{{"default_mode": "{text}"}}"#);
            let config = ConfigFile::from_json(&json).unwrap();
            assert_eq!(config.default_mode, expected);
            assert_eq!(config.default_mode.as_str(), text);
        }
        let config = ConfigFile {
            default_mode: super::ModeChoice::English,
            ..ConfigFile::default()
        };
        let text = config.to_json().unwrap();
        assert!(
            text.contains(r#""default_mode": "english""#),
            "序列化应写出小写模式值"
        );
        assert_eq!(ConfigFile::from_json(&text).unwrap(), config);
    }

    #[test]
    fn 默认中英模式值拼错不回退整体配置() {
        // 与主题同口径的宽松解析：未知值回退中文，用户已勾选的领域包不受影响。
        let text = r#"{
            "enabled_packs": ["it", "med"],
            "default_mode": "English"
        }"#;
        let config = ConfigFile::from_json(text).unwrap();
        assert_eq!(config.default_mode, super::ModeChoice::Chinese);
        assert_eq!(config.enabled_packs, vec!["it", "med"]);
        // 非字符串（如数字）同样降级，不整体失败。
        let config = ConfigFile::from_json(r#"{"default_mode": 5}"#).unwrap();
        assert_eq!(config.default_mode, super::ModeChoice::Chinese);
    }

    #[test]
    fn 日志级别字段缺失时回退默认警告级别() {
        // T-091（FR-060，D-72）：旧配置无 `log_level` 字段按默认 warn 处理，
        // 不递增 CONFIG_FORMAT_VERSION；字段缺省不破坏既有字段。
        let text = r#"{
            "enabled_packs": ["it"],
            "online_update": true
        }"#;
        let config = ConfigFile::from_json(text).unwrap();
        assert_eq!(config.log_level, crate::log_level::LogLevel::Warn);
        assert_eq!(config.enabled_packs, vec!["it"]);
        assert!(config.online_update);
    }

    #[test]
    fn 日志级别非法值回退警告不影响其它字段() {
        // T-073 同款宽松模式：未知值（含 trace/off）回退 warn，用户配置不整体失败；
        // 合法值大小写不敏感（DEBUG/debug 均为 Debug）。
        for (raw, expected) in [
            (r#""trace""#, crate::log_level::LogLevel::Warn),
            (r#""off""#, crate::log_level::LogLevel::Warn),
            (r#"5"#, crate::log_level::LogLevel::Warn),
            (r#""DEBUG""#, crate::log_level::LogLevel::Debug),
            (r#""debug""#, crate::log_level::LogLevel::Debug),
        ] {
            let config = ConfigFile::from_json(&format!(
                r#"{{"enabled_packs": ["med"], "log_level": {raw}}}"#
            ))
            .unwrap();
            assert_eq!(config.enabled_packs, vec!["med"], "输入 {raw}");
            assert_eq!(config.log_level, expected, "输入 {raw}");
        }
    }

    #[test]
    fn 日志级别序列化往返() {
        let config = ConfigFile {
            log_level: crate::log_level::LogLevel::Info,
            ..ConfigFile::default()
        };
        let text = config.to_json().unwrap();
        assert!(
            text.contains(r#""log_level": "info""#),
            "序列化应写出小写级别值"
        );
        assert_eq!(ConfigFile::from_json(&text).unwrap(), config);
    }

    #[test]
    fn 主题解析与序列化往返() {
        for (text, expected) in [
            ("light", super::ThemeChoice::Light),
            ("dark", super::ThemeChoice::Dark),
        ] {
            let json = format!(r#"{{"theme": "{text}"}}"#);
            let config = ConfigFile::from_json(&json).unwrap();
            assert_eq!(config.theme, expected);
            assert_eq!(config.theme.as_str(), text);
        }
        let config = ConfigFile {
            theme: super::ThemeChoice::Dark,
            ..ConfigFile::default()
        };
        let text = config.to_json().unwrap();
        assert!(
            text.contains(r#""theme": "dark""#),
            "序列化应写出小写主题值"
        );
        assert_eq!(ConfigFile::from_json(&text).unwrap(), config);
    }

    #[test]
    fn 主题拼错按自定义主题名处理且不回退整体配置() {
        // T-088 值类型扩展：非预置字符串一律视为自定义主题文件名；渲染时文件缺失
        // 回退浅色，但配置本身不失败、用户的领域包勾选不受影响。
        let text = r#"{
            "enabled_packs": ["it", "med"],
            "online_update": true,
            "theme": "bamboo-dark"
        }"#;
        let config = ConfigFile::from_json(text).unwrap();
        assert_eq!(
            config.theme,
            super::ThemeChoice::Custom("bamboo-dark".to_owned())
        );
        assert_eq!(config.enabled_packs, vec!["it", "med"]);
        assert!(config.online_update);
        // 非字符串（如数字或对象）仍降级，不整体失败。
        let config = ConfigFile::from_json(r#"{"enabled_packs": ["it"], "theme": 5}"#).unwrap();
        assert_eq!(config.theme, super::ThemeChoice::Light);
        assert_eq!(config.enabled_packs, vec!["it"]);
        // 空字符串与 null 也回退浅色。
        let config = ConfigFile::from_json(r#"{"theme": ""}"#).unwrap();
        assert_eq!(config.theme, super::ThemeChoice::Light);
        let config = ConfigFile::from_json(r#"{"theme": null}"#).unwrap();
        assert_eq!(config.theme, super::ThemeChoice::Light);
    }

    #[test]
    fn 自定义主题往返一致() {
        let config = ConfigFile {
            theme: super::ThemeChoice::Custom("bamboo-dark".to_owned()),
            ..ConfigFile::default()
        };
        let text = config.to_json().unwrap();
        assert!(
            text.contains(r#""theme": "bamboo-dark""#),
            "自定义主题应序列化为文件名本身：{text}"
        );
        let back = ConfigFile::from_json(&text).unwrap();
        assert_eq!(back.theme, config.theme);
        assert_eq!(back.theme.as_str(), "bamboo-dark");
    }

    #[test]
    fn 领域提权开关可关闭并往返一致() {
        let off = ConfigFile::from_json(r#"{"enable_domain_boost": false}"#).unwrap();
        assert!(!off.enable_domain_boost, "显式关闭应生效");
        let text = off.to_json().unwrap();
        assert_eq!(
            ConfigFile::from_json(&text).unwrap(),
            off,
            "开关配置应序列化往返一致"
        );
        // 旧配置文件（无该字段）加载时默认开。
        let legacy = ConfigFile::from_json(r#"{"enabled_packs": ["it"]}"#).unwrap();
        assert!(legacy.enable_domain_boost);
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

        // pack_ids 与 packs 一一对应（场景8 按 id 字典序重排的输入数据，D-16）。
        assert_eq!(plan.pack_ids, vec!["it", "med"]);
        assert_eq!(
            plan.packs
                .iter()
                .zip(plan.pack_ids.iter())
                .map(|(path, id)| {
                    path.file_name().and_then(|n| n.to_str()).unwrap_or("") == format!("{id}.zyct")
                })
                .collect::<Vec<_>>(),
            vec![true, true]
        );

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

    #[test]
    fn 每个已知包id都有展示信息() {
        // 新增包 id 却忘了补名称/简介时，这条必须失败。
        for id in super::KNOWN_PACK_IDS {
            let display = super::pack_display(id).unwrap_or_else(|| panic!("{id} 缺少展示信息"));
            assert!(!display.name.is_empty(), "{id} 名称为空");
            assert!(!display.summary.is_empty(), "{id} 简介为空");
            assert_ne!(display.name, *id, "{id} 的展示名不应退回包 id");
        }
    }

    #[test]
    fn 展示信息区分开发期产物() {
        // 开发期产物不进用户可勾选的领域包列表，界面据此过滤。
        for id in super::DISTRIBUTABLE_PACK_IDS {
            let display = super::pack_display(id).expect("可分发包必须有展示信息");
            assert!(!display.development, "{id} 是可分发包，不应标记为开发产物");
        }
        for id in ["real", "seed"] {
            assert!(
                super::pack_display(id)
                    .expect("开发产物也应有展示信息")
                    .development,
                "{id} 是开发产物"
            );
        }
        // 基础包始终加载，也不属于可分发的领域包。
        let base = super::pack_display("base").expect("基础包应有展示信息");
        assert!(!base.development);
        assert!(!super::is_distributable_pack("base"));
    }

    #[test]
    fn 未知包id没有展示信息() {
        assert_eq!(super::pack_display("nope"), None);
        assert_eq!(super::pack_display(""), None);
        // 大小写不匹配也应视为未知，避免界面出现两个"同一个包"。
        assert_eq!(super::pack_display("IT"), None);
    }
}
