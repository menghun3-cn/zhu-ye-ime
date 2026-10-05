//! 自定义主题文件（T-088，FR-048 主题文件/自定义主题）。
//!
//! 主题文件存于 `%APPDATA%\zhu-ye-ime\themes\<名称>.json`，一份文件同时描述候选窗
//! 与设置窗口两组配色（"候选窗与设置窗口同源读取"，S-2），键集分别对齐
//! `CandidateUiTheme`（7 键）与 `SettingsTheme`（15 键）：
//!
//! ```json
//! {
//!   "version": 1,
//!   "name": "竹影",
//!   "candidate": {
//!     "background": "#202020", "foreground": "#64B5F6", "secondary": "#9E9E9E",
//!     "border": "#42A5F5", "highlight_background": "#3A4A5C",
//!     "highlight_foreground": "#FF8A80", "marker": "#9E9E9E"
//!   },
//!   "settings": { "window": "#1F1F1F", "...": "..." }
//! }
//! ```
//!
//! 解析口径（方案设计 §14.5.3 风险条目）：
//! - 文件不存在 / JSON 损坏 / 版本不受支持 → `Err`，调用方整体回退当前主题；
//! - 未知顶层键与未知组内键忽略（向前兼容）；
//! - 键缺失或颜色值无法解析 → 该键记 `None`，调用方回退对应预设值（"缺键用默认值"）。
//!
//! 颜色字符串接受 `#RRGGBB` 与 `RRGGBB` 两种写法（均不区分大小写）。
//! 纯逻辑模块，无 Win32 依赖（`zhu-ye-core` 约束），颜色以 `0xRRGGBB` 数值表示，
//! 由消费方（候选窗/设置窗口）转换为各自的 `UiColor`。

use std::path::Path;

/// 主题文件格式版本；更高版本的文件应拒用（向上兼容由未知键忽略承担）。
pub const THEME_FILE_VERSION: u32 = 1;

/// 一个 `0xRRGGBB` 颜色值。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeColor(pub u32);

/// 候选窗配色节（键集 = `CandidateUiTheme` 7 键）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CandidatePalette {
    pub background: Option<ThemeColor>,
    pub foreground: Option<ThemeColor>,
    pub secondary: Option<ThemeColor>,
    pub border: Option<ThemeColor>,
    pub highlight_background: Option<ThemeColor>,
    pub highlight_foreground: Option<ThemeColor>,
    pub marker: Option<ThemeColor>,
}

/// 设置窗口配色节（键集 = `SettingsTheme` 15 键）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SettingsPalette {
    pub window: Option<ThemeColor>,
    pub nav_background: Option<ThemeColor>,
    pub nav_selected: Option<ThemeColor>,
    pub nav_selected_text: Option<ThemeColor>,
    pub nav_text: Option<ThemeColor>,
    pub title_text: Option<ThemeColor>,
    pub item_text: Option<ThemeColor>,
    pub secondary_text: Option<ThemeColor>,
    pub border: Option<ThemeColor>,
    pub accent: Option<ThemeColor>,
    pub control_background: Option<ThemeColor>,
    pub control_selected: Option<ThemeColor>,
    pub control_selected_text: Option<ThemeColor>,
    pub placeholder_text: Option<ThemeColor>,
    pub warn_text: Option<ThemeColor>,
}

/// 解析后的主题文件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeFile {
    /// 格式版本（已校验等于 [`THEME_FILE_VERSION`]）。
    pub version: u32,
    /// 主题显示名（文件内可选；缺省时由调用方用文件名代替）。
    pub name: Option<String>,
    /// 候选窗配色节。
    pub candidate: CandidatePalette,
    /// 设置窗口配色节。
    pub settings: SettingsPalette,
}

/// 从磁盘读取并解析主题文件；错误信息面向 UI 展示。
pub fn load_theme_file(path: &Path) -> Result<ThemeFile, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| format!("读取主题文件 {} 失败: {error}", path.display()))?;
    parse_theme_file(&text).map_err(|error| format!("{}: {error}", path.display()))
}

/// 解析主题文件 JSON 文本。
pub fn parse_theme_file(text: &str) -> Result<ThemeFile, String> {
    let root: serde_json::Value =
        serde_json::from_str(text).map_err(|error| format!("主题文件不是合法 JSON: {error}"))?;
    let object = root
        .as_object()
        .ok_or_else(|| "主题文件顶层必须是 JSON 对象".to_owned())?;

    let version = object
        .get("version")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "主题文件缺少整数 version 字段".to_owned())?;
    if version > u64::from(THEME_FILE_VERSION) {
        return Err(format!(
            "主题文件版本 {version} 高于当前支持的 {THEME_FILE_VERSION}，请升级输入法"
        ));
    }

    let name = object
        .get("name")
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned);

    let candidate = parse_candidate(object.get("candidate"));
    let settings = parse_settings(object.get("settings"));

    Ok(ThemeFile {
        version: version as u32,
        name,
        candidate,
        settings,
    })
}

/// 解析候选窗节；`None`（缺节）或非对象时按空节处理（全部键回退默认值）。
fn parse_candidate(value: Option<&serde_json::Value>) -> CandidatePalette {
    let object = value.and_then(serde_json::Value::as_object);
    let get = |key: &str| object.and_then(|map| map.get(key)).and_then(parse_color);
    CandidatePalette {
        background: get("background"),
        foreground: get("foreground"),
        secondary: get("secondary"),
        border: get("border"),
        highlight_background: get("highlight_background"),
        highlight_foreground: get("highlight_foreground"),
        marker: get("marker"),
    }
}

/// 解析设置窗口节；口径同 [`parse_candidate`]。
fn parse_settings(value: Option<&serde_json::Value>) -> SettingsPalette {
    let object = value.and_then(serde_json::Value::as_object);
    let get = |key: &str| object.and_then(|map| map.get(key)).and_then(parse_color);
    SettingsPalette {
        window: get("window"),
        nav_background: get("nav_background"),
        nav_selected: get("nav_selected"),
        nav_selected_text: get("nav_selected_text"),
        nav_text: get("nav_text"),
        title_text: get("title_text"),
        item_text: get("item_text"),
        secondary_text: get("secondary_text"),
        border: get("border"),
        accent: get("accent"),
        control_background: get("control_background"),
        control_selected: get("control_selected"),
        control_selected_text: get("control_selected_text"),
        placeholder_text: get("placeholder_text"),
        warn_text: get("warn_text"),
    }
}

/// 主题 id/文件名校验：只允许 ASCII 字母数字与 `-`/`_`，杜绝 `../` 等目录逃逸。
/// id 由 themes 目录扫描的文件名得到，配置里手写 `Custom("../x")` 之类一律视为
/// "无此主题"（T-088 / FR-048）。
#[must_use]
pub fn is_safe_theme_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
}

/// 颜色字符串解析：`#RRGGBB` 或 `RRGGBB`（不区分大小写）；非法返回 `None`。
#[must_use]
pub fn parse_color(value: &serde_json::Value) -> Option<ThemeColor> {
    let text = value.as_str()?;
    let text = text.strip_prefix('#').unwrap_or(text);
    if text.len() != 6 || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let rgb = u32::from_str_radix(text, 16).ok()?;
    Some(ThemeColor(rgb))
}

#[cfg(test)]
mod tests {
    use super::{load_theme_file, parse_color, parse_theme_file, ThemeFile, THEME_FILE_VERSION};
    use serde_json::json;

    #[test]
    fn 完整主题文件两节都解析() {
        let file = parse_theme_file(
            r##"{
                "version": 1,
                "name": "竹影",
                "candidate": {
                    "background": "#202020",
                    "foreground": "#64B5F6",
                    "highlight_background": "#3A4A5C"
                },
                "settings": { "window": "#1F1F1F", "accent": "#42A5F5" }
            }"##,
        )
        .expect("合法主题应解析成功");
        assert_eq!(file.version, THEME_FILE_VERSION);
        assert_eq!(file.name.as_deref(), Some("竹影"));
        assert_eq!(
            file.candidate.background,
            Some(super::ThemeColor(0x20_20_20))
        );
        assert_eq!(
            file.candidate.foreground,
            Some(super::ThemeColor(0x64_B5_F6))
        );
        assert_eq!(
            file.candidate.highlight_background,
            Some(super::ThemeColor(0x3A_4A_5C))
        );
        // 未写的键为 None（调用方回退默认值）。
        assert_eq!(file.candidate.marker, None);
        assert_eq!(file.settings.window, Some(super::ThemeColor(0x1F_1F_1F)));
        assert_eq!(file.settings.accent, Some(super::ThemeColor(0x42_A5_F5)));
        assert_eq!(file.settings.warn_text, None);
    }

    #[test]
    fn 未知键忽略且颜色写法宽松() {
        let file = parse_theme_file(
            r##"{
                "version": 1,
                "future_key": 42,
                "candidate": {
                    "background": "202020",
                    "foreground": "64b5f6",
                    "unknown_key": "#123456"
                }
            }"##,
        )
        .expect("未知键应忽略");
        assert_eq!(
            file.candidate.background,
            Some(super::ThemeColor(0x20_20_20))
        );
        assert_eq!(
            file.candidate.foreground,
            Some(super::ThemeColor(0x64_B5_F6))
        );
        assert_eq!(file.candidate.secondary, None);
    }

    #[test]
    fn 缺节与缺版本拒用或空节() {
        // JSON 损坏 / 顶层非对象 / 缺 version / 版本过高 → Err（整体拒用）。
        assert!(parse_theme_file("not json").is_err());
        assert!(parse_theme_file("[1,2]").is_err());
        assert!(
            parse_theme_file(r#"{"candidate":{}}"#).is_err(),
            "缺版本应拒用"
        );
        assert!(
            parse_theme_file(r#"{"version": 99}"#).is_err(),
            "高版本应拒用"
        );
        // candidate/settings 节整体缺失 → 无错，两节全空。
        let file = parse_theme_file(r#"{"version": 1}"#).unwrap();
        assert_eq!(file.candidate, super::CandidatePalette::default());
        assert_eq!(file.settings, super::SettingsPalette::default());
    }

    #[test]
    fn 颜色字符串解析() {
        let text = json!("#1E88E5");
        assert_eq!(parse_color(&text), Some(super::ThemeColor(0x1E_88_E5)));
        assert_eq!(
            parse_color(&json!("1e88e5")),
            Some(super::ThemeColor(0x1E_88_E5))
        );
        // 只剥一个 # 前缀：双 # 或后缀 # 均为非法。
        assert_eq!(parse_color(&json!("##1E88E5")), None);
        assert_eq!(parse_color(&json!("#1E88E5#")), None);
        assert_eq!(parse_color(&json!("red")), None);
        assert_eq!(parse_color(&json!("#12")), None);
        assert_eq!(parse_color(&json!("#GGGGGG")), None);
        assert_eq!(parse_color(&json!(123)), None);
        assert_eq!(parse_color(&json!(null)), None);
    }

    #[test]
    fn 颜色值缺失或非法降级回退() {
        let file = parse_theme_file(
            r##"{
                "version": 1,
                "candidate": { "background": "#GG0000", "foreground": 5, "secondary": "ok" }
            }"##,
        )
        .unwrap();
        assert_eq!(file.candidate.background, None, "非法颜色应降级");
        assert_eq!(file.candidate.foreground, None, "非字符串应降级");
    }

    #[test]
    fn 加载真实文件往返() {
        let dir = std::env::temp_dir().join(format!(
            "zhu-ye-theme-file-{}-{}",
            std::process::id(),
            crate::user_store::unix_now()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("sample.json");
        std::fs::write(
            &path,
            r##"{"version": 1, "candidate": {"background": "#000000"}}"##,
        )
        .unwrap();
        let file: ThemeFile = load_theme_file(&path).unwrap();
        assert_eq!(file.candidate.background, Some(super::ThemeColor(0)));
        assert!(load_theme_file(&dir.join("missing.json")).is_err());
    }

    #[test]
    fn 主题名称防目录逃逸() {
        assert!(super::is_safe_theme_name("zhu-ying"));
        assert!(super::is_safe_theme_name("A1_b-c"));
        assert!(!super::is_safe_theme_name(""));
        assert!(!super::is_safe_theme_name(".."));
        assert!(!super::is_safe_theme_name("../x"));
        assert!(!super::is_safe_theme_name("a/b"));
        assert!(!super::is_safe_theme_name("a\\b"));
        assert!(!super::is_safe_theme_name("a b"));
        assert!(!super::is_safe_theme_name("竹影"), "显示名不能当 id 用");
    }
}
