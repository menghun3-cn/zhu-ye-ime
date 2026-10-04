//! 设置窗口配色（纯逻辑，无 Win32 依赖）。
//!
//! 与候选窗配色相互独立：候选窗受 T-030 约束固定浅色（不读系统深浅色），设置窗口是常规
//! 桌面窗口，跟随系统深浅色（D-30）。两个窗口服务不同任务，配色口径不共享，只共享
//! `zhu-ye-ui` 原语 crate 的颜色与主题种类类型（T-081 抽取，S-10 收尾）。

use zhu_ye_ui::{SystemColors, UiColor, UiThemeKind};

/// 设置窗口配色。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsTheme {
    /// 窗口背景。
    pub window: UiColor,
    /// 左侧导航背景。
    pub nav_background: UiColor,
    /// 导航选中项背景。
    pub nav_selected: UiColor,
    /// 导航选中项文字。
    pub nav_selected_text: UiColor,
    /// 导航未选中项文字。
    pub nav_text: UiColor,
    /// 页标题文字。
    pub title_text: UiColor,
    /// 条目标题文字。
    pub item_text: UiColor,
    /// 条目说明与次要文字。
    pub secondary_text: UiColor,
    /// 分隔线与边框。
    pub border: UiColor,
    /// 强调色（选中控件）。
    pub accent: UiColor,
    /// 控件底色（未选中）。
    pub control_background: UiColor,
    /// 控件选中底色。
    pub control_selected: UiColor,
    /// 控件选中文字。
    pub control_selected_text: UiColor,
    /// 占位说明文字颜色。
    pub placeholder_text: UiColor,
    /// 异常/警示文字（注册状态异常、修复建议）。
    pub warn_text: UiColor,
}

/// 返回预设配色。
#[must_use]
pub fn settings_theme(kind: UiThemeKind) -> SettingsTheme {
    match kind {
        UiThemeKind::Light => SettingsTheme {
            window: UiColor::rgb(0xFF, 0xFF, 0xFF),
            nav_background: UiColor::rgb(0xF3, 0xF3, 0xF3),
            nav_selected: UiColor::rgb(0xE1, 0xEE, 0xFB),
            nav_selected_text: UiColor::rgb(0x0B, 0x57, 0xD0),
            nav_text: UiColor::rgb(0x42, 0x42, 0x42),
            title_text: UiColor::rgb(0x20, 0x20, 0x20),
            item_text: UiColor::rgb(0x20, 0x20, 0x20),
            secondary_text: UiColor::rgb(0x75, 0x75, 0x75),
            border: UiColor::rgb(0xDC, 0xDC, 0xDC),
            accent: UiColor::rgb(0x1E, 0x88, 0xE5),
            control_background: UiColor::rgb(0xF5, 0xF5, 0xF5),
            control_selected: UiColor::rgb(0x1E, 0x88, 0xE5),
            control_selected_text: UiColor::rgb(0xFF, 0xFF, 0xFF),
            placeholder_text: UiColor::rgb(0xB0, 0x6A, 0x00),
            warn_text: UiColor::rgb(0xC5, 0x0F, 0x1F),
        },
        UiThemeKind::Dark => SettingsTheme {
            window: UiColor::rgb(0x1F, 0x1F, 0x1F),
            nav_background: UiColor::rgb(0x2B, 0x2B, 0x2B),
            nav_selected: UiColor::rgb(0x2C, 0x3E, 0x50),
            nav_selected_text: UiColor::rgb(0x64, 0xB5, 0xF6),
            nav_text: UiColor::rgb(0xC8, 0xC8, 0xC8),
            title_text: UiColor::rgb(0xF0, 0xF0, 0xF0),
            item_text: UiColor::rgb(0xEC, 0xEC, 0xEC),
            secondary_text: UiColor::rgb(0x9E, 0x9E, 0x9E),
            border: UiColor::rgb(0x3C, 0x3C, 0x3C),
            accent: UiColor::rgb(0x42, 0xA5, 0xF5),
            control_background: UiColor::rgb(0x33, 0x33, 0x33),
            control_selected: UiColor::rgb(0x42, 0xA5, 0xF5),
            control_selected_text: UiColor::rgb(0x10, 0x10, 0x10),
            placeholder_text: UiColor::rgb(0xE0, 0xA8, 0x4A),
            warn_text: UiColor::rgb(0xFF, 0x9A, 0x8C),
        },
        UiThemeKind::HighContrast => settings_theme_from_system_colors(SystemColors {
            window: 0xFF00_0000,
            window_text: 0x00FF_FFFF,
            gray_text: 0x00FF_FFFF,
            highlight: 0x00FF_FF00,
            highlight_text: 0x0000_0000,
            btn_face: 0x0000_0000,
        }),
    }
}

impl SettingsTheme {
    /// 用自定义主题文件覆盖预设配色（T-088 / FR-048）：文件里给了的键覆盖，
    /// 缺键（或键值解析失败）回退 `self`（调用方传入的当前深浅预设）；高对比度
    /// 由系统接管，调用方在 `HighContrast` 时不调用本函数。
    ///
    /// 返回 `(覆盖后的配色, 实际覆盖的键数)`，键数供界面提示"应用了 N/15 键"。
    #[must_use]
    pub fn with_theme_file(self, file: &zhu_ye_core::ThemeFile) -> (SettingsTheme, usize) {
        // `ThemeColor(u32)` 与 `UiColor(u32)` 同为 0xRRGGBB 布局，可直接映射。
        let mut covered = 0usize;
        let mut overlay = |base: UiColor, color: Option<zhu_ye_core::ThemeColor>| {
            if let Some(color) = color {
                covered += 1;
                UiColor(color.0)
            } else {
                base
            }
        };
        let theme = SettingsTheme {
            window: overlay(self.window, file.settings.window),
            nav_background: overlay(self.nav_background, file.settings.nav_background),
            nav_selected: overlay(self.nav_selected, file.settings.nav_selected),
            nav_selected_text: overlay(self.nav_selected_text, file.settings.nav_selected_text),
            nav_text: overlay(self.nav_text, file.settings.nav_text),
            title_text: overlay(self.title_text, file.settings.title_text),
            item_text: overlay(self.item_text, file.settings.item_text),
            secondary_text: overlay(self.secondary_text, file.settings.secondary_text),
            border: overlay(self.border, file.settings.border),
            accent: overlay(self.accent, file.settings.accent),
            control_background: overlay(self.control_background, file.settings.control_background),
            control_selected: overlay(self.control_selected, file.settings.control_selected),
            control_selected_text: overlay(
                self.control_selected_text,
                file.settings.control_selected_text,
            ),
            placeholder_text: overlay(self.placeholder_text, file.settings.placeholder_text),
            warn_text: overlay(self.warn_text, file.settings.warn_text),
        };
        (theme, covered)
    }
}

/// 按系统高对比度配色构建设置窗口配色；系统色为 `0x00BBGGRR` 字节序。
#[must_use]
pub fn settings_theme_from_system_colors(colors: SystemColors) -> SettingsTheme {
    // 高对比度映射口径与 `candidate_ui::theme_from_system_colors` 完全一致（T-081
    // 抽取后本地化，设置窗口不再依赖 TSF 侧代码）：窗口/控件底=window，选中块=
    // highlight（文字取 highlight_text），主文字=window_text，弱化=gray_text，
    // 边框=btn_face，警示/占位沿用选中文字色。字节序 BGR(0x00BBGGRR)→RGB。
    let bgr_to_rgb =
        |color: u32| UiColor(((color & 0xFF) << 16) | (color & 0x00FF00) | (color >> 16));
    let window = bgr_to_rgb(colors.window);
    let foreground = bgr_to_rgb(colors.window_text);
    let secondary = bgr_to_rgb(colors.gray_text);
    let border = bgr_to_rgb(colors.btn_face);
    let highlight_background = bgr_to_rgb(colors.highlight);
    let highlight_foreground = bgr_to_rgb(colors.highlight_text);
    SettingsTheme {
        window,
        nav_background: window,
        nav_selected: highlight_background,
        nav_selected_text: highlight_foreground,
        nav_text: foreground,
        title_text: foreground,
        item_text: foreground,
        secondary_text: secondary,
        border,
        accent: highlight_background,
        control_background: window,
        control_selected: highlight_background,
        control_selected_text: highlight_foreground,
        placeholder_text: foreground,
        warn_text: highlight_foreground,
    }
}

#[cfg(test)]
mod tests {
    use super::{settings_theme, settings_theme_from_system_colors};
    use zhu_ye_ui::{SystemColors, UiColor, UiThemeKind};

    #[test]
    fn 浅色与深色在窗口底色上互异() {
        let light = settings_theme(UiThemeKind::Light);
        let dark = settings_theme(UiThemeKind::Dark);
        assert_ne!(light.window, dark.window);
        // 深色的每个通道都不高于浅色（确实是"深"）。
        let luma = |c: UiColor| (c.0 >> 16) + ((c.0 >> 8) & 0xFF) + (c.0 & 0xFF);
        assert!(luma(dark.window) < luma(light.window));
        assert!(luma(dark.nav_background) < luma(light.nav_background));
    }

    #[test]
    fn 选中与未选中控件颜色互异() {
        for kind in [UiThemeKind::Light, UiThemeKind::Dark] {
            let theme = settings_theme(kind);
            assert_ne!(theme.control_selected, theme.control_background);
            assert_ne!(theme.control_selected_text, theme.control_background);
            assert_ne!(theme.nav_selected, theme.nav_background);
            assert_ne!(theme.nav_selected_text, theme.nav_text);
        }
    }

    #[test]
    fn 高对比度配色取自系统色() {
        // `SystemColors` 字段是 COLORREF 的 `0x00BBGGRR` 字节序：低字节才是红。
        let colors = SystemColors {
            window: 0x0000_0000,      // 黑
            window_text: 0x00FF_FFFF, // 白
            gray_text: 0x00FF_FFFF,
            highlight: 0x0000_00FF, // 红
            highlight_text: 0x00FF_FFFF,
            btn_face: 0x0000_0000,
        };
        let theme = settings_theme_from_system_colors(colors);
        assert_eq!(theme.window, UiColor::rgb(0, 0, 0));
        assert_eq!(theme.item_text, UiColor::rgb(0xFF, 0xFF, 0xFF));
        assert_eq!(theme.nav_selected, UiColor::rgb(0xFF, 0, 0));
        assert_eq!(theme.nav_selected_text, UiColor::rgb(0xFF, 0xFF, 0xFF));
    }

    #[test]
    fn 预设高对比度与系统色路径给出同构结果() {
        // `settings_theme(HighContrast)` 只是用一个固定的高对比度色快照调用系统色路径。
        let preset = settings_theme(UiThemeKind::HighContrast);
        let explicit = settings_theme_from_system_colors(SystemColors {
            window: 0xFF00_0000,
            window_text: 0x00FF_FFFF,
            gray_text: 0x00FF_FFFF,
            highlight: 0x00FF_FF00,
            highlight_text: 0x0000_0000,
            btn_face: 0x0000_0000,
        });
        assert_eq!(preset, explicit);
        // 白色文本在两种字节序下都应为白。
        assert_eq!(preset.item_text, UiColor::rgb(0xFF, 0xFF, 0xFF));
        assert_eq!(preset.window, explicit.window);
    }

    #[test]
    fn 主题文件覆盖缺键回退预设并统计覆盖键数() {
        use zhu_ye_core::{parse_theme_file, ThemeColor};
        let light = settings_theme(UiThemeKind::Light);
        // 只给 3 键的单节自定义主题。
        let file = parse_theme_file(
            r##"{
                "version": 1,
                "name": "午夜蓝",
                "settings": {
                    "window": "#101418",
                    "nav_background": "1c2733",
                    "accent": "#42A5F5"
                }
            }"##,
        )
        .unwrap();
        let (theme, covered) = light.with_theme_file(&file);
        assert_eq!(covered, 3, "只覆盖文件里给出的 3 键");
        assert_eq!(theme.window, UiColor(ThemeColor(0x10_14_18).0));
        assert_eq!(theme.nav_background, UiColor(ThemeColor(0x1C_27_33).0));
        assert_eq!(theme.accent, UiColor(ThemeColor(0x42_A5_F5).0));
        // 缺键回退浅色预设。
        assert_eq!(theme.item_text, light.item_text);
        assert_eq!(theme.control_selected, light.control_selected);
        // 有候选节不影响设置窗（各读各节）。
        let file = parse_theme_file(
            r##"{
                "version": 1,
                "candidate": { "background": "#202020" }
            }"##,
        )
        .unwrap();
        let (theme, covered) = settings_theme(UiThemeKind::Dark).with_theme_file(&file);
        assert_eq!(covered, 0, "候选节不参与设置窗覆盖");
        assert_eq!(theme, settings_theme(UiThemeKind::Dark));
    }

    #[test]
    fn 主题文件键值非法按缺键回退() {
        use zhu_ye_core::parse_theme_file;
        let dark = settings_theme(UiThemeKind::Dark);
        // 颜色解析失败（"1234"）→ None → 回退深色预设。
        let file = parse_theme_file(
            r##"{
                "version": 1,
                "settings": { "window": "1234", "border": "#GGGGGG" }
            }"##,
        )
        .unwrap();
        let (theme, covered) = dark.with_theme_file(&file);
        assert_eq!(covered, 0);
        assert_eq!(theme, dark);
    }
}
